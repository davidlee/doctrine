# ISS-025: add_edge_to_dsl duplicate check is label-based but parse_dsl dedups by key

## Problem

`add_edge_to_dsl` (`src/concept_map.rs:1235`) rejects a duplicate edge by
comparing **labels**:

```rust
.find(|e| e.from_label == source && e.rel == rel && e.to_label == target)
```

But `parse_dsl` defines duplicate identity by **derived key**
(`(from_key, rel, to_key)`, `src/concept_map.rs:421`), where
`from_key = derive_node_key(label)`. Distinct label spellings can derive the same
key (e.g. `User Story` and `User-Story`). So adding `User-Story > rel > Goal`
when `User Story > rel > Goal` already exists slips the label check, writes the
line, and only collides at the next parse — `parse_dsl` then emits `DuplicateEdge`
and silently skips one edge, leaving the stored DSL self-inconsistent.

## Fix sketch

Make the dup check key-based: derive keys for `source`/`target`, compare against
parsed `(from_key, rel, to_key)` triples. Add a regression test with two
same-key/different-label endpoints.

## Provenance

Found during the SL-110 design review (Codex round 2, finding G1). SL-110 adds
`relabel_edge_in_dsl` with a correct key-based guard; this issue is the
pre-existing sibling gap in `add_edge_to_dsl`, left out of SL-110's scope
(frontend-polish slice, one scoped backend op). Likely `remove_edge_from_dsl`
warrants the same audit under key collision.
