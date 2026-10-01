# ISS-514: Review verbs refuse ordinary linked worktrees

## What

Review verbs refuse to run in an ordinary linked worktree / fork (only `review new`
succeeds; everything after it refuses). This contradicts:

- AGENTS.md: "If auditing / closing a feature, land it on a worktree".
- The capsule adopt guidance: "audit here, not on edge".

ISS-275 (closed) fixed the refusal for the dispatch coordination worktree only.

## Fix sketch

Either admit review verbs in linked worktrees (resolving the ledger the same way the
coord tree does), or change the guidance and make `review new` refuse too, so the
refusal arrives before any work.

## Evidence

`01a009a2` (2026-08-16), `01a0adfa` (09-17, new succeeds, rest refuse), `01a0ded3`
(09-26), `01a0e021` (09-26, capsule adopt). Earlier coord-tree instances `019fa925`,
`019fb17f` were covered by ISS-275.
