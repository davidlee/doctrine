// SPDX-License-Identifier: GPL-3.0-only
//! Presentation-neutral graph projection of `Catalog` (SL-071 PHASE-04).
//! Pure — no cordage dependency, no disk reads. Edges with unresolved or
//! unvalidated targets appear in the edge list but have no target node.
//! `neighbours(depth)` is deferred per design D10.

use std::collections::BTreeMap;

use super::hydrate::{Catalog, CatalogEdge, CatalogKey, EdgeTarget};
#[cfg(test)]
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

pub(crate) use super::hydrate::CatalogKey as NodeKey;

/// A node in the graph — the presentation-neutral view of one entity.
#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct CatalogNode {
    pub(crate) title: String,
    pub(crate) status: Option<String>,
    pub(crate) kind_label: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) memory_type: Option<String>,
}

impl CatalogGraph {
    /// Pure projection of a [`Catalog`] into a graph. Builds the node map
    /// from catalog entities and copies the edge list. No disk, no cordage.
    pub(crate) fn from_catalog(catalog: &Catalog) -> Self {
        let mut nodes = BTreeMap::new();
        for entity in &catalog.entities {
            let key = entity.key.clone();
            nodes.insert(
                key,
                CatalogNode {
                    title: entity.title.clone(),
                    status: entity.status.clone(),
                    kind_label: entity.kind_label,
                    memory_type: entity.memory_type.clone(),
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
    ///
    /// A node not present in the graph silently returns an empty vec —
    /// indistinguishable from a genuine zero-edge node.
    #[cfg_attr(not(test), expect(dead_code, reason = "tested; future consumer"))]
    pub(crate) fn outgoing(&self, node: &NodeKey) -> Vec<&CatalogEdge> {
        let CatalogKey::Numbered(_key) = node else {
            return vec![];
        };
        self.edges.iter().filter(|e| &e.source == node).collect()
    }

    /// All inbound edges whose `target` is `Resolved(key)` matching the given
    /// `node`. Edges with unresolved or unvalidated targets are excluded — an
    /// edge with no target node cannot "point to" a node (D10).
    ///
    /// A node not present in the graph silently returns an empty vec —
    /// indistinguishable from a genuine zero-incoming-edge node.
    #[cfg_attr(not(test), expect(dead_code, reason = "tested; future consumer"))]
    pub(crate) fn incoming(&self, node: &NodeKey) -> Vec<&CatalogEdge> {
        let CatalogKey::Numbered(_key) = node else {
            return vec![];
        };
        self.edges
            .iter()
            .filter(|e| match &e.target {
                EdgeTarget::Resolved(tgt) => tgt == node,
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
    use crate::catalog::test_helpers::*;
    use std::path::Path;

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
        seed_requirement(root, 5);
        seed_adr(root, 2, &[("supersedes", &["ADR-001"])]);
        seed_adr(root, 1, &[]);

        let graph = build_graph(root);

        // 4 entities → 4 nodes
        assert_eq!(graph.nodes.len(), 4, "expected 4 nodes");
        // 2 edges
        assert_eq!(graph.edges.len(), 2, "expected 2 edges");

        // Verify node content for one entity
        let sl001_node = graph.nodes.get(&CatalogKey::Numbered(EntityKey {
            prefix: "SL",
            id: 1,
        }));
        assert!(sl001_node.is_some());
        let node = sl001_node.unwrap();
        assert_eq!(node.title, "S1");
        assert_eq!(node.status.as_deref(), Some("proposed"));
        assert_eq!(node.kind_label, "SL");
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

        let sl_key = CatalogKey::Numbered(EntityKey {
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
        let absent_key = CatalogKey::Numbered(EntityKey {
            prefix: "REQ",
            id: 999,
        });
        let incoming_absent = graph.incoming(&absent_key);
        assert!(
            incoming_absent.is_empty(),
            "incoming must be empty for a target with only UnresolvedRef edges pointing at it"
        );

        // No incoming edges for the source entity either (no one points TO SL-001)
        let sl_key = CatalogKey::Numbered(EntityKey {
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
        seed_requirement(root, 5);

        let graph = build_graph(root);

        // 3 entities → 3 nodes, 2 edges
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);

        // REQ-005 has 2 incoming edges
        let req_key = CatalogKey::Numbered(EntityKey {
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
                    assert_eq!(
                        key,
                        &CatalogKey::Numbered(EntityKey {
                            prefix: "REQ",
                            id: 5
                        })
                    );
                }
                other => panic!("expected Resolved target, got {other:?}"),
            }
        }

        // SL-001 has 1 outgoing edge (to REQ-005)
        let sl001_key = CatalogKey::Numbered(EntityKey {
            prefix: "SL",
            id: 1,
        });
        let sl001_out = graph.outgoing(&sl001_key);
        assert_eq!(sl001_out.len(), 1);
        assert_eq!(sl001_out[0].source.canonical(), "SL-001");
    }
}
