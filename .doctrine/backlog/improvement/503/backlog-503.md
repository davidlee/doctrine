# IMP-503: Slow e2e tests over the live corpus and network

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

RV-411 F-6, F-8, F-20 (`doctrine show RV-411`). These tests produce nearly all
of the suite's slow tail (tests over 1 s):
- F-6: e2e_codex_install runs a real networked `npx skills add` on every call
  (about 24 s serial), though no assertion depends on it. Use `boot install`
  plus a fake `npx` on PATH.
- F-8: e2e_show_equivalence takes 21.6 s against the live corpus and pins live
  entity ids, and its unit guard asserts only `is_some()`. Make the unit route
  test exact, and run e2e over a one-entity-per-kind fixture.
- F-20: e2e_claude_install's blast-radius sweep walks gitignored node_modules
  twice. Use `git ls-files` and a single pass.
- F-7 (the doctor tests, which also run on the live corpus) is fixed under
  CHR-173.

Related: IMP-196, a lint that flags goldens reading the live corpus. It would
prevent this class of test.
