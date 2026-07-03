# ISS-212: Orchestrator-author phase-completion divergence: prepare-review reads primary-tree sheets while author flips completion from coord cwd

## Symptom

`dispatch sync --prepare-review` bails with `conformance registry incomplete:
recorded row for PHASE-NN, which is not a completed phase` for **every** phase,
even when the slice is genuinely 7/7 complete and boundaries are recorded.

## Root cause

Phase-completion sheets (`.doctrine/state/slice/<id>/phases/phase-NN.toml`) are
**per-worktree runtime state** (gitignored, not shared between worktrees).
`registry_completeness` (`src/state.rs:906`) reads the completed-set via
`completed_phase_ids(project_root=git::primary_worktree(root))`
(`src/dispatch.rs:1901`) — i.e. from the **primary** tree. But an
**orchestrator-author** (driving the phase inline, no worker) runs `slice phase
--status completed` with cwd in the **coord** worktree, so completions land on
coord's sheets and the primary's stay stale. Result: `completed_phase_ids` on
primary returns ∅ → every recorded boundary row reads as `Extra` → gate bails.

Note `record-boundary` / source-deltas DO root on primary (`read_source_deltas` /
`record_source_delta`), so the boundaries ledger is fine — only the completion
**flags** diverge. Bites orchestrator-author dispatch only; worker-driven dispatch
flips completion from root and never hits it.

## Workaround (used at SL-191 close)

`doctrine slice phase <id> PHASE-NN --status completed -p <primary-root>` for each
phase, replaying the completion onto the primary tree. Then prepare-review passes.

## Candidate fixes

- (a) `slice phase … completed` also stamps the primary registry the way
  `record-boundary` does; or
- (b) `prepare-review` reads the completed-set from the `dispatch/<slice>` tip
  (object db) the same way it reads the boundaries ledger; or
- (c) document the orchestrator-author "flip completion with `-p <primary>`" step.

Surfaced during SL-191 close (orchestrator-author, 2026-07-04); also captured in
`.doctrine/rfc/011/case-notes.md`.
