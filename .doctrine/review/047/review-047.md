# RV-047: Inquisition on SL-080 plan

**Facet:** plan  
**Raiser:** inquisitor  
**Target:** SL-080

## Brief — Lines of Interrogation

This tribunal examines the plan another agent prepared for SL-080, measured
against the locked design (design.md D1–D9), the slice scope (slice-080.md),
and governing canon (ADR-003, ADR-009). The plan is accused on these counts:

1. **Coverage.** Does every design decision (D1–D9) have a corresponding phase
   or explicit deferral, with no phantom dependencies?
2. **Phase coherence.** Are EN/EX/VT criteria concrete, testable, and traceable
   to design decisions — or are they ceremonial placeholders?
3. **Sequencing soundness.** Does phase ordering respect actual dependencies?
   Are the phantom-dependency and circular-dep tests passed?
4. **Scope fidelity.** Does the plan stay within the slice scope, or does it
   drift into non-goal territory (CLI verb, coverage engine, conduct
   enforcement)?
5. **ADR-009 F14.** Is the routing row added only when the skill ships —
   shipped-not-reachable?
6. **D8 compliance.** Does any phase reference a nonexistent `slice reconcile`
   CLI verb?
7. **No parallel implementation.** Any duplicated or redundant work?

The sanctioned doctrine:
- SL-080 design.md (D1–D9)
- SL-080 slice-080.md (scope, non-goals)
- ADR-003 §7/§11 (audit/reconcile seam, deferred machinery)
- ADR-009 §1 (FSM topology), F2/F14 (shipped-not-reachable)
- `install/review-ledger.md` (RV mechanics, not restated)

## Synthesis — Judgement and Sentencing

*In nomine Doctrinae, ego Inquisitor huius Tribunalis, auditis testibus et
perspectis scripturis, hanc sententiam fero.*

**The accused plan is NOT heretical.** It is directionally sound, structurally
complete, and stays within the slice scope. Four phases cover four scope items
with no phantom dependencies, no drifted non-goals, and no parallel
implementation. The ADR-009 F14 shipped-not-reachable constraint is correctly
gated at PHASE-04, and D8 compliance (no reference to a nonexistent CLI verb)
is verified. These are the marks of a plan that read the design.

Yet the accused has **confessed under cross-examination** to five taints, now
purged. The confessions and their penance:

| Finding | Severity | Sin | Penance |
|---|---|---|---|
| F-1 | major | EN criteria claim hard dependency where plan.md admits soft narrative sequencing — the two statements cannot both be true | Rewrite PHASE-02 EN-1 and PHASE-03 EN-1 to name the honest soft reason, not invent a structural gate |
| F-2 | minor | D9 principle (no re-audit, target inspection allowed) stated as settled in EN-1 but never demanded in any EX criterion — a skill could pass all EX checks yet violate the design | Add EX-7 to PHASE-01 demanding the D9 contract be stated in skill prose |
| F-3 | minor | PHASE-03 EN-2 delegates stale-prose verification to IMP-008 by reference — implementer must cross-reference an external document | Restate the four concrete items inline, with IMP-008 cited as provenance |
| F-4 | minor | Cross-skill coherence checked only at final phase — backtracking risk if prose misaligns | Tolerated consciously; phases are small, the cost of forward checks exceeds the cost of a late prose fix. Acknowledge in plan notes |
| F-5 | nit | PHASE-02 EN-1 rationale is circular — claims brief shape 'confirmed against D3' by having PHASE-01 done, but D3 is the design, not the skill | Rewrite to the honest reason plan.md already states: cross-reference the written reconcile skill |

**Ordered penance:** Apply F-1, F-2, F-3, and F-5 to the plan before first
execution. F-4 is tolerated — the Inquisitor accepts the risk calculus. Once the
plan is corrected, it shall stand approved in the sight of this Tribunal.

*Quod erat demonstrandum. The taint is lifted; the plan may proceed to
execution — but let no phase begin before these corrections are inscribed.*

> **HERESIS URITOR; DOCTRINA MANET**
