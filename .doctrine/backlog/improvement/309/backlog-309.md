# IMP-309: doctor check: published entries resolve against corpus

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

**Gap.** Publication admission (SL-223, `publication.rs`) is deliberately
**source-agnostic** — it validates the *declaration* (fields, licence, unique
address, well-formed logical address), not that each entry's `backing` actually
resolves to bytes. So the manifest can declare a logical address whose backing
was dropped (a hollow embed from the crane `cleanCargoSource` strip, or a stale
manifest after a source-root move) with **no signal until `library show` fails
at runtime**.

**Proposed check.** A `doctrine doctor` leg that, for every published entry,
asserts `Resolver::available(entry)` (i.e. `SourceAdapter::exists(backing)`) —
flagging any declared-but-unresolvable entry as drift. Catches the RustEmbed
re-embed footgun and crane embed-strip class at health-check time rather than at
first user `show`.

**Seam already landing in SL-227.** The `SourceAdapter::exists` probe and
`Resolver::available(entry)` (design §1, D-2) are built for the `library
list|tree` "unavailable but visible" mark — this check is a cheap second
consumer of the same probe. Deferred out of SL-227 scope per user ("at some
point"); captured so the seam is not rediscovered.

Related: [[QUE-172]]-adjacent publication surface; governed by SPEC-026. Check
whether `publication validate` already emits every entry (which would surface a
missing backing via `BackingSourceMissing`) before building a parallel doctor
leg — reuse, don't duplicate.
