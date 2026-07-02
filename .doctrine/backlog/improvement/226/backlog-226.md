# IMP-226: Remove skills.rs and skills CLI verb — move remaining live functions to install.rs

## Context

`skills install` was consolidated into `doctrine install` (SL-088). `doctrine
skills` now only has `list` (unreadable raw-text dump) and `help`.

`src/skills.rs` is ~2250 LOC. The `run_install` call chain is dead — marked
`#[expect(dead_code)]`, only called from its own tests. The CLI verb is
hidden (`#[command(hide = true)]`).

## What stays (move into `src/install.rs`)

- `discover()`, `select_for_install()`, `validate_filters()`
- `install_agents_for()` → `install_agent_def()` + link/dir helpers
- `install_for_other()` → `delegate_argv()`
- `install_hooks_plugin_for_claude()` + hook templating
- `InstallOtherArgs`, `Agent`, `ProcessRunner`, embedded assets

## What goes

- `run_install()` + `InstallArgs` (skills.rs version)
- `build_plan()`, `execute()`, `print_plan()`
- `materialise_canonical()`, `copy_skill()`, `staging_path()`
- `resolve_agents()`, `resolve_install_ids()`, `subset_ids()`
- `claude_links()`, `canonical_dir()`
- `run_list()`, `dispatch()`, `SkillsCommand`
- All dead-code tests
- The `skills` CLI verb + its `Family` entry

## Spike first

Confirm no external consumers of `doctrine skills list` before removing the
verb entirely.
