# structural close-gate: refuse reconcile→done when dispatched code unintegrated

## Context

Backlog IMP-102. Closing a **dispatched** slice depends on a human running
`doctrine dispatch sync --integrate` (close step-3a,
`plugins/doctrine/skills/close/SKILL.md:62`) to project the slice's journaled
refs onto trunk. Nothing structural stops `slice status <id> done` from
succeeding when that step was skipped or failed — the journaled code can sit
unintegrated while the slice is marked terminal. The teeth today are skill prose,
not the binary.

SL-121 fixes the *other* face of this hub (`git-ref-vs-worktree-placement`): it
makes `sync --integrate` itself leave clean repository state. IMP-102 is the
**structural backstop** — even with a clean integrate, a human who forgets to run
it must not be able to mark the slice `done`. Belt to SL-121's suspenders.

The close seam already carries two reverse-gates in `slice::run_status`
(`src/slice.rs:354`), both one-way shell→query couplings ADR-001 permits (the
queried module never imports `slice`):

- **blocker scan** (`src/slice.rs:374`) — `review::unresolved_blockers_for`,
  fires on both closure-seam crossings.
- **drift gate** (`src/slice.rs:394`) — `undischarged_drift`, fires on
  `reconcile → done` only.

This slice adds a **third** gate of the same shape: a dispatch-integration query
on the `reconcile → done` crossing.

## Scope & Objectives

1. **Integration query.** A narrow `dispatch`-layer function answering: *is this
   slice's journaled trunk-row OID an ancestor of trunk?* Returns a three-state
   answer — integrated / not-integrated / **not-dispatched** (no journal ref ⇒
   gate is silent; solo slices are unaffected). Reuses the journal read
   (`read_ledger::<Journal>` / the `run_show_journal_trunk_oid` logic,
   `src/dispatch.rs:135`) and `git::is_ancestor` (`src/git.rs:958`); share the
   inner logic with the existing show verb rather than duplicating it.

2. **Third close-gate.** In `run_status`, on the `reconcile → done` crossing only
   (mirroring the drift gate, leaning on the sole-seam-crosser invariant), call
   the query and **refuse** with a named, actionable token when the slice is
   dispatched-but-unintegrated. Composes with the existing two gates — any one can
   independently refuse.

3. **Layering.** Query lives in `dispatch` (or lower), called from the `slice`
   shell — same direction as the blocker/drift gates; `dispatch` must not import
   `slice`. No new cycle (ADR-001).

## Non-Goals

- **Changing `sync --integrate` behaviour** — that is SL-121's surface. This slice
  only *reads* integration state, never mutates trunk.
- **Auto-integrating at close.** The gate refuses and instructs; it does not run
  integrate for the user (ADR-006 orchestrator-sole-writer posture; close prose
  stays the actor).
- **Gating non-dispatched (solo) slices** — no journal ⇒ no gate.
- **Other crossings.** Only `reconcile → done`; not `audit → reconcile`, not any
  back-edge.

## Open Questions

- **OQ-1 — trunk ref source.** Hardcode `refs/heads/main`, or read a configured
  trunk (close passes `--trunk refs/heads/main` explicitly)? Default needed if
  unconfigured. → `/design`.
- **OQ-2 — failure-closed vs open on a malformed/absent journal.** A dispatched
  slice whose journal is unreadable: refuse (fail-closed, consistent with the
  dirty-tree refusal philosophy) or treat as not-dispatched? → `/design`.

## Summary

(to fill at close)

## Follow-Ups

(to fill at close)
