# REQ-321: Evidence reconstructability

## Statement

The evidence refs published by stage-1 are reconstructable from recorded state, by
**distinct** derivations:

- `review/<N>` = the `dispatch/<N>` tip tree, **minus** `.doctrine/dispatch/<N>` and
  every committed `Verified` orthogonal-marked path (`orthogonal.toml`), parented on the
  pinned fork-point.
- `phase/<N>-NN` = each committed `boundaries.toml` row's code tree, `.doctrine` stripped,
  chained off the pinned fork-point.

Given the same recorded inputs, re-deriving an evidence ref yields the same tree.

## Rationale

Reconstructability is what lets immutable evidence (REQ-312) be a durable audit
substrate rather than an opaque snapshot: a reviewer can independently re-derive what
was reviewed, and a future agent can reason about a past run from refs + ledger alone,
without the original working tree.
