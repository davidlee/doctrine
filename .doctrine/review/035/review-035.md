# Review RV-035 — reconciliation of SL-072

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Lines of attack: design conformance across module layout, route handler correctness, error model completeness, browser security policy, test coverage vs design §8 tables.

## Synthesis

SL-072 (Doctrine Map Server) is implemented conformance-clean against its design
(design.md). All 8 phases completed with bounded conventional commits on the
dispatch/072 coordination branch.

**Design conformance verified:**
- Module layout (assets, error, markdown, open, routes, shell, state) matches
design §1.
- All 7 route handlers present with correct error propagation, content types,
and body limits.
- Router consumed by serve() in mod.rs — no dead code.
- Loopback-only binding (127.0.0.1) enforced — no --host flag.
- REQ markdown returns 501 via dedicated MarkdownNotImplemented variant.
- Stderr capped at 8 KiB in CommandFailed.Error response.
- DOT_BODY_LIMIT = 1 MiB, kill_on_drop(true) on dot child.
- Browser security: markdown-it html:false, DOMPurify.sanitize(), SVG via
<img> data-uri (no inline injection).

**Test coverage:**
- 15 route integration tests matching design §8.2 table.
- 4 URL construction tests matching design §8.7.
- 4 focus validation tests.
- 4 fake DotRenderer tests + 2 conditional real tests.
- Error mapping tests per design §8.5.
- Full suite: 1394 tests green, clippy zero, fmt clean, just gate passes.

**Findings dispositioned:**
- F-1 (minor): Map serve missing --path flag → follow-up (auto-detect sufficient).
- F-2 (minor): Health omits dot version string → fix-now (fixed, verified).

**Standing risks:**
- No --host flag (design decision — loopback only).
- REQ markdown returns 501 — needs catalog-owned parent-spec lookup.
- CatalogGraph clone cost on every /api/graph request — negligible for
Doctrine-scale corpus.

**Closure recommendation:** The slice scope is complete and design-conformant.
The health fix (F-2) is committed. No blockers remain. Hand off to /close.
