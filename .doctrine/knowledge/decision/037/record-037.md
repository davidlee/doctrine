# DEC-037: Automatic observation enrichment is allowlisted and safe by construction

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

Automatic observation enrichment is field-by-field and allowlisted. V1 may
persist known safe values such as:

- harness, model, role, mode, and skill identifiers supplied by the execution
  integration;
- canonical Doctrine entity references;
- repository identity and revision; and
- repository-relative work locations.

Automatic enrichment must not sweep or persist environment variables, prompts,
tool arguments or output, command arguments, arbitrary process state, or
absolute paths beneath a user's home directory.

Explicitly supplied summary, detail, and facet values remain the caller's
responsibility. Existing precedence applies: explicit values override automatic
values, and automatic enrichment failures warn without blocking capture.

## Rationale

Broad context collection would make cheap capture a likely secret-ingestion and
noise path. Manual hard redaction is deliberately available as an exceptional
operational remedy, but Git-backed storage makes it too costly to serve as the
primary safety control.

An allowlist retains the execution metadata needed for mode/model analysis while
making the automatic path safe by construction and predictable to callers.

## Consequences

- Each enrichment adapter declares the exact fields it may source.
- Adding a new automatic field requires a deliberate schema and safety review.
- Values not on the allowlist may still be supplied explicitly when their facet
  schema permits them.
- Tests verify both positive enrichment and negative non-capture boundaries.
- Enrichment must not infer a sensitive value indirectly from disallowed raw
  inputs merely to bypass the allowlist.
