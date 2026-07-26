# DEC-038: Observation records use a typed self-contained TOML envelope

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

Each observation is one self-contained TOML document with:

1. exactly four stable core fields: `uid`, `kind`, `schema_version`, and
   `recorded_at`;
2. a kind-owned typed `payload` table;
3. zero or more independently versioned registered tables under `facets`; and
4. field-level `origins` entries for persisted facet values, constrained to
   fields actually present in their registered facet.

Facet value origin is `explicit` or `automatic`. Explicit values overlay
automatic candidates at field granularity; an explicit list replaces the
automatically inferred list for that field rather than merging it.

Omission means unknown or unavailable. It must not be normalised to an empty
string, empty list, `false`, or zero.

The storage path is:

`.doctrine/observations/<kind>/<year>/<month>/<uuid>.toml`

There is no Markdown companion or generic extension table.

## Rationale

The envelope keeps durable identity and routing stable while allowing kind and
facet schemas to evolve independently. Field-level origins preserve mixed
explicit/automatic enrichment truthfully; facet-level provenance would be
ambiguous whenever only one automatically inferred field was overridden.

Replacement rather than list merging keeps precedence deterministic and avoids
inventing item-level provenance rules.

## Consequences

- Serialization is deterministic and validation is registry-driven.
- A record is complete and movable without consulting a companion file.
- Readers can expose the stable core and raw unsupported content even when a
  newer payload or facet schema is not semantically understood.
- The file path is derived from validated `kind`, UTC `recorded_at`, and `uid`;
  readers validate that path and content agree.
- Origin tables cannot carry arbitrary metadata because their facet and field
  keys must resolve through the registry.
