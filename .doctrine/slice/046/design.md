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
  spec `Meta`, `members[].requirement` lives in `members.toml`, and tech-spec
  `interactions.toml` holds typed spec→spec `[[edge]]`s (`src/spec.rs`). The two
  ledger kinds also author outbound edges: RV's `[target].ref` (`src/review.rs`)
  and REC's `owning_slice`/`decision_ref` (`src/rec.rs`). All **11** `KINDS` rows
  are edge sources — the adapter must reach every one.
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
          src/{slice,spec,governance,backlog,review,rec}.rs  + pub(crate) relation_edges accessor
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
never impose its own order). **Mint-sequence contract (C4, the gate's contract):**
`backlog_order` keeps its existing three-step shape — (1) sort projected inputs
(`backlog_order.rs:184-190`), (2) **pre-intern every input in that sorted order in
a dedicated pass** (replacing the `builder.node()` loop at `:194-198`), (3) only
then emit edges with `resolve` (never `intern`). Folding `intern` into the
edge-emission loop would mint in dependency-reference order — a tie-break
regression. The byte-exact `backlog order` golden is the tripwire. `intern` is
mint-or-get; the caller must also assert distinct keys (C6 — duplicate `ItemId`
diverges mint-or-get from today's unconditional double-mint; backlog ids are unique
by construction, RSK-005):

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
//   spec::relation_edges(id)       -> descends_from / parent / members / interactions
//   governance::relation_edges(id) -> supersedes / related      (NOT superseded_by, NOT tags)
//   backlog::relation_edges(id)    -> slices / specs / drift     (NOT needs/after/triggers)
//   review::relation_edges(id)     -> reviews        ([target].ref, review.rs)
//   rec::relation_edges(id)        -> owning_slice    (decision_ref -> free-text dangler, no DEC kind)

fn outbound_for(kind: entity::Kind, id: u32) -> Vec<RelationEdge>      // one match over all 11 KINDS
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

**Id parse — ride the existing seam.** `inspect` resolves `<ID>` via
`integrity::kind_by_prefix` / `parse_canonical_ref` (integrity.rs:315/341) — no new
parser; `parse_canonical_ref` is promoted to `pub(crate)`. `EntityKey` renders its
canonical ref through the same `KINDS`/`listing::canonical_id` source `ItemId` uses.

**Render contract — conform to the uniform read surface (SL-025; SL-045 precedent).**
`inspect` is `show`-like (one entity, grouped outbound/inbound/dangler sections), but
honours the uniform list/show/render contract: a default human render plus **`--json`**
(agent-readable, the `InspectView` serialized). It rides `src/listing.rs` rendering
helpers where the grouped sections map onto them — SL-045's `spec req list` (landing
first) is the read-surface template to reuse, not re-invent
(mem.pattern.authoring.reuse-tuned-prior-art-verbatim).

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
| `interactions` | spec → spec (typed; see below) |
| `slices` | backlog → slice |
| `related` | governance → governance |
| `reviews` | review → any (the `[target].ref` subject) |
| `owning_slice` | rec → slice |

**~11 overlays** + an 11-entry `OverlayId → Label` map. **`interactions` (decision
B)**: tech-spec `interactions.toml` `[[edge]]` rows carry a **free-text `type`**
(`spec.rs:230`) that `EdgeAttrs{rank,age}` cannot hold — so all interactions share a
**single `interactions` overlay** (graph identity = the relation *class*, not the
type); the per-edge `type` is re-read from the source `Interaction` struct at render
(adapter owns the parse, cordage stays generic — consistent with D2). **`reviews`
(decision A)**: RV's `[target].ref` is a single outbound subject edge; its inbound
on the target renders "reviewed by RV-N". **`owning_slice` (decision A)**: REC's
optional `owning_slice` → SL; REC's `decision_ref` is a free-text DEC ref (no DEC
kind in `KINDS`) → **dangler**, like `drift`. **Not** overlays: `superseded_by` (a
**stored** reverse field — but ADR-004 §5-sanctioned canon, NOT projected because
inbound is the registry surface's *derived* job per ADR-004 §3; see §7 D4), `tags`
(free-text, not entity refs), `needs`/`after`/`triggers` (dep/seq/mask — SL-047),
and **`drift`**
— backlog `drift` is a free-text `Vec<String>` with **no DRIFT kind** in `KINDS`, so
its targets never resolve to a node; they surface as danglers (visibility preserved),
not edges. A drift string that *does* happen to be a resolvable canonical ref still
dangles unless its kind is admitted — there is no dedicated drift overlay.

### 5.4 Lifecycle, Operations & Dynamics

1. `inspect <ID>`: parse prefix → `entity::Kind` via `KINDS` (error on unknown).
2. Build the relation graph once: walk `KINDS` in table order; for each kind,
   `entity::scan_ids` (which already ignores symlinks/non-dirs — the `NNN-slug`
   skip is free) then **sort the ids ascending** (C5 — `scan_ids` returns
   `fs::read_dir` order, *unsorted*; every list caller sorts after, e.g.
   `slice.rs:1024`; omitting the sort makes node-mint + render order
   filesystem-dependent, breaching REQ-077); `intern` each entity into
   `Projection<EntityKey>` in that sorted order.
3. Second pass: for each entity, `outbound_for(kind, id)` → for each `RelationEdge`,
   `projection.resolve(target)` → resolved ⇒ `builder.edge(overlay_for(label), src,
   dst, EdgeAttrs::new(0,0))`; unresolved/free-text ⇒ push dangler.
4. `builder.build()` (no `OrderSpec` over reference overlays → no composition, no
   union-cycle pass touches them).
5. Query node = `projection.resolve(EntityKey of ID)`. Outbound = the entity's own
   `RelationEdge`s grouped by label. Inbound = for each overlay,
   `graph.in_edges(ov, node)` → `key_of(src)`, grouped by `label_of(ov)`. The
   `supersedes`-overlay inbound renders as "superseded by Y" — the derived
   reciprocal (no `superseded_by` field read; ADR-004 §3 derivation).
6. Render outbound / inbound / danglers in a fixed deterministic order. `inspect`
   **never reads `graph.provenance()`** (C7) — a benign symmetric-`related` 2-cycle
   produces a `Reject` `CycleDiagnostic` (`resolve.rs:172-177`) that must not leak
   into the relation view; diagnostics are a `validate`/SL-048 concern.

### 5.5 Invariants, Assumptions & Edge Cases

- **I1 — no *unique* edge is ever lost.** Reference overlays are
  `Reject`+`Unbounded`: `Reject` removes no edges (resolve.rs:185 — *"Resolved set
  unchanged — the cycle is preserved"*); `Unbounded` exempts them from arity
  eviction (pass1 touches only `AtMostOne`, `resolve.rs:120-124`). `incoming`
  indexes the **resolved** set (`lib.rs:694-695,711-735`), which for Reject+Unbounded
  **equals** the authored set. So `in_edges` enumerates **every unique** authored
  inbound edge, cycle or not. **Caveat (C3):** `resolve` groups into a
  `BTreeSet<Edge>` keyed `(rank,age,src,dst)` per overlay (`resolve.rs:18-42,91`);
  every reference edge carries `EdgeAttrs::new(0,0)`, so two authored rows with the
  same `(overlay,src,dst)` **collapse to one** — benign (a duplicate reference
  relation carries no distinguishing payload; rendering it twice is meaningless),
  but the claim is "every **unique** `(label,src,dst)` edge", not "every authored
  row". `Evict` is forbidden here — it removes edges and would silently drop a real
  relation. **Asserted/tested at the projection boundary** (incl. a duplicate-ref
  test asserting single-edge, no panic).
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
- **I4 — only `supersedes` is projected; the stored `superseded_by` is left
  untouched (ADR-004 §5 canon), not removed.** ADR-004 §5 sanctions stored
  `superseded_by` as *"the sole sanctioned reverse field"* (lifecycle carve-out:
  the predecessor's file is rewritten on the `superseded` status flip anyway, so
  co-writing it is zero marginal coupling and the only honest place a reader of the
  dead record finds its successor). The reader does **not** project that stored
  field — not because it is a violation, but because **inbound is the registry
  surface's derived job (ADR-004 §3)**: the reader projects only the canonical
  outbound `supersedes` and derives "superseded by" from `in_edges`. Projecting
  both would double-count and form a false 2-cycle. The stored field stays; SL-046
  removes nothing (see §7 D4; IMP-032's removal premise is void).
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
  Governance stores `superseded_by` as a reciprocal of `supersedes`. **This is
  ADR-004 §5-sanctioned canon, not a violation** (external review correction):
  §5's lifecycle carve-out names it *"the sole sanctioned reverse field"* — when
  ADR-B supersedes ADR-A, A's file is rewritten for the `superseded` status flip
  anyway, so co-writing `superseded_by` adds zero marginal coupling and is the only
  honest place a reader of the dead record finds its successor. The reader projects
  only the canonical outbound `supersedes` and derives the reciprocal from
  `in_edges`; it does **not** project the stored `superseded_by` — not because the
  field is a sin to remove, but because **ADR-004 §3 makes inbound the
  registry-backed surface's *derived* job**, and projecting both would
  double-count. SL-046 is reader-only and **removes nothing**. *IMP-032 caveat*:
  the filed IMP-032 ("derive it, don't store it") rests on a misreading of ADR-004
  — its removal premise is **void** (§5 sanctions the field); it is left on the
  backlog for SL-048 triage (user disposition: reframe D4 only), where the honest
  follow-up is at most a `validate` cross-check that stored and derived agree, never
  a removal. `related` is genuinely symmetric and legitimately appears on both sides
  — not redundant. *Note*: this
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
  with correct labels, **all 11 kinds** (incl. spec `Option` singles + `members.toml`
  + `interactions.toml`; RV `reviews`; REC `owning_slice`).
- **Dedupe (C3)** — two authored rows sharing `(label,src,dst)` surface as **one**
  inbound edge, no panic.
- **Scan-order determinism (C5)** — out-of-order planted entity dirs yield identical
  output (the ascending sort after `scan_ids`).
- **`superseded_by` (C8 / R3 / ADR-004)** — a fixture with stored `superseded_by`
  and **no** reciprocal `supersedes`: `inspect` reports **no** inbound from it
  (R3 drop) and the accessor reads no stored reverse field (ADR-004 §3).
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
- **REQ-091 (adapter)** — the three acceptance criteria are discharged: (1) node
  ids minted by the adapter from doctrine ids, opaque to the core; (2) every edge
  emitted from an authored outbound relation (ADR-004), no inbound/derived field
  authored; (3) diagnostics re-mapped to doctrine ids + relation-kind names via
  `Projection::remap_set` + the `OverlayId → Label` map.

## 10. Review Notes

### Internal adversarial pass (pre-plan) — integrated

- **A — `entity::Kind` exists, is the right type.** Confirmed (integrity.rs:37
  `kind: &'static entity::Kind`). `EntityKey` uses it; no change.
- **B — id parse already built.** `inspect` rides `integrity::kind_by_prefix` /
  `parse_canonical_ref` (integrity.rs:315/341) instead of a new parser; the latter
  is promoted `pub(crate)`. Integrated into §5.2.
- **C — `drift` overlay was a phantom.** `drift` is free-text with no `DRIFT` kind;
  its targets never resolve. Dropped from the overlay set (9 → 8); drift refs
  surface as danglers. Integrated into §5.3.
- **D — REQ-091 traceability.** All three criteria mapped (see §9).
- **SL-045 bearing.** SL-045 (requirement status-visibility read surface, `plan`,
  lands first) is the read-surface precedent: it rides the uniform list/show/render
  contract (SL-025) + `src/listing.rs` with `--columns`/`--json`. `inspect` must
  conform — at least `--json`. Integrated into §5.2 (render contract). No code
  collision (disjoint subsystems); the bearing is render-contract consistency.

### External adversarial pass (codex GPT-5.5, pre-plan) — integrated

Full record: `inquisition.md`. Every charge re-verified against source.

- **C1 (blocker) — RV + REC omitted.** `KINDS` has 11 kinds; the seam handled 4.
  RV has outbound `reviews` (`[target].ref`, `review.rs`), REC has `owning_slice`
  → SL + free-text `decision_ref` (`rec.rs:101-120`). **Decision A: include both**
  (overlay set 8 → ~11). The "all-entity" title is now honest. §5.1/§5.2/§5.3.
- **C2 (blocker) — spec `interactions.toml` dropped.** Outbound spec→spec `[[edge]]`
  with **free-text `type`** (`spec.rs:225-245`). **Decision B: single `interactions`
  overlay**, `type` re-read at render. §5.2/§5.3.
- **ADR-004 §5 correction (user-surfaced) — `superseded_by` is canon, not a
  violation.** D4 reframed: the stored reverse field is §5-sanctioned; the reader
  doesn't project it per §3 (inbound derived), not per a removal mandate. IMP-032's
  premise is void; left on the backlog for SL-048 triage. §5.3/§5.5 I4/§7 D4.
- **C3 (major) — I1 over-claimed.** `BTreeSet<Edge>` dedupes `(label,src,dst)` under
  uniform `EdgeAttrs(0,0)`; reworded to "every **unique** edge" — benign collapse.
  The cordage safety core (Reject non-mutating, Unbounded skips arity, resolved ==
  authored) is **sound** as verified. §5.5 I1.
- **C4 (major, most-likely-to-break) — mint-sequence unwritten.** D3 now mandates
  sort → pre-intern-all-in-sorted-order → emit; the byte-exact golden is the
  tripwire. §5.2.
- **C5 (major) — `scan_ids` unsorted.** Mandated ascending id sort. §5.4.
- **C6 (major, low risk) — duplicate-`ItemId` mint-or-get ≠ double-mint.** Hard
  distinct-key precondition. §5.2/§8 R2.
- **C7 (minor) — `related` cycle diagnostic noise.** `inspect` never reads
  `provenance()`. §5.4.
- **C8 (minor, confirmatory) — lone-`superseded_by` fixture.** §9.

**Direct follow-up to interrogate (IMP-034):** whether to refactor *all* relations
modelling to a uniform schema — a single generic `[[relation]] kind=… target=…`
surface across every kind vs today's bespoke per-kind typed fields. Surfaced by D4;
adjacent to IMP-006 (uniform verbs) and IMP-016 (cross-corpus links). Should be run
in parallel with, or as a direct successor to, this slice — and likely feeds the
relation-governance ADR that SL-048 needs.
