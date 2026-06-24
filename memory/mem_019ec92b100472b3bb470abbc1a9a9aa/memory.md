# Relating doctrine entities

Doctrine entities (slices, ADRs, specs, requirements) can be connected via
typed relations. The CLI is the validated seam — hand-editing relation rows
skips the legality check and risks malformed edges.

## CLI

The CLI is the source of truth: `doctrine link --help`,
`doctrine needs --help`, `doctrine after --help`,
`doctrine supersede --help`, `doctrine inspect --help`.

Key verbs and their roles:

- **link / unlink** — author or remove a tier-1 edge. The label must be
  writable for the source kind, and the target must resolve to a legal entity
  kind. Idempotent.
- **needs** — append a hard prerequisite (blocking dependency). Source must be
  a dep/seq-authoring kind (slice or backlog); target must be work-like.
  Idempotent.
- **after** — append a soft-sequence edge (ordering hint, not a hard block).
  Same target gate as `needs`. Idempotent.
- **supersede** — record that NEW supersedes OLD (ADR kind only). Flips OLD
  status to `superseded`. Idempotent.
- **inspect** — read relations from both directions (authored outbound +
  derived inbound) for a single entity. Use it to understand what an entity
  connects to, not to judge from a raw file.

## What still requires hand-editing

Not all relation classes are CLI-writable yet. Some edges (e.g. slice-to-ADR
references, cross-corpus product-to-product links) are prose-only — author
them in the source entity's `.toml` `[[relation]]` table, but prefer the CLI
where a verb exists.

See [[concept.doctrine.entity-engine]] for the relation model,
[[fact.doctrine.cli-source-of-truth]] for the CLI authority,
and [[concept.doctrine.reading-entities]] for the read-via-show rule.
