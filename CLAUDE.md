# aki-favicon-generator — Project Rules

Use global Aki rules from `~/.claude/CLAUDE.md` and stack rules from Aki-RULE.

This file defines project-specific facts and constraints only.

## Applicable Aki Rules
- Core: `RULE-agent-behavior.md`, `RULE-coding.md`, `RULE-docs.md`
- Stack: None (Rust/WASM library)

## Project facts
- **Type:** Rust crate compiled to WASM via `wasm-pack`, plus a Vue 3 / Vite demo.
- **Entry point:** `src/lib.rs` — exports `generate_favicon_set` and `FaviconOptions` to JS.
- **Output:** `pkg/` (generated, gitignored) — consumed by the `demo/` app or any external project.
- **Demo:** `demo/` — standalone Vite + Vue 3 app; run with `cd demo && npm i && npm run dev`.
- **Build:** `make build` → runs `wasm-pack build --target web --release` then `wasm-opt -Oz`.

## Constraints
- Code and comments: **English only** (core-coding rule).
- Do not add dependencies to `Cargo.toml` without checking bundle size impact.
- Do not enable `nodejs_compat` or Node-only APIs — WASM runs in the browser.
- `serde_json` is intentionally excluded from WASM; build `manifest.json` in JS instead.
- Outputs are intentionally limited to 4 image files + 1 manifest. See `docs/feat/artifact-matrix.md`.

## Project docs note
- Keep `docs/index.md` as the master index
- Prefer topic folders from Aki-RULE docs taxonomy: `feat`, `arch`, `plan`, `ref`, `research`
- Completed plans go to `docs/plan/done/`
- See `docs/` for architecture, artifact matrix, integration guides, and caveats.
