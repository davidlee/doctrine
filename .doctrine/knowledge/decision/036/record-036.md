# DEC-036: V1 ships five coherent observation facets

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

V1 registers five independently typed and versioned facets:

- `provenance`: exceptional recorder attribution and human witness or
  ratification;
- `execution`: harness, model, role, mode, skills, and available
  run/session/turn identifiers;
- `work_context`: repository and revision context, workflow stage, canonical
  entity references, and safe repository-relative locations;
- `correlation`: typed relationships to observation UUIDs or declared execution
  scopes; and
- `usage`: sourced and scoped resource counters.

All facets are optional unless a kind schema requires one. `friction` still
requires only a non-empty summary. A `measurement` requires at least one
registered measurement facet; in v1, `usage` is that facet.

Each facet has a closed field schema, validation rules, and its own version. The
set does not imply a general-purpose context or metadata map.

## Rationale

These facets separate five meanings that evolve differently: who exceptionally
attested to a record, how execution occurred, where work sat, what another
record measures or relates to, and which resources were measured. Collapsing
them would create an attractive nuisance for arbitrary metadata and make
filtering unreliable.

Keeping them optional preserves cheap capture. Automatic enrichment can add
structured context when available; absence remains truthful when it is not.

## Consequences

- Kind schemas declare allowed and required facets without redefining them.
- CLI and MCP inputs expose typed facet objects and reject unknown explicit
  fields.
- Query filters address registered facet fields by their stable facet and field
  names.
- New fields or facets require deliberate schema evolution rather than
  opportunistic key insertion.
- Sensitive or unstable host context is not automatically swept into
  `work_context`; enrichment is field-by-field.
