# SL-219 design — Estimate comparison domain

Status: drafted 2026-07-13, post-clarification loop + section-by-section
external review (codex GPT-5.5, five passes); governs implementation.
Governing contracts: RFC-019 (resolved; § domains/frames table), SL-213 design
(constraint layer — machinery reused via parameterization), SL-217 design
(elicit queue — probe kind rides D12/D14 precedents), ADR-001 (layering),
ADR-015 (priority scoring; amended by this slice's REV), STD-001, STD-002.
Product decisions Q1–Q4 locked with the user at clarification (2026-07-13).
IDE-039 (magnitude claims as ledgered evidence) captured during this design —
the anchor seam is kept honest for it (§2), never built here.

## Decision ledger

| id | decision | rationale (compressed) |
|---|---|---|
| D1 | Est-domain latent = the **operative scalar cost** per class — the quantity scoring divides by — NOT the authored range or its uncertainty (Phase E's feasible-region model, RV-260 F-5). Point anchors = β-resolved `est_cost` of authored estimates. Owned consequence: a sizing row can conflict with resolved points even where raw ranges overlap — correct under this latent; the `AnchorConflict` wording says so. Machinery (compile/project, D8 lemma) transfers; *consumption* differs (D2). RFC's "adjusts ranges minimally" superseded → scalar projection + C6 bounds display (deviation recorded) | Evidence is judged against what the engine actually charges. Interval anchors = band constraints — void the D8 lemma, rejected. Stale estimate is the likeliest defect; loudness is a feature (SL-213 D4 posture) |
| D2 | Scoring feed: `est_cost` ladder `authored > projected (non-Gauge) > bare anchor` — **source precedence only, no numeric-dominance claim**. Gauge renders, never divides. Projected may exceed or undercut the bare anchor; INV-2 restates to "bare anchor dominates every *authored* estimate". First-anchor regime flip (component members bare-anchor → projected) is a real scoring discontinuity, owned, regime-flip golden. REV against ADR-015 carries the restatements (Q1=A, Q2=A) | Conventional magnitudes may fill a numerator, never a denominator — value multiplies, cost divides; P8 gauge spread near zero in a divisor explodes `value_dim`. INV-2's intent (unpriced items must not win by default) is preserved: bare-no-evidence keeps the dominating divisor |
| D3 | Admissibility = `VALUE_BEARING + RECORD` derived from kind constants; RSK admitted; REV-kind deferred (trigger: a consumer of revision costing) (Q4=A) | A4's RSK exclusion is value-specific (worth = exposure, `risk_dim`); settling a risk is plain effort. Records: settle-cost is intrinsic (RFC A2). No parallel list — derivation property-tested |
| D4 | Elicit integration = `sizing-probe` candidate kind, existence-admitted (SL-217 D12 precedent), zero yield claim, `yield_basis: "calibration"`; engine yield-ranking of estimate questions stays Phase-E-gated (SL-217 D17 restated) (Q3=C) | Sizing session gets a deterministic driver without touching what Phase E owns; targets the C1 mask payoff **where it is actionable without curator judgement** — un-evidenced bare items. Gauge-masked / sizing-declined items are disclosed residual debt, not probe subjects (§4) |
| D5 | Frame `more-work` ("which is more work?"); `prefer-a` ⇒ edge `c_A > c_B` (winner = costlier); `equal` ⇒ cost-equality merge; `incomparable` ⇒ `NoConstraint` | Winner > loser matches the value-domain compile convention — `compile()` reused with zero semantic change |
| D6 | Pipeline order: est system compiles/projects first; every cost consumer reads **one resolved cost per item** — the post-ladder `est_cost` at the single consumption seam (`graph::est_cost`). Est system never reads values; no cycle. Priority-domain rows stay inert (SL-213 D2 stands); the future priority compiler MUST consume post-ladder resolved costs and ride this ordering (Deferred). Projected-cost movement re-runs value determinacy via the existing D18 reprobe dynamic | The `prefer-first` coupling is not hand-waved: no priority compiler exists, so nothing recompiles on cost movement; the contract for when one does is pinned now |
| D7 | `CostCtx` bare anchor (`max_upper + margin`) computed from authored uppers only — projected costs never move it | No feedback loop through the default |
| D8 | `project()` parameterized: `ProjectionParams { gauge_step, gauge_center }`; **no config reads inside `project`**. Value site passes `VALUE_PROJECTION_PARAMS` (the shipped constants) — byte-identical existing goldens pin the invariant. Estimate passes `(EST_GAUGE_STEP, ctx.absent)` — anchor-free cost components center their render-only gauge on the corpus's own bare anchor. `EST_GAUGE_STEP` named const, default 0.25, `[priority.estimate] gauge_step` seam (STD-001) | Per-call params isolate domains; the value invariant is established by goldens, not intent. Gauge center = the engine's own absent-cost stance, not a new arbitrary constant. P5/P6 use the step inside anchored components and ARE fed — sensitivity sweep extends to the est system |
| D9 | Finding variants gain a `domain` field **set at construction** by the shell that knows which system produced them; compile payloads stay domain-agnostic | "Compile untouched" demonstrated, not asserted |
| D10 | Est-domain anchor-review candidates deferred; value-system anchor-review only this slice. Trigger: eval evidence of est-anchor sterilisation (the C2 analog) | D12's warrant was value-domain eval evidence; none exists for est yet. Findings still surface est conflicts |
| D11 | Positivity is an estimate-domain **axiom**: feasible region = order constraints + point anchors + `c > 0`. All anchors > 0 (`authored_est_cost` floors at EPSILON); P6's clamp is the axiom made concrete (RV-265 F-1's negative-ceiling hazard structurally cannot arise). Phase E: est determinacy must carry the positivity term; D8-lemma re-check pinned in Deferred (augmentation step survives over positive dense reals) | Settle-cost is a nonnegative physical quantity; every consumption seam already floors. Without the axiom, "items below a 0.5 anchor pack into (0, 0.5)" is false (§3) |

## §1 Domain model & constraint semantics

**Frame table** (`wire.rs` `DOMAIN_FRAMES` grows one row): `estimate →
{ more-work }`. Frame implies domain at capture (S1 unchanged); users never
type a domain. Ask phrasing: "which is more work?" — winner is the costlier
item, edge `winner > loser` in cost (D5).

**Anchors — semantics stated (D1).** Est-system `AnchorMap` = `item →
authored_est_cost(bounds, cfg)` for admissible items with authored estimates.
The anchor is the *operative* cost, deliberately stronger than the range:
evidence is judged against what the engine actually charges; a conflict says
"sizing evidence contradicts the β-resolved costs — revise the estimate or
supersede the row". Consequences:

- Anchors move when β or an authored range moves — deterministic; re-runs est
  projection → resolved costs → value determinacy: one reprobe dynamic covers
  both domains (D6).
- Row-gating (mem.fact.comparison.anchor-attachment-row-gated-per-system): an
  authored estimate enters the est system only when a sizing row touches its
  item. No rows ⇒ no system ⇒ ladder falls through to bare anchor. Cold-start
  is a no-op by construction.
- Estimated **records** anchor too — a record with an authored estimate
  contributes anchor mass to chains through it even though nothing scores the
  record. The concrete mechanism behind the RFC's "records included".

**Admissibility.** `admissible_estimate_pair` beside `admissible_value_pair`,
derived from `VALUE_BEARING + RECORD` (D3), no parallel list; refusals name
the currency ("REV-001 has no comparable settle-cost — estimate comparison
admits work and record kinds"). Census property test extends: admitted ⇔
derivation, over `ALL_KINDS`.

**Resolution (R-rules).** Unchanged. `estimate` rows resolve `Active`;
per-domain split happens at compile-input selection, not resolution. R4 keeps
gating only `priority`. The identity key already carries `domain` —
cross-domain rows on one pair never collide or supersede across domains.

**Compilation.** Two independent `compile()` invocations per refresh: value
rows + value anchors; estimate rows + cost anchors. C2–C5 apply per system;
est-domain `AnchorConflict` wording per D1 (likeliest defect: stale estimate —
loudness is a feature).

## §2 Pipeline & module impact

**Pipeline (`comparison/store.rs`).** `Pipeline` refactors to two explicit
per-domain systems: `DomainSystem { constraint_set, anchors, projection }`,
fields `value` / `estimate`; `resolution` stays shared (one resolve pass over
all rows). Existing callers mechanically re-path
(`pipeline.constraint_set → pipeline.value.constraint_set`); the existing
suites green-unchanged are the proof (behaviour preservation, not API
preservation).

**Row-status routing (two systems, one ledger).** A row belongs to exactly
one domain; `CompilationStatus` is assigned by its owning domain's compile.
The two quarantine maps are disjoint by construction; `RowState` joins by row
uid. `compare list`'s status column reads each row's own domain system — no
value bias. `explain`'s value-source block reads `pipeline.value`; the cost-
source block reads `pipeline.estimate`. Elicit anchor-review sources from the
**value** system only (D10).

**Anchor seam (non-foreclosure, IDE-039).** `AnchorMap` remains the sole
anchor input to `compile` for both domains; sourcing is shell-side builders:
value anchors from authored `[value]` facets (unchanged); est anchors via
`authored_est_cost`. A future claim ledger (IDE-039) replaces the builders,
never the seam.

**One formula site.** `authored_est_cost(bounds, cfg)` — the β-skew
computation extracted from `graph.rs` as a shared pure helper — feeds both
the est anchor builder and the scoring ladder's authored branch. One
*formula* site + one *consumption-ladder* site (D6); a test pins builder
output == graph authored branch.

**Flow order (D6).**

```
scan → value anchors, est anchors (pure builders; authored_est_cost)
     → pipeline: resolve once; compile+project per domain
     → cost feed = est projection minus Gauge tier (named fn, tested)
     → graph build_from_with_cfg(…, value projection, cost feed)
     → value_dim / elicit / surfaces
```

**Cost-feed tier rule, executable:**

| projection case | provenance | fed to scoring? |
|---|---|---|
| P3 anchored class (incl. members merged in without own facet) | Authored | yes — covers the merge-hoisted bare member; items with their own authored facet never consult the feed (ladder order), so redundant entries are inert |
| P4/P5/P6 placements | Projected | yes |
| P7 no-floor-no-ceiling, P8 anchor-free component | Gauge | **no** (D2) — render only |

**Single consumption ladder (D6).** `graph.rs::est_cost`: authored bounds →
`authored_est_cost` (unchanged); else cost-feed lookup (EPSILON-floored); else
bare anchor `ctx.absent` (D7). All consumers already route through graph
costing (`item_costing`, SL-217) — projected-cost movement reaches elicit
eff-weights and the D18 reprobe with zero new machinery.

**Findings.** Variants gain `domain` at construction (D9); compile payloads
stay domain-agnostic.

**Module impact (design-target selectors).**

| path | change |
|---|---|
| `src/comparison/wire.rs` | `DOMAIN_ESTIMATE`, `FRAME_MORE_WORK`, frame-table row, `admissible_estimate_pair` |
| `src/comparison/store.rs` | `DomainSystem` split; two anchor maps; est projection; cost-feed fn |
| `src/comparison/project.rs` | `ProjectionParams`; no config reads; `VALUE_PROJECTION_PARAMS` const |
| `src/comparison/resolve.rs` | no semantic change (est rows resolve Active); doc updates |
| `src/priority/graph.rs` | `est_cost` ladder + cost-feed param on `build_from_with_cfg`; `authored_est_cost` extraction |
| `src/priority/surface.rs` | shell wiring: anchor builders, cost-feed filter, explain cost-source block |
| `src/priority/elicit.rs` | `sizing-probe` candidate kind (§4) |
| `src/priority/config.rs` | `EST_GAUGE_STEP` + `[priority.estimate] gauge_step` parse |
| `src/priority/findings.rs` | `domain` discriminator on comparison findings |
| `src/priority/render.rs` | cost-source fragments, probe render |
| `src/commands/compare.rs` | capture: est-pair admissibility branch; elicit render additions |
| `tests/e2e_compare_estimate.rs` | new e2e |

Not touched, stated: `comparison/compile.rs` (reused as-is; domain enters at
finding construction, D9), `comparison/query.rs` (determinacy untouched —
probes make no determinacy claims), `commands/guard.rs` (elicit already
READ-classed).

## §3 Projection, scoring feed & governance

**Positivity axiom (D11).** Est feasible region = order constraints + point
anchors + `c > 0`. All anchors > 0; P6's clamp is the axiom made concrete;
the RV-265 F-1 hazard structurally cannot arise. Fed costs > 0, invariant +
EPSILON floor at the consumption seam.

**Cost projection.** Same P1–P9 machinery via `ProjectionParams` (D8); the
*consumption contract* differs (D2).

**Compression-below-small-anchors — evidence vs convention, split
precisely.** N items evidenced strictly below a 0.5-shot anchor: evidence +
positivity axiom force every feasible assignment into `(0, 0.5)` — the ≥2×
`value_dim` boost relative to the anchor item is *forced*. The spacing within
the interval is *convention* (interpolation over a synthetic floor).
Conventional spacing feeding a divisor is the honest price of D2's projected
tier — accepted, documented artifact (SL-213 D14 posture): pinned by a
chain-of-3-below-small-anchor golden; C6 bounds render beside the point;
provenance `projected` labels it.

**Burndown interplay:** none — burndown consumes value, not cost. Stated to
close the SL-210 consistency question for this domain.

**Scoring feed (normative pin).** Ladder per D2/D6, source precedence only.
Zero est rows ⇒ empty system ⇒ empty feed ⇒ bitwise-identical scoring
(VT-pinned, SL-213 empty-projection-map precedent).

**Governance deliverable — REV against ADR-015 (REV-022 pattern).** One
revision, authored within this slice, approved before the scoring-feed phase
lands. Content:

1. **Estimate-source resolution** added to ADR-015, mirroring value-source
   resolution: *Authored* `[estimate]` bounds — an **operator pin**: policy
   override over accumulated sizing evidence, wins outright, and acts
   (β-resolved) as a point anchor in the est constraint system (pin framing
   per SL-217 product-critique §5; forward-compatible with IDE-039).
   *Projected* — items without authored bounds but with sizing evidence in an
   anchored component take the deterministic point projection. *Bare anchor* —
   `max_upper + margin` (unchanged; `1.0` empty-corpus fallback).
2. **INV-2 restated**: the bare anchor dominates every *authored* estimate;
   projected costs may exceed or undercut it. Original anti-inversion intent
   preserved: a bare item with NO evidence keeps the dominating divisor.
3. **Gauge-never-divides**: conventional magnitudes may fill a numerator,
   never a denominator.
4. **Positivity axiom** recorded (D11).
5. β/`margin`/`gauge_step` operator knobs; edits re-run est projection →
   costs → value determinacy (D18 reprobe, both domains).

REV drafted at plan time, approved before the phase that flips the scoring
feed on; earlier phases strictly additive and REV-independent.
Spec-coherence: ADR-015 is the only governance naming `est_cost` resolution;
SPEC-020 describes the facet, not scoring — re-checked at /close.

## §4 Elicit sizing probes

**Candidate kind.** `CandidateKind::SizingProbe` in the existing queue.
Subject pool: top-K frontier items with **no authored estimate and zero
est-domain active rows of any compilation status** — an `incomparable` row
retires the probe (the engine asked once; a different comparator is curator
territory per the Q1 queue/curator split; the subject keeps its annotations).
One probe per subject (D14 partition precedent). Gauge-component items are
never probed — mask annotation only; component-calibration probes deferred.

**Probe target, deterministic.** Median-cost item among top-K items with
authored estimates (even count → lower-cost middle; ties id-lexicographic).
Fallbacks: none estimated in top-K → median over all admissible items **with
authored estimates** (never projected- or gauge-costed targets — the anchored-
membership postcondition below must hold by construction, not by luck); none
anywhere → no probes + state detail "no estimated item to
calibrate against — estimate any item to seed sizing". Order-bearing answers
land the subject in the target's anchored component (`prefer` ⇒ Projected
placement; `equal` ⇒ anchored-class membership, provenance Authored);
`incomparable` mints nothing and retires the probe: **queue-driven sizing
yields anchored membership or nothing — never a gauge component**.

**Ranking & admission.** Existence-admitted (D12 precedent), never
yield-ranked: `guaranteed_yield: 0`, `yield_basis: "calibration"`,
`score: 0.0` — probes sink below yield-motivated entries via the existing
sort; order among probes is id order (documented behaviour, not a prominence
claim). No `yield_by_answer` — field omitted; no hypothetical machinery runs
for probes.

**State semantics (D15, precedence unchanged).** Probes are entries: an
un-probed un-sized top-K item ⇒ `Candidates`, gating `Stable` exactly as
anchor-review does — actionable sizing debt gates. Residual sizing debt with
**no entry** (sizing-declined subjects, gauge-masked items) is disclosed,
never gating: `Stable` is already scoped to current costs (SL-217 D7 — costs
sit outside the determinacy region; any estimate edit may move them). Stable
wording gains: "stable at current costs; N top-K items carry unresolved
sizing — costs may move on new evidence".

**Capture routing.** `--frame more-work` derives `domain = estimate` (S1);
admissibility branches by derived domain. Probe render carries the exact
answer command:
`doctrine compare record <subj> <target> --prefer a|b | --equal | --incomparable --frame more-work`.
Full v2 response set valid; `incomparable` compiles to `NoConstraint` as
everywhere.

**JSON.** Additive kind, schema stays v1 (D16 posture; consumers switch on
`kind`): spine + `subject { id, annotations }`, `target { id, estimate }`,
`ask { frame: "more-work", domain: "estimate", answers: [...] }`.

## §5 Surfaces

**Capture.** `--frame more-work` joins the closed flag vocabulary; derives
`domain = estimate` silently; help text carries the sizing one-liner ("which
is more work? — winner is the costlier"). Default frame stays `equal-effort`.

**`compare list`.** Status column routes per-row to the owning domain system
(§2). The existing frame column discloses domain — no new column.

**`explain`.** New cost-source block beside the S3 value-source block, three
shapes + one flag:

- `est_cost 5.9 — authored [2.0 ‥ 8.0] · β 0.65` (the operator pin,
  provenance now explicit)
- `est_cost 3.4 — projected · bounds (2.0 ‥ 5.65) · from 4 constraining
  sizing judgements (3 human, 1 agent)` — T7 rater-split disclosure;
  `NoConstraint` rows excluded from counts (S3 precedent)
- `est_cost 11.0 — bare anchor (max estimate 10.0 + margin 1.0)`
- gauge case: cost-source shows bare anchor (what scoring actually used) PLUS
  `sizing: gauge · ordered by N judgements, no estimated item in component —
  estimate any member to calibrate`. The render never implies gauge fed the
  divisor (D2 honesty).

**Findings.** Domain-tagged render (D9); est `AnchorConflict` wording per D1.
JSON parity, existing idiom.

**Elicit render.** Probe entries: rank, kind `sizing-probe`, ask line,
subject (with annotations), target (with estimate), exact answer command.
State-detail additions per §4.

## §6 Verification plan

Suites → rules pinned. VT/VA/VH ids minted at `/plan`.

1. **Wire**: frame-table row (`more-work → estimate`); admissibility census
   property over `ALL_KINDS` (admitted ⇔ `VALUE_BEARING + RECORD`
   derivation); refusal messages; v2 round-trip goldens with est-domain rows.
2. **Resolution**: est rows resolve Active; cross-domain identity-key
   separation (same pair, value + est rows: no collision, no cross-domain
   supersession); R3 within-session revision scoped per domain.
3. **Compilation (est system)**: anchors via `authored_est_cost` — a test
   pins builder output == graph authored branch (one formula site); C2 with
   cost anchors; C4 stale-estimate violation-closure golden; quarantine maps
   disjoint by row uid.
4. **Projection**: value call site passes `VALUE_PROJECTION_PARAMS` —
   existing goldens byte-identical (the D8 invariant); est goldens: anchored
   chain, P5 above-top-anchor, P6 compression-below-small-anchor (chain of 3
   under 0.5 — the §3 documented artifact), gauge component present in
   projection but absent from feed; `EST_GAUGE_STEP` sensitivity sweep
   (order-safety + provenance invariance).
5. **Scoring feed**: ladder goldens — authored beats feed; feed fills
   evidenced-bare; unevidenced-bare keeps bare anchor; gauge-masked item
   scores at bare anchor; merge-hoisted member fed at class anchor value;
   regime-flip golden (first anchor flips component members bare →
   projected); projected-above-bare-anchor golden (INV-2 restatement pin);
   fed-costs > 0 property + EPSILON floor at the feed branch.
6. **Behaviour preservation**: zero est-domain rows ⇒ bitwise-identical
   scoring; every existing priority + comparison suite passes unchanged;
   empty feed ⇒ identical graph build.
7. **Reprobe integration**: authored range edit moves the anchor ⇒ est
   projection + value determinacy re-run (D18 spanning both domains); β edit
   ditto.
8. **Probes**: pool predicate (no authored estimate ∧ zero est rows any
   status); incomparable-retires-probe golden (no re-probe loop);
   order-bearing answer ⇒ subject **fed on next refresh with non-Gauge
   provenance** (`prefer` ⇒ Projected; `equal` ⇒ Authored via class anchor),
   never bare-anchor fallthrough; target selection (median rule, even-count
   lower-middle, both fallback tiers, none-anywhere state detail); probes
   gate `Candidates`; sizing-debt disclosure on Stable; JSON kind golden;
   byte-determinism.
9. **Surfaces**: three cost-source shapes + gauge flag line; domain-tagged
   findings render + JSON parity; list status routing per domain.
10. **e2e** (`tests/e2e_compare_estimate.rs`): capture `more-work` → compile
    → project → feed → visible score shift in `explain`; full probe
    round-trip (elicit → record → re-elicit shows subject sized).

## RFC-019 deviations (design-stage, recorded)

1. **"Projection adjusts ranges minimally within bounds" superseded (D1).**
   Predates SL-213's pure-order + point-anchor settlement; the literal
   range-revision reading would move authored facets, which REV-022's
   anchors-win posture forbids. Scalar projection + C6 bounds display.
2. **Sizing probes are a scope addition** over the RFC's "sibling extension"
   text (Q3=C, sanctioned at clarification) — existence-admitted only; the
   RFC's cross-domain nomination stays Phase E.

## Resolved scope OQs

- Scope OQ-1 (per-domain vs domain-tagged ConstraintSet) → two independent
  systems, shared resolution (§1/§2).
- Scope OQ-2 (queue entry vs curator surface) → Q3=C, sizing probes (§4).
- Scope R1 (range facet vs scalar machinery) → dissolved by D1 (operative
  scalar latent).
- Scope R2 (corpus-wide perturbation) → behaviour-preservation VT (§6.6);
  feed only ever *adds* cost sources for evidenced items.

## Review history

- **Clarification loop** (2026-07-13) — four product forks locked with the
  user: Q1=A (projected cost feeds scoring, REV against ADR-015), Q2=A
  (gauge never divides), Q3=C (sizing-probe kind), Q4=A (VALUE_BEARING +
  RECORD). IDE-039 captured mid-design from SL-217 product-critique §5/§6.
- **Codex GPT-5.5, section-by-section** (2026-07-13, five passes, hostile):
  - §0–§1: D6 priority-coupling hand-wave (blocker — compiler-absence made
    explicit + future contract pinned); point-anchor-stronger-than-range
    under-argued (major — operative-scalar latent stated, D1); "verbatim"
    overclaim (major — narrowed to machinery-vs-consumption); ladder
    numeric-dominance ambiguity (major — source precedence pinned, INV-2
    restated).
  - §2: ProjectionParams behaviour-identity asserted not established
    (blocker — VALUE const + byte-identical goldens); duplicate est_cost
    formula sites (major — `authored_est_cost` shared helper); Projected-only
    feed filter not executable, merge-hoisted member dropped (major —
    everything-except-Gauge rule + tier table); two-system status reads
    value-biased (major — row→owning-domain routing); domain discriminator
    location unstated (minor — construction-site rule, D9).
  - §3: compression rationale overclaimed evidence (blocker — positivity
    axiom D11 + evidence/convention split); "verbatim" again (major).
  - §4: anchored-membership claim false under `incomparable` (blocker — pool
    predicate tightened, probe retirement, claim restated); Stable wording
    vs gauge-masked items (major — disclosed-not-gating via D7 posture).
  - §5–§6: probe postcondition wrong for `equal` (major — non-Gauge
    provenance contract). §5 surfaces survived unfound.
- **RV-273, codex GPT-5.5** (2026-07-13, hostile whole-doc pass, fresh
  context, ledgered) — two majors, both accepted as wording fixes: F-1 (D4
  rationale overclaimed "targets the mask exactly" vs §4's gauge-masked
  exclusion — D4 scoped to actionable-without-curator); F-2 (fallback
  target rule said "estimated items", ambiguous over projected-costed
  targets — pinned to authored estimates so the anchored-membership
  postcondition holds by construction). Cross-section D1/D2/D6 consistency,
  P-rule parameterization, SL-217 D15 interaction, and REV content survived
  unfound.

## Deferred (named seams, not built)

- Priority-domain compiler — must consume post-ladder resolved costs and
  ride the est-first ordering (D6); post-C sibling work.
- Est-domain anchor-review candidates (D10) — trigger: eval evidence of
  est-anchor sterilisation.
- Component-calibration probes — trigger: gauge components arising from
  manual capture (queue flow cannot create them, §4).
- Cross-domain yield ranking, `DecisionContext::Scoping` — Phase E
  (IMP-287); entry criterion RV-260 F-5 (estimate feasible-region model,
  which under D11 must carry the positivity term; D8-lemma re-check over
  positive reals pinned there).
- IDE-039 magnitude-claims ledger — anchor builders are the insertion point
  (§2); needs its own RFC.
- Ratio/band vocabulary — voids the D8 lemma (SL-217 Deferred, restated).
