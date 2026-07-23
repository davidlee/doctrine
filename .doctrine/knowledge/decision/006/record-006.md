# DEC-006: Code-review cadence — model-tier gate + tripwires, arm-agnostic

Per-phase code review defaults on when the worker model sits below a stated
adherence bar, regardless of dispatch arm — the arm is a transport property,
not an adherence property (the pi arm can run any model). Tripwires escalate to
mandatory at any tier: deleted tests in the import diff, "Deviations: NONE"
beside design-relevant divergence, waived/uncheckable VT, out-of-scope touches
(SL-222 PHASE-09 incident class). Gate lives in skill prose, not engine gating.
Rationale: SL-215 `design.md` D3.
