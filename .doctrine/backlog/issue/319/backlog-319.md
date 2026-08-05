# ISS-319: Fresh-id allocation fails open when the trunk ref is unreachable

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The defect

`git::trunk_entity_ids` returns `Ok(Vec::new())` when the trunk tree-ish cannot be
resolved or `ls-tree` yields nothing (`src/git.rs:1646-1668` — both the
`let Some(tree_ish) = … else` and the `let Some(listing) = … else` arms return an
empty vector, not an error).

`entity::next_id(local, trunk)` then computes `max(local ∪ trunk) + 1`
(`src/entity.rs:215-218`). With an empty trunk set it degrades silently to
`candidate_id(local)` — allocation from the working tree alone.

So an environment that cannot reach the trunk ref does not refuse to allocate. It
allocates from a partial view and mints an id another environment may already hold.
The degradation is invisible: no warning, no distinct status, no error.

ADR-006 D3 states the invariant this breaks: *"Minting is a trunk-side act. Ids are
allocated against the configured trunk ref (the id baseline …) before a worktree
forks."* The code honours that when the trunk is reachable and abandons it silently
when it is not.

## Why it matters now

Today the exposure is limited: ADR-006 D2 funnels all doctrine-mediated writes,
including id minting, through the orchestrator on the coordination branch, so parallel
workers do not allocate concurrently. The `GitRef` claim backend (`src/reserve.rs:309`)
covers the cross-clone case where a shared remote exists.

Under the capsule model an execution capsule has **no canonical ref access by
construction** — that is the authority boundary ADR-020 adopts, not a misconfiguration.
So every capsule that mints an entity hits this path, and hits it in exactly the
condition that produces collisions. A fail-open in a system whose other refusals are
fail-closed.

Found while researching SL-248 (`.doctrine/slice/248/research/research.md`, thread 3).

## Status of the finding

Structurally confirmed by reading both functions; **not yet reproduced**. A repro would
provision a repository with no reachable trunk ref, mint two entities of the same kind
from two such trees, and observe identical ids.

## Disposition candidates

- Refuse allocation when the trunk ref is configured but unreachable, distinguishing
  "no trunk configured" (legitimately local, e.g. a fresh project) from "trunk
  configured and unreachable" (currently indistinguishable).
- Or return a typed degradation the caller must handle, rather than an empty vector.

The capsule-specific answer is a separate question — see `QUE-208`. This item is the
existing fail-open, which is a defect independent of capsules.
