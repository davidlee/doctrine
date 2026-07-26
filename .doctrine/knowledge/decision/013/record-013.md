# DEC-013: Observations are cheap occurrence-level raw signals

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

Each observation captures one occurrence as a raw signal. Recording does not
require searching for duplicates, choosing an existing issue, identifying a root
cause, or consolidating related occurrences.

The capture path is deliberately cheap. Several observations of apparently similar
friction remain several observations.

## Rationale

Pre-capture deduplication and classification spend effort at the moment the system
is trying to measure, discourage recording, and replace frequency data with the
reporter's premature interpretation. Independent occurrence records preserve the
signals needed for later analysis to discover frequency, correlations, clusters,
and candidate causes.

This semantics also fits UUID-native identity from DEC-012: observations can be
created independently without coordination or a significance judgement.

## Downstream boundary

Consolidation is a later analytical or authoring act, not an observation mutation.
When a pattern becomes significant, it may produce or support an appropriate
authored object such as:

- an EVD knowledge record;
- a backlog item;
- another knowledge-record kind; or
- a future purpose-built numbered entity.

SL-231 does not define clustering, aggregation, promotion, or the authored object
that receives an identified pattern.
