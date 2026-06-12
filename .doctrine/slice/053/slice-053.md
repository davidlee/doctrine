# Terminal output polish: comfy-table listings + owo_colors

## Context

Doctrine's CLI listing surfaces render through a single shared seam in
`src/listing.rs`: `render_columns<R>` materialises rows into `Vec<Vec<String>>`,
which `render_table` aligns by space-padding each column to its widest cell
(final column unpadded, no separators, no header rule). Output is plain
monochrome ASCII.

Intent: add visual polish ("bling"). Two libraries:
- **comfy-table** — back `render_table` with a real table renderer, styled
  *minimalist*: no outer frame, no horizontal rules, **interior vertical column
  separators only**.
- **owo_colors** — colour cell content (ids, statuses, etc) on the shared
  listing surfaces.

The shared seam is the asset here: comfy-table and colour both land *behind* the
existing chokepoint, so this is not a parallel rendering implementation
(CLAUDE.md: no parallel implementation). The risk is the inverse — letting
colour or table logic leak into per-kind call sites instead of staying behind
`render_columns`/`render_table`.

## Scope & Objectives

In scope:
1. Adopt **comfy-table** behind `render_table` — minimalist preset: inner
   vertical column delimiters only, no outer border, no horizontal lines.
   `render_columns` and all per-kind list call sites keep their current
   signatures; only the renderer changes.
2. Adopt **owo_colors** for colour on the surfaces that ride the
   `render_columns` seam — backlog, rec, governance, slice, memory, review, spec,
   coverage_view. (`memory list` already rides `render_columns`, so OQ-1 is void;
   `coverage_view::render_table` is a thin wrapper over `render_columns`, not a
   parallel renderer.) `priority` (`survey_human`/`next_human`) calls
   `render_table` directly and gains the new `│` layout but stays monochrome —
   its colour is deferred (see Follow-Ups).
3. **Colour gating** — emit ANSI only to a colour-capable TTY; honour `NO_COLOR`
   and non-tty (piped) output. Piped/golden output stays byte-for-byte plain so
   the black-box CLI goldens have a deterministic, colour-free target.
4. Re-baseline the affected black-box goldens to the new minimalist table shape
   (vertical separators) — colour-free, since goldens capture piped output.

Out of scope (deferred — capture as follow-ups, do not implement here):
- Colouring the ad-hoc `writeln!(io::stdout(), …)` surfaces that bypass the
  shared seam (`adr.rs`, `policy.rs`, `install.rs`, `reconcile.rs`,
  `corpus.rs`). The eventual goal is colour on all/most screens; the design
  should leave a reusable colour seam so that extension is additive, but the
  surfaces themselves are not touched in this slice.
- Any change to filtering, sorting, column selection, or the `--columns` /
  `--format json` contracts.

## Affected surface

- `Cargo.toml` (workspace) + `crates`/root `Cargo.toml` — add `comfy-table`,
  `owo_colors` to `[workspace.dependencies]` and the binary's `[dependencies]`.
- `src/listing.rs` — `render_table` reimplemented over comfy-table; colour
  applied at the `render_columns` cell layer; new colour-gating helper.
- Black-box golden tests covering list surfaces — re-baselined.
- `memory list` call site — colour wired (see open question on the seam).

## Risks, assumptions, open questions

(Design `design.md` is canon; OQ-1..3 are resolved there — recorded here for the
scope trail.)

- **OQ-1 — RESOLVED (void).** `memory list` already renders through
  `listing::render_columns` (memory.rs:1312); colouring it is free. IMP-017 /
  IMP-018 concern the `--columns` flag, not the renderer, and stay out of scope.
- **OQ-2 — RESOLVED.** Capability resolved in the impure shell (`tty.rs`:
  `NO_COLOR` via `var_os` + `io::stdout().is_terminal()`, stdlib) and injected as
  a `bool` into the pure render layer. owo_colors used via *unconditional*
  colorize methods gated by that bool — **not** `if_supports_color` (it reads
  env/tty, which would poison the pure layer).
- **OQ-3 — RESOLVED.** comfy-table `presets::NOTHING` +
  `set_style(VerticalLines, '│')`; `ContentArrangement::Disabled` for
  determinism; comfy-table owns all width/padding (no "last column unpadded"
  special case survives). Feature set (`default-features=false` + `custom_styling`)
  proven by a spike at the head of phase 1.
- **RSK-1 — golden churn.** Table-shape change touches every list golden;
  scope the re-baseline carefully so a shape change can't hide a content
  regression. Re-baseline shape and content in separate reviewable steps.
- **RSK-2 — colour bleeding into goldens.** If gating is wrong, ANSI escapes
  leak into piped output and goldens turn flaky/locale-dependent. The gate is
  the correctness crux.
- **ASM-1** — comfy-table can express the minimalist style without a custom
  renderer; if it cannot, re-open the comfy-table adoption decision in `/design`
  rather than forcing it.
- **Lint** — new deps; respect repo clippy denies (no `print_stdout`; colour
  writes go through the existing `writeln!`-on-`io::stdout()` surfaces, not
  `print!`).

## Verification / closure intent

- Listing surfaces render with inner vertical separators and colour when run on
  a colour-capable TTY.
- Piped output (and therefore goldens) is byte-for-byte plain, colour-free, with
  the new minimalist table shape — re-baselined goldens pass.
- A test pins the colour gate: `NO_COLOR` / non-tty ⇒ zero ANSI in output.
- `just check` green; `cargo clippy` zero warnings.
- Deferred colour surfaces captured as follow-up backlog items.

## Follow-Ups

- Colour the **deferred-colour surfaces** via the colour seam this slice
  establishes: the ad-hoc `writeln!` output (adr/policy/install/reconcile/corpus)
  **and** `priority` (`survey_human`/`next_human`, which rides `render_table`
  directly). One follow-up item.
- `--color=auto|always|never` global flag (this slice ships auto-detection only).
