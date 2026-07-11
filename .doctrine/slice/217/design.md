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
| D6 | Static multipliers fold into effective weight: pair objective `f = m_A·c_B·v_A − m_B·c_A·v_B` | Live `value_dim` = coeff × kind_weight × tag_term × v / est_cost; the multipliers are per-item constants ≥ 0, so the objective stays 2-variable and closed-form. `m = 0` (zeroed coefficient/weight) ⇒ pair excluded, annotated. |
| D7 | Feasible region: one real per class, anchors exact, strict edges hold, **no positivity assumption**; costs are the scalar `est_cost` (bare-anchor ctx included), outside the region | Negative anchors are legal (`facet.rs` vt10 pins `value = -5.0`). RV-260 F-5: estimate uncertainty is not part of the region; an estimate edit moves `est_cost` and re-runs determinacy (the reprobe dynamic). |
| D8 | Marginal exactness lemma: per-class feasible marginal = its C6 interval; pair joint region = `box(A) × box(B) ∩ coupling` (coupling from condensed-DAG reachability / same-class) | For strict-order edges + point anchors over dense reals, any point of that set extends to a full feasible assignment (no gap arithmetic, D8 of SL-213; intermediates always fit). This is what makes joint-set determinacy closed-form — no LP. The SL-213 "box is not the oracle" warning is about *decisions from marginals without the coupling term*; box + coupling **is** the joint set for a pair, for this vocabulary. Vocabulary growth (ratio rows, bands) voids the lemma — revisit trigger named in Deferred. |
| D9 | `determined(a,b)`: sign-constancy of `f` over the pair joint region via corner/limit analysis; constant zero = `Tied`; open bounds are strict limits | Sup/inf of a linear 2-variable function over a box-with-one-coupling: corners + unbounded limits. Edge cases pinned as goldens: same-class pair with differing weights and interval spanning 0 ⇒ indeterminate; any `Unbounded` side with differing weights ⇒ indeterminate (limits dominate). |
| D10 | `hypothetical_yield` returns a **signed delta**; negative is real (hypothetical contradiction quarantines evidence on recompile) | Honest accounting; no second propagation engine — recompile via `compile` (pure, evidence-sized). Queue admission filters `guaranteed_yield > 0`; the negative case is exercised by test, not hidden. |
| D11 | Guaranteed yield = min over **order-bearing** answers (`prefer-a/prefer-b/equal`); `incomparable` disclosed separately in `yield_by_answer` | Strict min over the full v2 vocabulary degenerates: `incomparable` is always answerable and always yields 0 ⇒ every comparison scores 0. RFC's "certain progress whatever the human says" is read against the order-bearing space; the JSON discloses the exception rather than hiding it. |
| D12 | Anchor-review candidates: one per distinct suspect anchor from `AnchorConflict` quarantine pairs; answer space {anchor-removed, rows-retired}; **not K-gated** | Eval C2: one stale anchor sterilised 28% of the ledger via D4 closure — evidence budget must not pool in quarantine. Anchor-edit hypothetical modelled conservatively as anchor *removal* (any edit un-forces the closure at least as much); uphold modelled as conflicting rows retired. Impact still weights by frontier proximity of liberated rows. |
| D13 | Ranking: `score = guaranteed_yield × impact × confirm_boost`; impact = Σ rank-decay weights over the min-yield newly-determined pairs; `confirm_boost > 1` iff both participants' constraining evidence is agent-only (`RaterCounts.human == 0`) | Q5/T7 without D7 semantics: agent rows still determine; the boost only biases selection order toward regions where no human has spoken. Numeric shapes (`ELICIT_RANK_DECAY`, `ELICIT_CONFIRM_BOOST`) are named consts, implementation-owned tuning (ADR-015 posture). Tiebreak: id lexicographic; scores compared `total_cmp`. |
| D14 | Binary insertion is stateless: an un-constrained top-K item (zero constraining rows, no anchor) yields a comparison candidate against the projected median of its comparable set; the ledger *is* the bisection state | No session state, no cursor; each refresh re-derives the next probe from what the ledger now knows. Reason code `binary-insertion`. |
| D15 | Queue states: `candidates` / `stalled` / `stable`; stall render names the depth and disclaims stability; `stable` only via the determinacy predicate over every top-K pair | RFC stall ≠ stable (zero one-step yield can hide a bridge question). Exhaustion and stability are different facts; the render says which. |
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

### `determined`

```rust
pub(crate) struct PairSide {
  pub class: ClassId,
  pub eff_weight: f64,        // m_i · c_other  (D6; m_i ≥ 0, c > 0)
  pub bounds: ValueBounds,    // C6 interval
  pub anchor: Option<f64>,
}
pub(crate) enum Determinacy { Determined(std::cmp::Ordering), Indeterminate }
pub(crate) fn determined(reach: &Reachability, a: &PairSide, b: &PairSide) -> Determinacy
```

`f(v_A, v_B) = a.eff_weight·v_A − b.eff_weight·v_B`; determined iff `f`'s sign
is constant over `J(A,B)` — sup/inf via corner and limit analysis (both
anchored ⇒ point evaluation; open bounds are strict limits; `Unbounded` sides
evaluate limits at ±∞). Constant zero ⇒ `Determined(Equal)`. Pinned edge
cases: same-class pair, differing weights, interval spanning 0 ⇒
indeterminate (sign flips with the value's sign); unbounded side with
differing weights ⇒ indeterminate.

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
  pub guaranteed_yield: i64, pub impact: f64, pub score: f64,
  pub reasons: Vec<Reason>,            // { code, text } — findings JSON-parity idiom
  pub payload: EntryPayload,           // Comparison { a, b, ask } | AnchorReview { subject, exits }
}
pub(crate) fn assemble(inputs: &ElicitInputs, ctx: &DecisionContext) -> ElicitQueue
```

Candidate pool, three sources:

1. **Comparison pairs** — `indeterminate_pairs` over the top-K frontier items
   (K from `DecisionContext::Sequencing`), filtered by the existing capture
   admissibility (same gate `compare record` applies — no second rule set).
2. **Binary insertion (D14)** — top-K item with zero constraining rows and no
   anchor: candidate against the projected median of its comparable set,
   reason `binary-insertion`. Stateless; the ledger is the bisection state.
3. **Anchor-review (D12)** — one candidate per distinct suspect anchor
   appearing in `AnchorConflict` quarantine pairs. Answer space per D12;
   guaranteed yield = min of the two outcomes. Not K-gated; impact weights by
   frontier proximity of the rows each outcome would liberate.

Ranking (D13): `score = guaranteed_yield × impact × confirm_boost`;
admission `guaranteed_yield > 0`; `impact = Σ w(r)` over the min-yield
newly-determined pairs, `w(r) = 1/(1 + r)` with `r` = the better frontier rank
in the pair (named const shape `ELICIT_RANK_DECAY`); `confirm_boost =
ELICIT_CONFIRM_BOOST` iff both sides' constraining evidence is agent-only via
`constraining_counts_by_class`, else `1.0`. Determinism: BTree everywhere,
`total_cmp` on scores, id-lexicographic tiebreak; no float in any key.

States (D15): `Candidates`; `Stalled` when the pool is non-empty but no
candidate admits (render: "greedy one-step yield exhausted at depth K — not a
stability claim; bridge questions may exist"); `Stable` when every top-K pair
is `determined` over the joint set — the only path to a stability claim.

Cost: ≤ K(K−1)/2 + |suspect anchors| candidates; ≤ 3 recompiles each, linear
in active rows; reachability memoised per hypothetical. O(K²·|active|) as the
RFC budgeted. VT asserts completion on the eval corpus (32 rows, K = 8), not
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
"top-K **value_dim order** stable over the joint set" — D5 precision).

### JSON schema v1 (D16)

```json
{
  "schema": 1,
  "context": { "kind": "sequencing", "depth": 8 },
  "state": "candidates",
  "state_detail": "…",
  "entries": [
    {
      "rank": 1, "kind": "comparison",
      "guaranteed_yield": 3, "impact": 2.4, "score": 7.2,
      "reasons": [ { "code": "indeterminate-frontier-pair", "text": "…" },
                   { "code": "human-confirmation", "text": "…" } ],
      "participants": [
        { "id": "IMP-280",
          "value": { "provenance": "projected", "point": 2.6, "bounds": [2.5, 2.8] },
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
      "guaranteed_yield": 6, "impact": 1.9, "score": 11.4,
      "reasons": [ { "code": "stale-anchor-suspect", "text": "…" } ],
      "subject": { "id": "IMP-274", "anchor": 5.0,
                   "conflict_pairs": [["IMP-198", "IMP-274"]],
                   "quarantined_rows": ["uid…"] },
      "ask": { "answers": ["edit-anchor", "uphold-anchor"],
               "yield_by_answer": { "edit-anchor": 6, "uphold-anchor": 2 },
               "exits": { "edit-anchor": "doctrine value set IMP-274 <v>",
                          "uphold-anchor": "supersede or tombstone: <uids>" } }
    }
  ]
}
```

Byte-stable (BTree ordering); `null` estimate ⇒ bare. Bounds render `null` for
`Unbounded` sides. Participants are lean (no titles/summaries in JSON —
curator reads entities; additive schema-versioned fields later if wanted).

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
   golden, D9); unbounded-side limits; `Tied` constant-zero.
2. **Yield battery** (hand-computed small graphs): guaranteed yield =
   min-over-order-bearing-answers of newly-determined relevant pairs; `equal`
   answer merging classes; negative delta when a hypothetical contradicts a
   chain (D10); zero-yield bridge case ⇒ `Stalled`, not `Stable` (D15);
   `incomparable` yields 0 and is excluded from the min but disclosed (D11).
3. **Reprobe** (elicit + pipeline integration): synthetic stale-anchor
   conflict ⇒ anchor-review candidate ranked by un-quarantine payoff;
   `AnchorRemoved` hypothetical activates closure rows on recompile;
   `RowsRetired` counts its (smaller) liberation; estimate edit flips a
   determined pair back indeterminate ⇒ reprobe comparison resurfaces (D18).
4. **Binary insertion**: un-constrained top-K item ⇒ candidate vs projected
   median, reason `binary-insertion`; after answering, next refresh probes the
   correct half (stateless bisection, D14).
5. **Ranking**: confirm-boost all-else-equal pair (agent-only outranks
   human-touched); impact rank-decay ordering; admission filter drops
   non-positive guaranteed yield; id tiebreak.
6. **Determinism**: same merged file set + statuses + config + invocation
   params ⇒ byte-identical queue and `--json` output; shuffled session-file
   load order invariance (extends the SL-213 suite).
7. **Surfaces**: render goldens for all three states (stall wording names
   depth and disclaims stability; stable wording says "value_dim order");
   bare-estimate mask on projected-but-bare participant, absent on estimated;
   JSON schema goldens for both kinds; `--kind` filter; `--limit` display cap
   with full-pool ranking.
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
- Curation skill over the JSON surface — ships as skill text once the queue
  exists (Q1 posture).
