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

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-10-17 · ready · 46d68889d

### Produced
- design locked after RV-421 (19 findings, all verified; commits 4e2387bc3..b2669cf3e)
- amended on user rulings: DEC-379 — linked-worktree marker fallback; DEC-382 — hint covers doctrine's own writes; DEC-384 — one-time cleanup, customise refuses pre-stamp
- plan: PHASE-01..06 authored and materialised (b19bd71ab, d015f38c3)
- minted: ISS-522 — root-marker divergence install vs readers; ISS-523 — `--only-memory` unwired; ISS-524 — `--global` runs the project install

### Learned
- mem.pattern.design-run.review-disposition-route-and-vocab — severe design-review findings need `--route` at dispose or the lock refuses (hit here; existing memory not retrieved beforehand)

### Open
- none — DEC-378..385 settled; deferred work is on ISS-522..524
