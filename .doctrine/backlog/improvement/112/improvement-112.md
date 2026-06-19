# IMP-112: Wire estimate display (incl. confidence percentile framing) into the show path

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

The estimate display renderers (`src/estimate/display.rs`:
`format_bound`/`format_estimate_normal`/`format_estimate_verbose`) and the
confidence resolver (`estimate::resolve_confidence`) exist but carry no live call
site — they are `expect(dead_code)`. The human `slice show` path renders no
estimate/value facet; only `slice show --json` exposes them (via `SliceDoc`).

SL-104 legitimizes confidence at the spec tier (REQ + SPEC-020 amendment, percentile
framing — `lower`/`upper` as the project P-low/P-high band) but **defers all display
wiring** (its non-goal: no new features). This item is that deferred work.

## Scope

- Wire the estimate (and value) display renderers into the human `show` path.
- Frame the bounds with the resolved confidence percentiles (e.g.
  `Estimate: 2-8 espresso_shots (P10–P90)`), per the SL-104 confidence REQ — its
  whole intent is display framing.
- Removing the wiring lands clears the `expect(dead_code)` tripwires on
  `resolve_confidence`, the `DEFAULT_*_CONFIDENCE` consts, and `mod display`.

Descends SPEC-020. Author a slice when greenlit.
