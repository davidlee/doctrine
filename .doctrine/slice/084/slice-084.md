# Decompose dispatch harness routing: per-harness spawn templates, model selection, and agent-def parity for pi/codex/claude

## Context

`/dispatch` routes to two arms via a `codex/pi` vs `claude` split (ADR-011 D3).
The split is structurally correct — pi workers are subprocesses, claude workers
are in-session Agent tools — but the implementation lumps pi and codex under a
single `codex exec` spawn template, ignores model selection entirely, and has no
dispatch-worker agent definition for pi. Meanwhile `CLAUDE.md` is a symlink to
`AGENTS.md`, so Claude-specific reviewer/mcp instructions leak into pi agents.

**SL-081 dispatch field notes** (`notes.md`) provide empirical ground truth:
- `codex exec --cd "$D" -s workspace-write` edits source correctly but **cannot
  commit** in bwrap jails — the sandbox blocks `.git/` writes.
- pi's `subagent` tool (with `cwd: "$D"`) **can commit** — it inherits
  orchestrator filesystem permissions.
- `DOCTRINE_TRUNK_REF=main` is required in the bwrap jail because SSH push is
  disabled and `origin/HEAD` is stale.
- Worker marker blocks 3 e2e tests (`e2e_adr_cli_golden`); `just check` cannot
  be the baseline-verify for worker forks.
- AGENTS.md Claude instructions confirmed leaking (harmless in this session but
  structurally wrong).

ADR-011 D3's altitude table names the distinctions; this slice closes the gaps
where the skills and agent definitions have not caught up to the table, informed
by the SL-081 empirical data.

## Scope & Objectives

### 1. Per-harness spawn templates (fix `dispatch-subprocess`)

`dispatch-subprocess/SKILL.md` hardcodes `codex exec "<prompt>"` for both codex
and pi. This is wrong on two counts:

- **pi:** `codex exec` blocks `.git/` writes in bwrap jails (empirically
  confirmed, SL-081 notes). pi's `subagent` tool is the correct spawn mechanism:
  `subagent(agent="dispatch-worker", task="<prompt>", cwd="$D")`. The worker
  runs as a pi subprocess in the fork; cwd binding, marker, and
  `DOCTRINE_WORKER` env still apply (subprocess mechanism).
  - **Dependency:** `subagent` is not built-in pi — it requires the
    `pi-subagents` extension. Doctrine must either (a) declare it as a
    recommended/required extension in install docs, (b) detect its availability
    at dispatch time and refuse with a clear message if missing, or (c) provide
    a `pi -p` fallback for environments where the extension can't be installed.
- **codex:** `codex exec` may or may not work — untested. Stays as the codex
  spawn variant with a documented "untested" caveat.

- **Add a pi `subagent` spawn variant** to the subprocess arm: use the
  `subagent` tool with `agent: dispatch-worker`, `cwd: $D`, and the
  pre-distilled prompt as `task`.
- **Codex spawn stays as-is** (`codex exec`) with a note that it is
  codex-specific and untested (codex subagent system differs; this slice does
  NOT test codex end-to-end — see Non-Goals).
- The harness detection in the router must distinguish pi from codex (currently
  both lumped as "codex/pi"). Env marker cross-check: pi has `PI_HOME` or the
  orchestrator is itself a pi agent; codex detection TBD in design.

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

Neither arm currently addresses model choice. Empirical data (SL-081 notes):
the pi `worker` agent's default model (`deepseek/deepseek-v4-pro`) performed
correctly without override. Model selection should be controllable but not
mandatory.

- The `/dispatch` router gains a `model=<model_id>` dial (default:
  project-convention, not hardcoded).
- **pi:** model passed via `subagent`'s `model` parameter, or defaults to the
  agent-def YAML model.
- **claude:** model passed via `Agent`'s `model` field.
- **codex:** TBD (untested).
- Default model is documented, not enforced — it's a project preference.
- The `dispatch-worker` agent-def YAML should document the model override
  surface but not hardcode a default that contradicts project preference.

### 4. Harness-specific content in AGENTS.md / CLAUDE.md ✅ DONE (commit 227c3b0)

`CLAUDE.md` is no longer a symlink. It includes `@AGENTS.md` for shared
conventions plus a Claude-specific reviewer section. `AGENTS.md` is
harness-agnostic. Opted for approach (c): break symlink, `@AGENTS.md` include.

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

### Additional findings from SL-081 notes (document, not necessarily fix)

- **`DOCTRINE_TRUNK_REF`:** Required in bwrap jails where SSH push is disabled.
  The trunk ladder (`origin/HEAD` → `main`) breaks when remote is stale.
  Document the requirement; a ladder priority fix is out of scope (ADR-012
  territory).
- **Worker marker vs baseline-verify:** Worker marker causes 3 e2e tests to
  refuse (marker-mediated write refusal). `just check` cannot be the
  baseline-verify for worker forks without excluding these tests. Document the
  known limitation; a `just check --skip-e2e` or marker-aware test skip is a
  follow-up.
- **Cross-phase compile ripple:** File-disjoint batching is valid for editing
  but type changes in one phase can ripple into other phases' files (observed:
  PHASE-02 `CatalogKey` change broke `routes.rs` and `relation_graph.rs`).
  Document the risk; orchestrator compile-fix followup is a known cost of
  batching.

### Affected files

- `plugins/dispatch/SKILL.md` — router harness detection table (3-way + refuse catch-all)
- `plugins/dispatch-subprocess/SKILL.md` — harness→spawn-template table (pi `subagent` row, codex `codex exec` row)
- `plugins/dispatch-agent/SKILL.md` — no material change (Claude arm unchanged)
- ~~`AGENTS.md`~~ — already harness-agnostic (done)
- ~~`CLAUDE.md`~~ — already `@AGENTS.md` + Claude section (done)
- `.pi/agents/dispatch-worker.md` — **NEW** (pi dispatch worker agent definition with direct `model:` field)
- `.claude/agents/dispatch-worker.md` — no change

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

- **ASM-1:** pi's `subagent` tool's `cwd` parameter correctly binds the worker
  to the fork directory. Verified in SL-081: `subagent(agent="worker", cwd="$D")`
  produced correct source edits in the fork.
- **ASM-2:** ✅ RESOLVED — Model from direct `model:` field in agent-def YAML.
  This slice does not introduce runtime `model=` override or .toml templating.
- **ASM-3:** codex env marker unresearched; detection by explicit self-belief
  ("I am codex") with refuse fallback for unknown harnesses.
- **RSK-1:** ~~AGENTS.md separation may need a `doctrine boot` update~~ — resolved.
- **RSK-2:** ✅ RESOLVED — Model from direct `model:` field in agent-def YAML.
  No runtime override or .toml surface in this slice.
- **RSK-3:** ✅ RESOLVED — Detect `subagent` tool in tool list; refuse with
  install message if missing. No `pi -p` fallback (loses agent-def contract).
- **RSK-4 (new):** pi subagent workers lack bwrap confinement (codex arm has
  OS-floor bwrap profile). Acceptable — same posture as Claude arm; jail bounds
  all harnesses; marker is primary identity; R-5 belt is real protection.
- **RSK-5 (new):** Fork branch IS the phase ref on codex/pi arm — `gc --force`
  before `dispatch sync --prepare-review` destroys the deliverable irrecoverably.
  Documented as harness-table residual.

## Open Questions

- **OQ-1:** ✅ RESOLVED — AGENTS.md/CLAUDE.md separated in commit 227c3b0.
- **OQ-2:** ✅ RESOLVED — Codex detection by explicit self-belief check ("I am
  codex") with refuse catch-all for unknown harnesses. Env marker research
  deferred to codex spike.
- **OQ-3:** ✅ RESOLVED — Model from direct `model:` field in agent-def YAML.
  No runtime override or .toml surface in this slice.
- **OQ-4:** ✅ RESOLVED — Harness table within `/dispatch-subprocess` (Option C).
  Pi spawn via `subagent()` tool; codex via `codex exec`. One arm, two rows.
- **OQ-5:** `DOCTRINE_TRUNK_REF` — documented as a bwrap-jail requirement.
  Ladder fix is ADR-012 territory, not this slice.
- **OQ-6:** ✅ RESOLVED — Detect `subagent` tool in tool list; refuse with
  install message if missing. No `pi -p` fallback.

## Verification intent

- **VA:** Dispatch router correctly detects pi, codex, and claude (cross-check:
  self-belief vs env markers). Mismatch/unknown refuses, naming the cause.
- **VA:** pi `subagent` spawn template (`agent: dispatch-worker`, `cwd: $D`,
  `task: <prompt>`) is documented in the skill with correct parameter names.
- **VT:** pi worker launched via `subagent` arrives in the correct worktree cwd
  with disk marker present; prompt contains DOCTRINE_WORKER prefix instruction
  (confirmed in SL-081).
- **VA:** pi `dispatch-worker.md` agent definition carries the worker contract
  (source-only, single commit, verify, no `.doctrine/` writes).
- **VA:** AGENTS.md is harness-agnostic; Claude-specific content is only visible
  to Claude agents.
- **VT:** Dispatch skills build/install cleanly (no broken refs, install copies
  match source).

## Summary

The dispatch routing is structurally correct but the implementation has three
gaps confirmed by SL-081 empirical data: (1) pi's spawn template is wrong
(`codex exec` blocks git writes in bwrap; correct mechanism is `subagent` tool),
(2) pi needs a `dispatch-worker` agent definition matching the claude one, and
(3) model selection needs a surface in the skills. AGENTS.md leakage is already
fixed (CLAUDE.md now includes @AGENTS.md + Claude section).

## Follow-Ups

- codex dispatch end-to-end spike (separate slice / backlog item)
- codex dispatch-worker agent definition (requires codex subagent characterization)
- Project-level model selection policy (a project preference, not a slice)
