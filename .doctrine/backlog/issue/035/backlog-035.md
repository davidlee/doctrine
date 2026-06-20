# ISS-035: IMP-122 carries untyped related relation row, fails corpus migration test

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

`just check` is red on `main`:

```
backlog_corpus_keeps_dep_seq_typed_migrates_cross_kind_axes ... FAILED
.doctrine/backlog/improvement/122/backlog-122.toml: unexpected backlog
[[relation]] label `related` (dep/seq axes needs/after/triggers must stay typed)
```

`tests/e2e_relation_migration_storage.rs:322`.

## Cause

`backlog-122.toml` carries a free-form relation row:

```toml
[[relation]]
label = "related"
target = "SL-121"
```

The corpus-migration invariant forbids a backlog `[[relation]]` with the untyped
`related` label — backlog→slice membership belongs on the typed `slices` axis (cf.
ISS-034, which renders `slices: SL-121`), and the dep/seq axes
(`needs`/`after`/`triggers`) must stay typed. The `related` row is data drift,
likely a hand-edit or a pre-`SL-048`-cut residue.

## Fix (candidate)

Move the edge to the membership axis — express IMP-122↔SL-121 via the `slices`
relationship rather than a `related` `[[relation]]` row — or drop it if redundant.
Confirm `just check` green after.

## Provenance

Pre-existing on `main` (present at `f7a69b2e`, before SL-123 integrated at
`cd367759`). Unrelated to SL-123 (code-only: `worktree.rs` / `main.rs` /
dispatch-agent skill / verify-worker tests). Captured during SL-123 `/close` so the
red gate is not silently normalised; SL-123 closed on its own green scope.
