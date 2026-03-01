use image::{DynamicImage, ImageFormat, ImageReader};
use std::io::Cursor;

pub fn convert_image(
    input: &[u8],
    from: &str,
    to: &str,
    quality: Option<u8>,
) -> Result<Vec<u8>, String> {
    let in_format = parse_format(from)?;
    let out_format = parse_format(to)?;

    let img: DynamicImage = ImageReader::with_format(Cursor::new(input), in_format)
        .decode()
        .map_err(|e| format!("Failed to decode {from} image: {e}"))?;

    let mut output = Vec::new();
    let mut cursor = Cursor::new(&mut output);

    match out_format {
        ImageFormat::Jpeg => {
            let q = quality.unwrap_or(85);
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, q);
            img.write_with_encoder(encoder)
                .map_err(|e| format!("Failed to encode JPEG: {e}"))?;
        }
        _ => {
            img.write_to(&mut cursor, out_format)
                .map_err(|e| format!("Failed to encode {to}: {e}"))?;
        }
    }

    Ok(output)
}

pub fn svg_to_png(input: &[u8]) -> Result<Vec<u8>, String> {
    let svg_str = std::str::from_utf8(input).map_err(|e| format!("Invalid UTF-8 in SVG: {e}"))?;

    let tree = resvg::usvg::Tree::from_str(svg_str, &resvg::usvg::Options::default())
        .map_err(|e| format!("Failed to parse SVG: {e}"))?;

    let size = tree.size();
    let width = size.width().ceil() as u32;
    let height = size.height().ceil() as u32;

    if width == 0 || height == 0 {
        return Err("SVG has zero dimensions".into());
    }

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| "Failed to create pixmap".to_string())?;

    resvg::render(
        &tree,
        resvg::usvg::Transform::default(),
        &mut pixmap.as_mut(),
    );

    pixmap
        .encode_png()
        .map_err(|e| format!("Failed to encode PNG: {e}"))
}

fn parse_format(fmt: &str) -> Result<ImageFormat, String> {
    match fmt {
        "png" => Ok(ImageFormat::Png),
        "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
        "gif" => Ok(ImageFormat::Gif),
        "bmp" => Ok(ImageFormat::Bmp),
        "webp" => Ok(ImageFormat::WebP),
        "tiff" | "tif" => Ok(ImageFormat::Tiff),
        _ => Err(format!("Unsupported image format: {fmt}")),
    }
}
