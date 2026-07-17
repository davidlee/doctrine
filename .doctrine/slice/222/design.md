# SL-222 design — Ledgered estimate claims

Status: drafted 2026-07-17, post-clarification loop (four operator
adjudications Q1–Q4); governs implementation. Governing contracts: RFC-020
(Phase 2; T1–T7 invariants, RV-275 gate obligations), SL-220 design (locked;
the claims machinery this slice generalises — D1–D14 apply unless restated),
SL-219 design (est-domain constraint layer — D1 operative-scalar latent, D2
gauge-never-divides, D6 flow order, D7 no-feedback bare anchor, D8
ProjectionParams, D11 positivity axiom — all preserved), ADR-015 (amended by
this slice's REV), ADR-001 (layering), REV-023 (dissolved by this slice's
REV), REV-024 (its "rung deletes at Phase 2" clause fires here), STD-001,
STD-002.

Shorthand: "the value pass" = `comparison/claims.rs` as shipped by SL-220;
"operative cost" = the SL-219 D1 latent, `lower + β(upper − lower)` floored
at EPSILON.

## Decision ledger

| id | decision | rationale (compressed) |
|---|---|---|
| E1 | Estimate anchor payload = `{est_lower, est_upper}` — two new optional f64 columns on the v3 row, validation mirroring `estimate::validate` exactly (finite, `lower ≥ 0`, `upper ≥ lower`; single source, no policy fork). RFC-020's "range, skew, unit, confidence" payload sentence is a **recorded design-stage deviation**: the shipped facet is `{lower, upper}` only — skew and unit are global config, per-facet confidence does not exist (confidence is inherent in range width). *(Operator-adjudicated Q1)* | Migrate what exists; a confidence/skew column would mint authored numbers with zero consumption seam — the pattern RFC-020 itself rejects. Additive columns mean the deferred seams (structured assumptions, per-claim skew) arrive losslessly later |
| E2 | Frame `cost-anchor` (`FRAME_COST_ANCHOR`) joins `estimate → {more-work}`; `ANCHOR_FRAMES` grows it; form⇔frame biconditional and the per-domain payload-exactness matrix extend (estimate anchor: `est_lower`+`est_upper` present, `magnitude` absent; value anchor unchanged). **Additive within v3 — no version bump** (SL-220 D1/D2 as designed: parse lossless over absent optionals; validation is capture policy, not wire structure) | Phase 2 was priced at zero schema motion; this cashes that cheque |
| E3 | The claims pass generalises over a **domain payload**: one generic fold (partition → tier → winning tier → singleton/corroboration/conflict → route) parameterised by payload extraction, per-field mean, and operative-scalar function. Value instantiation = `f64` magnitude, operative ≡ identity — behaviour-preserving refactor of the value pass (assertion semantics + goldens unchanged; mechanical accessor re-path is the sole allowed churn — RV-282 F-3, §2). Estimate instantiation = `(f64, f64)` range, operative = the one formula site with skew passed as a pure input | RFC-020 T2 made executable in code, not just schema; two hand-maintained copies of tier/conflict/lens logic is the parallel-implementation smell |
| E4 | Same-tier conflict for ranges = **per-field mean** (mean of lowers, mean of uppers) over the full active winning-tier row multiset (D3 posture — no dedupe); conflict `distinct` counts distinct `(lower, upper)` pairs; the conflict interval `{low, high}` is over per-row **operative costs**. Linearity lemma, stated precisely *(RV-282 F-1)*: the **affine** β-resolution `lower + β(upper − lower)` distributes over the per-field mean exactly in the reals; the EPSILON floor composes **after** aggregation (`operative = floor_eps(affine(payload))`) and, being monotone convex, diverges from the mean-of-floored only when a row's affine cost dips below `EPSILON = 1e-12` — a sub-ε corner where determinism, not equality, is the invariant (payload validation keeps costs ≥ 0). The property test asserts float-tolerance equality of `affine(mean)` vs `mean(affines)`, plus the exact-composition rule; nothing downstream depends on exactness — the cached `operative` of the mean IS the resolved cost by definition. Closure: per-field means of valid ranges are valid. *(Operator-adjudicated Q3)* | D3 verbatim under affine linearity; no second aggregation rule smuggled in; the floor caveat named instead of overclaimed |
| E5 | Anti-laundering (D4 mirror): the estimate `anchor_map()` = operative costs of **Pin/Human** resolved claims only; Agent/Migrated claims are graph-ladder priors and never reach `compile`. **Disclosed behaviour delta**: SL-219's "estimated records anchor too" narrows — only anchored-tier record claims contribute anchor mass; a migrated record estimate is a prior and anchors nothing until a human re-asserts | The exact laundering failure RFC-020 dissolves, cost-side; the records delta is the flip working as designed and the REV must say so |
| E6 | The flip — `est_cost` resolves by the claim ladder: **pin > human claim > cost projection (non-Gauge) > agent claim > migrated claim > unmigrated `[estimate]` facet (transitional, dies this slice) > bare anchor**. Gauge-never-divides (D2) and the positivity axiom (D11) unchanged: payload validation gives `lower ≥ 0`, every operative cost floors at EPSILON. Verb re-plumbing rides the flip phase (an `estimate set` that stops writing facets before the ladder reads claims would silently change scoring) | REV-023's `authored > projected > bare anchor` dissolves exactly as REV-022 did for value; source precedence only, no numeric-dominance claim (INV-2 restated per E7) |
| E7 | Bare anchor re-sources: `max_upper` = max over **every active unlensed estimate-anchor row's `est_upper`** — any tier, winning or losing, conflict rows individually *(revised at RV-282 F-2: resolved/conflict-mean uppers under-dominate — two same-tier rows `[0,1]`,`[0,100]` mean to upper 50.5, and the bare anchor would sit below the asserted 100)* — plus transitional unmigrated-facet uppers; tombstoned/superseded rows excluded by resolution; projected costs never feed it (D7's no-feedback invariant preserved). Computed **inside the pipeline** after claim resolution (see §3 order fix, RV-282 F-4). INV-2 restated in the REV: the bare anchor dominates every *active asserted* range — true by construction under row-sourcing; projected costs may exceed or undercut it; a bare item with no evidence keeps the dominating divisor. *(Operator-adjudicated Q2; sourcing revised per RV-282 F-2)* | The bare anchor is a corpus-scale prior, not an authority contest; anchored-tiers-only collapses to `1.0` on a freshly migrated corpus and resurrects the ISS-057 inversion; row-sourcing makes domination a construction, not a hope |
| E8 | Verbs: `estimate set <id> <LOWER> <UPPER> \| -x N` gains mandatory `--rater human\|agent` (+ `--by/--basis/--lens/--note/--supersedes`) and mints an anchor row per invocation (D10 — no no-op guard; the current "estimate unchanged" no-op message dies); `estimate pin` + `estimate pin --retire` new, D13-gated (interactive-TTY + worker-refused class, mandatory `--by`); `estimate clear` tombstones all active unlensed estimate anchor rows, refuses under an active pin, `--lens` explicit. Supersession scope = identical (subject, domain, lens). No entity TOML is touched by any of them post-flip | Verb-for-verb the SL-220 §4 contract with the estimate payload; one admission path per domain (RV-275 F-5) |
| E9 | Migration: `scripts/migrate_estimate_facets.py`, the SL-220 D8/§5 contract verbatim (throwaway, stdlib-only, dirty-tree refusal, session-per-run, census, `--check`/`--execute`, doctrine-binary parse gate, per-file strip verification, lossless rollback). Idempotency key = **facet state (path, lower, upper)**; emits `rater = migrated` rows — never pins, though REV-023 called authored estimates "operator pins" (importing them as pins would fabricate constitutional weight, RV-275 F-3). Then the **Q4 deletion** (see E10-adjacent §5): with both domains migrated, rung 5 (both ladders), `EntityFacets.value/estimate`, the facet parse/consumption paths, and `facet_write`'s facet machinery delete; a scan-seam **key-presence tripwire** keeps the `UnmigratedFacet` finding alive for both domains, naming the remedy. *(Operator-adjudicated Q4)* | Fires the trigger SL-220 D6/§3/§4 and REV-024 pinned; never-migrated corpora lose facet contribution (disclosed re-rank) but never rot silently |
| E10 | Sizing probes recompose: the pool predicate's "zero est-domain active rows" narrows to **zero est-domain *pairwise* rows** (an anchor claim is an assertion, not sizing evidence — without the split, any claim row silently retires probes); "no authored estimate" becomes claim-aware and knob-composed — a probe retires iff an **anchored-tier** resolved claim exists, or (`demote_agent_evidence` unset) **any-tier** resolved claim exists. Probe **target** selection ("median-cost item among items with authored estimates") re-sources to items with anchored-tier resolved claims **intersected with estimate-pair admissibility** (`VALUE_BEARING + RECORD` derivation, SL-219 D3 — a human claim on an inadmissible governance kind is captured but never a target, else the probe's answer command would be refused at capture; RV-282 F-8) — forced by the anchored-membership postcondition (only Pin/Human claims enter `anchor_map()`; a migrated-claim target would strand the subject in a gauge component). Fallback state detail updates: "estimate any item to seed sizing" → the remedy names `estimate set --rater human` / `estimate pin` | SL-220 §3's demotion semantics ("a number, not an answer") extended to sizing; the postcondition "queue-driven sizing yields anchored membership or nothing" survives by construction |
| E11 | Governance: **this slice's REV against ADR-015** (REV-023's dissolution — §7); approved before the flip phase lands, with SPEC-020's estimate normative amendments in the same gate (D12 mirror). Earlier phases strictly additive and REV-independent. The REV also records REV-024's transitional-rung deletion clause as fired (value rung 5 dies here too) | Canon never stale about what `estimate set` does while the corpus is live |
| E12 | Rendering: `explain` cost-source block gains claim-tier shapes (§6); JSON `cost_source` is a **disclosed breaking token-set change** (D11-of-SL-220 mirror): `authored` and `authored (via class anchor)` removed; `pin`/`human-claim`/`agent-claim`/`migrated-claim`/`unmigrated-facet`/`anchored (via class anchor)` added; `projected`/`bare-anchor` + gauge flag byte-stable. `show`'s estimate line re-sources from the comparison pipeline (capture-lossless: a record's human estimate renders, annotated scoring-inert); absent evidence ⇒ line omitted. Demotion disclosure widens to estimate claims | Honest tokens; no code path emits `authored` post-flip |
| E13 | The one formula site **relocates to the estimate leaf**: `estimate::operative_cost(bounds: (f64, f64), skew: f64) -> f64` (EPSILON-floored — the floor is part of the formula). `priority::graph::authored_est_cost` delegates to it (callers migrate mechanically); `comparison::claims`' estimate instantiation consumes it with skew threaded as a pure input (the D8/date-uid pattern — no config reads in the pass) | `comparison` must not import `priority` (ADR-001 — priority already imports comparison; a back edge is a cycle); the leaf is the natural home and the one-site invariant survives the move |

## §1 Wire: the estimate anchor row

**Target behaviour.** An absolute sizing claim is one `[[judgement]]` row:
`form = anchor`, `domain = estimate`, `frame = cost-anchor`, payload
`est_lower`/`est_upper`. Everything else — supersession, tombstones,
within-session revision (R3 identity), cross-session concurrency, lens
capture, `rater`/`admission`/`basis`/`date`/`observed_at` semantics — is the
SL-220 §1 machinery, untouched.

**Column motion (all additive within v3 — E2, no version bump):**

| field | change | rule |
|---|---|---|
| `est_lower` | new `Option<f64>` | estimate-anchor payload; absent on every other row |
| `est_upper` | new `Option<f64>` | ditto |

**Validation matrix extensions** (`validate_judgement`; parse stays lossless):

- `form = anchor ∧ domain = estimate` ⇒ `est_lower` present ∧ `est_upper`
  present ∧ `magnitude` absent; the pair satisfies `estimate::validate`
  exactly (finite, `lower ≥ 0`, `upper ≥ lower`) — one validation source
  (E1), so the positivity axiom holds at capture.
- `form = anchor ∧ domain = value` ⇒ `est_lower`/`est_upper` absent
  (payload exactness cuts both ways).
- `frame = cost-anchor ⇔ (form = anchor ∧ domain = estimate)` — the
  form⇔frame biconditional extends; `anchor_frame_for(estimate) =
  cost-anchor`. A `more-work` frame on an anchor row, or `cost-anchor` on a
  pairwise row, is rejected at capture.
- Pairwise rows (`order`/`ratio`) ⇒ `est_lower`/`est_upper` absent.
- `rater = migrated` / `admission = pin` rules: unchanged, domain-agnostic
  (a migrated estimate row carries `observed_at`, no `date`; pin ⇒ human).

**Anchor subject admissibility** mirrors the current `estimate set` surface
(the facet is kind-agnostic — SPEC-020 §3): any entity id that resolves is
accepted; consumption is gated at the scoring seam (§3), the D7-of-SL-220
capture/consumption split. `admissible_estimate_pair` (`VALUE_BEARING +
RECORD`) remains the *pairwise* gate, untouched.

**Sample rows** (live human claim; migrated import):

```toml
[[judgement]]
uid = "…"
seq = 0
a = "SL-221"
form = "anchor"
domain = "estimate"
frame = "cost-anchor"
est_lower = 2.0
est_upper = 8.0
rater = "human"
by = "david"
basis = "ASM-014; assumes v3 wire reuse"
date = "2026-07-17"

[[judgement]]
uid = "…"
seq = 0
a = "IMP-118"
form = "anchor"
domain = "estimate"
frame = "cost-anchor"
est_lower = 1.0
est_upper = 3.0
rater = "migrated"
basis = "facet [estimate] .doctrine/backlog/imp-118.toml @ 9c01d2aa david 2026-06-28"
observed_at = "2026-07-17"
```

**Code impact (§1):** `src/comparison/wire.rs` — two columns,
`FRAME_COST_ANCHOR`, frame-table row, `ANCHOR_FRAMES` entry,
`anchor_frame_for` totality, validation-matrix arms, goldens.
`src/comparison/resolve.rs` — none semantic (anchor identity/lifecycle rules
are already form-keyed and domain-carried; estimate anchors ride them).

## §2 Claims pass: one generic fold, two payloads

**Target behaviour.** `resolve_claims` generalises (E3): the fold that
partitions by lens, tiers rows, picks the winning tier, distinguishes
corroboration from conflict, captures singleton attribution, and routes by
`is_anchored()` is domain-generic; a payload parameter supplies what varies.

```rust
/// What a domain's anchor claim asserts. Implementations: value (f64),
/// estimate ((f64, f64)).
pub(crate) trait ClaimPayload: Clone + PartialEq {
  /// Extract this domain's payload columns from a validated anchor row.
  fn extract(j: &Judgement) -> Option<Self>;
  /// E4 / D3: the same-tier aggregate over the winning-tier multiset —
  /// scalar mean for value, per-field mean for estimate.
  fn mean(rows: &[Self]) -> Self;
  /// The scalar the engine charges/credits — identity for value; the one
  /// formula site (E13) for estimate, with skew resolved by the caller.
  fn operative(&self, params: &Self::Params) -> f64;
  type Params;
}

pub(crate) struct ResolvedClaim<P: ClaimPayload> {
  pub payload: P,            // singleton payload, or the E4/D3 mean
  pub operative: f64,        // cached operative scalar of `payload`
  pub tier: ClaimTier,
  pub conflict: Option<ClaimConflict>,  // interval over per-row OPERATIVE values
  pub rows: u32,
  pub attribution: Option<ClaimAttribution>,
}

pub(crate) fn resolve_claims<P: ClaimPayload>(
  rows: &[(&Judgement, ResolutionStatus)],
  domain: &str,
  params: &P::Params,
) -> ClaimResolution<P>
```

Mechanics preserved verbatim from the value pass: own input selection over
{`Active`, `InertLens`} anchor rows of the given domain (RV-278 F-2);
`ClaimTier` / `is_anchored()` / `tier_of` unchanged and shared; findings fire
at every tier from the unlensed partition, reprobe nomination
anchored-tiers-only (D14); BTreeMaps + `total_cmp`-ordered folds keep
permutation invariance — for estimate, both per-field means are independently
permutation-free sums. `distinct` counts distinct payloads (distinct
`(lower, upper)` pairs for estimate — two rows disagreeing on range but
coinciding on operative cost still conflict, honestly, with a degenerate
interval). `ClaimFinding::Conflict`'s `{low, high}` carries the operative
interval for both domains (value: operative ≡ magnitude, so the shipped
finding shape is unchanged — zero churn).

**Value instantiation is a behaviour-preserving refactor — the gate stated
behaviourally (RV-282 F-3).** `P = f64`, `extract = j.magnitude`, `mean` =
the existing multiset mean, `operative` = identity, `Params = ()`. The
`ResolvedClaim.value` field is consumed throughout the existing battery and
downstream (`claims.rs` tests, `store.rs`, `graph.rs`, `surface.rs`,
`elicit.rs`), so "tests green unchanged" byte-for-byte is unachievable and
not claimed. The engine gate is the SL-220 §2/RV-277 PHASE-02 precedent:
**assertion semantics preserved, zero golden churn, mechanical accessor
re-path only** (`.value` → `.operative`, enumerated at phase-plan as the
allowed churn class); any assertion whose *expected values* change fails the
gate.

**Estimate instantiation.** `P = (f64, f64)`, `extract` =
`(est_lower, est_upper)`, `mean` = per-field (E4), `operative` =
`estimate::operative_cost(self, skew)` (E13), `Params = skew: f64` threaded
from config by the store shell (pure input — no config reads in the pass).
The E4 linearity lemma (`operative(mean(rows)) == mean(operative(rows))`, up
to float-fold order pinned by `total_cmp` sorting) is a property test, so the
cached `operative` of a conflict mean IS the mean of the interval's
generating values.

**Pipeline wiring (`store.rs`).** `Pipeline` gains
`estimate_claims: ClaimResolution<EstimatePayload>` beside `value_claims`
(which becomes `ClaimResolution<ValuePayload>` — type motion only). Input
split: `active_judgements`' pairwise/anchor views are already domain-blind;
the estimate claims pass selects `domain = estimate` anchor rows. The
estimate `DomainSystem` compiles with
`estimate_claims.anchor_map()` — operative costs of Pin/Human resolved
claims (E5) — replacing the shell-side `authored_est_cost` facet builder in
`priority/surface.rs` (deleted, the §2-of-SL-220 `comparison_anchor_map`
precedent). Row-gating at `compile` is unchanged and correct (an anchor with
no sizing rows constrains nothing); the graph ladder reads
`anchored`/`priors` directly (dual-seam, SL-220 scope-R1 resolution). D6 flow
order (est system first, cost feed, then value/graph) is untouched.

**Code impact (§2):** `src/comparison/claims.rs` (generic fold + estimate
instantiation + E4 property battery); `src/comparison/store.rs` (estimate
claims wiring, anchor-source swap, `RowSummary` claim-join extends to
estimate anchor rows); `src/estimate.rs` (`operative_cost` — E13);
`src/priority/graph.rs` (`authored_est_cost` delegates);
`src/priority/surface.rs` (facet est-anchor builder deleted; claims threaded).

## §3 The flip: `est_cost` ladder and the bare anchor

**Current behaviour.** `graph.rs::est_cost`: authored `[estimate]` bounds →
`authored_est_cost` (wins outright, and the facet anchors the est constraint
system) → cost-feed lookup → `ctx.absent` (bare anchor from authored uppers).

**Target ladder** (per item, first hit wins; consumption gated at the
existing scoring surface — capture stays kind-agnostic, §1):

1. **Anchored claim** — `estimate_claims.anchored` (Pin/Human; conflict
   means included): the resolved claim's operative cost. Also (via
   `anchor_map()`) anchors cost projection — the two seams the authored
   facet occupied, now with provenance.
2. **Cost projection** — the cost-feed lookup, unchanged machinery
   (non-Gauge provenances only; EPSILON floor at the branch).
3. **Agent-tier prior** — `priors` with `tier = Agent`: operative cost.
4. **Migrated-tier prior** — `tier = Migrated`: operative cost.
5. **Unmigrated `[estimate]` facet** (transitional — deleted at §5's final
   phase): `authored_est_cost`, consulted only when zero estimate claim
   rows exist for the item; presence fires the finding regardless of
   consumption (D6/RV-278 F-4 semantics, estimate-domain instance).
6. **Bare anchor** — `ctx.absent` (E7 sourcing below).

Rungs 1/3/4 are new; 2/6 are REV-023's rungs re-ranked; 5 is the migration
window. Gauge never divides (D2): gauge-tier placements remain absent from
the feed, so a gauge-masked item falls to rung 3/4 if it carries a claim —
an *asserted* range beats a conventional gauge — else rung 6, as today.

**Bare anchor (E7, revised at RV-282 F-2/F-4).** `max_upper` = max over
(a) **every active unlensed estimate-anchor row's `est_upper`** — any tier,
losing tiers and individual conflict rows included (domination by
construction; the winning-tier mean under-dominates), lensed rows excluded
(D5), tombstoned/superseded rows already reduced by resolution;
(b) transitional unmigrated-facet uppers (dies with rung 5), threaded in as
a pure input from the scan shell. Projected costs never enter (D7).
`margin` unchanged; empty-evidence corpus keeps the `1.0` fallback.

**Pipeline order (F-4) — the current architecture computes the gauge centre
from scanned facets *before* the comparison pipeline runs**
(`priority::graph::load_comparison_pipeline` builds `est_cfg` ahead of
`comparison::load_pipeline`; claims resolve inside `pipeline_from_sessions`),
so "gauge centring inherits automatically" is false as-is. The fix is a
deliberate re-siting: the bare anchor is **derived inside the pipeline** —
`pipeline_from_sessions` resolves estimate claims, folds `max_upper` over
active anchor-row uppers plus the facet-uppers input, applies `margin`
(threaded config), uses the result as the est system's
`ProjectionParams.gauge_center`, and exposes it as `Pipeline.bare_anchor`.
`CostCtx.absent` consumes the exposed value — the priority shell's own
facet-based computation deletes. One-site test: `CostCtx.absent ==
pipeline.bare_anchor == est gauge centre`, pinned.

**Coupling honesty (SL-219 D6, restated for claims).** `prefer-first`
compiles `v_A·c_B > v_B·c_A` over current costs; no priority compiler exists
yet, so nothing recompiles when claim-resolved costs move — the pinned
contract (any future priority compiler consumes post-ladder resolved costs
and rides est-first ordering) is unchanged, now with "post-ladder" meaning
E6's ladder. Claim-driven cost movement re-runs value determinacy via the
existing D18 reprobe dynamic, exactly as authored-range edits did.

**Behaviour-change accounting (scope R1), three VT classes:** (a) corpora
with no estimate anchor rows and no `[estimate]` facets score
bitwise-identically (empty estimate-claims pass); (b) corpora with facets
re-rank deliberately — rung 1 → rung 5, and after migration rung 4; the
pre/post ranking diff (`value_baseline.py` snapshots — the script is
domain-blind, it diffs full survey rankings) is the accepted evidence
artifact, wider than Phase 1's by the divisor position; (c) every existing
suite that authors no `[estimate]` facet stays green unchanged (engine
gate). The records-anchor delta (E5) is class (b) evidence, called out
separately in the audit artifact.

**The flip stated without euphemism (RV-278 F-4 discipline, est-domain
instance).** At the flip phase the facet est-anchor builder deletes, so
authored `[estimate]` facets stop anchoring cost projection **immediately
and permanently** — and migration does not restore it (a migrated claim
never anchors, E5). A corpus whose only absolute sizing magnitudes were
facets loses est-domain projection anchoring entirely until a human
re-asserts (`estimate set --rater human`, or a pin): anchored classes
dissolve to gauge components, the cost feed empties for them, and evidenced
items resolve at rungs 3–5 (their asserted operative costs) rather than
rung 2. Ranking degrades deterministically — every item with an asserted
range still carries its own cost; only the *propagation* of costs through
sizing comparisons pauses. The loud presence finding plus the `explain`
provenance lines are the re-assertion prompts.

**Determinacy / elicitation.** `demote_agent_evidence` widens to estimate
claims (E10): when set, agent/migrated-tier resolved costs leave sizing
probe-eligibility intact (a number, not an answer); when unset they retire
it. During the interregnum an **unmigrated-facet item counts as sized**
(rung 5 is an asserted range; mirrors pre-flip behaviour, knob-independent,
dies with the rung). Probe **targets** require anchored-tier claims (E10),
so post-flip pre-migration a facet-only corpus has no probe targets — the
"no estimated item to calibrate against" state detail fires with the
claim-era remedy; sizing probes wake as human claims land. Anchored-tier
estimate `ClaimFinding::Conflict` items enter the reprobe queue
knob-independently; agent/migrated conflicts never do (D14 — the shared
`nominates_reprobe` predicate already says so).

**Code impact (§3):** `src/priority/graph.rs` (`est_cost` signature gains
the estimate-claims view; `CostCtx` builder re-source; `build_from_with_cfg`
threading); `src/priority/surface.rs` (shell wiring); `src/priority/elicit.rs`
(§E10 pool predicate + target re-source, sizing-input assembly);
`src/priority/config.rs` (docs only — no new knobs).

## §4 Verb surface: `estimate set | pin | clear`

The SL-220 §4 contract, estimate payload; deltas only:

- **`estimate set <ID> <LOWER> <UPPER> | -x N --rater human|agent [--by
  --basis --lens --note --supersedes]`** — mints a session-of-one anchor row
  (`frame = cost-anchor`, `domain = estimate`, payload validated via
  `estimate::validate`). `-x/--exact` survives (`lower = upper = N`). The
  bounds-pairing refusals (LOWER-without-UPPER etc.) survive verbatim. The
  "estimate unchanged" no-op branch **dies** (D10: every invocation mints).
- **`estimate pin <ID> <LOWER> <UPPER> | -x N --by <who> [--basis --note
  --supersedes]`** and **`estimate pin <ID> --retire [--note]`** — the D13
  gate verbatim (interactive-TTY + worker-refused write class, `rater =
  human` + `admission = pin` stamped, mandatory `--by`).
- **`estimate clear <ID> [--note --lens]`** — tombstones all active unlensed
  estimate anchor rows on the subject; refuses under an active pin naming
  `pin --retire`; `--lens` explicit for lensed rows.
- **Supersession scope** — identical (subject, domain, lens); cross-domain
  targets refused (a value row can never supersede an estimate row — the
  identity key already carries domain).
- **Severance**: `run_estimate_set/clear` re-plumb from `facet_write` to the
  session mint at the flip phase (E6 sequencing); `facet_write`'s facet
  machinery dies at §5's deletion — the migration script is its last writer.
  `main.rs` write-class tests extend (estimate pin verbs join the refused
  class; `estimate_is_write` stays true).

## §5 Scripts, migration, and the deletion

**`scripts/migrate_estimate_facets.py`** — the SL-220 §5 contract with the
estimate mapping; contract-identical items not restated (dirty-tree refusal,
session-per-run, doctrine-binary parse gate, `--check`/`--execute`, census
abort-pre-strip, per-file tomllib strip verification, interruption safety via
rung 5, git-revert rollback):

- Scan every `[estimate]` table under the authored entity dirs; emit per
  facet one anchor row: `domain = estimate`, `frame = cost-anchor`,
  `est_lower/est_upper` from the facet, `rater = migrated`, `observed_at` =
  run date, no `date`, `basis` from git blame best-effort. **Never a pin**
  (E9): REV-023's "authored = operator pin" framing is not provenance — who
  typed the range is unrecorded, so it imports at the bottom of the ladder
  like every other unattributed magnitude.
- Idempotency key = facet state **(path, lower, upper)**; changed facet ⇒
  superseding re-import; census `facets_found == imported + already-imported
  + re-imported`, exactly one active migrated row per source facet.
- Strip removes the `[estimate]` table; `[risk]` and `[tags]` untouched.
- **`--check` is truly non-mutating (RV-282 F-7)** — a deliberate divergence
  from the SL-220 template, whose `--check` writes the session file before
  verifying (its own docstring admits it; combined with the dirty-tree
  refusal, a check blocks the subsequent `--execute`). Here `--check`
  computes rows and the census in memory and writes **nothing** (tree-hash
  verified in §8.10); only `--execute` emits the session, runs the parse
  gate, reconciles the census, and strips. The value script's defect is
  recorded, not repaired — it has already been consumed on this corpus.

**The deletion (Q4, fires SL-220's trigger).** A final phase, after both
censuses hold on this corpus:

- Rung 5 deletes from **both** ladders (`effective_raw_value`'s
  unmigrated-facet rung and §3's).
- **Full blast radius (RV-282 F-5)** — the raw facet model is replicated
  well beyond `EntityFacets`; the deletion inventory is: `catalog/scan.rs`
  (`read_facets`, `ScannedEntity.value/estimate`), `catalog/hydrate.rs`
  (`CatalogEntity` copies), `catalog/graph.rs` (`CatalogNode` copies), the
  per-kind document structs and their render/JSON paths in `backlog.rs`,
  `governance.rs`, `concept_map.rs`, `lazyspec.rs`, `slice.rs` (the value
  display side was re-sourced at SL-220 PHASE-06; the estimate display
  paths re-source here, riding the same shared-helper seam), plus
  `EntityFacets` itself. `src/value.rs` / `src/estimate.rs` keep only what
  claims consume (`validate` as the §1 validation source; `operative_cost`;
  display helpers rendering resolved claims). `facet_write`'s `[value]`/
  `[estimate]` set/clear machinery deletes (risk/tags survive).
- **Grep-gate, widened (F-5)**: not just `EntityFacets.value|estimate` —
  no parse or consumption of the `value`/`estimate` top-level TOML keys
  survives anywhere outside the tripwire: named grep targets
  `read_facets`, `estimate::parse_optional`, `value::parse_optional`,
  `.estimate`/`.value` field access on scan/hydrate/graph/document types.
- **Presence tripwire** (Q4=A): a scan-seam key-presence check — does the
  entity TOML carry a `value`/`estimate` top-level key — fires per domain,
  remedy-naming ("facet no longer read; import via
  scripts/migrate_*_facets.py — stdlib-only, any corpus root — or re-assert
  via `estimate set --rater human`"). Key-existence test, not a facet
  parse — malformed residue still trips it. **Finding shape reshapes
  (RV-282 F-9)**: the shipped `Finding::UnmigratedFacet` carries
  `{domain, entity, value: f64}` populated from a successful parse — a
  parse-free tripwire cannot fill the scalar (and an estimate residue has
  two bounds, a malformed one none). At the deletion phase the variant
  becomes magnitude-free `{domain, entity}`; render drops the magnitude
  ("unmigrated `[estimate]` facet present (unread)"); the findings-JSON
  churn is a disclosed breaking change riding E12's token release.
- Consequence, disclosed in the REV: a never-migrated corpus re-ranks
  (facet-valued items fall to defaults/bare anchor) with loud findings; the
  scripts are committed and corpus-agnostic, so the remedy is real.
- Grep-gate: no consumer of `EntityFacets.value`/`.estimate` survives; the
  repo-wide grep is a named verification step (SL-220 §6 precedent).

**Sequencing pin.** Baseline snapshot (script reuse, live corpus) precedes
the flip phase; E7's bare-anchor re-source ships **in** the flip phase —
before migration ever strips a facet, so `max_upper` never sees an
empty-input window; migration runs after the flip (its rows must resolve);
deletion runs last, post-census. A corpus that never migrates degrades per
rung 5 until deletion, per the tripwire after it.

## §6 Rendering: cost provenance

**`explain` cost-source block** — REV-023's shapes re-tokened; `projected`
and `bare anchor` shapes byte-stable; the `authored` shapes die:

- `est_cost 5.9 — pin [2.0 ‥ 8.0] · β 0.65 (david, 2026-07-17, basis ASM-014)`
- `est_cost 5.9 — human claim [2.0 ‥ 8.0] · β 0.65 (david, 2026-07-17)`
- `est_cost 5.6 — contested human claim · 2 claims, cost interval
  (4.2 ‥ 7.6), mean range [1.5 ‥ 8.0] — resolve by superseding row`
  (anchored tiers; agent/migrated conflicts render the interval with
  "calibrate via comparison" — D14)
- `est_cost 4.0 — anchored (via class anchor)` (the P3 merge-hoist row,
  provenance re-tokened from `authored (via class anchor)`)
- `est_cost 3.4 — projected · bounds (2.0 ‥ 5.65) · from 4 constraining
  sizing judgements (3 human, 1 agent)` (unchanged)
- `est_cost 2.7 — agent claim [1.0 ‥ 4.0] (claude, 2026-07-12) · below
  projection — no projection evidence exists`
- `est_cost 2.1 — migrated claim [1.0 ‥ 3.0] (unattributed · observed
  2026-07-17)`
- `est_cost 5.9 — unmigrated [estimate] facet — run
  scripts/migrate_estimate_facets.py` (rung 5, dies at §5)
- `est_cost 11.0 — bare anchor (max asserted upper 10.0 + margin 1.0)` —
  wording updates: the max is over asserted ranges (E7), no longer
  "max estimate" of authored facets.
- gauge flag line: unchanged.

**`ReasonKind` motion (view.rs).** `CostAuthored` retires (both variants);
`CostPin`, `CostClaim{tier, by, date, conflict}`, `CostUnmigratedFacet`
(transitional, dies at §5), `CostClassAnchor` join. JSON `cost_source` token
change per E12, pinned by a full post-flip vocabulary golden; golden churn on
fixtures authoring `[estimate]` facets is expected class-b evidence.
**Naming hazard (RV-277 F-5 precedent):** the NF-001 tripwire greps facet
symbol substrings — new symbols must avoid the `EstimateFacet`/`ValueFacet`
literals (`CostUnmigratedFacet`, not `EstimateFacetUnmigrated`); final names
checked against the tripwire at phase-plan.

**`show`.** The estimate line re-sources from the comparison pipeline
(SL-220 §6 shared-helper seam — the 9-fold render dedup already exists to
ride): resolved range + tier + attribution + unit
(`estimate 2.0–8.0 espresso_shots (human claim, david, 2026-07-17)`);
scoring-inert annotation for record kinds; absent evidence ⇒ line omitted.
`EstimationConfig` (unit, display confidence bounds) survives — it renders,
never resolves.

**Row surfaces / elicit / findings / disclosure.** Survey/next/blockers cost
cells (where rendered) re-path to resolved `(cost, tier)`; estimate
`ClaimFinding::Conflict` + estimate `UnmigratedFacet` join the findings
render/JSON domain-tagged (D9); `AGENT_DEMOTION_DISCLOSURE` names estimate
claims when the knob is set and a surfaced cost rests on rungs 3–5; elicit
probe/target fragments render claim tiers (E10 target = anchored-tier item).

## §7 Governance: the REV against ADR-015

One revision (REV-NNN at `/plan`), the REV-022/023/024 pattern, approved
with SPEC-020's estimate normative amendments **before the flip phase**
(E11/D12). Content:

1. **Estimate-source resolution rewritten**: REV-023's `authored (operator
   pin) > projected (non-Gauge) > bare anchor` dissolves into E6's ladder.
   Anchors feeding the est constraint layer come from pin/human tiers only
   (E5); same-tier conflict = per-field mean + operative-cost interval,
   surfaced, never silent (E4).
2. **The authored `[estimate]` facet is retired as an input**; `estimate
   set` appends ledgered claims; the pin verb is the operator-policy
   override, now attributed and governed. The "records anchor too" sentence
   narrows to anchored-tier claims — disclosed.
3. **INV-2 restated for the claims era** (E7): the bare anchor is computed
   over every asserted range (any-tier resolved claim; transitional facets)
   and dominates them all; projected costs may exceed or undercut it; a
   bare item with no evidence keeps the dominating divisor.
4. **Gauge-never-divides and the positivity axiom**: unchanged, restated
   with claim vocabulary (payload validation enforces `lower ≥ 0` at
   capture; operative costs floor at EPSILON).
5. **Fixed-policy clause**: the estimate ladder is policy, not a knob;
   `demote_agent_evidence`'s widening to estimate claims documented.
6. **Transitional rungs deleted** (both domains) — REV-024's Phase-2 clause
   recorded as fired; the presence tripwire + re-rank consequence for
   never-migrated corpora disclosed.

**Spec routing — with a REQ disposition map (RV-282 F-6).** SPEC-020's
authority over the estimate facet is *structured*, not just prose:
REQ-269–REQ-277 (the `EstimateFacet` model, optional `[estimate]` parse
seam, present-table validation, parse→hydrate→catalog preservation,
present/none display, policy-free graph exposure, kind-agnostic reuse,
schema forward-compat) and REQ-310 (resolved-bounds display) mandate the
machinery §5 deletes. The flip-gate amendment therefore carries an explicit
**disposition per REQ** — each rewritten to its claim-schema equivalent
(capture validation, resolved-claim display, pipeline exposure) or retired
with the deletion phase named — so no active REQ contradicts the shipped
binary at any phase boundary. Prose amendments (facet as authored surface,
set/clear writer semantics, hydration reader) ride the same gate;
PRD-014/SPEC-020 claim-schema retention REQs and PRD-011/SPEC-001 descent
prose remain reconciliation obligations, non-contradicting. RFC-020's
Phase 2 row moves to delivered-by-SL-222 at reconciliation (Phase 3 keeps
it open).

## §8 Verification plan

Suites → rules pinned; VT/VA/VH ids mint at `/plan`.

1. **Wire**: goldens — live human estimate anchor, pin, migrated, `-x`
   point; round-trip losslessness over the new optionals; validation matrix
   both directions (payload exactness per domain × form, cost-anchor⇔
   estimate-anchor biconditional, `estimate::validate` mirroring incl.
   negative-lower refusal, migrated/pin rules on estimate rows); v2 + SL-220
   v3 fixtures parse byte-identically.
2. **Claims pass**: the SL-220 gate battery re-instantiated for estimate
   payloads (tier ordering, permutation invariance, corroboration vs
   conflict over ranges, conflicting pins, cross-session concurrency, lens
   isolation non-vacuous both directions, anti-laundering property
   (`anchor_map()` ≡ anchored operative costs), no-compile-consumer, no-op
   duplicate posture); **E4 linearity property** — float-tolerance equality
   of `affine(mean)` vs `mean(affines)` over generated multisets, plus the
   floor-composes-after-aggregation rule and a sub-EPSILON corner case
   asserting determinism (RV-282 F-1); distinct-payload-same-operative
   conflict (degenerate interval, fires); value-pass refactor: assertion
   semantics + goldens unchanged, accessor re-path the sole churn class
   (E3 gate as restated, RV-282 F-3).
3. **Ladder (graph)**: each rung wins in isolation; adjacent-rung dominance
   (pin > human > projection > agent > migrated > facet > bare anchor);
   facet consulted only at zero claim rows; row-less human estimate claim
   resolves at rung 1 (row-gating footgun); gauge-masked item with an agent
   claim takes rung 3 (asserted beats gauge), without a claim keeps bare
   anchor; scoring-inert kinds paired capture/consumption over `ALL_KINDS`;
   demote-knob: rungs 3–4 retire sizing probes iff unset (both directions);
   anchored-tier estimate conflicts enter reprobe, agent/migrated never.
4. **Bare anchor (E7)**: migrated-only corpus keeps a dominating divisor
   (the Q2 disaster pin); claim-row uppers move `max_upper`, projected costs
   never do; **every active row upper feeds it — losing-tier and individual
   conflict rows included** (the RV-282 F-2 domination construction: bare
   anchor ≥ every active asserted upper + margin, property-tested); lensed
   claims don't; facet uppers do until deletion; empty corpus → 1.0;
   one-site pin `CostCtx.absent == pipeline.bare_anchor == est gauge centre`
   (the F-4 re-siting proof).
5. **Behaviour preservation**: no-estimate-rows-no-facets corpora bitwise
   identical; SL-219 est-domain suites (compile/project/feed/probe) green
   unchanged through the anchor-source swap where fixtures author no facets;
   enumerated class-b golden churn pinned at the flip phase.
6. **Probes (E10)**: pool predicate splits pairwise/anchor (an anchor row
   alone never counts as sizing evidence); retirement × knob matrix;
   interregnum: unmigrated-facet item counts as sized knob-independently
   (dies with rung 5); target selection over anchored-tier claims ∩
   estimate-admissible kinds (inadmissible-kind claim never targeted —
   RV-282 F-8), fallbacks re-pinned, facet-only corpus post-flip ⇒ no targets + state
   detail naming the claim-era remedy; anchored-membership postcondition
   golden (order-bearing answer against a claim-anchored target ⇒ non-Gauge
   provenance next refresh).
7. **Verbs**: mirror of SL-220 §8.6 for estimate (mandatory `--rater`,
   every-invocation-mints, supersession scope refusals incl. cross-domain,
   pin TTY/worker/`--by` gates, clear-under-pin refusal, `-x` point mint,
   bounds-pairing refusals preserved); write-class regression.
8. **Render**: cost-source shapes per rung incl. contested + class-anchor
   re-token; JSON `cost_source` full post-flip vocabulary golden (breaking
   change pinned); `show` estimate line (resolved, unit, inert annotation,
   absent-omitted); findings render/JSON parity; demotion disclosure;
   post-deletion grep-gate, widened per §5 (no `value`/`estimate` key parse
   or field consumer outside the tripwire — scan/hydrate/graph/document
   types enumerated); magnitude-free `UnmigratedFacet` shape + findings-JSON
   churn golden (RV-282 F-9).
9. **E2E** (`tests/e2e_estimate_claims.rs`): `estimate set` → claim →
   resolution → `est_cost` → `explain` provenance; pin overrides projection;
   human beats agent; migrated loses to projection; conflict → finding →
   superseding row; clear → tombstone → ladder falls through; sizing-probe
   round trip against a claim-anchored target; capture-to-scoring fixture.
10. **Scripts (operational)**: SL-220 §8.9 checklist re-run for the estimate
    script (fixture + live `--check`, census reconciliation, parse gate,
    strip verification, idempotent re-run, interrupted-state rung-5 shadow);
    **`--check` leaves the tree byte-identical** (tree-hash before/after —
    RV-282 F-7); pre-flip baseline + post-flip and post-migration diffs
    attached at audit (R1 evidence).
11. **VA**: RFC-020 T2 holds — the generic pass contains nothing
    domain-specific beyond the payload trait (reviewed against Phase 3's
    declared needs: hierarchy subjects ride admissibility only); REV +
    SPEC-020 amendments approved before the flip (audit-checked); E1
    deviation and E5 records delta recorded in the RFC at reconciliation.

## Resolved scope OQs / risks

- Scope OQ-1 (pin verb surface) → E8: `estimate pin` + `--retire`, D13 gate
  verbatim.
- Scope OQ-2 (mean of ranges) → E4 per-field mean + operative interval;
  linearity lemma (operator-adjudicated Q3).
- Scope OQ-3 (wire version) → E2: additive within v3 (SL-220 D1/D2, locked).
- Scope OQ-4 (REV routing) → E11: this slice's own REV; REV-024 explicitly
  left REV-023 standing.
- Scope R1 (divisor-wide re-rank) → §3 behaviour accounting; baseline diffs
  as audit evidence.
- Scope R2 (payload fidelity) → E1/E4: payload round-trips losslessly;
  supersession/census operate on payload; collapse happens only at
  consumption (`operative`).
- Scope A2 (reuse, not reimplement) → E3: one generic fold; the value
  battery green unchanged is the proof.

## Review history

- **Clarification loop** (2026-07-17, operator): Q1 payload = `{lower,
  upper}` (RFC payload sentence recorded as deviation; provenance rides the
  row; assumptions ride `basis`, structured assumptions + per-claim skew
  override deferred); Q2 bare anchor = any-tier claim sourcing +
  transitional facets (E7); Q3 per-field mean + operative interval (E4); Q4
  deletion honoured with presence tripwire (E9/§5). Skeleton (E1–E12)
  approved 2026-07-17.
- **Internal adversarial pass** (2026-07-17): interregnum probe semantics
  pinned; est-domain anchor drought stated without euphemism (§3); NF-001
  symbol-naming hazard noted (§6).
- **RV-282, codex (default model), hostile whole-doc + tree-verified**
  (2026-07-17): 2 blockers, 6 majors, 1 minor — all accepted and integrated
  in this revision. F-1 (blocker): exact linearity false under the EPSILON
  floor → lemma restated affine-exact/float-tolerant, floor composes after
  aggregation, sub-ε corner named (E4). F-2 (blocker): conflict-mean uppers
  under-dominate → bare anchor sources every active row upper; domination by
  construction (E7/§3). F-3: "battery green unchanged" impossible under the
  field rename → gate restated behaviourally (assertions + goldens; accessor
  re-path enumerated churn). F-4: gauge-centre order impossible in current
  architecture → bare anchor derived in-pipeline, exposed, one-site test.
  F-5: deletion blast radius missed replicated facet readers
  (scan/hydrate/graph/document structs) → inventory + widened grep-gate.
  F-6: SPEC-020 REQ-269–277/REQ-310 undisposed → explicit per-REQ
  disposition map in the flip gate. F-7: inherited `--check` mutates →
  non-mutating check pinned, template divergence recorded. F-8 (minor):
  probe targets need admissibility intersection → E10 restated. F-9:
  tripwire cannot fill `UnmigratedFacet{value: f64}` → magnitude-free
  variant at deletion, JSON churn disclosed.

## Deferred (named seams, not built)

- **Structured assumption citation** — validated ASM refs on claims with
  staleness propagation when an assumption flips (Q1); `basis` free text is
  the channel until a consumer exists.
- **Per-claim skew override** — β as claim content (Q1); forks the one
  formula site only when a consumer justifies it.
- Hierarchy admissibility (Phase 3); aggregation modes / cascade (RFC-020
  OQ-1 + ADR-018 REV); abstention anchor-analogue; estimate feasible-region
  / system confidence (Phase E, RV-260 F-5); cross-domain yield ranking
  (Phase E, IMP-287); ratio/band vocabulary (D8-lemma void); est-domain
  anchor-review candidates (SL-219 D10 trigger unchanged); lens-resolved
  claim surfacing (IDE-035).
