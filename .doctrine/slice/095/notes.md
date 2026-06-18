# SL-095 — Implementation notes

## PHASE-01 — Add `related` for slice & backlog

**Commit:** `db05de42` on `dispatch/095` (imported from `worker/SL-095-PHASE-01` at `d5cb0088`)

**What was done:**
- Added one RELATION_RULES row: `sources &[SLICE, ISS, IMP, CHR, RSK, IDE], Related, AnyNumbered, Tier::One, Writable`
- Inserted after the existing GOV `Related` row (SameKind). Two rows at one slot, same as `Supersedes` already has.
- Updated tests across `src/relation.rs` and `src/relation_graph.rs`:
  - `sources_match_table`: Related sources expanded from 3 (ADR,POL,STD) to 9
  - `lookup_keys_on_source_and_label`: slice Related now `Some` with `AnyNumbered`
  - `target_spec_matches_design`: disambiguated GOV (SameKind) vs SLICE/BACKLOG (AnyNumbered)
  - `read_block_rejects_illegal_source_label_pairs`: related now legal for slice
  - `validate_link_gates_source_label_and_policy`: related now writable for slice
  - `check_target_kind_enforces_target_kind`: added AnyNumbered target acceptance test
  - `reader_emitted_labels_equal_table_labels_per_source`: SL and ISS readers now emit Related
  - `validate_relations_reports_danglers_and_illegal_rows`: swapped illegal example to `descends_from`

**Verification:**
- All 1664 tests pass, `cargo clippy` clean
- `doctrine link SL-095 related ADR-010` → succeeds
- `doctrine link IMP-082 related SPEC-018` → succeeds
- `doctrine link SL-095 related FREE-TEXT` → refused (unknown kind prefix `FREE`)

**Surprises:**
- `src/knowledge.rs` got `cargo fmt` churn alongside the relation files (no semantic changes)
- The worker correctly halted on 3 test failures beyond `src/relation.rs` — file scope was too narrow
- `relation_graph.rs` tests needed updating for both `reader_emitted_labels_equal_table_labels_per_source` (backlog + slice readers) and `validate_relations_reports_danglers_and_illegal_rows` (illegal example)

**Bwrap jail note:**
- `doctrine worktree coordinate` created the worktree but the git admin directory wasn't persisted — needed manual `git worktree add` to attach
- This is likely a filesystem layer issue in the bwrap jail
