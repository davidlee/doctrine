# Unified harvest surface

## Context

Four skills independently re-implement the end-of-work harvest — surfacing what
was *produced* (work delta), *learned* (durable knowledge), and what *decisions*
remain open: `/notes`, `/handover`, `/next`, and the review-synthesis skills
(audit/reconcile). The templates drift; nothing carries a decision recorded in
notes through to the handover or the next agent's orientation. RFC-011 L2'
lever (compact durable handoff — carry-forward half; worker-assembly is L6,
SL-186/187). IMP-060 (relocate /handover into core) was the sibling fix, done.

Bundles IMP-042: the code-review skill is structurally orphaned — no backlog or
lifecycle awareness — and its findings are a harvest source like any other.
IMP-059 sits `after` IMP-042, so the integration lands first.

Originates from IMP-059 and IMP-042.

## Scope & Objectives

Ordered per the IMP-042 → IMP-059 `after` edge:

1. **Code-review corpus integration (IMP-042).** Give the shipped code-review
   skill backlog awareness (findings that outlive the session → `backlog new`)
   and lifecycle awareness (where in slice lifecycle its findings should land —
   review ledger vs backlog vs reconciliation). Concrete shape is design's job;
   the boundary reference is `using-doctrine.md` work/knowledge/decision.
2. **Harvest surface (IMP-059).** One entry point that, at the end of a coherent
   unit of work, collects the delta, surfaces open decisions, and records
   durable knowledge. `/notes`, `/handover`, `/next`, and review synthesis
   become projections of that one harvest — consuming its output, not
   re-deriving it. Key property: one entry point, one canonical output.
3. **Knowledge/decision sink = SL-214's `/knowledge` skill.** The harvest's
   "learned" and "open decisions" legs write ASM/DEC/QUE/CON records rather
   than inventing another template. Design must consume SL-214's design, not
   duplicate it.

All skill edits under `plugins/` (install source); POL-002 binds — no
repo-local couplings.

## Non-Goals

- New engine/CLI verbs — if the harvest needs a surface the CLI lacks,
  `/consult` before widening scope.
- Worker-assembly half of RFC-011 L2 (L6 — SL-186/187).
- Changing the adversarial review ledger protocol (ADR-007) — code-review
  integration routes *outputs*, it does not touch turn mechanics.
- The `/knowledge` skill itself (SL-214).

## Risks & Open Questions

- Harvest shape: dedicated skill/verb vs shared reference doc the four skills
  cite — design fork with real coupling consequences.
- Four consumer skills rewritten in one slice — regression surface is wide;
  behaviour-preservation for each skill's existing contract matters.
- Sequencing on SL-214: design can proceed in parallel, but execution of the
  knowledge-sink leg needs `/knowledge` landed (soft `after` edge, not a hard
  `needs` gate).
- IMP-042's body was empty — its scope is thin by record; step 1's boundary
  must be pinned in design before planning.

## Verification / Closure Intent

- VA: each projection skill (`/notes`, `/handover`, `/next`, review synthesis)
  demonstrably consumes the harvest output — no independent re-survey prompt
  remains in its body.
- VA: code-review skill routes a persistent finding to `backlog new` and a
  session-scoped finding to the ledger, per the boundary.
- VA: harvest's knowledge/decision legs produce records via the SL-214 surface.
- VH: POL-002 reflex — every edited `plugins/**` skill resolves in a fresh
  client.

## Summary

## Follow-Ups
