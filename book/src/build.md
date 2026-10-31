# Build & Delivery — Handfast

**Language:** Rust (tiger-style-rust), toolchain pin nightly-2026-09-25.

**Delivery shape (as built, v1.0.0):** the deterministic core compiles
to `wasm32-unknown-unknown`; `tools/build.mjs` (run via `just build`)
base64-inlines it into `web/shell.html` to produce the single
self-contained `web/index.html` (120 KiB). The shell is the DOM bridge:
canvas draw-list interpreter, pointer/keyboard input, generated
WebAudio, localStorage saves, and the 18+ gate that runs before the
core starts.

**Screens (7, core-owned):** Title / How to Play / Gameplay / Pause /
Settings / Victory / Game Over — plus the shell-side 18+ gate.

**Verify chain:** `just check` = `cargo test` (36 tests, half
validation / half adversarial) + `cargo clippy --all-targets -- -D
warnings` (clippy 0.1.100 resolved) + `cargo fmt --check`.
`just capture` re-renders `docs/previews/level-<n>-handfast.mp4` from
demo mode through the shared rasteriser and ffmpeg.

**Book:** `mdbook build book` renders this planning book to `book/html/`
with the NTRanch theme (dark background, hot pink).
