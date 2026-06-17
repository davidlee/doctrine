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
   → Promote to theme-level tokens alongside `--border` and `--bg`.
   `--border-light` (light: `#e0e0e0`, dark: `#444444`);
   `--bg-card` (light: `#fafafa`, dark: `#2a2a2a`). Remove all fallbacks.

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
   → Two mechanisms for two use cases: generic show/hide → `.u-hidden`
   utility class (15 TS sites + 5 inline HTML); page-mode visibility →
   `data-page-mode` attribute on `.layout` + CSS rules (6 TS sites in
   `setPageMode()`).

   **F-9 (nit).** Compound selectors like `.markdown-pane.fullscreen .markdown-body`
   encode DOM hierarchy that will break on any refactor.
   → Replace with flat modifier classes: `.markdown-body--fullscreen`,
   `.cm-diagnostics-panel__title`, `.kind-pill--active`. Preserve
   `:last-child` pseudo-class (is robust CSS, not a DOM-structure coupling
   issue).

   **F-10 (minor).** `.hidden` class name is dangerously broad — a utility-class
   collision magnet if any CSS framework ever enters.
   → Rename to a scoped utility: `.u-hidden` or use module-specific toggle
   classes.

3. **Preserve visual identity.** No visual regressions. The map explorer must
   render identically in light and dark modes after the refactor. Test by
   visual comparison (before/after screenshots or side-by-side browser tabs).

### Affected surface

- `web/map/src/style.css` — deleted; replaced by 10 modular files (see table below)
- `web/map/src/app.ts` — no import change (still `import './style.css'`);
  replace `style.display` toggles with `.u-hidden` classList operations (F-8)
- `web/map/src/concept-map.ts` — replace `style.display` with `.u-hidden`
  classList (F-8); replace `style="display:none"` in generated HTML string;
  update `.cm-diagnostics-panel h3` → `.cm-diagnostics-panel__title` (F-9)
- `web/map/src/render.ts` — replace 6 `style.display` assignments in
  `setPageMode()` with `data-page-mode` attribute + CSS rules (F-8); update
  class name renames (F-5, F-9)
- `web/map/src/search.ts` — rename `.nav-highlight` →
  `.entity-item--nav-highlight` (F-5)
- `web/map/index.html` — replace 4 `style="display:none"` inlines with
  class `u-hidden` (F-8)

Vite's CSS pipeline handles `@layer` and `@import` identically in dev and
prod; no config changes expected.

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

- **Cascade order sensitivity.** The `@import` order in `style.css` is the
  canonical cascade. Verified by matching section comment headers in each
  layout sub-file against the original — the order is identical.
- **Unlayered style escalation.** Any CSS rule outside a declared `@layer`
  block beats all layered rules per spec. Mitigation: every rule lives in an
  explicit layer block.
- **Visual regression.** Splitting CSS into layers can expose latent
  specificity assumptions. Mitigation: side-by-side visual comparison gate
  (all views, both colour schemes).
- **JS class name changes.** 9 class renames across CSS and TS — TypeScript
  won't catch `classList` string mismatches. Mitigation: visual smoke test
  covers all interactive states.
- **`data-page-mode` is a new mechanism.** Replaces 6 imperative
  `style.display` assignments in `setPageMode()` with declarative CSS rules.
  Functional equivalence verified by visual comparison.

## Verification / Closure Intent

1. `npm run build` — Vite bundles all `@layer`/`@import` cascade without error.
   `npm run dev` confirms HMR resolves layers identically.
2. Cascade-order audit: diff original section headers against `@import` order
   in new `style.css` — identical sequence.
3. Visual comparison — side-by-side browser tabs, each view in both light and
   dark mode: entity focus, concept-map focus, edge detail, fullscreen
   markdown, priority/DAG view, empty/error states. Zero visual differences.
4. Design-system audit:
   - `grep -n 'var(--.*,.*)' web/map/src/*.css` → no fallbacks
   - `grep -n '#[0-9a-fA-F]' web/map/src/concept-map.css` → zero hits
   - `grep -n ':root'` on all non-token CSS files → zero hits
   - `grep -rn 'style\.display' web/map/src/` → zero hits
5. `npx eslint` — zero warnings
6. `just check` — root package tests pass (no Rust changes)

## Follow-Ups

- IMP-085 broader JS/TS decomposition (this slice is the CSS subset)
- IMP-086-089 as sequenced from IMP-085
- Adopt a CSS linting tool (Stylelint) as a gate for the modularised files
