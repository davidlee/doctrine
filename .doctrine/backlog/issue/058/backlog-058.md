# ISS-058: doctrine link appends same-label relation rows non-contiguously, can break the contiguity storage gate

Surfaced during SL-176 close (2026-06-29). `doctrine link SL-176 governed_by ADR-018`
appended the new `[[relation]]` row at the **end** of the block — after the existing
`references(concerns)` rows — even though three `governed_by` rows already existed earlier.
This broke the same-label-contiguity storage invariant that
`tests/e2e_relation_migration_storage::relation_rows_of_one_label_are_contiguous` (added by
SL-176) enforces; `doctrine check quick` failed until the rows were hand-reordered.

## Detail

The write seam appears to **append-at-end** rather than insert-into / regroup the existing
same-label run. When the label being added already exists but is not the last label in the
block, the result is non-contiguous and the gate rejects it.

- **Observed on the 0.8.1 PATH binary.** Whether 0.9.0 (the SL-176 build) regroups on append
  is **unverified** — if it also appends-at-end, this is a latent defect on trunk, not just a
  stale-binary artifact. **First step: reproduce against a fresh build.**
- If the writer does not maintain contiguity, either (a) make `append_edge`/`link` insert the
  new row adjacent to the existing same-label run, or (b) canonically re-sort rows on write.
  Option (b) also fixes hand-authored disorder.

## Links

- Compounds the stale-binary caveat (`mem_019f025ee2027bf281f7d3a013bc9415`): on `edge`, even
  non-census write verbs can emit corpus that the SL-176 gates reject.
- Relation contract: SPEC-018; storage invariant test in `tests/e2e_relation_migration_storage.rs`.
