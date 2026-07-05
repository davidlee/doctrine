# Design SL-208: Consistent cozy-table help for all subcommand --help screens

## 1. Design Problem

`doctrine --help` renders a clean grouped cozy-table (SL-150). Every subcommand-level
`--help` screen (`doctrine worktree --help`, `doctrine dispatch --help`, etc.) falls
through to clap's default flat-list formatter — left-aligned descriptions that wrap
into illegible tangles when they exceed ~30 characters. The worst offenders
(`worktree` 15 subcommands, `dispatch` 10 subcommands, `revision` 8 subcommands)
have descriptions wrapping across 3-4 lines. The options section has the same
problem: arg names and their help text share a single ragged column.

## 2. Current State

- `src/main.rs` — intercepts `DisplayHelp` / `MissingSubcommand` errors. When
  `args` contain a real subcommand name, it deliberately falls through to
  `e.exit()` (clap's default rendering). When there's no subcommand, it renders via
  `render_top_level_help`.
- `src/commands/cli.rs` — `render_top_level_help` (flat `command │ description`
  table, no grouping here — the grouped version is the family-banded
  `render_grouped` surface), `render_commands_table` (3-col `--commands` table),
  `render_boot_map` (dense PUSH-tier spine + families).
- `src/listing.rs` — `render_columns` / `render_table` cozy-table machinery:
  `Column`, `ColumnPaint`, `RenderOpts`, `force_no_tty`, `trim_end`, `│` separators.
  Pure leaf — no clap, no tty reads.

## 3. Target State

Every `doctrine <cmd> --help` that has subcommands renders three clean sections:

```
<about text>

Usage: doctrine worktree [OPTIONS] <COMMAND>

Commands:
  provision           │ Copy allowlisted files into a worktree fork. The sole...
  check-allowlist     │ Check `.worktreeinclude` for invalid patterns...
  ...

Options:
  --color <COLOR>     Control colour output [default: auto] [possible values: auto, always, never]
  -h, --help          Print help
```

- **Subcommands**: `command │ description` cozy-table, `│` separators, same style
  as top-level help.
- **Options**: two-column layout — arg name padded to a shared width + 2-space
  gutter + help text. No `│`, no box-drawing chars, no borders. Wrapping
  continuation lines indent to the same column.
- **Leaf commands** (no subcommands): keep clap default — their `--help` is just
  about + usage + individual args, which clap renders adequately.
- **`--color never`**: no ANSI escapes anywhere, plain ASCII throughout.

## 4. Design Decisions

### D1: Intercept in `main.rs`, render from clap tree (same pattern as top-level)

The `DisplayHelp` error-handling arm in `main.rs` already has the shape:
intercept → check `has_real_subcommand` → if false, render custom. We extend it:
if `has_real_subcommand`, walk the clap command tree from `Cli::command()` to the
target, extract data, and render via the shared `listing` machinery.

Alternative considered: clap `help_template` — clap templates have no table-rendering
capability; `{subcommands}` is a pre-rendered placeholder. Not viable.

Alternative considered: post-process clap's default output — fragile (depends on
clap's exact format, which changes across versions). Rejected.

### D2: Options section — no borders, two-column padded layout

The options section renders as a clean two-column layout: arg name column
(compacted from clap's short/long/value_names), right-padded to a shared width,
help text column (verbatim from clap). No `│` separators, no box-drawing lines.

The alternative (full cozy-table with `│`) was considered but rejected — the `│`
noise on a dense options block is distracting. The borderless approach keeps it
scannable with minimal visual weight.

### D3: Leaf command pass-through

Commands with no sub-subcommands keep clap's default output. Their `--help` is
just about + usage + individual args with detailed per-arg help blocks — clap's
default rendering works fine for this shape. Gate: only render our table when
`cmd.get_subcommands().any(|s| !s.is_hide_set() && s.get_name() != "help")`.

### D4: `render_subcommand_help` lives in `src/commands/cli.rs`

Alongside `render_top_level_help`, `render_commands_table`, `render_boot_map`.
All four are pure functions over the clap tree. The interception glue stays in
`main.rs` (the thin binary shell).

## 5. Code Impact

### `src/main.rs`

Replace the `has_real_subcommand` fall-through. Change:

```rust
// Before:
if !has_real_subcommand {
    let help = ...render_top_level_help...;
    writeln!(stdout(), "{help}")?;
    return Ok(());
}
// Falls through to e.exit() for subcommand help — CLAP DEFAULT.

// After:
if has_real_subcommand {
    let path: Vec<&str> = args.iter()
        .filter(|a| !a.starts_with('-') && a != "help")
        .map(String::as_str)
        .collect();
    let help = crate::commands::cli::render_subcommand_help(&path, color, term_width);
    writeln!(stdout(), "{help}")?;
    return Ok(());
}
// No subcommand → render top-level help.
```

### `src/commands/cli.rs`

New functions:

```rust
/// Render the help output for a subcommand (e.g. `doctrine worktree --help`).
/// Walks the clap command tree to the target and renders:
///   about text → usage → subcommands table (│-separated) → options (borderless 2-col)
/// Leaf commands with no sub-subcommands get an empty table section and options rendered
/// borderless.
pub(crate) fn render_subcommand_help(
    path: &[&str],
    color: bool,
    term_width: Option<u16>,
) -> String;

/// Render the options/args of a clap Command as a borderless two-column layout.
/// Arg names (compacted short/long/value) are padded to the widest name + 2-space gutter.
/// Help text from clap verbatim. Wrapping continuation lines indent to the same column.
fn render_options_section(
    cmd: &clap::Command,
    color: bool,
    term_width: Option<u16>,
) -> String;
```

### `src/listing.rs`

No changes. `render_columns` and `render_table` are used as-is for the subcommands
table. The options section uses a new lightweight layout (inline in `cli.rs`, not
in `listing.rs` — it has no clap dependency, but it's a single-use helper tightly
coupled to the clap arg shape, so `cli.rs` is the right home).

## 6. Verification Impact

### Updated tests

| Test | Change |
|---|---|
| `help_snapshot_slice_subcommand` | Switch from `render_help()` to `render_subcommand_help`; assert new format |
| `help_snapshot_memory_subcommand` | Same |
| `help_snapshot_adr_subcommand` | Same |
| `help_snapshot_spec_subcommand` | Same |

### Unchanged tests (must stay green)

| Test | Why |
|---|---|
| `help_snapshot_top_level` | Top-level rendering unchanged |
| `commands_table_structure` | `--commands` rendering unchanged |
| All `listing.rs` tests | `render_columns` / `render_table` reused, not modified |
| All `write_class_tests` | Only help output changes — command dispatch identity preserved |

### New tests

1. `subcommand_help_uses_cozy_table` — output contains `│` separators, subcommand names, no clap default format markers
2. `subcommand_help_no_color_is_plain` — `color: false` → byte-clean, no ANSI
3. `subcommand_help_leaf_no_table` — a leaf command with no sub-subcommands has no `│` in its subcommands section (empty)
4. `options_section_no_borders` — Options section has no `│`, no box-drawing chars
5. `subcommand_help_has_about_and_usage` — output contains about text and `Usage:` line

## 7. Adversarial Review

### F-1 (minor): Subcommand depth gating precise condition

The design says "Subcommand with sub-subcommands". Gate condition: at any depth,
render a Commands table when `cmd.get_subcommands().any(|s| !s.is_hide_set() &&
s.get_name() != "help")`. Otherwise, skip the Commands section; still render the
borderless options section.

### F-2 (edge): `render_usage()` may emit ANSI under `--color never`

clap's `render_usage().to_string()` may include ANSI when clap thinks stdout is a
terminal. Since we bypass `e.exit()`, we must strip ANSI from the usage string when
`color == false`. Implementation: run through a strip-ANSI pass (or use `owo_colors`
`if_supports_color` gating — but usage comes from clap, not our paint functions).
Test: assert no ANSI in usage line under `color: false`.

### F-3 (imprecision): Options wrapping mechanism unnamed

Specify `textwrap::fill` for options help-text wrapping. Pattern:
`textwrap::fill(&help, width).split('\n').enumerate()` — first line gets the arg
name as prefix, continuation lines get a same-width blank indent. `textwrap` is
already in the dependency tree (transitive via other crates). When `term_width` is
`None` (no wrapping), the arg-name column still pads to the shared width but help
text is emitted verbatim without line breaks.

### F-4 (edge): `doctrine worktree help provision --help` path extraction

The path filter `a != "help"` strips intermediate `help` tokens, so
`doctrine worktree help provision --help` → `["worktree", "provision"]` — correct.
Test this edge explicitly.

### F-5 (housekeeping): Link SL-208 supersedes SL-150

SL-150 built top-level family-grouped help; SL-208 completes subcommand-level
table rendering. Record with `doctrine link` after design locks.

## 8. Open Questions

None — all design decisions resolved above.
