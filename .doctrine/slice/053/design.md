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

**Surface inventory (precise — adversarial-review corrected).** Colour flows
through the `render_columns` seam, and *only* there:

- **Colour + layout** (ride `render_columns`): backlog, rec, governance, slice,
  memory, review, spec, **coverage_view** (its `render_table` at
  coverage_view.rs:330 is a thin wrapper that delegates to
  `listing::render_columns` — it is *not* a parallel renderer).
- **Layout only, no colour** (call `listing::render_table` directly, bypassing
  the paint path): `priority/render.rs` (`survey_human`, `next_human`). These
  gain the `│` separators for free but stay monochrome this slice; their colour
  is deferred to the same follow-up as the ad-hoc `writeln!` surfaces, keeping
  the colour story to a single mechanism (the `render_columns` seam).

`render_table`'s signature is **unchanged** (`fn(&[Vec<String>]) -> String`) — it
never carried colour and does not now. Only `render_columns` gains the `color`
param, so the direct `render_table` callers (priority) are untouched.

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
                                            (force_no_tty — D6, load-bearing)
```

**Load-bearing purity guard (D6).** comfy-table built with `custom_styling`
transitively enables its `tty` feature (see §5); without intervention its content
formatter calls `Table::should_style()` → `is_tty()` → `stdout().is_terminal()` at
*format time* (table.rs:396,360,371). That is a tty read inside the pure render
layer and makes piped output terminal-dependent. `render_table` **must** call
`Table::force_no_tty()` before `to_string()`; this is not optional polish, it is
the seam that keeps `render_table` pure and the goldens stable. A test asserts
identical bytes under a forced-terminal vs forced-no-terminal stdout.

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
pub(crate) enum ColumnPaint<R> {
    None,
    Fixed(owo_colors::AnsiColors),
    /// Hue derived from the ROW, not the emitted cell text. The display cell may
    /// carry decoration the hue map can't match — `slice list` emits `done ⚠` /
    /// `bogus? ⚠` (slice.rs `decorated_status`), `review list` emits
    /// `open (await …)` (review.rs). The extractor reads the row's *semantic*
    /// status, so decoration never costs the cell its colour.
    ByValue(fn(&R) -> Option<owo_colors::AnsiColors>),
}

pub(crate) struct Column<R> {
    pub(crate) name: &'static str,
    pub(crate) header: &'static str,
    pub(crate) cell: fn(&R) -> String,
    pub(crate) paint: ColumnPaint<R>,       // NEW
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

**Shared status hue (no per-kind duplication).** One function maps a *bare*
status token to a hue. It is fed the row's semantic status by a per-column
extractor, **never** the decorated display cell:

```rust
fn status_hue(s: &str) -> Option<owo_colors::AnsiColors> {
    match s {
        "done" | "active" | "accepted" | "required" => Some(Green),
        "in_progress" | "started" | "design" | "plan" | "audit" | "reconcile" => Some(Yellow),
        "blocked" | "abandoned" | "contested" => Some(Red),
        _ => None, // proposed/open/ready/… → default colour
    }
}

// each kind's status column wires the row→token extractor against the row's
// real shape (row types are tuples — destructure to the bare status element):
//   slice:  ColumnPaint::ByValue(|(m, _)| status_hue(&m.status))        // (Meta, Option<PhaseRollup>)
//   review: ColumnPaint::ByValue(|(_, s, _)| status_hue(s.as_str()))    // (ReviewDoc, ReviewStatus, Await)
```

The shared map stays singular (no per-kind duplication of the token→hue table);
only the *source* is per-column — the raw status field on the row, not
`render_columns`' emitted string. This is the fix for the refuted "status cell
stays a bare token" premise (§9a F-4): `slice`/`review` decorate the cell, so a
match on emitted text would silently drop colour exactly where it's wanted. The
exact token→hue table is finalised at implementation against the live status
vocabularies (`doctrine <kind> --help` / the status enums); the design fixes the
*mechanism* (one shared map + per-column row extractor), not an authoritative
token list.

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
  table width → comfy-table never *arranges* against the terminal; column widths
  derive only from content. (comfy-table's default `Dynamic` arrangement consults
  terminal width via crossterm — that path is disabled.) Disabling arrangement is
  necessary but **not sufficient**: see the tty read below.
- comfy-table dependency (corrected against the resolved crate, 7.2.2):
  `default-features = false` + the **`custom_styling`** feature (needed so
  display-width measurement strips the ANSI that `render_columns` applies upstream,
  keeping separators aligned). **`custom_styling` is not crossterm-free** — its
  feature graph is `custom_styling = ["dep:ansi-str","dep:console","tty"]` and
  `tty = ["dep:crossterm"]` (Cargo.toml:41-48). We therefore **accept crossterm as
  a transitive dependency** (path A, user decision); the earlier "drop the
  crossterm/tty machinery" framing was refuted (§9a F-3). Crossterm being linked is
  harmless to determinism *only because* arrangement is `Disabled` (no width query)
  **and** `force_no_tty()` is called (no style/tty query — D6). Both are
  load-bearing; neither alone suffices.
- **Exact line shape — leading/trailing whitespace pinned (D7).** The old
  hand-rolled renderer produced **no leading space** (first cell at column 0) and
  **no trailing whitespace** (last column left unpadded). comfy-table does **not**
  reproduce this for free, and the `NOTHING` preset is a trap: it sets every border
  component to a literal *space* (presets.rs:154), so `style_exists(LeftBorder)` is
  true → a leading space is drawn, and the default column padding `(1,1)`
  (column.rs:48) appends a right pad to the final column → a trailing space. Both
  reverse the old property and bake fragile edge-whitespace into the goldens
  (editors/CI that strip trailing WS would corrupt them). The seam therefore pins
  the shape explicitly:
  - **Outer borders absent, not spaced.** Build the style by *removing* every
    border/corner/horizontal/intersection component (`remove_style`), then set only
    `TableComponent::VerticalLines` to `│`. Do **not** `load_preset(NOTHING)` (its
    components exist-as-space). `should_draw_left_border`/`_right_border` then return
    false (borders.rs:229) → no outer edge char.
  - **Per-column padding zeroes the outer edges.** Interior padding `(1,1)` gives
    the minimalist ` │ ` inner separator; then set the **first** column's left pad
    and the **last** column's right pad to `0`. Result: `id │ kind │ status │ title`
    — clean both edges, exactly matching the old no-leading/no-trailing property,
    while gaining the `│` separators. (render_table knows the column count at
    runtime, so the first/last zeroing is a post-build mutation.)
  - A determinism test asserts the **exact** bytes of a small table — including the
    absence of any leading or trailing space on every line — not merely "separators
    present".
- `render_table` re-appends the trailing `\n` (comfy-table's `to_string()` omits
  it; backlog.rs:1045 documents callers printing the result verbatim, relying on
  the seam's own newline). Empty grid → `""` preserved.

## 6. Verification alignment

New / changed evidence:

- **Pure colour tests** (in `listing.rs`): `render_columns(rows, cols, true)`
  contains ANSI escapes for painted columns + bold header; `(.., false)` contains
  zero ANSI. A width/alignment test with a painted column proves comfy-table's
  ANSI-aware measurement keeps separators aligned (no drift from the escapes).
- **Shape test (D7)**: a small table asserts exact bytes — no leading space on any
  line, no trailing whitespace on any line, ` │ ` interior separators.
- **force-no-tty determinism (D6)**: identical bytes from `render_table` whether
  stdout is a terminal or a pipe (the `force_no_tty()` guard neutralises the only
  tty read). Width measurement itself is `UnicodeWidthStr::width` — tty-independent —
  so this test pins the colour/style path, the only tty-sensitive surface.
- **Wide-char caveat (latent).** Width shifts from `chars().count()` to
  display-width (unicode-width via `custom_styling`); these diverge for CJK /
  combining / wide-emoji cells. No *current* golden seeds a wide cell — the `⚠`
  divergence marker only renders against a seeded state tree, which no fixture
  provides, and `—` (U+2014) is width 1 either way — so the re-baseline hides no
  present alignment change. This safety is *incidental*: the day a golden seeds a
  `done ⚠` slice row, alignment will shift. Noted, not blocking.
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
- **D6** — `render_table` calls `Table::force_no_tty()` before `to_string()`. The
  `custom_styling` → `tty` feature edge makes the content formatter read
  `stdout().is_terminal()` at format time; `force_no_tty` is the load-bearing seam
  that keeps the pure layer pure and piped output stable (§2, §9a F-3).
- **D7** — exact line shape is pinned (§5): outer borders *removed* (not the
  space-filled `NOTHING` preset), only `VerticalLines = │`, interior padding `(1,1)`
  with the first column's left pad and last column's right pad zeroed → no leading
  or trailing whitespace, matching the old renderer's property. Pinned by a
  byte-exact shape test (§9a F-6).

Residual: none blocking. ASM-1 is **resolved against crate 7.2.2** (not deferred):
comfy-table cannot give ANSI-aware width without `custom_styling`→`tty`→crossterm,
so crossterm is accepted (D1/path A) and neutralised by `ContentArrangement::
Disabled` + `force_no_tty()` (D6). The execute-time spike now *proves* that pair
deterministic, rather than testing whether crossterm can be dropped.

## 9a. Adversarial review (internal + two external passes)

Hostile self-review, then an external pass (codex/GPT-5.5, read-only) that
**refuted** internal F-3 and F-4 against the resolved crate manifest and live
source (rewrote F-3 feature graph, F-4 status_hue, added D6), then a **second
adversarial pass** (opus inquisitor, read-only) targeting the integration itself —
which surfaced the line-shape gap (F-6), corrected the §3 examples, and added the
cargo-group caveat (F-8). The three integrated findings verified sound; the F-4
generic refactor introduces no new heresy (const column tables still construct;
every painted kind has a bare-token status accessor; `force_no_tty` neutralises the
*only* tty read — width is `UnicodeWidthStr`, tty-independent).

- **F-1 — scope imprecision (integrated §1).** "Everything through
  `render_columns`" was loose. Corrected to a precise surface inventory:
  coverage_view rides the seam (colour free); priority calls `render_table`
  directly (layout only, colour deferred). No surface is silently dropped.
- **F-2 — ragged grids / behaviour preservation (dismissed).** comfy-table can
  misbehave on rows of unequal column count; the old hand-rolled `render_table`
  tolerated raggedness (`max` cols + `get(c)`). Verified every producer is
  rectangular: `render_columns` always emits header+uniform cells; priority's
  hand-built grids are fixed 7/5 columns with a matching header. **Invariant:**
  `render_table` is only ever handed rectangular grids. A guard test pins this so
  a future ragged caller fails loudly rather than mis-rendering.
- **F-3 — comfy-table feature gamble (REFUTED by external pass; design corrected
  §2/§5/§7).** Internal pass assumed `custom_styling` could work under
  `default-features = false` *without crossterm*. The external review checked the
  resolved manifest (comfy-table 7.2.2): `custom_styling = ["dep:ansi-str",
  "dep:console","tty"]`, `tty = ["dep:crossterm"]` — the feature is **inseparable
  from crossterm**. Worse, the `tty` feature makes the content formatter call
  `should_style()` → `is_tty()` → `stdout().is_terminal()` at format time
  (table.rs:396,360,371), so `ContentArrangement::Disabled` alone does **not**
  buy determinism or purity. Resolution: accept crossterm transitively (D1/path A)
  and add the mandatory `force_no_tty()` guard (**D6**). The phase-1 spike stands
  but its goalpost moves — it proves `Disabled + force_no_tty` deterministic, not
  that crossterm is absent.
- **F-4 — status_hue premise REFUTED by external pass; design corrected §3.**
  Internal pass claimed the status column "stays a bare token" so matching on
  emitted text is safe. False: `slice list` decorates the status cell itself —
  `done ⚠` / `bogus? ⚠` (slice.rs `decorated_status`) — and `review list` emits
  composite `open (await …)` (review.rs). `ByValue(fn(&str))` on the emitted cell
  would silently drop colour on exactly those surfaces. Fix: `ColumnPaint<R>`'s
  `ByValue` now takes `fn(&R)` and reads the row's **semantic** status; the shared
  `status_hue` token map is unchanged. Graceful `None` survives as a backstop, but
  is no longer load-bearing — the correct source is wired in.
- **F-5 — not behaviour-preserving by design (clarified §6).** The CLAUDE.md
  behaviour-preservation gate ("shared machinery suites stay green unchanged")
  targets the entity engine; this change deliberately alters listing output. The
  governing gate here is the `e2e_list_conformance` net, which *forces*
  acknowledgment of any shared-surface format change — the re-baseline is that
  acknowledgment, performed in an isolated commit (RSK-1).
- **Ripple correction.** ~13 `render_columns` call sites across 8 files (slice ×3,
  memory ×2, review ×2, spec ×2, backlog, rec, governance, coverage_view) plus
  in-crate tests gain the `color` arg; `render_table` callers (priority ×2) are
  unaffected (signature unchanged).
- **F-6 — line-shape gap (opus pass; integrated §5/§6, D7).** The design swapped
  renderers without pinning padding/border/trim. comfy's `NOTHING` preset fills
  borders with spaces → a leading space; default padding `(1,1)` → a trailing
  space — both *reverse* the old no-leading/no-trailing property and bake fragile
  edge-whitespace into goldens. Resolved by D7 (remove outer borders, zero the
  outer-edge pads) + a byte-exact shape test.
- **F-7 — fictional §3 examples (opus pass; fixed §3).** The illustrative
  `ByValue` accessors (`r.authored_status()`, `r.status.as_str()`) named methods
  the tuple row types don't have. Rewritten to destructure the real tuples
  (`(Meta, Option<PhaseRollup>)`, `(ReviewDoc, ReviewStatus, Await)`). Doc-accuracy
  only — the mechanism was always sound.
- **F-8 — cargo-group caveat (opus pass; documented §8).** Repo clippy `cargo`
  group is `deny`; the crossterm subtree could trip `multiple_crate_versions` on a
  duplicate-major collision. Not design heresy — a phase-1 `cargo tree -d` check;
  if a dup surfaces, an `expect`+reason allow on the bin. (The known
  new-*member*-metadata memory does **not** apply — deps land in an existing member.)

## 8. Code impact summary

| Path | Change |
|---|---|
| `Cargo.toml` (workspace + bin) | add `comfy-table` (`default-features=false`, features `["custom_styling"]` — pulls crossterm transitively, accepted per D1/path A) + `owo_colors` to `[workspace.dependencies]` and the bin `[dependencies]`. Phase-1 `cargo tree -d` check: the `cargo` clippy group is `deny`, so a crossterm-subtree `multiple_crate_versions` collision needs an `expect`+reason allow (§9a F-8) |
| `src/listing.rs` | `render_table` reimplemented over comfy-table (layout only; **`force_no_tty()` before `to_string()` — D6**); `render_columns` gains `color` param + paint application; `ColumnPaint<R>` enum (`ByValue` reads the row, not the cell); `status_hue`; hand-rolled width/pad maths deleted; pure colour + alignment + force-no-tty determinism tests |
| `src/tty.rs` (new) | `stdout_color_enabled()` — impure capability resolution |
| 11 `run_list` call sites | resolve + pass the `color` bool into `render_columns` |
| each kind's `Column` literals | id columns gain `Fixed(hue)`; status columns gain `ByValue(status_hue)`; others `None` |
| goldens (listed §6) | re-baseline to `│`-separated plain shape (separate commit) |

## 9. Phasing intent (for /plan)

Provisional, to keep RSK-1 clean:

1. **Renderer swap** — *spike first* (F-3): prove comfy-table with the resolved
   feature set (`default-features=false` + `custom_styling`, crossterm transitive)
   renders a throwaway table deterministically and terminal-size-independent under
   `ContentArrangement::Disabled` **+ `force_no_tty()`** — assert identical bytes
   with stdout forced-terminal vs forced-pipe (D6). Then swap comfy-table behind
   `render_table` (no colour yet), minimalist `│` separators, rectangular-grid
   guard test; re-baseline goldens. Pure shape change, isolated commit. If the
   spike fails, re-open D1 before any further work.
2. **Colour seam** — `tty.rs`, `color` param on `render_columns`, `ColumnPaint`,
   `status_hue`, paint the column literals across the ~13 sites, pure colour
   tests. Goldens stay green (piped ⇒ plain). priority stays monochrome (layout
   only).
3. **Follow-ups capture** — backlog items for: ad-hoc `writeln!` surface
   colouring **+ priority colour** (one item, the deferred-colour surfaces); and
   the `--color=auto|always|never` flag.
