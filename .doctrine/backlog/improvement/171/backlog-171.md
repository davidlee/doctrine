# IMP-171: Symmetric ledger+registry derive on codex/pi dispatch arm

Q-remainder follow-up to SL-154 (P-as-Q-foundation). SL-154 closes the
conformance-registry population leaks (ISS-051 solo, ISS-052 funnel) on the
**claude** arm via: prepare-review derives the registry from the dispatch
**ledger** (enforced beat), with a `check_completeness` gate as the pre-audit
safety net both arms. The codex/pi arm has **no dispatch ledger** (its step-8
`slice record-delta` writes the registry directly), so there is nothing to
derive from — it stays gate+manual-`record-delta`, not symmetric auto-derive.

This item: give codex/pi a unified dispatch-ledger write at the funnel Record
beat so prepare-review derives the registry **identically on both arms** (gate
remedy: manual → auto). Deferred from SL-154 because `plan_phases`
(`src/dispatch.rs`, unconditional on the ledger) couples a codex/pi ledger to
**codex/pi `phase/<N>` ref projection** turning on — a real, untested behaviour
change to codex/pi dispatch output that must land + be tested as its own
deliverable, not as an accidental side-effect of registry work. The SL-154
reproduction (SL-153) is claude-arm, so the codex/pi half was never validated by
the motivating case.

Relates: SL-154, ISS-052, RFC-004; codex/pi phase-refs touch RFC-005 territory.
