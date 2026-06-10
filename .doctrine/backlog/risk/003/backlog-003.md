# RSK-003: cordage Tarjan and level_of recursion depth overflows stack at multi-thousand graph depth

Two build-time passes in `crates/cordage/src/resolve.rs` recurse with depth equal
to the graph's depth:

- **`Tarjan::strongconnect` (resolve.rs:321)** — DFS over successors; recursion
  depth up to V along the longest DFS-tree chain. Drives all SCC / cycle detection
  (`cyclic_components`), called from pass-1/2 resolution and pass-3 U eviction.
- **`level_of` (resolve.rs:545)** — longest-path memoised DFS over predecessors;
  recursion depth = longest predecessor chain. Memoisation bounds *time* to O(V+E)
  but does nothing for *stack depth*.

At the design's H1/H2 scale (tens–hundreds of nodes) this is invisible. At a real
target of tens of thousands, a deep chain — routine in a large dependency graph —
drives recursion past the 8MB default main-thread stack and **panics**. This is a
crash, not a slowdown, so it is filed `impact=high`.

PHASE-02 saw and consciously deferred this: `phase-02.md:47` reads *"Tarjan
recursion depth at scale — non-concern at H1/H2 (hundreds of nodes, notes); keep
simple, iterative only if clippy/recursion lints complain."* Correct at hundreds;
void at tens of thousands. The deferral is the origin, not a defect in the prior
work.

**Fix direction.** Mechanical explicit-stack iterative rewrite of both functions —
no algorithmic change, same SCC/level results. The red test is a deep-chain graph
generator (linear spine of N nodes), built as part of the perf spike, asserting no
panic and correct results at N well past the recursion ceiling.

**Sequencing.** Belongs behind the perf spike (handover Task 2): the spike builds
the deep-chain + diamond generators that red this and [[RSK-002]], quantifies the
cliffs, and gates the fix. Do not fix blind — prove it first.

Secondary, same loop: the eviction fixpoint (`pass2_evict` resolve.rs:198,
`evict_layer_cycles` resolve.rs:478) rebuilds a full Tarjan from scratch per evicted
edge — O((1+K)·(V+E)), K = edges evicted. Acyclic / near-acyclic input stays
effectively linear; heavily-tangled input is quadratic. Lower priority than the
overflow (bounded by cycle density, not a crash), but it inherits the same recursive
Tarjan, so the iterative rewrite fixes its crash exposure too.

Related: SPEC-001 **H1** hypothesis ("small corpus") is the assumption this
violates; its revision to the real scale target is the upstream governance change.
RSK-002 (explain path-enumeration exponential) is the sibling scale cliff surfaced
by the same review.
