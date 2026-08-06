# Cargo `default-members` reaches packaging and publishing

`default-members` is a **package-selection** default, not a build-command one.
It selects for `cargo package` and `cargo publish` as much as for `build`,
`clippy` and `test`. Adding a workspace member to it to get the member into the
test set therefore also puts it into the release path.

Three measured facts (minimal workspace, `RV-346` `F-21`/`F-23`):

1. **Bare `cargo package` with two default members packages both.**
2. **`publish = false` does not make a bare `cargo publish` skip the member.**
   It fails the whole command:
   ``error: `sub` cannot be published. `package.publish` must be set to `true`
   or a non-empty list``. So the manifest key alone *breaks* releases rather
   than protecting them — keeping a member unreleased needs the key **and** a
   `-p <pkg>` on the publish recipe. The key states the intent durably; the
   flag is what lets the command run.
3. **`cargo fmt` ignores `default-members` entirely** and walks every workspace
   member. Only `lint`, `build` and `test` take the default set — so
   "default-members brings the crate into the checked set" is true of three
   legs, not four.

Adjacent, same source: **cargo `include` accepts gitignore-style negation.**
`include = ["/src/**", "!/src/lib.rs"]` omits that one file from the packaged
source — `cargo package --list` drops it and `cargo package` completes with a
warning that the target was ignored. So excluding a file from a published
tarball costs one pattern, not a replacement enumeration. Useful when a lib
target exists for a workspace sibling and should not become crates.io semver
surface — but it makes the published crate differ from the built one, which is
the same shape as the crane embed-strip trap in AGENTS.md, so assert it rather
than trust it.

See [[mem.pattern.review.sandboxed-agent-measures-its-own-cage]] for the round
these came from.
