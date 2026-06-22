# IMP-153: doctrine search table output uses hand-rolled format!() instead of comfy-table render_table

## Context

`doctrine search` prints results using a hand-rolled fixed-width format!() table:

```
{:12} {:8} {:24} {:>8}   {}
```

This wraps badly on narrow terminals, looks cheap (no column separators), and the
Score column is visually noisy — BM25 scores are an internal ranking detail.

The `listing` module already has `render_table` (comfy-table based) with proper
` │ ` column separators, ANSI-aware width measurement, and terminal-width wrapping
(via `RenderOpts.term_width`). Every other list surface uses it.

## Scope

1. Replace `Format::Table` arm in `search::run` with `listing::render_table`.
2. Drop the Score column — scores are an internal ranking axis, not displayed.
3. Thread `RenderOpts` through the dispatch call site so wrapping + colour work.
4. Keep `--context` snippet lines as indented rows below each table row.

### Out of scope

- The `--help` wrapper/wrapping issue is tracked separately in ISS-018.
- JSON format stays unchanged.
- No schema/entity changes.

## Acceptance (as-implemented)

- `doctrine search <query>` prints a proper ` │ `-separated table with columns
  ID, Kind, Status, Title (no Score). ID cyan, status coloured via `status_hue`,
  title zebra-striped.
- Piped output stays clean (no ANSI).
- Existing tests stay green.

## Remaining: --context integration (IMP-153 follow-up)

`--context` currently dumps all snippets as paragraphs *after* the table, which is
useless — they need to be interleaved per-result. Considered approaches:

1. **Snippet in Title cell** — cramped (~50% term width), bad for long snippets.
2. **Row-per-result mini tables** — inconsistent column widths across results.
3. **Structured text blocks** (metadata line + full-width snippet + blank-line
   separator) — readable but diverges from the table layout when `--context` is on.
4. **Blank first 3 cols, snippet in last col** — rect grid + render_table works,
   but snippet still gets only the Title column width, not full width.

None ideal. Best path probably: keep the clean table for no-context, and for
`--context` mode emit results as structured text blocks → full-width snippet
under each result with consistent metadata prefix. That or a `render_table` hack
with merged snippet spanning the full grid width (comfy-table doesn't support
colspan natively).

## Resolution (--context interleave)

Hybrid of approaches 1 and 3: still render the metadata through `render_columns`
(so ID/Kind/Status/Title stay comfy-table-aligned and coloured), then split that
output by line and interleave the full-width snippet beneath each row, blank line
between results. Non-`--context` path is the plain table, unchanged. No hand-rolled
alignment, no colspan hack; the snippet gets the full terminal width.

Rows + snippets are built in one index-parallel pass (replacing the prior
`filter_map`, which could drop a row and desync the interleave). The rendered table
is `header\nrow0\nrow1\n…` with no trailing empty line, so `lines()` yields
header-then-rows that zip cleanly against the snippet vec.
