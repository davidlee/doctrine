# IMP-090: style.css modularisation & CSS debt from RV-065

Post-SL-091 follow-up for the 10 findings in RV-065 (code-review of
`web/map/style.css`).

## Source

RV-065 — code-review of SL-073's style.css. 10 findings open, awaiting
responder disposition. All findings are non-blocking (3 major, 5 minor,
2 nit).

## Findings to resolve

- **F-1 (major):** Split 861-line monolithic style.css into per-subsystem
  files (base layout, concept-map, priority view) or at minimum document
  clear ownership boundaries.
- **F-2 (major):** Consolidate the second `:root {}` block (line 775,
  priority tokens) into the canonical `:root` block.
- **F-3 (major):** Define `--border-light` and `--bg-card` as real design
  tokens or remove the dead custom property names (currently always falling
  through to hardcoded fallbacks).
- **F-4 (minor):** Route all 18 hardcoded hex colours in the CM section
  through the custom property system.
- **F-5 (minor):** Adopt and document a consistent naming convention
  (currently BEM, flat, and SMACSS-ish classes coexist).
- **F-6 (minor):** Add `prefers-color-scheme: dark` variants for CM form
  controls, priority SVG nodes, and error/notice states.
- **F-7 (nit):** Move `--link` token definition into the canonical `:root`
  block (currently scattered across three locations).
- **F-8 (minor):** Replace JS inline `style.display` manipulation with
  class-based toggling for CM panels (two competing visibility systems).
- **F-9 (nit):** Flatten compound selectors like
  `.markdown-pane.fullscreen .markdown-body` to dedicated classes.
- **F-10 (minor):** Rename `.hidden` to a scoped name
  (`.relationship-table--hidden`) or an `is-hidden` data attribute to
  avoid utility-class collision risk.
