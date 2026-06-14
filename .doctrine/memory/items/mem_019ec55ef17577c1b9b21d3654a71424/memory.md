# Dispatch sync sources the run ledger from the branch tip tree, not the working filesystem

The `dispatch sync` projection verb (SL-064 PHASE-04+) must source the run-ledger
manifests (`boundaries.toml` / `orthogonal.toml` / `journal.toml` under
`.doctrine/dispatch/<slice>/`) from the **committed `dispatch/<slice>` tip tree**
— `git::read_path_at` (`cat-file -p <ref>:<path>`) → `read_ledger::<T>` — NOT from
the working filesystem.

`ledger::read_boundaries`/`read_orthogonal` (`std::fs::read_to_string`) are the
**funnel's** read-modify-write side; using them in the sync is a trap that only
works when the coordination worktree happens to be checked out on
`dispatch/<slice>`. It silently returns empty when run from any other checkout
(e.g. an e2e harness on `main`, or stage-2 from the parent/root) — phases and the
orthogonal-exclude come back empty, NO error.

**Why:** design §4.1 — stage-2 `--integrate` runs **after the coordination
worktree is removed**; all of B/C/CAS are plumbing against refs+objects in the
common git dir, reachable with no checkout. Sourcing the ledger from the branch
tip makes stage-1 and stage-2 identical and gives one consistent source (the
branch the verb projects), matching the tip/tree reads B/C already do via
`filter_tree`.

**How to apply:** any sync-side ledger read goes through the branch tip
(`read_path_at`/`read_ledger`); reserve the filesystem `read_*`/`record_*` for the
funnel, which legitimately runs inside the coordination worktree. PHASE-05
integrate must tree-read too. See [[mem.pattern.dispatch.claude-agent-worktree-integrates-commit-onto-parent]],
[[mem.pattern.dispatch.rtk-git-diff-stat-use-checkout-import]].
