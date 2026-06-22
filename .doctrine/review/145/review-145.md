# Review RV-145 — reconciliation of SL-137

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Audit of SL-137 (corpus-level relation query verb). Probes conformance
to the design contract (D1–D6), the pure/imperative split (design §5.1),
the diagnostics policy (design §5.4 F1), and behaviour preservation.
Target surface: `review/137` (PHASE-01 engine + PHASE-02 CLI shell).

## Synthesis

Three findings raised, all minor/nit — no blockers.

**F-1 (minor)**: The VT-3 four-axis AND-composition test omitted
`--unresolved` + `include_memory: true`. The implementation logic is
correct (all five filter gates are AND'd by sequenced `filter()` closures).
Fixed: rewrite with a catalog containing both resolved and unresolved
edges, and assert all four axes + include_memory narrow to the single
unresolved row.

**F-2 (minor)**: The VT-4 F4a gate-interaction test used a catalog with
no memory edges, so the empty result proved only that `source-kind: MEM`
match failed, not that the `include_memory` gate fires first. Fixed:
inject a `Raw`-label edge directly and assert both the gate-default
exclusion and the flag-admission with `--source-kind MEM`.

**F-3 (nit)**: Stale `expect(dead_code)` lint from PHASE-01's
"built before consumer" phase — removed now that PHASE-02 integrates
the callers.

All three findings verified. No design or governance changes were needed
— every `fix-now` was a code-only change within the declared file set.

## Reconciliation Brief

(empty — no spec/governance findings surfaced; all fixes were code-local)

- _Per-slice (direct edit)_: none
- _Governance/spec (REV)_: none

## Reconciliation Outcome

No-op reconcile — every finding was a code-level fix-now, applied to `review/137`
and verified as terminal. No spec/governance changes needed. Reconcile pass
complete — handoff to /close.
