# Review RV-112 — reconciliation of SL-114

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of SL-114 (canonical_id consolidation), self-audit, reviewed
against the **committed source surface** (PHASE-01 landed in `refactor(SL-114)`),
not a dispatch candidate — this slice ran solo, not via `/dispatch`.

**Lines of attack:**
1. Conformance: do the five landed edits match design.md §5.2 exactly (4 delegations
   + spec free-fn delete + 4 repoints), with the surviving signatures unchanged?
2. Behaviour preservation: is every `canonical_id` output byte-identical pre/post?
   The existing id-format tests are the oracle — they must be green *unchanged*.
3. Closure intent (slice §Closure intent): is `listing.rs:37` the **only** surviving
   `format!("{prefix}-{id:03}")` body, with no kind re-implementing it and the spec
   free fn gone?
4. Layering: `requirement → listing` adds no ADR-001 cycle.
5. Evidence integrity: VT-2 claims "full suite green" — interrogate the one red
   (`e2e_worktree_stamp`) and confirm it is external/pre-existing, not slice-caused.

## Synthesis

SL-114 lands clean. The slice did exactly what its design specified and nothing
more: four per-kind `canonical_id` bodies now delegate to the single authority
`listing::canonical_id`, and spec's same-output dual wrapper collapsed to the
method per D1 (free fn deleted, four callers repointed). Conformance (F-1) and
closure intent (F-2) are confirmed against the committed source — the only
surviving `format!("{prefix}-{id:03}")` body in `src/` is the authority at
`listing.rs:37`, and the spec free fn is gone. The change is behaviour-preserving:
the existing id-format tests are green *unchanged*, which is the whole proof for a
consolidation of this shape.

**Standing risk / consciously accepted tradeoff (F-3):** the repo-wide gate
(`just check`) is red on one test — `e2e_worktree_stamp::stamp_provisions_from_primary_when_hook_fires_inside_the_worker`.
This is worktree-provisioning code with zero coupling to `canonical_id`; the
worktree sources are unmodified by this slice and the test is red at committed
HEAD, an ISS-038-class dirty/shared-trunk condition compounded by a concurrent
agent's uncommitted work in this checkout. It is accepted as external noise on
VT-2's literal "full suite" claim, not a defect of SL-114 — minor severity, does
not gate close. The slice's own verification axes (VT-1, closure grep, clippy
zero-warn) are all green.

No design or governance drift surfaced: `design.md` matches the implementation,
`slice-114.md` already carries the scope drift note (8→4 sites) added at design
time, and `plan.toml` PHASE-01 conforms.

## Reconciliation Brief

### Per-slice (direct edit)
- None. `design.md`, `slice-114.md` (incl. its drift note), and `plan.toml` are
  already consistent with the landed implementation. No prose to reconcile.

### Governance/spec (REV)
- None. The slice touched no ADR, spec, requirement, or policy; no governance
  truth changed. No REV required.

**Handoff:** a clean reconciliation — nothing to write through either surface.
`/reconcile` confirms the no-op and hands to `/close`. The one external red (F-3,
`tolerated`/minor) is documented, non-blocking, and belongs to the worktree
workstream, not SL-114.

## Reconciliation Outcome

No-op. The reconciliation brief is empty — no per-slice edit and no REV needed.

### Direct edits applied
- None. `design.md`, `slice-114.md`, and `plan.toml` are already consistent with
  the landed implementation (F-1, F-2 `aligned`).

### REVs completed
- None. The slice touched no ADR/spec/requirement/policy — no governance truth
  changed.

### Withdrawn / tolerated
- F-3 `tolerated` (minor): VT-2's literal "full suite green" carries one external
  pre-existing red (`e2e_worktree_stamp`, ISS-038-class), not slice-attributable.
  Rationale recorded in the finding disposition; non-blocking.

Reconcile pass complete — handoff to /close.
