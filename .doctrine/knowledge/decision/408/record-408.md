# DEC-408: User-level codex write degrades with disclosure

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Amended (codex review, 2026-10-23)

User assent: "yes, agreed" to the SL-286 codex-review dispositions.

F6: `fsutil::write_atomic` writes a temp file and renames it over the target, which would replace a symlinked `config.toml` (home-manager users link it into the read-only nix store). The write therefore targets the resolved file. Empty `CODEX_HOME` = unset; relative `CODEX_HOME` degrades because codex's own handling of it is unprobed. F7: dry-run writes neither the user config nor the project retirement.