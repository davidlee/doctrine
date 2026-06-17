# Design: CSS modularisation (SL-093)

## Architecture

### Layers

```
@layer reset, tokens, layout, components;
```

| Layer | Priority | Contents | File |
|---|---|---|---|
| `reset` | lowest | `box-sizing: border-box` | `reset.css` |
| `tokens` | — | All custom properties + dark-mode overrides | `tokens.css` |
| `layout` | — | Body, grid, sidebar, search, filter, depth, entity-list, kind-pills, refresh, focus-header, graph/SVG, hover, markdown, fullscreen, tables, edge detail, legend, utilities | `layout.css` |
| `components` | highest | Concept-map authoring + priority/DAG view | `concept-map.css`, `priority.css` |

Custom properties (`tokens` layer) resolve globally regardless of layer, so
`tokens` needs no cascade priority relative to `layout`/`components`. Rules in
`components` always win over `layout` regardless of specificity. Within a
layer, normal specificity applies.

New slices add CSS by appending an `@import` to the appropriate layer in
`style.css` — no existing files touched.

### File structure

```
web/map/src/
  style.css          ← entry point (~10 lines)
  tokens.css         ← all custom properties (~80 lines)
  reset.css          ← box-sizing (~8 lines)
  layout.css         ← shared layout/CSS (~570 lines)
  concept-map.css    ← CM authoring UI (~140 lines)
  priority.css       ← priority/DAG (~65 lines)
```

### Entry point (`style.css`)

```css
@layer reset, tokens, layout, components;

@import './reset.css' layer(reset);
@import './tokens.css' layer(tokens);
@import './layout.css' layer(layout);
@import './concept-map.css' layer(components);
@import './priority.css' layer(components);
```

`app.ts` unchanged — `import './style.css'` still resolves through Vite.

### Cascade contract

- Rules in `components` always win over `layout` regardless of specificity.
- Within a layer, normal specificity rules apply.
- `!important` is banned — if a component needs to beat layout, it already
  lives in the higher layer.
- No `:root {}` block appears outside `tokens.css`.
- No `@media (prefers-color-scheme: dark)` block appears outside `tokens.css`.

## Naming convention

| Scope | Convention | Examples |
|---|---|---|
| Layout | Flat, descriptive | `.sidebar`, `.search-input`, `.graph-area` |
| Layout states | BEM `--modifier` | `.doctrine-node--focus`, `.entity-item--active`, `.depth-btn--active` |
| Components | Mandatory prefix | `cm-*`, `priority-*` |
| Component states | Prefix + `--modifier` | `.cm-edge-row--hidden`, `.priority-node--faded` |
| Utilities | `u-` prefix | `.u-hidden`, `.u-sr-only` |

Class renames (zero visual impact):

| Current | New | Affected files |
|---|---|---|
| `.hidden` | `.relationship-table--hidden` | `layout.css`, `app.ts`, `render.ts` |
| `.entity-item.active` | `.entity-item--active` | `layout.css`, `render.ts` |
| `.depth-btn.active` | `.depth-btn--active` | `layout.css`, `render.ts` |
| `.view-btn.active` | `.view-btn--active` | `layout.css`, `app.ts`, `render.ts` |
| `.nav-highlight` | `.entity-item--nav-highlight` | `layout.css`, `search.ts` |
| `.markdown-pane.fullscreen .markdown-body` | `.markdown-body--fullscreen` | `layout.css`, `render.ts` |
| `.cm-diagnostics-panel h3` | `.cm-diagnostics-panel__title` | `concept-map.css`, `concept-map.ts` |
| `.cm-diag-item:last-child` | `.cm-diag-item--last` (set by JS) | `concept-map.css`, `concept-map.ts` |

## Custom property token taxonomy

All tokens live in `tokens.css` within a single `:root` block + single
dark-mode `@media` block. Organised as:

```css
@layer tokens {
  :root {
    /* Kind palette (22 kinds) */
    --kind-SL: #4A90D9;
    /* ... */

    /* Theme (light) */
    --bg: #ffffff;
    --fg: #1a1a1a;
    --muted: #6b6b6b;
    --border: #e0e0e0;
    --hover-bg: #f5f5f5;
    --sidebar-bg: #f8f9fa;
    --link: #2563eb;

    /* Concept-map palette */
    --cm-primary: #16A085;
    --cm-primary-hover: #13876b;
    --cm-error-bg: #fde8e8;
    --cm-error-fg: #c0392b;
    --cm-error-border: #f5c6cb;
    --cm-success-bg: #e8f5e9;
    --cm-success-fg: #27ae60;
    --cm-success-border: #c8e6c9;
    --cm-warning-bg: #fffdf0;
    --cm-warning-fg: #856404;
    --cm-warning-border: #f0c040;
    --cm-warning-heading: #b8860b;
    --cm-warning-divider: #f5e6a3;
    --cm-btn-text: #ffffff;
    --cm-input-border: #d0d0d0;
    --cm-card-bg: #fafafa;
    --cm-card-border: #e0e0e0;

    /* Priority palette */
    --priority-actionable-bg: #27AE60;
    --priority-actionable-fg: #ffffff;
    --priority-blocked-bg: #E67E22;
    --priority-blocked-fg: #ffffff;
    --priority-terminal-bg: #95A5A6;
    --priority-terminal-fg: #ffffff;
    --priority-needs-edge: #C0392B;
    --priority-after-edge: #E67E22;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      /* Theme overrides */
      --bg: #1a1a1a;
      --fg: #e0e0e0;
      --muted: #9b9b9b;
      --border: #333333;
      --hover-bg: #2a2a2a;
      --sidebar-bg: #141414;
      --link: #60a5fa;

      /* CM overrides */
      --cm-primary: #1ABC9C;
      --cm-primary-hover: #16A085;
      --cm-error-bg: #3a1a1a;
      --cm-error-fg: #e07070;
      --cm-error-border: #6b3030;
      --cm-success-bg: #1a2a1a;
      --cm-success-fg: #4ecb7c;
      --cm-success-border: #2a4a2a;
      --cm-warning-bg: #2a2410;
      --cm-warning-fg: #c0a040;
      --cm-warning-border: #8b6914;
      --cm-warning-heading: #e0c060;
      --cm-warning-divider: #3a3410;
      --cm-btn-text: #1a1a1a;
      --cm-input-border: #444444;
      --cm-card-bg: #2a2a2a;
      --cm-card-border: #333333;

      /* Priority overrides */
      --priority-actionable-bg: #2ECC71;
      --priority-blocked-bg: #F39C12;
      --priority-terminal-bg: #7F8C8D;
      --priority-needs-edge: #E74C3C;
      --priority-after-edge: #F39C12;
    }
  }
}
```

Ghost tokens replaced:
- `--border-light` (undefined, fallback `#e0e0e0`) → `--cm-card-border`
- `--bg-card` (undefined, fallback `#fafafa`) → `--cm-card-bg`

## `style.display` → class-based toggling

Define utility in `layout.css`:

```css
.u-hidden { display: none; }
```

Replace all `style.display` manipulation with `classList` operations on
`.u-hidden`.

### concept-map.ts (6 sites)

```typescript
// Before: container.style.display = 'none'
// After:  container.classList.add('u-hidden')

// Before: container.style.display = 'block'
// After:  container.classList.remove('u-hidden')
```

Toggles `cm-edge-table`, `cm-diagnostics-panel`, `cm-add-edge-form`.

### render.ts (8 sites)

```typescript
// depthSelector, relationshipTable, tableToggle — toggle .u-hidden
// cmEdgeTable, cmAddForm, cmDiagPanel — toggle .u-hidden
```

### app.ts (8 sites)

```typescript
// Legend toggle, error banner, placeholder — toggle .u-hidden
```

### index.html (4 inlines)

```html
<!-- Before -->
<div class="legend-items priority-legend" style="display:none">
<div class="cm-diagnostics-panel" style="display:none;">
<div class="cm-edge-table" style="display:none;">
<div class="cm-add-edge-form" style="display:none;">

<!-- After -->
<div class="legend-items priority-legend u-hidden">
<div class="cm-diagnostics-panel u-hidden">
<div class="cm-edge-table u-hidden">
<div class="cm-add-edge-form u-hidden">
```

### concept-map.ts generated HTML (1 inline)

`renderAddEdgeForm` generates a `<div class="cm-add-error" style="display:none;">` in
its innerHTML string (line 306). Replace with `class="cm-add-error u-hidden"`.

## RV-065 findings resolved

| Finding | Resolution |
|---|---|
| **F-1** (major) — Monolithic file | Split into 6 files with `@layer` boundaries |
| **F-2** (major) — Second `:root` block | Consolidated into single `tokens.css` `:root` block |
| **F-3** (major) — Undefined `--border-light`/`--bg-card` | Replaced by `--cm-card-border`/`--cm-card-bg`, defined with dark variants |
| **F-4** (minor) — 18 hardcoded hex colours | 17 `--cm-*` tokens defined in `tokens.css`; zero raw hex in `concept-map.css` |
| **F-5** (minor) — Inconsistent naming | Prefix-scoped with BEM-modifier states per naming convention table above |
| **F-6** (minor) — Dark mode gaps | Every CM/priority token has a dark variant in `tokens.css` `@media` block |
| **F-7** (nit) — Scattered `--link` | Moved into canonical `:root` block with dark variant |
| **F-8** (minor) — `style.display` vs class toggling | All 22 JS assignments + 4 HTML inlines replaced with `.u-hidden` classList operations |
| **F-9** (nit) — Compound selectors | `.markdown-body--fullscreen`, `.cm-diagnostics-panel__title`, `.cm-diag-item--last` — flat, composable |
| **F-10** (minor) — `.hidden` too broad | `.relationship-table--hidden`; `u-` utility prefix prevents framework collision |

## Risks

- **Cascade order sensitivity.** The `@import` order in `style.css` is the
  canonical cascade. Module content must not depend on position within a layer
  beyond what `@layer` guarantees. Mitigation: all layout rules go in
  `layout.css` as a single flat sequence; no cross-module ordering dependency
  between `concept-map.css` and `priority.css`.
- **Unlayered style escalation.** Per CSS spec, any rule outside a declared
  layer beats all layered rules regardless of specificity. Mitigation:
  contract: every CSS rule in the project lives in an explicit `@layer` block.
  No bare selectors outside a layer. The `:root` blocks in `tokens.css`, the
  utilities in `layout.css`, and every component rule must be wrapped in its
  layer. This is enforced by file structure — each file contains exactly one
  `@layer` block.
- **layout.css size.** At ~570 lines, `layout.css` is a large file within a
  single layer. It preserves the exact cascade order of the original file,
  which has been battle-tested across 4 slices. Intra-layer specificity
  conflicts are resolved by normal cascade — same as today. Splitting further
  into sub-layers (`layout-grid`, `layout-sidebar`, etc.) would add ceremony
  without benefit since these sections don't compete on specificity.
- **Visual regression.** Splitting CSS into layers can expose latent
  specificity assumptions. Mitigation: visual comparison gate before closure
  (all views and states listed in Verification).
- **JS class name changes.** Renames in CSS must match renames in TS;
  TypeScript will catch some but not all mismatches (classList strings are
  opaque). Mitigation: visual smoke test covers all interactive states.
- **`.cm-diag-item--last` requires JS support.** The `:last-child` pseudo-class
  is replaced by an explicit modifier class. `renderDiagnostics` in
  `concept-map.ts` must add `.cm-diag-item--last` to the final `<div>` in its
  generated HTML.

## Verification

1. `npm run build` — Vite bundles all `@import` cascade without error
2. Visual comparison — side-by-side browser tabs (before build vs after),
   each view in both light and dark mode:
   - Entity focus: `#/focus/SL-072?depth=2` — sidebar, graph, hover, markdown, table
   - Concept map focus: `#/focus/CM-001?depth=2` — CM diagram, edge table, add form, diagnostics
   - Edge detail: `#/edge/e17`
   - Fullscreen markdown: toggle fullscreen on entity focus
   - Priority/DAG: actionability view
   - Empty/error states: search miss, server unreachable
   Zero visual differences expected.
3. Design-system audit:
   - `grep -n 'var(--.*,.*)' web/map/src/*.css` → no fallbacks (all tokens defined)
   - `grep -n '#[0-9a-fA-F]' web/map/src/concept-map.css` → zero hits
   - `grep -n ':root' web/map/src/layout.css web/map/src/reset.css web/map/src/concept-map.css web/map/src/priority.css` → zero hits
   - `grep -rn 'style\.display' web/map/src/` → zero hits in TS
4. `npx eslint` — zero warnings
5. `just check` — root package tests pass (no Rust changes)

## Open questions

- None. All design decisions are locked.
