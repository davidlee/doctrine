# SL-047 implementation notes

Durable cross-phase notes. Runtime progress lives in the gitignored phase sheets;
this file carries the API/seam contract each phase hands the next.

## PHASE-01 — priority graph adapter + scan seam (landed `cb03be6`)

**The D5 scan seam (`src/relation_graph.rs`).** Extracted the reusable half of the
old private `build_relation_graph` as a `pub(crate)` seam:

- `relation_graph::scan_entities(root) -> Vec<ScannedEntity{ key, kind, status, outbound }>`
  — the all-kind KINDS-walk (table order, ids sorted asc), one row per entity.
- `status_for` — per-prefix dispatch (mirrors `outbound_for`) reading each kind's
  AUTHORED status: REC → `None`; RV → `review::derived_status_string` over its finding
  ledger; all others → `meta::read_meta`.
- `inspect`'s `build_relation_graph` is now re-expressed on `scan_entities` — mints in
  the same scan order → **byte-identical** output (behaviour-preservation gate held).
- `EntityKey` promoted to `pub(crate)` so `priority` reuses it (no parallel key type).

**The third graph (`src/priority/graph.rs`).** `pub(crate) fn build() -> Result<PriorityGraph>`:
scan → consequence pre-pass (inbound tally over the work/lineage label SUBSET only:
`specs/requirements/slices/descends_from/parent/members`; `reviews`/`owning_slice`
EXCLUDED) → mint in `(consequence desc, canonical-id asc)` with the `backlog_order` C4
pre-intern pass + distinct-key assert → edges (reference/lineage → `ref_overlays`
resolve-only/dangler; `needs` → `dep_overlay` Reject B→A `(0,0)`; `after` →
`seq_overlay` Evict `(rank,age)`, kind-agnostic per DD-2) → `OrderSpec[dep Along,
seq Along]` → cordage `build()`.

`PriorityGraph` fields (what PHASE-02 reads): `graph, projection, attrs:
BTreeMap<EntityKey,NodeAttr>, consequence: BTreeMap<EntityKey,u32>, dep_overlay,
seq_overlay, ref_overlays: Vec<OverlayId>, dangling`. `NodeAttr{ kind:
&'static entity::Kind, status: Option<String>, promoted: bool }` — `status` is the
RAW authored string (NO workable/terminal classification yet — that is PHASE-02).
`promoted` from the typed `Resolution::Promoted` via new `backlog::dep_seq_for` /
`DepSeq` (yields needs/after + promoted, read once per backlog item).

**Thin accessors added:** `review::derived_status_string`, `backlog::dep_seq_for`
(+`DepSeq`).

### ⚠ Flag for PHASE-02 (carried into its worker prompt)

The priority adapter has **no live CLI consumer yet**, so `graph.rs` rides a
`not(test)`-scoped self-clearing `#[expect(dead_code)]` (module-level; item-level on
`ScannedEntity.kind`/`.status`) per `mem.pattern.lint.dead-code-expect-vs-cfg-test`.
PHASE-02/03 wiring a real caller must **remove** these suppressions (the `expect`
fails if the lint stops firing). `backlog::dep_seq_for`/`DepSeq` are reachable from
live code only via priority but are test-exercised; keep a live caller or they go dead.
