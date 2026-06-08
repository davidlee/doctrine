// SPDX-License-Identifier: GPL-3.0-only
//! Shared metadata-list substrate for authored numeric entities.
//!
//! Slices and ADRs are both numeric directories under a tree root, each holding
//! a sister `<stem>-<id>.toml` carrying the same four list fields. The reader,
//! status filter, and aligned formatter are status/path-parametric — they carry
//! zero per-kind knowledge — so they live here once and every kind calls them
//! (design SL-006 D4), parameterised by the toml *stem* (`"slice"` / `"adr"`).
//!
//! The stem is distinct from `entity::Kind.prefix` (`"SL"` / `"ADR"`): the stem
//! names the file (`slice-007.toml`), the prefix the canonical id (`SL-007`).
//!
//! This is CLI presentation plus an authored-toml reader — deliberately *not*
//! `entity.rs`, which stays a kind-blind scaffold engine free of presentation.
//! The clock seam lives in `crate::clock`; nothing here reads wall time.
//!
//! The generic table layout (`render_table`) used to live here too; SL-025
//! relocated it to the kind-blind read spine (`crate::listing`), which serves the
//! named (memory) and own-struct (backlog) kinds as well as these numeric ones.
//! `format_list` here is the numeric-kind grid that renders over it.

use std::fs;
use std::path::Path;

use anyhow::Context;
use serde::Deserialize;

use crate::entity;

/// The fields a reader extracts from a `<stem>-<id>.toml`. Unknown keys (the
/// `[relationships]` table, future sections) are ignored and preserved on disk.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct Meta {
    pub(crate) id: u32,
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) status: String,
}

/// Sort by id and, when a status is given, keep only matching rows.
pub(crate) fn sort_and_filter(mut rows: Vec<Meta>, status: Option<&str>) -> Vec<Meta> {
    rows.retain(|m| status.is_none_or(|s| m.status == s));
    rows.sort_by_key(|m| m.id);
    rows
}

/// Format rows as aligned `id  status  slug  title` lines, over the shared
/// `listing::render_table` (the layout authority relocated to the read spine,
/// SL-025).
pub(crate) fn format_list(rows: &[Meta]) -> String {
    let grid: Vec<Vec<String>> = rows
        .iter()
        .map(|m| {
            vec![
                format!("{:03}", m.id),
                m.status.clone(),
                m.slug.clone(),
                m.title.clone(),
            ]
        })
        .collect();
    crate::listing::render_table(&grid)
}

/// Parse the `Meta` of a single entity by id, reading `<stem>-<id>.toml` under
/// its numeric dir in `tree_root`.
pub(crate) fn read_meta(tree_root: &Path, stem: &str, id: u32) -> anyhow::Result<Meta> {
    let name = format!("{id:03}");
    let path = tree_root.join(&name).join(format!("{stem}-{name}.toml"));
    let text = fs::read_to_string(&path)
        .with_context(|| format!("{stem} {name} not found at {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("Failed to parse {}", path.display()))
}

/// Read and parse every `<stem>-<id>.toml` under `tree_root`. `scan_ids` yields
/// numeric dirs only, so `<id>-<slug>` symlinks and non-numeric entries are
/// skipped.
pub(crate) fn read_metas(tree_root: &Path, stem: &str) -> anyhow::Result<Vec<Meta>> {
    let mut metas = Vec::new();
    for id in entity::scan_ids(tree_root)? {
        metas.push(read_meta(tree_root, stem, id)?);
    }
    Ok(metas)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(id: u32, status: &str, slug: &str, title: &str) -> Meta {
        Meta {
            id,
            slug: slug.to_string(),
            title: title.to_string(),
            status: status.to_string(),
        }
    }

    /// Write a minimal `<stem>-<id>.toml` carrying the four list fields under its
    /// numeric dir — a true unit fixture, independent of any kind's scaffold.
    fn write_meta_toml(tree_root: &Path, stem: &str, id: u32, status: &str, slug: &str) {
        let name = format!("{id:03}");
        let dir = tree_root.join(&name);
        fs::create_dir_all(&dir).unwrap();
        let body = format!(
            "id = {id}\nslug = \"{slug}\"\ntitle = \"Title {id}\"\nstatus = \"{status}\"\n"
        );
        fs::write(dir.join(format!("{stem}-{name}.toml")), body).unwrap();
    }

    #[test]
    fn sort_and_filter_orders_by_id_and_filters_status() {
        let rows = vec![
            meta(2, "proposed", "b", "Two"),
            meta(1, "done", "a", "One"),
            meta(3, "proposed", "c", "Three"),
        ];

        let all = sort_and_filter(rows.clone(), None);
        assert_eq!(all.iter().map(|m| m.id).collect::<Vec<_>>(), vec![1, 2, 3]);

        let proposed = sort_and_filter(rows, Some("proposed"));
        assert_eq!(
            proposed.iter().map(|m| m.id).collect::<Vec<_>>(),
            vec![2, 3]
        );
    }

    #[test]
    fn format_list_renders_aligned_rows() {
        let rows = vec![
            meta(1, "started", "add-skill-removal", "Add skill removal"),
            meta(2, "proposed", "vendor-skills", "Vendor skills"),
        ];
        let out = format_list(&rows);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2);
        // "started" (7) pads to the width of "proposed" (8) for column alignment.
        assert!(lines[0].starts_with("001  started   add-skill-removal"));
        assert!(lines[0].ends_with("Add skill removal"));
        assert!(lines[1].starts_with("002  proposed  vendor-skills"));
    }

    #[test]
    fn format_list_empty_is_empty_string() {
        assert_eq!(format_list(&[]), "");
    }

    #[test]
    fn read_meta_reads_the_stem_toml() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_meta_toml(root, "slice", 1, "proposed", "my-slug");

        let m = read_meta(root, "slice", 1).unwrap();
        assert_eq!(m, meta(1, "proposed", "my-slug", "Title 1"));
    }

    #[test]
    fn read_meta_is_parameterised_by_stem() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // same id, different stem — the stem selects the file.
        write_meta_toml(root, "adr", 7, "accepted", "use-rust");

        let m = read_meta(root, "adr", 7).unwrap();
        assert_eq!(m.status, "accepted");
        // the wrong stem does not find it
        assert!(read_meta(root, "slice", 7).is_err());
    }

    #[test]
    fn read_metas_collects_every_numeric_and_skips_the_rest() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_meta_toml(root, "slice", 2, "proposed", "two");
        write_meta_toml(root, "slice", 1, "done", "one");
        // a `<id>-<slug>` symlink alias and a stray non-numeric dir are ignored
        // by scan_ids (numeric dirs only). read_metas yields scan order, not
        // sorted — sort_and_filter owns ordering — so compare as a set.
        std::os::unix::fs::symlink("001", root.join("001-one")).unwrap();
        fs::create_dir_all(root.join("notes")).unwrap();

        let mut ids: Vec<u32> = read_metas(root, "slice")
            .unwrap()
            .iter()
            .map(|m| m.id)
            .collect();
        ids.sort_unstable();
        assert_eq!(ids, vec![1, 2]);
    }
}
