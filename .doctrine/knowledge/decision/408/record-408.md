# DEC-408: User-level codex write degrades with disclosure

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Amended (codex review, 2026-10-23)

User assent: "yes, agreed" to the SL-286 codex-review dispositions.

F6: `fsutil::write_atomic` writes a temp file and renames it over the target, which would replace a symlinked `config.toml` (home-manager users link it into the read-only nix store). The write therefore targets the resolved file. Empty `CODEX_HOME` = unset; relative `CODEX_HOME` degrades because codex's own handling of it is unprobed. F7: dry-run writes neither the user config nor the project retirement.

## Amended (consent and codex CLI, 2026-10-23)

User rulings: "your recommendations are solid. I think -y grants consent, but probably the docs for it need to say so. prompt's good. decline: accept your rec. too". Evidence: EVD-033.

Replaces the direct-file rules: path resolution, symlink write-through and missing-file creation are codex's job now. A missing `CODEX_HOME` fails inside codex (EVD-033 item 3), so doctrine pre-checks and degrades with a clear reason rather than relaying codex's error alone.