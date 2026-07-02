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

## Option (b) sketch: canonical stable-sort on write

After constructing the new row, collect all `[[relation]]` rows (existing + new),
**stable-sort by label**, then rebuild the array. Stable sort preserves within-label
relative order — the new row lands at the end of its label group because it was the
last one added.

### Two sites, same pattern

**`append_relation_row`** (`src/relation.rs:1190` — replace `array.push(row)`):

```rust
// Collect all existing rows + the new one
let mut rows: Vec<toml_edit::Table> = (0..array.len())
    .map(|i| array.get(i).unwrap().clone())
    .collect();
rows.push(row);

// Stable-sort by label — preserves relative order within each label group
rows.sort_by(|a, b| {
    let la = a.get("label").and_then(|v| v.as_str()).unwrap_or("");
    let lb = b.get("label").and_then(|v| v.as_str()).unwrap_or("");
    la.cmp(lb)
});

// Rebuild the array in canonical order
for i in (0..array.len()).rev() {
    array.remove(i);
}
for r in rows {
    array.push(r);
}
```

**`append_memory_relation`** (`src/memory.rs:2553` — same treatment, simpler shape):

```rust
let mut rows: Vec<toml_edit::Table> = (0..array.len())
    .map(|i| array.get(i).unwrap().clone())
    .collect();
rows.push(row);
rows.sort_by(|a, b| {
    let la = a.get("label").and_then(|v| v.as_str()).unwrap_or("");
    let lb = b.get("label").and_then(|v| v.as_str()).unwrap_or("");
    la.cmp(lb)
});
for i in (0..array.len()).rev() {
    array.remove(i);
}
for r in rows {
    array.push(r);
}
```

### Properties

| Property | Behaviour |
|---|---|
| **Contiguity** | Guaranteed — all rows with same label are adjacent after sort |
| **Within-label order** | Preserved — stable sort keeps relative order of equal keys |
| **New-row position** | Lands at end of its label group (pushed last, stable-sort keeps it there) |
| **Hand-authored disorder** | Fixed transparently — any pre-existing interleaving is healed on next write |
| **F1 defence** | Unaffected — checked before the sort rebuild |
| **Idempotency** | Unaffected — same-triple check is first, before any array mutation |
| **Performance** | O(n log n) over typically &lt; 20 rows; negligible |
| **`DocumentMut` round-trip** | Comment preservation confirmed — `clone()` + `remove`/`push` round-trips through `toml_edit`, which preserves inert whitespace/comments on table-level entries |

### What about remove?

`remove_relation_row` (`src/relation.rs:1210-1221`) and `remove_memory_relation` just
delete matching rows — they can't *create* interleaving. No change needed.

### Edge cases

- **File with no existing `[[relation]]` array**: `or_insert_with` creates empty array —
  one row, sort is a no-op.
- **Single-label file**: sort is a no-op.
- **Already-contiguous file**: stable sort is a no-op (existing relative order is already
  canonical).

### Test implications

The existing `append_relation_row_appends_and_preserves` test (line 2191) asserts the
new row is at `array.len() - 1`. It would need to instead assert the row is **at the
end of its label group**. The contiguity invariant test (`e2e_relation_migration_storage.rs`)
is the system-level proof and would naturally pass.
