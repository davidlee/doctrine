# Review RV-394 — reconciliation of SL-266

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Conformance self-audit of SL-266 (solo, not dispatched — reviewed surface is
`edge` at `fd3c53453`+, the RV-393 fix commit). Mode: conformance.

Evidence: `doctrine check gate` exit 0 (8877 passed, 0 failed);
`doctrine slice verify-vt 266` 14/14 PASS; VH-1 (PHASE-02, "legible") and both
VA-1 attestations recorded in notes; RV-392 (PHASE-03) and RV-393 (pre-close
code review, F-1..F-5 verified) concluded.

Lines of attack:

1. `slice conformance 266` — undeclared / undelivered cells, separating
   SL-266's own touches from foreign commits inside the recorded phase ranges.
2. Design prose vs as-built: the DEC-307 VH-1 amendment (sec-3), PHASE-03 scan
   skip rules (sec-4), RV-393 F-4's `[design]`-only read (sec-5).
3. Plan criteria coverage and attestations (VT/VA/VH per phase).

## Synthesis

- **Overall:** solid.
- **Synopsis:** SL-266 delivers what ISS-299 asked for: the whole inquiry map
  reaches the user as a tree. It rides the full-detail envelope, has one render
  path (`design tree` = `show --format tree`), and delivery is configurable
  (relay line on a real map change, or sidecar silence). All 14 VT criteria
  pass, the gate is green (8877/0), VH-1 was judged legible, and both VA-1
  checks are attested. Two code reviews (RV-392 per-phase, RV-393 pre-close)
  concluded with every finding verified. No code defect remains. The six audit
  findings are canon drift. Three design sections lag the build: sec-3's drop
  rule (DEC-307 amended at VH-1), sec-4's scan skip rules (RV-392), and sec-5's
  config read (RV-393 F-4). The selector registry also under-declares
  supporting seams and tests, and over-declares `run.rs`. All of it goes to
  `/reconcile` below. Consciously accepted: F-6, the conformance noise from
  interleaved foreign commits (captured as friction). IMP-486 (typed MapNode
  fields) is owned follow-up from RV-393 F-3.
- **Haiku:** *Code ran ahead, true — / the design still draws old rails; / ink
  must catch the tree.*

## Reconciliation Brief

### Per-slice (direct edit)
- F-1 — `design.md` "The tree rendering", rule 2: replace the rail-less drop
  ("rails not drawn") with DEC-307's amended rule — dropped text keeps its
  ancestors' rails at `TREE_DROP_INDENT`.
- F-2 — `design.md` "Command surface and run resolution": add the two PHASE-03
  skip rules — a run whose slice record is unreadable, and a snapshot naming
  another slice, are skipped (listed as skipped, not chosen).
- F-3 — `design.md` "Delivery": the design writes read only the raw `[design]`
  entry via `dtoml::load_design_entry` (bare-table read), not
  `load_doctrine_toml(root)?`; `DoctrineToml` carries no `design` field; only
  TOML syntax errors refuse the writes.
- F-4 — **registry (load-bearing):** `doctrine slice selector add 266`
  `src/commands/guard.rs src/knowledge.rs src/slice.rs src/design_run/tests.rs
  tests/e2e_design_tree.rs tests/e2e_design_show_golden.rs
  tests/e2e_subcommand_help.rs tests/design_fixture/mod.rs` (intent
  design-target); mirror in `design.md` "Code impact" selector list.
- F-5 — **registry (load-bearing):** `doctrine slice selector rm 266
  src/design_run/run.rs`; mirror in "Code impact" (drop `run.rs` from the list
  and the "(or `run.rs`)" alternative).

### Governance/spec (REV)
- None. No ADR, policy, standard, PRD-019 or SPEC-029 claim diverges.

## Reconciliation Outcome

### Direct edits applied (`design.md`, out of band on the locked run — expected divergence)
- F-1 — "The tree rendering" rule 2: the drop now keeps the node's rails, and
  falls back to the bare `TREE_DROP_INDENT` only when fewer than
  `TREE_MIN_DROP_COLS` (16) columns remain; `TREE_MIN_DROP_COLS` added to the
  constants list.
- F-2 — "Command surface and run resolution": both skip causes (snapshot naming
  another slice; unreadable slice status), plus the canonical-directory
  candidate rule (RV-392 `F-2`).
- F-3 — "Delivery": config read is `dtoml::load_design_entry` / `design_entry`
  (bare-table, `[design]` only); `DoctrineToml` carries no `design` field; the
  shell call spelled `resolve_map_delivery(load_design_entry(root)?.as_ref())?`.
- F-4 — `slice selector add 266` (8 selectors, design-target); "Code impact"
  rows for `guard.rs`, `knowledge.rs` (and `record_titles` via `read_record`),
  `slice.rs`, the VT suites/fixture; selector list mirrored.
- F-5 — `slice selector rm 266 src/design_run/run.rs`; "(or `run.rs`)" and the
  list entry dropped.

`slice conformance 266` after: undelivered 0, conformant 20, undeclared 25 —
exactly F-6's tolerated foreign-commit paths.

### REVs completed
- None — the brief carried no governance/spec items.

### Withdrawn / tolerated
- F-6: tolerated — per-phase range registry cannot exclude interleaved foreign
  commits; rationale in the disposition.

Reconcile pass complete — handoff to /close.
