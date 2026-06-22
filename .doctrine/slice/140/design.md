# Design SL-140: Unify cordage traversal: reachable/spine_path/cone

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

IMP-020 identifies three traversal implementations in `crates/cordage/src/query.rs`
that re-assert the same visited-set / frontier BFS idiom independently:

- `reachable` — BFS outward, collecting a strict reachable set
- `spine_path` — a single-parent chain walk upward, collecting an ordered path
- `cone_on_overlay` — BFS upward, building a `node ↦ {preds}` adjacency map

The duplicated idiom is the visited-gates-enqueue, cycle-safe frontier loop. The
goal is to host that one invariant in a single place without bending any walk's
distinct output or termination semantics through indirection.

`extend_chains` (IMP-020's title) does not exist in the source — a historical or
never-written name. The three above are the active divergence.

## 2. Current State

The neighbour-lookup layer is *already* factored: `neighbours` (direction),
`predecessors` (overlay in-edges), `single_parent` (≤1 in-edge). The duplication is
the loop *around* those helpers, plus the cycle-safety invariant (F12/F47: a
visited set bounds the BFS over a degraded `Reject` view) re-asserted per function.

Callers (all internal): `Graph::reachable`, `Graph::spine_path`, and
`Graph::predecessor_cone` (→ `cone_on_overlay`) in `lib.rs`. `evaluate` does **not**
call `reachable` — channel propagation runs a separate condensation fold (RSK-004),
so it is out of scope.

| | loop shape | output | per-node record | start in output | termination extras | neighbour helper |
|---|---|---|---|---|---|---|
| `reachable` | frontier queue | set | membership | excluded | none | `neighbours` |
| `spine_path` | linear (≤1 succ) | ordered Vec | append | included | none | `single_parent` |
| `cone_on_overlay` | frontier queue | map node→predset | full pred-set | included (key) | degraded-SCC endpoints | `predecessors` |

The literally identical code is the `if visited.insert(x) { frontier.push_back(x) }`
gate plus the `while pop_front` shell (shared by `reachable` + `cone`; `spine` is a
degenerate linear case). Everything else differs by output shape and semantics.

## 3. Forces & Constraints

- **Behaviour-preservation gate** (project rule; RSK-010 base drift): cordage is a
  shared leaf under doctrine's priority engine. The existing suites are the proof —
  they must stay green *unchanged*. No public API change.
- **ADR-001** (module layering, cordage a leaf): the refactor stays leaf-internal.
- **Determinism**: `BTreeSet`/`BTreeMap`/FIFO iteration order must be preserved; no
  Hash-based collection may be introduced.
- **Project ethos**: "as simple as possible, but no simpler"; no parallel
  implementation; the abstraction must *reduce* the cost of a future change, not add
  indirection for its own sake.
- **Memory mem.pattern.review.interaction-bugs-hide-between-sound-parts**: cordage's
  historical blockers were interaction bugs between individually-sound parts. An
  over-general primitive splicing semantics across a closure boundary is exactly such
  a seam — a reason to keep the primitive narrow.

## 4. Guiding Principles

- Unify only what is genuinely the same shape. Hoist the *invariant*, not the
  superficial line-count.
- A walk whose distinct semantics cannot ride the primitive cleanly stays explicit
  and is *documented* as the deliberate exception, so a future maintainer does not
  "finish the job" and reintroduce the complexity.
- Equivalence of each rewrite must be argued against the original, then proven by the
  unchanged gate.

## 5. Proposed Design

### 5.1 System Model

One primitive, two thin callers, one documented exception:

- `walk_bfs(start, neighbours) -> Vec<NodeId>` — the single locus of the
  visited/frontier/cycle-safety invariant.
- `reachable` and `spine_path` become thin shapers over `walk_bfs`.
- `cone_on_overlay` keeps its explicit loop (it needs per-node pred-set *values* and
  expansion control at degraded-SCC entries — neither expressible through a
  discovery-order primitive without a heavier visitor+predicate abstraction that
  would obscure the SCC-endpoint logic). It shares the neighbour helpers as the reuse
  seam.

### 5.2 Interfaces & Contracts

The primitive — private to `query.rs`:

```rust
/// Breadth-first discovery order from `start` over the `neighbours` relation:
/// each reachable node yielded exactly once, `start` first. A FIFO frontier and a
/// visited set make it deterministic (given `neighbours` in adjacency-key order)
/// and cycle-safe over a degraded `Reject` view — the visited set bounds re-entry
/// (F12/F47). The shared loop behind `reachable` and `spine_path`.
fn walk_bfs<I>(start: NodeId, neighbours: impl Fn(NodeId) -> I) -> Vec<NodeId>
where
    I: IntoIterator<Item = NodeId>,
{
    let mut order = vec![start];
    let mut visited = BTreeSet::from([start]);
    let mut frontier = VecDeque::from([start]);
    while let Some(node) = frontier.pop_front() {
        for next in neighbours(node) {
            if visited.insert(next) {
                order.push(next);
                frontier.push_back(next);
            }
        }
    }
    order
}
```

Generic over `I: IntoIterator<Item = NodeId>` so `neighbours` may return a `Vec`
(reachable) *or* an `Option` (spine) with no wrapper allocation — the one
abstraction that fits both without a lowest-common-denominator type.

Public API on `Graph` (`reachable`, `spine_path`, `predecessor_cone`) is byte-for-byte
unchanged.

### 5.3 Data, State & Ownership

No new types, no new state. `walk_bfs` owns three locals (`order`, `visited`,
`frontier`); they do not escape. The neighbour helpers remain the shared lookup layer
under both the primitive and `cone`.

### 5.4 Lifecycle, Operations & Dynamics

**`reachable` — thin caller (strict set):**

```rust
pub(crate) fn reachable(
    out: &OutIndex,
    incoming: &InIndex,
    overlay: OverlayId,
    start: NodeId,
    direction: Direction,
) -> BTreeSet<NodeId> {
    walk_bfs(start, |node| neighbours(out, incoming, overlay, node, direction))
        .into_iter()
        .skip(1) // strict (I6/F8): drop `start`, always discovery index 0
        .collect()
}
```

Equivalence: discovery set = `{start} ∪ reachable`; `start` is never re-emitted
(visited seeded), so `skip(1)` removes exactly it, yielding the strict set.
`Direction::None ⇒ neighbours = ∅ ⇒ order == [start] ⇒ ∅` (F25 preserved).

**`spine_path` — thin caller (reversed chain):**

```rust
pub(crate) fn spine_path(incoming: &InIndex, overlay: OverlayId, node: NodeId) -> Vec<NodeId> {
    let mut chain = walk_bfs(node, |cur| single_parent(incoming, overlay, cur));
    chain.reverse(); // discovery is node→root; caller wants root→node
    chain
}
```

Equivalence: `single_parent` yields ≤1 node ⇒ discovery degenerates to the linear
chain `[node, parent, …, root]`; the visited set stops a surviving-`Reject` cycle at
re-entry exactly where the old `break` did; `reverse` gives ancestor-first. Output
identical.

**`cone_on_overlay` — explicit, with an added doc comment** marking it the deliberate
exception (records per-node pred-sets as map values; terminates at degraded-SCC
entries by recording-but-not-expanding) so the SCC-endpoint logic stays legible and
is not later forced onto `walk_bfs`. Loop body unchanged.

### 5.5 Invariants, Assumptions & Edge Cases

- `start` is `order[0]` and never re-emitted — the single fact from which both
  `reachable`'s strictness (`skip(1)`) and `spine`'s cycle-stop derive.
- Determinism: FIFO frontier + adjacency-key `neighbours` order ⇒ stable order; no
  Hash collection introduced.
- Cycle-safety (F12/F47): visited bounds re-entry on a surviving `Reject` cycle — now
  proven once for both riders instead of per-function.
- `cone` is untouched: its degraded-SCC early-out (`{node: {}}`) and endpoint
  recording are preserved verbatim.

## 6. Open Questions & Unknowns

None open. Resolved during design:

- OQ-1 (resolved): does `cone` ride the primitive? No — per-node pred-set values +
  expansion control don't fit discovery-order; forcing it needs a heavier abstraction
  that buys nothing cone doesn't already express. → D2.
- OQ-2 (resolved): does `spine` ride it or stay explicit? Rides it — `Option` is
  `IntoIterator`, so `single_parent` drops in with no adapter and the chain semantics
  stay legible via the neighbour fn. → D3.

## 7. Decisions, Rationale & Alternatives

- **D1 — Extract a discovery-order `walk_bfs`, not a visitor/fold primitive.** The
  honest shared shape across the two true BFS walks is "BFS discovery order." A
  fold/visitor primitive with an expansion predicate would be needed to also host
  `cone`, but that primitive is heavier and splits cone's trickiest logic across a
  closure boundary. *Alternative rejected:* one general `walk(neighbours, expand,
  visit)` for all three — negative ROI (more indirection to dedupe ~5 lines; cone
  still re-calls `predecessors` to build its map).
- **D2 — `cone_on_overlay` stays explicit, documented.** It records map *values* and
  blocks expansion at SCC entries; discovery-order can carry neither. Document the
  exception to prevent a future "unify all three" regression.
- **D3 — `spine_path` rides `walk_bfs`.** `single_parent: … -> Option<NodeId>` is
  `IntoIterator`; no adapter, cycle-stop preserved by visited, ancestor-order by
  `reverse`. One invariant locus over an explicit duplicate loop.

## 8. Risks & Mitigations

- **RSK-010 (base drift):** cordage is a shared leaf feeding doctrine's priority
  engine. *Mitigation:* behaviour-preservation gate — existing suites green,
  unchanged. The two rewrites are argued equivalent in §5.4.
- **Over-abstraction (interaction-seam risk):** per the cordage review memory, a
  primitive splicing semantics across a closure is where bugs hide. *Mitigation:*
  the primitive is narrow (discovery order only); `cone`'s semantics stay in one
  explicit place, not split.
- **Determinism regression:** *Mitigation:* no Hash collection introduced; FIFO +
  `BTree*` order preserved; `golden_net` / `scale_cliffs` cover it.

## 9. Quality Engineering & Validation

Behaviour-preservation only — **no new test assertions**:

- `reachability.rs` (8) — `reachable` across Along/Against/None, cycles, foreign
  overlay/node: covers the strict-set path and `None ⇒ ∅`.
- `explain.rs` (8) — `spine_path` (`AtMostOne` chain) and `predecessor_cone`: covers
  the spine rewrite and confirms `cone` untouched.
- `golden_net.rs` / `scale_cliffs.rs` — cycle-safety + determinism under degraded
  `Reject` views (the hoisted F12/F47 invariant).

No dedicated `walk_bfs` unit test: it is private and fully observed through the
public callers; a direct test would assert trivial implementation (ethos: test
behaviour, not trivial implementation). A coverage gap that lets a `walk_bfs`
regression pass is a pre-existing hole → follow-up, not an implementation test.

**Gate:** `cargo test -p cordage` green, zero assertion changes · `just check` green
workspace-wide · `cargo clippy` zero warnings (plain, bins/lib only — not
`--all-targets`).

## 10. Review Notes

Internal adversarial pass (design author), findings:

- **R1 — gate is real, verified empirically.** `reachable`: `reachability.rs`
  (Along/Against/None, a↔b cycle l.82–83, foreign overlay/node) **plus** two
  independent cross-checks — `condensation_fold.rs` re-derives the fold from
  `g.reachable` across a Max×CountDistinct net matrix, and `golden_net.rs:268`
  runs a naive BFS *sharing no traversal code*. `spine_path`:
  `reachability.rs` l.99–128 (chain root→m→n, single root, `None`, kept-parent
  tiebreak). `cone`: `explain.rs` (SCC endpoint, roots, multi-parent, isolated).
- **R2 — unification strengthens spine cycle coverage.** `spine_path`'s
  cycle-stop now rides `walk_bfs`, whose cycle-safety is directly asserted by
  `reachable`'s a↔b test. Residual: no *direct* assertion of `spine_path` on a
  cyclic `AtMostOne` Reject overlay — covered only transitively via the shared
  loop. **Non-blocking; optional follow-up characterization test**, not part of
  this slice's gate.
- **R3 — `skip(1)` / reverse equivalence re-confirmed.** `start` is `order[0]`
  and never re-emitted (visited seeded), safe even under a self-loop ⇒ `skip(1)`
  drops exactly `start`. Spine: `single_parent` yields ≤1 ⇒ strictly linear
  discovery ⇒ `reverse` is the ancestor-first chain; cycle re-entry excluded by
  visited exactly as the old `break`.
- **R4 — minor allocation note (accepted).** `spine_path` gains a `VecDeque`
  frontier it lacked; `reachable` transiently holds the `order` Vec alongside
  the collected set. Both O(path)/O(reachable) and negligible for a leaf
  refactor — not a regression worth a special case.
- **R5 — doctrinal.** Leaf-internal (ADR-001); pure layer (no clock/rng/git/disk);
  public API unchanged ⇒ external callers unaffected by construction. No cordage
  traversal ADR/standard/policy constrains this surface.
