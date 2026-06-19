# parse_dsl collapses duplicate edge triples into diagnostics, not edges

In `src/concept_map.rs`, `parse_dsl` deduplicates: when a line's
`(from_key, rel, to_key)` triple already appeared, it pushes a
`ConceptMapDiagnostic::DuplicateEdge { line, existing_line, .. }` and
`continue`s — **the second line never enters `parsed.edges`**.

Consequence for DSL-mutation dup guards: scanning `parsed.edges` for a collision
in a candidate (post-rewrite) DSL **misses** the dup, because the colliding line
was dropped. Two ways the existing ops handle it:

- `relabel_edge_in_dsl` / `rename_node_occurrence_in_dsl` — parse the **original**
  (dup-free) DSL and search `edges` for the would-be triple on a different line.
- `relabel_rel_all_in_dsl` (multi-line rewrite, atomic) — parse the **candidate**
  and inspect `parsed.diagnostics` for `DuplicateEdge` whose `line`/`existing_line`
  touches a rewritten line; reject the whole op (no partial write).

Same family as the SL-076 finding that `check()`/parse-time diagnostics are
easily dropped from the GET response — see
[[mem.fact.js.concept-map-diagnostic-variant-shape]] for the diagnostic JSON
shape. Related: [[SL-076: check() drops parse-time diagnostics]].
