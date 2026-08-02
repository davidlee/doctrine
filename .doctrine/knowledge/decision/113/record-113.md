# DEC-113: Report states adapter provenance; the adapter table denies unknown fields

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

Two parts, one against each half of the failure.

**The report states its own provenance.** Every unit figure in
`doctrine spec anchors` is accompanied by which languages had an adapter
declared, which of those ran, and what each returned. "no adapter declared" and
"an adapter ran and returned nothing" are different payloads, and the markdown
rendering leads with that statement rather than leaving a reader to infer it
from a zero. The provenance sits inside the report struct, not in a wrapper
around it — consistent with DEC-112, which declined an envelope.

**The adapter table carries `deny_unknown_fields`.** A misspelled key *inside* a
declared adapter table is a parse error naming the key, rather than a silent
fall to a default.

## What this does not fix, and why that is accepted

`DoctrineToml` unions six sub-configs, each `#[serde(default)]`, with no
`deny_unknown_fields` on the path. A misspelled *table name* is unknown at that
outer level, not inside the adapter struct, so this decision does not catch it.
The guard is partial and is labelled as partial.

The reason it is not made total here is that the total version is not an
attribute. `.doctrine/doctrine.toml` declares `[priority]` and `[reservation]`,
and neither is a field of `DoctrineToml` — both are projected out of band by
their own readers. `deny_unknown_fields` on the central struct as it stands
would reject doctrine's own config file. Deciding whether `dtoml` becomes the
sole owner of the whole file shape is real work with a migration cost for every
project, and it is not this slice's work. It is **IMP-384**, wanted soon rather
than someday: a breaking parse change is cheapest while the user count is
lowest.

The provenance statement is what makes the residual tolerable. A misspelled
table name still yields "0 adapters declared" — but that is a claim a reader can
check against their own `doctrine.toml` in seconds, where "29,310 loc dark" is
not.

## Why not the alternatives

- **Provenance alone**, without the table-level guard, leaves the inner typo
  silent when the guard for it costs one attribute on a struct this slice is
  authoring anyway.
- **An error exit when no adapter is declared** contradicts the scope's
  commitment that an absent adapter is an owned no-op. The spec-side half of the
  report — which specs anchor what, which anchors do not resolve — needs no
  adapter at all, and a project without one should still get it.

## Provenance

Settled at the `inq-8` fork of design run `dr-019fc13a` (SL-243). The config
tolerance was verified in the slice's research round; the `[priority]` /
`[reservation]` obstacle was verified against `src/dtoml.rs` and
`.doctrine/doctrine.toml` while settling this fork, and is what moved the
general fix to its own item rather than into this slice.

## Related

- [[DEC-112]] — the JSON shape this provenance block sits inside.
