# Review RV-270 — reconciliation of SL-217

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Post-implementation conformance audit of SL-217 (elicitation queue), all three
phases completed. Surface reviewed: the primary tree at HEAD (solo-executed
slice, no dispatch candidate branch). Lines of attack:

1. **Conformance algebra** — `slice conformance 217`: 8 undeclared paths,
   1 undelivered selector (`compile.rs`), 7 conformant. Each undeclared path
   dispositioned individually; registry (`slice selector`) is the load-bearing
   fix surface, design §4 the mirror.
2. **Design-ledger fidelity** — D1–D18 vs shipped behaviour; the pre-flagged
   in-phase calls: PHASE-02 source partition (constrained→comparison /
   unconstrained→median-probe), anchor-review admission split (existence, not
   `guaranteed_yield > 0`), RowsRetired→−1 honest negative (D10), PHASE-01
   `hypothetical_outcome` pair-sets beneath the sketch, baseline `&Reachability`
   param, class-keyed hypothetical remap, T3 `estimate` = scalar
   `est_cost`/null-when-bare.
3. **Verification evidence** — `verify-vt 217` all 12 VTs PASS; VT-4 corpus
   generated in-test vs the sheet's committed-fixture letter; VA-1 evidence in
   notes (compile/project diff empty, no pre-existing test modified); gate re-run
   at audit time.
4. **Read-only invariant (D18)** — `compare elicit` READ-classed; no runtime
   state, no persisted queue.
5. **Not agent-closable** — VH-1 human dogfood; `product-critique.md` provenance
   (landed in 15c816e2, handover marked it "not ours").

## Synthesis

The slice ships what the design locked, and the evidence is unusually strong:
all 12 VTs attribute PASS via `verify-vt`, `doctrine check gate` re-ran clean at
audit time (clippy 0, full workspace suite, fmt), and the behaviour-preservation
claim is proved at the diff level — `compile.rs`/`project.rs` untouched
(c9818d71..HEAD diff empty), no pre-existing test file modified, additive
exposures only (`store.rs` owned clones, `wire.rs` Clone derives, `graph.rs`
costing accessors, render/surface visibility). D18 read-only holds structurally:
`compare elicit` is READ-classed in `guard.rs`.

The findings cluster into three families, none blocking:

1. **Registry/prose drift (F-1, F-2).** The design §4 table under-declared the
   shell surface (guard/store/wire/graph/render + the e2e file) and over-declared
   `compile.rs` (pub fields sufficed — a stronger outcome than declared). Pure
   bookkeeping; the selector registry is the fix surface.
2. **As-built refinements the design should absorb (F-3, F-4).** The §1 query
   API grew a pair-set primitive (`hypothetical_outcome`) and a caller-held
   baseline to keep the D13 impact computation inside the §2 cost bound; §2
   gained two rules the design left implicit — the constrained/unconstrained
   source partition (D14 intent) and the anchor-review existence-admission
   (forced by D15 + eval C2 against the blanket `guaranteed_yield > 0` sentence).
   All four were recorded in-phase with rationale and are test-pinned; the design
   prose is the stale artifact, not the code.
3. **Conscious drift and behaviour facts (F-5, F-6, F-7).** The uphold −1
   negative yield is D10 honesty doing its job (pinned by test, refinement
   trigger noted); the scalar/null JSON `estimate` matches §3 exactly; the VT-4
   in-test corpus is tolerated drift from the sheet's committed-fixture letter —
   equivalent for the assertion, convertible on request.

**Standing risks / consciously accepted:** greedy one-step question quality
(design risk, curator layer above is the mitigation — unchanged); the D8 lemma
stays vocabulary-scoped (ratio/band rows void it — Deferred names the revisit
trigger); the two-suspects-per-conflict surfacing (the model cannot tell which
anchor is stale — D12 accepts this); `guaranteed_impact`/boost constants are
implementation-owned tuning pinned by policy goldens, not invariants (ADR-015
posture).

**Open pending User (this audit does not close them):** F-8 VH-1 dogfood
acceptance; F-9 `product-critique.md` keep-or-untrack.

## Reconciliation Brief

### Per-slice (direct edit)

- **F-1 — selector registry (load-bearing) + design §4 mirror:**
  `doctrine slice selector add` for `src/commands/guard.rs`,
  `src/comparison/store.rs`, `src/comparison/wire.rs`, `src/priority/graph.rs`,
  `src/priority/render.rs`, `tests/e2e_compare_elicit.rs`; add matching rows to
  design.md §4 (guard READ-classing D18; store/wire T2 exposures; graph costing;
  render human idiom; e2e per plan VT contract). `.doctrine/rfc/011/case-notes.md`
  and `slice/217/notes.md` are expected process noise — no selector.
- **F-2 — stale selector:** `doctrine slice selector rm src/comparison/compile.rs`;
  amend the §4 row to record that no accessors were needed (pub fields sufficed;
  propagation engine untouched).
- **F-3 — design.md §1 as-built update:** document `hypothetical_outcome ->
  {newly_determined, no_longer_determined}` beneath the `hypothetical_yield`
  wrapper (D13 needs pair sets, not counts); baseline `&Reachability` as a
  caller-held parameter (≤3 recompiles per candidate honest); class-keyed
  hypothetical remap (class id = smallest member; per-item fidelity note for
  future class-split concerns).
- **F-4 — design.md §2 pin:** (a) source partition — comparison pairs enumerate
  constrained pool items only; un-constrained top-K items belong to median-probe;
  (b) admission split — `guaranteed_yield > 0` gates yield-motivated sources
  (comparison, median-probe); anchor-review admits on suspect existence with
  `score = max(gy, 0) × impact` (zero/negative suspects sink, never vanish).
  Cite D15 precedence + eval C2 as warrant.

### Governance/spec (REV)

- None. No ADR, policy, standard, or spec is touched by any finding; RFC-019
  deviations were already recorded at design time (design.md §RFC-019
  deviations) and re-verified here.

## Reconciliation Outcome

### Direct edits applied

- **Selector registry (F-1, F-2 — load-bearing):** `slice selector add`
  design-target ×6 (`src/commands/guard.rs`, `src/comparison/store.rs`,
  `src/comparison/wire.rs`, `src/priority/graph.rs`, `src/priority/render.rs`,
  `tests/e2e_compare_elicit.rs`); `slice selector rm src/comparison/compile.rs`.
  Conformance now 13 conformant / 0 undelivered; residual undeclared = the two
  expected process artifacts (`rfc/011/case-notes.md`, `slice/217/notes.md` —
  IMP-282 filed).
- **design.md §4 (F-1/F-2 mirror):** six as-built rows added; compile.rs row
  removed with an as-built note (pub fields sufficed, engine untouched).
- **design.md §1 (F-3):** `hypothetical_outcome` pair-set primitive documented
  (signature block replaced); caller-held baseline `&Reachability`; class-keyed
  hypothetical remap + per-item-fidelity caveat.
- **design.md §2 (F-4):** source partition pinned (constrained→comparison,
  un-constrained→median-probe); anchor-review existence-admission pinned
  (`score = max(gy, 0) × impact`; D15 + eval C2 warrant).

### REVs completed

- None needed — brief carried no governance/spec items.

### Withdrawn / tolerated

- F-7: tolerated — VT-4 corpus generated deterministically in-test rather than
  committed fixtures; rationale in the finding disposition; User declined
  conversion at audit report.

Reconcile pass complete — handoff to /close.
