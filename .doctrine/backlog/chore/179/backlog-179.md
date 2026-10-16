# CHR-179: Shared test git helper

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

From `RV-420` F-11 (audit of `SL-281`). Commit `e523e95c0` lifted the jail
git-guard into four duplicated test git helpers, each repeating the literal
`"GIT_GUARD","off"`: `src/slice.rs`, `src/ledger.rs`, `src/dispatch.rs`,
`tests/e2e_dispatch_candidate.rs`.

## Scope

Extract one shared test git helper (in `test_support` / `tests/common`) with a
named constant for the guard env (STD-001). The guard is host-jail specific —
keep the knowledge in one place (POL-002).
