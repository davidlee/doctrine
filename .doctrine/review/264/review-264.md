# Review RV-264 — reconciliation of SL-210

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Facet:** reconciliation. **Mode:** conformance. **Self-audit** (raiser == responder).

**Reviewed surface (F-2 record).** The candidate interaction branch was not
materialised via `dispatch candidate create` (that landing ceremony is /close's
stage-2 job). Audit ran against the immutable impl-bundle evidence ref
**review/210 @ bf3a56efd0f2**, confirmed byte-identical in its authored tier to
the coordination tip **dispatch/210 @ 7f9fd0573943** (design.md + slice-210.toml
diffs IDENTICAL). Review verbs driven from the parent tree (edge); build /
verify-vt / clippy driven from a detached scratch worktree on review/210.

**Lines of attack + invariants held:**
- **Conformance algebra** — does what git touched match the canonical
  `design-target` selectors? (Confounder: edge's slice-210.toml is stale, so
  conformance-from-edge over-reports undeclared — resolved by reading the
  canonical registry directly. F-1.)
- **Design fidelity** — engine (comparison.rs) is the wire model 1:1; append-only
  + lossless + total-order invariants; closed RaterKind/RowForm enums; unknown
  frame/domain STRINGS round-trip; EX-4 clap fallback (`compare record`) recorded
  in canonical design.md.
- **Behaviour-preservation gate** — priority/survey/next/explain bit-identical
  (nothing consumes the ledger yet); VA-1 no golden churn.
- **Independent verification** — build, `slice verify-vt 210` (all six VT), and
  `clippy --workspace` reproduced on the review/210 surface, not trusted from the
  funnel's banked S6 evidence alone.

## Synthesis

SL-210 lands the comparison-ledger capture side of RFC-019 Phase A clean. Three
phases (pure engine → capture verb → list/withdraw/warn) funnel-landed via the
claude dispatch arm; audit reproduced the gate independently rather than
trusting banked evidence:

- **Build** green (doctrine 0.18.1) on review/210 after supplying the gitignored
  derived asset `web/map/dist/` the fresh worktree lacked (an environment
  artifact of the scratch checkout, not an SL-210 defect).
- **`slice verify-vt 210`** — all six VT PASS (PHASE-01/02/03 × VT-1/VT-2),
  reproducing the banked S6 result.
- **`clippy --workspace`** — zero warnings.
- **Conformance** — on the canonical landing surface all seven code paths
  (comparison.rs, commands/{compare,facet,mod,cli,guard}.rs, main.rs) are
  design-target-conformant; zero undelivered. The edge-run undeclared cells are
  a stale-selector artifact (F-1) plus two authored-tier `.doctrine` files that
  are never code selectors.

The implementation is a strict, correctness-preserving **superset** of the
design sketch (fallible `to_toml`, a `session_of_one` constructor centralising
wire-discriminator stamping, stderr for the scoring-inert warn) — no divergence
demanded a code or design edit (F-3). The engine holds its invariants under
test: golden byte-shape pins the wire contract, round-trip covers all-optionals
and bare rows, an unknown-frame row round-trips verbatim, and
`admit_set_is_value_bearing_minus_rsk` pins D6 to the `kinds::` constants for
every census kind. Append-only is enforced structurally — both capture and
withdraw share the single clobber-refusing `create_new` seam and each writes a
fresh session-of-one; withdraw refuses unknown-uid / tombstone-row / double
cases; list renders full uids and marks tombstoned rows display-only.

**Standing risks / consciously accepted tradeoffs:**
- **Edge/coord authored split (F-2).** Canonical design + selectors + ADR-001
  census live only on the coordination surface until stage-2 integrate. This is
  the designed dispatch pre-integrate state, not drift — but until /close runs
  the integrate, any tooling that reads edge's slice-210.toml/design.md sees the
  stale copy. Bounded and closes at integrate.
- **`.doctrine/comparisons/` exists nowhere yet** — created lazily on first real
  capture; tests used temp roots only. No production ledger to migrate.
- **Nothing consumes the ledger** — Phase A is pure capture; the
  behaviour-preservation gate is the proof that scoring is untouched.

No blockers. Ledger done (await=none).

## Reconciliation Brief

Every finding disposed **aligned** — no per-slice direct edit and no
governance/spec REV is required. The reconciled truth was already authored on
the coordination surface during dispatch (design.md EX-4 outcome, remediated
selectors, ADR-001 leaf census); reconcile confirms, it does not write.

### Per-slice (direct edit)
- None. design.md and slice-210.toml are already canonical on
  review/210 == dispatch/210; the edge copies are stale but are superseded, not
  corrected, by stage-2 integrate.

### Governance/spec (REV)
- None. The ADR-001 leaf-census line registering `comparison` is authored on the
  coordination surface and lands via integrate — not a standalone REV.

### Integrate-at-close (F-2 — /close stage-2, NOT a reconcile write surface)
- `dispatch sync --slice 210 --integrate --trunk refs/heads/main` projects the
  coordination authored tree (design.md, slice-210.toml, adr/001/layering.toml)
  and the review/210 impl-bundle onto main. This is the sole remaining write and
  belongs to /close, never to reconcile or to any pre-audit step.

## Reconciliation Outcome

**No-op reconcile.** All three findings are terminal and disposed `aligned` — no
per-slice direct edit and no governance/spec REV is required. The reconciled
truth (canonical design.md with the EX-4 clap-fallback outcome, the remediated
`design-target` selectors adding cli.rs/guard.rs, the ADR-001 leaf census
registering `comparison`) was already authored on the coordination surface
(dispatch/210 == review/210) during dispatch. Reconcile confirms; it writes
nothing.

### Direct edits applied
- None.

### REVs completed
- None.

### Withdrawn / tolerated
- None. All findings `verified` + `aligned` (rationale in each finding's response).

### Deferred to /close (not a reconcile surface)
- **F-2 stage-2 integrate.** `dispatch sync --slice 210 --integrate --trunk
  refs/heads/main` promotes the coordination authored tree + the review/210
  impl-bundle onto main. This is the only remaining write for the slice and is
  /close's job — reconcile does not touch it. Watch-item for /close: edge's
  slice-210.toml now carries `status = reconcile` (bumped this pass); confirm the
  integrate's class-routed projection reconciles the authored slice toml
  (canonical selectors from coord vs. the status progression on edge) rather than
  clobbering one with the other.

Reconcile pass complete — handoff to /close.
