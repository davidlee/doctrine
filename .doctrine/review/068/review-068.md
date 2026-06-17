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
