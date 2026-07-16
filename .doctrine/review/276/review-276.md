# Review RV-276 — reconciliation of SL-219

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed (F-2 record, ADR-012):** candidate interaction branch
`candidate/219/review-001` (41d08b0e) — merge of `refs/heads/review/219`
(impl-bundle, 04282549) onto `refs/heads/main`, provisioned worktree
`.doctrine/state/dispatch/candidate/cand-219-review-001`. Evidence refs
`review/219` + `phase/219-01..06` are immutable (R2); the drive log is
`dispatch/219:.doctrine/slice/219/notes.md`. Audit verbs run from the parent
tree; build/test evidence gathered in the candidate worktree.

**Lines of attack:**

1. **Mechanical conformance** — `slice conformance 219` in the candidate tree
   (its slice-219.toml carries the two late design-target declarations:
   `src/priority/view.rs`, `tests/e2e_compare_elicit.rs`). Every undeclared /
   undelivered cell dispositioned, not hand-waved.
2. **VT sweep** — `slice verify-vt 219` (17 VTs across PHASE-01..06) against
   candidate code; full unit + e2e suites; `doctrine check gate`.
3. **VA/VH criteria** — PHASE-02 VA-1 (compile.rs untouched), PHASE-04 VH-1
   (REV-023 user-approved before ladder commit) + VA-1 (ADR-015 estimate-source
   section post-apply, spec-coherence note), PHASE-06 VA-1 (design §6 checklist
   sweep — worker va1_map flags §6.7 reprobe as PARTIAL: verify or waive).
4. **Accepted deviations** (drive log) held against design intent: PHASE-02
   top-level `constraining_by_class`/`active_judgements` vs the three-field
   DomainSystem pin; PHASE-03 config-knob threading + signature growth;
   PHASE-05 Option B golden reconciliation (`incomparable` row, user-adjudicated);
   PHASE-06 fourth cost shape `authored (via class anchor)` vs design §5's
   "three shapes"; est-engagement gating of the cost-source block.
5. **Authored divergence** (stage-2 integrate is /close's): dispatch/219 carries
   notes.md + 2 selector rows edge lacks; edge carries REV-023 / amended ADR-015
   dispatch lacks. Audit records the reconciliation route, not the merge.
6. **Invariants held:** design D1–D11 (single formula site, D2 gauge-never-
   divides, D7 no-feedback bare anchor, D11 positivity), ADR-001 layering,
   behaviour preservation R2 (zero est rows ⇒ bitwise-identical scoring),
   RFC-019 posture (capture everything, infer only what is sound).

## Synthesis

**Closure story.** SL-219 landed all six phases through the claude-arm dispatch
funnel with clean boundaries at every S (regression diff clean, baseline 0
failures at every base). Audit evidence, gathered in the candidate worktree
(`candidate/219/review-001`): VT 17/17 PASS via `slice verify-vt`; full unit +
e2e suites green (exit 0; unit binary 3548 tests at PHASE-06 per drive log);
`doctrine check gate` clean. The two stated non-touches hold mechanically —
`git diff main...review/219` is empty for `src/comparison/compile.rs` and
`src/comparison/query.rs` (PHASE-02 VA-1; design §2 "not touched, stated").
The PHASE-04 human gate was honoured in sequence: REV-023 (ADR-015 estimate
provenance amendment) drafted per design §3's five numbered items,
user-approved, applied — status `done · approval=approved`, `doctrine boot
--check` clean on edge — before the ladder commit landed. Conformance algebra,
reconstructed across the two surfaces (F-3): 14 conformant, 0 undelivered, and
one undeclared residue that is the selector registry itself riding the funnel
boundary ranges — no source-path drift. Every drive-log deviation was
dispositioned: F-5/F-6 aligned (documented, invariant-preserving,
user-adjudicated where scope moved); F-4 is the one true canon gap — design
§5/§2 prose trails the landed surface — routed to reconcile as per-slice
direct edits.

**Standing risks.**
- *Pre-integrate divergence* (F-2): dispatch/219 carries notes.md (the drive
  log) and the two late selector rows; edge carries REV-023 + amended ADR-015.
  Converges mechanically at stage-2 integrate (/close). Post-integrate check:
  re-run `slice conformance 219` from trunk — expect 14 conformant / 0
  undelivered / no undeclared *source* paths.
- *Regime-flip discontinuity* is an owned design consequence (D2: first anchor
  flips component members bare → projected), golden-pinned, not a defect.
- *CHR-044* (worker-marker refusals red-line e2e write-goldens and hard-stop
  `check commit` locally) stands as a platform chore — environmental, does not
  touch SL-219 evidence (candidate-tree suites ran unmarked and green).

**Spec-coherence note for /close** (PHASE-04 VA-1): ADR-015 is now the only
governance naming `est_cost` resolution (estimate-source section present
post-REV-023, verified on edge); SPEC-020 describes the estimate facet, not
scoring. Re-check at /close per design §3.

**Harvest disposition.** The runtime phase sheets were already digested into
the drive log (dispatch/219 notes.md) during the drive; no further phase-sheet
harvest owed. Edge's notes.md is deliberately left untouched pre-integrate —
writing it would collide with the integrate merge. Platform signal defects
from F-3 minted as a backlog improvement.

## Reconciliation Brief

### Per-slice (direct edit)

- **design.md §5, cost-source block** (F-4): enumeration "three shapes + one
  flag" → four shapes — add `authored (via class anchor)` for facet-less
  members hoisted by an `equal` merge (the §2 cost-feed tier table's P3
  "members merged in without own facet" row, provenance Authored; exercised by
  the probe round-trip e2e). Add the gating sentence: the block renders only
  when the est system is engaged (est projection non-empty) — mirrors the
  value-source "bare divisor is a floor, not a citable source" posture; keeps
  every pre-SL-219 explain golden byte-identical; the standalone bare-anchor
  shape renders only in est-active corpora.
- **design.md §2, module-impact table** (F-4): add rows
  `src/priority/view.rs` — cost-source `ReasonKind` variants +
  `Explanation.cost_source` (SL-213 value-source precedent, additive) — and
  `tests/e2e_compare_elicit.rs` — PHASE-05 Option B golden reconciliation
  (user-adjudicated). Prose mirror only: the selector registry already carries
  both rows with notes (dispatch/219, commits a44daea5 / 63f5d7ca); **no
  `slice selector` verb needed** (F-2 — a duplicate edge-side registry write
  would collide with the integrate merge).

### Governance/spec (REV)

- None. The slice's governance obligation (estimate-source resolution ladder,
  INV-2 restatement, gauge-never-divides, positivity axiom, operator knobs)
  was discharged in-flight by REV-023, applied to ADR-015 before the ladder
  landed (F-1/F-5/F-6 raised nothing further against governance).

## Reconciliation Outcome

### Direct edits applied (design.md, edge)

- **§5 cost-source block** (F-4): "three shapes + one flag" → four shapes.
  Added the `est_cost — authored (via class anchor)` shape for the facet-less
  member hoisted into an anchored class by an `equal` merge (provenance
  Authored; the §2 P3 tier-table row), verified byte-faithful to the landed
  render (`render.rs:614`, `CostAuthored { pin: None }`; `view.rs:87-92`).
  Added the est-engagement gating sentence — the block renders only when est
  projection is non-empty (`render.rs:702` `if let Some(&ex.cost_source)`),
  mirroring the value-source floor-not-a-source posture; pre-SL-219 explain
  goldens stay byte-identical.
- **§2 module-impact table** (F-4): added the two design-target rows the prose
  mirror lacked — `src/priority/view.rs` (cost-source `ReasonKind` variants +
  `Explanation.cost_source`, SL-213 precedent) and `tests/e2e_compare_elicit.rs`
  (PHASE-05 Option B golden reconciliation). Prose mirror only — the selector
  registry already carries both on dispatch/219 (a44daea5 / 63f5d7ca); no
  `slice selector` verb (F-2: an edge-side write would collide with stage-2
  integrate).
- **§6.9 verification-plan mirror** (F-4, same divergence): "three cost-source
  shapes" → "four cost-source shapes" — the one-word count mirror of the §5
  shape addition, folded in with user assent rather than handed back as a new
  finding.

### REVs completed

- None owed. Governance was discharged in-flight by REV-023 (ADR-015
  estimate-source resolution), applied before the ladder landed.

### Withdrawn / tolerated

- F-3 tolerated: the two conformance-signal defects are platform behaviour, not
  SL-219 drift (rationale in the finding disposition); minted as a backlog
  improvement at harvest. No reconcile write.
- F-1 / F-2 / F-5 / F-6 verified-aligned: no reconcile write (documented,
  invariant-preserving, or already-correct on the integrating surface).

### Not a reconcile surface (noted, not edited)

- `render.rs:602` stale `// the three shapes'` doc comment — code on
  dispatch/219, outside the design.md reconcile surface; left for a code touch.

Reconcile pass complete — handoff to /close.
