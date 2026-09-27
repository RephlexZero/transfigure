use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, RgbImage, RgbaImage};
use std::io::Cursor;

use crate::pdf;

pub fn convert_image(
    input: &[u8],
    from: &str,
    to: &str,
    quality: Option<u8>,
) -> Result<Vec<u8>, String> {
    let decoded = decode(input, from)?;
    if to == "pdf" {
        return to_pdf(input, from, &decoded);
    }
    encode(&decoded.image, to, quality)
}

struct Decoded {
    image: DynamicImage,
    /// True when EXIF orientation rotated or flipped the pixels, so the
    /// original JPEG bytes no longer match what the viewer should see.
    reoriented: bool,
}

fn decode(input: &[u8], from: &str) -> Result<Decoded, String> {
    let image = match from {
        "ico" => decode_ico(input)?,
        "svg" => rasterize_svg(input)?,
        "heic" => decode_heic(input)?,
        "dds" => crate::dds::decode(input)?,
        _ => {
            let format = parse_format(from)?;
            let mut decoder = ImageReader::with_format(Cursor::new(input), format)
                .into_decoder()
                .map_err(|e| format!("Failed to read {from} image: {e}"))?;
            // Phone photos store rotation as EXIF metadata; apply it so the
            // converted file isn't sideways.
            let orientation = decoder.orientation().ok();
            let mut img = DynamicImage::from_decoder(decoder)
                .map_err(|e| format!("Failed to decode {from} image: {e}"))?;
            let mut reoriented = false;
            if let Some(o) = orientation
                && o != image::metadata::Orientation::NoTransforms
            {
                img.apply_orientation(o);
                reoriented = true;
            }
            return Ok(Decoded {
                image: drop_opaque_alpha(img),
                reoriented,
            });
        }
    };
    Ok(Decoded {
        image: drop_opaque_alpha(image),
        reoriented: false,
    })
}

/// An alpha channel that is fully opaque carries no information; dropping it
/// keeps outputs smaller and lets JPEG sources pass into PDFs untouched.
fn drop_opaque_alpha(img: DynamicImage) -> DynamicImage {
    match &img {
        DynamicImage::ImageRgba8(buf) if buf.pixels().all(|p| p[3] == 255) => {
            DynamicImage::ImageRgb8(img.to_rgb8())
        }
        DynamicImage::ImageLumaA8(buf) if buf.pixels().all(|p| p[1] == 255) => {
            DynamicImage::ImageLuma8(img.to_luma8())
        }
        _ => img,
    }
}

fn decode_heic(input: &[u8]) -> Result<DynamicImage, String> {
    let out = heic::DecoderConfig::new()
        .decode(input, heic::PixelLayout::Rgba8)
        .map_err(|e| format!("Failed to decode HEIC image: {e}"))?;
    RgbaImage::from_raw(out.width, out.height, out.data)
        .map(DynamicImage::ImageRgba8)
        .ok_or_else(|| "HEIC decoder returned malformed pixels".to_string())
}

/// Decode an ICO file and return its largest image.
fn decode_ico(input: &[u8]) -> Result<DynamicImage, String> {
    let icon_dir =
        ico::IconDir::read(Cursor::new(input)).map_err(|e| format!("Failed to read ICO: {e}"))?;
    let entry = icon_dir
        .entries()
        .iter()
        .max_by_key(|e| e.width() * e.height())
        .ok_or_else(|| "ICO file contains no images".to_string())?;
    let image = entry
        .decode()
        .map_err(|e| format!("Failed to decode ICO entry: {e}"))?;
    RgbaImage::from_raw(image.width(), image.height(), image.rgba_data().to_vec())
        .map(DynamicImage::ImageRgba8)
        .ok_or_else(|| "Failed to construct image from ICO data".to_string())
}

/// Vector art has no pixel size of its own. Small icons render at a usable
/// size (longest side 1024 px); larger artwork keeps its declared size.
fn rasterize_svg(input: &[u8]) -> Result<DynamicImage, String> {
    let tree = resvg::usvg::Tree::from_data(input, &resvg::usvg::Options::default())
        .map_err(|e| format!("Failed to parse SVG: {e}"))?;
    let size = tree.size();
    let longest = size.width().max(size.height());
    if longest <= 0.0 {
        return Err("SVG has zero dimensions".into());
    }
    let scale = if longest < 1024.0 {
        1024.0 / longest
    } else {
        1.0
    }
    .min(16384.0 / longest);
    let w = (size.width() * scale).round().max(1.0) as u32;
    let h = (size.height() * scale).round().max(1.0) as u32;
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(w, h).ok_or_else(|| "SVG is too large".to_string())?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    // tiny-skia stores premultiplied alpha.
    let mut rgba = pixmap.take();
    for px in rgba.chunks_exact_mut(4) {
        let a = px[3] as u32;
        if a != 0 && a != 255 {
            for c in &mut px[..3] {
                *c = ((*c as u32 * 255 + a / 2) / a).min(255) as u8;
            }
        }
    }
    RgbaImage::from_raw(w, h, rgba)
        .map(DynamicImage::ImageRgba8)
        .ok_or_else(|| "Failed to rasterize SVG".to_string())
}

// ── Encoding ────────────────────────────────────────────

fn has_alpha(img: &DynamicImage) -> bool {
    img.color().has_alpha()
}

fn is_float(img: &DynamicImage) -> bool {
    matches!(
        img,
        DynamicImage::ImageRgb32F(_) | DynamicImage::ImageRgba32F(_)
    )
}

fn is_16bit(img: &DynamicImage) -> bool {
    matches!(
        img,
        DynamicImage::ImageLuma16(_)
            | DynamicImage::ImageLumaA16(_)
            | DynamicImage::ImageRgb16(_)
            | DynamicImage::ImageRgba16(_)
    )
}

fn srgb_encode(linear: f32) -> u8 {
    let v = linear.clamp(0.0, 1.0);
    let s = if v <= 0.003_130_8 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round() as u8
}

/// HDR/EXR pixels are linear light; encode them as sRGB rather than clipping
/// the raw values, which would look far too dark.
fn tonemap(img: &DynamicImage) -> DynamicImage {
    let rgba = img.to_rgba32f();
    let (w, h) = rgba.dimensions();
    let mut out = RgbaImage::new(w, h);
    for (o, i) in out.pixels_mut().zip(rgba.pixels()) {
        *o = image::Rgba([
            srgb_encode(i[0]),
            srgb_encode(i[1]),
            srgb_encode(i[2]),
            (i[3].clamp(0.0, 1.0) * 255.0).round() as u8,
        ]);
    }
    drop_opaque_alpha(DynamicImage::ImageRgba8(out))
}

/// 8-bit RGB(A) — the common denominator most encoders accept.
fn to_8bit(img: &DynamicImage) -> DynamicImage {
    if is_float(img) {
        return tonemap(img);
    }
    if has_alpha(img) {
        DynamicImage::ImageRgba8(img.to_rgba8())
    } else {
        DynamicImage::ImageRgb8(img.to_rgb8())
    }
}

/// Composite onto white for formats without transparency.
fn flatten_white(img: &DynamicImage) -> RgbImage {
    let img = to_8bit(img);
    if !has_alpha(&img) {
        return img.to_rgb8();
    }
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut out = RgbImage::new(w, h);
    for (o, p) in out.pixels_mut().zip(rgba.pixels()) {
        let a = p[3] as u32;
        let mix = |c: u8| ((c as u32 * a + 255 * (255 - a) + 127) / 255) as u8;
        *o = image::Rgb([mix(p[0]), mix(p[1]), mix(p[2])]);
    }
    out
}

fn encode(img: &DynamicImage, to: &str, quality: Option<u8>) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    let mut cursor = Cursor::new(&mut output);
    let fail = |e: image::ImageError| format!("Failed to encode {}: {e}", to.to_uppercase());

    match to {
        "jpg" | "jpeg" => {
            let q = quality.unwrap_or(85).clamp(1, 100);
            let rgb = flatten_white(img);
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, q);
            if img.color().channel_count() <= 2 && !is_float(img) {
                DynamicImage::ImageRgb8(rgb)
                    .to_luma8()
                    .write_with_encoder(encoder)
            } else {
                rgb.write_with_encoder(encoder)
            }
            .map_err(fail)?;
        }
        "avif" => {
            let q = quality.unwrap_or(80).clamp(1, 100);
            // Speed 6 trades a little compression for much faster encodes,
            // which matters when running single-threaded in the browser.
            let encoder =
                image::codecs::avif::AvifEncoder::new_with_speed_quality(&mut cursor, 6, q);
            to_8bit(img).write_with_encoder(encoder).map_err(fail)?;
        }
        "ico" => return encode_ico(img),
        "png" | "tiff" | "tif" => {
            // Both keep 16 bits per channel.
            let format = if to == "png" {
                ImageFormat::Png
            } else {
                ImageFormat::Tiff
            };
            let out = if is_float(img) {
                tonemap(img)
            } else if is_16bit(img) {
                if has_alpha(img) {
                    DynamicImage::ImageRgba16(img.to_rgba16())
                } else {
                    DynamicImage::ImageRgb16(img.to_rgb16())
                }
            } else if img.color().channel_count() <= 2 && !has_alpha(img) {
                DynamicImage::ImageLuma8(img.to_luma8())
            } else {
                to_8bit(img)
            };
            out.write_to(&mut cursor, format).map_err(fail)?;
        }
        "gif" => {
            gif_ready(img)
                .write_to(&mut cursor, ImageFormat::Gif)
                .map_err(fail)?;
        }
        "tga" => return Ok(encode_tga(&to_8bit(img))),
        "bmp" | "webp" | "qoi" => {
            let format = parse_format(to)?;
            to_8bit(img).write_to(&mut cursor, format).map_err(fail)?;
        }
        _ => return Err(format!("Unsupported output image format: {to}")),
    }
    Ok(output)
}

/// Run-length encoded TGA with packets confined to one scanline, as the TGA
/// 2.0 spec requires; some readers reject packets that wrap across rows.
fn encode_tga(img: &DynamicImage) -> Vec<u8> {
    let alpha = has_alpha(img);
    let bpp = if alpha { 4 } else { 3 };
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut out = vec![0u8; 18];
    out[2] = 10; // RLE true-colour
    out[12..14].copy_from_slice(&(w as u16).to_le_bytes());
    out[14..16].copy_from_slice(&(h as u16).to_le_bytes());
    out[16] = (bpp * 8) as u8;
    // Top-left origin, plus the alpha bit count.
    out[17] = 0x20 | if alpha { 8 } else { 0 };
    let px = |x: u32, y: u32| {
        let p = rgba.get_pixel(x, y);
        [p[2], p[1], p[0], p[3]]
    };
    for y in 0..h {
        let mut x = 0;
        while x < w {
            let first = px(x, y);
            let mut run = 1;
            while x + run < w && run < 128 && px(x + run, y) == first {
                run += 1;
            }
            if run > 1 {
                out.push(0x80 | (run - 1) as u8);
                out.extend_from_slice(&first[..bpp]);
                x += run;
                continue;
            }
            // A raw packet runs until the next repeat (or 128 pixels).
            let start = x;
            let mut n = 0;
            while x < w && n < 128 && !(x + 1 < w && px(x, y) == px(x + 1, y)) {
                x += 1;
                n += 1;
            }
            if n == 0 {
                x += 1;
                n = 1;
            }
            out.push((n - 1) as u8);
            for i in start..start + n {
                out.extend_from_slice(&px(i, y)[..bpp]);
            }
        }
    }
    out
}

/// GIF transparency is on/off. Semi-transparent pixels are composited onto
/// white (as they'd appear on a typical page) and only near-invisible ones
/// stay transparent; thresholding alone would leave hard, dark fringes.
fn gif_ready(img: &DynamicImage) -> DynamicImage {
    let img = to_8bit(img);
    if !has_alpha(&img) {
        return img;
    }
    let mut rgba = img.to_rgba8();
    for p in rgba.pixels_mut() {
        let a = p[3] as u32;
        if a < 16 {
            *p = image::Rgba([0, 0, 0, 0]);
        } else if a < 255 {
            let mix = |c: u8| ((c as u32 * a + 255 * (255 - a) + 127) / 255) as u8;
            *p = image::Rgba([mix(p[0]), mix(p[1]), mix(p[2]), 255]);
        }
    }
    DynamicImage::ImageRgba8(rgba)
}

/// A multi-resolution icon: the standard sizes up to the source size, each
/// centred on a transparent square.
fn encode_ico(img: &DynamicImage) -> Result<Vec<u8>, String> {
    let rgba = to_8bit(img).to_rgba8();
    let (w, h) = rgba.dimensions();
    if w == 0 || h == 0 {
        return Err("Cannot encode an empty image as ICO".into());
    }
    let side = w.max(h);
    let mut square = RgbaImage::new(side, side);
    image::imageops::overlay(
        &mut square,
        &rgba,
        ((side - w) / 2) as i64,
        ((side - h) / 2) as i64,
    );

    let mut sizes: Vec<u32> = [16, 24, 32, 48, 64, 128, 256]
        .into_iter()
        .filter(|s| *s <= side)
        .collect();
    if sizes.is_empty() {
        sizes.push(side);
    }
    let mut dir = ico::IconDir::new(ico::ResourceType::Icon);
    for s in sizes {
        let scaled = if s == side {
            square.clone()
        } else {
            image::imageops::resize(&square, s, s, image::imageops::FilterType::Lanczos3)
        };
        let icon = ico::IconImage::from_rgba_data(s, s, scaled.into_raw());
        dir.add_entry(
            ico::IconDirEntry::encode(&icon).map_err(|e| format!("Failed to encode ICO: {e}"))?,
        );
    }
    let mut output = Vec::new();
    dir.write(Cursor::new(&mut output))
        .map_err(|e| format!("Failed to write ICO: {e}"))?;
    Ok(output)
}

fn to_pdf(input: &[u8], from: &str, decoded: &Decoded) -> Result<Vec<u8>, String> {
    let img = &decoded.image;
    let (w, h) = (img.width(), img.height());
    // Baseline JPEGs embed as-is: no generation loss and a smaller file.
    let expected = match img {
        DynamicImage::ImageLuma8(_) => Some(1),
        DynamicImage::ImageRgb8(_) => Some(3),
        _ => None,
    };
    if matches!(from, "jpg" | "jpeg")
        && !decoded.reoriented
        && let Some(components) = expected
        && jpeg_components(input) == Some(components)
    {
        return Ok(pdf::render_image(
            w,
            h,
            pdf::ImageData::Jpeg {
                data: input.to_vec(),
                components,
            },
        ));
    }
    let img8 = to_8bit(img);
    let alpha = has_alpha(&img8).then(|| img8.to_rgba8().pixels().map(|p| p[3]).collect());
    Ok(pdf::render_image(
        w,
        h,
        pdf::ImageData::Rgb {
            rgb: img8.to_rgb8().into_raw(),
            alpha,
        },
    ))
}

/// Component count from a JPEG's start-of-frame header (1 grey, 3 colour,
/// 4 CMYK), which decides whether the bytes can be embedded directly.
fn jpeg_components(data: &[u8]) -> Option<u8> {
    let mut i = 2;
    while i + 9 < data.len() {
        if data[i] != 0xff {
            return None;
        }
        let marker = data[i + 1];
        let len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
        if matches!(marker, 0xc0..=0xc3 | 0xc5..=0xc7 | 0xc9..=0xcb | 0xcd..=0xcf) {
            return data.get(i + 9).copied();
        }
        i += 2 + len;
    }
    None
}

fn parse_format(fmt: &str) -> Result<ImageFormat, String> {
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
        _ => Err(format!("Unsupported image format: {fmt}")),
    }
}
