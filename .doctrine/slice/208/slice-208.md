# Consistent cozy-table help for all subcommand --help screens

## Context

`doctrine --help` renders a clean, readable grouped table via `render_top_level_help()`
(comfy-table columns `command │ description`). Every subcommand-level `--help`
screen (`doctrine worktree --help`, `doctrine dispatch --help`, etc.) falls through to
clap's default formatter — a flat indented list with left-aligned descriptions that
wrap into an illegible tangle when the descriptions are longer than ~30 characters.

The worst offenders (`worktree`, `dispatch`, `revision`, `check`, `memory`) have
subcommand descriptions exceeding 80 chars and wrapping across 3-4 lines. Even
the "OK" ones (`config`, `export`, `map`) lack the consistent table formatting.

## Scope & Objectives

- Intercept `DisplayHelp` / `MissingSubcommand` errors at the subcommand level in
  `main.rs` (same interception point that currently only catches top-level help).
- Render every subcommand-level `--help` screen through a shared cozy-table
  formatter that matches the top-level style:
  - About text (first paragraph, like top-level)
  - Usage line (preserve clap's `.render_usage()` or generate our own)
  - Subcommands table: `command │ description` (comfy-table, same styling as top-level)
  - Arguments / Options (render from clap's arg list — format as a table or
    fall through to clap for this section)
- Cover ALL subcommands uniformly — no one-off intercept per command.
- Subcommands at depth 2+ (e.g. `doctrine slice status --help`) also use the same
  cozy-table rendering if they have sub-subcommands.
- Maintain `--help` text in existing help-snapshot tests. The help text visible to
  users changes (format only), but the information content is preserved or improved.

## Non-Goals

- Not changing the content of descriptions (only formatting).
- Not changing `--help` output for leaf subcommands with no sub-subcommands
  (clap's default is fine for a single-action command's args listing).
- Not rebuilding the arg/option rendering from scratch — we can still use clap
  for options/args at leaves and for the "about" content.

## Summary

In `main.rs`, extend the error-kind intercept that already exists for top-level
`DisplayHelp` to also catch `has_real_subcommand` cases. Write a new
`render_subcommand_help(path_segments, color, term_width)` function in
`src/commands/cli.rs` (alongside `render_top_level_help`, `render_commands_table`,
`render_boot_map`) that walks the clap command tree to the target subcommand and
renders its about, subcommands table, and options in the cozy-table style.

With `--color never` the output is plain ASCII — no ANSI escapes — matching the
plain-mode conformance of `render_top_level_help`.

## Follow-Ups

- Consider rendering leaf-command arg tables in cozy-table too (out of scope here).
