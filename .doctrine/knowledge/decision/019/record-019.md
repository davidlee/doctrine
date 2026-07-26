# DEC-019: Observation storage is one file partitioned by kind and date

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

Each observation or observation-control record occupies its own immutable file.
The authored corpus is partitioned first by observation kind and then by the year
and month of its core `recorded_at` value:

```text
.doctrine/observations/
  friction/
    YYYY/
      MM/
        <uuid>.toml
  control/
    YYYY/
      MM/
        <uuid>.toml
```

The UUID is the identity. The path is a deterministic storage location derived
from `kind`, `recorded_at`, and the UUID.

## Rationale

One file per capture makes independent observations clean add/add merges; no
shared counter, manifest, or append target is mutated during recording. Kind/date
partitioning prevents an unbounded flat directory while supporting natural
time-window scans, human inspection, and later archival of cold partitions.

UUID-prefix partitioning would distribute entries without providing those temporal
or operational seams. Session bundles would recreate append contention.

## Consequences

- Recording creates one new file and no shared authored index.
- Readers discover records by scanning partitions; any acceleration index is
  derived and disposable.
- SL-231 defines no retention or archival policy, but its layout permits a later
  capability to archive complete cold partitions without rewriting live records.
- Control records follow the same identity and partitioning rules as observations.
