# Doctrine knowledge records

Knowledge records capture **durable facts** that don't fit the other kinds —
they are not work items (backlog), not decisions (ADR), not rules
(policy/standard), and not runtime memory. They answer "what do we know?" for
long-lived project truths.

## Kinds

- **assumption** (`ASM-NNN`) — a premise taken as true, held until evidence
  overturns it. Status: `pending | proven | disproven | withdrawn`.
- **decision** (`DEC-NNN`) — a recorded choice whose rationale is worth
  preserving. Status: `pending | active | superseded | withdrawn`.
- **question** (`QUE-NNN`) — an open question worth answering later. Status:
  `open | answered | settled | withdrawn`.
- **constraint** (`CON-NNN`) — an observable limit or boundary. Status:
  `active | relaxed | removed | withdrawn`.

Each kind carries a `[facet]` table (kind-specific structured data) and an
`[evidence]` array (links to backing sources).

## CLI

- `doctrine knowledge new --kind <assumption|decision|question|constraint>`
  — scaffold a new record.
- `doctrine knowledge list` — survey records; hides settled states by default.
- `doctrine knowledge show <ID>` — reassemble one record (kind auto-detected).
- `doctrine knowledge status <ID> <state>` — transition its lifecycle state
  (the state must be in the kind's vocabulary).

## Membership test

A record belongs in knowledge, not the backlog, when it:
- Has durable existence — it persists beyond a work cycle.
- Describes something that *is*, not something to *do*.
- Is not a standing rule (policy/standard), not a decision's full ADR (use ADR
  for architecture-level choices), and not a transient observation (use memory
  for that).

See [[concept.doctrine.storage-model]] for the authored tier where these land,
[[signpost.doctrine.backlog]] for the work-intake counterpart, and
[[signpost.doctrine.file-map]] for the `.doctrine/knowledge/nnn/` layout.
