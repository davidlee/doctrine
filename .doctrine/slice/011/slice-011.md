# Cache-friendly session boot context

## Context

Every doctrine session re-pays to learn project governance. `/canon` reads
`CLAUDE.md`/`AGENTS.md`/ADRs/`doc/*`; CLAUDE.md itself nags `just list-memories`
"do it now"; `/route` describes the gate in a skill body. ADR list, memory
pointers, and the routing digest are all **stable governance state** that the
agent rediscovers via Read/Bash tool calls turn-by-turn — uncached, repeated
every session.

The spec-driver "preboot" design (research in `boot-research.local.md`) solved
this for a sibling tool: pre-generate a static governance snapshot into Claude
Code's **cacheable session-start prefix**, so it costs zero tool calls and only
busts cache when governance actually changes. This slice adapts that idea to
doctrine — **Claude-only, corrected mechanism, internal-call (no subprocess)**.

### What Claude Code actually caches (verified, 2026 — corrects the research doc)

The research doc's load path (`.claude/rules/<f>` symlink) is **unreliable**:
`.claude/rules/` is not generally auto-loaded into the prefix — it's lazy /
path-scoped (only a rules file with **no** `paths:` frontmatter loads at
startup). The documented, dependable mechanism is different:

- **CLAUDE.md `@path` imports are inlined into the cached prefix at session
  start** — the whole tree is resolved (≤4 hops) before turn 1. This is the
  load-bearing fact. Regenerating an imported file changes the prefix; an
  unchanged file keeps the cache warm.
- **Skill bodies and skill-internal `@`-refs are lazy** — loaded only on
  invocation, as conversation turns (cache-extending, not prefix). Confirms the
  research doc's core insight: governance must ride CLAUDE.md, not a skill.
- **SessionStart-hook `additionalContext` is a separate, non-cached user
  message** every session. So the hook must NOT push boot content through its
  stdout (spec-driver's `startup.sh` does — we drop that). The hook only
  regenerates the file; Claude loads it via the `@`-import.

→ Mechanism: **SessionStart hook regenerates the boot file → CLAUDE.md
`@`-imports it → cached prefix → content-diff write busts cache only on
governance change.**

## Scope & Objectives

- **`doctrine boot` verb** (name provisional). Assembles the governance snapshot
  and writes it via a content-diffing writer — write to disk **only if content
  changed** (cache key is file content; a no-op write would needlessly bust the
  prefix). Honours the pure/imperative split: a pure assembler
  (`inputs → snapshot string`) behind the existing impure shell that reads
  files / gathers listings / writes. **No subprocess** — doctrine is one binary,
  so listings come from in-process calls (the `adr`/`memory` list functions),
  not shelling out to itself (a strict improvement over spec-driver's exec hop).

- **Boot snapshot content** (the four, per scoping decision):
  - **Accepted ADRs** — `adr list -s accepted` rendered as a compact table.
  - **Memory pointers** — the memory index / `list` (the `just list-memories`
    nag, paid once into the prefix).
  - **Routing / process digest** — a condensed `/route` decision table + core
    process, so routing lives in the cached prefix, not a skill turn.
  - **Extensible listing seam** — the snapshot is built from a declared sequence
    of `(heading, source)` entries so **policies / standards slot in later**
    with no rework. Empty sources render a benign marker, never a crash.

- **Customizable governance surface (folded in as an input).** Add the
  user-editable `doctrine.md` analog (`doc/memories/customizable-governance-surface.md`):
  a project-authored file under `.doctrine/` that install **seeds-if-missing and
  never clobbers** (SL-010 override-hatch discipline), feeding the boot snapshot
  and referenced by `/canon`. This is the "articles of truth" pointer layer —
  not a competing source of truth to ADRs / `doc/*`.

- **New `src/boot.rs` module, two verbs.** `doctrine boot` regenerates the
  snapshot (the hook target); `doctrine boot install` wires the `@`-import + the
  per-harness session refresh. `install.rs`/`skills.rs` stay untouched (dodges the
  concurrent slice-012 edits). Boot file is **derived → gitignored**
  (`.doctrine/state/boot.md`).

- **Harness seam — Claude is not core-special (loosely coupled).** Vendor-specific
  wiring lives behind a `Harness` trait; core only produces the neutral snapshot +
  iterates selected harnesses. Ship two adapters: **Claude** (import into
  `CLAUDE.md` + a `settings.local.json` SessionStart hook) and **codex** (import
  into `AGENTS.md`, no hook). Each harness claims exactly **one** import file (so a
  ref never inlines the snapshot twice); the union is deduped by inode — this
  repo's `CLAUDE.md → AGENTS.md` symlink collapses to a single write. A third
  harness is a new impl, no core edit (static registry; not dynamic plugin
  loading — YAGNI). The `@`-import is an idempotent, never-clobber prepend.

- **Snapshot carries the resolved CLI path.** `doctrine boot` bakes its own
  `current_exe()` into the snapshot ("Invoking doctrine") so the agent reliably
  invokes the off-PATH binary — the same path the Claude hook uses (single source).

- **SessionStart hook (install).** Install writes/refreshes a `.claude/` hook +
  settings wiring that runs `doctrine boot` at session start (and on `clear`),
  errors swallowed. Rides the same managed-vs-foreign ownership rules as the
  skills symlink install (overwrite our own, never clobber a user's).

- **`/route` gains a boot-presence check** (per scoping decision — no separate
  `/boot` skill, no sigil handshake). One line: confirm the boot snapshot is in
  context; warn + point at `doctrine boot` if absent.

- **Rewrite root `AGENTS.md`** (= `CLAUDE.md` via the symlink) to lean on the new
  surface, done **late in the slice** once the boot mechanism exists. Today it
  hand-recites the full CLI, the `just list-memories` nag, and the process — all
  of which the boot snapshot now loads into the *same* cached prefix, so the
  prose **duplicates the snapshot and double-spends the prefix budget**. The
  rewrite: (a) delegate the recited CLI / governance listings to the boot snapshot
  (carry the `@`-import, not a copy); (b) orient around `/route` + the skill set
  rather than reciting commands (skills now drive the process). (The memory-story
  reconciliation — entity store vs the legacy `doc/memories/` bargain-bin — was
  **subsumed by SL-012**, which retired the bargain-bin and ported its notes; this
  rewrite no longer owns it.) Net: AGENTS.md shrinks to durable orientation the
  snapshot can't carry; the snapshot carries the volatile/derived governance.

## Decision: load via CLAUDE.md `@`-import, not a `.claude/rules` symlink

The research doc's `.claude/rules/<f>` symlink is **not** a dependable
cached-prefix loader (verified above). Doctrine loads the snapshot through a
CLAUDE.md `@`-import — documented, fully resolved into the prefix at startup,
cache-keyed by content. (A no-`paths`-frontmatter `.claude/rules` file is a
fallback that also startup-loads, but it is the less-documented path; the
`@`-import is primary. Final selection is a design-stage confirmation, not an
open scope question.)

## Non-Goals

- **No pi support.** The research doc's entire pi belt/suspenders machinery
  (`APPEND_SYSTEM.md`, `session_shutdown` extension, Nix wrapper) is dropped.
  Targets are Claude (CLAUDE.md `@`-import + SessionStart hook) and codex
  (AGENTS.md `@`-import) — both reached via the one generated, agent-neutral
  snapshot. codex has no SessionStart-hook equivalent, so its boot is
  regeneration-on-`doctrine`-run + the static AGENTS.md import (staleness window
  accepted; see open questions).
- **No `/boot` skill, no `Δ ∴ ⊤` sigil handshake, no boot-prompt JSON** from the
  hook. The `/route` gate already governs; it absorbs the presence check.
- **No per-turn / `additionalContext` injection** — defeats prefix caching.
- **No policy/standard content** — none exist yet; this ships only the
  extensible seam they will plug into.
- **No new ADR/memory query surface** — reuses existing list functions as-is.
- **Not rewriting the downstream seed** (`install/rules/AGENTS.md`, the thin
  stub) beyond adding the `@`-import wiring — only *this repo's* hand-maintained
  root `AGENTS.md` is rewritten. Consumers keep authoring their own.
- **No exec-config indirection** (spec-driver's `workflow.toml [tool] exec`) —
  unless hook binary resolution forces it (see open questions).

## Risks / open questions

- **CLAUDE.md ↔ AGENTS.md topology — RESOLVED in design (b).** Keep the
  `CLAUDE.md → AGENTS.md` symlink; the harness seam claims one import file each and
  dedups the union by inode, so the symlink collapses to a single ref-write. No
  unpicking needed.
- **Hook binary resolution.** spec-driver used `uv run spec-driver`; doctrine has
  no PATH entry in dev (`./target/debug/doctrine`) and an unknown invocation
  downstream (installed binary vs `cargo`). The hook must locate `doctrine`
  robustly — resolve in design (PATH probe, configured path, or skip-if-absent).
- **First-session bootstrap.** The hook regenerates the file, but Claude reads
  the `@`-import at the *same* startup — confirm ordering (guide says hook runs
  before the boot read, so a fresh file is picked up; a never-yet-generated
  import resolves to nothing the first time → benign, regenerated next start).
- **Snapshot size vs prefix budget** — keep listings compact (tables, not bodies;
  the DEC-091-4 lesson).
- **Two-writer coupling.** The boot snapshot duplicates data owned by ADR/memory
  stores; it must stay a pure projection (regenerable, never authoritative).
- **codex boot behavior (verify late in slice).** Unknown whether codex resolves
  AGENTS.md `@`-imports the way Claude inlines them, and whether its system
  prompt picks the snapshot up cleanly. No SessionStart-hook equivalent → no
  per-session regeneration; the import may serve a stale snapshot until the next
  `doctrine` invocation regenerates it. **Closure includes a live codex run** to
  confirm the boot context lands and the routing digest is honoured; resolve the
  staleness/regeneration story from what that shows.

## Summary

Adapt spec-driver's cache-friendly preboot to doctrine (Claude + codex, no pi),
with the load mechanism corrected to an `@`-import in CLAUDE.md *and* AGENTS.md
(the research doc's `.claude/rules` symlink does not reliably cache). A
`doctrine boot` verb
assembles a governance snapshot — accepted ADRs, memory pointers, routing digest,
and a user-editable governance surface — through a declared, policy/standard-
extensible listing seam, written content-diffed so the cached prefix only busts
when governance changes. Install wires the `@`-import and a SessionStart hook
that regenerates it; `/route` gains a presence check. Pure assembler behind the
existing impure shell; listings via in-process calls, no subprocess.

## Follow-Ups

- **Policy / standard listings** — plug into the extensible seam once those
  entities exist (the seam is built here; the sources are not).
- **`/canon` + governance surface** — broaden `/canon` (and plausibly
  `/preflight`, `/execute`) to load the customizable surface at their own points,
  per the parked memory's "load-on-route" note. This slice wires it into boot and
  `/canon`; the wider load points can follow.
- Retire `doc/memories/customizable-governance-surface.md` once its idea is
  realized here (or convert to a durable note).
