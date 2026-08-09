A test that walks a collection and asserts a property of every member is only as
honest as the members it skips. Written as an inline filter
(`.filter(|row| row.id != Foo)`), the skip is invisible in the failure output,
invisible in review, and silently absorbs a second exclusion later.

Shape that survives:

1. **Name it.** `const UNWALKED: RowId = ...;` with a doc comment saying *why*
   this member cannot be walked and what unblocks it. A reader of the test sees
   the hole without reading the predicate.
2. **Assert the count.** `assert_eq!(walked.len(), all.len() - 1)` — a renamed or
   removed member then reds instead of quietly widening the walk.
3. **Mutation-check the exclusion.** Point the constant at a different member and
   confirm the walk reds on the excluded one. If it stays green, either the walk
   is vacuous or the exclusion was never needed.

Corollary: prefer an invariant carried by the *types* over one asserted by a
walk. If the data structure holds one field where two would be needed to express
the defect, the defect cannot be spelled and needs no test — spend the test on
the shape of the collection instead (count, distinctness), which is what a walk
can still get wrong.

Measured in SL-248 PHASE-09 `T12`: `every_shipped_rows_control_is_seen_to_fail`
walks thirteen admission rows minus one blocked row.
