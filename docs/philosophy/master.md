# Favicon Generator Philosophy

## 1. Core Philosophy

The Favicon Generator is designed under a single-source concept: it does not create multiple distinct logos. Instead, it takes **one single image input** and exports multiple target-specific artifacts matching platform contracts.

Key tenets:
- **1 Universal artwork input**
- **1 Processing pipeline**
- **Multiple file exports** mapped to roles
- **No outer-shape assumptions** (let the OS control the crop)
- **Zero over-engineering** (no SVG auto-tracing, no monochrome badges, no dark-mode switching, no Windows tile legacy, and no Open Graph images which belong to social media sharing)

Our goal is to produce the absolute minimum set of icons needed for production PWA deployment:
```
favicon.ico
icon-192.png
icon-512-maskable.png
apple-touch-icon.png
manifest.json
```
Exactly **4 image files + 1 manifest**. No more, no less.

---

## 2. Universal Design Principle

Modern operating systems do not allow apps to decide their final outer shape.
- **Android launcher** masks icons into circles, squircles, rounded squares, teardrops, etc.
- **iOS** rounds corners automatically when placing Web Clips on the Home Screen.

Therefore, the generator must **never** pre-clip outer shapes (e.g. circle, squircle, rounded-rect) or output transparent margins for PWA/app icons.

The correct approach:
- **Square canvas**
- **Opaque background**
- **Brand symbol centered in the 76-80% safe zone**
- **Let the OS mask according to its platform styling**

An app icon should not be a "framed picture". It must be:
`Brand symbol + secure background area ready for OS masking.`

---

## 3. The Shape Fallacy

Common mistakes in icon generators:
- Pre-clipping into a circle
- Pre-clipping into a squircle/rounded rect
- Leaving transparent edges around a shaped icon
- Embedding iOS-style squircle frames

Problems caused by pre-clipping:
- **Double-clipping** on Android launchers (e.g. circles inside a circle launcher, leading to smaller symbols and ugly padding)
- **White borders/artifacts** on launchers applying their own mask over transparent margins
- **Poor rendering** on non-circular launchers
- **Loss of universal compatibility**

**Rule:** Do not pre-clip outer shapes for app icons. Do not make choices on behalf of the OS. Design the symbol and the background, nothing else.

---

## 4. Background Philosophy

While we refer to a "solid background," it is more accurately defined as an **opaque background**.

An opaque background can be:
- A solid color
- A subtle linear/radial gradient
- A brand atmosphere gradient

Requirements:
- **Never transparent** for PWA app icons (except `favicon.ico` which is a separate pipeline)
- No empty corners
- No excessive details that distract from the main symbol
- Clean and intentional edge pixels

---

## 5. Gradient Background Rule

A subtle gradient is highly recommended. 

An app icon is an object living within the visual ecosystem of the OS. Modern native icons typically have depth, lighting, gradients, or a subtle atmosphere. A completely flat icon feels like a basic web shortcut rather than a native app.

A gradient helps:
- Provide depth and native app feel
- Prevent the "logo pasted on color" look
- Establish brand atmosphere without adding distracting elements
- Make the icon stand out on the Home Screen

Keep gradients subtle:
- **Do:** Use subtle linear gradients, subtle radial glow, or 2-3 closely related colors.
- **Don't:** Use aggressive RGB transitions, complex patterns, textures, or high-contrast noise. The symbol must remain the focal point.

---

## 6. iOS Edge Fill Philosophy

iOS does not read PWA manifests for Web Clip icon properties. With `apple-touch-icon.png`, the OS expects a full-bleed opaque image. It automatically rounds the corners.

If the edge pixels or corners contain transparency or stray pixels:
- iOS may fill them with black, causing ugly black corners.
- It breaks the visual layout.

**Rule:** `apple-touch-icon.png` must have a fully opaque, edge-to-edge background. Gradients or backgrounds must cover the entire canvas including all 4 corners and borders. Never rely on the OS to handle transparent margins gracefully on iOS.

---

## 7. Three App Icon Files Share One Pipeline

Three of our output files share the same universal artwork source:
- `icon-192.png`
- `icon-512-maskable.png`
- `apple-touch-icon.png`

They are not three different designs. They are three exports of the same pipeline:
`Input image → Detect/crop symbol → Create square canvas → Apply opaque background/subtle gradient → Fit symbol inside 76-80% safe zone → Downsample → Encode PNG.`

Role variations:
- **`icon-192.png`** (192x192): PWA standard icon. Used as a general fallback for browsers and older Android systems. Purpose: `"any"`.
- **`icon-512-maskable.png`** (512x512): Android Adaptive Icon & Splash Screen. Requires full-bleed opaque background. Safe zone of 76-80% is critical. Purpose: `"maskable"`.
- **`apple-touch-icon.png`** (180x180): iOS Home Screen Web Clip. Requires full-bleed opaque background. Safe zone of 76-80% is critical. No transparency, no pre-clipped corners.

---

## 8. Safe Zone Rule

To ensure the icon remains intact regardless of the OS mask (circle, squircle, teardrop), the brand symbol must be confined to the center area.

```
Canvas: 100%
Safe zone: 76-80% (inner area)
Bleed area: 20-24% (margins)

┌─────────────────────────┐
│  background full-bleed  │
│   ┌─────────────────┐   │
│   │                 │   │
│   │   BRAND SYMBOL  │   │
│   │   76-80% zone   │   │
│   │                 │   │
│   └─────────────────┘   │
│  background full-bleed  │
└─────────────────────────┘
```
The brand symbol must not touch the canvas edges, but the background must fill the entire canvas.

---

## 9. Favicon.ico Is A Different Problem

`favicon.ico` is not an app icon. It is designed for browser tabs, bookmarks, and legacy desktop environments. It requires its own dedicated pipeline.

Key properties:
- Must contain **16x16** and **32x32** resolutions in a single ICO binary.
- This avoids fuzzy browser-side downscaling.
- Priority: extreme readability at small sizes.

---

## 10. Favicon Pipeline

Unlike app icons, favicons require high contrast, transparency, and minimal details.

The favicon pipeline:
`Input image → Extract/simplify main brand symbol → Remove app background (transparent canvas) → Center symbol in transparent canvas → Optional rounded-square normalization (for symbol definition) → Render 32x32 → Render 16x16 with optional sharpening → Package into ICO.`

**Rules:**
- **Transparent canvas**.
- **No circles by default** (circles reduce the visual area too much at 16px).
- **Optional rounded-square transparent normalization** (adds a transparent rounded box if the symbol lacks self-contained shape).
- Keep it clean: simplify details so the brand remains recognizable at 16x16.

---

## 11. Comparison: App Icons vs. Favicon

| Feature | App Icons (`icon-192.png`, `icon-512-maskable.png`, `apple-touch-icon.png`) | Favicon (`favicon.ico`) |
|---------|----------------------------------------------------------------------------|--------------------------|
| **Target** | OS Home Screen / App Launcher / Splash Screens | Browser Tabs / Bookmarks / History |
| **Background** | Opaque (solid color / subtle gradient) | Transparent (symbol-first) |
| **Margins** | Center safe-zone (76-80%), full-bleed background | Centered, transparent normalization if needed |
| **Masking** | Handled by OS (no pre-clipping) | Not masked (raw square/transparent symbol) |

---

## 12. Manifest Philosophy

The `manifest.json` is not for fixing icon bugs; it simply registers the metadata so the OS and browser know which files to load.

Minimal manifest template:
```json
{
  "name": "App Name",
  "short_name": "App",
  "start_url": "/",
  "display": "standalone",
  "theme_color": "#AUTO_DETECTED",
  "background_color": "#AUTO_DETECTED",
  "icons": [
    {
      "src": "/favicon/icon-192.png",
      "sizes": "192x192",
      "type": "image/png",
      "purpose": "any"
    },
    {
      "src": "/favicon/icon-512-maskable.png",
      "sizes": "512x512",
      "type": "image/png",
      "purpose": "maskable"
    }
  ]
}
```
`theme_color` and `background_color` are auto-detected from the dominant brand/background colors of the input image.

---

## 13. Summary Matrix

1. **`favicon.ico`**: 16x16 + 32x32, transparent, rounded-square transparent normalized, for desktop browser tabs.
2. **`icon-192.png`**: 192x192, PNG, PWA fallback icon, purpose: `"any"`.
3. **`icon-512-maskable.png`**: 512x512, PNG, full-bleed opaque/gradient bg, symbol in 76-80% safe zone, purpose: `"maskable"`.
4. **`apple-touch-icon.png`**: 180x180, PNG, full-bleed opaque/gradient bg, symbol in 76-80% safe zone, no pre-clipped corners, for iOS.
5. **`manifest.json`**: Minimal W3C metadata declaring PWA theme details and icon purposes.
