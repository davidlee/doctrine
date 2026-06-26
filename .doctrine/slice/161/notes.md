# Notes SL-161: DRY kind-registry seam: record-membership predicate + numbered-kind identity table

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Implementation Notes

### Two-worker file-disjoint dispatch

PHASE-01 and PHASE-02 touch disjoint file sets (PHASE-01: kinds, dep_seq,
partition, search, tag, relation, test_helpers; PHASE-02: scan, integrity).
Ran as parallel workers in isolated worktrees; both produced single clean
commits. Imported sequentially, verified, squashed to one coordination commit.

### Behaviour preservation

All 2607+ unit tests green throughout. Architecture layering gate passed.
The rewritten `is_record_predicate_matches_kinds_record` test is stronger
than the old pin — set equality over all KINDS vs RECORD.

### Recorded boundaries

Both phases recorded with `record-delta` sharing the same B→B+1 range
(single coordination commit).

## Risk Observations

None surfaced. The two audit findings (F-1 plan VA-1 imprecision, F-2 scan.rs
comment removal) were minor/nit and resolved as aligned/tolerated.

## Follow-up Work

- SL-159: after this lands, SL-159 can use the DRY membership — adding EVD+HYP
  edits `kinds::RECORD` + `numbered_kinds_registry!` (2 edits, not 17).
- CON→INV rename: edits the same two registry sites.
- GOV/BACKLOG groups: smaller mechanical DRY following this pattern.
- The plan's VA-1 criterion should be updated during reconciliation to list
  the full set of design-exempted sites (currently says "zero outside kinds.rs
  and dep_seq.rs:285" but design §5.4 exempts integrity.rs:821, relation.rs:1422,
  1427, and search.rs:39).
