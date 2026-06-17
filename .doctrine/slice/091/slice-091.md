# Frontend dev server with TypeScript, HMR, and hot reload

## Context

The `web/map/` frontend is embedded in the Rust binary via `rust-embed`
(`src/map_server/assets.rs`, `#[folder = "web/map/"]`). Every frontend change
requires a full `cargo build` — no hot reload, no TypeScript type checking.
Debug cycles are unnecessarily slow (~20 min wasted on trivial JS errors that
`tsc` would have caught instantly). The workflow breaks the fast feedback
loop that modern frontend tooling provides.

The JS files (~10 modules under `web/map/`) use a legacy pattern: IIFE-wrapped
scripts that attach to `window`, `/* global */` comment-dependency annotations,
and no module system. A bundler + TS migration is the natural cleanup path.

## Scope & Objectives

1. **Vite dev server** with HMR for `web/map/` — changes are visible in
   milliseconds, no Rust rebuild needed.
2. **TypeScript migration** — convert the JS files to `.ts` with strict type
   checking. Vite handles `.ts` → `.js` transpilation automatically.
3. **Module system** — replace `/* global */` + `window.X` pattern with ES
   module `import`/`export`. Vite bundles for production.
4. **Rust proxy** — in dev mode, the Rust map server proxies `/assets/*` and
   `/vendor/*` requests to the Vite dev server. In production, Vite-build
   output is embedded via `rust-embed` (or served as static files from a
   `dist/` directory).
5. **Keep the API surface identical** — no changes to HTML structure (`id`s,
   `class`es) or REST API routes. The frontend is a drop-in replacement from
   the Rust server's perspective.

## Non-Goals

- Rewriting the frontend logic — semantics stay the same.
- Changing the map server's REST API routes.
- Adding a CSS preprocessor (PostCSS, Sass, etc.) — CSS stays plain unless
  Vite's default CSS HMR is desired.
- Test framework or DOM testing (that's IMP-088).

## Affected surface

- `web/map/` — all JS files converted to `.ts`, new build config files added
- `src/map_server/assets.rs` — add dev-mode proxy or conditional embedding
- `src/map_server/routes.rs` — possibly add a catch-all proxy route in dev mode
- `Cargo.toml` — possibly add a feature flag (`dev-server`) or conditional
  compilation
- New files: `web/map/package.json`, `web/map/tsconfig.json`,
  `web/map/vite.config.ts`

## Risks & assumptions

- **Dev-only feature** — the Vite proxy must be gated behind a compile-time
  feature flag (`cfg(feature = "dev-server")`). Production builds must not
  depend on a running Vite process.
- **Vendor library types** — d3, d3-dag, markdown-it, DOMPurify need
  `@types/*` or ambient declarations. d3-dag may lack types entirely.
- **Global state pattern** — the current `state` object and `window.*`
  globals are shared across all modules. Migration to ES modules must
  preserve this shared state (export a singleton `state` object, import
  it where needed).
- **Node.js required for dev** — Vite needs Node.js available in the dev
  environment. The nix flake may need to add `nodejs`.
- **rust-embed debug-embed** — the current `debug-embed` feature embeds on
  every debug build regardless. The Vite proxy must bypass this. We may
  use `#[cfg_attr(feature = "dev-server", exclude)]` patterns or a
  compile-time path switch.

## Verification / closure intent

- `npm run dev` starts Vite, HMR works (edit a `.ts` file, browser updates
  instantly).
- `npm run build` produces production output in `dist/`.
- `tsc --noEmit` passes with zero errors.
- Rust binary with `--features dev-server` proxies to Vite; without the
  feature, embeds the `dist/` output.
- All existing map server integration tests pass unchanged.
- Manual smoke test: navigate the graph, search, filter, toggle views,
  edit concept maps — all function identically.

## Follow-Ups

- IMP-085 (code-quality hardening) becomes easier after TS migration.
- IMP-088 (test framework) benefits from the module system.
- IMP-086 (vendor SRI hashes) should pin the Vite-bundled versions.
