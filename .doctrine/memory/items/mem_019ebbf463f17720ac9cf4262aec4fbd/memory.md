# Explicit --slug must be charset-validated at resolve_slug, the single chokepoint for all authored-entity slugs

A user value that becomes a path component needs a charset wall at resolve_slug, not just a byte cap.

`input::resolve_slug` is the single chokepoint every authored kind's slug flows
through — `slice new`, `adr new`, `spec req add` all call it, and the resolved
slug is spliced straight into the `NNN-slug` / `requirement-NNN-slug` symlink name
(`requirement.rs`, etc). A byte-length cap is **not** filesystem safety: an
explicit `--slug` like `../../x`, `a/b`, `..`, or one with whitespace/control
chars passes a length check and traverses or breaks out of the entity dir
(path-component injection). `derive_slug` sanitizes only the *derived* path; an
explicit `--slug` bypasses it.

Wall: validate an explicit slug to the shape `derive_slug` already guarantees —
`^[a-z0-9]([a-z0-9-]*[a-z0-9])?$` (`slug_is_well_formed`) — at `resolve_slug`, so
every authored kind is sanitized at once (no parallel implementation). The TOML
sink is escaped separately (`toml_string` / `HOSTILE_SLUG`); that does nothing for
the filesystem sink — escape each sink for its own grammar.

Found in SL-049 RV-008 F-1 (the cap landed in PHASE-02 ISS-004 guarding length but
not separators). Same class as [[mem.pattern.render.toml-splice-escape-user-values]].
