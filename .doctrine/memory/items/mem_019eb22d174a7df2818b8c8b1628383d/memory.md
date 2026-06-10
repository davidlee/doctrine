# Interaction bugs hide between individually-sound parts — review the seams, not just the components

Across SL-036 (cordage) the design went through four adversarial review rounds.
**Every blocker that mattered was an interaction bug between two individually-sound
parts** — not a defect in any one component:

- **F11 / F45** — per-layer lexicographic `order_key` vs level-equality
  (level equality ≠ incomparability); both self-found while integrating *adjacent*
  machinery, neither external reviewer caught them.
- **F30 / F46** — the pass-1 arity eviction silently destroyed authored Reject
  cycles before pass-2 detection saw them (pass-pipeline × cycle-detection).
- **F31 / F32 / F33** — Degraded/taint mis-scoped three ways (degradation ×
  ordering): non-spec seeds, intra-SCC exclusion ambiguity, suffix-by-NodeId
  violating surviving U edges.

Per-section / per-component review **missed these three times**. They surfaced only
when (a) integrating a fix re-touched the adjacent machinery — *integrating a fix is
itself a review pass on adjacent code* — or (b) an external reviewer traced a
cross-pass data flow end to end.

## Recipe for /design, /inquisition, /audit

- Validate each component in isolation, then **explicitly walk the seams / data flow
  between sound components** as a distinct pass — the contract each pass hands the
  next (pass pipeline, overlay × ordering, degradation × traversal).
- Budget a dedicated **interaction / integration-contract** review pass; do not
  declare a round "diminishing" on per-section cleanliness alone while the
  cross-component contracts are unwalked.
- When you integrate a fix, re-review the adjacent machinery it touches — the fix is
  a fresh review pass, not just an edit.

Learned on a multi-pass / multi-overlay subsystem (cordage build pipeline +
order composition); the lesson generalises to any layered/pipelined subsystem.
Cordage-internal residual risks live in the slice's `audit.md` and RSK-001 / RSK-002,
not here — this is the reusable *method*.
