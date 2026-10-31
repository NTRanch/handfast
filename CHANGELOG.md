# Changelog — Handfast

All notable changes, newest first. Format: Keep a Changelog.

## [Unreleased]

## [1.0.0] — 2026-10-30

The full jam entry, playable end to end in one self-contained
`web/index.html`:

- **Mechanism group implemented** (Prima & Sloane, one two-body beat):
  coupled hand/body anchors with partner-carried contact (a planted
  hand is derived from Sloane's body every step — it cannot slide or
  float), guided-hand Appraisal Brush scoring, dual gaze channels,
  breathing as a perpetual additive layer, expression continuity
  through bounded ramps, and the beat structure approach → contact →
  guidance → held appraisal.
- **Three levels**: First Touch, The Brush's Path, Held Appraisal —
  escalating anchor chains, narrowing plant radii, longer holds.
- **Seven screens**: 18+ gate (shell), Title, How to Play, Gameplay,
  Pause, Settings, Victory, Game Over. Settings hold calm mode
  (reduced secondary motion), the iris-glow switch (a state-driven
  face layer, never permanent — Owner ruling), and sound.
- **Deterministic Rust core** (fixed point, seeded RNG, fixed step)
  compiled to wasm32 and inlined in the page; the same core runs the
  demo mode that produced the preview videos.
- Tests: 37 green (unit + integration, half validation / half
  adversarial: separation attacks, hostile coordinates, degenerate
  timing, malformed saves, invariant fuzz). Clippy 0.1.100 clean
  under `-D warnings`; rustfmt clean.
- Preview videos per level in `docs/previews/`; port-back note in
  `docs/PORT-BACK.md`.

## [0.1.0] — 2026-10-30

- Repo scaffolded 2026-10-30: bloch roadmap (kind=game, lane=restricted),
  planning, per-game skill, baseline Rust config, NTRanch-themed mdBook,
  brand-asset slots.
