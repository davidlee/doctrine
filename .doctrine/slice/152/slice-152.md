# Claude-arm WorktreeCreate worker creation

## Context

The claude `/dispatch` arm spawns workers via the `Agent` tool with
`isolation: worktree`. Today the **harness** creates that worktree, which is the
root of hazard **H1** (RFC-005): under shared-clone git-lock contention the
spawn silently falls back to the main worktree, where `baseRef:"head"` tracks a
**moving `main`** — the worker runs on a wrong/dirty/moving base instead of the
coordination tip B (ISS-034). SL-123 hardened a **post-run** `verify-worker`
belt (loud, pre-import halt) but the race is unchanged: throughput under churn is
zero, and the wasted-worker-run residual remains.

A `WorktreeCreate` hook **fully replaces** native creation. Proven empirically on
claude-code **2.1.181** (wtc-probe, 2026-06-25,
[[mem.pattern.dispatch.worktreecreate-replace-base-control]]): a named
`dispatch-worker` spawned with the hook active landed at a **doctrine-chosen
base** (`68250bcd`), overriding `baseRef:"head"`, in doctrine's own worktree
path. When doctrine is the creator there is **no native creation to fall back
to** — the H1 fallback cannot occur, and a hook failure aborts the spawn
fail-closed (the only hook event where any non-zero exit blocks). This converts
H1 from a permanent harness tax into a **fixable mechanism defect**, and lets the
claude arm create workers through the **same `doctrine worktree fork --worker`**
path the subprocess arm already uses (converging H3) while provisioning
gitignored build artefacts inside the hook (addressing H4 for the claude arm).

Origin: **IMP-072** (re-scoped — its "base control solved by placement" premise
was falsified by contention + the probe). Governing canon: **ADR-006**
(orchestrator-sole-writer worktree posture; D9 gitignored-allowlist
provisioning), **ADR-011** (harness-agnostic spawn interface, per-harness
capability altitude), **ADR-012** (dispatch integration topology). Supersedes the
placement-only decision in **SL-064** design §8 (option Y).

## Scope & Objectives

### Primary — WorktreeCreate hook (collapse-the-arms hypothesis)

**Hypothesis under test:** with doctrine as the worktree creator on the claude
arm, the claude and subprocess `/dispatch` arms collapse onto a **single**
`doctrine worktree fork --worker` path — eliminating **H1** (no native creation
left to fall back to) and converging **H3** (one creation seam, both harnesses).
This slice builds the hook to *test* that, not to assume it.

- A repo-global `WorktreeCreate` hook (claude harness) that **replaces** native
  worktree creation for `isolation:worktree` spawns and is **installed by
  `doctrine install`** (not hand-wired).
- **Out-of-band discrimination.** The matcher does **not** scope by `agent_type`
  (probed: the hook fires for `general-purpose` too; payload carries no
  `agent_type`). The orchestrator (sole writer, serial dispatch → race-free)
  drops a marker carrying the intended **base B** immediately before the worker
  spawn; the hook consumes it and forks at B.
- **Marker present → dispatch worker:** `doctrine worktree fork --worker` at base
  B, **fail-closed**, folding in ADR-006 D9 provisioning + the worker-marker
  stamp as one trusted act (mirrors the subprocess arm).
- **Marker absent → benign isolated subagent:** pass through — replicate default
  creation (`git worktree add <path> HEAD`) so non-dispatch isolated subagents
  still work. No silent base hijack of unrelated subagents.
- Wire the claude `/dispatch-agent` flow to drop the marker before spawn and to
  rely on hook-created placement instead of the cwd-placement hack.

### Secondary — idiomatic plugin packaging (partial progress)

Today doctrine wires its claude integration bespoke: a `WorktreeCreate` block in
`settings.local.json` and an `.mcp.json` in the project root. The idiomatic
Claude Code surface is a **plugin** — a directory with
`.claude-plugin/plugin.json` plus `hooks/hooks.json`, `.mcp.json`, and `agents/`
(ground truth: `code.claude.com/docs/en/plugins-reference.md`, fetched raw and
pinned to claude-code **2.1.181**).

- Take the **first step** toward that idiom by delivering the WorktreeCreate hook
  **inside a doctrine plugin's `hooks/hooks.json`** rather than a raw settings
  block — landing the plugin scaffold (`.claude-plugin/plugin.json`) the rest can
  later move onto.
- Migrating the MCP server (`.mcp.json`) and subagents (`agents/`) into the
  plugin is **deferred** (see Non-Goals) — this is a partial step, not a big-bang
  migration, and it is **droppable** if it threatens the primary (RSK-2).

## Non-Goals

- The subprocess arm (`doctrine worktree fork --worker` direct) — unchanged.
- Removing the SL-123 post-run `verify-worker` belt — it stays as defence in
  depth (this slice makes it rarely-triggered, not redundant).
- The `WorktreeRemove` lifecycle (cleanup) beyond what is needed to not leak the
  hook-created worktree — full removal-hook ownership is a possible follow-up.
- H2 integration hazards (ISS-038 / IMP-122) — separate.
- Non-claude harness altitude changes (ADR-011 is satisfied, not amended).
- **Full plugin migration** — moving the doctrine MCP server (`.mcp.json`) and
  subagents (`agents/`) into the plugin. This slice scaffolds the plugin and
  lands only the hook in it; the rest is follow-up.

## Affected surface (provisional)

> **Design resolved (see `design.md`).** Handshake (OQ-1) → D3: a **cwd-local
> bare-base marker**; branch/dir hook-derived from the payload `name`; no
> consume/serialization/correlation key. The **SubagentStart stamp hook is
> retired** on the claude arm (D2/R1) — `WorktreeCreate → fork --worker` becomes
> the single creation+provision+mark seam. Hook-failure UX (OQ-3) → fail-closed
> non-zero exit (legible by the harness abort). Two new verbs: `dispatch
> arm-spawn`, `worktree create-fork`. Probes P3→P2→P1 gate the locks.

- `doctrine install` hook-emission: emit the hook via an idiomatic plugin
  (`.claude-plugin/plugin.json` + `hooks/hooks.json`) rather than a bespoke
  `settings.local.json` `WorktreeCreate` block — plus the hook script/seam it
  points at.
- `src/worktree.rs` / `fork --worker` path (reuse, not reimplement — the hook
  shells the existing verb).
- The claude `/dispatch-agent` skill (drop marker; drop cwd-placement reliance).
- Marker read/write seam (orchestrator-sole-writer; pure/imperative split — the
  base is an input, not derived in a pure layer).

## Risks / Assumptions / Open Questions

- **OQ-1.** Marker handshake shape: file under runtime state vs env vs a CLI
  call the hook makes back into doctrine. Sole-writer + serial dispatch makes it
  race-free; parallel file-disjoint phases need a per-spawn key (the payload
  `name`? — only field available).
- **OQ-2.** Benign pass-through fidelity: does `git worktree add <path> HEAD`
  (detached) fully satisfy what the harness expects for a non-dispatch isolated
  subagent (branch name, `.worktreeinclude`)? `.worktreeinclude` is **not**
  processed under a hook — the pass-through must replicate it or accept the gap.
- **OQ-3.** Hook failure UX: a fail-closed abort surfaces how to the orchestrator?
  Confirm the abort is legible (vs a generic spawn error).
- **ASM-1.** `worktree.baseRef:"head"` stays the harness default; the hook
  overrides base regardless, so this is belt-not-load-bearing.
- **RSK-1.** Repo-global blast radius (ADR-011 D7 σ): every `isolation:worktree`
  subagent in the repo now routes through doctrine's hook. The benign
  pass-through must be robust or it breaks unrelated subagent use.
- **RSK-2.** Plugin-idiom scope creep: the secondary goal can swallow the slice.
  Guard — the hypothesis test (primary) is achievable without *any* plugin work;
  the plugin step is additive and droppable if it threatens the primary.
- **OQ-4.** Plugin install path: does `doctrine install` write + register a plugin
  directory (marketplace / `--plugin-dir` / skills-dir auto-load), or keep
  emitting settings and only scaffold the plugin layout? Pick the lightest step
  that is genuinely idiomatic, not a half-migration that breaks both.
- **OQ-5.** Hook-in-plugin fidelity: does a `WorktreeCreate` hook declared in a
  plugin's `hooks/hooks.json` fire identically to the same hook in
  `settings.local.json`? The 2.1.181 probe used a settings block — re-probe before
  relying on the plugin form.

## Verification / closure intent

- A dispatch worker spawned through the funnel lands at base B deterministically
  under simulated `main` churn (the H1 scenario) — no fallback-to-main.
- A non-dispatch `isolation:worktree` subagent still gets a working worktree
  (pass-through) and is **not** stamped/forked as a worker.
- Hook failure aborts the spawn fail-closed (no silent fallback).
- `doctrine install` emits the hook; existing dispatch suites stay green
  (behaviour-preservation on the subprocess arm).
- **(Secondary)** the WorktreeCreate hook ships inside a doctrine plugin
  (`hooks/hooks.json`) and fires identically to the settings-block form (OQ-5
  resolved) — partial plugin idiom landed without regressing the hook.

## Follow-Ups

- `WorktreeRemove` hook ownership (cleanup of hook-created worktrees).
- Reassess SL-123 belt scope once this lands (defence-in-depth vs prune).
