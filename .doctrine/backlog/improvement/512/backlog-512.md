# IMP-512: memory retrieve table output and --path wording invite misuse

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

Two retrieve ergonomics gaps, split from ISS-507 (fix sketch item 3):

- `memory retrieve --format table` prints full framed memory bodies, not a
  compact candidate list. Evidence: friction observation `01a0b9da`.
- `-p/--path` is the project root, not a scope probe (that is `--path-scope`).
  The retrieve-memory skill's "scoped to files" wording invites passing a file
  to `--path`.

## Fix sketch

1. Give retrieve's table format a compact row per hit (or document that
   retrieve's output is framed blocks by design and drop `table` as a choice).
2. Reword the retrieve-memory skill to name `--path-scope` for file scoping.
