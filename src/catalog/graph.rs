// SPDX-License-Identifier: GPL-3.0-only
//! Presentation-neutral graph projection of `Catalog` (SL-071 PHASE-04).
//! Pure — no cordage dependency, no disk reads. Edges with unresolved or
//! unvalidated targets appear in the edge list but have no target node.
//! `neighbours(depth)` is deferred per design D10.

use std::collections::{BTreeMap, BTreeSet};

use super::hydrate::{Catalog, CatalogEdge, CatalogKey, EdgeTarget, Units};
#[cfg(test)]
use super::scan::{EntityKey, ScanMode};

// ---------------------------------------------------------------------------
// CatalogGraph — a pure projection of Catalog into BTreeMap + Vec
// ---------------------------------------------------------------------------

/// The presentation-neutral graph: nodes indexed by key, edges as a flat list.
#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct CatalogGraph {
    pub(crate) nodes: BTreeMap<NodeKey, CatalogNode>,
    pub(crate) edges: Vec<CatalogEdge>,
    /// The project-wide estimation/value display units, projected verbatim from
    /// the source [`Catalog`] (SL-103 PHASE-03, design §5.5). Sealed onto the
    /// graph contract so `catalog graph` and `/api/graph` emit one top-level
    /// `units` resolution. Field names are graph-neutral (clear of the SPEC-001
    /// whole-word denylist).
    pub(crate) units: Units,
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
            units: catalog.units.clone(),
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

    // -----------------------------------------------------------------------
    // Projection filters (SL-226 PHASE-01) — each consumes self, returns Self
    // -----------------------------------------------------------------------

    /// Keep nodes whose uppercase kind prefix is in `prefixes`, dropping
    /// incident edges (D11). `prefixes` arrives already uppercase — validation
    /// is the command layer's job.
    pub(crate) fn filter_kinds(self, prefixes: &BTreeSet<String>) -> Self {
        let keep: BTreeSet<NodeKey> = self
            .nodes
            .keys()
            .filter(|k| {
                let kind = match k {
                    CatalogKey::Numbered(ek) => ek.prefix,
                    CatalogKey::Memory(_) => "MEM",
                };
                prefixes.contains(kind)
            })
            .cloned()
            .collect();
        self.filter_nodes(&keep)
    }

    /// Keep edges whose label `name()` equals `label` (D4). Never drops a node.
    /// A bare `"references"` match keeps roled references edges (D14).
    pub(crate) fn filter_label(self, label: &str) -> Self {
        let edges: Vec<CatalogEdge> = self
            .edges
            .into_iter()
            .filter(|e| e.label.name() == label)
            .collect();
        Self { edges, ..self }
    }

    /// Drop every `CatalogKey::Memory` node and all incident edges (D12).
    pub(crate) fn exclude_memory(self) -> Self {
        let keep: BTreeSet<NodeKey> = self
            .nodes
            .keys()
            .filter(|k| !matches!(k, CatalogKey::Memory(_)))
            .cloned()
            .collect();
        self.filter_nodes(&keep)
    }

    /// Terminal op: drop nodes with zero incident edges (D4).
    /// An edge incidents a node if it sources from it or resolves to it.
    pub(crate) fn drop_isolated(self) -> Self {
        let referenced: BTreeSet<NodeKey> = {
            let mut set = BTreeSet::new();
            for e in &self.edges {
                set.insert(e.source.clone());
                if let EdgeTarget::Resolved(ref tgt) = e.target {
                    set.insert(tgt.clone());
                }
            }
            set
        };
        let nodes: BTreeMap<NodeKey, CatalogNode> = self
            .nodes
            .into_iter()
            .filter(|(k, _)| referenced.contains(k))
            .collect();
        Self { nodes, ..self }
    }

    /// True iff the graph contains the given node key.
    pub(crate) fn contains(&self, key: &NodeKey) -> bool {
        self.nodes.contains_key(key)
    }

    // -----------------------------------------------------------------------
    // neighbourhood — undirected BFS bounded subgraph (SL-226 PHASE-02)
    // -----------------------------------------------------------------------

    /// Undirected, breadth-bounded BFS returning the owned subgraph within
    /// `depth` hops of `focus` (design §5.2). Boundary nodes (dist == depth)
    /// are included but their incident edges are NOT collected (D9).
    ///
    /// PRECONDITION: `focus` is present in `self.nodes`.
    pub(crate) fn neighbourhood(self, focus: &NodeKey, depth: u32) -> Self {
        // 1. Build adjacency maps in one pass over edges (O(E)).
        let mut out_adj: BTreeMap<NodeKey, Vec<usize>> = BTreeMap::new();
        let mut in_adj: BTreeMap<NodeKey, Vec<usize>> = BTreeMap::new();
        for (idx, edge) in self.edges.iter().enumerate() {
            out_adj.entry(edge.source.clone()).or_default().push(idx);
            if let EdgeTarget::Resolved(ref tgt) = edge.target {
                in_adj.entry(tgt.clone()).or_default().push(idx);
            }
        }

        // 2. Undirected BFS.
        let mut visited: BTreeSet<NodeKey> = BTreeSet::new();
        let mut collected: BTreeSet<usize> = BTreeSet::new();
        let mut queue: std::collections::VecDeque<(NodeKey, u32)> =
            std::collections::VecDeque::new();

        visited.insert(focus.clone());
        queue.push_back((focus.clone(), 0));

        while let Some((node, dist)) = queue.pop_front() {
            if dist >= depth {
                continue; // boundary node — do NOT expand
            }

            // Expand outbound edges.
            #[expect(clippy::indexing_slicing, reason = "adjacency indices from enumerate")]
            {
                for &idx in out_adj.get(&node).into_iter().flatten() {
                    collected.insert(idx);
                    let edge = &self.edges[idx];
                    if let EdgeTarget::Resolved(ref tgt) = edge.target
                        && visited.insert(tgt.clone())
                    {
                        queue.push_back((tgt.clone(), dist + 1));
                    }
                }

                // Expand inbound edges.
                for &idx in in_adj.get(&node).into_iter().flatten() {
                    collected.insert(idx);
                    let edge = &self.edges[idx];
                    if visited.insert(edge.source.clone()) {
                        queue.push_back((edge.source.clone(), dist + 1));
                    }
                }
            }
        }

        // 3. Rebuild the owned subset.
        let nodes: BTreeMap<NodeKey, CatalogNode> = self
            .nodes
            .into_iter()
            .filter(|(k, _)| visited.contains(k))
            .collect();
        #[expect(clippy::indexing_slicing, reason = "collected indices from enumerate")]
        let edges: Vec<CatalogEdge> = collected
            .into_iter()
            .map(|idx| self.edges[idx].clone())
            .collect();

        Self {
            nodes,
            edges,
            units: self.units,
        }
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Keep only the listed nodes; drop incident edges (edges whose source is
    /// dropped OR whose `Resolved` target is dropped). Dangling-target edges
    /// (UnresolvedRef/UnvalidatedText) live/die with their source only (D11).
    fn filter_nodes(self, keep: &BTreeSet<NodeKey>) -> Self {
        let nodes: BTreeMap<NodeKey, CatalogNode> = self
            .nodes
            .into_iter()
            .filter(|(k, _)| keep.contains(k))
            .collect();
        let edges: Vec<CatalogEdge> = self
            .edges
            .into_iter()
            .filter(|e| {
                if !keep.contains(&e.source) {
                    return false;
                }
                match &e.target {
                    EdgeTarget::Resolved(tgt) => keep.contains(tgt),
                    EdgeTarget::UnresolvedRef { .. } | EdgeTarget::UnvalidatedText { .. } => true,
                }
            })
            .collect();
        Self {
            nodes,
            edges,
            units: self.units,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;
    use crate::catalog::hydrate::{CatalogEdgeLabel, EdgeOrigin};
    use crate::catalog::test_helpers::*;
    use crate::relation::Role;
    use std::path::Path;

    /// Build a CatalogGraph from a small fixture via scan_catalog.
    fn build_graph(root: &Path) -> CatalogGraph {
        let catalog = crate::catalog::hydrate::scan_catalog(root, ScanMode::default())
            .expect("scan_catalog should succeed");
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
        seed_slice(root, 1, &[("references(implements)", &["REQ-005"])]);
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
        seed_slice(root, 1, &[("references(implements)", &["REQ-999"])]);

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
        seed_slice(root, 1, &[("references(implements)", &["REQ-005"])]);
        seed_slice(root, 3, &[("references(implements)", &["REQ-005"])]);
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

    // =======================================================================
    // SL-103 PHASE-03: facet + units projection onto the graph contract
    // =======================================================================

    /// Seed a slice with `[estimate]`/`[value]` table bodies appended verbatim
    /// after the meta keys (the standard `seed_slice` writes no facets). A
    /// slice's typed read validates a present `[estimate]`, so this seeds only
    /// well-formed facets; malformed-facet isolation is exercised via an ADR
    /// (`seed_adr_with_facets`), the kind-agnostic scan path.
    fn seed_slice_with_facets(root: &Path, id: u32, facets: &str) {
        write(
            root,
            &format!(".doctrine/slice/{id:03}/slice-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"s{id}\"\ntitle = \"S{id}\"\nstatus = \"proposed\"\n\
                 created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n{facets}"
            ),
        );
        write(
            root,
            &format!(".doctrine/slice/{id:03}/slice-{id:03}.md"),
            "scope\n",
        );
    }

    fn node_for<'a>(graph: &'a CatalogGraph, canonical: &str) -> &'a CatalogNode {
        graph
            .nodes
            .iter()
            .find(|(k, _)| k.canonical() == canonical)
            .map(|(_, n)| n)
            .unwrap_or_else(|| panic!("no node for {canonical}"))
    }

    /// PHASE-09: the graph no longer carries estimate/value facets (deleted).
    /// A faceted seed still yields a node; facet fields are absent.
    #[test]
    fn graph_node_no_longer_carries_facets() {
        let dir = tmp();
        let root = dir.path();
        seed_slice_with_facets(
            root,
            1,
            "[estimate]\nlower = 2\nupper = 8\n\n[value]\nvalue = 5\n",
        );

        let graph = build_graph(root);
        let node = node_for(&graph, "SL-001");
        assert_eq!(node.title, "S1");
        assert_eq!(graph.units.estimation, "espresso_shots");
        assert_eq!(graph.units.value, "magic_beans");
    }

    /// PHASE-09: contract JSON — estimate/value no longer appear on nodes.
    #[test]
    fn graph_contract_json_no_facet_keys() {
        let dir = tmp();
        let root = dir.path();
        seed_slice_with_facets(
            root,
            1,
            "[estimate]\nlower = 2\nupper = 8\n\n[value]\nvalue = 5\n",
        );

        let graph = build_graph(root);
        let json = serde_json::to_value(&graph).unwrap();

        assert!(json.get("nodes").is_some(), "missing nodes");
        assert!(json.get("edges").is_some(), "missing edges");
        assert!(json.get("units").is_some(), "missing units");
        let node = &json["nodes"]["SL-001"];
        assert!(
            node.get("estimate").is_none(),
            "estimate key no longer emitted"
        );
        assert!(node.get("value").is_none(), "value key no longer emitted");
    }

    /// SL-149 PHASE-04: the web-graph edge carries the `role` payload for a `references`
    /// edge, so the rendered edge label can show the role verb (`references(implements)`).
    /// A label-only edge omits the `role` key entirely (`skip_serializing_if`), so the
    /// shipped edge contract stays byte-identical for every non-`references` edge.    /// SL-149 PHASE-04: the web-graph edge carries the `role` payload for a `references`
    /// edge, so the rendered edge label can show the role verb (`references(implements)`).
    /// A label-only edge omits the `role` key entirely (`skip_serializing_if`), so the
    /// shipped edge contract stays byte-identical for every non-`references` edge.
    #[test]
    fn graph_references_edge_serializes_role_label_only_omits_it() {
        let dir = tmp();
        let root = dir.path();
        // SL-001 implements REQ-005 (roled) AND supersedes SL-002 (label-only).
        crate::catalog::test_helpers::write(
            root,
            ".doctrine/slice/001/slice-001.toml",
            "id = 1\nslug = \"s1\"\ntitle = \"S1\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"implements\"\ntarget = \"REQ-005\"\n\
             [[relation]]\nlabel = \"supersedes\"\ntarget = \"SL-002\"\n",
        );
        crate::catalog::test_helpers::write(root, ".doctrine/slice/001/slice-001.md", "scope\n");
        seed_requirement(root, 5);

        let graph = build_graph(root);
        let json = serde_json::to_value(&graph).unwrap();
        let edges = json["edges"].as_array().expect("edges array");

        // The `references` edge carries `role: "Implements"` (the Role serde variant,
        // matching the PascalCase `Validated` label convention the web layer snake-cases).
        let ref_edge = edges
            .iter()
            .find(|e| e["label"]["Validated"] == "References")
            .expect("references edge present");
        assert_eq!(
            ref_edge["role"], "Implements",
            "references edge carries its role payload: {ref_edge}"
        );

        // The label-only `supersedes` edge omits the `role` key entirely.
        let sup_edge = edges
            .iter()
            .find(|e| e["label"]["Validated"] == "Supersedes")
            .expect("supersedes edge present");
        assert!(
            sup_edge.get("role").is_none(),
            "label-only edge omits role: {sup_edge}"
        );
    }

    // =======================================================================
    // SL-226 PHASE-02 — neighbourhood BFS tests
    // =======================================================================

    /// Build a small hand-constructed graph with nodes A, B, C in a triangle:
    /// edges: 0=A→B, 1=A→C, 2=B→C.
    fn triangle_graph() -> CatalogGraph {
        let a = CatalogKey::Numbered(EntityKey {
            prefix: "SL",
            id: 1,
        });
        let b = CatalogKey::Numbered(EntityKey {
            prefix: "REQ",
            id: 2,
        });
        let c = CatalogKey::Numbered(EntityKey {
            prefix: "ADR",
            id: 3,
        });

        let mut nodes = BTreeMap::new();
        for (key, title, kind_label) in [(&a, "A", "SL"), (&b, "B", "REQ"), (&c, "C", "ADR")] {
            nodes.insert(
                key.clone(),
                CatalogNode {
                    title: title.to_string(),
                    status: None,
                    kind_label,
                    memory_type: None,
                },
            );
        }

        let edge = |source: &NodeKey, target: EdgeTarget| CatalogEdge {
            source: source.clone(),
            label: CatalogEdgeLabel::Validated(crate::relation::RelationLabel::References),
            role: None,
            descriptor: None,
            target,
            origin: EdgeOrigin {
                file: std::path::PathBuf::from("fixture"),
                field: None,
            },
        };

        let edges = vec![
            edge(&a, EdgeTarget::Resolved(b.clone())), // 0: A→B
            edge(&a, EdgeTarget::Resolved(c.clone())), // 1: A→C
            edge(&b, EdgeTarget::Resolved(c.clone())), // 2: B→C
        ];

        CatalogGraph {
            nodes,
            edges,
            units: Units {
                estimation: "hours".to_string(),
                value: "points".to_string(),
            },
        }
    }

    #[test]
    fn neighbourhood_triangle_boundary_edge_excluded_depth1() {
        let graph = triangle_graph();
        let a = CatalogKey::Numbered(EntityKey {
            prefix: "SL",
            id: 1,
        });

        let sub = graph.neighbourhood(&a, 1);

        // Nodes: A (focus), B, C (both boundary at dist=1)
        assert_eq!(sub.nodes.len(), 3, "focus + 2 boundary nodes");

        // Edges: A→B and A→C (from expanded focus A); B→C NOT collected
        // because B and C are boundary (dist >= depth).
        assert_eq!(sub.edges.len(), 2, "only focus-expanded edges collected");

        let sources: Vec<String> = sub.edges.iter().map(|e| e.source.canonical()).collect();
        assert!(
            sources.iter().all(|s| s == "SL-001"),
            "only edges from A (the focus) should be collected"
        );

        // Verify collected edges are A→B and A→C.
        let targets: BTreeSet<String> = sub
            .edges
            .iter()
            .map(|e| match &e.target {
                EdgeTarget::Resolved(k) => k.canonical(),
                _ => unreachable!(),
            })
            .collect();
        assert!(targets.contains("REQ-002"));
        assert!(targets.contains("ADR-003"));
    }

    #[test]
    fn neighbourhood_triangle_boundary_edge_excluded_depth0() {
        let graph = triangle_graph();
        let a = CatalogKey::Numbered(EntityKey {
            prefix: "SL",
            id: 1,
        });

        let sub = graph.neighbourhood(&a, 0);

        // depth 0: only the focus, zero edges.
        assert_eq!(sub.nodes.len(), 1, "focus only");
        assert!(sub.nodes.contains_key(&a));
        assert_eq!(sub.edges.len(), 0, "no edges at depth 0");
    }

    #[test]
    fn neighbourhood_triangle_boundary_edge_excluded_depth2() {
        let graph = triangle_graph();
        let a = CatalogKey::Numbered(EntityKey {
            prefix: "SL",
            id: 1,
        });

        let sub = graph.neighbourhood(&a, 2);

        // depth 2: all 3 nodes, all 3 edges.
        assert_eq!(sub.nodes.len(), 3);
        assert_eq!(sub.edges.len(), 3, "all edges within 2 hops");
    }

    #[test]
    fn neighbourhood_undirected_reach_through_incoming() {
        // Graph: A→B, C→B. Focus at B — should reach A and C via inbound edges.
        let a = CatalogKey::Numbered(EntityKey {
            prefix: "SL",
            id: 1,
        });
        let b = CatalogKey::Numbered(EntityKey {
            prefix: "REQ",
            id: 2,
        });
        let c = CatalogKey::Numbered(EntityKey {
            prefix: "ADR",
            id: 3,
        });

        let mut nodes = BTreeMap::new();
        for (key, title, kind_label) in [(&a, "A", "SL"), (&b, "B", "REQ"), (&c, "C", "ADR")] {
            nodes.insert(
                key.clone(),
                CatalogNode {
                    title: title.to_string(),
                    status: None,
                    kind_label,
                    memory_type: None,
                },
            );
        }

        let edge = |source: &NodeKey, target: EdgeTarget| CatalogEdge {
            source: source.clone(),
            label: CatalogEdgeLabel::Validated(crate::relation::RelationLabel::References),
            role: None,
            descriptor: None,
            target,
            origin: EdgeOrigin {
                file: std::path::PathBuf::from("fixture"),
                field: None,
            },
        };

        let edges = vec![
            edge(&a, EdgeTarget::Resolved(b.clone())), // 0: A→B
            edge(&c, EdgeTarget::Resolved(b.clone())), // 1: C→B
        ];

        let graph = CatalogGraph {
            nodes,
            edges,
            units: Units {
                estimation: "hours".to_string(),
                value: "points".to_string(),
            },
        };

        let sub = graph.neighbourhood(&b, 1);

        // Focus B + A and C reached via inbound edges.
        assert_eq!(sub.nodes.len(), 3, "B + inbound sources A and C");
        assert_eq!(sub.edges.len(), 2, "both edges collected (B expanded)");
    }

    #[test]
    fn neighbourhood_boundary_dangling_edge_not_collected_and_expanded() {
        // Graph: A→B (resolved), A→UNRESOLVED (dangling outbound),
        //        C→A (resolved inbound).
        // Focus A, depth 1:
        //   - A is expanded → A→B and A→UNRESOLVED collected.
        //   - B is boundary → B's edges (if any) NOT collected.
        //   - C is boundary → C's edges NOT collected.
        //   - Dangling target of A→UNRESOLVED is NOT enqueued.
        //
        // To also test the boundary-dangling case: add a dangling edge from B
        // (boundary node) and verify it is NOT collected.

        let a = CatalogKey::Numbered(EntityKey {
            prefix: "SL",
            id: 1,
        });
        let b = CatalogKey::Numbered(EntityKey {
            prefix: "REQ",
            id: 2,
        });
        let c = CatalogKey::Numbered(EntityKey {
            prefix: "ADR",
            id: 3,
        });

        let mut nodes = BTreeMap::new();
        for (key, title, kind_label) in [(&a, "A", "SL"), (&b, "B", "REQ"), (&c, "C", "ADR")] {
            nodes.insert(
                key.clone(),
                CatalogNode {
                    title: title.to_string(),
                    status: None,
                    kind_label,
                    memory_type: None,
                },
            );
        }

        let edge = |source: &NodeKey, target: EdgeTarget| CatalogEdge {
            source: source.clone(),
            label: CatalogEdgeLabel::Validated(crate::relation::RelationLabel::References),
            role: None,
            descriptor: None,
            target,
            origin: EdgeOrigin {
                file: std::path::PathBuf::from("fixture"),
                field: None,
            },
        };

        let edges = vec![
            edge(&a, EdgeTarget::Resolved(b.clone())), // 0: A→B
            edge(
                &a,
                EdgeTarget::UnresolvedRef {
                    raw: "UNKNOWN".to_string(),
                },
            ), // 1: A→UNRESOLVED (dangling, expanded node)
            edge(&c, EdgeTarget::Resolved(a.clone())), // 2: C→A
            edge(
                &b,
                EdgeTarget::UnresolvedRef {
                    raw: "B_DANGLING".to_string(),
                },
            ), // 3: B→dangling (boundary node)
        ];

        let graph = CatalogGraph {
            nodes,
            edges,
            units: Units {
                estimation: "hours".to_string(),
                value: "points".to_string(),
            },
        };

        let sub = graph.neighbourhood(&a, 1);

        // Nodes: A (focus), B, C (boundary).
        assert_eq!(sub.nodes.len(), 3);

        // Edges collected:
        //   - Edge 0 (A→B): yes (A expanded)
        //   - Edge 1 (A→UNRESOLVED): yes (A expanded, dangling target not enqueued)
        //   - Edge 2 (C→A): yes (A expanded, inbound from C)
        //   - Edge 3 (B→dangling): NO (B is boundary, not expanded)
        assert_eq!(
            sub.edges.len(),
            3,
            "edges from expanded A collected; boundary B's dangling edge excluded"
        );

        // Confirm the boundary node's dangling edge (idx 3) is absent.
        let has_b_dangling = sub
            .edges
            .iter()
            .any(|e| matches!(&e.target, EdgeTarget::UnresolvedRef { raw } if raw == "B_DANGLING"));
        assert!(
            !has_b_dangling,
            "boundary node's dangling edge must NOT be collected"
        );

        // Confirm the expanded node's dangling edge (idx 1) IS present.
        let has_a_dangling = sub
            .edges
            .iter()
            .any(|e| matches!(&e.target, EdgeTarget::UnresolvedRef { raw } if raw == "UNKNOWN"));
        assert!(
            has_a_dangling,
            "expanded node's dangling edge must be collected"
        );

        // Dangling target node not enqueued (no node to add).
        // Verified by node count already — only 3 real nodes, no phantom.
    }

    #[test]
    fn neighbourhood_duplicate_same_tuple_edges_both_collected() {
        // D13: duplicate same-tuple edges (distinct indices) are both collected.
        let a = CatalogKey::Numbered(EntityKey {
            prefix: "SL",
            id: 1,
        });
        let b = CatalogKey::Numbered(EntityKey {
            prefix: "REQ",
            id: 2,
        });

        let mut nodes = BTreeMap::new();
        nodes.insert(
            a.clone(),
            CatalogNode {
                title: "A".to_string(),
                status: None,
                kind_label: "SL",
                memory_type: None,
            },
        );
        nodes.insert(
            b.clone(),
            CatalogNode {
                title: "B".to_string(),
                status: None,
                kind_label: "REQ",
                memory_type: None,
            },
        );

        let edge_template = CatalogEdge {
            source: a.clone(),
            label: CatalogEdgeLabel::Validated(crate::relation::RelationLabel::References),
            role: None,
            descriptor: None,
            target: EdgeTarget::Resolved(b.clone()),
            origin: EdgeOrigin {
                file: std::path::PathBuf::from("fixture"),
                field: None,
            },
        };

        let graph = CatalogGraph {
            nodes,
            edges: vec![edge_template.clone(), edge_template],
            units: Units {
                estimation: "hours".to_string(),
                value: "points".to_string(),
            },
        };

        let sub = graph.neighbourhood(&a, 1);

        assert_eq!(sub.edges.len(), 2, "both duplicate edges collected (D13)");
    }

    // =======================================================================
    // SL-226 PHASE-01 — projection filter tests
    // =======================================================================

    // -----------------------------------------------------------------------
    // R3 regression net: serialization identity across filter operations
    // -----------------------------------------------------------------------

    /// Build a rich fixture covering every EdgeTarget variant, roled
    /// references, descriptor, and origin variance — then assert serialization
    /// identity before and after no-op operations.
    #[test]
    fn unfiltered_projection_serializes_identically() {
        // Hand-build a CatalogGraph with maximum variant coverage.
        let sl_key = CatalogKey::Numbered(EntityKey {
            prefix: "SL",
            id: 1,
        });
        let req_key = CatalogKey::Numbered(EntityKey {
            prefix: "REQ",
            id: 5,
        });
        let mem_key = CatalogKey::Memory("mem_abc".to_string());

        let mut nodes = BTreeMap::new();
        nodes.insert(
            sl_key.clone(),
            CatalogNode {
                title: "S1".to_string(),
                status: Some("proposed".to_string()),
                kind_label: "SL",
                memory_type: None,
            },
        );
        nodes.insert(
            req_key.clone(),
            CatalogNode {
                title: "R5".to_string(),
                status: Some("active".to_string()),
                kind_label: "REQ",
                memory_type: None,
            },
        );
        nodes.insert(
            mem_key.clone(),
            CatalogNode {
                title: "memory node".to_string(),
                status: None,
                kind_label: "MEM",
                memory_type: Some("assumption".to_string()),
            },
        );

        let edges = vec![
            // Resolved target, roled references edge with descriptor
            CatalogEdge {
                source: sl_key.clone(),
                label: CatalogEdgeLabel::Validated(crate::relation::RelationLabel::References),
                role: Some(Role::Implements),
                descriptor: Some("core concern".to_string()),
                target: EdgeTarget::Resolved(req_key.clone()),
                origin: EdgeOrigin {
                    file: std::path::PathBuf::from("slice/001"),
                    field: Some("references".to_string()),
                },
            },
            // UnresolvedRef target, label-only edge (no role, no descriptor)
            CatalogEdge {
                source: sl_key.clone(),
                label: CatalogEdgeLabel::Validated(crate::relation::RelationLabel::Supersedes),
                role: None,
                descriptor: None,
                target: EdgeTarget::UnresolvedRef {
                    raw: "SL-999".to_string(),
                },
                origin: EdgeOrigin {
                    file: std::path::PathBuf::from("slice/001"),
                    field: Some("supersedes".to_string()),
                },
            },
            // UnvalidatedText target, origin with field=None
            CatalogEdge {
                source: mem_key.clone(),
                label: CatalogEdgeLabel::Raw("drift".to_string()),
                role: None,
                descriptor: None,
                target: EdgeTarget::UnvalidatedText {
                    raw: "loose talk".to_string(),
                },
                origin: EdgeOrigin {
                    file: std::path::PathBuf::from("memory/mem_abc"),
                    field: None,
                },
            },
        ];

        let units = Units {
            estimation: "espresso_shots".to_string(),
            value: "magic_beans".to_string(),
        };

        let graph = CatalogGraph {
            nodes,
            edges,
            units,
        };

        let original_json = serde_json::to_value(&graph).unwrap();

        // Clone = no-op baseline
        let clone_json = serde_json::to_value(&graph.clone()).unwrap();
        assert_eq!(
            original_json, clone_json,
            "clone must not perturb serialization"
        );

        // Round-trip through a filter_kinds covering all kinds = identity
        let mut all_prefixes = BTreeSet::new();
        all_prefixes.insert("SL".to_string());
        all_prefixes.insert("REQ".to_string());
        all_prefixes.insert("MEM".to_string());
        let filtered = graph.clone().filter_kinds(&all_prefixes);
        let filtered_json = serde_json::to_value(&filtered).unwrap();
        assert_eq!(
            original_json, filtered_json,
            "filter_kinds with all kinds must not perturb serialization"
        );

        // filter_label with a matching label name = identity for that edge's
        // label, drops others — test the all-matching case by filtering on
        // "references" then on "supersedes" etc (edges don't have to all
        // survive — we just want the contract shape to be stable).
        // filter_label never drops nodes, so nodes invariant holds.
        let label_filtered = graph.clone().filter_label("references");
        let label_json = serde_json::to_value(&label_filtered).unwrap();
        // nodes must be identical
        assert_eq!(
            original_json["nodes"], label_json["nodes"],
            "filter_label must not drop nodes"
        );
        // units must be identical
        assert_eq!(
            original_json["units"], label_json["units"],
            "filter_label must not perturb units"
        );
        // Edges: only references edges survive
        let label_edges = label_json["edges"].as_array().unwrap();
        assert_eq!(label_edges.len(), 1, "only the references edge survives");
        assert_eq!(
            label_edges[0]["descriptor"], "core concern",
            "descriptor survives filter_label"
        );
    }

    // -----------------------------------------------------------------------
    // filter_kinds semantics
    // -----------------------------------------------------------------------

    #[test]
    fn filter_kinds_keeps_listed_prefixes_or_union() {
        let dir = tmp();
        let root = dir.path();

        seed_slice(root, 1, &[("references(implements)", &["REQ-005"])]);
        seed_requirement(root, 5);
        seed_adr(root, 2, &[("supersedes", &["ADR-001"])]);
        seed_knowledge(root, "ASM", 3, "K3", "active");

        let graph = build_graph(root);
        // 4 nodes: SL-001, REQ-005, ADR-002, ASM-003 (memory)
        assert_eq!(graph.nodes.len(), 4);

        // Keep only SL and REQ
        let mut prefixes = BTreeSet::new();
        prefixes.insert("SL".to_string());
        prefixes.insert("REQ".to_string());
        let filtered = graph.filter_kinds(&prefixes);

        // 2 nodes survive
        assert_eq!(filtered.nodes.len(), 2);
        assert!(filtered.contains(&CatalogKey::Numbered(EntityKey {
            prefix: "SL",
            id: 1,
        })));
        assert!(filtered.contains(&CatalogKey::Numbered(EntityKey {
            prefix: "REQ",
            id: 5,
        })));
        // ADR and MEM dropped
        assert!(!filtered.contains(&CatalogKey::Numbered(EntityKey {
            prefix: "ADR",
            id: 2,
        })));
        // Incident edges: SL→REQ edge survives (both ends kept),
        // ADR→ADR edge dropped (source dropped).
        assert_eq!(filtered.edges.len(), 1);
        assert_eq!(filtered.edges[0].source.canonical(), "SL-001");
        assert!(matches!(
            filtered.edges[0].target,
            EdgeTarget::Resolved(ref k) if k.canonical() == "REQ-005"
        ));
    }

    #[test]
    fn filter_kinds_duplicate_edges_both_survive() {
        // D13: edge identity is list index, not field tuple.
        // Build two identical edges; both must survive when their kinds match.
        let sl_key = CatalogKey::Numbered(EntityKey {
            prefix: "SL",
            id: 1,
        });
        let req_key = CatalogKey::Numbered(EntityKey {
            prefix: "REQ",
            id: 5,
        });

        let mut nodes = BTreeMap::new();
        nodes.insert(
            sl_key.clone(),
            CatalogNode {
                title: "S1".to_string(),
                status: Some("proposed".to_string()),
                kind_label: "SL",
                memory_type: None,
            },
        );
        nodes.insert(
            req_key.clone(),
            CatalogNode {
                title: "R5".to_string(),
                status: Some("active".to_string()),
                kind_label: "REQ",
                memory_type: None,
            },
        );

        let edge_template = CatalogEdge {
            source: sl_key.clone(),
            label: CatalogEdgeLabel::Validated(crate::relation::RelationLabel::References),
            role: Some(crate::relation::Role::Implements),
            descriptor: None,
            target: EdgeTarget::Resolved(req_key.clone()),
            origin: EdgeOrigin {
                file: std::path::PathBuf::from("slice/001"),
                field: Some("references".to_string()),
            },
        };

        let graph = CatalogGraph {
            nodes,
            edges: vec![edge_template.clone(), edge_template],
            units: Units {
                estimation: "hours".to_string(),
                value: "points".to_string(),
            },
        };

        let mut prefixes = BTreeSet::new();
        prefixes.insert("SL".to_string());
        prefixes.insert("REQ".to_string());
        let filtered = graph.filter_kinds(&prefixes);

        assert_eq!(
            filtered.edges.len(),
            2,
            "duplicate edges must both survive (identity = list index, D13)"
        );
    }

    // -----------------------------------------------------------------------
    // filter_label semantics
    // -----------------------------------------------------------------------

    #[test]
    fn filter_label_keeps_all_nodes_drops_nonmatching_edges() {
        let dir = tmp();
        let root = dir.path();

        seed_slice(
            root,
            1,
            &[
                ("references(implements)", &["REQ-005"]),
                ("supersedes", &["SL-002"]),
            ],
        );
        seed_slice(root, 2, &[]);
        seed_requirement(root, 5);

        let graph = build_graph(root);
        // 4 nodes: SL-001, SL-002, REQ-005, ADR? — no ADR seeded, just 3.
        // Wait: seed_slice(2) adds SL-002, so 3 nodes total.
        assert_eq!(graph.nodes.len(), 3);
        // 2 edges: references(implements)→REQ-005, supersedes→SL-002
        assert_eq!(graph.edges.len(), 2);

        let filtered = graph.filter_label("references");

        // All 3 nodes survive
        assert_eq!(filtered.nodes.len(), 3);
        // Only the references edge survives
        assert_eq!(filtered.edges.len(), 1);
        let edge = &filtered.edges[0];
        assert_eq!(edge.label.name(), "references");
    }

    #[test]
    fn filter_label_bare_references_matches_roled_edge_d14() {
        // D14: a bare "references" filter keeps roled references edges.
        let sl_key = CatalogKey::Numbered(EntityKey {
            prefix: "SL",
            id: 1,
        });
        let req_key = CatalogKey::Numbered(EntityKey {
            prefix: "REQ",
            id: 5,
        });

        let mut nodes = BTreeMap::new();
        nodes.insert(
            sl_key.clone(),
            CatalogNode {
                title: "S1".to_string(),
                status: Some("proposed".to_string()),
                kind_label: "SL",
                memory_type: None,
            },
        );
        nodes.insert(
            req_key.clone(),
            CatalogNode {
                title: "R5".to_string(),
                status: Some("active".to_string()),
                kind_label: "REQ",
                memory_type: None,
            },
        );

        let roled_edge = CatalogEdge {
            source: sl_key.clone(),
            label: CatalogEdgeLabel::Validated(crate::relation::RelationLabel::References),
            role: Some(crate::relation::Role::Implements),
            descriptor: Some("core".to_string()),
            target: EdgeTarget::Resolved(req_key.clone()),
            origin: EdgeOrigin {
                file: std::path::PathBuf::from("slice/001"),
                field: Some("references".to_string()),
            },
        };

        let graph = CatalogGraph {
            nodes,
            edges: vec![roled_edge],
            units: Units {
                estimation: "hours".to_string(),
                value: "points".to_string(),
            },
        };

        let filtered = graph.filter_label("references");
        assert_eq!(
            filtered.edges.len(),
            1,
            "bare 'references' filter must keep roled references edge (D14)"
        );
        assert_eq!(filtered.edges[0].label.name(), "references");
        assert!(filtered.edges[0].role.is_some());
    }

    // -----------------------------------------------------------------------
    // exclude_memory semantics
    // -----------------------------------------------------------------------

    #[test]
    fn exclude_memory_removes_memory_nodes_and_incident_edges() {
        let sl_key = CatalogKey::Numbered(EntityKey {
            prefix: "SL",
            id: 1,
        });
        let req_key = CatalogKey::Numbered(EntityKey {
            prefix: "REQ",
            id: 5,
        });
        let mem_key = CatalogKey::Memory("mem_abc".to_string());

        let mut nodes = BTreeMap::new();
        nodes.insert(
            sl_key.clone(),
            CatalogNode {
                title: "S1".to_string(),
                status: Some("proposed".to_string()),
                kind_label: "SL",
                memory_type: None,
            },
        );
        nodes.insert(
            req_key.clone(),
            CatalogNode {
                title: "R5".to_string(),
                status: Some("active".to_string()),
                kind_label: "REQ",
                memory_type: None,
            },
        );
        nodes.insert(
            mem_key.clone(),
            CatalogNode {
                title: "M".to_string(),
                status: None,
                kind_label: "MEM",
                memory_type: Some("assumption".to_string()),
            },
        );

        let edges = vec![
            // SL→REQ (both non-memory, survives)
            CatalogEdge {
                source: sl_key.clone(),
                label: CatalogEdgeLabel::Validated(crate::relation::RelationLabel::References),
                role: Some(crate::relation::Role::Implements),
                descriptor: None,
                target: EdgeTarget::Resolved(req_key.clone()),
                origin: EdgeOrigin {
                    file: std::path::PathBuf::from("slice/001"),
                    field: Some("references".to_string()),
                },
            },
            // MEM→SL (source is memory → edge dropped)
            CatalogEdge {
                source: mem_key.clone(),
                label: CatalogEdgeLabel::Raw("related".to_string()),
                role: None,
                descriptor: None,
                target: EdgeTarget::Resolved(sl_key.clone()),
                origin: EdgeOrigin {
                    file: std::path::PathBuf::from("memory/mem_abc"),
                    field: None,
                },
            },
            // SL→MEM (target is memory → edge dropped because target Resolved
            // references a dropped node)
            CatalogEdge {
                source: sl_key.clone(),
                label: CatalogEdgeLabel::Validated(crate::relation::RelationLabel::Related),
                role: None,
                descriptor: None,
                target: EdgeTarget::Resolved(mem_key.clone()),
                origin: EdgeOrigin {
                    file: std::path::PathBuf::from("slice/001"),
                    field: Some("related".to_string()),
                },
            },
        ];

        let graph = CatalogGraph {
            nodes,
            edges,
            units: Units {
                estimation: "hours".to_string(),
                value: "points".to_string(),
            },
        };

        let filtered = graph.exclude_memory();

        assert_eq!(filtered.nodes.len(), 2, "only SL and REQ survive");
        assert!(!filtered.contains(&mem_key));
        assert_eq!(
            filtered.edges.len(),
            1,
            "only SL→REQ edge survives; MEM-incident edges dropped"
        );
        assert_eq!(filtered.edges[0].source.canonical(), "SL-001");
    }

    // -----------------------------------------------------------------------
    // drop_isolated semantics
    // -----------------------------------------------------------------------

    #[test]
    fn drop_isolated_removes_only_edgeless_nodes() {
        let sl_key = CatalogKey::Numbered(EntityKey {
            prefix: "SL",
            id: 1,
        });
        let req_key = CatalogKey::Numbered(EntityKey {
            prefix: "REQ",
            id: 5,
        });
        let adr_key = CatalogKey::Numbered(EntityKey {
            prefix: "ADR",
            id: 2,
        });

        let mut nodes = BTreeMap::new();
        nodes.insert(
            sl_key.clone(),
            CatalogNode {
                title: "S1".to_string(),
                status: Some("proposed".to_string()),
                kind_label: "SL",
                memory_type: None,
            },
        );
        nodes.insert(
            req_key.clone(),
            CatalogNode {
                title: "R5".to_string(),
                status: Some("active".to_string()),
                kind_label: "REQ",
                memory_type: None,
            },
        );
        // ADR-002 has no edges → isolated
        nodes.insert(
            adr_key.clone(),
            CatalogNode {
                title: "A2".to_string(),
                status: Some("accepted".to_string()),
                kind_label: "ADR",
                memory_type: None,
            },
        );

        let edges = vec![CatalogEdge {
            source: sl_key.clone(),
            label: CatalogEdgeLabel::Validated(crate::relation::RelationLabel::References),
            role: Some(crate::relation::Role::Implements),
            descriptor: None,
            target: EdgeTarget::Resolved(req_key.clone()),
            origin: EdgeOrigin {
                file: std::path::PathBuf::from("slice/001"),
                field: Some("references".to_string()),
            },
        }];

        let graph = CatalogGraph {
            nodes,
            edges,
            units: Units {
                estimation: "hours".to_string(),
                value: "points".to_string(),
            },
        };

        let filtered = graph.drop_isolated();

        assert_eq!(filtered.nodes.len(), 2, "ADR-002 dropped (isolated)");
        assert!(filtered.contains(&sl_key));
        assert!(filtered.contains(&req_key));
        assert!(!filtered.contains(&adr_key));
        assert_eq!(filtered.edges.len(), 1, "edge survives");
    }

    #[test]
    fn drop_isolated_keeps_node_incident_via_incoming_only() {
        // A node referenced only as a Resolved target (no outbound edges)
        // must survive drop_isolated.
        let sl_key = CatalogKey::Numbered(EntityKey {
            prefix: "SL",
            id: 1,
        });
        let req_key = CatalogKey::Numbered(EntityKey {
            prefix: "REQ",
            id: 5,
        });

        let mut nodes = BTreeMap::new();
        nodes.insert(
            sl_key.clone(),
            CatalogNode {
                title: "S1".to_string(),
                status: Some("proposed".to_string()),
                kind_label: "SL",
                memory_type: None,
            },
        );
        nodes.insert(
            req_key.clone(),
            CatalogNode {
                title: "R5".to_string(),
                status: Some("active".to_string()),
                kind_label: "REQ",
                memory_type: None,
            },
        );

        // Only edge is SL→REQ; REQ has no outbound edges but IS an inbound
        // target — must survive.
        let edges = vec![CatalogEdge {
            source: sl_key.clone(),
            label: CatalogEdgeLabel::Validated(crate::relation::RelationLabel::References),
            role: Some(crate::relation::Role::Implements),
            descriptor: None,
            target: EdgeTarget::Resolved(req_key.clone()),
            origin: EdgeOrigin {
                file: std::path::PathBuf::from("slice/001"),
                field: Some("references".to_string()),
            },
        }];

        let graph = CatalogGraph {
            nodes,
            edges,
            units: Units {
                estimation: "hours".to_string(),
                value: "points".to_string(),
            },
        };

        let filtered = graph.drop_isolated();
        assert_eq!(filtered.nodes.len(), 2, "both nodes survive");
    }

    #[test]
    fn drop_isolated_duplicate_edges_both_survive() {
        // D13: duplicate edges must both survive drop_isolated.
        let sl_key = CatalogKey::Numbered(EntityKey {
            prefix: "SL",
            id: 1,
        });
        let req_key = CatalogKey::Numbered(EntityKey {
            prefix: "REQ",
            id: 5,
        });

        let mut nodes = BTreeMap::new();
        nodes.insert(
            sl_key.clone(),
            CatalogNode {
                title: "S1".to_string(),
                status: Some("proposed".to_string()),
                kind_label: "SL",
                memory_type: None,
            },
        );
        nodes.insert(
            req_key.clone(),
            CatalogNode {
                title: "R5".to_string(),
                status: Some("active".to_string()),
                kind_label: "REQ",
                memory_type: None,
            },
        );

        let edge_template = CatalogEdge {
            source: sl_key.clone(),
            label: CatalogEdgeLabel::Validated(crate::relation::RelationLabel::References),
            role: Some(crate::relation::Role::Implements),
            descriptor: None,
            target: EdgeTarget::Resolved(req_key.clone()),
            origin: EdgeOrigin {
                file: std::path::PathBuf::from("slice/001"),
                field: Some("references".to_string()),
            },
        };

        let graph = CatalogGraph {
            nodes,
            edges: vec![edge_template.clone(), edge_template],
            units: Units {
                estimation: "hours".to_string(),
                value: "points".to_string(),
            },
        };

        let filtered = graph.drop_isolated();
        assert_eq!(
            filtered.edges.len(),
            2,
            "duplicate edges both survive (D13)"
        );
    }
}
