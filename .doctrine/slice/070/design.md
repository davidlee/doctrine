# SL-070 Design: CLI list UI coloring & improvements

## 1. `ColumnPaint::Fixed` widens to `DynColors`

### Current

```rust
pub(crate) enum ColumnPaint<R> {
    None,
    Fixed(owo_colors::AnsiColors),
    ByValue(fn(&R) -> Option<owo_colors::AnsiColors>),
    PerToken { split: fn(&R) -> Vec<String>, render: fn(&str) -> String },
}
```

### Target

```rust
pub(crate) enum ColumnPaint<R> {
    None,
    Fixed(owo_colors::DynColors),
    ByValue(fn(&R) -> Option<owo_colors::DynColors>),
    PerToken { split: fn(&R) -> Vec<String>, render: fn(&str) -> String },
    /// Alternating two-hue foreground per data-row index, for zebra-striping the
    /// title column. `[even_hue, odd_hue]`; data row 0 uses `even_hue`. The header
    /// row is excluded — `render_columns` builds it separately.
    Alternate([owo_colors::DynColors; 2]),
}
```

`paint_cell` gains a `row_index: usize` parameter:

```rust
fn paint_cell<R>(cell: &str, paint: &ColumnPaint<R>, row: &R, color: bool, row_index: usize) -> String
```

`Alternate` returns a `String` (not `Option<DynColors>`), so it needs an early return
before the `hue` match — same pattern as `PerToken`:

```rust
fn paint_cell<R>(cell: &str, paint: &ColumnPaint<R>, row: &R, color: bool, row_index: usize) -> String {
    use owo_colors::OwoColorize;
    if !color { return cell.to_string(); }
    // PerToken early return (existing, unchanged)
    if let ColumnPaint::PerToken { split, render } = paint { ... }
    // Alternate early return (new)
    if let ColumnPaint::Alternate([even, odd]) = paint {
        let hue = if row_index % 2 == 0 { *even } else { *odd };
        return cell.color(hue).to_string();
    }
    // Existing hue match — Alternate is unreachable here, kept for exhaustiveness:
    let hue = match paint {
        ColumnPaint::Fixed(c) => Some(*c),
        ColumnPaint::ByValue(f) => f(row),
        ColumnPaint::None | ColumnPaint::PerToken { .. } | ColumnPaint::Alternate(_) => None,
    };
    match hue { Some(c) => cell.color(c).to_string(), None => cell.to_string() }
}
```

`render_columns` passes the data-row index via `enumerate()`:

```rust
grid.extend(rows.iter().enumerate().map(|(i, r)| {
    cols.iter().map(|c| paint_cell(&(c.cell)(r), &c.paint, r, color, i)).collect()
}));
```

### Call-site impact (mechanical)

Every `Fixed(AnsiColors::Cyan)` becomes `Fixed(DynColors::Ansi(AnsiColors::Cyan))` across
10 column definitions in 8 files: `slice.rs`, `backlog.rs`, `memory.rs`, `governance.rs`,
`spec.rs` (SPEC_COLUMNS + REQ_COLUMNS), `review.rs`, `coverage_view.rs`, `knowledge.rs`,
`rec.rs`.

`status_hue` signature changes to return `Option<DynColors>`, wrapping its existing
`AnsiColors` values in `DynColors::Ansi(…)`. `CoverageRow::status_hue` (in `coverage_view.rs`)
likewise widens — it delegates to `listing::status_hue`, so only its return type annotation changes.

### Invariant

`DynColors::Ansi(AnsiColors::Cyan)` emits byte-identical ANSI to bare `AnsiColors::Cyan` —
`DynColors` delegates `fmt_ansi_fg` to the inner color. Existing goldens stay green.

---

## 2. Gruvbox tag palette

Replace `TAG_PALETTE` with 12 truecolour `Rgb` entries, ordered for distinguishability:

```rust
use owo_colors::Rgb;

const TAG_PALETTE: [Rgb; 12] = [
    Rgb(204,  36,  29), // red           #cc241d
    Rgb(152, 151,  26), // green         #98971a
    Rgb(215, 153,  33), // yellow        #d79921
    Rgb( 69, 133, 136), // blue          #458588
    Rgb(177,  98, 134), // purple        #b16286
    Rgb(104, 157, 106), // aqua          #689d6a
    Rgb(214,  93,  14), // orange        #d65d0e
    Rgb(250, 189,  47), // bright yellow #fabd2f
    Rgb(131, 165, 152), // bright blue   #83a598
    Rgb(211, 134, 155), // bright purple #d3869b
    Rgb(142, 192, 124), // bright aqua   #8ec07c
    Rgb(254, 128,  25), // bright orange #fe8019
];
```

`segment_hue` returns `Option<DynColors>`, wrapping the `Rgb` entry:

```rust
fn segment_hue(seg: &str) -> Option<owo_colors::DynColors> {
    if seg.is_empty() { return None; }
    let len = u32::try_from(TAG_PALETTE.len()).unwrap_or(1);
    let index = usize::try_from(stable_hash(seg) % len).unwrap_or(0);
    TAG_PALETTE.get(index).map(|c| DynColors::Rgb(c.0, c.1, c.2))
}
```

`paint_tag` resolves the colon-separator hue once:

```rust
let white = DynColors::Ansi(AnsiColors::White);
```

Test impact: the hash→index mapping is stable; `strip_ansi` proofs stay green unchanged.

---

## 3. Alternating title column hue (zebra striping)

Two subtle gruvbox-adjacent foreground hues, applied to the title column only:

```rust
const TITLE_EVEN: DynColors = DynColors::Rgb(235, 219, 178); // #ebdbb2
const TITLE_ODD:  DynColors = DynColors::Rgb(213, 196, 161); // #d5c4a1
```

Each kind's title column changes `paint` from `None` to:

```rust
paint: ColumnPaint::Alternate([TITLE_EVEN, TITLE_ODD]),
```

The header row is bold (unchanged) and never alternated. Data row 0 is even, row 1 odd, etc.
No other column carries `Alternate` — title only.

---

## 4. Per-value column colour maps

Three pure lookup functions in `src/listing.rs`, wired via `ColumnPaint::ByValue`:

### 4a. Backlog kind hue

```rust
fn backlog_kind_hue(kind: &str) -> Option<DynColors> {
    match kind {
        "issue"       => Some(DynColors::Ansi(AnsiColors::Red)),
        "improvement" => Some(DynColors::Ansi(AnsiColors::Green)),
        "chore"       => Some(DynColors::Ansi(AnsiColors::Yellow)),
        "risk"        => Some(DynColors::Ansi(AnsiColors::Magenta)),
        "idea"        => Some(DynColors::Ansi(AnsiColors::Blue)),
        _             => None,
    }
}
```

Wired on `BL_COLUMNS` kind entry: `paint: ColumnPaint::ByValue(|i| backlog_kind_hue(i.kind.as_str()))`.

### 4b. Memory type hue

```rust
fn memory_type_hue(kind: &str) -> Option<DynColors> {
    match kind {
        "concept"  => Some(DynColors::Ansi(AnsiColors::Cyan)),
        "fact"     => Some(DynColors::Ansi(AnsiColors::Green)),
        "pattern"  => Some(DynColors::Ansi(AnsiColors::Magenta)),
        "signpost" => Some(DynColors::Ansi(AnsiColors::Blue)),
        "system"   => Some(DynColors::Ansi(AnsiColors::Yellow)),
        "thread"   => Some(DynColors::Ansi(AnsiColors::Red)),
        _          => None,
    }
}
```

### 4c. Memory trust hue

```rust
fn trust_hue(trust: &str) -> Option<DynColors> {
    match trust {
        "high"   => Some(DynColors::Ansi(AnsiColors::Green)),
        "medium" => Some(DynColors::Ansi(AnsiColors::Yellow)),
        "low"    => Some(DynColors::Ansi(AnsiColors::Red)),
        _        => None,
    }
}
```

All three live in `src/listing.rs` alongside `status_hue`. Call sites reference them via `listing::backlog_kind_hue`, etc.

---

## Code impact summary

| File | Change |
|------|--------|
| `src/listing.rs` | `ColumnPaint` enum: `Fixed` → `DynColors`, add `Alternate` variant; `paint_cell` adds `row_index` + `Alternate` early return; `render_columns` passes row index via `enumerate()`; `status_hue` → `Option<DynColors>`; `TAG_PALETTE` → gruvbox `Rgb`; `segment_hue` → `Option<DynColors>`; add `backlog_kind_hue`, `memory_type_hue`, `trust_hue`; `paint_tag` updates white resolve; 6 test `paint_cell` call sites add `row_index` |
| `src/slice.rs` | `SLICE_COLUMNS`: id paint → `DynColors::Ansi(Cyan)`, title paint → `Alternate([…])` |
| `src/backlog.rs` | `BL_COLUMNS`: id paint → `DynColors::Ansi(Cyan)`, kind paint → `ByValue(backlog_kind_hue)`, title paint → `Alternate([…])` |
| `src/memory.rs` | `MEMORY_COLUMNS`: id paint → `DynColors::Ansi(Cyan)`, type paint → `ByValue(memory_type_hue)`, trust paint → `ByValue(trust_hue)`, title paint → `Alternate([…])` |
| `src/governance.rs` | `GOV_COLUMNS`: id paint → `DynColors::Ansi(Cyan)`, title paint → `Alternate([…])` |
| `src/spec.rs` | `SPEC_COLUMNS`: id paint → `DynColors::Ansi(Cyan)`, title paint → `Alternate([…])`; `REQ_COLUMNS`: id paint → `DynColors::Ansi(Cyan)` (no title column → no Alternate) |
| `src/review.rs` | `REVIEW_COLUMNS`: id paint → `DynColors::Ansi(Cyan)`, title paint → `Alternate([…])` |
| `src/coverage_view.rs` | `COVERAGE_COLUMNS`: id paint → `DynColors::Ansi(Cyan)`; `CoverageRow::status_hue` return → `Option<DynColors>` (no title column → no Alternate) |
| `src/knowledge.rs` | `KN_COLUMNS`: id paint → `DynColors::Ansi(Cyan)`, title paint → `Alternate([…])` |
| `src/rec.rs` | `REC_COLUMNS`: id paint → `DynColors::Ansi(Cyan)`, title paint → `Alternate([…])` |

## Verification impact

- Extend VT-2 strip proof: coloured table → strip ANSI → byte-identical plain table
- VT-3: `NO_COLOR` / pipe suppresses all colour (existing, no regression)
- New tests: `paint_cell` with `Alternate` at row 0/1; `backlog_kind_hue` known+unknown; `memory_type_hue` known+unknown; `trust_hue` known+unknown
- New test: `paint_tag` with gruvbox palette produces valid ANSI, strips to matching plain
- New test: `render_columns` with `Alternate` applies zebra on title, not on other columns
- `paint_cell` call site tests in `listing.rs` re-test with `row_index` parameter

## Design decisions

| # | Decision | Rationale |
|---|----------|-----------|
| D1 | Widen `Fixed` to `DynColors` rather than add separate `FixedRgb` variant | Fewest code paths; `DynColors` already subsumes all color types; single `paint_cell` arm |
| D2 | Alternating foreground on title column, not full-row background | Simpler; avoids comfy-table padding/wrapping interaction; title is the widest column |
| D3 | Gruvbox `Rgb` truecolour for tags, `AnsiColors` for kind/type/trust | Tags need maximum distinguishability (many values, FNV-folded); kind/type/trust are small closed sets where 4-bit ANSI suffices |
| D4 | All hue maps live in `listing.rs`, not per-kind files | They're pure, kind-blind lookup tables, same as `status_hue`; per-kind files just wire the `ByValue` reference |
| D5 | `Alternate([DynColors; 2])` as a `ColumnPaint` variant, not a `RenderOpts` bool | Column authors declare which column stripes; avoids implicit "which column is title" logic in the spine |

## Open questions

None remaining — all four design sections accepted.
