# Playtest record — Handfast v1.0 (2026-10-30)

Playtest ran in **demo mode** (the core's scripted policy playing the
shipped build) — the PLANNING.md path for this programme; the
seahawk-brokered session named in roadmap R-303 was not available
(sandbox↔primo tunnel down all day, 2026-10-30) and the substitution is
recorded in `docs/BREAKAGE-LOG.md`. Demo mode exercises the exact
shipped wasm through the same input surface a player uses.

## What ran

- Demo policy completes all 3 levels unattended (cargo integration
  test `demo_mode_wins_every_level_unattended`).
- Scripted manual playthrough wins all 3 levels end to end
  (`every_level_is_winnable_end_to_end`).
- Capture pipeline rendered 12 s per level from the shipped core;
  frames inspected at viewing size (D-221).

## Observations (not verdicts)

- The beat reads: approach and plant are legible from the anchor ring
  and the gold hand ring; guidance strain is readable from the
  connecting arm and the strain arc; the appraisal ring's gold fill
  sweeps visibly.
- Guidance on level 1 completes in ~4 s of the 12 s capture; the
  appraisal hold dominates the frame time, as designed (the held beat
  is the longest shot).
- First visual pass caught hair strands crossing both faces; fixed to
  side locks and re-captured (before/after frames kept with the gate
  evidence in the programme directory).
- Demo scores land at 1000/1000 under perfect policy play; a human's
  wobble is what the scoring gradient (steadiness × smoothness) is
  built to measure. No human hands have played it yet — Owner play is
  the open item, and his ear/eye verdicts queue above this record.

## Audio

No audio was played aloud (estate silence rule). The shell's generated
WebAudio cues are code-reviewed only: plant blip, break thud, guide-
complete chime, victory arpeggio, menu tick — all gated on the sound
setting and started only after the 18+ gate click.
