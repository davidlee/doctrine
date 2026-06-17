# Review RV-068 — reconciliation of SL-091

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Review surface:** `candidate/091/review-001` (created from `review/091`
impl-bundle atop `main`). The dispatch coordination branch (`dispatch/091`) and
worker forks are immutable evidence refs; audit runs against this candidate.

**Lines of attack:**

1. **TypeScript strictness** — `tsc --noEmit` must report zero errors across all
   `web/map/src/*.ts` files. Validate that `strict` + `noUncheckedIndexedAccess` +
   `verbatimModuleSyntax` + `exactOptionalPropertyTypes` are all active and clean.

2. **ES module import coherence** — every `import` in `src/*.ts` must resolve to
   a real export at the declared path. Walk the full import graph manually.

3. **Behavioural parity** — the TS modules must preserve exactly the same external
   behaviour as the deleted `.js` files. Vitest contract tests (router, dot, model)
   and test.html smoke run are the primary evidence.

4. **Rust integration** — `assets.rs` `cfg_attr` must select `web/map/` for debug
   and `web/map/dist/` for release. `.gitignore` must exclude `dist/` and
   `node_modules/`. All existing `cargo test` must pass.

5. **Design conformance** — verify D1 (Vite proxy, no Rust feature flag), D2
   (functional-core/imperative-shell), D3 (leaf→root conversion order), D4 (bun),
   D5 (d3-dag from npm), D6 (no framework) are faithfully realised.

6. **Cleanup completeness** — PHASE-12 must be total: no stale `.js` source files,
   no vendor bundles, no `web/map/style.css` root copy.

7. **Known gaps** — `api.ts` line 55 inlined `normalizeConceptMap` (dedup deferred),
   d3-dag exports `graphStratify` not `dagStratify` (design.md D5 says `dagStratify`),
   test.html has no structured runner (IMP-088 deferred). Audit must confirm these
   are real, not introduce additional drift.

## Synthesis

SL-091 delivers a clean TypeScript migration of the `web/map/` frontend with Vite
dev server integration. The implementation faithfully realises all six design
decisions (D1–D6), passes every mechanical verification gate, and preserves
behavioural parity with the deleted legacy JavaScript codebase.

### What went right

- **Mechanical correctness.** `tsc --noEmit` (zero errors), `eslint --max-warnings=0`
  (zero warnings), Vitest (281/281 green), `cargo test -p doctrine` (all green),
  `cargo clippy` (zero warnings), `bun run build` (produces valid `dist/`). The
  full gate is clean.

- **Import coherence.** The ES module graph is internally consistent — every `import`
  in `src/*.ts` resolves to a real export. The `state` singleton is properly
  separated into its own module (`state.ts`) and re-exported by `model.ts`, giving
  clean isolation for Vitest mocking.

- **Rust integration is minimal.** One `cfg_attr` in `assets.rs` switches the embed
  folder between `web/map/` (debug) and `web/map/dist/` (release). No Rust feature
  flags, no route handler changes, no proxy code — D1 satisfied exactly.

- **Cleanup is total.** All 10 `.js` source files and 4 vendor bundles are gone.
  `web/map/style.css` root copy removed. `web/map/vendor/` contains only `README.md`.
  `.gitignore` correctly excludes `dist/` and `node_modules/`.

### Findings dispositioned

Three findings raised, all terminal:

- **F-1 (minor, tolerated):** `normalizeConceptMap` is duplicated between
  `model.ts` (exported, canonical) and `api.ts` (private, inlined). This was a
  conscious deferral — functionally correct, low risk, tracked as known follow-up.

- **F-2 (nit, fix-now):** Design.md D5 said `dagStratify` but d3-dag v1.2.1 exports
  `graphStratify`. Design text corrected to match reality.

- **F-3 (minor, tolerated):** `index.html` retains a static `/assets/style.css`
  `<link>` that 404s in release builds (the CSS is correctly served via Vite's
  hashed bundle link). Cosmetic only — styling works correctly. Accepted as minor
  drift.

### Standing risks

- **test.html has no structured runner.** IMP-088 (test framework) is deferred.
  Manual `<pre>` inspection is the only verification path. No CI exit code, no
  automation. This is an acknowledged, honest gap.

- **d3-dag API surface.** The npm d3-dag v1.2.1 API was confirmed to match the
  vendored bundle behavior. No behavioural regression risk.

- **Vite hashed asset paths.** The existing `/{*path}` wildcard route in
  `src/map_server/routes.rs` handles arbitrary asset paths, so hashed bundle
  names are served correctly. Verified by `bun run build` producing `dist/`
  with the expected structure.

### Conclusion

SL-091 is ready for reconciliation and closure. The implementation is clean,
verification is comprehensive, and the three findings are all minor/acknowledged.
No blockers.

## Reconciliation Brief

### Per-slice (direct edit)

- **design.md D5:** `dagStratify` → `graphStratify` — applied (F-2 fix-now).

### Governance/spec (REV)

None — no governance or spec artifacts require changes from this audit.

## Reconciliation Outcome

### Direct edits applied
- **design.md D5:** `dagStratify` → `graphStratify` (RV-068 F-2 fix-now). Applied
  during audit; design now matches npm d3-dag v1.2.1 API.

### REVs completed
None — no governance or spec items in the reconciliation brief.

### Tolerated
- **RV-068 F-1:** normalizeConceptMap duplication — conscious deferral, tracked as
  known follow-up.
- **RV-068 F-3:** index.html /assets/style.css static link — cosmetic 404 in
  release builds. Styling correctly applied via Vite hashed CSS bundle.

Reconcile pass complete — handoff to /close.
