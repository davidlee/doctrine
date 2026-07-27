# DEC-059: Inquiry map uses a revisioned runtime TOML snapshot

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

The SL-233 inquiry map is authoritative as a mutable TOML snapshot in
gitignored `.doctrine/state/**` runtime storage. It is schema-versioned for wire
evolution and carries a monotonic run revision used for compare-and-swap
mutation; “versioned” does not mean committed to Git.

Each accepted mutation atomically rewrites the snapshot and exposes a compact
material-change summary. Stable node identities and revision checks provide
inspectability and stale-writer refusal without making an append-only event log
or Markdown parsing convention canonical.

Accepted semantic outcomes remain durable through DEC/QUE/ASM records and the
eventual `design.md`; loss of the runtime snapshot has the recovery consequence
defined by DEC-057.
