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
fresh-as-of: 2026-09-24 · PHASE-01..06 landed; audited (RV-379 done; reconcile next)

### Produced

- DEC-278, DEC-279 (settled; shape SL-261). DEC-279 references DEC-100; DEC-278 references DEC-243.
- RV-374 (design review, concluded; F-1..F-6 verified; RFC-026 routes amended in prose).
- design.md locked (run dr-01a0d098, revision 37); plan.toml PHASE-01..06.
- ISS-477, IDE-056, ISS-478 (deferred; originate from SL-261).
- RV-375 (code review of PHASE-01, concluded; F-1 fixed in place, F-2 → ISS-478).
- RV-376 (code review of PHASE-02, concluded; F-1 tolerated, F-2 design-wrong → plan EX-4 amended).
- PHASE-03 (bfd38442e): `design adopt` verb, `run_adopt`, `refuse_adoption_at`, `Refusal::AdoptionLocked`, `ApplyRequest::bare`, report (rows, unchanged, reordered, head).
- PHASE-04 (58a4c57f3): `--diff` via `similar`; `SectionGroup::changed_since`.
- PHASE-05 (c80c38599): `adopt_authored` deleted and retired (RETIRED_KEYS row); refusals name the verb; tests migrated; RV-378 (code review, concluded; F-1..F-4 fixed in place).
- PHASE-06 (6d3811eee, 968cc8903, 5defe6c26): hymn + drafting wording; five memories; REV-057 (SPEC-029 + REQ-434; user-approved, applied, done).
- RV-379 (audit, done; F-1/F-2/F-5/F-6 → reconciliation brief, F-3 tolerated, F-4 aligned).
- PHASE-02: `run::Crossing { Ordinary, Adopt { expect } }`; pure `apply` takes `&Crossing`, caller-map check split to `refuse_invalid_markers` and run only when the wire's `adopt_authored` rides along (transitional). Shell `apply` is now the wire shell over `parse_payload` + `apply_pipeline(root, slice, PipelineInput, Stop, pre_write, fault) -> PipelineOutcome`; `AuthoredRead`/`read_authored` is the one read (also used by `start` and `read_authored_fingerprint`); report lines factored to `applied_lines`.
- PHASE-01: `RetiredKey`/`RETIRED_KEYS` (empty) + `retired_from` identity match in payload_contract; `Refusal::RetiredPayloadKey`; roster threaded through the contract_check walk via a private `refuse_unknown_keys_against`; `render_prompt_against`/`render_json_against` list retired rows after live rows (JSON member only when non-empty, so the published contract is byte-unchanged); VT-2 pins in design_run/tests.rs.

### Learned

- Adoption today reads design.md twice (RV-374 F-1); the verb must read once.
- Parser keeps no copy of a whitespace-only head; materialise drops it (RV-374 F-2).
- payload_contract::PAYLOAD is the one const contract; identity match needs it static (RV-374 F-5).
- PHASE-01: a roster row's owner must be a struct **today** — the never-live pin treats an enum owner as a fault while `walk_keys` does support enum owners, so the checked domain is narrower than the supported one (RV-375 F-2; ISS-478).
- A Rust `///` block binds to the *next* item, so inserting an item between a doc comment and its type silently steals the type's opening lines (RV-375 F-1: `RetiredKey` landed inside `PAYLOAD`'s doc). Place new items wholly before or after an existing doc block.
- The aligned no-op short-circuits before any parse, so a parser-readout probe of a just-materialised document needs a body-neutral divergence: a blank-line head, then `adopt --dry-run` (PHASE-05 D1; RV-379 F-6; mem_019facc2).
- On a locked run, regress BEFORE hand-editing: after divergence the regress payload is refused and `adopt` refuses a locked run (mem_019fdf95).
- `memory verify` refuses a dirty tree and each verify dirties it — verify+commit one memory at a time (observation cc5b841be).
- A phase's recorded range is one contiguous span; interleaved commits from a concurrent slice on `edge` ride along (RV-379 F-3).
- Design-run friction captured as observations (envelope lacks runbook; provenance nesting; cp- disposal shape; route token placement).

### Open

- Reconciliation brief in RV-379: selector registry adds, design.md sec-3/sec-6/sec-7 edits (F-1, F-2, F-5, F-6).
- Deferred: ISS-477, IDE-056, ISS-478.
