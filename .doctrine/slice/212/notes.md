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

## 2026-07-22 — reconciliation audit (RV-290)

- **Gate + tests green this session** (supersedes the prior "not run" note):
  `doctrine check gate` clean (no clippy warnings; corpus citation-lint noise is
  pre-existing/unrelated); `cargo test --test e2e_dispatch_candidate` → **35/35
  pass**, incl. ingest, D2-immunity (`..._immune_to_merge_config`), F-2
  clean-deletion projection (`..._projects_clean_deletion`), ordered-merge, and
  the refusal taxonomy; pure `validate_ingest_provenance` unit set + atomic-store
  regression present.
- **RV-289 resolved.** F-1 (blocker) + F-2 (major) were `answered/design-revised`;
  confirmed both are *implemented + tested* (F-1: `CANDIDATE_WORKTREE_SUBPATH`
  guard + refusal/acceptance tests; F-2: `read-tree --reset -u T_c` exact
  projection + regression) → verified terminal. RV-289 now `done · await=none`.
- **RV-290 (this audit) `done · await=none`.** Two conformance findings, both
  non-blocking: F-1 minor — `src/mcp_server/dispatch.rs` undeclared (a benign
  `MergeTree::Conflict { .. }` enum-ripple from PHASE-01) → reconcile adds the
  `design-target` selector; F-2 nit — the slice's own `plan.toml`/`slice-212.toml`
  churn → aligned (inherent conformance noise).
- **No governance write owed** — REV-030 already applied; ADR-012 §Verification
  operator-ingest cases realised as VTs. Follow-ups IMP-303 (exact-OID audit
  binding) and IMP-305 (crash-resume/restage) exist and are `related`-linked;
  nothing new to file.
- Implementation matches the locked design across every load-bearing invariant
  (D1 predicate, D2 one-engine, D7 atomic store, write-once pre-state gate,
  fail-closed guards). Handoff → `/reconcile` (sole reconciliation-brief item:
  the F-1 selector add).
