# Doctrine reference docs

Doctrine ships two reference documents to every project **published**, not
projected: read them on demand with `doctrine library show reference/<name>.md`
(ADR-019 — the eager projection base is deliberately minimal, so no copy lands
on disk). They are the durable prose authorities for *how* to operate
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

The two above are the *general* authorities. Doctrine also ships **domain
reference docs** for specific subsystems, same PULL tier and format:

- **`dispatch-mechanics.md`** — the `/dispatch` fork→verify→import→land funnel:
  the mental model plus the git-plumbing invariants and failure modes. Pairs
  with the `mem.signpost.doctrine.dispatch` retrieval index for the sharp
  mid-operation traps.

These are shipped reference docs (the pull tier) — reachable on demand via
their published address, never eagerly copied into the project. The boot
snapshot and shipped memories are separate push surfaces; these docs are the
pull surface for deliberate lookup.

See [[mem.signpost.doctrine.install]] for the installation path,
[[mem.concept.doctrine.reading-entities]] for why to read via `show` not raw
files, and [[mem.fact.doctrine.cli-source-of-truth]] for the CLI authority.
