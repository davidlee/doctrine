# SL-091 Implementation Plan

## Rationale

The plan converts `web/map/` from plain JS + rust-embed to TypeScript + Vite,
module by module, leaf to root. Each phase is a single-file conversion with
a `tsc --noEmit` gate — no multi-module debugging.

## Sequencing

**PHASE-00 (Scaffold)** sets up the toolchain before any code changes. The
project configs (`package.json`, `tsconfig.json`, `vite.config.ts`) are
created, `types.ts` defines shared interfaces, and `index.html` is updated
to load a single module script. `bun install` fetches dependencies. At the
end of this phase, `tsc --noEmit` passes on an empty app.ts stub, and `bun
run dev` starts Vite.

**PHASE-01 through PHASE-04 (Pure leaf modules)** convert modules with no
DOM or state dependencies: `router.ts`, `api.ts`, `dot.ts`, `svg.ts`. These
are the safest conversions — pure functions or thin wrappers over fetch/DOM
APIs. Each phase deletes its `.js` original after the `.ts` replacement is
verified.

**PHASE-05 (model.ts)** is the critical inflection point. It converts the
`state` singleton and all pure graph functions. This is the largest single
module and the one every other module depends on. Placed after the leaf
modules because they import only types from it, not runtime values.

**PHASE-06 through PHASE-09 (DOM-heavy modules)** depend on model, dot, api,
and svg being converted already. `render.ts` builds DOM, `search.ts` wires
events, `concept-map.ts` renders/edits CMs, `priority.ts` runs D3 layouts.
Each phase adds one module and verifies `tsc --noEmit`.

**PHASE-10 (app.ts + test.html + vendor cleanup)** is the integration phase.
The entry point wires everything together. `test.html` is converted to ES
modules (served via Vite). Old vendor bundles and the root style.css are
removed — npm packages replace vendored libraries.

**PHASE-11 (Rust integration)** changes the rust-embed folder to use
`cfg_attr`: debug embeds `web/map/` (always present), release embeds
`web/map/dist/` (requires `bun run build` first). `.gitignore` adds
`dist/` and `node_modules/`. All existing map server tests must pass.

**PHASE-12 (Cleanup)** removes any remaining stale files, records durable
memory for the new workflow, and reconciles the slice scope.

## Parallelization potential

PHASE-01 through PHASE-04 are file-disjoint and could run in parallel
(router.ts, api.ts, dot.ts, svg.ts share no runtime imports). PHASE-05 is a
hard sequential gate (all later phases import from model). PHASE-06 through
PHASE-09 are also file-disjoint after model is done. PHASE-10 depends on all
prior conversions. PHASE-11 and PHASE-12 are sequential finalization.
