# ISS-530: Clean candidate leaks worktree and ref on ledger write failure

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`dispatch candidate create --worktree`, clean arm (`src/dispatch.rs`
`candidate_create`): the candidates ledger row is written after the worktree is
created and provisioned. If `write_candidates` fails there, the detached
worktree and the candidate ref are left standing with no row. The conflicted
arm compensates this case (`rollback_conflict_worktree`); the clean arm does
not.

Predates SL-285. SL-285 made the clean arm compensate a *provisioning* failure
(worktree then ref); this later step remains uncovered. Found by the SL-285
PHASE-02 review.
