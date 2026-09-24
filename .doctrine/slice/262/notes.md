# Notes SL-262: Envelope names the next move

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-24 · plan + PHASE-01 · 25c23a2d0

### Produced
- design locked (run rev 38): sec-1..9 in design.md (commits d1f5edd03, f8a13cd41); no code yet
- RV-382 — codex design pass, F-1..F-5 fixed + verified
- DEC-290..DEC-294 — forward edge, DEC-124 narrowing + envelope compat rule, GateFacts, placement/bounds, skipped-check disclosure
- close-time bookkeeping owed (design sec-6): IMP-390 close, IMP-367 disposition note, IMP-372 override-bound note
- reconcile-time REV owed (design sec-6): PRD-019 REQ-414, SPEC-029 REQ-437 + responsibilities
- PHASE-01 refactor (25c23a2d0): `GateFacts` + `commands::design::gate_facts` (the only constructor); `gate::forward_unmet` extracted from `advance`; `AuthoredState`/`observe_watermark`/`divergence_refusal` moved to `design_run::document`. No envelope change; all suites green with no edits under `tests/`.

### Learned
- mem_01a0d305ce2b76c1b18f04abd572cbb8 — inquiry disposal goes through a cp- checkpoint
- observations: research baseline drifts on its own scope delta; inert-key discovery for inquiry disposal
- **`&derived` → `&derived.gate` churn is in `design_run` tests' `advance(...)` calls (8), not the `DerivedInput` literals (5).** `#[derive(Default)]` on `GateFacts` absorbs every `..DerivedInput::default()` site; keeping `causes_of`/`assert_holds` on `&DerivedInput` and passing `&derived.gate` inside spared ~18 call sites.
- clippy `shadow_unrelated` is denied for the bin: a new `let declared` in `apply` collided with the pre-existing `|declared|` closure in `declaration_fingerprint`.

### Open
- PHASE-02 next: derive `Forward` from snapshot + `GateFacts`, render on JSON/prompt/status/resume, delete `next_obligation`, envelope → v2, regenerate the show golden. `envelope::project`/`assemble` gain `&GateFacts` + `slice_ref`; `envelope_turn`/`run_resume` build facts via `gate_facts(…, None)`.
- residual audit probes named in "Further review passes" below still stand

## Design surface triage (2026-09-24, design run exploring)

Source: `research/research.md` (✓ rows) plus these checks at the cited sites.
Superseded by the locked design and DEC-290..DEC-294 — kept as the exploring-stage record only.

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

## Further review passes (2026-09-24, after RV-382 concluded)

RV-382 (codex gpt-6-sol, adversarial) raised F-1..F-5; all fixed and verified.
A further pass is not needed before lock. If one were run it would probe:
the `forward_unmet`/`advance` equivalence under batched submissions (acts
recorded in the same payload as the stage move), and the cause-cap
constant's derivation once the growing-run fixture measures real row sizes.
Both are better checked against code at audit than re-argued on the design.
