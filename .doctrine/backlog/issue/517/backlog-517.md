# ISS-517: memory retrieve is silently empty in a repo with no git remote

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`doctrine memory retrieve` returns nothing, with no explanation, in a repo that
has no git remote. That is a silent degraded read, which STD-003 ("no silent
skip") forbids: the empty result should say why.

Split from ISS-507 (fix sketch item 3). Evidence: friction observation `01a0e814`.

## Fix sketch

Find where retrieval keys on the remote (repo identity / anchoring) and disclose
the degraded read — a stderr notice or a footer — instead of an empty page.
