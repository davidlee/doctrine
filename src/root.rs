// SPDX-License-Identifier: GPL-3.0-only
//! Project-root detection, shared by `install` and `skills`.

use std::path::{Path, PathBuf};

use anyhow::Context;

/// Default markers that identify a project root when walking up from CWD.
pub(crate) fn default_markers() -> Vec<String> {
    vec![
        ".git".to_string(),
        ".jj".to_string(),
        ".project".to_string(),
        "Cargo.toml".to_string(),
    ]
}

/// Resolve the project root.
///
/// An `explicit` path is used as-is. Otherwise walk up from CWD until a
/// directory contains any of `markers`.
pub(crate) fn find(explicit: Option<PathBuf>, markers: &[String]) -> anyhow::Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path);
    }

    let cwd = std::env::current_dir().context("Failed to get current working directory")?;

    find_from(&cwd, markers).ok_or_else(|| {
        anyhow::anyhow!(
            "No project root found. Walked up from '{}' looking for any of: {:?}",
            cwd.display(),
            markers,
        )
    })
}

/// Walk up from `start` (inclusive) to the first ancestor containing any of
/// `markers`. Unlike [`find`], this takes an explicit start directory — the
/// ambient-surfacing hook (SL-205) discovers the root from the stdin `cwd`, not
/// the process cwd — and returns `None` (rather than an error) when no marker is
/// found, so the caller can fold a missing root into a fail-open no-op.
pub(crate) fn find_from(start: &Path, markers: &[String]) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        for marker in markers {
            if ancestor.join(marker).exists() {
                return Some(ancestor.to_path_buf());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// A marker name no real ancestor will carry. `find_from` takes its markers
    /// as a parameter, so the walk can be tested against a tree the test wholly
    /// owns — no dependence on whether `.git` / `Cargo.toml` happens to sit above
    /// `TMPDIR` (ISS-281, where a stray `/tmp/.git` made exactly that assumption
    /// false).
    const MARKER: &str = ".iss281-test-marker";

    fn markers() -> Vec<String> {
        vec![MARKER.to_string()]
    }

    /// The canonical path of a fresh tempdir — `TMPDIR` may be a symlink, and
    /// `find_from` returns ancestors of whatever it was given, so expectations
    /// must be built from the same canonical form.
    fn canonical_tempdir() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let canonical = fs::canonicalize(dir.path()).unwrap();
        (dir, canonical)
    }

    /// The walk is inclusive of `start`: a marker in the start dir itself is a
    /// hit, not skipped in favour of an ancestor.
    #[test]
    fn start_dir_itself_is_a_candidate() {
        let (_d, root) = canonical_tempdir();
        fs::create_dir(root.join(MARKER)).unwrap();

        assert_eq!(find_from(&root, &markers()), Some(root));
    }

    /// From a descendant, the walk climbs to the marked ancestor.
    #[test]
    fn walks_up_to_a_marked_ancestor() {
        let (_d, root) = canonical_tempdir();
        fs::create_dir(root.join(MARKER)).unwrap();
        let deep = root.join("a/b/c");
        fs::create_dir_all(&deep).unwrap();

        assert_eq!(find_from(&deep, &markers()), Some(root));
    }

    /// With two marked ancestors, the NEAREST wins — the walk stops at the first
    /// hit rather than continuing to the outermost.
    #[test]
    fn nearest_marked_ancestor_wins() {
        let (_d, outer) = canonical_tempdir();
        fs::create_dir(outer.join(MARKER)).unwrap();
        let inner = outer.join("inner");
        fs::create_dir(&inner).unwrap();
        fs::create_dir(inner.join(MARKER)).unwrap();
        let deep = inner.join("x/y");
        fs::create_dir_all(&deep).unwrap();

        assert_eq!(find_from(&deep, &markers()), Some(inner));
    }

    /// The miss path: no marker anywhere in the ancestry ⇒ `None`, not an error.
    /// This is the fail-open the ambient-surfacing hook folds into "emit nothing"
    /// (INV-2), and the assertion vt9 could not make hermetically while it
    /// depended on the real marker set (ISS-281).
    #[test]
    fn unmarked_ancestry_yields_none() {
        let (_d, root) = canonical_tempdir();
        let deep = root.join("a/b");
        fs::create_dir_all(&deep).unwrap();

        assert_eq!(find_from(&deep, &markers()), None);
    }

    /// Any one of several markers is enough, and a marker that is a plain file
    /// counts as much as a directory — `.git` is a dir in a worktree but a file
    /// in a submodule / linked worktree, and the predicate is `exists()`.
    #[test]
    fn any_marker_matches_and_a_file_marker_counts() {
        let (_d, root) = canonical_tempdir();
        fs::write(root.join(".second-marker"), b"gitdir: elsewhere").unwrap();

        let many = vec![MARKER.to_string(), ".second-marker".to_string()];
        assert_eq!(find_from(&root, &many), Some(root));
    }

    /// An empty marker set can never hit — the walk exhausts and folds to `None`
    /// rather than defaulting to the start dir.
    #[test]
    fn empty_marker_set_yields_none() {
        let (_d, root) = canonical_tempdir();
        assert_eq!(find_from(&root, &[]), None);
    }
}
