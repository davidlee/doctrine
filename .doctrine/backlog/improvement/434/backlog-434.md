# IMP-434: Capsule-execution agent defs and a /capsule-driver skill

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

Capsule execution (the RFC-025 / ADR-020 line, exercised live on SL-251) is
being driven through harness subagents, but `install/agents/claude/` ships
exactly one definition — `dispatch-worker` — and that one is bound to the
`/dispatch` contract: `isolation: worktree`, no MCP, no commit, source-delta
hand-back. None of that fits a capsule agent running in the main worktree.
So every capsule agent falls back to `general-purpose`: the catch-all with
`tools: *`, inherited model, no role prompt, and no doctrine contract.

That fallback is the whole problem. The agents doing the work are the ones with
the least contract, and the operator has to re-specify the entire regime in the
spawn prompt every time. The prompt currently in field use (verbatim, so the
distillation has a source):

> you're implementing SL-251 — your role is super-orchestrator. Your discipline
> is to use as little context as possible so that you can complete all 7 phases
> of implementation without exceeding 250k tokens. To do this: spawn subagents
> in the main worktree. Do not use worktrees; do not dispatch. There are no
> other agents active in this vm. There is no confinement on subagent commands.
> You should spawn (opus) orchestrators — each orchestrator alternately spawning
> planners (opus) and workers (sonnet / opus, depending) per phase, verifying
> their work, and escalating (back through you) only if necessary. You and your
> orchestrators have authority to adapt the plan to discharge (or evolve) the
> design if necessary; you should reserve human intervention for high-impact
> decisions or governance conflicts where correct intent cannot be determined.
> Expect one orchestrator to last 2–4 phases before you need to spawn another;
> all agents are most effective (in cost and output quality) at < 250k, so aim
> to keep their work carefully bounded and have your orchestrators do the same.
> Halt when phase implementation is complete (do not begin audit). Begin.

## Proposed shape

Three agent definitions under `install/agents/claude/`, plus one skill:

| artefact | role |
|---|---|
| `capsule-orchestrator` | owns 2–4 phases; alternates planner and worker spawns, verifies hand-backs, escalates upward only when it must |
| `capsule-phase-planner` | expands one phase's authored plan entry into the runtime sheet (`/phase-plan` altitude) |
| `capsule-worker` | executes one phase TDD red/green/refactor in the main worktree; no `.doctrine/` authored writes |
| `/capsule-driver` skill | the interactive super-orchestrator session: sets the token discipline, spawns orchestrators, holds the escalation floor |

The tiering (super-orchestrator → orchestrator → planner/worker) is what keeps
each context under the ~250k band where cost and output quality hold up. The
agent defs are where that discipline stops being prompt boilerplate and becomes
a contract.

Note the deliberate difference from dispatch: **no worktree isolation, no
confinement, agents write in the main worktree and can commit**. DEC-134 —
interactive orchestration persists outside fresh phase capsules — is the
governing decision for why the driver session is not itself a capsule.

## Open questions

Mostly frontmatter. Per `docs/claude/subagents.md` the available knobs are
`name`, `description`, `tools`, `disallowedTools`, `model`, `permissionMode`,
`maxTurns`, `skills`, `mcpServers`, `hooks`, `memory`, `background`, `effort`,
`isolation`, `color`, `initialPrompt`.

- **Q1 — `tools` / `disallowedTools` per tier.** Planner is read+plan (no
  `Edit`/`Write`?); worker is read/write/bash; orchestrator needs `Agent` to
  spawn at all. Does the orchestrator need `Edit` itself, or is it purely a
  spawner+verifier?
- **Q2 — `mcpServers`.** Which tiers get the doctrine MCP server? The
  orchestrator plausibly needs `observation_record` and the phase verbs; the
  worker arguably should hand friction back rather than record it (the
  `dispatch-worker` posture). Unlike a dispatch fork, an unconfined main-worktree
  agent *can* reach the server — so this is a policy choice, not a capability
  limit.
- **Q3 — `skills` preload.** Preloading `/phase-plan` into the planner and
  `/execute` into the worker injects the full skill text at startup — that is
  exactly the per-agent context spend the tiering exists to bound. Measure
  before defaulting it on.
- **Q4 — `model` and `effort`.** The field prompt pins opus for orchestrator and
  planner, sonnet-or-opus for workers "depending". Encode a default in the def
  and let the spawn override, or leave `inherit`?
- **Q5 — `permissionMode`.** The driver session asserts "no confinement on
  subagent commands". Does that become `bypassPermissions` in the def, or stay
  a session-level posture the operator opts into?
- **Q6 — `doctrine-role`.** `dispatch-worker` carries a `doctrine-role: worker`
  key and renders `{{ prompt resolve --role worker }}`. Do the capsule tiers get
  their own roles in the hymns corpus, or reuse `worker`?
- **Q7 — halt boundary.** The field prompt halts at "phase implementation
  complete, do not begin audit". Should that boundary live in the
  `/capsule-driver` skill, the orchestrator def, or both?
- **Q8 — name binding.** `dispatch-worker`'s `name:` is pinned to
  `DISPATCH_WORKER_AGENT_TYPE` in `src/worktree/mod.rs` by a drift test. Do the
  capsule agent types need equivalent constants (STD-001, no magic strings), or
  are they only ever named by a skill's prose?

## Gotcha for whoever picks this up

A changed agent def in `install/agents/claude/` does **not** reach the live
session — `doctrine install` cannot reseat an existing `.claude/agents/<name>.md`
(no `--force`). Write the gitignored installed copy directly to test, then keep
the two in sync. See `mem_019f3310133b7ae1aec9021785a03ea2`.
