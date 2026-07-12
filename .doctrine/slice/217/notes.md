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

## PHASE-02 — pre-implementation decisions (recorded 2026-07-12, code not yet written)

- **Admission split (design-internal reconciliation, flag at audit):** the D13
  `guaranteed_yield > 0` admission filter applies to yield-motivated candidate
  sources (comparison, median-probe). Anchor-review candidates admit on
  suspect EXISTENCE. Warrant: D15 pins "anchor-review can admit while every
  top-K pair is determined" and "a live stale-anchor suspect stays Candidates
  — standing evidence-debt"; a blanket filter would drop the typical
  single-path suspect (complete cited-closure retirement reactivates nothing —
  uphold yield is 0 unless closures overlap) and contradict both D15 and eval
  C2's warrant. Zero-yield suspects rank by the same score formula (sink to
  bottom, never vanish).
- **ElicitInputs is plain pure data** — { active rows, anchors, frontier
  (ranked ids), costing map (multiplier, est_cost, bare_estimate flag),
  projection }. Building costing from PriorityGraph is PHASE-03 shell work.
- **assemble() compiles baseline + reachability internally** (single source,
  evidence-sized).
- **confirm_boost predicate:** both sides `human == 0 && agent ≥ 1` — zero-
  evidence items are not "agent-calibrated" (median-probe subjects must not
  claim the boost).
- **Impact rank map:** class → best frontier rank among top-K members;
  `w(r) = 1/(1 + ELICIT_RANK_DECAY·r)`, r 0-based; classes outside top-K get
  r = depth (documented, not design-pinned).
- **Suspect anchor identity:** every distinct anchored ENTITY appearing in an
  AnchorConflict pair (C4 pair tokens are class ids — resolve to anchored
  members via input AnchorMap + cs.classes). RowsRetired = complete row-uid
  set whose quarantine entry cites that entity.
- **Classless frontier item** (zero rows): PairSide { class: own entity id,
  bounds unbounded, no anchor } — Free coupling; compile mints the class under
  any hypothetical.

## PHASE-02 — implementation findings (code written, NOT yet gated/committed)

Landed `src/priority/elicit.rs` (`assemble` + queue model), `[priority.elicit]`
config (consts `ELICIT_DEPTH/LIMIT/RANK_DECAY/CONFIRM_BOOST` + `ElicitConfig`
parse/clamp), retired the `comparison/query.rs` staged `dead_code` gate
(`pub(crate) use query::*;`), gated `hypothetical_yield` + `indeterminate_pairs`
per-item (elicit uses `hypothetical_outcome` + per-pair weights instead), and
gated the whole `elicit` module staged-ahead of its PHASE-03 command consumer.
15 tests green (11 elicit + 4 config). Gate + full suite NOT yet run.

**Design deviations / in-phase readings to surface at audit:**

- **Source partition (in-phase call, NOT design-pinned):** comparison source
  enumerates indeterminate pairs among *constrained* pool items only;
  *un-constrained* top-K items are owned by median-probe (one probe each). The
  design lists the three sources without a dedup rule; this partition stops a
  brand-new (zero-row, no-anchor) item flooding K comparison pairs — honours
  D14's calibration intent. Recorded in `elicit.rs` module doc.
- **Admission split** already recorded above (comparison/median-probe gate on
  `guaranteed_yield > 0`; anchor-review admits on existence). Implemented:
  anchor-review entries always emit, `score = max(gy,0)·impact` so zero/negative
  suspects sink but never vanish.
- **`ElicitInputs` carries `rank_decay` + `confirm_boost`** (the D13 numeric
  shapes) as pure config inputs alongside `depth` (from `DecisionContext`). The
  PHASE-03 shell fills them from `cfg.elicit`.
- **Costing is an input, not computed here:** `ItemCosting { multiplier,
  est_cost, bare_estimate }` per entity id. PHASE-03 builds it from
  `PriorityGraph` (`m = coeff.value × kind_weight × tag_term`, D6). Keeps
  `assemble` pure/graph-free.

**Behaviour findings (pinned by tests):**

- **RowsRetired can go NEGATIVE when it retires an anchored entity's ONLY
  rows.** In the A(1)>B>C(3) conflict, uphold (retire the complete cited
  closure {j0,j1}) leaves A and C with no row evidence, so `compile` drops
  their anchors (anchor-without-rows ⇒ no class, `compile.rs` C1) and the
  anchored (A,C) pair *reopens* → uphold yield −1, not 0. `guaranteed_yield =
  min(revise +2, uphold −1) = −1`. This is the honest D10 negative-delta; the
  test (`anchor_review_min_over_resolving_uphold_below_removal`) pins it. Any
  future "uphold keeps the anchor live" refinement changes this number.
- **Zero-yield comparison bridge is real and self-demonstrating:** with
  differing per-pair costs over a value interval spanning zero (T>A,B>L, L<0),
  the objective `2·v_A − v_B` stays sign-mixed under *every* order-bearing
  answer, so guaranteed yield is 0 → admission drops it → `Stalled`, never
  `Stable` (one test covers both admission-drop and the D15 stall/stable split).
- **Two suspects per conflict:** an `AnchorConflict` always names two anchors,
  so a single stale-anchor conflict yields TWO anchor-review candidates (both
  suspects surfaced; the model can't tell which is stale — D12 accepts this).

**Gate outcome (T8):** `doctrine check gate` exit 0 — clippy zero-warnings,
full workspace suite green (3433 + all pre-existing priority/comparison suites,
UNCHANGED — behaviour-preservation holds; no pre-existing test file touched).
Gate surfaced six pedantic/style fixes in the new `elicit.rs`, all
behaviour-neutral: `assemble(ctx: DecisionContext)` now by-value (Copy 8-byte,
`trivially_copy_pass_by_ref`); `map_or` over `map().unwrap_or`; a `PairSide`
doc backtick; and two `#[expect(clippy::integer_division)]` (pair-count `/2`,
median index) + one `#[expect(clippy::too_many_arguments)]` on the private
`build_comparison` helper (repo-sanctioned idiom, cf. governance.rs/spec.rs).
**Design-note for audit:** `rank_map` + `depth` thread through four assembler
fns as one impact-band context — a bundling opportunity deferred to keep the
finish-line behaviour-neutral (would reshape four call sites).
