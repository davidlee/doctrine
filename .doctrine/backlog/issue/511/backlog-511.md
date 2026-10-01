# ISS-511: Creating and disposing a node in one design apply refuses with unknown node

## What

A single `design apply` submission that creates a node and then disposes (or otherwise
references) it is refused with `unknown node: <id>` — naming a node the same batch
creates. The message reads as a typo, not an ordering/visibility rule.

Possibly the same root as ISS-360 (batch declare refuses an in-batch parent chain):
in-batch references resolve against pre-batch state.

## Fix sketch

Resolve references against the batch's own creations, or refuse with a message that
says "created in this submission; dispose in a later one".

## Evidence

`01a0b972` (2026-09-19, multi-node declare batch), `01a0e26f` (2026-09-27, create +
checkpoint disposal).
