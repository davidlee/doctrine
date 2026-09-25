# Does a run-level traversal mutation owe a change row?

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What was observed

During `SL-264`'s design run, an `apply` carrying only a `traversal` change
(`{"traversal":{"cursor":"inq-2"}}`) advanced the revision 9 → 10 and emitted
**no change row**: the apply's own output listed no rows, and the envelope read
`changes_since_baseline=0` afterwards. The cursor did move (`cursor inq-2`,
`active_path inq-1 > inq-2`), so the mutation landed.

## Why it is only an idea

`REQ-478`'s description is broad — "Every mutation the run records is reported by
at least one change row, so a submission that landed is distinguishable from one
whose payload was discarded" — but its acceptance criteria are narrower: "An
apply that **records an act** emits a change row identifying the act's subject
and kind." A traversal declaration is a run-level field, not an act, so the
criterion does not plainly cover it.

The cost side is real: `ChangeEvent`'s vocabulary is closed and roster-driven
(`REQ-478`'s third criterion, `DEC-238`), so "emit a row" means minting a member
and driving it in the fixture ladder — not a one-line change.

## What needs deciding

1. Is a run-level traversal mutation one the run "records"? (It bumps the
   revision, which argues yes.)
2. If the envelope renders the new cursor directly, is a row owed for
   *discoverability* — or is the envelope the row? (`IMP-389` is the adjacent
   item: the cursor is declared, never derived, and was unset on 16 of 17 runs.)
3. Does this belong with `ISS-450`'s family (a mutation that lands but
   under-reports), or is it deliberately row-free?

## Why it was noticed

The envelope's `changes` line is how a caller sees what a submission did. A
revision bump with an empty change list is the one reading the `RFC-031` T1
fitness bar is meant to drive to zero — "submissions that succeed but change
nothing, or fail but change something".

## References

- `REQ-478` — the change-row obligation and its act-scoped criteria
- `ISS-450` (resolved) — the sibling case: a `needs` edge at node creation emitted no row
- `SL-264` — the run this was observed in; `IMP-389` — the traversal-cursor item
