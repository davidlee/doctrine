# Pre-design research stage v1

## Context

RFC-011 instrumentation shows tokens bleed in orientation, not execution — and
a recurring cluster of it traces to insufficient research before design and
planning: inquisition finds design assumptions not grounded in actual code,
plan selectors under- or mis-represent the real touch-set, and plan assumptions
fail against implementation reality (IDE-044 § Problem).

The missing piece is a persisted research artefact that exists *before*
design.md is authored. IDE-044 § First-slice shape records the shaping
conversation that fixed this slice's kernel; the rest of the idea (spec-coverage
trigger heuristics, findings→globbed memories, incremental refresh triggers,
reconciliation prose) is deferred to later slices.

## Scope & Objectives

The kernel: **research artefact + pre-design round**, four deliverables.

1. **`/research` skill** (new, unrouted — invoked from lifecycle skills, not a
   routing-table stage). It is the single conventions surface: the artefact
   contract (mandated sections for the two research threads — governance
   applicability; system map / hotspots / relevant memories), citation forms
   (durable ids, `file:line`), the refresh advisory, and the deferral rule —
   skills say "use a research agent", the *project* defines what that means
   (CLAUDE.md / governance § research: scripts, `claude -p`, or harness
   subagents). Doctrine holds no executable runner seam (POL-002).
2. **Storage**: runtime tier — `.doctrine/state/research/SL-NNN/`, with a
   convenience symlink from the slice folder (the `phases` precedent).
   Per-worktree by construction, so no dispatch split-brain; end-of-slice
   harvest is explicit.
3. **Mint verb + staleness stamp**: a small CLI verb that creates the research
   dir + symlink and stamps a staleness baseline — slice id, date, and a
   `path → content-hash` map of the design/scope docs researched against,
   reusing the SL-040 `ContentSet` primitive (`src/contentset.rs`, the
   `review prime` warm-cache mechanism). Staleness is advisory (`diff` →
   "research may be stale"), never a gate. Absence-is-defined means the
   pre-design round (no design.md yet) and later refresh rounds share one
   mechanism. Exact verb shape is a design decision.
4. **Consumption hooks**: `/slice`, `/design`, `/plan`, `/phase-plan` each get
   a short pointer — run/refresh research per `/research`, plus a
   skill-specific note on how to consume the results (design assertions cite
   the research doc; plan selectors draft from the hotspot map).

Selector payoff rides along free: a hotspot map existing before `/plan` means
selectors are drafted from evidence, and SL-180's design-time dry-run already
provides the checking half. No new selector machinery here.

## Non-Goals

- Spec-coverage trigger heuristics ("is this surface well-specced?") — needs
  real research docs to exist first; `/spec-coverage-assessment` covers the
  manual path meanwhile.
- Findings → globbed memories (auto-reveal on file inspection). Later slice;
  the settled design is orchestrator-distills-at-harvest, researchers stay
  read-only.
- Incremental refresh automation and review/phase-plan triggered passes —
  refinement loop on an artefact that must exist first; this slice ships only
  the advisory staleness check.
- Executable runner configuration (`[research]` command seam in
  `doctrine.toml`) — viable later if arms converge on subprocess-shaped
  runners; slice 1 defers via prose convention.
- Managing, spawning, or wrapping research agents in any way.
- Reconciliation prose additions for surfaced spec/memory inconsistencies.

## Affected surface

- `plugins/doctrine/skills/` — new `research/SKILL.md`; consumption hooks in
  `slice/`, `design/`, `plan/`, `phase-plan/` SKILL.md files.
- `.agents/skills/` — installed mirror of the above.
- `src/` — mint verb (command layer), path helpers, `ContentSet` consumer;
  likely a small new engine leaf for the baseline stamp.
- `install/` — reference-doc touch if the conventions warrant a pointer
  (`using-doctrine.md` / `governance.md`); decided at design.

## Risks, assumptions, open questions

- **A1**: prose-convention runner deferral is enough for both arms today
  (pi scripts exist; claude arm can use subagents or `claude -p`).
- **OQ-1**: one research doc or two (per thread)? Ideally eval-informed;
  educated guess at design time is acceptable.
- **OQ-2**: mint verb shape — standalone kind-verb vs `slice research <id>`
  subcommand; and whether `paths`/`show` surfaces are worth minting in v1.
- **OQ-3**: does the staleness advisory surface in `doctrine status` /
  `reports next`, or only in the skill flow? Lean: skill flow only in v1.
- **R1**: skill-prose hooks without any enforcement may under-deliver
  (agents skip the round). Mitigation: the mint verb makes the artefact's
  absence visible; evals under RFC-011 judge whether harder gating is needed.

## Verification / closure intent

- Mint verb + stamp covered by VT (unit + e2e: dir/symlink creation, baseline
  stamp shape, staleness diff advisory, absence semantics).
- Skill content and consumption hooks are VA/VH: a dry-run slice exercising
  the round end-to-end (research doc authored pre-design, cited from design,
  selectors drafted from the map).
- Closure evidence includes at least one real slice driven with the research
  round, with token/friction observations recorded to RFC-011 case-notes.
