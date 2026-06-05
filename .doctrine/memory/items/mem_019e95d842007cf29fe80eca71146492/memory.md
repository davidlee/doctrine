# Stale CARGO_BIN_EXE makes e2e tests spawn-fail

Integration tests (e.g. `tests/e2e_memory_anchoring.rs`) embed
`CARGO_BIN_EXE_doctrine` at **compile time**. If the `doctrine` binary was last
built under a different mount path (e.g. `/workspace/doctrine` in a bwrap jail)
the embedded path is stale, and the test spawns it → `NotFound`. Not a real
failure.

Fix — force a test-target rebuild so the path is recaptured in the current mount:

```
touch tests/*.rs
cargo test            # or: just check
```

- Symptom: `e2e_*` tests fail with spawn/`NotFound`, while `--bin doctrine`
  unit tests pass.
- Cause is environmental (mount path), not the code under test.
