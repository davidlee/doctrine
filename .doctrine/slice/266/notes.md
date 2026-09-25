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

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open
