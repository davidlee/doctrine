// SPDX-License-Identifier: GPL-3.0-only
//! Shared paths helper module — engine-tier, no clap imports.
//!
//! Provides file-classification, directory scanning, and selection logic
//! for entity directories (SL-139 PHASE-01).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;

// ---------------------------------------------------------------------------
// Pure types (no clap)
// ---------------------------------------------------------------------------

/// A classified set of file paths within one entity directory.
#[derive(Debug)]
pub(crate) struct EntityPathSet {
    /// The canonical identity TOML file (always present).
    pub(crate) toml: PathBuf,
    /// The identity Markdown file, if the entity carries one.
    pub(crate) md: Option<PathBuf>,
    /// All other regular, non-excluded files, sorted lexicographically.
    pub(crate) others: Vec<PathBuf>,
}

/// Which file classes to select from an [`EntityPathSet`].
#[derive(Debug)]
pub(crate) struct PathSelection {
    pub(crate) toml: bool,
    pub(crate) md: bool,
    pub(crate) entity: bool,
    pub(crate) single: bool,
}

// ---------------------------------------------------------------------------
// Exclusion filter
// ---------------------------------------------------------------------------

/// Returns `true` if `name` is an editor temp file, backup, or vim swap file.
///
/// Excludes names that:
/// - start with `.` (hidden files like `.DS_Store`)
/// - start with `#` (emacs autosave like `#file#`)
/// - end with `~` (emacs backup)
/// - end with `.swp` (vim swap)
/// - are exactly `.orig` or `.bak` (patch/merge artifacts)
pub(crate) fn is_excluded_name(name: &str) -> bool {
    if name.starts_with('.') || name.starts_with('#') {
        return true;
    }
    if name.ends_with('~')
        || Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("swp"))
    {
        return true;
    }
    name == ".orig" || name == ".bak"
}

// ---------------------------------------------------------------------------
// Directory scanner
// ---------------------------------------------------------------------------

/// Scan an entity directory and classify every regular file.
///
/// Only regular files are considered (symlinks and subdirectories are
/// skipped). The exclusion filter ([`is_excluded_name`]) discards editor
/// detritus. Files are classified by exact filename match against the
/// identity TOML and optional identity MD; anything else becomes an "other"
/// entry, sorted lexicographically.
///
/// Every returned path is **relative to `root`**. An error is returned if
/// the identity TOML file is not found.
pub(crate) fn scan_entity_dir(
    dir: &Path,
    identity_toml: &Path,
    identity_md: Option<&Path>,
    root: &Path,
) -> anyhow::Result<EntityPathSet> {
    let toml_name = identity_toml
        .file_name()
        .and_then(|n| n.to_str())
        .context("identity TOML path has no filename component")?;
    let md_name = identity_md
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str());

    let mut found_toml: Option<PathBuf> = None;
    let mut found_md: Option<PathBuf> = None;
    let mut others: Vec<PathBuf> = Vec::new();

    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory {}", dir.display()))?;

    for entry in entries {
        let entry = entry.with_context(|| format!("Failed to read entry in {}", dir.display()))?;
        let file_type = entry.file_type().with_context(|| {
            format!(
                "Failed to get file type for entry in {}",
                dir.display()
            )
        })?;

        // Only regular files — skip symlinks and subdirs.
        if !file_type.is_file() {
            continue;
        }

        let name = entry.file_name();
        let name_str = name
            .to_str()
            .context("Non-UTF-8 filename in entity directory")?;

        if is_excluded_name(name_str) {
            continue;
        }

        let abs_path = entry.path();
        let rel_path = abs_path
            .strip_prefix(root)
            .with_context(|| {
                format!(
                    "Failed to make {} root-relative to {}",
                    abs_path.display(),
                    root.display()
                )
            })?
            .to_path_buf();

        if name_str == toml_name {
            found_toml = Some(rel_path);
        } else if Some(name_str) == md_name {
            found_md = Some(rel_path);
        } else {
            others.push(rel_path);
        }
    }

    let toml = found_toml.context(format!(
        "Identity TOML file '{}' not found in {}",
        toml_name,
        dir.display()
    ))?;

    others.sort();

    Ok(EntityPathSet {
        toml,
        md: found_md,
        others,
    })
}

// ---------------------------------------------------------------------------
// Selector logic
// ---------------------------------------------------------------------------

/// Select paths from an [`EntityPathSet`] according to a [`PathSelection`].
///
/// When no selector is set (`toml`, `md`, `entity` all false) the canonical
/// order is returned: TOML → MD (if present) → others (lexicographic).
///
/// When **any** selector is set, only the explicitly-selected classes are
/// returned; others are excluded. `entity` acts as `toml` + `md` combined.
/// `single` truncates the result to the first path.
///
/// An error is returned when `--md` or `--entity` is requested but the
/// entity has no MD file. Missing MD is tolerated in the default all-files
/// mode.
///
/// Returns `Vec<String>` of root-relative paths; this function does **not**
/// write to stdout.
pub(crate) fn select_paths(
    set: &EntityPathSet,
    sel: &PathSelection,
) -> anyhow::Result<Vec<String>> {
    let any_selector = sel.toml || sel.md || sel.entity;
    let need_md = sel.md || sel.entity;

    let mut paths: Vec<String> = Vec::new();

    if any_selector {
        // Only explicitly-selected classes.
        if sel.toml || sel.entity {
            paths.push(set.toml.to_string_lossy().into_owned());
        }
        if need_md {
            let md = set.md.as_ref().context(
                "MD file selected (--md or --entity) but not found in entity directory",
            )?;
            paths.push(md.to_string_lossy().into_owned());
        }
    } else {
        // Default: all files in canonical order.
        paths.push(set.toml.to_string_lossy().into_owned());
        if let Some(md) = &set.md {
            paths.push(md.to_string_lossy().into_owned());
        }
        for other in &set.others {
            paths.push(other.to_string_lossy().into_owned());
        }
    }

    if sel.single && !paths.is_empty() {
        paths.truncate(1);
    }

    Ok(paths)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // -----------------------------------------------------------------------
    // VT-1: PathSelection combos
    // -----------------------------------------------------------------------

    /// Build a temp scenario: `<root>/dir/` containing `item.toml`,
    /// `item.md`, and `extra.txt`.
    fn temp_set(
        toml_name: &str,
        md_name: Option<&str>,
        extra: &[&str],
    ) -> (tempfile::TempDir, EntityPathSet) {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let dir = root.join("dir");
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join(toml_name), "toml").unwrap();
        if let Some(md) = md_name {
            fs::write(dir.join(md), "md").unwrap();
        }
        for e in extra {
            fs::write(dir.join(e), e).unwrap();
        }

        let set = scan_entity_dir(
            &dir,
            Path::new(toml_name),
            md_name.map(Path::new),
            root,
        )
        .unwrap();
        (tmp, set)
    }

    #[test]
    fn select_all_when_no_selectors_set() {
        let (_tmp, set) = temp_set("item.toml", Some("item.md"), &["a.txt", "z.txt"]);
        let paths = select_paths(
            &set,
            &PathSelection {
                toml: false,
                md: false,
                entity: false,
                single: false,
            },
        )
        .unwrap();
        // canonical order: TOML → MD → others (lexicographic)
        assert_eq!(paths, vec!["dir/item.toml", "dir/item.md", "dir/a.txt", "dir/z.txt"]);
    }

    #[test]
    fn select_toml_only() {
        let (_tmp, set) = temp_set("item.toml", Some("item.md"), &["a.txt"]);
        let paths = select_paths(
            &set,
            &PathSelection {
                toml: true,
                md: false,
                entity: false,
                single: false,
            },
        )
        .unwrap();
        assert_eq!(paths, vec!["dir/item.toml"]);
    }

    #[test]
    fn select_md_only() {
        let (_tmp, set) = temp_set("item.toml", Some("item.md"), &["a.txt"]);
        let paths = select_paths(
            &set,
            &PathSelection {
                toml: false,
                md: true,
                entity: false,
                single: false,
            },
        )
        .unwrap();
        assert_eq!(paths, vec!["dir/item.md"]);
    }

    #[test]
    fn select_entity_gives_toml_and_md() {
        let (_tmp, set) = temp_set("item.toml", Some("item.md"), &["a.txt"]);
        let paths = select_paths(
            &set,
            &PathSelection {
                toml: false,
                md: false,
                entity: true,
                single: false,
            },
        )
        .unwrap();
        assert_eq!(paths, vec!["dir/item.toml", "dir/item.md"]);
    }

    #[test]
    fn select_single_truncates() {
        let (_tmp, set) = temp_set("item.toml", Some("item.md"), &["a.txt"]);
        let paths = select_paths(
            &set,
            &PathSelection {
                toml: false,
                md: false,
                entity: false,
                single: true,
            },
        )
        .unwrap();
        assert_eq!(paths, vec!["dir/item.toml"]);
    }

    #[test]
    fn select_toml_and_md_explicit() {
        let (_tmp, set) = temp_set("item.toml", Some("item.md"), &["a.txt"]);
        let paths = select_paths(
            &set,
            &PathSelection {
                toml: true,
                md: true,
                entity: false,
                single: false,
            },
        )
        .unwrap();
        assert_eq!(paths, vec!["dir/item.toml", "dir/item.md"]);
    }

    // -----------------------------------------------------------------------
    // VT-2: Exclusion filter
    // -----------------------------------------------------------------------

    #[test]
    fn is_excluded_name_rejects_hidden_files() {
        assert!(is_excluded_name(".DS_Store"));
        assert!(is_excluded_name(".gitignore"));
    }

    #[test]
    fn is_excluded_name_rejects_hash_prefixed_files() {
        assert!(is_excluded_name("#file#"));
        assert!(is_excluded_name("#emacs-autosave#"));
    }

    #[test]
    fn is_excluded_name_rejects_tilde_suffix() {
        assert!(is_excluded_name("file~"));
        assert!(is_excluded_name("notes.md~"));
    }

    #[test]
    fn is_excluded_name_rejects_swp_suffix() {
        assert!(is_excluded_name(".file.swp"));
        assert!(is_excluded_name("notes.swp"));
    }

    #[test]
    fn is_excluded_name_rejects_orig_and_bak() {
        assert!(is_excluded_name(".orig"));
        assert!(is_excluded_name(".bak"));
    }

    #[test]
    fn is_excluded_name_accepts_regular_names() {
        assert!(!is_excluded_name("readme.md"));
        assert!(!is_excluded_name("slice-001.toml"));
        assert!(!is_excluded_name("plan.toml"));
        assert!(!is_excluded_name("design.md"));
        assert!(!is_excluded_name("notes.txt"));
    }

    // -----------------------------------------------------------------------
    // VT-3: Scanner skips symlinks and subdirs, returns only regular files
    // -----------------------------------------------------------------------

    #[test]
    fn scanner_skips_symlinks_and_subdirs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let dir = root.join("ent");
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join("item.toml"), "toml").unwrap();
        fs::write(dir.join("readme.md"), "md").unwrap();
        fs::create_dir(dir.join("subdir")).unwrap();

        #[cfg(unix)]
        std::os::unix::fs::symlink("item.toml", dir.join("link-to-toml")).unwrap();

        let set = scan_entity_dir(
            &dir,
            Path::new("item.toml"),
            Some(Path::new("readme.md")),
            root,
        )
        .unwrap();

        // Only regular files: item.toml + readme.md
        assert_eq!(set.toml.to_string_lossy(), "ent/item.toml");
        assert_eq!(set.md.as_ref().unwrap().to_string_lossy(), "ent/readme.md");
        assert!(set.others.is_empty());
    }

    // -----------------------------------------------------------------------
    // VT-4: Root-relative path conversion
    // -----------------------------------------------------------------------

    #[test]
    fn paths_are_root_relative() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let dir = root.join("a").join("b");
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join("thing.toml"), "x").unwrap();
        fs::write(dir.join("extra.log"), "y").unwrap();

        let set = scan_entity_dir(
            &dir,
            Path::new("thing.toml"),
            None,
            root,
        )
        .unwrap();

        assert_eq!(set.toml.to_string_lossy(), "a/b/thing.toml");
        assert_eq!(set.others, vec![PathBuf::from("a/b/extra.log")]);
    }

    // -----------------------------------------------------------------------
    // VT-5: Canonical ordering: TOML → MD → lexicographic others
    // -----------------------------------------------------------------------

    #[test]
    fn canonical_ordering_is_toml_then_md_then_others_sorted() {
        let (_tmp, set) = temp_set(
            "item.toml",
            Some("item.md"),
            &["z.md", "a.toml", "m.txt"],
        );
        let paths = select_paths(
            &set,
            &PathSelection {
                toml: false,
                md: false,
                entity: false,
                single: false,
            },
        )
        .unwrap();
        assert_eq!(
            paths,
            vec![
                "dir/item.toml",
                "dir/item.md",
                "dir/a.toml",
                "dir/m.txt",
                "dir/z.md",
            ]
        );
    }

    // -----------------------------------------------------------------------
    // Error: missing identity TOML
    // -----------------------------------------------------------------------

    #[test]
    fn missing_identity_toml_is_error() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let dir = root.join("ent");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("extra.txt"), "x").unwrap();

        let err = scan_entity_dir(
            &dir,
            Path::new("item.toml"),
            None,
            root,
        )
        .unwrap_err();
        assert!(err.to_string().contains("not found"));
        assert!(err.to_string().contains("item.toml"));
    }

    // -----------------------------------------------------------------------
    // Error: MD missing when --md or --entity selected
    // -----------------------------------------------------------------------

    #[test]
    fn missing_md_errors_when_md_selected() {
        let (_tmp, set) = temp_set("item.toml", None, &[]);
        let err = select_paths(
            &set,
            &PathSelection {
                toml: false,
                md: true,
                entity: false,
                single: false,
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("MD file selected"));
    }

    #[test]
    fn missing_md_errors_when_entity_selected() {
        let (_tmp, set) = temp_set("item.toml", None, &[]);
        let err = select_paths(
            &set,
            &PathSelection {
                toml: false,
                md: false,
                entity: true,
                single: false,
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("MD file selected"));
    }

    // -----------------------------------------------------------------------
    // Missing MD tolerated in default all-files mode
    // -----------------------------------------------------------------------

    #[test]
    fn missing_md_tolerated_in_all_files_mode() {
        let (_tmp, set) = temp_set("item.toml", None, &["extra.txt"]);
        let paths = select_paths(
            &set,
            &PathSelection {
                toml: false,
                md: false,
                entity: false,
                single: false,
            },
        )
        .unwrap();
        assert_eq!(paths, vec!["dir/item.toml", "dir/extra.txt"]);
    }

    // -----------------------------------------------------------------------
    // Extra: single with entity selector
    // -----------------------------------------------------------------------

    #[test]
    fn entity_single_truncates_to_first() {
        let (_tmp, set) = temp_set("item.toml", Some("item.md"), &[]);
        let paths = select_paths(
            &set,
            &PathSelection {
                toml: false,
                md: false,
                entity: true,
                single: true,
            },
        )
        .unwrap();
        assert_eq!(paths, vec!["dir/item.toml"]);
    }
}
