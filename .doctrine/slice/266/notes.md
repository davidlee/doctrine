# Notes SL-266: Inquiry map tree view

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design triage (exploring, 2026-09-25)

Evidence: `research/research.md` (runtime, gitignored) — cited below by its
X-/F- labels.

- **Reference, not spec.** The tree mockup is a screenshot of *hydra* (a
  friend's MIT Rust tool built from the user's original idea). It sets the feel;
  its vocabulary (e.g. "cauterised") is not ours.
- **Constraining governance.** SPEC-029 REQ-433 (one envelope, every rendering
  a projection of it) → the tree is an envelope rendering, so the envelope
  gains an opt-in whole-map field (X1). PRD-019 REQ-417 (blocked is derived),
  REQ-425 (never imply the map is complete), success measure "without asking
  the agent to summarise" (X5). STD-001 named constants; STD-003 disclose the
  auto-chosen run.
- **Shaping decisions (proposed).** `--format tree` + `design tree` alias;
  relay obligation rides the envelope, gated on config + derived map change,
  not the digest-bound `inquiry.md` fragment (X2).
- **Open questions.** Envelope field shape; flag partition for tree; default
  run resolution (X4); `map_view` name/default; what counts as a map change;
  glyphs/columns/width; SL-264 `blocking` marker (X6).
- **Assumptions.** The per-turn envelope is not digest-bound the way fragments
  are (inferred — verify in design).
- **Risks.** SL-264 (ready, uncommitted in the primary tree) edits the same
  structs — implementation waits for it. Prune/defer carry no reason (F4);
  out of scope.

## Further review (2026-09-26, after RV-388 concluded)

No further pass needed. RV-388 ran three raiser rounds (codex): 20 findings,
all verified, round 3 raised nothing new. The round-2/3 edits (wrapping,
relay scope, run-selection wording, lazy `[design]` config) were each
re-verified by the raiser. What remains unproven is implementation-level
(wrap goldens, config refusals) and is pinned by VT-7, VT-10 and VT-12, which
the implementation review and audit will exercise.

## PHASE-02 — tree renderer (2026-09-26)

- **VH-1 amended the wrapping rule** (user, recorded on DEC-307): dropped text
  keeps its rails, one level in; the bare 8-column indent is the fallback only
  below `TREE_MIN_DROP_COLS` (16) beside the rails. Legend marks and letters
  are coloured like the nodes. `design.md` sec-3 rule 2 still states the
  locked (rail-less) form — reconcile at `/reconcile`.
- `MapAnswer::Record.form` stays on the model: the locked anatomy prints
  `REC-NNN <title>` for both forms, and `form` is carried by `json --full`
  serialisation — a live reader, so no model change.
- **Gap closed: the envelope had no canonical slice ref** for the header/footer
  (the leaf cannot format ids). User chose option 1: `TurnEnvelope.slice_ref`,
  filled from `project()`'s shell-formatted argument (DEC-292), serialised — an
  additive json key, version unchanged (DEC-291). `tests/e2e_design_show_golden.rs`
  gains the key (outside the declared affected surface; shows as undeclared).
- VH-1 verdict: "legible", after three amendments (rails on dropped text; legend
  marks/letters coloured; cursor suffix open-cyan bold, `pinned` magenta) —
  all recorded on DEC-307.
- Boundary tightened to `a51c614bb^..c47451f1a`; SL-267/SL-264 commits
  interleave it and ride in conformance's undeclared cell.
- Header counts partition the map: open excludes derived-blocked.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-26 · PHASE-02 completed, gated, VT-1..6 pass, VH-1 accepted · d7f49ae0a

### Produced

- DEC-303..DEC-310 (design decisions); IMP-472, IMP-473; RV-388 (design ledger)
- SL-266 PHASE-01, landed as `3e55c758b` (merge of `3ba03030e` — `unsettled_needs`
  + the `blockers()` sweep — and `3e745780f` — the `Full`-only whole-map envelope
  field with `project`'s `titles`/`selection`)
- SL-266 PHASE-02 on edge: `a51c614bb`, `d4c6f8ba9`, `78d8e0b44`, `c47451f1a`
  (renderer; VH-1 amendments; `TurnEnvelope.slice_ref`); DEC-307 amended
  (`4362c2516`, `cf02f2735`)

### Learned

- Observations (`records/`): 03, 94, 97, c6, 71, db (design-run friction);
  56 (`MapAnswer::Record.form` unread by either rendering), 58 (the fork guard
  vs a concurrent-agent tree), af (`land` refuses `tree-unclean`)
- `mem.pattern.worktree.solo-land-refuses-unclean-shared-tree`
- `blockers()` held a *second* blocked derivation that no `is_blocked` grep
  could reach — sweep the shape, not the callee
  (`mem.pattern.review.sweep-defect-class-not-instance`)
- A solo phase's boundary is never auto-recorded: `land`'s merge commit is
  rejected as `code_end` (non-merge required), so `slice record-delta` is the
  *normal* route for solo, not an escape hatch.
- `mem.pattern.render.textwrap-no-hyphenation-for-ids`; observation `c1`
  (envelope lacked a canonical slice ref)
- `mem.pattern.design-run.leaf-rule-binds-tests` bit: no `crate::` in
  `design_run` tests either (strip-ansi spelled locally in `tree.rs`)

### Open

- ~~`MapAnswer::Record.form` rendered by neither surface~~ — settled at
  PHASE-02: `json --full` carries it; no model change.
- `design.md` sec-3 rule 2 still states the rail-less drop — reconcile to
  DEC-307's amendment.
- PHASE-04's relay line must reuse the tree's `TREE_COMMAND` (widen to
  `pub(crate)`), not re-spell `doctrine design tree` (STD-001).
- `src/design_run/tests.rs` is a PHASE-04 `VT-1` test_file and currently reads
  `UNATTRIBUTABLE` (not modified by this slice yet) — expected, not a gap.
