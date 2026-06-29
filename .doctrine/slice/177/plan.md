# Implementation Plan SL-177: Default value for valueless value-bearing kinds

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two phases, split along the **two consumers of the shared seam** (design §5.1,
RV-191 F-1). PHASE-01 builds the seam and wires the scoring-side consumer
(`base_score`); PHASE-02 wires the burndown-side consumer (SL-176's `raw_value`).
The split is not cosmetic: PHASE-02 *modifies SL-176's code* and so gates on
SL-176 being present, while PHASE-01 stands entirely on current `base_score`.

## Sequencing & Rationale

**Whole-slice gate: `SL-177 needs SL-176`.** Both phases run after SL-176 lands.
SL-176 is in dispatch now; it removes the old `slices`→optionality credit, adds
the burndown post-pass, and re-baselines its own goldens. Letting it land first
means PHASE-01's golden re-baseline starts from SL-176's post-state — no churn
conflict on the shared `e2e_priority_*` goldens.

**PHASE-01 — the scoring change.** Leaf-up: `kinds::VALUE_BEARING` +
`is_value_bearing` first (the named set, distinct from `is_work_like` which
includes REV — F-3), then the priority-tier `effective_raw_value` accessor +
`DEFAULT_VALUE` (cohesion home — F-4), then `base_score` routes through it, then
`surface.rs` consumes the same predicate. TDD: VT-2/VT-3/VT-4 drive the default,
exclusion, and no-clamp behaviours red→green. The deliberate red is the broad
fixture set (design §9.1) — base_score units, graph.rs score-consequence tests,
and **both** e2e goldens. **Grep the full set before touching code** (F-2): every
valueless SL/backlog fixture asserting `value_dim`/`score == 0` is re-baselined;
VA-1 is the conscious confirmation those golden diffs are the intended change, not
regressions. The behaviour-preservation gate (design §9.2) covers only unrelated
behaviour — the `surface` view (set-preserving) and the non-sentinel status of
`value_dim == 0`.

**PHASE-02 — the burndown wiring.** One real change: SL-176's `raw_value(src)` /
`raw_value(I)` (in `src/priority/graph.rs`) call `effective_raw_value(..)`. This
is the line that makes the default *matter* — without it the whole slice is inert
to burndown (the F-1 cardinal correction). VT-1 is the regression guard: a
valueless slice in a delivering status burns down a fulfilled item; it fails if
the retrofit is missed and `raw_value` still reads `f.value` raw. INV-4 (one seam,
no raw `f.value` read for scoring/burndown) is the invariant PHASE-02 closes.

## Notes

- **No code before SL-176 lands.** If asked to start while SL-176 is still
  dispatching, stop — PHASE-02 has nothing to retrofit and PHASE-01 risks golden
  churn against SL-176's in-flight changes.
- **Tunability deferred (OQ-1).** `DEFAULT_VALUE` is a hard const; a later swap to
  a `PriorityConfig` field is local to the accessor — not planned here.
- **IMP-211** carries the render legibility follow-up (RV-191 F-5); out of plan.
- **VT-3 REV-seed risk (phase-plan to confirm).** VT-3 proves REV exclusion at the
  scoring seam, which assumes a REV both reaches `base_score` and is seedable in
  the graph.rs test harness (no `seed_revision` helper exists today). At
  phase-plan: confirm REV flows through `base_score`; if it does, build the seed
  helper; if it does not, VT-3 collapses to the VT-1 set canary plus a record
  (ASM) runtime case — adjust the mandate then, do not pre-commit code to a path
  that may not exist.
