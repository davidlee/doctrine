# Bucket edges by component once, never filter the full edge set per component

Cordage cycle eviction (`crates/cordage/src/resolve.rs`) processes each cyclic
component to its own fixpoint. The trap: building a component's induced
sub-edge-set by filtering the **whole** overlay edge set —
`edges.iter().filter(|e| component.contains(src) && component.contains(dst))` —
is O(E) per component, so N disjoint components cost **O(components·E)**. On N
disjoint 2-cycles (E ≈ 2N) that is O(N²): 20k cycles took 113s and blew the
`many_small_cycles_evict_in_linear_time` gate (RSK-224). SL-043 had localized the
re-Tarjan but left this partition scan quadratic.

**Do X:** cyclic components are vertex-disjoint, so partition the edges into
per-component buckets in ONE O(E) pass, then hand each bucket to its component's
fixpoint loop. `bucket_by_component` does this: build `node → component_id`, then
bucket each edge whose endpoints share a component. Result is byte-identical to
the per-component filter (set-identity gate green). 113s → 0.46s (debug, ~240×).

- Both eviction sites share the helper: `pass2_evict` (intra-overlay) and
  `evict_layer_cycles` (cross-layer U). Don't reintroduce the per-component
  filter in either.
- The residual `dense_evict` superlinearity (EXC-2,
  `eviction_fixpoint_scales_superlinearly`, `#[ignore]`d) is a DIFFERENT cost —
  a single dense cycle's re-Tarjan-per-eviction — and is correctness-locked (the
  global-min-one-at-a-time order defines the evicted set). Bucketing does not
  touch it.

Sibling perf fix same session: `evaluate` was walking the neighbour view twice —
`condensation` now returns the quotient `succ` adjacency for the fold to reuse
instead of recomputing it per SCC (`crates/cordage/src/query.rs`). General
lesson: in cordage's SCC/condensation passes, compute a quotient/partition ONCE
and thread it, don't rebuild per node/component.

See [[mem.pattern.priority.scc-condensation-dp-order]] (build the condensation
explicitly and DP in reverse-topo) and [[mem.fact.cordage.in-edges-excludes-evicted]].
