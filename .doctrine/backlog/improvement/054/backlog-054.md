# IMP-054: SL-056 gc orchestrator env-leg test asserts dual-cause token not the named verb (proxy)

Source: RV-016 finding F-11 (reconciliation review of SL-056), severity minor / follow-up.

## Detail

`tests/e2e_worktree_gc.rs:602` — the exhaustive Orchestrator env-set refusal test checks
the dual-cause message token but does NOT assert the refused verb is named
(`fork`/`import`/`land`/`gc`). Per
`mem.pattern.review.guard-test-asserts-property-not-proxy` the test should assert the
PROPERTY (this specific verb refused + named), not the shared token proxy: a regression
that names the wrong verb would still pass.

## Fix

Assert the backtick-delimited verb name per member.
