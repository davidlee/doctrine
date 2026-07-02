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

- **Observed on the 0.8.1 PATH binary.** Confirmed on 0.9.0/trunk (2026-07-02): the defect is latent on trunk, not a stale-binary artifact. The root cause is `append_relation_row` (`src/relation.rs:1126-1192`) which always appends at the END of the `[[relation]]` array via `array.push(row)` — it makes zero effort to insert adjacent to existing same-label rows. Primary vector: `doctrine link` → `run_link` → `append_edge` → `append_relation_row`. Secondary vector: `doctrine supersede` (same path). Observed secondary victim: `.doctrine/slice/190/slice-190.toml` — `related`(IMP-191) appended after `governed_by` rows, breaking contiguity (SL-180 RV-216 audit).
- If the writer does not maintain contiguity, either (a) make `append_edge`/`link` insert the
  new row adjacent to the existing same-label run, or (b) canonically re-sort rows on write.
  Option (b) also fixes hand-authored disorder.

## Links

- Compounds the stale-binary caveat (`mem_019f025ee2027bf281f7d3a013bc9415`): on `edge`, even
  non-census write verbs can emit corpus that the SL-176 gates reject.
- Relation contract: SPEC-018; storage invariant test in `tests/e2e_relation_migration_storage.rs`.
