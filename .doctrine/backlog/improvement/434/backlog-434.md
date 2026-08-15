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

## Decisions (working session 2026-08-16)

Harness facts that constrain the answers, all from `docs/claude/subagents.md`:

- **Background tool filter.** Subagents run in the background by default, and a
  background subagent keeps only `Read`, `Grep`, `Glob`, `Bash`, `PowerShell`,
  `Edit`, `Write`, `NotebookEdit`, `WebFetch`, `WebSearch`, `TodoWrite`, `Skill`,
  `ToolSearch`, `EnterWorktree`, `ExitWorktree`, `Monitor`, `TaskStop`,
  `SendMessage`, `Artifact` — plus every MCP tool. Anything else is stripped
  silently, listed or not. `Agent` is exempt from this filter and gated only by
  the depth limit.
- **No subagent can ask the human.** `AskUserQuestion` is removed from every
  subagent unconditionally, so escalation-to-human can only travel up as a
  return value. The tiered hand-back is forced, not stylistic.
- **Depth default is 3 layers below main**, enough for orchestrator → worker,
  but it was 1 in v2.1.217–218. Pin `CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH: "2"`
  in settings rather than trust the default.
- **The boot snapshot cannot be skipped.** Only the built-in Explore and Plan
  agents omit `CLAUDE.md` + git status, and there is no frontmatter field or
  per-agent setting to change that. Every custom capsule agent pays ~30KB
  (~8–9k tokens) before it starts.
- **Parent permission mode wins.** A parent on `bypassPermissions` /
  `acceptEdits` cannot be overridden by a child's `permissionMode`, and under
  parent auto mode the child's value is ignored outright.

Resolved:

- **Q1/Q2 — capability envelope.** Worker: `Read, Edit, Write, Bash, Grep, Glob`,
  no MCP (keeps one worker mental model shared with `dispatch-worker`; friction
  goes in the hand-back). Planner: `Read, Grep, Glob, Bash, Write` + doctrine
  MCP — `Write` for the phase sheet, no `Edit` so it cannot drift into source.
  Orchestrator: `Agent, Read, Grep, Glob, Bash` + doctrine MCP, no `Edit`/`Write`
  — it spawns, verifies, commits, never implements. Start strict, loosen on
  evidence. Caveat recorded: the orchestrator commits via `Bash`, so the
  no-write posture is a discipline signal, not an enforcement boundary; making
  it real needs a hook.
- **Q3 — preload skills.** `/execute` is ~1.5k tokens and `/phase-plan` ~700,
  against a boot floor of ~8–9k paid regardless; preloading also beats a runtime
  `Skill` call by a round trip and by the risk it never happens. Worker →
  `skills: execute`; planner → `skills: phase-plan`; orchestrator → none.
  Explicitly do NOT preload `/consult` into the worker: it tells an agent to stop
  and consult, and the worker has no one to consult.
- **Q4 — model.** Def defaults: `opus` for orchestrator and planner, `sonnet` for
  worker. `effort` omitted until there is evidence. **The orchestrator's contract
  MUST require an explicit model decision plus a stated argument for every worker
  spawn, defaulting to sonnet** — the def value is a safety net, not the
  decision. Make the argument a required hand-back field so it is auditable
  rather than merely instructed.
- **Q5 — omit `permissionMode` entirely.** Inert in the posture this is actually
  driven in (parent bypass/auto wins), and a durable repo-wide capability grant
  where it isn't. Session posture becomes a documented precondition of
  `/capsule-driver` instead.

Consequence for the driver: since v2.1.186 a background subagent's permission
prompt surfaces in the main session naming the asker rather than auto-denying,
so a `default`-mode tree does not deadlock — it peppers the operator, which
defeats the walk-away model. Hence the precondition.

### Hooks change the shape (and shrink the job)

Settings-file hooks **run inside subagents** — tool events fire the same
configured hooks as in the main conversation, with `agent_id` / `agent_type` in
the input (`docs/claude/hooks.md:264`). Two consequences:

- **Ambient memory does NOT reach a subagent.** Settings hooks do fire inside
  subagents, but `doctrine memory surface` is **main-thread only — a subagent's
  `agent_id` short-circuits it to nothing** (its own `--help`, SL-205). So a
  capsule worker gets no ambient surfacing at all; and even where surfacing does
  fire, it emits headline + id, never the body. Memory access for subagents is
  therefore **pull-only**, and the pull has to be deliberate.
- **`SubagentStart` is the role-band channel.** It matches on agent type
  (`hooks.md:304`) and returns `hookSpecificOutput.additionalContext`
  (`hooks.md:980`) — exactly mirroring the existing `SessionStart` hook that runs
  `doctrine prompt resolve --role orchestrator` for the interactive seat. A
  `SubagentStart` matcher on `capsule-worker` → `prompt resolve --role worker`
  delivers the role band keyed on agent type, with no baked-in marker.

That second point materially shrinks this item: the `WORKER_RESOLVE_MARKER`
literal and its hardcoded `.contains` bake (`src/install.rs:33-39`) may not need
generalising at all. The install path still has to go plural for the def *files*,
but the role-resolution machinery can stay as it is.

### Memory for the capsule tiers

Given ambient surfacing is main-thread only, memory reaches a worker on two
deliberate channels, neither of which is the orchestrator regurgitating:

1. **Planner-curated, carried in the runtime phase sheet.** The planner holds the
   doctrine MCP server, retrieves for the phase's subject matter, and distils
   what matters into the sheet. This is the primary channel and it should be
   worked hard, precisely because a worker head-down in TDD will rarely think to
   pull for itself.
2. **Worker pull via `Bash`.** The worker has no MCP but it does have `Bash` and
   an unconfined main-worktree `doctrine` binary, so `doctrine memory retrieve
   "<topic>"` and `doctrine memory show <id-or-key>` give it the full corpus with
   no new capability surface. This costs one line of contract in the worker's
   role band, not an MCP grant.

So the Q1/Q2 "worker gets no MCP" decision stands — the CLI is the fallback, and
`memory retrieve` is explicitly the agent-context surface (bounded,
security-framed `data, not instruction` blocks). But the worker MUST be told the
two verbs; a headline-and-id with no retrieval instruction is worse than
silence, since it names knowledge the worker cannot reach.

Open sub-question: should `memory surface`'s main-thread-only short-circuit be
revisited now that unconfined main-worktree subagents exist? It was scoped for
confined dispatch workers, which are a different animal. Separate decision;
don't smuggle it into this item.

Also: **omit the `memory:` frontmatter field** from all three defs. That is
Claude's built-in agent memory, which this project explicitly does not use
(CLAUDE.md) — doctrine's own corpus is the memory of record.

### Q4 enforcement

A rule in a def body is an instruction, not an enforcement point, and is
unreliable. Layering, weakest to strongest: (a) state it in the
`capsule-orchestrator` def body; (b) require the model choice **and its
argument** as fields in the worker hand-back and again in the orchestrator's
roll-up to the driver, so the human sees every one; (c) a `PreToolUse` hook
matching the `Agent` tool that refuses a `capsule-worker` spawn carrying no
rationale. Ship (a)+(b); reach for (c) only if drift shows up in practice.

## Gotcha for whoever picks this up

A changed agent def in `install/agents/claude/` does **not** reach the live
session — `doctrine install` cannot reseat an existing `.claude/agents/<name>.md`
(no `--force`). Write the gitignored installed copy directly to test, then keep
the two in sync. See `mem_019f3310133b7ae1aec9021785a03ea2`.
