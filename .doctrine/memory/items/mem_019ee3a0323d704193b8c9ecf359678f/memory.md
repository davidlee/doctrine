# SubagentStart hook process cwd is the worker worktree, breaking auto-stamp provision

Claude Code runs the `SubagentStart` hook with **process cwd = the spawned
subagent's own worktree** (`.claude/worktrees/agent-<id>`), NOT the orchestrator
tree. `run_stamp_subagent` (`src/worktree.rs:2110`) resolves the copy SOURCE via
`root::find` from the *process* cwd, assuming it is the orchestrator tree. Under
the real harness, source resolves to the worker worktree — identical to the
payload `cwd` (the fork) — so `verify_sibling_worktree` bails
`fork path is the source tree itself; refusing to provision`
(`src/worktree.rs:417`). The worker is left **unstamped**.

## Empirically proven (IMP-046 fresh-session probe, 2026-06-20)

Spawned a `dispatch-worker` subagent via the Agent tool at `isolation: worktree`
with tracer hooks on `SubagentStart`. Confirmed, in order:
- the hook **fires** for the subagent;
- `matcher: "dispatch-worker"` **matches** on the payload `agent_type` (a matcher'd
  tracer and a catch-all tracer both fired; matcher scoping works);
- the payload `cwd` is the **worker worktree**, `agent_type` correct;
- hook **process `pwd` == payload `cwd`** == the worker worktree — the decisive fact;
- the real stamp command then fails: `fork path is the source tree itself` → no marker.

So the full firing/matcher/cwd chain is sound; the defect is purely the
SOURCE-resolution assumption. Manual hand-stamp succeeds ONLY because it is run
from the orchestrator cwd (`echo '{"cwd":"<worker>"}' | doctrine worktree marker
--stamp-subagent`), making source ≠ fork — which is why ISS-011 notes operators
hand-stamp to unblock workers.

**Why:** the stamp design's load-bearing comment ("the SubagentStart hook fires
inside [the orchestrator tree]") is false for the Agent-tool worktree-isolation
path. The auto-stamp can never provision from the hook as currently written.

**How to apply:** the SOURCE for provision must come from somewhere other than the
hook process cwd — e.g. the repo's primary worktree (`git worktree list` / the
main checkout via `--git-common-dir`), passed explicitly — not `root::find` on the
process cwd. Until fixed, the claude dispatch arm's workers come up unstamped and
must be hand-stamped (or are caught fail-closed by the marker-absent rule, ADR-006
D2a). Routed to [[ISS-011]] as Defect C.

Related: [[mem_019ebfd16f8e7d61bcc01d2050c9db1a]] (Agent worktree is harness-born,
not fork-provisioned), [[mem_019ec84b97407b40a04e595d16dd1f06]] (stamp hook
silently no-fires for nested-session workers), [[mem_019ec0a5bdb274b3a7cc1d5eaf4e34c5]]
(SubagentStart is un-failclosable).
