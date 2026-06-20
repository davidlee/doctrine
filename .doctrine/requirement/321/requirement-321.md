# REQ-321: Evidence reconstructability

## Statement

The evidence refs published by stage-1 are reconstructable from the recorded model: the
pinned fork-point, the dispatch branch tip, the committed boundaries, and the tree
filters together determine `review/<N>` and `phase/<N>-NN` deterministically. Given the
same inputs, re-deriving an evidence ref yields the same tree.

## Rationale

Reconstructability is what lets immutable evidence (REQ-312) be a durable audit
substrate rather than an opaque snapshot: a reviewer can independently re-derive what
was reviewed, and a future agent can reason about a past run from refs + ledger alone,
without the original working tree.
