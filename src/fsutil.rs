// SPDX-License-Identifier: GPL-3.0-only
//! Shared filesystem primitives — the path-containment chokepoint and the
//! accountable, atomic create operations used by both the scaffold engine
//! (`entity.rs`) and the runtime-state writer (`state.rs`, slice-004 D3).
//!
//! The split between those modules is of *contracts* (scaffold-once vs
//! mutate-in-place), not of IO: both reach disk through the same safe-join and
//! create-new primitives, so path-containment (H1) and create-new semantics
//! are implemented exactly once.

use std::fs::{self, File, OpenOptions};
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, bail};

/// Join a descriptor `rel` path under `tree_root`, rejecting absolute paths
/// and any `..` that would escape the tree (H1). The single chokepoint
/// through which a descriptor path reaches the filesystem.
pub(crate) fn safe_join(tree_root: &Path, rel: &Path) -> anyhow::Result<PathBuf> {
    if rel.is_absolute() {
        bail!(
            "Artifact path {} must be relative to the entity tree",
            rel.display()
        );
    }
    if rel.components().any(|c| c == Component::ParentDir) {
        bail!(
            "Artifact path {} must not escape the entity tree",
            rel.display()
        );
    }
    Ok(tree_root.join(rel))
}

/// Create a file, failing atomically if it already exists. `create_new(true)`
/// collapses the existence check and creation into one syscall — no TOCTOU
/// window (slice-004 D4 / finding 1). The caller decides what `AlreadyExists`
/// means: a refusal (the engine) or skip-if-present (the state writer).
pub(crate) fn create_new_file(path: &Path) -> std::io::Result<File> {
    OpenOptions::new().write(true).create_new(true).open(path)
}

/// Write `bytes` to `path` atomically: write a sibling temp file in the *same
/// directory*, then `rename` it over the target. The rename is atomic on a
/// single filesystem, so a concurrent reader sees either the old file or the
/// fully-written new one — never a torn write (slice-007 M6). The temp sits
/// beside the target (not `$TMPDIR`) so the rename never crosses a mount.
/// The pid keeps two processes' temps from colliding.
pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or_else(|| anyhow::anyhow!("path has no parent dir: {}", path.display()))?;
    let name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("path has no file name: {}", path.display()))?;
    let tmp = dir.join(format!(
        ".{}.{}.tmp",
        name.to_string_lossy(),
        std::process::id()
    ));
    fs::write(&tmp, bytes).with_context(|| format!("Failed to write temp {}", tmp.display()))?;
    fs::rename(&tmp, path)
        .with_context(|| format!("Failed to rename {} -> {}", tmp.display(), path.display()))
}

/// Whether a *real* directory sits at `path` — `symlink_metadata` does not
/// follow links, so a symlink (even one pointing at a directory) reports
/// `false`. Used to tell a pre-existing/concurrently-created dir apart from a
/// file or symlink squatting a path component during component-wise creation.
pub(crate) fn is_real_dir(path: &Path) -> bool {
    matches!(fs::symlink_metadata(path), Ok(m) if m.is_dir())
}

/// Ensure a symlink at `link` points at `target`: create it if absent, replace
/// it if it is a symlink to somewhere else, leave it if already correct, and
/// **error** if a real file or directory squats the path (finding 10). The
/// convenience link is kept honest but is never authority — callers resolve by
/// id regardless of what this link says.
pub(crate) fn set_symlink(link: &Path, target: &Path) -> anyhow::Result<()> {
    match fs::symlink_metadata(link) {
        Ok(m) if m.file_type().is_symlink() => {
            let current = fs::read_link(link)
                .with_context(|| format!("Failed to read symlink {}", link.display()))?;
            if current != target {
                fs::remove_file(link)
                    .with_context(|| format!("Failed to replace symlink {}", link.display()))?;
                symlink(target, link)?;
            }
            Ok(())
        }
        Ok(_) => bail!(
            "Refusing to replace non-symlink {} with a symlink",
            link.display()
        ),
        Err(e) if e.kind() == ErrorKind::NotFound => symlink(target, link),
        Err(e) => Err(e).with_context(|| format!("Failed to stat {}", link.display())),
    }
}

fn symlink(target: &Path, link: &Path) -> anyhow::Result<()> {
    std::os::unix::fs::symlink(target, link)
        .with_context(|| format!("Failed to create symlink {}", link.display()))
}

/// The disposition of one [`copy_selected`] candidate.
#[derive(Debug)]
pub(crate) enum CopyOutcome {
    /// Copied into the fork.
    Copied,
    /// Deliberately not copied (symlink escapes the tree / targets the withheld
    /// tier / is not a regular file); the reason is for a skip+warn line.
    Skipped(String),
}

/// Copy one repo-relative file from a **canonical** `source_root` into a
/// **canonical** `fork_root`, refusing anything whose real location escapes the
/// source tree (SL-029 §3 copy safety, B5 — `safe_join` is insufficient because
/// a symlink *component* can escape even with no `..`).
///
/// Both roots MUST already be canonical (the caller canonicalizes once). The
/// source path is resolved with [`fs::canonicalize`], so a symlink component or a
/// symlink pointing out-of-tree is caught; for a symlink whose target stays
/// in-tree, `target_withheld` decides whether that target lands in the
/// coordination tier (passed in to keep `fsutil` free of any git/worktree
/// dependency — no module cycle). The destination parent is canonicalized after
/// creation so a symlink component cannot redirect the write out of the fork;
/// the final path is then built by join (the dest leaf does not exist yet, R-c).
pub(crate) fn copy_selected(
    source_root: &Path,
    fork_root: &Path,
    rel: &Path,
    target_withheld: &dyn Fn(&Path) -> bool,
) -> anyhow::Result<CopyOutcome> {
    let src = source_root.join(rel);
    let meta = fs::symlink_metadata(&src).with_context(|| format!("stat {}", src.display()))?;

    let real = fs::canonicalize(&src).with_context(|| format!("canonicalize {}", src.display()))?;
    if !real.starts_with(source_root) {
        return Ok(CopyOutcome::Skipped(format!(
            "{} resolves outside the source tree",
            rel.display()
        )));
    }
    if meta.file_type().is_symlink() {
        let real_rel = real
            .strip_prefix(source_root)
            .map_err(|e| anyhow::anyhow!("strip source prefix: {e}"))?;
        if target_withheld(real_rel) {
            return Ok(CopyOutcome::Skipped(format!(
                "{} targets the withheld tier",
                rel.display()
            )));
        }
    }
    if !real.is_file() {
        return Ok(CopyOutcome::Skipped(format!(
            "{} is not a regular file",
            rel.display()
        )));
    }

    let dest = fork_root.join(rel);
    let parent = dest
        .parent()
        .ok_or_else(|| anyhow::anyhow!("dest {} has no parent", dest.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    let parent_canon =
        fs::canonicalize(parent).with_context(|| format!("canonicalize {}", parent.display()))?;
    if !parent_canon.starts_with(fork_root) {
        return Ok(CopyOutcome::Skipped(format!(
            "{} destination escapes the fork",
            rel.display()
        )));
    }
    let name = dest
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("dest {} has no file name", dest.display()))?;
    let final_dest = parent_canon.join(name);
    fs::copy(&real, &final_dest)
        .with_context(|| format!("copy {} -> {}", real.display(), final_dest.display()))?;
    Ok(CopyOutcome::Copied)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- safe_join (H1 path containment) ---

    #[test]
    fn safe_join_accepts_a_tree_relative_path() {
        let joined = safe_join(Path::new("/tree"), Path::new("003/x.toml")).unwrap();
        assert_eq!(joined, Path::new("/tree/003/x.toml"));
    }

    #[test]
    fn safe_join_rejects_absolute_paths() {
        let err = safe_join(Path::new("/tree"), Path::new("/etc/passwd")).unwrap_err();
        assert!(err.to_string().contains("must be relative"));
    }

    #[test]
    fn safe_join_rejects_parent_escape() {
        let err = safe_join(Path::new("/tree"), Path::new("../../etc/passwd")).unwrap_err();
        assert!(err.to_string().contains("must not escape"));
    }

    // --- create_new_file (atomic clobber refusal) ---

    #[test]
    fn create_new_file_refuses_an_existing_target() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x");
        assert!(create_new_file(&path).is_ok());
        let err = create_new_file(&path).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
    }

    // --- write_atomic (temp+rename swap) ---

    #[test]
    fn write_atomic_creates_then_overwrites_leaving_no_temp() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.toml");

        write_atomic(&path, b"v1").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "v1");

        write_atomic(&path, b"v2").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "v2");

        // the swap leaves only the target — no stray `.tmp` sibling.
        let names: Vec<String> = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["x.toml"]);
    }

    // --- is_real_dir (squat detection) ---

    #[test]
    fn is_real_dir_distinguishes_dirs_files_and_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("d");
        fs::create_dir(&real).unwrap();
        let file = dir.path().join("f");
        fs::write(&file, "x").unwrap();
        let link = dir.path().join("l");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        assert!(is_real_dir(&real));
        assert!(!is_real_dir(&file));
        // a symlink to a dir is NOT a real dir — it must not be silently traversed
        assert!(!is_real_dir(&link));
        assert!(!is_real_dir(&dir.path().join("absent")));
    }

    // --- set_symlink (verified convenience-link refresh) ---

    #[test]
    fn set_symlink_creates_replaces_and_keeps() {
        let dir = tempfile::tempdir().unwrap();
        let link = dir.path().join("phases");

        // absent → created
        set_symlink(&link, Path::new("../target-a")).unwrap();
        assert_eq!(fs::read_link(&link).unwrap(), Path::new("../target-a"));

        // wrong target → replaced
        set_symlink(&link, Path::new("../target-b")).unwrap();
        assert_eq!(fs::read_link(&link).unwrap(), Path::new("../target-b"));

        // already correct → idempotent no-op
        set_symlink(&link, Path::new("../target-b")).unwrap();
        assert_eq!(fs::read_link(&link).unwrap(), Path::new("../target-b"));
    }

    #[test]
    fn set_symlink_errors_on_a_real_file_squatting_the_path() {
        let dir = tempfile::tempdir().unwrap();
        let squat = dir.path().join("phases");
        fs::write(&squat, "not a symlink").unwrap();

        let err = set_symlink(&squat, Path::new("../target")).unwrap_err();
        assert!(err.to_string().contains("Refusing to replace non-symlink"));
        // untouched
        assert_eq!(fs::read_to_string(&squat).unwrap(), "not a symlink");
    }

    // --- copy_selected (SL-029 B5 copy safety) ---

    fn canon_roots() -> (tempfile::TempDir, tempfile::TempDir, PathBuf, PathBuf) {
        let src = tempfile::tempdir().unwrap();
        let fork = tempfile::tempdir().unwrap();
        let src_canon = fs::canonicalize(src.path()).unwrap();
        let fork_canon = fs::canonicalize(fork.path()).unwrap();
        (src, fork, src_canon, fork_canon)
    }

    #[test]
    fn copy_selected_copies_a_plain_nested_file() {
        let (src, _fork, src_canon, fork_canon) = canon_roots();
        let f = src.path().join("nested/data.txt");
        fs::create_dir_all(f.parent().unwrap()).unwrap();
        fs::write(&f, "hello").unwrap();

        let never = |_p: &Path| false;
        let out = copy_selected(
            &src_canon,
            &fork_canon,
            Path::new("nested/data.txt"),
            &never,
        )
        .unwrap();
        assert!(matches!(out, CopyOutcome::Copied));
        assert_eq!(
            fs::read_to_string(fork_canon.join("nested/data.txt")).unwrap(),
            "hello"
        );
    }

    #[test]
    fn copy_selected_refuses_an_out_of_tree_symlink() {
        let (src, _fork, src_canon, fork_canon) = canon_roots();
        let outside = tempfile::tempdir().unwrap();
        let secret = outside.path().join("secret");
        fs::write(&secret, "s").unwrap();
        std::os::unix::fs::symlink(&secret, src.path().join("link")).unwrap();

        let never = |_p: &Path| false;
        let out = copy_selected(&src_canon, &fork_canon, Path::new("link"), &never).unwrap();
        assert!(matches!(out, CopyOutcome::Skipped(_)));
        assert!(!fork_canon.join("link").exists());
    }

    #[test]
    fn copy_selected_refuses_a_symlink_into_the_withheld_tier() {
        let (src, _fork, src_canon, fork_canon) = canon_roots();
        let statefile = src.path().join(".doctrine/state/boot.md");
        fs::create_dir_all(statefile.parent().unwrap()).unwrap();
        fs::write(&statefile, "boot").unwrap();
        std::os::unix::fs::symlink(&statefile, src.path().join("link")).unwrap();

        let withheld = |p: &Path| p.starts_with(".doctrine/state");
        let out = copy_selected(&src_canon, &fork_canon, Path::new("link"), &withheld).unwrap();
        assert!(matches!(out, CopyOutcome::Skipped(_)));
        assert!(!fork_canon.join("link").exists());
    }
}
