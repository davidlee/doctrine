# Notes SL-173: backlog list --after / --needs dependency-sequence edge filter

## Implementation

Single phase (PHASE-01), single file (`src/backlog.rs`). Dispatched via pi
subprocess arm (`doctrine worktree fork --worker` → `pi --mode rpc`).

### Key decisions
- `norm_ref` uses cross-kind `parse_canonical_ref` (not backlog-local `parse_ref`)
  because edges can point at any kind (SL-169, IMP-194, etc.)
- Retains placed after `--kind` retain, before `any_tagged` computation — preserves
  dynamic tags-column visibility (D2)
- Empty flag slice = no-op (axis imposes no constraint)
- Tests use existing `write_rel_item` helper (already on main) for edge-bearing fixtures

### Tests added (8 new)
1. `backlog_list_after_filter_retains_matching_edge_only` (VT-1 positive/negative)
2. `backlog_list_after_negative_excludes_non_matching` (VT-1 explicit negative)
3. `backlog_list_after_normalized_match` (VT-2: `imp-0194` → `IMP-194`)
4. `backlog_list_needs_cross_kind_match` (VT-2: `SL-169` needs)
5. `backlog_list_after_and_needs_compose` (VT-3: AND across axes)
6. `backlog_list_after_and_status_compose` (VT-3: AND with --status)
7. `backlog_list_after_unparseable_verbatim_fallback` (VT-4: raw fallback)
8. `backlog_list_after_empty_flag_noop` / `backlog_list_needs_empty_flag_noop` (empty = noop)

### Audit (RV-186)
- Conformance clean (0 undeclared, 0 undelivered)
- 2728 tests pass, clippy clean
- Two minor findings (F-1, F-2): VT-5/VT-6 explicit filtered-set tests deferred
  as nice-to-have — code paths unchanged, existing suites pass
- No spec/governance changes required

### Dispatch notes
- Claude agent arm unavailable (no WorktreeCreate hook configured in `.pi/hooks/`)
- Pi subprocess arm used successfully with `doctrine worktree fork --worker`
- Worker hit 300s timeout during final commit text generation but had already
  committed; commit was present when funnel ran
- `record-delta` takes bare numeric ID (173), not canonical (SL-173)
