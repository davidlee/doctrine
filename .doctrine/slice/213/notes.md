# Notes SL-213: Comparison constraint layer

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01 — schema v2 wire model + module split + capture (completed 2026-07-11)

Commits: `91f952ef` (T1 mechanical split), `3b8a1c96` (v2 wire + capture).
`doctrine slice verify-vt SL-213`: PHASE-01 VT-1/VT-2/VT-3 all PASS. Full bin
suite 3306 green; `doctrine check gate` clean.

- **Split first, redefine second.** T1 (`comparison.rs` → `comparison/wire.rs`
  + re-exporting `mod.rs`) committed separately with the suite passing
  UNCHANGED — the behaviour-preservation proof is the commit boundary.
- **Clap response group was a non-event.** `#[command(group =
  ArgGroup::new("response").required(true).multiple(false))]` + `group =
  "response"` on `--prefer`/`--equal`/`--incomparable`; usage renders
  `<--prefer <PREFER>|--equal|--incomparable>`. The SL-210
  subcommand-interaction precedent did not recur.
- **Eq derives dropped** from `Judgement`/`ComparisonSession` (`magnitude:
  Option<f64>`); nothing downstream needed `Eq`.
- **Design deviations: none.** A2 (kebab-case response tokens), A4 (no
  `--magnitude` flag — OQ-6), A6 (per-domain frame table replaces
  VALUE_FRAMES; parse-level string losslessness retained, closed table
  enforced at validate) all per design.
- **For PHASE-02:** `resolve.rs` consumes `wire::Judgement.supersedes`
  (already corpus-validated at capture, but R2 must still handle unknown
  uids from hand-merged files as load-time warnings, not errors);
  `domain_for_frame` / `DOMAIN_FRAMES` in wire.rs is the single
  domain-derivation seam.

## PHASE-02 — row-validity resolution (completed 2026-07-11, dispatch worker)

Coord commits: `c581a862` (import, worker author), `c9c75579` (boundary).
17 tests added; suite 3324 green; regression diff vs base clean.

- **Resolution rows borrow (`&'a Judgement`)** — wire `Judgement` has no
  `Clone` and wire.rs is frozen; matches compile's planned
  `&[&Judgement]` input. PHASE-03 consumes borrowed rows.
- **`ResolutionStatus::Malformed`** added for R2 cycle participants (design
  enum had no deactivated-by-cycle variant); cycle detail rides separate
  `MalformedSupersession { cycle }` finding data — PHASE-06 findings consume.
- **Warning shape**: `UnknownSupersedesTarget { row, target }`.
- **R3 identity key**: pair unordered `{a,b}`; winner = max-seq
  NON-TOMBSTONED row in group; session uid in grouping keeps cross-session
  rows concurrent.
- **Dead-code idiom for not-yet-consumed tiers**:
  `#![cfg_attr(not(test), expect(dead_code, reason = ...))]` at module head —
  plain `#[expect(dead_code)]` breaks under test build
  (`-D unfulfilled-lint-expectations`). PHASE-03/04 inherit the same trap;
  remove attrs as consumers land.
- **EntityLifecycle** minimal: `Terminal`, `Superseded { by }`. StatusMap
  populated in PHASE-05.

## PHASE-03 — constraint compilation (completed 2026-07-11, dispatch worker)

Coord commits: `4c439a80` (import), `c455d22e` (boundary). 28 tests; suite
3352 green; regression diff clean.

- **Tarjan decision (design §1 open question): LOCAL.** cordage reuse
  confirmed misfit — `scc_partition` (query.rs:363) and `pass2_cycles`
  (resolve.rs:160) private, `CycleDiagnostic` fields private, no public
  constructor. Local **Kosaraju** in compile.rs with
  deliberate-duplication comment; chosen over Tarjan to avoid
  lowlink bookkeeping under `unwrap_used`/`indexing_slicing` denials.
  cordage public API untouched.
- **ClassId = smallest member entity id** (union-find keeps lexicographic
  min root). Edges: `BTreeMap<(ClassId, ClassId), BTreeSet<RowUid>>`.
- **QuarantineReason payloads**: `PreferenceCycle { classes }` sorted;
  `AnchorConflict { pairs }` ordered x⇝y with anchor(x) ≤ anchor(y); C2
  pairs name entity ids, C4 pairs name class ids.
- **C4**: floor/ceiling two-pass is the implementation; normative path form
  supplies per-edge pair attribution; equivalence debug-asserted every
  compile + pinned on branching goldens. Post-C3 DAG makes
  reach-membership ⇔ edge-on-simple-path exact.
- **C5 property test**: deterministic enumeration, 4 entities × 4^6
  responses × 3 anchor configs = 12,288 ledgers vs independent Kahn+BFS
  feasibility checker. Zero infeasible outputs.
- **Policy seam**: `enum QuarantinePolicy { Symmetric }`.
- **resolve.rs untouched** — compile consumes wire's `&[&Judgement]` only;
  resolve's dead-code attrs stay until PHASE-05/06 wire the pipeline.
- `ConstraintSet::status_of` assigns CompilationStatus per row; RowState
  composition deferred to PHASE-06 per design.

## PHASE-04 — projection + gauge (completed 2026-07-11, dispatch worker)

Coord commits: `a2a9855d` (import), `e4aa314e` (boundary). 26 tests; suite
3378 green; all 22 prototype scenarios (S1–S8, Y1–Y7, N1–N4) verbatim at
1e-4; regression diff clean.

- **⚠ ADJUDICATE AT AUDIT — P8 gauge-scope tension (design vs prototype).**
  Prototype computes gauge spread over GLOBAL height H and takes the
  anchored branch when any anchor exists anywhere; design §3 P1/P8/P12 read
  per weakly-connected component. Divergence only with ≥2 disjoint
  anchor-free components: S2's f/g pendant couples to the diamond's H=3
  (0.8/0.4) instead of per-component (1.333/0.667). Implementation follows
  the PROTOTYPE (plan EX-1 pins it verbatim); pinned transparently by the
  `s2_partial_order_gauge` golden + module-header note. P12 locality
  property-tested in the anchored regime with scope stated. Design wording
  vs prototype needs human adjudication; not a code defect.
- **Fixtures go through real compile()** from judgement fixtures — classes/
  edges honest, `quarantined.is_empty()` asserted per scenario.
- `Projection = BTreeMap<String, (f64, ValueProvenance)>`;
  `ProjectionCfg { gauge_step, default_value }` pure inputs — PHASE-05 homes
  shipped consts (GAUGE_STEP 0.25 in priority/config.rs, DEFAULT_VALUE 1.0
  already in priority/graph.rs).
- Generator reuse: PHASE-03's deterministic enumeration; sweep 0.05–1.0 via
  integer millis.
- `as`-cast denial: `count_f64` helper (`f64::from(u32::try_from ...)`,
  findings.rs idiom) for graph counts.

## PHASE-05 — priority wiring (completed 2026-07-11, dispatch worker)

Coord commits: `9641872c` (import), `4d37b713` (boundary), `56e092bb`
(selector fix). 12 tests added; suite 3390 green (verified on synced tree);
zero existing assertions edited (EX-1 preservation held).

- **R6 "decomposed" carried assumption RESOLVED VACUOUS** (phase-plan):
  decomposition encodes only as spec `parent` edges (registry.rs:36); the
  design rule for comparison rows is a behavioural no-op — no detection
  code. Audit should confirm-and-close the R6 row on this note.
- **StatusMap/AnchorMap builders live in priority/graph.rs, NOT store.rs**
  (deviation from phase sheet A1, correct per ADR-001): they need
  priority::partition + catalog::scan, both ABOVE comparison in the layer
  table; store.rs takes pre-built maps as parameters and stays engine-tier.
  Layering gate verified.
- **Projection keying**: `EntityKey::canonical()` shared between projection
  and priority maps — no normalization layer.
- **GAUGE_STEP**: const 0.25 in priority/config.rs + `[priority.gauge]`
  `step` override (clamp_general idiom); crosses into comparison as plain
  f64 via ProjectionCfg (no reverse import).
- **Selector gap found at import**: src/priority/** was scope-relevant
  only; VT-4's mandated test_file src/priority/config.rs needed
  design-target intent → added + committed mid-funnel (56e092bb).
- **Coord-tree staleness footgun** (orchestrator-side, durable):
  dispatch_import/conclude are object-db only — the checked-out dispatch
  branch advances but the working tree/index do NOT; post-land sync
  (`git restore --source=HEAD --staged --worktree`) is now a funnel beat in
  this drive. See RFC-011 case notes SL-213-drive-4.
- NF-001 facet-symbol tripwire: graph.rs tests must not name ValueFacet
  literally — seed disk + scan instead (worker hit and fixed).
- `CompilationStatus`/`status_of` still dead-code-scoped — PHASE-06 is the
  consumer.

## PHASE-06 — surfaces, findings, determinism e2e (completed 2026-07-11, dispatch worker)

Coord commits: `380ac232` (import), `026d7ff3` (boundary). 5 e2e tests in new
tests/e2e_compare_inference.rs + surface plumbing; suite 3390 green;
regression diff clean.

- **10th display token `malformed`** — design §4 S2 enumerates 9, predating
  PHASE-02's `ResolutionStatus::Malformed`; list renders it. Design doc
  catch-up candidate at reconcile.
- **RowState lives in resolve.rs** (A1), documented same-layer sibling ref
  to compile::CompilationStatus.
- **Pipeline artifact**: store.rs `RowSummary`/`Pipeline` +
  `load_pipeline`/`pipeline_from_sessions`; `load_projection` retired into
  `Pipeline.projection`. explain() does its own scan + pipeline (mirrors
  findings() one-scan style) instead of graph::build.
- **Findings**: `PreferenceCycle{classes,rows}`, `AnchorConflict{anchors,
  rows}` one per distinct anchor pair, `AnchorGaugeDisconnect{entities}`
  aggregate + gated on non-empty cs.anchors, `MalformedSupersession{rows}`.
- **e2e route around worker jail**: write verbs (compare record/withdraw)
  refused by commands/guard.rs Write classification in-jail — e2e
  hand-authors wire-shape session TOML; read verbs spawn the binary.
- **Fixture correction (audit note)**: `compare_list_orders_by_total_key`
  previously varied SessionHeader.date with constant Judgement.date; list
  now orders by resolve()'s display key (judgement date) — fixture stamped
  matching dates (capture reality), asserted ORDER unchanged. Behaviour
  delta is design-intended (D13 list rides resolution ordering).
- **Un-plumbed**: `Resolution.unknown_supersedes` has no surface consumer
  (doc-commented on Pipeline); not a finding — candidate future disclosure.
- **ValueDefault 4th explain shape rejected** — broke explain_human golden;
  design names three shapes only.
- **VH-1 candidates** (human eyeball pending, orchestrator harvest):
  status=tombstoned + legacy [withdrawn] marker redundant on one line;
  AnchorConflict phrasing "anchors X=1.0 vs Y=2.0 conflict"; gauge line
  says "no anchor in component" for both P7 and P8 cases —
  distinguishable if desired.
- **verify-vt at drive end**: PHASE-01 PASS ×3; PHASE-02..06 all
  UNATTRIBUTABLE ("keyword present but file not modified by this slice") —
  expected pre-`prepare-review` (registry derivation from committed
  boundaries ledger happens there); NO FAILs. Boundary rows committed per
  phase via dispatch_conclude_phase.
