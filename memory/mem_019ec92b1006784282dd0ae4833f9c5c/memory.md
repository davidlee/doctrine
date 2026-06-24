# Recording doctrine memories

Doctrine memory lets you capture durable facts, patterns, gotchas, and
constraints so future agents retrieve them instead of rediscovering them.

## CLI

The CLI is the source of truth: `doctrine memory --help`, never guess.
Key verbs: record, verify, find, retrieve, show, list, validate, edit, tag,
status. Every flag and subcommand shape is verified at the binary — a guessed
flag is a stale flag.

## What memory records

Each memory carries structured fields (type, scope, trust, git anchor) in
`memory.toml` and prose body in `memory.md`, plus a `mem.<key>` symlink alias
for key-based lookup. Records live under `.doctrine/memory/items/` (local) or
`memory/` (shipped corpus, via `--global`).

## Verification

A verified memory has been attested against a specific working-tree commit.
Unverified memories carry a caveat in retrieval. Verification refuses a dirty
tree — no false attestation. Records decay: a memory attested against a commit
from 50 commits ago carries a staleness penalty in ranking.

## Retrieval and trust

`doctrine memory retrieve` supplies data-not-instruction blocks for agent
context. It applies a **non-bypassable trust holdback**: low-trust,
high-severity memories are suppressed. Use `find` (holdback-exempt) to
inspect what `retrieve` withheld. `show` provides the full body of one
memory by uid or key.

See [[concept.doctrine.memory-model]] for the two-faces model,
[[concept.doctrine.storage-model]] for the storage rule,
and [[fact.doctrine.cli-source-of-truth]] for the CLI.
