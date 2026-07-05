# Ambient memory surfacing via harness hooks

## Context

Doctrine memory retrieval is **pull-only**: `memory retrieve` surfaces nothing
unless an agent explicitly invokes it with a scope probe (`--path-scope`,
`--glob`, `--command`, `--tag`). In practice agents forget to pull before
touching an unfamiliar surface — the exact failure the memory corpus exists to
prevent. Durable footgun memories (e.g. "Dispatch integrate --edge advance leg
is not FF-gated") sit unread at the moment they would have mattered.

The Claude Code harness exposes a `PreToolUse` hook that can inject
`additionalContext` (a system-reminder, "data not instruction") alongside a tool
call. This lets the corpus **push** relevant memories into context at the moment
of action — keyed on the path being read/edited or the command about to run —
without the agent choosing to pull.

This slice was scoped from **IDE-032**. A working two-hook prototype already
runs in this repo (see Design Inputs); the slice turns the validated prototype
into a decision about what — if anything — doctrine should ship.

## Scope & Objectives

- Decide **whether and how** doctrine surfaces scoped memories ambiently at
  tool-call time, and **where that machinery lives** given it is harness-specific.
- Two proven surfaces:
  - **Path surface** — `PreToolUse(Read|Edit|Write)` → `retrieve --path-scope
    <file> --glob <file>`. Relevance comes from *path specificity* (leaf files
    hit sharp; generic dirs hit fuzzy).
  - **Command surface** — `PreToolUse(Bash)` → `retrieve --command <cmd>`,
    admitted by **memory metadata** (`severity ≥ high`), not a host-command
    allowlist. Relevance comes from the *severity gate*; footgun-grade only.
- Cross-cutting behaviour proven in the prototype and in scope to formalise:
  session dedup (shared seen-set), per-fire cap, `⚠stale` caveat flag, injected
  payload trimmed to `[triage-field] title — uid`.

## Non-Goals

- **Not** the read-before-write / scaffold-template token footgun — that is
  **IDE-033** (parked, MCP-authoring direction). Sibling, separate slice.
- **Not** changing the retrieval engine. `retrieve` and its scope probes already
  exist and are the sole seam — this slice is glue + placement, no new query
  code (DRY; ride the existing seam).
- **Not** blocking or gating tool calls. The surface is purely advisory; it must
  never deny a call.
- **Not** a `--tag` or `CwdChanged` surface in v1 (candidate follow-ups, see
  Follow-Ups).

## Governing constraints (design must honour)

- **POL-002 (platform independence).** The command allowlist prototype hard-codes
  host tool names (`git`/`cargo`/`just`) — a load-bearing dependency on host
  convention, **prohibited**. The shipped admission rule must rest on a
  doctrine-owned contract (memory `severity`/`scope` metadata), which it now does.
  Likewise, tuning `doctrine new` stdout to shout "read first" (the IDE-033
  reject) leaked harness behaviour into the neutral CLI — same violation class.
- **Harness-agnosticism (ADR-011 precedent).** A `PreToolUse` hook is
  Claude-Code-specific. Doctrine's neutral core must not bake it in; it belongs
  as an **opt-in harness supplement** (install template / band / `memory sync
  install`-style wiring), never as core enforcement. *Where it ships is the
  central design decision.*
- **Hooks fail OPEN** (`mem_019f1a5cd5937cf3a0824f52e3ff4724`): only `exit 2`
  blocks; any other failure lets the call proceed. Good for an advisory surface —
  a `retrieve` error simply drops the injection, no harm. Design must preserve
  fail-open (never `exit 2`).
- **Trust holdback** already suppresses low-trust/high-severity memories inside
  `retrieve`; the command surface's severity gate composes cleanly with it
  (surfaces high-trust/high-severity, holdback hides low-trust/scary).

## Affected surface (coarse; /design refines)

- `install/` — likely home for a shipped, opt-in hook template + wiring docs.
- `.claude/hooks/` prototype scripts (gitignored here) — source of the template.
- Possibly the MCP surface / `memory sync install` if wiring is doctrine-driven.
- No change expected to `src/` retrieval code.

## Design Inputs (prototype + findings)

Working prototype committed at `.doctrine/slice/205/prototype/`:
- `memory-probe.sh` — path surface, `PreToolUse(Read|Edit|Write)`.
- `command-probe.sh` — command surface, `PreToolUse(Bash)`, severity-gated.

Empirical findings from the probe session:
- Path scope: leaf-file hits sharp (`src/backlog.rs` → 3 dead-on footguns);
  generic paths (`.claude/settings.local.json`, a bare slice `.md`) → fuzzy
  orientation hits. Precision tracks path specificity.
- Command scope: allowlist version worked but violated POL-002; severity-metadata
  admission replaced it — `doctrine dispatch integrate --edge` surfaced the exact
  FF-gate footgun, `git commit`'s weak none-severity hit correctly suppressed,
  `ls` silent.
- Payload: dropping `type` + `trust` (command) + echoed path/cmd trimmed ~30-40%
  per line; adding `⚠stale` (from `staleness`) is the highest-value field.
- `retrieve` accepts a **uid prefix** in `show` (kept full uid anyway —
  ambiguous-match risk > modest token save).
- `additionalContext` cap is 10k chars; hot-reloads without session restart;
  injected context does **not** satisfy the harness read-bit (relevant to
  IDE-033, not here).

## Risks / Assumptions / Open Questions

- **OQ-1** Where does the shipped hook live and how is it wired? (install
  template the user opts into? `memory sync install` extension? band supplement?)
- **OQ-2** Dedup is session-scoped via a runtime file. Ship as-is, or is
  cross-session suppression wanted? Runtime-tier placement (gitignored).
- **OQ-3** Admission tuning: is `severity ≥ high` the right floor, or include
  `major`/`medium`? Should the path surface also gate (probably not — its
  none-severity leaf hits are useful)?
- **OQ-4** One hook script parameterised by surface, or two? (prototype is two.)
- **OQ-5** Per-fire cap and total-per-session budget — what bounds context spend?
- **R-1** Subprocess cost: command surface spawns `retrieve` on *every* Bash.
  Cheap (silent when no hit) but non-zero; measure.
- **A-1** Assumes the Claude harness `PreToolUse`/`additionalContext` contract is
  stable enough to depend on for an opt-in supplement (not core).

## Verification / Closure intent

- Shipped artifact (template + wiring) surfaces correct scoped memories on a
  clean install, opt-in, without touching neutral core behaviour.
- POL-002 and harness-agnosticism conformance argued explicitly.
- Fail-open preserved (never `exit 2`); advisory-only.
- Prototype findings reconciled: what shipped, what was cut, what deferred.

## Follow-Ups

- **Harness ports.** This slice targets the Claude Code hook contract. Sibling
  ports for other harnesses are follow-up slices, each riding the same
  doctrine-owned admission contract but its own harness seam:
  - **pi** — surface via pi's pre-tool / equivalent hook mechanism.
  - **codex** — surface via codex's equivalent seam.
  The neutral core (the `retrieve` scope probes + severity/staleness metadata)
  is shared; only the harness glue differs per port.
- **Candidate POL-003 — harness-agnostic supplement placement.** Promote the
  ADR-011-precedent principle used here (harness-specific behaviour ships as an
  opt-in supplement resting on doctrine-owned contracts, never baked into neutral
  core) into an explicit **policy**, so future harness integrations inherit the
  rule rather than re-deriving it. Captured as a backlog idea; decide altitude
  (POL vs ADR) when authored.
- `--tag` surface and `CwdChanged` (coarse dir-entry) surface — deferred.
- IDE-033 (gate-free MCP authoring) — sibling, separate slice.
