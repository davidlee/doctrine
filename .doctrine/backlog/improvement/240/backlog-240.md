# IMP-240: Solo-fork audit path: fail-fast review new, runbook or state-carry

## Problem

Auditing a **solo fork** (a `/slice`-driven feature branch worked in a worktree,
NOT `/dispatch`) has no clean path. Two distinct defects, both surfaced during the
SL-192 audit (RV-238):

1. **Mint-then-strand trap.** `review new` *succeeds* on a fork, but `review
   raise` / `dispose` / `status` *refuse* it — the review baton lives in
   parent-tree gitignored state (IMP-024). So an agent mints an RV on the fork and
   is immediately unable to drive it: a stranded, un-driveable entity. The verbs
   disagree about whether a fork is a valid review context.

2. **No solo-fork audit analog.** A dispatched slice audits against a published
   candidate branch (`dispatch candidate create/status`). A solo fork has none.
   The baton needs the parent tree, but the parent tree lacks BOTH the code AND
   the completed-phase runtime state (`.doctrine/state` is fork-local/gitignored,
   so the parent shows `0/N`). This forces a **land-first + manual phase-state
   reconstruction** workaround (land the fork to edge, replay `slice phase …
   completed` ×N, `slice status … audit`) before the audit can run — undocumented,
   error-prone, and it lands unreviewed code before the review exists.

## Fix directions (pick during design)

- **(1)** `review new` should **refuse on a fork** — fail fast, symmetric with
  `raise`/`dispose` (IMP-024). Or make the baton fork-reachable so all review
  verbs work on a fork. Either removes the mint-then-strand asymmetry.
- **(2)** Either **document a solo-fork audit runbook** (the land-first ritual as
  a sanctioned path), provide a **candidate-analog for solo forks**, or **carry
  phase-state on land** so the parent tree reflects the fork's completed phases
  without manual replay.

## Provenance

SL-192 audit RV-238 — process friction (recorded, not an SL-192 defect). Detail
in RV-238 `## Synthesis` "Standing items → Process friction" and RFC-011
case-notes `[audit; SL-192-audit-238]`. Related: IMP-024 (baton parent-tree
residency, the root of defect 1).
