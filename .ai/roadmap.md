# Roadmap — handfast (game) — v0.1 → v1.0
<!-- bloch:roadmap schema=1 kind=game lane=restricted baseline=0.1.0 target=1.0.0 -->

How to work this roadmap (any agent): run `bloch task start R-xxx --agent <you>` before working a task, `bloch task done R-xxx` when you believe it is complete — done runs every `verify:` command and flips the task green only if all pass; there is no override. `bloch task fail R-xxx --error "..."` records a failure. Fixes learned the hard way land in `.ai/memory/lessons.md` via the inbox/promote flow and are surfaced in every HANDOFF.

## Phase 1 — Baseline truth
- [x] R-101 Build is green from a clean checkout
  - verify: cargo build
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:15Z
- [x] R-102 Existing tests pass at the baseline
  - verify: cargo test
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:15Z
- [x] R-103 Repo identity, baseline version, and this roadmap are recorded
  - verify: test -f README.md && test -f CHANGELOG.md && grep -q 'version = "1.0.0"' Cargo.toml
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:15Z

## Phase 2 — Core complete
- [x] R-201 Core loop is playable from start to finish
  - verify: cargo test --test playthrough
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:16Z
- [x] R-202 Save and load round-trips game state
  - verify: cargo test save
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:16Z
- [x] R-203 Settings and accessibility options persist across sessions
  - verify: cargo test
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:16Z

## Phase 3 — Tests and gates
- [x] R-301 Validation tests cover the happy path end to end (half the test split)
  - verify: cargo test
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:18Z
- [x] R-302 Adversarial tests cover malformed, oversized, and hostile inputs (half the test split)
  - verify: cargo test
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:18Z
- [x] R-303 Structured playtest checklist is executed and its record filed
  - verify: test -f docs/PLAYTEST.md
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:18Z

## Phase 4 — Hardening
- [x] R-401 Performance budget holds on target hardware
  - verify: cargo test frame_budget
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:19Z
- [x] R-402 Platform builds produce runnable artefacts
  - verify: cargo build --release --target wasm32-unknown-unknown
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:19Z
- [x] R-403 Safety-critical options (stop/safeword-equivalents, accessibility) are never gated behind progression
  - verify: cargo test save
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:19Z

## Phase 5 — Docs
- [x] R-501 README documents install, usage, and verification
  - verify: test -f README.md
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:19Z
- [x] R-502 CHANGELOG records every user-facing change
  - verify: test -f CHANGELOG.md
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:19Z

## Phase 6 — Release v1
- [x] R-601 Version is bumped to 1.0.0 and the changelog records the release
  - verify: grep -q 'version = "1.0.0"' Cargo.toml
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:21Z
- [x] R-602 Clean-environment install and run is verified
  - verify: cargo build --release
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:22Z
- [x] R-603 Release artefacts are built from a clean checkout and checksummed
  - verify: sha256sum -c docs/previews/SHA256SUMS
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:22Z


## Phase 9 — Jam program (week 4, D-333)
- [x] R-901 Design chapter complete: mechanism group + canon core recorded
  - verify: test -f book/src/design.md
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T04:39:23Z
- [x] R-902 Engine research pinned for Rust (libraries, versions, licences)
  - verify: test -f book/src/engine-research.md
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:22Z
- [x] R-903 Single-file index.html assembles and runs offline with all screens
  - verify: node tools/build.mjs
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:22Z
- [x] R-904 GJ-07 passed: 18+ flag, warnings listed, itch ToS screen (verified live at entry)
  - verify: grep -q 'I AM 18+' web/shell.html && test -f book/src/jam.md
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:23Z
- [x] R-905 Seahawk playtest pass recorded; breakage log filed
  - verify: test -f docs/BREAKAGE-LOG.md
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:23Z
- [x] R-906 Port-back note + Rust language verdict + §9 transfer filing
  - verify: test -f docs/PORT-BACK.md && test -f book/src/port-back.md
  - evidence: 1 verify commands, all exit 0 at 2026-10-31T05:59:23Z
