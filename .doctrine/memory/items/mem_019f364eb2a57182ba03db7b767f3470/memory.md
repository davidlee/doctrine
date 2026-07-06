# SubagentStart fires for nested (subagent-initiated) spawns; payload has no parent discriminator

**Claim.** A `SubagentStart` command hook fires for **subagent-initiated**
`Agent`-tool spawns, not only main-thread spawns. Its payload carries
`session_id`, `transcript_path`, `cwd`, `prompt_id`, `agent_id`, `agent_type`,
`hook_event_name` — and **no spawner/parent id**. A hook therefore **cannot**
distinguish a main-thread-initiated spawn from a subagent-initiated one from the
payload alone (`CLAUDE_CODE_CHILD_SESSION` is also not a reliable discriminator —
[[mem_019ec84b53...]]).

**Evidence (SL-206 P0, 2026-07-06).** Confined `dispatch-orchestrator` (agent
`add711b568b9d372e`, jailed at coord root `.dispatch/SL-209`) nest-spawned a
`dispatch-worker` via the `Agent` tool. The observer `SubagentStart` shim logged
**two** entries under one `prompt_id`: the orchestrator (main-initiated) AND the
worker `afd0c43fe5815021e` (subagent-initiated). Both fired; neither payload named
its spawner. Preserved: `scratchpad/p0-subagentstart.log`.

**Field-diff of the two entries (checked 2026-07-06) — no lineage discriminator.**
`session_id`, `transcript_path`, `prompt_id`, `hook_event_name` are **IDENTICAL**
across top-level and nested. Only `agent_id`, `agent_type`, and `cwd` differ; there
is no `parent_agent_id` / depth / `spawned_by`, and **no `env` in the payload**.
The `cwd` difference is an **isolation artifact** (the worker was
`isolation:worktree`), NOT a spawner signal — a *no-iso* nested spawn inherits the
parent's cwd, and the escalation grandchild (no-iso, `dispatch-orchestrator`-typed)
lands at the jailed parent's cwd. So a cwd-based nomination gate is spoofable. The
hook's *process* env (`CLAUDE_CODE_CHILD_SESSION=1`) was constant session-wide and
also does not distinguish top-level from nested. ⇒ nomination cannot gate on any
payload/env identity; only an out-of-jail intent token discriminates.

**Security consequence for SL-206 A2 nomination.** An unjail allowlist keyed on
`agent_type`-match alone (via `SubagentStart`) auto-nominates **any**
`dispatch-orchestrator`-typed spawn — including one a *jailed* `Agent`-holder
requested (e.g. a `general-purpose` subagent invoking
`Agent(subagent_type:"dispatch-orchestrator")`). That is a jailed→unjailed
**escalation**. Type-match is forgeable-by-invocation and the hook can't see who
asked. The fix is a **single-slot intent token** ([[mem.fact.dispatch.single-slot-arming-rendezvous]]-style)
written only by the unconfined principal (main thread) to a path outside every
jail (primary `.doctrine/state/`), consumed by the `SubagentStart` hook: nominate
iff `type-match ∧ token-present`. Two-man rule = identity ∧ intent.

**Also.** Further contradicts [[mem_019ec84b53...]]'s "nested no-fire" — here the
nested spawn DID fire. Treat that older claim as version-fragile.

See `.doctrine/slice/206/unjail-direction.md` §5 (security), and
[[mem.fact.dispatch.confined-orchestrator-nested-worker-machinery-works]] (same probe).
