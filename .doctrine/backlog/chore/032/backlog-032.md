# CHR-032: Consolidate scattered "dispatch/" branch-prefix magic string to one named constant (STD-001)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

The dispatch coordination-branch prefix `dispatch/` is a magic string repeated at
least at: `src/ledger.rs` (~415/451/1035/1038), `src/worktree/create.rs:240`,
`src/slice.rs` (~5424/5439), `src/state.rs:516` — as `format!("dispatch/{slice:03}")`
and `refs/heads/dispatch/{…}`. STD-001 (required: single-source named constants).

SL-181 introduces `DISPATCH_BRANCH_PREFIX` for its new `is_coordination_worktree`
predicate but deliberately does **not** retro-fit the existing call sites (scope
control). This chore retro-fits them all to the one constant.

Surfaced during SL-181 design. Low risk, mechanical. Refs: STD-001, SL-181.
