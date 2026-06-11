# Clean-HEAD worktree binary beats stash when concurrent WIP breaks the build

In the bubblewrap jail the workspace is shared across concurrent agent sessions
(e.g. one slice's authoring while another slice's PHASE work is mid-flight). A
sibling session can leave **non-compiling uncommitted WIP** in `src/` (seen
SL-021 vs SL-040 PHASE-02: a half-added `Review` command — `Facet::parse`
missing, `todo!()` match arm). `cargo run`/`cargo build` then fail, blocking any
task that needs the `doctrine` binary.

**Do NOT `git stash`** to clear it — file mtimes inside the session window and an
in-window sibling commit (`8bece6b` landed *between* two of my commits) signal a
**live** concurrent writer; stashing yanks files out from under it and corrupts
that session.

**Instead:** build a clean binary from a detached-HEAD git worktree and run *that*
against the real workspace — HEAD compiles (committed green), entity data lives in
`.doctrine/` and is read at runtime, so a HEAD-built binary authors correctly:

```
git worktree add --detach ~/doctrine-head HEAD
cargo build -q --manifest-path ~/doctrine-head/Cargo.toml   # CARGO_TARGET_DIR is
# jail-redirected to ~/.cargo/doctrine-target-jail/debug/doctrine — binary lands there
~/.cargo/doctrine-target-jail/debug/doctrine spec new ...    # run from /workspace/doctrine
git worktree remove --force ~/doctrine-head                  # cleanup when done
```

Never run `cargo` from the dirty main dir afterward (it rebuilds the broken
sources). Disjoint commit paths (`.doctrine/spec/**` vs sibling's `src/**`) mean
the interleaved commits never conflict. See [[mem.pattern.build.jail-target-redirect]].
