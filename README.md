> **Game page:** the full storefront page for this game lives at [docs/GAME-PAGE.md](docs/GAME-PAGE.md).

# Handfast

**Nice Thoughts Ranch** · Torogao Jam Program week 4 (Wave A — Animation) · Prima & Sloane · **Rust**

One two-body beat — a handoff, a held gaze, a hand guided into position.

An 18+ game built for the weekly Gameotron jam on itch.io (NTRanch), as one
rung of the 14-week Torogao system-validation program (D-333/D-334/D-335/D-336).
The full planning book lives in [`book/`](book/) (mdBook, NTRanch theme);
this repo's bloch roadmap is [`.ai/roadmap.md`](.ai/roadmap.md).

- **Delivery:** one self-contained `index.html` (see `docs` in the book).
- **Cast/art law (D-292):** existing keeper art only; a missing asset is an
  abstract stand-in, logged — never a generation job.
- **Quality bar:** Laplace3D-class animation or the medium's equivalent
  (book chapter: Quality Bar).
- **Audio:** nothing is ever played aloud by an agent; the Owner presses play.

Status: **v1.0.0 (2026-10-30)** — playable end to end.

## Play

Open `web/index.html` in a browser (no server, no assets, no network).
An 18+ gate with content warnings comes first — nothing starts, and no
sound plays, until you enter.

Your hand is Prima's. Bring it to Sloane's body and **press** (mouse
down or Space) when the anchor ring glows to plant it. Once planted,
the hand is *carried by Sloane* — it rides her breathing body and
cannot drift. Then:

- **Guidance** — keep the planted hand travelling with the Appraisal
  Brush along her body. Strain past the tear line for half a second
  breaks contact (three breaks end the beat).
- **Held Appraisal** — rest the gaze ring on the pulsing point until
  the regard meter fills. This is the beat's longest shot.

**Esc / P** pauses. Arrow keys + Enter work the menus; in play, arrows
can drive the hand and Space presses. Settings hold calm mode (reduced
secondary motion), the iris-glow switch, and sound. Progress and best
scores persist in the page's local storage.

### Levels

1. **First Touch** — Hip → Heart. Learn the plant; generous radii.
2. **The Brush's Path** — Hip → Shoulder → Cheek. Longer road, tighter
   strain.
3. **Held Appraisal** — Shoulder → Heart → Cheek. The longest hold,
   the deepest breathing — and at the peak of her regard, Sloane's
   irises glow (a switchable layer: Settings can turn it off).

### Preview videos

Rendered from the shipped core in demo mode (`tools/capture.mjs`) —
the game playing itself:

- [Level 1 — First Touch](docs/previews/level-1-handfast.mp4)
- [Level 2 — The Brush's Path](docs/previews/level-2-handfast.mp4)
- [Level 3 — Held Appraisal](docs/previews/level-3-handfast.mp4)

## Architecture

A deterministic Rust core (fixed point, seeded RNG, fixed step) emits
an abstract draw list on a 960×960 logical stage; the page shell
interprets it on canvas, owns input and generated WebAudio, and holds
the 18+ gate. The same core runs natively under `cargo test` and in
Node for the capture pipeline. Staging is abstract/symbolic per D-292
(no keeper art existed for this cast; the stand-in is the shipped
staging, not a placeholder for a generation job). Mechanism validated
against the fathom measurements of references 5447477 and 12369020 —
see `book/` and `docs/PORT-BACK.md`.

## Build & verify

```sh
just build    # cargo wasm32 build -> base64 -> web/index.html
just check    # cargo test (37, half adversarial) + clippy -D warnings + fmt
just capture  # re-render docs/previews/*.mp4 from demo mode
just book     # build the mdBook planning book
```

Toolchain pin: nightly-2026-09-25 (`rust-toolchain.toml`).
