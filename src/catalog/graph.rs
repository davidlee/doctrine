// SPDX-License-Identifier: GPL-3.0-only
//! Presentation-neutral graph projection of `Catalog` (SL-071 PHASE-04).
//! Pure — no cordage dependency, no disk reads. Edges with unresolved or
//! unvalidated targets appear in the edge list but have no target node.
//! `neighbours(depth)` is deferred per design D10.

use std::collections::BTreeMap;

use super::hydrate::{Catalog, CatalogEdge, EdgeTarget};
use super::scan::EntityKey;

// ---------------------------------------------------------------------------
// CatalogGraph — a pure projection of Catalog into BTreeMap + Vec
// ---------------------------------------------------------------------------

/// The presentation-neutral graph: nodes indexed by key, edges as a flat list.
#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct CatalogGraph {
    pub(crate) nodes: BTreeMap<NodeKey, CatalogNode>,
    pub(crate) edges: Vec<CatalogEdge>,
}

/// The identity of a node in the graph. Currently a single variant; future
/// node types (e.g. free-text targets rendered as nodes) could extend it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum NodeKey {
    Entity(EntityKey),
}

impl serde::Serialize for NodeKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            NodeKey::Entity(key) => key.canonical().serialize(serializer),
        }
    }
}

/// A node in the graph — the presentation-neutral view of one entity.
#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct CatalogNode {
    pub(crate) title: String,
    pub(crate) status: Option<String>,
    pub(crate) kind_label: &'static str,
}

impl CatalogGraph {
    /// Pure projection of a [`Catalog`] into a graph. Builds the node map
    /// from catalog entities and copies the edge list. No disk, no cordage.
    pub(crate) fn from_catalog(catalog: &Catalog) -> Self {
        let mut nodes = BTreeMap::new();
        for entity in &catalog.entities {
            let key = NodeKey::Entity(entity.key);
            nodes.insert(
                key,
                CatalogNode {
                    title: entity.title.clone(),
                    status: entity.status.clone(),
                    kind_label: entity.kind.prefix,
                },
            );
        }
        Self {
            nodes,
            edges: catalog.edges.clone(),
        }
    }

    /// All outbound edges whose `source` is the given `node`, including those
    /// with `UnresolvedRef` or `UnvalidatedText` targets. Callers must handle
    /// the case where an edge has no target node in the graph (D10).
    #[cfg_attr(not(test), expect(dead_code, reason = "tested; future consumer"))]
    pub(crate) fn outgoing(&self, node: &NodeKey) -> Vec<&CatalogEdge> {
        let NodeKey::Entity(key) = node;
        self.edges.iter().filter(|e| &e.source == key).collect()
    }

    /// All inbound edges whose `target` is `Resolved(key)` matching the given
    /// `node`. Edges with unresolved or unvalidated targets are excluded — an
    /// edge with no target node cannot "point to" a node (D10).
    #[cfg_attr(not(test), expect(dead_code, reason = "tested; future consumer"))]
    pub(crate) fn incoming(&self, node: &NodeKey) -> Vec<&CatalogEdge> {
        let NodeKey::Entity(key) = node;
        self.edges
            .iter()
            .filter(|e| match &e.target {
                EdgeTarget::Resolved(tgt) => tgt == key,
                EdgeTarget::UnresolvedRef { .. } | EdgeTarget::UnvalidatedText { .. } => false,
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// Write `root/<rel>` with `body`, creating parents.
    fn write(root: &Path, rel: &str, body: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    fn tmp() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    /// Format `[[relation]]` rows from (label, targets) pairs.
    fn relation_rows(edges: &[(&str, &[&str])]) -> String {
        let mut rows = String::new();
        for (label, targets) in edges {
            for t in *targets {
                rows.push_str(&format!(
                    "[[relation]]\nlabel = \"{label}\"\ntarget = \"{t}\"\n"
                ));
            }
        }
        rows
    }

    /// Seed a slice entity (toml + md) with the given `[[relation]]` edges.
    fn seed_slice(root: &Path, id: u32, edges: &[(&str, &[&str])]) {
        write(
            root,
            &format!(".doctrine/slice/{id:03}/slice-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"s{id}\"\ntitle = \"S{id}\"\nstatus = \"proposed\"\n\
                 created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n{}",
                relation_rows(edges)
            ),
        );
        write(
            root,
            &format!(".doctrine/slice/{id:03}/slice-{id:03}.md"),
            "scope\n",
        );
    }

    /// Seed an ADR entity (toml + md) with optional `supersedes` array.
    fn seed_adr(root: &Path, id: u32, supersedes: &[&str]) {
        let rels = if supersedes.is_empty() {
            String::new()
        } else {
            let refs: Vec<String> = supersedes.iter().map(|s| format!("\"{s}\"")).collect();
            format!("\n[relationships]\nsupersedes = [{}]\n", refs.join(", "))
        };
        write(
            root,
            &format!(".doctrine/adr/{id:03}/adr-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"a{id}\"\ntitle = \"A{id}\"\nstatus = \"accepted\"\n\
                 created = \"2026-01-01\"\nupdated = \"2026-01-01\"{rels}"
            ),
        );
        write(
            root,
            &format!(".doctrine/adr/{id:03}/adr-{id:03}.md"),
            "body\n",
        );
    }

    /// Build a CatalogGraph from a small fixture via scan_catalog.
    fn build_graph(root: &Path) -> CatalogGraph {
        let catalog =
            crate::catalog::hydrate::scan_catalog(root).expect("scan_catalog should succeed");
        CatalogGraph::from_catalog(&catalog)
    }

    // -----------------------------------------------------------------------
    // VT-1: from_catalog yields correct node and edge counts
    // -----------------------------------------------------------------------

    #[test]
    fn graph_from_catalog_node_edge_counts() {
        let dir = tmp();
        let root = dir.path();

        // SL-001 → REQ-005 (resolved), ADR-002 → ADR-001 (resolved)
        seed_slice(root, 1, &[("requirements", &["REQ-005"])]);
        slice_requirement(root, 5);
        seed_adr(root, 2, &["ADR-001"]);
        seed_adr(root, 1, &[]);

        let graph = build_graph(root);

        // 4 entities → 4 nodes
        assert_eq!(graph.nodes.len(), 4, "expected 4 nodes");
        // 2 edges
        assert_eq!(graph.edges.len(), 2, "expected 2 edges");

        // Verify node content for one entity
        let sl001_node = graph.nodes.get(&NodeKey::Entity(EntityKey {
            prefix: "SL",
            id: 1,
        }));
        assert!(sl001_node.is_some());
        let node = sl001_node.unwrap();
        assert_eq!(node.title, "S1");
        assert_eq!(node.status.as_deref(), Some("proposed"));
        assert_eq!(node.kind_label, "SL");
    }

    /// Seed a requirement entity (edge target only).
    fn slice_requirement(root: &Path, id: u32) {
        write(
            root,
            &format!(".doctrine/requirement/{id:03}/requirement-{id:03}.toml"),
            &format!("id = {id}\nslug = \"r{id}\"\ntitle = \"R{id}\"\nstatus = \"active\"\n"),
        );
        write(
            root,
            &format!(".doctrine/requirement/{id:03}/requirement-{id:03}.md"),
            "r\n",
        );
    }

    // -----------------------------------------------------------------------
    // VT-2: outgoing returns edges with UnresolvedRef targets
    // -----------------------------------------------------------------------

    #[test]
    fn outgoing_includes_unresolved_targets() {
        let dir = tmp();
        let root = dir.path();

        // SL-001 → REQ-999 (dangling canonical ref)
        seed_slice(root, 1, &[("requirements", &["REQ-999"])]);

        let graph = build_graph(root);

        // 1 node, 1 edge
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.edges.len(), 1);

        let sl_key = NodeKey::Entity(EntityKey {
            prefix: "SL",
            id: 1,
        });
        let outgoing = graph.outgoing(&sl_key);
        assert_eq!(outgoing.len(), 1, "outgoing must include the dangling edge");

        // The edge's target is UnresolvedRef
        let edge = outgoing[0];
        assert_eq!(
            edge.target,
            EdgeTarget::UnresolvedRef {
                raw: "REQ-999".to_string()
            }
        );
    }

    // -----------------------------------------------------------------------
    // VT-3: incoming does NOT return edges with UnresolvedRef/UnvalidatedText
    // -----------------------------------------------------------------------

    #[test]
    fn incoming_excludes_unresolved_and_unvalidated() {
        let dir = tmp();
        let root = dir.path();

        // SL-001 has two edges: one dangling ref, one unvalidated text
        write(
            root,
            ".doctrine/slice/001/slice-001.toml",
            "id = 1\nslug = \"s1\"\ntitle = \"S1\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [[relation]]\nlabel = \"requirements\"\ntarget = \"REQ-999\"\n\
             [[relation]]\nlabel = \"drift\"\ntarget = \"loose talk\"\n",
        );
        write(root, ".doctrine/slice/001/slice-001.md", "scope\n");

        let graph = build_graph(root);

        // No incoming edges for the absent REQ-999 target (UnresolvedRef)
        let absent_key = NodeKey::Entity(EntityKey {
            prefix: "REQ",
            id: 999,
        });
        let incoming_absent = graph.incoming(&absent_key);
        assert!(
            incoming_absent.is_empty(),
            "incoming must be empty for a target with only UnresolvedRef edges pointing at it"
        );

        // No incoming edges for the source entity either (no one points TO SL-001)
        let sl_key = NodeKey::Entity(EntityKey {
            prefix: "SL",
            id: 1,
        });
        let incoming_sl = graph.incoming(&sl_key);
        assert!(incoming_sl.is_empty(), "SL-001 has no incoming edges");
    }

    // -----------------------------------------------------------------------
    // VT-4: incoming correctly returns edges pointing TO a resolved entity
    // -----------------------------------------------------------------------

    #[test]
    fn incoming_resolved_entity() {
        let dir = tmp();
        let root = dir.path();

        // SL-001 → REQ-005, SL-003 → REQ-005 (two sources pointing TO REQ-005)
        seed_slice(root, 1, &[("requirements", &["REQ-005"])]);
        seed_slice(root, 3, &[("requirements", &["REQ-005"])]);
        slice_requirement(root, 5);

        let graph = build_graph(root);

        // 3 entities → 3 nodes, 2 edges
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);

        // REQ-005 has 2 incoming edges
        let req_key = NodeKey::Entity(EntityKey {
            prefix: "REQ",
            id: 5,
        });
        let incoming = graph.incoming(&req_key);
        assert_eq!(incoming.len(), 2, "REQ-005 should have 2 incoming edges");

        // Both incoming edges have source SL-001 and SL-003
        let sources: Vec<String> = incoming.iter().map(|e| e.source.canonical()).collect();
        assert!(sources.contains(&"SL-001".to_string()), "missing SL-001");
        assert!(sources.contains(&"SL-003".to_string()), "missing SL-003");

        // Each edge's target is Resolved(REQ-005)
        for edge in &incoming {
            match &edge.target {
                EdgeTarget::Resolved(key) => {
                    assert_eq!(key.prefix, "REQ");
                    assert_eq!(key.id, 5);
                }
                other => panic!("expected Resolved target, got {other:?}"),
            }
        }

        // SL-001 has 1 outgoing edge (to REQ-005)
        let sl001_key = NodeKey::Entity(EntityKey {
            prefix: "SL",
            id: 1,
        });
        let sl001_out = graph.outgoing(&sl001_key);
        assert_eq!(sl001_out.len(), 1);
        assert_eq!(sl001_out[0].source.canonical(), "SL-001");
    }
}
