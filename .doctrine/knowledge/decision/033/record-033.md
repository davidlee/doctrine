# DEC-033: V1 partitions observations but does not automate retention

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

V1 relies on the accepted kind/year/month partitioning and one-file-per-record
layout to bound the size and contention of each storage unit. It does not
automatically archive, expire, export, or prune observations.

The repository-wide corpus therefore retains observations indefinitely and may
grow without bound. This is an explicit v1 limitation, not an implied retention
guarantee.

## Rationale

Partitioning removes the immediate single-file failure modes: append contention,
merge conflicts, increasingly expensive edits, and manual section archival.
Actual bounded retention would require decisions about evidentiary value,
retention periods, an archive destination, export verification, destructive
pruning, and whether searches span cold storage.

Those decisions need observed corpus volume and use patterns. An in-repository
archive path would improve navigation but would not materially reduce Git
storage.

## Consequences

- List and search cover all retained partitions by default.
- No record silently ages out of the corpus.
- Operators may perform deliberate manual archival or hard deletion, but those
  are outside the v1 interface and must not be presented as automated policy.
- Corpus size and query latency should be measured so a later slice can justify
  and design retention or external archival.
