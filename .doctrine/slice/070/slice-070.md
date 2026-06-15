# CLI list UI coloring & improvements

## Context

The CLI `list` surfaces share the `src/listing.rs` spine (SL-025). The spine
already carries colour support: status hues, a fixed id-column colour (Cyan),
bold headers, and a 10-colour ANSI tag palette. This slice extends that colour
surface in three dimensions: row alternation for visual separation, a richer
gruvbox-truecolour tag palette, and per-value colouring of additional columns
(kind, type, trust) that are currently plain.

## Scope & Objectives

### 1. Alternating row colour (zebra striping)

Add a subtle background colour alternation so the eye can track which row a
column belongs to when columns are wide or wrapped. The alternation is a
foreground OR background hue toggle on the title column (the widest field) —
applied in `render_columns` row by row. Gated on `RenderOpts::color` like all
other colour. No extra `RenderOpts` bool needed; alternating is always on when
`color` is true.

**Design decision (to be resolved in /design):** foreground hue alternation on
the title column vs full-row background ANSI. Foreground is simpler and avoids
comfy-table padding interactions; background may conflict with wrapped cells.

### 2. Gruvbox tag palette

Replace `TAG_PALETTE` (10 `AnsiColors` entries) with a gruvbox-inspired palette
using `owo_colors::Rgb` truecolour. This requires widening `ColumnPaint::Fixed`
from `AnsiColors` to `DynColors` so the type system permits truecolour at the
column-paint seam.

Gruvbox palette (ordered for maximum distinguishability):
- `#cc241d` (red), `#98971a` (green), `#d79921` (yellow), `#458588` (blue),
  `#b16286` (purple), `#689d6a` (aqua), `#d65d0e` (orange),
  `#fabd2f` (bright yellow), `#83a598` (bright blue), `#d3869b` (bright purple),
  `#8ec07c` (bright aqua), `#fe8019` (bright orange)

The FNV-1a fold (`segment_hue`) stays; only the palette entries change.

### 3. Colour for backlog `kind` column

Map each backlog item kind to a distinct, stable hue via `ColumnPaint::ByValue`:

| kind        | hue        |
|-------------|------------|
| issue       | Red        |
| improvement | Green      |
| chore       | Yellow     |
| risk        | Magenta    |
| idea        | Blue       |

### 4. Colour for memory `type` column

Map each memory kind to a distinct, stable hue:

| type      | hue     |
|-----------|---------|
| concept   | Cyan    |
| fact      | Green   |
| pattern   | Magenta |
| signpost  | Blue    |
| system    | Yellow  |
| thread    | Red     |

### 5. Colour for memory `trust` column

Map trust level to a hue signalling severity/confidence:

| trust  | hue      |
|--------|----------|
| high   | Green    |
| medium | Yellow   |
| low    | Red      |

## Non-Goals

- Colour for governance `kind` column (adr/policy/standard share a common
  layout; their kind is implicit in the command verb — `adr list` is already
  self-identifying).
- Colour for spec `kind` column (product vs tech — same reasoning; already clear
  from `PRD-`/`SPEC-` prefix in id).
- Custom colour themes beyond the gruvbox palette.
- Changing table layout, separator characters, or padding.
- Per-user colour configuration (colours are hard-coded, like the current design).

## Affected surface

| file            | what changes                                           |
|-----------------|--------------------------------------------------------|
| `src/listing.rs`| `ColumnPaint::Fixed` → `DynColors`; `TAG_PALETTE` → gruvbox Rgb; `paint_tag` → use new palette; add `kind_hue`/`trust_hue` maps; `render_columns` → alternating row hue on title; `paint_cell` → accept `DynColors` |
| `src/backlog.rs`| `BL_COLUMNS` → add `ColumnPaint::ByValue` on kind column |
| `src/memory.rs` | `MEMORY_COLUMNS` → add `ColumnPaint::ByValue` on type and trust columns |

## Risks

- **Truecolour terminal support**: `\x1b[38;2;R;G;Bm` sequences are supported
  in virtually all modern terminals (≥2015), but fallback to 4-bit ANSI on
  ancient/vt100-only terminals will show degraded colours. Acceptable — the
  current 4-bit palette is already the worst case.
- **Alternating row + column colour interaction**: alternating title foreground
  must not clash with the title's own hue (title is currently uncoloured).
- **`DynColors` widening**: every existing `ColumnPaint::Fixed(AnsiColors::Cyan)`
  call site needs updating to `Fixed(DynColors::Ansi(AnsiColors::Cyan))` — a
  mechanical change across 6 files but a wide diff.

## Verification / closure intent

- Every coloured column renders with the expected hue on a real terminal
- `--json` output is byte-identical before and after (colours are table-only)
- `NO_COLOR` and pipe/redirect suppress all colour (existing VT-3/4 hold)
- Stripping ANSI from coloured table output reproduces the plain layout
  (existing VT-2 strip proof extended)
- Round-trip: `doctrine slice list`, `backlog list`, `memory list`, `spec list`,
  `review list`, `adr list` all render with correct new colours
- `cargo test` green; `just check` zero clippy warnings

## Follow-Ups

- Consider colour for `review` facet and `spec` kind once feedback on the core
  columns lands.
- Truecolour → 4-bit ANSI degradation path if demanded (low priority — modern
  terminals are universal).
