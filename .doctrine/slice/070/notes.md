# SL-070 Implementation notes

## Completed

- 3 commits on `main`:
  - `a98b7de` — Type widening: ColumnPaint::Fixed → DynColors, mechanical call-site updates
  - `05d549d` — Gruvbox truecolour tag palette (12 Rgb entries)
  - `bbe484e` — Per-value hue maps (backlog kind, memory type, memory trust) + alternating title zebra + tests

- 1323 tests pass, zero clippy warnings.

## Surprises

- `clippy::manual_is_multiple_of` flagged `row_index % 2 == 0` — changed to `row_index.is_multiple_of(2)`.
- `coverage_view.rs` and `knowledge.rs` and `rec.rs` also define columns — not listed in the initial scope doc. Caught during adversarial review, added to design before implementation.
- `REQ_COLUMNS` (spec.rs) and `COVERAGE_COLUMNS` (coverage_view.rs) have no title column — no `Alternate` wiring needed there.

## Rough edges

- `TITLE_EVEN`/`TITLE_ODD` hues (`#ebdbb2` / `#d5c4a1`) are subtle gruvbox foreground colours that may be hard to distinguish on very bright terminals. Consider making them slightly darker (e.g. `#d5c4a1` / `#bdae93`) if feedback indicates poor contrast.
- The gruvbox tag palette uses 24-bit truecolour. Terminals older than ~2015 will see fallback colours (terminal-dependent). No fallback path implemented — this is acceptable per the design's risk assessment.

## Follow-ups

- Consider colour for `review` facet column and `spec` kind differentiation per scope doc.
- Truecolour → 4-bit ANSI degradation path if demanded (low priority).

## Audit (RV-033)

- Review done: 2 findings, both verified, 0 unresolved.
- **F-1 (major, fixed):** TITLE_EVEN/TITLE_ODD RGB tuples mismatched design.md.
  Comments claimed `#ebdbb2`/`#d5c4a1` but values were `(235,235,235)` and
  `(215,184,57)`. Corrected to match design: `Rgb(235,219,178)` and
  `Rgb(213,196,161)`. Tests remain green.
- **F-2 (minor, tolerated):** EX-3 visual verification on a real terminal
  cannot be done in this environment. Automated evidence (1324 tests, clippy
  clean) is strong; a human must confirm before final close.

## Commits

All committed on `main`: `a98b7de`, `05d549d`, `bbe484e`.
