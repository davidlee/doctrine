# SL-217 design — Elicitation queue

Status: drafted 2026-07-12, post-clarification loop; governs implementation.
Governing contracts: RFC-019 (resolved, §Pair selection as revised at the
2026-07-11 review); SL-213 design (shipped constraint layer — D12 anticipated
this slice's query API); ADR-001 (layering), ADR-015 (priority scoring; numeric
internals implementation-owned), ADR-016 (relation roles); STD-001, STD-002.
Design inputs: `.doctrine/rfc/019/phase-b-evaluation.md` caveats C1–C3; the
five preflight dispositions baked into the slice scope (Q1–Q5).

## Decision ledger

| id | decision | rationale (compressed) |
|---|---|---|
| D1 | Verb: `doctrine compare elicit`; answer leg stays `compare record` / `value set`; no TTY interaction | Rides SL-210 D1's domain-neutral top-level `compare` group. RFC's sketch `value elicit` predates that settlement — no `value` verb group exists. Queue/curator split: `elicit` is curated read-only output; capture stays open. |
| D2 | Frontier depth K: config `[priority.elicit] depth` (default `ELICIT_DEPTH = 8`) + `--depth` per-invocation override | K = the team's standing pull-horizon (planning-cadence fact ⇒ config); sessions legitimately widen it (flag). Determinism contract names K an explicit input. `gauge.step` seam precedent. |
| D3 | Decision context is a pluggable seam: `DecisionContext::Sequencing { depth }` ships alone; `Scoping { budget }` is Phase E's slot | The agency fixed-scope case (~hundreds of items) is the *scoping* context — decision-relevance there is membership stability in the boundary band, self-limiting, never K² over the corpus. Yield machinery is context-blind (apply → recompile → count); only the relevant-pair predicate varies. Seam now, so Phase E composes without reshape. |
| D4 | Module split by layer: `comparison/query.rs` (predicates, pure leaf) + `priority/elicit.rs` (queue assembly) + `commands/compare.rs` arm (thin shell) | Same rationale as `comparison_status_map` (graph.rs): the thing needing both tiers lives in the tier that legally sees both. Predicates test without priority fixtures; queue rides the priority fixtures it needs anyway. D12's "Phase C designs its own predicates" lands beside compile/project as anticipated. |
| D5 | Determinacy = `value_dim`-order determinacy, NOT full-score-order | RFC-normative ("the decision the engine actually takes from value is the ordering of value_dim"). Full score couples items via risk/leverage/burndown — exact treatment is an LP over the order polytope. Frontier rank (full score at current projection) is pool selector + impact weight only, never a stability claim; renders say "value_dim order" precisely. |
| D6 | Static multipliers fold into effective weight: pair objective `f = m_A·c_B·v_A − m_B·c_A·v_B` | Live `value_dim` = coeff × kind_weight × tag_term × v / est_cost; the multipliers are per-item constants ≥ 0, so the objective stays 2-variable and closed-form. `m = 0` accounting pinned (web review): the item's value cannot move its `value_dim`, so `m = 0` pairs are excluded from the pool AND from the stability obligation — they are value-insensitive, not indeterminate; when exclusions exist the stable render is scoped ("stable among value-sensitive items; N pairs value-insensitive (zero weight)"). Both-zero pairs are structurally tied for `value_dim`. |
| D7 | Feasible region: one real per class, anchors exact, strict edges hold, **no positivity assumption**; costs are the scalar `est_cost` (bare-anchor ctx included), outside the region | Negative anchors are legal (`facet.rs` vt10 pins `value = -5.0`). RV-260 F-5: estimate uncertainty is not part of the region; an estimate edit moves `est_cost` and re-runs determinacy (the reprobe dynamic). |
| D8 | Marginal exactness lemma: per-class feasible marginal = its C6 interval; pair joint region = `box(A) × box(B) ∩ coupling` (coupling from condensed-DAG reachability / same-class) — **proved in §1, not merely tested** (web review, mandatory) | For strict-order edges + point anchors over dense reals, any point of that set extends to a full feasible assignment (constructive proof, §1). This is what makes joint-set determinacy closed-form — no LP. The SL-213 "box is not the oracle" warning is about *decisions from marginals without the coupling term*; box + coupling **is** the joint set for a pair, for this vocabulary. Property test backs the proof with a naive backtracking extension oracle over generated small anchored DAGs (no LP/SMT dependency). Vocabulary growth (ratio rows, bands) voids the lemma — revisit trigger named in Deferred. |
| D9 | `determined(a,b)`: `SignRange` of `f` over the pair joint region via the pinned extremum algorithm (§1): closure-vertex enumeration (box corners + coupling-boundary∩box-edge intersections + infinite limits) → open-interval range rule | Web review: "corner/limit analysis" alone under-specified — the coupling boundary `v_A = v_B` can define the infimum where no box corner does. Open convex region + non-constant linear `f` ⇒ range is the OPEN interval `(inf, sup)` of the closure extrema, so `Mixed ⇔ inf < 0 < sup` with no attainment bookkeeping; constant/degenerate rows handled explicitly. Returns `SignRange { NegativeOnly, ZeroOnly, PositiveOnly, Mixed }` — never pretends open-set infima are attained. |
| D10 | `hypothetical_yield` returns a **signed delta**; negative is real (hypothetical contradiction quarantines evidence on recompile) | Honest accounting; no second propagation engine — recompile via `compile` (pure, evidence-sized). Queue admission filters `guaranteed_yield > 0`; the negative case is exercised by test, not hidden. |
| D11 | Guaranteed yield = min over **order-bearing** answers (`prefer-a/prefer-b/equal`); `incomparable` disclosed separately in `yield_by_answer`; every entry carries `yield_basis` naming the answer space its min ranges over | Strict min over the full v2 vocabulary degenerates: `incomparable` is always answerable and always yields 0 ⇒ every comparison scores 0. RFC's "certain progress whatever the human says" is read against the order-bearing space; the JSON discloses the exception rather than hiding it. `yield_basis` (web review): comparison = `order-bearing-answers`, anchor-review = `canonical-resolving-actions` — the numbers stay spine-comparable for ranking, but a curator is told the semantics differ. |
| D12 | Anchor-review candidates: one per distinct suspect anchor from `AnchorConflict` quarantine pairs; answer space {anchor-removed, rows-retired}; guaranteed yield = min over **resolving** answers; **not K-gated** | Eval C2: one stale anchor sterilised 28% of the ledger via D4 closure — evidence budget must not pool in quarantine. Removal is the *optimistic* revise model (an edit to a still-conflicting value un-forces nothing), so a strict min over "any edit" degenerates exactly as `incomparable` does for comparisons — same cure as D11: min over answers that *resolve* the tension (revise-to-consistent ≈ removal; uphold = rows retired); a still-conflicting re-author is a non-resolution, disclosed, outside the min. `RowsRetired` set pinned precisely (web review): the COMPLETE set of rows cited in that suspect anchor's `AnchorConflict` quarantine entries — deliberately pessimistic (a user may supersede one stale row and restore most of the closure; the model retires all of it, so real uphold yield ≥ modelled). Impact still weights by frontier proximity of liberated rows. |
| D13 | Ranking: `score = guaranteed_yield × guaranteed_impact × confirm_boost`; `guaranteed_impact` = **min over the argmin-yield answers** of Σ rank-decay weights over that answer's newly-determined pairs (RV-269 F-2: worst-case-coherent and answer-space invariant — never a token-spelling tiebreak); `confirm_boost > 1` iff both participants' constraining evidence is agent-only (`RaterCounts.human == 0`) — reason code/text says exactly that: `agent-only-calibration`, "both items currently calibrated only by agent evidence" | Q5/T7 without D7 semantics: agent rows still determine; the boost only biases selection order toward regions where no human has spoken. Web review: the predicate does NOT establish that this candidate confirms a load-bearing agent-authored ordering (that needs dependency tracing of determinacy on agent-only edges — a distinct future candidate type, Deferred); the reason wording claims only what `constraining_counts_by_class` knows. Boost-vs-yield interaction explicitly accepted: a tuned boost CAN outrank a yield gap — pinned by a policy golden at current constants, not fenced by an invariant (ADR-015 numeric posture). Numeric shapes (`ELICIT_RANK_DECAY`, `ELICIT_CONFIRM_BOOST`) are named consts, implementation-owned tuning. Tiebreak: id lexicographic; scores compared `total_cmp`. |
| D14 | **Median-probe calibration** (renamed from "binary insertion" — web review): an un-constrained top-K item (zero constraining rows, no anchor) yields a comparison candidate against the projected median of its comparable set; stateless — each refresh re-derives the next probe from what the ledger now knows | Honest naming: this is median-guided calibration, NOT guaranteed logarithmic bisection — projection spacing is conventional, graphs branch, quarantines move values, `equal` merges classes, so successive probes need not halve a well-defined ordered set. Reason code `median-probe`. No session state, no cursor. |
| D15 | Queue states, precedence pinned: entries non-empty ⇒ `candidates` (all three sources count — anchor-review/median-probe can admit while every top-K pair is determined); entries empty ∧ every top-K pair determined ⇒ `stable`; entries empty ∧ some pair indeterminate ⇒ `stalled`. Stall render names the depth and disclaims stability; `stable` only via the determinacy predicate, and the claim is **internal-order stability**: "value_dim order among the CURRENT top-K frontier items is stable" — never prefix-membership stability | RFC stall ≠ stable (zero one-step yield can hide a bridge question). Exhaustion and stability are different facts; the render says which. A determined top-K with a live stale-anchor suspect is NOT "stable" — the tension is standing evidence-debt. Web review: the algorithm establishes order stability among current members only; whether an outside item can DISPLACE into the top-K is a full-score question (risk/leverage/burndown coupling) that D5 already cut — claiming membership stability would contradict D5. Challenger-fringe extension (pairs against K+1‥K+F) is a named deferred seam. |
| D16 | JSON schema v1 (§3): versioned envelope, kind-tagged entries, common spine (`rank/kind/guaranteed_yield/impact/score/reasons/ask`), kind payloads under distinct keys, structured `code`+`text` reasons; **lean participants** (ids + value/estimate block; no body summaries) | Curator sorts/filters on the spine without kind-switching; findings' JSON-parity idiom. Summaries are additive schema-versioned fields if ever wanted; human render fetches context itself. |
| D17 | Bare-estimate mask (Q4/C1) is an annotation on participants (`projection masked by bare estimate`), never a candidate kind; no engine yield-ranking of estimate questions | Phase E gate (estimate feasible-region model, RV-260 F-5). Curator nominates estimates; engine only discloses the mask. |
| D18 | `elicit` is read-only end to end: no runtime state, no persisted queue, derived output only | Capture loop = ledger round-trip; estimate/value edits re-surface reprobes through the same determinacy check with no clock and no staleness heuristic. |

## §1 Query API (`comparison/query.rs`)

Pure leaf: imports `compile` (and `wire` types) only. No priority imports, no
disk/clock/rng.

### Feasible region (normative)

An assignment maps each equality class to one real. Feasible iff every anchored
class sits exactly at its anchor and every retained strict edge `winner >
loser` holds. Nothing else constrains it: values may be negative (D7);
`incomparable` rows constrain nothing; quarantined rows are outside the
retained set by construction (C3/C4 ran in `compile`).

Costs: each item's scalar `est_cost` (the engine's own resolution — estimate
midpoint under skew β, bare-anchor `max_upper + margin` ctx, floored at
`EPSILON`, hence always > 0). Not varied inside the region (D7).

### Marginal exactness (D8) and the pair joint region

For the shipped vocabulary — strict order edges + point anchors, no gap
arithmetic — the feasible marginal of a class is exactly its C6 interval, and
for a pair `(A, B)` the joint region is exactly

```
J(A,B) = interval(A) × interval(B) ∩ coupling(A,B)
coupling ∈ { v_A > v_B   (reach: A ⇝ B in the condensed DAG),
             v_A < v_B   (B ⇝ A),
             v_A = v_B   (same class),
             ⊤           (order-incomparable) }
```

Reachability is computed once per refresh over the retained class DAG
(memoised — one forward pass per top-K class; the retained graph is a DAG
post-C3) and shared across all pair checks and hypotheticals.

**Proof of the lemma (D8 — the load-bearing claim).** Setting: retained class
DAG `G` (satisfiable, C5), anchor function `α` on some classes. A point-anchor
system over strict edges is satisfiable iff for every anchored pair `P ⇝ Q`,
`α(P) > α(Q)` (standard: topologically assign unanchored classes strictly
between the max assigned/anchored value strictly below and the min strictly
above — dense reals always leave room in a non-empty open interval, and C5
gives non-emptiness).

*Augmentation step.* Claim: anchoring one unanchored class `X` at any
`x ∈ (l_X, u_X)` (its C6 interval against the current anchor set) preserves
satisfiability. `u_X` is the *minimum* anchor above, so every anchored
`P ⇝ X` has `α(P) ≥ u_X > x`; symmetrically every anchored `X ⇝ Q` has
`α(Q) ≤ l_X < x`; anchored pairs not involving `X` are untouched. So the
augmented system satisfies the anchor-pair criterion. ∎(step)

*Pair extension.* Take `(x, y) ∈ box(A) × box(B) ∩ coupling`. Anchor `A` at
`x` — valid by the step. Now anchor `B` at `y` against the *augmented* system:
`B`'s interval may have tightened only via the new anchor `x` on `A` — i.e.
only when `A ⇝ B` (new upper `min(u_B, x)`) or `B ⇝ A` (new lower
`max(l_B, x)`) or same class (forced `y = x`). In each case the coupling term
supplies exactly the missing inequality (`y < x`, `y > x`, `y = x`
respectively); incomparable classes tighten nothing. So `y` lies in `B`'s
augmented interval, the step applies again, and the doubly-augmented system is
satisfiable — a full feasible assignment extending `(x, y)` exists. The
reviewer's shared-intermediate case is covered without special handling: an
anchored `Z` with `A ⇝ Z ⇝ B` bounds BOTH boxes (`l_A ≥ α(Z)`, `u_B ≤ α(Z)`),
and an unanchored intermediate keeps a non-empty open interval by C5 +
density. ∎

The lemma is vocabulary-scoped: strict edges + point anchors only. Ratio rows
or band constraints void it (Deferred). Backing test: property suite with a
naive backtracking extension oracle over generated small anchored DAGs — the
production path stays closed-form; the oracle is test-only, no LP/SMT
dependency.

### `determined`

```rust
pub(crate) struct PairSide {
  pub class: ClassId,
  pub eff_weight: f64,        // m_i · c_other  (D6; m_i ≥ 0, c > 0)
  pub bounds: ValueBounds,    // C6 interval
  pub anchor: Option<f64>,
}
pub(crate) enum SignRange { NegativeOnly, ZeroOnly, PositiveOnly, Mixed }   // web review: never
pub(crate) fn determined(reach: &Reachability, a: &PairSide, b: &PairSide) -> SignRange
// pretends an open-set infimum is attained; Determined ⇔ !Mixed
```

`f(v_A, v_B) = a.eff_weight·v_A − b.eff_weight·v_B`. Pinned extremum
algorithm (web review — "corner analysis" alone is insufficient: under
`v_A > v_B` the coupling boundary `v_A = v_B` can define the infimum where no
box corner does):

1. **Degenerate rows first.** Both weights zero ⇒ `ZeroOnly`. Same-class
   coupling ⇒ substitute `v = v_A = v_B`: `g(v) = (w_A − w_B)·v` over the
   class interval — 1-D sign read (point interval when anchored).
2. **Closure extrema.** Over the CLOSURE of `J` (closed box ∩ closed
   half-plane), a linear `f` attains its extrema on the finite vertex set:
   box corners, intersections of `v_A = v_B` with box edges, plus directional
   limits `±∞` for each `Unbounded` side (evaluate `f`'s growth sign along
   the recession directions). Enumerate; take `inf`, `sup` (either may be
   `±∞`).
3. **Open-interval range rule.** `J`'s interior is open and convex and `f` is
   non-constant there, so `f(J°) = (inf, sup)` exactly — attainment
   bookkeeping is unnecessary: `Mixed ⇔ inf < 0 < sup`; `PositiveOnly ⇔
   inf ≥ 0` (with `sup > 0`); `NegativeOnly ⇔ sup ≤ 0` (with `inf < 0`);
   boundary zeros of the closure are outside the strict region.

Pinned edge cases: same-class pair, differing weights, interval spanning 0 ⇒
`Mixed` (sign flips with the value's sign); unbounded side with differing
weights ⇒ `Mixed` (limits dominate); coupling-boundary infimum case (golden —
the extremum on `v_A = v_B`, no corner attains it); both-anchored ⇒ point
evaluation.

### `hypothetical_yield` (D10)

```rust
pub(crate) enum Hypothetical<'a> {
  Answer(Judgement),                   // synthetic order-bearing row (D11)
  AnchorRemoved(&'a str),              // anchor-review: edit modelled as removal (D12)
  RowsRetired(&'a BTreeSet<RowUid>),   // anchor-review: uphold — conflicting rows dropped
}
pub(crate) fn hypothetical_yield(
  active: &[&Judgement], anchors: &AnchorMap,
  hypo: &Hypothetical, relevant: &[(PairSide, PairSide)],
) -> i64   // |determined_after| − |determined_before| over `relevant`; signed
```

Recompiles via `compile` (pure, evidence-sized — no second propagation
engine); rebuilds reachability for the hypothetical set; counts. Negative
deltas are real (a contradicting hypothetical quarantines structure).

### `indeterminate_pairs`

```rust
pub(crate) fn indeterminate_pairs(reach: &Reachability, pool: &[PairSide]) -> Vec<(ClassId, ClassId)>
```

All-pairs over the pool (≤ K(K−1)/2), filtered by `determined`.

`compile.rs` gains at most narrow read accessors (class-of, edge iteration)
— no semantic change; the existing suites are the proof (behaviour-preservation
gate).

## §2 Queue assembly (`priority/elicit.rs`)

Pure over `(Pipeline, frontier order, est_cost map, statuses, EliqCfg)`. The
one impure load stays at the command shell: `surface` scan +
`load_comparison_pipeline` (existing seams; nothing new touches disk).

```rust
pub(crate) enum DecisionContext { Sequencing { depth: usize } }   // D3; Scoping is Phase E's slot
pub(crate) struct ElicitQueue { pub state: QueueState, pub entries: Vec<QueueEntry> }
pub(crate) enum QueueState { Candidates, Stalled { depth: usize }, Stable { depth: usize } }
pub(crate) enum CandidateKind { Comparison, AnchorReview }
pub(crate) struct QueueEntry {
  pub kind: CandidateKind,
  pub guaranteed_yield: i64, pub guaranteed_impact: f64, pub score: f64,
  pub yield_basis: YieldBasis,         // OrderBearingAnswers | CanonicalResolvingActions (D11)
  pub reasons: Vec<Reason>,            // { code, text } — findings JSON-parity idiom
  pub payload: EntryPayload,           // Comparison { a, b, ask } | AnchorReview { subject, exits }
}
pub(crate) fn assemble(inputs: &ElicitInputs, ctx: &DecisionContext) -> ElicitQueue
```

Candidate pool, three sources:

1. **Comparison pairs** — `indeterminate_pairs` over the top-K frontier items
   (K from `DecisionContext::Sequencing`), filtered by the existing capture
   admissibility (same gate `compare record` applies — no second rule set).
2. **Median-probe calibration (D14)** — top-K item with zero constraining
   rows and no anchor: candidate against the projected median of its
   comparable set, reason `median-probe`. Stateless; each refresh re-derives
   the probe from the ledger. Heuristic, not bisection (D14 rationale).
3. **Anchor-review (D12)** — one candidate per distinct suspect anchor
   appearing in `AnchorConflict` quarantine pairs. Answer space per D12;
   guaranteed yield = min over the two *resolving* outcomes (revise-to-
   consistent modelled as removal; uphold as rows retired). Surface honesty
   (RV-269 F-3): the answer token is `revise-anchor`, and its yield is
   explicitly conditional — the exit command (`value set`) admits
   non-resolving values, so JSON carries a `yield_note` and the render says
   "assumes the revision clears the conflict; a still-conflicting value
   re-surfaces this candidate". A non-resolving revision is a non-answer, not
   counted progress. Not K-gated; impact weights by frontier proximity of the
   rows each outcome would liberate.

Ranking (D13): `score = guaranteed_yield × guaranteed_impact × confirm_boost`;
admission `guaranteed_yield > 0`; per answer, `impact = Σ w(r)` over that
answer's newly-determined pairs, `w(r) = 1/(1 + r)` with `r` = the better
frontier rank in the pair (named const shape `ELICIT_RANK_DECAY`);
`guaranteed_impact` = min impact over the answers attaining the min yield
(RV-269 F-2 — worst-case both in count and in placement; invariant under
answer-token renames); `confirm_boost = ELICIT_CONFIRM_BOOST` iff both sides'
constraining evidence is agent-only via `constraining_counts_by_class`, else
`1.0`. Determinism: BTree everywhere,
`total_cmp` on scores, id-lexicographic tiebreak; no float in any key.

States (D15, precedence pinned): entries non-empty ⇒ `Candidates`; entries
empty ∧ every value-sensitive top-K pair `determined` ⇒ `Stable`; entries
empty ∧ some pair indeterminate ⇒ `Stalled` (render: "greedy one-step yield
exhausted at depth K — not a stability claim; bridge questions may exist").
All three candidate sources gate `Stable`: a determined top-K with a live
stale-anchor suspect stays `Candidates` — the tension is standing
evidence-debt. The `Stable` claim is internal-order only (D15): "value_dim
order among the current top-K frontier items is stable" — membership
displacement from outside K is a full-score question, out of scope by D5.
`m = 0` exclusions scope the claim further and are disclosed (D6).

Hypothetical-row hygiene (web review — implementation trap): synthetic
answer rows carry a fresh synthetic session identity so they can never
trigger R3 within-session implicit supersession against real rows; pinned by
test.

Cost (RV-269 F-1 — honest bound, wider than the RFC's): ≤ K(K−1)/2 + S
candidates, S = distinct suspect anchors; ≤ 3 recompiles each, linear in
active rows; reachability memoised per hypothetical. Total O((K² + S)·|active|)
— NOT the RFC's bare O(K²·|active|), because anchor-review is deliberately not
K-gated (D12). The extra term is **evidence-bounded, not corpus-scaled**: a
suspect anchor exists only where an `AnchorConflict` quarantine pair cites it,
so S ≤ |quarantined rows| ≤ |ledger| — the refresh stays evidence-sized end to
end. VT asserts completion on the eval corpus (32 rows, K = 8), not
benchmarks; the preflight assumption stands unless a corpus falsifies it.

Bare-estimate mask (D17): a participant with a projected/gauge value and no
estimate facet carries annotation `projection masked by bare estimate` (its
`value_dim` is denominator-anchored regardless of projection — eval C1). JSON
+ render; no findings variant unless the render genuinely needs the shared
idiom (lean default: none).

## §3 Surfaces & capture loop

### CLI

```
doctrine compare elicit [--depth K] [--limit N] [--kind comparison|anchor-review] [--json]
```

New `Elicit(ElicitArgs)` arm in `commands/compare.rs` (thin shell). `--depth`
overrides `[priority.elicit] depth` (`ELICIT_DEPTH = 8` default, clamp idiom
per `gauge.step`); `--limit` caps display (`ELICIT_LIMIT = 5`) — the full pool
is still computed (ranking needs it); `--kind` filters entries post-ranking.
No TTY interaction (D1): every rendered entry carries the exact answer
command; the capture loop is a ledger round-trip (D18).

### Human render

Existing `render.rs` structured idiom. Per entry: rank, kind, ask line,
participants with fetched context — title, status, S3 value-source shapes
reused verbatim, estimate-or-bare (with mask ⚠), deps/risk one-liner —
reasons, answer command. Footer = state line (D15 wording; `Stable` says
"**value_dim order among the current top-K frontier items** stable over the
joint set" — D5/D15 precision: internal order, current members, never
prefix-membership; scoped further when `m = 0` exclusions exist, D6).

### JSON schema v1 (D16)

```json
{
  "schema": "doctrine.elicit-queue",
  "version": 1,
  "context": { "kind": "sequencing", "depth": 8 },
  "state": "candidates",
  "state_detail": "…",
  "entries": [
    {
      "rank": 1, "kind": "comparison",
      "guaranteed_yield": 3, "guaranteed_impact": 2.4, "score": 7.2,
      "yield_basis": "order-bearing-answers",
      "reasons": [ { "code": "indeterminate-frontier-pair", "text": "…" },
                   { "code": "agent-only-calibration", "text": "both items currently calibrated only by agent evidence" } ],
      "participants": [
        { "id": "IMP-280",
          "value": { "provenance": "projected", "point": 2.6,
                     "bounds": { "lower": { "kind": "open", "value": 2.5 },
                                 "upper": { "kind": "open", "value": 2.8 } } },
          "estimate": null,
          "annotations": ["projection masked by bare estimate"] },
        { "id": "IMP-270", "value": { "…": "…" }, "estimate": 3.5, "annotations": [] }
      ],
      "ask": { "frame": "equal-effort", "domain": "value",
               "answers": ["prefer-a", "prefer-b", "equal", "incomparable"],
               "yield_by_answer": { "prefer-a": 3, "prefer-b": 4, "equal": 5, "incomparable": 0 } }
    },
    {
      "rank": 2, "kind": "anchor-review",
      "guaranteed_yield": 6, "guaranteed_impact": 1.9, "score": 11.4,
      "yield_basis": "canonical-resolving-actions",
      "reasons": [ { "code": "stale-anchor-suspect", "text": "…" } ],
      "subject": { "id": "IMP-274", "anchor": 5.0,
                   "conflict_pairs": [["IMP-198", "IMP-274"]],
                   "quarantined_rows": ["uid…"] },
      "ask": { "answers": ["revise-anchor", "uphold-anchor"],
               "yield_by_answer": { "revise-anchor": 6, "uphold-anchor": 2 },
               "yield_note": "revise-anchor yield assumes a RESOLVING revision (conflict removed); a still-conflicting value yields nothing and re-surfaces this candidate next refresh. uphold-anchor models retiring the COMPLETE cited closure — real yield may exceed it",
               "exits": { "revise-anchor": ["doctrine value set IMP-274 <v>"],
                          "uphold-anchor": ["doctrine compare record … --supersedes <uid>",
                                            "doctrine compare withdraw <uid>"] } }
    }
  ]
}
```

Byte-stable (BTree ordering); `null` estimate ⇒ bare. Bounds are structural —
`{ kind: open|closed|unbounded, value? }` mirrors the `Bound` enum (web
review: `[null, 2.8]` loses open/closed semantics). `guaranteed_impact` is
the D13 min-over-argmin-yield-answers value — named exactly. `yield_basis`
names the answer space each entry's min ranges over (D11) — spine numbers
stay comparable for ranking, semantics disclosed. `exits` values are arrays
of suggested actions (uphold is not one executable command). Participants
are lean (no titles/summaries in JSON — curator reads entities; additive
version-gated fields later if wanted).

## §4 Code impact (design-target selectors)

| path | change |
|---|---|
| `src/comparison/query.rs` | new — §1 predicates + reachability |
| `src/comparison/mod.rs` | `mod query` + re-exports |
| `src/comparison/compile.rs` | narrow read accessors only; no semantic change |
| `src/priority/elicit.rs` | new — §2 queue assembly |
| `src/priority/config.rs` | `ELICIT_DEPTH`, `ELICIT_LIMIT`, `ELICIT_RANK_DECAY`, `ELICIT_CONFIRM_BOOST` + `[priority.elicit]` parse |
| `src/priority/surface.rs` | bare-estimate mask plumb if S3 block carries it |
| `src/commands/compare.rs` | `Elicit` arm, render, JSON emit |
| `src/priority/mod.rs` | `mod elicit` |

## §5 Verification plan

Suites → rules pinned. VT/VA/VH ids minted at `/plan`.

1. **Determinacy battery** (query.rs unit): chain-ordered pair with equal
   weights ⇒ determined; chain-ordered, cheaper winner ⇒ determined
   (positive-interval case); chain-ordered, costlier winner, positive
   intervals ⇒ indeterminate unless anchor squeeze; interval-box-overlapping
   but chain-coupled pair ⇒ determined (the box is not the oracle — coupling
   term does the work); both-anchored ⇒ point evaluation; same-class +
   differing weights + interval spanning 0 ⇒ indeterminate (negative-domain
   golden, D9); unbounded-side limits; `ZeroOnly` constant-zero;
   coupling-boundary infimum golden (extremum on `v_A = v_B`, unattained at
   any box corner — web review); D8 property suite: naive backtracking
   extension oracle over generated small anchored DAGs (multiple anchored
   ancestors/descendants, shared intermediates, open + unbounded sides,
   equality merges) confirms every claimed joint-region point extends.
2. **Yield battery** (hand-computed small graphs): guaranteed yield =
   min-over-order-bearing-answers of newly-determined relevant pairs; `equal`
   answer merging classes; hypothetical `equal` between differently-anchored
   classes ⇒ C2 quarantine on recompile, negative delta accounted (web
   review); negative delta when a hypothetical contradicts a chain (D10);
   zero-yield bridge case ⇒ `Stalled`, not `Stable` (D15); `incomparable`
   yields 0 and is excluded from the min but disclosed (D11); synthetic
   hypothetical rows carry a fresh session identity — never trigger R3
   within-session supersession against real rows (web review).
3. **Reprobe** (elicit + pipeline integration): synthetic stale-anchor
   conflict ⇒ anchor-review candidate ranked by un-quarantine payoff;
   `AnchorRemoved` hypothetical activates closure rows on recompile;
   `RowsRetired` counts its (smaller) liberation; min-over-resolving-answers
   (D12) pinned by a case where uphold < removal; `RowsRetired` = the
   complete cited closure of the suspect anchor, pinned (web review);
   estimate edit flips a determined pair back indeterminate ⇒ reprobe
   comparison resurfaces (D18); determined top-K + live suspect anchor ⇒
   state stays `Candidates` (D15 precedence golden); internally-determined
   top-K with an indeterminate K-vs-(K+1) relation still renders `Stable`
   with the internal-order wording — the wording golden pins that the claim
   is member-scoped (web review).
4. **Median-probe calibration**: un-constrained top-K item ⇒ candidate vs
   projected median, reason `median-probe`; successive-refresh golden on a
   *branching* gauge graph demonstrating the heuristic (probes narrow but
   need not halve — documented behaviour, not a bisection contract; D14).
5. **Ranking**: confirm-boost all-else-equal pair (agent-only outranks
   human-touched); impact rank-decay ordering; guaranteed-impact min over
   argmin-yield answers pinned by a case where two answers tie on yield with
   different impacts (RV-269 F-2); admission filter drops non-positive
   guaranteed yield; id tiebreak; zero-multiplier pair (`m = 0` via zeroed
   coefficient/kind_weight/tag_term) excluded from the pool AND the stability
   obligation AND annotated, incl. the one-zero-weight case over a
   negative-spanning interval (RV-269 F-4, D6, web review); policy goldens:
   high-yield/low-impact vs low-yield/prominent-pair ordering, and a
   boost-outranks-yield-gap case at current constants (documented behaviour,
   D13).
6. **Determinism**: same merged file set + statuses + config + invocation
   params ⇒ byte-identical queue and `--json` output; shuffled session-file
   load order invariance (extends the SL-213 suite).
7. **Surfaces**: render goldens for all three states (stall wording names
   depth and disclaims stability; stable wording says "value_dim order");
   bare-estimate mask on projected-but-bare participant, absent on estimated;
   JSON schema goldens for both kinds incl. the anchor-review `yield_note`
   conditional-yield disclosure (RV-269 F-3); `--kind` filter; `--limit`
   display cap with full-pool ranking.
8. **Behaviour preservation**: no ledger / no invocation ⇒ every existing
   priority + comparison suite passes unchanged; compile/project semantics
   untouched (accessors only).
9. **Cost ceiling**: eval corpus (32 rows, K = 8) completes within the
   existing test-time envelope — assertion of completion, not a benchmark.

## RFC-019 deviations (design-stage, recorded)

1. **Verb** — `compare elicit`, not the RFC sketch `value elicit` (D1;
   SL-210 D1 precedent post-dates the sketch).
2. **Guaranteed yield over order-bearing answers** (D11) — the RFC's
   "minimum over the answers" degenerates under the v2 vocabulary
   (`incomparable` always yields 0); refined, with JSON disclosure.
3. **Anchor-review candidates** (D12) — deliberate scope addition over the
   RFC's Phase C text; eval C2 (28% sterilisation) is the warrant, sanctioned
   at slice scoping (Q3).

## Resolved OQs

- Scope OQ "verb naming" → D1. OQ "K ownership" → D2/D3. OQ "queue-entry
  JSON schema" → D16. OQ "predicate API home" → D4.
- Carried assumptions confirmed: recompile-per-hypothetical within budget
  (§2 cost, VT 9); one kind-tagged queue surface (D16); C3 cycle-path
  assurance stays the SL-213 prototype battery (unchanged here).

## Review history

- **Internal adversarial pass** (2026-07-12) — three fixes: D12 anchor-edit
  yield reframed min-over-resolving-answers; D13 argmin tie pinned; D15 state
  precedence pinned (all three candidate sources gate `Stable`).
- **RV-269, codex GPT-5.5** (2026-07-12, hostile, ledgered) — verdict
  approve-after-fixes; all four findings accepted fix-now: F-1 *major* (cost
  bound understated — restated O((K² + S)·|active|), S evidence-bounded);
  F-2 *major* (lexicographic argmin tie made score token-spelling-dependent —
  replaced with guaranteed-impact = min over argmin-yield answers); F-3
  *major* (`edit-anchor` yield hid non-resolving edits — token `revise-anchor`,
  conditional yield disclosed via `yield_note` + render wording); F-4 *minor*
  (D6 zero-multiplier branch unpinned — VT added, §5.5). D8 lemma, D5 scope
  cut, negative-domain honesty, layering survived attack unfound.
- **Web GPT-5.5 pass** (2026-07-12, same design version post-RV-269) —
  verdict: approve with five mandatory clarifications, all accepted: D8
  proof written into §1 (constructive augmentation + induction; backed by a
  test-only backtracking extension oracle, no LP/SMT dep); D9 pinned as an
  explicit extremum algorithm returning `SignRange` (closure-vertex
  enumeration + open-interval range rule; coupling-boundary infimum golden);
  stability claim narrowed to internal-order among current top-K members
  (prefix-membership is a full-score question D5 already cut;
  challenger-fringe deferred); anchor-review yield distinguished via
  `yield_basis` + `RowsRetired` pinned to the complete cited closure
  (pessimistic, stated); "binary insertion" renamed median-probe calibration
  (heuristic, not bisection). Further accepted: `confirm_boost` reason
  narrowed to `agent-only-calibration` (participant-level counts are all it
  knows; true confirmation candidates deferred); boost-can-outrank-yield-gap
  explicitly accepted + policy goldens; D6 `m = 0` pairs excluded from the
  stability obligation with scoped render; JSON hardening (schema id string
  + version int, structural open/closed bounds, `guaranteed_impact` named
  exactly, exits as action arrays); verification additions §5.1/2/3/4/5
  (extension oracle, boundary extremum, outsider wording golden, one-zero
  negative-domain, closure label, branching median-probe, fresh-session
  hypothetical identity, anchored-unequal `equal` quarantine). Lean-JSON
  participant reads tolerated (prior user call; additive fields later).

## Deferred (named seams, not built)

- `DecisionContext::Scoping { budget }` — Phase E (estimate feasible-region
  model is its entry criterion, RV-260 F-5); slots into D3's seam.
- D7 rank-aware quarantine / agent-testimony demotion — pre-Phase-D
  obligation; `confirm_boost` (D13) is selection-order bias only, no
  determinacy semantics.
- Engine yield-ranking of estimate questions — Phase E; curator nomination
  until then (D17).
- REQ kinds join `VALUE_BEARING` — IMP-281 (surfaced at this design's Q2;
  sequence before/with Phase E).
- D8 marginal-exactness lemma is vocabulary-scoped: ratio rows or band
  constraints void it — the phase admitting them must revisit `determined`
  (LP or richer propagation).
- Challenger-fringe extension (pairs against K+1‥K+F) toward prefix-
  membership stability — blocked on the full-score determinacy question D5
  cut; internal-order is the honest shipped claim (D15, web review).
- True human-confirmation candidate kind (dependency tracing of determinacy
  on agent-only edges — offers a *determined* pair for confirmation) —
  distinct from D13's `agent-only-calibration` selection bias; RFC territory
  ("determined pairs may still deserve human attention"), post-D7.
- Curation skill over the JSON surface — ships as skill text once the queue
  exists (Q1 posture).
