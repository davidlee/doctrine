# SL-014 Implementation Plan

## Rationale

Two phases, ordered by dependency. PHASE-01 (`boot --emit`) is a standalone CLI
addition with zero coupling to the hook install — it's tested in isolation, rides the
existing `build_sections` + `render_boot` paths, and the factored `build_and_render`
helper is the only shared surface with `regenerate`.

PHASE-02 wires the codex hook install through the existing `plan_hook` merge core.
It adds the `HookSpec::boot_emit` constructor, the `is_doctrine_emit_command`
ownership predicate, extracts `install_hook_to_file` as a DRY helper (behaviour-preserving
for the Claude callers), expands `PrintedFallback` to carry path+snippet, and adds
trust surfacing + spike coexistence warning to the `wire()` output.

The phase boundary is sharp: PHASE-01 is pure boot machinery (no hook knowledge);
PHASE-02 is pure hook install (no boot CLI knowledge beyond `HookSpec::boot_emit`'s
command string).

## Phase sequencing

### PHASE-01: `boot --emit` CLI flag

Standalone. Extracts `build_and_render(root, exec) -> String` from `regenerate`'s
internals, makes `regenerate` = `build_and_render` + `write_if_changed`, and adds
`run_emit` = `build_and_render` + `write_if_changed` + `write!(stdout, ...)`. The
`--emit` CLI flag is mutually exclusive with `--check`.

Why first: no coupling to hooks. The `build_and_render` refactor is a pure
extraction — existing `regenerate` tests stay green as the behaviour gate.

### PHASE-02: Codex hook install + trust surfacing

Adds the full codex arm. Rides the existing JSON merge core (`plan_hook`,
`hook_array_mut`, `drop_owned_hooks`, `owned_positions`) unchanged — the
`.codex/hooks.json` structure is identical to `.claude/settings.local.json`.

Key design decisions carried from the design:
- `install_hook_to_file` extraction avoids parallel implementation — both
  `install_claude_hook` and `install_codex_hook` are one-line calls
- `PrintedFallback` becomes a struct variant with `hook_file` + `snippet`, so
  `wire()` prints the right file and command for each harness
- Ownership predicate uses suffix-strip (` boot --emit`) + shared
  `is_doctrine_program` — same pattern as the three existing predicates
- Trust instructions print on `Wired`/`Refreshed`, skip on `None`
- Spike coexistence detection: after writing, re-parse and check for >1
  `SessionStart` entry with the same matcher
