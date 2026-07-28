# Notes SL-237: Primary-resolve runtime phase state

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-29 · stage: research complete, pre-design · b57e0fb91

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
- ADR-006's "per-worktree by construction" sits in **Context**, not a Decision —
  context drift, not a decision conflict.

### Open

- **DEC (open)** — the resolution mechanism: `PrimaryRoot` newtype threaded from
  the shell (preferred) vs process-level memoisation vs per-call-site resolution.
  See `research.md` § Design-input deltas 3.
- **DEC (open)** — what becomes of `slice status --across-trees`
  (`src/slice.rs:1121`): retire, or repurpose to render landed-vs-sheet?
- **QUE (open)** — do SPEC-012 / SPEC-021 / PRD-015 carry any REQ mandating
  per-tree phase state, or load-bearing on `--across-trees` / `reconcile-phases`?
  Sweep was NOT done (thread failure); a narrow re-run was launched 2026-07-29.
- **QUE (open)** — migrate `boundaries_path` (`src/state.rs:716`) onto the same
  mechanism in this slice, or defer? Same layering smell, never bitten (not on a
  fan-out path).
- **ASM (carried)** — no repo ⇒ the given root is its own primary. Load-bearing
  for ~20 existing tests on non-git tempdirs; treat as a design invariant, not
  error handling.
- **Correction pending** — `slice-237.md` objective 5 is misstated ("retire the
  composite"). `resolve_phase_truth` *narrows*: loses coord-vs-local, keeps
  landed-vs-sheet. `/design` must fix the scope doc.
