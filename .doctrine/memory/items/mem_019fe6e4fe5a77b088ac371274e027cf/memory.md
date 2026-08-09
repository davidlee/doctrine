# The `env!` ban reaches member crates, and `doctrine-control`'s route is `HostFacts::env_var`

Two deltas on [[mem.fact.testing.runtime-manifest-dir]], which owns the general
rule (resolve at runtime; `env!` bakes the building tree's path). Both cost a
full `doctrine check gate` cycle in `SL-248` `PHASE-10`.

**1. The guard is cross-package.** `tests/e2e_no_baked_paths.rs` (`CHR-014` /
`SL-162`) walks `src/`, `tests/` **and `crates/`**, so a workspace member crate
trips a *root package* integration test. The member can compile, pass
`cargo test -p <member>` and pass clippy while the gate reds — and the failure
names a file in a package you were not testing. Non-comment lines only, so the
prose documenting the rule does not self-match.

**2. In `crates/doctrine-control`, `std::env::var` is not the answer.** The
parent memory recommends it, and this crate bans it through `disallowed_methods`
(`clippy.toml`). The sanctioned reader is `HostFacts::env_var` (`host.rs:67`,
wrapping `var_os`) on `SystemHost`. The crate is bin-only with no `tests/`
directory, so `test_support::repo_root()` is out of reach entirely.

**3. Handle the `None` arm so the caller reds.** A test that resolves no
directory must fail loudly. Returning an empty file list makes an audit that
read nothing pass every assertion it then makes about what it read — assert a
positive control (the walk reached a file you know is there) before trusting a
clean result.
