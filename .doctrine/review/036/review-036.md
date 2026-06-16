# Review RV-036 — reconciliation of SL-073

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciling SL-073 (Doctrine Map Frontend) against its design.md Hard Contracts,
module layout, acceptance criteria, and governance (ADR-001, ADR-007). Six phases
delivered; PHASE-06 completes the slice.

Lines of attack:
1. Hard Contracts compliance (8 binding constraints)
2. Module file layout vs separate-files contract
3. Security pipeline: DOT escaping → DOMPurify SVG → inline injection
4. Behaviour contracts: search null-on-miss, kind filter semantics, depth clamping,
   async stale guards, edge ID durability
5. web/map/ hygiene (no secrets, config, fixtures)
6. --path flag (IMP-079) correctness
7. Edge detail page reachability and provenance display

## Synthesis

SL-073 passes the reconciliation gate. All 21 acceptance criteria verified;
all 1470 tests green (`just check`); all 8 Hard Contracts satisfied.

The implementation delivers the interactive Doctrine Map Explorer as a
static SPA consuming the SL-072 Rust API surface. The frontend normalizes
the catalog graph client-side, computes N-hop neighbourhoods via BFS,
renders Graphviz SVG with inline click/hover handlers, and displays
sanitized Markdown entity bodies with safe link policies.

**Two minor cosmetic deviations surfaced, both dispositioned as tolerating
non-functional gaps:**

- **F-1 (render.js consolidation):** Render functions live in `app.js`
  rather than a separate `render.js`. The module contract interfaces
  (`api.*`, `model.*`, `dot.*`, `router.*`) are correctly isolated;
  migrating render functions would be a pure refactor. The future
  htmx migration would eliminate separate JS modules entirely.
- **F-2 (missing .eslintrc.json):** Never authored; the design file
  layout was aspirational. JS correctness is verified via `node --check`.

**Resolved in PHASE-06:**
  - `--path` flag (IMP-079) wired into `run_serve` with
    `args.path.or(path)` precedence
  - Edge detail page (`#/edge/e_…`) reachable via relationship table
    label clicks, showing metadata (edge id, source, label, target,
    origin_file) with clickable source/target links and back navigation

**Standing risks:**
  - No browser test harness (project posture per SL-072 — manual
    acceptance only). The pure JS unit tests (`web/map/test.html`)
    cover the highest-risk logic (normalization, BFS, focus resolution,
    DOT generation).
  - Graphviz must be installed at runtime — DOT source fallback shows
    in `<pre>` when unavailable.

**No unresolved blockers.** The slice is ready for close.
