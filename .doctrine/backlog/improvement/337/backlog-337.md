# IMP-337: Worker absolute-path reads silently hit primary tree instead of fork — stale content

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Resolution — obsolete (2026-08-14, `SL-254` reconcile)

Dissolved, and by a stronger mechanism than this item asked for.

The defect was possible because an in-session `Agent` worker shared the parent
session's filesystem view, so an absolute path resolved against the primary tree
whatever the worker believed its cwd to be. `SL-254` replaced that arm with a
bwrap-confined subprocess: the worker is spawned under `--ro-bind / /` plus a
read-write bind of its own worktree, with `--chdir` into it. A stale read of the
primary tree is not silently wrong — the primary tree's contents are simply not
what an absolute path inside the fork resolves to, and writes there fail.

The item wanted a warning. The OS now supplies the boundary.

Provenance: `SL-254` Follow-Ups `OQ-2` · `DEC-202` · `ADR-008`.
