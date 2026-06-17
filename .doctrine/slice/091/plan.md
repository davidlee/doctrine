# SL-091 Implementation Plan

## Rationale

The plan converts `web/map/` from plain JS + rust-embed to TypeScript + Vite,
module by module, leaf to root. Each phase is a single-file conversion with
a `tsc --noEmit` AND `eslint --max-warnings=0` gate — type safety plus
architectural conformance. No multi-module debugging.

## Test discipline

Three pure-function modules (router, dot, model) use a **test-designer /
satisfier split**: a separate agent designs Vitest behaviour-contract tests
against the old `.js` module's actual outputs (RED), then a different agent
implements the `.ts` module until all tests pass (GREEN).

Behaviour-contract tests encode the module's contract — *"given this input,
this output"* — not its internals. Tests survive refactoring because they
test behaviour, not structure. The designer/satisfier separation keeps tests
honest: the satisfier cannot weaken a test to make sloppy code pass, because
a different agent wrote the test.

Why only these three: they are pure functions with nontrivial logic (BFS,
hash parsing, DOT generation), testable without a DOM. DOM-heavy modules
(render, svg, concept-map, priority) depend on IMP-088 (test framework) and
are verified by `test.html` regression + manual smoke. `api.ts` gains
nothing from mocking fetch — its VA items already cover contract shape.

Tests run via `bun run test` (vitest run). The `build` script gates on
`test` (typecheck → lint → test → vite build).

## Sequencing

**PHASE-00 (Scaffold)** sets up the toolchain before any code changes. The
project configs (`package.json`, `tsconfig.json`, `vite.config.ts`,
`vitest.config.ts`, `eslint.config.js`) are created: tsconfig hardened
beyond `strict` (noUncheckedIndexedAccess, verbatimModuleSyntax, etc.),
eslint flat config with typescript-eslint strictTypeChecked +
stylisticTypeChecked plus LLM-damage-containment rules and
restricted-syntax bans, vitest extending vite config for node environment.
`types.ts` defines shared interfaces, and `index.html` is updated to load
a single module script. `bun install` fetches dependencies including eslint
and vitest packages. At the end of this phase, `tsc --noEmit` AND
`eslint --max-warnings=0` pass on an empty app.ts stub, and `bun run dev`
starts Vite.

**PHASE-01 through PHASE-04 (Pure leaf modules)** convert modules with no
DOM or state dependencies. **PHASE-01 (router.ts)** and **PHASE-03
(dot.ts)** use the test-designer/satisfier split: a separate agent designs
Vitest behaviour-contract tests first (RED), then the satisfier makes them
green. **PHASE-02 (api.ts)** and **PHASE-04 (svg.ts)** are straightforward
conversions — api.ts verified by VA items, svg.ts DOM-only. **Old .js files
are retained** as test scaffold throughout — no .js file is deleted until
PHASE-10, so `test.html` (which references them as bare globals) remains
runnable at every intermediate phase.

**PHASE-05 (model.ts)** is the critical inflection point and uses the
test-designer/satisfier split. It converts the `state` singleton and all
pure graph functions — the largest and most logic-dense module. Behaviour-
contract tests target the nontrivial algorithms: BFS neighbourhood,
normalization, focus resolution, cmNeighbourhood, comparators. Every other
module depends on model.ts; these tests are the migration's strongest
correctness backstop. Old `model.js` is retained for test.html until
PHASE-10.

**PHASE-06 through PHASE-09 (DOM-heavy modules)** depend on model, dot, api,
and svg being converted already. `render.ts` builds DOM, `search.ts` wires
events, `concept-map.ts` renders/edits CMs, `priority.ts` runs D3 layouts.
Each phase adds one module and verifies `tsc --noEmit` + `eslint`. Old .js
files retained throughout.

**PHASE-10 (app.ts + test.html + .js deletion + vendor cleanup)** is the big
integration phase — the only phase that deletes anything. Before any
deletion, test.html assertion pass/fail is recorded as a baseline at the end
of PHASE-09. `app.ts` is converted as the entry point. `test.html` is
converted from bare-global `<script>` tags to `<script type="module">` with
explicit import statements for all converted modules — test logic is
preserved unchanged, only imports replace bare-global references like
`model.xxx()`. Then ALL old .js source files (router.js, api.js, dot.js,
svg.js, model.js, render.js, search.js, concept-map.js, priority.js,
app.js) are deleted in one atomic wave. Old vendor bundles and root
style.css are removed — npm packages replace vendored libraries.
`test.html` is served via Vite at `/test.html` (not embedded in production —
dev artifact only). After conversion, the VT-5 baseline comparison confirms
no test assertions were semantically altered by the migration.

**PHASE-11 (Rust integration)** changes the rust-embed folder to use
`cfg_attr`: debug embeds `web/map/` (always present), release embeds
`web/map/dist/` (requires `bun run build` first). `.gitignore` adds
`dist/` and `node_modules/`. `just gate` must include `cd web/map && bun
run lint` alongside `cargo clippy`. All existing map server tests must pass.

**PHASE-12 (Cleanup)** removes any remaining stale files, records durable
memory for the new workflow, and reconciles the slice scope. **Acknowledges
F-13**: `test.html` has no structured test runner, no CI exit code, no
automation. The agent must manually read a `<pre>` element. IMP-088 (test
framework adoption) is deferred — this is a known, honest gap.

## Parallelization potential

PHASE-01 through PHASE-04 are file-disjoint and could run in parallel
(router.ts, api.ts, dot.ts, svg.ts share no runtime imports). PHASE-05 is a
hard sequential gate (all later phases import from model). PHASE-06 through
PHASE-09 are also file-disjoint after model is done. PHASE-10 depends on all
prior conversions. PHASE-11 and PHASE-12 are sequential finalization.
