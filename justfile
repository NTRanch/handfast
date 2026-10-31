# Handfast — task recipes (wired at the v1.0 build, 2026-10-30).
# Toolchain: estate pin nightly-2026-09-25 (rust-toolchain.toml).
# RUSTC_WRAPPER is emptied in the build script and the check recipe:
# the sandbox sccache shim breaks this crate's builds.

# Assemble the single-file jam entry (wasm32 -> base64 -> web/index.html)
build:
    node tools/build.mjs

# Run the week's gates: unit + integration tests (50/50 split), clippy
check:
    PATH="$HOME/.cargo/bin:$PATH" RUSTC_WRAPPER= cargo test
    PATH="$HOME/.cargo/bin:$PATH" RUSTC_WRAPPER= cargo clippy --all-targets -- -D warnings
    PATH="$HOME/.cargo/bin:$PATH" RUSTC_WRAPPER= cargo fmt --check

# Render the per-level preview videos from the shipped core (demo mode)
capture:
    node tools/capture.mjs

# Build the planning book
book:
    mdbook build book
