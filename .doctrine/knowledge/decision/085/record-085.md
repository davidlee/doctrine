# DEC-085: Existing-design import admits tiered question sources

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

An explicit existing-design import uses tiered sources to seed its inquiry
frontier:

- Every non-terminal `QUE` with a direct `shapes SL-NNN` edge becomes a durable
  open inquiry anchored to that canonical record.
- A conventional doc-local `OQ-*` entry is recognised only within the authored
  design's explicit Open Questions section. It becomes an unverified
  `imported-prose` inquiry node carrying its source label, location, and content
  fingerprint.

Imported prose nodes are not knowledge records, accepted truth, stage
clearance, or review evidence. Doctrine assigns them fresh run-local `inq-*`
identity.

When an OQ explicitly cites a canonical QUE ID, one inquiry node carries both
the durable record and prose source. Otherwise Doctrine never deduplicates by
text similarity. It may flag possible overlap for inspection, but the user or
agent must reconcile it explicitly.

This policy gives durable graph state its proper authority while retaining a
narrow migration bridge for questions that existed only in conventionally
structured prose.
