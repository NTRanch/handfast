# Breakage log — Handfast

Every breakage found in playtest or build lands here with: what broke, the
evidence, the bloch attempt id, and the fix or the filed finding. Copied to
bloch at the week's close (the log is the input, bloch is the record).

## 2026-10-30 — v1.0 build

1. **Sandbox clang missing.** First cargo build failed at link:
   `/usr/bin/clang` symlink absent in the sandbox. Fixed with the
   estate's `~/workspace/bin/ensure-sandbox-clang` (clang 23.1.3).
   Environment breakage, not code. (R-101 attempt.)
2. **wasm32 target absent from the pin.** `rustup target list
   --installed --toolchain nightly-2026-09-25` showed only the host
   target. Installed with `rustup target add wasm32-unknown-unknown`;
   the full wasm route was used — the native-only fallback in the
   programme plan was not needed. (R-101.)
3. **Scaffold clippy gate inherited four findings** (a manual `Range`
   impl, `.is_multiple_of()`, a needless `Pass`, a derivable `Default`)
   plus a dead-code warning on the baseline, all under the estate's
   `-D warnings`. Fixed curatively in the baseline commit; the
   scaffold's `just check` stub (referencing a Makefile that never
   existed) was replaced with real recipes at v1.0. (R-102.)
4. **Planted-hand invariant was mis-stated in the first test draft.**
   The carried-hand test measured hand-to-*current-anchor* distance
   during Guidance and reported 96px "drift" — but during Guidance the
   hand legitimately travels the body path between anchors. The
   invariant was redefined to distance-to-path (segment polyline);
   with that fixed, two real defects surfaced and were fixed: the
   render hand spring was not synchronised for a free hand, and the
   plant instant did not snap the render hand (a click-teleport plant
   left up to ~311px of spring convergence in the fuzz). Contact
   correctness is now: logical hand derived from the partner anchor
   every tick; render hand springs only while planted, after arriving
   with the plant. (R-201/R-301.)
5. **Hair strands crossed both faces.** The D-221 frame check on the
   first captured videos showed the centre hair strand hanging straight
   down each figure's face. Strands re-anchored as side locks
   (−56/−47/+54) starting at the hairline; re-captured and re-checked —
   faces clear. (R-303.)
6. **Seahawk playtest route unavailable.** Roadmap R-303 names a
   seahawk-brokered playtest; the sandbox↔primo tunnel was down for
   the whole build day (heartbeat failing since 03:15 PDT), so the
   playtest ran in demo mode against the shipped core instead, per
   PLANNING.md's demo-capture path. Recorded in `docs/PLAYTEST.md`.
   The seahawk session remains an open follow-up, not a silent swap.
   (R-303.)
