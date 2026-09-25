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

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-26 · PHASE-01 landed, gated, verified · 3e55c758b

### Produced

- DEC-303..DEC-310 (design decisions); IMP-472, IMP-473; RV-388 (design ledger)
- SL-266 PHASE-01, landed as `3e55c758b` (merge of `3ba03030e` — `unsettled_needs`
  + the `blockers()` sweep — and `3e745780f` — the `Full`-only whole-map envelope
  field with `project`'s `titles`/`selection`)

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

### Open

- `MapAnswer::Record.form` is rendered by neither surface — decide at PHASE-02.
- `src/design_run/tests.rs` is a PHASE-04 `VT-1` test_file and currently reads
  `UNATTRIBUTABLE` (not modified by this slice yet) — expected, not a gap.
