# RSK-230: Integrate --edge advance leg is not FF-gated

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The hazard

`dispatch sync --integrate`'s FF property is **not uniform across legs**. The
`--edge` (standing aggregate ref) leg does not fast-forward-gate, at plan time or
at advance time:

- `plan_edge_row` / `plan_candidate_edge_row` (`src/dispatch.rs`) are explicitly
  commented *"Not ff-gated"* — a standing aggregate of local work.
- `advance_row` → `advance_pure_ref` for a not-checked-out target calls
  `git::update_ref_cas` with **no `is_ancestor` check**. The CAS guards
  concurrency only (refuses a *moved* target via `expected_old`), never ancestry.

By contrast the trunk projection rows FF-gate at plan time
(`plan_trunk_row` / `plan_candidate_trunk_row`, `ensure!(is_ancestor(...))`), and
`advance_checked_out` FF-gates at advance time.

**Consequence:** the `--edge` leg can advance the aggregate ref to a tree that is
not a descendant of its current tip — a content/corpus regression with no FF
guard in the way. Only SL-166's **g3** corpus-clobber gate stands between that
and lost authored content, which is what makes g3 load-bearing *now* rather than
future insurance.

Anchor: `mem.fact.dispatch.edge-advance-leg-not-ff-gated`
(`mem_019f0746d1aa76a0bbe20cbf7ad07593`, verified/high). Surfaced as **RV-176
F-2** (blocker) against the SL-166 design, which had falsely claimed both advance
legs were FF-only.

## Why it is filed now

**SL-239** (dispatch trunk promotion + close-time edge gate) declares this an
explicit Non-Goal and promised the follow-up. SL-239 rejected IMP-267's
"auto-merge `main` → `edge` at close" option precisely *because* of this gap: a
gate that refuses and instructs is safe on an un-FF-gated leg; an auto-advance is
not.

## The open question this needs

Not obviously a bug to be fixed. A standing aggregate of local work may be
*intended* to accept non-FF advances — the comment reads as deliberate. So the
design call is: **is non-FF legal on the aggregate leg by intent, or is the
comment covering an unexamined gap?**

- If legal by intent → the guarantee should be *documented* as asymmetric
  (SPEC-022's git interaction model), and g3's load-bearing role stated, so no
  future design repeats RV-176 F-2's false claim.
- If not → FF-gate `plan_edge_row` symmetrically with the trunk rows, and decide
  what the escape hatch is for a legitimately divergent aggregate.

Adjacent posture history: **RFC-006** (resolved) considered reversing the FF-only
posture for *trunk* integrate; this is the mirror question on the aggregate leg.
