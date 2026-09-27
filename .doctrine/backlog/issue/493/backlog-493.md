# ISS-493: Binary-emitted text cites repo-private ids

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The defect

ADR-024 bans repo-private ids from shipped text, and ISS-309 sweeps the shipped
corpus (`install/`, `memory/`). Text the **binary** emits is shipped too, and
ISS-309 does not cover it: clap help, refusal and error messages, and MCP tool
descriptions and schema strings. A client agent reads all of these.

RV-397 `F-10` (SL-268 PHASE-09) fixed the review surface's own help and MCP
descriptions. The same class remains elsewhere, for example:

- the shared list flag `--columns` help, on every kind's `list`: "(JSON rows
  are faithful/full — SL-037 D7)";
- the review fork refusal (`resolve_review_root`): "review verbs are not
  supported on a worktree fork (IMP-024)". A unit test in the review module pins it,
  so fixing it is an intentional test change.

## Proposed

Grep every `///` clap doc comment, `bail!`/`anyhow!` message, and MCP
`description` string for `\b[A-Z]{2,4}-[0-9]{3}\b` and doc-local ids (`D7`,
`PHASE-05`). Classify each hit per `reference/shipped-corpus-authoring.md`: a
format illustration (`RV-007`, `SL-024` as an example ref) stays, and a citation
is dropped or replaced with a verb or `reference/` pointer. A test that scans
the `tools/list` output and `--help` trees would keep it from regressing.
