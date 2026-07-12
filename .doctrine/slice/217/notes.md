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

## PHASE-03 — T1/T2 walking skeleton (in_progress, committed)

Landed the `compare elicit` arm + the input-assembly shell (design §3), green +
gated. Remaining: full render/JSON fidelity (T3/T4 goldens), `--kind`/`--limit`
golden (T5), determinism/capture-loop/cost-ceiling e2e (T6–T8), VA-1/VH-1.

**Three additive, behaviour-neutral exposures (STOP did NOT fire; VA-1 holds —
3433 pre-existing bin tests green UNCHANGED):**

- `comparison::Pipeline.active_judgements: Vec<Judgement>` + `anchors: AnchorMap`
  — `load_pipeline` computes `active: Vec<&Judgement>` (store.rs:140) borrowing
  the locally-loaded `sessions`, which drop on return (that lifetime is WHY it
  couldn't be exposed by reference). Store OWNED clones; the shell borrows them
  into `assemble`. Required deriving `Clone` on `Judgement` (+ `Clone` on its
  `RaterKind`/`RowForm` fields — **`Clone` only, NOT `Copy`**: `Copy` trips
  `trivially_copy_pass_by_ref` on the pre-existing `&RaterKind`/`&RowForm`
  params).
- `PriorityGraph.cost_ctx: CostCtx` (additive field) + `item_costing(key, cfg)
  -> (multiplier, est_cost, bare_estimate)` read method. `m = coeff.value ×
  kind_weight × tag_term` (D6) is derivable from PUBLIC `cfg` + `NodeAttr.
  {facets,kind}` — no `base_score` change. `tag_term` extracted to a free fn so
  `base_score` + `item_costing` share ONE definition (no parallel impl). Bare
  `est_cost` reuses the build-time `cost_ctx` anchor (identical to the base
  pre-pass).
- `assemble` recompiles its ConstraintSet internally from `active` — it does NOT
  consume `Pipeline.constraint_set` (self-contained pure; shell supplies raw
  `active`).

**Shell shape:** `run_elicit` = scan→`graph::build` + `load_comparison_pipeline_
for_root` → `build_elicit_inputs` (frontier top-K by final score, id-lex
tiebreak; costing over all scored entities; active/anchors/projection off the
pipeline) → `assemble(Sequencing{depth})` → human/JSON render. `compare elicit`
is READ-classed in `guard.rs` (D18). Verified end-to-end on the live ledger.

**Deferred to T3/T4 (marked in `compare.rs` doc comments):** JSON participant S3
value shapes + structural bounds + annotations + anchor `exits`; human render
fetched-context/mask/reasons prose + full D15 footer wording (stall/stable/
outsider/m=0). Current render is a spartan-but-real spine; T3/T4 ENRICH (additive
fields), not rewrite.

## PHASE-03 — T3 JSON schema-v1 fidelity (in_progress, committed)

Enriched the `--json` envelope to design §3/D16 exactly. STOP did NOT fire — the
handover's readiness question (every §3 field has a source) resolved cleanly:

- **Participant `value` block** — shell join `Participant.id → value source`.
  `provenance`+`point` map from `surface::value_source_reason` (the SINGLE
  authored>projected>gauge precedence, now `pub(crate)`; body untouched so
  `explain` goldens hold). Structural `bounds {kind: open|closed|unbounded,
  value?}` from a NEW `surface::class_bounds_structural` (reads
  `constraint_set.bounds[class]` — the `Bound` enum survives here; the human
  `explain` path deliberately flattens via the pre-existing lossy `class_bounds`,
  which stays). Web-review requirement: `[null, 2.8]` erases open/closed, so this
  surface keeps the structural form. `bounds` omitted when the entity has no
  compiled class (authored floor, no interval).
- **`estimate` field — DESIGN CALL, flag at audit.** §3 draws a SCALAR (`3.5`) +
  null-when-bare, and D7 says "costs are the scalar `est_cost`". So
  `estimate = bare ? null : est_cost` (from `graph.item_costing`), NOT the raw
  `{lower,upper}` facet. The value/estimate asymmetry is intentional: value
  bounds are decision-relevant (structural, web review); estimate is disclosure
  context (scalar). Bare disclosed by the D17 mask annotation, never a
  synthesized number. Verified live: IMP-202 authored point 2.8 no-bounds (no
  class) + est 1.15; IMP-255 authored closed/closed@2.5 + est 1.155.
- **Kind-specific ask (schema fidelity).** Split the shared `ask_json`:
  comparison ask carries `frame`(equal-effort)/`domain`(value) constants (D1),
  NO `yield_note`/`exits`; anchor ask carries `yield_note` + `exits` (per-answer
  suggested-action arrays keyed by the answer tokens, now `pub(crate)` on
  `elicit` — STD-001 one definition). This fixes the T2 spine emitting
  `yield_note: null` on comparison entries.
- **Testability refactor:** `render_elicit_json` split into pure
  `elicit_envelope(...) -> Value` (byte-stable anchor) + thin writer (no trailing
  newline, `render.rs::finish` contract). `RenderCtx` carries graph+pipeline+cfg
  + a canonical→`EntityKey` map, built once from the same load (no second scan).

**Tests:** 5 pure shaper goldens in `compare.rs` (`bound_json`,
`value_block_json` projected+classless, `anchor_exits_json` populated+empty,
`comparison_ask_json` frame/no-anchor-fields). Full both-kind ENVELOPE goldens
(VT-1 keywords `yield_note`/`median`/`stall`) live in `tests/e2e_compare_elicit.rs`
by `plan.toml` VT-1 `test_file` contract — authored in T6–T8. Priority (180) +
inference e2e (5) green unchanged (behaviour-preservation).

## PHASE-03 — T4 human render fidelity (in_progress, committed)

Enriched `render_elicit_human` to the design §3 structured idiom. Per entry:
spine (rank/kind/score/yield/impact) → ask line → participants with fetched
context (title + status from `NodeAttr`, S3 value shape via the reused
`render::value_source_fragment` — now `pub(crate)`, the SINGLE value-shape
template shared with `explain`, estimate cell or `est —`) → reasons prose →
the exact `answer:` command (comparison → `compare record` over the pair;
anchor → revise/uphold, mirroring the JSON `exits`). Mask ⚠ line rides the
participant annotations (single source — the engine's bare flag, not
re-derived).

`state_footer` replaces the T2 skeleton `state_line` with the full D15 wording:
stall names the depth + disclaims stability; **stable** claims value_dim order
among the CURRENT top-K members and explicitly says "not membership"
(D5/D15 — the outsider case needs no separate branch, the member-scoping IS the
disclaimer); scoped by `excluded_value_insensitive` when m=0 pairs exist (D6).
Takes the whole `ElicitQueue` (not just state) for the exclusion count.

**House idiom:** human builders return `Vec<String>` + `.concat()` (mirrors
`render::explain_human`) — the repo clippy denies BOTH `push_str(&format!())`
(`format_push_string`) and `let _ = writeln!()` (`let_underscore_must_use`), so
`push(format!())` into a Vec is the sanctioned append.

**Tests:** `state_footer_names_depth_disclaims_and_scopes_m0` (3 states + m=0
scope), `answer_command_per_kind` (both kinds). Full render byte-goldens (3
states + mask present/absent) are VT-1 e2e (T6–T8). Clippy 0, 33 module tests
green.

## PHASE-03 — T5/T6–T8 e2e goldens (in_progress, committed)

T5 filter code already shipped in T1/T2 (`displayed()` — post-ranking `--kind`
then `--limit` display cap over the fully-ranked pool); its golden lives in the
e2e file. Authored `tests/e2e_compare_elicit.rs` (10 tests, mirrors
`e2e_compare_inference.rs` — black-box CLI, hand-authored `capture()` sessions
since `compare record` is WRITE-refused in-jail; `compare elicit` is READ so runs
in-jail). All the VT keyword contracts land here (verify-vt reads the
`test_file`):

- **VT-1** — `json_anchor_review_carries_yield_note_exits_and_suspects`
  (`yield_note` + exits + both suspects, the `A(1)>B>C(3)` conflict);
  `json_median_probe_surfaces_for_unconstrained_item` (`median`-probe reason +
  bare mask present); `json_comparison_carries_value_bounds_and_estimate_mask_split`
  (structural bounds + the estimate present/absent + mask absent/present split,
  via a frontier-pair corpus — one estimated pair, one bare);
  `render_stall_names_depth_and_disclaims_stability` (`stall`, the zero-yield
  bridge `T(5)>A,B>L(-5)` with differing A/B costs);
  `render_stable_is_member_scoped`; `kind_filter_and_limit_cap_the_view`.
- **VT-2** — `shuffled_load_order_yields_byte_identical_queue_and_json` (human +
  `--json` byte-identical across 3 fixed permutations).
- **VT-3** — `capture_loop_round_trip_consumes_the_answered_pair` (median-probe
  subject → hand-author the answer row → the now-constrained item drops its
  probe on refresh).
- **VT-4** — `cost_ceiling_eval_corpus_completes` — **DEVIATION (flag at
  audit):** the 32-row/K=8 frozen snapshot is GENERATED deterministically in-test
  (`seed_eval_corpus`, ISS-100‥131 chain) rather than committed as static files
  under `tests/fixtures/elicit_eval_corpus/`. Equivalent for the completion
  assertion (frozen by code, never the live ledger) and far lighter than 60+
  committed TOMLs; the sheet's "commit fixtures/" is the only unmet letter. Flag
  for the User at audit — trivially convertible to committed files if preferred.

**Fixture-shaping facts (durable):** authored `[value]` facets ARE anchors
(`comparison_anchor_map`); an estimate facet perturbs `est_cost` enough to flip a
median-probe pair to a zero-yield `Stalled` — so the estimated-participant golden
uses a FRONTIER-PAIR (two constrained-but-mutually-indeterminate items), not a
median probe. `doctrine check gate` exit 0; full workspace suite green.

## PHASE-03 — T9/T10 behaviour preservation + finish

**VA-1 (EX-3) evidence — behaviour-preservation holds:**
- `git diff c9818d71..HEAD -- src/comparison/compile.rs src/comparison/project.rs`
  is EMPTY — the propagation engine is untouched (not even accessors). The only
  comparison-tier change is `store.rs` (+16, the T2 owned-`active_judgements` /
  `anchors` exposures) + `wire.rs` (+6, the T2 `Clone` derives).
- surface/render changes are additive read accessors + visibility only
  (`value_source_reason` body UNCHANGED → explain goldens hold;
  `class_bounds_structural` new; `value_source_fragment` `pub(crate)`).
- No pre-existing test file modified: `e2e_compare_inference.rs` /
  `e2e_priority_golden.rs` diff EMPTY; grep of all `src/**` deletions for
  `#[test]` / `fn *test` / `mod tests` is EMPTY (no pre-existing test body
  removed — all test changes are additive).
- Full workspace suite green with no ledger / no invocation (`doctrine check
  gate` exit 0).

**T10:** `doctrine check gate` exit 0 (clippy 0 + full test + fmt). PHASE-03
flipped `completed`. `verify-vt 217` VT-1..4 attributed PASS once the phase
completed (before completion they read `UNATTRIBUTABLE` — the conformance range
`[code_start, code_end]` is only closed by the flip). **VH-1 (human dogfood) is
NOT agent-closable** — flagged for the User: run `doctrine compare elicit` over
this repo's live backlog and confirm the queue renders answerable,
reasons-attached questions with sane state wording. Verified live during dev
(the queue rendered median-probe comparisons with structural value blocks +
answer commands + the stable/candidates footer), but the acceptance is the
User's.
