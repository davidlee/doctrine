# Claude Agent worktree is harness-born, not fork-provisioned — SubagentStart hook must provision and stamp

When an orchestrator spawns a claude dispatch worker via the `Agent` tool with
`isolation: worktree`, **the harness creates the linked worktree itself** — the
orchestrator never runs `doctrine worktree fork`, so **ADR-006 D9 provisioning
(the gitignored-allowlist copy, withheld tier excluded) has not happened** and the
disk marker has not been stamped. Both are normally `fork --worker`'s job on the
codex/pi path; on the claude path there is no fork.

The only orchestrator-trusted code that runs in that worktree *before* the worker
is the **SubagentStart hook**, and it is the only seam that can discriminate a
dispatch worker: its payload carries `agent_type` (the orchestrator-controlled
`subagent_type`), whereas the `WorktreeCreate` hook payload carries only `name` —
**WorktreeCreate cannot tell a dispatch worker from a benign isolated subagent.**

⇒ The SubagentStart hook (gated on `agent_type == dispatch-worker` AND a
linked-worktree `payload.cwd`) must **provision THEN stamp** — not just stamp. A
design that only stamps leaves the worker unprovisioned; a design that provisions
via WorktreeCreate brands every isolated subagent.

**Why:** missed across 7 SL-056 inquisition rounds and again in the clean-rewrite
first draft (caught as internal-review finding SR-1) — stamping is the visible job,
provisioning is the silent prerequisite that rides the same harness gap.

**How to apply:** the hook command is two acts behind one `agent_type` gate
(`doctrine worktree provision <cwd>` then `marker --stamp-subagent`), reading `cwd`
from the **payload**, never the hook's own process cwd (the hook runs at the
orchestrator root). Relates to
[[mem.pattern.dispatch.claude-subagentstart-worker-identity]] (the empirical hook
facts) and [[mem.pattern.dispatch.spawn-backend-harness-agnostic-no-free-env-seam]]
(the agnostic floor). SL-056 design.md §4b.
