# IMP-384: doctrine.toml rejects unknown keys

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`DoctrineToml` (`src/dtoml.rs`) parses tolerantly: every field is
`#[serde(default)]` and there is no `deny_unknown_fields` anywhere on the path,
so a misspelled table name parses clean and yields defaults. The module says so
in its own docs — "every other top-level key is ignored".

A typo in `doctrine.toml` is therefore silent. The cost is not uniform across
tables: a mistyped `[verification]` means the declared command does not run and
the author notices at once, but a mistyped table feeding a *report* produces a
plausible-looking answer with nothing attached to say the input was never read.
SL-243 hit that case at `inq-8` and guarded its own table (DEC-113); the general
defect is this item.

Timing argument, from the user: the project will never have fewer users than it
has now, so a breaking parse change is cheapest today.

## Why it is not a one-line attribute

`.doctrine/doctrine.toml` currently declares `[priority]` and `[reservation]`.
Neither is a field of `DoctrineToml` — both are projected out of band by their
own readers (`src/reserve.rs` holds the `[reservation]` shape). Adding
`deny_unknown_fields` to the central struct as it stands would reject doctrine's
own config file.

So the work is first a question about ownership: does `dtoml` become the single
owner of the whole file shape — which is what SL-057 D2's "one parser owns the
whole `doctrine.toml` shape" already claims, and which is presently only partly
true — or do the out-of-band tables stay out of band and get declared as
knowingly-ignored fields? The first is the coherent answer and the larger change.

## Scope when taken

- Resolve the ownership question above; `[priority]` and `[reservation]` are the
  two known out-of-band tables, and the search for others is part of the work.
- `deny_unknown_fields` on `DoctrineToml` and on each sub-config.
- Error text that names the unknown key and the file path. A nearest-match
  suggestion is the difference between a guard and a good guard, given the whole
  point is catching typos.
- A migration note: this turns previously-accepted config files into a hard
  parse error, including any table a project added speculatively.

## This has no spec to author against

SL-243's `/spec-coverage-assessment` census found `src/dtoml.rs` — 203 loc that
every project config read passes through — **anchored by no spec**. Each
`doctrine.toml` *table* is governed by its consuming spec (`[verification]` →
SPEC-002 REQ-254–257, `[dispatch]` → SPEC-021, `[estimation]` → SPEC-020), but
the shared reader's own behaviour — tolerance, defaulting, error posture — is
governed by nothing.

That per-table pattern is coherent and worth preserving. What is missing is
governance for exactly the surface this item changes. So scoping this work
includes deciding where the reader's behaviour is governed, not only changing it.

## Related

- DEC-113 — SL-243's local guard on its own adapter table, and the reasoning for
  why the central change did not ride that slice.
- SPEC-002 — owns the `[verification]` contract, the closest governed table.
