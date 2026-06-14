# tempfile is a dev-dependency only — production git plumbing uses a git-dir ScratchIndex for a throwaway GIT_INDEX_FILE

`tempfile` is declared **only** under `[dev-dependencies]` in `Cargo.toml`, so it
is unavailable to non-test (bin/lib) code. Production git plumbing that needs a
**throwaway `GIT_INDEX_FILE`** (e.g. a tree-filter that must stage into a scratch
index without touching the live index/working tree) therefore cannot reach for
`tempfile::NamedTempFile`.

## The fix — a git-dir scratch index, no new dependency

Put the throwaway index **inside the repo's git dir** and remove it on drop:

```rust
struct ScratchIndex { path: std::path::PathBuf }
impl ScratchIndex {
    fn new(root: &Path) -> Result<Self, CaptureError> {
        let git_dir = git_text(root, &["rev-parse", "--absolute-git-dir"])?;
        let path = Path::new(&git_dir).join(format!("doctrine-filter-index.{}", std::process::id()));
        drop(std::fs::remove_file(&path)); // clear a crashed run's leftover
        Ok(Self { path })
    }
}
impl Drop for ScratchIndex { fn drop(&mut self) { drop(std::fs::remove_file(&self.path)); } }
```

Then thread it as `GIT_INDEX_FILE`: `read-tree`/`rm --cached`/`write-tree` all
write that scratch index, never the live one. git treats an **absent** index file
as empty, so no pre-seeding is needed.

## Why not just promote tempfile to a runtime dep?

That is a dependency-surface change (a new crate in the production binary) — worth
a `/consult`, and unnecessary here. The git-dir scratch is self-contained,
same-filesystem (no cross-device rename), and naturally scoped to the repo. Use
`drop(remove_file(..))` not `let _ =` (the latter trips
`clippy::let_underscore_must_use`, see [[mem.pattern.lint.clippy-denies]]).

Single-writer orchestrator ⇒ the pid-suffixed name is collision-free. Surfaced in
SL-064 PHASE-03 `filter_tree` (`src/git.rs`).
