//! DirectDraw Surface (game texture) decoding: block-compressed BC1–BC7 via
//! bcdec_rs, and uncompressed layouts described by bit masks. Only the first
//! surface (top mip level) is read.

use ddsfile::{Dds, DxgiFormat, FourCC};
use image::{DynamicImage, Rgb32FImage, RgbaImage};

#[derive(Clone, Copy)]
enum Bc {
    Bc1,
    Bc2,
    Bc3,
    Bc4(bool),
    Bc5(bool),
    Bc6(bool),
    Bc7,
}

impl Bc {
    fn block_bytes(self) -> usize {
        match self {
            Bc::Bc1 | Bc::Bc4(_) => 8,
            _ => 16,
        }
    }
}

fn block_format(dds: &Dds) -> Option<Bc> {
    if let Some(f) = dds.get_dxgi_format() {
        use DxgiFormat as D;
        return Some(match f {
            D::BC1_Typeless | D::BC1_UNorm | D::BC1_UNorm_sRGB => Bc::Bc1,
            D::BC2_Typeless | D::BC2_UNorm | D::BC2_UNorm_sRGB => Bc::Bc2,
            D::BC3_Typeless | D::BC3_UNorm | D::BC3_UNorm_sRGB => Bc::Bc3,
            D::BC4_Typeless | D::BC4_UNorm => Bc::Bc4(false),
            D::BC4_SNorm => Bc::Bc4(true),
            D::BC5_Typeless | D::BC5_UNorm => Bc::Bc5(false),
            D::BC5_SNorm => Bc::Bc5(true),
            D::BC6H_Typeless | D::BC6H_UF16 => Bc::Bc6(false),
            D::BC6H_SF16 => Bc::Bc6(true),
            D::BC7_Typeless | D::BC7_UNorm | D::BC7_UNorm_sRGB => Bc::Bc7,
            _ => return None,
        });
    }
    let code = dds.header.spf.fourcc.as_ref()?.0;
    Some(match code {
        FourCC::DXT1 => Bc::Bc1,
        FourCC::DXT2 | FourCC::DXT3 => Bc::Bc2,
        FourCC::DXT4 | FourCC::DXT5 => Bc::Bc3,
        FourCC::ATI1 | FourCC::BC4_UNORM => Bc::Bc4(false),
        FourCC::BC4_SNORM => Bc::Bc4(true),
        FourCC::ATI2 => Bc::Bc5(false),
        FourCC::BC5_SNORM => Bc::Bc5(true),
        _ => return None,
    })
}

pub fn decode(input: &[u8]) -> Result<DynamicImage, String> {
    let dds =
        Dds::read(std::io::Cursor::new(input)).map_err(|e| format!("Failed to read DDS: {e}"))?;
    let (w, h) = (dds.get_width() as usize, dds.get_height() as usize);
    if w == 0 || h == 0 {
        return Err("DDS has zero dimensions".into());
    }
    if let Some(bc) = block_format(&dds) {
        return decode_blocks(&dds.data, w, h, bc);
    }
    if let Some(f) = dds.get_dxgi_format() {
        let swap = match f {
            DxgiFormat::R8G8B8A8_UNorm | DxgiFormat::R8G8B8A8_UNorm_sRGB => false,
            DxgiFormat::B8G8R8A8_UNorm
            | DxgiFormat::B8G8R8A8_UNorm_sRGB
            | DxgiFormat::B8G8R8X8_UNorm => true,
            other => return Err(format!("Unsupported DDS format {other:?}")),
        };
        let mut rgba = dds
            .data
            .get(..w * h * 4)
            .ok_or("DDS data is truncated")?
            .to_vec();
        if swap {
            for px in rgba.as_chunks_mut::<4>().0 {
                px.swap(0, 2);
            }
        }
        if f == DxgiFormat::B8G8R8X8_UNorm {
            for p in rgba.as_chunks_mut::<4>().0 {
                p[3] = 255;
            }
        }
        return RgbaImage::from_raw(w as u32, h as u32, rgba)
            .map(DynamicImage::ImageRgba8)
            .ok_or_else(|| "DDS data is truncated".into());
    }
    decode_masked(&dds, w, h)
}

fn decode_blocks(data: &[u8], w: usize, h: usize, bc: Bc) -> Result<DynamicImage, String> {
    let (bw, bh) = (w.div_ceil(4), h.div_ceil(4));
    let needed = bw * bh * bc.block_bytes();
    let data = data.get(..needed).ok_or("DDS data is truncated")?;
    let blocks = data.chunks_exact(bc.block_bytes());

    if let Bc::Bc6(signed) = bc {
        let mut img = Rgb32FImage::new(w as u32, h as u32);
        let mut block = [0f32; 4 * 4 * 3];
        for (i, src) in blocks.enumerate() {
            bcdec_rs::bc6h_float(src, &mut block, 4 * 3, signed);
            let (bx, by) = (i % bw * 4, i / bw * 4);
            for y in 0..4 {
                for x in 0..4 {
                    if bx + x < w && by + y < h {
                        let o = (y * 4 + x) * 3;
                        img.put_pixel(
                            (bx + x) as u32,
                            (by + y) as u32,
                            image::Rgb([block[o], block[o + 1], block[o + 2]]),
                        );
                    }
                }
            }
        }
        return Ok(DynamicImage::ImageRgb32F(img));
    }

    let mut img = RgbaImage::new(w as u32, h as u32);
    let mut block = [0u8; 4 * 4 * 4];
    for (i, src) in blocks.enumerate() {
        match bc {
            Bc::Bc1 => bcdec_rs::bc1(src, &mut block, 16),
            Bc::Bc2 => bcdec_rs::bc2(src, &mut block, 16),
            Bc::Bc3 => bcdec_rs::bc3(src, &mut block, 16),
            Bc::Bc7 => bcdec_rs::bc7(src, &mut block, 16),
            Bc::Bc4(signed) => {
                let mut r = [0u8; 16];
                bcdec_rs::bc4(src, &mut r, 4, signed);
                for (p, v) in block.as_chunks_mut::<4>().0.iter_mut().zip(r) {
                    *p = [v, v, v, 255];
                }
            }
            Bc::Bc5(signed) => {
                // Usually a normal map: reconstruct Z so it previews sensibly.
                let mut rg = [0u8; 32];
                bcdec_rs::bc5(src, &mut rg, 8, signed);
                for (p, v) in block
                    .as_chunks_mut::<4>()
                    .0
                    .iter_mut()
                    .zip(rg.as_chunks::<2>().0)
                {
                    let (x, y) = (v[0] as f32 / 127.5 - 1.0, v[1] as f32 / 127.5 - 1.0);
                    let z = (1.0 - x * x - y * y).max(0.0).sqrt();
                    *p = [v[0], v[1], ((z + 1.0) * 127.5) as u8, 255];
                }
            }
            Bc::Bc6(_) => unreachable!("handled above"),
        }
        let (bx, by) = (i % bw * 4, i / bw * 4);
        for y in 0..4 {
            for x in 0..4 {
                if bx + x < w && by + y < h {
                    let o = (y * 4 + x) * 4;
                    img.put_pixel(
                        (bx + x) as u32,
                        (by + y) as u32,
                        image::Rgba([block[o], block[o + 1], block[o + 2], block[o + 3]]),
                    );
                }
            }
        }
    }
    Ok(DynamicImage::ImageRgba8(img))
}

/// Uncompressed legacy layouts (A8R8G8B8, R5G6B5, L8, …) from their bit masks.
fn decode_masked(dds: &Dds, w: usize, h: usize) -> Result<DynamicImage, String> {
    let pf = &dds.header.spf;
    let bits = pf.rgb_bit_count.ok_or("Unsupported DDS pixel format")? as usize;
    if !matches!(bits, 8 | 16 | 24 | 32) {
        return Err(format!("Unsupported DDS bit depth {bits}"));
    }
    let bpp = bits / 8;
    let data = dds.data.get(..w * h * bpp).ok_or("DDS data is truncated")?;
    let channel = |mask: Option<u32>, px: u32, default: u8| -> u8 {
        match mask {
            Some(m) if m != 0 => {
                let shift = m.trailing_zeros();
                let max = m >> shift;
                (((px & m) >> shift) * 255 / max) as u8
            }
            _ => default,
        }
    };
    let mut img = RgbaImage::new(w as u32, h as u32);
    for (i, px) in data.chunks_exact(bpp).enumerate() {
        let mut v = 0u32;
        for (k, b) in px.iter().enumerate() {
            v |= (*b as u32) << (8 * k);
        }
        let r = channel(pf.r_bit_mask, v, 0);
        let luminance = pf.g_bit_mask.unwrap_or(0) == 0 && pf.b_bit_mask.unwrap_or(0) == 0;
        let (g, b) = if luminance {
            (r, r)
        } else {
            (channel(pf.g_bit_mask, v, 0), channel(pf.b_bit_mask, v, 0))
        };
        let a = channel(pf.a_bit_mask, v, 255);
        img.put_pixel((i % w) as u32, (i / w) as u32, image::Rgba([r, g, b, a]));
    }
    Ok(DynamicImage::ImageRgba8(img))
}
