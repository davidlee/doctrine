# Design SL-036: cordage graph core crate

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Build `crates/cordage`: the **generic, product-neutral graph-evaluation core**
that realises SPEC-001 **D1**. It owns a node set plus typed directed overlays,
per-overlay cycle policy, reverse index, reachability, deterministic ordering,
generic channel propagation, and structured provenance/explanation — **with no
doctrine or product vocabulary** (REQ-079; Appendix B forbidden list). Doctrine's
policy and adapter layers (later slices) build on it by path dependency; cordage
is the deepest leaf (ADR-001).

## 2. Current State

`Cargo.toml` is a single-member workspace (`members = ["."]`); no `crates/` dir.
No priority engine exists. The lint posture lives in `[workspace.lints.{rust,clippy}]`
and is opted into per-member via `[lints] workspace = true` — `disallowed_types`
(BTree-not-Hash), `indexing_slicing`, `as`-cast ban, `unwrap_used`/`expect_used`
deny (bins/lib; tests exempt under plain `cargo clippy`). The authored capture
schema cordage will *eventually* consume (`needs`/`after`/`triggers`, PRD-009) is
blessed but unbuilt — irrelevant to this slice, which has no doctrine consumer yet.

## 3. Forces & Constraints

- **Boundary purity is load-bearing (REQ-079, D2).** No product noun in the core;
  the §9 boundary test governs every placement. Channels (`blocked`/`actionable`/
  `consequence`) are doctrine *meaning* → they cannot exist in the core at all,
  only as policy-supplied values fed in and read out.
- **Determinism (REQ-077, D7).** Same inputs → byte-identical order + provenance.
  No clock/RNG/Hash-iteration in any ordering path.
- **Pure leaf (ADR-001).** No IO, no clock, no git; pure functions + plain data.
- **Reverse edges are derived, never stored (ADR-004, REQ-074).**
- **Degrade, never falsify (REQ-076).** A `dep` cycle must not yield a false
  topological order.
- **Small corpus (H1/H2).** Tens–hundreds of nodes — naive O(V+E) algorithms suffice.

## 4. Guiding Principles

- **The overlay is the one general primitive** (DD1). Tree, dependency, sequencing,
  membership are all overlays differing only by config — no privileged tree type.
- **One mechanism, many channels** (DD2). Every channel is one topological-order
  monoid fold; only the combinator/direction/seed vary.
- **Structured, not magic** (DD2/REQ-077). Provenance and explanation are
  structured data (node-id paths, evicted edges); prose rendering is policy's job.
- **Generality bounded to the known floor** (DD4). Build the full engine but hold
  the combinator vocab to `{Max, Any, All, Count-distinct}`; REQ-080 makes later
  additions anticipated and non-breaking.

## 5. Proposed Design

### 5.1 System Model

Uniform substrate (DD1): a `Graph` is a `NodeId` arena + N typed overlays, each
configured `{cycle_policy, arity}`. No distinct tree primitive — the "spine" is a
policy role: an overlay declared `arity = AtMostOne` yields O(depth) unique-root
ancestry (`spine_path`) and is the cheap default-in for tree behaviour. Multi-parent
membership = an `arity = Unbounded` overlay carrying a rollup combinator.

```text
  policy / adapter  (later slices, in the doctrine crate — NOT here)
        │  builds overlays, supplies channel seeds, renders prose
  ┌─────┴───────────────────────────────────────────────┐  crates/cordage (leaf)
  │  GraphBuilder → build()  →  Graph + Provenance        │
  │    model · build · cycle · order · channel · reach · explain · graph
  └───────────────────────────────────────────────────────┘
        no doctrine vocabulary · pure · zero runtime deps
```

### 5.2 Interfaces & Contracts

```rust
// ── identity ─────────────────────────────────────────────────
pub struct NodeId(u32);      // opaque, insertion-ordered; adapter maps doctrine id ↔ this
pub struct OverlayId(u16);   // opaque; policy holds the meaning (dep/seq/membership)

// ── overlay config (DD1) ─────────────────────────────────────
pub enum CyclePolicy { Reject, Evict }          // D5: dep rejects, seq evicts
pub enum Arity       { AtMostOne, Unbounded }   // AtMostOne = spine-capable
pub struct OverlayConfig { cycle_policy: CyclePolicy, arity: Arity }
pub struct EdgeAttrs { rank: i32, age: u64 }    // opaque; (rank asc, age asc) eviction order

// ── channels (DD2) ───────────────────────────────────────────
pub enum Combinator  { Max, Any, All, CountDistinct }   // commutative monoids
pub enum Direction   { Backward, Upward, None }
pub enum ChannelValue { Flag(bool), Score(i64), Count(u32) }
pub struct ChannelSpec { overlay: OverlayId, combinator: Combinator, direction: Direction }

// ── build → query ────────────────────────────────────────────
GraphBuilder::new().overlay(cfg) -> OverlayId; .node() -> NodeId;
                   .edge(ov, src, dst, attrs); .build() -> Result<Graph>  // cycles resolved

impl Graph {
    fn out_edges(&self, ov, n) -> &BTreeSet<Edge>;
    fn in_edges (&self, ov, n) -> &BTreeSet<Edge>;          // reverse index (REQ-074 primitive)
    fn reachable(&self, ov, n, dir) -> BTreeSet<NodeId>;
    fn spine_path(&self, ov, n) -> Vec<NodeId>;             // ov must be AtMostOne
    fn order_key(&self, n) -> OrderKey;  fn ordered(&self) -> Vec<NodeId>;
    fn evaluate(&self, spec: &ChannelSpec, seed: &BTreeMap<NodeId, ChannelValue>) -> Channel;
    fn provenance(&self) -> &Provenance;  fn explain(&self, n) -> Explanation;
}
```

**The propagation contract** — after `build()` every overlay is acyclic, so a
channel is a single topo-order monoid fold (no iteration):

```
value(n) = combinator.combine( seed(n), fold{ value(m) : m ∈ direction-neighbours(n) } )
```

`Any`/`All`/`Max` are idempotent → one pass is exact (diamond reconvergence is a
no-op). `CountDistinct` is the exception: `|{ m ∈ reachable(n, ov, dir) : seed(m).flag }|`
— distinct-reachable-set size, never path-multiplicity (DD2).

### 5.3 Data, State & Ownership

- **Storage:** overlays as `BTreeMap<OverlayId, BTreeMap<NodeId, BTreeSet<Edge>>>`;
  reverse index symmetric, built once at `build()` (derived, ADR-004). BTree
  throughout → deterministic iteration is structural, not incidental.
- **Ownership (D1):** core owns the mechanism; **policy** owns channel meaning,
  classification, rendering; **adapter** owns the doctrine-id↔NodeId map and the
  `age` ordinal. `evaluate` is caller-driven (policy passes spec+seed per channel)
  — the graph holds **no** channel registry, stays stateless w.r.t. channel meaning.

### 5.4 Lifecycle, Operations & Dynamics

`build()` runs three deterministic passes:

1. **Per-overlay cycle resolution.**
   - `Reject`: detect SCCs, no mutation; each non-trivial SCC → `CycleDiagnostic`,
     marked **degraded** (excluded from clean topo, placed by `NodeId` fallback) —
     never a false order (REQ-076). `build()` still returns `Ok` (cycle is data).
   - `Evict`: while a non-trivial SCC exists, evict the **globally-minimal
     participating edge** by `(rank asc, age asc)`, recompute, repeat. Unique min
     (total order) → deterministic; each eviction strictly reduces edges →
     terminates ≤ `|E|`. Every eviction → `EvictedEdge{IntraOverlayCycle}`.
2. **D9 union — dep authoritative, seq yields.** Resolve dep → `dep_level(n)` by
   longest-path layering (`0` if no prerequisites, else `1 + max` over dep-targets).
   A `seq` edge is honoured only between dep-incomparable nodes; one that dep already
   orders is redundant (dropped) or contradicts → **evicted by the same `(rank,age)`
   rule** as `EvictedEdge{UnionCycleVsDep}`. Surviving intra-level seq edges topo-order
   the level.
3. **`order_key`** = `( dep_level(n), seq_pos_within_level(n), node_id(n) )`,
   lexicographic ascending (do-earliest-first). The `NodeId` tail guarantees totality
   — ordering never falls to map-iteration order.

`explain(n)` (D11 — always walks to root) assembles **structured paths only**:

```rust
pub struct Explanation { node: NodeId, order_key: OrderKey,
    blockers: Vec<Vec<NodeId>>,      // transitive dep chains to root
    spine:    Option<Vec<NodeId>>,   // canonical ancestry if a spine overlay exists
    evicted:  Vec<EvictedEdge> }     // evictions touching n
pub struct Channel { values: BTreeMap<NodeId, ChannelValue>,
    contributors: BTreeMap<NodeId, Vec<NodeId>> }   // Any→witness, Max→argmax, Count→set
pub struct Provenance { cycles: Vec<CycleDiagnostic>, evictions: Vec<EvictedEdge> }
```

No `String` prose, no channel name, no doctrine noun anywhere in these — rendering
is policy's (D1).

### 5.5 Invariants, Assumptions & Edge Cases

- **I1.** Every overlay is acyclic post-`build()` (reject-degraded SCCs aside, which
  never linearize).
- **I2.** `dep` blocking is never overridden by a `seq` preference — `dep_level`
  dominates `order_key` (D9).
- **I3.** Recompute from identical inputs → identical `order_key`, `Channel`,
  `Provenance` (REQ-077).
- **I4.** No authored mutation — eviction is a build-time derived resolution; inputs
  are consumed, never written back (storage rule, D8).
- **Assumption A1.** `age` is total + stable across recomputes (adapter contract;
  test-supplied here).
- **Edge cases:** empty graph; single node no edges; self-loop (trivial SCC →
  reject diagnoses / evict drops); disjoint cycles (each loses its own min edge);
  a `seq` edge that only closes a cycle against stronger edges → evicts itself (no-op).

## 6. Open Questions & Unknowns

- **OQ-1 (soft, internal).** Module decomposition — `reach` may fold into `graph`;
  settled during implementation, not load-bearing.
- **OQ-2 (deferred, not blocking).** Whether `ChannelValue` ever needs a fourth
  variant or a generic value type. Held closed at the 3-variant enum for v1; REQ-080
  makes a later widening non-breaking.
- **Upstream (flagged, not this slice).** DD1 departs from SPEC-001's literal "tree +
  typed-DAG overlays" toward "typed overlays, one conventionally the spine." This is
  a SPEC-001 node-model note to make deliberately (post-lock revision), tracked in
  slice-036.md T1. Likewise T2 (ordered/unordered member groups) is expressible with
  no new primitive (presence/absence of an intra-sibling order thread); whether
  "ordered" becomes a first-class membership property is a downstream policy call.

## 7. Decisions, Rationale & Alternatives

- **DD1 — uniform overlay substrate, no distinct tree primitive; spine is a policy
  role made ergonomic.** Resolves T1. *Alt rejected:* a `Tree` primitive + overlays
  (A) — splits propagation across two substrates (rollup over tree *and* overlay) and
  can't hold multi-parent membership. *Alt rejected:* pure uniform with spine as core
  privilege (B) — loses cheap unique-root; chosen middle (C) keeps `spine_path` as an
  unprivileged ergonomic.
- **DD2 — closed commutative-monoid combinator enum over a generic `ChannelValue`,
  direction per overlay; channels are policy-defined instances.** *Alt rejected:*
  arbitrary combinator closure — can't guarantee commutativity → determinism risk the
  core can't enforce. *Alt rejected:* core-defined channel enum — a boundary violation
  (channels are product meaning). Non-idempotent combinators defined over the distinct
  reachable set.
- **DD3 — hand-roll, no graph dependency.** `petgraph` would cover ~15% (single-graph
  topo/SCC) while the bulk — multi-overlay orchestration, eviction-to-fixpoint, D9
  union, monoid propagation — is custom regardless; a dep adds a determinism audit and
  a diagnostic-shape mismatch (REQ-076 wants node-ids+edge-kinds). Small corpus removes
  the scale argument.
- **DD4 — full engine, combinator vocab floored.** *Alt rejected:* defer propagation
  (B) — fractures one coherent leaf and the mechanism is fully fixture-testable now.
  The REQ-079 vocabulary-free suite is the validating consumer.

## 8. Risks & Mitigations

- **R1 — hand-rolled topo/SCC bug.** *Mit:* small textbook algorithms + the explicit
  cycle fixtures are mandated acceptance tests; determinism re-run check.
- **R2 — designing API ergonomics with no production consumer.** *Mit:* hold vocab to
  the floor; REQ-080 makes additions non-breaking; the boundary suite exercises every
  path (combinators × directions × cycle policies).
- **R3 — `CountDistinct` double-counting over diamonds.** *Mit:* defined over the
  distinct reachable *set* (idempotent framing), explicit fixture.
- **R4 — boundary erosion.** A future contributor leaks a doctrine noun. *Mit:* the
  no-doctrine-dependency in `Cargo.toml` makes leakage a compile error, not a style nit;
  the vocabulary-free suite is the standing proof.

## 9. Quality Engineering & Validation

Black-box, vocabulary-free `tests/` (overlays `a`/`b`, channels `Flag`/`Count`):

| Req | Evidence |
|---|---|
| **REQ-079** boundary | no doctrine dep in `Cargo.toml`; whole suite passes on structural identifiers only. |
| **REQ-076** reject | dep cycle → `CycleDiagnostic` names nodes+edges, SCC degrades, remainder orders. |
| **REQ-092** evict | seq cycle → min-`(rank,age)` edge evicted to fixpoint, in provenance. |
| **REQ-077** determinism | build twice → identical `order_key` + `Provenance` + contributor traces; union fixture `A —dep→ B`, `B —seq→ A`. |
| **REQ-080** seam | a fresh channel via existing combinators works with no core change; `Combinator` doc-marked as the curated extension point. |

TDD red/green/**refactor** per phase (sequenced by `/plan`). `[lints] workspace = true`;
`just check` zero-warnings after every file. Pure throughout; `age` test-supplied.

## 10. Review Notes

(Adversarial pass pending — §Process step 6.)
