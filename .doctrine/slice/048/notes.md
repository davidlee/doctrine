# Notes SL-048: Structural cross-corpus relation edges: governance seam + spec-ADR + product-product

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Shipped shape (PHASE-05/06)

- **Write seam** is one generic pair, not per-kind: `relation::append_edge` /
  `remove_edge` (edit-preserving `toml_edit`, idempotent), driven by the `link` /
  `unlink` verbs over `RELATION_RULES` (`validate_link` → `check_target_kind` →
  append/remove). No per-kind write code — the ADR-010 "unified write seam" claim is
  realised by the table + one generic pair, not the per-kind write accessors the ADR
  originally sketched.
- **`validate` gained relation teeth** at the **command layer** (`main::run_validate`
  composes `integrity::id_integrity_findings` + `relation_graph::validate_relations`)
  — deliberately NOT in `integrity.rs`, which would pull `relation_graph` into the
  leaf and cycle (ADR-001). Any future relation check rides `validate_relations`.
- **Three validate checks**, all report-only: `[[relation]]` danglers (Unvalidated
  skipped), `read_block` IllegalRows (hand-edited off-table rows / mis-ordered typed
  table), and the supersession cross-check (stored `superseded_by` via the typed
  `governance::supersession_pair` seam vs the `supersedes` reciprocal).

## Decisions & findings harvested (full record in RV-013)

- **D — EOF/F1 defence is REFUSE, not re-home.** `append_relation_row` refuses a
  hand-edited file whose typed table trails the `[[relation]]` array rather than
  re-homing it. Design permitted either; refuse is the safe default (no silent
  layout surgery on a hand-edited file).
- **F — latent X5 inbound-render gap (PHASE-04), fixed in PHASE-05.** `render_inbound`
  had never moved to the new `inbound_name`, so `governed_by`/`consumes` would render
  their derived inbound backwards. Latent (no edge exercised it; goldens held). Now
  table-driven via `relation::inbound_name`; `inbound_name == name()` for legacy
  labels keeps goldens byte-identical; `--json` keeps the raw label (R2-M3).
- **OD-3 still stands.** Governance `supersedes`/`superseded_by` stay typed until the
  transactional supersede verb ships — owned by **IMP-006**; the read-side guard (the
  cross-check) shipped here and **IMP-032** is reclassified to it.

## Dispatch drive record

P05 driven via `/dispatch` (worker fork `sl048-phase05`, base `89662cb`, delta
`b62d527`); P06 inline authoring (`4ce6d91`). Concurrent SL-056 work committed on
shared `main` throughout — handled by explicit staging, never `git add -A`.
