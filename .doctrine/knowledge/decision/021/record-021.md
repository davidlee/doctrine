# DEC-021: Observation records are self-contained TOML documents

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

Each observation and observation-control record is one self-contained TOML
document. It has no Markdown companion. The friction summary and optional detail
are typed payload strings in that document.

Observation documents are created and corrected through the observation interface,
not by a hand-editing workflow.

## Rationale

The observation is a machine-readable event record rather than an authored entity
with separate structured and prose tiers. A TOML+Markdown pair would double the
filesystem objects for every capture, require consistency across two files, and
create orphan-half failure modes. It would also weaken the invariant that one
capture creates one independent file.

Self-contained TOML keeps creation atomic at the record boundary and makes scanning,
validation, transfer, and later archival straightforward.

## Consequences

- Long detail uses TOML multiline-string encoding.
- The writer owns serialization; users are not expected to maintain the wire format
  manually.
- Append-only supersession and retraction remain the ordinary correction mechanism
  from DEC-018.
- Human-readable `show` output is a projection of the TOML document, not a stored
  prose tier.
