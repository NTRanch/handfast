# Planning — Handfast

- **Program:** Torogao Jam Program (D-333), week 4 of 14, Wave A — Animation
- **Cast:** Prima & Sloane
- **Language:** Rust (tiger-style-rust)
- **Canon core:** One two-body beat — a handoff, a held gaze, a hand guided into position.

## Mechanism group (D-334: primary + coupled + persistence/revelation + feel)

- Two-body staging and occlusion
- Contact attempts (a research problem, not a ceiling — breakage here is expected and is the point)
- Dual gaze — both girls' gaze channels live in the same scene
- The Appraisal Brush scoring the guided hand — Sloane's appraisal inside Prima's induction register
- Two presiding styles in one room (induction vs protocol)

## Validates

The animation pipeline's universal stress test (a believable two-body scene) at prototype scale, plus whether two girls' systems interact or merely coexist.

## Port-back

Contact/occlusion findings to the cross-medium pipeline goal; cross-presiding interaction notes; Rust verdict (the home-language baseline).

## Delivery

Compile to wasm32, inline the module (base64) in the single index.html; JS is loader and DOM bridge only. Toolchain home: sandbox.

## Open preflight items

- GJ-06-class start gate for this week's toolchain (later weeks gate their
  own preflight; only W1's gate covers the program start).
- GJ-07: 18+ flag, content warnings listed, itch ToS screen — verified live
  at entry (the NSFW allowance clause is Owner-reported; re-verify on page).
- Engine-research chapter in the book: confirm the pinned library versions
  before the first build commit.
