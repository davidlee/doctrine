// SPDX-License-Identifier: GPL-3.0-only
//! Presentation-neutral graph projection of `Catalog` (SL-071 PHASE-04).
//! Pure — no cordage dependency, no disk reads. Edges with unresolved or
//! unvalidated targets appear in the edge list but have no target node.
//! `neighbours(depth)` is deferred per design D10.

use std::collections::BTreeMap;

use super::hydrate::{Catalog, CatalogEdge, CatalogKey, EdgeTarget, Units};
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
    /// The entity's optional `[estimate]` facet, projected from the source
    /// [`CatalogEntity`] (SL-103 PHASE-03, design §4.3). Absent ⇒ omitted from
    /// the serialized contract entirely (`skip_serializing_if`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) estimate: Option<crate::estimate::EstimateFacet>,
    /// The entity's optional `[value]` facet, projected from the source
    /// [`CatalogEntity`] (SL-103 PHASE-03, design §4.3). Absent ⇒ omitted from
    /// the serialized contract entirely (`skip_serializing_if`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) value: Option<crate::value::ValueFacet>,
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
                    estimate: entity.estimate.clone(),
                    value: entity.value.clone(),
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

    /// Seed an ADR with `facets` appended verbatim. The ADR scan path never
    /// type-checks `[estimate]`/`[value]`, so a malformed (or kind-agnostic
    /// present) facet survives to `read_facets` — the per-facet isolation seam.
    fn seed_adr_with_facets(root: &Path, id: u32, facets: &str) {
        write(
            root,
            &format!(".doctrine/adr/{id:03}/adr-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"a{id}\"\ntitle = \"A{id}\"\nstatus = \"accepted\"\n\
                 created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n{facets}"
            ),
        );
        write(
            root,
            &format!(".doctrine/adr/{id:03}/adr-{id:03}.md"),
            "body\n",
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

    /// VT-1: a faceted slice projects `estimate{lower,upper}` + `value{value}`
    /// onto its graph node, and the project-wide `units` resolve onto the graph.
    #[test]
    fn faceted_slice_projects_estimate_value_and_units() {
        let dir = tmp();
        let root = dir.path();
        seed_slice_with_facets(
            root,
            1,
            "[estimate]\nlower = 2\nupper = 8\n\n[value]\nvalue = 5\n",
        );

        let graph = build_graph(root);

        let node = node_for(&graph, "SL-001");
        assert_eq!(
            node.estimate,
            Some(crate::estimate::EstimateFacet {
                lower: 2.0,
                upper: 8.0
            })
        );
        assert_eq!(node.value, Some(crate::value::ValueFacet { value: 5.0 }));

        // Units resolve from the (absent) doctrine.toml to the sub-config defaults.
        assert_eq!(graph.units.estimation, "espresso_shots");
        assert_eq!(graph.units.value, "magic_beans");
    }

    /// VT-2: a non-faceted entity still gets a node, but `estimate`/`value` are
    /// `None` and are OMITTED entirely from the serialized contract.
    #[test]
    fn non_faceted_entity_omits_facet_keys() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, &[]);

        let graph = build_graph(root);

        let node = node_for(&graph, "SL-001");
        assert!(node.estimate.is_none());
        assert!(node.value.is_none());

        let json = serde_json::to_value(node).unwrap();
        assert!(
            json.get("estimate").is_none(),
            "absent estimate must be omitted from the contract"
        );
        assert!(
            json.get("value").is_none(),
            "absent value must be omitted from the contract"
        );
    }

    /// VT-3: end-to-end per-facet isolation — a malformed `[estimate]` next to a
    /// valid `[value]` on an ADR: the node is carried with `estimate: None`, an
    /// `Error` diagnostic is present, and the sibling `value` survives on the
    /// node. (ADR, not slice — a slice rejects a malformed `[estimate]` in its
    /// own typed read before the kind-agnostic scan, per the PHASE-01 finding.)
    #[test]
    fn malformed_estimate_isolated_from_valid_value_on_graph_node() {
        let dir = tmp();
        let root = dir.path();
        seed_adr_with_facets(
            root,
            1,
            "[estimate]\nlower = 5\nupper = 2\n\n[value]\nvalue = 7\n",
        );

        let catalog =
            crate::catalog::hydrate::scan_catalog(root).expect("scan_catalog should succeed");
        let graph = CatalogGraph::from_catalog(&catalog);

        let node = node_for(&graph, "ADR-001");
        assert!(node.estimate.is_none(), "malformed estimate drops to None");
        assert_eq!(
            node.value,
            Some(crate::value::ValueFacet { value: 7.0 }),
            "sibling value facet stays on the node"
        );

        let errors: Vec<_> = catalog
            .diagnostics
            .iter()
            .filter(|d| d.severity == super::super::diagnostic::Severity::Error)
            .collect();
        assert_eq!(errors.len(), 1, "expected one Error diagnostic");
        assert_eq!(errors[0].field.as_deref(), Some("estimate"));
    }

    /// VT-4: round-trip durability — the normalized bounds are byte-identical
    /// from scan → catalog → graph (integer TOML bounds normalize to the same
    /// f64 on every hop).
    #[test]
    fn facet_bounds_durable_scan_to_catalog_to_graph() {
        let dir = tmp();
        let root = dir.path();
        seed_slice_with_facets(
            root,
            1,
            "[estimate]\nlower = 3\nupper = 11\n\n[value]\nvalue = 4\n",
        );

        let catalog =
            crate::catalog::hydrate::scan_catalog(root).expect("scan_catalog should succeed");
        let entity = catalog
            .entities
            .iter()
            .find(|e| e.key.canonical() == "SL-001")
            .unwrap();
        let graph = CatalogGraph::from_catalog(&catalog);
        let node = node_for(&graph, "SL-001");

        assert_eq!(entity.estimate, node.estimate);
        assert_eq!(entity.value, node.value);
        assert_eq!(
            node.estimate,
            Some(crate::estimate::EstimateFacet {
                lower: 3.0,
                upper: 11.0
            })
        );
    }

    /// VT-5: kind-agnostic — an `[estimate]` authored on a NON-slice TOML (ADR)
    /// surfaces on its graph node.
    #[test]
    fn estimate_on_non_slice_kind_surfaces_on_graph_node() {
        let dir = tmp();
        let root = dir.path();
        seed_adr_with_facets(root, 1, "[estimate]\nlower = 1\nupper = 6\n");

        let graph = build_graph(root);
        let node = node_for(&graph, "ADR-001");
        assert_eq!(
            node.estimate,
            Some(crate::estimate::EstimateFacet {
                lower: 1.0,
                upper: 6.0
            })
        );
        assert!(node.value.is_none());
    }

    /// VT-6: contract JSON shape — serializing the graph emits a top-level
    /// `units` (with graph-neutral `estimation`/`value` keys), `nodes` with
    /// per-node `estimate{lower,upper}`/`value{value}`, and `edges`.
    #[test]
    fn graph_contract_json_shape_is_graph_neutral() {
        let dir = tmp();
        let root = dir.path();
        seed_slice_with_facets(
            root,
            1,
            "[estimate]\nlower = 2\nupper = 8\n\n[value]\nvalue = 5\n",
        );

        let graph = build_graph(root);
        let json = serde_json::to_value(&graph).unwrap();

        // Top-level contract keys.
        assert!(json.get("nodes").is_some(), "missing nodes");
        assert!(json.get("edges").is_some(), "missing edges");
        let units = json.get("units").expect("missing units");
        assert_eq!(units["estimation"], "espresso_shots");
        assert_eq!(units["value"], "magic_beans");

        // Per-node facet shape with graph-neutral field names.
        let node = &json["nodes"]["SL-001"];
        assert_eq!(node["estimate"]["lower"], 2.0);
        assert_eq!(node["estimate"]["upper"], 8.0);
        assert_eq!(node["value"]["value"], 5.0);
    }
}
