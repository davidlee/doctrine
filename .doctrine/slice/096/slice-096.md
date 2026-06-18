# Knowledge-record relation seam (SPEC-019 FR-005)

## Context

**SL-059** shipped Slice A — the standalone knowledge-record entity surface:
four `RecordKind`s on the engine with per-kind lifecycles, typed facets,
evidence, prefix→kind resolution, priority partition entries, and the
`doctrine knowledge new/show/list/status` CLI (`knowledge.rs` is fully live).

**This slice (Slice B)** delivers SPEC-019 **FR-005**: the outbound relation and
spawn-work seam over the cross-corpus relation contract (**SPEC-018**,
**ADR-010**). Record `RELATION_RULES` rows, two minted `RelationLabel` variants,
the `outbound_for` dispatch arm, and the `show`/`inspect` render of relation edges.

The core semantic design decision amends SPEC-019 D6: source-set extensions
are rejected for `specs`/`slices`/`requirements` because the inbound rendering
would lie. Instead, one epistemic label `shapes` / `shaped_by` covers all
record→artefact influence.

Supersession (FR-006) is deferred — gated on the unbuilt IMP-006 transactional
verb. IMP-006 and the FR-006 follow-up are backlogged with a priority boost.

## Scope & Objectives

1. **Two new `RelationLabel` variants** — `Shapes` (record → epistemic-impact
   set) and `Spawns` (record → backlog items). Source-set extensions for
   `GovernedBy` and `Drift` only (inbound names are semantically neutral).

2. **`RELATION_RULES` rows** for the RECORD source group, plus the source-set
   extensions.

3. **`KnowledgeRecord` gains `tier1` edges** — read in `read_record` via
   `tier1_edges`, rendered in `format_show` / `show_json`, returned by the
   `relation_edges` accessor.

4. **`outbound_for` dispatch arm** — replaced the empty SL-059 stub with a
   real `knowledge::relation_edges` call.

5. **`Shapes` target: explicit epistemic-impact set** — PRD, SPEC, REQ, SLICE,
   ISS/IMP/CHR/RSK/IDE, ADR/POL/STD, ASM/DEC/QUE/CON. Excludes REV, REC, RV, CM.

6. **Record↔record relations** use `shapes` — not separate `informs`/`bears_on`
   labels (IMP-053 collapsed).

## Non-Goals

- **Supersession (FR-006)** — deferred; `Supersedes` RECORD `LifecycleOnly`
  rule row, cross-kind matrix, and transactional verb gated on IMP-006.
- **IMP-047** — trinary actionability (unchanged).
- **IMP-053** — separate record↔record labels (collapsed into `shapes`).
- **No template changes** — `[evidence]` ends the file; `append_edge` creates
  `[[relation]]` on first `link`.

## Affected Surface

- `src/relation.rs` — 2 new `RelationLabel` variants; `name()`/`from_name()`
  arms; 2 new `RELATION_RULES` rows; extend `GovernedBy`/`Drift` sources;
  local kind aliases; RECORD source-group const
- `src/knowledge.rs` — `tier1` field on `KnowledgeRecord`; read in
  `read_record`; new `relation_edges` accessor; extend `format_show`/`show_json`
- `src/catalog/scan.rs` — swap `outbound_for` empty arm for real dispatch
- `src/relation_graph.rs` (tests) — update SL-059 VT-1; exact-coverage invariant
- `src/relation.rs` (tests) — golden churn for new labels

## Risks & Assumptions

- **R1 — distinct_labels golden churn.** Two new labels change declaration order. One-shot update.
- **R2 — behaviour preservation.** Shared entity engine and existing per-kind suites stay green.
- **A1:** SL-059 is shipped and green. No records exist yet.
- **A2:** `append_edge` creates `[[relation]]` array on first use; extant templates need no change.

## Verification / Closure Intent

- `doctrine link ASM-001 shapes SPEC-004` succeeds
- `doctrine link DEC-001 spawns ISS-001` succeeds
- `doctrine link ASM-001 shapes RV-001` refused (not in target set)
- `doctrine link ASM-001 spawns SL-001` refused (spawns targets only backlog)
- `doctrine inspect SPEC-004` shows `shaped_by: [ASM-001]`
- `doctrine inspect ISS-001` shows `spawned_by: [DEC-001]`
- `doctrine link ASM-001 governed_by ADR-004` succeeds
- `doctrine inspect ADR-004` shows `governs: [ASM-001]` (proving neutral inbound)
- Exact-coverage invariant extended and green
- Existing suites green; `just gate` clean
- SPEC-019 FR-005 status → `done`

## Summary

The knowledge-record relation seam delivers SPEC-019 FR-005: two new labels
(`shapes`/`spawns`) on the cross-corpus relation contract, source-set
extension for `governed_by`, tier1 edge read-and-render in
`knowledge show`, and the live `outbound_for` dispatch arm — all riding the
already-shipped `link`/`unlink` verb.

## Follow-Ups

- **FR-006 / IMP-006** — supersession verb + RECORD `Supersedes` rule row
- **IMP-047** — trinary actionability / record gating
- **IMP-083** — knowledge-record memory signpost
- **IDE-006** — constraint facet extensions
