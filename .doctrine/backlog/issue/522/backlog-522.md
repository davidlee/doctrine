# ISS-522: Readers detect project roots with fewer markers than install

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

`doctrine install` finds the project root with the manifest's
`[root_markers]` (`install/manifest.toml`: `.git`, `.project`, `Cargo.toml`,
`pyproject.toml`, `justfile`; `src/install.rs:208-210`). `prompt resolve`
(`src/commands/prompt.rs:118`), `boot` (`src/boot.rs:435-453`), `status`
(`src/status.rs:286-288`) and `worktree provision` use `root::default_markers()`
(`src/root.rs:8-16`: `.git`, `.jj`, `.project`, `Cargo.toml`).

In a project marked only by `pyproject.toml` or `justfile`, install succeeds
but the session hook, `boot` and `status` cannot find the root. SL-282's
currency advisories inherit this. The two sets also disagree on `.jj`.

## Fix direction

One marker policy for every root lookup (manifest-backed, or one shared
constant), with a pyproject-only and justfile-only e2e from install through
the readers.

Surfaced by RV-421 F-9 (SL-282 design review); deferred by the user 2026-10-17.
