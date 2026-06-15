# Doctrine storage model and the storage rule

Doctrine splits every byte it writes into three tiers. Know which one you are in
before you write — putting data in the wrong tier is the most common way to
corrupt an artifact.

- **Authored** — committed, diffable, reviewed: the `*.toml` + `*.md` pairs under
  `.doctrine/slice/nnn/` and `.doctrine/adr/nnn/`. This is the contract.
- **Runtime state** — gitignored, disposable, `rm -rf`-able: `.doctrine/state/`,
  the `phases` symlink, `handover.md`, `boot.md`. This is progress.
- **Derived** — regenerable indexes / caches; gitignored.

**The storage rule:** structured, queried, or derived data goes in TOML; prose
goes in MD; **never** put queried or derived data in a prose body. A slice's
relations, lifecycle status, and plan criteria live in `slice-nnn.toml` /
`plan.toml`; rationale and narrative live in `slice-nnn.md` / `plan.md`. A
hand-edited markdown body must not carry data a command would otherwise produce
(e.g. don't paste `slice list` output into `plan.md`).

**Contract vs progress:** authored files state intent and never track progress.
Progress — which phase is `in_progress`, what is blocked — lives only in the
runtime state tree under `.doctrine/state/`. If you are editing an authored file
to record status, you are in the wrong tier.

Point of truth: the `## storage model (the storage rule)` section of the repo's
`CLAUDE.md`, and `doc/entity-model.md` (`## The storage rule`), which generalises
it across every entity type. See [[fact.doctrine.storage-tiers]] for the tier
cheat-sheet, [[concept.doctrine.entity-engine]] for the engine that enforces the
TOML/MD split, and [[signpost.doctrine.file-map]] to locate the directories.
For the reading consequence of this rule, see
[[concept.doctrine.reading-entities]].
