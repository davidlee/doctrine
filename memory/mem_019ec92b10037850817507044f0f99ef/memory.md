# Doctrine reference docs

Doctrine ships two reference documents to every installed project under
`.doctrine/`. They are the durable prose authorities for *how* to operate
doctrine and *what* its vocabulary means — separate from the shipped memory
corpus (which orients via `find`/`retrieve`) and the CLI (which is the source
of truth for command shapes).

- **`using-doctrine.md`** — How to *operate* doctrine: which verb for which
  intent, how to read and edit artifacts, and the rules that keep authored
  state coherent. Names verbs and states discipline; never reproduces
  `doctrine --help` flag tables.
- **`glossary.md`** — Vocabulary and ids: every entity kind, its abbreviation,
  reference forms, and directory layout. Cite entities by their padded id
  (e.g. `<SLICE-ID>`, `<ADR-ID>`, `<REQ-ID>`), never by slug alone.

These are shipped reference docs (ADR-005 PULL tier) — they install once and
stay inert unless the installer is re-run. The boot snapshot and shipped
memories are separate push surfaces; these docs are the pull surface for
deliberate lookup.

See [[signpost.doctrine.install]] for the installation path,
[[concept.doctrine.reading-entities]] for why to read via `show` not raw
files, and [[fact.doctrine.cli-source-of-truth]] for the CLI authority.
