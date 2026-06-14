# Claude dispatch-agent worker commit integrates onto the parent branch, not an isolated fork

Observed across all three SL-062 phases (2026-06-14): a `/dispatch-agent` worker
spawned via the `Agent` tool with `isolation: worktree` does NOT preserve the
funnel's orchestrator-sole-writer split. The worker's single non-merge commit ends
up **directly on the coordination branch (`main`)** — no registered worktree
remained (`.git/worktrees/<name>/HEAD` absent), `main@{0}` in the reflog is the
worker's commit, and the worker could see the parent tree's foreign untracked files
(so it ran IN the shared tree, not an isolated checkout).

**Why:** the `Agent` `isolation: worktree` integrates the worker's commit back onto
the parent branch on completion (the worktree is created, committed in, then
collapsed onto the parent). It is NOT the codex/pi `worktree fork --worker` model
where the delta stays on a separate fork branch for the orchestrator to `import`.

**Consequence:** the strict funnel steps that normally gate BEFORE the commit — the
R-5 `.doctrine/`/`.claude/` belt and the combined-tree verify — necessarily run
**POST-landing** on the claude arm. A straying worker's bad commit is already on
shared `main` and would need a revert (disruptive on a live shared branch).

**How to apply (claude arm, until SL-064 lands a dedicated coordination worktree):**
- Harden every worker brief: stage ONLY declared files by exact path, NEVER `git add
  -A`/`git commit -a` — foreign untracked sit in the shared tree.
- After each worker returns, run the orchestrator checks on the LANDED commit: net
  diff `B..S` is exactly the declared files, R-5 clean, `S^==B` (or linear past
  foreign commits), and `just gate` green on the combined tree. Trust this, not the
  worker's self-report or the IDE/LSP diagnostics (which read cross-worktree stale —
  e.g. a phantom `lib.rs` finding in a crate with no `lib.rs`).
- The funnel's correctness GOALS still held on SL-062 (clean, R-5, linear, verified);
  only the sole-writer MECHANISM was bypassed.

Related: [[mem.system.dispatch.orchestrator-on-shared-main-contention-cost]] (the
dedicated-coordination-worktree open question SL-064 picks up),
[[mem.pattern.dispatch.glob-add-sweeps-foreign-untracked-on-shared-main]] (the
exact-path-staging discipline), [[mem.pattern.tooling.stale-lsp-diagnostics-after-build]].
