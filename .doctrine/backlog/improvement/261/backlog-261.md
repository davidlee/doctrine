# IMP-261: link verb TOML block-order coupling

## Problem

`doctrine slice selector add` writes the `[[selector]]` array-of-tables below
the `[[relation]]` block in the slice TOML. A subsequent `doctrine link` then
refuses with F1: "typed table `[selector]` is authored AFTER the
`[[relation]]`" — forcing a manual TOML re-home of the selector block above
the relation array.

Witnessed twice:
- SL-191-scope: `slice selector add` followed by `link` → F1 refusal → manual
  Edit to reorder TOML blocks
- IMP-216/CHR-036 session: `link IMP-216 related CHR-036` refused on
  `[value]`-after-`[[relation]]` ordering → had to reverse direction

Root cause: two write verbs on the same file have an implicit ordering
dependency they don't coordinate. The `link` verb expects `[[relation]]` to be
the LAST array-of-tables in the TOML, but `selector add` places `[[selector]]`
after it. The `append_relation_row` F1 guard was added for the TOML contiguity
invariant (SL-176, ISS-058), but it doesn't account for ordering between
different array-of-tables kinds.

## Fix direction

- **Fix `selector add`**: home the `[[selector]]` blocks above the
  `[[relation]]` array, so `link` can always append. Simplest and most
  contained.
- **Or fix `link`**: tolerate trailing array-of-tables (non-`[[relation]]`)
  after the relation block — append before the first non-relation
  array-of-tables rather than at EOF.
- **Or fix the F1 guard**: instead of "no typed table after `[[relation]]`",
  check only that same-label `[[relation]]` rows are contiguous. Other
  array-of-tables after the relation block are fine.
- The ADR-004 reciprocity workaround (link from the other direction) works but
  depends on knowledge the error message doesn't suggest.

## Related

- RFC-011 case-notes: SL-191-scope, IMP-216/CHR-036 session
- ISS-058 (contiguity storage gate — the invariant F1 enforces, closed)
- ADR-004 (reciprocity derived — the workaround)
