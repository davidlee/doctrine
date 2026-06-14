# ISS-015: dispatch claude-arm funnel: import verb corrupt-patch on valid diff, and .doctrine/dispatch lacks gitignore negation for boundaries.toml

Two SL-064 dispatch-funnel wiring defects surfaced by the first real claude-arm
`/dispatch` run (SL-066 PHASE-02):

**1. `doctrine worktree import` corrupt-patch on a valid diff.**
`worktree import --base B --fork S` failed with `git command failed: apply --3way
--index: error: corrupt patch at <stdin>:1341` on a clean single-commit fork whose
net diff `B..S` (1341 lines) `git apply --3way --index --check`es **cleanly** when
captured to a file and applied manually. `git` is the real binary (no rtk shim), so
this is an internal generation/stdin-pipe defect — likely the diff is piped to
`git apply` without its trailing newline / final hunk terminator. Workaround used:
import via `git checkout <fork> -- <net-diff paths>` (the
`mem.pattern.dispatch.rtk-git-diff-stat-use-checkout-import` path) after verifying
all belts (`S^==B`, single non-merge, no `.doctrine/`/`.claude/` touch) by hand.
Fix: make the import verb's diff→apply robust (emit via `git diff` to a temp file or
`git apply` from a process that preserves the patch byte-exact), with a regression
test on a multi-hundred-line fork.

**2. `.doctrine/dispatch` has no `.gitignore` negation.**
`dispatch record-boundary` writes `.doctrine/dispatch/<slice>/boundaries.toml`, and
`prepare-review` **tree-reads it from the dispatch branch tip**
(`src/dispatch.rs:121`, `mem.pattern.dispatch.sync-tree-reads-ledger-not-worktree`)
— so it must be committed. But this repo's blanket `.doctrine/*` ignore has no
`!.doctrine/dispatch/` negation, so `git add` refuses it (the same "adr trap" as
`mem.pattern.install.authored-entity-wiring`). Worked around with `git add -f`.
Fix: add the negation for the committable boundary file (scoped so the CAS journal /
disposable runtime stays ignored), per the authored-entity-wiring seam.
