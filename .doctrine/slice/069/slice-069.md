# Shipped memory corpus as a cohesive client onboarding anchor

## Context

Doctrine ships 14 orientation memories (the ADR-002 class: `repo=""`,
`anchor_kind=none`, scoped, evergreen) embedded in the binary and surfaced
through `doctrine memory find`/`retrieve`. These orient an agent driving
doctrine in *any* repo. But the corpus was authored as a proof-of-concept for
SL-018, not as a complete client onboarding surface — many of doctrine's
capabilities have no shipped memory coverage.

See [research.md](research.md) for the full catalogue, gap analysis, and
non-shipped memories with client-relevant content.
See [design.md](design.md) for the technical design.

## Scope & Objectives

1. Author 13 new shipped memories covering capability gaps across installation,
   boot snapshot, reading entities, reference docs, relating entities, memory
   recording, backlog, ADRs, specs, requirements, audit, revisions, and
   policies/standards (review deferred to post-SL-068).
2. Trim the boot snapshot Memory section to signpost-type only (IMP-007).
3. Add `boot --check` governance section populatedness warning + boot snapshot
   nudge comment for empty Policies/Standards sections (IMP-015).
4. Refresh the existing 14 shipped memories for post-SL-018 staleness —
   CLI verb coverage, directory layout, lifecycle stages, conventions,
   cross-references, duplication, scope correctness.
5. Integration: embed, sync, retrieval surface test, full gate.

## Non-Goals

- Review (RV kind) shipped memory — deferred until SL-068 lands.
- Worktree/dispatch, thread visibility — Tier 4, advanced.
- Lifting `corpus::lint_master` out of `#[cfg(test)]` — out of scope.
- Solving the self-updating-shipped-memories problem (OQ-1) — surfaced in design,
  not solved.

## Summary

27 shipped memories (13 new + 14 existing). 5 phases: author new (PHASE-01),
boot snapshot changes (PHASE-02), corpus refresh (PHASE-04), integration
(PHASE-05). PHASE-01 and PHASE-02 are file-disjoint and parallelisable.
PHASE-04 is handover-friendly with an explicit consistency checklist.

## Follow-Ups

- Review (RV kind) shipped memory after SL-068 lands.
- Self-updating shipped memories mechanism (OQ-1).
