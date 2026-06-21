# Review RV-127 — reconciliation of SL-134

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Audit of SL-134 (Risk facet CLI verb — `doctrine risk set/clear`) against its
design.md, plan.toml, plan.md, and the implemented code on refs/heads/review/134
(commits c0cabc45, c2891a9b).

### Lines of attack

1. **Behaviour-preservation gate.** All existing `facet_write` tests (VT-1 through
   VT-8), estimate tests (VT-8 through VT-12), and value tests (VT-10) must pass
   unchanged. `set_facet_mixed` is additive, not a replacement.

2. **Design conformance.** Every design decision (D1–D8), every verification
   criterion (VT-1 through VT-18), and every non-goal must be reflected in the
   implemented code. The command surface, kind gate, echo format, and at-least-one
   guard must match design.md precisely.

3. **Phase plan fidelity.** PHASE-01 touches only `src/facet_write.rs`. PHASE-02
   touches only `src/commands/facet.rs`, `src/commands/cli.rs`,
   `src/commands/guard.rs`, `src/backlog.rs`. No other files are modified.
   No `.doctrine/` or `.claude/` changes.

4. **Gate quality.** `cargo clippy` zero warnings; all tests pass; `just gate`
   green. The one known pre-existing intermittent test (`sync_produces_all_shipped_dirs`)
   is unrelated to SL-134.

5. **Pure/impure split.** `set_facet_mixed` is pure (no disk, no clock). IO lives
   in `apply_set_mixed`/`edit_in_place`. Command handlers are thin shells.

6. **Forward-compat.** Non-managed facet keys survive writes. Shape-errors on
   non-table present are loud, not silent. Comments are preserved.

7. **No scope creep.** No `risk show`, `risk list`, risk history, or additive
   `controls` subcommands — all correctly left as non-goals. No `FacetField::Float`
   variant (D5).

## Synthesis

SL-134 delivers exactly what it promised: a `doctrine risk set/clear` CLI verb
that mirrors the existing `estimate`/`value` pattern. Two clean commits, five
source files touched, zero regressions.

### Closure story

The shared leaf (`FacetField` + `set_facet_mixed`) was built first under the
behaviour-preservation gate — every existing VT-1 through VT-8 passed unchanged.
The command layer then wired `RiskSetArgs`/`RiskClearArgs`, the kind gate, the
at-least-one axis guard, and full echo matching the estimate/value house style.
All 75 facet-related tests pass (17 original + 5 VT-13–17 + ~53
estimate/value/risk). `cargo clippy` zero warnings. `just gate` green.

The three findings raised are minor/nit and have been dispositioned:

- **F-1 (tolerated):** VA-1/VA-2 CLI help inspection tests are missing from the
  test suite — plan.toml over-specified verification checks that go beyond what
  design.md requires. All design-level VTs are covered; CLI behaviour confirmed
  manually.
- **F-2 (tolerated):** The `ItemKind::ALL` visibility bump (const → pub(crate)
  const) is unused by this slice. Design D7 described `read_kind` as iterating
  `ALL`, but the implementation reads the `kind` string directly — simpler and
  correct. The bump is harmless dead code; reverting it would touch a shared
  module for no functional gain.
- **F-3 (aligned):** VT numbering drift between design.md (VT-12 = invalid token,
  VT-18 = absent table) and plan.toml (VT-12 = absent table). Acknowledged in
  code comments; cosmetic only.

No blockers. No spec or governance findings requiring reconciliation-brief
handoff to `/reconcile`.

### Standing risks

- `set_facet_mixed` duplicates the table-alloc and shape-check logic from
  `set_facet` — future DRY candidate, not a current defect.
- `read_kind` reads the full entity TOML to extract one field — cheap for
  single-use, would benefit from caching in a batch scenario.

### Tradeoffs consciously accepted

- **D5 (no Float variant):** `FacetField` has only `Str` and `Arr`. Adding
  `Float` later if a mixed-type facet needs it. The existing `set_facet`
  continues to serve the float-only estimate/value path.
- **D3 (replace semantics for --controls):** Controls list is replaced on every
  `set` — no additive subcommand. A future improvement can add `risk controls
  add/remove` if needed.
- **D4 (at-least-one axis guard):** Users can assess likelihood and impact in
  separate calls. `exposure()` already treats missing axes as zero.

## Reconciliation Brief

No spec or governance findings were raised. All three findings (F-1, F-2, F-3)
are code/plan-level: two tolerated, one aligned. No design.md, ADR,
`.doctrine/spec/tech/`, or governance artefact requires revision.

The slice is ready for `/reconcile` → `/close` with a clean reconciliation
brief — no REV or per-slice direct edits needed.

## Reconciliation Outcome

Reconcile pass: **no-op.**

All three findings (F-1, F-2, F-3) were either tolerated with rationale or
aligned — no writes to any authored artefact required.

| Finding | Disposition | Rationale |
|---------|-------------|-----------|
| F-1 | tolerated | VA-1/VA-2 are plan-level over-specification; all design VTs covered |
| F-2 | tolerated | `ALL` bump is harmless dead code; reverting touches shared module for no gain |
| F-3 | aligned | VT numbering drift is cosmetic; acknowledged in code comments |

No REV authored. No per-slice direct edits. Slice SL-134 is ready for `/close`.
