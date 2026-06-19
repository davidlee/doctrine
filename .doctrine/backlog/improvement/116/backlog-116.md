# IMP-116: Deliver boot snapshot to pi via APPEND_SYSTEM.md

## Problem

`doctrine boot install` prepends `@.doctrine/state/boot.md` to `AGENTS.md` as
the sole delivery mechanism for the codex (pi) harness. Pi does not resolve `@`
references within context files — the line arrives as literal text. The LLM must
choose to read the file (fragile, costs a round-trip), and may not.

The Claude harness gets a `SessionStart` hook that runs `doctrine boot`;
codex is deliberately import-only (no hook). The `@` reference alone is
insufficient.

## Design

Pi natively supports `.pi/APPEND_SYSTEM.md` — content is auto-appended to every
session's system prompt with zero round-trips. `doctrine boot install` should
generate this file (full boot snapshot content) as the primary pi delivery
vehicle, under the existing `Harness::Codex` arm in `boot::install_refresh`.

Per-harness delivery after:

| Harness | Primary | Fallback |
|---|---|---|
| codex (pi) | `.pi/APPEND_SYSTEM.md` — full boot content | `@` ref in `AGENTS.md` |
| claude | `SessionStart` hook → `doctrine boot` | `@` ref in `CLAUDE.md` |

The existing `@` import in context files stays as the portable fallback and
coordination trigger.
