//! Color detection — auto-detect background_color and theme_color from pixels.

use image::RgbaImage;

/// Returns (theme_color_hex, background_color_hex).
pub fn detect(img: &RgbaImage) -> (String, String) {
    let (w, h) = img.dimensions();

    // ── Background: sample the 4 corners ────────────────────────────────────
    let corners = [
        img.get_pixel(0, 0),
        img.get_pixel(w - 1, 0),
        img.get_pixel(0, h - 1),
        img.get_pixel(w - 1, h - 1),
    ];

    // All 4 corners must be opaque and similar in color
    let all_opaque = corners.iter().all(|p| p[3] >= 250);
    let bg_rgb = if all_opaque {
        let base = corners[0];
        let similar = corners[1..].iter().all(|c| {
            let diff: i32 = (0..3)
                .map(|i| (base[i] as i32 - c[i] as i32).pow(2))
                .sum();
            diff < 1000
        });
        if similar {
            Some((base[0], base[1], base[2]))
        } else {
            None
        }
    } else {
        None
    };

    let background_color = match bg_rgb {
        Some((r, g, b)) => rgb_to_hex(r, g, b),
        None => "#000000".to_string(),
    };

    // ── Theme: dominant vibrant color (excluding background) ─────────────────
    // Sample a downscaled 50×50 grid for speed
    let small_w = 50u32;
    let small_h = 50u32;
    let step_x = (w / small_w).max(1);
    let step_y = (h / small_h).max(1);

    let mut best_score: f32 = 0.0;
    let mut best_rgb = (255u8, 255u8, 255u8);

    for y in (0..h).step_by(step_y as usize) {
        for x in (0..w).step_by(step_x as usize) {
            let p = img.get_pixel(x, y);
            let (r, g, b, a) = (p[0], p[1], p[2], p[3]);

            if a < 200 { continue; }

            // Skip pixels too close to the background color
            if let Some((r_bg, g_bg, b_bg)) = bg_rgb {
                let diff: i32 = [
                    (r as i32 - r_bg as i32).pow(2),
                    (g as i32 - g_bg as i32).pow(2),
                    (b as i32 - b_bg as i32).pow(2),
                ]
                .iter()
                .sum();
                if diff < 1000 { continue; }
            }

            // Convert to HSV to filter out near-black and near-white pixels
            let (_, s, v) = rgb_to_hsv(r, g, b);
            if v < 0.2 { continue; }              // too dark
            if v > 0.95 && s < 0.1 { continue; } // too white / achromatic

            let score = s * 3.0 + v;
            if score > best_score {
                best_score = score;
                best_rgb = (r, g, b);
            }
        }
    }

    let theme_color = if best_score > 0.0 {
        rgb_to_hex(best_rgb.0, best_rgb.1, best_rgb.2)
    } else {
        "#ffffff".to_string()
    };

    (theme_color, background_color)
}

pub fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

pub fn hex_to_rgb(hex: &str) -> Result<(u8, u8, u8), String> {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 {
        return Err(format!("Invalid hex color: {hex}"));
    }
    let r = u8::from_str_radix(&h[0..2], 16).map_err(|_| format!("Bad hex: {hex}"))?;
    let g = u8::from_str_radix(&h[2..4], 16).map_err(|_| format!("Bad hex: {hex}"))?;
    let b = u8::from_str_radix(&h[4..6], 16).map_err(|_| format!("Bad hex: {hex}"))?;
    Ok((r, g, b))
}

fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;

    let cmax = rf.max(gf).max(bf);
    let cmin = rf.min(gf).min(bf);
    let delta = cmax - cmin;

    let v = cmax;
    let s = if cmax == 0.0 { 0.0 } else { delta / cmax };
    // Hue is unused by callers (only s and v are checked), but kept for API completeness.
    // Use rem_euclid to guarantee [0, 360) even when the raw quotient is negative.
    let h = if delta == 0.0 {
        0.0
    } else if cmax == rf {
        60.0 * (((gf - bf) / delta).rem_euclid(6.0))
    } else if cmax == gf {
        60.0 * (((bf - rf) / delta) + 2.0)
    } else {
        60.0 * (((rf - gf) / delta) + 4.0)
    };

    (h, s, v)
}
