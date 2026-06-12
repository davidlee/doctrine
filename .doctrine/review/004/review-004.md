# Review RV-004 — reconciliation of SL-044

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of SL-044 (reconcile writer + closure gate, SPEC-002 B). All
three phases `completed`; lifecycle `started`. Findings originate from an
external adversarial code review (Opus reviewer, verdict *revision-required*),
each independently re-verified against the committed source before being raised
here — citations point at the confirmed line, not the reviewer's claim.

**The invariant under test (NF-001 / REQ-114).** Observed coverage must never
reach authored status *through a function*. This slice is where authored truth is
finally written, so the audit holds it to the design's own layered wall (§5.6,
D-B7): (1) signature isolation of the status-select fn, (2) the drift `Verdict`
consumed only by the prompt builder, (3) a **behavioural** verdict-independence
test on the residual write call site. Lines of attack:

- **Does the proof touch the write path?** The wall is only as real as the test
  that exercises `run()`. A verdict-independence test that never calls `run()`
  proves a tautology, not the invariant. (→ F1)
- **Does the structural argument overclaim?** A signature that "the compiler
  proves" only constrains the function body, not the call site that feeds it.
  (→ F2)
- **Close-gate soundness.** The discharge predicate and drift gate consume the
  *whole authored REC corpus* (hand-editable), not just writer output — so any
  "the writer only emits one-delta RECs" assumption is latent, not enforced.
  (→ F3)
- **Atomicity vs NF-003.** "Exactly one atomic REC" must hold for the *act*
  (status write + ledger entry), not just the REC file in isolation. (→ F5)
- **Project doctrine.** No parallel implementation (CLAUDE.md); tests that assert
  nothing; stringly-typed seam logic where a typed classifier exists. (→ F4/F6/F7)

**Out of scope / not re-decided here:** the governing decisions (SPEC-002 D7–D9,
ADR-003, ADR-009 FSM + closure seam). Tensions route back through `/consult`,
not this ledger.
