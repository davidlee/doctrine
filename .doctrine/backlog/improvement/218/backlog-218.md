# IMP-218: Survey UX: replace BLOCKED column with red blocking-item styling, add pagination, add --hide-blocked

## Current state

`doctrine survey` renders a "BLOCKED" badge in a dedicated column when an item
has blockers, and a "blocker" column with the direct blocker ID. The column
header is unnamed (empty). Blocked items are always shown.

`doctrine next` already has pagination (`--limit`, `--offset`, `--page`,
default 20) but excludes blocked items entirely.

## Desired changes

1. **Remove the BLOCKED column.** Instead, render blocked rows' `id` cell in red
   (the row whose `act == Blocked`). No separate badge column.

2. **Add pagination** to `survey` matching `next`'s contract: `--limit` (default
   20), `--offset`, `--page`. Keep 20 as the default limit.

3. **Show BLOCKED items by default**, but add `--hide-blocked` to suppress them.
   This keeps survey as the "complete picture" view while letting users focus on
   unblocked work when wanted.

4. **Keep `next` unchanged** — it already excludes blocked items and has
   pagination. The two commands stay distinct: `survey` = everything with
   pagination + optional hide-blocked; `next` = actionable-only (already
   unblocked-only).

## Implementation sketch

### Files touched

| File | Change |
|---|---|
| `src/commands/cli.rs` | Add `--limit`, `--offset`, `--page`, `--hide-blocked` to `Survey` variant; call `resolve_page_offset` for both `Survey` and `Next` |
| `src/priority/mod.rs` | Extract `resolve_page_offset()`; new `SURVEY_LIMIT_DEFAULT` constant; wire new args through `run_survey()` |
| `src/priority/surface.rs` | `survey()` accepts `hide_blocked: bool` |
| `src/priority/render.rs` | Extract `paginated()` helper; refactor `next_human` to use it; add pagination to `survey_human()`; remove "act" column from `SURVEY_COLS`; red styling via `ByValue` on `id` column for blocked rows |
| golden tests | regen after output changes |

### DRY extractions

**`paginated()` helper** — `src/priority/render.rs`

```rust
/// Slice rows into `(visible_page, optional_footer)` per limit/offset. Footer is
/// `None` when uncapped (`limit == 0`) or all rows fit. Single source for the
/// slice math + `limit == 0` guard — used by both `next_human` and `survey_human`.
fn paginated<T>(rows: &[T], limit: usize, offset: usize) -> (&[T], Option<String>) {
    let total = rows.len();
    let start = offset.min(total);
    let end = if limit == 0 { total } else { (start + limit).min(total) };
    let visible = rows.get(start..end).unwrap_or(&[]);
    let shown = visible.len();
    let footer = if limit != 0 && shown < total {
        Some(listing::format_truncation_notice(shown, total, offset, limit))
    } else {
        None
    };
    (visible, footer)
}
```

`next_human` refactored to call `paginated()` (keeping its `any_tagged` D7 gate
over the visible slice — computed AFTER the paginate call). `survey_human` calls
it identically.

**`resolve_page_offset()`** — `src/priority/mod.rs`

```rust
/// Validate `--page`/`--limit`/`--offset` and resolve to a concrete offset.
/// Single source for both `Survey` and `Next` dispatch in `cli.rs`.
pub(crate) fn resolve_page_offset(
    page: Option<usize>, limit: usize, offset: usize
) -> anyhow::Result<usize> {
    if page == Some(0) { anyhow::bail!("--page must be >= 1"); }
    if limit == 0 && page.is_some() { anyhow::bail!("--page requires a positive --limit"); }
    Ok(match page {
        Some(p) => (p - 1) * limit,
        None => offset,
    })
}
```

Used in `cli.rs` for both `Command::Survey` and `Command::Next` dispatch arms.

### Accepted duplication

CLI arg definitions (`--limit`, `--offset`, `--page`) are duplicated across the
`Survey` and `Next` enum variants — inescapable with clap derive (flatten only
works within a single struct variant, not across enum arms). Three fields each;
the extracted `resolve_page_offset` collapses the validation duplication.

### STD-001: zero new magic strings

Column names in `SURVEY_COLS`/`SURVEY_DEFAULT` are struct-bound (the
`Column.name` field drives `select_columns` lookup, and the `*_DEFAULT` slices
are named module constants). Removing the `"act"` column drops one entry; no
new string literals introduced.

The single new string literal is `"--hide-blocked"` on the clap arg — a clap
identifier, not a governance-significant value.

### Detailed changes

#### 1. `src/commands/cli.rs` — `Survey` variant

Add to the existing `Survey { all, format, json, path }`:

```rust
/// Max rows to show (default 20). Use 0 for uncapped.
#[arg(long, default_value_t = crate::priority::SURVEY_LIMIT_DEFAULT)]
limit: usize,

/// Skip first N rows (default 0).
#[arg(long, default_value_t = 0)]
offset: usize,

/// Page number (1-based; sugar over --offset). Mutually exclusive with --offset.
#[arg(long, conflicts_with = "offset")]
page: Option<usize>,

/// Exclude blocked items.
#[arg(long)]
hide_blocked: bool,
```

Dispatch calls `resolve_page_offset(page, limit, offset)?` then passes all args
through to `run_survey()`.

#### 2. `src/priority/mod.rs`

New constant:
```rust
pub(crate) const SURVEY_LIMIT_DEFAULT: usize = 20;
```

New signature (accepts pagination + hide-blocked):
```rust
pub(crate) fn run_survey(
    path: Option<PathBuf>,
    all: bool,
    hide_blocked: bool,
    format: Format,
    json: bool,
    render: RenderOpts,
    limit: usize,
    offset: usize,
) -> anyhow::Result<()> {
    let root = root(path)?;
    let rows = surface::survey(&root, all, hide_blocked)?;
    let out = if json || format == Format::Json {
        render::survey_json(&rows)?
    } else {
        render::survey_human(&rows, render, limit, offset)
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
}
```

#### 3. `src/priority/surface.rs` — `survey()`

Adds `hide_blocked: bool` parameter. After sorting, filters:
```rust
if hide_blocked {
    rows.retain(|d| d.act == Actionability::Actionable);
}
```

#### 4. `src/priority/render.rs`

**Remove the "act" column** from `SURVEY_COLS` (index 3 — the empty-header
BLOCKED badge column). `SURVEY_COLS` goes from 7 columns to 6:

```
id | kind | status | score | blocker | title
```

`SURVEY_DEFAULT` drops `"act"`.

**Red styling for blocked rows.** Change the `id` column paint from `Fixed(Cyan)`
to `ByValue` that returns `Red` when `act == Blocked`, `Cyan` otherwise:

```rust
Column {
    name: "id",
    header: "id",
    cell: |r| r.id.clone(),
    paint: ColumnPaint::ByValue(|r| {
        if matches!(r.act, Actionability::Blocked) {
            Some(DynColors::Ansi(Red))
        } else {
            Some(DynColors::Ansi(Cyan))
        }
    }),
},
```

**New `survey_human` signature** with pagination:
```rust
pub(crate) fn survey_human(
    rows: &[SurveyRow],
    opts: RenderOpts,
    limit: usize,
    offset: usize,
) -> String {
    if rows.is_empty() {
        return "(no eligible work)\n".to_string();
    }
    let (visible, footer) = paginated(rows, limit, offset);
    let sel: Vec<&Column<SurveyRow>> = SURVEY_COLS.iter().collect();
    let mut out = listing::render_columns(visible, &sel, opts);
    if let Some(f) = footer {
        out.push_str(&f);
    }
    out
}
```

### Visual change

**Before:**
```
id       kind  status           score  blocker  title
ISS-001  ISS   open                   1.0              My Issue
ISS-002  ISS   open     BLOCKED  0.5   ISS-001         Blocked Issue
```

**After:**
```
id       kind  status   score  blocker  title
ISS-001  ISS   open             1.0               My Issue
ISS-002  ISS   open     0.5     ISS-001           Blocked Issue
```
Where `ISS-002`'s `id` is rendered in **red** (blocked), `ISS-001` in cyan
(actionable). No separate BLOCKED column.

### What stays unchanged

- `next` command — untouched
- `survey --json` output — `actionability` field and `blockers` array carry the
  same information
- `Actionability::badge()` — kept (still used internally, not called from
  survey columns)
- `survey_for_map()` — web map server path unchanged (no pagination there)
- `SURVEY_DEFAULT` updates to match the new column set but the pattern is
  identical to existing `*_DEFAULT` constants

### Test impact

- Golden regen for any e2e test that captures `survey` table output (header
  line changes — "act" column gone, pagination footer may appear)
- New unit tests for `survey_human` pagination (VT-1 to VT-5 analogues in
  `render.rs` tests)
- `survey_for_map` tests unchanged
- Existing `next` golden tests unchanged (refactored internals, identical
  output)
