# ASM-015: codex mcp get reports the user-layer entry

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Invalidated (2026-10-23)

User observation in this repo (trusted): `codex mcp get doctrine` printed the project-level `sh -c` wrapper — the effective entry — while the user layer held a bare `command = "doctrine"`. `get` reports the merged, effective config, not the user layer. DEC-406/DEC-408 amended: classification reads the user config file directly.