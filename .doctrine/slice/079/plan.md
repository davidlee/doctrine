# Implementation Plan SL-079: Finish the CLI colour story: deferred surfaces + --color flag

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

Three phases, ordered by dependency: foundation plumbing first (IMP-038 +
IMP-040), then the two colour-surface phases (IMP-039a priority tables,
IMP-039b status lines). PHASE-02 and PHASE-03 are independent of each
other — both rest on PHASE-01's plumbing — but PHASE-02 is sequenced first
because priority tables touch the shared `render_columns` seam and
`status_hue` map, which validate the colour infrastructure before the
simpler writeln! surfaces.

All three improvements rest on the same pure/impure boundary SL-053
established: capabilities resolved in the shell (`tty.rs`), injected as
plain `bool` into the pure layer (`listing.rs`). No new impurities.

## Sequencing & Rationale

### PHASE-01 — Foundation (IMP-038 + IMP-040)

Combines two independent but co-located changes:

- **IMP-038** (column model validation): a single `debug_assert!` in
  `select_columns` — the smallest possible change with the highest
  leverage (catches misconfigured defaults at construction time in debug;
  release backstop via existing `pick()` error).

- **IMP-040** (--color flag): adds `resolve_color` to `tty.rs`, `--color`
  flag on `Cli`, and the `color: bool` injection seam on
  `CommonListArgs::into_list_args`. These ~13 mechanical call sites are
  the thinnest possible change — every handler already has `cli` in scope
  via `#[command(subcommand)]` destructuring.

Combined into one phase because both are plumbing-only: no user-visible
colour changes, no new rendering behaviour. The phase is small (~40 lines
of code + tests), and the combined tests validate the flag plumbing end to
end before colour surfaces are added.

### PHASE-02 — Priority table colour (IMP-039a)

The largest change: replaces two hand-built grid assemblers
(`survey_human`, `next_human`) with `Column` arrays routed through
`render_columns`. Lines deleted > lines added — the hand-built grid
construction (vec-of-vecs, manual header row, manual cell extraction) is
replaced by declarative column definitions.

Key risk: golden test churn. The design proves byte-identity under
`color: false` by structural analysis — `paint_header` and `paint_cell`
return raw strings at the `!color` early return, so the grid passed to
`render_table` is identical. The VT-2 test re-verifies this against the
existing golden snapshot.

Column definitions are verified cell-by-cell against the hand-built grid
(VT-4): each `cell` closure is compared to the corresponding expression
in the current `grid.push(vec![...])` block.

### PHASE-03 — Status-line colour (IMP-039b)

The simplest phase: one pure helper (`status_colored`) in `listing.rs`,
wired into five handler functions. Each handler resolves `color` once at
the top and wraps the status word. Revision is the only dual-status
surface — two `status_colored` calls joined by literal `" → "`.

The `status_hue` map requires no modification — it already covers every
token the five surfaces emit (`accepted`, `required`, `active`, `done`,
`design`, `plan`, `started`, `abandoned`, `contested`, `blocked`).
Unmapped tokens (`proposed`, `draft`, `default`, `deprecated`, `retired`,
`open`, `answered`, `obsolete`, `waived`, `held`, `testing`, `validated`,
`invalidated`) stay grey deliberately — the map is a conservative subset
by design (design §9.5).

The revision `run_approve` approval-status line is intentionally excluded
(design §1, RV-045 F-1) — approval is a distinct axis from lifecycle.

## Notes

- No file-disjoint phases. All three touch `src/listing.rs` in some way
  (PHASE-01: select_columns, PHASE-03: status_colored). Serial execution
  is required.
- PHASE-01 is the gate: if `resolve_color` or `into_list_args(color:)`
  breaks any existing test, stop and fix before PHASE-02.
- Priority golden tests: if VT-2 fails (byte divergence under color:false),
  re-examine cell closures before considering re-baseline. RSK-1 analysis
  is sound — divergence indicates implementation error, not design flaw.
- `cargo clippy` zero warnings after each phase; `just check` (fast
  inner-loop) between changes, `just gate` before commit.
