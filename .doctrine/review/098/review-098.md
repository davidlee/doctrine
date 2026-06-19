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

## Synthesis

**Closure story.** Five findings from the first VH walk: F-1 (item-3 alignment,
fixed `0a9d1e1b`), F-2 (catch-all VH, resolved into F-4/F-5), F-3 (tolerated
/simplify nits). The two blockers of substance — F-4 (item-4 edit model rejected)
and F-5 (item-5 D2) — were resolved by **Revision 2**: design re-authored on main
(`8d0c3dfd` — D4→D5/D6 per-cell pencils + `edit all` scope; D2 reversed) and plan
extended (`b4aed545` — PHASE-06/07/08), then implemented (backend `6c3378f6`,
frontend `3e93e1ac`, D2 reversal `ce11f2cb`). The second VH walk (this pass)
accepted F-4 (1a: edit-all is a pure scope toggle — one-vs-all visible only on a
repeated relation with no node focused; impl conforms to D5/D6) and F-5
(actionability→semantic switch on non-member focus).

That walk surfaced **F-6** — every relationship-table link double-hashed the URL
(`'#' + buildHash(…)` where `buildHash` already `#`-prefixes), so `parseHash`
failed and focus cleared, emptying the table. Pre-existing since SL-091
(`0d4f549e`); only table links broke (graph/list clicks route through `setFocus`,
no extra prefix). Fixed `b0d12a3d` (drop the prefix at five sites + round-trip
test). Scope-expansion into SL-110 was the user's explicit call — it blocked the
item-5 VH and is in-theme with focus navigation.

**Standing risks / tradeoffs accepted.**
- **Edit-all observability (F-4, 1a).** Conscious: D5 makes `edit all` a silent
  scope modifier; its effect is only visible post-submit and is masked by diagram
  label-dedup + neighbourhood filtering. Correct per design; a future affordance
  (affected-count / row highlight) remains a possible UX refinement, not a defect.
- **F-3 nits tolerated** — local `ViewMode` alias + exported `highlightViewButtons`
  (test surface); behaviour-neutral, owned by /simplify.

**Gate state.** SL-110 code gates green on the candidate at `b0d12a3d`: vitest
**340**, tsc, eslint 0, vite build, `cargo` behaviour-preservation (`--bin`, 1902).
`just check` shows ONE unrelated red — `e2e_relation_migration_storage` panics on
the **live main corpus** (`ISS-030`'s `backlog-030.toml` carries a `related` label
on a dep/seq axis that must stay typed). Not SL-110 code, not on this branch;
main-corpus data added in `cb2c96b5`. **Explicitly left untouched per user
decision** — recorded here, not silently absorbed; triage belongs to a separate
backlog item if pursued.

## Reconciliation Brief

### Per-slice (direct edit)
- **None.** `design.md` and `plan.toml` on `main` were re-authored for Revision 2
  (`8d0c3dfd`, `b4aed545`) ahead of implementation — design D5/D6 + reversed D2 +
  PHASE-06/07/08 already match the shipped code. The candidate branch's copies are
  a stale pre-fork snapshot (irrelevant; close merges code, main's authored truth
  is current). No prose drift to reconcile.

### Governance/spec (REV)
- **None.** No ADR / spec / requirement status change. F-6 is a frontend bug fix
  with no design impact (it makes the already-intended navigation work).

### Harvest
- Durable gotcha → memory: `buildHash` returns a `#`-prefixed string; never
  prepend `'#'` to it in an href/hash assignment (the F-6 double-hash trap).
- Process note: SL-091 shipped the latent broken table-link navigation undetected
  (no link round-trip test). The F-6 round-trip test now guards it.

## Reconciliation Outcome

All 6 findings are terminal (`verified`); remediation landed in-flight during
audit/Revision 2, not via reconcile-time writes.

### Direct edits applied
- **None.** `design.md` / `plan.toml` on `main` were re-authored for Revision 2
  (`8d0c3dfd`, `b4aed545`) ahead of implementation; D5/D6 + reversed D2 +
  PHASE-06/07/08 already match the shipped code. No prose drift to reconcile.

### REVs completed
- **None.** No ADR / spec / requirement status change. F-6 is a frontend bug fix
  with no design impact.

### Withdrawn / tolerated
- F-3: tolerated — `/simplify`-owned, behaviour-neutral (local `ViewMode` alias;
  `highlightViewButtons` exported for VT-2). Rationale in finding disposition.

### Harvest deferred to /close
- Durable gotcha → memory: `buildHash` returns a `#`-prefixed string; never
  prepend `'#'` to it (the F-6 double-hash trap).
- Process note: SL-091 shipped the latent broken table-link nav undetected; the
  F-6 round-trip test now guards it.

Reconcile pass complete — no writes to reconciled-truth surfaces required.
Handoff to /close.
