# ISS-209: cordage denylist tests bake compile-time CARGO_MANIFEST_DIR path, causing spurious rebuilds between jail and non-jail environments

## Context

`crates/cordage/tests/denylist.rs` uses `env!("CARGO_MANIFEST_DIR")` twice (lines
72, 132) to resolve the crate root for a file-walk scan. This bakes the
compile-time absolute path into the test binary.

Inside the jail: `/workspace/doctrine/crates/cordage`
Outside the jail: `/home/david/projects/doctrine/crates/cordage` (or wherever)

This changes cargo's fingerprint → unnecessary recompilation when switching between
jail and non-jail environments. Same class as SL-162 / CHR-014, which was resolved
for the main doctrine crate's `src/` and `tests/` by switching to runtime
`CARGO_MANIFEST_DIR` via `test_support::repo_root()`.

## Gap in existing guard

`tests/e2e_no_baked_paths.rs` (the CHR-014 regression guard) only scans `src/` and
`tests/` directories. It does not scan `crates/cordage/`, so this instance is
missed. The test passes green today despite the violation.

## Fix surface

1. **`denylist.rs`** — replace the two `env!("CARGO_MANIFEST_DIR")` with
   `std::env::var("CARGO_MANIFEST_DIR").expect(…)` at runtime. Cordage is
   zero-dependency (REQ-079) so the helper stays inlined (3 lines).
2. **`e2e_no_baked_paths.rs`** — add `crates/` to the `rs_files` scan to catch
   future regressions across all workspace crates.

## Related

- CHR-014 / SL-162: original fix for main crate's baked paths
- ADR-008: jail build isolation
- `src/test_support::repo_root()`: the runtime resolver pattern to mirror
