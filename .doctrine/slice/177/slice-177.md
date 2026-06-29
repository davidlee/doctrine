# Default value floor for valueless actionable kinds

## Context

SL-176's `fulfils` priority **value-burndown** (objective 3) reduces a backlog
item's value by the lifecycle-gated raw `value` of the slices fulfilling it. The
subtraction is **value-denominated** — it only means anything for entities that
carry a value. A *valueless* actionable entity (no explicit `value` facet) scores
its value dimension at zero today (`src/priority/graph.rs:115`, the `None` branch),
so burndown has nothing to denominate against and the item is invisible to the
undelivered-value signal.

SL-176 calls out the fix as a **separate sibling slice, soft dependency**
(D-value-floor-sibling, user-locked 2026-06-29): give value-bearing actionable
kinds a **default value of 1.0** when none is authored, so burndown works
uniformly. SL-176 lands and its suites stay green on explicitly-valued entities
without this slice; this slice closes the valueless gap.

`references(concerns)` SL-176 — this is its soft prerequisite for the valueless case.

## Scope & Objectives

1. **Default value = 1.0** for value-bearing **actionable kinds {slice, backlog}**
   when no `value` facet is authored. The floor is applied where per-entity value
   feeds priority scoring (`src/priority/graph.rs` value-dim, the `None` branch),
   not by mutating authored TOML.
2. **Knowledge records excluded.** SL-158 trinary actionability: records are
   gating / estimable but **not value-bearing**. The floor must not apply to them.
3. Burndown (SL-176) then denominates correctly for previously-valueless items —
   their floored 1.0 is the value SL-176's post-pass subtracts against.

## Non-Goals

- **The burndown post-pass itself** — owned by SL-176. This slice only supplies
  the floor it consumes.
- **`fulfils` coverage-% derived display** — deferred follow-up, carried open in
  SL-176's ledger. Not here.
- **Estimate/cost defaults** — only the *value* facet floor is in scope; the cost
  denominator and estimate anchoring are untouched (SL-172 territory).
- **Governance ratification** — if a default-value rule needs canon, it rides
  reconciliation, not design/plan (mirrors SL-176's posture).

## Affected surface

- `src/priority/graph.rs` — value-dimension scoring; the `None`-value branch is
  where the floor lands.
- `src/value.rs` — value facet model (if the default belongs nearer the type).
- `src/priority/config.rs` — only if the floor is made config-tunable (open
  question for `/design`).

## Risks / Assumptions / Open questions

- **OQ-1** Is the floor a hard constant (1.0) or a config coefficient? SL-176
  fixes the *value* at 1.0; tunability is a `/design` call.
- **OQ-2** Kind-membership source: how does scoring know an entity is a
  value-bearing actionable kind {slice, backlog} vs an excluded record? Confirm
  the SL-158 actionability classification is reachable at the graph.rs site.
- **A-1** Applying the floor at the scoring seam (not in authored TOML) is
  correct — keeps storage honest, no derived data written to prose/TOML.
- **R-1** Coupling to SL-176: soft. If SL-176's burndown denomination changes
  shape before this lands, re-check the `None`-branch contract.

## Verification / closure intent

- A valueless slice/backlog item scores its value dimension as if `value = 1.0`.
- A valueless knowledge record is **unaffected** (still no value dimension).
- An explicitly-valued entity is **unchanged** (floor only fills the `None` case).
- SL-176's existing suites stay green (behaviour-preservation on the shared
  scoring path).

## Follow-Ups

- `fulfils` coverage-% derived display (deferred from SL-176).
