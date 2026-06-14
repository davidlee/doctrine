# claude Agent isolation worktree forks the Bash-tool cwd HEAD; cd into the coord tree to place workers at B

**SL-067 dispatch, controlled probe (2026-06-14).** The /dispatch orchestrator
session was rooted at `main` (HEAD a438902), but `dispatch/067`'s base `B` was
`26a3125` (one commit behind — the resolved trunk). A no-commit probe
`dispatch-worker` `Agent` with `isolation: worktree` was spawned **after `cd`-ing the
Bash tool's cwd into the coordination worktree**; the worker's worktree HEAD came
back **== `26a3125` (B)**, not the session-root `main` HEAD.

**The concrete handle is the Bash tool's persistent cwd**, not the immovable Claude
Code session project root. This makes [[mem.pattern.dispatch.claude-isolation-worktree-forks-orchestrator-session-head]]
actionable from a main-rooted session: you do NOT need to relaunch the session in the
coord tree. Instead:
- `cd` Bash into `/path/to/.worktrees/dispatch-<slice>` (HEAD == B) before EVERY
  worker spawn; the Agent worktree forks that cwd's HEAD.
- **Serial dependent phases self-base:** after the funnel commits the batch
  (`B→B+1`) *in the coord tree*, the coord tree's checked-out HEAD advances, so the
  next worker (Bash cwd still the coord tree) forks `B+1` — phase N+1 sees phase N's
  code with zero extra plumbing. `worktree.baseRef='head'` is honoured against the
  Bash cwd.
- Keep the Bash cwd parked in the coord tree across the whole drive loop; only step
  out (to `main`/root) for authored writes — slice status, audit, memory.

Verified each batch with `verify-worker`/direct `S^==B`: both PHASE-01 and PHASE-02
workers forked exactly their intended base.
