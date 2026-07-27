# Review prime ignores non-file selector entries

## Context

`review prime` resolves every selector against tracked Git entries and passes
the resulting paths to the content-set hasher. Tracked slug symlinks under
Doctrine entity roots resolve to directories, so a selector such as
`.doctrine/spec/**` fails with `IsADirectory` before the review can be primed.
RV-315 F-1 records the failure and ISS-259 owns the defect.

## Scope & Objectives

- Make selector resolution pass only hashable tracked file content to the
  content-set engine, preserving `contentset::compute` as a hash-what-it-is-given
  leaf.
- Cover the tracked symlink-to-directory case with a regression that primes a
  review whose slice selects an entity-root glob.
- Preserve existing review and content-set behaviour for ordinary tracked files,
  deleted files, and literal selectors.

## Non-Goals

- Changing selector syntax or selector intent semantics.
- Following directory symlinks and hashing their descendant trees.
- Changing review-ledger lifecycle, content-set serialization, or SL-233's
  design.

## Summary

Filter non-file Git index entries at the selector-to-fileset seam, before
content hashing. The implementation should distinguish Git entry kind from
working-tree resolution rather than teaching the shared content-set leaf about
review-specific selector expansion.

Affected surfaces are `src/review.rs` and the focused review-prime regression
tests. `src/contentset.rs` is relevant for behaviour-preservation verification
but is not an intended change target.

Closure requires the new regression to fail before the implementation, then
pass; the existing review/content-set suites must remain green unchanged; and
`doctrine review prime RV-315` must succeed against SL-233's current selectors.

## Follow-Ups

None known.
