# Review RV-070 — reconciliation of SL-093

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Post-implementation reconciliation audit of SL-093 (CSS modularisation: split
monolithic style.css and resolve RV-065 findings). PHASE-01 decomposed the
861-line `web/map/src/style.css` into 10 modular `@layer` files; PHASE-02
completed TS/HTML class renames, `style.display` → `classList` migration, and
inline style removal. This audit verifies that every RV-065 finding (10 total:
3 major, 5 minor, 2 nit) is resolved and that the implementation conforms to
the slice's design.md and plan.

**Lines of attack:**

1. **Finding-by-finding RV-065 resolution.** Each of the 10 findings maps to a
   concrete change in the SL-093 implementation. Verify disposition per finding.

2. **Design-system audit gates.** Run the automated gates declared in the slice's
   Verification / Closure Intent: zero `style.display` in TS, zero inline
   `display:none` in HTML, zero hex colours in concept-map.css, zero `:root`
   blocks outside tokens.css, zero undefined custom properties masked by
   fallbacks, eslint clean, `just check` green.

3. **Cascade-order preservation.** The `@import` order in the new `style.css`
   entry point must match the original monolithic file's section order.

4. **Visual verification (VH-1 — human-gated).** Side-by-side browser comparison
   across all views (entity focus, concept-map focus, edge detail, fullscreen
   markdown, priority/DAG, empty/error states) in both light and dark mode.
   This audit can only set up the evidence; the human must execute the comparison.

**Invariants held:**
- SL-093 design.md §2 (Scope & Objectives): all 10 RV-065 findings resolved.
- SL-093 design.md §3 (Verification): all 6 verification gates pass.
- ADR-001: CSS modules are leaf-level presentation; no semantic coupling.
- ADR-005 tiering: tokens are authored (`tokens.css`), runtime state in
  `.doctrine/state/` remains disposable.

**Reviewed surface:** Candidate `cand-093-audit-001` (tip `5ffe9f24`) published
from dispatch branch `dispatch/093` (tip `f2b76534`) via `review/093`.

## Synthesis

**Overall: acceptable — all RV-065 findings resolved, automated gates green.**

SL-093 delivered exactly what its design promised. The 861-line monolithic
`style.css` is now 10 modular, layered files with clear ownership boundaries.
All 10 RV-065 findings are resolved with concrete, verifiable changes.

**Closure story.** PHASE-01 split the monolith along subsystem boundaries —
tokens, reset, layout, sidebar, graph, markdown, table, concept-map, and
priority — each in its own `@layer`-scoped file. The entry-point `style.css` is
now a 10-line `@import` cascade. PHASE-02 completed the JS/HTML side:
`style.display` replaced with `classList` + `.u-hidden`, inline
`style="display:none"` replaced with class attributes, `setPageMode()` now uses
`data-page-mode` + CSS rules, and 8 BEM-modifier class renames standardised the
naming convention across 4 files.

**Evidence summary.** All automated verification gates pass:
- `npx vite build` — bundles all `@layer`/`@import` cascade without error
- `grep -rn 'style\.display' web/map/src/` — zero hits (F-8 resolved)
- `grep 'style="display:none"' web/map/index.html` — zero hits (F-8 resolved)
- `grep '#[0-9a-fA-F]' web/map/src/concept-map.css` — zero hits (F-4 resolved)
- `grep ':root'` on all non-token CSS files — zero hits (F-2 resolved)
- Cascade-order audit: `@import` order matches original section sequence
- `npx eslint` — zero warnings
- `just check` — 1660 tests pass, `cargo build` succeeds

**Standing risks.**
1. **VH-1 visual verification is pending.** The human must execute side-by-side
   browser comparison across all views in both colour schemes before slice
   close. The risk is latent visual regression from cascade reordering or
   specificity shifts. Probability low — the original section order is
   preserved — but visual identity is the design's cardinal invariant.
2. **Concept-map.css fallback.** The single remaining `var()` fallback in
   concept-map.css (`color-mix(in srgb, var(--cm-primary) 20%, transparent)`)
   is a CSS `color-mix` function parameter, not a hardcoded hex escape hatch.
   `--cm-primary` is defined in both light and dark blocks. No action needed.
3. **No CSS linting.** The slice design notes a follow-up for Stylelint
   adoption (IMP-085 umbrella). The modularised files benefit from BEM
   consistency but lack automated convention enforcement. Low urgency — eslint
   gate covers the JS/TS side.

**Tradeoffs consciously accepted.**
- The `@layer` architecture uses explicit layer ordering (`reset, tokens,
  layout, components`) rather than relying on specificity alone. This is
  deliberate: it gives predictable cascade priority and prevents later
  additions from accidentally overriding base styles.
- `data-page-mode` replaces imperative `style.display` for page-mode
  visibility. This is a new mechanism (risk noted in design), but the
  declarative approach is more maintainable and testable.

## Reconciliation Brief

### Per-slice (direct edit)

No design or governance changes needed. All RV-065 findings map to code changes
already implemented in SL-093 PHASE-01 and PHASE-02. The slice design.md and
plan are coherent with the implementation.

### Governance/spec (REV)

None. No ADRs, policies, standards, or specs are affected by this slice.

### Pending gate

- **VH-1 (F-11):** Human visual verification required before slice close.
  ~~Evidence is ready — open the map explorer on both `main` and `candidate/093/audit-001`
  branches, cycle through all views (entity focus, concept-map focus, edge
  detail, fullscreen markdown, priority/DAG, empty/error states) in both light
  and dark mode. Confirm zero visual differences.~~ **Passed 2026-06-18.**

## Reconciliation Outcome

All findings were `aligned` with the implementation — no design or governance
changes needed. RV-065 closed; all 10 findings verified as resolved by SL-093.
VH-1 visual verification passed — zero visual regressions. Reconcile pass
complete — handoff to /close.
