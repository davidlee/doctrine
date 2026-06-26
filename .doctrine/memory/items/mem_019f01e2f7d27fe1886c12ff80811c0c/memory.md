# Claude dispatch worker cannot get per-worktree env via hooks

A Claude `Agent`-tool worker spawned with `isolation: worktree` **inherits the
orchestrator's environment frozen at spawn time** and there is **no hook channel to
give it a different env var** (e.g. a per-worktree `CARGO_TARGET_DIR`).

Verified empirically (SL-156 Probe 2: probe hook in `settings.local.json`,
hot-reloaded, throwaway worktree-isolated worker):

- **SubagentStart** fires for the worker with `cwd` = the worktree, but its process
  has **`CLAUDE_ENV_FILE` UNSET** → cannot persist env (it can only inject
  `additionalContext` text). Corroborates [[mem_019ebfb61ba870219aafc14f8dc7da3b]].
- **CwdChanged** (the only hook that *would* carry `CLAUDE_ENV_FILE` per-directory)
  **does NOT fire** when the harness places the worker in its worktree.
- Inside the worker's Bash, **`CLAUDE_ENV_FILE` is unset** — the env-file channel
  does not exist in the subagent at all.
- The worker's `CARGO_TARGET_DIR` was the jail-wide value, un-overridden; its other
  env vars matched the orchestrator's pre-spawn snapshot.

`CLAUDE_ENV_FILE` (SessionStart/CwdChanged/FileChanged) works fine for the
**orchestrator's own** subsequent Bash calls — just not across the subagent boundary.

**How to apply:** do not design per-worker env injection for the claude arm around
hooks or `CLAUDE_ENV_FILE`. Channels that DO reach the worker: the worker prompt
(per-call compliance — fragile), or a tool that derives env from its own cwd at run
time (e.g. a justfile `export CARGO_TARGET_DIR := …/wt/<basename(toplevel)>`, which
overrides the inherited env for every recipe — harness-independent, both arms).
General principle: worker identity / config cannot ride a free env seam
([[mem_019ebeeda9c27f03808fdeeafb0e93cc]]); orchestrator-controlled per-spawn
channels are the WorktreeCreate payload cwd
([[mem_019efe28d60b7d51998f1f7912b8e7b8]]) and the prompt, not env.
