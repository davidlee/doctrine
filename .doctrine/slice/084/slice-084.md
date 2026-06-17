# Decompose dispatch harness routing: per-harness spawn templates, model selection, and agent-def parity for pi/codex/claude

## Context

`/dispatch` routes to two arms via a `codex/pi` vs `claude` split (ADR-011 D3).
The split is structurally correct — pi workers are subprocesses, claude workers
are in-session Agent tools — but the implementation lumps pi and codex under a
single `codex exec` spawn template, ignores model selection entirely, and has no
dispatch-worker agent definition for pi. Meanwhile `CLAUDE.md` is a symlink to
`AGENTS.md`, so Claude-specific reviewer/mcp instructions leak into pi agents.

ADR-011 D3's altitude table names the distinctions; this slice closes the gaps
where the skills and agent definitions have not caught up to the table.

## Scope & Objectives

### 1. Per-harness spawn templates (fix `dispatch-subprocess`)

`dispatch-subprocess/SKILL.md` hardcodes `codex exec "<prompt>"` for both codex
and pi. pi does not have `codex exec` — it is `pi -p "prompt"` (confirmed:
`pi-subagents` runner spawns `{ command: "pi", args: ["-p", task] }`).

- **Add a distinct pi spawn variant** to the subprocess arm: `pi -p
  "<pre-distilled prompt>"` (or the pi subprocess equivalent that binds cwd).
- **Codex spawn stays as-is** with a note that it is codex-specific and untested
  (codex subagent system differs; this slice does NOT test codex end-to-end —
  see Non-Goals).
- The harness detection in the router must distinguish pi from codex (currently
  both lumped as "codex/pi"). An env marker cross-check (`PI_VERSION` / `PI_HOME`
  / lack of `CODEX_*`) suffices.

### 2. pi dispatch-worker agent definition

pi's `subagent` tool supports agent definitions (`/workspace/doctrine/.pi/agents/`).
There is no `dispatch-worker` agent for pi.

- **Create `dispatch-worker.md`** in `.pi/agents/` — a pi equivalent of
  `.claude/agents/dispatch-worker.md`, adapted to pi's tool surface and conventions.
- **Model:** the existing pi worker agent uses `deepseek/deepseek-v4-pro`. The
  dispatch worker should follow the same model convention, but the skill must
  document how to override (model is a project preference, not baked into the
  framework).
- The agent definition must carry the worker contract (source-only mutation,
  single non-merge commit, verify-before-commit, structured report, no
  `.doctrine/` writes).

### 3. Model selection surface in the dispatch skills

Neither arm currently addresses model choice. The pre-distilled worker prompt
should carry a model hint the orchestrator can set. For pi this is the
`--model` flag on `pi -p`; for claude the `model` field on `Agent`.

- The `/dispatch` router gains a `model=<model_id>` dial (default:
  project-convention, not hardcoded).
- The spawn templates in both arms carry the model override where the harness
  supports it.
- Default model is documented, not enforced — it's a project preference.

### 4. Harness-specific content in AGENTS.md / CLAUDE.md

`CLAUDE.md → AGENTS.md`, and AGENTS.md contains Claude-specific instructions:

- "default reviewer: codex mcp — use default (GPT-5.5) for external adversarial
  reviews. Opus sub-agent is also useful for variety on subsequent passes."

This is read by pi agents and misapplied (pi has no `codex mcp`, no Opus).

- **Extract harness-specific instructions** into a conditional or separate file
  that each harness reads selectively. Options (to be resolved in design):
  - (a) Move Claude-specific content into `.claude/CLAUDE.md` or a settings hook,
    keep AGENTS.md harness-agnostic.
  - (b) Guard with a harness-detection preamble in AGENTS.md.
  - (c) Break the symlink — `CLAUDE.md` gets Claude instructions, `AGENTS.md`
    stays shared conventions only.

### 5. Router harness detection refinement

The dispatch router currently checks "codex/pi vs claude" as two blocks with
a `CLAUDECODE` env marker cross-check. This is insufficient to distinguish pi
from codex within the "subprocess-capable" block.

- **Add pi-specific env detection** (`PI_VERSION` or equivalent) to the routing
  table.
- **Add codex-specific env detection** (whatever codex exposes — to be
  researched in design).
- The three-way routing becomes: pi → `/dispatch-subprocess` (pi variant),
  codex → `/dispatch-subprocess` (codex variant), claude → `/dispatch-agent`.

### Affected files

- `plugins/dispatch/SKILL.md` — router harness detection table
- `plugins/dispatch-subprocess/SKILL.md` — spawn templates (add pi variant)
- `plugins/dispatch-agent/SKILL.md` — may need model selection docs
- `AGENTS.md` — extract Claude-specific instructions
- `.pi/agents/dispatch-worker.md` — NEW (pi dispatch worker agent definition)
- `.claude/agents/dispatch-worker.md` — may need model field docs
- `.claude/skills/` — no change expected (installed copies are derived)

## Non-Goals

- **codex end-to-end dispatch testing.** codex has its own subagent system
  (different from Claude's Agent tool and different from pi's `pi-subagents`
  runner). This slice adds the codex spawn template but does NOT test or validate
  it end-to-end; that is a separate spike.
- **ADR amendments.** ADR-011's altitude table is correct; this slice implements
  what the table already describes, not changes the architecture.
- **Model as a framework primitive.** Model choice is a project preference, not
  a doctrine framework concern. The slice adds the surface to control it; it does
  not ship a default model policy.
- **pi subagent model selection mechanics.** pi's `subagent` tool model resolution
  (CLI flag vs agent-def YAML vs session default) is pi internals; the skill just
  passes the flag.
- **Dispatch-worker agent for codex.** codex's subagent system is not yet
  characterized; deferred to the codex spike.

## Risks & Assumptions

- **ASM-1:** pi's `-p` flag accepts a model override (`--model`) and respects it
  in subprocess mode. To be verified in design.
- **ASM-2:** codex has an env marker that distinguishes it from pi (to be
  researched).
- **RSK-1:** AGENTS.md separation may need a `doctrine boot` update to handle
  harness-conditional content — scope risk.
- **RSK-2:** pi subagent `--model` flag may interact unexpectedly with agent-def
  YAML model overrides (which takes precedence?). Mitigation: test the flag path.

## Open Questions

- **OQ-1:** Should AGENTS.md/CLAUDE.md separation be in this slice or split to a
  follow-up? It is mechanically simple (move content / break symlink) but may
  interact with `doctrine boot` expectations. Listed in scope for now; can be
  deferred to a follow-up if design reveals coupling.
- **OQ-2:** What env marker reliably distinguishes pi from codex? `PI_VERSION` is
  set inside the pi binary's process but may not propagate to the orchestrator.
- **OQ-3:** Does pi's `--model` flag override or compose with the agent-def YAML
  model field? The spawn template must emit the right incantation.

## Verification intent

- **VA:** Dispatch router correctly detects pi, codex, and claude (cross-check:
  self-belief vs env markers). Mismatch/unknown refuses, naming the cause.
- **VT:** pi spawn template `pi -p --model <M> "<prompt>"` launches a worker that
  arrives in the correct worktree cwd with `DOCTRINE_WORKER=1` set.
- **VA:** pi `dispatch-worker.md` agent definition carries the worker contract
  (source-only, single commit, verify, no `.doctrine/` writes).
- **VA:** AGENTS.md is harness-agnostic; Claude-specific content is only visible
  to Claude agents.
- **VT:** Dispatch skills build/install cleanly (no broken refs, install copies
  match source).

## Summary

The dispatch routing is structurally correct but the implementation has three
gaps: (1) pi needs its own spawn template (not `codex exec`), (2) pi needs a
`dispatch-worker` agent definition matching the claude one, and (3) model
selection needs a surface in the skills. Additionally, Claude-specific reviewer
instructions in AGENTS.md leak into pi agents. This slice closes all four gaps
without changing the ADR-level architecture. codex end-to-end validation is
explicitly deferred.

## Follow-Ups

- codex dispatch end-to-end spike (separate slice / backlog item)
- codex dispatch-worker agent definition (requires codex subagent characterization)
- Project-level model selection policy (a project preference, not a slice)
