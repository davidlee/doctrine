# IMP-183: Surface estimate/value facets in show and inspect for all estimable kinds

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

Surfaced during SL-158 (`/design`, 2026-06-26). The `estimate` / `value` facet
commands are kind-agnostic (`src/estimate.rs`: *"facet is kind-agnostic"*; no kind
gate in `src/commands/facet.rs::run_estimate_set` / `run_value_set`), and
`entity::id_path` resolves every kind's path — including knowledge records
(`.doctrine/knowledge/<kind>/NNN/record-NNN.toml`). So `estimate set ASM-001 …`
writes a `[estimate]` table and round-trips cleanly (`RawRecordToml` has no
`deny_unknown_fields` — the table is ignored by the knowledge reader, not rejected).
**Confirmed working in the CLI** (user-exercised).

## The gap

The written facet is **not surfaced** by `doctrine <kind> show` (nor, for the
knowledge kinds, by `knowledge inspect`). It is authored and consumed by the
priority engine (feeds `base_score`) but invisible to the human read surface.

## Ask

Ensure every **estimable** kind surfaces its `estimate` / `value` facets in `show`
— preferably in `inspect` too. Audit which kinds the facet commands accept
(estimate/value are gate-free; `risk` is gated to risk-items and additionally
collides on the `[facet]` table name with knowledge records' typed kind-facet) and
make the read surface match the write surface.

## Notes

- Distinct from SL-158's gating scope; SL-158 only adds a confirmatory design note
  that estimate/value are admissible on records.
- Watch the `[facet]` table-name collision: risk's `[facet]` ≠ knowledge's typed
  `[facet]`. Don't conflate them when surfacing.
