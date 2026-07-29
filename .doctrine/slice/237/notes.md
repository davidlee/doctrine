# Notes SL-237: Primary-resolve runtime phase state

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-29 · stage: design authored, internal adversarial pass integrated · 26ecb5f4

### Produced

- SL-237 — scope doc (`slice-237.md`), selectors seeded, status `design`.
- CHR-050 — runtime-state scope audit for `.doctrine/state/dispatch/` + `review/`
  (the deliberately-excluded siblings). Committed.
- `research/research.md` — pre-design evidence artefact, self-serve (both spawned
  threads failed). **Runtime tier, gitignored** — harvest its durable findings at
  close before it is discarded.
- obs `019fa916-1a31` — `.doctrine/state/` flat-drawer survey cost.
- obs `019fa932-8c8f` — `pi --print` buffering makes a working research thread
  indistinguishable from a hung one.
- obs `019fa953-d669` + **ISS-276** — research agent scripts hang on tool-using
  prompts (3/3 failed); trivial no-tool prompt returns instantly. Committed.
- obs `019faac3-3fe6` — a multi-file grep silently omitted matches from its
  FIRST file argument; would have falsified the fan-out finding.
- `design.md` — authored, internal adversarial pass integrated (§10, 4 findings).
- **DEC-095..098** — the four locked design decisions.
- **REV-043** — restates ADR-006 D2/D4 + SPEC-012 § tier merge-safety.
- design-target selectors recorded (6 paths).

### Learned

Durable findings — candidates for `/record-memory` at close, not yet recorded:

- `primary_worktree` (`src/git.rs:564`) forks `git worktree list --porcelain`
  **per call, uncached**. Any path constructor that resolves it internally
  inherits that cost per invocation.
- `slice list` fans `phase_rollup` over **every** slice meta
  (`src/slice.rs:2158-2164`); with 221 slice dirs here, a per-call git resolution
  in `phases_dir` becomes ~221 subprocesses. The fan-out is at command level, not
  in the inner loops.
- `run_reconcile_phases`' refuse-when-live bail (`src/slice.rs:1236-1244`)
  **precedes** its gather (`:1252`), so its `coord_rows` is always `None` — one of
  `gather_truth_inputs`' two callers has never exercised the coord branch.
- ~~ADR-006's "per-worktree by construction" sits in **Context**, not a
  Decision.~~ **Superseded in `/design`.** True of Context force 2, but the
  research round stopped there. ADR-006 **D4** ("runtime state stays
  gitignored/per-worktree") and **D2** ("the coordination/runtime tier is
  withheld") are **Decisions**, and both are falsified. Hence REV-043.
- **There are TWO 221× fan-outs, not one.** `list_rows` (`slice.rs:2161`) *and*
  `lazyspec::load_slices` (`lazyspec.rs:495` → `load_plan:518` →
  `phase_rollup:535`). The research round found only the first.
- `conformance_outcome` (`slice.rs:2852`) states ISS-269's defect in its own doc
  comment: *"`root` resolves both the shared registry (via the primary worktree)
  and the local phase-sheet state tree."*
- A newtype makes a fan-out **visible at review, not impossible**. The missed
  second fan-out is the proof; don't let a type stand in for a call-site check.

### Open

All pre-design opens are settled. Remaining:

- **Design lock** — internal adversarial pass is integrated; the design has **not**
  yet had an external hostile pass, and the user has not yet granted explicit
  approval. `/plan` is gated on that.
- **REV-043 is `proposed`** — not approved, not applied. It must land before or
  with the slice's close, or the code ships falsifying an accepted Decision.
- **ASM (carried)** — no repo ⇒ the given root is its own primary. Load-bearing
  for ~20 existing tests on non-git tempdirs; a design invariant (§5.5 I-1), not
  error handling.

Settled this session:

- **DEC-095** — resolution mechanism: `PrimaryRoot` newtype minted in the shell.
- **DEC-096** — `resolve_phase_truth` narrows (keeps the `Some` matrix);
  `--across-trees` → `--truth`.
- **DEC-097** — mint the `phases` symlink in the primary only.
- **DEC-098** — migrate `boundaries_path` in-slice; it becomes infallible.
- **QUE (settled)** — spec sweep: no blocking REQ in SPEC-012 / SPEC-021 / PRD-015.
- **QUE (settled)** — worker read exposure: **accepted explicitly**, not fenced
  (`design.md` §7 D5); now a stated non-goal and named in REV-043.
- **OQ-1 (closed in review)** — DEC-098's contract change hides nothing; both
  propagating sites fail closed for independent reasons (`design.md` §10 R-1).
- **Correction landed** — `slice-237.md` objective 5 now reads "narrow", and
  objectives 6/7 were added.
