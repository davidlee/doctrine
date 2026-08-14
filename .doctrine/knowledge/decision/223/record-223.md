# DEC-223: Solo in-place and solo worktree execution remain trusted modes

## Context

The capsule programme changes the authority boundary for dispatched work. It
does not imply that every execution path must pay capsule provisioning,
quarantine, normalization, and admission costs.

## Decision

Doctrine retains two trusted solo execution modes outside capsule dispatch:

- in-place execution in the control-plane repository, with ordinary verified,
  path-limited commits;
- isolated solo execution in a main-repository worktree, using the surviving
  worktree provisioning, `land`, and ancestry-based `gc` machinery.

Neither mode is forced through capsule ingestion or admission. Capsule execution
is the separate untrusted-worker lane. A later migration of either solo mode
would require its own decision and motivation.

## Consequences

- The simple `/execute` path remains low ceremony.
- Worktree support cannot be deleted wholesale at capsule cutover.
- Shared semantics may be reused across modes, but their trust and commit
  mechanisms remain distinct.
