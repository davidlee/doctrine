# Notes SL-078: Chore sweep: grid test, spec-010 rename, supersede recovery test

Durable per-slice scratchpad — tracked in git.

## CHR-006 — SPEC-010 rename sweep

**Additional edits beyond design map**: Two `skills install` references (gitignore
self-enforcement section and Decision D4) were found during verification and
corrected. The design's line-number map was approximate — the verification step
(`doctrine spec show SPEC-010`) was essential for completeness.

**Design map corrections**: The design listed 6 edits with line numbers, but:
- L31-32 and L115-118 were duplicates (both pointed to the same L36 reference)
- L130 was the same as L26 (both pointed to the same `skills list`/`skills install` surface line)
- L137-142 mapped to L115-124 in the actual file
- Actual unique edits needed: 8 (6 design-specified + 2 extra found at verification)

## CHR-008 — Supersede torn-state recovery test

**No surprises.** The test was straightforward — `catalog::test_helpers` provided
all needed infrastructure. `run_supersede`'s existing flow handles torn-state
recovery naturally without a dedicated recovery code path.

**Key invariant proven**: `push_str_if_absent` is idempotent — re-running
supersede after a partial write (NEW written, OLD not) completes OLD's
superseded_by entry and status flip without duplicating NEW's supersedes entry.

## Pre-existing issues

`src/spec.rs` had unfinished IMP-058 changes (added `req_bodies` parameter to
`render` and `show_json` but test call sites weren't updated). Fixed 4 clippy
errors (shadow_unrelated, indexing_slicing, collapsible_if, implicit_clone) to
pass the gate. These fixes are uncommitted — belongs to IMP-058's scope.
