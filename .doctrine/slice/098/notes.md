# SL-098 notes — design phase

## RV-078 Inquisition (2026-06-18)

Design arraigned before the Inquisition. Eight findings raised and resolved —
two blockers requiring redesign before plan.

### Summary of findings

| # | Severity | Finding | Disposition |
|---|---|---|---|
| F-1 | blocker | `/design` skill already has requirements pass at state 3 — design proposes adding duplicate at sub-step 4a | design-wrong: redesign to acknowledge current state or drop §3 |
| F-2 | blocker | REQ-DNN handle is structured metadata as prose — violates storage rule (AGENTS.md § storage model; the design's citation of doc/entity-model.md is moot — that tree is SL-082-bound for erasure) | design-wrong: define TOML facet for implied requirements |
| F-3 | major | `plan.toml [requirements]` as dead fields corrupts storage tier model | design-wrong: move to plan.md prose or make tooling read it |
| F-4 | major | Orphan placement depends on altitude framework that does not exist | design-wrong: add /consult guardrail + backlog dependency |
| F-5 | major | `/plan` skill says `[requirements]` stays empty — design contradicts without acknowledging | design-wrong: acknowledge current instruction and explain change |
| F-6 | minor | Orphan section position in reconciliation brief undefined | design-wrong: nest under Governance/spec (REV) |
| F-7 | minor | Walkthrough scenarios assume all amendments in place — not incremental | design-wrong: add per-phase walkthroughs or dependency labels |
| F-8 | nit | Line-number references to entity-model.md will rot | fix-now: replace with section names |

### Key design decisions needed before plan

1. **REQ-DNN storage**: TOML facet (`[[implied_req]]`) or continue with prose-only? Blocked on F-2 resolution.
2. **Requirements pass placement**: Keep at state 3 (current), move to between states 4-5, or drop the /design section entirely? Blocked on F-1 resolution.
3. **`[requirements]` in plan.toml**: Move to plan.md prose, or make tooling-read? Blocked on F-3 resolution.
4. **Altitude assessment**: Gate on IMP-097, or proceed with `/consult` guardrail?

### Backlog follow-ups

- IMP-097: Altitude assessment framework for requirement placement (created from RV-078 F-4)
- IMP-096: Pre-existing — requirements capture and refinement skills

### Memory recorded

- `mem_019ed9f59a8f7f6398145b4a99c59f62`: Design staleness gotcha — skill-amendment designs must read current skill files
