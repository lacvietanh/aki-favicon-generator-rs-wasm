//! aki-favicon-generator — WASM public API
//!
//! Single entry point exported to JS.
//! All heavy lifting lives in the sub-modules.

mod color;
mod encoder;
mod transform;

use wasm_bindgen::prelude::*;

/// Options passed from JS to WASM.
/// Uses a plain struct to avoid pulling serde_json into the WASM bundle.
#[wasm_bindgen]
pub struct FaviconOptions {
    /// Hex string, e.g. "#1a73e8". None = auto-detect from the image.
    #[wasm_bindgen(getter_with_clone)]
    pub theme_color: Option<String>,
    /// Hex string. None = auto-detect from the image.
    #[wasm_bindgen(getter_with_clone)]
    pub background_color: Option<String>,
    /// Safe-zone ratio for the maskable icon (default: 0.80).
    pub safe_zone: f32,
    /// Whether to generate a diagonal gradient background for solid icons.
    pub fill_gradient: bool,
    /// Hex string for the gradient highlight color. None = auto-detect.
    #[wasm_bindgen(getter_with_clone)]
    pub gradient_color: Option<String>,
}

#[wasm_bindgen]
impl FaviconOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> FaviconOptions {
        FaviconOptions {
            theme_color: None,
            background_color: None,
            safe_zone: 0.80,
            fill_gradient: true,
            gradient_color: None,
        }
    }

    pub fn with_theme_color(mut self, color: String) -> Self {
        self.theme_color = Some(color);
        self
    }

    pub fn with_background_color(mut self, color: String) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn with_safe_zone(mut self, ratio: f32) -> Self {
        self.safe_zone = ratio.clamp(0.5, 1.0);
        self
    }

    pub fn with_fill_gradient(mut self, gradient: bool) -> Self {
        self.fill_gradient = gradient;
        self
    }

    pub fn with_gradient_color(mut self, color: String) -> Self {
        self.gradient_color = Some(color);
        self
    }
}

/// Output returned to JS — each field is the raw bytes of the corresponding file.
/// The JS caller zips them with fflate and offers a download.
#[wasm_bindgen]
pub struct FaviconSet {
    // favicon.ico — ICO binary containing 16×16 and 32×32
    #[wasm_bindgen(getter_with_clone)]
    pub favicon_ico: Vec<u8>,

    // icon-192.png — original transparency preserved, purpose: any
    #[wasm_bindgen(getter_with_clone)]
    pub icon_192: Vec<u8>,

    // icon-48.png — original transparency preserved, purpose: any (SEO/Discovery)
    #[wasm_bindgen(getter_with_clone)]
    pub icon_48: Vec<u8>,

    // icon-96.png — original transparency preserved, purpose: any (SEO/Discovery)
    #[wasm_bindgen(getter_with_clone)]
    pub icon_96: Vec<u8>,

    // icon-512-maskable.png — solid bg + logo inside 80% safe zone, purpose: maskable
    #[wasm_bindgen(getter_with_clone)]
    pub icon_512_maskable: Vec<u8>,

    // apple-touch-icon.png — 180×180, solid bg, no transparency
    #[wasm_bindgen(getter_with_clone)]
    pub apple_touch_icon: Vec<u8>,

    // Auto-detected colors (or overridden via options) for the JS manifest builder
    #[wasm_bindgen(getter_with_clone)]
    pub theme_color: String,
    #[wasm_bindgen(getter_with_clone)]
    pub background_color: String,
}

/// Main entry point — callable from JS on the Main Thread or inside a Worker.
///
/// # Arguments
/// * `image_bytes` — Raw bytes of the input PNG/JPEG file (any size).
/// * `options`     — FaviconOptions from the UI form.
///
/// # Returns
/// `FaviconSet` containing raw bytes for all output files.
/// The JS caller zips them with fflate and offers a download.
#[wasm_bindgen]
pub fn generate_favicon_set(
    image_bytes: &[u8],
    options: &FaviconOptions,
) -> Result<FaviconSet, JsValue> {
    // 1. Decode source image (PNG or JPEG)
    let img = image::load_from_memory(image_bytes)
        .map_err(|e| JsValue::from_str(&format!("Image decode error: {e}")))?
        .into_rgba8();

    // 2. Color detection
    let (auto_theme, auto_bg) = color::detect(&img);
    let theme_color = options.theme_color.clone().unwrap_or(auto_theme);
    let background_color = options.background_color.clone().unwrap_or(auto_bg);

    let bg_rgb = color::hex_to_rgb(&background_color)
        .map_err(|e| JsValue::from_str(&e))?;

    // For gradient highlight, default to pure white if not specified or invalid.
    // White represents a strong light source.
    let grad_rgb = if let Some(gc) = &options.gradient_color {
        color::hex_to_rgb(gc).unwrap_or((255, 255, 255))
    } else {
        (255, 255, 255)
    };

    // 3. Generate icons
    //
    // favicon.ico: 16×16 + 32×32 embedded
    let favicon_ico = encoder::ico::encode_multi(&img, &[16, 32])
        .map_err(|e| JsValue::from_str(&e))?;

    // icon-192.png: resize only, keep transparency
    let icon_192 = {
        let resized = transform::resize_exact(&img, 192);
        encoder::png::encode(&resized)
            .map_err(|e| JsValue::from_str(&e))?
    };

    // icon-48.png: resize only, keep transparency
    let icon_48 = {
        let resized = transform::resize_exact(&img, 48);
        encoder::png::encode(&resized)
            .map_err(|e| JsValue::from_str(&e))?
    };

    // icon-96.png: resize only, keep transparency
    let icon_96 = {
        let resized = transform::resize_exact(&img, 96);
        encoder::png::encode(&resized)
            .map_err(|e| JsValue::from_str(&e))?
    };

    // icon-512-maskable.png: solid bg + logo in safe zone
    let icon_512_maskable = {
        let fitted = transform::fit_safe_zone(&img, 512, options.safe_zone, bg_rgb, options.fill_gradient, grad_rgb);
        encoder::png::encode(&fitted)
            .map_err(|e| JsValue::from_str(&e))?
    };

    // apple-touch-icon.png: 180×180, solid bg, safe zone 80% (no transparency — avoids iOS black-fill bug;
    // iOS rounds corners itself so keeping an 80% zone prevents logo edges from being clipped)
    let apple_touch_icon = {
        let fitted = transform::fit_safe_zone(&img, 180, 0.80, bg_rgb, options.fill_gradient, grad_rgb);
        encoder::png::encode(&fitted)
            .map_err(|e| JsValue::from_str(&e))?
    };

    Ok(FaviconSet {
        favicon_ico,
        icon_192,
        icon_48,
        icon_96,
        icon_512_maskable,
        apple_touch_icon,
        theme_color,
        background_color,
    })
}
