# claude-arm dispatch: place coordination worktree inside the cwd-jail; funnel workarounds

## The rule

When running `/dispatch`'s **claude arm** under a harness that confines the
persistent Bash cwd to the primary working directory (e.g. a bubblewrap jail
rooted at `/workspace/<repo>`), **create the coordination worktree INSIDE that
primary dir** — use the `.dispatch/SL-<n>` convention, not an outside sibling.

The claude arm places workers at base B by parking the Bash cwd in the coord
tree before spawn (Agent `isolation: worktree` forks the cwd's HEAD under
`worktree.baseRef='head'`; see mem_019ec65ecbc77282bad7e10a5240ad27). If the
coord tree is an **outside sibling** (`/workspace/<repo>-dispatch-N`), the jail
**silently reverts** cwd to the project root on the next Bash call — `cd` never
sticks, the session stays on `main`, and the worker forks `main` instead of B.
Fatal for dependent phases: a worker forked off `main` can't see prior phases'
code committed only on `dispatch/<slice>`.

Empirical test (SL-111, 2026-06-19): `cd` into a path *inside* `/workspace/doctrine`
persists across calls; `cd` to the outside sibling reverts. Confirm with a bare
`cd <coord> ; pwd` in a separate call before trusting placement.

## Two funnel workarounds also hit on the claude arm (SL-111)

- **`verify-worker` refuses `unstamped`** — Agent-tool worktrees carry no worker
  marker (only subprocess-arm `worktree fork --worker` stamps). verify-worker is
  diagnostic-only; prove base==B directly instead: `git -C <wt> rev-parse HEAD^ == B`,
  `rev-list --count B..HEAD == 1`, `merge-base --is-ancestor B HEAD`. (IMP-072
  would stamp at creation.)
- **`worktree import` corrupts the patch** — it strips the trailing newline before
  `git apply`, yielding `corrupt patch at <stdin>:<last-line>` on a perfectly valid
  single-commit delta (ISS-032). Workaround = its exact equivalent:
  `git diff B..fork > p.patch ; git apply --3way --index p.patch` (newline
  preserved), then continue the funnel (verify → branch-point-check → one commit
  → record-boundary).

## Provenance / related

- ISS-031 (placement precondition), ISS-032 (import newline bug), ISS-029
  (the original missing-cd hazard). Three distinct claude-arm gaps surfaced
  funnelling one slice (SL-111 PHASE-03).
- mem_019ec65ecbc77282bad7e10a5240ad27 — Agent isolation:worktree forks Bash cwd
  HEAD; cd into coord tree to place at B (the mechanism this rule depends on).
- mem_019ec6142d3b71008f2149a6d84ba981 — base controllable by placement under
  baseRef=head.
