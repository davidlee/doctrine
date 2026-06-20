# Design: Web map RFC entity visualization

## Current behaviour

The backend (`catalog::scan`, `catalog::graph`, `map_server`) already handles RFC
entities correctly — they are scanned via `integrity::KINDS`, their edges dispatched
via `outbound_for`'s `"RFC"` arm, and they appear in the `/api/graph` JSON. The
frontend (`web/map/src/`) has three gaps:

| Gap | File | Effect |
|---|---|---|
| No `RFC` in `NODE_STYLES` | `web/map/src/dot.ts` | Nodes fall through to `DEFAULT_NODE_STYLE` (grey `#95A5A6`) |
| No `--kind-RFC` CSS variable | `web/map/src/tokens.css` | Sidebar kind pill references undefined variable (transparent) |
| No RFC kind-filter checkbox | `web/map/index.html` | No way to show/hide RFC nodes in sidebar |

## Target behaviour

RFC entities render with:
- **Node fill:** `#7F8C8D` (warm grey — distinct from REC's `#95A5A6`, signals "no governance weight")
- **Font colour:** `#ffffff` (white — legible on warm grey)
- **Shape:** `box` (matching ADR/POL/RV/REC/REV — standard authored-kind shape)
- **Kind pill:** coloured with `--kind-RFC: #7F8C8D`
- **Filter:** toggleable via a new "RFC — Discussions" checkbox in the sidebar filter grid

## Code impact

### `web/map/src/dot.ts` — one line

After the `REC` entry in `NODE_STYLES` (alphabetical position), insert:

```ts
RFC:  { fill: '#7F8C8D', font: '#ffffff', shape: 'box' },
```

### `web/map/src/tokens.css` — two lines

In `:root` (light theme), after `--kind-REV`:
```css
--kind-RFC: #7F8C8D;
```

In `@media (prefers-color-scheme: dark)`, after `--kind-REV`:
```css
--kind-RFC: #7F8C8D;
```

(Warm grey works in both themes — no dark-theme adjustment needed.)

### `web/map/index.html` — one line

In the `.filter-grid`, after the REV checkbox row (the revision/governance-neutral cluster), insert:
```html
<label class="kind-checkbox"><input type="checkbox" checked data-kinds="RFC"> <span class="kind-abbrev">RFC</span> <span class="kind-desc">Discussions</span></label>
```

## Verification

1. **Frontend dev server:** `cd web/map && bun run dev`
2. **RFC entity exists:** `doctrine rfc show RFC-001` (created from proposal #14)
3. **Confirm in browser:**
   - Kind-filter checkbox "RFC — Discussions" is visible in sidebar, checked by default
   - RFC-001 appears in the entity list with a warm-grey kind pill
   - Click RFC-001: DOT graph renders with a warm-grey node box
   - Uncheck RFC checkbox: RFC-001 disappears from entity list (filter works)
4. **Dark mode verification:** toggle OS preference, confirm warm grey remains legible

No backend changes. No test suite changes (the frontend has no test infrastructure for colour/filter values).

## Design decisions

| Decision | Choice | Rationale |
|---|---|---|
| Colour | `#7F8C8D` warm grey | Distinct from REC's `#95A5A6`; muted tone signals "no governance weight"; works in light and dark themes |
| Filter label | "RFC — Discussions" | Matches the existing pattern (`<kind-abbrev>` + `<kind-desc>`), describes RFC's role |
| Shape | `box` | Standard authored-kind shape; no need for distinguishing shape when colour already differentiates |

## Open questions

None. Colour resolved, all three changes are mechanical.

## Affected surface

- `web/map/src/dot.ts` — `NODE_STYLES`
- `web/map/src/tokens.css` — `:root` and `@media (prefers-color-scheme: dark)` blocks
- `web/map/index.html` — `.filter-grid`
