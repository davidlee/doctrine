# Notes SL-091: Frontend dev server with TypeScript, HMR, and hot reload

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-05 (completed, 8618a9ec)

- 126 behaviour-contract tests (1559 lines) by test designer → 281 total GREEN
- model.ts 439 lines + state.ts expanded to full AppState
- types.ts: RawEdge.target made `| null` for unresolvable targets
- 4 test assertion fixes by satisfier (test designer errors — see phase sheet)
- import verb bug: `doctrine worktree import` failed with "corrupt patch" — manual pipeline worked
- api.ts still has inlined normalizeConceptMap — dedup deferred
- model.js retained (test.html unaffected)

## PHASE-06 (completed, 1b3ceffd)

- render.ts 780 lines — straight conversion from render.js (494 lines)
- Module-specific types: GraphPaneOpts, RelationshipTableOpts, EdgeDetailOpts, RenderedElements
- setViewMode → setPageMode per design contract
- render.js retained

## PHASE-07 (completed, 979276bf)

- search.ts 265 lines — straight conversion from search.js (170 lines)
- search.js retained

## PHASE-08 (completed, ae015026)

- concept-map.ts 335 lines — converted from concept-map.js (198 lines)
- Module-specific types: CmDiagramOpts, CmEdgeTableOpts, CmAddEdgeFormOpts
- concept-map.js retained

## PHASE-09 (completed, 0564d6e0)

- priority.ts 229 lines — converted from priority.js (205 lines)
- d3-dag v1.2.1 exports `graphStratify` not `dagStratify` (design.md has `dagStratify`)
- priority.js retained

## Known issues (carried forward)

- **import verb**: `doctrine worktree import` fails with "corrupt patch at <stdin>:723" — use manual `git diff B..fork | git apply --3way --index` pipeline
- **api.ts dedup**: normalizeConceptMap still inlined at api.ts line 55 — import from model.ts when convenient

## PHASE-10 (completed, 002de17d)

- Converted 171-line app.js IIFE to 925-line app.ts ES module.
- `setViewMode` renamed to `setPageMode` (render.ts conversion).
- `window.renderCmDiagnostics` → `export function renderCmDiagnostics()`.
- `priority.renderGraph` TS API differs from JS: `view` param (not `layout`),
  no hover callbacks, no zoom toggle (existing priority.ts shape).
- `CmDiagramOpts.cm` typed `ConceptMap` but receives `CmNeighbourhood` —
  `as unknown as ConceptMap` cast as workaround (no concept-map.ts changes).
- All innerHTML assignments avoided via `el()` + `replaceChildren()` to satisfy
  eslint `no-restricted-syntax`.
- app.js still present, all .js files retained.
- 281/281 tests green; tsc + eslint zero errors/warnings.

## PHASE-11 (completed, 7db19296)

- test.html converted to `<script type="module">` with explicit ES imports.
- Baseline: 219 assertion labels captured via last-quoted-arg extraction.
- First baseline attempt corrupted (captured expected values not labels for
  assertEqual) — fixed with paren-matched extraction.
- padId defined inline (not exported by model.ts).
- window.renderCmDiagnostics set from app.ts import.
- All 8 bare `<script src="/assets/X.js">` tags removed.
- test.html served by Vite at /test.html.
- No .js files deleted.

## PHASE-12 (completed, bf008bbd)

- All 10 old .js source files deleted: api.js, app.js, concept-map.js, dot.js,
  model.js, priority.js, render.js, router.js, search.js, svg.js.
- 4 vendor bundles deleted: d3-dag.min.js, d3.v7.min.js, markdown-it.min.js,
  purify.min.js.
- web/map/style.css already gone (moved to src/ in PHASE-00).
- Only eslint.config.js remains as web/map/*.js.
- vendor/ contains only README.md.
- tsc + eslint + 281 tests green after deletion.
- Verified: find web/map -maxdepth 1 -name '*.js' → only eslint.config.js.

## PHASE-13 (completed, 0fb88607)

- assets.rs: single `#[folder]` → `#[cfg_attr(debug_assertions...)]`/
  `#[cfg_attr(not(debug_assertions)...)]` for debug/release folder selection.
- .gitignore: added web/map/dist/ and web/map/node_modules/.
- All cargo tests green (0 failures, workspace-wide).

## PHASE-14 (completed)

- test-baseline.txt removed (build artifact, not needed post-verification).
- No stale files remaining. web/map/ layout: config files, index.html,
  test.html, src/*.ts, public/vendor/github-markdown.css, dist/ (gitignored),
  node_modules/ (gitignored).
- Memory recorded: web frontend dev workflow (mem_019ed651a1e273c2).
- IMP-088 (test framework) acknowledged as deferred — test.html has no
  structured runner or CI exit code.
