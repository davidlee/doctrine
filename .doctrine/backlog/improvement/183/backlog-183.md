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

## Kinds audit (2026-07-03)

| Kind | `show` surfaces? | `estimate`/`value` fields in serde struct? |
|---|---|---|
| **Slice** | Yes | `SliceDoc` has `estimate: Option<EstimateFacet>`, `value: Option<ValueFacet>` |
| **Backlog** (ISS/IMP/CHR/RSK/IDE) | No | `Doc` has `facet: Option<RiskFacet>` only; `[estimate]`/`[value]` TOML sections ignored by serde |
| **Knowledge** (ASM/OBS/etc.) | No | `RawRecordToml` has `deny_unknown_fields = false` → TOML parses but fields dropped |
| **ADR / Policy / Standard** | ✅ Now surfaced (IMP-183) | `Doc` (governance.rs) has `estimate`/`value` with lenient deser |
| **Spec** | ✅ Now surfaced (IMP-183) | `Spec` has `estimate`/`value` with lenient deser |
| **Revision** | ✅ Now surfaced (IMP-183) | `RevDoc` has `estimate`/`value` with lenient deser |
| **Review** | ✅ Now surfaced (IMP-183) | `ReviewDoc` has `estimate`/`value` with lenient deser |
| **REC** | ✅ Now surfaced (IMP-183) | `RecDoc` has `estimate`/`value` with lenient deser |
| **Concept-map** | ✅ Now surfaced (IMP-183) | `ConceptMapDoc` has `estimate`/`value` with lenient deser |
| **Memory** | ✅ Now surfaced (IMP-183) | `Memory` has `estimate`/`value` parsed from `RawMemoryToml.extra` |

The facet commands (`estimate set`/`value set`) are kind-agnostic — they write via
`entity::id_path` to any kind's TOML. But only `SliceDoc` currently deserialises
the `[estimate]`/`[value]` sections into its `show` output.

## Notes

- Distinct from SL-158's gating scope; SL-158 only adds a confirmatory design note
  that estimate/value are admissible on records.
- Watch the `[facet]` table-name collision: risk's `[facet]` ≠ knowledge's typed
  `[facet]`. Don't conflate them when surfacing.
- IMP-246 (`backlog list` columns + JSON) depends on this item.
