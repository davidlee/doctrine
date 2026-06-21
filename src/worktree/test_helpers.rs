/// Test-only git helpers — creates repos, runs commands. Shared across test
/// modules in `worktree/`.
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(test)]
pub(crate) fn git(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("spawn git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// A primary git repo with a base commit; returns the canonical root.
#[cfg(test)]
pub(crate) fn init_repo(dir: &Path) -> PathBuf {
    fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    fs::write(dir.join("seed"), "x").unwrap();
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "base"]);
    fs::canonicalize(dir).unwrap()
}
