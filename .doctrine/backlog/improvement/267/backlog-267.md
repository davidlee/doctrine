# IMP-267: close transition gate: refuse done when unmerged dispatch branches exist

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context (revised)

SL-197's implementation was properly merged into `main` via the dispatch
candidate merge (`bcd93294`). The close metadata commits (audit/reconcile/close)
were written to `edge`. After close, `edge` was never synced with `main` — the
primary worktree (where agents operate) lacked the CPT code that `main` carried.
RV-250 F-1 (original) was incorrect; the correction is documented in the
synthesis.

## What's needed

The `close` (or `done`) transition should verify that the primary worktree
(`edge`) is in sync with `main` after dispatch-integrated work. Options:
- Refuse `done` when `edge` has no path to `main`'s implementation commits
- Auto-merge `main` → `edge` as part of the close sequence
- At minimum, warn if `edge` lacks code changes present on `main`

## Notes

This is less urgent than the original F-1 framing suggested — the implementation
landed correctly on `main`. The gap is an edge/main sync hygiene issue, not a
missing-code emergency.

## Sliced into SL-239 (2026-07-29) — with two corrections

Scoped as objective 2 of **SL-239**, paired with IMP-308 half B (the opposite
direction of the same seam). Two facts established at scope time that revise this
card:

1. **The mechanism already exists — the gap is a missing gate, not missing
   machinery.** `dispatch sync --integrate --edge <ref>` already advances a
   standing aggregate ref to the `review/<slice>` bundle. It is simply *optional
   and unverified at close*. And the gate has a DRY seam: **SL-126** (done) added
   the third close-gate in `slice::run_status` asking *"is this slice's journaled
   **trunk**-row OID an ancestor of trunk?"* (three-state
   integrated/not-integrated/not-dispatched). This card is the **edge-row
   analogue** of that same question — generalise that gate over the row's target
   ref rather than adding a parallel one.

2. **Option 2 above ("auto-merge `main` → `edge` as part of close") is unsafe as
   written and SL-239 does not do it.** Per
   `mem.fact.dispatch.edge-advance-leg-not-ff-gated` (verified/high),
   `plan_edge_row` is explicitly *"Not ff-gated"* and `advance_pure_ref`'s CAS
   guards concurrency, not ancestry — so that leg can already move `edge` to a
   non-descendant tree. A gate that refuses and instructs is safe; an
   auto-advance is not. Also matches SL-126's own Non-Goal (no auto-integrate at
   close) and ADR-006's orchestrator-sole-writer posture. Filed the underlying
   asymmetry as **RSK-230**.

Left open for SL-239's design-lock: **refuse or warn?** SL-126 refuses on the
trunk row because unintegrated code is a correctness failure; an unsynced `edge`
is hygiene (this card's own Notes say so), so refusing may be disproportionate.
