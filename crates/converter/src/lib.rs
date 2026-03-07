use serde::Deserialize;

pub mod archive;
mod audio;
mod document;
mod image_conv;
mod spreadsheet;

/// All input file formats the converter can handle.
/// This is the single source of truth for the file picker's `accept` attribute.
pub const ALL_INPUT_FORMATS: &[&str] = &[
    // images
    "png", "jpg", "jpeg", "webp", "gif", "bmp", "tiff", "tif", "ico", "qoi", "tga", "hdr", "dds",
    "exr", "svg", // audio
    "mp3", "flac", "ogg", "wav", // documents
    "md", "markdown", "html", "txt", "docx", "rtf", "pdf", // data / config
    "csv", "tsv", "json", "yaml", "yml", "toml", // encoding
    "base64",
];

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
        (f, t) if is_image_input_format(f) && is_image_output_format(t) => {
            image_conv::convert_image(input, f, t, config.quality)
        }
        ("svg", "png") => image_conv::svg_to_png(input),

        // Documents
        ("md" | "markdown", "html") => document::markdown_to_html(input),
        ("md" | "markdown", "txt" | "text") => document::markdown_to_text(input),
        ("md" | "markdown", "pdf") => document::markdown_to_pdf(input),
        ("html", "md" | "markdown") => document::html_to_markdown(input),
        ("html", "pdf") => document::html_to_pdf(input),
        ("docx", "txt" | "text") => document::docx_to_text(input),
        ("docx", "html") => document::docx_to_html(input),
        ("rtf", "txt" | "text") => document::rtf_to_text(input),
        // PDF
        ("pdf", "txt" | "text") => document::pdf_to_text(input),
        ("pdf", "html") => document::pdf_to_html(input),
        ("txt" | "text", "pdf") => document::text_to_pdf(input),

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
        "png" => vec![
            "jpg", "webp", "avif", "gif", "bmp", "tiff", "qoi", "tga", "ico",
        ],
        "jpg" | "jpeg" => vec![
            "png", "webp", "avif", "gif", "bmp", "tiff", "qoi", "tga", "ico",
        ],
        "webp" => vec![
            "png", "jpg", "avif", "gif", "bmp", "tiff", "qoi", "tga", "ico",
        ],
        "gif" => vec![
            "png", "jpg", "webp", "avif", "bmp", "tiff", "qoi", "tga", "ico",
        ],
        "bmp" => vec![
            "png", "jpg", "webp", "avif", "gif", "tiff", "qoi", "tga", "ico",
        ],
        "tiff" | "tif" => vec![
            "png", "jpg", "webp", "avif", "gif", "bmp", "qoi", "tga", "ico",
        ],
        "ico" => vec![
            "png", "jpg", "webp", "avif", "gif", "bmp", "tiff", "qoi", "tga",
        ],
        "qoi" => vec![
            "png", "jpg", "webp", "avif", "gif", "bmp", "tiff", "tga", "ico",
        ],
        "tga" => vec![
            "png", "jpg", "webp", "avif", "gif", "bmp", "tiff", "qoi", "ico",
        ],
        "hdr" => vec!["png", "jpg", "webp", "avif", "bmp", "tiff", "tga"],
        "dds" => vec!["png", "jpg", "webp", "avif", "bmp", "tiff", "tga"],
        "exr" => vec!["png", "jpg", "webp", "avif", "bmp", "tiff", "tga"],
        "svg" => vec!["png"],
        "mp3" | "flac" | "ogg" | "wav" => vec!["wav"],
        "md" | "markdown" => vec!["html", "txt", "pdf"],
        "html" => vec!["md", "pdf"],
        "docx" => vec!["txt", "html"],
        "rtf" => vec!["txt"],
        "pdf" => vec!["txt", "html"],
        "txt" | "text" => vec!["pdf"],
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

/// Formats that can be read as image input (excludes AVIF, which needs dav1d C lib for decode).
fn is_image_input_format(fmt: &str) -> bool {
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
            | "ico"
            | "qoi"
            | "tga"
            | "hdr"
            | "dds"
            | "exr"
    )
}

/// Formats that can be written as image output (includes AVIF via pure-Rust ravif encoder).
fn is_image_output_format(fmt: &str) -> bool {
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
            | "avif"
            | "ico"
            | "qoi"
            | "tga"
            | "hdr"
            | "dds"
            | "exr"
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
            | "avif"
            | "ico"
            | "qoi"
            | "tga"
            | "hdr"
            | "dds"
            | "exr"
            | "svg"
            | "md"
            | "markdown"
            | "html"
            | "txt"
            | "text"
            | "docx"
            | "rtf"
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
            | "pdf"
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
        assert!(fmts.contains(&"html"));
        assert!(fmts.contains(&"txt"));
        assert!(fmts.contains(&"pdf"));
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

    /// Every format listed in ALL_INPUT_FORMATS must return at least one output
    /// option from get_output_formats.  This ensures the file picker never
    /// accepts a file type for which no conversion is offered.
    #[test]
    fn all_input_formats_have_output_options() {
        for fmt in ALL_INPUT_FORMATS {
            let outputs = get_output_formats(fmt);
            assert!(
                !outputs.is_empty(),
                "ALL_INPUT_FORMATS contains '{}' but get_output_formats returns nothing for it",
                fmt
            );
        }
    }

    /// ALL_INPUT_FORMATS must not list any format that is_known_format doesn't
    /// recognise, otherwise detect_format would silently drop those files.
    #[test]
    fn all_input_formats_are_known() {
        for fmt in ALL_INPUT_FORMATS {
            assert!(
                is_known_format(fmt),
                "ALL_INPUT_FORMATS contains '{}' which is not in is_known_format",
                fmt
            );
        }
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
    fn convert_png_to_avif() {
        let png = create_test_png();
        let result = convert(&png, r#"{"from":"png","to":"avif"}"#);
        assert!(
            result.is_ok(),
            "Expected AVIF output, got: {:?}",
            result.err()
        );
    }

    #[test]
    fn convert_png_to_qoi() {
        let png = create_test_png();
        let result = convert(&png, r#"{"from":"png","to":"qoi"}"#);
        assert!(result.is_ok());
        let qoi = result.unwrap();
        // QOI magic: "qoif"
        assert_eq!(&qoi[..4], b"qoif");
    }

    #[test]
    fn convert_qoi_roundtrip() {
        let png = create_test_png();
        let qoi = convert(&png, r#"{"from":"png","to":"qoi"}"#).unwrap();
        let back = convert(&qoi, r#"{"from":"qoi","to":"png"}"#);
        assert!(back.is_ok());
        assert_eq!(&back.unwrap()[..4], b"\x89PNG");
    }

    #[test]
    fn convert_png_to_tga() {
        let png = create_test_png();
        let result = convert(&png, r#"{"from":"png","to":"tga"}"#);
        assert!(result.is_ok());
    }

    #[test]
    fn convert_png_to_ico() {
        let png = create_test_png();
        let result = convert(&png, r#"{"from":"png","to":"ico"}"#);
        assert!(result.is_ok());
        let ico = result.unwrap();
        // ICO magic: 00 00 01 00
        assert_eq!(&ico[..4], &[0x00, 0x00, 0x01, 0x00]);
    }

    #[test]
    fn convert_ico_to_png() {
        // Create an ICO from a PNG, then convert back to PNG
        let original_png = create_test_png();
        let ico = convert(&original_png, r#"{"from":"png","to":"ico"}"#).unwrap();
        let png = convert(&ico, r#"{"from":"ico","to":"png"}"#);
        assert!(png.is_ok());
        assert_eq!(&png.unwrap()[..4], b"\x89PNG");
    }

    #[test]
    fn convert_docx_to_text() {
        // Create a minimal valid DOCX ZIP file containing word/document.xml
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>Hello World</w:t></w:r></w:p>
    <w:p><w:r><w:t>Second paragraph</w:t></w:r></w:p>
  </w:body>
</w:document>"#;
        let docx = make_minimal_docx(xml);
        let result = convert(&docx, r#"{"from":"docx","to":"txt"}"#).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains("Hello World"));
        assert!(text.contains("Second paragraph"));
    }

    #[test]
    fn convert_docx_to_html() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>Content</w:t></w:r></w:p>
  </w:body>
</w:document>"#;
        let docx = make_minimal_docx(xml);
        let result = convert(&docx, r#"{"from":"docx","to":"html"}"#).unwrap();
        let html = String::from_utf8(result).unwrap();
        assert!(html.contains("<html>") || html.contains("<p>"));
        assert!(html.contains("Content"));
    }

    #[test]
    fn convert_docx_missing_document_xml() {
        // A ZIP that doesn't contain word/document.xml
        let entries = vec![archive::ArchiveEntry {
            name: "unrelated.txt".into(),
            data: b"not a docx".to_vec(),
        }];
        let zip = archive::create_zip(&entries).unwrap();
        let result = convert(&zip, r#"{"from":"docx","to":"txt"}"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("word/document.xml"));
    }

    #[test]
    fn convert_rtf_to_text() {
        let rtf = br#"{\rtf1\ansi Hello World}"#;
        let result = convert(rtf, r#"{"from":"rtf","to":"txt"}"#);
        assert!(result.is_ok(), "RTF conversion failed: {:?}", result.err());
        let text = String::from_utf8(result.unwrap()).unwrap();
        assert!(text.contains("Hello"));
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

    /// Build a minimal DOCX ZIP containing only `word/document.xml` with the
    /// provided XML content. Just enough to test our extraction logic.
    fn make_minimal_docx(document_xml: &str) -> Vec<u8> {
        let entries = vec![archive::ArchiveEntry {
            name: "word/document.xml".into(),
            data: document_xml.as_bytes().to_vec(),
        }];
        archive::create_zip(&entries).unwrap()
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

    // ── detect_format: pdf / txt ─────────────────────

    #[test]
    fn detect_format_pdf_and_txt() {
        assert_eq!(detect_format("report.pdf"), Some("pdf".into()));
        assert_eq!(detect_format("README.txt"), Some("txt".into()));
        assert_eq!(detect_format("notes.text"), Some("text".into()));
        assert_eq!(detect_format("REPORT.PDF"), Some("pdf".into()));
    }

    // ── get_output_formats: pdf / html / txt ─────────

    #[test]
    fn output_formats_for_pdf() {
        let fmts = get_output_formats("pdf");
        assert!(fmts.contains(&"txt"));
        assert!(fmts.contains(&"html"));
    }

    #[test]
    fn output_formats_for_txt() {
        let fmts = get_output_formats("txt");
        assert!(fmts.contains(&"pdf"));
    }

    #[test]
    fn output_formats_for_html_includes_pdf() {
        let fmts = get_output_formats("html");
        assert!(fmts.contains(&"md"));
        assert!(fmts.contains(&"pdf"));
    }

    // ── convert: PDF generation (text → pdf) ─────────

    #[test]
    fn text_to_pdf_produces_valid_header() {
        let result = convert(b"Hello World", r#"{"from":"txt","to":"pdf"}"#);
        assert!(result.is_ok(), "text→pdf failed: {:?}", result.err());
        let pdf = result.unwrap();
        assert!(pdf.starts_with(b"%PDF-"), "output should start with %PDF-");
        assert!(
            pdf.windows(5).any(|w| w == b"%%EOF"),
            "output should contain %%EOF"
        );
    }

    #[test]
    fn text_to_pdf_non_empty_output() {
        let result = convert(b"Line 1\nLine 2\nLine 3", r#"{"from":"txt","to":"pdf"}"#).unwrap();
        // A real PDF with 3 lines should be several hundred bytes at least
        assert!(result.len() > 200, "PDF output unexpectedly small");
    }

    #[test]
    fn text_to_pdf_empty_input() {
        // Empty text should still produce a valid PDF
        let result = convert(b"", r#"{"from":"txt","to":"pdf"}"#);
        assert!(result.is_ok());
        let pdf = result.unwrap();
        assert!(pdf.starts_with(b"%PDF-"));
    }

    #[test]
    fn text_to_pdf_with_special_chars() {
        // Parentheses and backslashes must be escaped in PDF string literals
        let result = convert(
            b"Price: (100) + (200) = (300) \\taxed",
            r#"{"from":"txt","to":"pdf"}"#,
        );
        assert!(result.is_ok(), "special chars failed: {:?}", result.err());
        let pdf = result.unwrap();
        assert!(pdf.starts_with(b"%PDF-"));
    }

    #[test]
    fn text_to_pdf_multipage() {
        // ~60 lines per page; send 120 lines to force 2 pages
        let long_text = (0..120)
            .map(|i| format!("Line number {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let result = convert(long_text.as_bytes(), r#"{"from":"txt","to":"pdf"}"#).unwrap();
        assert!(result.starts_with(b"%PDF-"));
        // With multiple pages the PDF will be larger
        assert!(result.len() > 1000);
    }

    #[test]
    fn text_to_pdf_long_line_wraps() {
        // A single line longer than CHARS_PER_LINE should still produce valid PDF
        let long_line = "A".repeat(300);
        let result = convert(long_line.as_bytes(), r#"{"from":"txt","to":"pdf"}"#);
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with(b"%PDF-"));
    }

    // ── convert: PDF reading (pdf → text) ────────────

    #[test]
    fn pdf_text_roundtrip() {
        // Generate a PDF from known text, then extract the text back out.
        let original = b"Hello from Transfigure PDF roundtrip test";
        let pdf = convert(original, r#"{"from":"txt","to":"pdf"}"#).unwrap();
        let extracted = convert(&pdf, r#"{"from":"pdf","to":"txt"}"#);
        assert!(extracted.is_ok(), "pdf→txt failed: {:?}", extracted.err());
        let text = String::from_utf8(extracted.unwrap()).unwrap();
        assert!(
            text.contains("Hello from Transfigure"),
            "extracted text should contain original content, got: {text:?}"
        );
    }

    #[test]
    fn pdf_to_html_contains_pre_tag() {
        let pdf = convert(b"Sample text", r#"{"from":"txt","to":"pdf"}"#).unwrap();
        let html_result = convert(&pdf, r#"{"from":"pdf","to":"html"}"#);
        assert!(
            html_result.is_ok(),
            "pdf→html failed: {:?}",
            html_result.err()
        );
        let html = String::from_utf8(html_result.unwrap()).unwrap();
        assert!(html.contains("<pre>") || html.contains("<html>"));
    }

    #[test]
    fn pdf_to_text_invalid_data() {
        let result = convert(b"this is not a pdf", r#"{"from":"pdf","to":"txt"}"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Failed to load PDF"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn pdf_to_html_invalid_data() {
        let result = convert(b"\x00\x01\x02\x03", r#"{"from":"pdf","to":"html"}"#);
        assert!(result.is_err());
    }

    // ── convert: Markdown → PDF ───────────────────────

    #[test]
    fn markdown_to_pdf_produces_valid_pdf() {
        let md = b"# Title\n\nSome **bold** paragraph text.";
        let result = convert(md, r#"{"from":"md","to":"pdf"}"#);
        assert!(result.is_ok(), "md→pdf failed: {:?}", result.err());
        let pdf = result.unwrap();
        assert!(pdf.starts_with(b"%PDF-"));
    }

    #[test]
    fn markdown_to_pdf_content_preserved() {
        let md = b"Hello Markdown PDF content here";
        let pdf = convert(md, r#"{"from":"md","to":"pdf"}"#).unwrap();
        // Round-trip: extract text from the generated PDF
        let text_result = convert(&pdf, r#"{"from":"pdf","to":"txt"}"#);
        assert!(text_result.is_ok(), "{:?}", text_result.err());
        let text = String::from_utf8(text_result.unwrap()).unwrap();
        assert!(
            text.contains("Hello Markdown PDF"),
            "expected content in extracted text, got: {text:?}"
        );
    }

    // ── convert: HTML → PDF ───────────────────────────

    #[test]
    fn html_to_pdf_produces_valid_pdf() {
        let html = b"<h1>Title</h1><p>Hello world.</p>";
        let result = convert(html, r#"{"from":"html","to":"pdf"}"#);
        assert!(result.is_ok(), "html→pdf failed: {:?}", result.err());
        let pdf = result.unwrap();
        assert!(pdf.starts_with(b"%PDF-"));
    }

    #[test]
    fn html_to_pdf_strips_tags() {
        let html = b"<p>Stripped content here</p>";
        let pdf = convert(html, r#"{"from":"html","to":"pdf"}"#).unwrap();
        // After stripping, "Stripped content here" should still appear in the PDF
        let text = convert(&pdf, r#"{"from":"pdf","to":"txt"}"#).unwrap();
        let s = String::from_utf8(text).unwrap();
        assert!(
            s.contains("Stripped content"),
            "HTML content should survive PDF roundtrip, got: {s:?}"
        );
    }

    // ── csv / tsv edge cases ──────────────────────────

    #[test]
    fn csv_to_json_quoted_fields() {
        let csv = b"name,note\n\"Smith, John\",\"has, commas\"";
        let result = convert(csv, r#"{"from":"csv","to":"json"}"#).unwrap();
        let json_str = String::from_utf8(result).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v[0]["name"], "Smith, John");
        assert_eq!(v[0]["note"], "has, commas");
    }

    #[test]
    fn json_to_csv_missing_fields_empty() {
        // Row 2 is missing "age" → that column should be empty string
        let json = br#"[{"name":"Alice","age":30},{"name":"Bob"}]"#;
        let result = convert(json, r#"{"from":"json","to":"csv"}"#).unwrap();
        let s = String::from_utf8(result).unwrap();
        assert!(s.contains("Bob"));
        assert!(s.lines().count() >= 3); // header + 2 rows
    }

    #[test]
    fn csv_single_row_no_data() {
        // Header only, no data rows
        let csv = b"col1,col2\n";
        let result = convert(csv, r#"{"from":"csv","to":"json"}"#).unwrap();
        let v: serde_json::Value =
            serde_json::from_str(&String::from_utf8(result).unwrap()).unwrap();
        assert!(v.as_array().unwrap().is_empty());
    }

    // ── yaml / toml round-trips ───────────────────────

    #[test]
    fn yaml_to_json_roundtrip_string_values() {
        let yaml = b"city: Paris\ncountry: France";
        let json = convert(yaml, r#"{"from":"yaml","to":"json"}"#).unwrap();
        let v: serde_json::Value = serde_json::from_str(&String::from_utf8(json).unwrap()).unwrap();
        assert_eq!(v["city"], "Paris");
        assert_eq!(v["country"], "France");
    }

    #[test]
    fn toml_to_json_section_table() {
        let toml = b"[server]\nport = 8080\nhost = \"localhost\"";
        let json = convert(toml, r#"{"from":"toml","to":"json"}"#).unwrap();
        let v: serde_json::Value = serde_json::from_str(&String::from_utf8(json).unwrap()).unwrap();
        assert_eq!(v["server"]["port"], 8080);
        assert_eq!(v["server"]["host"], "localhost");
    }

    // ── markdown edge cases ───────────────────────────

    #[test]
    fn markdown_code_block_to_html() {
        let md = b"```rust\nfn main() {}\n```";
        let html = convert(md, r#"{"from":"md","to":"html"}"#).unwrap();
        let s = String::from_utf8(html).unwrap();
        assert!(s.contains("<code") || s.contains("main"));
    }

    #[test]
    fn markdown_list_to_text() {
        let md = b"- item one\n- item two\n- item three";
        let text = convert(md, r#"{"from":"md","to":"txt"}"#).unwrap();
        let s = String::from_utf8(text).unwrap();
        assert!(s.contains("item one"));
        assert!(s.contains("item two"));
    }

    #[test]
    fn html_to_markdown_links() {
        let html = b"<p>Visit <strong>here</strong></p>";
        let md = convert(html, r#"{"from":"html","to":"md"}"#).unwrap();
        let s = String::from_utf8(md).unwrap();
        assert!(s.contains("here"));
        assert!(s.contains("**"));
    }
}
