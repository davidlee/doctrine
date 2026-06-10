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
pub enum Combinator  { Max, Any, All, CountDistinct }   // commutative monoids; each owns a value domain
pub enum Direction   { Backward, Upward, None }
pub enum ChannelValue { Flag(bool), Scalar(i64), Count(u32) }   // Any/All→Flag · Max→Scalar · CountDistinct→Count
pub struct ChannelSpec { overlay: OverlayId, combinator: Combinator, direction: Direction }

// ── ordering composition (F1: generic, no dep/seq names) ─────
// Policy supplies the precedence: "layer the reject overlay first, the evict
// overlay within-level, then fallback." The core composes lexicographically and
// knows none of the overlays' meaning.
pub struct OrderLayer { overlay: OverlayId, direction: Direction }
pub struct OrderSpec  { layers: Vec<OrderLayer> }   // NodeId fallback is always implicit + last

// ── build → query ────────────────────────────────────────────
GraphBuilder::new().overlay(cfg) -> OverlayId; .node() -> NodeId;
                   .edge(ov, src, dst, attrs)
                   .order_spec(OrderSpec)            // policy precedence; build resolves union vs it
                   .build() -> Result<Graph>         // per-overlay + cross-layer cycles resolved

impl Graph {
    fn out_edges(&self, ov, n) -> &BTreeSet<Edge>;
    fn in_edges (&self, ov, n) -> &BTreeSet<Edge>;          // reverse index (REQ-074 primitive)
    fn reachable(&self, ov, n, dir) -> BTreeSet<NodeId>;    // STRICT — excludes n (F8)
    fn spine_path(&self, ov, n) -> Vec<NodeId>;             // follows the kept parent post arity-resolution (F6/F7)
    fn order_key(&self, n) -> OrderKey;  fn ordered(&self) -> Vec<NodeId>;   // per the build's OrderSpec
    fn evaluate(&self, spec: &ChannelSpec, seed: &BTreeMap<NodeId, ChannelValue>) -> Channel;
    fn provenance(&self) -> &Provenance;  fn explain(&self, n) -> Explanation;
}
```

`order_key` is now generic (F1): `OrderKey` is the lexicographic tuple of each
layer's longest-path level in its direction, with `NodeId` as the implicit total
tail. The doctrine recipe "dep-topology → seq-rank → fallback" (D9/D10) is just
policy passing `OrderSpec{ layers: [reject_overlay@Backward, evict_overlay@…] }` —
the core never names dep or seq.

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

`build()` runs four deterministic passes; all overlay/layer references are by
opaque `OverlayId` from the policy-supplied `OrderSpec` — no dep/seq names (F1):

1. **Arity enforcement (F7).** For each `AtMostOne` overlay, a node with >1 incoming
   edge keeps the `(rank asc, age asc)`-minimal parent; the rest → `EvictedEdge
   {ArityViolation}` in provenance (deterministic, surfaced not silent). This makes
   `spine_path` single-valued by construction.
2. **Per-overlay cycle resolution (D5/REQ-092).**
   - `Reject`: detect SCCs, no mutation; each non-trivial SCC → `CycleDiagnostic`,
     marked **degraded** — never a false order (REQ-076). `build()` still returns `Ok`
     (cycle is data).
   - `Evict`: while a non-trivial SCC exists, evict the **globally-minimal
     participating edge** by `(rank asc, age asc)`, recompute, repeat. Unique min
     (total order) → deterministic; each eviction strictly reduces edges →
     terminates ≤ `|E|`. Every eviction → `EvictedEdge{IntraOverlayCycle}`.
3. **Cross-layer union resolution (D9 — earlier layer authoritative) (F2).** Walking
   the `OrderSpec` layers in precedence order, compose the partial order incrementally.
   For each edge `u→v` in layer *k*: if the composed order of layers `< k` already
   places `v` before `u` (the edge would reverse a higher-precedence decision), it is
   a contradiction → **evict by `(rank asc, age asc)`** as `EvictedEdge{UnionCycleVsLayer}`.
   An edge consistent-but-redundant with the earlier order is dropped silently (the
   earlier layer already encodes it). Edges between earlier-incomparable nodes survive
   and order within that eligible set. (D9's "dep authoritative, seq yields" is the
   2-layer instance.)
4. **`order_key` materialization (D7/REQ-077).** Per node, the lexicographic tuple
   `( level_in_layer_0, level_in_layer_1, …, node_id )` — each entry the node's
   longest-path level (`0` if no in-direction predecessor, else `1 + max`) in that
   resolved layer. `NodeId` tail guarantees totality → ordering never falls to
   map-iteration order. **Degraded SCC nodes (F9):** assigned `level = u32::MAX`
   (saturating sentinel) in the affected layer → sorted after all clean nodes, among
   themselves by `NodeId` — present and deterministic, but never falsely linearized.

`explain(n)` (D11 — always walks to root) assembles **structured paths only**:

```rust
pub struct Explanation { node: NodeId, order_key: OrderKey,
    blockers: Vec<Vec<NodeId>>,      // transitive dep chains to root
    spine:    Option<Vec<NodeId>>,   // canonical ancestry if a spine overlay exists
    evicted:  Vec<EvictedEdge> }     // evictions touching n
pub struct Channel { values: BTreeMap<NodeId, ChannelValue>,
    contributors: BTreeMap<NodeId, Vec<NodeId>> }   // Any→witness, Max→argmax, Count→set
pub struct Provenance { cycles: Vec<CycleDiagnostic>, evictions: Vec<EvictedEdge> }
pub struct EvictedEdge { overlay: OverlayId, edge: EdgeRef, reason: EvictReason }
pub enum   EvictReason { ArityViolation, IntraOverlayCycle, UnionCycleVsLayer }
```

No `String` prose, no channel name, no doctrine noun anywhere in these — rendering
is policy's (D1).

### 5.5 Invariants, Assumptions & Edge Cases

- **I1.** Every overlay is acyclic post-`build()` (reject-degraded SCCs aside, which
  never linearize).
- **I2 (generic, post-F1).** An earlier `OrderSpec` layer is never overridden by a
  later one — the lexicographic tuple makes layer 0 dominate layer 1, etc. (D9's
  "dep authoritative, seq yields" is the 2-layer case; the core states it without
  naming dep/seq.)
- **I3.** Recompute from identical inputs → identical `order_key`, `Channel`,
  `Provenance` (REQ-077).
- **I4.** No authored mutation — eviction is a build-time derived resolution; inputs
  are consumed, never written back (storage rule, D8).
- **I5 (F3 — combinator/value contract).** Each `Combinator` owns a `ChannelValue`
  domain (`Any`/`All`→`Flag`, `Max`→`Scalar`, `CountDistinct`→`Count`). A seed entry
  of a mismatched variant collapses to the combinator's **identity** (`All`→true,
  `Any`/`Max`→false/`i64::MIN`, `Count`→0) — deterministic and non-panicking (no
  `unwrap`), documented as a caller precondition rather than enforced by types in v1.
- **I6 (F8 — reachable is strict).** `reachable(n)` excludes `n`; `CountDistinct`
  therefore counts strict-reachable contributors, never `n`'s own seed.
- **Assumption A1.** `age` is total + stable across recomputes (adapter contract;
  test-supplied here).
- **Edge cases:** empty graph; single node no edges; self-loop (trivial SCC →
  reject diagnoses / evict drops); disjoint cycles (each loses its own min edge); a
  later-layer edge that only closes a cycle against a higher-precedence layer →
  evicts itself (no-op); a node with >1 parent on an `AtMostOne` overlay → keeps the
  min, others `ArityViolation`.

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
| **DD1 rollup (F5)** | `Unbounded` membership overlay, a node with **2 parents**, `Upward` `All`/`CountDistinct` → aggregates from both parents correctly; `spine_path` on an `AtMostOne` overlay returns the single kept path; `CountDistinct` over a diamond counts the distinct node once (R3). |
| **arity (F7)** | >1 parent on `AtMostOne` → min kept, rest `EvictedEdge{ArityViolation}`. |
| **union (F2)** | 3-layer `OrderSpec` where a layer-2 edge contradicts layer-0 → `EvictedEdge{UnionCycleVsLayer}`, layer-0 order preserved. |

TDD red/green/**refactor** per phase (sequenced by `/plan`). `[lints] workspace = true`;
`just check` zero-warnings after every file. Pure throughout; `age` test-supplied.

## 10. Review Notes

### Adversarial self-review (round 1) — 9 findings, all integrated

- **F1 (significant, boundary leak) — FIXED.** `order_key`/`dep_level` hardcoded
  "dep"/"seq" by name → the core deciding authoritative overlay + prerequisite
  direction = doctrine meaning inside the neutral core (fails §9/REQ-079). Fix:
  `order_key` is now generic over a policy-supplied `OrderSpec { layers }`; the core
  composes lexicographically and union-resolves cross-layer contradictions without
  naming any overlay. D9/D10's "dep-topology → seq-rank → fallback" becomes policy
  passing a 2-layer spec. (§5.2, §5.4, I2.)
- **F5 (significant, verification gap) — FIXED.** §9 omitted the `Upward` rollup over
  a multi-parent `Unbounded` overlay — the DD1/T1 headline. Added the rollup fixture
  row (multi-parent aggregation, `spine_path`, diamond `CountDistinct`). (§9.)
- **F2 (moderate) — FIXED.** Union-eviction detection underspecified ("contradicts").
  Now precise: per layer-*k* edge, contradiction = earlier-layer composed order already
  reverses it → evict `(rank,age)`; `EvictReason::UnionCycleVsLayer`. (§5.4 pass 3.)
- **F3 (moderate) — FIXED.** Combinator↔`ChannelValue` pairing had no contract. I5:
  each combinator owns a domain; mismatched seed → identity, non-panicking, documented
  precondition. (§5.5 I5.)
- **F7 (moderate) — FIXED.** `Arity` was decorative. Now build-time pass 1 enforces
  `AtMostOne` (keep min parent, rest `ArityViolation`), making `spine_path`
  single-valued by construction. (§5.4 pass 1.)
- **F9 (moderate) — FIXED.** Degraded-SCC `order_key` undefined. Now `level = u32::MAX`
  sentinel → sorted after clean nodes, among themselves by `NodeId`. (§5.4 pass 4.)
- **F4 (minor) — FIXED.** `Score(i64)` flirted with Appendix-B "urgency scoring" →
  renamed `Scalar(i64)` (neutral; it is just the `Max` domain). (§5.2.)
- **F6 (minor) — FIXED.** `spine_path` precondition resolved via F7 — it follows the
  single kept parent post arity-resolution, no `Result` needed. (§5.2.)
- **F8 (minor) — FIXED.** `reachable` inclusivity undefined → declared strict
  (excludes `n`); `CountDistinct` counts strict-reachable only. (§5.2, I6.)

Net: F1 changed the public ordering interface (`OrderSpec`); the rest tightened
contracts and verification. No finding overturned a DD. Design stands.

### External pass

(Pending — `/inquisition` or external reviewer, user's call.)
