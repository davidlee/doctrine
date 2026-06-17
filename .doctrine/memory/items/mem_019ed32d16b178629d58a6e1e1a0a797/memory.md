# Flag-plumbing fencepost: grep for all callers of the pre-flag detection function when adding a global override

SL-079 added `--color=always|never` flag via `resolve_color(cli.color)`. All
`CommonListArgs` surfaces (12 call sites) and priority/status-line handlers were
migrated, but `coverage_view::run()` still called `stdout_color_enabled()` directly
because it uses its own `render_table` wrapper rather than
`CommonListArgs::into_list_args`.

Lesson: when replacing auto-detection with a flag-gated resolver, grep the entire
codebase for callers of the pre-flag function (`stdout_color_enabled`, NOT just
`into_list_args`) to catch bypass surfaces. The "~13 call sites" count in the
design was a close-enough heuristic that hid the 14th.

