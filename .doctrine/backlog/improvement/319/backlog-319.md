# IMP-319: Subprocess workers need brokered observation writes to the primary corpus

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

SL-231 is designing repository-wide observation capture: worker observations must
land in the primary corpus so friction from rejected, abandoned, or deleted worker
branches is not systematically lost.

The Claude dispatch arm can expose the narrow observation-write operation as an MCP
tool. Its server runs outside the worker's additional bwrap jail and can write the
primary tree. The subprocess arm cannot reuse that mechanism as currently deployed:
its stdio MCP process is a child of the jailed codex/pi worker and inherits the
read-only boundary. RSK-225 records this arm divergence.

Without another egress mechanism, subprocess observations remain trapped in the
worker worktree and disappear when that fork is discarded, biasing the corpus
toward successfully integrated work.

## Desired outcome

Provide a narrowly authorized subprocess-arm path that writes an observation
through SL-231's validated, create-only operation into the primary repository
corpus while preserving worker confinement.

Candidate mechanisms include:

- a persistent MCP service launched outside the subprocess jail;
- an orchestrator-owned observation relay;
- a purpose-built broker endpoint; or
- a funnel import channel dedicated to immutable observation files.

The mechanism must preserve UUID replay semantics, return the primary sink path,
and expose no general filesystem-write capability.

## Boundaries

- Do not weaken the subprocess bwrap write wall.
- Do not grant a broad writable Doctrine MCP surface to workers; RSK-225 remains
  the governing security risk.
- Do not make observation capture depend on worker-branch integration.
- Reuse SL-231's observation validator/writer rather than implementing a parallel
  wire format.

## Verification direction

- A confined subprocess worker records an observation absent from its local
  worktree and present in the primary observation corpus.
- Replaying the same UUID returns the original record; conflicting intent is
  rejected.
- No path outside the observation corpus is writable through the broker.
- A discarded worker fork leaves its recorded observation available in the
  primary corpus.
