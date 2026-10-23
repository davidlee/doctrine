# Review RV-426 — reconciliation of SL-285

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Surface: edge after `worktree land --fork sl-285` (98b7907cd); solo fork, not a
dispatch candidate. Conformance mode.

Lines of attack:
- Code vs design.md (rev 31) sec-1..6, DEC-393/396/397/399/401/402, and notes
  § Reconcile carry (binding deltas decided in-flight, incl. QUE-229 re-tier).
- Every decision consequence has an exercising test (fatal legs, refusals,
  drift, disclosure lines).
- `slice conformance` cells: undeclared paths are scope creep or missing selectors.
- PHASE-01..03 EX/VT/VA/VH met; `doctrine check gate` green on edge.

Evidence: pi `review` (max) conformance pass over the fork (session 6babff1f);
its notes.md view predated the edge-side carry, so claims resting on notes were
re-checked against edge.
