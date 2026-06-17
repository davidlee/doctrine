# SL-084 Design: Dispatch harness routing decomposition

## Current vs target behavior

**Current:** The dispatch router lumps pi and codex under a single `codex exec`
spawn template in `/dispatch-subprocess`. No harness table — just a binary
codex/pi vs claude split. No model selection surface. No pi dispatch-worker agent
definition. `CLAUDE.md` was a symlink to `AGENTS.md`, leaking Claude-specific
instructions into pi agents (fixed in scope, commit 227c3b0). SL-081 empirical
data confirmed `codex exec --cd "$D" -s workspace-write` cannot commit git in
bwrap jails — workers produce source deltas but `.git/` writes are blocked.

**Target:** Three-harness dispatch with per-harness spawn templates in a single
`/dispatch-subprocess` arm using a harness→spawn table. Pi workers invoked via
`subagent` tool (not `codex exec`). Codex retains `codex exec` as a legacy
placeholder with documented "untested" caveat. Claude unchanged (Agent tool arm).
Model selection via direct `model:` field in agent-def YAML frontmatter —
project-local, no installer templating in this slice. Pi dispatch-worker agent
definition created. Harness detection extended to three-way with self-belief +
env cross-check.

---

## D1 — Harness architecture: table within `/dispatch-subprocess`

**Decision: Option C.**

The two current arms are `/dispatch-subprocess` (codex/pi) and `/dispatch-agent`
(claude). Pi is a hybrid: subprocess mechanism (cwd binding, marker,
DOCTRINE_WORKER) but invoked via the `subagent` tool rather than a raw shell
command. A third arm (`/dispatch-pi`) would duplicate cwd/marker/env
infrastructure; routing pi to `/dispatch-agent` would conflate two different
spawn models (Agent tool vs subagent tool).

Instead, `/dispatch-subprocess` gains a **harness→spawn-template table**. Each
row: harness name, spawn mechanism, identity notes, residuals. The shared
infrastructure (fork, marker, cwd, env contract, DOCTRINE_WORKER) stays once.

```markdown
| Harness | Spawn mechanism | Identity | Notes |
|---|---|---|---|
| pi       | `subagent(agent="dispatch-worker", task="<prompt>", cwd="$D")` | Marker (primary) + prompt self-arm (prefix `DOCTRINE_WORKER=1` per command) | Requires `pi-subagents` extension; detected via `PI_HOME` env + self-belief. **Residual:** fork branch IS the phase ref — gc only after `dispatch sync --prepare-review`. |
| codex    | `env -C "$D" DOCTRINE_WORKER=1 $fork_env codex exec "<prompt>"` | Marker (primary) + env `DOCTRINE_WORKER=1` (orchestrator-set) | Legacy placeholder — untested end-to-end; detected by explicit self-belief ("I am codex"), no env marker known. Env-marker validation deferred to codex spike. **Residual:** same gc-after-sync ordering constraint as pi. |
```

This is a natural extension of the existing two-spawn-variant pattern
(unconfined `env -C` and confined bwrap).

### Router change

The dispatch router's routing table loses the "codex / pi" lump and gains two
rows:

```markdown
| pi | `/dispatch-subprocess` (pi row) |
| codex | `/dispatch-subprocess` (codex row) |
| claude | `/dispatch-agent` |
```

---

## D2 — Harness detection: self-belief + env cross-check

**Decision: Option B.** Fall back to Option C (`doctrine detect-harness` CLI
verb) if self-belief preamble proves token-expensive in practice.

The router's detection becomes three-way:

| Harness | Env marker | Self-belief cross-check |
|---|---|---|
| pi | `PI_HOME` (set by pi binary at startup) | Agent states "I am pi" |
| claude | `CLAUDECODE` | Agent states "I am claude" |
| codex | Unknown (to be researched) | Agent states "I am codex" |

- Mismatch → refuse, naming the cause ("self-belief=pi but no `PI_HOME` in env")
- Detection order: pi (`PI_HOME` + self-belief) → claude (`CLAUDECODE` + self-belief) → codex (self-belief="codex", no env marker known) → refuse (unknown harness). Elimination alone ("not claude, not pi → codex") is fragile: a fourth harness would silently route to codex. Explicit three-branch check with a refuse catch-all.
- Codex detection is provisional — a codex spike should characterize the env marker.

The pre-distilled worker prompt carries a harness-identifier field so workers
can confirm their harness without self-detection logic.

---

## D3 — pi-subagents extension dependency

**Decision: Option A — detect + refuse.**

The `subagent` tool requires `pi-subagents` extension. At dispatch time, the
router checks if the `subagent` tool is available by inspecting its own tool
list (the LLM's available function/tool surface — a self-check, not a `bash`
command). If `subagent` is absent:

> "pi dispatch requires the `pi-subagents` extension. Install: `pi install pi-subagents`"

No silent fallback. A `pi -p` fallback (Option B) loses the agent-def contract
(source-only discipline, structured report, no `.doctrine/` writes) — that
contract is load-bearing. Installation is handled elsewhere (a future unified
`doctrine install` verb will provision all harness-specific artifacts; backlog
item to follow).

---

## D4 — Model selection surface

**Decision: Option B — agent-def default only for this slice.**

Model selection is project preference, not framework internals. This slice
documents the model override surface in the skills and places a direct `model:`
field in `.pi/agents/dispatch-worker.md` as the project-local default. The
orchestrator spawns with no `model=` parameter — the agent-def carries the model.

**Deferred:** `.doctrine/agent-models.toml` config surface and installer
templating. These are future work (unified `doctrine install` backlog item).
This slice cannot implement or verify the templating flow.

**Deferred:** Per-spawn runtime `model=` override on `subagent()`. The
orchestrator always uses the agent-def model for now. Runtime override is a
future enhancement.

---

## D5 — pi dispatch-worker agent definition

**Decision: Create `.pi/agents/dispatch-worker.md`.**

Direct `model:` field in frontmatter; project-local default.

```yaml
---
name: dispatch-worker
description: Doctrine dispatch worker — executes ONE slice phase in an
  isolated git worktree and hands back a single source-delta commit.
  Spawned by the /dispatch orchestrator; never touches .doctrine/
  authored state, runtime state, or memory.
tools: read, edit, write, bash
---

You are a **doctrine dispatch worker**. The orchestrator (the `/dispatch` funnel)
spawns you into an isolated git worktree to execute exactly ONE slice phase, then
return a source delta — you are a constrained writer, not the orchestrator.

Your contract:

- **Mutate SOURCE only.** Edit tracked/untracked source files in the worktree.
  Do NOT write `.doctrine/` authored trees, runtime state, or memory — those
  are the orchestrator's, and an import touching them is rejected.
- **Stay inside your declared file set.** Straying breaks the file-disjoint batch.
- **Verify before you commit.** Run the orchestrator-supplied verify command;
  a red verify is reported back, never committed.
- **Commit exactly ONE non-merge commit** descended from the supplied base —
  the importable delta unit. No multi-commit history, no merge, no rebase.
- **Hand back a structured report** (what changed, verify result, notes), not
  a doctrine artifact.
- **DOCTRINE_WORKER self-arm:** For any command that needs worker-mode
  behavior, prefix with `DOCTRINE_WORKER=1` (e.g., `DOCTRINE_WORKER=1 cargo build`).
  Do NOT assume persistent shell env — bash invocations may run in separate
  shells. The disk marker (stamped by the orchestrator pre-spawn) is your
  primary identity; DOCTRINE_WORKER is a fail-open optimisation. The real
  protection is the orchestrator's import-time R-5 belt — never rely on
  self-arm alone.
```

### Tool set rationale

Pi tools: `read, edit, write, bash`. No standalone `grep`/`glob` — `bash`
covers `grep`, `find`, `ls`. This is the minimal set for source editing +
verification in a worktree. No `web_search`, `subagent` — the worker operates
in a local worktree with pre-distilled context.

### Relationship to existing `.pi/agents/*.md`

The existing agents (`worker.md`, `scout.md`, `planner.md`, etc.) are
project-local conventions set up by a previous pi agent, not in the installer.
`dispatch-worker.md` follows the same pattern — a project-local agent definition
committed to the repo.

---

## D6 — Spawn template for pi

**Decision:** The pi spawn "template" in `/dispatch-subprocess` describes the
`subagent` tool invocation:

```
subagent(
  agent: "dispatch-worker",
  task: "<pre-distilled prompt>",
  cwd: "$D"
)
```

### cwd binding

The `subagent` tool's `cwd` parameter sets the spawned `pi -p` process cwd to
the fork directory `$D`. Confirmed working in SL-081 — the worker's source
edits land in the worktree, not the coordination root.

### DOCTRINE_WORKER identity

Consistent with SL-056's keystone design: "identity rides *disk* — which every
harness has — not a process env seam." The orchestrator stamps the disk marker
via `fork --worker` before spawn. The `subagent` tool has no env parameter, so
the worker self-arms via prompt (prefix `DOCTRINE_WORKER=1` per command — same
approach as the Claude arm, which also relies on prompt-level self-arm since
the Agent tool has no env seam). This is a *codex/pi optimisation of the marker*, not the identity itself
(SL-056 Charge XIII). The import-time R-5 belt is the real protection; self-arm
fails open.

The pi arm loses the env-arm optimisation that codex has (`env -C "$D"
DOCTRINE_WORKER=1 ...`) — the orchestrator can set it externally for codex, but
not for pi. This is acceptable; the Claude arm has the same constraint (Agent
tool, no env seam) and is proven in operation.

### Pre-distilled prompt (unchanged)

The orchestrator pre-distills the worker prompt per the existing dispatch router
contract: policy digest, design excerpts, pre-fetched memories, task spec +
declared file set, mandatory verify command, self-arm mandate, escalation
contract. Workers never read boot/governance.

### Cross-references to prior dispatch slices

- **SL-056** (orchestrator spawn seam): Established disk-marker identity as
  harness-agnostic primary; DOCTRINE_WORKER as codex/pi optimisation. Pi
  subagent spawn inherits this design directly — marker primary, env secondary,
  fail-open.
- **SL-064** (coordination-branch isolation): Pi workers fork from the
  coordination branch (`dispatch/<slice>`), same as all arms. No change.
- **SL-068** (dispatch candidates for safe audit): Pi workers produce native
  fork branches (`phase/<slice>-NN`), same as codex. The candidate interaction
  branch workflow is identical.

---

## Code impact

### Affected files

| Path | Change |
|---|---|
| `plugins/doctrine/skills/dispatch/SKILL.md` | Router harness detection table → three-way (pi / codex / claude). Self-belief + env cross-check per D2. |
| `plugins/doctrine/skills/dispatch-subprocess/SKILL.md` | Add harness→spawn-template table (pi subagent row, codex `codex exec` row). Update spawn prose. |
| `plugins/doctrine/skills/dispatch-agent/SKILL.md` | No material change (Claude arm unchanged). Optionally document model selection surface. |
| `.pi/agents/dispatch-worker.md` | **New.** Pi dispatch-worker agent definition with direct `model:` field. |
| `install/routing-process.md` | No change (dispatch routing is internal to the dispatch skill, not the route skill table). |

### Non-affected files

- `AGENTS.md` / `CLAUDE.md` — already separated (scope item 4, commit 227c3b0)
- `src/worktree.rs` — worker marker, fork provisioning unchanged
- `.claude/agents/dispatch-worker.md` — unchanged
- `install/` installer source — future work (unified `doctrine install`)
- `.doctrine/agent-models.toml` — deferred to future installer work

---

## Verification

| Criterion | Mode | Description |
|---|---|---|
| VA | Agent | Router detects pi (`PI_HOME` + self-belief), claude (`CLAUDECODE` + self-belief), codex (self-belief="codex"); unknown harness refuses |
| VA | Agent | pi spawn section in `/dispatch-subprocess` describes `subagent(agent="dispatch-worker", task="...", cwd="$D")` with correct parameter names |
| VT | Test | pi worker launched via `subagent` with `cwd="$D"` arrives in correct worktree directory with disk marker present; prompt contains DOCTRINE_WORKER prefix instruction (confirmed in SL-081) |
| VA | Agent | pi `dispatch-worker.md` agent definition carries worker contract (source-only, single commit, verify, no `.doctrine/` writes) |
| VA | Agent | Harness→spawn table in `/dispatch-subprocess` has pi row (subagent) and codex row (`codex exec`) with distinct identity notes |
| VA | Agent | Router prose instructs checking tool list for `subagent` availability before pi spawn; missing → refuse with extension install message |
| VT | Test | Skills install cleanly; no broken cross-references between dispatch skills |

---

## Remaining open questions

### OQ-2 (codex env marker) — deferred to codex spike

Codex detection uses explicit self-belief only ("I am codex") with no known env
marker. Unknown/non-pi/non-claude does NOT imply codex — it refuses. Env-marker
characterization is deferred to the codex spike.

### OQ-5 (DOCTRINE_TRUNK_REF ladder) — documented, not fixed

`DOCTRINE_TRUNK_REF=main` is required in bwrap jails where SSH push is disabled.
The trunk ladder (`origin/HEAD` → `main`) breaks when remote is stale. ADR-012
territory — not this slice. Documented in scope notes.

---

## Known limitations

### Codex arm untested (deferred to codex spike)

The codex `codex exec` spawn template is preserved from the current skill with
a documented "untested end-to-end" caveat. The codex subagent system, env marker,
and spawn mechanics are not characterized. The codex row in the harness table
is a placeholder until a separate codex spike validates it.

### Pi subagent has no bwrap confinement

The codex arm has a confined bwrap profile (OS-floor worker isolation — ro-binds
the marker, restricts filesystem access). Pi `subagent` workers inherit the
orchestrator's filesystem permissions — no additional confinement layer. This is
the same posture as the Claude arm (Agent tool has full fs access within the
jail). Acceptable because:
- The jail itself (bwrap) bounds all harnesses
- The marker is the identity; the R-5 belt is the real protection
- Future bwrap confinement for pi subagents would require a `subagent` sandbox
  parameter or a pi-level confinement primitive — neither exists today

### Model selection surface deferred

The `.doctrine/agent-models.toml` config surface and installer-templating
flow are deferred to future `doctrine install` work. This slice places a
direct `model:` field in `.pi/agents/dispatch-worker.md` as the project-local
default. Runtime `model=` override on spawn is also deferred.

## Non-goals (unchanged from scope)

- codex end-to-end dispatch testing (separate spike)
- ADR amendments
- Model as a framework primitive
- pi subagent model selection mechanics (pi internals)
- Unified `doctrine install` verb (backlog item)
- `doctrine detect-harness` CLI verb (punted unless D2 Option B proves expensive)
