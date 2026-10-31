# Port-Back — Handfast

Filed at the v1.0 build, 2026-10-30. The full torogao/lumen-rig filing
lives in `docs/PORT-BACK.md`; this chapter carries the verdicts.

- **Verdict:** plays. All three levels complete end to end under the
  demo policy and under scripted manual play (cargo integration tests);
  the mechanism group — coupled anchors, partner-carried contact,
  guided-hand brush scoring, dual gaze, breathing layer, expression
  ramps, beat grammar — holds under a 20 000-step adversarial fuzz.
- **Numbers:** planted-hand positional error stays within the tear
  radius by construction (contact is derived from the partner's anchor
  every tick, never stored in world space); demo win on all levels;
  frame draw lists ≤1 024 ops by cap; preview videos ~95 KiB each.
  Contact-zone acceptance metric for torogao: ≥95% of frames within an
  intimate shot (from the reference's measured 564/581 = 97%).
- **Language verdict (Rust):** the fixed-point + draw-list core cost
  nothing at runtime and bought byte-determinism across native tests,
  browser wasm, and the Node capture pipeline from one source. The
  fight was the FFI seam (hand-rolled string buffers, one contained
  `unsafe` module) — acceptable, and the pattern ports to any
  wasm-target language lane.
- **Breakage:** see `docs/BREAKAGE-LOG.md` (clang symlink, wasm target
  install, the planted-hand invariant redefinition, the plant-snap
  render fix, the face-crossing hair strand caught by the visual gate).
- **§9 transfer filing:** the coupled-anchor contact data model, the
  beat grammar, expression-ramp law, and breathing-as-layer are
  technique with **content_label: sfw** and may transfer to the SFW
  lane. The cast, staging text, and appraisal framing are NSFW-pure
  and stay in this lane. The glowing-iris-as-layer rule is an Owner
  ruling (2026-10-30) and travels with the technique filing.
