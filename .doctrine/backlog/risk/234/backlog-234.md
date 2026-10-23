# RSK-234: Baked /nix/store doctrine path breaks after garbage collection

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Bake sites resolve the binary through `resolve_exec()` (`current_exe`), which on
Linux yields the resolved path. A nix-installed doctrine resolves to
`/nix/store/<hash>-doctrine/bin/doctrine`. After an upgrade and garbage
collection that path disappears, breaking every baked consumer until `install`
re-runs: the codex user-level MCP entry (SL-286, DEC-404 notes it) and the codex
hooks in `.codex/hooks.json`. Possible mitigation: prefer a stable profile path
(e.g. the PATH-resolved, non-canonicalised `doctrine`) when it resolves to the
same file as `current_exe`.
