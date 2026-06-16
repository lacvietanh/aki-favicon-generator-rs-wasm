# Build & Integration Guide

This guide explains how to compile the Rust WebAssembly engine, run the local demo server, integrate the generator into JS-based applications, and implement the output files in production.

---

## 1. Prerequisites

You need the Rust toolchain and `wasm-pack` installed on your machine:

```bash
# Install rustup (compiler toolchain)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
cargo install wasm-pack
```

---

## 2. Compile WebAssembly Engine

A `Makefile` is provided for compiling and sizing optimized binaries:

```bash
# Build and run wasm-opt to compress binary for size
make build

# Clean cargo cache, pkg, and node dependencies
make clean
```

The underlying compile sequence executed is:
```bash
wasm-pack build --target web --release
wasm-opt -Oz pkg/aki_favicon_generator_bg.wasm -o pkg/aki_favicon_generator_bg.wasm
```

---

## 3. Run the Development Server

To run the local Vite + Vue 3 demonstration environment:

```bash
cd demo
npm install
npm run dev -- --host
```

### Local PWA Testing Architecture:
The Vite configuration includes custom HTTP middleware that serves PWA assets directly at the root `http://localhost:5173/`:
- **API Endpoint**: `POST /api/deploy` receives base64 payloads of manifest and icon data and stores them in Node's RAM.
- **Route Interceptor**: Intercepts `GET /manifest.json`, `GET /sw.js`, and `GET /favicon/*` to serve these dynamically updated cached assets.
- **Dynamic Head Refresh**: Once deployed, the demo page reloads the manifest link with a cache-buster query parameter (`/manifest.json?v=...`) to trigger browser PWA install prompts directly on the main page.

---

## 4. Client SDK Integration

Call the WebAssembly module directly on the Main Thread (Lanczos3 downsampling takes <1s, avoiding Web Worker overhead for common targets):

```js
import init, { generate_favicon_set, FaviconOptions } from './pkg/aki_favicon_generator.js'
import { zipSync } from 'fflate'

async function createFavicons(imageArrayBuffer, appName = 'App', shortName = 'App') {
  // Initialize WebAssembly engine (idempotent call)
  await init() 

  const imageBytes = new Uint8Array(imageArrayBuffer)
  const options = new FaviconOptions() // Default: auto-detect palette colors

  // Run heavy processing in WASM
  const result = generate_favicon_set(imageBytes, options)

  const manifest = {
    name: appName,
    short_name: shortName,
    start_url: '/',
    display: 'standalone',
    theme_color: result.theme_color,
    background_color: result.background_color,
    icons: [
      { src: '/favicon/icon-192.png', sizes: '192x192', type: 'image/png', purpose: 'any' },
      { src: '/favicon/icon-512-maskable.png', sizes: '512x512', type: 'image/png', purpose: 'maskable' },
    ],
  }

  // Compile ZIP archive entirely client-side
  return zipSync({
    'favicon/favicon.ico':             result.favicon_ico,
    'favicon/icon-192.png':            result.icon_192,
    'favicon/icon-512-maskable.png':   result.icon_512_maskable,
    'favicon/apple-touch-icon.png':    result.apple_touch_icon,
    'favicon/manifest.json':           new TextEncoder().encode(JSON.stringify(manifest, null, 2)),
  })
}
```

---

## 5. HTML Production Integration

Copy the generated `favicon/` folder assets into your web project root and add the following 4 lines inside your index HTML `<head>`:

```html
<!-- Integration Header template -->
<link rel="icon" href="/favicon/favicon.ico" sizes="32x32">
<link rel="apple-touch-icon" sizes="180x180" href="/favicon/apple-touch-icon.png">
<link rel="icon" type="image/png" sizes="192x192" href="/favicon/icon-192.png">
<link rel="manifest" href="/favicon/manifest.json">
```

> [!NOTE]
> **Strictly Not Needed:** `favicon.svg`, `browserconfig.xml`, `mstile-*`, or `safari-pinned-tab.svg` are obsolete. Modern browsers fall back correctly to the standard set above.
