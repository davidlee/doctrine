<!-- Shipped reference (ADR-005 PULL tier). Edit the source in
     `install/using-doctrine.md`; the installed copy at `.doctrine/using-doctrine.md`
     is inert. Names verbs and states discipline — it never reproduces
     `doctrine --help`; ask the CLI for exact flags. -->

# Using doctrine

How to *operate* doctrine: which verb for which intent, how to read and edit the
artifacts, and the rules that keep authored state coherent. For **vocabulary and
ids** see `glossary.md`; for the **workflow** (route → slice → design → plan →
execute → close) see the routing digest. For **exact command shapes and flags**,
ask `doctrine <command> --help` — this doc names verbs, never their flag tables.

## Which verb for which intent

Ad-hoc operations (the workflow doc owns the phase *sequence*; this is the
reach-for-it map):

| intent | verb |
|---|---|
| read an entity (all tiers, synthesized) | `doctrine <kind> show <ID>` |
| survey what exists | `doctrine <kind> list` |
| scope a change | `doctrine slice new` |
| transition a phase (e.g. flip `in_progress` → `completed`) | `doctrine slice phase` |
| record a durable fact | `doctrine memory record` |
| find / retrieve a memory | `doctrine memory find` · `doctrine memory retrieve` |
| regenerate the boot snapshot | `doctrine boot` |
| check a slice's phase rollup | `doctrine slice list` |

`<kind>` is `slice`, `spec`, `adr`, `memory`, … (see `glossary.md`). Ask
`doctrine <kind> --help` for the subcommands and flags each verb takes.

## Reading entities — always via `show`

Read an entity through `doctrine <kind> show <ID>`, never by opening one raw
file. An entity is stored across tiers: structured data in `*.toml`, prose in
`*.md`. `show` synthesizes both. A `*.md` body may be **empty by design** — its
substance living in the sibling `*.toml` — so judging an entity "hollow" from its
prose tier alone is a false reading. When in doubt, `show` it.

## Storage tiers — what goes where

Three tiers; know which one you are writing:

- **Authored** (`*.toml` + `*.md`, committed): structured/queried data in TOML,
  prose in MD. **Never put queried or derived data in prose** — it goes stale and
  lies. Lifecycle fields (e.g. a `status`) live in the TOML and are hand-edited
  there.
- **Runtime state** (under `.doctrine/state/`): disposable, gitignored progress —
  never commit it, never record progress in an authored file.
- **Derived**: regenerable indexes / caches — gitignored.

**Hand-edit vs verb.** Reach for a verb to create or transition an entity; hand-
edit the TOML for fields no verb yet owns (cite the CLI gap if so). Prose is always
hand-edited. Keep each datum on its correct side of the tier split.

## Edit-preserving rules

- **Ids are identity, and immutable.** Phase ids (`PHASE-01`) and criteria ids
  (`EN-1`/`EX-1`/`VT-1`/`VA-1`/`VH-1`) are never renumbered or reused — **edits
  append**. The slug is never authoritative; cite the prefixed id.
- **Cite the durable id**, never a mobile membership label (`FR-`/`NF-` move per
  spec — cite the `REQ-NNN` they label). Reference forms: `glossary.md`.
- Preserve surrounding structure when hand-editing — match the file's existing
  shape rather than reformatting it.

## Pointers

- `glossary.md` — kinds, ids, reference forms, verification taxonomy.
- `doctrine <command> --help` — the authoritative, self-documenting command shapes.
