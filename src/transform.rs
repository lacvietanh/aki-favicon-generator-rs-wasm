//! Image transformations: resize, safe-zone fitting, solid background flatten.

use image::{imageops, RgbaImage, Rgba};

/// Resize to exactly `size × size` using Lanczos3 — transparency preserved.
pub fn resize_exact(img: &RgbaImage, size: u32) -> RgbaImage {
    imageops::resize(img, size, size, imageops::FilterType::Lanczos3)
}

/// Produce an icon with a solid background and the logo scaled into the safe zone.
///
/// Used for:
/// - icon-512-maskable.png (safe_zone = 0.80)
/// - apple-touch-icon.png  (safe_zone = 0.80 — iOS clips corners, so an 80% zone
///                          prevents logo edges from being cut)
///
/// # Arguments
/// * `img`        — Source RGBA image
/// * `canvas_size`— Output dimension in pixels (e.g. 512 or 180)
/// * `safe_zone`  — Content-to-canvas ratio (0.5–1.0)
/// * `bg_rgb`     — Solid background color
pub fn fit_safe_zone(
    img: &RgbaImage,
    canvas_size: u32,
    safe_zone: f32,
    bg_rgb: (u8, u8, u8),
) -> RgbaImage {
    let content_size = (canvas_size as f32 * safe_zone) as u32;
    let offset = (canvas_size - content_size) / 2;

    // Create a solid-color canvas
    let bg_pixel = Rgba([bg_rgb.0, bg_rgb.1, bg_rgb.2, 255]);
    let mut canvas = RgbaImage::from_pixel(canvas_size, canvas_size, bg_pixel);

    // Resize logo to fit the content area
    let resized = imageops::resize(img, content_size, content_size, imageops::FilterType::Lanczos3);

    // Overlay logo onto the canvas (alpha compositing)
    imageops::overlay(&mut canvas, &resized, offset as i64, offset as i64);

    canvas
}
