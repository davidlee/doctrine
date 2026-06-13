# SL-054 Design — Table cell wrapping for terminal-width-constrained output

## 1. Current vs target behaviour

**Current (SL-053).** `render_table` delegates layout to comfy-table with
`ContentArrangement::Disabled` + `force_no_tty()`: column widths derive purely
from content, never from the terminal. This bought byte-stable output
terminal-vs-pipe (deterministic goldens) but means a wide table on a narrow
terminal **overflows** — comfy-table emits each row at full content width and the
terminal hard-wraps mid-cell, shredding the `│` alignment.

**Target.** Terminal-width-aware cell wrapping **when rendering to a TTY**:
comfy-table measures column widths against the terminal and wraps over-long cells
onto multiple lines, `│` separators drawn down every wrapped line. Piped output
is **unchanged** — no width measurement, no wrapping — so goldens stay frozen and
colour-free.

The injection pattern is the one SL-053 established (D3, the `color` bool): the
impure shell resolves terminal width and injects it as `Option<u16>` into the
pure render layer. `None` ⇒ the current deterministic path, byte-for-byte.

## 2. Architecture — the pure/impure boundary holds (spike-proven)

The pure/imperative split (slices-spec § Architecture; CLAUDE.md; ADR-001 leaf ←
engine ← command) forbids env/tty/clock/rng/git/disk reads in the pure layer.
`render_table` is a **pure leaf** — its module header asserts "Nothing here reads
the clock, rng, git, or disk." SL-054 must not breach that.

The scope doc's **ASM-1 proposed dropping `force_no_tty()` under the wrapping
path** — which would re-open the `custom_styling → tty → stdout().is_terminal()`
read SL-053 fought to close (the load-bearing D6 guard; recorded memory
`mem.pattern.render.comfy-table-custom-styling-pulls-tty`). **A spike refuted
ASM-1.**

### Spike record (comfy-table 7.2.2, `custom_styling`)

`ContentArrangement::Dynamic` + `set_width(24)` was rendered **with** and
**without** `force_no_tty()`:

```
force_no_tty=true  | lines=4 | has_esc=false      force_no_tty=false | lines=4 | has_esc=false
 id │ a very long title                            id │ a very long title
    │ that should wrap                                │ that should wrap
    │ across several                                  │ across several
    │ lines here                                      │ lines here
```

- **Wrapping is orthogonal to `force_no_tty`.** Both arms wrap to 4 identical
  lines. Wrapping is the *arrangement* axis (Dynamic measures + splits); the tty
  consult `force_no_tty` suppresses is the *styling* axis. We set **no**
  comfy-table styling attributes (only the `│` separator + padding), so the
  styling axis is inert for output bytes regardless.
- **Conclusion:** keep `force_no_tty()` **unconditional**. Switch only the
  arrangement. The pure leaf stays tty-free — **no carve-out, no purity/readability
  trade.**

```
shell (impure)                          pure layer (src/listing.rs)
──────────────                          ───────────────────────────
tty::stdout_color_enabled() ─bool────▶  RenderOpts { color, term_width }
tty::stdout_terminal_width()           render_columns(rows, cols, opts)
  is_terminal()? ──Option<u16>───────▶    ├─ paint cells (when color)
  crossterm::terminal::size()              └─ render_table(grid, term_width)
                                                ├─ Some(w) → Dynamic + set_width(w)
                                                ├─ None    → Disabled
                                                └─ force_no_tty() ALWAYS (D6, load-bearing)
```

## 3. The RenderOpts seam (D1 — F3 folded from the SL-053 review)

SL-053's review flagged `render_columns(rows, cols, color: bool)` threading a
naked `bool` through ~13 call sites; SL-054 needs a *second* render axis through
the same sites. Rather than thread a second positional, bundle once:

```rust
/// Render-time options resolved ONCE in the impure shell and injected into the
/// pure render layer (SL-053 D3 pattern, generalised — SL-054). Every future
/// render axis is a new field here, not a new positional through ~13 call sites.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RenderOpts {
    /// Emit ANSI colour (SL-053 PHASE-02). `false` ⇒ byte-clean (piped/goldens).
    pub(crate) color: bool,
    /// Terminal width for cell wrapping (SL-054). `None` ⇒ no measurement, the
    /// deterministic SL-053 path (piped/goldens). `Some(w)` ⇒ Dynamic wrap to `w`.
    pub(crate) term_width: Option<u16>,
}

pub(crate) fn render_columns<R>(rows: &[R], cols: &[&Column<R>], opts: RenderOpts) -> String;
pub(crate) fn render_table(rows: &[Vec<String>], term_width: Option<u16>) -> String;
```

**Why the split.** `render_table` is the lower primitive (3 callers: `render_columns`
+ priority ×2) and consumes **only** `term_width` — it has never carried colour
(colour is applied upstream in `paint_cell` before the grid reaches it). A single
`Option<u16>` positional there is proportionate, not creep. The *wide* seam
(`render_columns`, ~13 sites) takes the struct. `ListArgs.color: bool` becomes
`ListArgs.render: RenderOpts`.

**Two wrappers ride along.** `coverage_view::render_table(rows, columns, color)` is
a thin wrapper over `render_columns` (not a parallel renderer); its signature flips
`color` → `opts: RenderOpts` and forwards. `priority::render::{survey_human,
next_human}` call `listing::render_table` directly — each gains a
`term_width: Option<u16>` param (they stay monochrome — colour deferred — but they
**do** wrap; they are TTY human surfaces).

`Default` makes `RenderOpts::default()` = `{ color: false, term_width: None }` =
the deterministic path, so every `..Default::default()` test and the `boot`
projection stay plain with no change.

**Exact caller inventory (grep-derived, not "~13").** The external review (codex,
GPT-5.5) flagged the fuzzy count as a miss-risk — the empty-result early-return in
`rec::run_list` is an easy-to-skip `render_columns` site. The complete production
surface:

| # | site | arg today | note |
|---|------|-----------|------|
| 1 | `backlog.rs:1022` | `color` | |
| 2 | `coverage_view.rs:362` | `color` | inside the `coverage_view::render_table` wrapper |
| 3 | `slice.rs:1043` | `color` | |
| 4 | `spec.rs:1259` | `color` | block rows |
| 5 | `spec.rs:1451` | `color` | req list |
| 6 | `memory.rs:1319` | `color` | |
| 7 | `governance.rs:85` | `color` | |
| 8 | `review.rs:951` | `color` | |
| 9 | `rec.rs:576` | `color` | **empty-branch early-`return match`** — easy to miss |
| 10 | `rec.rs:589` | `color` | populated branch |

Ten production `render_columns` calls across 8 surfaces. Each `run_list` already
threads its `color` from `ListArgs`; replacing `ListArgs.color: bool` with
`ListArgs.render: RenderOpts` makes the change mechanical at each.

Test-helper callers update mechanically too (all inside `#[cfg(test)]`): `slice.rs:1331/1338`,
`memory.rs:3390`, `review.rs:2337`, plus the in-crate `listing.rs` tests — they pass
`false` literals → `RenderOpts::default()` / `RenderOpts { color: true, .. }`.

`listing::render_table` direct callers: `priority::render::{survey_human:46,
next_human:71}` (gain `term_width`) and `coverage_view.rs:460` (via its wrapper).

**Shell resolution = 3 points** (unchanged): `CommonListArgs::into_list_args`
(main.rs:112 — the *single* `color` resolution serving all ~10 list subcommands
dispatched at main.rs:1831–2217), `coverage_view::run`, and `priority::mod` run.

**Behaviour-preservation gate.** This is shared-machinery churn (the entity
engine's list spine). The existing in-crate + black-box suites are the proof —
they must stay green. The `color = false` / `term_width = None` defaults guarantee
the piped path is byte-identical; the in-crate callers update mechanically from
`, false)` / `, color)` to `, RenderOpts { .. }` / `, opts`. **No caller currently
passes `color: true` that a careless `..Default::default()` could silently flip to
mono** — every coloured site threads the shell-resolved `color`, never a literal
`true`, so the field-for-field rewrite preserves it.

## 4. render_table — arrangement switch, everything else unconditional (D2)

```rust
pub(crate) fn render_table(rows: &[Vec<String>], term_width: Option<u16>) -> String {
    if rows.is_empty() { return String::new(); }
    // ... rectangular-grid debug_assert (unchanged) ...
    let mut table = Table::new();
    // Per-grid structural floor (D3, F-B from the external review): Dynamic only
    // when `w` can actually fit THIS grid — borders + padding + ≥1 content col each.
    // A 6–7-col priority table needs ~25+ cols; comfy-table `available_content_width`
    // saturating_subs to ~0 below that and forces 1 char/col → unreadable slivers.
    // Below the grid minimum, Disabled (clean overflow) beats Dynamic (garbage).
    let cols = rows.first().map_or(0, Vec::len);
    let fits = |w: u16| usize::from(w) >= grid_min_width(cols);   // borders+padding+cols
    match term_width {
        Some(w) if fits(w) => { table.set_content_arrangement(ContentArrangement::Dynamic); table.set_width(w); }
        _                  => { table.set_content_arrangement(ContentArrangement::Disabled); }
    }
    table.force_no_tty();                 // UNCONDITIONAL — D6 purity guard (spike: orthogonal to wrap)
    // ... strip components, set VerticalLines '│' (unchanged) ...
    // ... add rows, zero outer-edge padding (unchanged) ...
    // ... per-line trim_end + single '\n' (UNCONDITIONAL — see RSK-4 below) ...
}
```

**RSK-4 refuted — trim stays unconditional.** The scope doc proposed gating
`trim_end` to the `Disabled` path, assuming Dynamic fills cells differently. The
spike output shows Dynamic **still** pads every line to its column width (`a very
long title ` and `that should wrap  ` carry trailing fill). So `trim_end` is
needed under *both* arrangements; gating it off under Dynamic would re-introduce
trailing whitespace. Trailing spaces on a wrapped line are never significant, and
`trim_end` strips spaces *after* any owo reset codes (resets precede the fill), so
colour is preserved. **Keep the trim unconditional.**

The outer-edge padding-zero loop (D7 from SL-053) is also unchanged — it operates
on column padding, independent of arrangement.

## 5. Width resolution in the impure shell (D3)

```rust
// src/tty.rs — the impure shell, mirroring stdout_color_enabled's pure split.
pub(crate) fn stdout_terminal_width() -> Option<u16> {
    terminal_width(
        std::io::IsTerminal::is_terminal(&std::io::stdout()),
        crossterm::terminal::size().ok().map(|(cols, _rows)| cols),
    )
}

/// Pure decision, both impurities injected (testable without a real tty).
/// `None` ⇒ no wrapping (the deterministic SL-053 path).
fn terminal_width(is_tty: bool, cols: Option<u16>) -> Option<u16> {
    if !is_tty { return None; }
    match cols {
        Some(w) if w >= MIN_WRAP_WIDTH => Some(w),
        _ => None,   // 0 / unreadably-narrow / unavailable ⇒ fall back to no-wrap
    }
}

/// Coarse shell-side pre-filter for degenerate sizes (`size() == 0`, headless /
/// unreadably-narrow terminals): below it, skip wrapping and emit clean overflow.
/// NOT the authoritative fit test — that is grid-dependent (`render_table`'s
/// `grid_min_width`, §4); the shell has no grid, so this protects nothing the grid
/// floor wouldn't and only adds a cheap shell-side cutoff.
const MIN_WRAP_WIDTH: u16 = 16;
```

**Two-tier floor (D3, F-B).** The width gate is split across the two layers that
own the two facts:
- `tty.rs` (shell) knows the *terminal* but not the *grid* → the coarse
  `MIN_WRAP_WIDTH` degenerate guard only.
- `render_table` (pure) knows the *grid* (column count + minimalist `│` style) but
  not the terminal → the real `grid_min_width(cols)` structural fit test. When
  `w < grid_min_width`, fall back to `Disabled` (clean overflow) rather than feed
  Dynamic a budget it will shred. `grid_min_width(cols) = borders(cols) +
  padding(cols) + cols` (≥1 content char per visible column), matching
  comfy-table's own `available_content_width` accounting so the design's fit test
  agrees with the library's. `cols == 0` (empty grid) returns early before either
  test (existing `rows.is_empty()` guard).

**OQ-1 resolved — no `NO_WRAP` env gate.** Width follows isatty alone. `NO_COLOR`
does **not** gate wrapping (monochrome-wrapped output is legitimate — the two axes
are independent). A manual override is deferred to a future `--width=N` flag
(Follow-Ups), not a bespoke env var now.

**D4 — two independent probes.** `stdout_color_enabled()` and
`stdout_terminal_width()` each do their own `is_terminal()` check (two cheap
syscalls per list invocation). No shared shell-side state; mirrors the SL-053
shape. The shell resolves capability into `RenderOpts` at **three** points:
`CommonListArgs::into()` (main.rs — the 8 colour+layout list kinds),
`coverage_view::run()`, and `priority::mod` run (which resolves `term_width` and
passes it to `survey_human`/`next_human` → `render_table`).

**Purity.** `crossterm::terminal::size()` is an `ioctl`/syscall — impure, lives in
`tty.rs` (the shell), injected as `Option<u16>`. The pure layer never reads it
(RSK-2 honoured).

**Erratum (PHASE-03 dispatch).** The pre-implementation claim that `terminal::size`
needed *no `Cargo.toml` edit* because `crossterm` is "already transitive via
comfy-table" was **wrong** — it conflated *graph-presence* with *nameability*.
comfy-table re-exports only `crossterm::style::{Attribute, Color}` (never
`terminal::size`), so naming `crossterm::terminal::size()` requires declaring
`crossterm` as a **direct** dependency. The achievable invariant is therefore "no
new *compiled crate*", not "no `Cargo.toml` edit": the direct `crossterm = "0.29"`
matches the version already resolved in the lockfile via comfy-table, so it adds
**zero new compiled weight** — it only puts the path on the extern prelude.
(Consulted + approved; supersedes the original "no new dependency / no `Cargo.toml`
edit" claim replaced inline above.)

## 6. Verification

- **VT — wraps under Some.** `render_table(grid, Some(40))` over a wide cell
  yields multi-line rows; every line carries the `│` separator; no line exceeds
  the budget after `trim_end`.
- **VT — None is byte-identical.** The existing SL-053 exact-shape tests
  (`render_table_line_shape_minimalist_vertical_separators`,
  `render_table_aligns_a_middle_column_the_slice_case`, …) are re-run with `, None)`
  threaded and must pass **unchanged** — they *are* the None-invariance proof (the
  determinism property: piped output is width-free). No new fixture needed.
- **VT — painted wide cell wraps + colour survives (RSK-3).** A `ByValue`-painted
  wide cell rendered with `RenderOpts { color: true, term_width: Some(w) }` wraps,
  and stripping ANSI reproduces the plain wrapped layout (comfy-table re-emits the
  SGR per wrapped segment — spike-confirmed). **This doubles as the first test that
  exercises a `ByValue` column under wrapping**, extending the SL-053-review
  coverage (the `paint_cell` ByValue path) into the wrap dimension.
- **VT — grid floor falls back (F-B).** `render_table(wide_7col_grid, Some(20))`
  produces the **Disabled** byte-output (overflow), not a Dynamic sliver render —
  `20 < grid_min_width(7)`. And `render_table(grid, Some(w))` for `w` just at/above
  `grid_min_width` does wrap. Pins the structural-minimum boundary.
- **VT — `terminal_width` pure seam.** `terminal_width(false, _) == None`;
  `terminal_width(true, Some(80)) == Some(80)`; `terminal_width(true, Some(0)) ==
  None`; `terminal_width(true, Some(8)) == None` (below floor). The live isatty
  branch in `stdout_terminal_width` is documented-not-driven (a pty is out of
  scope), mirroring `color_enabled`.
- **VT — goldens stay green UNCHANGED.** No re-baseline. A moved golden signals
  width leaking into piped output (mirrors SL-053 PHASE-02 VT-4).
- `just check` green; `cargo clippy` (plain) zero warnings.

## 7. Decisions

- **D1** — `RenderOpts { color, term_width }` bundles render axes at the
  `render_columns` seam (F3, folded from the SL-053 review); `render_table` takes
  a bare `term_width: Option<u16>` (its only render option; 3 callers).
- **D2** — `render_table` switches arrangement on `term_width`
  (`Some → Dynamic+set_width`, `None → Disabled`); `force_no_tty()`, edge-zeroing,
  and `trim_end` stay **unconditional**.
- **D3** — width resolved in `tty.rs` via crossterm, pure split
  (`terminal_width(is_tty, cols)`), coarse `MIN_WRAP_WIDTH = 16` degenerate guard
  there; the real per-grid `grid_min_width(cols)` fit test lives in `render_table`
  (pure, owns the grid), falling back to `Disabled` when `w` can't fit the grid
  (F-B, external review). `0`/narrow ⇒ `None`.
- **D4** — two independent isatty probes; no shared shell state.
- **D5** — OQ-1: no `NO_WRAP`; width follows isatty; manual override deferred to a
  `--width` flag follow-up.
- **D6** — piped path byte-identical (`RenderOpts::default()`); goldens frozen.

## 8. Risks, assumptions, open questions

- **RSK-1 (golden churn)** — *retired by D6.* Piped ⇒ `None` ⇒ no width path; a
  determinism test pins it. Verify no golden exercises `Some(..)` before commit.
- **RSK-2 (crossterm purity)** — *honoured.* `terminal::size()` stays in `tty.rs`.
- **RSK-3 (wrap + colour)** — *spike-confirmed handled;* pinned by a VT.
- **RSK-4 (trim under Dynamic)** — *refuted by spike;* trim stays unconditional.
- **ASM-1 (Dynamic vs force_no_tty)** — *refuted by spike;* force_no_tty
  unconditional. The scope doc's ASM-1 is superseded by this design (§2).
- **ASM-2 (CONFIRMED — comfy-table 7.2.2 source)** — `set_width(w)` treats `w` as
  the **total table width** (separators + padding + content). Verified in-tree:
  `set_width` sets `table.width = Some(w)` (`table.rs:255`), consumed as
  `table_width` in `arrangement::arrange` → `dynamic::arrange`, where
  `available_content_width` *subtracts* `count_border_columns` + per-column padding
  from `w` (`dynamic.rs:135`). So `w` is the full-table budget, not content-only.
  We pass the full terminal column count. Second-order (codex): the outer-edge
  padding-zero loop runs *after* measurement, so comfy measures padding it won't
  render → the *rendered* width is **≤ `w`** (strictly narrower, never overflows).
  The discrepancy is one-directional and safe; the only requirement is that the
  minimalist `│`-only style is set *before* arrangement runs (it is — style strip
  precedes row add), so `count_border_columns` matches the rendered separators.
- **OQ (closed)** — OQ-1 resolved (D5). No open questions remain.

## 9. Follow-Ups

- `--width=N` CLI flag for manual override / scripted wrapping.
- `--no-wrap` flag (the manual counterpart to the auto floor), if demand surfaces.
