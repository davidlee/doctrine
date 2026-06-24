# Runtime CARGO_MANIFEST_DIR beats env! under shared target

`env!("CARGO_MANIFEST_DIR")` resolves at **compile time** — it bakes the building
tree's absolute path into the binary. Under the jail's shared `CARGO_TARGET_DIR`
(`~/.cargo/doctrine-target-jail`, one cache across all worktrees), cargo's
fingerprint reuses a binary built in tree W when tests run from another tree, so a
`env!`-baked path reads a dead/wrong location once W is reaped
(`read … /tmp/<removed-worktree>/…: No such file`).

**Fact (verified empirically):** cargo also sets `CARGO_MANIFEST_DIR` in the test
process's **runtime** environment, pointed at the **invoking** tree — for both unit
and integration test binaries. So `std::env::var("CARGO_MANIFEST_DIR")` is correct
regardless of which tree compiled the binary.

**How to apply:** in test code, resolve repo-relative paths at runtime, never via the
`env!` macro. Doctrine ships `test_support::repo_root()` (`src/test_support.rs`) for
this — one source, shared into the bin unit tests via `#[cfg(test)] mod test_support`
and into integration tests via a `#[path]` include in `tests/common/mod.rs` (separate
compilation units cannot see `cfg(test)` lib items). The
`e2e_no_baked_manifest_dir` guard bans the macro from creeping back.

This is the fix axis CHR-014 closed — footgun #1 (path-baking). The distinct stale
*artifact* axis is [[mem.fact.build.rebuild-stale-skips-test-binaries]] (IMP-004).
