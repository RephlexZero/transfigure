use wasm_bindgen::JsCast;
use web_sys::js_sys;

pub fn format_icon(ext: &str) -> &'static str {
    match ext {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "tiff" | "tif" | "svg" | "avif"
        | "ico" | "qoi" | "tga" | "hdr" | "dds" | "exr" => "🖼️",
        "md" | "markdown" | "html" | "txt" | "text" => "📝",
        "docx" | "rtf" | "pdf" => "📄",
        "csv" | "tsv" | "json" => "📊",
        "yaml" | "yml" | "toml" => "⚙️",
        "base64" => "🔐",
        _ => "📄",
    }
}

pub fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

pub fn mime_type_for(ext: &str) -> &'static str {
    match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "tiff" | "tif" => "image/tiff",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        "qoi" => "image/qoi",
        "tga" => "image/x-tga",
        "hdr" => "image/vnd.radiance",
        "dds" => "image/vnd.ms-dds",
        "exr" => "image/x-exr",
        "html" => "text/html",
        "md" | "markdown" => "text/markdown",
        "txt" | "text" => "text/plain",
        "rtf" => "application/rtf",
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "csv" => "text/csv",
        "tsv" => "text/tab-separated-values",
        "json" => "application/json",
        "yaml" | "yml" => "application/x-yaml",
        "toml" => "application/toml",
        _ => "application/octet-stream",
    }
}

pub fn make_output_name(original_name: &str, target_ext: &str) -> String {
    if let Some((stem, _)) = original_name.rsplit_once('.') {
        format!("{stem}.{target_ext}")
    } else {
        format!("{original_name}.{target_ext}")
    }
}

pub fn download_blob(data: &[u8], original_name: &str, target_ext: &str) {
    let output_name = make_output_name(original_name, target_ext);
    download_blob_raw(data, &output_name, mime_type_for(target_ext));
}

pub fn download_blob_raw(data: &[u8], filename: &str, mime: &str) {
    let array = js_sys::Uint8Array::from(data);
    let parts = js_sys::Array::new();
    parts.push(&array.buffer());

    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type(mime);

    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &opts).unwrap();
    let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let a = document.create_element("a").unwrap();
    a.set_attribute("href", &url).unwrap();
    a.set_attribute("download", filename).unwrap();

    let el: &web_sys::HtmlElement = a.unchecked_ref();
    el.click();

    web_sys::Url::revoke_object_url(&url).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── format_icon ─────────────────────────────────

    #[test]
    fn format_icon_images() {
        assert_eq!(format_icon("png"), "🖼️");
        assert_eq!(format_icon("jpg"), "🖼️");
        assert_eq!(format_icon("svg"), "🖼️");
    }

    #[test]
    fn format_icon_documents() {
        assert_eq!(format_icon("md"), "📝");
        assert_eq!(format_icon("html"), "📝");
    }

    #[test]
    fn format_icon_data() {
        assert_eq!(format_icon("csv"), "📊");
        assert_eq!(format_icon("json"), "📊");
    }

    #[test]
    fn format_icon_config() {
        assert_eq!(format_icon("yaml"), "⚙️");
        assert_eq!(format_icon("toml"), "⚙️");
    }

    #[test]
    fn format_icon_unknown() {
        assert_eq!(format_icon("xyz"), "📄");
    }

    // ── format_size ─────────────────────────────────

    #[test]
    fn format_size_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn format_size_kilobytes() {
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
    }

    #[test]
    fn format_size_megabytes() {
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(2 * 1024 * 1024 + 512 * 1024), "2.5 MB");
    }

    // ── mime_type_for ───────────────────────────────

    #[test]
    fn mime_type_images() {
        assert_eq!(mime_type_for("png"), "image/png");
        assert_eq!(mime_type_for("jpg"), "image/jpeg");
        assert_eq!(mime_type_for("gif"), "image/gif");
        assert_eq!(mime_type_for("webp"), "image/webp");
    }

    #[test]
    fn mime_type_text() {
        assert_eq!(mime_type_for("html"), "text/html");
        assert_eq!(mime_type_for("csv"), "text/csv");
        assert_eq!(mime_type_for("txt"), "text/plain");
    }

    #[test]
    fn mime_type_unknown() {
        assert_eq!(mime_type_for("xyz"), "application/octet-stream");
    }

    // ── make_output_name ────────────────────────────

    #[test]
    fn make_output_name_replaces_extension() {
        assert_eq!(make_output_name("photo.png", "jpg"), "photo.jpg");
        assert_eq!(make_output_name("doc.md", "html"), "doc.html");
    }

    #[test]
    fn make_output_name_no_extension() {
        assert_eq!(make_output_name("README", "txt"), "README.txt");
    }

    #[test]
    fn make_output_name_multiple_dots() {
        assert_eq!(make_output_name("my.file.png", "jpg"), "my.file.jpg");
    }
}
