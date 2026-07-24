// SPDX-License-Identifier: GPL-3.0-only
//! Entity markdown lookup — path derivation + async read (SL-072 PHASE-03).
//!
//! The map server's markdown surface: resolve an [`EntityKey`] to its `.md`
//! body on disk via the same `crate::kinds::KINDS` table that drives the catalog
//! scan.  Memory kinds (ASM/DEC/QUE/CON) use the same `kind.dir`/`stem`
//! convention — their stem is `"record"`, so the path is
//! `{kind.dir}/{id:03}/record-{id:03}.md`.

use std::path::{Path, PathBuf};

use crate::entity;
use crate::fsutil::safe_join;
use crate::map_server::error::MapServerError;
use crate::memory::{MEMORY_ITEMS_DIR, MEMORY_SHIPPED_DIR};

/// Return the Markdown body for a known entity key.
///
/// Reads the `.md` file at the path derived by [`entity_md_path`].
/// Returns [`MapServerError::EntityNotFound`] when the file does not exist.
pub(crate) async fn read_entity_markdown(
    root: &Path,
    key: &crate::catalog::scan::EntityKey,
) -> Result<String, MapServerError> {
    let path = entity_md_path(root, key)?;
    tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => MapServerError::EntityNotFound(key.canonical()),
            _ => MapServerError::Other(e.into()),
        })
}

/// Read a memory entity's markdown body from local overrides first, then
/// shipped memory records.
pub(crate) async fn read_memory_markdown(root: &Path, uid: &str) -> Result<String, MapServerError> {
    for dir in [MEMORY_ITEMS_DIR, MEMORY_SHIPPED_DIR] {
        let dir_path = safe_join(root, Path::new(dir)).map_err(MapServerError::Other)?;
        let md_path = safe_join(&dir_path, Path::new(uid))
            .map_err(MapServerError::Other)?
            .join("memory.md");
        match tokio::fs::read_to_string(&md_path).await {
            Ok(body) => return Ok(body),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(MapServerError::Other(e.into())),
        }
    }
    Err(MapServerError::EntityNotFound(uid.to_string()))
}

/// Read a memory body and rewrite its resolvable `[[mem.…]]` wikilinks into
/// map-router focus anchors (`#/focus/<uid>`), so inline references are
/// clickable in the web view (IMP-264).
///
/// `collect_all` is a disk scan; markdown fetches are user-paced (one per node
/// focus), so the cost is acceptable — cache if it ever bites.
pub(crate) async fn read_memory_markdown_linked(
    root: &Path,
    uid: &str,
) -> Result<String, MapServerError> {
    let body = read_memory_markdown(root, uid).await?;
    // Wikilink resolution is best-effort — a corpus-scan hiccup must not fail a
    // body fetch. On error, serve the body with its links unrewritten.
    let Ok(all) = crate::memory::collect_all(root) else {
        return Ok(body);
    };
    let (known_uids, key_to_uid) = crate::memory::known_link_maps(&all);
    let title_by_uid: std::collections::BTreeMap<String, String> = all
        .iter()
        .map(|m| (m.uid.clone(), m.title.clone()))
        .collect();
    Ok(crate::links::linkify_wikilinks(
        &body,
        &known_uids,
        &key_to_uid,
        &title_by_uid,
    ))
}

/// Derive the `.md` file path for an entity key.
///
/// Known kinds use the catalog convention: `<kind.dir>/<nnn>/<stem>.md`.
/// Memory kinds (ASM, DEC, QUE, CON) use the same `kind.dir`/`stem`
/// convention — their stem is `"record"`, so the path follows the same
/// template: `{kind.dir}/{id:03}/record-{id:03}.md`.
///
/// Requirements (`REQ`) return [`MapServerError::MarkdownNotImplemented`]
/// because their markdown body depends on a parent spec lookup that is
/// unresolved in SL-072.
///
/// Unknown prefixes return [`MapServerError::BadEntityId`].
fn entity_md_path(
    root: &Path,
    key: &crate::catalog::scan::EntityKey,
) -> Result<PathBuf, MapServerError> {
    if key.prefix == "REQ" {
        return Err(MapServerError::MarkdownNotImplemented("REQ"));
    }
    let kind_ref = crate::kinds::kind_by_prefix(key.prefix)
        .ok_or_else(|| MapServerError::BadEntityId(key.canonical()))?;
    Ok(entity::id_path(
        root,
        kind_ref.kind,
        key.id,
        entity::Ext::Md,
    ))
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;
    use crate::catalog::scan::EntityKey;

    /// Helper: construct an EntityKey with a static prefix.
    fn key(prefix: &'static str, id: u32) -> EntityKey {
        EntityKey { prefix, id }
    }

    /// A doctype root for path-only tests — no disk needed.
    fn root() -> &'static Path {
        Path::new("/corpus")
    }

    #[test]
    fn path_for_slice() {
        let p = entity_md_path(root(), &key("SL", 1)).unwrap();
        assert_eq!(p, PathBuf::from("/corpus/.doctrine/slice/001/slice-001.md"));
    }

    #[test]
    fn path_for_adr() {
        let p = entity_md_path(root(), &key("ADR", 12)).unwrap();
        assert_eq!(p, PathBuf::from("/corpus/.doctrine/adr/012/adr-012.md"));
    }

    #[test]
    fn path_for_memory_kind() {
        let p = entity_md_path(root(), &key("ASM", 1)).unwrap();
        assert_eq!(
            p,
            PathBuf::from("/corpus/.doctrine/knowledge/assumption/001/record-001.md")
        );
    }

    #[test]
    fn req_returns_not_implemented() {
        let err = entity_md_path(root(), &key("REQ", 1)).unwrap_err();
        match err {
            MapServerError::MarkdownNotImplemented(prefix) => {
                assert_eq!(prefix, "REQ");
            }
            other => panic!("expected MarkdownNotImplemented, got {:?}", other),
        }
    }

    #[test]
    fn path_for_concept_map() {
        let p = entity_md_path(root(), &key("CM", 1)).unwrap();
        assert_eq!(
            p,
            PathBuf::from("/corpus/.doctrine/concept-map/001/concept-map-001.md")
        );
    }

    #[test]
    fn unknown_prefix_returns_bad_entity_id() {
        let err = entity_md_path(root(), &key("BOGUS", 1)).unwrap_err();
        match err {
            MapServerError::BadEntityId(ref id) => {
                assert_eq!(id, "BOGUS-001");
            }
            other => panic!("expected BadEntityId, got {:?}", other),
        }
    }

    // == read_entity_markdown integration tests (temp dir) ==

    #[tokio::test]
    async fn read_returns_file_content() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Create a minimal slice dir with its .md file.
        let dir = root.join(".doctrine/slice/001");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("slice-001.md"), "# hello\n").unwrap();

        let content = read_entity_markdown(root, &key("SL", 1)).await.unwrap();
        assert_eq!(content, "# hello\n");
    }

    #[tokio::test]
    async fn read_missing_file_returns_entity_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Create the dir but NOT the .md file.
        let dir = root.join(".doctrine/slice/001");
        std::fs::create_dir_all(&dir).unwrap();

        let err = read_entity_markdown(root, &key("SL", 1)).await.unwrap_err();
        match err {
            MapServerError::EntityNotFound(ref id) => {
                assert_eq!(id, "SL-001");
            }
            other => panic!("expected EntityNotFound, got {:?}", other),
        }
    }

    fn memory_paths(root: &Path, uid: &str) -> (PathBuf, PathBuf) {
        (
            root.join(format!(".doctrine/memory/items/{uid}/memory.md")),
            root.join(format!(".doctrine/memory/shipped/{uid}/memory.md")),
        )
    }

    #[tokio::test]
    async fn read_memory_returns_items_content_when_present() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let uid = "test-memory-item";
        let (items_md, _) = memory_paths(root, uid);
        std::fs::create_dir_all(items_md.parent().unwrap()).unwrap();
        std::fs::write(&items_md, "# local override\n").unwrap();

        let content = read_memory_markdown(root, uid).await.unwrap();
        assert_eq!(content, "# local override\n");
    }

    #[tokio::test]
    async fn read_memory_falls_back_to_shipped_when_items_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let uid = "test-memory-item";
        let (_, shipped_md) = memory_paths(root, uid);
        std::fs::create_dir_all(shipped_md.parent().unwrap()).unwrap();
        std::fs::write(&shipped_md, "# shipped body\n").unwrap();

        let content = read_memory_markdown(root, uid).await.unwrap();
        assert_eq!(content, "# shipped body\n");
    }

    #[tokio::test]
    async fn read_memory_missing_in_both_locations_returns_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let uid = "missing-memory-item";

        let err = read_memory_markdown(root, uid).await.unwrap_err();
        match err {
            MapServerError::EntityNotFound(ref missing_uid) => {
                assert_eq!(missing_uid, uid);
            }
            other => panic!("expected EntityNotFound, got {:?}", other),
        }
    }

    /// Seed a minimal memory (`memory.toml` + `memory.md`) under items/.
    fn seed_memory(root: &Path, uid: &str, key: Option<&str>, title: &str, body: &str) {
        let key_line = key.map_or(String::new(), |k| format!("memory_key = \"{k}\"\n"));
        let dir = root.join(format!(".doctrine/memory/items/{uid}"));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("memory.toml"),
            format!(
                "memory_uid = \"{uid}\"\n\
                 {key_line}\
                 schema_version = 1\n\
                 memory_type = \"pattern\"\n\
                 status = \"active\"\n\
                 title = \"{title}\"\n\
                 summary = \"summary\"\n\
                 created = \"2026-01-01\"\n\
                 updated = \"2026-01-01\"\n\
                 [scope]\n\
                 workspace = \"default\"\n\
                 [git]\n\
                 repo = \"repo\"\n\
                 [trust]\n\
                 level = \"medium\"\n\
                 [ranking]\n\
                 severity = \"none\"\n\
                 weight = 0\n"
            ),
        )
        .unwrap();
        std::fs::write(dir.join("memory.md"), body).unwrap();
    }

    #[tokio::test]
    async fn linked_reader_resolves_wikilink_to_focus_anchor() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let target_uid = "mem_00000000000000000000000000000001";
        let source_uid = "mem_00000000000000000000000000000002";
        seed_memory(
            root,
            target_uid,
            Some("mem.pattern.target"),
            "Target Mem",
            "target body\n",
        );
        seed_memory(
            root,
            source_uid,
            Some("mem.pattern.source"),
            "Source Mem",
            "see [[mem.pattern.target]] and [[mem.pattern.missing]]\n",
        );

        let out = read_memory_markdown_linked(root, source_uid).await.unwrap();

        // Resolvable key → focus anchor with the target's title as label.
        assert!(
            out.contains(&format!("[Target Mem](#/focus/{target_uid})")),
            "expected focus anchor, got: {out}"
        );
        // Unresolvable key stays literal — no dead link.
        assert!(
            out.contains("[[mem.pattern.missing]]"),
            "unresolved link mangled: {out}"
        );
    }

    #[tokio::test]
    async fn read_memory_propagates_non_not_found_io_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let uid = "broken-memory-item";
        let items_dir = root.join(format!(".doctrine/memory/items/{uid}"));
        std::fs::create_dir_all(items_dir.parent().unwrap()).unwrap();
        std::fs::write(&items_dir, "not a directory").unwrap();

        let err = read_memory_markdown(root, uid).await.unwrap_err();
        match err {
            MapServerError::Other(other) => {
                let io = other.downcast_ref::<std::io::Error>().unwrap();
                assert_eq!(io.kind(), std::io::ErrorKind::NotADirectory);
            }
            other => panic!("expected Other, got {:?}", other),
        }
    }
}
