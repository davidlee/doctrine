# IMP-226: Remove skills.rs and skills CLI verb — move remaining live functions to install.rs

## Context

`skills install` was consolidated into `doctrine install` (SL-088). `doctrine
skills` now only has `list` (unreadable raw-text dump) and `help`.

`src/skills.rs` is ~2250 LOC. The `run_install` call chain is dead — marked
`#[expect(dead_code)]`, only called from its own tests. The CLI verb is
hidden (`#[command(hide = true)]`).

## Spike: confirmed no external consumers

Zero references to `doctrine skills list` outside of src/ test code. No e2e
tests, no justfile, no scripts, no docs call it. Safe to remove the CLI verb.

## What stays (move into `src/install.rs`)

Core entry points (called from `install::run()`):
- `discover()`, `select_for_install()`, `validate_filters()`, `select()`
- `resolve_runner()`, `resolve_runner_with()`, `program_available()`
- `install_agents_for()` → `install_agent_def()`
- `install_for_other()` → `delegate_argv()`
- `install_hooks_plugin_for_claude()` → `template_hooks_commands()`, `template_command()`

Types / traits:
- `Meta`, `Entry`, `Agent`, `InstallOtherArgs`
- `Runner` trait, `ProcessRunner`
- `PluginAssets` (RustEmbed)

Link system (called from `install_agent_def()`):
- `classify_link()`, `write_link()`, `staging_path()`
- `foreign_reason()`, `ForeignReason`, `Link`

Path helpers:
- `install_base()`, `claude_dir()`, `agent_canonical_dir()`
- `claude_agents_dir()`, `pi_agents_dir()`
- `relative_path()`, `relative_target()`, `lexists()`

Parsing:
- `parse_meta()`, `parse_agent()`

Tests: ~21 tests for the above functions move with them.

## What goes

Dead CLI: `SkillsCommand`, `dispatch()`, `run_list()`

Dead install chain: `run_install()`, `InstallArgs` (skills.rs version),
`build_plan()`, `execute()`, `print_plan()`

Dead helpers (no caller outside the dead chain): `materialise_canonical()`,
`copy_skill()`, `canonical_dir()`, `resolve_agents()`, `resolve_install_ids()`,
`subset_ids()`, `claude_links()`, `AgentPlan`, `Plan`, `Canonical`

Dead tests: ~18 tests (claude_links, materialise, subset_ids,
resolve_install_ids, only_memory_selects, resolve_agents, build_plan, execute,
run_install, canonical_dir)

CLI wiring to remove:
- `Command::Skills` variant in `commands/cli.rs` (+ dispatch match arm)
- `Command::Skills` match arm in `commands/guard.rs` (`SkillsCommand::List → Read`)
- `mod skills;` in `main.rs`
- Tests in `main.rs`: `skills_list_is_read`, `skills_install_is_gone`,
  `only_memory_conflicts_with_skill`, `only_memory_conflicts_with_domain`

## Design notes

- `staging_path()` is listed as "keep" above (corrected from the original
  card) — it's called from `write_link()` → `install_agent_def()`.
- After the move, `skills.rs`'s calls into `install::embedded_asset()`,
  `install::prompt_confirm()`, `install::ensure_gitignored()` become self-calls
  (remove `crate::install::` prefix).
- No circular dependency risk: `install.rs` already imports from
  `crate::skills`; after the move those become definition-site resolution.
- `ProcessRunner` consolidates into `install.rs` — no separate
  `process_runner.rs` extraction needed (trivial struct + trait).
- `AgentPlan` / `Plan` / `Canonical` have zero callers outside the dead install
  chain — safe to delete.
- The module-level `#![expect(dead_code)]` goes with the file.
