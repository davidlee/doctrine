# IDE-018: General user-definable metadata facility across entity kinds

## Summary

Add a general, user-definable metadata surface — key/value pairs attachable to
any entity — as an alternative to the current pattern of adding one-off typed
slots to the entity schema for every new facet (estimate, priority, constraint,
dates, etc.).

## Use cases

- **External references**: URLs to GitHub issues, PRs, wikis, Jira tickets,
  Notion pages, design docs, RFC discussions — any durable external link the
  user wants to attach without forking the schema.
- **File links**: paths into the repo (or external repos) — e.g. linking a
  slice to the source files it touches, or an ADR to the config it governs.
- **Proposed / scheduling dates**: target-dates, soft deadlines, review windows.
- **Milestones**: grouping items into user-defined milestones without needing a
  first-class milestone entity.
- **GitHub issues**: cross-references to the GitHub issue tracker.
- **Custom workflow fields**: things the user cares about that doctrine doesn't
  model yet — team, reviewer, effort, etc.

## Design considerations

### Types

Metadata values could carry a type (string, date, URL, integer, reference-to-entity,
etc.) so tooling can validate, render, and query appropriately.

### Kind-appropriateness

Certain metadata keys could be declared appropriate (or inappropriate) for certain
entity kinds — e.g., `github_issue` makes sense on a backlog item or slice but
less so on an ADR. Possibly a per-kind allowlist or a free-form convention.

### Required / optional

A metadata key could be declared required for certain kinds — e.g., every slice
*must* carry a `proposed_date` — or required only under certain conditions (status
transitions, etc.).

### Relationship to existing typed facets

Existing typed facets (estimate, priority, constraint, created/updated dates) could
be re-expressed as built-in metadata keys with types, or could coexist as the
authoritative structured layer while metadata serves the unmodeled remainder.

### Scope

- Per-entity key/value store (TOML `[metadata]` table or `[[metadata]]` array-of-tables)
- CLI verbs: `doctrine meta set <ID> <key> <value> [--type …]`, `doctrine meta show <ID>`, `doctrine meta list --key …`
- Query/survey: filter/sort backlog list by metadata values
- Export: include metadata in JSON/TOML export

## Sequencing

These specific facets should ship first — a general metadata facility is best
informed by the concrete patterns that emerge from them. IDE-018 is sequenced
after all of them (`after` edges):

- IMP-108: authored created/updated dates on the spec schema
- IMP-112: wire estimate display
- IMP-118: item-level authored-priority slot
- IDE-006: constraint facet — owner + immutability/enforceability axis
- IDE-013: estimate/value change history (time-series of facet edits)
- IMP-142: expose `related` link label for backlog kinds (the `drift` workaround
  used to record this idea's relationships is the motivation)
