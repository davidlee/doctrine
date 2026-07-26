# DEC-016: Friction capture requires only a summary

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

The only friction-specific input required to record a friction observation is a
non-empty summary. Detail is optional. Typed facets are populated automatically
where context is available and remain optional where absence does not make the raw
signal meaningless.

Capture is cheap, permissive, and error-tolerant where preserving a partial
observation is more useful than rejecting it.

## Rationale

Requiring the reporter to classify impact, search for duplicates, identify a cause,
estimate wasted effort, or assemble execution metadata would recreate the friction
the mechanism is intended to measure. Those interpretations also bias later
frequency and correlation analysis.

Automatic context improves analytical value without increasing reporting effort.
Failure to collect optional context must not discard an otherwise valid occurrence.

## Consequences

- A summary-only invocation is valid.
- Detail, execution context, references, and measurements may enrich the record.
- Automatically captured fields retain honest absence when unavailable; the
  recorder does not invent placeholders or estimates.
- Validation remains strict for supplied structured values even though their
  absence may be tolerated.
- Category, severity, suspected cause, duplicate identity, and estimated waste are
  not mandatory capture fields.
