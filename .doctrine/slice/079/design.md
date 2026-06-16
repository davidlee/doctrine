# SL-079 Design — Finish the CLI colour story: deferred surfaces + --color flag

## 1. Current vs target behaviour

**Current** (post-SL-053):

| Surface | Mechanism | Colour? | `--color` flag? |
|---|---|---|---|
| List tables (backlog, rec, governance, slice, memory, review, spec, coverage_view) | `render_columns` → `render_table` | Yes (auto-detect) | No |
| Priority tables (`survey`, `next`) | Hand-built grid → `render_table` directly | No | No |
| Status-bearing `writeln!` lines (adr, policy, standard, knowledge, revision) | `writeln!(io::stdout(), …)` bare | No | No |

**Target:**

| Surface | Mechanism | Colour? | `--color` flag? |
|---|---|---|---|
| List tables (unchanged) | `render_columns` → `render_table` | Yes | Yes — overrides auto-detect |
| Priority tables | `render_columns` → `render_table` | Yes | Yes |
| Status-bearing `writeln!` lines | Point-colour via injected `bool`, reusing `status_hue` | Yes | Yes |

All surfaces resolve colour from a single `--color=auto|always|never` flag
(precedence: `never` > `always` > auto-detect, where auto-detect = `NO_COLOR` >
isatty). The column model validates default-set membership at construction time
(debug-assertion; release backstop exists via the existing `pick()` error).

## 2. Architecture — the pure/impure boundary

No new impurities. The existing pattern is preserved:

```
shell (impure)                                 pure layer
──────────────                                 ──────────
clap::ColorChoice (--color flag) ─┐
tty::stdout_color_enabled()       ├──bool──▶ render_columns(rows, cols, opts)
  (NO_COLOR + isatty)             │             ├─ priority tables (NEW)
                                  │             └─ list tables (unchanged)
                                  │
                                  └──bool──▶ status_colored(status, color)
                                                ├─ adr, policy, standard,
                                                │  knowledge, revision (NEW)
                                                └─ reuses status_hue
```

`tty::resolve_color(mode: ColorChoice) -> bool` is the new shell-side resolver
that merges the flag with auto-detection. The pure layer sees only a `bool` —
unchanged from SL-053.

`status_colored` is a pure helper in `listing.rs` (sibling to `status_hue`):
takes a `status: &str` + `color: bool`, returns the status string optionally
wrapped in owo ANSI via `status_hue`. Pure — both inputs injected.

**Load-bearing invariants (carried forward from SL-053):**
- D3: owo's unconditional colorize methods, gated on the injected `bool`. Never
  `if_supports_color`.
- D6: `render_table` calls `force_no_tty()` before `to_string()` — unchanged.
- D7: table line shape (no leading/trailing whitespace, `│` separators) —
  unchanged.

## 3. Data shapes

### 3a. `--color` resolution (new, `src/tty.rs`)

```rust
use clap::ColorChoice;

/// Resolve the effective colour bool from the CLI flag + auto-detection.
/// `Never` beats `NO_COLOR` beats isatty; `Always` beats non-TTY.
/// The single shell-side authority for colour capability.
pub(crate) fn resolve_color(mode: ColorChoice) -> bool {
    match mode {
        ColorChoice::Never  => false,
        ColorChoice::Always => true,
        ColorChoice::Auto   => stdout_color_enabled(),
    }
}
```

`stdout_color_enabled()` is unchanged. The new function is a thin composable
wrapper — the pure `color_enabled(no_color, is_tty)` beneath it stays untouched
(no new test burden on the pure seam).

### 3b. Status-line colour helper (new, `src/listing.rs`)

```rust
/// Wrap a status token in ANSI colour via the shared [`status_hue`] map, gated
/// on the injected `color` bool. Pure — both inputs injected. When `color` is
/// false, returns the status unchanged (zero ANSI). When `color` is true and the
/// status is unmapped, returns the status unchanged (no colour).
pub(crate) fn status_colored(status: &str, color: bool) -> String {
    use owo_colors::OwoColorize;
    if !color {
        return status.to_string();
    }
    match status_hue(status) {
        Some(hue) => status.color(hue).to_string(),
        None => status.to_string(),
    }
}
```

Lives in `src/listing.rs` next to `status_hue` — the hue map is its natural
dependency. The impure shell (`tty.rs`) does not import it; callers in the
command layer import both `tty::resolve_color` and `listing::status_colored`.

### 3c. Priority `Column` arrays (new, `src/priority/render.rs`)

**Survey (7 columns):**

```rust
use crate::listing::{Column, ColumnPaint, status_hue, TITLE_EVEN, TITLE_ODD};
use owo_colors::{DynColors, AnsiColors::Cyan};

const SURVEY_COLS: &[Column<SurveyRow>] = &[
    Column { name: "id",      header: "id",
             cell: |r| r.id.clone(),
             paint: ColumnPaint::Fixed(DynColors::Ansi(Cyan)) },
    Column { name: "kind",    header: "kind",
             cell: |r| r.kind.clone(),
             paint: ColumnPaint::None },
    Column { name: "status",  header: "status",
             cell: |r| r.status.clone(),
             paint: ColumnPaint::ByValue(|r| status_hue(&r.status)) },
    Column { name: "blocked", header: "",
             cell: |r| r.act.badge().to_string(),
             paint: ColumnPaint::ByValue(|r| status_hue(r.act.token())) },
    Column { name: "cons",    header: "cons",
             cell: |r| r.consequence.to_string(),
             paint: ColumnPaint::None },
    Column { name: "blocker", header: "blocker",
             cell: |r| r.blockers.first().cloned().unwrap_or_default(),
             paint: ColumnPaint::None },
    Column { name: "title",   header: "title",
             cell: |r| r.title.clone(),
             paint: ColumnPaint::Alternate([TITLE_EVEN, TITLE_ODD]) },
];
```

- `id` — Cyan fixed hue (distinct from per-kind id hues on kind-specific
  surfaces; priority rows are mixed-kind, one hue is neutral).
- `status` — reuses `status_hue` (green/yellow/red per the shared map).
- `blocked` badge — `ByValue` on `act.token()` ("blocked" → Red; "actionable"
  → unmapped → plain). Cell text is `""` for actionable (invisible anyway) and
  `"BLOCKED"` for blocked (visible, red).
- `title` — zebra-striped via `Alternate` (matches all other list surfaces).

**Next (5 columns):**

```rust
const NEXT_COLS: &[Column<NextRow>] = &[
    Column { name: "id",       header: "id",
             cell: |r| r.id.clone(),
             paint: ColumnPaint::Fixed(DynColors::Ansi(Cyan)) },
    Column { name: "kind",     header: "kind",
             cell: |r| r.kind.clone(),
             paint: ColumnPaint::None },
    Column { name: "status",   header: "status",
             cell: |r| r.status.clone(),
             paint: ColumnPaint::ByValue(|r| status_hue(&r.status)) },
    Column { name: "unblocks", header: "unblocks",
             cell: |r| r.blocking.len().to_string(),
             paint: ColumnPaint::None },
    Column { name: "title",    header: "title",
             cell: |r| r.title.clone(),
             paint: ColumnPaint::Alternate([TITLE_EVEN, TITLE_ODD]) },
];
```

### 3d. Priority function signatures (changed)

Before:
```rust
pub(crate) fn survey_human(rows: &[SurveyRow], term_width: Option<u16>) -> String;
pub(crate) fn next_human(rows: &[NextRow], term_width: Option<u16>) -> String;
```

After:
```rust
pub(crate) fn survey_human(rows: &[SurveyRow], opts: RenderOpts) -> String;
pub(crate) fn next_human(rows: &[NextRow], opts: RenderOpts) -> String;
```

Implementation: check `rows.is_empty()` first (return the existing empty-string
message), else `render_columns(rows, &cols.iter().collect::<Vec<_>>(), opts)`.
Never call `render_table` directly — every cell goes through `paint_cell`.

## 4. IMP-038 — Column model: validate defaults at construction

In `select_columns`, before the `match requested`:

```rust
debug_assert!(
    requested.is_some() || default.iter().all(|n| available.iter().any(|c| c.name == *n)),
    "default column `{}` not in available set [{}]",
    default.iter().find(|n| !available.iter().any(|c| c.name == *n)).unwrap_or(&"?"),
    available.iter().map(|c| c.name).collect::<Vec<_>>().join(", ")
);
```

`debug_assert!` catches the bug during development/test. In release, the
existing `pick()` → `Err(unknown column)` backstop still fires — the fix makes
it earlier and clearer in debug, not new behaviour.

The assertion is at the top of `select_columns`, before any work, so it acts as
a construction-time gate: if a kind defines a default column name not in its
`available` set, the test suite panics immediately with the offending name and
the valid set.

## 5. IMP-040 — `--color` flag wiring

### CLI

Top-level `Cli` struct gains one field:

```rust
/// Control colour output
#[arg(long, default_value = "auto", global = true)]
color: clap::ColorChoice,
```

`clap::ColorChoice` provides `Auto`/`Always`/`Never` with built-in
`ValueEnum` parsing (case-insensitive: `auto`, `always`, `never`). No custom
enum needed.

### Injection into existing list surfaces

Each list command handler currently does approximately:

```rust
let color = tty::stdout_color_enabled();
let args = ListArgs { render: RenderOpts { color, .. }, .. };
```

Becomes:

```rust
let color = tty::resolve_color(cli.color);
let args = ListArgs { render: RenderOpts { color, .. }, .. };
```

~13 mechanical call sites. `cli` is the parsed top-level `Cli` — `--color` is
global, so every subcommand handler already has access.

### Injection into priority surfaces

`run_survey` and `run_next` gain a `ColorChoice` parameter:

```rust
pub(crate) fn run_survey(
    path: Option<PathBuf>, all: bool, format: Format, json: bool,
    color_choice: clap::ColorChoice,  // NEW
) -> anyhow::Result<()> {
    let opts = RenderOpts {
        color: crate::tty::resolve_color(color_choice),
        term_width: crate::tty::stdout_terminal_width(),
    };
    render::survey_human(&rows, opts)
}
```

Call sites in `main.rs` pass `cli.color`.

### Injection into status-line surfaces

Each handler (adr, policy, standard, knowledge, revision) resolves once at the
top:

```rust
let color = tty::resolve_color(cli.color);
```

Then wraps the status word: `listing::status_colored(status.as_str(), color)`.

## 6. Code impact summary

| Path | Change |
|---|---|
| `src/listing.rs` | `select_columns`: `debug_assert!` default-set validation (IMP-038). New `status_colored(status, color)` pure helper (IMP-039b). No signature changes to `render_columns`/`render_table`/`ColumnPaint`/`status_hue`. |
| `src/tty.rs` | New `resolve_color(mode: ColorChoice) -> bool` (IMP-040). Imports `clap::ColorChoice`. `stdout_color_enabled()` and `color_enabled()` unchanged. |
| `src/priority/render.rs` | `survey_human`/`next_human`: delete hand-built grid + `render_table` call, replace with `SURVEY_COLS`/`NEXT_COLS` arrays + `render_columns`. Signatures: `(rows, opts: RenderOpts)`. Empty-list messages preserved (checked before `render_columns`). New `use` imports for `Column`, `ColumnPaint`, `status_hue`, `TITLE_EVEN`, `TITLE_ODD`, `DynColors`. (IMP-039a) |
| `src/priority/mod.rs` | `run_survey`/`run_next`: gain `color_choice: ColorChoice` param; build `RenderOpts` from `resolve_color` + `stdout_terminal_width`; pass to render fns. (IMP-039a + IMP-040) |
| `src/main.rs` | Top-level `--color` flag on `Cli`. `run_survey`/`run_next` calls pass `cli.color`. ~13 list handlers: `stdout_color_enabled()` → `resolve_color(cli.color)`. (IMP-040) |
| `src/adr.rs` | Handler: inject `color` bool, wrap status with `listing::status_colored`. (IMP-039b) |
| `src/policy.rs` | Same pattern. |
| `src/standard.rs` | Same pattern. |
| `src/knowledge.rs` | Same pattern (`state` is the status word). |
| `src/revision.rs` | Same pattern (both `from` and `to` statuses). |
| Priority golden tests | Byte-identical: piped output under `color: false` produces the same bytes (headers plain, cells plain, same `│` layout). No re-baseline needed. |
| E2E golden tests | Unchanged (piped → plain). |

**No new dependencies.** `clap::ColorChoice` is in the existing `clap` dep.
`owo_colors` is already present. All imports are from existing crates.

## 7. Verification alignment

New / changed evidence:

- **Column model (VT):** test panics when a `Column` array defines a default not
  in its `available` set. Test passes when all defaults are valid. (IMP-038)
- **`resolve_color` (VT):** pure-table test over all 3 modes × the 2 tty arms:
  `Never` → false regardless; `Always` → true regardless; `Auto` → defers to
  `color_enabled(no_color, is_tty)`. The pure `color_enabled` already has
  coverage. (IMP-040)
- **`status_colored` (VT):** mapped status + `color: true` → ANSI present;
  unmapped status + `color: true` → plain; any status + `color: false` → plain.
  (IMP-039b)
- **Priority colour (VT):** `survey_human(rows, opts)` with `color: true` →
  ANSI in id, status, blocked-badge, and title columns; headers bold. `color:
  false` → zero ANSI, byte-identical to current goldens. (IMP-039a)
- **Priority empty (VT):** empty rows return the existing `"(no eligible
  work)\n"` / `"(nothing actionable)\n"` strings — behaviour preserved.
- **Existing priority goldens:** byte-identical under `color: false` —
  re-verified, no re-baseline needed (RSK-1 does not fire).
- **Existing list goldens:** byte-identical (piped → plain, no surface change).
- **Lint:** `just check` green; `cargo clippy` zero warnings.

## 8. Decisions

- **D1** — Priority tables route through `render_columns` (single colour
  mechanism per SL-053 design intent). Column arrays defined in
  `priority/render.rs`. Signatures: `(rows, opts: RenderOpts)`.
- **D2** — Status-bearing `writeln!` surfaces (adr, policy, standard, knowledge,
  revision) gain point-colour via `listing::status_colored`, reusing the shared
  `status_hue` map. No new hue map, no per-surface colour logic.
- **D3** — `--color=auto|always|never` is a global flag (`clap::ColorChoice`).
  Resolved by `tty::resolve_color`. Precedence: `Never` > `Always` >
  auto-detect (`NO_COLOR` > isatty). The pure layer sees only a `bool`.
- **D4** — Column model validates defaults at construction via `debug_assert!`
  in `select_columns`. Release backstop: the existing `pick()` error.
- **D5** — No new dependencies. All colour uses the existing owo_colors stack;
  `--color` is pure stdlib via `clap::ColorChoice`.
- **D6** — Priority empty-list messages preserved (checked before
  `render_columns`; `render_columns` returns `""` when empty).
- **D7** — Priority `id` column hue is Cyan (neutral, mixed-kind surface).
  Status column reuses `status_hue` unchanged. Title column zebra-striped
  (matches all other list surfaces).

Residual open questions: none.

## 9. Self-review findings

1. **`debug_assert!` justification.** The assertion fires only in debug builds.
   In release, the existing `pick()` error is the backstop — a bad default
   column still fails with a clear `unknown column` message. The assertion adds
   earlier, clearer detection during development/test. Accepted (D4).

2. **Priority golden churn (RSK-1) — does not fire.** Under `color: false`
   (piped/golden path), `paint_header` returns the raw string, `paint_cell`
   returns the raw string, and `render_table` is called with the same grid
   contents. Output bytes are **identical** to the current goldens. Verified by
   structural analysis of `render_columns` and `paint_cell` — no re-baseline
   needed.

3. **`status_colored` location.** Lives in `listing.rs` next to `status_hue`
   (the hue map is its natural dependency). Not in `tty.rs` (which is the
   impure capability shell, not a formatting library). Accepted.

4. **`Actionability::token()` return values.** `"actionable"` and `"blocked"`.
   `status_hue` maps `"blocked"` → Red. `"actionable"` falls through to
   `None` (uncoloured) — but the blocked-badge cell text is `""` for
   actionable rows, so there's nothing visible to colour anyway. Correct.

5. **`status_hue` map unchanged.** The existing conservative-subset map covers
   every status token the five targeted surfaces emit (`accepted`, `required`,
   `active`, `design`, `plan`, `started`, `done`, `abandoned`, `contested`,
   `blocked`). `proposed`/`superseded`/`deprecated`/`recommended`/`optional`/
   `open`/`resolved` stay grey — deliberate, not missed. No modification needed.

6. **No `select_columns` involvement for priority.** Priority surfaces don't
   support `--columns` — they call `render_columns` with the full `&[&Column]`
   array directly, not through `select_columns`. This is correct: priority is
   not a user-filterable list surface.

7. **Adversarial review pass.** Self-review complete. Findings integrated above.
   Offer external review or advance to `/plan`.
