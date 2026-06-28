# SL-172 — notes

## Audit (RV-189) — 2026-06-28

Reconciliation audit complete. Reviewed the **candidate interaction branch**
`candidate/172/review-001` (base `main`, merge of `review/172`), admitted and
pinned to RV-189 (`f7627058`). Full suite **2755 + integration green, 0 failed**;
clippy `--workspace` **zero warnings**; both declared e2e goldens green unchanged.

**Findings (all terminal, no blocker):**
- **F-1 (minor, →reconcile per-slice):** `est_cost` signature deviates from
  design §5.2 — landed `Option<(f64,f64)>` + `ctx` by-value, not
  `Option<&EstimateFacet>`. Deliberate: honours NF-001 (graph.rs cost fn must not
  name facet types). Impl correct; design stale → amend design.md §5.2.
- **F-2 (minor, →reconcile per-slice):** design §307-308 over-declared two e2e
  goldens as deliberate-recompute targets; neither moved (green unchanged) →
  correct design prose to verify-unchanged.
- **F-3 (major, →reconcile REV):** owed governance — ADR-015 §1+§2+§4 still
  describe the midpoint model; SPEC-020 REQ-310/FR-011 v1 aggregation deferral not
  yet lifted. Canon lags shipped behaviour until the REV lands (not a blocker by
  design — reconcile's write surface).
- **F-4 (minor, aligned):** core cost-model + config invariants (INV-1/2/3,
  clamps, surface parity) verified green. No change.

Full prose: **RV-189 `## Synthesis` + `## Reconciliation Brief`**.

**Out-of-scope captures:** dispatch-harness incidentals → RFC-011 case-notes;
candidate-worktree missing gitignored embed assets (`web/map/dist`) → CHR-030 +
RFC-011 case-note `[audit; SL-172-RV-189-audit]`.

Next: `/reconcile` (RV-189 brief is the input), then `/close`.
