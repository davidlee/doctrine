# CLI list defaults hide slug; opt-in reveal + format control

## Context

Backlog IMP-009. Every numbered / own-struct `list` table renders a `slug`
column — backlog, slice, spec, adr, requirement, governance. Slugs are long and
volatile (e.g. IMP-016's 90-char slug) and dominate table width, while the
durable identity is the prefixed id and the slug is *never authoritative*
(CLAUDE.md reference form). So the slug column is noise in the default survey
view.

The shared read spine (SL-025, `src/listing.rs`) already centralises the
invariant axes — filter (`FilterFields`), `Format`, `render_table`, the JSON
envelope. The *variant* axis (column projection) stays per-kind: each kind's
`format_rows` builds the grid and bakes `slug` in. JSON rows carry `slug` as
data (SL-025 D7), and the filter substr/regex domains search `slug` via
`FilterFields` — both independent of what the table chooses to *display*.

## Scope & Objectives

- Default table `list` output omits the `slug` column for the kinds that
  currently show it.
- Add an opt-in that reveals slug again ("format options"). The exact surface is
  a design decision (e.g. a boolean threaded through the shared `ListArgs` seam
  and applied uniformly), not fixed here.
- Keep the behaviour uniform across kinds by riding the shared `listing.rs`
  seam, not ad-hoc per-kind flags.

## Non-Goals

- No change to JSON output — `slug` stays in JSON rows (data, not presentation).
- No change to filter semantics — `slug` stays searchable via substr/regex.
- No change to single-entity `show` — `slug` is still shown there.
- Not a general column-selection framework / arbitrary `--columns`; design may
  scope only the minimal slug opt-in.
- Not the IMP-013 shared list+show shape lift, nor the IMP-014 golden harness —
  related surface, but their sequencing is a design question, not a deliverable
  of this slice.
- Memory `find`/`list` (keyed, no slug column) is unaffected.

## Affected Surface

- `src/listing.rs` — `ListArgs` / `Format` seam carries the reveal option
  through the leaf.
- `src/main.rs` — `CommonListArgs` clap bundle gains the flag.
- per-kind grid builders — `src/backlog.rs`, `src/slice.rs`, `src/spec.rs`,
  `src/adr.rs`, `src/requirement.rs`, `src/governance.rs` (`format_rows` / list
  headers).

## Risks, Assumptions, Open Questions

- RISK: golden / conformance tests pinning list output churn across every
  affected kind; no IMP-014 harness yet to catch regressions cheaply (mem
  `conformance-asserts-surface`, `black-box-cli-golden`).
- ASSUMPTION: slug is *hidden from the default table*, not removed — id + title
  carry identity; slug persists in JSON, the filter domains, and `show`.
- OQ-1: opt-in mechanism — a boolean (`--slug` / `--wide`) vs a `--columns`
  selector. (design)
- OQ-2: does the reveal flag live in the shared seam (uniform) or per-kind?
  (working assumption: shared seam.)
- OQ-3: sequencing against IMP-013 (shared list+show shape) — ride it or precede
  it. (design)

## Summary

A presentation-only change: drop `slug` from the default `list` tables across
the numbered/own-struct kinds and add an opt-in to reveal it, threaded through
the shared `src/listing.rs` seam. JSON, filtering, and `show` are untouched.

## Follow-Ups
