//! Every conversion the UI offers, run against real files made by other tools
//! (see `fixtures/generate.py`), with each output checked for validity and
//! content rather than just "didn't error".

use std::path::Path;

use serde_json::Value;

const MARKER: &str = "Transfigure fixture — Café naïve 42";

fn fixtures() -> Vec<(String, Vec<u8>)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut files: Vec<(String, Vec<u8>)> = std::fs::read_dir(&dir)
        .expect("fixtures directory")
        .filter_map(|e| {
            let path = e.ok()?.path();
            let name = path.file_name()?.to_str()?.to_string();
            if name.ends_with(".py") {
                return None;
            }
            Some((name, std::fs::read(&path).ok()?))
        })
        .collect();
    files.sort();
    files
}

fn ext(name: &str) -> String {
    converter::detect_format(name).unwrap_or_else(|| panic!("fixture {name} has no known format"))
}

fn convert(data: &[u8], from: &str, to: &str) -> Result<Vec<u8>, String> {
    converter::convert(
        data,
        &serde_json::json!({ "from": from, "to": to, "quality": 85 }).to_string(),
    )
}

fn text_of(data: &[u8], from: &str) -> Result<String, String> {
    let bytes = if from == "txt" {
        data.to_vec()
    } else {
        convert(data, from, "txt")?
    };
    String::from_utf8(bytes).map_err(|e| format!("output is not UTF-8: {e}"))
}

// ── Images ──────────────────────────────────────────────

fn decode_image(data: &[u8], fmt: &str) -> Result<image::DynamicImage, String> {
    match fmt {
        "ico" => {
            let dir = ico::IconDir::read(std::io::Cursor::new(data)).map_err(|e| e.to_string())?;
            let e = dir
                .entries()
                .iter()
                .max_by_key(|e| e.width())
                .ok_or("empty ICO")?;
            let img = e.decode().map_err(|e| e.to_string())?;
            Ok(image::DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(img.width(), img.height(), img.rgba_data().to_vec())
                    .ok_or("bad ICO pixels")?,
            ))
        }
        "avif" => {
            // No pure-Rust AVIF decoder here; validate the container instead.
            if data.len() > 12 && &data[4..12] == b"ftypavif" {
                Ok(image::DynamicImage::new_rgb8(0, 0))
            } else {
                Err("not an AVIF file".into())
            }
        }
        // TGA has no magic number, so it can't be sniffed.
        "tga" => image::load_from_memory_with_format(data, image::ImageFormat::Tga)
            .map_err(|e| e.to_string()),
        _ => image::load_from_memory(data).map_err(|e| e.to_string()),
    }
}

fn flatten(img: &image::DynamicImage) -> image::RgbImage {
    let rgba = img.to_rgba8();
    image::RgbImage::from_fn(rgba.width(), rgba.height(), |x, y| {
        let p = rgba.get_pixel(x, y);
        let a = p[3] as u32;
        let mix = |c: u8| ((c as u32 * a + 255 * (255 - a)) / 255) as u8;
        image::Rgb([mix(p[0]), mix(p[1]), mix(p[2])])
    })
}

fn mean_abs_diff(a: &image::RgbImage, b: &image::RgbImage) -> f64 {
    let total: u64 = a
        .as_raw()
        .iter()
        .zip(b.as_raw())
        .map(|(x, y)| x.abs_diff(*y) as u64)
        .sum();
    total as f64 / a.as_raw().len().max(1) as f64
}

fn check_image(src: &[u8], from: &str, to: &str, out: &[u8]) -> Result<(), String> {
    if to == "pdf" {
        let doc = lopdf::Document::load_mem(out).map_err(|e| format!("invalid PDF: {e}"))?;
        return (doc.get_pages().len() == 1)
            .then_some(())
            .ok_or_else(|| "expected one page".into());
    }
    let reference = decode_image(&convert(src, from, "png")?, "png")?;
    let img = decode_image(out, to)?;
    if to == "avif" {
        return Ok(());
    }
    let (rw, rh) = (reference.width(), reference.height());
    if to == "ico" {
        let side = rw.max(rh).min(256);
        return (img.width() == side && img.height() == side)
            .then_some(())
            .ok_or_else(|| format!("ICO is {}x{}, expected {side}²", img.width(), img.height()));
    }
    if (img.width(), img.height()) != (rw, rh) {
        return Err(format!(
            "size {}x{}, expected {rw}x{rh}",
            img.width(),
            img.height()
        ));
    }
    let diff = mean_abs_diff(&flatten(&reference), &flatten(&img));
    let limit = match to {
        "jpg" => 12.0,
        "gif" => 16.0,
        _ => 1.0,
    };
    if diff > limit {
        return Err(format!(
            "pixels differ from reference by {diff:.1} (limit {limit})"
        ));
    }
    Ok(())
}

// ── Audio ───────────────────────────────────────────────

struct Wav {
    channels: u16,
    rate: u32,
    frames: usize,
    left: Vec<f32>,
}

fn read_wav(data: &[u8]) -> Result<Wav, String> {
    let mut r = hound::WavReader::new(std::io::Cursor::new(data)).map_err(|e| e.to_string())?;
    let spec = r.spec();
    let scale = (1i64 << (spec.bits_per_sample - 1)) as f32;
    let samples: Vec<f32> = r
        .samples::<i32>()
        .map(|s| s.map(|v| v as f32 / scale))
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    let ch = spec.channels as usize;
    Ok(Wav {
        channels: spec.channels,
        rate: spec.sample_rate,
        frames: samples.len() / ch,
        left: samples.iter().step_by(ch).copied().collect(),
    })
}

/// Frequency from zero crossings, over the steady middle of the clip.
fn pitch(samples: &[f32], rate: u32) -> f32 {
    let mid = &samples[samples.len() / 4..samples.len() * 3 / 4];
    let crossings = mid.windows(2).filter(|w| w[0] < 0.0 && w[1] >= 0.0).count();
    crossings as f32 * rate as f32 / mid.len() as f32
}

fn check_audio(name: &str, to: &str, out: &[u8]) -> Result<(), String> {
    let wav_bytes = if to == "wav" {
        out.to_vec()
    } else {
        convert(out, to, "wav")?
    };
    let wav = read_wav(&wav_bytes)?;
    let want_channels = if name.contains("mono") { 1 } else { 2 };
    if wav.channels != want_channels {
        return Err(format!(
            "{} channels, expected {want_channels}",
            wav.channels
        ));
    }
    if wav.rate != 44100 {
        return Err(format!("{} Hz, expected 44100", wav.rate));
    }
    let secs = wav.frames as f32 / wav.rate as f32;
    if !(0.55..=0.75).contains(&secs) {
        return Err(format!("duration {secs:.3}s, expected ≈0.6s"));
    }
    let f = pitch(&wav.left, wav.rate);
    if (f - 440.0).abs() > 15.0 {
        return Err(format!("left channel pitch {f:.0} Hz, expected 440 Hz"));
    }
    Ok(())
}

// ── Documents ───────────────────────────────────────────

fn check_document(from: &str, to: &str, out: &[u8]) -> Result<(), String> {
    let text = match to {
        "pdf" | "docx" => text_of(out, to)?,
        _ => String::from_utf8(out.to_vec()).map_err(|e| format!("not UTF-8: {e}"))?,
    };
    let text = text.replace("&amp;", "&");
    if !text.contains("Café naïve 42") {
        return Err(format!(
            "marker text missing or mangled; got: {}",
            text.chars().take(160).collect::<String>()
        ));
    }
    // Structure survives from formats that carry it.
    if to == "md" && matches!(from, "html" | "docx" | "odt") {
        for needle in [
            "# Transfigure fixture",
            "**bold**",
            "*italic*",
            "[link](https://example.com",
            "First item",
            "| Name",
        ] {
            if !text.contains(needle) {
                return Err(format!("Markdown lacks {needle:?}:\n{text}"));
            }
        }
    }
    if to == "html" && matches!(from, "md" | "docx" | "odt" | "rtf") {
        for needle in ["<strong>bold</strong>", "<em>italic</em>"] {
            if !text.contains(needle) {
                return Err(format!("HTML lacks {needle:?}"));
            }
        }
    }
    Ok(())
}

// ── Tables ──────────────────────────────────────────────

fn table_rows(data: &[u8], fmt: &str) -> Result<Vec<Vec<String>>, String> {
    let csv_bytes = match fmt {
        "csv" => data.to_vec(),
        "json" => {
            let v: Value = serde_json::from_slice(data).map_err(|e| e.to_string())?;
            let arr = v.as_array().ok_or("JSON is not an array")?;
            let mut rows = vec![vec!["id".to_string(), "name".into(), "zip".into()]];
            for r in arr {
                rows.push(
                    ["id", "name", "zip"]
                        .iter()
                        .map(|k| match &r[*k] {
                            Value::String(s) => s.clone(),
                            other => other.to_string(),
                        })
                        .collect(),
                );
            }
            return Ok(rows);
        }
        other => convert(data, other, "csv")?,
    };
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(csv_bytes.as_slice());
    rdr.records()
        .map(|r| {
            r.map(|r| r.iter().map(str::to_string).collect())
                .map_err(|e| e.to_string())
        })
        .collect()
}

fn check_table(to: &str, out: &[u8]) -> Result<(), String> {
    let rows = table_rows(out, to)?;
    if rows.len() != 4 {
        return Err(format!("{} rows, expected 4", rows.len()));
    }
    let find = |col: &str| rows[0].iter().position(|h| h == col);
    let (Some(name), Some(zip)) = (find("name"), find("zip")) else {
        return Err(format!("header row wrong: {:?}", rows[0]));
    };
    if rows[1][name] != "Café" || rows[2][name] != "Naïve, Inc." {
        return Err(format!(
            "names wrong: {:?} / {:?}",
            rows[1][name], rows[2][name]
        ));
    }
    if rows[1][zip] != "00501" {
        return Err(format!("leading zero lost: zip is {:?}", rows[1][zip]));
    }
    Ok(())
}

/// A nested config flattened into one record with dotted column names.
fn check_config_table(to: &str, out: &[u8]) -> Result<(), String> {
    let csv_bytes = if to == "csv" {
        out.to_vec()
    } else {
        convert(out, to, "csv")?
    };
    let text = String::from_utf8(csv_bytes).map_err(|e| e.to_string())?;
    let mut lines = text.lines();
    let header = lines.next().unwrap_or("");
    for col in ["title", "owner.name", "numeric_string"] {
        if !header.split(',').any(|h| h == col) {
            return Err(format!("missing column {col}: {header}"));
        }
    }
    if lines.count() != 1 {
        return Err("expected one data row".into());
    }
    Ok(())
}

// ── Structured data ─────────────────────────────────────

fn as_json(data: &[u8], fmt: &str) -> Result<Value, String> {
    let bytes = if fmt == "json" {
        data.to_vec()
    } else {
        convert(data, fmt, "json")?
    };
    serde_json::from_slice(&bytes).map_err(|e| format!("not JSON: {e}"))
}

/// XML carries only text and has a single root element, so compare with
/// scalars as strings, empty values dropped and a lone root unwrapped.
fn loosen(v: &Value) -> Value {
    match v {
        Value::Object(o) if o.len() == 1 && o.values().all(|v| v.is_object() || v.is_array()) => {
            loosen(o.values().next().expect("one"))
        }
        Value::Object(o) => Value::Object(
            o.iter()
                .filter(|(_, v)| {
                    !matches!(v, Value::Null) && v.as_array().is_none_or(|a| !a.is_empty())
                })
                .map(|(k, v)| (k.clone(), loosen(v)))
                .collect(),
        ),
        Value::Array(a) => Value::Array(a.iter().map(loosen).collect()),
        Value::String(s) => Value::String(s.clone()),
        Value::Null => Value::String(String::new()),
        other => Value::String(other.to_string()),
    }
}

fn check_structured(src: &[u8], from: &str, to: &str, out: &[u8]) -> Result<(), String> {
    let mut original = as_json(src, from)?;
    let mut back = as_json(out, to)?;
    if to == "xml" {
        original = loosen(&original);
        back = loosen(&back);
    }
    // TOML documents must be tables, so a top-level array is wrapped.
    if to == "toml" && original.is_array() {
        back = back["items"].take();
    }
    if original != back {
        return Err(format!(
            "round trip changed the data:\n{original}\n→\n{back}"
        ));
    }
    if from != "xml" && original.is_object() && back["numeric_string"] != "007" {
        return Err("string \"007\" was turned into a number".into());
    }
    Ok(())
}

// ── The matrix ──────────────────────────────────────────

#[test]
fn every_offered_conversion_produces_valid_output() {
    let mut failures = Vec::new();
    let mut count = 0;
    for (name, data) in fixtures() {
        let from = ext(&name);
        let targets = converter::get_output_formats(&from);
        assert!(!targets.is_empty(), "{name}: no outputs offered");
        for to in targets {
            count += 1;
            let result = convert(&data, &from, to).and_then(|out| {
                // `TRANSFIGURE_DUMP=dir` keeps every output for inspection
                // with external tools.
                if let Ok(dir) = std::env::var("TRANSFIGURE_DUMP") {
                    let _ = std::fs::write(Path::new(&dir).join(format!("{name}.{to}")), &out);
                }
                if out.is_empty() && to != "bin" {
                    return Err("empty output".into());
                }
                match to {
                    "base64" => {
                        let back = convert(&out, "base64", "bin")?;
                        (back == data)
                            .then_some(())
                            .ok_or_else(|| "base64 round trip differs".into())
                    }
                    "bin" => (String::from_utf8_lossy(&out) == MARKER)
                        .then_some(())
                        .ok_or_else(|| "decoded bytes differ".into()),
                    _ if name.starts_with("image") => check_image(&data, &from, to, &out),
                    _ if name.starts_with("audio") => check_audio(&name, to, &out),
                    _ if name.starts_with("doc") => check_document(&from, to, &out),
                    "csv" | "tsv" | "xlsx" | "json" if name.starts_with("data") => {
                        check_table(to, &out)
                    }
                    "csv" | "tsv" | "xlsx" if name.starts_with("config") => {
                        check_config_table(to, &out)
                    }
                    _ if name.starts_with("data") || name.starts_with("config") => {
                        check_structured(&data, &from, to, &out)
                    }
                    _ => Err("no check for this fixture".into()),
                }
            });
            if let Err(e) = result {
                failures.push(format!("{name} → {to}: {e}"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {count} conversions failed:\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
    eprintln!("{count} conversions verified");
}

#[test]
fn file_picker_accepts_only_convertible_formats() {
    for fmt in converter::ALL_INPUT_FORMATS {
        let outs = converter::get_output_formats(fmt);
        let min = if *fmt == "base64" { 1 } else { 2 };
        assert!(outs.len() >= min, "{fmt} offers only {outs:?}");
    }
}

#[test]
fn phone_photo_orientation_is_applied() {
    // A 3×2 JPEG tagged "rotate 90° CW" (EXIF orientation 6) must come out 2×3.
    let img = image::RgbImage::from_pixel(3, 2, image::Rgb([200, 30, 30]));
    let mut jpg = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut jpg),
        image::ImageFormat::Jpeg,
    )
    .unwrap();
    // Minimal APP1 Exif segment: big-endian TIFF with one Orientation entry.
    let mut exif = b"Exif\0\0MM\0\x2a\0\0\0\x08\0\x01".to_vec();
    exif.extend_from_slice(&[0x01, 0x12, 0x00, 0x03, 0, 0, 0, 1, 0, 6, 0, 0]);
    exif.extend_from_slice(&[0, 0, 0, 0]);
    let mut tagged = jpg[..2].to_vec();
    tagged.extend_from_slice(&[0xff, 0xe1]);
    tagged.extend_from_slice(&((exif.len() + 2) as u16).to_be_bytes());
    tagged.extend_from_slice(&exif);
    tagged.extend_from_slice(&jpg[2..]);

    let png = convert(&tagged, "jpg", "png").unwrap();
    let out = image::load_from_memory(&png).unwrap();
    assert_eq!((out.width(), out.height()), (2, 3));
}
