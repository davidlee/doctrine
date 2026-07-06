# Workflow-spawned agents do not receive the Agent tool

The Claude Code **Workflow** runtime strips the `Agent` (Task) tool from every
subagent it spawns via `agent()`. Workflow agents are **leaves by construction** —
they cannot nest-spawn children, regardless of what their agent def's `tools:`
list declares. This is a property of the *spawn primitive*, not the def.

## Proof (two-path probe, Claude Code 2.1.198)

Same `dispatch-orchestrator` def, spawned two ways, each asked only to introspect
its own toolset:

| Spawn path | def | `Agent` tool? | tools returned |
|---|---|---|---|
| Main-thread **Agent tool** (`subagent_type`) | `dispatch-orchestrator` | **YES** | Read, Edit, Write, Bash, **Agent**, +6 dispatch MCP |
| **Workflow** `agent({agentType})` | `dispatch-orchestrator` | **NO** | Read, Edit, Write, Bash, +6 dispatch MCP, `StructuredOutput` (no `Agent`, no Grep/Glob) |
| **Workflow** `agent()` (default) | `general-purpose` (`Tools: *`) | **NO** | Artifact, Bash, Edit, Read, ReportFindings, SendUserFile, Skill, ToolSearch, Write, `StructuredOutput` — no `Agent`; `ToolSearch select:Agent` → "no matching deferred tools" |

The third row is **decisive**: even a `Tools: *` (wildcard) def gets `Agent`
stripped in a workflow — so the strip is NOT an allowlist intersection against
the def, it is the workflow runtime **unconditionally** withholding `Agent`. Task
primitives (`TaskCreate`, `SendMessage`) are present but none accept a
`subagent_type`, so they cannot spawn a subagent either. Nesting is capped at one
level — this matches the documented `workflow()` "one level only" rule, now
confirmed for the `Agent` tool too. The workflow path also **strips Grep/Glob**
and **injects `StructuredOutput`** — it applies its own tool profile, overriding
the def's `tools:`.

Docs confirm the *general* subagent rule but never promise it for workflow agents:
`docs/claude/subagents.md:376` (listing `Agent` in `tools` grants nest-spawn),
`:784` (subagents can nest-spawn since v2.1.172), `:790` (depth limit 5). Those
apply to the **Agent-tool** spawn path only.

## Consequence for dispatch / `/drive-slice` (SL-206)

A confined dispatch orchestrator spawned **as a workflow agent** cannot reach the
working **claude-arm** worker spawn (the harness-privileged worktree fork-create,
which needs the `Agent` tool). It falls back to the **pi-arm** bootstrap script's
bash `git worktree add`, which fails on the RO shared `.git` in the jail
(`fatal: cannot lock ref refs/heads/dispatch/w-NNN-pNN`). Net: no worker fork,
no delta to import, no phase ever reaches Completed. Placement (coord-root cwd)
and SL-199 positional arming both proved out first — the defect is downstream of
both, and is the **wrong harness primitive**, not placement or git permission.

**Fix direction:** drive via an **Agent-tool-spawned** orchestrator subagent
(which retains `Agent`, proven above), OR have the workflow **script** spawn the
worker directly as a leaf — a workflow `agent(isolation:'worktree')` DOES mint the
confined fork at the armed base ([[mem_019f331005d776c1a65c65bfe59581bf]], SQ3) and
DOES reach doctrine MCP ([[mem_019f328b116a7172ba7eabef25ed979d]]). The wall is
strictly the *spawn-a-child* step, which only `Agent` unlocks.

## Refines / relates

- [[mem_019f328b116a7172ba7eabef25ed979d]] — "MCP-funnel viable for a
  workflow-spawned orchestrator" is TRUE for the funnel *reads/writes* but does
  NOT imply the orchestrator can spawn a worker; this fact is the missing caveat.
- [[mem_019f331005d776c1a65c65bfe59581bf]] — positional arming holds for a
  workflow-spawned worker fork; the orchestrator layer is what a workflow can't
  realize.
- [[mem_019f328ad54c70318abc6f7e1db767fc]] — WorktreeCreate hook payload cwd =
  session working-dir (placement mechanics).
