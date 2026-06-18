# Implementation Plan SL-096: Knowledge-record relation seam (SPEC-019 FR-005)

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two phases deliver the knowledge-record relation seam: vocabulary + read path
(PHASE-01), then wiring into the graph + render + gate (PHASE-02). The split
mirrors the existing SL-048 precedent (vocabulary ahead of consumers) and keeps
each phase independently testable.

## Sequencing & Rationale

**PHASE-01 builds the pure infrastructure first** — `RelationLabel` variants,
`RELATION_RULES` rows, the `GovernedBy` source-set extension, `KnowledgeRecord.tier1`,
and the `relation_edges` accessor. These are all new code in existing modules
(`relation.rs`, `knowledge.rs`) with no consumer wiring yet. The labels and rules
are tested in isolation (round-trip, lookup, target-kind refusal). The tier1
read path is tested with seeded and authored records.

**PHASE-02 wires the infrastructure into the graph and render surfaces** — the
`outbound_for` dispatch arm replaces the SL-059 stub, `format_show`/`show_json`
render `shapes`/`spawns`/`governed_by` axes, the SL-059 VT-1 test is updated,
and the exact-coverage invariant is extended for the RECORD source group. The
phase ends with `just gate` and the existing-suite behaviour-preservation gate.

**Why not one phase?** The RELATION_RULES changes (golden churn) and the
`KnowledgeRecord` struct change (new field) are each independently risky.
Separating them lets PHASE-01 prove the vocabulary is correct before PHASE-02
attaches consumers. A failed golden update or a misconfigured target set stays
in PHASE-01 rather than cascading into graph failures.

**Why not three or more?** The change is tightly coupled within each phase:
PHASE-01's labels, rules, and read path must all compile together (same crate).
PHASE-02's dispatch, render, and invariant extensions are similarly coupled.
Splitting finer than this would produce phases that don't compile independently.

### What's NOT in scope

- **Template changes** — `[evidence]` ends the file; `append_edge` creates
  `[[relation]]` on first `link`. No template modification needed.
- **Supersession** — FR-006 deferred to IMP-006 / IMP-093.
- **`unlink`** — already shipped (SL-048); no record-specific work.
- **`knowledge list` columns for relation counts** — list doesn't show
  relations (consistent with backlog).

## Notes

- **`dead_code` expect (relation.rs)** — will NARROW but not fully self-clear in
  PHASE-01. `read_record` → `tier1_edges` → `read_block` exercises `from_name`,
  `RELATION_RULES`, `lookup`, `canonical_position`, `IllegalRow`, `IllegalReason`.
  Remaining dead symbols: `validate_link`, `check_target_kind`, `inbound_name`,
  `append_edge`/`remove_edge` (write seam), `writable_labels_for`,
  `owning_verb_for`. The expect comment must be updated to reflect the new state.
  PHASE-02 (`outbound_for` wire-in) plus any `link` verb use will clear the rest.

- **Enum order** — `Shapes` and `Spawns` insert between `Contextualizes` and
  `GovernedBy` (alphabetic per `Ord` derive). The RELATION_RULES rows must
  appear at the same position to keep `enum_ord_matches_relation_rules_label_order`
  green.

- **Tests that MUST be updated in PHASE-01:**
  - `every_variant_appears_in_the_table::ALL` — add Shapes, Spawns.
  - `inbound_name_equals_name_except_the_three_inverted` — add Shapes, Spawns to
    the allowed-inverted list ("shaped_by", "spawned_by").
  - `tier_partition_matches_design::tier_one` — add Shapes, Spawns.
  - `target_spec_matches_design` — add Shapes (Kinds, epistemic-impact set) and
    Spawns (Kinds, backlog items only) assertions.
  - `lookup_keys_on_source_and_label` — extend with Shapes/Spawns lookups.

- **`render_record_toml` round-trip gap** — the test-only renderer does not emit
  `[[relation]]` rows. VT-8 must read `record.tier1` directly rather than round-
  tripping through the renderer. The existing byte-stable round-trip tests
  (`populated_record_round_trips_byte_stable_per_kind`) are unaffected because
  their fixtures carry no `[[relation]]` rows (tier1 is empty on both sides).

- **PHASE-02 exact-coverage** — `reader_emitted_labels_equal_table_labels_per_source`
  needs ASM/DEC/QUE/CON fixtures with one edge per legal axis (shapes, spawns,
  governed_by). `sources_match_shipped_accessors` in relation.rs needs Shapes/Spawns
  entries.

- **SL-059 VT-1 test** becomes `knowledge_kinds_author_outbound_edges` — replace
  the empty-assertion with a fixture that authors `[[relation]]` rows and asserts
  `outbound_for` round-trips them.

- **`from_name` round-trip** is verified by the `debug_assert_eq!(label.name(), name)`
  in `from_name` already — the two new arms in the `match` self-verify. VT-1 is a
  by-product of compilation + debug test runs.
