# SL-119: Wire boot snapshot to pi via APPEND_SYSTEM.md

## Context

`doctrine boot install` delivers the governance snapshot to Claude via a
`SessionStart` hook. For pi, the sole delivery mechanism is `@.doctrine/state/boot.md` prepended to
`AGENTS.md` — a text reference pi does not resolve. The LLM may not read it;
when it does, it costs a round-trip.

Pi natively supports `.pi/APPEND_SYSTEM.md` (auto-appended to every session's
system prompt) and extensions with `session_start` events (the pi-native
equivalent of Claude's hooks). IMP-116 scoped the design; this slice implements
it.

## Scope & Objectives

1. **Boot sentinel.** `render_boot` emits a stable HTML comment sentinel
   (`<!-- BOOT-SENTINEL: doctrine-governance-snapshot -->`) so the context-file
   guard can distinguish "boot content is already in context" from "need to
   read it."

2. **AGENTS.md sentinel guard.** `plan_boot_import` writes an augmented `@`
   line: a conditional telling the LLM to read the file only if the sentinel
   isn't already in its context.

3. **APPEND_SYSTEM.md symlink.** `install_refresh` for the pi harness creates
   `.pi/APPEND_SYSTEM.md → ../../.doctrine/state/boot.md`. Creates `.pi/` if
   absent (part of the `doctrine install` setup path).

4. **Pi session_start extension.** `install_refresh` for the pi harness
   generates `.pi/extensions/doctrine/index.ts` — a small extension that runs
   `doctrine boot` on `session_start`. The resolved doctrine binary path is
   baked into the generated file at install time. Named `doctrine` (not
   `doctrine-boot-refresh`) so future handlers (e.g. MCP tool registration)
   slot in alongside.

5. **Existing behaviour preserved.** The `@` import in context files, Claude
   hook, baseRef, and MCP registration are untouched. All existing boot tests
   stay green.

## Non-Goals

- Changing Claude delivery mechanism
- Changing how boot.md itself is rendered (beyond the sentinel)
- Re-architecting `install_refresh` (new pi legs fit the existing match arm)
- Any pi-specific behaviour beyond the harness wire-up

## Affected Surface

| File | Change |
|---|---|
| `src/boot.rs` | Sentinel in `render_boot`; `install_refresh` pi arm expanded |
| `src/boot.rs` | `plan_boot_import` sentinel-guard logic |
| `src/boot.rs` | New pure helpers: `plan_append_system`, `plan_pi_extension` |
| `.pi/APPEND_SYSTEM.md` | Created as symlink (gitignored; `.pi/` already gitignored) |
| `.pi/extensions/doctrine/index.ts` | Generated TS extension |

## Risks & Assumptions

- Pi resolves symlinks when reading `.pi/APPEND_SYSTEM.md` — verified (standard
  POSIX, pi uses Node.js `fs.readFileSync`)
- Pi loads extensions from `.pi/extensions/` on session start — verified (pi
  extension discovery)
- The sentinel is stable (not regenerated per-session) so the AGENTS.md
  conditional doesn't rot
- Symlink dangles before first `doctrine boot` — acceptable; install writes it,
  extension runs boot on first session start, closing the window immediately

## Verification

- Unit: `render_boot` output contains sentinel
- Unit: `plan_boot_import` with sentinel in content → `Present` (no rewrite)
- Unit: `plan_boot_import` without sentinel → `Add` with guard line
- Unit: `plan_append_system` symlink plan (dry-run path)
- Unit: `plan_pi_extension` generates valid TS with baked binary path
- Unit: `plan_pi_extension` skips foreign (user-modified) extension files
- Integration: `doctrine boot install --agent pi` creates symlink + extension
- Integration: `doctrine boot install` auto-detects pi harness and wires it
- Integration: existing Claude install path unchanged
- Integration: idempotent re-run is no-op
- Unit: `RefreshReport` defaults (`NotApplicable`) for Claude arm
- Behaviour-preservation: all existing boot tests green

## Follow-Ups

- MCP tool registration can be added to `.pi/extensions/doctrine/index.ts`
  alongside the `session_start` handler.
- Harness variant renamed from `Harness::Codex` to `Harness::Pi` in this slice.
