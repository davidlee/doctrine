# Notes SL-217: Elicitation queue

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01 — Query predicates (completed 2026-07-12)

Landed `src/comparison/query.rs` (commit ab432d62); 22-test battery, VT-1/2/3
pass `verify-vt`. `compile.rs` untouched — `ConstraintSet` pub fields sufficed,
so behaviour preservation holds by construction.

**Design deviations to surface at audit (recorded in-phase, rationale in the
phase sheet and commit):**

- `hypothetical_outcome(...) -> { newly_determined, no_longer_determined }`
  added beneath the design-sketch `hypothetical_yield -> i64` (now a thin
  wrapper). D13's `guaranteed_impact` needs the newly-determined PAIR SETS
  (rank-decay over pairs), not a bare count — without this, PHASE-02 would
  re-recompile per answer and break the §2 cost bound.
- Baseline `&Reachability` is a parameter (design sketch recompiled nothing
  for "before"): the caller holds the memoised baseline, keeping ≤3 recompiles
  per candidate honest.
- Hypothetical remap keys on `PairSide.class` (class id = smallest member
  entity id). Under a C2-split hypothetical, an anchor hoisted from a
  non-representative member correctly leaves the representative's fragment —
  the battery pins this. **PHASE-02 note:** if per-ITEM fidelity under class
  splits ever matters (two frontier items sharing a class), `PairSide` needs
  the item entity; with class-level pairs (current design reading) it does not.

**Implementation findings:**

- Corner enumeration needs a representative in-region coordinate (0.0) for a
  side with no finite endpoint — rays report growth but not the OTHER side's
  endpoint values when its weight is zero (RV-269 F-4 shape caught this red).
- A fully unbounded coupling-boundary segment contributes a constant sample
  (`f(0,0)`) that no ray reports when `w_A = w_B` — the equal-weights
  chain-pair case caught this during derivation.
- The coupling-boundary golden (`coupling_boundary_infimum_golden`) was
  mutation-checked: disabling the boundary-clip block flips it red
  (corner-only enumeration reports PositiveOnly for the Mixed case).
- R3 trap is structural, not just conventional: the hypothetical path augments
  the post-resolve ACTIVE set, so resolution never sees synthetic rows; the
  sentinel uid (`synthetic-hypothetical:`) additionally rules out uid
  collision/supersession. Test pins both.
- `query` is NOT re-exported from `comparison/mod.rs` yet — staged-ahead
  module-level `cfg_attr(not(test), expect(dead_code))`; PHASE-02 adds
  `pub(crate) use query::*;` and retires the gate.
