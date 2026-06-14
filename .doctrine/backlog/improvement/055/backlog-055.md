# IMP-055: SL-056 land/import worker-mode tests use bare contains substrings not backtick-delimited verb (proxy)

Source: RV-016 finding F-12 (reconciliation review of SL-056), severity minor / follow-up.

## Detail

`tests/e2e_worktree_land.rs:361` (and the import peer) assert the refusal with bare
substring `contains("land")` / `contains("import")`, which match incidental occurrences
(e.g. `island`, `important`, path fragments) rather than the backtick-delimited verb
token. Proxy weakness (`mem.pattern.review.guard-test-asserts-property-not-proxy`).

## Fix

Assert the backtick-delimited verb form (e.g. `` `land` `` / `` `import` ``).
