# Review RV-098 — reconciliation of SL-110

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Review surface.** SL-110 was driven by `/dispatch` (5 phases, claude arm,
sole-writer). Audit runs against the **candidate interaction branch**
`candidate/110/review-001` (`cand-110-review-001`), base `main@69b34330`, source
`review/110@5d6d0200` — **18 files, +939/−66, tree-identical to the phase tip**.
The evidence refs `dispatch/110` + `review/110` are immutable (R2). The raw
`main..review/110` diff is polluted (main advanced past the fork base
`98e88d64` with SL-117/SL-109), so the candidate is the correct surface.

**What this audits.** Conformance of the 5-item implementation to the locked
`design.md` (2 codex passes integrated) and `plan.toml`, plus the
behaviour-preservation gate.

**Evidence gathered.**
- `just check` (candidate worktree): `cargo fmt` + `cargo clippy` **clean**.
- eslint `web/map/`: **0 warnings** (node_modules symlinked from main tree —
  fresh worktree had none; environment artifact, not a code issue).
- vitest: **337 passed / 8 files** (matches the dispatch receipt PHASE-04 count).
- `cargo test --bin doctrine`: **1902 passed, 0 failed, 1 ignored** —
  behaviour-preservation gate holds; relabel_edge's 7 dedicated tests green
  (persist+preserve, key-collision dup, no-op, not-found, empty-field, routes
  200, old/new aliases) covering PHASE-01 VT-1..VT-4.

**Lines of attack / where bodies are buried.**
1. **Item 3 (PHASE-05)** — does the CSS-only change meet design EX-1 ("the 'all'
   row aligns with the per-kind checkbox rows, same inset/grid")? The
   `.filter-toggle-all` lives in `.filter-header` (`justify-content:
   space-between`, right-floated beside the "Filter" label) — a different
   container from the left-aligned `.filter-grid` rows. The change achieves
   intra-row checkbox↔label spacing parity, *not* left-edge column alignment.
   EX-2 says touch markup *if the misalignment is structural* — it is.
2. **VH-1 acceptance** — items 2/3/4/5 carry a dev-server visual VH-1 the tests
   cannot close; the dispatch receipt carried these forward.
3. **Carry-forward /simplify deviations** — local `ViewMode` alias in `model.ts`;
   `highlightViewButtons` exported though design §Item 1 said "module-private";
   PHASE-02 worker exported a test-only surface.

**Code-conformance read (clean).** `relabel_edge_in_dsl` matches the design spec
step-for-step (trim+empty-reject → no-op short-circuit before the dup scan →
label-matched line → key-based dup guard excluding the matched line → rel-segment
rewrite preserving source/target bytes). `focusTransition`/`requiredMode`,
`hoverDetailHtml` (escape-all, `hoverPane` delegates), and the `renderView`
`focusChanged`-gated derive all match D1/D2/D3/D4. Pure/imperative split holds.
