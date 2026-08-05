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

The CLI helps: the `completed` flip prints `correct with: doctrine slice
record-delta …` when the end oid looks wrong. Read that line rather than
scrolling past it.

## Related

[[mem.pattern.audit.conformance-boundary-pollution]] is the mirror image —
foreign commits landing *inside* a correct range produce false undeclared
*noise*. This one produces false *silence*. Both are the same root cause: the
registry range is the whole input, and nothing else validates it.

[[mem_019f1b67752c7470aa529e0d0b87a547]] — `code_start_oid` binds to HEAD at the
`in_progress` flip; never rewrite that commit.

## The repair itself has a trap (ISS-317)

Step 3 above says `--commit` for a single-commit phase, `--start`/`--end` for a
multi-commit one. **The CLI's own advisory tells you the opposite**, and it is
the line you are reading at the moment of the flip:

```
warning: PHASE-NN boundary spans 7 commits — any that are not this phase's are
attributed to it; review before audit:
  …
correct with: doctrine slice record-delta 244 PHASE-NN --commit <this phase's own code tip>
```

`--commit S` records exactly `[S^, S]`. So following that line on a multi-commit
phase replaces a range that was too WIDE with one that is one commit long. And
the advisory returns early when the span is under two commits — so it fires
**only** in the case where its own remedy is wrong.

It is also a trade down. Too wide over-reports: foreign paths land in
`conformance`'s `undeclared` cell where a reader sees and dismisses them. Too
narrow under-reports **silently**: real deliverables read `undelivered`,
`verify-vt` reports `UNATTRIBUTABLE`, and nothing says the range was truncated.

Observed on SL-244 PHASE-08 (seven commits → one), found at audit as `RV-345`
`F-1`/`F-8`. Until `ISS-317` lands, repair a polluted end with the explicit form:

    doctrine slice record-delta NNN PHASE-NN --start <parent of first code commit> --end <own code tip>

The sheet's stamped `code_start_oid` is the `--start` you want; it is already
correct in the common case, since only the END is polluted by post-flip commits.
