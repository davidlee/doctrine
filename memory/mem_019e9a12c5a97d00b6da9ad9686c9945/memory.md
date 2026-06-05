# Doctrine storage tiers: authored vs runtime vs derived

Every file doctrine touches sits in one of three tiers — know which one you're
writing before you write it. The boundary is defined in the project `CLAUDE.md`
storage model; this points at it.

- **authored** — committed, diffable, reviewed. Structured data in TOML, prose
  in MD, under `.doctrine/slice/nnn/` and `.doctrine/adr/nnn/`. Never put
  queried or derived data in prose (the storage rule). This is the only tier
  you hand-edit and commit.
- **runtime state** — gitignored, disposable, `rm -rf`-able. Lives under
  `.doctrine/state/` (the `phases` symlink, `handover.md`, `boot.md`). Progress
  and per-run tracking go here, never into authored files.
- **derived** — regenerable indexes and caches; gitignored. If a tool can
  rebuild it, it belongs here.

The split is also an ownership boundary: managed trees (runtime/derived, plus
tool-shipped content) are doctrine's to regenerate and yours to leave alone;
authored trees are yours to write. Putting progress in an authored file or
hand-editing a managed one breaks that contract.

Related: [[concept.doctrine.storage-model]] for the model and the storage rule,
[[signpost.doctrine.file-map]] for where each tier lives on disk, and
[[fact.doctrine.cli-source-of-truth]] for asking the CLI before you guess.
