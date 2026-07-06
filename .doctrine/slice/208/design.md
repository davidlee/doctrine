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
- **Leaf commands** (no subcommands): skip the Commands table (nothing to
  table-ify) but still render the borderless Options section.
- **`--color never`**: no ANSI escape codes; box-drawing chars (│) are preserved
  (matching the top-level help table contract — no terminal styling).

## 4. Design Decisions

### D1: Intercept in `main.rs`, render from clap tree (same pattern as top-level)

The `DisplayHelp` error-handling arm in `main.rs` already has the shape:
intercept → check `has_real_subcommand` → if false, render custom. We extend it:
if `has_real_subcommand` AND the error kind is `DisplayHelp` or
`DisplayHelpOnMissingArgumentOrSubcommand` (NOT `MissingSubcommand`), walk the
clap command tree from `Cli::command()` to the target, extract data, and render
via the shared `listing` machinery. `MissingSubcommand` keeps clap's default —
error exit, standard diagnostic message. Scripts that run `doctrine worktree`
without a verb should not silently succeed because the formatter wanted a nicer table.

The path is resolved by walking the clap tree: `Cli::command().find_subcommand(seg[0])`,
then `sub.find_subcommand(seg[1])`, etc. — NOT by filtering raw argv strings (which
would mistake option values like `never` in `--color never` for subcommand names).
`--color` is extracted from the raw args via a simple `--color` lookup before path
resolution.

Alternative considered: clap `help_template` — clap templates have no table-rendering
capability; `{subcommands}` is a pre-rendered placeholder. Not viable.

Alternative considered: post-process clap's default output — fragile (depends on
clap's exact format, which changes across versions). Rejected.

### D2: Options section — no borders, two-column padded layout

The options section renders as a clean two-column layout. The arg name column is
built from clap's own per-arg rendered representation (short, long, value names,
defaults, possible values, required/optional markers, env var annotations — the
full info contract clap already generates). We extract this from clap's per-arg
`get_long_help()` and `get_help()` which carry the baked-in annotations, rather
than hand-assembling from individual getters that would silently drop semantics.

The arg name is right-padded to the widest name across all visible args + a
2-space gutter. The help text column uses clap's per-arg help verbatim. No `│`
separators, no box-drawing lines.

Help text wrapping uses `textwrap::fill` (a new direct dependency in Cargo.toml —
it is NOT assumed transitive). First line gets the arg name as prefix; continuation
lines get a same-width blank indent. When `term_width` is `None`, help text is
emitted verbatim without line breaks.

The alternative (full cozy-table with `│`) was considered but rejected — the `│`
noise on a dense options block is distracting. The borderless approach keeps it
scannable with minimal visual weight. This layout is a thin inline helper in
`cli.rs` — it is NOT a second table implementation; it's key-value padding that
reuses `RenderOpts` for color/width but does not need the grid machinery from
`listing.rs` (no separators, no column alignment across multiple rows — just
per-row padding).

### D3: Leaf commands skip Commands table, keep Options section

Commands with no visible non-help sub-subcommands skip the Commands table (there
are no subcommands to table-ify). They still get the borderless Options section.
Rationale: leaf commands like `doctrine onboard --help` and `doctrine serve --help`
have args/options that benefit from the cleaner two-column layout even without
subcommands.

Gate: the Commands table renders when `cmd.get_subcommands().any(|s| !s.is_hide_set()
&& s.get_name() != "help")`. The Options section always renders if the command has
visible args.

### D4: `render_subcommand_help` lives in `src/commands/cli.rs`

Alongside `render_top_level_help`, `render_commands_table`, `render_boot_map`.
All four are pure functions over the clap tree. The interception glue stays in
`main.rs` (the thin binary shell).

## 5. Code Impact

### `src/main.rs`

Only `DisplayHelp` and `DisplayHelpOnMissingArgumentOrSubcommand` get custom
rendering. `MissingSubcommand` keeps clap's default (error exit). Path resolution
walks the clap command tree, not raw argv:

```rust
// Replace the has_real_subcommand fall-through:
if has_real_subcommand
    && !matches!(e.kind(), clap::error::ErrorKind::MissingSubcommand)
{
    // Walk clap tree to resolve path segments (NOT raw argv filtering).
    // Extract --color from raw args before path resolution.
    let color_raw = args.iter()
        .position(|a| a == "--color")
        .and_then(|i| args.get(i + 1).map(|v| v.as_str()))
        .unwrap_or("auto");
    let color = crate::tty::resolve_color(match color_raw {
        "always" => clap::ColorChoice::Always,
        "never" => clap::ColorChoice::Never,
        _ => clap::ColorChoice::Auto,
    });
    let term_width = crate::tty::stdout_terminal_width();
    let path_segments: Vec<&str> = args.iter()
        .filter(|a| !a.starts_with('-') && *a != "help" && *a != color_raw)
        .map(String::as_str)
        .collect();
    let help = crate::commands::cli::render_subcommand_help(&path_segments, color, term_width);
    writeln!(stdout(), "{help}")?;
    return Ok(());
}
// MissingSubcommand, or no subcommand → fall through to e.exit() or top-level render.
```

### `src/commands/cli.rs`

New functions:

```rust
/// Render the help output for a subcommand (e.g. `doctrine worktree --help`).
/// Walks the clap command tree via `find_subcommand` iteratively — each segment
/// must resolve, else returns "unknown command" message. Renders:
///   about text → usage → subcommands table (│-separated, no header row) → options (borderless 2-col)
/// Leaf commands with no sub-subcommands skip the Commands table; always render
/// the borderless options section if the command has visible args.
pub(crate) fn render_subcommand_help(
    path: &[&str],
    color: bool,
    term_width: Option<u16>,
) -> String;

/// Render the options/args of a clap Command as a borderless two-column layout.
/// Arg names are extracted from clap's own per-arg rendered representation
/// (preserving the full info contract: defaults, possible values, value names,
/// required/optional markers, env var annotations). Padded to the widest name +
/// 2-space gutter. Help text is wrapped via `textwrap::fill` with continuation
/// lines indented to the same column. When `term_width` is None, no wrapping.
fn render_options_section(
    cmd: &clap::Command,
    color: bool,
    term_width: Option<u16>,
) -> String;
```

### `src/listing.rs`

No changes. The subcommands table uses `render_table` directly (NOT
`render_columns`, which always emits a header row). The grid is built manually —
no header row, just sanitized data rows — matching the top-level help style where
families are the structure, not column headers. The parent command name is already
clear from `Usage: doctrine worktree`, so a "command │ description" header is noise.

### `Cargo.toml`

New direct dependency: `textwrap` (for options help-text wrapping). Currently not
a direct dep — must be added explicitly. `owo_colors` is already a direct dep
(for ANSI stripping in the usage line when `color == false`).

## 6. Verification Impact

### Updated unit tests (in `src/main.rs` test module)

| Test | Change |
|---|---|
| `help_snapshot_slice_subcommand` | Switch from `render_help()` to `render_subcommand_help`; assert new format |
| `help_snapshot_memory_subcommand` | Same |
| `help_snapshot_adr_subcommand` | Same |
| `help_snapshot_spec_subcommand` | Same |

### New unit tests (in `src/main.rs` test module)

1. `subcommand_help_uses_cozy_table` — output contains `│` separators, subcommand names, no clap default format markers
2. `subcommand_help_no_color_is_plain` — `color: false` → no ANSI escapes in any section (usage, subcommands, options)
3. `subcommand_help_leaf_skips_commands_table` — a leaf command's output has no Commands heading, but has the Options section
4. `options_section_no_borders` — Options section has no `│`, no box-drawing chars
5. `subcommand_help_has_about_and_usage` — output contains about text and `Usage:` line
6. `subcommand_help_path_walks_clap_tree` — path `["worktree", "help", "provision"]` resolves correctly (strips intermediate `help` tokens)
7. `subcommand_help_ansi_stripped_from_usage_plain_mode` — clap-generated usage line is ANSI-free under `color: false`

### New black-box integration tests (in `tests/`)

These exercise the actual binary via `std::process::Command` — the risky
intercept/exit-status path in `main.rs`, not just the pure renderer:

| Test | What it proves |
|---|---|
| `cli_subcommand_help_worktree` | `doctrine worktree --help` exits 0, contains `│` separators and subcommand names |
| `cli_subcommand_help_plain_mode` | `doctrine --color never worktree --help` exits 0, no ANSI escapes |
| `cli_subcommand_help_depth2` | `doctrine memory sync --help` exits 0, contains cozy-table |
| `cli_missing_subcommand_errors` | `doctrine worktree` without a verb exits non-zero (MissingSubcommand preserves error contract) |
| `cli_help_help_path` | `doctrine worktree help provision --help` exits 0, renders help for provision |
| `cli_top_level_help_unchanged` | `doctrine --help` exits 0, contains family headings (top-level unchanged) |

### Unchanged tests (must stay green)

| Test | Why |
|---|---|
| `help_snapshot_top_level` | Top-level rendering unchanged |
| `commands_table_structure` | `--commands` rendering unchanged |
| All `listing.rs` tests | `render_columns` / `render_table` reused, not modified |
| All `write_class_tests` | Only help output changes — command dispatch identity preserved |

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
