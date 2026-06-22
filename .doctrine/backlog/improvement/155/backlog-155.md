# IMP-155: Per-harness + per-model agent instruction injection

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-155.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

Doctrine agents receive instructions from three tiers, but only two have a home:

| Tier | Known when | Examples | Current mechanism |
|------|-----------|----------|------------------|
| **Universal** | always | AGENTS.md rules, governance, conventions | AGENTS.md + boot snapshot (sentinel-gated) |
| **Harness-specific** | session init | "MCP tools available" vs "none"; spawn model; confinement | Ad-hoc — tool defs injected by harness, but no structured *instruction* layer |
| **Model-specific** | **unspecified** | "you're DeepSeek — weak at multi-step git"; "you're Gemini — no thinking mode" | **Nowhere** |

The boot snapshot already solves tier 1. Tier 2 is partially addressed by IMP-116
(pi APPEND_SYSTEM.md), but there's no canonical *directory* for per-harness or
per-model instruction files, and no `doctrine boot` extension to resolve them.

The existing `.doctrine/agents/` directory has an embryo of the harness pattern:

```
.doctrine/agents/
├── AGENTS.md                  # universal
├── dispatch-worker.md         # general worker agent
├── claude/dispatch-worker.md  # claude-specific worker
└── pi/dispatch-worker.md      # pi-specific worker
```

But this is only for *agent definitions* (the `subagent_type: dispatch-worker`
profile). There is no parallel structure for *orchestration-agent instructions*
(the instructions the agent running `dispatch setup` / `/execute` / `/audit`
receives about its own harness and model).

## Design

### 1. Extend `.doctrine/agents/` with instruction tiers

```
.doctrine/agents/
├── AGENTS.md                       # universal (existing)
├── harness/                        # NEW — per-harness instructions
│   ├── pi.md                       #   pi-specific: "MCP tools available; prefer them"
│   ├── claude-code.md              #   claude-code-specific
│   └── _fallback.md                #   fallback: "no MCP tools; use CLI with --json"
├── model/                          # NEW — per-model behavioural guidance
│   ├── anthropic/
│   │   ├── claude-sonnet-4.md      #   precise tool-user, good at git reasoning
│   │   ├── claude-sonnet.md        #   family-level fallback
│   │   └── _default.md
│   ├── google/
│   │   ├── gemini-2.5-pro.md
│   │   └── _default.md
│   ├── deepseek/
│   │   └── _default.md             #   "weaker at multi-step git — prefer doctrine CLI over raw git"
│   └── _fallback.md                #   global fallback
├── dispatch-worker.md              # existing — agent definition
├── claude/dispatch-worker.md       # existing
└── pi/dispatch-worker.md           # existing
```

The *instruction files* (`harness/*.md`, `model/**/*.md`) are plain markdown —
user-authored, user-edited. The path *is* the resolution key.

### 2. Extend `doctrine boot` with model emission

Two flags, one new:

```
doctrine boot [--harness <name>] [--emit-model-instructions --model <id>]
```

- `--harness <name>` — **existing** (or added in IMP-116): compose harness-tier
  instructions into the boot snapshot, written to boot.md (sentinel-gated).
- `--emit-model-instructions --model <id>` — **new, read-only, idempotent**:
  resolve `model/<vendor>/<family>` tree, emit matched file to stdout. Writes
  nothing. Carries a lightweight MODEL-BOOT sentinel so handover can skip re-query.

The two calls are independent and composable. The harness can call both at session
init (ideal), or the agent can call `--emit-model-instructions` after init (fallback).

### 3. Model resolution algorithm

`--model anthropic/claude-sonnet-4`:

```
split at "/" → vendor="anthropic", rest="claude-sonnet-4"
1. .doctrine/agents/model/anthropic/claude-sonnet-4.md   → exact match? use it
2. .doctrine/agents/model/anthropic/claude-sonnet.md     → strip last -segment? use it
3. .doctrine/agents/model/anthropic/_default.md           → vendor default
4. .doctrine/agents/model/_fallback.md                    → global fallback
5. emit nothing
```

Step 2 walks up by dropping trailing `-`-delimited segments until it finds a file
or hits the vendor prefix. The user controls specificity by naming.

### 4. Graceful degradation

Any missing file = no output for that tier. Model not in tree = silence (or
global `_fallback.md`). No harness file = agent operates on universal only.
Nothing breaks — model instructions are fine-tuning, not required.

### 5. Self-identification contract

The model id must reach the agent somehow. Two paths:

- **Preferred**: the harness passes `--model` to `doctrine boot` at session init.
  pi would inject `DOCTRINE_MODEL=anthropic/claude-sonnet-4` or call both boot
  variants before the agent runs. The agent sees a unified context.
- **Fallback**: the harness instruction tells the agent to self-identify,
  potentially choosing from a list, then call `doctrine boot --emit-model-instructions
  --model "<id>"`. More fragile but works when harness can't or won't inject.

The boot command supports both by being stateless and composable.

### 6. Reconciliation with existing `.doctrine/agents/`

The existing `dispatch-worker.md` files are *agent definitions* (name, description,
tools, model), not *instructions* for the orchestrator agent. They remain where
they are. The new `harness/` and `model/` directories are an orthogonal axis —
instruction for the *reading agent*, not definition of a *subagent*.

The existing `pi/dispatch-worker.md` and `claude/dispatch-worker.md` already
demonstrate per-harness agent definitions. The new harness instruction files
(`harness/pi.md`, `harness/claude-code.md`) would contain complementary content:
tool availability, spawn notes, confinement constraints — things the orchestrator
needs to know, not things a worker needs.

### 7. Content split

**Harness instructions** — about *mechanism*:
- "MCP tools available for: review, memory, backlog" (or "no MCP tools — use CLI")
- "Worker spawn is via subprocess with env injection" vs "Agent isolation:worktree"
- "Thinking mode available" / "not available"
- "Session model resolved at init" / "model may change per request"

**Model instructions** — about *behaviour and constraints*:
- "Weak at multi-step git — prefer `doctrine` CLI over raw git"
- "Precise tool-user, strong at structure — prefer MCP tools over bash"
- "Context window limit, known footguns in this project"
- "This model not tested on dispatch funnel — use serial mode"

The harness tells the agent *what it can do*; the model tells it *how to adapt*.
They are orthogonal.

## Related

- IMP-116 — Deliver boot snapshot to pi via APPEND_SYSTEM.md. The `--harness` flag
  and harness instruction files would compose into the same boot pipeline.
- ADR-011 — Harness-agnostic orchestrator. Establishes per-harness capability
  altitude table (infrastructure layer). This item extends the concept to the
  *instruction* layer.
- `.doctrine/agents/claude/dispatch-worker.md` — existing per-harness agent definition.
