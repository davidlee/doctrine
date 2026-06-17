# CSS modularisation: split monolithic style.css and resolve RV-065 findings

## Context

RV-065 code-reviewed `web/map/src/style.css` — the 861-line stylesheet for the
Doctrine Map Explorer. The file is attributed to SL-073 PHASE-01 but has
organically accreted CSS for SL-075 (SVG node styling), SL-076 (concept map
authoring UI), and the priority/DAG view without modularisation.

Ten findings were raised (3 major, 5 minor, 2 nit), none blocking but together
represent organisational debt that will compound with every additional slice.
IMP-090 tracks this CSS debt. IMP-085 (web frontend code-quality hardening) is
a broader umbrella covering JS/TS decomposition; this slice scopes the CSS
subset exclusively.

The file is imported as a single global stylesheet via `app.ts` line 1:
`import './style.css'`. Vite handles bundling. No CSS framework is in use;
all styles are hand-authored.

## Scope & Objectives

1. **Modular decomposition.** Split the monolithic CSS into per-concern files:
   - `style.css` — entry point, imports only (`@import` or Vite `import`)
   - `tokens.css` — single canonical `:root` block with all custom properties
     (kind palette, theme vars, priority vars) + `prefers-color-scheme` dark block
   - `reset.css` — box-sizing reset
   - `layout.css` — grid layout, sidebar, main content area
   - `sidebar.css` — search, filter, depth selector, entity list, kind pills,
     refresh, focus header
   - `graph.css` — graph area, SVG nodes, hover pane, placeholder
   - `markdown.css` — markdown pane, fullscreen, toolbar
   - `table.css` — relationship table, edge detail, edge legend
   - `concept-map.css` — CM authoring UI (edit toggle, add-edge form, edge table,
     remove/rename, diagnostics, view toggle)
   - `priority.css` — priority/DAG view nodes, edges, zoom

2. **Resolve all 10 RV-065 findings:**

   **F-1 (major).** Monolithic file, no modularisation across 4+ subsystems.
   → Resolved by objective 1 — the file split itself.

   **F-2 (major).** Second `:root {}` block at line ~775 redefines priority
   custom properties 710 lines after the canonical block closed.
   → Consolidate priority custom properties into the single canonical `:root`
   block in `tokens.css`.

   **F-3 (major).** `--border-light` and `--bg-card` are used throughout the CM
   section with hardcoded fallbacks but are *never defined*.
   → Define `--border-light` (light: `#e0e0e0`, dark: `#333333`) and
   `--bg-card` (light: `#fafafa`, dark: `#2a2a2a`) in `tokens.css`. Remove
   all hardcoded fallbacks once tokens exist.

   **F-4 (minor).** 18 distinct hardcoded hex colours in the CM section bypass
   the custom property system.
   → Promote hardcoded colours to custom properties or use existing theme
   variables where equivalent. At minimum, define CM-specific tokens
   (`--cm-*`) for the distinct semantic colours.

   **F-5 (minor).** Three naming conventions (BEM `doctrine-node--focus`, flat
   `cm-edge-row`, SMACSS-ish `.active`/`.hidden`) coexist with no rule.
   → Standardise on a single convention. BEM-lite is already dominant for the
   node classes; extend it consistently. Scoped prefixes (`cm-*`, `priority-*`)
   are already in use — formalise as the module-boundary rule: each CSS file
   uses its prefix namespace.

   **F-6 (minor).** Dark mode covers the base layout but leaves CM form controls,
   priority SVG nodes, and error states in light-only purgatory.
   → Add dark variants for CM form controls, priority nodes, and every
   hardcoded error/success/notice background.

   **F-7 (nit).** `--link` scattered in three places instead of canonical `:root`.
   → Move `--link` into the single `tokens.css` `:root` block with its dark
   variant in the `prefers-color-scheme: dark` block.

   **F-8 (minor).** JS inline `style.display` manipulation competes with CSS
   class-based toggling — two visibility mechanisms, same file.
   → Audit the TypeScript source for `style.display` assignments and replace
   with class-based toggling (`.visible`/`.hidden` or module-specific
   toggle classes). This touches TS files, not just CSS.

   **F-9 (nit).** Compound selectors like `.markdown-pane.fullscreen .markdown-body`
   encode DOM hierarchy that will break on any refactor.
   → Flatten where safe, or add a dedicated class (`.markdown-body--fullscreen`)
   on the inner element when fullscreen activates.

   **F-10 (minor).** `.hidden` class name is dangerously broad — a utility-class
   collision magnet if any CSS framework ever enters.
   → Rename to a scoped utility: `.u-hidden` or use module-specific toggle
   classes.

3. **Preserve visual identity.** No visual regressions. The map explorer must
   render identically in light and dark modes after the refactor. Test by
   visual comparison (before/after screenshots or side-by-side browser tabs).

### Affected surface

- `web/map/src/style.css` — decomposed into 10 files; becomes an import-only
  entry point
- `web/map/src/app.ts` — update CSS import (single entry import unchanged if
  using `@import` cascade; one additional import per module if flat)
- `web/map/src/concept-map.ts` — replace `style.display` with class toggling
  (F-8)
- `web/map/src/render.ts` — replace `style.display` with class toggling (F-8)
- `web/map/src/priority.ts` — replace `style.display` with class toggling (F-8)
- `web/map/src/search.ts` — replace `style.display` with class toggling (F-8,
  if applicable)

Vite's CSS pipeline (`vite.config.ts`) handles `@import` natively; no config
changes expected.

### Module decomposition table

| New file | Source lines (approx) | From style.css sections |
|---|---|---|
| `style.css` | ~10 | Entry: `@import` cascade |
| `tokens.css` | ~80 | Custom Properties (both `:root` blocks merged) + dark mode + `--link` |
| `reset.css` | ~8 | Reset / box-sizing |
| `layout.css` | ~20 | Grid layout + Body |
| `sidebar.css` | ~260 | Search, Filter, Depth, Entity list, Kind pills, Refresh, Focus header |
| `graph.css` | ~110 | Graph area, SVG nodes, Hover pane, Placeholder |
| `markdown.css` | ~100 | Markdown pane, Fullscreen, Toolbar |
| `table.css` | ~100 | Relationship table, Edge detail, Edge legend |
| `concept-map.css` | ~170 | CM authoring UI (all `cm-*` classes) |
| `priority.css` | ~90 | Priority/DAG view |

## Non-Goals

- JS/TS modular decomposition (IMP-085 — separate umbrella)
- Dependency pinning (IMP-086)
- Theme toggle widget (IMP-087)
- Test framework adoption (IMP-088)
- Semantic HTML / ARIA (IMP-089)
- New visual design or behaviour changes
- CSS framework adoption
- Build tooling changes (Vite stays as-is)

## Risks

- **Visual regression.** Splitting and reorganising CSS can silently change
  cascade order or specificity. Mitigation: `@import` order preserves the
  existing cascade; visual comparison gate required before closure.
- **Import order sensitivity.** Vite resolves `@import` in source order. The
  existing file's top-to-bottom order is the canonical cascade; the import
  list must match it exactly.
- **F-8 JS changes.** Replacing `style.display` with class toggling touches
  the TypeScript source — functional behaviour change, not pure CSS refactor.
  Keep scope tight: change only the visibility mechanism, not the logic.

## Verification / Closure Intent

1. `npm run build` (or `bun run build`) succeeds — Vite bundles all CSS
   modules without error
2. `npm run dev` renders the map explorer with identical visual appearance in
   both light and dark modes (before/after visual comparison)
3. Each `@import`d file is self-contained: no undefined custom property
   references within any module (audit via grep for `var(--` without
   corresponding definition in the import chain)
4. No `:root {}` block appears outside `tokens.css`
5. No hardcoded hex colour remains in `concept-map.css` (all promoted to
   `--cm-*` or existing theme tokens)
6. No `style.display` assignment remains in the TypeScript source
7. `.hidden` is renamed to `.u-hidden` or module-specific equivalent
8. `npx eslint` — zero warnings
9. `just check` — root package tests pass (no Rust changes, gate is a
   formality)
10. All RV-065 findings are addressed; each disposition recorded in a closure
    note

## Follow-Ups

- IMP-085 broader JS/TS decomposition (this slice is the CSS subset)
- IMP-086-089 as sequenced from IMP-085
- Adopt a CSS linting tool (Stylelint) as a gate for the modularised files
