# SCC-condensed DP needs explicit component-DAG topo order

Reverse `graph.ordered()` is NOT reverse-topo of the *condensed* (SCC-collapsed)
graph. A seq/`after` edge can perturb an SCC member's level and place it before an
external dependent of another member, so a DP that computes a component's value at
the first member hit in reverse `ordered()` reads that dependent's value as
unresolved (0.0).

**Pattern:** for any DP over an SCC-condensed graph, build the condensation
explicitly — nodes = `provenance().cycles()` SCCs (filtered to the relevant
overlay) + singletons for every other node; edges = inter-component edges — then
topo-sort THAT graph (post-order DFS) and DP in reverse-topo order. Dedupe
cross-component neighbours into a set so a neighbour touching >1 SCC member counts
once per component, not once per member.

Surfaced by RV-137 F-1/F-2 against the `src/priority/graph.rs`
`consequence_post_pass` leverage DP. See [[mem.signpost.doctrine.review]].
