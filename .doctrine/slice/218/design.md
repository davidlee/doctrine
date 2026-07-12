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
| D4 | Tension predicate = `value_dim`-order vs delivery-order inversions where the **surfaced member is on the rendered page and the preferred counterparty ranges over the full frontier order** (RV-271 F-4), classified by cause: **Structure** (surviving seq/dep constraint forces the inversion; callout cites the edge) vs **Composition** (full-score dimensions — risk_dim, leverage, optionality — lift the surfaced item; callout cites component deltas). Wording never dresses a full-score claim as a value claim. | RFC-019 Phase D targets structure overrides ("never silently resolved"). Composition divergence is ADR-015 working as designed but surprising — accurate to show, only when asked (D5 of this ledger). D5/SL-217 compliance: divergence attributed to named score dimensions is not a value-order claim. |
| D5 | Render defaults: `explain` renders **both** classes; `next` renders **structure-only** by default, a verbosity flag additionally pulls composition callouts in. Flag spelling settled at implementation against existing `next` CLI conventions. | `explain` is the drill-down where "why is this above that" is being asked. Composition-divergence always-on in `next` would drown the structure signal. |
| D6 | Every callout carries an **evidence grade** for the value_dim ordering it asserts: `Determined` (joint-feasible-set predicate over the verdict system, with that system's constraining counts), `AgentProposed` (knob-on only: determined in the full system, not in the human system — order proposed by agent evidence, unconfirmed), or `Projected` (gauge spacing / defaults — no determining evidence in any system). Counts always come from **the system that produced the verdict** (RV-271 F-2/F-3). | Critique #6: point-projected value_dim can invert on manufactured cardinal spacing the feasible set does not support; the render must not overclaim. Two-state vocabulary collapsed demoted agent evidence into "no determining evidence" — false under T7, and it made the disclosure line fight the wording. Anchored pairs come out `Determined` through the same predicate (anchors are point constraints) — one predicate, no special case. |
| D7 | Detection = **pure fn** in new `src/priority/tension.rs`. Grading reuses the **shipped** determinacy machinery exactly as the elicit queue does: `PairSide { eff_weight = m·c_other }` + `determined()` (SL-217 D6 — the predicate is already multiplier-aware; there is no "raw objective" to generalise, RV-271 F-1/F-7). Assembly lives where elicit's does — the priority layer already imports `comparison` (elicit.rs, graph.rs, surface.rs, findings.rs); ADR-001 governs leaf ← engine ← command, and both modules are engine-tier. Render via new `ReasonKind` arm(s); all human wording through `reason_line()` fragments (single source, REQ-072 AC3). | One predicate, one truth per question: grades and elicit-queue verdicts must never disagree about the same pair (both knob-aware, same system selection). An earlier draft posited a command-tier injection seam on a false layering premise (RV-271 F-5) — dissolved, not built. |

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
  /// Verdict system determines the ordering; counts are THAT system's
  /// constraining rows (human system when knob-on — RV-271 F-3).
  Determined { counts: RaterCounts },
  /// Knob-on only: full system determines, human system does not — the
  /// ordering is proposed by agent evidence, unconfirmed (RV-271 F-2).
  /// Counts are the full system's, labelled as agent-proposed.
  AgentProposed { counts: RaterCounts },
  /// No determining evidence in any consulted system.
  Projected,
}
```

### Predicate

Inputs: the **full** frontier order `F` in delivery order (`frontier_order`,
`src/priority/order.rs:76` — the complete actionable list, not the page),
the rendered page bound `K`, per-node `value_dim` (`BaseScore`,
`src/priority/graph.rs:62`), surviving seq/dep edges
(`surviving_seq_predecessors`, order.rs:39, plus dep edges), score component
maps.

**Window (RV-271 F-4):** the *surfaced* member `B` must be on the rendered
page (`pos(B) < K`); the *preferred* counterparty `A` ranges over the whole
frontier order, on-page or below the cutoff. Page-bounding both members
would let a surfaced row silently outrank a higher-value item sitting just
below the cutoff — exactly the silent-resolution path Phase D exists to
close. Complexity `O(K·N)` pair scan — still trivial.

**Zero-multiplier exclusion (RV-271 F-6, inherits SL-217 D6):** pairs where
either member has `m = 0` are excluded from tension claims — the item's
value cannot move its own `value_dim`, so a "ranks above on value_dim" claim
is value-insensitive, not a tension. When exclusions occur on the page, the
render scopes itself the way SL-217 D6 pinned ("N pairs value-insensitive,
zero weight") rather than silently narrowing.

For each remaining pair `(A, B)` with `pos(B) < pos(A)` and
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

Ties (`value_dim` equal) are not tensions. Per-pair bounded reachability
(DFS over surviving edges, memoized). Determinism: pairs emitted in
delivery-order position order; `total_cmp` for float comparisons (house
style).

**`explain` considered set (RV-271 F-4)**: the explained id participates if
it is anywhere on the frontier order — as surfaced member (its page-rank
position vs higher-value items anywhere) or as preferred counterparty
(off-page but outranking a page row on value_dim; explain is exactly where
that displaced-item story must surface). Only ids not on the frontier at all
(not actionable) get the disclosure line "not on the current frontier — no
tension analysis".

`value_dim` here is the point projection the engine actually consumed —
detection does not re-derive value. The *grade* (D6) is what keeps the
callout honest when the projection's ordering is not feasible-set-backed.

### Grading (in-priority, shipped machinery — RV-271 F-1/F-5/F-7)

The shipped determinacy predicate is **already multiplier-aware**: SL-217 D6
folded static multipliers into the pair objective
`f = m_A·c_B·v_A − m_B·c_A·v_B`, and the elicit queue builds
`PairSide { eff_weight = m_self · c_other }` (`side_vs`,
`src/priority/elicit.rs:258`) before calling `determined()`
(`src/priority/query`-seam, consumed at elicit.rs:489-520). There is no raw
`v/c` objective to generalise; an earlier draft of this section claimed
otherwise and is corrected. `m = 0` accounting is likewise already pinned by
SL-217 D6 (value-insensitive, excluded — see §2 predicate).

Grading therefore **reuses that exact machinery** — same `PairSide`
construction, same `determined()`, same closed-form joint-region evaluation
(D8 lemma untouched). Assembly follows the elicit pattern *inside* the
priority layer, which already imports `comparison` (elicit.rs, graph.rs,
surface.rs, findings.rs — ADR-001 constrains leaf ← engine ← command, not
these co-tier engine modules); no command-tier injection seam, no new
cross-layer API. Grades are computed **only for detected pairs** (no corpus
sweep); entities with no comparison class ⇒ `Projected`.

**One truth per question (F-1/F-7 obligation):** the tension grade and the
elicit queue consult the *same* predicate under the *same* system selection.
Knob-on, **both** read the human system for determinacy verdicts — the
elicit queue's determinacy source moves with the knob (PHASE-01, §1), so
`next`/`explain` can never call an ordering determined while `compare
elicit` still offers the pair, or vice versa. A cross-surface agreement
golden pins this (VT-I).

**System selection per grade (F-2/F-3):**

- Knob-off: one system (full). `Determined` counts = the full system's
  `constraining_counts_by_class` (`src/comparison/compile.rs:156`) summed
  over the pair's classes — agent rows disclosed in the count, T7 ship
  posture. `AgentProposed` is unreachable.
- Knob-on: verdicts from the human system. `Determined` counts = the
  **human system's** counts (the rows that actually retired the question —
  never agent rows the verdict excluded). Pair determined in the full
  system but not the human system ⇒ `AgentProposed` with the full system's
  counts, labelled as unconfirmed agent proposal. Determined in neither ⇒
  `Projected`.

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

Structure, agent-proposed (knob-on):

> tension: SL-014 ranks above SL-009 on value_dim (agent-proposed — 4 agent
> judgements, unconfirmed); SL-009 surfaces first — `after SL-009` sequence
> survives.

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
| `src/priority/surface.rs` | thread tensions into next/explain rows; grading assembly (elicit pattern) |
| `src/priority/mod.rs` | wiring: detection + grading on next/explain paths |
| `src/priority/elicit.rs` | determinacy source switch (knob-aware); disclosure line |
| `src/comparison/compile.rs` | human-subset compile entry |
| `src/comparison/query.rs` | no new API expected — existing `determined()`/`PairSide` reused by priority-side grading; row retained as fence in case exports need widening |
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
  tie (not a tension), equal-full-score tiebreak (excluded),
  **off-page preferred counterparty detected** (F-4), `m = 0` pair excluded
  with scoped disclosure (F-6), determinism.
- VT-G: grade multiplier correctness — fixture where kind/tag multipliers
  invert value_dim order relative to raw v/c order; grade tracks the
  value_dim claim (shipped eff_weight machinery, SL-217 D6).
- VT-H: explain — off-frontier id gets disclosure line, no invented
  tensions; off-page-but-on-frontier id renders its displaced-counterparty
  tension.
- VT-I: cross-surface agreement (F-1/F-7) — same corpus, both knob states:
  every pair the tension render grades `Determined` is absent from the
  elicit queue's indeterminate set, and every `AgentProposed`/`Projected`
  tension pair with admissible members is offerable; grades and queue never
  disagree.
- VT-J: grade vocabulary (F-2/F-3) — knob-on fixture: agent-only-determined
  pair reads `AgentProposed` (full-system counts, "unconfirmed" wording),
  human-determined pair reads `Determined` with human-system counts only
  (no agent rows cited); knob-off same corpus reads `Determined` with mixed
  counts disclosed.
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
