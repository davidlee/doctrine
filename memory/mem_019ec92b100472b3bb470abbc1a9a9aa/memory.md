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

## Spec-internal edges (CLI-writable)

Tech spec lineage and interaction edges are now CLI-writable via `spec`
subcommands — no hand-editing required:

- **`doctrine spec edit <TECH> --descends-from <PRD>`** — set the product a
  tech spec descends from. `--clear-descends-from` removes it.
- **`doctrine spec edit <TECH> --parent <TECH>`** — set a tech spec parent.
  `--clear-parent` removes it. Acyclicity is validated before any write.
- **`doctrine spec interactions add <TECH> <TECH> --type <text>`** — append
  an interaction edge. Target-idempotent (remove + add to re-type).
- **`doctrine spec interactions remove <TECH> <TECH>`** — remove all
  interaction edges to the target spec.

See [[concept.doctrine.entity-engine]] for the relation model,
[[fact.doctrine.cli-source-of-truth]] for the CLI authority,
and [[concept.doctrine.reading-entities]] for the read-via-show rule.
