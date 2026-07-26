# DEC-031: V1 observations retain trustworthy raw usage measurements

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

V1 includes an optional registered `usage` facet for trustworthy raw resource
measurements. The facet records machine-reported counters together with:

- the measurement source;
- the scope or interval to which the counters apply; and
- the applicable typed units and counter categories.

Recorders must not estimate missing measurements. An absent value means unknown
or unavailable; zero is valid only when the source measured zero. Derived totals,
attribution claims, efficiency scores, comparisons, and benchmarks are not
stored as observation facts.

## Rationale

Usage data available during execution is generally not recoverable later.
Retaining exact source measurements now preserves the possibility of future
token-consumption and efficiency analysis without pulling that analysis into the
collection slice.

Source and scope are necessary to prevent unlike counters from appearing
comparable. Explicit absence semantics avoid turning missing telemetry into
false zeroes.

## Consequences

- Usage remains optional and never blocks friction capture.
- Automatic enrichment may attach the facet only when the harness or provider
  exposes a machine-readable measurement.
- A caller may relay such a measurement explicitly, but the facet must still
  identify its original measurement source and scope.
- The registered facet schema, rather than a generic metrics map, controls its
  accepted fields and units.
- Reporting consumers decide whether measurements are sufficiently compatible
  for aggregation or comparison.
