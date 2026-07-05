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

Ship ambient memory surfacing as a **binary subcommand + thin `hooks.json`
glue**, riding the established `doctrine worktree pretooluse` seam (a
Claude-`PreToolUse`-shaped command living in the binary under the ADR-011
per-harness altitude). The prototype's bash/jq logic becomes Rust; the neutral
`retrieve` query is called **in-process**, not re-exec'd.

- **New per-harness adapter** — one binary subcommand (working name `doctrine
  memory surface`): reads the `PreToolUse` hook envelope on stdin, discriminates
  the surface on `tool_name`, calls the retrieve query in-process, emits
  `additionalContext`, `exit 0` always (advisory — never `permissionDecision`).
- Two surfaces, one command (discriminated on `tool_name`, exactly as
  `run_pretooluse` discriminates Bash vs Edit|Write):
  - **Path surface** (`Read|Edit|Write`) → retrieve `path_scope`/`glob` on the
    file. Relevance from *path specificity* (leaf sharp, generic fuzzy).
    **Ungated** — a leaf file's none-severity hits are the useful ones.
  - **Command surface** (`Bash`) → retrieve `command`, admitted by **memory
    metadata** (`severity ≥ high`), not a host-command allowlist. Footgun-grade.
- **Audience: main-thread only** (`agent_id.is_none()`) — the inverse of the
  jail's subagent-only gate. Subagents get their footguns via their spawn prompt.
- Cross-cutting behaviour to formalise: per-`session_id` dedup (seen-set file
  under `.doctrine/state/`, auto-fresh per session), per-fire cap, `⚠stale`
  caveat flag, injected payload trimmed to `[triage-field] title — uid`.
- Tuning (caps, severity floor) ships as **hardcoded named constants** (STD-001),
  config knobs deferred.
- **Tuning log** — a runtime-tier log (`.doctrine/state/`, gitignored) records
  each fire: surface, scope key, fetched vs admitted vs suppressed (severity),
  surfaced uids. The data source for retuning the floor/caps over real usage.

## Non-Goals

- **Not** the read-before-write / scaffold-template token footgun — that is
  **IDE-033** (parked, MCP-authoring direction). Sibling, separate slice.
- **Not** changing the retrieval engine. `retrieve`'s query already exists and is
  the sole seam — this slice adds a hook-envelope adapter that calls it, no new
  query code (DRY; ride the existing seam).
- **Not** a shipped shell script. The prototype scripts stay as committed design
  reference; the ship vehicle is the binary subcommand (jail precedent).
- **Not** blocking or gating tool calls. Advisory only — emits `additionalContext`,
  never `permissionDecision`; `exit 0` always.
- **Not** subagent surfacing in v1 (main-thread only; follow-up).
- **Not** config-driven tuning in v1 (hardcoded constants; follow-up).
- **Not** a `--tag` or `CwdChanged` surface in v1 (candidate follow-ups).

## Governing constraints (design must honour)

- **Jail precedent (`doctrine worktree pretooluse`).** A Claude-`PreToolUse`-shaped
  command already lives in the binary: pure `decide`/`render`, impure shell, `exit
  0` always. Its module doc claims the ADR-011 per-harness altitude explicitly. The
  surfacing adapter is the second instance of the same pattern — this is the
  load-bearing precedent that settles placement.
- **POL-002 (platform independence).** The command allowlist prototype hard-codes
  host tool names (`git`/`cargo`/`just`) — a load-bearing dependency on host
  convention, **prohibited**. The shipped admission rule rests on a doctrine-owned
  contract (memory `severity` metadata); the neutral CLI gains no host-command
  knowledge. Satisfied *structurally*, not by discipline.
- **Harness-agnosticism (ADR-011 altitude).** The `PreToolUse` envelope is
  Claude-specific; the retrieve query is neutral. The split is clean: neutral core
  = the query (untouched); per-harness adapter = the new hook-shaped subcommand,
  the sanctioned altitude the jail already occupies. Wiring is thin `hooks.json`
  glue in the shipped `plugins/doctrine/` plugin.
- **Hooks fail OPEN** (`mem_019f1a5cd5937cf3a0824f52e3ff4724`): only `exit 2`
  blocks; any other failure lets the call proceed. Good for an advisory surface —
  a `retrieve` error simply drops the injection, no harm. Design must preserve
  fail-open (never `exit 2`).
- **Trust holdback** already suppresses low-trust/high-severity memories inside
  `retrieve`; the command surface's severity gate composes cleanly with it
  (surfaces high-trust/high-severity, holdback hides low-trust/scary).

## Affected surface (design-refined)

- `src/memory.rs` — new `MemoryCommand::Surface` (the hook adapter) + pure
  helpers (admission filter, dedup-diff, block-format) and impure shell (stdin,
  in-process retrieve, seen-set + log IO, emit).
- `src/retrieve.rs` — new thin `retrieve_rows` + `SurfaceRow` projection
  (composes the existing `load_query`/`query`/`check_retrievable` path); expose
  `severity_rank` `pub(crate)`. Query logic itself unchanged.
- `src/commands/guard.rs` — classify `MemoryCommand::Surface` as `Read` (the
  `Command::Memory` match is exhaustive — omission fails to compile).
- `plugins/doctrine/hooks/hooks.json` — new `PreToolUse` matcher entries
  (`Read|Edit|Write` + `Bash`) → `${DOCTRINE_BIN} memory surface`. Separate from
  the jail entries (opposite audience — jail skips main thread, surface fires for
  it).
- Behaviour-preservation gate: existing memory + retrieve suites stay green
  unchanged.

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

## Open Questions (resolved in /design) / Risks / Assumptions

- **OQ-1 → RESOLVED.** Ships as a binary subcommand + `hooks.json` glue (jail
  precedent), not an install template / band / shell script.
- **OQ-2 → RESOLVED.** Dedup session-scoped, keyed by the stdin `session_id`
  (auto-fresh per session); seen-set file under `.doctrine/state/` (gitignored).
- **OQ-3 → RESOLVED.** Asymmetric: command surface `severity ≥ high`, path
  surface ungated (leaf none-severity hits are the useful ones). Hardcoded consts.
- **OQ-4 → RESOLVED.** One command, discriminated on `tool_name` (jail precedent).
- **OQ-5 → RESOLVED (values pending real-usage tuning).** Per-fire cap hardcoded
  (path 3, cmd 2); a session budget is a candidate follow-up; the tuning log feeds
  retuning.
- **R-1** Hook-process spawn on every main-thread tool call. Retrieve is now
  in-process (no re-exec); cost is one binary spawn — the same the jail already
  pays. Measure; silent-when-no-hit keeps it cheap.
- **A-1** Assumes the Claude `PreToolUse`/`additionalContext` contract is stable
  enough to depend on for a per-harness adapter (same assumption the jail makes).

## Verification / Closure intent

- Pure helpers (admission filter, dedup-diff, block-format) unit-tested with
  synthetic rows (VT) — the jail's `decide`/`render` test shape.
- Hook-shaped command tested with synthetic stdin → emitted `additionalContext`
  JSON (main-thread fires; subagent `agent_id` present emits nothing; retrieve
  error emits nothing; `exit 0` always).
- POL-002 + harness-agnosticism argued: retrieve query untouched; adapter at the
  ADR-011 altitude; admission on doctrine-owned metadata.
- Behaviour-preservation: existing memory suites green unchanged.
- Prototype findings reconciled: what shipped, what was cut, what deferred.

## Follow-Ups

- **Harness ports.** This slice targets the Claude Code hook contract. Sibling
  ports for other harnesses are follow-up slices, each riding the same
  doctrine-owned admission contract but its own harness seam:
  - **pi** — surface via pi's pre-tool / equivalent hook mechanism.
  - **codex** — surface via codex's equivalent seam.
  The neutral core (the `retrieve` scope probes + severity/staleness metadata)
  is shared; only the harness glue differs per port.
- **Subagent surfacing.** v1 is main-thread only; widening to subagents (with a
  tighter cap / higher floor to respect their budgets) is a deferrable knob.
- **Config knobs.** v1 tuning is hardcoded constants; promote caps/floor to
  `.doctrine/config` once the tuning log yields real-usage data.
- **Session budget.** A total-per-session surfaced cap (beyond per-fire caps)
  if the log shows context spend creeping.
- **Candidate POL-003 — harness-agnostic supplement placement.** Promote the
  ADR-011-precedent principle used here (harness-specific behaviour ships as an
  opt-in supplement resting on doctrine-owned contracts, never baked into neutral
  core) into an explicit **policy**, so future harness integrations inherit the
  rule rather than re-deriving it. Captured as a backlog idea; decide altitude
  (POL vs ADR) when authored.
- `--tag` surface and `CwdChanged` (coarse dir-entry) surface — deferred.
- IDE-033 (gate-free MCP authoring) — sibling, separate slice.
