---
name: handfast
description: Build and validate Handfast (Torogao Jam Program week 4, Rust). Use when working on the Handfast jam entry — its mechanism group, Rust build chain, single-file delivery, and port-back.
---

# Handfast — week 4 jam entry

**Canon core:** One two-body beat — a handoff, a held gaze, a hand guided into position.
**Cast:** Prima & Sloane · **Language:** Rust (tiger-style-rust)

## Mechanism group

- Two-body staging and occlusion
- Contact attempts (a research problem, not a ceiling — breakage here is expected and is the point)
- Dual gaze — both girls' gaze channels live in the same scene
- The Appraisal Brush scoring the guided hand — Sloane's appraisal inside Prima's induction register
- Two presiding styles in one room (induction vs protocol)

## Build

- Delivery: Compile to wasm32, inline the module (base64) in the single index.html; JS is loader and DOM bridge only. Toolchain home: sandbox.
- Planning book: `book/` (mdBook; `mdbook build book`).
- Engine/library pins: book chapter "Engine Research" — read before adding
  any dependency; the from-scratch register (book chapter "Write From
  Scratch") names what must NOT be replaced by a library.

## Gates

1. bloch: start the roadmap item before work; `bloch task done` earns green.
2. Single-file assembly runs offline with all ≥6 screens.
3. Seahawk playtest pass; breakage filed to bloch.
4. Visual gate (D-221) against the Laplace3D quality bar (book chapter).
5. Port-back note + language verdict + §9 filing (book chapter "Port-Back").

## Standing law

D-292 keeper art only · audio silence (metrics only, Owner presses play) ·
18+ surfaces flagged (GJ-07) · restricted lane.
