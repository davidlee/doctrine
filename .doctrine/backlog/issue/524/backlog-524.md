# ISS-524: install --global still runs the project install

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

`doctrine install --help` says `-g, --global`: "Install to the user directory
instead of the project". But `run` builds and executes the project base plan
(`build_plan`, including the project `doctrine.toml`) before any mode check
(`src/install.rs:207-236`); `--global` only redirects harness convergence to
the user scope and skips the gitignore block.

## Fix direction

Define the `--global` leg set (no project writes), with dry-run and live
e2e fixtures. Sibling: ISS-523 (`--only-memory` unwired).

Surfaced by RV-421 F-13 (SL-282 design review).
