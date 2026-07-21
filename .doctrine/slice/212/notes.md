# Notes SL-212: Ingest hand-resolved trunk merge

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## 2026-07-22 — final projection review

- Opened **RV-289** with two unresolved findings: F-1 blocker (the generic
  linked-worktree cwd test also rejects the intended dispatch coordination
  worktree) and F-2 major (`checkout-index -af` leaves paths deleted from
  `T_c` behind in the working tree).
- Git 2.54.0 linked-worktree probe confirmed the normal conflict path works:
  unmerged stages block an early commit; resolve + `git add` + `git commit`
  yields ordered parents `[base, source]`, and worktree-private merge metadata
  resolves under `.git/worktrees/<name>/`.
- A second probe confirmed `git read-tree --reset -u T_c` removes a cleanly
  deleted path and leaves the index tree equal to `T_c`; this is evidence for
  `/plan`, not a design edit.
- Review/notes remain uncommitted: the sandbox exposes `.git/index` read-only,
  so `git add`/commit and `doctrine memory record` both fail creating
  `.git/index.lock`. No code changed; `doctrine check gate` was not run.
