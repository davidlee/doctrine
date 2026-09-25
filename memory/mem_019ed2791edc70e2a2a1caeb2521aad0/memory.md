# Doctrine knowledge records

Knowledge records capture **durable facts** that don't fit the other kinds —
they are not work items (backlog), not decisions (ADR), not rules
(policy/standard), and not runtime memory. They answer "what do we know?" for
long-lived project truths.

## Kinds

- **assumption** (`ASM-NNN`) — a premise taken as true, held until evidence
  overturns it. Status: `held | testing | validated | invalidated | obsolete`;
  `new` seeds `held`.
- **decision** (`DEC-NNN`) — a recorded choice whose rationale is worth
  preserving. Status: `proposed | accepted | rejected | superseded`; `new`
  seeds `proposed`.
- **question** (`QUE-NNN`) — an open question worth answering later. Status:
  `open | answered | obsolete`; `new` seeds `open`.
- **constraint** (`CON-NNN`) — an observable limit or boundary. Status:
  `active | waived | superseded | retired`; `new` seeds `active`.
- **evidence** (`EVD-NNN`) — a captured datum with provenance that supports
  or disputes other records. Status: `captured | disputed | confirmed |
  retracted | superseded`; `new` seeds `captured`. `confirmed` is deliberately
  non-terminal — it can be reopened or superseded by subsequent evidence.
- **hypothesis** (`HYP-NNN`) — a testable proposed answer to a question.
  Status: `proposed | confirmed | refuted`; `new` seeds `proposed`.
- **concept** (`CPT-NNN`) — a named domain concept worth a durable, citable
  definition. Status: `draft | active | retired`; `new` seeds `draft`.

Each kind carries a `[facet]` table (kind-specific structured data; empty for
`concept`) and an `[evidence]` array (links to backing sources).

## Evidentiary edges

Evidence (`EVD`) and hypothesis (`HYP`) records can be linked to other
knowledge records through the `supports` and `disputes` relation edges,
authored via `doctrine link EVD-001 supports DEC-002`. These edges trace
provenance — which evidence supports or disputes which epistemic claim.
EVD's `confirmed` status is deliberately non-terminal: it can be reopened
or superseded by subsequent evidence.

## CLI

The CLI is the source of truth: `doctrine knowledge --help`.
Key verbs: `new`, `list`, `show`, `inspect`, `edit`, `status`, `settle`,
`paths`. `settle` is the **resolving verb** — it moves a record to a resolving
state and captures the disposition that resolves it in one write.

## Membership test

A record belongs in knowledge, not the backlog, when it:
- Has durable existence — it persists beyond a work cycle.
- Describes something that *is*, not something to *do*.
- Is not a standing rule (policy/standard), not a decision's full ADR (use ADR
  for architecture-level choices), and not a transient observation (use memory
  for that).

See [[mem.concept.doctrine.storage-model]] for the authored tier where these land,
[[mem.signpost.doctrine.backlog]] for the work-intake counterpart, and
[[mem.signpost.doctrine.file-map]] for the `.doctrine/knowledge/nnn/` layout.
