// SPDX-License-Identifier: GPL-3.0-only
//! Entity markdown lookup — path derivation + async read (SL-072 PHASE-03).
//!
//! The map server's markdown surface: resolve an [`EntityKey`] to its `.md`
//! body on disk via the same `integrity::KINDS` table that drives the catalog
//! scan.  Memory kinds (ASM/DEC/QUE/CON) use the same `kind.dir`/`stem`
//! convention — their stem is `"record"`, so the path is
//! `{kind.dir}/{id:03}/record-{id:03}.md`.

use std::path::{Path, PathBuf};

use crate::integrity;
use crate::map_server::error::MapServerError;

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
    let kind_ref = integrity::kind_by_prefix(key.prefix)
        .ok_or_else(|| MapServerError::BadEntityId(key.canonical()))?;
    let dir = root.join(kind_ref.kind.dir).join(format!("{:03}", key.id));
    Ok(dir.join(format!("{}-{:03}.md", kind_ref.stem, key.id)))
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
}
