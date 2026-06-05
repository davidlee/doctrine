---
name: design
description: Use when a slice needs architectural shaping before implementation — decision triage, weighing tradeoffs and alternatives, and section-by-section validation of design.md until the decisions lock. Routed to from /route once a slice exists.
---

# Design (compact variant)

> Experimental compressed port of the design skill. Not the active skill —
> `SKILL.md` is. Kept for comparison / later A/B.

You are translating scoped intent into an implementable design, recorded in the
slice's `design.md`. `design.md` is canon for design intent: if it and a later
plan conflict, reconcile via `design.md` first.

Inputs: the slice folder, any existing `design.md`, and related artifacts,
source material, and research.

## Workflow

Complete in order; each stage depends on the one before it.

1. **Explore context.** Read the slice scope, related `doc/*` specs, prior art,
   and recent commits. Run `/canon` so the governing ADRs, policies, and
   standards are in view. `/retrieve-memory` for gotchas on the surface. Then
   triage the design surface out loud: open questions that must resolve, risks
   and underspecified areas, assumptions you carry, the decisions that shape
   everything downstream, and the constraints that bound them.

2. **Ask clarifying questions** — one at a time. Drive toward enough clarity to
   lock a design. For each unresolved question, pick the most impactful or
   naturally next one, consider its implications, then offer 2-3 options with
   tradeoffs and a recommendation. Prefer multiple-choice; open-ended is fine.
   Focus on purpose, constraints, success criteria, verification strategy.
   Continue until the user accepts your summary. Then ensure the slice scope
   (`slice-nnn.md`) still reflects the shared understanding before proceeding.

3. **Present the design section by section** — never dump the whole thing at
   once. Get approval per section. When a section shapes later ones, present it
   first and treat the rest as provisional until the foundation is coherent.
   Prefer concrete detail over hand-wavy prose:
   - current vs target behaviour
   - module responsibility boundaries; coupling / cohesion analysis
   - types / interfaces / function signatures; data structures & algorithms
   - example data shapes; data-flow boundaries; invariants & edge conditions
   - code-impact summary (paths + intended changes)
   - verification alignment — what evidence must change or be added
   - design decisions and remaining open questions

   Do targeted research where needed to keep the design fitted to the real
   implementation surface.

4. **Write `design.md`** and commit.

5. **Adversarial review.** Run `/inquisition` against the design — a hostile pass
   for vague sections, hidden assumptions, weak verification, missing
   code-impact detail, and misread or weakly-applied governance. Integrate the
   findings (this may send you back to step 2 or 3). If the pass exposes
   governance conflicts or ambiguous authority, stop and `/consult` rather than
   guessing. Reconcile `slice-nnn.md` so scope, risks, and open questions still
   match the revised design.

6. **Offer next steps.** After integrating the internal pass, offer the user a
   choice: (a) a prompt for an external adversarial reviewer, or (b) proceed to
   `/plan`. Do not presume approval — multiple review rounds may be needed.

## Guardrails

- Do not present "the whole design" as settled before the foundational sections
  and decisions are validated.
- Do not hide unresolved assumptions inside polished prose — name them.
- Do not confuse detailed design with implementation planning.
- Do not treat a polished full-file rewrite as progress while hard questions
  remain open.
- Do not move to `/plan` while `slice-nnn.md` tells an older story than the design.
- Do not treat governance as optional background when the design makes
  architectural or workflow choices.
