# Consolidate installer commands: single DWIM install with per-agent opt-in

## Context

Currently, fully installing or updating doctrine in a project requires five
distinct commands:

```bash
doctrine install          # base files, dirs, gitignore
doctrine memory sync       # shipped memory corpus
doctrine boot install      # @-import wiring + session hooks
doctrine claude install    # skills for Claude + dispatch-worker agent + SubagentStart hook
npx skills add davidlee/doctrine --agent pi -y  # skills for non-Claude agents (delegated)
```

SL-084 is adding pi subagent definitions, making the surface even more
fragmented. The user wants a single `doctrine install` that does the right
thing, with per-agent opt-in for the more invasive steps.

## Scope & Objectives

### 1. Unified `doctrine install` CLI surface

`doctrine install` gains the agent/skill selection flags currently on `doctrine
claude install`: `--agent`, `--skill`, `--domain`, `--only-memory`, `--global`.
These join the existing `-p`, `--dry-run`, `-y`.

```
doctrine install [--agent <name>...] [--skill <id>...] [--domain <name>...]
                 [--only-memory] [--global] [--dry-run] [-y] [-p <path>]
```

### 2. Base install always runs first

The existing file/dir/gitignore manifest install runs unconditionally. This is
non-invasive (creates missing dirs, skips existing files, appends gitignore
entries idempotently).

### 3. Forward steps with individual prompts

After the base install, each "forward" (invasive) step is presented with its
own confirmation prompt. `-y` skips all prompts and proceeds with everything.
`--dry-run` prints the full plan including forward steps.

The forward steps, in order:

| Step | Invasive action | Default (no flags) |
|------|----------------|---------------------|
| Memory sync | Materializes shipped corpus into `.doctrine/memory/shipped/` | Prompted |
| Boot install | Wires `@`-import into AGENTS.md/CLAUDE.md + session hooks | Prompted |
| Skills install (per detected agent) | Symlinks skills + agent defs + hooks | Prompted per agent |

Agent auto-detection follows the existing `resolve_agents` logic (`.claude/`
directory → Claude; explicit `--agent` names override). Non-Claude agents
delegate to `npx skills` as today.

### 4. `doctrine claude install` removed as primary command

The `claude install` subcommand is removed from the public CLI. `skills
install` (already a hidden deprecated alias) may also be removed. The
underlying machinery (`skills::run_install`, `boot::install_claude_hook`,
etc.) is preserved — it's now called from `install::run`, not from its own
command entry point.

### 5. Standalone focused commands preserved

- `doctrine memory sync` — standalone, unchanged
- `doctrine boot install` — standalone, unchanged
- `doctrine skills list` — standalone, unchanged

These remain as fine-grained knobs for scripting/CI.

### Affected files

- `src/main.rs` — CLI surface: move agent flags to `Install`, remove `ClaudeCommand::Install`
- `src/install.rs` — orchestrate forward steps, prompt logic
- `src/skills.rs` — `run_install` becomes an internal function called from install
- `src/boot.rs` — `run_install` (boot) callable from install
- `src/memory.rs` — `sync` callable from install
- `install/` — no manifest changes (the shipped files are correct)
- `plugins/` — no changes (skills content unchanged)

## Non-Goals

- Changing the external `npx skills` delegation mechanism
- Adding new agent types or skill domains
- SL-084's pi agent definition *content* (that slice owns the content; this
  slice ensures the consolidated install path ships it)
- Replacing `npx skills` with a native Rust implementation
- Changing the boot snapshot or memory corpus content

## Risks & Assumptions

- **ASM-1:** `npx`/Node is available when delegating to non-Claude agents.
  Existing assumption; not introduced here.
- **RSK-1:** Removing `claude install` may break user scripts. Mitigation:
  keep as hidden alias for one release cycle, printing a migration hint.
- **RSK-2:** The forward-step prompt flow must not be confusing. Each prompt
  must be clear about what will be written where. The `--dry-run` output
  serves as the reference plan.

## Verification intent

- **VT:** `doctrine install --dry-run` prints the full plan including forward
  steps with agent detection.
- **VT:** `doctrine install -y` completes all steps (files + memory + boot +
  claude skills) without prompting.
- **VT:** `doctrine install` (no flags, answer "n" to all prompts) does only
  the base install — files, dirs, gitignore — and exits clean.
- **VT:** `doctrine install --agent pi` delegates to `npx skills` for pi and
  installs pi agent defs if present.
- **VA:** `doctrine claude install` is gone from `--help`; `doctrine --help`
  shows only the consolidated `install`.
- **VT:** Existing standalone commands (`memory sync`, `boot install`, `skills
  list`) continue to work unchanged.

## Summary

One command to rule them all: `doctrine install` does base files + asks about
each invasive step. `-y` says yes to everything. The forward steps (memory
sync, boot wiring, per-agent skills) are individually prompted so the user
stays in control. `--dry-run` shows what will happen. Standalone focused
commands remain for scripting.
