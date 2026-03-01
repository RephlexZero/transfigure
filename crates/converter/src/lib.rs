use serde::Deserialize;

pub mod archive;
mod audio;
mod document;
mod image_conv;
mod spreadsheet;

#[derive(Deserialize)]
pub struct ConvertConfig {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub quality: Option<u8>,
}

pub fn convert(input: &[u8], config_json: &str) -> Result<Vec<u8>, String> {
    let config: ConvertConfig =
        serde_json::from_str(config_json).map_err(|e| format!("Invalid config: {e}"))?;

    let from = config.from.to_lowercase();
    let to = config.to.to_lowercase();

    match (from.as_str(), to.as_str()) {
        // Image conversions
        (f, t) if is_image_format(f) && is_image_format(t) => {
            image_conv::convert_image(input, f, t, config.quality)
        }
        ("svg", "png") => image_conv::svg_to_png(input),

        // Documents
        ("md" | "markdown", "html") => document::markdown_to_html(input),
        ("md" | "markdown", "txt" | "text") => document::markdown_to_text(input),
        ("html", "md" | "markdown") => document::html_to_markdown(input),

        // Spreadsheet / data
        ("csv", "json") => spreadsheet::csv_to_json(input),
        ("json", "csv") => spreadsheet::json_to_csv(input),
        ("csv", "tsv") => spreadsheet::csv_to_tsv(input),
        ("tsv", "csv") => spreadsheet::tsv_to_csv(input),

        // Audio
        (f, "wav") if is_audio_format(f) => audio::to_wav(input, f),

        // Encoding
        ("base64", _) => document::base64_decode(input),
        (_, "base64") => document::base64_encode(input),

        // Config formats
        ("json", "yaml" | "yml") => document::json_to_yaml(input),
        ("yaml" | "yml", "json") => document::yaml_to_json(input),
        ("toml", "json") => document::toml_to_json(input),
        ("json", "toml") => document::json_to_toml(input),

        _ => Err(format!(
            "Unsupported conversion: {} → {}",
            config.from, config.to
        )),
    }
}

/// Returns the list of output formats available for a given input extension.
pub fn get_output_formats(input_ext: &str) -> Vec<&'static str> {
    let ext = input_ext.to_lowercase();
    match ext.as_str() {
        "png" => vec!["jpg", "webp", "gif", "bmp", "tiff"],
        "jpg" | "jpeg" => vec!["png", "webp", "gif", "bmp", "tiff"],
        "webp" => vec!["png", "jpg", "gif", "bmp", "tiff"],
        "gif" => vec!["png", "jpg", "webp", "bmp", "tiff"],
        "bmp" => vec!["png", "jpg", "webp", "gif", "tiff"],
        "tiff" | "tif" => vec!["png", "jpg", "webp", "gif", "bmp"],
        "svg" => vec!["png"],
        "mp3" | "flac" | "ogg" | "wav" => vec!["wav"],
        "md" | "markdown" => vec!["html", "txt"],
        "html" => vec!["md"],
        "csv" => vec!["json", "tsv"],
        "tsv" => vec!["csv"],
        "json" => vec!["csv", "yaml", "toml"],
        "yaml" | "yml" => vec!["json"],
        "toml" => vec!["json"],
        "base64" => vec!["bin"],
        _ => vec!["base64"],
    }
}

/// Detects the format from a filename extension.
pub fn detect_format(filename: &str) -> Option<String> {
    let ext = filename.rsplit('.').next()?;
    let lower = ext.to_lowercase();
    if is_known_format(&lower) {
        Some(lower)
    } else {
        None
    }
}

fn is_image_format(fmt: &str) -> bool {
    matches!(
        fmt,
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "tiff" | "tif"
    )
}

fn is_audio_format(fmt: &str) -> bool {
    matches!(fmt, "mp3" | "flac" | "ogg" | "wav")
}

fn is_known_format(fmt: &str) -> bool {
    matches!(
        fmt,
        "png"
            | "jpg"
            | "jpeg"
            | "webp"
            | "gif"
            | "bmp"
            | "tiff"
            | "tif"
            | "svg"
            | "md"
            | "markdown"
            | "html"
            | "txt"
            | "text"
            | "csv"
            | "tsv"
            | "json"
            | "yaml"
            | "yml"
            | "toml"
            | "base64"
            | "mp3"
            | "flac"
            | "ogg"
            | "wav"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── detect_format ───────────────────────────────

    #[test]
    fn detect_format_known_extensions() {
        assert_eq!(detect_format("photo.png"), Some("png".into()));
        assert_eq!(detect_format("doc.MD"), Some("md".into()));
        assert_eq!(detect_format("data.JSON"), Some("json".into()));
        assert_eq!(detect_format("archive.tar.gz"), None); // gz is not known
        assert_eq!(detect_format("image.JPEG"), Some("jpeg".into()));
    }

    #[test]
    fn detect_format_no_extension() {
        assert_eq!(detect_format("README"), None);
    }

    #[test]
    fn detect_format_unknown_extension() {
        assert_eq!(detect_format("file.xyz"), None);
    }

    #[test]
    fn detect_format_dotfile() {
        assert_eq!(detect_format(".gitignore"), None);
    }

    // ── get_output_formats ──────────────────────────

    #[test]
    fn output_formats_for_images() {
        let fmts = get_output_formats("png");
        assert!(fmts.contains(&"jpg"));
        assert!(fmts.contains(&"webp"));
        assert!(!fmts.contains(&"png")); // shouldn't convert to self
    }

    #[test]
    fn output_formats_for_markdown() {
        let fmts = get_output_formats("md");
        assert_eq!(fmts, vec!["html", "txt"]);
    }

    #[test]
    fn output_formats_for_json() {
        let fmts = get_output_formats("json");
        assert!(fmts.contains(&"csv"));
        assert!(fmts.contains(&"yaml"));
        assert!(fmts.contains(&"toml"));
    }

    #[test]
    fn output_formats_case_insensitive() {
        assert_eq!(get_output_formats("PNG"), get_output_formats("png"));
    }

    #[test]
    fn output_formats_unknown_defaults_to_base64() {
        assert_eq!(get_output_formats("xyz"), vec!["base64"]);
    }

    // ── convert: config parsing ─────────────────────

    #[test]
    fn convert_invalid_json() {
        let result = convert(b"hello", "not json");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid config"));
    }

    #[test]
    fn convert_unsupported_pair() {
        let result = convert(b"data", r#"{"from":"png","to":"html"}"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported conversion"));
    }

    // ── convert: markdown ───────────────────────────

    #[test]
    fn convert_markdown_to_html() {
        let md = b"# Hello\n\nWorld";
        let result = convert(md, r#"{"from":"md","to":"html"}"#).unwrap();
        let html = String::from_utf8(result).unwrap();
        assert!(html.contains("<h1>"));
        assert!(html.contains("Hello"));
    }

    #[test]
    fn convert_markdown_to_text() {
        let md = b"**bold** text";
        let result = convert(md, r#"{"from":"md","to":"txt"}"#).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains("bold"));
        assert!(!text.contains("**"));
    }

    #[test]
    fn convert_html_to_markdown() {
        let html = b"<h1>Title</h1><p>Paragraph</p>";
        let result = convert(html, r#"{"from":"html","to":"md"}"#).unwrap();
        let md = String::from_utf8(result).unwrap();
        assert!(md.contains("# Title"));
    }

    // ── convert: base64 ─────────────────────────────

    #[test]
    fn convert_to_base64_roundtrip() {
        let data = b"Hello, World!";
        let encoded = convert(data, r#"{"from":"bin","to":"base64"}"#).unwrap();
        let decoded = convert(&encoded, r#"{"from":"base64","to":"bin"}"#).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn convert_invalid_base64() {
        let result = convert(b"not!valid!base64!!!", r#"{"from":"base64","to":"bin"}"#);
        assert!(result.is_err());
    }

    // ── convert: CSV / JSON / TSV ───────────────────

    #[test]
    fn convert_csv_to_json() {
        let csv = b"name,age\nAlice,30\nBob,25";
        let result = convert(csv, r#"{"from":"csv","to":"json"}"#).unwrap();
        let json_str = String::from_utf8(result).unwrap();
        let data: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert!(data.is_array());
        assert_eq!(data.as_array().unwrap().len(), 2);
        assert_eq!(data[0]["name"], "Alice");
    }

    #[test]
    fn convert_json_to_csv() {
        let json = br#"[{"name":"Alice","age":30},{"name":"Bob","age":25}]"#;
        let result = convert(json, r#"{"from":"json","to":"csv"}"#).unwrap();
        let csv_str = String::from_utf8(result).unwrap();
        assert!(csv_str.contains("Alice"));
        assert!(csv_str.contains("Bob"));
    }

    #[test]
    fn convert_csv_to_tsv() {
        let csv = b"a,b\n1,2";
        let result = convert(csv, r#"{"from":"csv","to":"tsv"}"#).unwrap();
        let tsv = String::from_utf8(result).unwrap();
        assert!(tsv.contains("a\tb"));
        assert!(tsv.contains("1\t2"));
    }

    #[test]
    fn convert_tsv_to_csv() {
        let tsv = b"a\tb\n1\t2";
        let result = convert(tsv, r#"{"from":"tsv","to":"csv"}"#).unwrap();
        let csv = String::from_utf8(result).unwrap();
        assert!(csv.contains("a,b"));
    }

    // ── convert: config formats ─────────────────────

    #[test]
    fn convert_json_to_yaml() {
        let json = br#"{"key":"value","num":42}"#;
        let result = convert(json, r#"{"from":"json","to":"yaml"}"#).unwrap();
        let yaml = String::from_utf8(result).unwrap();
        assert!(yaml.contains("key:"));
        assert!(yaml.contains("value"));
    }

    #[test]
    fn convert_yaml_to_json() {
        let yaml = b"name: test\ncount: 5";
        let result = convert(yaml, r#"{"from":"yaml","to":"json"}"#).unwrap();
        let json: serde_json::Value =
            serde_json::from_str(&String::from_utf8(result).unwrap()).unwrap();
        assert_eq!(json["name"], "test");
        assert_eq!(json["count"], 5);
    }

    #[test]
    fn convert_json_to_toml() {
        let json = br#"{"title":"Test","version":1}"#;
        let result = convert(json, r#"{"from":"json","to":"toml"}"#).unwrap();
        let toml = String::from_utf8(result).unwrap();
        assert!(toml.contains("title"));
        assert!(toml.contains("Test"));
    }

    #[test]
    fn convert_toml_to_json() {
        let toml = b"name = \"example\"\nversion = 1";
        let result = convert(toml, r#"{"from":"toml","to":"json"}"#).unwrap();
        let json: serde_json::Value =
            serde_json::from_str(&String::from_utf8(result).unwrap()).unwrap();
        assert_eq!(json["name"], "example");
        assert_eq!(json["version"], 1);
    }

    // ── convert: image (uses real PNG bytes) ────────

    #[test]
    fn convert_png_to_jpg() {
        // Minimal 1x1 red PNG
        let png = create_test_png();
        let result = convert(&png, r#"{"from":"png","to":"jpg"}"#);
        assert!(result.is_ok());
        let jpg = result.unwrap();
        // JPEG starts with FF D8
        assert_eq!(&jpg[..2], &[0xFF, 0xD8]);
    }

    #[test]
    fn convert_png_to_webp() {
        let png = create_test_png();
        let result = convert(&png, r#"{"from":"png","to":"webp"}"#);
        assert!(result.is_ok());
    }

    #[test]
    fn convert_invalid_image_data() {
        let result = convert(b"not an image", r#"{"from":"png","to":"jpg"}"#);
        assert!(result.is_err());
    }

    // ── archive tests ───────────────────────────────

    #[test]
    fn archive_zip_roundtrip() {
        let entries = vec![
            archive::ArchiveEntry {
                name: "hello.txt".into(),
                data: b"Hello".to_vec(),
            },
            archive::ArchiveEntry {
                name: "world.txt".into(),
                data: b"World".to_vec(),
            },
        ];
        let zip_data = archive::create_zip(&entries).unwrap();
        // ZIP magic bytes: PK\x03\x04
        assert_eq!(&zip_data[..4], b"PK\x03\x04");

        // Verify we can read it back
        let reader = std::io::Cursor::new(&zip_data);
        let mut zip = zip::ZipArchive::new(reader).unwrap();
        assert_eq!(zip.len(), 2);

        let mut file = zip.by_name("hello.txt").unwrap();
        let mut contents = String::new();
        std::io::Read::read_to_string(&mut file, &mut contents).unwrap();
        assert_eq!(contents, "Hello");
    }

    #[test]
    fn archive_tar_gz_roundtrip() {
        let entries = vec![archive::ArchiveEntry {
            name: "test.txt".into(),
            data: b"test content".to_vec(),
        }];
        let data = archive::create_tar_gz(&entries).unwrap();
        // gzip magic bytes: 1f 8b
        assert_eq!(&data[..2], &[0x1f, 0x8b]);

        // Decompress and read tar
        let decoder = flate2::read::GzDecoder::new(std::io::Cursor::new(&data));
        let mut tar = tar::Archive::new(decoder);
        let entries: Vec<_> = tar.entries().unwrap().collect();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn archive_tar_xz() {
        let entries = vec![archive::ArchiveEntry {
            name: "file.bin".into(),
            data: vec![1, 2, 3, 4, 5],
        }];
        let data = archive::create_tar_xz(&entries).unwrap();
        // XZ magic bytes: FD 37 7A 58 5A 00
        assert_eq!(&data[..6], &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]);
    }

    #[test]
    fn archive_7z() {
        let entries = vec![
            archive::ArchiveEntry {
                name: "a.txt".into(),
                data: b"alpha".to_vec(),
            },
            archive::ArchiveEntry {
                name: "b.txt".into(),
                data: b"bravo".to_vec(),
            },
        ];
        let data = archive::create_7z(&entries).unwrap();
        // 7z magic bytes: 37 7A BC AF 27 1C
        assert_eq!(&data[..6], &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]);
    }

    #[test]
    fn archive_empty_entries() {
        let entries: Vec<archive::ArchiveEntry> = vec![];
        // All archive formats should handle empty entries gracefully
        assert!(archive::create_zip(&entries).is_ok());
        assert!(archive::create_tar_gz(&entries).is_ok());
        assert!(archive::create_tar_xz(&entries).is_ok());
        assert!(archive::create_7z(&entries).is_ok());
    }

    #[test]
    fn archive_large_file() {
        let entries = vec![archive::ArchiveEntry {
            name: "big.bin".into(),
            data: vec![0xAB; 100_000],
        }];
        assert!(archive::create_zip(&entries).is_ok());
        assert!(archive::create_7z(&entries).is_ok());
    }

    // ── helpers ─────────────────────────────────────

    fn create_test_png() -> Vec<u8> {
        use image::{ImageBuffer, Rgba};
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_pixel(1, 1, Rgba([255, 0, 0, 255]));
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
        buf.into_inner()
    }

    /// Synthesise a short mono sine-wave WAV (440 Hz, 44100 Hz sample rate, 16-bit PCM).
    fn create_test_wav_mono() -> Vec<u8> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut buf = std::io::Cursor::new(Vec::new());
        let mut writer = hound::WavWriter::new(&mut buf, spec).unwrap();
        for i in 0_u32..2205 {
            // 50 ms
            let s = (i16::MAX as f32
                * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin())
                as i16;
            writer.write_sample(s).unwrap();
        }
        writer.finalize().unwrap();
        buf.into_inner()
    }

    /// Synthesise a short stereo WAV (48000 Hz sample rate, 16-bit PCM).
    fn create_test_wav_stereo() -> Vec<u8> {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 48000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut buf = std::io::Cursor::new(Vec::new());
        let mut writer = hound::WavWriter::new(&mut buf, spec).unwrap();
        for i in 0_u32..4800 {
            // 100 ms stereo (left + right)
            let s = (i16::MAX as f32
                * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 48000.0).sin())
                as i16;
            writer.write_sample(s).unwrap(); // L
            writer.write_sample(s).unwrap(); // R
        }
        writer.finalize().unwrap();
        buf.into_inner()
    }

    // ── convert: audio ──────────────────────────────

    #[test]
    fn audio_wav_to_wav_roundtrip() {
        let wav = create_test_wav_mono();
        let result = convert(&wav, r#"{"from":"wav","to":"wav"}"#);
        assert!(result.is_ok(), "WAV→WAV failed: {:?}", result.err());
        let out = result.unwrap();
        // Output must start with RIFF header
        assert_eq!(&out[..4], b"RIFF");
        assert_eq!(&out[8..12], b"WAVE");
        // Output must be a valid WAV readable by hound
        let reader = hound::WavReader::new(std::io::Cursor::new(&out)).unwrap();
        let spec = reader.spec();
        assert_eq!(spec.channels, 1);
        assert_eq!(spec.sample_rate, 44100);
        assert_eq!(spec.bits_per_sample, 16);
    }

    #[test]
    fn audio_stereo_wav_preserves_channels_and_rate() {
        let wav = create_test_wav_stereo();
        let result = convert(&wav, r#"{"from":"wav","to":"wav"}"#);
        assert!(result.is_ok(), "{:?}", result.err());
        let reader = hound::WavReader::new(std::io::Cursor::new(result.unwrap())).unwrap();
        let spec = reader.spec();
        assert_eq!(spec.channels, 2);
        assert_eq!(spec.sample_rate, 48000);
    }

    #[test]
    fn audio_wav_sample_count_preserved() {
        // 2205 input samples → same count of 16-bit output samples
        let wav = create_test_wav_mono();
        let out = convert(&wav, r#"{"from":"wav","to":"wav"}"#).unwrap();
        let mut reader = hound::WavReader::new(std::io::Cursor::new(out)).unwrap();
        let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
        assert_eq!(samples.len(), 2205);
    }

    #[test]
    fn audio_invalid_mp3_returns_error() {
        let result = convert(
            b"this is not an mp3 file at all",
            r#"{"from":"mp3","to":"wav"}"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn audio_empty_input_returns_error() {
        let result = convert(b"", r#"{"from":"mp3","to":"wav"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn audio_invalid_flac_returns_error() {
        let result = convert(b"\x00\x01\x02\x03garbage", r#"{"from":"flac","to":"wav"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn audio_invalid_ogg_returns_error() {
        let result = convert(b"not an ogg file", r#"{"from":"ogg","to":"wav"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn audio_detect_format_extensions() {
        assert_eq!(detect_format("track.mp3"), Some("mp3".into()));
        assert_eq!(detect_format("song.flac"), Some("flac".into()));
        assert_eq!(detect_format("audio.ogg"), Some("ogg".into()));
        assert_eq!(detect_format("clip.wav"), Some("wav".into()));
        assert_eq!(detect_format("SONG.MP3"), Some("mp3".into()));
    }

    #[test]
    fn audio_output_formats_mp3() {
        assert_eq!(get_output_formats("mp3"), vec!["wav"]);
    }

    #[test]
    fn audio_output_formats_flac() {
        assert_eq!(get_output_formats("flac"), vec!["wav"]);
    }

    #[test]
    fn audio_output_formats_ogg() {
        assert_eq!(get_output_formats("ogg"), vec!["wav"]);
    }

    #[test]
    fn audio_output_formats_wav() {
        assert_eq!(get_output_formats("wav"), vec!["wav"]);
    }

    #[test]
    fn audio_unsupported_output_format_errors() {
        // Audio → non-WAV should be an unsupported conversion
        let result = convert(b"data", r#"{"from":"mp3","to":"png"}"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported conversion"));
    }
}
