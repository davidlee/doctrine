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
}
