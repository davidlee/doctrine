# Design SL-046: Cross-kind relation graph spine: all-entity adapter + related/inbound query

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Doctrine projects only the 5 backlog kinds into a graph (`src/backlog_order.rs`),
and produces a single output — an ordering. No command answers *"what is this
entity related to, and what references it?"* across kinds. The governance
`[relationships]` block is parsed but **inert — never queried**. SL-046 builds the
connective **reader**: an all-kind projection into `cordage` and a `doctrine
inspect <ID>` surface showing an entity's authored **outbound** relations and its
**derived inbound** references (ADR-004: reciprocity derived, never stored).

This is the spine of the graph-relations work (SL-046 → SL-047 → SL-048). It is
buildable under current canon (PRD-011 §2 already scopes the graph to all kinds;
REQ-074 owns the inbound view) with no spec revision. It ships **relation
visibility, not prioritisation** — actionability/ranking is SL-047.

## 2. Current State

- **`crates/cordage/`** — the generic graph core (SL-036/038/043), consumed
  unchanged. Relevant API: `GraphBuilder::{node, overlay, edge, order_spec,
  build}`; `Graph::{in_edges, out_edges, reachable, provenance}`;
  `OverlayConfig::new(CyclePolicy, Arity)`; `EdgeAttrs{rank, age}`. `CyclePolicy`
  is `Reject | Evict` only — there is **no tolerate variant**.
- **`src/backlog_order.rs`** — the only adapter. Backlog-specific node key
  `ItemId(ItemKind, u32)`; projects `OrderInput{needs, after, exposure, created}`;
  builds two overlays (`needs` Reject / `after` Evict) + an `OrderSpec`; reads back
  `ordered()` / `dep_cycles()` / `overrides()`. Holds the id↔node bimap
  (`by_item`/`by_node`) inline. Pure — sees only `OrderInput`, never disk.
- **Per-kind relation readers** — each kind owns a **private** `struct
  Relationships` used only on the `show` path: slice `{specs, requirements,
  supersedes}` (`src/slice.rs`); governance `{supersedes, superseded_by, related,
  tags}` (`src/governance.rs`, shared by ADR/POL/STD); backlog `{slices, specs,
  drift, needs, after, triggers}` (`src/backlog.rs`). Spec lineage is **not** in a
  `[relationships]` table — `descends_from`/`parent` are `Option<String>` on the
  spec `Meta`, and `members[].requirement` lives in a separate `members.toml`
  (`src/spec.rs`).
- **`src/integrity.rs`** — `KINDS`, the corpus-wide id table (`entity::Kind` +
  stem per numbered kind); the single source for the all-kind scan and prefix→kind
  resolution.

## 3. Forces & Constraints

- **ADR-004** — relations are stored outbound-only; reciprocity is derived. No new
  stored reverse field anywhere (REQ-074 / REQ-078).
- **ADR-001** — module layering leaf ← engine ← command, no cycles.
- **SPEC-001 D1** — `cordage` is product-neutral and **locked**; consumed as-is, no
  doctrine vocabulary added to the crate (REQ-079).
- **Behaviour-preservation gate** — `backlog order` output byte-identical; existing
  `backlog_order`/`cordage` suites stay green **unchanged**.
- **Determinism** (REQ-077) — no clock / RNG / `HashMap` iteration order; repo bans
  `HashSet`/`HashMap` (use BTree).
- **Tolerate dangling/free-text refs** — `drift` and governance `related` are not
  forward-validated (mem.pattern.entity.free-text-ref-not-forward-validated); map
  to a node only when the target resolves, else surface a dangler — never panic.
- **No parallel implementation** — ride the existing bimap/projection seam; the
  corpus walk must skip the `NNN-slug` symlink alias
  (mem.pattern.entity.corpus-walk-skip-slug-symlink).
- **kind-is-data, not a trait** (mem.pattern.entity.kind-is-data-not-trait) — the
  cross-kind dispatch is a data-driven match over `entity::Kind`, not a per-kind
  trait.

## 4. Guiding Principles

- The **reader comes first**; capture (new authored edges) is SL-048. Building a
  reader over the already-authored relations is what makes SL-048's future edges
  land live rather than inert.
- **Outbound is canonical; inbound is derived.** The reader projects only the
  authored outbound direction and derives every reciprocal from `in_edges`.
- **cordage stays generic; doctrine vocabulary stays in the adapter.** The relation
  *kind* is encoded as overlay identity, never leaked into the core.
- **Tolerate, don't trust.** Free-text and dangling targets are surfaced, never
  fatal.

## 5. Proposed Design

### 5.1 System Model

Three new pieces across the ADR-001 layers, plus a thin accessor per kind:

```
command:  src/main.rs              `inspect` handler — id-parse, call adapter, render
engine:   src/relation_graph.rs    all-kind scan → Projection + ref overlays → query
          src/{slice,spec,governance,backlog}.rs   + pub(crate) relation_edges accessor
leaf:     src/projection.rs        Projection<K> — bimap / intern / resolve / remap
          crates/cordage/          consumed, unchanged
```

`relation_graph` (engine) → `projection` (leaf) → `cordage`. `backlog_order`
(engine) also rides `projection`. No cycle. `relation_graph` builds a **separate**
`Graph` from `backlog_order` — they share the projection *type*, never a graph
instance or a scan.

### 5.2 Interfaces & Contracts

**The shared projection primitive (leaf, OQ3-B).** Generic over a `Copy + Ord`
key; *passive* — `intern` mints `NodeId`s in caller call-order (NodeId allocation
order is behaviour-relevant for `backlog_order`'s tie-break, so the primitive must
never impose its own order):

```rust
pub(crate) struct Projection<K: Copy + Ord> {
    by_key:  BTreeMap<K, NodeId>,
    by_node: BTreeMap<NodeId, K>,
}
impl<K: Copy + Ord> Projection<K> {
    fn intern(&mut self, builder: &mut GraphBuilder, key: K) -> NodeId; // mint-or-get
    fn resolve(&self, key: K) -> Option<NodeId>;                        // get-only
    fn key_of(&self, node: NodeId) -> Option<K>;
    fn remap_set(&self, nodes: &BTreeSet<NodeId>) -> BTreeSet<K>;
}
```

`backlog_order` rides `Projection<ItemId>` (its scan + overlays + `OrderSpec`
otherwise unchanged); `relation_graph` rides `Projection<EntityKey>`, the all-kind
analog `EntityKey { kind: entity::Kind, id: u32 }` (renders the canonical ref via
the same `KINDS` source `ItemId` uses).

**The extraction seam (engine).** A uniform triple, a per-kind accessor reading
each kind's own private `Relationships`, and a data-driven dispatch — no trait:

```rust
pub(crate) struct RelationEdge { label: RelationLabel, target: String } // target = canonical ref

// each module exposes, reading its OWN Relationships (parsing stays put — cohesion):
//   slice::relation_edges(id)      -> specs / requirements / supersedes
//   spec::relation_edges(id)       -> descends_from / parent / members
//   governance::relation_edges(id) -> supersedes / related      (NOT superseded_by, NOT tags)
//   backlog::relation_edges(id)    -> slices / specs / drift     (NOT needs/after/triggers)

fn outbound_for(kind: entity::Kind, id: u32) -> Vec<RelationEdge>      // one match over KINDS
```

**The query (engine → command).**

```rust
pub(crate) struct InspectView {
    id: String,
    outbound: Vec<(RelationLabel, Vec<String>)>,   // authored, grouped by label
    inbound:  Vec<(RelationLabel, Vec<String>)>,   // derived via in_edges, grouped by label
    danglers: Vec<(RelationLabel, String)>,        // unresolved / free-text outbound targets
}
fn inspect(id: &str) -> Result<InspectView>;
```

### 5.3 Data, State & Ownership

- **Adapter owns**: the all-kind scan, `Projection<EntityKey>`, edge emission, the
  `OverlayId → RelationLabel` map (~9 entries), dangler collection, diagnostic
  re-mapping (D1's adapter responsibilities).
- **Each kind's module owns**: parsing its own relations into `RelationEdge`s
  (cohesion — the adapter never re-parses TOML).
- **cordage owns**: nodes, overlays, edges, the reverse index, cycle resolution.
- **Nothing owns a stored reverse field** — inbound is recomputed every query from
  `in_edges` (ADR-004 / REQ-074 / REQ-078).

**Overlay set** — keyed by relation **label** (label = overlay identity, OQ2-B);
the same label from different source kinds shares one overlay. All
`Reject`/`Unbounded`:

| Overlay | Edges (src → dst) |
|---|---|
| `specs` | slice, backlog → spec |
| `requirements` | slice → req |
| `supersedes` | slice→slice, governance→governance |
| `descends_from` | spec → prd |
| `parent` | spec → spec |
| `members` | spec → req |
| `slices` | backlog → slice |
| `drift` | backlog → drift |
| `related` | governance → governance |

**Not** overlays: `superseded_by` (derived = inbound of `supersedes`, the IMP-032
reader rule), `tags` (free-text, not entity refs), `needs`/`after`/`triggers`
(dep/seq/mask — SL-047).

### 5.4 Lifecycle, Operations & Dynamics

1. `inspect <ID>`: parse prefix → `entity::Kind` via `KINDS` (error on unknown).
2. Build the relation graph once: walk `KINDS` in table order; for each kind walk
   its entity dirs (numeric id order, **skipping the `NNN-slug` symlink**); `intern`
   each entity into `Projection<EntityKey>`.
3. Second pass: for each entity, `outbound_for(kind, id)` → for each `RelationEdge`,
   `projection.resolve(target)` → resolved ⇒ `builder.edge(overlay_for(label), src,
   dst, EdgeAttrs::new(0,0))`; unresolved/free-text ⇒ push dangler.
4. `builder.build()` (no `OrderSpec` over reference overlays → no composition, no
   union-cycle pass touches them).
5. Query node = `projection.resolve(EntityKey of ID)`. Outbound = the entity's own
   `RelationEdge`s grouped by label. Inbound = for each overlay,
   `graph.in_edges(ov, node)` → `key_of(src)`, grouped by `label_of(ov)`. The
   `supersedes`-overlay inbound renders as "superseded by Y" — the derived
   reciprocal (no `superseded_by` field read).
6. Render outbound / inbound / danglers in a fixed deterministic order.

### 5.5 Invariants, Assumptions & Edge Cases

- **I1 — no edge is ever lost.** Reference overlays are `Reject`+`Unbounded`:
  `Reject` removes no edges (resolve.rs — *"Resolved set unchanged; the cycle is
  preserved"*); `Unbounded` exempts them from arity eviction (pass1 touches only
  `AtMostOne`). So `in_edges` enumerates **every** authored inbound edge,
  cycle or not. `Evict` is forbidden here — it removes edges and would silently
  drop a real relation. **Asserted/tested at the projection boundary.**
- **I2 — direct-only is composition-free.** `inspect` does one-hop `in_edges` per
  overlay; it never composes overlays or walks `reachable`. So **no acyclicity is
  assumed of any overlay or of their union** — a cross-overlay cycle is invisible
  and harmless to a one-hop lookup. (Union-acyclicity becomes SL-047's concern when
  it builds an `OrderSpec` / walks `reachable`.)
- **I3 — acyclicity assumptions, per kind**: single-direction cross-kind refs
  (`specs`/`requirements`/`slices`/`drift`/`members`) are **structurally acyclic**
  (disjoint source/target kinds) — `Reject` never fires. Lineage
  (`supersedes`/`descends_from`/`parent`) is **intent-acyclic, unvalidated** — a
  loop is diagnosed and degrades safely (I1 still holds). `related` **can cycle
  benignly** (symmetric) — the diagnostic is suppressed for `inspect` (a
  `validate`/SL-048 concern); `in_edges` stays complete.
- **I4 — `supersedes`/`superseded_by` kept distinct conceptually but only
  `supersedes` is projected** — so the reciprocal pair never forms a false 2-cycle,
  and the inbound is the single derived truth (IMP-032).
- **Edge cases**: unknown prefix → clean error; entity with no relations → empty
  outbound/inbound (not an error); dangling/free-text target → dangler, never a
  panic; a node referenced by nothing → empty inbound.

## 6. Open Questions & Unknowns

All three scoped OQs are **resolved** (see §7 D1–D3). Remaining unknowns are
downstream, not blocking:

- The plan/audit file-set sources for triggers (SPEC-001 D6) — SL-047+, unrelated.
- Whether `inspect` should later surface reference-overlay cycle *diagnostics* —
  deferred to a `validate` integration (SL-048).
- Whether **all** relations modelling should be refactored to a uniform schema —
  filed as a direct follow-up to interrogate (IMP-034), parallelisable with or
  after this slice; see §7 D4 note and §10.

## 7. Decisions, Rationale & Alternatives

- **D1 — CLI surface: dedicated `inspect <ID>`, relation-only, direct-only.**
  SPEC-001 already reserves `inspect` as the registry-backed inbound surface
  (FR-005/REQ-074). SL-046 ships it relation-only; SL-047 layers
  actionability/blockers onto the **same verb** (additive evolution; no parallel
  `related`+`inspect` split). Direct-only — no `--transitive`; chain-walking lands
  with SL-047's `explain`/`blockers` where ordering gives a reason to need it.
  *Alts rejected*: enrich `<kind> show` (couples the cross-kind view into every
  per-kind renderer; no single SL-047 home); a separate `related` verb (two verbs
  answering overlapping questions).
- **D2 — overlay typing: one `Reject`/`Unbounded` overlay per relation label
  (label = overlay identity).** `EdgeAttrs` is `{rank, age}` only — it cannot carry
  a relation-kind label — so the kind must live in overlay identity (a tiny
  `OverlayId → Label` map) rather than a per-edge sidecar. cordage's overlay **is**
  the edge-type mechanism; using it as designed is not a workaround for the
  no-edit-cordage constraint — editing cordage to add a generic edge label would be
  redundant with overlay identity and breach D1 neutrality. *Alt rejected*: single
  `ref` overlay + an `(src,dst) → Kind` sidecar — duplicates cordage's
  overlay-as-edge-type in a parallel per-edge structure that can drift (the
  parallel-implementation smell).
- **D3 — adapter structure: extract a generic `Projection<K>`; two thin adapters
  ride it.** The genuinely shared machinery is the id↔node bimap + intern/resolve/
  remap (D1's adapter component). `backlog_order` (`ItemId`) and `relation_graph`
  (`EntityKey`) each keep their own scan/overlays/output; only the bimap is shared.
  Backlog behaviour is preserved by construction (its scan + overlays + `OrderSpec`
  untouched; the existing green suite is the proof). *Alts rejected*: generalise
  `backlog_order` in place (forces every kind through backlog-shaped `OrderInput`;
  edits `build()` at the gate); standalone with no sharing (duplicates the
  bimap/dangler/remap — the "no parallel implementation" line, ×4 future consumers).
- **D4 — reader rule: project canonical outbound only; derive reciprocals.**
  Governance stores `superseded_by`, a stored reciprocal of `supersedes` — the
  ADR-004 violation this work exists to remove. The reader projects only
  `supersedes` and derives the reciprocal from `in_edges`; it does **not** project
  `superseded_by` (projecting both double-counts). The stored field's removal +
  migration is capture-side → **SL-048** (filed IMP-032). `related` is genuinely
  symmetric and legitimately appears on both sides — not redundant. *Note*: this
  surfaced a broader question — whether the per-kind relation *modelling* itself
  (three private `Relationships` structs, spec lineage on `Meta` vs a table, mixed
  cardinality) should be unified — interrogated as a direct follow-up (IMP-034),
  out of scope for this reader.

## 8. Risks & Mitigations

- **R1 — `backlog order` regression from the `Projection` swap.** *Mitigation*: the
  primitive is passive (caller-controlled mint order); the existing byte-exact
  `backlog order` golden + `backlog_order` unit suite stay green unchanged (the
  gate). If any drift appears, the swap is wrong.
- **R2 — duplicate node key corrupts the cordage bimap (RSK-005).** *Mitigation*:
  canonical ids are globally unique by prefix; `intern` is mint-or-get (idempotent
  per key); assert distinctness at the projection boundary.
- **R3 — a lone `superseded_by` with no reciprocal `supersedes` is dropped by the
  reader.** *Mitigation*: accepted for v1 (the canonical direction is
  authoritative); the stored-vs-derived gap is exactly what IMP-032/SL-048
  reconciles. A `validate` cross-check is a follow-up, not SL-046.
- **R4 — free-text/dangling refs panic the scan.** *Mitigation*: `resolve` returns
  `Option`; unresolved ⇒ dangler; corpus walk skips `NNN-slug` symlinks.

## 9. Quality Engineering & Validation

- **Outbound correctness** — per kind, `outbound_for` returns the authored relations
  with correct labels (incl. spec `Option` singles + `members.toml`).
- **Derived inbound correctness** — over a seeded multi-kind corpus, `inspect`
  reports correct inbound per kind, including the `supersedes` reciprocal rendered
  as "superseded by"; structural proof **no stored reverse field is read**
  (ADR-004/REQ-074).
- **Dangler tolerance** — free-text/dangling targets surface as danglers, no panic.
- **Determinism** — identical output under input permutation (REQ-077).
- **Behaviour-preservation gate** — `backlog order` byte-identical; `backlog_order`
  + `cordage` suites green unchanged.
- **`Projection<K>`** — unit-tested over both `ItemId` and `EntityKey`
  (mint-or-get idempotence; caller-order mint; remap round-trip).
- **Core neutrality** — `cordage` test suite carries no doctrine vocabulary
  (REQ-079) — unchanged, since the crate is untouched.

## 10. Review Notes

(Adversarial pass + integration recorded here.)

**Direct follow-up to interrogate (IMP-034):** whether to refactor *all* relations
modelling to a uniform schema — a single generic `[[relation]] kind=… target=…`
surface across every kind vs today's bespoke per-kind typed fields. Surfaced by D4;
adjacent to IMP-006 (uniform verbs) and IMP-016 (cross-corpus links). Should be run
in parallel with, or as a direct successor to, this slice — and likely feeds the
relation-governance ADR that SL-048 needs.
