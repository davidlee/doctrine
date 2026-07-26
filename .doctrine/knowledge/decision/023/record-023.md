# DEC-023: Explicit observation context overrides best-effort enrichment

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

Observation enrichment obeys this precedence and failure policy:

1. explicit caller-supplied facet values win;
2. automatic detection fills missing fields only;
3. populated facets retain their origin, such as `explicit`, `environment`,
   `mcp`, or `detected`;
4. invalid explicit structured values fail validation;
5. unavailable or malformed automatic context warns but does not block a valid
   observation; and
6. an explicit/detected conflict preserves the explicit value and warns.

Automatic enrichment never overrides caller intent or prevents a valid
summary-only friction capture.

## Rationale

Detection is best-effort and may be stale, unavailable, or unable to distinguish
several execution contexts. Rejecting capture because enrichment failed would make
the auxiliary context more authoritative than the raw signal. Conversely, silently
merging values without origin would make later analysis overconfident about
metadata quality.

Explicit precedence allows correction and embedding contexts to supply what the
local process cannot know. Origin metadata and conflict warnings preserve the basis
needed to judge those values later.

## Consequences

- No default value pretends that model, harness, role, or mode was measured.
- Partial automatically populated facets are valid where the facet schema permits
  them.
- Measurement facets retain sufficient provenance to distinguish measured,
  reported, and inferred values.
- Strictness applies to data a caller chose to supply; tolerance applies to
  enrichment the recorder attempted on the caller's behalf.
