# IMP-222: doctor: warn on accumulated dispatch worktree junk + gc affordance

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

Surfaced during SL-182 PHASE-05 design (symmetric-import pivot, 2026-07-01). The
claude dispatch arm leaves each worker's `isolation:worktree` tree **on disk**
post-return (custom `create-fork` WorktreeCreate hook + no WorktreeRemove hook ⇒
Claude does not auto-remove — see [[mem.fact.claude.worktree-remove-auto-teardown]]).
The orchestrator removes each tree after importing it (`git worktree remove`), so
in the happy path nothing accumulates. But if the orchestrator **crashes or is
interrupted mid-drive**, spent worker trees under `.worktrees/agent-<id>` (and
their `git worktree` registrations) pile up.

## Want

- `doctrine doctor` (or `worktree doctor`): warn when dispatch worktree
  junk accumulates beyond a threshold — orphaned `.worktrees/agent-*` dirs and/or
  prunable `git worktree list` entries.
- An obvious gc affordance to reap them (likely fronting `git worktree prune` +
  the existing `worktree gc` / `remove_worktree_dir` machinery).

Not new capability so much as an operability check + a discoverable cleanup path.
Low urgency — the happy path self-cleans.
