# WorktreeRemove auto-destroys a subagent worktree on finish

When an `isolation:worktree` subagent **finishes**, Claude Code fires
`WorktreeRemove` and, for git worktrees, **automatically runs `git worktree
remove`** — tearing down the worktree directory. The `WorktreeRemove` hook has
**no decision control**; its failures are logged in debug mode only. Source:
`docs/claude/hooks.md:2442` (behaviour), `:680` + `:814` (no decision control /
failures debug-only).

**Consequence:** any uncommitted work in that worktree is **destroyed** in the
race between subagent-done and removal. A worker confined with ro-`.git` (cannot
self-commit) leaves its result as an uncommitted worktree diff — which is lost
unless captured BEFORE teardown. The fix: a doctrine `WorktreeRemove` (and/or
`SubagentStop`) hook that snapshots `git -C <worktree> diff` (+ untracked) to a
patch OUTSIDE the worktree, which the orchestrator then imports.

**Lifecycle asymmetry (load-bearing):** the **harness** owns the claude
`Agent`-arm worktree (auto-removed on finish), whereas the **orchestrator** owns
the pi/subprocess-arm worktree (`worktree fork --worker` → import → orchestrator
removes). They are NOT lifecycle-equivalent — "import the live worktree after
the worker returns" is safe only on the pi arm.

- Verified against `docs/claude/hooks.md`, 2026-07-01. Origin: SL-182 RV-200 F-3.
- See [[mem.fact.claude.pretooluse-hook-fail-open]],
  [[mem.fact.dispatch.single-slot-arming-rendezvous]].
