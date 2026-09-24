# REQ-480: `boot install` plans and installs the generated pi extensions — `index.ts`, `mcp.ts`, and `surface.ts` — through one descriptor core (four-way action, foreign-skip, regenerate-on-change) and reports each as its own `RefreshReport` leg; `surface.ts` is the neutral-wire adapter that maps pi's `read`/`edit`/`write` and `bash` tool results onto doctrine's neutral envelope, writes it to `memory surface --input neutral` on an async, bounded, fail-open child, and appends the returned block.

## Statement

<!-- The sister TOML's `description` field is the primary, normative statement.
     Prose here may elaborate, expand upon, or disambiguate it — never
     duplicate it. -->

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
