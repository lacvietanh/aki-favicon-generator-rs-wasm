//! ICO binary encoder — multiple sizes in a single file.
//!
//! Format ICO specification:
//! https://en.wikipedia.org/wiki/ICO_(file_format)
//!
//! Structure:
//!   [ICONDIR header]
//!   [ICONDIRENTRY × N]   ← directory entries (one per size)
//!   [PNG/BMP data × N]   ← actual pixel data for each size
//!
//! Modern ICO files embed PNG data directly (supported since Vista+).
//! We use PNG-in-ICO to avoid BMP complexity and leverage Deflate compression.

use image::{imageops, RgbaImage};
use std::io::Write;

// ICO format constants (ICONDIR / ICONDIRENTRY fields)
const ICO_TYPE: u8 = 1;   // idType = 1 means ICO (2 = CUR)
const ICO_PLANES: u8 = 1; // wPlanes — must be 1
const ICO_BPP: u8 = 32;   // wBitCount — 32-bit RGBA

/// Encode a source image into an ICO binary containing multiple sizes.
///
/// # Arguments
/// * `img`   — Source RGBA image
/// * `sizes` — Sizes to embed, e.g. `&[16, 32]`
pub fn encode_multi(img: &RgbaImage, sizes: &[u32]) -> Result<Vec<u8>, String> {
    // 1. Resize each size, apply rounded corners, and encode to PNG bytes
    let png_blobs: Vec<Vec<u8>> = sizes
        .iter()
        .map(|&size| {
            let mut resized = imageops::resize(img, size, size, imageops::FilterType::Lanczos3);
            let radius = size as f32 * 0.16;
            apply_rounded_corners(&mut resized, radius);
            super::png::encode(&resized)
        })
        .collect::<Result<Vec<_>, _>>()?;

    // 2. Compute offsets: ICONDIR(6) + ICONDIRENTRY(16) × N, then data blobs
    let n = sizes.len();
    let data_start_offset = 6 + 16 * n; // byte offset where image data begins

    let mut offsets: Vec<usize> = Vec::with_capacity(n);
    let mut cursor = data_start_offset;
    for blob in &png_blobs {
        offsets.push(cursor);
        cursor += blob.len();
    }

    // 3. Build the binary buffer
    let total_size = cursor;
    let mut buf: Vec<u8> = Vec::with_capacity(total_size);

    // ICONDIR: idReserved(2) + idType(2) + idCount(2)
    buf.write_all(&[0u8, 0]).unwrap();             // Reserved, must be 0
    buf.write_all(&[ICO_TYPE, 0]).unwrap();        // Type: 1 = ICO
    buf.write_all(&(n as u16).to_le_bytes()).unwrap();

    // ICONDIRENTRY × N
    for (i, &size) in sizes.iter().enumerate() {
        let sz_byte = if size >= 256 { 0u8 } else { size as u8 }; // 0 encodes 256
        buf.write_all(&[sz_byte]).unwrap();                    // bWidth
        buf.write_all(&[sz_byte]).unwrap();                    // bHeight
        buf.write_all(&[0u8]).unwrap();                        // bColorCount (0 = no palette)
        buf.write_all(&[0u8]).unwrap();                        // bReserved
        buf.write_all(&[ICO_PLANES, 0]).unwrap();              // wPlanes = 1
        buf.write_all(&[ICO_BPP, 0]).unwrap();                 // wBitCount = 32 (RGBA)
        let data_len = png_blobs[i].len() as u32;
        buf.write_all(&data_len.to_le_bytes()).unwrap();        // dwBytesInRes
        buf.write_all(&(offsets[i] as u32).to_le_bytes()).unwrap(); // dwImageOffset
    }

    // PNG data blobs
    for blob in &png_blobs {
        buf.write_all(blob).unwrap();
    }

    Ok(buf)
}

/// Applies a gentle rounded corner mask with simple anti-aliasing to an RGBA image.
fn apply_rounded_corners(img: &mut RgbaImage, radius: f32) {
    let (width, height) = img.dimensions();
    let r = radius;
    let w = width as f32;
    let h = height as f32;

    for y in 0..height {
        for x in 0..width {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let mut dx = 0.0;
            let mut dy = 0.0;
            let mut in_corner = false;

            if px < r && py < r {
                // Top-left
                dx = r - px;
                dy = r - py;
                in_corner = true;
            } else if px > w - r && py < r {
                // Top-right
                dx = px - (w - r);
                dy = r - py;
                in_corner = true;
            } else if px < r && py > h - r {
                // Bottom-left
                dx = r - px;
                dy = py - (h - r);
                in_corner = true;
            } else if px > w - r && py > h - r {
                // Bottom-right
                dx = px - (w - r);
                dy = py - (h - r);
                in_corner = true;
            }

            if in_corner {
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > r + 0.5 {
                    let pixel = img.get_pixel_mut(x, y);
                    pixel.0[3] = 0; // Make transparent
                } else if dist > r - 0.5 {
                    let factor = 1.0 - (dist - (r - 0.5));
                    let pixel = img.get_pixel_mut(x, y);
                    pixel.0[3] = (pixel.0[3] as f32 * factor).round() as u8;
                }
            }
        }
    }
}
