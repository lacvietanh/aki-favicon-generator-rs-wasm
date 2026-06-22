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
    gradient: bool,
    grad_rgb: (u8, u8, u8),
) -> RgbaImage {
    let content_size = (canvas_size as f32 * safe_zone) as u32;
    let offset = (canvas_size - content_size) / 2;

    // Create a solid-color canvas
    let mut canvas = RgbaImage::new(canvas_size, canvas_size);
    
    let cx = canvas_size as f32 / 2.0;
    let cy = canvas_size as f32 / 2.0;
    
    // Simulate a convex surface hit by a light source from top-left (approx 35 degrees)
    // The highlight point is offset top-left.
    let hx = cx - canvas_size as f32 * 0.25;
    let hy = cy - canvas_size as f32 * 0.25;
    
    // Extremely wide gradient
    let max_dist = canvas_size as f32 * 1.5;

    for y in 0..canvas_size {
        for x in 0..canvas_size {
            if gradient {
                let dx = x as f32 - hx;
                let dy = y as f32 - hy;
                let dist = (dx * dx + dy * dy).sqrt();
                
                // Extremely strong and wide easing
                let t = (dist / max_dist).clamp(0.0, 1.0);
                
                // Non-linear easing to make the highlight pop and degrade smoothly over a wide area
                let brightness = (1.0 - t).powf(1.8);
                
                let r = (grad_rgb.0 as f32 * brightness + bg_rgb.0 as f32 * (1.0 - brightness)) as u8;
                let g = (grad_rgb.1 as f32 * brightness + bg_rgb.1 as f32 * (1.0 - brightness)) as u8;
                let b = (grad_rgb.2 as f32 * brightness + bg_rgb.2 as f32 * (1.0 - brightness)) as u8;
                
                canvas.put_pixel(x, y, Rgba([r, g, b, 255]));
            } else {
                canvas.put_pixel(x, y, Rgba([bg_rgb.0, bg_rgb.1, bg_rgb.2, 255]));
            }
        }
    }

    // Resize logo to fit the content area
    let resized = imageops::resize(img, content_size, content_size, imageops::FilterType::Lanczos3);

    // Overlay logo onto the canvas (alpha compositing)
    imageops::overlay(&mut canvas, &resized, offset as i64, offset as i64);

    canvas
}
