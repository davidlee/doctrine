# DEC-034: Observation readers tolerate unsupported schemas with diagnostics

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

Observation readers are forward-tolerant:

- they decode and expose the stable core of a structurally valid observation
  even when its kind, facet, or schema version is unsupported;
- exact `show` preserves and renders unsupported payload or facet content
  generically rather than discarding it;
- list, search, and validation surface explicit compatibility diagnostics; and
- unsupported content is not eligible for semantic filtering or resolved
  interpretation that requires a schema the reader does not understand.

Writers remain strict. An explicitly requested unsupported kind, facet, field, or
schema version is rejected. Best-effort automatic enrichment follows the
existing error-tolerant rule: unsupported enrichment is omitted with a warning.

Readers must never silently ignore unsupported content or imply that a partial
projection is complete.

## Rationale

A reusable, append-only corpus will outlive individual CLI and MCP versions.
Failing the entire collection because one producer is newer makes compatibility
needlessly brittle; silently dropping fields makes queries and displays
misleading.

The stable core provides a safe minimum projection while diagnostics preserve
the distinction between corrupt data and valid data whose semantics are newer
than the reader.

## Consequences

- Record parsing separates structural validity from semantic support.
- Compatibility diagnostics identify the record UUID and unsupported
  kind/facet/version.
- Corpus-wide validation may report unsupported schemas without classifying
  otherwise well-formed records as corrupt.
- Queries over supported fields remain deterministic but disclose when
  unsupported records could not be evaluated for a requested semantic filter.
- A newer writer cannot use tolerance as permission to emit arbitrary,
  unregistered metadata.
