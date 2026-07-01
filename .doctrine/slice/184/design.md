# SL-184 Design: Rename memory find → search + adopt shared listing spine

## 1. Rename surface

All verb references shift uniformly:

| Current | Target | Location |
|---|---|---|
| `MemoryCommand::Find` | `MemoryCommand::Search` | `src/memory.rs` variant def + dispatch arm |
| `FindRetrieveArgs` | keep (B); add `--columns` | `src/memory.rs` |
| `run_find` | `run_search` | `src/retrieve.rs` |
| `format_find_table` | removed (replaced by `listing::render_columns`) | `src/retrieve.rs` |
| `format_find_json` | `format_search_json` | `src/retrieve.rs` — typed row struct renamed |
| `MemoryFindRow` | `MemorySearchRow` | `src/retrieve.rs` |
| `find_for_mcp` | `search_for_mcp` | `src/retrieve.rs` |
| `FindForMcp` | `SearchForMcp` | `src/retrieve.rs` |
| MCP tool `memory_find` | `memory_search` | `src/mcp_server/tools.rs` def + handler + onboard table + sibling tool description prose refs |
| MCP sibling tool description `memory_find` refs | `memory_search` | `memory_retrieve`, `memory_show`, `memory_list` tool descriptions |
| MCP golden `kind: "memory_find"` | `"memory_search"` | `src/mcp_server/tools.rs` handler envelope |
| E2E test `memory_find` | `memory_search` | `tests/e2e_mcp_server.rs` |
| E2E test fn name | rename | `tests/e2e_mcp_server.rs`, `tests/e2e_memory_sync.rs` |
| `each_new_shipped_memory_finds_by_scoped_search…` | rename | `tests/e2e_memory_sync.rs` |

### Deprecation alias

`#[command(alias = "find")]` on `MemoryCommand::Search` — clap's hidden alias, no stderr
notice (adversarial finding: the doc comment on the variant should say `Search` not `Find`).
The `Find` variant is removed entirely (no separate redirect dead code).

This overrides IMP-220's original proposal of a stderr deprecation notice — a silent
alias is simpler and avoids noise for users who type `find` from habit. IMP-220 should
be updated to reflect this resolution.

## 2. Shared listing spine

`Candidate<'a>` is the row type. 15 columns, 8 default — exactly matching
`format_find_table`'s current output order.

### Column definitions (`src/retrieve.rs`)

`Candidate<'a>` borrows `Memory`, so a `const` would pin `'a` to `'static`. Instead,
a function returns the array — the closures are non-capturing fn pointers, so
the array is static data regardless (adversarial finding #1).

```rust
fn search_columns() -> [Column<Candidate<'_>>; 15] {
    [
        Column { name: "uid",       header: "uid",       cell: |c| c.memory.uid.clone(),
            paint: ColumnPaint::Fixed(DynColors::Ansi(AnsiColors::Cyan)) },
    Column { name: "type",      header: "type",      cell: |c| c.memory.kind.as_str().to_owned(),
        paint: ColumnPaint::ByValue(|c| listing::memory_type_hue(c.memory.kind.as_str())) },
    Column { name: "status",    header: "status",    cell: |c| c.memory.status.as_str().to_owned(),
        paint: ColumnPaint::ByValue(|c| listing::status_hue(c.memory.status.as_str())) },
    Column { name: "staleness", header: "staleness", cell: |c| c.staleness.label().to_owned(),
        paint: ColumnPaint::None },
    Column { name: "trust",     header: "trust",     cell: |c| scrub_line(&c.memory.trust_level),
        paint: ColumnPaint::ByValue(|c| listing::trust_hue(&c.memory.trust_level)) },
    Column { name: "severity",  header: "severity",  cell: |c| scrub_line(&c.memory.severity),
        paint: ColumnPaint::None },
    Column { name: "spec",      header: "spec",      cell: |c| c.scope_match.map_or("-", |s| s.dim.label()).to_owned(),
        paint: ColumnPaint::None },
    Column { name: "title",     header: "title",     cell: |c| scrub_line(&c.memory.title),
        paint: ColumnPaint::Alternate([TITLE_EVEN, TITLE_ODD]) },
    Column { name: "key",       header: "key",       cell: |c| c.memory.key.as_deref().unwrap_or(ABSENT_CELL).to_owned(),
        paint: ColumnPaint::None },
    Column { name: "created",   header: "created",   cell: |c| c.memory.created.clone(),
        paint: ColumnPaint::None },
    Column { name: "updated",   header: "updated",   cell: |c| c.memory.updated.clone(),
        paint: ColumnPaint::None },
    Column { name: "weight",    header: "weight",    cell: |c| c.memory.weight.to_string(),
        paint: ColumnPaint::None },
    Column { name: "verification", header: "verification", cell: |c| c.memory.verification_state.clone(),
        paint: ColumnPaint::None },
    Column { name: "lifespan",  header: "lifespan",  cell: |c| c.memory.lifespan.map_or(ABSENT_CELL.to_owned(), |l| l.to_string()),
        paint: ColumnPaint::None },
    Column { name: "reviewed",  header: "reviewed",  cell: |c| c.memory.reviewed.clone(),
        paint: ColumnPaint::None },
    ]
}

const SEARCH_DEFAULT: &[&str] = &["uid", "type", "status", "staleness", "trust", "severity", "spec", "title"];
```

### `--columns` flag

Added to `FindRetrieveArgs` in `src/memory.rs`:
```rust
#[arg(long, help = "Column projection for search table output (ignored by retrieve)")]
pub(crate) columns: Option<Vec<String>>,
```

Only the search path reads it. The `--columns` flag also appears in `retrieve --help`
(shared struct), but retrieve ignores it — acceptable per scope (adversarial finding #3).

Dispatch in `run_search` (adversarial finding #2):
```rust
let columns = args.columns.as_ref();
let sel = listing::select_columns(&search_columns(), SEARCH_DEFAULT, columns)?;
// Slice by index range (not collect refs) so render_columns gets &[Candidate]:
let end = ranked.len().min(offset + limit.unwrap_or(usize::MAX));
let visible = &ranked[offset..end];
let body = listing::render_columns(visible, &sel, RenderOpts { color, term_width: None });

// JSON still uses &[&Candidate] — collect refs from the same slice:
let json_body = format_search_json(&visible.iter().collect::<Vec<_>>());
```

`color: bool` added as a parameter to `run_search`, sourced from
`crate::tty::stdout_color_enabled()` at the dispatch site (adversarial finding #4).

### JSON output

Keeps typed `MemorySearchRow` / `format_search_json` — same shape, renamed struct.
Envelope kind string: `"memory_search"`.

### Pagination

`listing::format_truncation_notice` already used — no change.

## 3. MCP mapping

Tool def: name `"memory_find"` → `"memory_search"`, description updated.
Handler: match arm, inner envelope `kind`, calls `search_for_mcp`.
Onboard table: `|doctrine memory search|memory_search||`.
Sibling tool descriptions: `memory_retrieve`, `memory_show`, `memory_list` prose
references to `memory_find` → `memory_search`.

## 4. Code impact summary

### Diffs by file

| File | Changes |
|---|---|
| `src/memory.rs` | Rename `MemoryCommand::Find` → `Search` with `alias = "find"`. Add `columns` field to `FindRetrieveArgs`. Update dispatch arm. |
| `src/retrieve.rs` | `run_find` → `run_search`. Remove `format_find_table`. Add `SEARCH_COLUMNS` + `SEARCH_DEFAULT`. `format_find_json` → `format_search_json`. `MemoryFindRow` → `MemorySearchRow`. `find_for_mcp` → `search_for_mcp`. `FindForMcp` → `SearchForMcp`. Wire `listing::render_columns` in `run_search`. |
| `src/mcp_server/tools.rs` | Tool def + handler + onboard table + sibling tool description prose rename. |
| `tests/e2e_mcp_server.rs` | Tool name strings, test fn names, golden values. |
| `tests/e2e_memory_sync.rs` | Test fn name. |
| `src/commands/guard.rs` | Update `MemoryCommand::Find` match arm ref. |
| `src/main.rs` | Update test refs to `MemoryCommand::Find`. |

### Verification alignment (adversarial finding #5)

- `format_find_table` tests (`format_find_row_carries_full_uid…`, `format_find_scrubs_a_newline_title`,
  `format_find_empty_is_empty_string`) are rewritten or removed: the function is gone.
  Replace with column-definition assertions or simplified `output.contains(...)` tests.
- `writer_capture_run_find` asserts `output.contains("Writer capture test")` —
  title text still appears inside comfy-table cells → passes.
- `run_find_rejects_limit_zero` and `find_for_mcp_returns_rows_with_key_field` rename
  and pass identically (surface logic unchanged).
- Column-projection tests for `search_columns()` + `--columns` flag.
- Table output changes from hand-rolled to comfy-table — byte-goldens will shift
  (alignment chars, header row, colours). Update golden assertions.
- MCP E2E tests: tool name + kind string.

### Design decisions

1. **`FindRetrieveArgs` keeps its name** — the rename is work-surface noise (4 refs).
2. **Silent clap alias** — no deprecation stderr notice; simplest path.
3. **`Candidate` as row type** — the pre-built Candidate already aggregates all
   derived signals (staleness, scope_match, lexical, exact_key) into one ref.
4. **15 columns, 8 default** — the non-default columns (key, created, updated,
   weight, verification, lifespan, reviewed) are available via `--columns` for
   users who want a richer view.
