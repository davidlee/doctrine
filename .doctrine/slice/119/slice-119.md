# SL-119: Wire boot snapshot to pi via APPEND_SYSTEM.md

## Context

`doctrine boot install` delivers the governance snapshot to Claude via a
`SessionStart` hook. For pi (codex), the sole delivery mechanism is
`@.doctrine/state/boot.md` prepended to `AGENTS.md` — a text reference pi
does not resolve. The LLM may not read it; when it does, it costs a round-trip.

Pi natively supports `.pi/APPEND_SYSTEM.md` (auto-appended to every session's
system prompt) and extensions with `session_start` events (the pi-native
equivalent of Claude's hooks). IMP-116 scoped the design; this slice implements
it.

## Scope & Objectives

1. **Boot sentinel.** `render_boot` emits a stable HTML comment sentinel
   (`<!-- BOOT-SENTINEL: doctrine-governance-snapshot -->`) so the context-file
   guard can distinguish "boot content is already in context" from "need to
   read it."

2. **AGENTS.md sentinel guard.** `ensure_boot_import` writes an augmented `@`
   line: a conditional telling the LLM to read the file only if the sentinel
   isn't already in its system prompt.

3. **APPEND_SYSTEM.md symlink.** `install_refresh` for `Harness::Codex` creates
   `.pi/APPEND_SYSTEM.md → ../../.doctrine/state/boot.md`. Conditional: only
   when `.pi/` or `.codex/` exists, or `--agent codex`.

4. **Pi session_start extension.** `install_refresh` for `Harness::Codex`
   generates `.pi/extensions/doctrine-boot-refresh.ts` — a small extension that
   runs `doctrine boot` on `session_start`. The resolved doctrine binary path is
   baked into the generated file at install time.

5. **Existing behaviour preserved.** The `@` import in context files, Claude
   hook, baseRef, and MCP registration are untouched. All existing boot tests
   stay green.

## Non-Goals

- Changing Claude delivery mechanism
- Changing how boot.md itself is rendered (beyond the sentinel)
- Re-architecting `install_refresh` (new Codex legs fit the existing match arm)
- Any pi-specific behaviour beyond the harness wire-up

## Affected Surface

| File | Change |
|---|---|
| `src/boot.rs` | Sentinel in `render_boot`; `install_refresh` Codex arm expanded |
| `src/boot.rs` | `ensure_boot_import` sentinel-guard logic |
| `src/boot.rs` | New pure helpers: `plan_append_system`, `plan_codex_extension` |
| `.pi/APPEND_SYSTEM.md` | Created as symlink (gitignored; `.pi/` already gitignored) |
| `.pi/extensions/doctrine-boot-refresh.ts` | Generated TS extension |

## Risks & Assumptions

- Pi resolves symlinks when reading `.pi/APPEND_SYSTEM.md` — verified (standard
  POSIX, pi uses Node.js `fs.readFileSync`)
- Pi loads extensions from `.pi/extensions/` on session start — verified (pi
  extension discovery)
- The sentinel is stable (not regenerated per-session) so the AGENTS.md
  conditional doesn't rot
- Symlink dangles before first `doctrine boot` — acceptable; install writes it,
  boot regenerates before next session

## Verification

- Unit: `render_boot` output contains sentinel
- Unit: `plan_boot_import` with sentinel in content → `Present` (no rewrite)
- Unit: `plan_boot_import` without sentinel → `Add` with guard line
- Unit: `plan_append_system` symlink plan (dry-run path)
- Unit: `plan_codex_extension` generates valid TS with baked binary path
- Integration: `doctrine boot install --agent codex` creates symlink + extension
- Integration: existing Claude install path unchanged
- Integration: `doctrine boot install` without `.pi/` dir is a no-op for codex leg
- Behaviour-preservation: all existing boot tests green

## Follow-Ups

- None expected. The symlink + extension pattern may inform future harness
  integrations.
