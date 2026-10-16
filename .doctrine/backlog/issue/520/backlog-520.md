# ISS-520: Global agent-def bake is overwritten by the last project to install

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The problem

`doctrine install --global` writes agent-def canon copies to
`$HOME/.doctrine/agents/`, shared by every project. Defs containing
`WORKER_RESOLVE_MARKER` are baked from the **installing project's** on-disk hymn
overlay and seal set (`install_agent_def` → `bake_worker_def`,
`src/install.rs`). So the global def reflects whichever project installed last,
and every other project's sessions load that project's worker context.

Raised by RV-418 F-3 during SL-281 design. SL-281 stops global installs from
pruning (DEC-377) but leaves the bake as-is.

## Options

- Bake only the framework corpus for global installs (no project overlay).
- Refuse to bake marker defs globally; tell the user to install per project.
- Keep per-project canon even for global harness links (links into the
  project, not `$HOME`). Breaks the point of global.

## Related

- SL-281, DEC-377.
