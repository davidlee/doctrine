# Table cell wrapping for terminal-width-constrained output

## Context

SL-053 adopted comfy-table behind `render_table` with `ContentArrangement::Disabled`
+ `force_no_tty()` — column widths derive purely from content, never from the
terminal. This buys byte-stable output terminal-vs-pipe (goldens are
deterministic) but means a wide table on a narrow terminal overflows rather than
wrapping cells onto multiple lines.

This slice adds terminal-width-aware cell wrapping **when rendering to a TTY**.
Piped output stays deterministic — no width measurement, no wrapping — so the
goldens remain colour-free and width-independent.

The established impure-shell→pure-layer injection pattern (D3 from SL-053: the
`color` bool) is reused: the shell resolves terminal width, injects it as
`Option<u16>`.

## Scope & Objectives

In scope:
1. Bundle the render axes into one `RenderOpts { color, term_width }` injected at
   the `render_columns` seam (folds F3 from the SL-053 code review — replaces the
   naked `color: bool` rather than threading a second positional through ~13 sites);
   `render_table` takes a bare `term_width: Option<u16>` (its only render option).
   `None` → piped/deterministic (current behaviour).
2. When `term_width` is `Some(w)`:
   - Set `ContentArrangement::Dynamic` with `set_width(w)` — comfy-table measures
     column widths and wraps cells that exceed the terminal.
   - `force_no_tty()` stays **unconditional** (a spike refuted ASM-1: wrapping is
     the arrangement axis, orthogonal to the styling tty-consult force_no_tty
     suppresses — the pure leaf stays tty-free, no purity/readability trade).
   - Keep the minimalist `│` separator shape, outer-edge-zeroing, and per-line
     `trim_end` from SL-053 D7 intact and **unconditional** (a spike refuted RSK-4:
     Dynamic still fills cells to column width, so the trim is needed here too).
3. When `term_width` is `None`:
   - `ContentArrangement::Disabled` — byte-for-byte the current SL-053 behaviour
     (force_no_tty already unconditional). Goldens stay green untouched.
4. Resolve terminal width in the impure shell — `tty.rs` gains a
   `stdout_terminal_width() -> Option<u16>` using crossterm (already a transitive
   dep via comfy-table's `custom_styling` → `tty`). Returns `None` when piped; width
   follows isatty **alone** (design D5 — no `NO_COLOR`/`NO_WRAP` env gating; the
   colour and wrap axes are independent). A coarse `MIN_WRAP_WIDTH=16` guards the
   degenerate `size()==0` case; the real per-grid fit test lives in `render_table`.
5. Thread `RenderOpts` through all ~13 `render_columns` call sites (replacing the
   `color` bool); flip `coverage_view::render_table`'s wrapper to `RenderOpts`; pass
   `term_width` to `priority::render::{survey_human,next_human}` (third shell point,
   `priority::mod` run — they wrap but stay monochrome).
6. Tests: `render_table` with `term_width = Some(40)` produces multi-line cells
   for wide content; `term_width = None` is byte-identical to current
   `render_table` output. A determinism test asserts piped output is width-free.
7. Re-baseline affected goldens if any (expected: none — goldens capture piped
   output, which stays `term_width = None`).

Out of scope:
- `--width=N` CLI flag (auto-detection only).
- Changing the deferred-colour surfaces (adr/policy/install/reconcile/corpus,
  priority).
- Any change to JSON output.
- Adaptive column reordering or hide-to-fit strategies.

## Affected surface

- `src/tty.rs` — add `stdout_terminal_width() -> Option<u16>` (crossterm
  `terminal::size`, gated by isatty alone; coarse `MIN_WRAP_WIDTH=16` degenerate floor).
- `src/listing.rs` — `render_table` gains `term_width: Option<u16>` + a per-grid
  `grid_min_width` fit test; `render_columns` takes `RenderOpts` (replaces `color:
  bool`); `ListArgs.color` becomes `ListArgs.render: RenderOpts`.
- **10 production `render_columns` call sites** (exact, grep-derived — design §3):
  backlog, coverage_view-wrapper, slice, spec ×2, memory, governance, review, rec ×2
  (incl. the `rec.rs:576` empty-branch early-return). Plus 4 `#[cfg(test)]` helpers.
- `priority/render.rs` (×2: `survey_human`, `next_human`) — call `render_table`
  directly; add `term_width` arg (they wrap, stay monochrome).
- 3 shell resolution points: `CommonListArgs::into_list_args` (one point, ~10 list
  subcommands), `coverage_view::run`, `priority::mod` run.

## Risks, assumptions, open questions

- **RSK-1 — golden churn.** If any golden test runs against a terminal-width path
  (shouldn't — they're piped), the re-baseline would mask a regression. Mitigation:
  verify no golden exercises `term_width = Some(…)` before committing.
- **RSK-2 — crossterm purity.** `terminal::size()` reads the terminal via
  `ioctl`/syscall — impure. It stays in `tty.rs` (the shell), never crosses into
  the pure layer.
- **RSK-3 — wrapping + colour.** Wrapped lines carry ANSI resets at line boundaries;
  comfy-table's `custom_styling` (already enabled) handles this. A test pins
  coloured cells wrapping without visible escape leakage.
- **RSK-4 — trailing whitespace. REFUTED (spike).** `Dynamic` *still* fills each
  line to its column width, so per-line `trim_end()` is needed under both
  arrangements. The trim stays **unconditional**; gating it off would re-introduce
  trailing whitespace. (Trailing spaces follow any owo reset, so colour survives.)
- **OQ-1 — NO_WRAP. RESOLVED (design D5).** No `NO_WRAP` env gate. Width follows
  isatty alone (`!isatty` → `None` → no wrap); `NO_COLOR` does not gate wrapping
  (monochrome-wrapped is valid). Manual override deferred to a `--width` flag
  follow-up.
- **ASM-2 — set_width accounting. CONFIRMED (comfy-table 7.2.2 source).** `set_width(w)`
  is the *total* table width (borders+padding+content); `dynamic::arrange` subtracts
  borders+padding from `w`. Zeroing outer padding after measurement renders ≤`w` —
  safe, one-directional. (design §8.)
- **F-B — MIN_WRAP_WIDTH floor too low. FIXED (external review, design D3/§4).** A
  flat 16 admits garbage renders for wide (6–7-col) tables. Split: coarse 16 in the
  shell for the degenerate case; real per-grid `grid_min_width` fit test in
  `render_table` → fall back to `Disabled` (overflow) when `w` can't fit the grid.
- **ASM-1 — Dynamic + force_no_tty contradiction. REFUTED (spike).** Not a
  contradiction: wrapping is the arrangement axis, `force_no_tty` the styling axis.
  `Dynamic + set_width` wraps identically with force_no_tty on or off (we set no
  comfy styling). `force_no_tty()` stays **unconditional** — the pure leaf keeps
  the SL-053 D6 tty-free guard, at zero readability cost.

## Verification / closure intent

- `render_table` with `term_width = Some(40)` wraps cells; narrow terminals show
  multi-line rows with `│` separators drawn across wrapped lines.
- `render_table` with `term_width = None` is byte-identical to current (SL-053)
  output — all existing tests pass unchanged.
- `tty.rs::stdout_terminal_width()` returns `None` when piped; `Some(w)` on a TTY.
- `just check` green; `cargo clippy` zero warnings.
- Existing colour tests still pass (wrapping + colour coexistence).

## Follow-Ups

- `--width=N` CLI flag for manual override / scripted wrapping.
- `--no-wrap` flag (manual counterpart to the auto floor), if demand surfaces.
