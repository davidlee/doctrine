# DEC-044: Address observations by UUID-derived authoritative paths

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

An observation's authoritative path is a pure function of its UUID, independent
of kind and recorded time:

```text
.doctrine/observations/records/<random-tail-shard>/<uuid>.toml
```

The shard is derived from stable hexadecimal digits in UUIDv7's random tail,
not its time-ordered prefix. Kind and `recorded_at` remain validated envelope
fields but do not participate in identity lookup. Atomic create-new at the
computed path therefore arbitrates global UUID uniqueness and replay without a
shared registry or corpus scan.

A chronological filesystem view may later be generated as disposable,
gitignored relative symlinks under `by-month/<year>/<month>/`. That view is
derived navigation only: capture and queries neither create nor trust it, and
missing or malformed links cannot affect observation validity.
