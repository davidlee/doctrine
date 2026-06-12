# SL-053 Design — Terminal output polish: comfy-table listings + owo_colors

## 1. Current vs target behaviour

**Current.** Every `*list` surface renders through one shared seam in
`src/listing.rs`:

- `render_columns<R>(rows, cols)` bakes a header row + one cell-row per item into
  a `Vec<Vec<String>>` grid and hands it to `render_table`.
- `render_table(grid)` hand-rolls the layout: per-column width via
  `chars().count()`, a two-space `COL_GAP`, every column left-padded to its
  widest cell **except the last** (no trailing whitespace), terminated by a
  single `\n`. Empty input → `""` (header suppressed).
- Output is monochrome ASCII. Eleven kinds ride this seam (meta, coverage_view,
  rec, governance, priority/render, backlog, spec, slice, memory, review, +
  listing's own tests).

**Target.**

- `render_table` delegates **all** layout and width measurement to
  **comfy-table** — minimalist style: inner vertical `│` column separators only,
  no outer frame, no horizontal/header rules. The hand-rolled width/pad maths
  (`chars().count()`, `COL_GAP`, the pad loop, the last-column-unpadded special
  case) is **deleted**; comfy-table becomes the sole layout/measurement
  authority. This is the explicit motivation for adopting the dependency: avoid
  hand-rolling layout and measurement.
- Listing surfaces gain colour via **owo_colors**: bold headers, fixed-hue id
  columns, status coloured by value. Colour is emitted **only** to a
  colour-capable TTY; piped output is byte-for-byte plain.

## 2. Architecture — the pure/impure boundary

The pure/imperative split (slices-spec § Architecture; CLAUDE.md) forbids env,
tty, clock, rng, git, or disk reads in the pure layer. Colour *capability*
detection reads `NO_COLOR` and isatty — both impure. Therefore capability is
resolved in the thin command shell and **injected** as a `bool` into the pure
render layer (the established date/uid injection pattern).

```
shell (impure)                       pure layer (src/listing.rs)
──────────────                       ───────────────────────────
tty::stdout_color_enabled() ─bool──▶ render_columns(rows, cols, color)
  NO_COLOR set?  (var_os)              ├─ header cells → bold      (when color)
  io::stdout().is_terminal()          ├─ data cells  → ColumnPaint (when color)
                                       └─ render_table(grid) ──▶ comfy-table layout
```

**Doctrinal load-bearing decision (D3):** colour uses owo_colors' *unconditional*
colorize methods (`.green()`, `.bold()`, …), gated by the injected `color` bool.
We do **not** use `owo_colors::if_supports_color` — it reads env + tty at
apply-time, which would push impurity into the pure render layer. The bool is the
single authority; capability detection happens once, in the shell.

Responsibility split stays clean:

- `render_columns` — semantics: header, per-column paint, colour gating. Pure.
- `render_table` — layout only: comfy-table grid + separators + trailing `\n`.
  Header-agnostic and colour-agnostic (it lays out whatever strings it is given).

## 3. Data shapes

```rust
// src/listing.rs (pure)

/// How a column's data cells are coloured when colour is enabled.
pub(crate) enum ColumnPaint {
    None,
    Fixed(owo_colors::AnsiColors),
    ByValue(fn(&str) -> Option<owo_colors::AnsiColors>),
}

pub(crate) struct Column<R> {
    pub(crate) name: &'static str,
    pub(crate) header: &'static str,
    pub(crate) cell: fn(&R) -> String,
    pub(crate) paint: ColumnPaint,          // NEW
}

pub(crate) fn render_columns<R>(
    rows: &[R],
    cols: &[&Column<R>],
    color: bool,
) -> String;

// layout only — comfy-table; ContentArrangement::Disabled; re-appends '\n'
fn render_table(grid: &[Vec<String>]) -> String;
```

- Header cells are bolded in `render_columns` when `color` is true; the header
  row is otherwise an ordinary grid row (minimalist style has no header rule, so
  `render_table` needs no header concept).
- Data cells: `render_columns` consults each column's `ColumnPaint` and wraps the
  raw `cell` output via owo when `color` is true; when false (or `None`), the raw
  string passes through unchanged.

**Shared status hue (no per-kind duplication).** One function maps a status
token to a hue, reused by every kind's status column via
`ColumnPaint::ByValue(status_hue)`:

```rust
fn status_hue(s: &str) -> Option<owo_colors::AnsiColors> {
    match s {
        "done" | "active" | "accepted" | "required" => Some(Green),
        "in_progress" | "started" | "design" | "plan" | "audit" | "reconcile" => Some(Yellow),
        "blocked" | "abandoned" | "contested" => Some(Red),
        _ => None, // proposed/open/ready/… → default colour
    }
}
```

The exact token→hue table is finalised at implementation against the live status
vocabularies (`doctrine <kind> --help` / the status enums); the design fixes the
*mechanism* (one shared `ByValue` fn), not an authoritative token list.

## 4. Colour capability resolution (impure shell)

New thin module `src/tty.rs`:

```rust
pub(crate) fn stdout_color_enabled() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;                      // var_os — repo bans std::env::var
    }
    std::io::IsTerminal::is_terminal(&std::io::stdout())   // stdlib, no dep
}
```

Each of the eleven `run_list` (and the few local `render_default`-style)
call sites resolves the bool and passes it into `render_columns`. ~11 one-line
edits — mechanical, keeps the pure layer pure.

`NO_COLOR` precedence over isatty follows the `no-color.org` convention: presence
(even empty) disables colour.

## 5. Determinism — the correctness crux

Piped output must be byte-stable and terminal-size-independent, or the black-box
goldens flake.

- `Table::set_content_arrangement(ContentArrangement::Disabled)` and never set a
  table width → comfy-table never queries the terminal; column widths derive only
  from content. (comfy-table's default `Dynamic` arrangement consults terminal
  width via crossterm — that path is disabled.)
- comfy-table dependency: `default-features = false` to drop the crossterm/tty
  width machinery; enable the **`custom_styling`** feature so display-width
  measurement strips ANSI (colour applied upstream in `render_columns` must not
  desync alignment). Exact feature set is verified at execute against the
  resolved crate version — if `custom_styling` cannot be enabled without
  crossterm, re-open here rather than forcing it (ASM-1).
- `render_table` re-appends the trailing `\n` (comfy-table's `to_string()` omits
  it; backlog.rs:1045 documents callers printing the result verbatim, relying on
  the seam's own newline). Empty grid → `""` preserved.

## 6. Verification alignment

New / changed evidence:

- **Pure colour tests** (in `listing.rs`): `render_columns(rows, cols, true)`
  contains ANSI escapes for painted columns + bold header; `(.., false)` contains
  zero ANSI. A width/alignment test with a painted column proves comfy-table's
  ANSI-aware measurement keeps separators aligned (no drift from the escapes).
- **`tty.rs` test**: `NO_COLOR` present ⇒ `false` (env injected per-test; the
  isatty branch is exercised only indirectly, documented as such).
- **Golden re-baseline**: the shared-surface format change (separators) trips
  `tests/e2e_list_conformance.rs` *by design* — it exists to force acknowledgment
  of any `listing.rs` format change at the shared surface — plus the per-verb
  goldens (`e2e_list_columns_golden`, `e2e_adr_cli_golden`,
  `e2e_coverage_view_golden`, `e2e_inspect_golden`, `e2e_priority_golden`,
  `e2e_standard_cli_golden`). These run against piped output ⇒ colour-free; they
  re-baseline to the `│`-separated plain shape only.
- **RSK-1 mitigation**: the golden re-baseline lands in a commit **separate** from
  any logic change, so a pure shape-churn diff cannot mask a content regression.
- `just check` green; `cargo clippy` zero warnings.

## 7. Decisions & residual open questions

- **D1** — Adopt comfy-table as the sole layout/measurement authority (user
  decision; motivation: avoid hand-rolling layout/measurement maths).
- **D2** — Rich per-column paint: `Column<R>` gains a `ColumnPaint` field; status
  coloured by value via a shared `status_hue` (user decision).
- **D3** — Inject the colour bool from the impure shell; no `if_supports_color`
  in the pure layer (doctrine: pure/imperative split).
- **D4** — OQ-1 (memory-list seam) is void: `memory list` already renders through
  `listing::render_columns` (memory.rs:1312). Colouring it is free. IMP-017 /
  IMP-018 concern the `--columns` flag, not the renderer, and stay out of scope.
- **D5** — `--color=auto|always|never` flag is **out of scope**; auto-detection
  (`NO_COLOR` + isatty) only. Captured as a follow-up.

Residual: none blocking. ASM-1 (comfy-table can express the minimalist style and
ANSI-aware width without re-introducing terminal-width dependence) is verified at
execute; failure re-opens D1.

## 8. Code impact summary

| Path | Change |
|---|---|
| `Cargo.toml` (workspace + bin) | add `comfy-table` (`default-features=false`, `custom_styling`) + `owo_colors` to `[workspace.dependencies]` and the bin `[dependencies]` |
| `src/listing.rs` | `render_table` reimplemented over comfy-table (layout only); `render_columns` gains `color` param + paint application; `ColumnPaint` enum; `status_hue`; hand-rolled width/pad maths deleted; pure colour + alignment tests |
| `src/tty.rs` (new) | `stdout_color_enabled()` — impure capability resolution |
| 11 `run_list` call sites | resolve + pass the `color` bool into `render_columns` |
| each kind's `Column` literals | id columns gain `Fixed(hue)`; status columns gain `ByValue(status_hue)`; others `None` |
| goldens (listed §6) | re-baseline to `│`-separated plain shape (separate commit) |

## 9. Phasing intent (for /plan)

Provisional, to keep RSK-1 clean:

1. **Renderer swap** — comfy-table behind `render_table` (no colour yet),
   minimalist separators, determinism guards; re-baseline goldens. Pure shape
   change, isolated.
2. **Colour seam** — `tty.rs`, `color` param, `ColumnPaint`, `status_hue`, paint
   the column literals, pure colour tests. Goldens stay green (piped ⇒ plain).
3. **Follow-ups capture** — backlog items for ad-hoc `writeln!` surface colouring
   and the `--color` flag.
