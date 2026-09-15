# verify-vt reads UNATTRIBUTABLE for a file new to the current phase until it flips completed

`doctrine slice verify-vt` judges each VT by five ordered gates (`check_vt`,
`src/vtgate.rs`). Gate 4: if the VT's `test_file` is **not** in the slice's
`modified_files` set, the verdict is **UNATTRIBUTABLE** ("keyword present but
`<path>` not modified by this slice") — *before* keywords are even matched.

`modified_files` is built in the shell (`run_verify_vt`, `src/slice.rs`) from
`crate::state::read_source_deltas(root, id)` — the slice's **source-delta
registry**. That registry is populated per phase **only when the phase flips to
`completed`** (the conformance-boundary close: `slice phase --status completed`
captures `code_end_oid = HEAD` and records the `code_start..code_end` changed
files). Files touched by *earlier* completed phases are already in the set.

## The trap (SL-212 PHASE-04)

A file **first-touched in the current, still-`in_progress` phase** (e.g. PHASE-04
added `src/commands/guard.rs`, untouched by PHASE-01..03) is NOT yet in any
recorded source-delta. So its VT reads **UNATTRIBUTABLE mid-phase even after you
committed the code and the test passes** — the keyword IS present; the file just
isn't attributed to the slice yet. Do **not** chase this as a bug (wrong pointer,
missing keyword, uncommitted edit) — it is gate 4 working as designed.

## How to apply

- Expect UNATTRIBUTABLE for any VT whose `test_file` is new to the phase you're
  executing. It resolves to PASS automatically once you `slice phase <id>
  <PHASE-NN> --status completed` (the flip records the delta including that file).
- Sequence at phase close: commit all code → **flip completed** → *then* re-run
  `verify-vt`. Verifying before the flip under-reports newly-introduced files.
- A file touched by an earlier completed phase attributes immediately (its delta
  is already recorded) — this only bites files the current phase introduces.
- Contrast the two other UNATTRIBUTABLE-adjacent facts: gate 5 matches keywords
  **raw** over host source (POL-002, [[mem.pattern.gate.host-source-no-language-syntax]]);
  a genuinely wrong pointer (plumbing landed in a different seam) is corrected by
  editing the VT row's `test_file`, NOT by inlining code to satisfy the grep.

## On the dispatch arm the recorder is `record-boundary`, not the phase flip

Verified SL-233 PHASE-02 (2026-07-29). In a **dispatch coordination worktree**
the `slice phase --status completed` handler *self-skips* its boundary capture —
the funnel beat records instead (the `/execute` skill says so explicitly: "It
self-skips in a dispatch coordination context, where the funnel beat is the
recorder instead"). So the registry is populated by

```
doctrine dispatch record-boundary --slice N --phase PHASE-NN \
  --code-start <pre-code HEAD> --code-end <phase tip>
```

and **VT verdicts flip UNATTRIBUTABLE → PASS at that moment, before the phase is
`completed`**. Observed: all three PHASE-02 VTs read UNATTRIBUTABLE immediately
after the source commit, and PASS immediately after `record-boundary`, with the
phase still `in_progress`.

The body's rule is unchanged in substance — attribution comes from the recorded
source-delta registry, never from the working tree — only the verb that writes it
differs by arm. On the solo arm it is the completed flip; on the dispatch arm it
is `record-boundary`.

Corollary: if you amend the phase's code after recording (a refactor commit, say),
re-run `record-boundary` with the new `--code-end`. It UPSERTs by phase. Two costs
to know: it lands its own `dispatch-funnel:` commit and leaves the worktree copy
stale (ISS-274 — `git restore --source=HEAD --staged --worktree -- <boundaries.toml>`),
and the *previous* funnel commit now falls inside the widened range, which
surfaces `.doctrine/dispatch/<N>/boundaries.toml` as conformance-undeclared noise
([[mem.pattern.conformance.boundary-start-oid-pollution]]).

## The worse case: a phase flipped retrospectively never attributes at all

Verified SL-238 PHASE-02 (2026-08-16). The rule above assumes the phase was
flipped `in_progress` **before** its code landed — that flip is what stamps
`code_start_oid`. Execute first and flip afterwards (both transitions recorded
retrospectively) and the recorded range is empty or truncated, so **every** one
of that phase's VT rows reads `UNATTRIBUTABLE` permanently, not just the rows
whose file is new. Flipping `completed` does not fix it; there is no start to
diff from.

SL-238 PHASE-01 sat that way through its whole harvest: eight rows, all
`UNATTRIBUTABLE`, while its files were in the slice's selector list and its code
was committed under `feat(SL-238)` scopes. Selector membership is NOT the
attribution mechanism and does not rescue it.

The repair is `record-delta`'s escape-hatch mode, naming both ends:

```
doctrine slice record-delta <id> PHASE-NN --start <first own commit>^ --end <own code tip>
```

All eight rows read `PASS` immediately after. Foreign commits between your first
and last ride along in the range — expected, and visible in `slice conformance`'s
undeclared cell.

**Why it is worth catching**: `UNATTRIBUTABLE` is non-halting (exit 0, not a
`Fail`), so a retrospectively-flipped phase silently withholds the VT evidence an
audit is going to want, and nothing goes red to tell you. When you inherit a
slice, re-run `verify-vt` over the **completed** phases before trusting that
their criteria are attributed.

## The same blind spot, seen from `slice conformance`

Verified SL-256 PHASE-03 (2026-09-09). `slice conformance <id>` folds the
**boundary registry**, and the registry only holds phases that have been
recorded — i.e. flipped `completed`. So mid-phase it does not report your phase:
it reports the picture as of the last completed one. Files the current phase has
already edited and committed still read `undelivered`, and paths it has already
added are missing from `undeclared`. Nothing says the range is stale; the output
just looks like the phase did nothing.

That is the same root as the `UNATTRIBUTABLE` rule above — the attribution
machinery cannot see an in-flight phase — and it bites earlier, because `EX`
criteria routinely ask for a clean conformance read *before* the completed flip.

The in-phase read bypasses the registry with the boundary the flip already
stamped:

```
doctrine slice conformance <id> --against <code_start_oid>..HEAD [--strict]
```

`code_start_oid` is in `.doctrine/state/slice/<N>/phases/phase-NN.toml`, written
by the `in_progress` transition. Read the range as one phase's delta: `undelivered`
will list selectors *other* phases delivered, which is not a violation.

`--strict` (only valid with `--against`) exits nonzero on any undeclared path,
including `.doctrine/` artefacts a code-selector fence does not govern — a
coverage cell, an observation record, a backlog item. Expect the nonzero and read
the cell, rather than treating the exit code as the verdict.

**Refinement, SL-259 PHASE-03 (2026-09-15) — the completed flip is not what
clears `UNATTRIBUTABLE`; the registry ROW is, and you can write it yourself.**

Measured directly. PHASE-03 ran `verify-vt` mid-phase with the phase
`in_progress` and read `UNATTRIBUTABLE` on all five criteria — *keyword present
but `<file>` not modified by this slice*. Then, still `in_progress`:

```
doctrine slice record-delta <id> PHASE-NN --start <code_start_oid> --end HEAD
```

and the same five read `PASS`. No status change in between. So the operative
cause is an absent boundary row, not an in-flight status, and `record-delta` is
the direct fix rather than a workaround — use the raw `--start/--end` mode for a
multi-commit phase, which is the normal shape when a phase commits as it goes.

The conformance half above is unchanged and really does need the flip:
`slice conformance <id>` refused with *recorded row for PHASE-NN, which is not a
completed phase* until the status moved. Two different gates with two different
preconditions, which is why one rule for both misleads.

Worth knowing at phase-plan time: a phase whose `EX` asks for green `verify-vt`
can get it before the flip; one that asks for a clean conformance read cannot,
and needs the `--against <code_start_oid>..HEAD` form above.
