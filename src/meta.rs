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
//! The numeric-kind list grid (formerly `format_list`) now lives on the spine too,
//! per-kind; `meta` keeps only the authored-toml reader and the sort-by-id helper.

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
    #[serde(default)]
    pub(crate) tags: Vec<String>,
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

/// The id, and only the id, of a `<stem>-<id>.toml` — the scan-path reader
/// (SL-040 D2). Serde ignores every other key, so a kind whose authored toml is
/// intentionally **status-less** (review — its status is derived, D-C8) scans for
/// `.id` cleanly, while the strict [`Meta`] above stays unchanged: a genuinely
/// corrupt status-bearing toml with a missing `status` still hard-fails at every
/// `read_meta` caller (`show`/`list`/render). Leniency is confined to this path —
/// `validate`'s id scan, which only ever needs the `id` (design §5 / R-a).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct IdOnly {
    pub(crate) id: u32,
}

/// Read just the `id` of a single entity's `<stem>-<id>.toml` — the [`IdOnly`]
/// scan-path reader. Used by `integrity::scan_kind`, the one place a status-less
/// kind (review) must be read without tripping the strict [`Meta`] (D2).
pub(crate) fn read_id(tree_root: &Path, stem: &str, id: u32) -> anyhow::Result<u32> {
    let name = format!("{id:03}");
    let path = tree_root.join(&name).join(format!("{stem}-{name}.toml"));
    let text = fs::read_to_string(&path)
        .with_context(|| format!("{stem} {name} not found at {}", path.display()))?;
    let parsed: IdOnly =
        toml::from_str(&text).with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(parsed.id)
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
            tags: Vec::new(),
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

    /// Write a status-LESS `<stem>-<id>.toml` carrying only `id`/`slug`/`title`
    /// — review's intentionally derived-status authored shape (SL-040 D2).
    fn write_statusless_toml(tree_root: &Path, stem: &str, id: u32) {
        let name = format!("{id:03}");
        let dir = tree_root.join(&name);
        fs::create_dir_all(&dir).unwrap();
        let body = format!("id = {id}\nslug = \"sl\"\ntitle = \"T {id}\"\n");
        fs::write(dir.join(format!("{stem}-{name}.toml")), body).unwrap();
    }

    /// SL-040 D2 (VT-1, the scan-path half): the id-only reader scans a
    /// status-less toml for `.id` cleanly — review need not seed a derived status.
    #[test]
    fn read_id_scans_a_statusless_toml() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_statusless_toml(root, "review", 7);
        assert_eq!(read_id(root, "review", 7).unwrap(), 7);
    }

    /// SL-040 D2 (VT-1, the preserved-invariant half): the strict `Meta` reader
    /// still HARD-FAILS on a status-less status-BEARING toml — leniency is confined
    /// to `read_id`; `read_meta` keeps the "missing status is corruption" contract.
    #[test]
    fn read_meta_still_hard_fails_on_a_missing_status() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_statusless_toml(root, "slice", 7);
        let err = read_meta(root, "slice", 7).unwrap_err();
        assert!(
            err.to_string().contains("Failed to parse"),
            "missing status must be a hard parse error: {err}"
        );
    }

    #[test]
    fn read_metas_collects_every_numeric_and_skips_the_rest() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_meta_toml(root, "slice", 2, "proposed", "two");
        write_meta_toml(root, "slice", 1, "done", "one");
        // a `<id>-<slug>` symlink alias and a stray non-numeric dir are ignored
        // by scan_ids (numeric dirs only). read_metas yields scan order, not
        // sorted — each kind owns ordering on the spine — so compare as a set.
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
