# IDE-044: Research stage before design and planning

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

A lot of wasted tokens trace to insufficient research during design/planning:

- inquisition finds design assumptions not grounded in actual code or research
- plan selectors under- or mis-represent the files that actually need to change
- plan assumptions don't hold in the face of implementation reality

The codebase is now large enough that a dedicated research step as a
prerequisite to design / plan / phase-planning looks worth its cost.

Counterpoint worth weighing during shaping: much better spec coverage might
reduce the need — and would likely be a better, more efficient route to the
same results than throw-away research.

## Sketch (ideal shape)

- **Automatic pre-design research round**, two threads:
  1. finding and testing applicability of governance (specs, ADRs, policies, …)
  2. mapping high-level systems, likely code hotspots, and relevant memories
- **Spec-coverage trigger**: lack of relevant/appropriate coverage (heuristics
  TBD) prompts creating new spec coverage during the slice, using the research
  material. Research doesn't re-litigate what's well specced.
- **Reconciliation loop**: inconsistencies in spec or memories surfaced during
  design or implementation are corrected during slice reconciliation.
- **Generic agent deferral**: research agents are configured outside doctrine;
  doctrine defers to them generically rather than managing them.
- **Access**: agents are read-only, but DO get the memory MCP tool & hook(s)
  where harness-appropriate (in tension with the read-only point — resolve
  during shaping).
- **Persistence**: a research summary (or a set of research documents) is
  persisted to disk — probably gitignored, in the slice folder.
- **Selector feedback**: research informs selectors directly; there is a
  process for ensuring/checking selectors are correct.
- **Incremental refresh**: additional research is triggered as the design is
  clarified or changed; the summary document is updated. Planning, adversarial
  review, and phase planning may all trigger further passes (only if
  necessary), integrated into the (or possibly a separate) research summary.
- **No split-brain**: research documents must not cause main-vs-worktree
  split-brain even if updated during dispatch. Research is harvested at end of
  slice if there's potential value.
- **Findings become globbed memories**: important research findings are
  captured as memories with appropriate path globs, so they auto-reveal via
  the file-inspection hook when an agent later touches the relevant files.

## First-slice shape (shaped 2026-07-25)

Kernel: **research artefact + pre-design round**. Everything else in the
sketch is a consumer of that artefact or a refinement trigger on it.

Settled in shaping conversation:

- **Storage**: runtime tier — `.doctrine/state/research/SL-NNN/` with a
  convenience symlink from the slice folder (the `phases` precedent).
  Dissolves dispatch split-brain by construction (runtime state is
  per-worktree, never merged); makes end-of-slice harvest explicit.
- **Artefact form**: free markdown under a conventions contract (mandated
  sections, durable-id + `file:line` citation forms) — no schema. Nothing
  queries it and it isn't durable, so structure doesn't earn its cost.
  Possible exception: a stamped staleness baseline (slice id, date, and a
  `path → content-hash` map of the design/scope docs researched against) —
  the one thing a checker might read. Not a version int and not a git OID:
  reuse the SL-040 `ContentSet` primitive (`src/contentset.rs`), which
  `review prime` already uses for warm-cache drift. Absence-is-defined means
  the pre-design round (no design.md yet) and later refresh rounds use the
  same `diff` mechanism; the check is advisory, not a gate. Decide at design.
- **Runner seam**: no executable config seam in slice 1. Skills say "use a
  research agent"; the *project* defines what that means (CLAUDE.md /
  `.doctrine/governance.md` § research) — scripts (`pi-scout`/`pi-research`),
  `claude -p`, or harness-native subagents. POL-002-clean; "defer, don't
  manage" preserved. A command-naming config seam is viable later if
  subprocess (`claude -p`) turns out to be the common mechanism across arms.
- **Skill wiring**: `/slice`, `/design`, `/plan`, `/phase-plan` all point at
  one common research-conventions surface, plus a skill-specific note on how
  each consumes the results.
- **`/research`**: new skill, **not routed** — invoked from within the
  lifecycle skills, not a routing-table stage. Likely candidate to *be* the
  common conventions surface.
- **Memory capture**: researchers stay read-only; the orchestrating agent
  distills the research doc into globbed memories at harvest. (Deferred to a
  later slice; resolves the read-only vs memory-MCP tension with no new
  access surface.)
- **CLI surface**: a minimal mint verb (create dir + symlink + stamp
  staleness header) probably worth it, particularly if `claude -p` becomes
  the mechanism. Shape decided at design.
- **Selectors**: no new machinery in slice 1 — hotspot map existing before
  `/plan` means selectors are drafted from evidence; SL-180's design-time
  dry-run already provides checking.

Deferred to later slices: spec-coverage trigger heuristics; findings→globbed
memories; incremental refresh + review/phase-plan triggered passes;
reconciliation prose additions.

Open (design-time, ideally eval-informed): one research doc vs two (per
thread); staleness header structured vs prose; mint-verb shape.

## Neighbors

- `/preflight` skill — bounded up-front research posture; this proposes a
  durable, persisted, lifecycle-integrated stage rather than a conduct step.
- IMP-104 — general pi subagent spawn pattern for scout roles. Project-local
  only: addressed here via `scripts/pi-scout` + `scripts/pi-research` and a
  CLAUDE.md directive preferring them over built-in subagents. Those scripts
  do not (yet) have access to memory tools — relevant to the read-only vs
  memory-MCP tension above.
- IMP-310 — selector-conformance tech spec; IMP-162, IMP-256 (resolved),
  SL-180 — selector-correctness cluster this would feed from the front.
- IDE-016 — agent UX research on efficient read surfaces.
- RFC-011 — token-efficiency benchmarking that motivates this.
