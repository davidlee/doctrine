# Correct a claude-arm worker delta in place (orchestrator resets fork branch, commits worker's staged bytes) — no re-dispatch

When a claude-arm dispatch worker has already `worker_commit`-ed C1 and a small
correction is needed, do NOT reject + re-dispatch a fresh worker (throws away the
whole turn, ~120k tokens). The orchestrator lands the worker's *corrected
working-tree bytes* as one fresh commit. Empirically verified SL-206 PHASE-02
(2026-07-06, this harness, Opus 4.8).

## Why the worker can't self-amend

- `worker_commit` mints exactly ONE commit at `HEAD==B`. Once C1 exists, tip≠B ⇒
  it refuses `stale-record`. No amend/replace mode.
- The worker CANNOT `git reset --soft B` to retry: its linked-worktree git dir
  `.git/worktrees/<agent>/` is **read-only under the jail** — `git reset` fails
  `cannot lock ref 'HEAD': ... Read-only file system`. So the worker cannot move
  its own branch ref.
- BUT the worker's **Edit/Write tools still work** (the worktree *files* are RW;
  only `.git` ref writes are RO). A compaction-revived worker (`SendMessage` to
  the agentId — the `dispatch/agent-<id>` branch suffix) can apply the correction
  to the working tree, verify it (tests/lint), and hand back the uncommitted diff.

## The orchestrator lands it (land-not-rewrite)

The orchestrator's view of `.git/worktrees/<agent>/` is **RW** — the RO is the
worker's jail mount, not the host. So:

```
WT=.dispatch/SL-<n>/.worktrees/agent-<id>
git -C "$WT" reset --soft <B>            # branch → B; corrected bytes staged
git -C "$WT" add -- <changed files>
git -C "$WT" commit --author="<worker author>" -F - -- <changed files> <<'MSG'
... corrected message ...
MSG
# then the normal branch-based funnel:
dispatch_import{slice:<n>, name:"dispatch/agent-<id>"}
dispatch_conclude_phase{...}; dispatch_reap{...}
```

This is **land-not-rewrite**: the orchestrator commits the worker's OWN staged
bytes verbatim (authors nothing), preserving the worker `author` while the
committer becomes the orchestrator/`dispatch@doctrine` — exactly the identity
split `dispatch_import` would have produced. Confirm `C1'^==B`, single non-merge,
only-source files (R-5), then import.

## Recipe gotchas

- `git commit ... -F - -- <paths>` — the `-F -` MUST precede the `--` pathspec.
  Put it after and git reads `-F -` as pathspecs (`pathspec '-F' did not match`),
  no-op.
- Worktree-free `dispatch_import` composes coord-tip ⊕ **branch tip** (object-db);
  it never reads the worktree. So the correction must be *committed* to the fork
  branch first — an uncommitted working-tree diff would NOT be imported.
- Since coord-tip==B, the composed S tree == the fork's checkout, so you can run
  the prove gate + full suite in the worker worktree (a materialized S) even
  though the coord worktree stays stale after import.

Relates to [[mem.pattern.dispatch.unarmed-agent-worker-runs-in-coord-tree]] and
the `worker_commit` / `dispatch_import` funnel (SL-198/199).
