# IMP-439: Conformance is blind to commits between phase boundaries

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What happens

`doctrine slice conformance <id>` computes its undeclared/undelivered/conformant
algebra from the **union of the per-phase source-delta boundary rows**, and each
row is a half-open range `(code_start_oid, code_end_oid]`. Nothing constrains one
phase's `code_end_oid` to equal the next phase's `code_start_oid`. When they
differ, every commit in between is attributed to no phase and is invisible to
conformance — not reported as undeclared, not reported at all.

## Where it bit

`SL-251`'s registry (`.doctrine/state/slice/251/boundaries.toml`):

    PHASE-03  code_end_oid   = 017c59742
    PHASE-04  code_start_oid = be679d544

Two commits sit in that gap — `18e35c2e5` and `be679d544` — and both carried
real regressions that `slice conformance` could not see:

- three stray root-level `capsule-*.md` files duplicating `.claude/agents/`,
  plus `.claude/settings.json` losing its trailing newline (commit message: the
  bare word `doctrine`);
- `.doctrine/workflows/drive-slice.js` resurrected, which `SL-254` PHASE-10 had
  deliberately deleted and for which no embedded source remains.

Both were found by reading the commit range by hand at `/audit` (`RV-362` `F-11`,
`F-12`). Conformance reported eight undeclared paths and named neither.

## Why it matters

`/audit` step 2 tells the auditor to run `slice conformance` and read the algebra
as the *mechanical* drift signal — the leads the human read then works from. The
skill is careful to say conformance is "necessary, not sufficient" and says
*where to look*. This defect is one level worse than insufficiency: for the gap
region it cannot say where to look at all, while presenting a complete-looking
three-cell answer. An auditor who trusts the cells is silently under-covered, and
the size of the blind region is invisible.

Note this is distinct from the existing boundary items, which are all about a
row being *wrong*: `ISS-268` (a row spanning a refresh-base attributes trunk to
the phase), `ISS-307` (completion flip captures HEAD), `IMP-175` (stale
`code_start_oid`), `ISS-317` (advisory truncates a multi-commit span). Those
concern the correctness of a recorded range. This one concerns the **space
between** correctly-recorded ranges.

## Candidate fixes

1. **Report the gap rather than close it.** Compute the union of the recorded
   rows against the span `[first phase start, last phase end]` and emit a fourth
   cell — `unattributed (N commits)`, with the oids — whenever the union is
   proper. Cheapest, and it preserves the honesty the other three cells have:
   conformance keeps saying *where to look* and stops silently omitting a region.
2. **Make contiguity an invariant** at `record-delta` / the solo phase-binding,
   refusing or warning when a new row's `code_start_oid` is not the previous
   row's `code_end_oid`. Stronger, but it fights legitimate cases — a slice may
   genuinely interleave unrelated commits, and forcing contiguity would attribute
   them to a phase that did not produce them, which is `ISS-268`'s defect
   arriving from the other direction.
3. **Do nothing mechanical; state the limit** in `/audit`'s step 2, so the
   auditor knows to read the raw range. Weakest — it is the guidance that already
   failed here, since the skill's existing caveat is about sufficiency, not about
   coverage.

(1) is the recommendation: it is a report change, not a policy change, and it
makes the blind region visible without deciding what belongs in it.
