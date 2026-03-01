use image::{DynamicImage, ImageFormat, ImageReader};
use std::io::Cursor;

pub fn convert_image(
    input: &[u8],
    from: &str,
    to: &str,
    quality: Option<u8>,
) -> Result<Vec<u8>, String> {
    // ICO input: decode to a DynamicImage via the ico crate
    let img: DynamicImage = if from == "ico" {
        decode_ico(input)?
    } else {
        let in_format = parse_decode_format(from)?;
        ImageReader::with_format(Cursor::new(input), in_format)
            .decode()
            .map_err(|e| format!("Failed to decode {from} image: {e}"))?
    };

    // ICO output: encode via ico crate
    if to == "ico" {
        return encode_ico(&img);
    }

    let out_format = parse_encode_format(to)?;
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

/// Decode an ICO file and return the largest image as a DynamicImage.
fn decode_ico(input: &[u8]) -> Result<DynamicImage, String> {
    let cursor = Cursor::new(input);
    let icon_dir = ico::IconDir::read(cursor).map_err(|e| format!("Failed to read ICO: {e}"))?;
    let entry = icon_dir
        .entries()
        .iter()
        .max_by_key(|e| e.width() * e.height())
        .ok_or_else(|| "ICO file contains no images".to_string())?;
    let image = entry
        .decode()
        .map_err(|e| format!("Failed to decode ICO entry: {e}"))?;
    let rgba =
        image::RgbaImage::from_raw(image.width(), image.height(), image.rgba_data().to_vec())
            .ok_or_else(|| "Failed to construct RGBA image from ICO data".to_string())?;
    Ok(DynamicImage::ImageRgba8(rgba))
}

/// Encode a DynamicImage as a single-image ICO file.
fn encode_ico(img: &DynamicImage) -> Result<Vec<u8>, String> {
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    // ico crate panics if width/height < 1 — guard here
    if width == 0 || height == 0 {
        return Err("Cannot encode zero-dimension image as ICO".into());
    }
    let ico_img = ico::IconImage::from_rgba_data(width, height, rgba.into_raw());
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
    icon_dir.add_entry(
        ico::IconDirEntry::encode(&ico_img)
            .map_err(|e| format!("Failed to encode ICO entry: {e}"))?,
    );
    let mut output = Vec::new();
    icon_dir
        .write(Cursor::new(&mut output))
        .map_err(|e| format!("Failed to write ICO: {e}"))?;
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

/// Formats supported for **decoding** (reading input files).
/// AVIF decode requires `dav1d` (C binding) — excluded.
/// ICO is handled separately via the `ico` crate.
fn parse_decode_format(fmt: &str) -> Result<ImageFormat, String> {
    match fmt {
        "png" => Ok(ImageFormat::Png),
        "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
        "gif" => Ok(ImageFormat::Gif),
        "bmp" => Ok(ImageFormat::Bmp),
        "webp" => Ok(ImageFormat::WebP),
        "tiff" | "tif" => Ok(ImageFormat::Tiff),
        "qoi" => Ok(ImageFormat::Qoi),
        "tga" => Ok(ImageFormat::Tga),
        "hdr" => Ok(ImageFormat::Hdr),
        "dds" => Ok(ImageFormat::Dds),
        "exr" => Ok(ImageFormat::OpenExr),
        _ => Err(format!("Unsupported input image format: {fmt}")),
    }
}

/// Formats supported for **encoding** (writing output files).
/// Includes AVIF (pure-Rust ravif encoder) in addition to all decode formats.
/// ICO is handled separately via the `ico` crate.
fn parse_encode_format(fmt: &str) -> Result<ImageFormat, String> {
    match fmt {
        "png" => Ok(ImageFormat::Png),
        "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
        "gif" => Ok(ImageFormat::Gif),
        "bmp" => Ok(ImageFormat::Bmp),
        "webp" => Ok(ImageFormat::WebP),
        "tiff" | "tif" => Ok(ImageFormat::Tiff),
        "avif" => Ok(ImageFormat::Avif),
        "qoi" => Ok(ImageFormat::Qoi),
        "tga" => Ok(ImageFormat::Tga),
        "hdr" => Ok(ImageFormat::Hdr),
        "dds" => Ok(ImageFormat::Dds),
        "exr" => Ok(ImageFormat::OpenExr),
        _ => Err(format!("Unsupported output image format: {fmt}")),
    }
}
