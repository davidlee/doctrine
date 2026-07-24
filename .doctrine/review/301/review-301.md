# Review RV-301 — reconciliation of SL-226

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Post-implementation conformance audit of SL-226 (CLI graph emitter + ego-view),
4 phases landed on the pi arm (dispatch/226; impl bundle review/226 = 848f3b5c).
Reviewed surface: the impl bundle review/226 (evidence refs dispatch/226,
review/226 are immutable; no interaction candidate recorded — audited the bundle
tree directly in a detached worktree).

**Lines of attack / invariants held:**

- Acceptance: every phase EX + VT criterion in plan.toml, re-verified on a
  fresh binary — not the drive-time conclude-gate alone.
- Design fidelity: D1–D15 + R1–R4 (design.md §7/§8) as the conformance yardstick,
  with focus on the RV-298-hardened decisions (D9 traversal-collected edges, D11
  owned projection, D13 index identity, D6 focus classification, D10 shape/style
  split).
- STD-001 (named-constant style tables), ADR-001 layering (catalog::dot=engine
  row present + gate-critical), behaviour-preservation (concept_map suite green
  after the D8 dot_escape lift).
- Mechanical conformance algebra (`slice conformance 226`) as the where-to-look
  signal, not a verdict.
- VA-1 real-corpus render discharge on a host with graphviz.

**Evidence gathered (fresh audit binary, review/226 tree):**

- VTs green: `catalog::` 80/0 (graph.rs + dot.rs), `commands::graph` 10/0,
  `concept_map` 148/0 (VT-E preservation).
- VA-1 reproduced: `graph SL-226 --depth 2 --format dot` exit 0, empty
  graph-side stderr; `dot -Tsvg` exit 0, 29,118-byte SVG; zero
  `shape="box,rounded"` (D10). Sole dot stderr = Fontconfig env artifact.
- Clippy `--workspace` zero warnings (EX-3).
- Conformance: undeclared 0 · conformant 8 · undelivered 1 (layering.toml, F-3).

## Synthesis

SL-226 is conformant and closure-ready. The implementation rides the RV-298
design faithfully at every load-bearing decision: the projection is an owned
`CatalogGraph` subset (D11) so JSON reuses the existing `Serialize` impl and the
mirror-drift class is structurally impossible; BFS collects edges only from
expanded nodes (D9) with edge identity by list index (D13); focus is classified
full-then-filtered in the command layer with the D6 error split; the DOT emitter
applies the D10 shape/style correction (no invalid `box,rounded`), D5 ghosts, D14
roled `references(role)` labels, and D15 tooltip omission, under a total R4 sort
order proven byte-deterministic. All acceptance criteria re-verified green on a
fresh binary, VA-1 discharged by real-corpus render.

Four findings, all minor/nit, none gating:

- **F-1** (style-table representation): the sole genuine design deviation — the
  worker shipped `node_style()`/`edge_color()` match fns where locked design
  §5.2/§5.3 prescribed `NODE_STYLES`/`EDGE_COLORS` slice-consts. Values correct,
  STD-001 intent met, no behavioural impact. Adjudicated (user) to accept the
  code and reconcile canon to it — the standing tradeoff consciously accepted is
  that a locked-design illustration was superseded by an equivalent-and-idiomatic
  implementation, and canon is corrected to match rather than the code churned.
- **F-2** (VT-1 keyword-provenance): consequence of F-1 — `NODE_STYLES`/
  `EDGE_COLORS` survive only in a test comment, so the raw-byte vtgate is
  comment-satisfied. Standing risk is bounded: the behavioural assertions are
  strong, plan VT-1 is immutable, and no POL-002-compliant gate fix exists — a
  known, accepted limitation (IMP-228 rationale), tolerated.
- **F-3** (conformance undelivered layering.toml): a pi-arm topology artifact —
  the gate-critical row is delivered in the bundle and integrates to main, but
  rode an orchestrator governance commit outside the worker source-delta registry.
  Tolerated; the only live action is a close-time check that the row reaches main.
- **F-4** (PRD-016 §2 boundary): the pre-declared governance deferral; routed to
  reconcile as a REV.

Standing risks at close: (1) the layering row must be present on main
post-integrate (self-protecting — close's `doctrine check gate` clippy leg fails
without it); (2) canon and a governance spec both need the reconcile writes below
before the slice tells the truth.

## Reconciliation Brief

### Per-slice (direct edit)
- **design.md §5.2 & §5.3** (F-1): replace the `NODE_STYLES: &[(&str, NodeStyle)]`
  / `EDGE_COLORS: &[(&str, EdgeColor)]` slice-const prescription with the as-built
  representation — `node_style(prefix) -> NodeStyle` / `edge_color(label_lower)
  -> EdgeColor` match-lookup fns plus `DEFAULT_NODE_STYLE` / `DEFAULT_EDGE_COLOR`
  consts. Keep the STD-001 justification and the D10 shape/style split note; only
  the table *representation* changes. Values unchanged.

### Governance/spec (REV)
- **PRD-016 §2** (F-4): REV modify — carve out the in-workflow, pipe-composable
  consumption surface (`doctrine graph`, RFC-001) from the blanket static-file
  interchange demotion, so the demotion sentence no longer reads as excluding
  SL-226's surface.

### Close-time verification (not a write — a gate check for /close)
- **F-3**: after `dispatch sync --slice 226 --integrate --trunk refs/heads/main`,
  confirm `.doctrine/adr/001/layering.toml` on main carries
  `"catalog::dot" = "engine"` (gate-critical). `doctrine check gate` at close
  exercises this via the clippy layering leg on a fresh binary.

### Off-surface (no reconcile write — recorded for provenance)
- plan.toml PHASE-03 VT-1 keywords (F-2): immutable-append, not a reconcile
  surface. No action; tolerated per the ledger.

## Reconciliation Outcome

### Direct edits applied
- design.md §5.3 (RV-301 F-1): style-tables prose reconciled from the authored
  `NODE_STYLES`/`EDGE_COLORS` slice-const prescription to the as-built
  `node_style()`/`edge_color()` match-lookup fns + `DEFAULT_*` consts. STD-001 /
  DEC-008 / D10 justification preserved; values unchanged.

### REVs completed
- REV-033 (`reconcile-sl-226`): done — `modify` PRD-016 §2 out-of-scope bullet,
  appending a carve-out so the static-file-interchange demotion no longer reads
  as excluding `doctrine graph`'s in-workflow DOT/JSON stdout surface (RFC-001,
  SL-226). Covers RV-301 F-4. Manual-landed; rationale + before/after in
  revision-033.md.

### Withdrawn / tolerated (no write)
- F-2: tolerated — VT-1 keyword-provenance comment residual; no POL-002-compliant
  fix; behavioural assertions strong. Rationale in the finding disposition.
- F-3: tolerated — conformance `undelivered` layering.toml is a pi-arm topology
  artifact; the row is delivered in the impl bundle and integrates to main. Not a
  reconcile write — carried to /close as a gate check (confirm `catalog::dot =
  engine` on main post-integrate; self-protecting via close's `doctrine check
  gate` clippy layering leg).

Reconcile pass complete — every brief item resolved, no half-applied REVs.
Handoff to /close.
