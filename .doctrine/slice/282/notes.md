# Notes SL-282: Install explains itself

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design review (RV-421)

2026-10-17. Three passes: one Claude pass (F-1..F-8), two codex `gpt-6-sol`
passes (F-9..F-19 raised; contests on F-11, F-13, F-14 re-answered). All 19
verified. User rulings amended DEC-379 (linked-worktree marker), DEC-382
(hint covers doctrine's own writes) and DEC-384 (one-time cleanup;
customise refuses before the stamp). Deferred: ISS-522, ISS-523, ISS-524.

**Further review pass: none needed before lock.** The last pass verified
every answer and its sweep of the revised passages found nothing material.
What remains is implementation risk, which tests settle better than another
design read:

- `write_installed_version` preserving this repo's commented `doctrine.toml`
  byte-for-byte outside the key;
- the `wire` → `plan_wire` / `apply_wire` split leaving `boot install`
  output unchanged;
- `read_install_marker`'s main-worktree check across git layouts
  (linked, `--separate-git-dir`, non-git);
- legacy-starter removal touching only the seven identical copies here.

## Implementation notes (audit harvest from phase sheets)

- Boot-guard upgrade iterates `PRIOR_BOOT_REF_GUARDS` newest-first: guards grow
  by appended sentences, so an older prior can be a prefix of a newer one.
- `Touched` coverage is best-effort: hook/MCP writes report `Changed` even when
  created; memory sync, convergence and npx skills report no paths by
  construction — the commit hint inherits those gaps.
- Refused boot wiring (settings fallback, MCP fallback, foreign pi extension)
  fails the install leg (RV-422 F-7); a conflicting `worktree.baseRef` does not —
  it is a respected user override.
- Linked-worktree marker lookup: only a non-git `rev-parse` exit or
  `git_dir == common` is "ordinary checkout"; every other failure is Unlocated
  (RV-422 F-3).

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-10-17 · close · adf0240f2

### Produced
- PHASE-01..07 implemented (43dbc692c..71ebbfe86); PHASE-07 appended for the downgrade gate split from PHASE-03
- audit RV-422: 10 findings; fixes e50b1a131 (F-1, F-2, F-3, F-5, F-7, F-9, F-10)
- reconcile: REV-073 — SPEC-023, REQ-322, REQ-329 to starters-on-demand; DEC-382 amended (hint filter, one command per line); design + selectors (adf0240f2)
- design locked after RV-421; DEC-379, DEC-382, DEC-384 amended on user rulings
- minted: ISS-522 — root-marker divergence install vs readers; ISS-523 — `--only-memory` unwired; ISS-524 — `--global` runs the project install; CHR-180 — two stale memories

### Learned
- mem.pattern.design-run.review-disposition-route-and-vocab — severe design-review findings need `--route` at dispose
- mem.pattern.rust.pathbuf-eq-ignores-trailing-separator — PathBuf equality hid a rendered trailing `/`

### Open
- none — PHASE-06 VH-1 passed (user scratch-repo check, 2026-10-17); deferred work is on ISS-522..524, CHR-180
