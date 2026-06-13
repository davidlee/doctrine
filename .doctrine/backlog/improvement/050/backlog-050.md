# IMP-050: Scope+ship SPEC-019 FR-005: knowledge-record relation seam (RECORD source-group, minted relate/spawns labels, outbound_for arm, record reader)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

**Slice B of the SPEC-019 cut** (FR-005 / REQ-064). Follows SL-059 (Slice A, the
standalone surface). Scope when triaged:

- A `RECORD` source-group joining the existing labels with matching target sets:
  `specs`/`slices`/`requirements`/`drift`/`governed_by` (record→ADR rides
  `governed_by`; no v1 peer-relate — SPEC-019 D6).
- **Two minted `RelationLabel` variants** (reuse foreclosed by the table): a
  record→backlog-item *relate* label (record→risk is this label aimed at `RSK`)
  and a `spawns` origin label — each with wire name, `inbound_name`, rule row.
- An `outbound_for` dispatch arm + a record `relation_edges` reader; extend the
  exact-coverage invariant (`sources_match_shipped_accessors`) and
  `every_variant_appears_in_the_table`.

Rides the shipped `link`/`unlink` writer (IMP-048 done) — **label-design-blocked,
not verb-blocked**. **Coordinates with SL-058** (relation surface tooling) and
IMP-016/IMP-035 on the shared `RELATION_RULES`/`outbound_for` sites — sequence,
do not collide. Authoring slices are not dispatchable.
