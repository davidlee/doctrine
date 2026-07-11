# Notes SL-213: Comparison constraint layer

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01 — schema v2 wire model + module split + capture (completed 2026-07-11)

Commits: `91f952ef` (T1 mechanical split), `3b8a1c96` (v2 wire + capture).
`doctrine slice verify-vt SL-213`: PHASE-01 VT-1/VT-2/VT-3 all PASS. Full bin
suite 3306 green; `doctrine check gate` clean.

- **Split first, redefine second.** T1 (`comparison.rs` → `comparison/wire.rs`
  + re-exporting `mod.rs`) committed separately with the suite passing
  UNCHANGED — the behaviour-preservation proof is the commit boundary.
- **Clap response group was a non-event.** `#[command(group =
  ArgGroup::new("response").required(true).multiple(false))]` + `group =
  "response"` on `--prefer`/`--equal`/`--incomparable`; usage renders
  `<--prefer <PREFER>|--equal|--incomparable>`. The SL-210
  subcommand-interaction precedent did not recur.
- **Eq derives dropped** from `Judgement`/`ComparisonSession` (`magnitude:
  Option<f64>`); nothing downstream needed `Eq`.
- **Design deviations: none.** A2 (kebab-case response tokens), A4 (no
  `--magnitude` flag — OQ-6), A6 (per-domain frame table replaces
  VALUE_FRAMES; parse-level string losslessness retained, closed table
  enforced at validate) all per design.
- **For PHASE-02:** `resolve.rs` consumes `wire::Judgement.supersedes`
  (already corpus-validated at capture, but R2 must still handle unknown
  uids from hand-merged files as load-time warnings, not errors);
  `domain_for_frame` / `DOMAIN_FRAMES` in wire.rs is the single
  domain-derivation seam.
