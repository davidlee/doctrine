# Doctrine reconciliation records signpost

Reconciliation records (REC kind) are the atomic audit trail of a single
status change on a governed entity. Each REC captures what changed
(requirement, specification, policy), who authorised it, and why.

## When to create a REC

A REC is created whenever a status transition is applied to an entity under
governance — typically during audit reconciliation (`/audit` closeout) or
revision application. The REC is the unit of evidence: it proves the
transition was intentional and reviewed.

## CLI

The CLI is the source of truth: `doctrine rec --help`, never guess.
Key verbs: `new`, `list`, `show <ID>`.

## Where they live

RECs live under `.doctrine/rec/nnn/`. Each is a `rec-nnn.toml` +
`rec-nnn.md` pair (TOML holds the structured status change; MD holds
the operator note and rationale).

See [[signpost.doctrine.requirements]] for reconciliation targets,
and [[signpost.doctrine.file-map]] for the directory layout.
