# isolation:worktree teardown is conditional on the WorktreeCreate hook

**CORRECTED 2026-07-01 (SL-182 PHASE-05 live probe).** The earlier claim —
"Claude always auto-runs `git worktree remove` when an `isolation:worktree`
subagent finishes" — is **only true in the NATIVE path** (no `WorktreeCreate`
hook installed). It does **NOT** hold for the shipped doctrine config.

## What is actually true

Claude's auto-cleanup applies to worktrees **it created itself**. A
`WorktreeCreate` hook **"replaces the default git behavior entirely"**
(`docs/claude/hooks.md:2390`) — Claude hands off creation and receives only a
path string, so it cannot assume the path is a git worktree. Per
`hooks.md:2442`: *"If you configured a WorktreeCreate hook … pair it with a
WorktreeRemove hook to handle cleanup. **Without one, the worktree directory is
left on disk.**"*

Doctrine ships `create-fork` **as** the `WorktreeCreate` hook
(`plugins/doctrine/hooks/hooks.json`) and ships **no** `WorktreeRemove` hook. So
on the real claude dispatch arm, when the subagent finishes the worktree is
**LEFT ON DISK**, its worker diff (tracked + untracked) **intact**.

## Live probe (2026-07-01, claude-code 2.1.x)

One `isolation:worktree` general-purpose subagent, production hooks
(`create-fork` WorktreeCreate present, no WorktreeRemove). Post-return:
`.worktrees/agent-<id>` still on disk, in `git worktree list` (detached HEAD ==
B), ` M AGENTS.md` + `?? <untracked>` both present. Contrast: PHASE-01's F-T2
observed auto-`git worktree remove` — but that probe carried **no** WorktreeCreate
hook (native path). That native observation was wrongly generalised into the
design's teardown premise.

## Consequence (supersedes the old asymmetry claim)

The claude `Agent`-arm and pi/subprocess-arm are now **lifecycle-symmetric**: the
**orchestrator** owns the worktree on both. Post-return the orchestrator reads the
Agent footer's `worktreePath` (tree still alive), runs `verify-worker --dir` on
it, imports the live working-tree diff (`git -C <wt> diff HEAD` + untracked →
`worktree import --patch`), then `git worktree remove`s it. **"Import the live
worktree after the worker returns" is safe on BOTH arms** — no capture-before-
teardown machinery is required. `verify-worker --dir` fail-closes
(`no-worker-head`) if a tree is ever unexpectedly gone, so "no WorktreeRemove
hook" is an enforced invariant, not just documented.

The SubagentStop diff-capture funnel ([[mem.fact.claude.subagentstop-awaited-tree-intact-capture-seam]])
is therefore **not needed** for the claude arm (its timing facts remain valid
harness knowledge; its role as *the* funnel seam is superseded).

- Corrected against live probe + `docs/claude/hooks.md:2390,:2442`, 2026-07-01.
  Origin: SL-182 RV-200 F-3 (original, wrong); PHASE-05 probe (correction).
- See [[mem.fact.claude.pretooluse-hook-fail-open]],
  [[mem.fact.dispatch.single-slot-arming-rendezvous]].
