# REQ-480: The pi boot adapter plans and installs owned `index.ts`, `mcp.ts` and `surface.ts` in `.doctrine/state/pi/extensions` through the shared ownership-aware descriptor core (generate absent, regenerate changed owned files, skip foreign files), reporting each file separately, and ensures `.pi/settings.json` lists `../.doctrine/state/pi/extensions` while preserving foreign entries. A malformed or symlinked settings file, or an `extensions` key that is not an array, is left untouched and the required entry is named; that outcome is reported, a settings write counts as touched, and the fallback leaves the boot leg incomplete. A write prints a project-trust notice that promises `/reload` loads the changes only when the registration was written, and never claims activation. `surface.ts` keeps the neutral-envelope, async, bounded, fail-open memory-surface adapter contract. Codex writes no pi files.

## Statement

<!-- The sister TOML's `description` field is the primary, normative statement.
     Prose here may elaborate, expand upon, or disambiguate it — never
     duplicate it. -->

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
