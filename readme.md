# aki-favicon-generator

Client-side WASM favicon generator — produces a PWA-compliant icon set from a single PNG/JPEG input in under 1 second, with zero server cost.

## Quick start

```bash
# 1. Install wasm-pack (one time)
cargo install wasm-pack

# 2. Build WASM + optimize
make build
# Output: pkg/aki_favicon_generator.js + pkg/aki_favicon_generator_bg.wasm

# 3. Run the web demo
cd demo && npm install && npm run dev
# Open http://localhost:5173, upload a PNG/JPEG, download favicon.zip
```

**Result:** a single `.zip` containing `favicon.ico`, `icon-192.png`, `icon-512-maskable.png`, `apple-touch-icon.png`, and `manifest.json` — all PWA-compliant.

- **Philosophy & design principles** — universal design, shape fallacy, safe-zone rule, background gradients, favicon pipeline details → [docs/philosophy/master.md](docs/philosophy/master.md)
- **Architecture & design** — goals, safe-zone rule, data flow, memory constraints → [docs/arch/design.md](docs/arch/design.md)
- **Artifact matrix** — output files, manifest template, MVP scope, caveats → [docs/feat/artifact-matrix.md](docs/feat/artifact-matrix.md)
- **Build & integration** — build commands, JS API, Nuxt 4 setup, HTML snippet → [docs/ref/build-integration.md](docs/ref/build-integration.md)

## Live PWA Testing

The demo site includes a **Live PWA Testing** feature.
It compiles a minimal PWA test page using your generated favicon assets and deploys it live to `https://test.akivn.net` via local `rsync` over SSH (requires a configured `akicloud` host in your SSH configuration). This allows instant end-to-end testing on physical devices (mobile/tablet) by scanning a generated QR code.

