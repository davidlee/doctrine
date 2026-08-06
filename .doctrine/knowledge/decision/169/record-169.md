# DEC-169: Wire-key tables are pinned by a serde key-set test

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Why "derived" was not on offer

The inquiry node asked whether the table is derived from the wire types or
authored as literals. Rust has no reflection over struct fields, so deriving it
would mean a proc macro written for one table — new machinery, and machinery
that has to be understood by everyone who later touches `Declaration`.

## The property that actually matters

Authoring is therefore forced, and the real risk is not that the table is
hand-written but that it goes stale. A `Declaration` field added later that
never joins the table would be honoured for no subject kind, or refused for
none — silently, which is ISS-318's own defect class reappearing inside the fix
for it.

## The pin

Serde already carries the fact needed. Every field on `Declaration` is
`skip_serializing_if` — `Sparse::is_omitted` for the sparse ones,
`Option::is_none` for the rest — so a fully-populated value serialises to
exactly the wire keys and nothing else. `resolved_record` is `#[serde(skip)]`,
so it stays out, which is correct: it is not a wire key and the shell supplies
it between DEC-086 steps 4 and 6.

Comparing that key set against the table's turns a new field into a test
failure. Both sides derive from the type's own serde form, so no key is
re-typed as a literal (STD-001).

## Scope note

This table is `Declaration`'s keys against the design-run **subject** kinds —
`inq-`, `sec-`, `att-`, `fnd-`, `cp-`. It is not facet keys against record
kinds, and nothing in it depends on knowledge governance. That is the fact
that lets objective 3 ship in phase 1 under [[DEC-165]].
