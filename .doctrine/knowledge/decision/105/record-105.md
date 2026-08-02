# DEC-105: Authored watermark guard tiered by write target

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

This record supersedes [[DEC-092]]. Rules 1 and 2 are carried forward unchanged;
only rule 3 is restated, and it is restated by splitting rather than weakening.

## What DEC-092 got right, and the one thing it did not

DEC-092 is sound about the *shape* of the problem. The runtime revision is a
counter inside a gitignored snapshot that a foreign editor never touches, so it
cannot detect a hand-edit to `design.md`; the watermark closes that gap; and
`adopt_authored` has to be an explicit protocol rather than a bypass or the
live-run recovery path refuses itself. None of that changes.

What it did not anticipate is that one run-advancing verb writes the authored
document *itself*. DEC-092's rule 3 speaks of "the snapshot write", and for two
of the three re-check sites that is exactly right — `design.rs:988` and `:1223`
guard verbs whose write is a runtime-snapshot write. `materialise` is the third,
and its sequence is:

1. re-check the watermark (`design.rs:1382`)
2. write `design.md` (`entity::write_body(.., BodyMode::Replace)`, `:1399`)
3. re-baseline the watermark from the body it just rendered (`:1406`)
4. write the snapshot

Step 2 sits *inside* the window rule 3 describes as ending at step 1. And step 3
is what turns a bounded race into an unbounded one: the watermark now records
Doctrine's own output, so the next entry check compares Doctrine against Doctrine,
finds alignment, and reports nothing. DEC-092 admitted a residual whose whole
justification was that the *next* entry would catch it. On this path there is
nothing left to catch.

The difference in kind matters more than the difference in size. Delayed detection
means the user's bytes survive and the disagreement surfaces one verb later.
Self-certified overwrite means the user's bytes are gone and no verb will ever
say so.

## Why the stronger guarantee, rather than a franker caveat

The tempting move is to widen the admitted residual — say plainly that
`materialise` can lose an edit, and be done. That is rejected because DEC-092's
own reasoning rules it out. Its rationale states the premise the weaker guarantee
rests on:

> `with_turn` holds a writer lock and hashes the very file it atomically replaces;
> a design run hashes `design.md` while writing a runtime snapshot, a checkpoint
> journal, and possibly an authored record. Those differences are why the crossing
> needs its own admission protocol (rule 2) and why the mid-invocation guarantee
> is bounded (rule 3).

That is a *derivation*, not a preference — and its premise is false for exactly
one verb. `materialise` hashes and replaces the same file. So the reason DEC-092
gave for accepting the weaker guarantee does not apply where the cost is highest,
and the mechanism it named as unavailable is in fact sitting in the tree:
`with_turn_hooked` (`src/review.rs:2056`) acquires a `LockGuard`, performs an
entry compare-and-swap, and exposes a `mid_turn` hook its own comment calls "the
pre-write CAS test seam".

Nothing beneath the write seam supplies the guarantee either.
`fsutil::write_atomic` is a temp-write plus rename with no compare-and-swap
(`src/fsutil.rs:52-75`), and `entity::write_body` skips only the byte-identical
case (`src/entity.rs:772-776`) — so a hand-edit that differs from the render is
always overwritten.

## The rule, restated

**Rule 3a — runtime-snapshot writes keep the weaker guarantee.** Re-read and
re-fingerprint immediately before the write; abandon on divergence without
advancing the run. The comparison-to-rename residual stands, discharged by
next-entry detection, because the file hashed is genuinely not the file replaced.

**Rule 3b — the authored-document write carries the stronger guarantee.**
`materialise` takes a writer lock and compare-and-swaps against the fingerprint it
just checked, and it re-baselines the watermark **only** after confirming that the
bytes on disk are the bytes it wrote. A failed confirmation abandons the
re-baseline and reports divergence rather than certifying a replacement it cannot
vouch for.

Rule 3b is deliberately stated as an obligation on the *authored* write, not on
`materialise` by name: any future verb that writes `design.md` inherits it.

## The load-bearing unknown

`LockGuard::acquire(root, id)` is keyed on a review id. Rule 3b needs either that
key generalised or a sibling guard for the design run, and which of those is right
is a real design question about shared machinery — not a detail. That unknown is
why this is a decision record rather than a bug fix, and it is the first thing
PHASE-15 has to settle. The fallback, if generalisation proves infeasible within
the phase, is the second alternative: restore detection (confirm-before-rebaseline)
without preserving the edit, and record the remaining loss honestly rather than
silently.

## Bounds

This record says nothing about the pure/shell split, which RV-324 verified intact
with a positive control, and nothing about digest single-sourcing (RV-324 F-5) —
though the extraction instinct is the same, and the two may turn out to be one
decision. F-5's fix shape is reserved for slice close.

Governance surface: `SPEC-029`'s watermark responsibility carries the same
"before every snapshot write" wording and is corrected by `REV-044`. The code
change both authorise lands in SL-233 PHASE-15.
