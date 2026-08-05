# design_run tests may not name `crate::` either

`src/design_run/` is a leaf with crate out-degree **zero** — it names `crate::`
nowhere (`legacy.rs:335` states the rule). The reason is not style: five
integration binaries `#[path]`-include the whole tree standalone —

```rust
#[path = "../src/design_run/mod.rs"]
mod design_run;
```

— into a test crate where `crate::` is *that* crate, not the doctrine binary.

**The trap.** The rule is usually stated about shipped code, so a
`#[cfg(test)] mod tests` block reaches for `crate::test_support::repo_root()`
the way every other module's tests do. That compiles clean under
`cargo test --bin doctrine` and fails to compile every `e2e_design_*` binary —
a suite most inner loops skip. `cfg(test)` is on for those crates too (they are
`--test` builds), so the leaf's unit suite rides into all of them.

**How to apply.** A leaf that needs a test-support helper spells it locally:

```rust
fn repo_root() -> std::path::PathBuf {
    std::env::var("CARGO_MANIFEST_DIR").map(std::path::PathBuf::from)
        .expect("a cargo-driven test run sets CARGO_MANIFEST_DIR")
}
```

Runtime `CARGO_MANIFEST_DIR`, never `env!` (CHR-014 — the shared-target layout
lets a binary built in one worktree run in another). `legacy.rs:168` is the
in-tree precedent for the same move on the shipped side.

The shared helper is genuinely unreachable from both crates under one path: the
bin sees `crate::test_support`, the integration crates see
`common::test_support` (`tests/common/mod.rs` `#[path]`-includes the same file).

Related: [[mem.fact.doctrine.storage-tiers]].
