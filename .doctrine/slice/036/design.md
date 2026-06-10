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
- **One mechanism, many channels** (DD2). Every channel is one monoid fold over
  a reachability-defined fold set (`{n} ∪ reachable` for the idempotent
  combinators; *strict* `reachable` for `CountDistinct` — F34); the topo-order
  fold is the acyclic-case implementation (F15); only the combinator/direction/
  seed vary.
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
pub struct NodeId(u32);      // opaque; builder-allocated, monotonic, no deletion in v1 (F29);
                             // adapter maps doctrine id ↔ this
pub struct OverlayId(u16);   // opaque; policy holds the meaning. u16 cap documented;
                             // exceeding it is a build input error (F29)

// ── overlay config (DD1) ─────────────────────────────────────
pub enum CyclePolicy { Reject, Evict }          // D5: reject diagnoses, evict resolves
pub enum Arity       { AtMostOne, Unbounded }   // AtMostOne = spine-capable
pub struct OverlayConfig { cycle_policy: CyclePolicy, arity: Arity }
pub struct EdgeAttrs { rank: i32, age: u64 }    // higher rank wins keeps; lower rank evicted
                                                // first (D4; mechanical wording, F44).
// Eviction key (total, F17): (rank asc, age asc, src, dst) — lowest-key evicted first.
// ALL eviction SELECTION compares by THIS key, never by adjacency-set order (F37;
// Provenance OUTPUT order stays (overlay, edge), F21); the NodeId tail makes
// determinism core-internal, independent of A1.

// ── channels (DD2) ───────────────────────────────────────────
pub enum Combinator  { Max, Any, All, CountDistinct }   // commutative monoids; each owns a value domain
pub enum Direction   { Along, Against, None }   // structural (F13): Along walks out_edges,
                                                // Against walks in_edges, None = no traversal
pub enum ChannelValue { Flag(bool), Scalar(i64), Count(u32) }
// Seed/output domains (F15/F16): Any/All consume+emit Flag · Max consumes+emits Scalar ·
// CountDistinct consumes Flag seeds, emits Count — Flag(true) contributes its node,
// Flag(false) contributes NOTHING (F45): a closure holding present-but-all-false seeds
// folds to Count(0) — real data ("zero distinct"), distinct from absent (nothing seeded).
// Channels are single-overlay in v1 (F28); multi-overlay channels are policy
// composition or a REQ-080 extension.
pub struct ChannelSpec { overlay: OverlayId, combinator: Combinator, direction: Direction }

// ── ordering composition (F1: generic, no dep/seq names) ─────
// Policy supplies the precedence; the core composes the layers into one order
// structure U (§5.4 pass 3) and knows none of the overlays' meaning.
pub struct OrderLayer { overlay: OverlayId, direction: Direction }  // Direction::None malformed here (F22)
pub struct OrderSpec  { layers: Vec<OrderLayer> }   // precedence order; empty = pure-NodeId order;
                                                    // NodeId fallback is always implicit + last.
                                                    // An overlay appears in AT MOST ONE layer, either
                                                    // direction (F38) — keeps every U arc attributable
                                                    // to exactly one (layer, orientation).

// ── build → query ────────────────────────────────────────────
GraphBuilder::new().overlay(cfg) -> OverlayId; .node() -> NodeId;
                   .edge(ov, src, dst, attrs)
                   .order_spec(OrderSpec)
                   .build() -> Result<Graph, BuildError>
// Err = malformed input ONLY (F14): unknown node/overlay id, overlay repeated in
// OrderSpec in any direction (F38 — strengthens F22's duplicate-(overlay,direction)),
// Direction::None layer, overlay cap. Cycles,
// evictions, degradation are NEVER Err — they are data in Provenance; build succeeds.

impl Graph {
    fn out_edges(&self, ov, n) -> /* iterator or set view — impl-level, OQ-1 (F29) */;
    fn in_edges (&self, ov, n) -> /* mirror */;             // reverse index (REQ-074 primitive)
    fn reachable(&self, ov, n, dir) -> BTreeSet<NodeId>;    // STRICT — excludes n (F8); pure edge
                                                            // traversal — total + cycle-safe even on
                                                            // degraded overlays (F12)
    fn spine_path(&self, ov, n) -> Option<Vec<NodeId>>;     // None unless overlay is AtMostOne (F23);
                                                            // follows the kept parent (F6/F7)
    fn order_key(&self, n) -> OrderKey;  fn ordered(&self) -> Vec<NodeId>;   // per the build's OrderSpec
    fn evaluate(&self, spec: &ChannelSpec, seed: &BTreeMap<NodeId, ChannelValue>) -> Channel;
    fn provenance(&self) -> &Provenance;  fn explain(&self, n) -> Explanation;
}
// Query methods given a foreign/unknown id return empty/None — defined, non-panicking (F14).
```

`OrderKey` (F11 — replaces round 1's per-layer tuple; suffix re-keyed F33):
`(Level, NodeId)` with `Level::{Finite(u32), Degraded(u32)}`. **Both** variants
carry the node's longest-path level in the **composed order structure `U`**
(§5.4 pass 3) — one DAG holding every surviving layer edge; `U` is acyclic
post-pass-3, so the level recurrence is total over ALL nodes with no sentinel
(F12). Taint (§5.4 pass 4) decides the *variant* only. `Degraded(_)` sorts after
every `Finite(_)` — sound because taint propagates downstream, so no surviving
`U` edge runs tainted→clean; within a variant `(level, NodeId)` keeps totality.
`ordered()` therefore respects *every* surviving `U` edge, including between
degraded nodes — round 2's bare-`NodeId` degraded suffix violated surviving
clean-layer edges inside the suffix and made I2 literally false (F33). The
round-1 lexicographic per-layer tuple was unsound: equal levels in
an earlier layer do not mean *incomparable* in it (layer-0 `a→b` with isolated
`c`; layer-1 `b→c` → the tuple sorts `c` before `b` though nothing conflicts).
Precedence lives where it belongs — in pass-3 *eviction authority* (earlier
layers are never evicted against) — not in key position. The doctrine recipe
"dep-topology → seq-topology → fallback" (D9/D10) is still just policy passing a
2-layer `OrderSpec` — the core never names dep or seq.

**The propagation contract (F15; fold set split per combinator class, F34):**

```
Any/All/Max:    value(n) = combinator-fold of the PRESENT seeds over {n} ∪ reachable(n, ov, dir)
CountDistinct:  value(n) = |{ m ∈ reachable(n, ov, dir) : seed(m) = Flag(true) }|
```

Round 2's single universal formula was false for `CountDistinct` on both facets
F34 names: self-inclusion (single seeded node under the inclusive closure ⇒
`Count(1)` vs I6's strict ⇒ no contributor) and SCCs (strict closures differ per
member, so members may legitimately differ by exactly the member-seed
contributions). `CountDistinct` is deliberately strict (I6) and counts only
`Flag(true)` seeds — `Flag(false)` is a *present* seed contributing nothing
(F45), so an all-false fold set yields `Count(0)`: real data, distinct from
absence. On an acyclic
overlay the idempotent combinators (`Any`/`All`/`Max`) compute this as a single
topo-order fold; `CountDistinct` folds a `BTreeSet<NodeId>` accumulator (set
union — commutative + idempotent, a genuine monoid) and projects to `Count` at
read, so diamond reconvergence is structurally a no-op (R3) and DD2's "one
mechanism" claim is honest: one fold, two set conventions. Reachability is
well-defined on cyclic edge sets, so
the contract stays total even over a degraded Reject overlay; the condensation
corollary — nodes of one SCC share a closure and therefore a value — holds for
the **idempotent** combinators over SCCs of the *traversal view* (an authored
SCC broken by arity, F30, reads post-arity), NOT for `CountDistinct`, whose
strict closures differ per SCC member (F34) — cycles degrade
*order*, they do not falsify *reachability* (REQ-076). Seed contract (F16): a
node missing from the seed map contributes nothing; a node whose fold set holds
no present seed is **absent** from `Channel.values` — no combinator identity
ever escapes as an output (a fabricated `Scalar(i64::MIN)` or vacuous
`Flag(true)` would be indistinguishable from data). A seed of the wrong variant
is a deterministic `ChannelDiagnostic` and treated as absent — surfaced, never
silently coerced; a seed keyed by a foreign/unknown `NodeId` is likewise a
deterministic `ChannelDiagnostic{UnknownSeedNode}` and ignored — surfaced, never
silently dropped (F41). `Direction::None` is per-combinator (F35):
`Any`/`All`/`Max` emit the node's own present, variant-valid seed (else absent);
`CountDistinct` under `None` is **always absent** — its strict fold set is empty
— never a fabricated `Count(0)` or a `Flag` leaking into the `Count` domain.

### 5.3 Data, State & Ownership

- **Storage:** overlays as `BTreeMap<OverlayId, BTreeMap<NodeId, BTreeSet<Edge>>>`;
  reverse index symmetric, built once at `build()` (derived, ADR-004). BTree
  throughout → deterministic iteration is structural, not incidental. `Edge`
  ordering is explicit, never derive-incidental (F21): out-sets order by
  `(dst, rank, age)`, in-sets by `(src, rank, age)` — **traversal determinism
  only**. Every eviction *selection* compares by the F17 eviction key
  `(rank, age, src, dst)`, never by adjacency-set order: an implementation
  taking `.iter().next()` from an adjacency set picks a deterministic but WRONG
  minimum (F37). (`Provenance` *output* order stays `(overlay, edge)`, F21 — a
  reporting sort, distinct from the selection key.)
- **Ownership (D1):** core owns the mechanism; **policy** owns channel meaning,
  classification, rendering; **adapter** owns the doctrine-id↔NodeId map and the
  `age` ordinal. `evaluate` is caller-driven (policy passes spec+seed per channel)
  — the graph holds **no** channel registry, stays stateless w.r.t. channel meaning.

### 5.4 Lifecycle, Operations & Dynamics

`build()` runs four deterministic passes; all overlay/layer references are by
opaque `OverlayId` from the policy-supplied `OrderSpec` — no dep/seq names (F1):

1. **Arity enforcement (F7, corrected F19, keep key totalized F36).** For each
   `AtMostOne` overlay, a node with >1 incoming edge keeps the
   **`(rank, age, src, dst)`-maximal** incoming edge — the keep key is the F17
   total key, not bare `(rank, age)`, so two parents tying on `(rank, age)`
   resolve deterministically by `src` (D4: higher rank wins the keep); the rest
   are evicted lowest-key-first → `EvictedEdge{ArityViolation}` in provenance
   (deterministic, surfaced not silent). Round 1 said *minimal* — backwards against
   D4/D5, where eviction always removes the weakest. This makes `spine_path`
   single-valued by construction. Arity resolution is deliberately orthogonal to
   `cycle_policy` (F27): it is a *resolution* with provenance, not a validation —
   a `Reject` overlay still resolves arity (no new policy axis under DD4's floor;
   revisit if a consumer needs degrade-on-arity).
2. **Per-overlay cycle resolution (D5/REQ-092).**
   - `Reject`: detect SCCs **on the authored, pre-arity edge set** (F30), no
     mutation; each cyclic component (multi-node SCC **or
     self-loop**, F20) → `CycleDiagnostic`, marked **degraded** — never a false
     order (REQ-076). `build()` still returns `Ok` (cycle is data). Pre-arity
     matters: pass-1 eviction can break an authored cycle before detection
     (`AtMostOne`+`Reject`: `a→b` rank 1, `c→b` rank 2, `b→a` — arity keeps
     `c→b`, evicts `a→b`; the post-arity set is acyclic), and post-arity
     detection would silently relabel an authored hard cycle as a routine
     `ArityViolation`, losing D5's "authoring error to fix" diagnostic (REQ-076
     surfacing). Two cycle concepts, two jobs (F30 narrowed by F46): the
     **authored SCC drives the `CycleDiagnostic`** — the authoring error is
     always surfaced; **order degradation (pass-3 intra-SCC exclusion + pass-4
     taint seeding) keys to the post-arity SCCs** — only what is *still* cyclic
     in the traversal view degrades order. Round 3's authored-SCC-everywhere
     rule excluded surviving valid resolved edges from `U` (the example above:
     `b→a` survives arity, the post-arity view `c→b→a` is a clean DAG, yet
     `b→a` was dropped and `ordered()` could invert it) — avoidable destruction
     of real precedence, not useful conservatism. Post-arity SCCs are sub-SCCs
     of authored ones (eviction only removes edges), so the diagnostic always
     covers the degraded set. F27's arity-resolves-under-Reject stands;
     `reachable`/`evaluate` still read the post-arity traversal view.
   - `Evict`: while a cyclic component exists (self-loops included, F20), evict the
     **globally-minimal participating edge** by the total key `(rank asc, age asc,
     src, dst)` (F17) — participating = belonging to any cyclic SCC; disjoint
     cycles each lose their own minima across iterations — recompute, repeat.
     Unique min (total order) → deterministic; each eviction strictly reduces
     edges → terminates ≤ `|E|`. Every eviction → `EvictedEdge{IntraOverlayCycle}`.
3. **Cross-layer composition into `U` (D9 — earlier layer authoritative) (F2,
   rewritten F10).** Build the **order structure `U`** — a DAG separate from the
   overlay edge sets (I7/F18). Walk the `OrderSpec` layers in precedence order; for
   layer *k*, orient its overlay's resolved edges per the layer direction and
   insert them **all** into `U`; then, while `U` contains a cycle, evict the
   `(rank asc, age asc, src, dst)`-minimal **layer-k** edge participating in any
   cycle → `EvictedEdge{UnionCycleVsLayer}`; repeat to fixpoint before the next
   layer. Any new cycle must contain a layer-k edge (`U` was acyclic before the
   layer), so an evictable edge always exists; earlier-layer edges are never
   evicted (authority). Round 1's pairwise check ("reversal already implied by
   layers <k") was unsound — layer-k edges individually consistent with the prior
   closure can jointly close a cycle (prior `a→b`; layer-k `{b→c, c→a}`);
   batch-insert + SCC eviction catches the composite case, is iteration-order-free
   (global min over the participating set), and terminates (each eviction removes
   an edge). Exclusion of a Reject overlay's degraded edges from `U` is
   **intra-SCC only** (F32): an edge is withheld iff **both endpoints lie in the
   same degraded post-arity SCC (F46)**; boundary-crossing edges enter `U`
   normally so taint can reach dependents — for `a↔b` degraded plus `b→c`,
   dropping every SCC-incident edge would hand `c` a clean `Finite` level while
   its position depends on the cyclic component (falsification — the exact hole
   F12's taint exists to plug). Pass-3
   eviction removes an edge from `U` **only** — the overlay edge sets that
   `reachable`/`evaluate` read are untouched (I7): a cross-layer *ordering*
   conflict must never mutate a *channel* value on an overlay that is itself
   valid. (D9's "dep authoritative, seq yields" is the 2-layer instance.)
4. **`order_key` materialization (D7/REQ-077, rewritten F11/F12, re-rewritten
   F31/F33).** `U` is acyclic post-pass-3, so the longest-path recurrence
   (`level(n) = 0` if no `U`-predecessor, else `1 + max over preds`) is **total
   over all nodes** — no sentinel, no overflow (F12). Taint is a separate mark
   deciding only the `Level` *variant*. **Taint seeds** = every node of a
   degraded **post-arity** SCC (F46) of an overlay **the `OrderSpec` references** (F31 — a cycle in
   an overlay you are not ordering by must not destroy the order you are: pass 2
   still diagnoses it, but it contributes neither edges nor taint to the order);
   taint then propagates to every `U`-descendant of a seed (their positions
   depend on the cyclic part — linearizing them as clean would falsify).
   Tainted → `Degraded(level)`, clean → `Finite(level)`; `Degraded(_) >
   Finite(_)` (sound: no surviving `U` edge runs tainted→clean), `(level,
   NodeId)` within each variant → total. `ordered()` stays total and respects
   every surviving `U` edge **including inside the degraded suffix** (F33 —
   round 2's bare-`NodeId` suffix violated a surviving clean-layer edge between
   two degraded nodes: Reject `a↔b` degraded, clean layer contributes `b→a`
   surviving in `U`, suffix `a,b` contradicted it and I2 with it). The suffix is
   *presence + surviving-edge order*, never a claimed linearization of the
   excluded cyclic constraints (REQ-076); degradation is visible in `order_key`
   itself, the causing SCCs in `Provenance.cycles`.

`explain(n)` (D11 — always walks to root) assembles **structured paths only**,
role-agnostically (F13 — round 1's `blockers`/`spine` fields leaked policy roles
into the core's own public structs):

```rust
pub struct Explanation { node: NodeId, order_key: OrderKey,
    paths:   BTreeMap<OverlayId, Vec<Vec<NodeId>>>,  // transitive predecessor chains, per overlay.
                                                     // A chain ends at a root OR at the first node
                                                     // of a degraded post-arity SCC (F47) — degraded
                                                     // SCC members are chain ENDPOINTS only, never
                                                     // walked through (finite + deterministic on
                                                     // cyclic Reject views; a node inside an SCC
                                                     // gets [[n]]); the cycle itself is explained
                                                     // by Provenance.cycles, not a path
    evicted: Vec<EvictedEdge> }      // evictions with n as an endpoint (src or dst) — F26
pub struct Channel { values: BTreeMap<NodeId, ChannelValue>,
    contributors: BTreeMap<NodeId, BTreeSet<NodeId>>,   // Any→witnesses · All→false: the present-false
                                                        // seeds; true: the present-true set (F43) ·
                                                        // Max→argmax · Count→the set;
                                                        // tie-break = min NodeId among maximal (F21)
    diagnostics: Vec<ChannelDiagnostic> }   // sorted by NodeId (F16); at most one per node —
                                            // UnknownSeedNode wins over variant checks (F41)
pub struct Provenance { cycles: Vec<CycleDiagnostic>, evictions: Vec<EvictedEdge> }
                       // both sorted by (overlay, edge) — never detection order (F21);
                       // eviction SELECTION uses the F17 eviction key, never
                       // adjacency-set order (F37)
pub struct EvictedEdge { overlay: OverlayId, edge: EdgeRef, reason: EvictReason }
pub enum   EvictReason { ArityViolation, IntraOverlayCycle, UnionCycleVsLayer }
pub struct ChannelDiagnostic { node: NodeId, reason: ChannelDiagReason }
pub enum   ChannelDiagReason { SeedVariantMismatch { expected: ValueKind, actual: ValueKind },
                               UnknownSeedNode }
pub enum   ValueKind { Flag, Scalar, Count }   // ChannelValue discriminant (F40). Combinator NOT
                                               // carried: Channel is per-evaluate; the spec is in
                                               // the caller's hand (F40 partial)
```

No `String` prose, no channel name, no doctrine noun anywhere in these — rendering
is policy's (D1). A policy that wants a "spine" labels one `AtMostOne` overlay's
path itself; the core privileges none (`spine_path` remains the DD1 ergonomic
accessor, not an `Explanation` field).

### 5.5 Invariants, Assumptions & Edge Cases

- **I1 (restated F12, tightened F48).** The composed order structure `U` and
  every `Evict` overlay's resolved edge set are acyclic post-`build()` — and
  ONLY those. A `Reject` overlay's traversal view **may remain cyclic** — its
  cycles are diagnosed and degraded, never linearized, and `reachable`/
  `evaluate`/`explain` are cycle-safe over it by definition (F12/F47); round 3's
  "acyclic is a property of resolved order/traversal views" over-claimed.
- **I2 (generic, post-F1/F11; made literally true F33).** An earlier `OrderSpec`
  layer is never evicted against by a later one, and every edge surviving into
  `U` is respected by `ordered()` — **including edges between degraded nodes**:
  the degraded suffix orders by surviving-`U` level, not bare `NodeId`. (D9's
  "dep authoritative, seq yields" is the 2-layer case; the core states it
  without naming dep/seq.)
- **I3.** Recompute from identical inputs → identical `order_key`, `Channel`,
  `Provenance` (REQ-077).
- **I4.** No authored mutation — eviction is a build-time derived resolution; inputs
  are consumed, never written back (storage rule, D8).
- **I5 (F3, rewritten F16, extended F35/F41/F45 — seed contract).** Each
  `Combinator` owns seed/output
  domains (`Any`/`All`→`Flag`, `Max`→`Scalar`, `CountDistinct`→`Flag`-in/
  `Count`-out). A missing seed contributes nothing; a node whose fold set holds no
  present seed is absent from `Channel.values`; a mismatched-variant seed → a
  deterministic `ChannelDiagnostic`, treated as absent; a foreign-`NodeId` seed
  entry → `ChannelDiagnostic{UnknownSeedNode}`, ignored (F41). `CountDistinct`
  counts `Flag(true)` seeds only — present-all-false folds to `Count(0)`, real
  data (F45). `Direction::None` is per-combinator: `Any`/`All`/`Max` → own
  present variant-valid seed; `CountDistinct` → always absent (F35). No
  combinator identity
  ever appears as an output value — `Scalar(i64::MIN)` in `values` is always real
  data. (Round 1's silent identity-collapse hid policy bugs and was
  indistinguishable from legitimate extremes.)
- **I6 (F8 — reachable is strict).** `reachable(n)` excludes `n`; `CountDistinct`
  therefore counts strict-reachable contributors, never `n`'s own seed.
- **I7 (F18 — eviction scope).** Pass-3 (`UnionCycleVsLayer`) evictions remove
  edges from `U` only; the per-overlay edge sets feeding `reachable`/`evaluate`
  are untouched. Channel values are invariant under presence/shape of `OrderSpec`.
- **Assumption A1 (demoted, F17).** `age` total + stable across recomputes remains
  the adapter's *semantic* contract, but core determinism no longer depends on it —
  the `(rank, age, src, dst)` eviction key is total core-internally.
- **Edge cases:** empty graph; single node no edges; self-loop (**always cyclic**,
  F20 — reject diagnoses, evict drops; never "trivial"); disjoint cycles (each
  loses its own min edge); a later-layer edge set that only closes a cycle against
  a higher-precedence layer → loses its own minimum (no-op for earlier layers);
  >1 parent on `AtMostOne` → keeps the `(rank, age, src, dst)`-max — equal
  `(rank, age)` parents resolve by `src` (F36), others `ArityViolation`;
  authored cycle broken by arity on a `Reject` overlay → `CycleDiagnostic` still
  emitted (F30) but order NOT degraded — degradation keys to post-arity SCCs, so
  surviving resolved edges stay in `U` (F46); cycle in a `Reject` overlay **outside** the
  `OrderSpec` → diagnostic emitted, order untouched, nothing `Degraded` (F31);
  degraded SCC with an outbound edge (`a↔b`, `b→c`) → `c` `Degraded` (F32);
  degraded nodes joined by a surviving clean-layer `U` edge → suffix respects it
  (F33);
  sparse/empty seed maps (absence, not identity); `Scalar(i64::MIN)` seed (real
  data, F16); all-present-false `Flag` fold set → `Count(0)`, not absent (F45);
  foreign `NodeId` seed entry → `UnknownSeedNode`, ignored (F41); duplicate
  identical edges (set semantics dedupe); empty `OrderSpec`
  (pure-`NodeId` order); queries with a foreign id (empty/`None`, non-panicking,
  F14); `Direction::None` in a `ChannelSpec` → per-combinator (F35):
  `Any`/`All`/`Max` emit the own present variant-valid seed, `CountDistinct`
  always absent; `reachable(_, None)` = ∅ (F25).

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
  Round-2 addition (F19-adjacent): SPEC-001 D9/D10's phrase "seq *rank* within a
  dep-eligible set" reads as *seq-topology* refinement — `rank` is
  conflict-resolution strength (eviction), never an `order_key` input; a post-lock
  SPEC wording note, same channel as T1.

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
  reachable set. Round-2 refinement (F15): the *semantic* contract for every
  combinator is the fold over the reachable closure; the topo fold is the
  acyclic-case implementation, `CountDistinct`'s set-union accumulator the general
  one — "one mechanism" stated honestly. Round-3 refinement (F34): the fold *set*
  splits per combinator class — inclusive `{n} ∪ reachable` for the idempotent
  combinators, strict `reachable` for `CountDistinct` — one fold, two set
  conventions; the universal single formula was false at self-inclusion and SCCs.
- **DD3 — hand-roll, no graph dependency.** `petgraph` would cover ~15% (single-graph
  topo/SCC) while the bulk — multi-overlay orchestration, eviction-to-fixpoint, D9
  union, monoid propagation — is custom regardless; a dep adds a determinism audit and
  a diagnostic-shape mismatch (REQ-076 wants node-ids+edge-kinds). Small corpus removes
  the scale argument.
  - **Reconcile (2026-06-11, post-close).** The "small corpus removes the scale
    argument" sentence is **void** — SPEC-001 H1 was revised from tens–hundreds to
    ~tens of thousands of nodes. The *decision* (hand-roll) still holds, re-grounded:
    the eviction-to-fixpoint / multi-overlay / D9 / propagation bulk is custom at any
    scale, and a linear **iterative** SCC/topo is std-only and cheap at tens of
    thousands. But the recursive implementations shipped here (`Tarjan::strongconnect`,
    `level_of`) now violate the linear+iterative premise — see RSK-003 (stack overflow)
    and RSK-002 (explain exponential), gated by the perf spike before any fix.
- **DD4 — full engine, combinator vocab floored.** *Alt rejected:* defer propagation
  (B) — fractures one coherent leaf and the mechanism is fully fixture-testable now.
  The REQ-079 vocabulary-free suite is the validating consumer.

## 8. Risks & Mitigations

- **R1 — hand-rolled topo/SCC bug.** *Mit:* small textbook algorithms + the explicit
  cycle fixtures are mandated acceptance tests; determinism re-run check; property
  tests against a naive independent oracle for SCC/topo (F24).
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
| **REQ-076** reject | `Reject`-overlay cycle → `CycleDiagnostic` names nodes+edges, SCC degrades, remainder orders. (Overlay-neutral fixture vocabulary, F42 — dep/seq stay in policy docs.) |
| **REQ-092** evict | `Evict`-overlay cycle → min-eviction-key edge evicted to fixpoint, in provenance. |
| **REQ-077** determinism | build twice → identical `order_key` + `Provenance` + contributor traces; union fixture `a —x→ b` (earlier layer), `b —y→ a` (later layer). |
| **REQ-080** seam | a fresh channel via existing combinators works with no core change; `Combinator` doc-marked as the curated extension point. |
| **DD1 rollup (F5)** | `Unbounded` membership overlay, a node with **2 parents**, `Against`-direction `All`/`CountDistinct` → aggregates from both parents correctly; `spine_path` on an `AtMostOne` overlay returns the single kept path; `CountDistinct` over a diamond counts the distinct node once (R3). |
| **arity (F7/F36/F39)** | >1 parent on `AtMostOne` → total-key **max** kept (F39 fixed the stale "min"), rest `EvictedEdge{ArityViolation}`; equal-`(rank,age)` parents resolve by `src` (F36). |
| **arity×reject pipeline (F30/F46)** | `AtMostOne`+`Reject`, arity breaks the authored cycle → `CycleDiagnostic` still emitted alongside the `ArityViolation`; post-arity view acyclic → nothing `Degraded`, `ordered()` respects every surviving resolved edge (`c,b,a` for the F30 example). |
| **explain on cycles (F47)** | `Reject` `a↔b` + `a→x`: `explain(x)`'s predecessor chain ends at `a` (degraded-SCC member = endpoint only, never walked through); `explain(a)` paths = `[[a]]`; terminates, deterministic, cycle reported via `Provenance.cycles`. |
| **degrade scope (F31)** | cycle in a `Reject` overlay not in the `OrderSpec` → clean spec order preserved, diagnostic emitted, nothing `Degraded`. |
| **taint crossing (F32)** | `a↔b` (`Reject`, in spec) + `b→c` → `c` `Degraded`; intra-SCC edges absent from `U`. |
| **suffix order (F33)** | degraded `a`,`b` + surviving clean-layer `b→a` → suffix orders `b` before `a` (`U` level, not `NodeId`). |
| **CountDistinct set (F34/F45)** | seeded single node + `Direction::None` → absent; SCC `a↔b`, seed on `a` only → per-member values per the strict contract; all-`Flag(false)` fold set → `Count(0)` ≠ absent. |
| **None per combinator (F35)** | `Max`+`None` → own seed; `CountDistinct`+`None` → absent. |
| **eviction key (F37)** | fixture where adjacency-set min ≠ eviction-key min → the eviction-key-minimal edge is evicted. |
| **foreign seed (F41)** | unknown `NodeId` in seed map → `ChannelDiagnostic{UnknownSeedNode}`, ignored, deterministic. |
| **union (F2)** | 3-layer `OrderSpec` where a layer-2 edge contradicts layer-0 → `EvictedEdge{UnionCycleVsLayer}`, layer-0 order preserved. |
| **union composite (F10)** | prior `a→b`; layer-k `{b→c, c→a}` → exactly one layer-k edge evicted (total-key min), `U` acyclic, no overflow / non-termination. |
| **refinement (F11)** | layer-0 `a→b` with `c` incomparable; layer-1 `b→c` → order `a,b,c` (a surviving later-layer edge is never violated). |
| **degraded taint (F12)** | reject-SCC + clean successor → successor `Degraded` (no `1+MAX` overflow); `ordered()` total, degraded suffix by `NodeId`. |
| **self-loop (F20)** | under both policies: reject diagnoses, evict drops. |
| **seed contract (F16)** | `Scalar(i64::MIN)` seed ≠ absence; mismatched variant → `ChannelDiagnostic`; sparse seed map; no fabricated identity in `values`. |
| **eviction scope (I7/F18)** | byte-identical channel values with and without an `OrderSpec` that evicts. |
| **ties (F21)** | equal `Max` seeds → min-`NodeId` argmax; `contributors` + `Provenance` ordering pinned. |
| **determinism+ (F24)** | edge-insertion-order permutation → byte-identical outputs; SCC/topo property-tested against a naive oracle; forbidden-vocabulary denylist scan over `crates/cordage/**` (code, docs, tests). |

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

### Adversarial external review (round 2) — GPT-5.5 + Opus

Two independent external passes against the round-1 design: GPT-5.5 (41 findings:
6 blocker / 16 significant / 15 moderate / 4 minor; cited `G-b/s/m/n` + list
position) and Opus (14 findings, cited `O-R2-xx`). Heavy overlap; deduped to 20
integrated (F10–F29, continuing round-1 numbering) + 3 rejected. One additional
blocker (F11) was found by us while integrating F10 — neither reviewer landed it.

- **F10 (blocker, union soundness) — FIXED.** [O-R2-01; G-b3, G-b4, G-m37]
  Pairwise "reversal implied by layers <k" provably misses composite cycles
  (prior `a→b`; layer-k `{b→c, c→a}`) and was iteration-order-dependent. Pass 3
  rewritten: batch-insert the layer into `U`, SCC-detect, evict the
  total-key-minimal layer-k participating edge to fixpoint. (§5.4 pass 3.)
- **F11 (blocker, ordering soundness — self-found during integration).** The
  round-1 lexicographic per-layer level tuple violated surviving later-layer
  edges: level equality in an earlier layer ≠ incomparability (layer-0 `a→b`,
  isolated `c`; layer-1 `b→c` → tuple sorts `c` before `b` though nothing
  conflicts). `OrderKey` = (longest-path level in `U`, `NodeId`); layer
  precedence lives in pass-3 eviction authority, not key position. (§5.2, §5.4
  pass 4, I2.)
- **F12 (blocker, degraded model) — FIXED.** [O-R2-02; G-b2, G-s16, G-s17,
  G-m26, G-m27] `u32::MAX` sentinel overflowed `1+max` in clean successors; I1
  contradicted Reject's no-mutation; `reachable`/`evaluate`/`ordered` on degraded
  overlays undefined. Now: `Level::{Finite, Degraded}` enum + downstream taint,
  I1 restated over resolved views, `reachable` cycle-safe by definition,
  `evaluate` total via closure semantics (F15), `ordered()` total with a
  documented degraded suffix. (G-s16's exclude-and-diagnose and G-s17's
  separate-partition alternatives rejected in favour of closure totality —
  cycles degrade order, they do not falsify reachability.) (§5.4 pass 4, I1.)
- **F13 (blocker, boundary) — FIXED.** [O-R2-03, O-R2-09, O-R2-10; G-b6, G-m33,
  G-m34, G-m35] `Explanation.blockers` + "dep chains" comment = Appendix-B leak
  round 1 missed in the core's own structs; `Direction::{Backward, Upward}`
  domain-flavoured; `Explanation.spine` re-privileged the DD1 policy role. Now
  `paths: BTreeMap<OverlayId, …>`, `Direction::{Along, Against, None}` defined by
  adjacency index, spine field gone (`spine_path` accessor stays per DD1);
  denylist scan added to §9. (§5.2, §5.4, §9.)
- **F14 (blocker, API contract) — FIXED.** [G-b1, G-m23, G-m24] `Result<Graph>`
  vs "cycles still Ok" ambiguity: `Err` = malformed input only (unknown ids,
  duplicate layer, `None` layer, overlay cap); degradation is Provenance data;
  foreign-id queries defined (empty/`None`). (§5.2.)
- **F15 (significant, semantics) — FIXED.** [O-R2-04; G-s12] "One topo fold" was
  false for `CountDistinct` (secretly computed via `reachable`). Semantic
  contract now: fold of present seeds over the reachable closure, all
  combinators; topo fold = acyclic-case implementation; `CountDistinct` =
  set-union accumulator (genuine monoid) → `Count` at read; condensation extends
  totality over degraded overlays. DD2 refined, not overturned. (§4, §5.2, §7.)
- **F16 (significant, seed contract) — FIXED.** [O-R2-05, O-R2-11; G-s13, G-s14,
  G-s15] Identity-collapse hid policy bugs; `i64::MIN` ambiguity; `All`
  vacuous-true; missing-seed undefined. Now absence semantics (no identity ever
  escapes into `values`), mismatch → `ChannelDiagnostic`; I5 rewritten. (§5.2, I5.)
- **F17 (significant, determinism) — FIXED.** [O-R2-08; G-s9] `(rank, age)` ties
  → non-deterministic min, determinism hostage to adapter contract A1. Total key
  `(rank, age, src, dst)`; A1 demoted to semantic contract. (§5.2, §5.4, A1.)
- **F18 (significant, eviction scope) — FIXED.** [O-R2-06] Pass-3 eviction scope
  was unspecified; removing from the overlay edge set would have let an ordering
  conflict mutate channel values. New I7: `U`-only; channel values invariant
  under `OrderSpec`. (§5.4 pass 3, I7, §9.)
- **F19 (significant, rank direction) — FIXED.** [G-s19; verified against
  SPEC-001 D4/D5] Internal contradiction: pass 1 *kept* the `(rank, age)`-min
  parent while D4 says higher rank = stronger and D5 evicts the min. Pass 1 now
  keeps the max. (§5.4 pass 1.)
- **F20 (significant, self-loops) — FIXED.** [O-R2-07; G-m25] "Trivial SCC"
  wording would have let self-loops through undiagnosed (then `level(n) =
  1 + level(n)`). Self-loops are always cyclic; fixtures under both policies.
  (§5.4 pass 2, edge cases, §9.)
- **F21 (significant, output determinism) — FIXED.** [G-s10, G-s21, G-s22;
  O-R2-14] `Edge` `Ord`, `contributors` ordering, witness/argmax ties,
  `Provenance` vec ordering all pinned to explicit stable keys. (§5.3, §5.4, §9.)
- **F22 (moderate, OrderSpec validation) — FIXED.** [G-s8; O-R2-13]
  Unknown/duplicate/`Direction::None` layers → build error; empty spec =
  pure-`NodeId` order. (§5.2.)
- **F23 (moderate) — FIXED.** [G-s7] `spine_path` → `Option`; `None` off
  `AtMostOne` overlays. (§5.2.)
- **F24 (moderate, verification) — FIXED.** [O-R2-12; G-m30, G-m31, G-m32] §9
  rows added: union composite, refinement, degraded taint, self-loops, seed
  contract, eviction scope, ties, edge-permutation determinism, naive-oracle
  property tests, denylist scan. (G-m31 partial: node-insertion order is input
  identity — the valid determinism test is edge-insertion permutation.)
- **F25 (moderate) — FIXED.** [G-m28] `Direction::None` contract: seed-only
  channel, empty reachability, forbidden in `OrderLayer`. (Edge cases.)
- **F26 (clarify) — FIXED.** [G-s20] "Evictions touching n" = n is an endpoint
  (src or dst). (§5.4.)
- **F27 (partial) — DOC.** [G-b5] Arity-resolution-under-Reject kept ("silently
  rewriting" premise false — eviction is surfaced in provenance; no new policy
  axis under DD4's floor) but now documented as deliberately orthogonal, with the
  revisit trigger. (§5.4 pass 1.)
- **F28 (partial) — DOC.** [G-s11] Single-overlay channels stated as the v1
  contract; multi-overlay = policy composition or a REQ-080 extension. (§5.2.)
- **F29 (minor batch) — FIXED.** [G-n38, G-n39, G-n40, G-n41] Id caps +
  monotonic/no-deletion documented; accessor return shape deferred to impl
  (OQ-1); core diagnostics carry overlay ids + edge refs ("edge kinds" is
  SPEC-quoted requirement language, adapter remaps).

Rejected:

- [G-m29] `Count(u32)` overflow — count ≤ |V| ≤ the `u32` domain by `NodeId(u32)`
  construction; cannot overflow.
- [G-m36] `reachability_trace()` — `contributors` already carries the per-node
  contributing set (deterministic post-F21); full path traces are a REQ-080
  extension with no consumer.
- [G-s18] rank-in-`order_key` — `rank` is conflict-resolution strength only;
  ordering comes from edge topology. SPEC-001 D9/D10 "seq *rank*" wording flagged
  upstream instead (§6).

Net: two external blockers were real algorithm bugs (F10 composite-cycle miss,
F12 sentinel overflow), one internal contradiction was confirmed against SPEC-001
(F19), and one further blocker surfaced during integration (F11 — the per-layer
tuple itself). The public interface changed again (Direction names, `OrderKey`
shape, `Explanation`/`Channel` shapes, build error contract). No DD overturned;
DD2 refined (F15), DD1 reinforced (F13). Design stands, pending user sign-off.

### Adversarial external review (round 3, final) — web + GPT-5.5 + Opus

Three external passes, pre-deduped by the user to 15 combined findings
(`R3-01`–`R3-15`, source map supplied with the review). All 15 accepted (2
partial, 1 with an alternative fix), 0 rejected, none duplicates of F1–F29 or
bare re-raises of the notes.md known-open items. One further finding (F45) was
self-found while re-deriving the F34 machinery — the round-2 pattern (F11)
repeating. Two genuine blocker *families* neither earlier round caught: the
pass-1→pass-2 pipeline interaction (F30) and the Degraded/taint mis-scoping
(F31/F32/F33).

- **F30 (blocker, pipeline order) — FIXED.** [web R3-01] Arity eviction (pass 1)
  can break an authored cycle before Reject detection (pass 2) sees it
  (`a→b` r1, `c→b` r2, `b→a`): the authored hard cycle was silently relabelled a
  routine `ArityViolation` — no `CycleDiagnostic`, violating D5's "authoring
  error to fix, surfaced" (verified against SPEC-001 D5/REQ-076). Fix: Reject
  cycle detection runs on the **authored (pre-arity) edge set**; the authored
  SCC is the one cycle concept (diagnostic + degraded mark + pass-3 exclusion).
  Conservative when arity breaks the cycle — degrade-not-falsify permits that
  direction; F27 stands, traversal stays post-arity. (§5.4 pass 2, §9.)
- **F31 (blocker, taint scope) — FIXED.** [Opus] Pass-2 degraded marks fed
  pass-4 taint for **every** Reject overlay, OrderSpec member or not — a cycle
  in an overlay you are not ordering by destroyed the order you are (clean spec
  overlay `a→b→c` + foreign `a↔b` ⇒ everything `Degraded`). Fix: taint seeds
  only from degraded SCCs of overlays the `OrderSpec` references; non-spec
  cycles still diagnose, contribute nothing to order. (§5.4 pass 4, §9.)
- **F32 (blocker, exclusion ambiguity) — FIXED.** [web R3-02; Opus M6] "Edges of
  degraded SCCs never enter `U`" read naturally as all-incident-edges, which
  drops `b→c` for `a↔b`+`b→c` and hands `c` a clean `Finite` level — taint
  defeated, falsification. Fix: exclusion is intra-SCC only (both endpoints in
  the same degraded SCC); boundary-crossing edges enter `U` and carry taint.
  Fixture: `a↔b`, `b→c` ⇒ `c` degraded. (§5.4 pass 3, §9.)
- **F33 (blocker, suffix order / I2 contradiction) — FIXED.** [GPT R3-01; Opus
  M2] The bare-`NodeId` degraded suffix violated surviving `U` edges between
  degraded nodes (Reject `a↔b` + clean-layer `b→a` surviving ⇒ suffix `a,b`) —
  I2 was literally false. Fix: `Level::{Finite(u32), Degraded(u32)}`, both
  carrying the longest-path level in `U` (acyclic ⇒ total, no sentinel); taint
  decides the variant only; suffix orders `(level, NodeId)`. Sound because taint
  propagates downstream ⇒ no tainted→clean surviving edge; I2 now holds
  including inside the suffix. (§5.2, §5.4 pass 4, I2, §9.)
- **F34 (significant, contract split) — FIXED.** [web R3-03; GPT R3-02] The
  universal formula `value(n) = fold over {n} ∪ reachable(n)` was false for
  `CountDistinct` (I6 strict): self-inclusion (seeded single node under `None` ⇒
  `Count(1)` vs no contributor) and condensation ("SCC members share a value" —
  strict closures differ per member). Fix: contract split per combinator class —
  idempotent over the inclusive closure, `CountDistinct` over strict reachable;
  condensation corollary scoped to idempotent combinators over traversal-view
  SCCs. DD2 refined: one fold, two set conventions. (§4, §5.2, §7, §9.)
- **F35 (significant, None × CountDistinct) — FIXED.** [GPT R3-04; web R3-04]
  "`None` ⇒ value = own present seed" broke `CountDistinct`'s `Flag`-in/
  `Count`-out domain (three incompatible readings). Fix: `None` is
  per-combinator — `Any`/`All`/`Max` emit the own present variant-valid seed;
  `CountDistinct` is always absent (strict fold set empty). (§5.2, I5, edge
  cases, §9.)
- **F36 (significant, keep-key totality) — FIXED.** [GPT R3-03; web R3-05]
  Pass-1 keep was stated as bare `(rank, age)`-max — two equal parents left the
  survivor traversal-dependent. Fix: keep = `(rank, age, src, dst)`-maximal
  incoming edge (the F17 total key); equal-`(rank,age)` tie fixture added.
  (§5.4 pass 1, edge cases, §9.)
- **F37 (significant, key vs storage order) — FIXED.** [web R3-06] Adjacency
  sets order by `(dst|src, rank, age)` while eviction needs `(rank, age, src,
  dst)` — `.iter().next()` on a participating set is deterministic but
  wrong-minimal. Fix: the eviction key is named the sole comparator for every
  eviction *selection*; adjacency order is traversal-only; `Provenance` output
  order stays `(overlay, edge)` per F21 (reporting sort ≠ selection key);
  determinism-suite row added. (§5.2, §5.3, §9.)
- **F38 (significant, provenance attribution) — FIXED, alternative fix.** [web
  R3-07] One overlay could appear `Along` and `Against` in one `OrderSpec` (only
  duplicate `(overlay, direction)` was malformed), producing two `U` arcs per
  `EdgeRef` that `EvictedEdge{overlay, edge, reason}` cannot distinguish.
  Reviewer offered enrich-or-forbid; chose **forbid**: an overlay appears in at
  most one layer, any direction (build `Err`, strengthens F22). Simpler, keeps
  provenance shape; bidirectional ordering of one overlay has no consumer and is
  a REQ-080 reopen if ever wanted. (§5.2.)
- **F39 (moderate, stale text) — FIXED.** [GPT R3-05; web R3-10] §9 arity row
  still said "min kept" — a literal implementer of the acceptance table would
  rebuild the bug F19 fixed, and the test would bless it. Row now says total-key
  max + tie coverage. (§9.)
- **F40 (moderate, diagnostic shape) — PARTIAL.** [web R3-08]
  `ChannelDiagnostic{node, SeedVariantMismatch}` carried no expected/actual.
  Accepted: `SeedVariantMismatch { expected: ValueKind, actual: ValueKind }`
  (`ValueKind` = value discriminant). Rejected the `combinator` field: `Channel`
  is the result of one `evaluate(spec, …)` call — the spec is already in the
  caller's hand; carrying it duplicates call context into every diagnostic.
  (§5.4.)
- **F41 (moderate, foreign seeds) — FIXED.** [web R3-09] Seed-map entries with
  unknown `NodeId`s were unspecified (queries were defined, F14; seeds were
  not). Fix: deterministic `ChannelDiagnostic{UnknownSeedNode}`, entry ignored;
  at most one diagnostic per node, `UnknownSeedNode` wins over variant checks.
  (§5.2, I5, §9.)
- **F42 (moderate, fixture vocabulary) — FIXED.** [web R3-11] §9 rows narrated
  with dep/seq while claiming a vocabulary-free suite — acceptance evidence that
  reads as implementation guidance re-imports policy vocabulary. Rows rewritten
  overlay-neutral (`Reject`/`Evict` overlay, layers `x`/`y`); REQ ids retained
  (requirement references, not fixture vocabulary). (§9.)
- **F43 (minor, contributors gap) — FIXED.** [Opus] `contributors` was undefined
  for `All`. Pinned: `false` → the present-false seeds (falsifiers); `true` →
  the present-true set; ordering structural as for the others. (§5.4.)
- **F44 (minor, wording hygiene) — PARTIAL.** [web R3-12] "stronger preference"
  in core-contract comments leans semantic. Reworded mechanically ("higher rank
  wins keeps; lower rank evicted first"). Checked against SPEC-001 Appendix B:
  "preference" is NOT on the forbidden list — and D4 itself says "stronger
  preference" — so this is hygiene, not a REQ-079 violation; the denylist scan
  is unchanged. (§5.2, §5.4 pass 1.)
- **F45 (significant, self-found integrating F34/F35).** `CountDistinct`'s
  treatment of a present `Flag(false)` seed was unpinned — "fold of PRESENT
  seeds" implied false seeds count, the set-union framing implied they don't,
  and `Count(0)`-vs-absent was undefined. Pinned: `Flag(true)` contributes its
  node, `Flag(false)` contributes nothing; an all-false fold set yields
  `Count(0)` (real data — "seeded, zero distinct"), absence stays
  nothing-seeded. Symmetric with `Any` emitting `Flag(false)` over all-false
  seeds; F16's no-identity-escapes rule intact. (§5.2, I5, §9.)

Net: round 3 was NOT diminishing — four blockers, all in round-2's freshest
machinery (the pass pipeline and the Degraded/taint model), exactly where the
reviewer prompt pointed. Both blocker families are interaction bugs between
individually-reviewed parts, invisible to per-section review. `OrderKey` changed
shape again (`Degraded(u32)`); `OrderSpec` validation tightened (F38);
diagnostics enriched (F40/F41). No DD overturned; DD2 refined again (F34/F45).
Design stands, pending user sign-off.

### Adversarial external review (round 4) — GPT-5.5 via codex MCP

Run directly in-session against the round-3 integration (commit `2abb4e2`),
targeting the round-3 rewrites. 3 findings, all accepted, 0 rejected.

- **F46 (blocker, F30 over-correction) — FIXED.** [R4-01] Round 3's
  "authored SCC is the one cycle concept" over-applied to pass 3/4: with the F30
  example (`a→b` r1, `c→b` r2, `b→a`; arity evicts `a→b`), the surviving valid
  edge `b→a` was excluded from `U` for being intra-authored-SCC although the
  post-arity view `c→b→a` is a clean DAG — `ordered()` could invert a real
  resolved precedence edge. Not falsification (nodes were Degraded, surfaced)
  but avoidable destruction of valid order. Fix: split the jobs — the
  **authored** SCC drives the `CycleDiagnostic` (F30's actual point, intact);
  **order degradation** (pass-3 intra-SCC exclusion + pass-4 taint seeds) keys
  to **post-arity** SCCs, so only what is still cyclic degrades order.
  Post-arity SCCs ⊆ authored SCCs (eviction only removes edges) — the
  diagnostic always covers the degraded set. This was the exact alternative
  weighed and rejected for simplicity during F30 integration; the reviewer
  supplied the concrete cost that flips the call. (§5.4 pass 2/3/4, edge cases,
  §9.)
- **F47 (significant, explain totality) — FIXED.** [R4-02]
  `Explanation.paths` promised "predecessor chains to root" but a cyclic Reject
  traversal view has no root — the contract was impossible (loop, silent
  truncation, or omission). Fix: a chain ends at a root OR at the first node of
  a degraded post-arity SCC — SCC members are chain *endpoints*, never walked
  through; a node inside an SCC gets `[[n]]`; the cycle itself is explained by
  `Provenance.cycles`. Finite + deterministic on every input. (§5.4
  `Explanation`, §9.)
- **F48 (moderate, I1 over-claim) — FIXED.** [R4-03] I1's tail ("'acyclic' is a
  property of resolved order/traversal views") contradicted the design's own
  Reject semantics — a Reject traversal view may stay cyclic post-build. I1
  tightened to the actual guarantee: `U` + `Evict` resolved sets acyclic, and
  ONLY those; Reject views handled by cycle-safe traversal (F12/F47). (I1.)

Net: round 4 found no new machinery bugs — F46 is the cost of a round-3
integration *choice* (the simplicity-vs-precision call on F30, now reversed
with evidence), F47/F48 are contract gaps in surfaces the earlier rounds never
aimed at. Severity trend finally diminishing: blocker count 2→4→1 across
external rounds, and round 4's blocker required no new mechanism — only
re-keying two existing references from authored to post-arity SCCs.

### Lock

**Design locked by user sign-off, 2026-06-10** (post round 4, commit `96fe162`).
48 findings (F1–F48) integrated across four adversarial rounds; 0 DDs
overturned. Known-opens deliberately deferred to the first consumer
(notes.md "Round-4 outcome"): explanation path-enumeration combinatorics,
full-downstream taint extent, pre-consumer API churn. Subsequent changes to
this design require re-opening review, not silent edits.
