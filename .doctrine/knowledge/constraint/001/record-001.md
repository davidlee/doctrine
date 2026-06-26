# CON-001: Association ≠ gating — semantic shapes must not force actionability gates

The central requirement from RFC-008. One must be able to associate an epistemic
record wherever it is *semantically* sensible without that association forcing an
*insensible actionability* effect.

## Why it matters

Without this constraint, every `shapes` edge from an unsettled record would gate
its target, coupling association to actionability. The S2 scenario from RFC-008:

> A proposed decision bears on a risk semantically, but must not block it.
> `DEC-005(proposed) ──shapes──▷ RSK-004[open]` → RSK-004 must remain open, not BLOCKED.

Blanket projection (M-P) and kind-filtered projection (M-Pk) both fail S2:
M-P gates everything, M-Pk gates anything with a "work kind" target — but gate-ness
is an intent, not a property of the target's kind.

## What remains

The live options (M-Pr and M-E) both satisfy this constraint by placing gate-intent
in the author's hands per edge. M-Pr does it via a `{gates, informs}` role on
`shapes`; M-E does it via an orthogonal `gates` axis separate from `shapes`.

## References

- RFC-008 § The requirement (the thing to protect), § S2
- QUE-001 — the D-a fork this constraint gates
- SPEC-019 — epistemic record spec (the consumer)
- SL-158 — parked implementation slice
