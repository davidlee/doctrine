# REV REV-003 — reconcile SL-103

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

Reconcile of SL-103 (RV-100). Two governance writes against SPEC-020's authored
truth; both findings `verified`, remediation is reconcile's to land (audit changes
no spec).

## Reconcile narrative (SL-103)

- **[RV-100 F-1] — introduce FR-010 (value graph exposure).** SL-103 ships the
  value facet onto the catalog/graph contract symmetrically with estimate (design
  D1/§7), but no requirement homed that exposure: FR-006/REQ-274 names estimate
  only, FR-007..009 (REQ-278/279/280) govern the value *model/validation/unit*, not
  its graph projection. Mint FR-010 as the value-facet sibling to FR-006, homed in
  SPEC-020 (which already owns both facets — the audit's "rename SPEC-020 / mint a
  sibling spec" fork was premised on a stale title; no rename needed). Then bind
  SL-103 → the new REQ. Statement: "Expose each node's value magnitude and project
  value unit through the same policy-free catalog/graph contract as the estimate
  facet."

- **[RV-100 F-3] — modify REQ-274 (ratify "project unit" by reachability).**
  REQ-274 acceptance enumerates "the project unit" among per-estimated-node fields.
  D2 carries units as ONE top-level `units{estimation,value}` block, not per-node
  duplication — units are project-wide constants reachable from every node via the
  graph the consumer holds. Ratify that interpretation durably in the requirement
  text so "per node" reads as reachability, not literal placement.

  - *before* (acceptance[0]): "…lower, upper, **the project unit**, the node's
    relations/edges, and lifecycle state where available."
  - *after*: "…lower, upper, **the project unit (a project-wide constant reachable
    from every node via the held graph, not duplicated per node)**, the node's
    relations/edges, and lifecycle state where available."

- **[RV-100 F-2]** — SL-103 → REQ-280 trace edge (`scan_catalog`/`resolve_units`
  realises REQ-280, §5.4). Landed directly via `link` (a slice's own relation,
  not a spec-truth change); recorded here for the audit trail, no `[[change]]` row.
