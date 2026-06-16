# SL-079 — Finish the CLI colour story: deferred surfaces + --color flag

## Context

SL-053 (terminal output polish) shipped comfy-table listings + owo_colors colour
behind the `render_columns` seam. Two surfaces were explicitly deferred to a
follow-up slice (SL-053 §1, D5, §9 phase 3):

- **Priority surfaces** (`survey_human` / `next_human`) — call `render_table`
  directly, bypassing the `render_columns` colour seam. They gained `│` layout
  but stayed monochrome.
- **Ad-hoc `writeln!` surfaces** — render outside the shared listing seam
  entirely (adr, policy, install, reconcile, corpus detail views).

SL-053 also deferred the `--color=auto|always|never` flag (D5) — it shipped
auto-detection only (`NO_COLOR` + isatty, resolved in `tty::stdout_color_enabled`).

A small column-model fix (IMP-038) is a natural prerequisite: validate the
default column set against the available set at construction rather than lazily
on first request, so that adding colour-bearing columns to the priority surface
doesn't fail late.

## Scope & Objectives

In scope:

1. **IMP-038** — Column model: validate the default column set against the
   available set at construction time (fail early, clear message), not on first
   request. This is a small correctness fix that removes a latent footgun before
   colour-bearing columns are added to new surfaces.

2. **IMP-039** — Colour the two deferred listing surfaces via the *existing*
   `render_columns` seam:
   - **Priority surfaces** (`survey_human`, `next_human`): route through
     `render_columns` instead of calling `render_table` directly. This picks up
     colour, `ColumnPaint`, and the shared `status_hue` for free — no second
     colour mechanism.
   - **Ad-hoc `writeln!` surfaces**: bring under colour without forking the
     mechanism. Prefer routing through `render_columns` where the surface is
     tabular; for detail/prose surfaces (adr, policy, standard, knowledge,
     revision lifecycle status lines), apply colour via the same injected `color: bool`
     resolved in the shell, using owo_colors' unconditional methods gated by
     that bool. Never use `if_supports_color` in the pure layer (D3).

   The purity rule is unchanged: capability resolved in the impure shell
   (`tty::stdout_color_enabled`), injected as a `bool` into the pure render
   layer. Piped output stays byte-for-byte plain; goldens stay colour-free.

3. **IMP-040** — Add `--color=auto|always|never` global flag. The flag overrides
   the auto-detected `color` bool *before* it is injected into `render_columns`.
   The pure render layer is unchanged — this is a shell-side addition only.
   Precedence (design decision): `--color=never` beats `NO_COLOR` beats isatty;
   `--color=always` beats non-TTY pipe. The flag is an explicit user override.

Out of scope:
- **IMP-044** (RenderOpts migration for priority surfaces) — seam-uniformity
  cleanup, not colour-critical. Priority now routes through `render_columns`
  (D1); `RenderOpts` is threaded but the internal `render_table` still takes
  bare `term_width`. Capture IMP-044 for a follow-up.
- **IMP-056** (coverage kebab-case formatter) — separate subsystem, separate
  concern.
- Any change to filtering, sorting, column selection, or `--columns`/`--format`.
- Colour for non-listing TUI surfaces (map explorer, etc).
- Ad-hoc `writeln!` surfaces beyond the five status-bearing lines (creation
  confirmations, dispatch, worktree boot, etc — stay monochrome).

## Affected surface

- `src/listing.rs` — column model validation at construction (IMP-038); may need
  a minor signature change to `render_columns` or `Column` if priority routing
  requires it (IMP-039).
- `src/priority/render.rs` — `survey_human` / `next_human`: route through
  `render_columns` instead of calling `render_table` directly (IMP-039).
- `src/tty.rs` — extend `stdout_color_enabled` to accept an optional `--color`
  override, or add a new shell-side resolver (IMP-040).
- Ad-hoc writeln! call sites (IMP-039): `src/adr.rs`, `src/policy.rs`,
  `src/standard.rs`, `src/knowledge.rs`, `src/revision.rs` — colour applied at each
  call site via the injected `color: bool`.
- CLI argument parsing — new `--color` flag (IMP-040).
- Black-box goldens — existing goldens stay green (piped output is plain);
  priority goldens may shift if the routing change alters the output shape
  (e.g., header row if `render_columns` adds one).

## Risks, assumptions

- **ASM-1** — Priority routing through `render_columns` is a better fit than
  extending a second colour path into `render_table`. If `render_columns`
  cannot accommodate priority's row shape without disproportionate adaptation,
  re-open at `/consult`.
- **RSK-1** — Priority golden churn from routing change. Analysis shows goldens
  are byte-identical under `color: false` (piped path) — no re-baseline needed
  (design §9.2). If implementation diverges, re-baseline in a separate commit.
- **No new dependencies.** All work uses the existing owo_colors + comfy-table
  stack; the `--color` flag is a pure stdlib addition via `clap::ColorChoice`.

Open questions resolved in design:
- OQ-1: priority routes through `render_columns` with `Column<SurveyRow>` /
  `Column<NextRow>` arrays defined in `priority/render.rs`.
- OQ-2: status-bearing `writeln!` lines only (adr, policy, standard, knowledge,
  revision) — point-colour via `listing::status_colored`.
- OQ-3: `--color` flag is global — governs tables and status lines.

## Verification / closure intent

- Priority surfaces (`survey`, `next`) render with colour on a TTY, plain when
  piped.
- Ad-hoc writeln! surfaces (adr, policy, standard, knowledge, revision lifecycle)
  show coloured output on a TTY, plain when piped.
- `--color=never` suppresses colour even on a TTY; `--color=always` emits colour
  even when piped; `--color=auto` (default) follows `NO_COLOR` + isatty.
- `NO_COLOR` is honoured; `--color=never` wins over `--color=auto`.
- Column model validates default set at construction — a missing default column
  panics at construction time with a clear message, not on first render.
- `just check` green; `cargo clippy` zero warnings.
- All existing goldens pass (colour-free piped output).

## Follow-Ups

- IMP-044 — RenderOpts migration for priority surfaces (seam uniformity).
- IMP-056 — Coverage status kebab-case formatter (separate concern).
