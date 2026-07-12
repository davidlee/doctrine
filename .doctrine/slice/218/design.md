# SL-218 design — Tension narrative (RFC-019 Phase D)

Approved decisions from the design conversation (2026-07-12), mechanism
detail, code impact, verification. Constraints inherited: SL-217 D5 (claims
are `value_dim`-order claims, never full-score-order), D13 (`confirm_boost`
is selection-order bias only), D15 (internal-order stability wording);
RFC-019 T7 (agent trust policy); product-critique tensions #3 (agent
authority) and #6 (cardinal from ordinal); ADR-015 (composition semantics
unchanged); ADR-001 (layering).

## Decision ledger

| # | Decision | Rationale |
|---|---|---|
| D1 | D7 knob = **excluded-from-determinacy**: knob-on, every `determined()` verdict consumed by surfaces comes from a constraint system compiled from **human rows only**; agent rows keep constraining bounds, projection, and queue seeding. | RFC-019 T7's named variant that preserves the operator-and-agent product: agent evidence proposes orderings, never retires a question. Inert-until-confirmed guts projection; quarantine-on-rank needs the dependency tracing SL-217 parked post-D7. `confirm_boost` (D13) keeps its no-determinacy-semantics contract untouched. |
| D2 | Knob ships as plain config: `doctrine.toml [priority.compare] demote_agent_evidence` (bool, default `false` = current behaviour). Contract recorded here: **any stakeholder-facing surface (session mode, web elicitation — the OQ-1 slice) MUST require knob-on**; the gate lands with that slice, not this one. | T7: demotion is mandatory before stakeholder sessions treat the ordering as elicited truth. Default-off keeps SL-213/217 behaviour and goldens byte-identical. |
| D3 | D7 = **PHASE-01** of this slice, entrance for the narrative phases. | Contained change (human-subset compile + predicate variant + knob + disclosure); a separate slice repeats full ceremony for ~one phase and buys no isolation the seam maps show a need for. |
| D4 | Tension predicate = `value_dim`-order vs delivery-order inversions **over the rendered frontier top-K**, classified by cause: **Structure** (surviving seq/dep constraint forces the inversion; callout cites the edge) vs **Composition** (full-score dimensions — risk_dim, leverage, optionality — lift the surfaced item; callout cites component deltas). Wording never dresses a full-score claim as a value claim. | RFC-019 Phase D targets structure overrides ("never silently resolved"). Composition divergence is ADR-015 working as designed but surprising — accurate to show, only when asked (D5 of this ledger). D5/SL-217 compliance: divergence attributed to named score dimensions is not a value-order claim. |
| D5 | Render defaults: `explain` renders **both** classes; `next` renders **structure-only** by default, a verbosity flag additionally pulls composition callouts in. Flag spelling settled at implementation against existing `next` CLI conventions. | `explain` is the drill-down where "why is this above that" is being asked. Composition-divergence always-on in `next` would drown the structure signal. |
| D6 | Every callout carries an **evidence grade** for the value_dim ordering it asserts: `Determined` (joint-feasible-set predicate, knob-aware, with constraining-evidence counts) vs `Projected` (gauge spacing / default values — no determining evidence). | Critique #6: point-projected value_dim can invert on manufactured cardinal spacing the feasible set does not support; the render must not overclaim. Anchored pairs come out `Determined` through the same predicate (anchors are point constraints) — one predicate, no special case. |
| D7 | Detection = **pure fn** in new `src/priority/tension.rs`; the priority layer never imports `comparison` — evidence grades are computed at command tier and injected as data. Render via new `ReasonKind` arm(s); all human wording through `reason_line()` fragments (single source, REQ-072 AC3). | ADR-001 layering; pure/imperative split; REQ-072 render discipline. |

## §1 PHASE-01 — D7 demotion knob

### Current behaviour

`determined()` (`src/comparison/query.rs:152`) evaluates the pair joint
region over the constraint system compiled from **all** active rows; `rater`
provenance is carried (`RaterCounts`, `src/comparison/compile.rs:126`,
`constraining_counts_by_class` :156) but never consulted by determinacy.
Fourteen agent rows close a question exactly as fourteen human rows do
(critique #3). `confirm_boost` (`src/priority/elicit.rs:650`) biases
selection order toward agent-only regions but cannot reopen anything.

### Target behaviour

New config:

```toml
[priority.compare]
demote_agent_evidence = false   # default: current behaviour
```

Loaded alongside `ElicitConfig` (`src/priority/config.rs` precedent):
`CompareConfig { demote_agent_evidence: bool }`, key name single-sourced as a
named const (STD-001).

Knob-on semantics — **two compiles, one truth per question**:

- **Full system** (all active rows): unchanged; feeds bounds, projection
  (P1–P8), queue pool, `value_dim` consumed by scoring. Agent evidence keeps
  shaping what the engine *proposes*.
- **Human system**: a fresh, self-consistent compile over the row subset
  `rater == human` (anchors are authored ⇒ always present). Its own
  quarantine passes (C2–C4) run on the subset; a human row quarantined in
  the full system (e.g. cycling with agent rows) may legitimately survive
  here — each system's verdicts are honest per-system.
- Every **determinacy** consumer (elicit queue states/candidates,
  `hypothetical_outcome`, this slice's evidence grades) reads `determined()`
  over the **human system** when the knob is on.

Falls out free: agent-"closed" pairs read indeterminate under knob-on, so
they re-enter the elicit queue as live candidates — the human-confirmation
pressure T7 wants, with no new candidate kind (the true dependency-traced
confirmation kind stays deferred, SL-217 Deferred list).

Disclosure: when knob-on, `compare elicit` and `explain` surfaces append one
line — "agent evidence demoted: agent judgements propose orderings but do
not retire questions". Wording via a shared fragment, not per-surface
prose.

Cost: one extra compile + reachability build on paths that already recompile
per hypothetical; no algorithmic change.

### Invariants

- INV-1 (knob-off identity): `demote_agent_evidence = false` ⇒ all existing
  **comparison / elicit / inference** outputs byte-identical (SL-213/217
  suites are the proof — behaviour-preservation gate). Scope note: priority
  `next`/`explain` goldens change *intentionally* at PHASE-03 where tension
  callouts appear (knob-independent feature); those diffs are reviewed, not
  covered by this invariant. At PHASE-01 close, **all** existing suites are
  byte-identical knob-off.
- INV-2 (agent evidence never retires): knob-on, no pair whose determining
  constraint set requires an agent row reads `Determined`.
- INV-3 (D13 unchanged): `confirm_boost` still consulted only in selection
  scoring, both knob states.

## §2 PHASE-02 — tension detection

### Types (new `src/priority/tension.rs`, pure)

```rust
pub struct Tension {
  pub preferred: String,        // entity id ranked higher on value_dim
  pub surfaced: String,         // entity id delivery-order surfaces first
  pub cause: TensionCause,
  pub grade: EvidenceGrade,     // injected, computed at command tier
}

pub enum TensionCause {
  /// A surviving seq/dep path constrains `preferred` behind `surfaced`.
  Structure { edge: StructuralEdge },     // cites the concrete constraint
  /// No structural path; full-score dimensions lift `surfaced`.
  Composition { deltas: ComponentDeltas } // risk_dim / leverage / optionality diffs
}

pub enum EvidenceGrade {
  Determined { counts: RaterCounts },  // joint-set predicate, knob-aware
  Projected,                           // gauge spacing / defaults only
}
```

### Predicate

Inputs: the rendered frontier page `F` in delivery order (`frontier_order`,
`src/priority/order.rs:76`), per-node `value_dim` (`BaseScore`,
`src/priority/graph.rs:62`), surviving seq/dep edges
(`surviving_seq_predecessors`, order.rs:39, plus dep edges), score component
maps.

For each pair `(A, B)` with `pos(B) < pos(A)` in `F` and
`value_dim(A) > value_dim(B)`:

- **Structure** iff `A` is reachable from `B` over the **full** surviving
  seq/dep graph (transitive; NOT page-restricted — a path routing through a
  node outside the top-K is still structure, and page-restriction would
  misclassify it as Composition). Only the *pair* must be on the page. Cite
  the first edge on one such path.
- else, if full scores compare equal (`total_cmp`) — **excluded**: the
  inversion is an id-tiebreak artifact, not an override; rendering it would
  assert a cause that does not exist.
- else **Composition**; deltas = per-dimension score differences.

Ties (`value_dim` equal) are not tensions. Complexity `O(K²)` pair scan with
per-pair bounded reachability (DFS over surviving edges, memoized) —
trivial at page scale. Determinism: pairs emitted in delivery-order position
order; `total_cmp` for float comparisons (house style).

**`explain` considered set**: same default page as `next`. If the explained
id is on the page, its tensions (both classes) render; if not, the section
discloses "not on the current frontier — no tension analysis" rather than
inventing a hypothetical position for a non-surfaced item.

`value_dim` here is the point projection the engine actually consumed —
detection does not re-derive value. The *grade* (D6) is what keeps the
callout honest when the projection's ordering is not feasible-set-backed.

### Grade injection (command tier)

`run_next` / `run_explain` (`src/priority/mod.rs`) already own the impure
assembly. After detection returns pairs, command tier maps entity →
comparison class and calls the determinacy predicate (human system when
knob-on) **only for detected pairs** (≤ K² checks, no corpus sweep);
entities with no class / no evidence ⇒ `Projected`. Grades decorate the
`Tension` rows before they reach the view. Priority stays free of
`comparison` imports (ADR-001); comparison exposes one query fn taking pair
ids **plus per-item effective multipliers and costs**, returning grade data.

**Objective generalisation (multiplier correctness).** The grade asserts a
claim about `value_dim` order, and `value_dim = m·v / est_cost` where
`m = value_coeff × kind_weight × tag_multiplier` (ADR-015) — per-item
config-derived constants, `m ≥ 0` (tag floor). SL-217's determinacy
objective `v_A·c_B − v_B·c_A` tests raw `v/c` order, which multipliers can
invert relative to `value_dim` order. The pair-grade query therefore
evaluates the sign range of **`m_A·v_A·c_B − m_B·v_B·c_A`** over the joint
region — still linear in `(v_A, v_B)`, same closed-form machinery (SL-217
D8 lemma unaffected: the region is unchanged, only the objective's constant
coefficients differ). `m = 0` degenerates gracefully (term drops; sign from
the surviving term). Existing elicit-queue determinacy is NOT touched — it
keeps SL-217's raw objective and semantics; the generalised objective exists
only behind the pair-grade query this slice adds.

**Counts** in `Determined { counts }` come from the existing
`constraining_counts_by_class` seam (`src/comparison/compile.rs:156`),
summed over the pair's classes — the same provenance surface `explain`
already discloses ("bounds from 2 human + 14 agent judgements").

## §3 PHASE-03 — render

### Surfaces

- **`explain <id>`**: new "tensions" section listing every tension involving
  the id, both classes, after the score section.
- **`next`**: structure callouts under the affected rows, capped at
  `TENSION_MAX_CALLOUTS` (named const) per page; verbosity flag adds
  composition callouts. JSON always carries the full structured list
  (schema: `tensions: [{preferred, surfaced, cause, edge?, deltas?, grade,
  counts?}]`) — the cap is a human-render bound only.

### Wording (goldens pin these shapes)

Structure, determined:

> tension: SL-014 ranks above SL-009 on value_dim (determined — 3 human
> judgements); SL-009 surfaces first — `after SL-009` sequence survives.

Structure, projected:

> tension: SL-014 ranks above SL-009 on value_dim (projected order — no
> determining evidence); SL-009 surfaces first — `needs SL-009` holds.

Composition (explain / verbose next):

> SL-009 surfaces above SL-014 on full score (leverage +2.1, risk +0.8);
> on value_dim alone SL-014 ranks higher (determined — 2 human + 1 agent).

Constraints on wording: claims name `value_dim` explicitly (SL-217 D5);
never "stable"/membership language (D15); grade always present (D6); agent
counts disclosed when they constrain (T7 disclosure posture). New
`ReasonKind` arm(s) in `src/priority/view.rs`; lines produced once in
`reason_line()` / shared fragments (`src/priority/render.rs:298`).

## Code impact (design-target)

| Path | Change |
|---|---|
| `src/priority/tension.rs` | new — pure detection types + fn |
| `src/priority/config.rs` | `CompareConfig` + `[priority.compare]` load |
| `src/priority/view.rs` | `ReasonKind` tension arm(s) |
| `src/priority/render.rs` | tension fragments; explain section; next callouts + cap |
| `src/priority/surface.rs` | thread tensions into next/explain rows |
| `src/priority/mod.rs` | command-tier wiring: detection + grade injection |
| `src/priority/elicit.rs` | determinacy source switch (knob-aware); disclosure line |
| `src/comparison/compile.rs` | human-subset compile entry |
| `src/comparison/query.rs` | pair-grade query fn (id-keyed, knob-aware) |
| `src/commands/cli.rs` | `next` verbosity flag |
| `src/commands/compare.rs` | disclosure line on elicit render |
| `tests/e2e_priority_golden.rs` | tension callout goldens |
| `tests/e2e_compare_elicit.rs` | knob-on queue/disclosure goldens |
| `tests/e2e_compare_inference.rs` | knob-on determinacy cases |

## Verification

- VT-A (INV-1): full existing SL-213/217 + priority suites green unchanged,
  knob absent/false.
- VT-B (INV-2): knob-on, agent-only-evidence pair reads indeterminate; same
  pair knob-off reads determined; anchored pair determined both states.
- VT-C: human-subset quarantine self-consistency — human row in an
  agent-involved cycle: quarantined full-system, retained human-system.
- VT-D: detection unit fixtures — no-tension, structure (direct + transitive
  edge, incl. path through an off-page node), composition, mixed, value_dim
  tie (not a tension), equal-full-score tiebreak (excluded), determinism.
- VT-G: grade objective with multipliers — fixture where kind/tag
  multipliers invert value_dim order relative to raw v/c order; grade
  reflects the value_dim claim (raw objective would answer wrongly); `m = 0`
  degeneracy case.
- VT-H: explain off-frontier id — disclosure line, no invented tensions.
- VT-E: wording goldens — the three shapes above + cap behaviour + JSON
  schema fields; D5/D15 phrasing pinned.
- VT-F: knob-on elicit — previously agent-determined pair re-enters queue as
  candidate; state line reflects it.
- VA: render wording reviewed against product-critique tensions #3/#6.

## Deferred (named seams, not built)

- Session mode + read-write web surface (RFC-019 OQ-1) — carries the D2
  knob-on gate contract.
- True human-confirmation candidate kind (dependency tracing on agent-only
  edges) — post-D7, RFC territory (SL-217 Deferred).
- Per-audience grading/demotion (per-rater tiers beyond human|agent) — T4
  territory, not this slice.
