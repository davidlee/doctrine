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

- The `dead_code` expect at the module level of `relation.rs` (SL-048 PHASE-02)
  may already have self-cleared; verify before adding new expects. New
  `Shapes`/`Spawns` labels are wired to consumers in PHASE-02, so no new
  dead-code suppression should be needed.
- The `distinct_labels` golden in `relation.rs` tests is ordered by
  `RelationLabel` enum discriminant. `Shapes` and `Spawns` insert between
  `Contextualizes` and `GovernedBy` (alphabetic per the `Ord` derive).
  Update the golden to reflect the new order.
- The SL-059 VT-1 test `knowledge_kinds_author_no_outbound_never_panic`
  asserts `edges.is_empty()` — this becomes false once records author edges.
  Replace with a fixture test that authors `[[relation]]` rows and asserts
  `outbound_for` returns the expected edges.
