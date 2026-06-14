# Notes SL-064: Coordination-branch isolation: dedicated worktree + integration-sync seam for dispatch

Durable per-slice scratchpad - tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Planning

2026-06-14: Authored the SL-064 executable plan and materialised runtime phase
tracking. The plan has seven phases: governance/OQ-D fence, coordination
worktree creation, projection plumbing and run ledger, prepare-review sync,
integrate/replay sync, source skill alignment, and end-to-end proof. Slice
status was advanced to `ready`; planning changes were committed in the same turn
under a `plan(SL-064)` commit.

Verification run for planning: `doctrine slice phases 064`,
`doctrine slice status 064 ready`, `doctrine slice list --filter
coordination-worktree`, `git diff --check`, and ASCII scan over the new plan
files. No code was changed, so `just check` was not run.
