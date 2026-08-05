# Conformance reads the recorded boundary row, not the diff

A phase's conformance verdict is meaningless until its boundary row exists and
ends at the phase's own code tip. A clean read can mean *nothing was checked*.

## Why

`slice conformance` folds the **recorded** boundary registry
(`.doctrine/state/slice/NNN/boundaries.toml`), not a git diff. Two states both
produce a reassuring answer for the wrong reason:

1. **No row at all.** The solo binding writes the row on the `completed` flip.
   Before that flip, a phase mid-execution has no row, so conformance reports no
   undeclared code path because it read no code delta. This is the dangerous one:
   the output is indistinguishable from a genuinely clean surface.
2. **A row ending past the phase.** The binding stamps `code_end_oid` at HEAD.
   If HEAD has moved onto later, unrelated commits — a backlog item worked
   between the last phase commit and the flip — those land inside the phase's
   range and are attributed to it.

State 1 hid two real undeclared paths on SL-244 PHASE-06, one of them the
phase's own 158-line pure renderer. The clean verdict was reported to the user
before it was true.

## How to apply

At phase close, in this order:

1. Flip the phase `completed` (this writes the row).
2. **Read the row** — `tail .doctrine/state/slice/NNN/boundaries.toml` — and
   check `code_end_oid` is the phase's own last code commit, not HEAD.
3. Correct it if not. `slice record-delta NNN PHASE-NN --commit S` for a
   single-commit phase; `--start <parent-of-first> --end <last>` for a
   multi-commit one (`--commit` takes exactly one).
4. *Then* run `slice conformance` and believe it.

The CLI helps on both counts: the `completed` flip prints `tighten with:
doctrine slice record-delta … --start … --end …` whenever the span is more than
one commit, and `record-delta --commit` now refuses a range that would drop
commits the phase is already known to cover. Read those lines rather than
scrolling past them.

## Related

[[mem.pattern.audit.conformance-boundary-pollution]] is the mirror image —
foreign commits landing *inside* a correct range produce false undeclared
*noise*. This one produces false *silence*. Both are the same root cause: the
registry range is the whole input, and nothing else validates it.

[[mem_019f1b67752c7470aa529e0d0b87a547]] — `code_start_oid` binds to HEAD at the
`in_progress` flip; never rewrite that commit.

## Why step 3 splits the modes — and why a tightened range is still not exact

The two failure modes are not symmetric, which is the whole reason step 3 refuses
to collapse into one command. Too wide over-reports: foreign paths land in
`conformance`'s `undeclared` cell where a reader sees and dismisses them. Too
narrow under-reports **silently**: real deliverables read `undelivered`,
`verify-vt` reports `UNATTRIBUTABLE`, and nothing says the range was truncated.

`ISS-317` was the CLI contradicting that. Its advisory used to close with
`correct with: … --commit <this phase's own code tip>`, and `--commit S` records
exactly `[S^, S]`; since the advisory returns early under two commits, it fired
**only** in the case where its own remedy was wrong. Observed on SL-244 PHASE-08
— seven commits → one, found at audit as `RV-345` `F-1`/`F-8`.

Both halves are closed now: the advisory prescribes the range shape, and
`record-delta --commit` refuses outright when it would drop commits covered by a
span already known for the phase — from the recorded row, or from the sheet's
stamped `code_start_oid`. `--force` overrides, for the phase that really is one
commit with foreign commits landing before it.

What that does NOT buy you is an exact range. A boundary row is one contiguous
span, upserted one-per-phase and read as a two-dot tree diff, so a foreign commit
interleaved between your own first and last still rides along — into the
`undeclared` cell, where it is visible. That is the tightest boundary the model
can express: read `undeclared` rather than assuming a tightened range is clean.

The sheet's stamped `code_start_oid` is usually the `--start` you want — in the
common case only the END is polluted, by commits landing after the flip.

