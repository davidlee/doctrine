# Repo clippy bans std::env::var (disallowed_methods) — use var_os

`clippy.toml` `disallowed-methods` denies `std::env::var` ("Use typed
configuration loading instead") and `std::env::set_var` ("Global process
mutation is test-hostile"). The gate is `-D clippy::disallowed-methods`, so a
direct `std::env::var("X")` fails `just check`.

`std::env::var_os` is NOT banned (`skills.rs` reads `HOME` through it). To read a
flag-style env var, compare the `OsString`:

```rust
std::env::var_os("DOCTRINE_WORKER").as_deref() == Some(std::ffi::OsStr::new("1"))
```

Sibling bans live alongside: `disallowed-types` forbids `HashMap`/`HashSet`
(see [[mem.pattern.lint.disallowed-types-collections]]); the broader denies in
[[mem.pattern.lint.clippy-denies]]. Run the gate as plain `cargo clippy`, never
`--all-targets` ([[mem.pattern.build.jail-target-redirect]] for the build path).
