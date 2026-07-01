# Claude-arm dispatch Fork path: worktree persists + footer worktreePath — proven live (VH-1)

SL-182 VH-1: armed-branch Fork-path worker tree persists post-return; footer carries worktreePath; full funnel green.

## What was proven (live, 2026-07-01, SL-182 VH-1)

A claude `Agent` worker spawned with `isolation: worktree` from the dispatch spawn
dir (armed at explicit base B → **Fork** path, checked out on `dispatch/<name>`):

- **Footer `worktreePath` is present on the armed BRANCH case**, not just the
  detached Passthrough case that the earlier probe covered (`mem_019efe28…` P2 probed
  a detached tree with `worktreeBranch: undefined`). Derive identity from it:
  `name = basename(worktreePath)`, `branch = dispatch/<name>`. Do NOT read
  `worktreeBranch` (undefined for hook-created trees).
- **The tree persists on disk post-return** — registered in `git worktree list`,
  HEAD==B, tracked+untracked delta intact. Because `create-fork` is the
  `WorktreeCreate` hook and **no** `WorktreeRemove` hook ships, the harness does not
  auto-reap (`hooks.md:2390/2442`). This is what lets the orchestrator import the
  live tree. INV-6.
- **The 5-step funnel runs green live:** footer → `verify-worker --dir --base B
  --branch dispatch/<name>` → `import --from-worktree <wt> --base B` (exit 0,
  tracked+untracked both `--index`-staged) → reap `git worktree remove --force`
  **gated on import exit 0 / committed delta** (F-3 — never reap an unimported delta).
- **Confinement held:** bash writes above the worktree + into the host repo hit a
  read-only filesystem (the bwrap jail); the Write tool was blocked outside the
  worktree; self-commit failed on ro-`.git`. Canaries byte-intact.

## Caveat (do not overclaim)

The Write-tool escape (B3) was denied by the **harness** `isolation: worktree` guard,
which pre-empts doctrine's own `Edit|Write` pretooluse deny. So doctrine's *specific*
Edit|Write hook firing for a subagent is inferential (config-verified + INV-4
unit-tested); the **Bash** wall is the one exercised live. Defense-in-depth: the
vector is closed either way.

## Method note

VH-1 used a **synthetic** worker delta (a README line + one untracked file) because
the slice had no phase left to land. The S1 regression capture/diff and
`record-boundary` beats are **skipped** for such a mechanism probe — a regression
suite is meaningless on throwaway junk and a boundary row would pollute the registry.
The load-bearing import belt (`.doctrine/`/`.claude/` reject, HEAD==B, tree_clean)
still runs real against the live tree.

Supersedes the "pinned at VH-1" hedge in SL-182 design §5.5 ASM / §6 OQ-2 residual.
Related: [[mem.fact.claude.worktree-remove-auto-teardown]],
[[mem.pattern.doctrine.phase-complete-clobbers-boundary]].
