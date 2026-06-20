# Implementation Plan SL-130: Web map: RFC entity visualization

## Overview

Single-phase plan. The change touches three files in `web/map/src/` (frontend
only) to add RFC visual identity to the entity graph — node fill colour, CSS
custom property, and kind-filter checkbox. No backend changes needed.

## Sequencing & Rationale

The three edits are independent (no order dependency), but the natural sequence
is (1) `tokens.css` so the CSS variable exists, (2) `index.html` so the filter
toggle exists, (3) `dot.ts` so the DOT rendering uses the colour. Verification
is manual via `bun run dev` against the live RFC-001 entity.

## Notes

- RFC-001 was created manually from proposal #14 as test fodder.
- No automated tests for visual rendering — VH-1 (human verification) is the mode.
- The warm grey (#7F8C8D) was chosen to signal "no governance weight" and
  is distinct from REC's #95A5A6.
