use serde::Deserialize;

pub mod archive;
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
    )
}
