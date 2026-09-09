# REV REV-055 — reconcile SL-256

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

<!-- Why this revision: what authored truth needs to change and why, the scope of
     the staged delta, and (for ADR/POL/STD/prose rows) the before/after excerpts
     the structured payload only labels. Seeded at `revision new`. -->

One row, and it is a lifecycle status field nobody moved when the evidence
landed. `SL-256` implemented `REQ-478` (SPEC-029 `FR-009`, *report every recorded
mutation on the change log*) across three phases; the requirement is still
`pending`.

## Reconcile narrative (SL-256)

- **`RV-364` `F-1` — `REQ-478` `pending` → `active`.** All three of the
  requirement's acceptance criteria are discharged by named checks, and the
  coverage cell verifies against a runnable positive control rather than a
  backfilled attestation:

  | criterion | evidence |
  |---|---|
  | an apply that records an act emits a row identifying the act's subject and kind | `tests/e2e_design_state.rs:1243`, `:1265`, `:1293`, `:1328` — one check per recording path (bare `agent_declaration`, `checkpoint_act` without a disposition, a disposing act, the run-level `acceptance`), each asserting event, subject *and* ordered terms |
  | every emittable member is driven by the fixture ladder; an undriven member fails | `:1090`, iterating `EMITTABLE` |
  | retired vocabulary is readable-only and carries no emission obligation | `:1119`, plus the compile-time `EMITTABLE ⊆ READABLE` assert in `change_log.rs` |

  The coverage cell (`.doctrine/slice/256/coverage.toml`, mode `VT`, anchored at
  `c0c69f634`) binds `cargo test --test e2e_design_state` with a
  `--matcher-pattern` naming one new test's own pass line. That is what makes it
  evidence: a pass-count pattern would also match a run in which the new checks
  were never compiled, whereas this one fails unless `ISS-355`'s own case ran and
  passed. `doctrine coverage verify 256` is green.

  The status field was the only governance item in `RV-364`'s reconciliation
  brief. Every other finding landed on a per-slice surface by direct edit, was
  tolerated with rationale on the ledger, or already has an owner outside this
  slice (`IMP-445`, `IMP-437`, `IMP-282`, `ISS-315`).
