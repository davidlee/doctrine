# CHR-034: Dispatch funnel: enforce cargo fmt --check before committing phase cuts

## Context

Surfaced by the SL-190 audit (RV-234 F-3). The dispatched impl bundle
(`review/190`) carried rustfmt drift across 9 SL-190-touched files; the drift was
only caught when `doctrine check gate` (which runs `cargo fmt`, mutating) reformatted
them during audit. Whitespace-only, no semantic impact, but it means the dispatch
funnel / verify-worker committed phase cuts that were **not** `cargo fmt`-clean.

## Chore

Add a `cargo fmt --check` (or `doctrine check`-equivalent) gate to the dispatch
funnel before a phase cut is committed — either in the worker's end-green step or in
the orchestrator's verify-worker / funnel-commit beat. Fail-fast on drift so the
committed `phase/*` and `review/*` refs are format-canonical and integration merges
don't accrete style noise.

## Notes

- Low priority — hygiene, not correctness.
- Touches the dispatch orchestrator/worker path; coordinate with the RFC-005 topology
  churn rather than landing in isolation.

