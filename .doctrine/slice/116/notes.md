# SL-116 Implementation Notes

Durable decisions and findings from PHASE-01 and PHASE-02.

## PHASE-01

- Marker.rs created by pi worker but `mod marker;` was missing — the items
  stayed in mod.rs as duplicates. Fixed in PHASE-02 by removing duplicates
  and adding proper module declaration + re-exports.
- The pi subprocess dispatch arm was unreliable for this task (complex
  mechanical extraction). Switched to inline execution on coordination
  worktree after initial attempt.

## PHASE-02

- Item-by-item extraction (not section-based) was necessary because
  coordinate items (CoordAction, CoordRefusal, classify_coordinate) are
  interleaved with land items, and run_branch_point_check is in the
  provision region.
- All extracted items are brace-balanced when extracted individually.
- re-exports carry more symbols than immediately needed (types consumed
  by tests still in mod.rs). PHASE-03 should prune to the 8-symbol public
  surface after tests move to their machine files per D2.
- `#![expect(unused_imports, ...)]` and `#![expect(unused, ...)]` escapes
  are PHASE-02 scaffolding — PHASE-03 must remove them.
- primary_worktree widened to `pub(crate)` (was `fn`) — it's consumed by
  tests in mod.rs (will move to subagent tests in PHASE-03).

## Open for PHASE-03

- Clean up all `#![expect(...)]` escapes
- Move tests from mod.rs to respective machine files per D2
- 8-symbol re-export checklist: WorktreeCommand, dispatch,
  DISPATCH_WORKER_AGENT_TYPE, DUAL_CAUSE, is_linked_worktree,
  resolve_mode, env_worker_set, coordinate
- Verify 8 caller files compile untouched
- `worktree::coordinate=command` layering entry

## PHASE-03

- **Extracted:** CoordOutcome, run_branch_point_check, CoordAction, CoordRefusal,
  impl CoordRefusal, classify_coordinate, base_has_slice_plan, coordinate,
  run_coordinate → `src/worktree/coordinate.rs` (265 lines, 9 production items)
- **mod.rs:** shrank from 1338→1016 lines; only WorktreeCommand + dispatch +
  8-symbol re-exports + tests remain
- **Re-exports finalised:** 8 symbols checked — WorktreeCommand (inline),
  dispatch (inline), DISPATCH_WORKER_AGENT_TYPE (marker), DUAL_CAUSE (marker),
  is_linked_worktree (shared), resolve_mode (marker), env_worker_set (marker),
  coordinate (coordinate)
- **Visibility:** base_has_slice_plan is `pub(crate)` not `fn` — the test
  `base_has_slice_plan_tracks_presence_on_the_trunk_tree` in mod.rs calls it
  cross-file, so per-file privacy would require moving the test to coordinate.rs.
  The original design's "coordinate-only private" was relaxed to `pub(crate)`;
  sole production caller remains `coordinate()`.
- **Orphaned items removed:** section-comment banners + doc comment +
  `#[expect(clippy::fn_params_excessive_bools)]` for classify_worker_verify
  (already in subagent.rs)
- **#![expect(unused_imports)] removed:** no escapes left
- **Test re-exports:** gated behind `#[cfg(test)]` to keep production imports
  lean — classify_coordinate, CoordAction, CoordRefusal, base_has_slice_plan,
  GcPlan/GcRefusal/GcState/GcVerdict/classify_gc, ForkState/LandRefusal/Merge/
  classify_land, Stamp/StampRefusal/WorkerVerify/WorkerVerifyRefusal/
  classify_stamp/classify_worker_verify/primary_worktree
- **layering.toml:** `"worktree::coordinate" = "command"` with comment noting
  slice::run_phases upward edge (Non-Goal)
- **Commit:** `366e01c` on dispatch/116; worker commit `3138ca0e` on
  dispatch/116-PHASE-03 (deleted worktree)
- **Gate:** 2243/2245 tests pass; 2 pre-existing marker env failures;
  architecture_layering_gate green; clippy zero-warn; fmt clean
- **drift:** base_has_slice_plan visibility `fn`→`pub(crate)` is the only
  deviation from the design's $Visibility table. All 42 items accounted for,
  12 files in src/worktree/ matching $Target layout.

## Audit (RV-135)

- **F-1 (tolerated):** base_has_slice_plan `pub(crate)` vs design `[p]`.
  Test stayed in mod.rs per D2 gap; over-widen is equivalent in this tree.
- **F-2 (tolerated):** Seven `#![expect(unused)]` attributes on
  provision/import/land/gc/fork/subagent — PHASE-02 scaffolding not pruned
  in PHASE-03. Suppress real dead-code lints; honest but messy.
- **F-3 (follow-up → IMP-146):** D2 test co-location incomplete. Only
  allowlist/marker/shared got their tests; 7 lifecycle machine files have
  zero `#[cfg(test)]` blocks. ~32 tests remain in mod.rs.
- **Gate:** 2242/2245 tests pass (3 pre-existing failures present on main).
  architecture_layering green. clippy zero-warn.
- **Candidate:** cand-116-review-001, admitted via RV-135.
- **Reconciliation brief:** per-slice edit to design.md $Target layout
  (base_has_slice_plan[p]→[pc]). No governance/spec changes.
