# SL-073 Implementation Notes

## PHASE-06 (in progress — 2026-06-16)

### Edge detail page (EX-1)

- Relationship table label column now renders as `<a>` tag linking to `#/edge/e_…`
- ``render()` dispatches `route.view === 'edge'` to `renderEdgeDetail()`
- `renderEdgeDetail()` renders metadata table (edge id, source (clickable), label, target (clickable), origin_file)
- Back link returns to the current focus view
- Edge not found displays error message
- `state.focusId` is preserved through edge view (not overwritten by edge route)

### --path flag (EX-3)

- Added `#[arg(long)] path: Option<PathBuf>` to `MapServeArgs`
- Wired into `run_serve`: `crate::root::find(args.path.or(path), &crate::root::default_markers())`
- Test `map_serve_path_flag_passed_to_root_find` verifies flag parsing and precedence

### SVG sanitization (EX-4)

- Already satisfied by PHASE-03: DOMPurify with `USE_PROFILES: {svg: true}` strips `<script>`, event handlers, `<foreignObject>`, external URL attributes
- DOT strings are quoted/escaped before server-side Graphviz rendering
- Entity titles with special chars are handled by escapeHtml (hover pane), dotQuote (DOT), and DOMPurify (SVG)

### Refresh (EX-2)

- Already satisfied by PHASE-05: `wireRefresh()` clears `markdownCache`, increments `graphRenderSeq`, re-fetches, re-resolves focus, re-renders

## PHASE-05 (complete — 2026-06-16)

**Commit:** `4bf7a2c` — feat(SL-073): PHASE-05 interactive UI

Executed inline. Replaced old renderList/renderFocus/renderDotEditor with
shell-DOM-targeted interactive UI. Wired all sidebar surfaces (filter
checkboxes, search, depth buttons, refresh) and main area panes (focus
header, entity list, relationship table). Uses router.parseHash for
routing, model.normalizeGraph for data, renderGraphPane/renderMarkdownPane
from PHASE-03/04.

**Key design decisions:**
- Kind filter collected from checkbox labels, split by "/" for compound groups
  (e.g. "ADR/POL" → ["ADR", "POL"])
- Search live-filters via model.searchFilter then applies kindFilter on top;
  Enter uses model.findFocus with null-on-miss (Esc clears)
- Relationship table edges filtered by source kind when kindFilter active
- Refresh clears markdownCache, increments graphRenderSeq, re-fetches,
  re-resolves focus
- Bootstrap wires interactive surfaces before fetch; render() uses state.
  dotAvailable from health check

**Removed dead code:** parseHash (old), apiGet/apiGetJSON/apiGetText,
renderList, renderFocus, renderDotEditor, renderDotSvg — replaced by
shell-DOM targeting.

**Watch-outs for PHASE-06:**
--path CLI flag (IMP-079, Rust change in src/commands/map.rs)
- Edge detail page (#/edge/e_…) reachable from relationship table
  edge ID click — currently not wired; relationship table links go to
  focus view
- SVG sanitization edge cases (verify DOMPurify SVG profile)
- 21-item acceptance checklist
- Existing dotAvailable local check in renderGraphPane vs state.dotAvailable
  — now consistent (both read state.dotAvailable)
- `el()` helper still present but unused since shell DOM is static — safe
  to leave; PHASE-06 can remove if desired

## PHASE-04 (complete — 2026-06-16)

**Commit:** `3c4de37` — feat(SL-073): PHASE-04 markdown rendering

Executed inline. Added api.fetchMarkdown and renderMarkdownPane with
cache, stale-request guard (state.focusId comparison), applyLinkPolicy
(external→_blank+noopener, relative→strip, anchor→preserve). Error
states: 404 muted, 501 info, 500 error.

## PHASE-03 (complete — 2026-06-16)

**Commit:** `6b08ac7` — feat(SL-073): PHASE-03 DOT/SVG rendering

Executed inline (dispatch worker failed — worked in-tree instead of forking).
Created dot.js (dotQuote, nodeAttrs, edgeAttrs, graphToDot) with 19-kind
colour→Graphviz mapping. SVG pipeline added to app.js: renderGraphPane
(stale-render guard via graphRenderSeq), wireSvgHandlers (click→setFocus,
hover→renderHoverPane via `<g class="node"><title>` extraction), escapeHtml
helper. Graphviz unavailable state: error + DOT in `<pre>`.

**Verification:** `node --check` pass (api.js, dot.js, app.js). `just check`
green (1406 tests). 16 new unit tests (5 dotQuote, 11 graphToDot).

**Watch-outs for PHASE-04/05:**
- app.js has legacy `parseHash()` (from SL-072) alongside new `router.parseHash()`
  — PHASE-05 reconciliation needed
- Bootstrap still uses old render path; PHASE-05 redesign of render() needed
- `state.dotAvailable` from model.js vs local `dotAvailable` in app.js — bootstrap
  should sync them but current code reads health into local var
- hover-detail pane CSS class names need styling in PHASE-05

**Dispatch failure notes:** Worker spawned via subagent tool did not execute
`/worktree` skill — worked in-tree directly. Result: partial work on wrong
branch, incomplete (missing api.renderDot, index.html dot.js tag). Recovered by
reverting and executing inline. Root cause: Agent tool + worktree skill
integration — worker didn't self-fork per rung-3 contract.

## PHASE-02 (complete — 2026-06-16)

**Commit:** `c75781e` — feat(SL-073): PHASE-02 model + routing layer

Data layer delivered via dispatch funnel (worker in worktree fork, deepseek-v4-pro).
Created api.js (ApiError class, fetchGraph, fetchHealth), model.js (encodePart,
normalizeGraph, resolveFocus, findFocus, neighbourhood, searchFilter, kinds),
router.js (parseHash, buildHash, setFocus, setEdge). Global `state` object declared
in model.js with graphRenderSeq (consumer: PHASE-03).

**Verification:** `just check` green (1394 tests). 100/100 console.assert unit tests
pass in test.html. Funnel: precond clean → S^==B → R-5 clean → apply clean →
verify green → branch-point stationary → committed.

**Key design points carried forward:**
- `encodePart(s)`: non-[A-Za-z0-9-] → `_HH` hex codepoint for URL-safe edge IDs
- `kindPrefix` derived from id by splitting at first digit (`"SL-072"` → `"SL"`)
- Edge target check: `edge.target.Resolved !== undefined` (Unresolved → skip)
- `padId(n)`: `(n < 100 ? (n < 10 ? '00' : '0') : '') + n` — ES5, no padStart
- `findFocus` returns null on miss (no fallback) — Hard Contract verified in tests
- `state` global declared ONCE in model.js; router reads `state.depth` for default

**Watch-outs for PHASE-03:**
- `Array.from` is ES6 — use manual iteration for Map/Set→array conversion
- `Map.forEach` param order is (value, key), not (key, value)
- Router parseHash uses `window.location.hash`; mock in Node tests
- test.html fixture pattern reusable for dotQuote/graphToDot tests in PHASE-03

## PHASE-01 (complete — 2026-06-16)

**Commit:** `4e55b57` — feat(SL-073): PHASE-01 static shell

Static app shell delivered via dispatch funnel (worker in worktree fork). Replaced
`web/map/index.html` and `web/map/style.css` entirely. CSS Grid layout (280px sidebar
+ 1fr main), 19 kind colour custom properties with light/dark theme via
`@media (prefers-color-scheme: dark)`. All 12 kind-group filter checkboxes present,
depth selector (0-3), placeholder content in all panel regions. Vendor scripts
loaded in `<head>`. Kind pills use `var(--kind-*)` backgrounds.

**Verification:** `just check` green (1394 tests). Server verified at
`doctrine map serve --port 3001` — HTML and CSS served correctly from the
RustEmbed bundle. Visual acceptance (VA-1, VH-1) pending human review.

**Watch-outs:**
- Binary at `./target/debug/doctrine` is stale — CARGO_TARGET_DIR redirects to
  `/home/david/.cargo/doctrine-target-jail`. Always use the latter or run through
  `just check` which uses the correct path.
- HEADS UP: `doctrine` CLI is the source of truth for command shapes — the binary at
  `./target/debug/doctrine` may be stale if CARGO_TARGET_DIR is set.
- Layout uses CSS Grid, not flexbox. Verify later phases don't accidentally use
  flexbox for the main layout.
- Sidebar pills use `background: var(--kind-PREFIX)` with color: #fff — text colour
  assignment deferred per design (not AA tested).
