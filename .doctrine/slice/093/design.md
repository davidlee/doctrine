# Design: CSS modularisation (SL-093)

## Architecture

### Layers

```
@layer reset, tokens, layout, components;
```

| Layer | Priority | Contents | File(s) |
|---|---|---|---|
| `reset` | lowest | `box-sizing: border-box` | `reset.css` |
| `tokens` | — | All custom properties + dark-mode overrides | `tokens.css` |
| `layout` | — | Body, grid, sidebar, main, search, filter, depth, entity-list, kind-pills, refresh, focus-header, graph/SVG, hover, markdown, fullscreen, tables, edge detail, legend, utilities, page-mode visibility | `layout.css`, `sidebar.css`, `graph.css`, `markdown.css`, `table.css` (all `@import`ed into the same `layout` layer, cascade order preserved by import sequence) |
| `components` | highest | Concept-map authoring + priority/DAG view | `concept-map.css`, `priority.css` |

Custom properties (`tokens` layer) resolve globally regardless of layer, so
`tokens` needs no cascade priority relative to `layout`/`components`. Rules in
`components` always win over `layout` regardless of specificity. Within a
layer, normal specificity applies — the `@import` order of layout sub-files
preserves the original cascade exactly.

New slices add CSS by appending an `@import` to the appropriate layer in
`style.css` — no existing files touched.

### File structure

```
web/map/src/
  style.css          ← entry point (~15 lines)
  tokens.css         ← all custom properties (~80 lines)
  reset.css          ← box-sizing (~8 lines)
  layout.css         ← grid + body + main + page-mode visibility (~65 lines)
  sidebar.css        ← search, filter, depth, entity-list, kind-pills, refresh, focus-header (~260 lines)
  graph.css          ← graph area, SVG nodes, hover pane, placeholder (~110 lines)
  markdown.css       ← markdown pane, fullscreen, toolbar (~100 lines)
  table.css          ← relationship table, edge detail, legend (~100 lines)
  concept-map.css    ← CM authoring UI (~140 lines)
  priority.css       ← priority/DAG (~65 lines)
```

### Entry point (`style.css`)

```css
@layer reset, tokens, layout, components;

@import './reset.css' layer(reset);
@import './tokens.css' layer(tokens);
@import './layout.css' layer(layout);
@import './sidebar.css' layer(layout);
@import './graph.css' layer(layout);
@import './markdown.css' layer(layout);
@import './table.css' layer(layout);
@import './concept-map.css' layer(components);
@import './priority.css' layer(components);
```

`app.ts` unchanged — `import './style.css'` still resolves through Vite.

### Cascade contract

- Rules in `components` always win over `layout` regardless of specificity.
- Within the `layout` layer, `@import` order is the canonical cascade — identical
  to the original top-to-bottom file order. Each sub-file's position is
  verified by matching its section comment headers to the original.
- Every CSS rule lives in an explicit `@layer` block. No bare selectors outside
  a layer — this prevents unlayered style escalation (per CSS spec, unlayered
  beats all layers).
- `!important` is banned — if a component needs to beat layout, it already
  lives in the higher layer.
- No `:root {}` block appears outside `tokens.css`.
- No `@media (prefers-color-scheme: dark)` block appears outside `tokens.css`.

### Vite processing note

Vite resolves CSS `@import` and `@layer` identically in dev (HMR-injected) and
prod (bundled). No config changes needed. Verified: Vite 5+ supports `@layer`
in both modes; `@import url() layer(name)` is part of the CSS Cascading and
Inheritance Level 5 spec, shipped in all modern browsers.

## Naming convention

| Scope | Convention | Examples |
|---|---|---|
| Layout | Flat, descriptive | `.sidebar`, `.search-input`, `.graph-area` |
| Layout states | BEM `--modifier` | `.doctrine-node--focus`, `.entity-item--active`, `.depth-btn--active` |
| Components | Mandatory prefix | `cm-*`, `priority-*` |
| Component states | Prefix + `--modifier` | `.priority-node--focus`, `.priority-node--hover` |
| Utilities | `u-` prefix | `.u-hidden`, `.u-sr-only` |

Class renames (zero visual impact):

| Current | New | Affected files |
|---|---|---|
| `.hidden` | `.relationship-table--hidden` | `table.css`, `app.ts`, `render.ts` |
| `.entity-item.active` | `.entity-item--active` | `sidebar.css`, `render.ts` |
| `.depth-btn.active` | `.depth-btn--active` | `sidebar.css`, `render.ts` |
| `.view-btn.active` | `.view-btn--active` | `table.css`, `app.ts`, `render.ts` |
| `.nav-highlight` | `.entity-item--nav-highlight` | `sidebar.css`, `search.ts` |
| `.markdown-pane.fullscreen .markdown-body` | `.markdown-body--fullscreen` | `markdown.css`, `render.ts` |
| `.cm-diagnostics-panel h3` | `.cm-diagnostics-panel__title` | `concept-map.css`, `concept-map.ts` |

**Not renamed** (robust CSS, not DOM-structure coupling):
- `.cm-diag-item:last-child` — `:last-child` is a CSS pseudo-class that
  automatically tracks DOM changes; it is not a brittle compound selector.
- `.entity-item.active .kind-pill` — becomes `.kind-pill--active` on the pill
  element itself (set by JS when the entity-item becomes active), eliminating
  the descendant selector.

## Custom property token taxonomy

All tokens live in `tokens.css` within a single `:root` block + single
dark-mode `@media` block. Organised as:

```css
@layer tokens {
  :root {
    /* Kind palette (22 kinds) */
    --kind-SL: #4A90D9;
    /* ... (20 more kind tokens unchanged) */
    --kind-REV: #A04000;

    /* Theme (light) */
    --bg: #ffffff;
    --fg: #1a1a1a;
    --muted: #6b6b6b;
    --border: #e0e0e0;
    --border-light: #e0e0e0;   /* was undefined; promoted from CM fallback */
    --bg-card: #fafafa;         /* was undefined; promoted from CM fallback */
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
      --border-light: #444444;
      --bg-card: #2a2a2a;
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

### Design decisions

- **`--border-light` and `--bg-card` are theme-level, not CM-specific.** They
  were undefined ghost tokens with CM hardcoded fallbacks. Rather than
  narrowing them to `--cm-card-border`/`--cm-card-bg`, they are promoted to
  proper theme tokens alongside `--border` and `--bg`. Any future component
  needing a lighter border or card background uses these tokens. The CM
  section consumes them as `var(--border-light)` and `var(--bg-card)` — no
  fallbacks needed.
- **CM palette tokens (`--cm-*`) are component-specific.** They capture
  semantic colours unique to the concept-map authoring surface (error,
  success, warning states). These are not theme-level because they have no
  meaning outside the CM UI. If another component later needs an error state,
  a cross-component error palette should be designed then — not now.

## `style.display` → class-based toggling

Two distinct use cases, two mechanisms:

### 1. Generic visibility toggles → `.u-hidden`

Define utility in `layout.css`:

```css
.u-hidden { display: none; }
```

| Site count | Files | Context |
|---|---|---|
| 6 | `concept-map.ts` | `renderEdgeTable`, `renderDiagnostics`, `renderAddEdgeForm` — show/hide CM panels |
| 6 | `app.ts` | Legend toggle (priority ↔ entity legend), error banner, diagnostics placeholder |
| 5 inline | `index.html` (4) + `concept-map.ts` generated HTML (1) | `style="display:none"` → class `u-hidden` |

Replace `style.display = 'none'`/`'block'` with `classList.add('u-hidden')` /
`classList.remove('u-hidden')`.

### 2. Page-mode visibility → `data-page-mode` + CSS

`setPageMode()` in `render.ts` controls element visibility based on which view
is active (entity-graph, actionability, concept-map, edge). These are semantic
page-mode switches, not generic show/hide toggles. Using `.u-hidden` loses the
semantic intent.

**Mechanism:** Set `data-page-mode` on the `.layout` root element in TS; CSS
rules handle visibility.

```css
/* Page-mode visibility — in layout.css */
.layout[data-page-mode="edge"] .depth-selector,
.layout[data-page-mode="edge"] .relationship-table,
.layout[data-page-mode="edge"] .table-toggle { display: none; }

.layout[data-page-mode="concept-map"] .relationship-table,
.layout[data-page-mode="concept-map"] .table-toggle { display: none; }
```

```typescript
// render.ts — setPageMode
export function setPageMode(mode: 'entity-graph' | 'actionability' | 'concept-map' | 'edge'): void {
  const layout = document.querySelector<HTMLElement>('.layout');
  if (layout) layout.dataset.pageMode = mode;

  // CM containers: hide AND clear when leaving concept-map mode (CM-specific logic)
  if (mode !== 'concept-map') {
    elements.cmEdgeTable?.classList.add('u-hidden');
    if (elements.cmEdgeTable) elements.cmEdgeTable.innerHTML = '';
    elements.cmAddEdgeForm?.classList.add('u-hidden');
    if (elements.cmAddEdgeForm) elements.cmAddEdgeForm.innerHTML = '';
    elements.cmDiagnosticsPanel?.classList.add('u-hidden');
    if (elements.cmDiagnosticsPanel) elements.cmDiagnosticsPanel.innerHTML = '';
  }
}
```

| Site count | File | Context |
|---|---|---|
| 6 | `render.ts` `setPageMode()` | Page-mode visibility for depthSelector, relationshipTable, tableToggle, cmEdgeTable, cmAddForm, cmDiagnosticsPanel |
| 3 | `app.ts` (lines 709, 723, 727) | Error banner visibility (generic toggle — `.u-hidden`) |

**Total display manipulation sites: 21 TS + 5 inline HTML = 26.**

### Summary

| Mechanism | Sites | Files |
|---|---|---|
| `.u-hidden` classList toggle | 15 TS + 5 inline | `concept-map.ts`, `app.ts`, `index.html` |
| `data-page-mode` + CSS | 6 TS | `render.ts` `setPageMode()` |

## Pre-existing coupling acknowledged

| Issue | Source | Disposition |
|---|---|---|
| `data-key` attributes in concept-map.ts | SL-076 | Pre-existing DOM coupling (same fragility class as F-9 compound selectors). Out of scope for this slice. |

## RV-065 findings resolved

| Finding | Resolution |
|---|---|
| **F-1** (major) — Monolithic file | Split into 10 files with `@layer` boundaries: `tokens.css`, `reset.css`, `layout.css`, `sidebar.css`, `graph.css`, `markdown.css`, `table.css`, `concept-map.css`, `priority.css`, `style.css` (entry) |
| **F-2** (major) — Second `:root` block | Consolidated into single `tokens.css` `:root` block |
| **F-3** (major) — Undefined `--border-light`/`--bg-card` | Promoted to theme-level tokens; defined with dark variants. All fallbacks removed. |
| **F-4** (minor) — 18 hardcoded hex colours | 15 `--cm-*` tokens defined in `tokens.css`; remaining 3 (`--border-light`, `--bg-card`, `--cm-input-border`) use existing or promoted theme tokens. Zero raw hex in `concept-map.css`. |
| **F-5** (minor) — Inconsistent naming | Prefix-scoped with BEM-modifier states per naming convention table above |
| **F-6** (minor) — Dark mode gaps | Every CM/priority token has a dark variant in `tokens.css` `@media` block |
| **F-7** (nit) — Scattered `--link` | Moved into canonical `:root` block with dark variant |
| **F-8** (minor) — `style.display` vs class toggling | All 26 display manipulations replaced. Generic toggles → `.u-hidden`; page-mode visibility → `data-page-mode` + CSS. |
| **F-9** (nit) — Compound selectors | `.markdown-body--fullscreen`, `.cm-diagnostics-panel__title`, `.kind-pill--active` — flat, composable. `:last-child` preserved (is robust CSS). |
| **F-10** (minor) — `.hidden` too broad | `.relationship-table--hidden`; `u-` utility prefix prevents framework collision |

## Risks

- **Cascade order sensitivity.** The `@import` order in `style.css` is the
  canonical cascade. Verified by matching section comment headers in each
  layout sub-file against the original `style.css` — the order is identical.
- **Unlayered style escalation.** Per CSS spec, any rule outside a declared
  layer beats all layered rules regardless of specificity. Mitigation:
  contract: every CSS rule in the project lives in an explicit `@layer` block.
  No bare selectors outside a layer.
- **Visual regression.** Splitting CSS into layers can expose latent
  specificity assumptions. Mitigation: visual comparison gate before closure
  (all views and states listed in Verification).
- **JS class name changes.** Renames in CSS must match renames in TS;
  TypeScript will catch some but not all mismatches (classList strings are
  opaque). Mitigation: visual smoke test covers all interactive states.
- **`data-page-mode` approach is new mechanism.** The `setPageMode` refactor
  introduces a `data-page-mode` attribute on `.layout`. This replaces 6
  imperative `style.display` assignments with declarative CSS rules.
  Functional equivalence is verified by the visual comparison gate.

## Verification

1. `npm run build` — Vite bundles all `@import` cascade without error.
   `npm run dev` confirms HMR resolves `@layer`/`@import` identically.
2. Cascade-order audit: diff original `style.css` section headers against
   `@import` order in new `style.css` — identical sequence.
3. Visual comparison — side-by-side browser tabs (before build vs after),
   each view in both light and dark mode:
   - Entity focus: `#/focus/SL-072?depth=2` — sidebar, graph, hover, markdown, table
   - Concept map focus: `#/focus/CM-001?depth=2` — CM diagram, edge table, add form, diagnostics
   - Edge detail: `#/edge/e17`
   - Fullscreen markdown: toggle fullscreen on entity focus
   - Priority/DAG: actionability view
   - Empty/error states: search miss, server unreachable
   Zero visual differences expected.
4. Design-system audit:
   - `grep -n 'var(--.*,.*)' web/map/src/*.css` → no fallbacks (all tokens defined)
   - `grep -n '#[0-9a-fA-F]' web/map/src/concept-map.css` → zero hits
   - `grep -n ':root' web/map/src/layout.css web/map/src/reset.css web/map/src/sidebar.css web/map/src/graph.css web/map/src/markdown.css web/map/src/table.css web/map/src/concept-map.css web/map/src/priority.css` → zero hits
   - `grep -rn 'style\.display' web/map/src/` → zero hits in TS
5. `npx eslint` — zero warnings
6. `just check` — root package tests pass (no Rust changes)

## Open questions

- None. All design decisions are locked.
