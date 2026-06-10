# RSK-002: cordage explain() path enumeration is exponential on diamond lattices (SL-036 R-B/N-2)

`Explanation.paths` (`BTreeMap<OverlayId, Vec<Vec<NodeId>>>`) enumerates every
chain to root. A diamond lattice has exponentially many paths in depth, so the
materialised `Vec<Vec<NodeId>>` blows up — bites even at ~50 nodes. F47 bounds
*termination* (chains stop at roots or degraded post-arity SCC entry, never
recursing into a cycle), but it does not bound the *combinatorics*.

This is a pre-existing, design-acknowledged risk on SL-036's design §10 Lock
known-open list — NOT introduced by PHASE-05 — and it is owned by the first
consumer (no current exposure: every SL-036 VT fixture is ≤3 nodes by design, so
no test exercises a wide lattice).

Fix direction: return the predecessor sub-DAG (or the direct predecessors + one
canonical chain) and let the policy layer enumerate on demand, rather than
materialising all chains in the core.

Refs: SL-036 audit.md N-2 / R-B; design.md §10 Lock; notes.md design-stage
("Explanation path enumeration blowup").
