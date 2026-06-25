# Plan — SL-155

## Rationale

Single phase. The work is a tight set of targeted edits across 6 files plus one
new function (~150 lines) in `src/revision.rs`. No phase has a dependency on
another — the one-liners and the list verb are file-disjoint and can be done in
any order. Grouping them into one phase keeps the TDD cycle tight: write tests,
implement, gate, done.

## Sequencing

1. Write tests first (red): drift canary + list rows tests + template test
2. Implement cluster A one-liners (C1-C3, G5a, I1)
3. Add `tags` to `RevDoc`, template, revision TOMLs
4. Build `list_rows` + `run_list` + `RevisionCommand::List`
5. Run `doctrine supersede ADR-012 ADR-004` (G5b)
6. Gate: `just gate`, tests green, lint zero
7. Notes: record implementation notes
