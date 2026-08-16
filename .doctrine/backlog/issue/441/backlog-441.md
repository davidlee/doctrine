# ISS-441: verify-vt PASSes criteria for phases that have not been implemented

`doctrine slice verify-vt` reports **`PASS` for `VT` rows belonging to phases
whose work has not started**, once two independent conditions coincide:

1. the row's `test_file` has entered the slice's source-delta registry — because
   *some other* phase modified it; and
2. the row's `keywords` happen to already appear somewhere in that file.

Neither condition has anything to do with the criterion being satisfied.

## Observed

`SL-238`, immediately after PHASE-03 landed (`4ad589192`). PHASE-03 is the first
phase to modify `src/backlog.rs`, which is also the mandated `test_file` for many
later rows. Before it landed, those rows read `UNATTRIBUTABLE` — correct, since
nothing had landed for them. After it landed:

- `PHASE-04/VT-2` — keywords `["terminal", "boundary"]` → **`PASS`**. PHASE-04 is
  unimplemented. PHASE-03's test `dep_seq_ref_findings_covers_terminal_items_and_both_axes`
  supplies "terminal"; "boundary" already occurs in the file.
- `PHASE-04/VT-4`, `PHASE-04/VT-5`, `PHASE-05/VT-5`, `PHASE-08/VT-6` — same shape.

Sibling rows in the same phases correctly read `FAIL` (`probe_boundary`, `Axis`,
`probe_item_refs` are genuinely absent), so the output interleaves true negatives
with false positives in one block.

## Why it matters

`verify-vt` is audit evidence. A `PASS` on an unimplemented phase is a false
statement of verification, and it is the *quiet* direction of wrong — `FAIL` and
`UNATTRIBUTABLE` both invite scrutiny, `PASS` closes it. An auditor reading the
summary at close has no signal distinguishing a row that passed because its test
exists from one that passed because a common English word appears in a
6,000-line file.

The exposure grows with keyword genericity. Rows keyed on `terminal`, `json`,
`stderr`, `keeps`, `rank` are near-certain to match any large file; rows keyed on
a distinctive identifier are safe. So the failure is silent, uneven, and
correlates with how carelessly the criterion was written — not with anything the
phase did.

## Candidate directions

Not a design; the fix belongs to whoever picks this up.

- Gate the keyword check on the phase's own status — a row for a phase that is
  not `completed` should not be eligible for `PASS` at all. Cheapest, and
  probably sufficient.
- Attribute per phase rather than per slice: require the `test_file` to fall in
  *this phase's* recorded source delta, not the slice's union. `record-delta`
  already stores per-phase ranges, so the data exists.
- Require the keyword to appear in the phase's own diff hunks rather than
  anywhere in the file.

The first two are independent and compose.

## Provenance

`SL-238` PHASE-03 harvest, 2026-08-17. Not a defect in `SL-238` — the slice's own
eighteen landed rows are genuine — but it degrades the evidence that slice is
going to present at audit, so it should be understood before `SL-238` closes.
