# SubagentStop fires awaited and tree-intact before WorktreeRemove — the capture seam

Live-proven on claude-code 2.1.x / NixOS bwrap 0.11.2 (SL-182 PHASE-01 probe,
two `isolation:worktree` subagents driven via the `Agent` tool). Complements
[[fact.claude.worktree-remove-auto-teardown]] (which establishes the tree IS
destroyed) by pinning the *ordering and timing* that makes diff-capture viable.

## What is true

1. **Awaited / blocking.** SubagentStop runs to completion before the subagent is
   allowed to stop, and it **honours `exit 2`**: a one-shot exit-2 held the stop —
   the subagent was re-activated, then stopped again on the next pass (two `STOP`
   events ~3s apart for the same `agent_id`). So the hook can veto/hold a stop.
   (Stop-loop hazard: an unconditional `exit 2` blocks the subagent forever —
   sentinel-guard any exit-2 to one-shot.)
2. **Tree-intact, before teardown.** When the hook runs, the worktree is still on
   disk: `[ -d <wt> ]` and `git -C <wt> diff` both succeed. SubagentStop fires
   **before** the harness's `git worktree remove`. ⇒ capturing
   `git -C <wt> diff` (+ `--cached` + untracked tar) into a path OUTSIDE the
   worktree is the **funnel-import seam** for the claude dispatch arm (no
   self-commit needed; the orchestrator imports the captured patch).
3. **Payload shape.** SubagentStop stdin carries `agent_id`, `agent_transcript_path`
   (alias `transcript_path`), `cwd` (= the worktree), `agent_type`, plus
   `stop_hook_active`, `session_id`, etc. It carries **NO `worktree_path`** (RV-202
   confirmed live) — you must reconstruct the worktree yourself.
4. **Correlator.** Resolve the worktree from `agent_id`:
   - (a) `<repo>/.worktrees/agent-<agent_id>` — the live `agent_id` token equals
     the worktree-name suffix; simplest, **winner by probe order**.
   - (c) an `agent_id → cwd` map written by a **SubagentStart** hook — robust
     fallback, independent of the naming convention if it ever drifts.
   - (b) the hook process `cwd` / parse `agent_transcript_path` — also resolves.
   Validate any candidate with `git -C <p> rev-parse --is-inside-work-tree` before
   trusting it (skips stale `.worktrees/*` orphans not in `git worktree list`).

## Why it matters

This is the load-bearing fact behind dispatch **Path L** (SL-182): the claude
Agent arm's worktree is harness-owned and auto-removed
([[fact.claude.worktree-remove-auto-teardown]]), so the worker cannot self-commit
(`.git` ro for linked worktrees) and any "import the diff" cadence MUST snapshot
**before** teardown. SubagentStop being awaited + tree-intact is what makes that
snapshot possible without racing the remove.

## How to apply

- Register the capture on **SubagentStop** (plugin `hooks.json` or
  `settings.local.json` — both load at session start, no hot-reload). Pair with a
  **SubagentStart** recorder if you want correlator (c).
- Capture to a path OUTSIDE the worktree; the capture path must **always `exit 0`**
  (never block the stop) — gate any blocking assertion behind a one-shot sentinel.
- See [[pattern.dispatch.claude-worktree-subagent-bwrap-confinement]] for the
  PreToolUse(Bash)+bwrap confinement wall that runs alongside this.
