// SPDX-License-Identifier: GPL-3.0-only
//! Pure DOT (Graphviz) emitter for a [`CatalogGraph`] (SL-226 PHASE-03).
//! Deterministic, styled, byte-identical on repeat calls.
//!
//! Port of `web/map/src/dot.ts::graphToDot` with the D10 split (rounded → style,
//! not shape), D14 (roled references label), D15 (tooltip), and R4 sort order.

use std::collections::BTreeSet;

use super::graph::{CatalogGraph, NodeKey};
use super::hydrate::{CatalogEdgeLabel, EdgeTarget};

// ---------------------------------------------------------------------------
// Escaping
// ---------------------------------------------------------------------------

/// Escape `"`, `\`, and newlines for DOT string literals.
/// Lifted from `src/concept_map.rs` (D8); single source going forward.
pub(crate) fn dot_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            _ => out.push(ch),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Style tables (STD-001)
// ---------------------------------------------------------------------------

struct NodeStyle {
    fill: &'static str,
    font: &'static str,
    rounded: bool,
}

/// Look up [`NodeStyle`] by kind prefix. Rounded=true only for SL, PRD, SPEC.
#[rustfmt::skip]
#[expect(clippy::match_same_arms, reason = "lookup table with same values for distinct keys")]
fn node_style(prefix: &str) -> NodeStyle {
    match prefix {
        "SL"   => NodeStyle { fill: "#4A90D9", font: "#ffffff", rounded: true  },
        "ADR"  => NodeStyle { fill: "#7B4FBF", font: "#ffffff", rounded: false },
        "POL"  => NodeStyle { fill: "#7B4FBF", font: "#ffffff", rounded: false },
        "STD"  => NodeStyle { fill: "#9B59B6", font: "#ffffff", rounded: false },
        "PRD"  => NodeStyle { fill: "#E67E22", font: "#222222", rounded: true  },
        "SPEC" => NodeStyle { fill: "#E67E22", font: "#222222", rounded: true  },
        "REQ"  => NodeStyle { fill: "#F39C12", font: "#222222", rounded: false },
        "ISS"  => NodeStyle { fill: "#C0392B", font: "#ffffff", rounded: false },
        "IMP"  => NodeStyle { fill: "#C0392B", font: "#ffffff", rounded: false },
        "CHR"  => NodeStyle { fill: "#C0392B", font: "#ffffff", rounded: false },
        "RSK"  => NodeStyle { fill: "#C0392B", font: "#ffffff", rounded: false },
        "IDE"  => NodeStyle { fill: "#27AE60", font: "#222222", rounded: false },
        "RV"   => NodeStyle { fill: "#1ABC9C", font: "#222222", rounded: false },
        "REC"  => NodeStyle { fill: "#95A5A6", font: "#222222", rounded: false },
        "RFC"  => NodeStyle { fill: "#7F8C8D", font: "#ffffff", rounded: false },
        "ASM"  => NodeStyle { fill: "#3498DB", font: "#ffffff", rounded: false },
        "DEC"  => NodeStyle { fill: "#3498DB", font: "#ffffff", rounded: false },
        "QUE"  => NodeStyle { fill: "#8E44AD", font: "#ffffff", rounded: false },
        "CON"  => NodeStyle { fill: "#8E44AD", font: "#ffffff", rounded: false },
        "REV"  => NodeStyle { fill: "#A04000", font: "#ffffff", rounded: false },
        "CM"   => NodeStyle { fill: "#16A085", font: "#ffffff", rounded: false },
        _      => DEFAULT_NODE_STYLE,
    }
}

const DEFAULT_NODE_STYLE: NodeStyle = NodeStyle {
    fill: "#95A5A6",
    font: "#222222",
    rounded: false,
};

struct EdgeColor {
    color: &'static str,
    fontcolor: &'static str,
}

/// Look up [`EdgeColor`] by label `name()` in lower-case.
#[rustfmt::skip]
#[expect(clippy::match_same_arms, reason = "lookup table with same colors for distinct labels")]
fn edge_color(label_lower: &str) -> EdgeColor {
    match label_lower {
        "specs"           => EdgeColor { color: "#4A90D9", fontcolor: "#2563eb" },
        "requirements"    => EdgeColor { color: "#4A90D9", fontcolor: "#2563eb" },
        "descends_from"   => EdgeColor { color: "#4A90D9", fontcolor: "#2563eb" },
        "parent"          => EdgeColor { color: "#4A90D9", fontcolor: "#2563eb" },
        "members"         => EdgeColor { color: "#4A90D9", fontcolor: "#2563eb" },
        "supersedes"      => EdgeColor { color: "#E67E22", fontcolor: "#c2410c" },
        "revises"         => EdgeColor { color: "#E67E22", fontcolor: "#c2410c" },
        "governed_by"     => EdgeColor { color: "#7B4FBF", fontcolor: "#6d28d9" },
        "related"         => EdgeColor { color: "#7B4FBF", fontcolor: "#6d28d9" },
        "decision_ref"    => EdgeColor { color: "#7B4FBF", fontcolor: "#6d28d9" },
        "consumes"        => EdgeColor { color: "#27AE60", fontcolor: "#166534" },
        "interactions"    => EdgeColor { color: "#27AE60", fontcolor: "#166534" },
        "contextualizes"  => EdgeColor { color: "#27AE60", fontcolor: "#166534" },
        "slices"
        | "owning_slice"  => EdgeColor { color: "#16A085", fontcolor: "#0f766e" },
        "reviews"         => EdgeColor { color: "#64748b", fontcolor: "#475569" },
        "drift"           => EdgeColor { color: "#C0392B", fontcolor: "#991b1b" },
        _                 => DEFAULT_EDGE_COLOR,
    }
}

const DEFAULT_EDGE_COLOR: EdgeColor = EdgeColor {
    color: "#888888",
    fontcolor: "#555555",
};

// ---------------------------------------------------------------------------
// render()
// ---------------------------------------------------------------------------

/// Render a projected [`CatalogGraph`] to deterministic styled Graphviz DOT.
///
/// `focus` highlights a single node (penwidth=3); others get penwidth=1.
/// Ghost nodes are emitted for unresolved/unvalidated edge targets.
pub(crate) fn render(graph: &CatalogGraph, focus: Option<&NodeKey>) -> String {
    // Shell
    let mut lines: Vec<String> = vec![
        "digraph G {".to_string(),
        "  rankdir=LR;".to_string(),
        "  bgcolor=\"transparent\";".to_string(),
        "  nodesep=0.45;".to_string(),
        "  ranksep=0.8;".to_string(),
        String::new(),
    ];

    // 1. Real nodes (BTreeMap → already key order)
    for (key, node) in &graph.nodes {
        let prefix = match key {
            super::hydrate::CatalogKey::Numbered(ek) => ek.prefix,
            super::hydrate::CatalogKey::Memory(_) => "MEM",
        };
        let style = node_style(prefix);

        let id = key.canonical();
        let label = if prefix == "MEM" {
            node.title.clone()
        } else {
            key.canonical()
        };

        let penwidth = if let Some(f) = focus
            && key == f
        {
            3
        } else {
            1
        };

        let style_str = if style.rounded {
            "filled,rounded"
        } else {
            "filled"
        };

        // Tooltip: "{id}: {title} · {kind} · {status}" — omit ` · {status}` when None (D15)
        let tooltip = build_tooltip(&id, &node.title, node.kind_label, node.status.as_deref());

        let escaped_id = dot_escape(&id);
        let escaped_label = dot_escape(&label);
        let escaped_tooltip = dot_escape(&tooltip);

        lines.push(format!(
            "  \"{escaped_id}\" [label=\"{escaped_label}\", style=\"{style_str}\", fillcolor=\"{fill}\", fontcolor=\"{font}\", shape=\"box\", penwidth={penwidth}, tooltip=\"{escaped_tooltip}\"];",
            fill = style.fill,
            font = style.font,
        ));
    }

    // 2. Ghost nodes (D5) — dedup distinct raws from UnresolvedRef / UnvalidatedText
    let mut ghost_raws: BTreeSet<&str> = BTreeSet::new();
    for edge in &graph.edges {
        match &edge.target {
            EdgeTarget::UnresolvedRef { raw } | EdgeTarget::UnvalidatedText { raw } => {
                ghost_raws.insert(raw.as_str());
            }
            EdgeTarget::Resolved(_) => {}
        }
    }
    for raw in &ghost_raws {
        let escaped = dot_escape(raw);
        lines.push(format!(
            "  \"?:{escaped}\" [label=\"{escaped}\", style=\"dashed\", color=\"#888888\", fontcolor=\"#888888\", shape=\"box\"];",
        ));
    }

    // 3. Edges — sort by R4 tuple
    let mut indexed_edges: Vec<(usize, &super::hydrate::CatalogEdge)> =
        graph.edges.iter().enumerate().collect();

    indexed_edges.sort_by(|(idx_a, a), (idx_b, b)| {
        let source_a = a.source.canonical();
        let source_b = b.source.canonical();

        let display_a = display_label(a);
        let display_b = display_label(b);

        let target_a = target_scalar(&a.target);
        let target_b = target_scalar(&b.target);

        source_a
            .cmp(&source_b)
            .then_with(|| display_a.cmp(&display_b))
            .then_with(|| target_a.cmp(&target_b))
            .then_with(|| idx_a.cmp(idx_b))
    });

    for (_idx, edge) in &indexed_edges {
        let source_canonical = edge.source.canonical();
        let target_scalar = target_scalar(&edge.target);
        let display_label = display_label(edge);

        let label_lower = edge.label.name().to_lowercase();
        let edge_col = edge_color(&label_lower);

        let esc_source = dot_escape(&source_canonical);
        let esc_target = dot_escape(&target_scalar);
        let esc_display = dot_escape(&display_label);

        lines.push(format!(
            "  \"{esc_source}\" -> \"{esc_target}\" [label=\"{esc_display}\", color=\"{color}\", fontcolor=\"{fontcolor}\"];",
            color = edge_col.color,
            fontcolor = edge_col.fontcolor,
        ));
    }

    // 4. Close
    lines.push("}".to_string());

    lines.join("\n") + "\n"
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Build the tooltip string: "{id}: {title} · {kind}" plus optional " · {status}".
fn build_tooltip(id: &str, title: &str, kind: &str, status: Option<&str>) -> String {
    let mut s = format!("{id}: {title} \u{00b7} {kind}");
    if let Some(st) = status {
        s.push_str(" \u{00b7} ");
        s.push_str(st);
    }
    s
}

/// The edge label to display: `references(role)` for roled references, else `label.name()`.
fn display_label(edge: &super::hydrate::CatalogEdge) -> String {
    if let CatalogEdgeLabel::Validated(crate::relation::RelationLabel::References) = &edge.label
        && let Some(role) = edge.role
    {
        format!("references({})", role.name())
    } else {
        edge.label.name().to_string()
    }
}

/// Render the target end of an edge as a scalar string for DOT.
fn target_scalar(target: &EdgeTarget) -> String {
    match target {
        EdgeTarget::Resolved(k) => k.canonical(),
        EdgeTarget::UnresolvedRef { raw } | EdgeTarget::UnvalidatedText { raw } => {
            format!("?:{raw}")
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
    use crate::catalog::graph::CatalogNode;
    use crate::catalog::hydrate::{CatalogEdge, EdgeOrigin};
    use crate::relation::Role;
    use std::collections::BTreeMap;

    /// A real node line starts with `  "prefix-nnn"`, does NOT contain `->`, and is not a ghost.
    fn is_real_node_line(line: &str) -> bool {
        let trimmed = line.trim_start();
        trimmed.starts_with('"')
            && !trimmed.starts_with("\"?:")
            && !trimmed.contains("->")
            && trimmed.contains("[label=")
    }

    /// Build a rich fixture covering: SL (rounded), ADR (not rounded), PRD (rounded),
    /// a ghost node, a focus node, a status:None node, roled references edges,
    /// and a non-references edge. This fixture is used by multiple VT-1 tests.
    fn rich_fixture() -> (CatalogGraph, NodeKey, String) {
        use crate::catalog::hydrate::CatalogKey;

        let sl_key = CatalogKey::Numbered(crate::catalog::scan::EntityKey {
            prefix: "SL",
            id: 1,
        });
        let adr_key = CatalogKey::Numbered(crate::catalog::scan::EntityKey {
            prefix: "ADR",
            id: 1,
        });
        let prd_key = CatalogKey::Numbered(crate::catalog::scan::EntityKey {
            prefix: "PRD",
            id: 1,
        });
        let req_key = CatalogKey::Numbered(crate::catalog::scan::EntityKey {
            prefix: "REQ",
            id: 1,
        });

        // focus_key is SL-001
        let focus_key = sl_key.clone();

        let mut nodes = BTreeMap::new();
        nodes.insert(
            sl_key.clone(),
            CatalogNode {
                title: "Fix the thing".to_string(),
                status: Some("proposed".to_string()),
                kind_label: "SL",
                memory_type: None,
            },
        );
        nodes.insert(
            adr_key.clone(),
            CatalogNode {
                title: "Use Rust".to_string(),
                status: Some("accepted".to_string()),
                kind_label: "ADR",
                memory_type: None,
            },
        );
        nodes.insert(
            prd_key.clone(),
            CatalogNode {
                title: "Widget".to_string(),
                status: None, // no status → tooltip omits trailing segment
                kind_label: "PRD",
                memory_type: None,
            },
        );
        nodes.insert(
            req_key.clone(),
            CatalogNode {
                title: "Fast startup".to_string(),
                status: Some("active".to_string()),
                kind_label: "REQ",
                memory_type: None,
            },
        );

        let edge_fixture = |source: &NodeKey,
                            label: crate::relation::RelationLabel,
                            role: Option<Role>,
                            target: EdgeTarget|
         -> CatalogEdge {
            CatalogEdge {
                source: source.clone(),
                label: CatalogEdgeLabel::Validated(label),
                role,
                descriptor: None,
                target,
                origin: EdgeOrigin {
                    file: std::path::PathBuf::from("fixture"),
                    field: None,
                },
            }
        };

        let edges = vec![
            // SL→PRD roled references (implements)
            edge_fixture(
                &sl_key,
                crate::relation::RelationLabel::References,
                Some(Role::Implements),
                EdgeTarget::Resolved(prd_key.clone()),
            ),
            // SL→UNKNOWN (ghost, UnresolvedRef)
            CatalogEdge {
                source: sl_key.clone(),
                label: CatalogEdgeLabel::Validated(crate::relation::RelationLabel::References),
                role: Some(Role::Concerns),
                descriptor: None,
                target: EdgeTarget::UnresolvedRef {
                    raw: "UNKNOWN".to_string(),
                },
                origin: EdgeOrigin {
                    file: std::path::PathBuf::from("fixture"),
                    field: None,
                },
            },
            // SL→REQ supersedes (label-only, no role)
            edge_fixture(
                &sl_key,
                crate::relation::RelationLabel::Supersedes,
                None,
                EdgeTarget::Resolved(req_key.clone()),
            ),
            // ADR→PRD references (no role)
            edge_fixture(
                &adr_key,
                crate::relation::RelationLabel::References,
                None,
                EdgeTarget::Resolved(prd_key.clone()),
            ),
        ];

        let ghost_raw = "UNKNOWN".to_string();

        let graph = CatalogGraph {
            nodes,
            edges,
            units: crate::catalog::hydrate::Units {
                estimation: "hours".to_string(),
                value: "points".to_string(),
            },
        };

        (graph, focus_key, ghost_raw)
    }

    // -----------------------------------------------------------------------
    // VT-1: rich fixture — NODE_STYLES, EDGE_COLORS, render
    // -----------------------------------------------------------------------

    #[test]
    fn render_node_styles_rounded_sl_and_not_adr_no_invalid_shape() {
        let (graph, _focus, _) = rich_fixture();
        let output = render(&graph, None);

        // SL node gets style="filled,rounded"
        assert!(
            output.contains("style=\"filled,rounded\""),
            "SL node must be filled,rounded: {output}"
        );
        // ADR node gets style="filled" (not rounded)
        // We can check that the ADR node line contains 'style="filled"' but NOT ',rounded'
        let adr_line = output
            .lines()
            .find(|l| l.contains("\"ADR-001\"") && is_real_node_line(l))
            .unwrap();
        assert!(
            adr_line.contains("style=\"filled\""),
            "ADR node should have style=filled: {adr_line}"
        );
        assert!(
            !adr_line.contains("style=\"filled,rounded\""),
            "ADR node must NOT be rounded: {adr_line}"
        );

        // The invalid shape="box,rounded" must NEVER appear
        assert!(
            !output.contains("shape=\"box,rounded\""),
            "shape=\"box,rounded\" is INVALID and must NOT appear"
        );

        // All real nodes have shape="box"
        for line in output.lines() {
            if is_real_node_line(line) {
                assert!(
                    line.contains("shape=\"box\""),
                    "real node must have shape=box: {line}"
                );
            }
        }
    }

    #[test]
    fn render_edge_colors_references_implements_colored_and_penwidth_focus() {
        let (graph, focus, _) = rich_fixture();
        let output = render(&graph, Some(&focus));

        // references(implements) edge displayed with roled label
        assert!(
            output.contains("references(implements)"),
            "should display roled references as references(implements)"
        );
        // supersedes edge (label-only, no role) gets EDGE_COLORS for "supersedes"
        let sup_line = output.lines().find(|l| l.contains("supersedes")).unwrap();
        assert!(
            sup_line.contains("color=\"#E67E22\""),
            "supersedes edge color: {sup_line}"
        );
        assert!(
            sup_line.contains("fontcolor=\"#c2410c\""),
            "supersedes fontcolor: {sup_line}"
        );

        // Focus node penwidth=3
        let sl_line = output
            .lines()
            .find(|l| l.contains("\"SL-001\"") && is_real_node_line(l))
            .unwrap();
        assert!(
            sl_line.contains("penwidth=3"),
            "focus node must have penwidth=3: {sl_line}"
        );

        // Non-focus nodes penwidth=1
        let adr_line = output
            .lines()
            .find(|l| l.contains("\"ADR-001\"") && is_real_node_line(l))
            .unwrap();
        assert!(
            adr_line.contains("penwidth=1"),
            "non-focus node must have penwidth=1: {adr_line}"
        );
    }

    #[test]
    fn render_ghost_node_dashed_and_tooltip_status_none_omitted() {
        let (graph, _, ghost_raw) = rich_fixture();
        let output = render(&graph, None);

        // Ghost node with style="dashed"
        let ghost_line = output
            .lines()
            .find(|l| l.contains(&format!("?:{}", ghost_raw)))
            .unwrap();
        assert!(
            ghost_line.contains("style=\"dashed\""),
            "ghost node must be dashed: {ghost_line}"
        );
        assert!(
            ghost_line.contains("color=\"#888888\""),
            "ghost color: {ghost_line}"
        );

        // PRD node has status:None → tooltip omits trailing ` · ` segment
        let prd_line = output
            .lines()
            .find(|l| l.contains("\"PRD-001\"") && is_real_node_line(l))
            .unwrap();
        // tooltip should contain "PRD-001: Widget · PRD" but NOT " · "
        // We can verify: the middle dot appears once (between title and kind), not twice
        assert!(
            prd_line.contains("\u{00b7} PRD\""),
            "tooltip should have '· PRD': {prd_line}"
        );
        // There should be no " · " after "PRD"
        let after_prd = prd_line.split("\u{00b7} PRD").nth(1).unwrap_or("");
        assert!(
            !after_prd.contains('\u{00b7}'),
            "no trailing status segment when status is None: {prd_line}"
        );
    }

    #[test]
    fn render_node_styles_fill_matches_constant_per_kind() {
        let (graph, _, _) = rich_fixture();
        let output = render(&graph, None);

        // SL → #4A90D9
        let sl_line = output
            .lines()
            .find(|l| l.contains("\"SL-001\"") && is_real_node_line(l))
            .unwrap();
        assert!(
            sl_line.contains("fillcolor=\"#4A90D9\""),
            "SL fill: {sl_line}"
        );
        assert!(
            sl_line.contains("fontcolor=\"#ffffff\""),
            "SL font: {sl_line}"
        );

        // ADR → #7B4FBF
        let adr_line = output
            .lines()
            .find(|l| l.contains("\"ADR-001\"") && is_real_node_line(l))
            .unwrap();
        assert!(
            adr_line.contains("fillcolor=\"#7B4FBF\""),
            "ADR fill: {adr_line}"
        );

        // PRD → #E67E22
        let prd_line = output
            .lines()
            .find(|l| l.contains("\"PRD-001\"") && is_real_node_line(l))
            .unwrap();
        assert!(
            prd_line.contains("fillcolor=\"#E67E22\""),
            "PRD fill: {prd_line}"
        );
    }

    #[test]
    fn render_all_nodes_have_shape_box() {
        let (graph, _, _) = rich_fixture();
        let output = render(&graph, None);

        // Every real node line must contain shape="box"
        for line in output.lines() {
            if is_real_node_line(line) {
                assert!(
                    line.contains("shape=\"box\""),
                    "every real node must have shape=box: {line}"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // VT-2: deterministic ghost — byte-identical on repeat
    // -----------------------------------------------------------------------

    #[test]
    fn deterministic_ghost_byte_identical_on_repeat() {
        // Build a fixture with colliding edge sort keys and multiple distinct ghosts.
        // Two edges with same source+label+target but different indices.
        let a = crate::catalog::hydrate::CatalogKey::Numbered(crate::catalog::scan::EntityKey {
            prefix: "SL",
            id: 1,
        });
        let b = crate::catalog::hydrate::CatalogKey::Numbered(crate::catalog::scan::EntityKey {
            prefix: "REQ",
            id: 2,
        });

        let mut nodes = BTreeMap::new();
        nodes.insert(
            a.clone(),
            CatalogNode {
                title: "A".to_string(),
                status: Some("proposed".to_string()),
                kind_label: "SL",
                memory_type: None,
            },
        );
        nodes.insert(
            b.clone(),
            CatalogNode {
                title: "B".to_string(),
                status: Some("active".to_string()),
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

        // Two identical edges (same source+label+target, distinct indices)
        // Plus two distinct ghost refs
        let edges = vec![
            edge_template.clone(),
            edge_template.clone(),
            CatalogEdge {
                source: a.clone(),
                label: CatalogEdgeLabel::Validated(crate::relation::RelationLabel::References),
                role: None,
                descriptor: None,
                target: EdgeTarget::UnresolvedRef {
                    raw: "GHOST_A".to_string(),
                },
                origin: EdgeOrigin {
                    file: std::path::PathBuf::from("fixture"),
                    field: None,
                },
            },
            CatalogEdge {
                source: a.clone(),
                label: CatalogEdgeLabel::Validated(crate::relation::RelationLabel::Supersedes),
                role: None,
                descriptor: None,
                target: EdgeTarget::UnvalidatedText {
                    raw: "GHOST_B".to_string(),
                },
                origin: EdgeOrigin {
                    file: std::path::PathBuf::from("fixture"),
                    field: None,
                },
            },
        ];

        let graph = CatalogGraph {
            nodes,
            edges,
            units: crate::catalog::hydrate::Units {
                estimation: "hours".to_string(),
                value: "points".to_string(),
            },
        };

        let output1 = render(&graph, None);
        let output2 = render(&graph, None);

        assert_eq!(output1, output2, "render must be byte-identical on repeat");
        assert!(!output1.is_empty());

        // Both ghosts present
        assert!(output1.contains("?:GHOST_A"));
        assert!(output1.contains("?:GHOST_B"));

        // Both ghost nodes are dashed
        let ghost_a_line = output1.lines().find(|l| l.contains("?:GHOST_A")).unwrap();
        assert!(ghost_a_line.contains("style=\"dashed\""));
        let ghost_b_line = output1.lines().find(|l| l.contains("?:GHOST_B")).unwrap();
        assert!(ghost_b_line.contains("style=\"dashed\""));
    }
}
