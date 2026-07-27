# ISS-259: review prime fail-hards on tracked slug symlinks

Raised as RV-315 F-1 during the SL-233 design inquisition, which could not
prime its own ledger.

## Symptom

```
$ doctrine review prime RV-315
Error: hash the slice selector fileset

Caused by:
    Is a directory (os error 21)
```

`doctrine review prime RV-314` (SL-232, four literal `.rs` selectors) succeeds —
the fault is the glob-expansion path, not the ledger.

## Root cause

1. `src/review.rs:2641` `resolve_selectors_to_fileset` expands every non-literal
   selector against `git ls-files` (`src/review.rs:2656`).
2. `git ls-files` lists doctrine's own minted slug symlinks at mode `120000` —
   e.g. `.doctrine/spec/product/001-slices -> 001`, a symlink to a **directory**:

   ```
   $ git ls-files -s .doctrine/spec/product | awk '$1=="120000"'
   120000 0f30166… 0  .doctrine/spec/product/001-slices
   ```
3. `src/contentset.rs:119` calls `std::fs::read(root.join(rel))`, which follows
   the symlink to a directory and returns `ErrorKind::IsADirectory`. The arm at
   line 123 forgives only `NotFound`; line 124 propagates everything else.

## Blast radius

Any slice whose selectors glob an entity root — `.doctrine/spec/**`,
`.doctrine/slice/**`, `.doctrine/adr/**`, `.doctrine/backlog/**` — cannot be
primed at all. `slice::selector_paths` unions **every** intent, so a merely
`scope-relevant` glob is enough to trigger it (SL-233's case).

The engine breaks on its own convention: AGENTS.md requires those symlinks be
committed — "doctrine entity creation commands mint a symlink with the title
slug as a convenience. Commit these with the entity itself."

## Fix

Either forgive `IsADirectory` alongside `NotFound` in `contentset::compute`, or
filter non-blob entries at fileset resolution (`git ls-files -s`, mode not
`120000`/`160000`). The second is the more honest seam — a symlink is not
content the review is tracking — but the first is a one-line stop-loss.

Prefer whichever leaves `contentset::compute` a pure hash-what-you-are-given
leaf; it is shared machinery, so the existing suites are the behaviour-
preservation proof.

## Verification

Regression test: a tracked symlink-to-directory inside a globbed selector primes
cleanly and is either hashed by link target or excluded — decide which, and
assert it. Then `doctrine review prime` against a slice selecting
`.doctrine/spec/product/**`.
