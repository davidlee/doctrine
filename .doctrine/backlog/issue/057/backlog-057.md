# ISS-057: Priority scoring penalises items with estimates by ranking estimate-bare items above them

## Observation

IMP-120 surfaced without an estimate. The priority scoring formula likely
treats a missing estimate as zero cost (or minimal), which gives it a higher
score than items that carry honest 1–6 point estimates — the estimator is
penalised for doing the work. An item with no estimate is not a zero-cost item;
it is an item whose cost is *unknown*.

## Desired behaviour

Estimate-bare items should either:

1. **Be flagged explicitly** in `backlog list` output so the bias is visible
   (e.g. a `⚠ no estimate` column or sort-affecting marker).
2. **Be deprioritised** — treat missing estimate as high uncertainty (wide
   default bounds) rather than zero, so they don't leapfrog estimated items.
3. **Both** — flag + deprioritise.

## Terrain

- `src/priority.rs` — the scoring formula; locate where estimate feeds in and
  how the absent case is handled.
- `src/commands/backlog.rs` — list rendering; where a visibility marker would
  be added.
