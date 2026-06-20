# REQ-313: Pinned fork-point with refresh-base as the sole explicit advance

## Statement

Stage-1 projections (`review/<N>`, `phase/<N>-NN`) are parented on the pinned
fork-point `trunk_base_B = merge-base(dispatch/<N>, trunk)` resolved at
`prepare-review` time — **never the live trunk tip**. The fork-point advances by one
mechanism only: `refresh-base`, a real `git merge --no-ff` of trunk into `dispatch/<N>`
in the coordination worktree, after which the operator re-runs `prepare-review`. There
is no silent-reparenting path.

## Rationale

A foreign commit landing on trunk between `coordinate` and `sync` must not silently
reparent the projection and distort its diff. Pinning the fork-point keeps projection
diffs exact; making every advance an explicit merge commit on the dispatch branch makes
base movement auditable rather than implicit (RV-030 F-1).
