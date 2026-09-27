# Review-capable worktree locus

## Context

The review verbs refuse any linked worktree that is not a dispatch
coordination tree. `resolve_review_root` (`src/review/turn.rs:169`) asks
`classify_worktree_role` (`src/worktree/shared.rs:77`), which decides by branch
shape alone: `dispatch/<NNN>` is `coord`, every other linked tree is `fork`.
The refusal protects the turn baton — runtime state under `.doctrine/state/` —
which a worker fork cannot co-write because its state tier is withheld
(IMP-024).

Branch shape is a proxy for the real property: **does this tree own its state
tier?** Two kinds of tree own their state but fail the proxy:

- a **solo fork** — a `/slice` branch worked in a `/worktree` (IMP-240
  defect 2, RV-238);
- an **adopted capsule worktree** — a capsule landed with its state half
  adopted into it (RV-398 `F-5`, ISS-494).

Neither can open an audit ledger where the code and phase state are, so the
audit has to land the code on the parent tree first, before any review exists.
SL-268 did exactly that.

Separately, `review new` (`run_new`, `src/review/verbs.rs:79`) skips the guard,
so it mints an RV that every later verb then refuses (IMP-240 defect 1,
ISS-484).

## Scope & Objectives

1. **A doctrine-owned locus declaration.** A tree can be declared to own its
   state tier and so be review-capable, through a contract doctrine owns — not
   inferred from a host's branch names or directory layout (POL-002 facet 1).
   The guard tests the declaration (the property), with `coord` staying
   review-capable. The mechanism is `/design`'s call.
2. **One guard for the whole verb family.** `review new` refuses in the same
   trees the other verbs refuse, before allocating an id (ISS-484).
3. **The audit path from a declared tree works end to end:** open the RV, drive
   it to `done`, then land. What happens to the tree's runtime state on landing
   (phase status, baton) is a design question.
4. **Guidance.** `/audit` and `review-ledger.md` say where an audit can run and
   how a tree becomes review-capable (IMP-190).
5. **Consumer.** `scripts/oubliette.sh back` declares its landing worktree, and
   its advice returns to "audit here" (project-local; validates the contract).

## Non-Goals

- Worker forks co-writing the parent's baton, or parallel raisers (IMP-024).
- Recognising any host branch format (`capsule/`, `feature/`, …) in the engine.
- Changing dispatch coordination trees or `dispatch candidate`.
- Provisioning embed assets in general audit worktrees (IMP-291); the capsule
  path already provisions (ISS-494).

## Summary

Replace the branch-shape proxy with a declared, doctrine-owned property —
"this tree owns its state tier" — so solo forks and adopted capsules can be
audited before landing, and make `review new` obey the same guard.

## Risks & Open Questions

- **R1 — false capability.** A worker fork that declares itself would write a
  baton the parent never sees. The declaration must be refusable in a worker
  fork (worker mode), and fail closed on doubt.
- **OQ-1 — where the declaration lives.** Runtime state in the tree (gitignored,
  per-tree) seems right; authored state would ride the branch into the parent.
- **OQ-2 — landing.** After a declared tree lands, does its phase status and
  review state reach the parent tree, or is close run in the declared tree?
- **OQ-3 — reuse.** Can `worktree coordinate`'s markerless coord role or the
  worker-mode marker be reused, rather than adding a third mechanism?

## Verification

- VT: the guard admits a declared tree, a primary tree and a coord tree; it
  refuses an undeclared linked tree and a worker fork (even if declared).
- VT: `review new` in a refused tree allocates nothing.
- VT: an RV opened in a declared tree runs raise → dispose → verify → conclude
  to `done`.
- VA: audit skill and `review-ledger.md` state the rule; POL-002 check on the
  mechanism by design review.

## Follow-Ups
