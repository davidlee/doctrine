# DEC-094: Review worktree guard tests role, not linkage

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

**Decision.** `review`'s root guard refuses a *worker fork*, not every linked
worktree. A dispatch **coordination** tree may drive a review; its baton lives
in its own gitignored state tree.

## Context

SL-233's plan makes an RV ledger the second half of all three design-gate
entrance criteria, and those sketches live in the coordination worktree because
that is where ADR-012's topology puts a slice's authored work during dispatch.
`resolve_review_root` guarded on `is_linked_worktree`, so every baton-driving
review verb bailed there — stranding the gates in the only tree they can run in,
with no in-tree workaround.

## Alternatives weighed

1. **Route the gate reviews through the primary tree** — author each sketch on
   `edge`, review it there, promote and `refresh-base` it into the coordination
   tree. No code change, all supported seams. Rejected: it pays the round trip
   three times, and it gives a slice's authored gate artefacts a second writer
   outside the coordination tree, against ADR-012's sole-writer intent.
2. **Narrow the guard** (taken). The guard's own rationale — a fork's `WITHHELD`
   tier hides the parent's gitignored state — describes a worker fork and not a
   coordination tree, which is the sole writer of `dispatch/<NNN>` and carries
   its own state tier. So this was a defect, not a missing feature.
3. **Run the adversarial pass informally and transcribe findings later.**
   Rejected without deliberation: the entrance criterion requires the ledger
   terminal *as the gate*, and transcribing after inverts it. plan.md put these
   gates on the authored tier precisely to stop that downgrade.

## What made it cheap

The classification already existed as a pure function — `classify_worktree_role
(branch, linked)`, with the numeric-suffix test that separates a coordination
branch (`dispatch/233`) from a worker fork's `dispatch/<agent>`. It moved to
`crate::worktree` beside `is_linked_worktree`, a home both command-tier callers
already reach (`review` declares a `worktree` edge in ADR-001's layering map),
so no tier moved and nothing was duplicated.

**No state relocation was needed** — the open question going in. `state_dir` is
root-derived, so admitting the tree is the entire fix: the baton lands in the
coordination tree's own gitignored state with nothing to contend over. The
comment calling that a "parent-tree locus" was describing the guard rather than
the mechanism. Baton loss degrades gracefully by design — absent reads as cold
and recomputes (D-C4a).

## Consequences

- Forks still refuse; [[IMP-024]]'s parallel-raiser funnel is untouched.
- A review's baton is now per-invoking-tree. A review continued from a different
  tree after integrate recomputes rather than corrupts, which is the designed
  behaviour, not a new hazard.
- Implemented as a small backlog item rather than a slice, with the user's
  explicit acceptance under the project's small-item rule. See [[ISS-275]].

Settled 2026-07-29 by consult during SL-233 PHASE-02's entrance gate.
