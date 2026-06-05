# Stale CARGO_BIN_EXE makes e2e tests spawn-fail

Integration tests (e.g. `tests/e2e_memory_anchoring.rs`) embed
`CARGO_BIN_EXE_doctrine` at **compile time**. Jail and host bind the repo at
different absolute paths (`/workspace/doctrine` in the bwrap jail vs
`/home/david/dev/doctrine` on the host) but historically shared one `target/`.
So a jail-built e2e test binary carried the jail mount path and spawn-failed
(`NotFound`) when the same binary was run on the host — and vice versa. Not a
real failure; environmental (mount path), not the code under test.

Structural fix (SL-016 follow-up, flake.nix `jailEnvOptions`): the jail sets
`CARGO_TARGET_DIR=/home/david/.cargo/doctrine-target-jail` (in-jail HOME appears
as /home/david, backed by the persisted out-of-tree host `/home/agent/.cargo`).
Jail builds into its own target dir; host keeps default `target/`. No shared
artifacts → no cross-mount clobber. Effective on jail relaunch (first build cold).

Fallback (older builds, or if the dirs ever share again) — force a test-target
rebuild so the path is recaptured in the current mount:

```
touch tests/*.rs
cargo test            # or: just check
```

- Symptom: `e2e_*` tests fail with spawn/`NotFound`, while `--bin doctrine`
  unit tests pass.
- Cause is environmental (mount path), not the code under test.
