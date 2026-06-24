# WorktreeCreate hook-replace gives deterministic dispatch base control (proven 2.1.181)

A custom `WorktreeCreate` hook **fully replaces** Claude Code's native
`git worktree add` for an `Agent isolation:worktree` spawn — including a
programmatic spawn of a **named** subagent (`dispatch-worker`). The hook creates
the worktree at a base **doctrine chooses out-of-band** and prints the path on
stdout; Claude uses that worktree as-is. **Empirically proven on claude-code
2.1.181** (wtc-probe, 2026-06-25): a named `dispatch-worker` spawned with the
hook active landed in doctrine's path (`.worktrees/wtc-probe`, not
`.claude/worktrees/agent-*`) at doctrine's chosen base `68250bcd` — **overriding
`worktree.baseRef="head"`** (orchestrator HEAD was `adac3738`).

**Why this matters — it collapses the H1 wrong-base hazard.** H1's mechanism is
that under shared-clone lock contention `isolation:worktree` silently falls back
to the main worktree, where `baseRef:head` tracks a **moving `main`**
([[mem.signpost.doctrine.dispatch-claude-arm-wrong-base]]). When **doctrine is
the creator** there is no native creation to fall back to: base is doctrine's,
and a hook failure aborts the spawn fail-closed (any non-zero exit blocks — the
only hook event that does). The race cannot place the worker on a wrong/moving
base.

**The thin payload does NOT block this.** The `WorktreeCreate` stdin payload on
2.1.181 is still minimal — `{session_id, transcript_path, cwd, hook_event_name,
name:"agent-<hex>"}`, **no `agent_type`, no base, no target path** (confirms
[[mem_019ec093bd7b71518489dd187b77f0f0]], now verified on 2.1.181 with a *named*
worker, not just an unnamed one). The earlier conclusion drawn from that payload
— *"not buildable → fall back to SubagentStart-stamp"* — was a **wrong turn**: it
conflated *thin payload* with *can't control base*. The hook needs neither base
nor path in the payload because it **sets** them; doctrine supplies the base via
a known ref / dropped file (orchestrator is sole writer).

**Corrects the IMP-072 premise.** IMP-072 was filed *"base control already solved
by placement; hook NOT needed for base control"* — resting on
[[mem_019ec6142d3b71008f2149a6d84ba981]] (worker forks orchestrator HEAD *when
main is static*). Contention falsified the placement premise; this probe shows
the hook **is** the deterministic base-control mechanism placement only
approximated.

**The matcher does NOT scope — confirmed (same probe).** A
`WorktreeCreate matcher:"dispatch-worker"` fired for BOTH a `dispatch-worker` and
a `general-purpose` isolated subagent — the hook is **repo-global**, intercepting
every `isolation:worktree` spawn. Unlike `SubagentStart` (whose payload carries
`agent_type`, so its matcher works), `WorktreeCreate`'s payload has no
`agent_type`, so Claude cannot scope it. This is the real cost (ADR-011 D7 σ
blast-radius): the production hook must **discriminate without a matcher**.

**How to apply.** Build the claude dispatch arm's worker creation as a repo-global
`WorktreeCreate` hook that branches on an **out-of-band orchestrator marker**
(doctrine is sole writer; serial dispatch makes it race-free): the orchestrator
drops the intended base + a "dispatch worker pending" flag immediately before the
`Agent` spawn; the hook consumes it → `doctrine worktree fork --worker` at base B,
fail-closed, folding in ADR-006 D9 provisioning + the worker-marker stamp as one
trusted act ([[mem_019ebfd16f8e7d61bcc01d2050c9db1a]]). With **no** marker (a
benign isolated subagent), the hook must **pass through** — replicate default
creation (`git worktree add <path> HEAD`) so non-dispatch subagents still work.
