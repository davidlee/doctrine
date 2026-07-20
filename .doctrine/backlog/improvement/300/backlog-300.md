# IMP-300: dispatch funnel: sync coord worktree after object-db ref advances

## Problem

`dispatch_import` and `dispatch_conclude_phase` advance the coord branch ref
(`dispatch/<N>`) via object-db only — they write commits through `merge-tree` +
`commit_tree` without touching the checked-out working tree or index. After
every funnel write, the coord worktree sits at the OLD tip (B), showing every
landed file as a staged deletion (reverse-diff).

This is a severe footgun: a pathless `git commit` in this window would commit
mass reversions of the just-landed delta. The orchestrator must manually
`git restore --source=HEAD --staged --worktree -- <paths>` after EVERY
`dispatch_import` and `dispatch_conclude_phase` call.

## Impact

Recurred **6+ times** across SL-199 through SL-222 (RFC-011 case-notes
analysis §2). Every batch-import in a dispatched drive pays the manual sync
tax. The regression-diff verify beat also runs on stale content if the sync is
missed.

## Evidence

- SL-206-drive-p04: coord worktree stale post-import
- SL-210-drive: must re-sync after EVERY MCP funnel write; skills don't mention
  this beat
- SL-213-drive-4-correction: object-db only, "severe footgun"
- SL-219-drive-1e5229fa: reverse-diff `git status` + manual restore round-trips
- SL-220 PHASE-07 conclude: boundaries.toml staged deletion; prepare-review
  clobbered the just-landed row
- SL-221 orch session: sync needed before regression verify

## Proposed fix

Add `--sync-worktree` flag to `dispatch_import` and `dispatch_conclude_phase`,
or auto-refresh the checked-out tree after every ref advance. The precondition
is safe: the tree is belt-verified clean pre-import, so a `git restore
--source=HEAD --staged --worktree` is deterministic and cannot clobber
uncommitted work.

Alternatively, the import/conclude tools could refresh the working tree
automatically as part of their write (opt-out via `--no-sync-worktree` if
needed for backward compatibility).
