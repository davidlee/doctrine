# CHR-173: Repair vacuous tests surfaced by RV-411

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Tests that pass whatever the code does. Found by RV-411 (`doctrine show RV-411`,
each finding carries file:line evidence). Fix ISS-500 first; it is the one
where vacuity hid a live bug.

| RV-411 | unit | gist |
|---|---|---|
| F-1 | memory.rs | draft-expiry test checks only the date helper (ISS-500) |
| F-7 | e2e_doctor_golden | superset test runs zero assertions on a clean corpus |
| F-9 | boot.rs | governance-filter negatives search slugs the table never prints |
| F-10 | priority/graph.rs | optionality/mint-order fixtures use the removed `slices` label; fossil rung helpers |
| F-11 | e2e_review_golden, e2e_claude_install | early-return worker skips on a dead premise (32/48 empty green) |
| F-12 | memory.rs | `paths_*` copy `run_paths`; validate tests assert their own setup |
| F-15 | review/tests.rs | prime-recompute test can't see a stale value; `clears_concluded` negative rows untested |
| F-19 | git.rs, e2e_design_state | tautological disjunction; derive-only tests; pure parsers untested |

Done means: each named test fails under the mutation its RV finding describes.
Check that by hand-mutating in a scratch copy, or with cargo-mutants if adopted.
