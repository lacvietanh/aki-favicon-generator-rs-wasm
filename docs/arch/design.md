# Technical Architecture & System Design Specification

This document defines the system topology, structural patterns, and optimization constraints of the Aki Favicon Generator. It is written for engineering maintenance, ensuring strict adherence to **SOLID**, **DRY**, and high-performance WebAssembly memory practices.

---

## 1. System Topology

The system is partitioned into three distinct execution contexts with strict boundary lines, enforcing the **Single Responsibility Principle (SRP)** at the application layer:

```
┌────────────────────────────────────────────────────────────┐
│                    Client Browser Context                  │
│                                                            │
│  ┌───────────────────────────┐    ┌─────────────────────┐  │
│  │   UI Dashboard (Vue 3)    │◄──►│ ZIP/JSON (JS-Only)  │  │
│  └───────────────────────────┘    └─────────────────────┘  │
└──────────────────────────────┬─────────────────────────────┘
                               │ (WASM Bridge - Direct Memory)
                               ▼
┌────────────────────────────────────────────────────────────┐
│                  WebAssembly Context (Rust)                │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Core Engine (color, transform, encoder)              │  │
│  └──────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────┘
                               ▲
                               │ (Local HTTP Intercept)
                               ▼
┌────────────────────────────────────────────────────────────┐
│                Vite Dev Server (Node Context)              │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ In-Memory PWA Host / API Deploy Middleware           │  │
│  └──────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────┘
```

| Context | Responsibility | Key Constraints |
|:---|:---|:---|
| **Vue 3 UI** | User interaction, options configuration, preview rendering, and state management. | No pixel manipulations. |
| **JS Helpers** | ZIP archive compilation (`fflate`), JSON stringifying, PWA installation lifecycle, and DOM metadata manipulation. | Offloads intensive binary computations to WASM. |
| **Rust WASM** | Multi-size Lanczos3 downsampling, color analysis, rounded corner masking, PNG encoding, and ICO structure packing. | No filesystem or network access. |
| **Node Middleware** | In-memory storage of deployed artifacts, dynamic route interception for PWA live-testing. | Zero disk writes (avoids file-watcher rebuild loops). |

---

## 2. SOLID Implementation Patterns

### Single Responsibility Principle (SRP)
Within the WebAssembly context, code is segregated into specialized sub-modules:
- `lib.rs`: Exposes the public WASM-bindgen interface and coordinates high-level execution flow.
- [color.rs](file:///Volumes/DEV/wasm/aki-favicon-generator/src/color.rs): Performs color histogram scanning and hex parsing.
- [transform.rs](file:///Volumes/DEV/wasm/aki-favicon-generator/src/transform.rs): Handles scaling, padding, and safe-zone clipping calculations.
- `encoder/png.rs`: Deals strictly with zlib/deflate compression of the pixel buffer.
- [ico.rs](file:///Volumes/DEV/wasm/aki-favicon-generator/src/encoder/ico.rs): Assembles multi-directory ICO structures and implements rounded corner masks.

### Open/Closed Principle (OCP)
The parameters passed to the engine are encapsulated inside the `FaviconOptions` struct. This struct uses the builder pattern:
```rust
#[wasm_bindgen]
impl FaviconOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self { ... }
    pub fn with_theme_color(mut self, color: String) -> Self { ... }
    pub fn with_background_color(mut self, color: String) -> Self { ... }
    pub fn with_safe_zone(mut self, ratio: f32) -> Self { ... }
}
```
This design allows adding new configuration parameters (e.g. customized corner radius, specific bleed scales) without altering the signature of the entry point `generate_favicon_set`.

### Liskov Substitution Principle (LSP)
All transformations and encoding steps operate uniformly on `image::RgbaImage` buffers. Functions in `transform.rs` and `encoder/*.rs` do not care about the origin of the image data, treating all pixel buffers consistently according to standard RGBA structures.

### Interface Segregation Principle (ISP)
Instead of passing complex serialized JSON strings back and forth across the WASM boundary (which incurs severe serialization and copying costs), the interface communicates via primitive fields and typed properties in the `FaviconSet` struct:
```rust
#[wasm_bindgen]
pub struct FaviconSet {
    pub favicon_ico: Vec<u8>,
    pub icon_192: Vec<u8>,
    pub icon_512_maskable: Vec<u8>,
    pub apple_touch_icon: Vec<u8>,
    pub theme_color: String,
    pub background_color: String,
}
```
The client accesses only the exact byte buffers it requires, keeping the WASM bridge interface lean and efficient.

### Dependency Inversion Principle (DIP)
Core algorithms are decoupled from the physical assets or browser UI. The engine relies on abstract data input bytes (`&[u8]`) and produces standard output vectors (`Vec<u8>`), making the Rust code highly testable in standard CLI environments independent of WebAssembly bindings or browser host environments.

---

## 3. DRY (Don't Repeat Yourself) Rules

To prevent algorithmic duplication:
- **Rescaling Logic**: All Lanczos3 downsampling is centralized in `transform::resize_exact` and invoked uniformly for the 192px and ICO targets.
- **Composition Canvas**: Safe-zone embedding is handled exclusively by `transform::fit_safe_zone`, reused by both the 512px maskable icon and the 180px apple-touch-icon.
- **Base64 Encoding**: In the frontend, UTF-8 safe base64 conversions are channeled through a single unified `stringToBase64` helper.

---

## 4. Memory Management & Browser Safety

WebAssembly runs inside the browser's sandbox with a limited linear memory space. Large input images (such as a 12 MP phone camera upload) can easily cause Out-Of-Memory (OOM) crashes if memory is managed poorly.

### Memory Optimization Rules:
1. **Sequential Computations**: Targets are processed sequentially (e.g. processing `apple_touch_icon`, then freeing buffers, then processing `icon_512_maskable`). Parallel operations are forbidden to prevent simultaneous peak memory allocations.
2. **Immediate Discarding**: Intermediary buffers (like raw resized pixel vectors) are discarded immediately after encoding. Only the final compressed PNG/ICO streams are returned.
3. **No Heavy Parser Dependencies**: JSON parsing is omitted from the Rust build. `serde_json` is not imported; manifest generation is executed strictly in JavaScript.

---

## 5. WebAssembly Optimization Profile

To maintain high load speeds, the release bundle must remain small (Target: **≤ 500 KB** gzipped). The compilation pipeline uses the following size-reducing configurations:

```toml
# Cargo.toml optimization settings
[profile.release]
opt-level = "z"         # Optimize for size
lto = true              # Enable Link Time Optimization
codegen-units = 1       # Reduce parallel code generation to optimize LTO
panic = "abort"         # Eliminate panic formatting strings
strip = true            # Remove symbols and debug info
```

Post-compilation, the binary is processed with `wasm-opt`:
```bash
wasm-opt -Oz pkg/aki_favicon_generator_bg.wasm -o pkg/aki_favicon_generator_bg.wasm
```
This step optimizes WebAssembly instruction layouts, stripping unused imports and shrinking the bundle by another 20-30%.
