# Notes SL-262: Envelope names the next move

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open

## Design surface triage (2026-09-24, design run exploring)

Source: `research/research.md` (✓ rows) plus these checks at the cited sites.

### New findings beyond research

- **X6 — runbook and contract rows sit outside `TurnEnvelope`.** `run_resume`
  appends `runbook_section` and `contract_section` after `envelope::resume`
  (`src/commands/design.rs:2840-2847`). `show --format prompt` and JSON never
  carry the runbook row. Deriving the next move needs runbook standing inside
  `project()`, which pulls that row onto the envelope (`DEC-064`, `REQ-433`: no
  projection carries a field the envelope lacks).
- Apply builds `DerivedInput` inline (`design.rs:2091`); read path builds none.
  A shared builder is the DRY seam (research delta 4).

### Open questions (inquiry map candidates)

1. Wire shape of the next move; fate of `next_obligation` key. (blocking)
2. `DEC-124` refinement + envelope compatibility rule. (blocking)
3. Read-path fact assembly: one builder shared with apply; truthfulness of
   observed facts on reads. (blocking)
4. Which projections carry the rows; elision class under `REQ-424`/`REQ-437`.
5. Disclosure of `regressed` blindness on reads (`STD-003`).
6. Guidance edits (hymn, fragments) pointing at the new rows.

### Risks

- False "unmet" rows if a read skips `observed_facts` (fail-closed, `gate.rs:1436`).
- Read-path cost of `satisfied` over the cumulative set (X4) — measure.
- Snapshot serde: removing `next_obligation` must still read old snapshots
  (no `deny_unknown_fields` on `RunHeader` — ✓ research).

### Assumptions

- Every gate condition is evaluable read-only given `DerivedInput` (verifications
  empty on read).

### Constraining governance

`DEC-064`, `SPEC-029` (`REQ-433`, `REQ-437`, contract table), `PRD-019`
(`REQ-414`, `REQ-416`), `DEC-067`, `DEC-066`/`DEC-120`/`DEC-126`, `DEC-101`,
`DEC-065`/`DEC-060` (storage rule), `DEC-124`, `DEC-261`, `STD-001`, `STD-003`,
`ADR-001`.
