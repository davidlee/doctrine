# Workflow isolation:worktree leaf cannot commit in the jail

A Claude Code **Workflow** `agent({isolation:'worktree'})` leaf is minted as a
**linked git worktree on the primary `.git`**, and in the bubblewrap jail that
shared `.git` is **read-only**. So the leaf **cannot produce a git commit by any
path**.

## Proof (probe `wf_01ed2bf2-ff8`, one leaf)

```
pwd                = /workspace/doctrine/.worktrees/wf_01ed2bf2-ff8-1
git_dir            = /workspace/doctrine/.git/worktrees/wf_01ed2bf2-ff8-1
git_common_dir    = /workspace/doctrine/.git          # linked worktree, shared store
git_common_writable = RO
git add            → fatal: Unable to create '.../wf_..-1/index.lock': Read-only file system
git commit         → exit 1 (nothing staged)
```

The working-tree **fs is RW** (`echo probe > _commit_probe.txt` succeeded) —
only the git **metadata store** (`.git`, incl. the per-worktree index/lock) is
walled. This is the SAME wall that fails the pi-arm `git worktree add`.

## Both commit paths are closed for a workflow leaf

1. **plain `git commit`** — RO shared `.git` (this probe).
2. **`worker_commit` MCP** — refuses. `run_worker_commit` resolves the opaque
   agent via `resolve_agent(root, agent)` to an already-registered
   `DispatchRecord` (provisioned by `dispatch setup` / `create-fork`, marker
   stamped). A **harness-minted** workflow worktree is NOT a `DispatchRecord`, so
   resolution returns `unknown-agent` (`src/mcp_server/worker_commit.rs:219-224`,
   test `worker_commit_unknown_agent_refuses`).

## Why the claude-arm worker CAN commit (the contrast)

Same RO `.git` — but the claude-arm worker is (a) spawned into a worktree that
**doctrine registered as a `DispatchRecord`**, and (b) commits via the
**server-side, UNCONFINED** `worker_commit` bypass (the MCP server process has RW
`.git`; the worker never touches `.git` directly). A workflow leaf has neither:
no registration, and no way to register (its only worktree-creating path, git in
the jail, is walled).

## Consequence for `/drive-slice` (SL-206)

Combined with [[mem_019f36028bca7411b33fde4981aaba85]] (workflow leaves can't
nest-spawn — no `Agent`), a workflow leaf can **neither spawn a child nor
commit**. Therefore a **Workflow script cannot drive committing dispatch work in
the jail** — not as orchestrator, not as direct worker. The proven committing
path (claude arm) requires an **`Agent`-tool-spawned** worker in a **registered
`DispatchRecord`** committing via server-side `worker_commit` — all three of
which a workflow denies.

Forces the SL-206 fix off the workflow form (Architecture **A** — an
`Agent`-tool-spawned orchestrator, i.e. the existing claude-arm dispatch shape),
OR new server-side machinery to adopt+commit a harness-minted worktree
(Architecture **B'** — extends the `worker_commit` family). Retracts D
(workflow-direct-worker) entirely.

## Relates

- [[mem_019f36028bca7411b33fde4981aaba85]] — no `Agent` tool (the other half).
- [[mem_019f331005d776c1a65c65bfe59581bf]] — the fork DOES mint at the armed base;
  positional arming is real, but the fork it mints is still commit-walled.
- [[mem_019f328ad54c70318abc6f7e1db767fc]] — WorktreeCreate hook / placement.
