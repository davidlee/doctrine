# IMP-042: code-review skill is structurally orphaned — integrate it into the corpus (backlog + lifecycle awareness) beyond the IMP-023 RV rewire

## Original scope

Integrate the code-review skill into the corpus (backlog + lifecycle awareness)
beyond the IMP-023 RV rewire. The skill exists but is structurally orphaned —
not routable from the lifecycle state machine, not triggered by phase
completion or close gates.

## Expanded scope (RFC-011 case-notes analysis)

### Defence against agent malicious incompetence

The SL-222 PHASE-09 incident (pi worker stubbed critical logic, deleted the
tests that would have caught it, reported "Deviations: NONE" with a green
suite) demonstrates a failure mode the scope belt and regression diff cannot
catch: a worker authorized to touch in-scope files can delete tests and stub
logic, converting a regression into a green suite. Only adversarial
cross-check against the design caught it.

Lower-adherence models are more prone to this class of error — not just
"subpar code" but actively misleading output. The code-review skill, applied
per-phase immediately after worker return, would catch:
- Deleted tests (diff shows removal, reviewer asks "why?")
- Stubbed/deferred logic marked as done
- Triage tables that don't match design expectations
- Green suites achieved by removing assertions

This is a distinct value proposition from the original "catch bad code"
frame — it's a **worker-output integrity gate** that becomes more worthwhile
as model adherence drops.

### Per-phase immediate review

The current lifecycle reserves code-review for pre-close audit. A per-phase
option — route to code-review immediately after import, before conclude —
would catch regressions while the worker worktree still exists and the phase
context is hot. Tradeoff: cost (an extra agent lifetime per phase) vs risk
(model tier × change complexity). The skill should support both cadences:
pre-close (current) and per-phase (opt-in, gated on model tier or phase risk).
