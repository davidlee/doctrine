# ISS-027: add_edge_to_dsl duplicate check is label-based, not key-based

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`add_edge_to_dsl` (`src/concept_map.rs`) guards against duplicate edges by
comparing **labels** (`e.from_label == source && e.rel == rel && e.to_label ==
target`), but `parse_dsl` defines duplicate identity by **derived key**
`(from_key, rel, to_key)`. Two distinct label spellings that derive the same key
(`User Story` vs `User-Story`) slip the label check and collide only at parse,
leaving the stored DSL with a silently-dropped duplicate.

The SL-110 CM mutations (`relabel_edge`, and the rev-2 `rename_node_occurrence` /
`relabel_rel_all`) all guard **key-based** (`derive_node_key`) — `add_edge` is the
remaining label-based outlier. Fix: switch `add_edge_to_dsl`'s dup check to the
same key-based identity.

Surfaced in SL-110 design (D6 / Follow-ups); out of that slice's scope.
