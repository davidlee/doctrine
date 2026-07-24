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

## Neighbors

- `/preflight` skill — bounded up-front research posture; this proposes a
  durable, persisted, lifecycle-integrated stage rather than a conduct step.
- IMP-104 — general pi subagent spawn pattern for scout roles (spawn
  mechanics this would defer to).
- IMP-310 — selector-conformance tech spec; IMP-162, IMP-256 (resolved),
  SL-180 — selector-correctness cluster this would feed from the front.
- IDE-016 — agent UX research on efficient read surfaces.
- RFC-011 — token-efficiency benchmarking that motivates this.
