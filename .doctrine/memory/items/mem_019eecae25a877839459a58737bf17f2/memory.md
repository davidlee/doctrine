# cordage in_edges already excludes evicted edges

For an `Evict`-policy overlay, cordage's `in_edges(overlay, n)` / `out_edges(overlay, n)`
return only the **surviving** edges — the edge cordage evicted to linearize a cycle is
NOT enumerated. `provenance().evictions()` still records the evicted edge (both
directed entries for a 2-cycle).

Consequence (SL-133 PHASE-05 `next` frontier sort): the design (§5.4) prescribed
"surviving seq edges = seq_overlay minus `provenance().evictions()`" out of caution.
Empirically the subtraction is a **no-op today** because `in_edges` already excludes
the evicted edge. The explicit `evicted_seq_edges` subtraction is kept as a
design-aligned safeguard — correct if cordage's enumeration ever changes to include
evicted edges. Verified by VT-7's evicted-seq case (`vt7_evicted_seq_edge_does_not_
reimpose_precedence`), which passes either way.

Don't assume raw-overlay enumeration includes evicted edges. See [[mem.pattern.priority.scc-condensation-dp-order]].
