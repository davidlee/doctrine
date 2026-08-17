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

## Second route to the same failure (SL-238 audit, 2026-08-17)

The mechanism above is scoped to phases *not yet implemented*. It also fires on
a phase that IS implemented and whose criteria were deliberately **retired**.

`SL-238` PHASE-02 authored three characterisation tests (`after_prune_pins…`,
`backlog_after_prune_pins…`, `after_remove_pins…`) whose whole purpose was to be
superseded later — and PHASE-06 `EX-3`, PHASE-07 `EX-5` and PHASE-08 `EX-3` each
oblige the *replacement* to name what it replaced and why the old assertion no
longer holds. So every superseded test's name survives in its successor's doc
comment. `verify-vt` finds the keyword, and PHASE-02's three rows report `PASS`
forever against tests that no longer exist.

Measured at audit: `grep -c` over `tests/e2e_dep_seq_verbs.rs` finds
`after_prune_pins` 9×, `backlog_after_prune_pins` 3×, `after_remove_pins` 2× —
all inside supersession comments; `grep -nE '^fn '` lists none of them. Positive
control: `after_prune_noop`, a live test, matches twice.

The consequence is worse than condition 2's, because it is **not** a coincidence
of careless keywording: the plan's own supersession discipline manufactures the
false positive, so it will recur in any well-run slice that pins a before-state.
The first two candidate directions above do not catch it — the phase IS
`completed` and the file IS in its delta. The third (require the keyword in the
phase's own diff hunks) does, and a fourth would too: let a row declare itself
superseded by a later phase, and report `SUPERSEDED` rather than `PASS`.

Raised as `RV-363` `F-2` — the `SL-238` reconciliation audit ledger.
