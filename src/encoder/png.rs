//! PNG encoder helper.

use image::RgbaImage;
use std::io::Cursor;

/// Encode an RgbaImage to PNG bytes in memory (no file I/O).
pub fn encode(img: &RgbaImage) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    let cursor = Cursor::new(&mut buf);
    img.write_to(&mut std::io::BufWriter::new(cursor), image::ImageFormat::Png)
        .map_err(|e| format!("PNG encode error: {e}"))?;
    Ok(buf)
}
