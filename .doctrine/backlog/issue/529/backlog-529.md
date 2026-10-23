# ISS-529: Fork rollback misses binding on non-dispatch branch

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`worktree fork --worker --slice N --phase PHASE-NN` binds a dispatch record
keyed by the fork **name** derived from the directory (`coord_and_name(dir)`,
`src/worktree/fork.rs` `run_fork`), before the worktree act. On a later failure,
`rollback_fork` deletes the record only when the **branch** strips the
`dispatch/` prefix (`fork.rs` `rollback_fork`, step 1b). A bound fork on any
other branch name — or one whose branch suffix differs from its dir name —
strands the record.

Predates SL-285 (SL-228 PHASE-04). SL-285 widens exposure: provisioning now
regenerates the derived tier, a new post-bind failure mode. Found by the
SL-285 PHASE-02 review (pi review profile). Repair: pass the bound name into
the rollback rather than re-deriving it from the branch.
