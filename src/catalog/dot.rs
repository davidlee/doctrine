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

/// Escape text for an HTML-like DOT label (`label=<…>`), which is parsed as
/// XML: only `&`, `<`, and `>` are structural.
fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Render wrapped text (real newlines) as HTML-like label lines.
fn html_lines(wrapped: &str) -> String {
    wrapped
        .lines()
        .map(html_escape)
        .collect::<Vec<_>>()
        .join("<BR/>")
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
// Label wrapping (SPIKE)
// ---------------------------------------------------------------------------

/// Target line width, in characters, for a wrapped node title.
const LABEL_WRAP_COLS: usize = 22;

/// Font family for node and edge labels. Monospace keeps `LABEL_WRAP_COLS`
/// honest — a wrapped line's character count is its real width.
const LABEL_FONT: &str = "monospace";

/// Greedily pack `chunks` into lines of at most `cols` characters, each line
/// joined by `glue`, the lines joined by real newlines — `dot_escape` turns
/// those into the DOT centred-line escape. A chunk wider than `cols` takes a
/// line of its own rather than being split.
fn pack_lines<'a>(chunks: impl Iterator<Item = &'a str>, glue: &str, cols: usize) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for chunk in chunks {
        if current.is_empty() {
            current.push_str(chunk);
        } else if current.chars().count() + glue.len() + chunk.chars().count() <= cols {
            current.push_str(glue);
            current.push_str(chunk);
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(chunk);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines.join("\n")
}

/// Word-wrap a title: chunks are whitespace-separated words, rejoined with a
/// space.
fn wrap_title(text: &str, cols: usize) -> String {
    pack_lines(text.split_whitespace(), " ", cols)
}

/// Wrap a memory key (`mem.pattern.dispatch.spawn-backend-…`), which carries no
/// whitespace to break on: chunks end *after* a `.` or `-`, so a delimiter
/// stays with the line it terminates and no glue is needed.
fn wrap_key(key: &str, cols: usize) -> String {
    let mut chunks: Vec<&str> = Vec::new();
    let mut start = 0;
    for (idx, ch) in key.char_indices() {
        if ch == '.' || ch == '-' {
            chunks.push(&key[start..=idx]);
            start = idx + ch.len_utf8();
        }
    }
    if start < key.len() {
        chunks.push(&key[start..]);
    }
    pack_lines(chunks.into_iter(), "", cols)
}

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
        format!("  node [fontname=\"{LABEL_FONT}\"];"),
        format!("  edge [fontname=\"{LABEL_FONT}\"];"),
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
        // SPIKE: titles in labels (hardcoded on; no flag yet). Line 1 is the
        // citable handle — the canonical id, or for a memory its readable key
        // (the memory's canonical form is an opaque uid). Title wraps beneath.
        let handle = if prefix == "MEM" {
            node.memory_key
                .as_deref()
                .map(|k| wrap_key(k, LABEL_WRAP_COLS))
        } else {
            Some(id.clone())
        };
        let wrapped_title = html_lines(&wrap_title(&node.title, LABEL_WRAP_COLS));
        // HTML-like label (`label=<…>`) — the only way to weight part of a
        // label. `record` shapes carry ports, not formatting.
        let label = match handle {
            Some(h) => format!("<B>{}</B><BR/>{wrapped_title}", html_lines(&h)),
            None => wrapped_title,
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
        let escaped_tooltip = dot_escape(&tooltip);

        lines.push(format!(
            "  \"{escaped_id}\" [label=<{label}>, style=\"{style_str}\", fillcolor=\"{fill}\", fontcolor=\"{font}\", shape=\"box\", penwidth={penwidth}, tooltip=\"{escaped_tooltip}\"];",
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

    /// Wrap a node map in a `CatalogGraph` with no edges and the standard unit
    /// labels — the shape most label tests need.
    fn graph_of(nodes: BTreeMap<NodeKey, CatalogNode>) -> CatalogGraph {
        CatalogGraph {
            nodes,
            edges: Vec::new(),
            units: crate::catalog::hydrate::Units {
                estimation: "hours".to_string(),
                value: "points".to_string(),
            },
        }
    }

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
                memory_key: None,
            },
        );
        nodes.insert(
            adr_key.clone(),
            CatalogNode {
                title: "Use Rust".to_string(),
                status: Some("accepted".to_string()),
                kind_label: "ADR",
                memory_type: None,
                memory_key: None,
            },
        );
        nodes.insert(
            prd_key.clone(),
            CatalogNode {
                title: "Widget".to_string(),
                status: None, // no status → tooltip omits trailing segment
                kind_label: "PRD",
                memory_type: None,
                memory_key: None,
            },
        );
        nodes.insert(
            req_key.clone(),
            CatalogNode {
                title: "Fast startup".to_string(),
                status: Some("active".to_string()),
                kind_label: "REQ",
                memory_type: None,
                memory_key: None,
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
                memory_key: None,
            },
        );
        nodes.insert(
            b.clone(),
            CatalogNode {
                title: "B".to_string(),
                status: Some("active".to_string()),
                kind_label: "REQ",
                memory_type: None,
                memory_key: None,
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
    // ── labels: handle over wrapped title (IMP-454) ─────────────────────────

    #[test]
    /// Titles wrap greedily on whitespace, packing each line up to `cols`.
    fn wrap_title_packs_words_up_to_the_column_budget() {
        assert_eq!(
            wrap_title("Render probes kitty graphics support before rendering", 22),
            "Render probes kitty\ngraphics support\nbefore rendering"
        );
    }

    #[test]
    /// A word wider than the budget takes its own line rather than being split.
    fn wrap_title_never_splits_a_word() {
        assert_eq!(
            wrap_title("a supercalifragilistic b", 8),
            "a\nsupercalifragilistic\nb"
        );
    }

    #[test]
    /// Memory keys carry no whitespace, so they break after `.`/`-`, with the
    /// delimiter staying on the line it terminates.
    fn wrap_key_breaks_after_delimiters() {
        assert_eq!(
            wrap_key("mem.pattern.dispatch.spawn-backend-harness", 22),
            "mem.pattern.dispatch.\nspawn-backend-harness"
        );
    }

    #[test]
    /// HTML-like labels are parsed as XML: `&`, `<`, `>` must be entities.
    fn html_escape_covers_structural_characters() {
        assert_eq!(html_escape("a & b <c> d"), "a &amp; b &lt;c&gt; d");
    }

    #[test]
    /// A node label is its bolded handle, then its wrapped title, in an
    /// HTML-like label — the only DOT form that can weight part of a label.
    fn node_label_bolds_the_handle_above_the_title() {
        let (graph, focus_key, _) = rich_fixture();
        let output = render(&graph, Some(&focus_key));

        let sl_line = output
            .lines()
            .find(|l| l.trim_start().starts_with("\"SL-001\""))
            .expect("SL-001 node line");
        assert!(
            sl_line.contains("label=<<B>SL-001</B><BR/>Fix the thing>"),
            "expected bolded handle over title, got: {sl_line}"
        );
    }

    #[test]
    /// A memory node's handle is its readable key — its canonical form is an
    /// opaque uid — and the key wraps like any other handle.
    fn memory_node_handle_is_its_readable_key() {
        use crate::catalog::hydrate::CatalogKey;

        let uid = "mem_019ebeeda9c27f03808fdeeafb0e93cc";
        let key = CatalogKey::Memory(uid.to_string());
        let mut nodes = BTreeMap::new();
        nodes.insert(
            key,
            CatalogNode {
                title: "Spawn backends stay harness-agnostic".to_string(),
                status: Some("active".to_string()),
                kind_label: "MEM",
                memory_type: Some("pattern".to_string()),
                memory_key: Some("mem.pattern.dispatch.spawn-backend".to_string()),
            },
        );
        let output = render(&graph_of(nodes), None);
        let line = output
            .lines()
            .find(|l| l.contains(uid))
            .expect("memory node line");

        assert!(
            line.contains("<B>mem.pattern.dispatch.<BR/>spawn-backend</B>"),
            "expected wrapped key as a bolded handle, got: {line}"
        );
        assert!(
            !line.contains(&format!("<B>{uid}")),
            "the opaque uid must not be the handle: {line}"
        );
    }

    #[test]
    /// A memory with no authored key has no citable handle — the label is just
    /// its wrapped title, unbolded.
    fn memory_node_without_a_key_falls_back_to_its_title() {
        use crate::catalog::hydrate::CatalogKey;

        let uid = "mem_019ebeeda9c27f03808fdeeafb0e93cc";
        let mut nodes = BTreeMap::new();
        nodes.insert(
            CatalogKey::Memory(uid.to_string()),
            CatalogNode {
                title: "Unkeyed memory".to_string(),
                status: Some("active".to_string()),
                kind_label: "MEM",
                memory_type: Some("fact".to_string()),
                memory_key: None,
            },
        );
        let line = render(&graph_of(nodes), None)
            .lines()
            .find(|l| l.contains(uid))
            .expect("memory node line")
            .to_string();

        assert!(line.contains("label=<Unkeyed memory>"), "got: {line}");
    }
}
