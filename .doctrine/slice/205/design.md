# Design SL-205: Ambient memory surfacing via harness hooks

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Doctrine memory retrieval is **pull-only**: nothing surfaces unless an agent
explicitly invokes `retrieve` with a scope probe. Agents forget to pull before
touching an unfamiliar surface — the exact failure the corpus exists to prevent.
Durable footgun memories sit unread at the moment they would have mattered.

Ship an **ambient push**: at tool-call time, key on the path being touched or the
command about to run, surface the relevant scoped memories into context —
advisory, without the agent choosing to pull. The mechanism is a Claude
`PreToolUse` hook; the design question is what doctrine ships so this rests on a
neutral core, not a bag of host-coupled shell.

## 2. Current State

- `retrieve` exposes the query the surfacing needs: `path_scope`/`glob`/`command`
  scope probes, rows carrying `severity`/`staleness`/`trust`/`title`/`uid`, and a
  non-bypassable trust holdback (`check_retrievable`). Reachable in-process via
  `load_query → query → check_retrievable` (`src/retrieve.rs`).
- `doctrine worktree pretooluse` (`src/worktree/pretooluse.rs`) is a
  Claude-`PreToolUse`-shaped command **already living in the binary**: reads the
  hook envelope on stdin, pure `decide`/`render`, impure shell, `exit 0` always
  (deny is *data*, never an exit code). Its module doc claims the ADR-011
  per-harness altitude explicitly. It walls **subagents** and fast-paths the
  orchestrator (`agent_id.is_none() → emit nothing`).
- The shipped Claude plugin `plugins/doctrine/hooks/hooks.json` is **thin glue**:
  every hook is `${DOCTRINE_BIN:-doctrine} <verb>`. It already wires
  `PreToolUse(Bash)` and `PreToolUse(Edit|Write)` → `worktree pretooluse`.
- A working two-hook bash/jq prototype (`.doctrine/slice/205/prototype/`) proved
  the surfaces empirically; it is design reference, **not** the ship vehicle.

## 3. Forces & Constraints

- **Jail precedent.** `worktree pretooluse` is the load-bearing pattern: a
  hook-shaped command in the binary at the ADR-011 altitude. Surfacing is the
  second instance of the same pattern — placement is settled by precedent, not
  reinvented.
- **POL-002 (platform independence).** The prototype's command allowlist
  hard-codes host tool names (`git`/`cargo`/`just`) — prohibited load-bearing on
  host convention. Admission must rest on a **doctrine-owned contract** (memory
  `severity` metadata); the neutral CLI gains no host-command knowledge.
- **ADR-011 (harness-agnostic altitude).** The `PreToolUse` envelope is
  Claude-specific; the retrieve query is neutral. Split cleanly: neutral core =
  the query (untouched); per-harness adapter = the new hook-shaped subcommand.
- **Hooks fail OPEN** (`mem.fact.claude.pretooluse-hook-fail-open`): only `exit 2`
  blocks. Advisory surface must **never** `exit 2`; any failure emits nothing.
- **ADR-001 layering** (leaf ← engine ← command) and **DRY / no parallel impl**:
  ride the existing query seam; the only new query-adjacent code composes it.
- **STD-001** (no magic strings): floor/caps are named constants.
- **Behaviour-preservation gate**: existing memory/retrieve suites stay green
  unchanged.

## 4. Guiding Principles

- **Advisory, never authority.** Emits `additionalContext` only — structurally
  incapable of blocking a call.
- **Neutral core untouched; harness glue thin.** New code is an adapter + one
  rows helper that composes the query, plus `hooks.json` matcher lines.
- **Relevance from the cheapest signal.** Path surface: path-specificity (leaf
  sharp, generic fuzzy) — ungated. Command surface: the `severity` gate —
  footgun-grade only.
- **Opinionated, tunable from data.** Ship hardcoded constants; log the funnel so
  the floor/caps retune from real usage.

## 5. Proposed Design

### 5.1 System Model

Three layers (ADR-001):

| Layer | Home | Responsibility |
|---|---|---|
| Neutral query core | `src/retrieve.rs` | *unchanged*: `load_query → query → check_retrievable` (holdback). |
| Rows helper (new, thin) | `src/retrieve.rs` | `retrieve_rows(root, scope, fetch) -> Result<Vec<SurfaceRow>>` — composes the query path, applies the retrieve holdback, projects to a lean metadata row. No body read, no framing. |
| Per-harness adapter (new) | `src/memory.rs` | `MemoryCommand::Surface` — hook envelope in → `additionalContext` out. Pure helpers + impure shell. |
| Command classification (edit) | `src/commands/guard.rs` | add `MemoryCommand::Surface` to the `Read` arm (the `Command::Memory` match is exhaustive — omission fails to compile). Reads memories; writes only runtime-tier files. |

**Why a rows helper rather than an existing consumer.** `search_for_mcp` is
*find* semantics (holdback-exempt — wrong; we want the holdback). `run_retrieve`
reads bodies and frames them with a nonce (too heavy; we need only metadata). So
a thin `retrieve_rows` reusing `load_query`/`query`/`check_retrievable`, projecting
`Candidate → SurfaceRow`. This is the **only** new query-adjacent code; no parallel
logic.

### 5.2 Interfaces & Contracts

**New verb:** `doctrine memory surface` — reads a `PreToolUse` envelope on stdin,
emits the `hookSpecificOutput`/`additionalContext` line (or nothing), exits 0
always.

**Envelope (serde, all optional, `#[serde(default)]` → malformed folds to
emit-nothing):**
```rust
struct SurfaceInput {
    session_id: Option<String>,   // seen-set key
    agent_id:   Option<String>,   // Some ⇒ subagent ⇒ skip (v1)
    cwd:        Option<String>,   // root discovery
    tool_name:  Option<String>,   // Read|Edit|Write|Bash discriminator
    tool_input: SurfaceToolInput, // { file_path, command }
}
```

**Lean projection (new, `retrieve.rs`):**
```rust
pub(crate) struct SurfaceRow {
    pub uid: String,
    pub title: String,
    pub severity: String,
    pub stale: bool,   // ⚠ flag — computed from the Staleness enum, not a string compare (F2)
    pub trust: String,
}
```

**Pure helpers (`memory.rs`, unit-tested):**
```rust
enum Surface { Path, Command }
fn admits(row: &SurfaceRow, surface: Surface) -> bool;   // Path ⇒ true; Command ⇒ severity_rank ≤ high
fn dedup_diff(rows: Vec<SurfaceRow>, seen: &[String]) -> Vec<SurfaceRow>;
fn cap(rows: Vec<SurfaceRow>, n: usize) -> Vec<SurfaceRow>;
fn format_block(rows: &[SurfaceRow], surface: Surface) -> Option<String>;  // empty ⇒ None
```

**Command admission reuses the engine ordinal** — no string list (fixes the
prototype's dead `"major"`; see D3). Requires `retrieve::severity_rank` exposed
`pub(crate)` (currently private — F1):
```rust
fn admits_command(row: &SurfaceRow) -> bool {
    retrieve::severity_rank(&row.severity) <= retrieve::severity_rank(SEV_FLOOR) // "high"
}
```

**Emitted shape (advisory invariant):**
```json
{"hookSpecificOutput":{"hookEventName":"PreToolUse","additionalContext":"<block>"}}
```
Never `permissionDecision`/`updatedInput`. The emitted type has no decision field.

**Block format** (per-surface triage field):
- Path: `Doctrine memories for this file:` · `- [<trust> ⚠stale?] <title> — <uid>`
- Command: `Doctrine footguns (severity-gated):` · `- [<severity> ⚠stale?] <title> — <uid>`

**Wiring** — `plugins/doctrine/hooks/hooks.json` gains `PreToolUse` entries for
`Read|Edit|Write` and `Bash` → `${DOCTRINE_BIN:-doctrine} memory surface`.
**Separate** entries from the jail's: the jail skips the main thread, surfacing
fires *for* it (opposite audiences — cannot share a consumer). Consequence
(F4): a main-thread `Bash`/`Edit`/`Write` now spawns *two* hook processes — the
jail's (which no-ops immediately on `agent_id.is_none()`) and surface's. Both
are cheap; the surface adds one spawn on the path the jail already fast-paths.

### 5.3 Data, State & Ownership

Named constants (STD-001, in the adapter):

| const | value | meaning |
|---|---|---|
| `SEV_FLOOR` | `"high"` | command floor (critical\|high admitted) |
| `CAP_PATH` | `3` | path per-fire cap |
| `CAP_COMMAND` | `2` | command per-fire cap |
| `FETCH_LIMIT` | `8` | retrieve slate before admission/cap |
| `KEY_LOG_MAX` | (small) | command-string truncation in the log |

Runtime-tier artifacts (`.doctrine/state/`, gitignored, `rm -rf`-able):

- **Seen-set** — `mem-surface-seen-<session_id>.txt`, one file per session. New
  `session_id` → new empty file → all fresh. No active GC v1 (files are tiny;
  candidate follow-up).
- **Tuning log** — `mem-surface.log`, one JSONL line per fire:
  `{session, surface, key, fetched, admitted, suppressed_sev, surfaced, uids}`.
  The `fetched → admitted → suppressed_sev → surfaced` funnel is the retuning
  data source. `key` truncated (`KEY_LOG_MAX`). Log-write failure never affects
  the emit. **Logged whenever retrieve ran** (any main-thread fire), including
  when the emit is nothing — the suppression cases are exactly what retunes the
  floor. Subagent-skipped fires (INV-3) run no retrieve and log nothing.

Ownership: the adapter owns both files; the query core owns nothing new.

### 5.4 Lifecycle, Operations & Dynamics

One fire:
```
stdin ─▶ parse ─▶ [agent_id.is_some() ⇒ emit nothing, exit 0]     (main-thread only)
                        │
      tool_name ─┬─ Read|Edit|Write ─▶ scope = path_scope+glob(file), Surface::Path
                 └─ Bash ───────────▶ scope = command(cmd),          Surface::Command
                        │
   retrieve_rows(root, scope, FETCH_LIMIT) ─▶ Vec<SurfaceRow>   (holdback applied in-core)
                        │  PURE:
   admits(·,surface) ─▶ dedup_diff(·,seen) ─▶ cap(·,cap_for(surface)) ─▶ format_block
                        │  IMPURE:
   append surfaced uids to seen-set · append funnel line to log
                        │
   emit additionalContext (or nothing) · exit 0
```

Root resolved by the standard cwd-based discovery every memory command uses
(stdin `cwd`; `CLAUDE_PROJECT_DIR` fallback anchor, as in the jail).

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1 (advisory).** Never emits `permissionDecision`/`updatedInput`; cannot
  block a call.
- **INV-2 (fail-open).** Every failure (unparseable stdin, retrieve `Err`, no
  root, IO error) ⇒ emit nothing, `exit 0`. Never `exit 2`.
- **INV-3 (main-thread only, v1).** `agent_id.is_some()` ⇒ emit nothing.
- **INV-4 (holdback composes).** The retrieve trust-holdback runs in-core before
  the severity gate; surfacing cannot bypass it.
- **INV-5 (behaviour preservation).** `retrieve_rows` composes the query path;
  existing suites stay green unchanged.
- **Edge:** empty result / all-deduped ⇒ `format_block` returns `None` ⇒ emit
  nothing. Cap-bounded output (≤3 rows) never approaches the 10k `additionalContext`
  ceiling — no truncation logic.

## 6. Open Questions & Unknowns

All slice-scope OQs resolved during clarification (see slice-205.md); none open at
design lock. Deferred as follow-ups (not unknowns): subagent surfacing, config
knobs, session budget, `--tag`/`CwdChanged` surfaces.

## 7. Decisions, Rationale & Alternatives

- **D1 — Ship as a binary subcommand + `hooks.json` glue, not a shell script.**
  Follows the `worktree pretooluse` precedent (ADR-011 altitude). Alt: install
  template / band supplement / shipped bash script — rejected: a parallel shell
  mechanism duplicating query logic, host-coupled, harder to test. The prototype
  gave the fast-iteration phase; shipping wants the durable seam.
- **D2 — One command, `tool_name`-discriminated (two surfaces).** Mirrors
  `run_pretooluse`'s Bash-vs-Edit|Write discrimination. Alt: two commands —
  rejected (needless duplication of the shell).
- **D3 — Command admission via `severity_rank`, not a string list.** The retrieve
  scale is `critical < high < medium < low < none`; the prototype's
  `["critical","major","high"]` carried a dead `"major"` from a bleeding second
  scale. Reusing the core ordinal is DRY and fixes the latent bug.
- **D4 — Main-thread only (v1).** Inverse of the jail. Subagents run curated,
  budget-tight prompts that already distil their footguns; injection risks
  derailing them. Widening is a deferrable knob.
- **D5 — Asymmetric gating.** Path ungated (leaf none-severity hits are the
  useful ones); command `severity ≥ high` (footgun-grade). The asymmetry is why
  both surfaces earn their keep.
- **D6 — Hardcoded constants + tuning log (config deferred).** Advisory surface
  ships opinionated; a config contract is premature before real-usage data. STD-001
  constants promote to config trivially later.
- **D7 — Seen-set keyed by stdin `session_id`.** Auto-fresh per session; no manual
  reset. Alt: cross-session dedup — rejected (a footgun worth surfacing recurs
  across sessions; per-session is the right forgetting).

## 8. Risks & Mitigations

- **R1 — Hook-process spawn on every main-thread tool call.** Retrieve is
  in-process (no re-exec); cost is one binary spawn. On `Bash`/`Edit`/`Write`
  this is *additional* to the jail's spawn (which no-ops on the main thread) —
  two spawns where there was one (F4); on `Read` it is the only spawn (jail
  doesn't wire `Read`). Silent-when-no-hit keeps it cheap. Mitigation: measure;
  the tuning log records fire volume.
- **R2 — Context spend from over-surfacing.** Per-fire caps + per-session dedup
  bound it; the funnel log surfaces creep → session-budget follow-up if needed.
- **R3 — Harness contract drift.** Depends on Claude's
  `PreToolUse`/`additionalContext` schema — the same assumption the jail makes.
  Fail-open means drift degrades to silence, not breakage.
- **R4 — `.doctrine/state/` litter** from per-session seen files. Tiny, gitignored,
  rm-able; GC is a follow-up, not a v1 need.

## 9. Quality Engineering & Validation

Pure helpers (synthetic `Vec<SurfaceRow>`): VT-1 `admits_command` (critical/high
in, medium/low/none out); VT-2 path ungated; VT-3 `dedup_diff`; VT-4 `cap`; VT-5
`format_block` (path `[trust]` / command `[severity]` shapes, `⚠stale`, empty ⇒
`None`).

Command shell (synthetic stdin → emitted JSON, `run_pretooluse` shape): VT-6
main-thread Read fires; VT-7 subagent emits nothing; VT-8 Bash sub-floor emits
nothing; VT-9 unparseable stdin ⇒ nothing, exit 0; VT-10 advisory invariant
(`additionalContext` present, never `permissionDecision`/`updatedInput`).

Rows helper (`retrieve.rs`): VT-11 `retrieve_rows` — holdback applied (low-trust∧
high-severity absent); `SurfaceRow` fields projected.

VA-1 (live battery): install → wire `hooks.json` → restart/`/reload-plugins`;
main-thread Read of a path-scoped file surfaces, Bash of a footgun-scoped command
surfaces, a spawned subagent surfaces nothing.

Behaviour-preservation: existing `memory` + `retrieve` suites green unchanged.

## 10. Review Notes

### Internal adversarial pass (design lock)

Confirmed by probing the actual source; all integrated above.

- **F1 — `severity_rank` private.** `admits_command` calls `retrieve::severity_rank`
  from `memory.rs`, but it is `fn severity_rank` (private). Resolution: expose
  `pub(crate)` (a leaf ordinal, no new coupling). Integrated §5.2.
- **F2 — Staleness magic-string compare.** `SurfaceRow.staleness: String` +
  `== "stale"` is a brittle STD-001 smell. Resolution: `SurfaceRow.stale: bool`
  computed from the `Staleness` enum in `retrieve_rows`. Integrated §5.2.
- **F3 — guard.rs exhaustive match.** `Command::Memory` classifies every
  `MemoryCommand` with no catch-all; `Surface` omission fails to compile.
  Resolution: add `Surface` to the `Read` arm; `src/commands/guard.rs` added as a
  4th design-target. Integrated §5.1 + selectors.
- **F4 — R1 spawn undercount.** Main-thread `Bash`/`Edit`/`Write` spawns *two*
  hook processes (jail no-op + surface), not one. Honesty fix; still cheap.
  Integrated §5.2, §8-R1.
- **F5 — stale coarse selector.** `install/**` `scope-relevant` predates the
  reframe off `install/`. Resolution: removed; coarse fence retargeted to the
  design-target set.
- **F6 — log on suppression.** Clarified: the tuning log fires on every
  main-thread retrieve, including empty/all-suppressed emits — those are the
  retuning signal. Integrated §5.3.

No governance conflicts surfaced (POL-002 satisfied structurally; ADR-011 altitude
matches the jail precedent; ADR-001 layering respected). No `/consult` trigger.
