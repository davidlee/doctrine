# Claude dispatch-agent worktree forks off the origin/main tracking ref — update-ref it per phase to control the worker base

The `/dispatch-agent` arm spawns the worker via the `Agent` tool with
`isolation: worktree`. **Claude bases that worktree on `refs/remotes/origin/main`**
— NOT session HEAD, NOT the dispatch base `B`. This is the concrete shape of the
arm's confessed base-pinning residual (M1): the base is "opaque and not
orchestrator-controlled" by doctrine, but it IS controllable via the local
tracking ref.

Observed on the SL-066 run: local `main` was 32 commits ahead of `origin/main`
(unpushed). The worker forked off stale `origin/main` (`7e2bc4b`), missing all the
SL-066 authored state + the SL-064 source, and correctly refused. Same root cause
breaks `doctrine worktree coordinate`, whose trunk ladder is
`origin/HEAD → main → master` (`src/git.rs` `trunk_ladder`).

**Handle (local-only, no network, reversible):**

```sh
git update-ref refs/remotes/origin/main <desired-base>
git symbolic-ref refs/remotes/origin/HEAD refs/remotes/origin/main   # if needed
```

- Before batch 1: set it to the dispatch base `B` (= local `main` tip when that's
  where the authored slice state lives).
- **Per phase:** each phase's worker must fork from the PRIOR phase's result
  (`dispatch/<slice>` tip = `B+1`), not from `B`. So **before every spawn**, point
  the tracking ref at the current `dispatch/<slice>` HEAD:
  `git update-ref refs/remotes/origin/main $(git -C <coord-wt> rev-parse HEAD)`.
  Then the worker's `S` parents on `B'`, and `import`'s `S^==B'` belt passes.

A real `git fetch` later corrects the tracking ref; until then it intentionally
"lies" so Claude's worktree isolation forks from the right commit. The proper fix
is pushing `main` to `origin` (then no override is needed), but that's an
outward-facing act — the ref-update is the no-network workaround.
