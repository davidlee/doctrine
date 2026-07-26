# DEC-015: Observation core has four fields; provenance is a typed facet

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

Every observation has exactly four fields in its stable reusable core:

- `uid`;
- `kind`;
- `schema_version`; and
- `recorded_at`.

Provenance is not a core field. Observation kinds may admit or require a typed
provenance facet when authorship, witnessing, ratification, model, role, harness,
or another producer distinction materially affects interpretation.

## Rationale

For most observations, provenance adds little beyond "some agent". Making a rich
producer structure universal would impose capture and schema cost without adding
meaning. Exceptional cases do matter, but their distinctions are domain-sensitive:
an execution observation may care about model and harness, while a human-attested
observation may distinguish author, witness, and ratifier.

Keeping those distinctions in typed facets preserves them where valuable without
forcing one impoverished provenance shape onto every observation kind.

## Consequences

- Observation kinds declare which facets are optional or required.
- Absence of a provenance facet is valid unless the kind says otherwise.
- The core does not carry an arbitrary metadata map.
- A friction-observation schema may still require execution context where SL-231's
  measurement goals depend on it; this decision does not make all provenance
  optional at the domain level.
