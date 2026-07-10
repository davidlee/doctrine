# SL-213 design — Comparison constraint layer

Status: drafted 2026-07-11, post-clarification loop; governs implementation.
Governing contract: RFC-019 (resolved) as revised under external reviews 2–3;
REV-022 (applied); ADR-001 (layering), ADR-015 (priority scoring + value
provenance amendment). The four review-3 design obligations are settled here
(D2–D5) — this document is the record the RFC binds to the design gate.

Design evidence: `projection-prototype.py` (this directory) — the executable
prototype whose scenario battery (S1–S8, Y1–Y7) validated the projection rules
in §3 and seeds the golden suite in §5.

## Decision ledger

| id | decision | rationale (compressed) |
|---|---|---|
| D1 | Schema v1 retired outright; in-place redefinition, `COMPARISON_VERSION = 2`, `version ≠ 2` is a parse error | Verified zero exposure: no release tag contains `src/comparison.rs` (checked v0.18.1 and earlier); no session files exist on disk. The review-3 compat obligation's predicate is vacuous — no mapping, no dual reader, no upgrade verb, no compat goldens. One integer bump gives strays a clean error. |
| D2 | (ob. 3) `prefer-first` reclassified to a new inert `priority` domain — "C-minimal" | Filing non-value evidence under `value` forces a permanent frame-exception on every future consumer; the resolution identity key contains `domain`, so reclassifying later means remapping a populated ledger. Empty-ledger window makes this the cheapest moment. Review 3 sanctioned reclassification. No priority-domain compiler this slice. |
| D3 | (ob. 1) Degraded SCCs: member-level retention with within-SCC quarantine; no condensation edges; no member equality constraints; "tied" is display/projection-layer only | Constraint set stays a subset of evidenced rows — non-manufacture by construction (the A>B>C>A + A>D case cannot create B–D). Member equality would leak external bounds through the tie (manufacture by equality). External uncontradicted edges survive at member level. |
| D4 | (ob. 2) Anchor conflicts: violation-closure quarantine — every edge participating in any anchor-contradicting structure goes inert; anchors win (REV-022) | Semantically-defined set (forced-floor/forced-ceiling DAG passes) → order-invariant determinism, polynomial by choice, provably feasibility-restoring, no fabricated culprit. Minimum-cardinality repair rejected: NP-hard core + arbitrary scapegoat among optima. Loudness is a feature: likeliest defect is a stale anchor. |
| D5 | (ob. 4) dissolved by D1 | The deterministic v1→v2 mapping has nothing to map. |
| D6 | Degradation philosophy, unified: a contradiction quarantines its entire evidencing structure; no implicit culprit; exits are explicit (supersede / tombstone / edit anchor). Deliberate contrast with cordage's FiLo `(rank, age)` cycle eviction for `after` edges | `after` edges are authored declarations — latest intent is authority, eviction is editing. Comparison rows are elicited evidence — recency is not truth (review 1), and silent eviction would duplicate the explicit `supersedes` mechanism. Consequence: re-asking a pair cannot clear a contradiction (concurrent evidence, T6) — findings must direct to `--supersedes`. |
| D7 | Quarantine policy is a pure input to `compile` | T7's rank-aware variant (agent testimony sacrificed first) slots in when the demotion knob lands; symmetric ships. |
| D8 | Pure order semantics at tier 2; no ε, no gap arithmetic; `equal` compiles to exact node merge; the v2 band-tolerance column is dead | Strictness is order information; a chain of k rows implying kε is manufactured magnitude and makes long chains spuriously infeasible under close anchors. ε survives only as tier-3 float hygiene (existing `EPSILON` idiom). Kills OQ-B3. |
| D9 | Projection: budgeted interpolation, reverse-topological greedy, per weakly-connected component; gauge spread for anchor-free components (§3) | Prototype-validated. Budgeted beats midpoint decisively (even spacing vs top-heavy crowding). BT-style fitting (earlier RFC language) superseded: statistical machinery with no repeated-sampling signal under pure order + session-of-one. |
| D10 | Gauge convention: height-above-own-sinks ("longest demonstrated dominance chain"); spread in `(0, 2·DEFAULT_VALUE)`; anchor-incomparable classes sit at `DEFAULT_VALUE` | Any cross-arm interleaving in gauge mode is manufactured; pick the simple convention and state it. Sensitivity bounded: one row moves its arm ≤ one level (Y3). The pinning loop is the real answer: cross-judgement and anchor pins comply with minimal motion (Y4/Y5/Y7). |
| D11 | Provenance vocabulary: `authored > projected > gauge > default`; gauge is a sub-tier of projected for ADR-015 purposes, distinct label in surfaces | The honest answer to "why is this 1.3?". No governance edit: ADR-015's amendment establishes authored > projected > default; gauge refines the middle tier. |
| D12 | `ConstraintSet` is the retained tier-2 artifact (OQ-B4): in-crate, derived-tier, in-memory only, never serialized; edges carry supporting row uids | Phase C's determinacy/queue consumes it; `explain` traces conclusions to evidence through it. No query API yet — Phase C designs its own predicates. |
| D13 | `compare list` gains a single derived `RowStatus` column + `--active-only`; deep diagnostics stay in `explain`/findings (OQ-B2) | The pipeline computes it anyway; a user must be able to see which rows are load-bearing. |
| D14 | Anchor-value collision and gauge-disconnect discontinuity are accepted, documented artifacts (P15/P7), pinned as expected golden outputs | Order-safe; provenance rendering disambiguates; de-gridding steps would be magic for aesthetics. |

## §1 Module boundaries and data shapes

`src/comparison.rs` (pure leaf, imports only `kinds`, sole consumer
`commands/compare.rs`) grows into `src/comparison/`, mirroring the `priority/`
directory style. Tiers map one-to-one onto submodules:

| module | layer | responsibility | depends on |
|---|---|---|---|
| `comparison/wire.rs` | leaf | schema v2 wire model: parse / serialize / validate | `kinds` |
| `comparison/resolve.rs` | leaf | tier 1 → active set: R-rules, `RowStatus` | `wire` |
| `comparison/compile.rs` | leaf | tier 2: equality merge, strict digraph, C-rules, quarantine, bounds | `resolve` |
| `comparison/project.rs` | leaf | tier 3: P-rules placement + gauge | `compile` |
| `comparison/store.rs` | engine | the one impure seam: `load_sessions(root)` moves here from `commands/compare.rs` (the `coverage_store` precedent) | `wire`, fs |

`mod.rs` re-exports the wire vocabulary so `crate::comparison::X` paths compile
unchanged — the Phase A wire suite passes without edits (behaviour-preservation
gate). Everything except `store.rs` is pure over `(sessions, anchors,
statuses)`: no clock, no disk, no rng.

Type inventory (signatures abbreviated; field order and derive sets at
implementation):

```rust
// wire.rs — v2 only (D1)
pub enum Response { PreferA, PreferB, Equal, Incomparable }
pub struct Judgement {
  uid, seq, a, b,
  response: Response,
  domain, frame,                    // per-domain frame table (D2):
                                    //   value    → { equal-effort }
                                    //   priority → { prefer-first }
  form: RowForm,
  magnitude: Option<f64>,           // ratio column; parsed, uncompiled (OQ-6)
  supersedes: Option<String>,       // explicit supersession target uid
  lens, rater, by, note, date,
}
pub const COMPARISON_VERSION: u32 = 2;
pub const DOMAIN_PRIORITY: &str = "priority";

// resolve.rs
pub enum RowStatus {
  Active, Superseded { by: String }, Tombstoned,
  InertLens, InertDomain, InertLifecycle,
}
pub fn resolve(sessions: &[ComparisonSession], statuses: &StatusMap) -> Resolution
// Resolution: Vec<(Judgement, RowStatus)> in (date, session_uid, seq) display order

// compile.rs
pub enum Bound { Unbounded, Open(f64), Closed(f64) }   // Closed only via anchor / merge with anchored class
pub struct ValueBounds { lower: Bound, upper: Bound }
pub enum QuarantineReason { PreferenceCycle { .. }, AnchorConflict { .. } }
pub struct ConstraintSet {                             // D12
  classes: ..,                                         // equality-merged classes (entity id → class)
  edges: ..,                                           // class→class strict edges + supporting row uids
  anchors: BTreeMap<ClassId, f64>,
  quarantined: BTreeMap<RowUid, QuarantineReason>,
  bounds: BTreeMap<ClassId, ValueBounds>,
}
pub fn compile(active: &[&Judgement], anchors: &AnchorMap) -> ConstraintSet

// project.rs
pub enum ValueProvenance { Authored, Projected, Gauge }  // Default = absence from the map
pub fn project(cs: &ConstraintSet) -> Projection         // id → (f64, ValueProvenance)
```

Integration points:

- `priority/graph.rs` — `build_from_with_cfg` gains a `projected: &ProjectionMap`
  input; `effective_raw_value` extends to `authored > projected > gauge >
  DEFAULT_VALUE`; projected magnitudes flow into `value_dim` AND burndown
  identically (SL-210 consistency requirement). Empty map ⇒ bitwise-identical
  behaviour.
- The impure load happens once where the graph is built (`surface.rs` shell):
  `store::load_sessions` → `resolve` → `compile` → `project` → map into build.
- `priority/findings.rs` — three new variants (§4 S4), existing detector +
  `ReasonKind` idiom, named-const thresholds per STD-001.
- `commands/compare.rs` — capture vocabulary + `--supersedes`; `list` status
  column; thins to shell (loading moved to `store.rs`).

SCC detection: prefer reusing cordage's Tarjan / `CycleDiagnostic` machinery
(Reject-mode diagnostics, evictions off). If its API doesn't fit a non-entity
graph cleanly, a small local Tarjan in `compile.rs` is the sanctioned fallback
— deliberate duplication with reason, noted at implementation.

Determinism idiom throughout: BTree collections, uid tiebreaks, no float in
any key.

## §2 Resolution & compilation semantics (normative)

### Resolution — tier 1 → active set

Input: parsed sessions + entity status map. Output: every row tagged with
exactly one `RowStatus`. First matching rule wins:

- **R1 Tombstone** — a tombstone evicts its target by uid → `Tombstoned`.
- **R2 Explicit supersession** — a row named by any active row's `supersedes`
  → `Superseded { by }`. Chains resolve transitively; a superseding row that is
  itself tombstoned does not revive its target (R1 evaluates for the
  superseding row first). A `supersedes` pointing at an unknown uid is a
  load-time warning; resolution ignores it.
- **R3 Implicit revision** — within a single session file only: same identity
  key `(pair, domain, frame, form, lens, rater)`, higher `seq` wins. Cross-
  session same-key rows are concurrent evidence — both `Active`; conflicts are
  tier-2's job, never a lexicographic winner.
- **R4 Domain inertness** — `domain = priority` rows → `InertDomain` (charter
  below; no compiler this slice). `response = incomparable` stays `Active` but
  compiles to no constraint — recorded as asked, selector fodder.
- **R5 Lens inertness** — lens-tagged rows → `InertLens` for pooled `value_dim`
  (T5); reported per-lens in surfaces.
- **R6 Lifecycle** — event-effect table:

  | entity event | its rows |
  |---|---|
  | terminal status (done / rejected) | inert for elicitation, **active for inference** — the pair keeps informing others' bounds via chains |
  | superseded entity | `InertLifecycle` + reprobe hint against the successor; no silent transfer |
  | decomposed entity | rows stay parent-scoped; children inherit nothing (A3) |

Deterministic across any merge order of session files. `(date, session_uid,
seq)` is display order only. Duplicate uids (cherry-picks) collapse to one row.

**`priority` domain charter (D2).** Rows answer: *under a binding capacity
cutoff, which do you keep?* — where deferral may mean never. Value-oriented
but cost-confounded (preferring A can mean more valuable, or half the size, or
both), hence never compiled to `v_A > v_B`; the natural future consumer is a
compiler with a cost model in hand (`v_A·c_B > v_B·c_A` shape). Elicitation
prompts must stretch from fixed-scope projects to sprint commitment without
changing row semantics. Frame implies domain at capture; users never type a
domain.

### Compilation — active set → `ConstraintSet`

- **C1 Vocabulary** — `equal-effort` + `prefer-a/b` → strict edge winner >
  loser. `equal` → equality merge (union-find; classes are the graph's nodes).
  Authored `value` facets → point anchors on classes. Pure order semantics
  (D8): anchors are the only magnitude source. `magnitude` parsed, uncompiled.
- **C2 Equal-vs-anchors** — an `equal` row merging two classes anchored at
  different values: both anchors stand, the `equal` row is quarantined
  `AnchorConflict` (anchors win, REV-022).
- **C3 Cycle quarantine (D3)** — SCCs over strict edges: every within-SCC
  edge's supporting rows → `PreferenceCycle`. External member-level edges
  untouched. No member equality; no condensation edges; "tied" rendering is
  projection/display only.
- **C4 Anchor-conflict quarantine (D4)** — violation-closure: forced-floor /
  forced-ceiling DAG passes; every edge whose tail-floor crosses head-ceiling
  has its rows quarantined `AnchorConflict`, naming both anchors. Pure order
  comparison — no gap arithmetic.
- **C5 Feasibility invariant** — post C3+C4 the system is satisfiable,
  provably. `compile` returning an infeasible set is a bug: debug assertion +
  property test.
- **C6 Bounds** — per class: lower = max anchor strictly below, upper = min
  anchor strictly above; `Closed` only via anchor on the class. Display
  projection only — never decision input (the joint set is the truth; Phase C
  reasons over `ConstraintSet`, not the interval box).
- **C7 One philosophy (D6)** — findings name the exit: supersede (`--supersedes
  <uid>`), tombstone, or edit the anchor. Re-asking without superseding cannot
  clear a contradiction (R3 concurrency) — reprobe hints must say so.
- **C8 Policy seam (D7)** — quarantine policy is a pure input; symmetric ships,
  T7 rank-aware slots in later.

## §3 Projection & gauge (normative; prototype-validated)

Structure:

- **P1 Component scope** — per weakly-connected component of the post-
  quarantine `ConstraintSet`; components are independent (locality is
  structural).
- **P2 Processing order** — merged classes in reverse topological order (lowest
  first), uid-sorted tiebreak; every value is a pure function of already-placed
  descendants + anchors.

Anchored components:

- **P3 Anchors exact** — anchored class = authored value, provenance
  `Authored`; feasibility guaranteed by C5, debug-asserted.
- **P4 Budgeted interpolation** — unanchored class with floor `f` (max over
  direct successors' placed values) and ceiling `c` (min anchor above):
  `value = f + (c − f) / (d_up + 1)`, `d_up` = longest remaining path up to the
  ceiling-defining anchor. Even chain spacing; midpoint is the `d_up = 1` case.
  Provenance `Projected`.
- **P5 Unbounded above** — `value = f + GAUGE_STEP` (named const, STD-001;
  prototype used 0.25 — final value at implementation).
- **P6 Unbounded below** — synthetic positive floor
  `max(0, c − GAUGE_STEP·(d_down + 1))`, then P4 upward. Never manufactures
  negatives (scenario S4).
- **P7 Anchor-incomparable classes** — in an anchored component but no directed
  path to/from any anchor: `DEFAULT_VALUE`, provenance `Gauge`, with the
  explicit hint ("no order path to any anchor — compare against an anchored
  item to place it"). The discontinuity is documented behaviour (D14).

Anchor-free components:

- **P8 Gauge spread** — `value = 2·DEFAULT_VALUE·(h + 1)/(H + 2)`, `h` =
  longest-path height above the component's sinks, `H` = component max height.
  Spread in `(0, 2·DEFAULT_VALUE)`, centred, positive, order-respecting.
  Provenance `Gauge`.
- **P9 Gauge narrative, stated (D10)** — cross-arm interleaving ranks by
  "longest demonstrated dominance chain"; a convention, not evidence. Ties land
  exactly where evidence is silent. First anchor in a component retires the
  gauge (P3–P7 take over).

Contract (property-tested):

- **P10 Order-consistency** — every strict edge strictly respected; every
  equality exact. No NaN; total order extends the existing suite.
- **P11 Determinism** — same active set ⇒ bitwise-identical projection on any
  replica.
- **P12 Locality** — evidence delta in component X moves nothing in disjoint
  component Y.
- **P13 Minimal-motion pinning** — a new cross-judgement moves only classes it
  newly constrains; demonstrated as goldens (Y4/Y7), not promised as a theorem.
- **P14 Affine equivariance, scoped** — within anchor-bracketed spans,
  shifting/scaling anchors shifts/scales projections identically; unbounded
  tails move by absolute `GAUGE_STEP`. The scope limit is stated, not hidden.
- **P15 Known artifacts accepted (D14)** — anchor-value collision (Y5): order-
  safe, disambiguated by provenance. Insertion non-monotonicity of unrelated
  gaps: out of contract; Phase C's empirical entry criterion judges projection
  quality on a real ledger.

Downstream: `effective_raw_value` resolves `Authored > Projected > Gauge >
DEFAULT_VALUE` (D11); `value_dim` and burndown consume identically.

## §4 Surfaces

- **S1 Capture** — `--prefer <a|b|ref>` becomes one arm of a mutually-exclusive
  response group: `--prefer` | `--equal` | `--incomparable` (exactly one).
  New `--supersedes <uid>`, validated against the loaded corpus at capture
  (unknown uid = hard error — the only moment a human is present). `--frame
  prefer-first` derives `domain = priority` silently; help text carries the
  charter one-liner. Breaking flag changes are free pre-release (D1).
- **S2 `list`** — `RowStatus` column (`active`, `superseded→uid`, `tombstoned`,
  `quarantined(cycle)`, `quarantined(anchors)`, `inert(lens|domain|lifecycle)`)
  + `--active-only`. Existing listing `Format`/`RenderOpts` machinery.
- **S3 `explain`** — value-source block, three shapes:
  - `value 5.0 — authored` (+ finding reference when quarantines cite it)
  - `value 4.4 — projected · bounds (2.0 ‥ 8.0) · from 5 judgements (3 human, 2 agent)` — the rater split is the T7 disclosure
  - `value 1.3 — gauge · ordered by 4 judgements, no anchor in component · set a value on any member to calibrate`
  Existing `view`/`render` structured-reason idiom; `--json` carries the same
  fields structurally.
- **S4 Findings** — `PreferenceCycle` (members + quarantined row uids +
  supersession-directing reprobe hint, C7), `AnchorConflict` (both anchors with
  values + quarantined uids + exits), `AnchorGaugeDisconnect` (P7 hint).
  Inert `priority`-domain rows: one-line disclosure in `explain` only ("N
  prefer-first judgements recorded — not value-bearing; no consumer yet") —
  not a finding; nothing is wrong.

## §5 Verification plan

Suites → rules pinned. VT/VA/VH criteria ids minted at `/plan` from this
section.

1. **Wire** — v2 goldens (shape, response vocabulary, magnitude, supersedes);
   `version ≠ 2` rejected with the pre-release message; per-domain frame
   admissibility.
2. **Resolution** — one test per R-rule; cross-session concurrency (both
   active); supersession chain + tombstoned superseder; duplicate-uid collapse;
   merge-order invariance.
3. **Compilation** — the D3 golden (A>B>C>A + A>D: B–D/C–D independence, A>D
   retained, finding lists exactly the cycle rows); 2-cycle; violation-closure
   goldens (single-row conflict, chain conflict, overlapping paths); C2;
   C5 feasibility as a property test over generated ledgers.
4. **Projection** — prototype battery as goldens (S1–S8, Y1–Y7 verbatim);
   property tests P10–P12, P14 over generated components; D14 artifacts pinned
   as expected outputs.
5. **Behaviour preservation** — no `comparisons/` dir and empty dir: every
   existing priority suite passes unchanged; `build_from_with_cfg` with an
   empty projection map is bitwise-identical to today.
6. **Integration** — `effective_raw_value` resolution order incl. gauge tier;
   burndown consumes projected values identically to `value_dim`.
7. **Surfaces** — status column render; three explain shapes; findings render +
   JSON parity.
8. **Determinism** — shuffled session-file load order ⇒ identical resolution,
   `ConstraintSet`, projection; no-NaN/total-order suite extended over
   projected values.

## RFC-019 deviations (design-stage, recorded)

1. **v1 compat retired, not implemented (D1).** The RFC's "additive,
   version-gated v2 + v1 parseable forever" was protective armor for deployed
   data; verified vacuous pre-release (no tag ships `comparison.rs`; no session
   files exist). In-place redefinition, version bump, hard reject of v1.
2. **BT-style fit superseded (D9).** RFC's "BT inside provably-positive
   components" predates the pure-order decision (D8); no repeated-sampling
   signal exists to fit. Budgeted interpolation replaces it.
3. **Band-tolerance column dead (D8).** `equal` is an exact merge; the v2
   band column ships never, not "if design picks ε-bands".

A corresponding note is appended to RFC-019's prose; the RFC's resolved status
is unaffected (review 3 delegated exactly these choices to this design gate).

## Resolved OQs

- OQ-B1 → D9/D10 (§3). OQ-B2 → D13 (§4 S2). OQ-B3 → D8 (dead). OQ-B4 → D12.
- OQ-6 (ratio elicitation) stays open at RFC level; v2 carries the column only.

## Deferred (named seams, not built)

- Priority-domain compiler (cost-model cross products) — post-C sibling work.
- T7 rank-aware quarantine — slots into the C8 seam with the demotion knob.
- Determinacy predicate, elicitation queue — Phase C, gated on empirical
  evaluation of this slice's output against a real ledger.
- Phase D narrative surfaces; agent-demotion knob mandatory before stakeholder
  surfaces.
