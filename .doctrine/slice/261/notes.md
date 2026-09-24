# Notes SL-261: Design adopt verb

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design triage (2026-09-24, design run `dr-01a0d098`, exploring)

Inputs: scope, `research/research.md`, `DEC-100`, `DEC-243`,
`contract_check.rs`, `run.rs:855-935`, memory
`mem.pattern.design-run.correcting-a-locked-run`.

Open questions (candidate inquiry nodes):

- Q1 **Retiring `adopt_authored` from the wire.** `DEC-243`'s roster exists so
  *stored* data carrying a once-known key stays readable. `adopt_authored` is
  never stored (receipts hold a digest, `snapshot.rs:76-82`), so the readable
  half does not apply; what is needed is a write-path refusal that names the
  replacement. Options: (a) a retired wire-key roster `(type, key, remedy)`
  beside `PAYLOAD`, read by `refuse_unknown_keys` → a distinct refusal;
  (b) a one-off match on the key name. Lean (a): one static table, the shape
  `DEC-243` anticipated, no second mechanism.
- Q2 **Where `admit`'s three inputs come from.** Research X3: verb reads
  `run_uid` + `known_revision` from the snapshot, mints `submission_id`.
  Lean yes; the protection is the fingerprint CAS plus the pre-write re-check.
- Q3 **Bare `adopt` basis.** Without `--expect`, the entry read is the basis.
  The new decision must say so, and that `--dry-run` then `--expect` is the
  reviewed path.
- Q4 **Report shape.** Changed / unchanged / added sections + invalidated
  acts/reviews. Changed rows and invalidation rows exist; unchanged/added are
  shell-side set differences. `--dry-run` = same report, no write.
- Q5 **Seam.** Verb builds an internal apply request (new internal-only
  instruction) vs a dedicated core entry point. Must keep `run_apply`'s
  journal-before-snapshot order (`DEC-083`/`DEC-086`) and
  `PreWriteBasis::AdmittedAt`.
- Q6 **Locked runs.** Out of scope; the verb inherits whatever `apply` does at
  `locked` (memory: regress to `reviewing` first). Confirm, don't change.

Risks: e2e tests use adoption as a parser probe (move to `--dry-run`); stale
guidance in hymn/`drafting.md`/two memories.

Assumptions: ASM — caller `sections` map adds no protection (research: holds ✓,
`run.rs:877-895` only compares, `document::parse` enforces completeness).

Governance: new DEC superseding `DEC-100`'s carried rule 2; REV on `SPEC-029`
(`REQ-434`, command family). `DEC-243` (Q1), `DEC-244` (unknown-key refusal),
`STD-001`, `STD-003`, `ADR-001`.

## Review passes

`RV-374` (codex, gpt-6-sol high; concluded 2026-09-24) — six findings, all
fixed and verified. A further pass would probe the implementation, not the
design: whether `apply_pipeline`'s split keeps `DEC-250` mint hoisting intact
in code, and whether the injection-seam test for the one-read rule actually
reaches between the reads. Those belong to the phase VTs and the
implementation code review, so no further design pass is needed.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open
