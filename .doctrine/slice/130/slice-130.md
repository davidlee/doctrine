# Web map: RFC entity visualization

## Context

The RFC kind was introduced by SL-122 (governed by ADR-014) as a first-class,
governance-neutral discussion artifact. The backend corpus scan (`integrity::KINDS`,
`outbound_for` dispatch, `catalog::scan`) already covers RFC entities — they are
scanned as part of every catalog hydration and would appear in the `CatalogGraph`
data served by the map server's `/api/graph` endpoint.

However, the web map frontend (`doctrine map serve`, implemented in `web/map/src/`)
has no RFC-specific rendering support:

- **No `NODE_STYLES` entry** in `web/map/src/dot.ts` for `RFC` — falls through to the
  gray default `DEFAULT_NODE_STYLE` (poor visual differentiation from REC/fallback).
- **No `--kind-RFC` CSS variable** in `web/map/src/tokens.css` — the entity list
  kind pill has no colour assigned; the inline `background: var(--kind-RFC)` would
  render as an unset CSS variable (transparent).
- **No kind-filter checkbox** in `web/map/index.html` — RFC entities are invisible
  to the sidebar filter UI; a user cannot toggle RFC nodes on/off.
- **No edge-legend entry** for RFC's `related` edge usage — RFC edges use the
  existing `related` label (already in the legend), so the edge-colour gap is
  minimal, but the node colour is entirely absent.

No RFC entities exist in the project yet (no `rfc/` directory on disk), so the
visual gap has not been noticed. But the moment someone creates an RFC, it will
render with a fallback gray fill — indistinguishable from the `REC` kind at a
glance.

## Scope & Objectives

Add RFC to the web map's visual identity, matching the pattern established by every
other authored kind (ADR, POL, RV, REV, CM, etc.):

- **`web/map/src/dot.ts`**: add `RFC: { fill, font, shape }` to `NODE_STYLES`.
  Choose a distinct colour that fits the existing palette. (`RFC` is
  governance-neutral deliberation — orange/amber-adjacent would risk confusion
  with `PRD/SPEC`; a distinct teal, or a muted purple distinct from `ADR/POL`'s
  purple, or a warm grey.)
- **`web/map/src/tokens.css`**: add `--kind-RFC: <colour>` to both light and dark
  themes.
- **`web/map/index.html`**: add a kind-filter checkbox row for `RFC` in the
  filter grid, keeping the existing grouping pattern.

The backend requires no changes: the catalog scan, graph projection, and API
endpoints already serve RFC nodes and edges correctly.

## Non-Goals

- No changes to the Rust backend (catalog, graph, API, or map server).
- No changes to concept-map rendering.
- No changes to the actionability view or priority graph.
- No RFC-specific properties or special-casing beyond visual identity (no
  special legend section for RFC, no custom status colours, no RFC-specific
  tooltip content).
- No RFC entity migration or creation.
- No tests for the front-end changes (the web map has no front-end test suite;
  manual verification suffices for colour/checkbox additions).

## Affected Surface

- `web/map/src/dot.ts` — one new `RFC` key in `NODE_STYLES`.
- `web/map/src/tokens.css` — `--kind-RFC` in `:root` (light) and
  `@media (prefers-color-scheme: dark)` (dark) blocks.
- `web/map/index.html` — one new `<label class="kind-checkbox">` row in the
  `.filter-grid`.

## Risks / Assumptions / Open Questions

- A1: The selected colour will be reviewed by the user before this slice closes.
  Colour choice is a design call, not a correctness issue — changeable in a
  follow-up edit.
- OQ-1: What colour for RFC? Candidates for discussion during design:
  - Teal-green (`#1ABC9C` — distinct from `RV`'s `#1ABC9C`? Actually `RV` already
    uses that exact hex; use a different teal or green).
  - Warm grey (`#7F8C8D` — distinct from `REC`'s `#95A5A6`? Close but workable
    with a colour shift).
  - Muted coral / salmon (`#E74C3C` — distinct from `ISS`'s `#C0392B`).
  - Soft yellow (`#D4AC0D` — distinct but may need dark-mode text contrast).
  - Sky blue mid-tone (`#5DADE2` — distinct from `SL`'s `#4A90D9`).
  The design phase should resolve this with a concrete proposal.

## Verification / Closure Intent

- `doctrine map serve` renders a seeded RFC entity with the chosen colour,
  visible in the entity list (kind pill coloured) and in the DOT graph
  (node fill/outline matching the style table).
- The kind-filter checkbox for RFC appears in the sidebar and toggles RFC
  nodes in the entity list.

## Follow-Ups

- (None anticipated — this is a narrow visual integration.)

## References

- SL-122 — RFC kind: first-class discussion artifact.
- ADR-014 — RFC: governance-neutral first-class kind, precursor to Revision.
- `web/map/src/dot.ts` — `NODE_STYLES` table.
- `web/map/src/tokens.css` — CSS custom properties.
- `web/map/index.html` — kind-filter grid.
