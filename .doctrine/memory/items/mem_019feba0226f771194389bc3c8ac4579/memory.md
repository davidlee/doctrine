The `closure-resolver` seam in `crates/doctrine-control/src/backend/bubblewrap.rs`
imposes four requirements that are not obvious from the config keys, each
enforced by a different function. Read before writing any resolver.

1. **Echo the queried path itself.** `expand_closure_root` extends the bound set
   with *only what the resolver returned*; it never adds the realised root it
   was given. A resolver that reports dependencies but not the binary leaves the
   binary unbound. (`nix-store --query --requisites` happens to include the
   queried path, which is why this is easy to miss.)
2. **Every line must be an absolute path.** `closure_members` refuses a
   non-absolute line with `ResolverPathNotAbsolute`.
3. **Every returned path must exist.** `expand_closure_root` refuses with
   `ResolverPathAbsent`. This is what makes `linux-vdso.so.1` — pathless, and
   not a file — fatal if a loader-trace resolver forwards it unfiltered.
4. **Exit 0, and the status is checked *before* the output is parsed.** That
   order is deliberate: a resolver that failed and still printed plausible paths
   must refuse *as a failed resolver*, not be believed.

Also: output is capped at `RESOLVER_OUTPUT_LIMIT` (1 MiB), each member is
resolved through symlinks before binding, and members are bound **individually,
never their common parent** (`SL-248` `PHASE-05` `EX-10`).

## Admission

The resolver's argv[0] basename is checked against the interpretation policy's
forbidden executables by `admit_resolver` (`provision.rs`), at step 6 — *after*
the policy binds, so a phase refinement can forbid a resolver the same
transaction would otherwise have run.

## The path had no live exercise before SL-252

`resolved_closure_root` → `expand_closure_root` → `closure_members` was driven
only by `FixtureQuery` doubles in unit tests. The conformance fixture declared
no `closure-roots` at all. `SL-252` routes the fixture through it deliberately,
so the production closure path gets exercised for real.

Related: [[mem.fact.linker.ld-trace-loaded-objects-is-the-closure]].
