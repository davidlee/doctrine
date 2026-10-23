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

## Amended (ASM-015 invalidated, 2026-10-23)

`codex mcp get` reports the effective (merged) entry, not the user layer (user observation; ASM-015). The user-layer read is a direct, read-only parse of the user config file; the write stays `codex mcp add`.

## Amended (RV-427, 2026-10-23)

Adversarial design review (pi, RV-427). Integrated within the user's rulings; disclosed to the user.

- F-1: "degraded, not failed" means `LegOutcome::Done` with a disclosure line: no codex CLI or no home is an environment fact, so the install still completes and records its version. Only project-entry problems (PathShadow, Malformed, Unreadable) and codex command failures hold the install incomplete.
- F-4: an unreadable or unparseable user config degrades with its own cause.
- F-9: an empty `CODEX_HOME` degrades like a relative one (codex's handling is unprobed), instead of counting as unset.

## Note (RV-427 F-16)

The F6 amendment above says an empty `CODEX_HOME` counts as unset. That is superseded by the RV-427 amendment and the current `choice`: an empty `CODEX_HOME` degrades.