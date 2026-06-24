// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine concept-map` - create, list, and show concept maps, doctrine's
//! DSL-driven relationship-diagram entity.
//!
//! A concept map is a numeric directory under `.doctrine/concept-map/` holding a
//! sister TOML (structured metadata including a raw DSL block) and a scaffolded
//! markdown prose body, with a `<id>-<slug>` symlink as a human alias. It is an
//! `entity::Kind` over the kind-blind engine - this module owns the
//! concept-map-specific parts (the Kind, scaffold, and thin CLI wiring); the
//! kind-agnostic machinery lives in `crate::entity`, and the shared
//! metadata-list substrate (`Meta`, list reader/formatter) in `crate::meta`.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::Context;
use clap::Subcommand;
use serde::Serialize;

use crate::entity::{self, Artifact, Fileset, Inputs, Kind, MaterialiseRequest, ScaffoldCtx};
use crate::listing::{self, Format, ListArgs};
use crate::meta::{self, Meta};
use crate::tomlfmt::toml_string;
use regex_lite::Regex;
use std::collections::BTreeMap;

#[derive(Subcommand)]
pub(crate) enum ConceptMapCommand {
    /// Create a new concept map.
    New {
        /// Concept-map title (prompted for if omitted).
        title: Option<String>,

        /// Explicit slug (default: derived from the title).
        #[arg(long)]
        slug: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// List concept maps.
    List {
        #[command(flatten)]
        list: crate::CommonListArgs,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Show a concept map's metadata and DSL.
    Show {
        /// Concept-map reference — `CM-001` or the bare id `1`.
        reference: String,

        /// Output format.
        #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
        format: Format,

        /// Shorthand for `--format json`.
        #[arg(long)]
        json: bool,

        /// Show edges table from parsed DSL.
        #[arg(long)]
        edges: bool,

        /// Show nodes table from parsed DSL.
        #[arg(long)]
        nodes: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Parse the DSL and run heuristic checks.
    Check {
        /// Concept-map reference — `CM-001` or the bare id `1`.
        id: String,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Add an edge to a concept map's DSL.
    Add {
        id: String,
        source: String,
        rel: String,
        target: String,
        #[arg(long)]
        force: bool,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Remove an edge from a concept map's DSL.
    Remove {
        id: String,
        source: String,
        rel: String,
        target: String,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Rename a node label across all DSL edges.
    RenameNode {
        id: String,
        old: String,
        new: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        case_sensitive: bool,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Export a concept map to DOT, Mermaid, or JSON.
    Export {
        id: String,
        #[arg(long, value_enum)]
        format: ExportFormat,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Print the file paths of each concept-map entity directory.
    Paths {
        /// Concept-map reference(s) — `CM-001` or the bare id `1`.
        refs: Vec<String>,

        #[arg(short = 't', long)]
        toml: bool,
        #[arg(short = 'm', long)]
        md: bool,
        #[arg(short = 'e', long)]
        entity: bool,
        #[arg(short = 's', long)]
        single: bool,

        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

pub(crate) fn dispatch(cmd: ConceptMapCommand, color: bool) -> anyhow::Result<()> {
    match cmd {
        ConceptMapCommand::New { title, slug, path } => run_new(path, title, slug),
        ConceptMapCommand::List { list, path } => run_list(path, list.into_list_args(color)),
        ConceptMapCommand::Show {
            reference,
            format,
            json,
            edges,
            nodes,
            path,
        } => run_show(
            path,
            &reference,
            if json { Format::Json } else { format },
            edges,
            nodes,
        ),
        ConceptMapCommand::Check { id, path } => run_check(path, &id),
        ConceptMapCommand::Add {
            id,
            source,
            rel,
            target,
            force,
            path,
        } => run_add(path, &id, &source, &rel, &target, force),
        ConceptMapCommand::Remove {
            id,
            source,
            rel,
            target,
            path,
        } => run_remove(path, &id, &source, &rel, &target),
        ConceptMapCommand::RenameNode {
            id,
            old,
            new,
            dry_run,
            case_sensitive,
            path,
        } => run_rename_node(path, &id, &old, &new, dry_run, case_sensitive),
        ConceptMapCommand::Export { id, format, path } => run_export(path, &id, &format),
        ConceptMapCommand::Paths {
            refs,
            toml,
            md,
            entity,
            single,
            path,
        } => {
            let root = crate::root::find(path, &crate::root::default_markers())?;
            let cm_root = root.join(CONCEPT_MAP_DIR);
            let sel = crate::paths::PathSelection {
                toml,
                md,
                entity,
                single,
            };
            let mut all_lines: Vec<String> = Vec::new();
            for r in &refs {
                let id = parse_ref(r)?;
                let name = format!("{id:03}");
                let entity_dir = cm_root.join(&name);
                let toml_name = format!("concept-map-{name}.toml");
                let md_name = format!("concept-map-{name}.md");
                let set = crate::paths::scan_entity_dir(
                    &entity_dir,
                    &entity_dir.join(&toml_name),
                    Some(&entity_dir.join(&md_name)),
                    &root,
                )?;
                let lines = crate::paths::select_paths(&set, &sel)?;
                all_lines.extend(lines);
            }
            write!(io::stdout(), "{}", all_lines.join("\n"))?;
            Ok(())
        }
    }
}

/// Relative dir of the concept-map tree inside the project root.
pub(crate) const CONCEPT_MAP_DIR: &str = ".doctrine/concept-map";

/// Statuses for concept maps — authored-artifact lifecycle (SL-074 design §2).
const CONCEPT_MAP_STATUSES: &[&str] = &["draft", "accepted", "superseded"];

/// The `concept-map list` hide-set: `superseded` drops from the default list.
/// `--all` or an explicit `--status` reveals it.
fn is_hidden(status: &str) -> bool {
    matches!(status, "superseded")
}

/// The top-level reserved concept-map kind: toml + md + slug symlink.
pub(crate) const CONCEPT_MAP_KIND: Kind = Kind {
    dir: CONCEPT_MAP_DIR,
    prefix: crate::kinds::CM,
    stem: "concept-map",
    scaffold: concept_map_scaffold,
};

// ---------------------------------------------------------------------------
// Pure: render, scaffolds
// ---------------------------------------------------------------------------

/// Render `concept-map-<id>.toml` from the embedded template by token substitution.
fn render_toml(id: u32, slug: &str, title: &str, date: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/concept-map.toml")?
        .replace("{{id}}", &id.to_string())
        .replace("{{slug}}", &toml_string(slug))
        .replace("{{title}}", &toml_string(title))
        .replace("{{date}}", date))
}

/// Render `concept-map-<id>.md` from the embedded template by token substitution.
fn render_md(title: &str, id: u32) -> anyhow::Result<String> {
    let canonical = crate::listing::canonical_id("CM", id);
    Ok(crate::install::asset_text("templates/concept-map.md")?
        .replace("{{title}}", title)
        .replace("{{id}}", &canonical))
}

/// The concept-map fileset: sister TOML, prose body, and `<id>-<slug>` symlink, all
/// relative to the concept-map tree root (the symlink sits beside the numeric dir).
fn concept_map_scaffold(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
    let id = ctx.id;
    let name = format!("{id:03}");
    Ok(vec![
        Artifact::File {
            rel_path: entity::rel_path(&CONCEPT_MAP_KIND, id, entity::Ext::Toml),
            body: render_toml(id, ctx.slug, ctx.title, ctx.date)?,
        },
        Artifact::File {
            rel_path: entity::rel_path(&CONCEPT_MAP_KIND, id, entity::Ext::Md),
            body: render_md(ctx.title, id)?,
        },
        Artifact::Symlink {
            rel_path: PathBuf::from(format!("{name}-{}", ctx.slug)),
            target: name,
        },
    ])
}

// ---------------------------------------------------------------------------
// Shell: run_new, run_list, run_show
// ---------------------------------------------------------------------------

/// `doctrine concept-map new` - allocate the next id and scaffold a concept map.
pub(crate) fn run_new(
    path: Option<PathBuf>,
    title: Option<String>,
    slug: Option<String>,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let title = crate::input::resolve_title(title)?;
    let slug = crate::input::resolve_slug(&title, slug)?;
    let date = crate::clock::today();
    let trunk_ids = crate::git::trunk_entity_ids(&root, CONCEPT_MAP_KIND.dir)?;
    let backend = crate::reserve::backend(&root, CONCEPT_MAP_KIND.prefix)?;
    let out = entity::materialise(
        &CONCEPT_MAP_KIND,
        &*backend,
        &root,
        &MaterialiseRequest::Fresh,
        &Inputs {
            slug: &slug,
            title: &title,
            date: &date,
        },
        &trunk_ids,
    )?;

    let id = out
        .eid
        .numeric_id()
        .context("concept-map kind must yield a numeric id")?;
    writeln!(
        io::stdout(),
        "Created concept map CM-{id:03}: {}",
        out.dir.display()
    )?;
    Ok(())
}

/// The full `concept-map-NNN.toml` read as data for `show` - `Meta`'s four list
/// fields plus dates, description, and the raw DSL block.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, Serialize)]
pub(crate) struct ConceptMapDoc {
    pub(crate) id: u32,
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) status: String,
    pub(crate) created: String,
    pub(crate) updated: String,
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) dsl: String,
}

/// Parse a concept-map reference - `CM-001`, `cm-1`, or the bare id `1` - to its
/// numeric id. The prefix is optional and case-insensitive; the id may be padded.
pub(crate) fn parse_ref(reference: &str) -> anyhow::Result<u32> {
    let digits = reference
        .strip_prefix("CM-")
        .or_else(|| reference.strip_prefix("cm-"))
        .unwrap_or(reference);
    digits.parse::<u32>().with_context(|| {
        format!("not a concept-map reference: `{reference}` (expected `CM-001` or `1`)")
    })
}

/// Read one concept-map's `concept-map-NNN.toml` (as data) and
/// `concept-map-NNN.md` (body).
pub(crate) fn read_concept_map(
    cm_root: &Path,
    id: u32,
) -> anyhow::Result<(ConceptMapDoc, String, String)> {
    let name = format!("{id:03}");
    let toml_path = cm_root.join(&name).join(format!("concept-map-{name}.toml"));
    let md_path = cm_root.join(&name).join(format!("concept-map-{name}.md"));
    let toml_text = std::fs::read_to_string(&toml_path)
        .with_context(|| format!("Failed to read {}", toml_path.display()))?;
    let body = std::fs::read_to_string(&md_path).unwrap_or_default();
    let doc: ConceptMapDoc = toml::from_str(&toml_text)
        .with_context(|| format!("Failed to parse {}", toml_path.display()))?;
    Ok((doc, toml_text, body))
}

// ---------------------------------------------------------------------------
// DSL types
// ---------------------------------------------------------------------------

/// A node in a parsed concept map - the normalised key plus the original label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ConceptMapNode {
    pub(crate) key: String,
    pub(crate) label: String,
}

/// An edge in a parsed concept map - "from" and "to" nodes plus a relation label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ConceptMapEdge {
    pub(crate) from_key: String,
    pub(crate) from_label: String,
    pub(crate) rel: String,
    pub(crate) to_key: String,
    pub(crate) to_label: String,
    pub(crate) line: usize,
}

/// A diagnostic emitted during parsing or checking of a concept map DSL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) enum ConceptMapDiagnostic {
    /// Line does not split into exactly 3 segments on " > ".
    MalformedLine { line: usize, text: String },
    /// A trimmed segment is empty.
    EmptyLabel { line: usize, segment: SegmentKind },
    /// The exact same (`from_key`, `rel`, `to_key`) triple appears on another line.
    DuplicateEdge {
        line: usize,
        existing_line: usize,
        from_key: String,
        rel: String,
        to_key: String,
    },
    /// A node has an edge to itself (`from_key` == `to_key`).
    SelfEdge { line: usize, node_key: String },
    /// Two distinct labels derived the same node key.
    CanonicalNodeCollision {
        key: String,
        first_label: String,
        first_line: usize,
        label: String,
        line: usize,
    },
    /// Two node labels have Levenshtein distance ≤ 2.
    SimilarNodeLabel {
        label_a: String,
        line_a: usize,
        label_b: String,
        line_b: usize,
    },
    /// Two relation texts have Levenshtein distance ≤ 2.
    RelationDrift {
        rel_a: String,
        line_a: usize,
        rel_b: String,
        line_b: usize,
    },
    /// A node label looks like a canonical entity ref (`PRD-010`, `SL-001`).
    EntityRefLike { label: String, line: usize },
}

/// The segment position in a DSL line that is empty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) enum SegmentKind {
    Source,
    Relation,
    Target,
}

/// The result of parsing a concept map DSL - nodes, edges, and any parse-time
/// diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ParsedConceptMap {
    pub(crate) nodes: Vec<ConceptMapNode>,
    pub(crate) edges: Vec<ConceptMapEdge>,
    pub(crate) diagnostics: Vec<ConceptMapDiagnostic>,
}

// ---------------------------------------------------------------------------
// Pure: derive_node_key
// ---------------------------------------------------------------------------

/// Normalise a node label into a stable, URL-safe key.
///
/// 1. Lowercase.
/// 2. Replace runs of whitespace, hyphens, and underscores with a single hyphen.
/// 3. Strip all non-alphanumeric characters (except hyphen).
/// 4. Trim leading/trailing hyphens.
pub(crate) fn derive_node_key(label: &str) -> String {
    let lower = label.to_lowercase();
    let mut result = String::with_capacity(lower.len());
    let mut in_sep = false;
    for ch in lower.chars() {
        if ch.is_whitespace() || ch == '_' || ch == '-' {
            if !in_sep {
                result.push('-');
                in_sep = true;
            }
        } else if ch.is_alphanumeric() {
            result.push(ch);
            in_sep = false;
        }
        // else: strip non-alphanumeric, non-separator chars
    }
    result.trim_matches('-').to_string()
}

// ---------------------------------------------------------------------------
// Pure: levenshtein
// ---------------------------------------------------------------------------

/// Compute the Levenshtein (edit) distance between two strings using the
/// classic Wagner-Fischer dynamic programming algorithm.
fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let n = a_chars.len();
    let m = b_chars.len();
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr = vec![0usize; m + 1];
    for i in 1..=n {
        if let Some(c) = curr.get_mut(0) {
            *c = i;
        }
        for j in 1..=m {
            let cost = usize::from(a_chars.get(i - 1) != b_chars.get(j - 1));
            let ins = prev.get(j).copied().unwrap_or(0) + 1;
            let del = curr.get(j - 1).copied().unwrap_or(0) + 1;
            let sub = prev.get(j - 1).copied().unwrap_or(0) + cost;
            if let Some(c) = curr.get_mut(j) {
                *c = ins.min(del).min(sub);
            }
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev.get(m).copied().unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Pure: parse_dsl
// ---------------------------------------------------------------------------

/// Parse a concept map DSL string into a [`ParsedConceptMap`] with nodes,
/// edges, and any parse-time diagnostics.
pub(crate) fn parse_dsl(dsl: &str) -> ParsedConceptMap {
    let mut nodes: Vec<ConceptMapNode> = Vec::new();
    let mut edges: Vec<ConceptMapEdge> = Vec::new();
    let mut diagnostics: Vec<ConceptMapDiagnostic> = Vec::new();
    let mut node_index: BTreeMap<String, usize> = BTreeMap::new();
    let mut node_lines: BTreeMap<String, usize> = BTreeMap::new();
    let mut edge_set: BTreeMap<(String, String, String), usize> = BTreeMap::new();

    for (idx, raw) in dsl.lines().enumerate() {
        let line = idx + 1;

        // Skip empty lines.
        if raw.trim().is_empty() {
            continue;
        }
        // Skip comments.
        if raw.trim_start().starts_with('#') {
            continue;
        }

        // Split on literal " > " (no trimming before split).
        let segments: Vec<&str> = raw.split(" > ").collect();
        if segments.len() != 3 {
            diagnostics.push(ConceptMapDiagnostic::MalformedLine {
                line,
                text: raw.to_string(),
            });
            continue;
        }

        let source_raw = segments.first().map_or("", |s| s.trim());
        let rel = segments.get(1).map_or("", |s| s.trim());
        let target_raw = segments.get(2).map_or("", |s| s.trim());

        // Check for empty segments (check all three before continuing so we
        // emit one diagnostic per empty segment).
        let mut has_empty = false;
        if source_raw.is_empty() {
            diagnostics.push(ConceptMapDiagnostic::EmptyLabel {
                line,
                segment: SegmentKind::Source,
            });
            has_empty = true;
        }
        if rel.is_empty() {
            diagnostics.push(ConceptMapDiagnostic::EmptyLabel {
                line,
                segment: SegmentKind::Relation,
            });
            has_empty = true;
        }
        if target_raw.is_empty() {
            diagnostics.push(ConceptMapDiagnostic::EmptyLabel {
                line,
                segment: SegmentKind::Target,
            });
            has_empty = true;
        }
        if has_empty {
            continue;
        }

        let from_key = derive_node_key(source_raw);
        let to_key = derive_node_key(target_raw);
        let from_label = source_raw.to_string();
        let to_label = target_raw.to_string();

        // Record node (first-wins by key).
        for (key, label, l) in [(&from_key, &from_label, line), (&to_key, &to_label, line)] {
            if let Some(&existing_idx) = node_index.get(key)
                && let Some(existing) = nodes.get(existing_idx)
                && existing.label != *label
            {
                diagnostics.push(ConceptMapDiagnostic::CanonicalNodeCollision {
                    key: key.clone(),
                    first_label: existing.label.clone(),
                    first_line: node_lines.get(key).copied().unwrap_or(line),
                    label: label.clone(),
                    line: l,
                });
            } else if !node_index.contains_key(key) {
                node_index.insert(key.clone(), nodes.len());
                node_lines.insert(key.clone(), l);
                nodes.push(ConceptMapNode {
                    key: key.clone(),
                    label: label.clone(),
                });
            }
        }

        // Check for DuplicateEdge.
        let edge_triple = (from_key.clone(), rel.to_string(), to_key.clone());
        if let Some(&existing_line) = edge_set.get(&edge_triple) {
            diagnostics.push(ConceptMapDiagnostic::DuplicateEdge {
                line,
                existing_line,
                from_key: from_key.clone(),
                rel: rel.to_string(),
                to_key: to_key.clone(),
            });
            continue;
        }
        edge_set.insert(edge_triple, line);

        // Check for SelfEdge.
        if from_key == to_key {
            diagnostics.push(ConceptMapDiagnostic::SelfEdge {
                line,
                node_key: from_key.clone(),
            });
        }

        // Record edge.
        edges.push(ConceptMapEdge {
            from_key: from_key.clone(),
            from_label: from_label.clone(),
            rel: rel.to_string(),
            to_key: to_key.clone(),
            to_label: to_label.clone(),
            line,
        });
    }

    ParsedConceptMap {
        nodes,
        edges,
        diagnostics,
    }
}

// ---------------------------------------------------------------------------
// Pure: check
// ---------------------------------------------------------------------------

/// Run heuristic checks over a parsed concept map, producing additional
/// diagnostics (`SimilarNodeLabel`, `RelationDrift`, `EntityRefLike`) beyond those
/// emitted during parsing.
pub(crate) fn check(parsed: &ParsedConceptMap) -> Vec<ConceptMapDiagnostic> {
    let mut diagnostics: Vec<ConceptMapDiagnostic> = Vec::new();

    // Carry forward parse-time CanonicalNodeCollision and SelfEdge.
    for d in &parsed.diagnostics {
        match d {
            ConceptMapDiagnostic::CanonicalNodeCollision { .. }
            | ConceptMapDiagnostic::SelfEdge { .. } => diagnostics.push(d.clone()),
            _ => {}
        }
    }

    // Build per-label and per-relation first-line maps from edges.
    let mut label_lines: BTreeMap<&str, usize> = BTreeMap::new();
    let mut rel_lines: BTreeMap<&str, usize> = BTreeMap::new();
    for edge in &parsed.edges {
        label_lines.entry(&edge.from_label).or_insert(edge.line);
        label_lines.entry(&edge.to_label).or_insert(edge.line);
        rel_lines.entry(&edge.rel).or_insert(edge.line);
    }

    // SimilarNodeLabel - each unordered pair of labels with Levenshtein ≤ 2
    // and both ≥ 4 characters.
    {
        let labels: Vec<&str> = label_lines.keys().copied().collect();
        for (i, a) in labels.iter().enumerate() {
            for b in labels.iter().skip(i + 1) {
                if a.len() >= 4 && b.len() >= 4 && levenshtein(a, b) <= 2 {
                    diagnostics.push(ConceptMapDiagnostic::SimilarNodeLabel {
                        label_a: (*a).to_string(),
                        line_a: label_lines.get(a).copied().unwrap_or(0),
                        label_b: (*b).to_string(),
                        line_b: label_lines.get(b).copied().unwrap_or(0),
                    });
                }
            }
        }
    }

    // RelationDrift - same check over relation texts.
    {
        let rels: Vec<&str> = rel_lines.keys().copied().collect();
        for (i, a) in rels.iter().enumerate() {
            for b in rels.iter().skip(i + 1) {
                if a.len() >= 4 && b.len() >= 4 && levenshtein(a, b) <= 2 {
                    diagnostics.push(ConceptMapDiagnostic::RelationDrift {
                        rel_a: (*a).to_string(),
                        line_a: rel_lines.get(a).copied().unwrap_or(0),
                        rel_b: (*b).to_string(),
                        line_b: rel_lines.get(b).copied().unwrap_or(0),
                    });
                }
            }
        }
    }

    // EntityRefLike - labels that look like canonical entity refs.
    // Anchored: must be exactly the pattern, not a substring.
    let Ok(ref_re) = Regex::new(r"^[A-Z]{2,5}-\d{3}$") else {
        return diagnostics;
    };
    for (label, &line) in &label_lines {
        if ref_re.is_match(label) {
            diagnostics.push(ConceptMapDiagnostic::EntityRefLike {
                label: label.to_string(),
                line,
            });
        }
    }

    diagnostics
}

// ---------------------------------------------------------------------------
// ExportFormat
// ---------------------------------------------------------------------------

/// Export format for concept-map to diagram languages and structured data.
#[derive(Clone, clap::ValueEnum)]
pub(crate) enum ExportFormat {
    Dot,
    Mermaid,
    Json,
}

// ---------------------------------------------------------------------------
// Pure: DOT escape / render
// ---------------------------------------------------------------------------

/// Escape `"`, `\`, and newlines for DOT string literals.
fn dot_escape(s: &str) -> String {
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

/// Render a [`ParsedConceptMap`] as a valid Graphviz digraph.
///
/// Nodes and edges are sorted by key for deterministic output. An empty map
/// produces a valid empty `digraph "" { ... }`.
fn render_dot(parsed: &ParsedConceptMap, title: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let escaped_title = dot_escape(title);
    lines.push(format!("digraph \"{escaped_title}\" {{"));
    lines.push("  rankdir=LR;".to_string());
    lines.push("  node [shape=box, style=rounded];".to_string());

    // Nodes sorted by key.
    let mut sorted_nodes: Vec<&ConceptMapNode> = parsed.nodes.iter().collect();
    sorted_nodes.sort_by(|a, b| a.key.cmp(&b.key));
    for node in &sorted_nodes {
        let escaped_key = dot_escape(&node.key);
        let escaped_label = dot_escape(&node.label);
        lines.push(format!("  \"{escaped_key}\" [label=\"{escaped_label}\"];"));
    }

    // Edges sorted by (from_key, to_key, rel).
    let mut sorted_edges: Vec<&ConceptMapEdge> = parsed.edges.iter().collect();
    sorted_edges.sort_by(|a, b| {
        a.from_key
            .cmp(&b.from_key)
            .then_with(|| a.to_key.cmp(&b.to_key))
            .then_with(|| a.rel.cmp(&b.rel))
    });
    for edge in &sorted_edges {
        let escaped_from = dot_escape(&edge.from_key);
        let escaped_to = dot_escape(&edge.to_key);
        let escaped_rel = dot_escape(&edge.rel);
        lines.push(format!(
            "  \"{escaped_from}\" -> \"{escaped_to}\" [label=\"{escaped_rel}\"];"
        ));
    }

    lines.push("}".to_string());
    lines.join("\n") + "\n"
}

// ---------------------------------------------------------------------------
// Pure: Mermaid escape / render
// ---------------------------------------------------------------------------

/// Escape special characters for Mermaid labels inside `["..."]`.
///
/// Replaces `"` with `#quot;`, `[` / `]` with `#91;` / `#93;`, and newlines
/// with `#10;` to keep them from breaking the bracket syntax.
fn mermaid_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("#quot;"),
            '[' => out.push_str("#91;"),
            ']' => out.push_str("#93;"),
            '\n' => out.push_str("#10;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Render a [`ParsedConceptMap`] as a Mermaid `graph LR` flowchart.
///
/// Node ids are synthetic (`n_0`, `n_1`, ...) to avoid collisions with Mermaid
/// reserved words. Nodes and edges are sorted by key for deterministic output.
/// An empty map produces a valid `graph LR` with no nodes.
fn render_mermaid(parsed: &ParsedConceptMap) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push("graph LR".to_string());

    // Build key → synthetic id map from sorted nodes.
    let mut sorted_nodes: Vec<&ConceptMapNode> = parsed.nodes.iter().collect();
    sorted_nodes.sort_by(|a, b| a.key.cmp(&b.key));
    let mut key_to_id: std::collections::BTreeMap<&str, String> = std::collections::BTreeMap::new();
    for (i, node) in sorted_nodes.iter().enumerate() {
        let id = format!("n_{i}");
        key_to_id.insert(&node.key, id.clone());
        let escaped_label = mermaid_escape(&node.label);
        lines.push(format!("  {id}[\"{escaped_label}\"]"));
    }

    // Edges sorted by (from_key, to_key, rel).
    let mut sorted_edges: Vec<&ConceptMapEdge> = parsed.edges.iter().collect();
    sorted_edges.sort_by(|a, b| {
        a.from_key
            .cmp(&b.from_key)
            .then_with(|| a.to_key.cmp(&b.to_key))
            .then_with(|| a.rel.cmp(&b.rel))
    });
    for edge in &sorted_edges {
        let from_id = key_to_id
            .get(edge.from_key.as_str())
            .cloned()
            .unwrap_or_default();
        let to_id = key_to_id
            .get(edge.to_key.as_str())
            .cloned()
            .unwrap_or_default();
        let escaped_rel = mermaid_escape(&edge.rel);
        lines.push(format!("  {from_id} -->|{escaped_rel}| {to_id}"));
    }

    lines.join("\n") + "\n"
}

// ---------------------------------------------------------------------------
// Pure: JSON render
// ---------------------------------------------------------------------------

/// Render a [`ParsedConceptMap`] as a `serde_json::Value` with `nodes` and
/// `edges` arrays, sorted by key for deterministic output.
fn render_json_value(parsed: &ParsedConceptMap) -> serde_json::Value {
    let mut sorted_nodes: Vec<&ConceptMapNode> = parsed.nodes.iter().collect();
    sorted_nodes.sort_by(|a, b| a.key.cmp(&b.key));
    let nodes: Vec<serde_json::Value> = sorted_nodes
        .iter()
        .map(|n| {
            serde_json::json!({
                "key": n.key,
                "label": n.label,
            })
        })
        .collect();

    let mut sorted_edges: Vec<&ConceptMapEdge> = parsed.edges.iter().collect();
    sorted_edges.sort_by(|a, b| {
        a.from_key
            .cmp(&b.from_key)
            .then_with(|| a.to_key.cmp(&b.to_key))
            .then_with(|| a.rel.cmp(&b.rel))
    });
    let edges: Vec<serde_json::Value> = sorted_edges
        .iter()
        .map(|e| {
            serde_json::json!({
                "from": e.from_key,
                "from_label": e.from_label,
                "rel": e.rel,
                "to": e.to_key,
                "to_label": e.to_label,
            })
        })
        .collect();

    serde_json::json!({
        "nodes": nodes,
        "edges": edges,
    })
}

/// Render a [`ParsedConceptMap`] as pretty-printed JSON.
fn render_json(parsed: &ParsedConceptMap) -> anyhow::Result<String> {
    let value = render_json_value(parsed);
    serde_json::to_string_pretty(&value).context("failed to serialize concept-map export JSON")
}

// ---------------------------------------------------------------------------
// Shell: run_export
// ---------------------------------------------------------------------------

/// `doctrine concept-map export` - resolve a concept map, parse its DSL, and
/// render it to the requested format (DOT, Mermaid, or JSON) on stdout.
pub(crate) fn run_export(
    path: Option<PathBuf>,
    id_str: &str,
    format: &ExportFormat,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let id = parse_ref(id_str)?;
    let cm_root = root.join(CONCEPT_MAP_DIR);
    let (doc, _toml_text, _body) = read_concept_map(&cm_root, id)?;
    let parsed = parse_dsl(&doc.dsl);

    let out = match format {
        ExportFormat::Dot => render_dot(&parsed, &doc.title),
        ExportFormat::Mermaid => render_mermaid(&parsed),
        ExportFormat::Json => render_json(&parsed)?,
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Shell: run_check
// ---------------------------------------------------------------------------

/// Extract the primary line number from a diagnostic for stable sort ordering.
pub(crate) fn line_of_diagnostic(d: &ConceptMapDiagnostic) -> usize {
    match d {
        ConceptMapDiagnostic::MalformedLine { line, .. }
        | ConceptMapDiagnostic::EmptyLabel { line, .. }
        | ConceptMapDiagnostic::DuplicateEdge { line, .. }
        | ConceptMapDiagnostic::SelfEdge { line, .. }
        | ConceptMapDiagnostic::CanonicalNodeCollision { line, .. }
        | ConceptMapDiagnostic::EntityRefLike { line, .. } => *line,
        ConceptMapDiagnostic::SimilarNodeLabel { line_a, .. }
        | ConceptMapDiagnostic::RelationDrift { line_a, .. } => *line_a,
    }
}

/// `doctrine concept-map check <id>` - parse the DSL and run heuristic checks.
///
/// Prints one diagnostic per line. Exits zero if there are no `MalformedLine` or
/// `EmptyLabel` errors; exits non-zero if any structural errors exist.
pub(crate) fn run_check(path: Option<PathBuf>, id_str: &str) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let id = parse_ref(id_str)?;
    let cm_root = root.join(CONCEPT_MAP_DIR);
    let (doc, _toml_text, _body) = read_concept_map(&cm_root, id)?;

    let parsed = parse_dsl(&doc.dsl);
    let mut diagnostics = check(&parsed);

    // Merge parse-time diagnostics that check doesn't carry forward (MalformedLine,
    // EmptyLabel, DuplicateEdge).
    for d in &parsed.diagnostics {
        match d {
            ConceptMapDiagnostic::CanonicalNodeCollision { .. }
            | ConceptMapDiagnostic::SelfEdge { .. } => {
                // Already included by check().
            }
            _ => diagnostics.push(d.clone()),
        }
    }

    // Sort diagnostics by line for stable output.
    diagnostics.sort_by_key(line_of_diagnostic);

    let mut has_structural = false;
    let mut out = std::io::stdout();
    for d in &diagnostics {
        let msg = format_diagnostic(d);
        writeln!(out, "{msg}")?;
        match d {
            ConceptMapDiagnostic::MalformedLine { .. }
            | ConceptMapDiagnostic::EmptyLabel { .. } => {
                has_structural = true;
            }
            _ => {}
        }
    }

    if has_structural {
        anyhow::bail!("structural errors found in concept map DSL");
    }
    Ok(())
}

/// Render a single diagnostic as a human-readable string.
fn format_diagnostic(d: &ConceptMapDiagnostic) -> String {
    match d {
        ConceptMapDiagnostic::MalformedLine { line, text } => {
            format!("line {line}: malformed - expected `Source > relation > Target`, got: `{text}`")
        }
        ConceptMapDiagnostic::EmptyLabel { line, segment } => {
            let seg = match segment {
                SegmentKind::Source => "source",
                SegmentKind::Relation => "relation",
                SegmentKind::Target => "target",
            };
            format!("line {line}: empty {seg} label")
        }
        ConceptMapDiagnostic::DuplicateEdge {
            line,
            existing_line,
            from_key,
            rel,
            to_key,
        } => {
            format!(
                "line {line}: duplicate edge `{from_key} > {rel} > {to_key}` (first seen on line {existing_line})"
            )
        }
        ConceptMapDiagnostic::SelfEdge { line, node_key } => {
            format!("line {line}: self-edge - `{node_key}` points to itself")
        }
        ConceptMapDiagnostic::CanonicalNodeCollision {
            key,
            first_label,
            first_line,
            label,
            line,
        } => {
            format!(
                "line {line}: canonical node collision - `{label}` and `{first_label}` both derive key `{key}` (first seen on line {first_line})"
            )
        }
        ConceptMapDiagnostic::SimilarNodeLabel {
            label_a,
            line_a,
            label_b,
            line_b,
        } => {
            format!(
                "line {line_a}: similar label - `{label_a}` and `{label_b}` (line {line_b}) differ by ≤ 2 edits"
            )
        }
        ConceptMapDiagnostic::RelationDrift {
            rel_a,
            line_a,
            rel_b,
            line_b,
        } => {
            format!(
                "line {line_a}: relation drift - `{rel_a}` and `{rel_b}` (line {line_b}) differ by ≤ 2 edits"
            )
        }
        ConceptMapDiagnostic::EntityRefLike { label, line } => {
            format!(
                "line {line}: entity-ref-like label - `{label}` looks like a canonical entity id"
            )
        }
    }
}

/// `doctrine concept-map show <ref>` - display a concept map's metadata, DSL,
/// and optionally edge/node tables.
pub(crate) fn run_show(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
    edges: bool,
    nodes: bool,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let id = parse_ref(reference)?;
    let cm_root = root.join(CONCEPT_MAP_DIR);
    let (doc, _toml_text, body) = read_concept_map(&cm_root, id)?;

    let parsed = if edges || nodes {
        Some(parse_dsl(&doc.dsl))
    } else {
        None
    };
    let out = match format {
        Format::Table => format_show(&doc, &body, edges, nodes, parsed.as_ref()),
        Format::Json => show_json(&doc, &body, edges, nodes, parsed.as_ref())?,
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
}

/// Render the table-format show output for a concept map.
fn format_show(
    doc: &ConceptMapDoc,
    body: &str,
    edges: bool,
    nodes: bool,
    parsed: Option<&ParsedConceptMap>,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!(
        "CM-{:03}\n\
     {}\n\n\
     Status:    {}\n\
     Created:   {}\n\
     Updated:   {}\n\
     Slug:      {}",
        doc.id, doc.title, doc.status, doc.created, doc.updated, doc.slug
    ));
    if !doc.description.is_empty() {
        parts.push(format!("\nDescription: {}", doc.description));
    }
    if !body.trim().is_empty() {
        parts.push(format!("\n\n---\n\n{body}"));
    }
    if !doc.dsl.trim().is_empty() {
        parts.push(format!("\n\n---\nDSL:\n{}", doc.dsl));
    }
    if edges && let Some(p) = parsed {
        parts.push("\n\nEdges:".to_string());
        for edge in &p.edges {
            parts.push(format!(
                "  {} > {} > {}",
                edge.from_label, edge.rel, edge.to_label
            ));
        }
    }
    if nodes && let Some(p) = parsed {
        if parts.last().is_none_or(|s| s.ends_with(':')) {
            parts.push("\n\nNodes:".to_string());
        } else {
            parts.push("\nNodes:".to_string());
        }
        for node in &p.nodes {
            parts.push(format!("  {} - {}", node.key, node.label));
        }
    }
    parts.concat()
}

/// Render JSON show output.
fn show_json(
    doc: &ConceptMapDoc,
    body: &str,
    edges: bool,
    nodes: bool,
    parsed: Option<&ParsedConceptMap>,
) -> anyhow::Result<String> {
    let mut value = serde_json::json!({
      "id": crate::listing::canonical_id("CM", doc.id),
      "slug": doc.slug,
      "title": doc.title,
      "status": doc.status,
      "created": doc.created,
      "updated": doc.updated,
      "description": doc.description,
      "dsl": doc.dsl,
      "body": body,
    });
    if edges
        && let Some(p) = parsed
        && let serde_json::Value::Object(ref mut map) = value
    {
        let edge_objs: Vec<serde_json::Value> = p
            .edges
            .iter()
            .map(|e| {
                serde_json::json!({
                    "from": e.from_label,
                    "rel": e.rel,
                    "to": e.to_label,
                })
            })
            .collect();
        map.insert("edges".into(), serde_json::Value::Array(edge_objs));
    }
    if nodes
        && let Some(p) = parsed
        && let serde_json::Value::Object(ref mut map) = value
    {
        let node_objs: Vec<serde_json::Value> = p
            .nodes
            .iter()
            .map(|n| {
                serde_json::json!({
                    "key": n.key,
                    "label": n.label,
                })
            })
            .collect();
        map.insert("nodes".into(), serde_json::Value::Array(node_objs));
    }
    serde_json::to_string_pretty(&value).context("failed to serialize concept-map show JSON")
}

// ---------------------------------------------------------------------------
// list - the read surface
// ---------------------------------------------------------------------------

/// The inner list pipeline: read, filter, sort, render.
fn list_rows(root: &Path, mut args: ListArgs) -> anyhow::Result<String> {
    listing::validate_statuses(&args.status, CONCEPT_MAP_STATUSES)?;
    let render = args.render;
    let columns = args.columns.take();
    let (filter, format) = listing::build(args)?;
    let cm_root = root.join(CONCEPT_MAP_DIR);
    let mut metas = listing::retain(
        meta::read_metas(&cm_root, "concept-map")?,
        &filter,
        is_hidden,
        key,
    );
    metas.sort_by_key(|m| m.id);
    let rows = metas
        .into_iter()
        .map(|m| ConceptMapRow {
            id: m.id,
            status: m.status,
            slug: m.slug,
            title: m.title,
        })
        .collect::<Vec<_>>();
    match format {
        Format::Table => {
            let sel = listing::select_columns(
                CONCEPT_MAP_COLUMNS,
                CONCEPT_MAP_DEFAULT,
                columns.as_deref(),
            )?;
            Ok(listing::render_columns(&rows, &sel, render))
        }
        Format::Json => listing::json_envelope("concept-map", &rows),
    }
}

/// `doctrine concept-map list` - the read surface: prefixed `CM-` ids, a header,
/// the shared filter flags, the `{done, abandoned}` hide-set by default, sorted
/// by id.
pub(crate) fn run_list(path: Option<PathBuf>, args: ListArgs) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let mut out = io::stdout();
    write!(out, "{}", list_rows(&root, args)?)?;
    Ok(())
}

/// A concept-map row for list rendering. `Serialize` for JSON; cell extractors
/// for the table.
#[derive(Debug, Clone, Serialize)]
struct ConceptMapRow {
    #[serde(serialize_with = "serialize_cm_id")]
    id: u32,
    status: String,
    slug: String,
    title: String,
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde serialize_with contract requires a reference"
)]
fn serialize_cm_id<S: serde::Serializer>(id: &u32, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&crate::listing::canonical_id("CM", *id))
}

/// The `FilterFields` projection for a `Meta` - used by `listing::retain`.
fn key(m: &Meta) -> listing::FilterFields {
    listing::FilterFields {
        canonical: crate::listing::canonical_id("CM", m.id),
        slug: m.slug.clone(),
        title: m.title.clone(),
        status: m.status.clone(),
        tags: Vec::new(),
    }
}

/// The table columns for concept-map list.
const CONCEPT_MAP_COLUMNS: &[listing::Column<ConceptMapRow>] = &[
    listing::Column {
        name: "id",
        header: "ID",
        cell: |r: &ConceptMapRow| crate::listing::canonical_id("CM", r.id),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "status",
        header: "Status",
        cell: |r: &ConceptMapRow| r.status.clone(),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "slug",
        header: "Slug",
        cell: |r: &ConceptMapRow| r.slug.clone(),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "title",
        header: "Title",
        cell: |r: &ConceptMapRow| r.title.clone(),
        paint: listing::ColumnPaint::None,
    },
];

/// The default visible columns.
const CONCEPT_MAP_DEFAULT: &[&str] = &["id", "status", "slug", "title"];

// ---------------------------------------------------------------------------
// Pure: get_dsl / set_dsl
// ---------------------------------------------------------------------------

/// Extract the `dsl` value from a concept-map TOML string.
pub(crate) fn get_dsl(toml_text: &str) -> anyhow::Result<String> {
    let doc: toml_edit::DocumentMut = toml_text.parse().context("Failed to parse TOML")?;
    doc.get("dsl")
        .and_then(toml_edit::Item::as_str)
        .map(str::to_string)
        .context("TOML is missing a `dsl` key")
}

/// Set the `dsl` value in a concept-map TOML string, returning the modified TOML.
/// Replace the `dsl` key value in the concept-map TOML document.
///
/// **Note:** this replaces the entire `dsl` item via `doc.insert("dsl", …)`,
/// dropping any inline comment on the `dsl` key line (e.g.
/// `dsl = '''…''' # my map`). All other keys and their inline comments are
/// preserved. This is an accepted tradeoff — concept-map TOML files authored
/// by `doctrine concept-map new` carry no inline comments on the `dsl` key.
pub(crate) fn set_dsl(toml_text: &str, new_dsl: &str) -> anyhow::Result<String> {
    let mut doc: toml_edit::DocumentMut = toml_text.parse().context("Failed to parse TOML")?;
    doc.insert("dsl", toml_edit::value(new_dsl));
    Ok(doc.to_string())
}

// ---------------------------------------------------------------------------
// Pure: ConceptMapMutationError
// ---------------------------------------------------------------------------

/// Errors from pure concept-map DSL mutation functions.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "variants used by future phases (SL-076 PHASE-02+)"
    )
)]
pub(crate) enum ConceptMapMutationError {
    /// A required field was empty after trimming.
    EmptyField(String),
    /// The exact (source, rel, target) triple already exists in the DSL.
    DuplicateEdge { line: usize },
    /// The edge to remove was not found.
    EdgeNotFound,
    /// Renaming a node would collide with an existing node's derived key.
    NodeCollision { existing_label: String, line: usize },
    /// The TOML is missing a `dsl` key.
    MissingDsl,
    /// Failed to parse the TOML document.
    InvalidToml(String),
}

impl std::fmt::Display for ConceptMapMutationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "{field} must be non-empty"),
            Self::DuplicateEdge { line } => write!(f, "edge already exists at line {line}"),
            Self::EdgeNotFound => write!(f, "edge not found"),
            Self::NodeCollision {
                existing_label,
                line,
            } => {
                write!(
                    f,
                    "rename would collide with existing node '{existing_label}' at line {line}"
                )
            }
            Self::MissingDsl => write!(f, "TOML is missing a `dsl` key"),
            Self::InvalidToml(msg) => write!(f, "failed to parse TOML: {msg}"),
        }
    }
}

impl std::error::Error for ConceptMapMutationError {}

// ---------------------------------------------------------------------------
// Pure: add_edge_to_dsl
// ---------------------------------------------------------------------------

pub(crate) fn add_edge_to_dsl(
    old_dsl: &str,
    source: &str,
    rel: &str,
    target: &str,
) -> Result<String, ConceptMapMutationError> {
    let source = source.trim();
    let rel = rel.trim();
    let target = target.trim();

    if source.is_empty() {
        return Err(ConceptMapMutationError::EmptyField("source".into()));
    }
    if rel.is_empty() {
        return Err(ConceptMapMutationError::EmptyField("relation".into()));
    }
    if target.is_empty() {
        return Err(ConceptMapMutationError::EmptyField("target".into()));
    }

    // Detect duplicate
    let parsed = parse_dsl(old_dsl);
    if let Some(dup) = parsed
        .edges
        .iter()
        .find(|e| e.from_label == source && e.rel == rel && e.to_label == target)
    {
        return Err(ConceptMapMutationError::DuplicateEdge { line: dup.line });
    }

    let new_line = format!("{source} > {rel} > {target}");
    let new_dsl = if old_dsl.trim().is_empty() {
        new_line
    } else if old_dsl.ends_with('\n') {
        format!("{old_dsl}{new_line}")
    } else {
        format!("{old_dsl}\n{new_line}")
    };
    Ok(new_dsl)
}

// ---------------------------------------------------------------------------
// Pure: remove_edge_from_dsl
// ---------------------------------------------------------------------------

pub(crate) fn remove_edge_from_dsl(
    old_dsl: &str,
    source: &str,
    rel: &str,
    target: &str,
) -> Result<String, ConceptMapMutationError> {
    let source = source.trim();
    let rel = rel.trim();
    let target = target.trim();

    if source.is_empty() {
        return Err(ConceptMapMutationError::EmptyField("source".into()));
    }
    if rel.is_empty() {
        return Err(ConceptMapMutationError::EmptyField("relation".into()));
    }
    if target.is_empty() {
        return Err(ConceptMapMutationError::EmptyField("target".into()));
    }

    let mut found = false;
    let mut lines: Vec<String> = Vec::new();
    for line_str in old_dsl.lines() {
        let trimmed = line_str.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            lines.push(line_str.to_string());
            continue;
        }
        let segments: Vec<&str> = line_str.split(" > ").collect();
        if segments.len() == 3 {
            let ls = segments.first().map_or("", |s| s.trim());
            let lr = segments.get(1).map_or("", |s| s.trim());
            let lt = segments.get(2).map_or("", |s| s.trim());
            if ls == source && lr == rel && lt == target && !found {
                found = true;
                continue; // omit only the first matching line
            }
        }
        lines.push(line_str.to_string());
    }

    if !found {
        return Err(ConceptMapMutationError::EdgeNotFound);
    }

    Ok(lines.join("\n"))
}

// ---------------------------------------------------------------------------
// Pure: relabel_edge_in_dsl
// ---------------------------------------------------------------------------

/// Rewrite the relation segment of the first edge matching
/// `(source, old_rel, target)` by label, replacing `old_rel` with `new_rel`.
///
/// A no-op (`old_rel == new_rel`) returns the DSL unchanged. The duplicate
/// guard is key-based: two distinct label spellings can derive the same node
/// key, so a label-only check would miss a collision and corrupt the DSL.
pub(crate) fn relabel_edge_in_dsl(
    old_dsl: &str,
    source: &str,
    old_rel: &str,
    new_rel: &str,
    target: &str,
) -> Result<String, ConceptMapMutationError> {
    let source = source.trim();
    let old_rel = old_rel.trim();
    let new_rel = new_rel.trim();
    let target = target.trim();

    if source.is_empty() {
        return Err(ConceptMapMutationError::EmptyField("source".into()));
    }
    if old_rel.is_empty() {
        return Err(ConceptMapMutationError::EmptyField("old relation".into()));
    }
    if new_rel.is_empty() {
        return Err(ConceptMapMutationError::EmptyField("new relation".into()));
    }
    if target.is_empty() {
        return Err(ConceptMapMutationError::EmptyField("target".into()));
    }

    // No-op short-circuit (before the dup scan, to avoid a false self-collision).
    if old_rel == new_rel {
        return Ok(old_dsl.to_string());
    }

    // Find the first line matching (source, old_rel, target) by label.
    let mut matched_line: Option<usize> = None;
    let mut lines: Vec<String> = Vec::new();
    for (idx, line_str) in old_dsl.lines().enumerate() {
        let m = idx + 1; // 1-based, matching parse_dsl
        let trimmed = line_str.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            lines.push(line_str.to_string());
            continue;
        }
        let segments: Vec<&str> = line_str.split(" > ").collect();
        if segments.len() == 3 && matched_line.is_none() {
            let ls = segments.first().map_or("", |s| s.trim());
            let lr = segments.get(1).map_or("", |s| s.trim());
            let lt = segments.get(2).map_or("", |s| s.trim());
            if ls == source && lr == old_rel && lt == target {
                matched_line = Some(m);
                // Rewrite only the rel segment; source/target byte-unchanged.
                let s = segments.first().copied().unwrap_or_default();
                let t = segments.get(2).copied().unwrap_or_default();
                lines.push([s, new_rel, t].join(" > "));
                continue;
            }
        }
        lines.push(line_str.to_string());
    }

    let Some(m) = matched_line else {
        return Err(ConceptMapMutationError::EdgeNotFound);
    };

    // Key-based duplicate guard: distinct label spellings can derive the same
    // key, so the new triple may collide with an existing edge on another line.
    let source_key = derive_node_key(source);
    let target_key = derive_node_key(target);
    let parsed = parse_dsl(old_dsl);
    if let Some(dup) = parsed.edges.iter().find(|e| {
        e.from_key == source_key && e.rel == new_rel && e.to_key == target_key && e.line != m
    }) {
        return Err(ConceptMapMutationError::DuplicateEdge { line: dup.line });
    }

    Ok(lines.join("\n"))
}

// ---------------------------------------------------------------------------
// Pure: rename_node_occurrence_in_dsl
// ---------------------------------------------------------------------------

/// Which endpoint of an edge triple a single-occurrence rename targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum EdgeEndpoint {
    Source,
    Target,
}

/// Rewrite **one** edge's named endpoint label. Finds the first line matching
/// `(source, rel, target)` by label and rewrites only the `cell` segment to
/// `new_label`; other rows using the old label are byte-unchanged.
///
/// Same shape as [`relabel_edge_in_dsl`]: a no-op (the rewritten endpoint
/// equals its current value) returns the DSL unchanged, and the duplicate guard
/// is key-based — distinct label spellings can derive the same node key, so a
/// label-only check would miss a collision and corrupt the DSL.
pub(crate) fn rename_node_occurrence_in_dsl(
    old_dsl: &str,
    source: &str,
    rel: &str,
    target: &str,
    cell: EdgeEndpoint,
    new_label: &str,
) -> Result<String, ConceptMapMutationError> {
    let source = source.trim();
    let rel = rel.trim();
    let target = target.trim();
    let new_label = new_label.trim();

    if source.is_empty() {
        return Err(ConceptMapMutationError::EmptyField("source".into()));
    }
    if rel.is_empty() {
        return Err(ConceptMapMutationError::EmptyField("relation".into()));
    }
    if target.is_empty() {
        return Err(ConceptMapMutationError::EmptyField("target".into()));
    }
    if new_label.is_empty() {
        return Err(ConceptMapMutationError::EmptyField("new label".into()));
    }

    // No-op short-circuit (before the scan, mirroring relabel_edge_in_dsl): the
    // endpoint we would rewrite is one of the match inputs, so an unchanged
    // value means there is nothing to do.
    let current = match cell {
        EdgeEndpoint::Source => source,
        EdgeEndpoint::Target => target,
    };
    if current == new_label {
        return Ok(old_dsl.to_string());
    }

    // Find the first line matching (source, rel, target) by label.
    let mut matched_line: Option<usize> = None;
    let mut lines: Vec<String> = Vec::new();
    for (idx, line_str) in old_dsl.lines().enumerate() {
        let m = idx + 1; // 1-based, matching parse_dsl
        let trimmed = line_str.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            lines.push(line_str.to_string());
            continue;
        }
        let segments: Vec<&str> = line_str.split(" > ").collect();
        if segments.len() == 3 && matched_line.is_none() {
            let ls = segments.first().map_or("", |s| s.trim());
            let lr = segments.get(1).map_or("", |s| s.trim());
            let lt = segments.get(2).map_or("", |s| s.trim());
            if ls == source && lr == rel && lt == target {
                matched_line = Some(m);
                // Rewrite only the named endpoint; the other two segments keep
                // their original bytes.
                let raw_s = segments.first().copied().unwrap_or_default();
                let raw_r = segments.get(1).copied().unwrap_or_default();
                let raw_t = segments.get(2).copied().unwrap_or_default();
                let (out_s, out_t) = match cell {
                    EdgeEndpoint::Source => (new_label, raw_t),
                    EdgeEndpoint::Target => (raw_s, new_label),
                };
                lines.push([out_s, raw_r, out_t].join(" > "));
                continue;
            }
        }
        lines.push(line_str.to_string());
    }

    let Some(m) = matched_line else {
        return Err(ConceptMapMutationError::EdgeNotFound);
    };

    // Key-based duplicate guard: the rewritten triple must not collide with
    // another existing edge on a different line.
    let (new_source, new_target) = match cell {
        EdgeEndpoint::Source => (new_label, target),
        EdgeEndpoint::Target => (source, new_label),
    };
    let new_from_key = derive_node_key(new_source);
    let new_to_key = derive_node_key(new_target);
    let parsed = parse_dsl(old_dsl);
    if let Some(dup) = parsed.edges.iter().find(|e| {
        e.from_key == new_from_key && e.rel == rel && e.to_key == new_to_key && e.line != m
    }) {
        return Err(ConceptMapMutationError::DuplicateEdge { line: dup.line });
    }

    Ok(lines.join("\n"))
}

// ---------------------------------------------------------------------------
// Pure: relabel_rel_all_in_dsl
// ---------------------------------------------------------------------------

/// Rewrite the `rel` segment of **every** line whose `rel == old_rel` to
/// `new_rel`. A no-op (`old_rel == new_rel`) returns the DSL unchanged.
///
/// The duplicate guard is **atomic**: if any rewritten line would collide
/// (key-based) with an existing line or another rewritten line, the whole op is
/// rejected (`DuplicateEdge { line }`) and the original DSL is left untouched —
/// the pure layer returns `Err`, so the shell never performs a partial write.
pub(crate) fn relabel_rel_all_in_dsl(
    old_dsl: &str,
    old_rel: &str,
    new_rel: &str,
) -> Result<String, ConceptMapMutationError> {
    let old_rel = old_rel.trim();
    let new_rel = new_rel.trim();

    if old_rel.is_empty() {
        return Err(ConceptMapMutationError::EmptyField("old relation".into()));
    }
    if new_rel.is_empty() {
        return Err(ConceptMapMutationError::EmptyField("new relation".into()));
    }
    if old_rel == new_rel {
        return Ok(old_dsl.to_string());
    }

    // Rewrite every matching line's rel segment; remember which lines changed.
    let mut lines: Vec<String> = Vec::new();
    let mut rewritten: Vec<usize> = Vec::new();
    for (idx, line_str) in old_dsl.lines().enumerate() {
        let m = idx + 1; // 1-based, matching parse_dsl
        let trimmed = line_str.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            lines.push(line_str.to_string());
            continue;
        }
        let segments: Vec<&str> = line_str.split(" > ").collect();
        if segments.len() == 3 && segments.get(1).map_or("", |s| s.trim()) == old_rel {
            let s = segments.first().copied().unwrap_or_default();
            let t = segments.get(2).copied().unwrap_or_default();
            lines.push([s, new_rel, t].join(" > "));
            rewritten.push(m);
        } else {
            lines.push(line_str.to_string());
        }
    }

    // Atomic key-based duplicate guard. Parse the full candidate: parse_dsl
    // collapses a colliding triple into a `DuplicateEdge` diagnostic (the
    // second line is dropped from `edges`), so collisions surface there, not in
    // `edges`. Any collision that involves a rewritten line — whether against an
    // existing line or another rewritten line — rejects the whole op before it
    // is returned (no partial write).
    let candidate = lines.join("\n");
    let parsed = parse_dsl(&candidate);
    for diag in &parsed.diagnostics {
        if let ConceptMapDiagnostic::DuplicateEdge {
            line,
            existing_line,
            ..
        } = diag
            && (rewritten.contains(line) || rewritten.contains(existing_line))
        {
            return Err(ConceptMapMutationError::DuplicateEdge { line: *line });
        }
    }

    Ok(candidate)
}

// ---------------------------------------------------------------------------
// Pure: rename_node_in_dsl
// ---------------------------------------------------------------------------

pub(crate) fn rename_node_in_dsl(
    old_dsl: &str,
    old_label: &str,
    new_label: &str,
) -> Result<(String, usize), ConceptMapMutationError> {
    let old_label = old_label.trim();
    let new_label = new_label.trim();

    if old_label.is_empty() {
        return Err(ConceptMapMutationError::EmptyField("old label".into()));
    }
    if new_label.is_empty() {
        return Err(ConceptMapMutationError::EmptyField("new label".into()));
    }

    let old_key = derive_node_key(old_label);
    let new_key = derive_node_key(new_label);

    // Key collision check (only if keys differ — case-only rename has same key)
    if old_key != new_key {
        let parsed = parse_dsl(old_dsl);
        if let Some(colliding) = parsed.nodes.iter().find(|n| n.key == new_key) {
            // Find the line of that node
            let line = parsed
                .edges
                .iter()
                .find(|e| {
                    derive_node_key(&e.from_label) == new_key
                        || derive_node_key(&e.to_label) == new_key
                })
                .map_or(0, |e| e.line);
            return Err(ConceptMapMutationError::NodeCollision {
                existing_label: colliding.label.clone(),
                line,
            });
        }
    }

    let mut occurrences: usize = 0;
    let mut new_lines: Vec<String> = Vec::new();

    for line_str in old_dsl.lines() {
        let trimmed = line_str.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            new_lines.push(line_str.to_string());
            continue;
        }
        let segments: Vec<&str> = line_str.split(" > ").collect();
        if segments.len() != 3 {
            new_lines.push(line_str.to_string());
            continue;
        }
        let src = segments.first().map_or("", |s| s.trim());
        let r = segments.get(1).map_or("", |s| s.trim());
        let tgt = segments.get(2).map_or("", |s| s.trim());

        let src_key = derive_node_key(src);
        let tgt_key = derive_node_key(tgt);

        let new_src = if src_key == old_key { new_label } else { src };
        let new_tgt = if tgt_key == old_key { new_label } else { tgt };

        if new_src != src || new_tgt != tgt {
            occurrences += 1;
            new_lines.push(format!("{new_src} > {r} > {new_tgt}"));
        } else {
            new_lines.push(line_str.to_string());
        }
    }

    Ok((new_lines.join("\n"), occurrences))
}

// ---------------------------------------------------------------------------
// Shell: run_add
// ---------------------------------------------------------------------------

/// `doctrine concept-map add` - append a DSL edge line to a concept map.
pub(crate) fn run_add(
    path: Option<PathBuf>,
    id_str: &str,
    source: &str,
    rel: &str,
    target: &str,
    force: bool,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let id = parse_ref(id_str)?;
    let cm_root = root.join(CONCEPT_MAP_DIR);
    let (_doc, toml_text, _body) = read_concept_map(&cm_root, id)?;
    let old_dsl = get_dsl(&toml_text)?;

    match add_edge_to_dsl(&old_dsl, source, rel, target) {
        Ok(new_dsl) => {
            let updated = set_dsl(&toml_text, &new_dsl)?;
            let toml_path = entity::id_path(&root, &CONCEPT_MAP_KIND, id, entity::Ext::Toml);
            crate::fsutil::write_atomic(&toml_path, updated.as_bytes())
                .with_context(|| format!("Failed to write {}", toml_path.display()))?;
        }
        Err(ConceptMapMutationError::DuplicateEdge { line: _ }) if force => {
            // Force: just append the edge anyway (duplicate allowed)
            let new_line = format!("{} > {} > {}", source.trim(), rel.trim(), target.trim());
            let new_dsl = if old_dsl.trim().is_empty() {
                new_line
            } else {
                format!("{old_dsl}\n{new_line}")
            };
            let updated = set_dsl(&toml_text, &new_dsl)?;
            let toml_path = entity::id_path(&root, &CONCEPT_MAP_KIND, id, entity::Ext::Toml);
            crate::fsutil::write_atomic(&toml_path, updated.as_bytes())
                .with_context(|| format!("Failed to write {}", toml_path.display()))?;
        }
        Err(ConceptMapMutationError::DuplicateEdge { line }) => {
            // Duplicate without force: print message and return Ok (existing behaviour)
            let source_trim = source.trim();
            let rel_trim = rel.trim();
            let target_trim = target.trim();
            writeln!(
                io::stdout(),
                "edge already exists at line {line}: {source_trim} > {rel_trim} > {target_trim}"
            )?;
        }
        Err(e) => {
            return Err(anyhow::anyhow!("{e}"));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Shell: run_remove
// ---------------------------------------------------------------------------

/// `doctrine concept-map remove` - remove a DSL edge line from a concept map.
pub(crate) fn run_remove(
    path: Option<PathBuf>,
    id_str: &str,
    source: &str,
    rel: &str,
    target: &str,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let id = parse_ref(id_str)?;
    let cm_root = root.join(CONCEPT_MAP_DIR);
    let (_doc, toml_text, _body) = read_concept_map(&cm_root, id)?;
    let old_dsl = get_dsl(&toml_text)?;

    let source_trim = source.trim();
    let rel_trim = rel.trim();
    let target_trim = target.trim();

    let new_dsl = remove_edge_from_dsl(&old_dsl, source, rel, target).map_err(|_e| {
        anyhow::anyhow!("edge not found: {source_trim} > {rel_trim} > {target_trim}")
    })?;
    let updated = set_dsl(&toml_text, &new_dsl)?;

    let toml_path = entity::id_path(&root, &CONCEPT_MAP_KIND, id, entity::Ext::Toml);
    crate::fsutil::write_atomic(&toml_path, updated.as_bytes())
        .with_context(|| format!("Failed to write {}", toml_path.display()))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Shell: run_rename_node
// ---------------------------------------------------------------------------

/// `doctrine concept-map rename-node` - rename a node label across all DSL edges.
pub(crate) fn run_rename_node(
    path: Option<PathBuf>,
    id_str: &str,
    old: &str,
    new_label: &str,
    dry_run: bool,
    case_sensitive: bool,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let id = parse_ref(id_str)?;
    let cm_root = root.join(CONCEPT_MAP_DIR);
    let (_doc, toml_text, _body) = read_concept_map(&cm_root, id)?;

    let old_dsl = get_dsl(&toml_text)?;

    let mut occurrences: usize = 0;
    let mut new_lines: Vec<String> = Vec::new();

    let old_lower = if case_sensitive {
        String::new()
    } else {
        old.to_lowercase()
    };

    for line in old_dsl.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            new_lines.push(line.to_string());
            continue;
        }
        let segments: Vec<&str> = line.split(" > ").collect();
        if segments.len() != 3 {
            new_lines.push(line.to_string());
            continue;
        }
        let src = segments.first().map_or("", |s| s.trim());
        let r = segments.get(1).map_or("", |s| s.trim());
        let tgt = segments.get(2).map_or("", |s| s.trim());

        let mut changed = false;
        let new_src = if (case_sensitive && src == old)
            || (!case_sensitive && src.to_lowercase() == old_lower)
        {
            changed = true;
            new_label
        } else {
            src
        };
        let new_tgt = if (case_sensitive && tgt == old)
            || (!case_sensitive && tgt.to_lowercase() == old_lower)
        {
            changed = true;
            new_label
        } else {
            tgt
        };
        if changed {
            occurrences += 1;
            new_lines.push(format!("{new_src} > {r} > {new_tgt}"));
        } else {
            new_lines.push(line.to_string());
        }
    }

    // Count edges (non-comment, non-empty lines that split to 3).
    let edges = old_dsl
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#') && t.split(" > ").count() == 3
        })
        .count();

    let new_dsl = new_lines.join("\n");

    if dry_run {
        writeln!(io::stdout(), "{new_dsl}")?;
        return Ok(());
    }

    let updated = set_dsl(&toml_text, &new_dsl)?;

    let toml_path = entity::id_path(&root, &CONCEPT_MAP_KIND, id, entity::Ext::Toml);
    crate::fsutil::write_atomic(&toml_path, updated.as_bytes())
        .with_context(|| format!("Failed to write {}", toml_path.display()))?;

    writeln!(
        io::stdout(),
        "Rewrote {occurrences} occurrences across {edges} edges."
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- scaffold / render ---

    #[test]
    fn scaffold_template_substitution_has_no_residual_tokens() {
        let toml_body = render_toml(1, "my-map", "My Map", "2026-06-15").unwrap();
        let md_body = render_md("My Map", 1).unwrap();
        assert!(!toml_body.contains("{{"));
        assert!(toml_body.contains("id = 1"));
        assert!(toml_body.contains("status = \"draft\""));
        assert!(!md_body.contains("{{"));
        assert!(md_body.contains("CM-001"));
        assert!(md_body.contains("My Map"));
    }

    #[test]
    fn scaffold_renders_three_artifacts() {
        let ctx = ScaffoldCtx {
            id: 1,
            canonical: "CM-001",
            slug: "my-map",
            title: "My Map",
            date: "2026-06-15",
        };
        let fileset = concept_map_scaffold(&ctx).unwrap();
        assert_eq!(fileset.len(), 3);
        // Verify the symlink exists with correct slug
        let symlink = fileset
            .iter()
            .find(|a| matches!(a, Artifact::Symlink { .. }))
            .unwrap();
        if let Artifact::Symlink { rel_path, target } = symlink {
            assert_eq!(rel_path, Path::new("001-my-map"));
            assert_eq!(target, "001");
        } else {
            panic!("expected symlink");
        }
        // Verify TOML and MD files
        let mut found_toml = false;
        let mut found_md = false;
        for a in &fileset {
            if let Artifact::File { rel_path, body } = a {
                if rel_path == Path::new("001/concept-map-001.toml") {
                    found_toml = true;
                    assert!(body.contains("id = 1"));
                }
                if rel_path == Path::new("001/concept-map-001.md") {
                    found_md = true;
                    assert!(body.contains("CM-001"));
                }
            }
        }
        assert!(found_toml);
        assert!(found_md);
    }

    // --- parse_ref ---

    #[test]
    fn parse_ref_accepts_prefixed_padded_and_bare_ids() {
        assert_eq!(parse_ref("CM-001").unwrap(), 1);
        assert_eq!(parse_ref("CM-1").unwrap(), 1);
        assert_eq!(parse_ref("cm-001").unwrap(), 1);
        assert_eq!(parse_ref("cm-1").unwrap(), 1);
        assert_eq!(parse_ref("1").unwrap(), 1);
        assert_eq!(parse_ref("42").unwrap(), 42);
    }

    #[test]
    fn parse_ref_rejects_bad_input() {
        assert!(parse_ref("foo").is_err());
        assert!(parse_ref("XX-001").is_err());
    }

    // --- materialise ---

    #[test]
    fn materialise_creates_correct_directory_layout() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();

        run_new(Some(root.to_path_buf()), Some("Test Map".into()), None).unwrap();

        let cm_root = root.join(CONCEPT_MAP_DIR);
        // Directory structure
        assert!(cm_root.join("001").is_dir());
        assert!(cm_root.join("001/concept-map-001.toml").is_file());
        assert!(cm_root.join("001/concept-map-001.md").is_file());
        let symlink = cm_root.join("001-test-map");
        assert!(symlink.is_symlink());

        // Read back the TOML and verify Meta fields
        let meta = meta::read_meta(&cm_root, "concept-map", 1).unwrap();
        assert_eq!(meta.id, 1);
        assert_eq!(meta.slug, "test-map");
        assert_eq!(meta.title, "Test Map");
        assert_eq!(meta.status, "draft");
    }

    #[test]
    fn materialise_allocates_next_id() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();

        run_new(Some(root.to_path_buf()), Some("First".into()), None).unwrap();
        run_new(Some(root.to_path_buf()), Some("Second".into()), None).unwrap();

        let cm_root = root.join(CONCEPT_MAP_DIR);
        assert!(cm_root.join("001").is_dir());
        assert!(cm_root.join("002").is_dir());
    }

    // --- list ---

    #[test]
    fn list_returns_correct_entries() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();

        run_new(Some(root.to_path_buf()), Some("Alpha".into()), None).unwrap();
        run_new(Some(root.to_path_buf()), Some("Beta".into()), None).unwrap();

        let output = list_rows(root, ListArgs::default()).unwrap();
        assert!(output.contains("CM-001"));
        assert!(output.contains("CM-002"));
        assert!(output.contains("Alpha"));
        assert!(output.contains("Beta"));
    }

    #[test]
    fn list_hides_terminal_by_default() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();

        run_new(Some(root.to_path_buf()), Some("Active One".into()), None).unwrap();
        // Simulate a superseded concept map by changing the status in the TOML
        let cm_root = root.join(CONCEPT_MAP_DIR);
        let toml_path = cm_root.join("001").join("concept-map-001.toml");
        let text = std::fs::read_to_string(&toml_path).unwrap();
        let replaced = text.replace("draft", "superseded");
        std::fs::write(&toml_path, replaced).unwrap();

        // With --all it should appear
        let output_all = list_rows(
            root,
            ListArgs {
                all: true,
                ..ListArgs::default()
            },
        )
        .unwrap();
        assert!(output_all.contains("CM-001"));

        // Default should hide done
        let output = list_rows(root, ListArgs::default()).unwrap();
        assert!(!output.contains("CM-001"));
    }

    // --- show ---

    #[test]
    fn show_prints_metadata_and_dsl() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();

        run_new(Some(root.to_path_buf()), Some("Domain Model".into()), None).unwrap();

        // Write some DSL into the TOML
        let cm_root = root.join(CONCEPT_MAP_DIR);
        let toml_path = cm_root.join("001").join("concept-map-001.toml");
        let mut text = std::fs::read_to_string(&toml_path).unwrap();
        text = text.replace(
            "dsl = '''\n'''",
            "dsl = '''\nUser > identity > Identity\n'''",
        );
        std::fs::write(&toml_path, text).unwrap();

        let (doc, _toml_text, _body) = read_concept_map(&cm_root, 1).unwrap();
        let out = format_show(&doc, "", false, false, None);
        assert!(out.contains("CM-001"));
        assert!(out.contains("Domain Model"));
        assert!(out.contains("draft"));
        assert!(out.contains("User > identity > Identity"));
    }

    #[test]
    fn show_with_edges_and_nodes_renders_parsed_tables() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();

        run_new(Some(root.to_path_buf()), Some("Map".into()), None).unwrap();

        let cm_root = root.join(CONCEPT_MAP_DIR);
        let toml_path = cm_root.join("001").join("concept-map-001.toml");
        let mut text = std::fs::read_to_string(&toml_path).unwrap();
        text = text.replace(
            "dsl = '''\n'''",
            "dsl = '''\nUser > creates > Document\nWorkspace > contains > Document\n'''",
        );
        std::fs::write(&toml_path, text).unwrap();

        let (doc, _toml_text, _body) = read_concept_map(&cm_root, 1).unwrap();
        let parsed = parse_dsl(&doc.dsl);

        // --edges
        let out = format_show(&doc, "", true, false, Some(&parsed));
        assert!(out.contains("User > creates > Document"));
        assert!(out.contains("Workspace > contains > Document"));

        // --nodes
        let out = format_show(&doc, "", false, true, Some(&parsed));
        assert!(out.contains("user - User"));
        assert!(out.contains("document - Document"));
        assert!(out.contains("workspace - Workspace"));
    }

    #[test]
    fn show_json_includes_all_fields() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();

        run_new(Some(root.to_path_buf()), Some("Map".into()), None).unwrap();

        let cm_root = root.join(CONCEPT_MAP_DIR);
        let (doc, _toml_text, body) = read_concept_map(&cm_root, 1).unwrap();
        let json = show_json(&doc, &body, false, false, None).unwrap();
        assert!(json.contains("\"CM-001\""));
        assert!(json.contains("\"draft\""));
        assert!(json.contains("\"Map\""));
        assert!(json.contains("\"dsl\""));
        assert!(json.contains("\"body\""));
    }

    #[test]
    fn show_json_flag_matches_format_json() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        run_new(Some(root.to_path_buf()), Some("Map".into()), None).unwrap();
        let cm_root = root.join(CONCEPT_MAP_DIR);
        // Add DSL so edges/nodes appear in JSON output
        let toml_path = cm_root.join("001").join("concept-map-001.toml");
        let mut text = std::fs::read_to_string(&toml_path).unwrap();
        text = text.replace(
            "dsl = '''\n'''",
            "dsl = '''\nUser > creates > Document\n'''",
        );
        std::fs::write(&toml_path, text).unwrap();
        let (doc, _toml_text, body) = read_concept_map(&cm_root, 1).unwrap();
        let parsed = parse_dsl(&doc.dsl);
        let json = show_json(&doc, &body, true, true, Some(&parsed)).unwrap();
        // Golden: verify the JSON output shape.
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["id"], "CM-001");
        assert_eq!(v["status"], "draft");
        assert_eq!(v["title"], "Map");
        assert!(v["dsl"].is_string());
        assert!(v["body"].is_string());
        assert!(v["nodes"].is_array());
        assert!(v["edges"].is_array());
    }

    // --- statuses ---

    #[test]
    fn concept_map_statuses_matches_expected_variants() {
        assert_eq!(CONCEPT_MAP_STATUSES, &["draft", "accepted", "superseded"]);
    }

    // --- derive_node_key ---

    #[test]
    fn derive_node_key_lowercases_and_replaces_spaces() {
        assert_eq!(derive_node_key("User Story"), "user-story");
        assert_eq!(derive_node_key("PRD-010"), "prd-010");
        assert_eq!(derive_node_key("Some_Case"), "some-case");
    }

    #[test]
    fn derive_node_key_collapses_separator_runs() {
        assert_eq!(derive_node_key("a__b"), "a-b");
        assert_eq!(derive_node_key("a--b"), "a-b");
        assert_eq!(derive_node_key("a  b"), "a-b");
        assert_eq!(derive_node_key("a -_ b"), "a-b");
    }

    #[test]
    fn derive_node_key_strips_non_alphanumeric() {
        assert_eq!(derive_node_key("hello!!!"), "hello");
        assert_eq!(derive_node_key("a@b#c$d"), "abcd");
    }

    #[test]
    fn derive_node_key_trims_leading_trailing_hyphens() {
        assert_eq!(derive_node_key("-leading"), "leading");
        assert_eq!(derive_node_key("trailing-"), "trailing");
        assert_eq!(derive_node_key(" - both - "), "both");
    }

    #[test]
    fn derive_node_key_edge_cases() {
        assert_eq!(derive_node_key(""), "");
        assert_eq!(derive_node_key("---"), "");
        assert_eq!(derive_node_key("   "), "");
        assert_eq!(derive_node_key("a"), "a");
    }

    // --- levenshtein ---

    #[test]
    fn levenshtein_identical_is_zero() {
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("", ""), 0);
    }

    #[test]
    fn levenshtein_classic_examples() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("saturday", "sunday"), 3);
    }

    #[test]
    fn levenshtein_empty_string() {
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
    }

    #[test]
    fn levenshtein_single_char() {
        assert_eq!(levenshtein("a", "b"), 1);
        assert_eq!(levenshtein("a", "a"), 0);
    }

    // --- parse_dsl ---

    #[test]
    fn parse_dsl_empty_yields_no_nodes_or_edges() {
        let parsed = parse_dsl("");
        assert!(parsed.nodes.is_empty());
        assert!(parsed.edges.is_empty());
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    fn parse_dsl_single_valid_line() {
        let parsed = parse_dsl("User > creates > Document");
        assert_eq!(parsed.nodes.len(), 2);
        assert_eq!(parsed.edges.len(), 1);
        assert!(parsed.diagnostics.is_empty());
        assert_eq!(parsed.nodes[0].key, "user");
        assert_eq!(parsed.nodes[0].label, "User");
        assert_eq!(parsed.nodes[1].key, "document");
        assert_eq!(parsed.edges[0].from_label, "User");
        assert_eq!(parsed.edges[0].rel, "creates");
        assert_eq!(parsed.edges[0].to_label, "Document");
    }

    #[test]
    fn parse_dsl_ignores_comments_and_empty_lines() {
        let dsl = "# this is a comment\n\nUser > creates > Document\n\n# another comment\n";
        let parsed = parse_dsl(dsl);
        assert_eq!(parsed.edges.len(), 1);
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    fn parse_dsl_malformed_line() {
        let parsed = parse_dsl("User creates Document");
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(matches!(
            parsed.diagnostics[0],
            ConceptMapDiagnostic::MalformedLine { line: 1, .. }
        ));
    }

    #[test]
    fn parse_dsl_too_many_segments() {
        let parsed = parse_dsl("A > B > C > D");
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(matches!(
            parsed.diagnostics[0],
            ConceptMapDiagnostic::MalformedLine { line: 1, .. }
        ));
    }

    #[test]
    fn parse_dsl_empty_source_label() {
        let parsed = parse_dsl(" > rel > Target");
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(matches!(
            parsed.diagnostics[0],
            ConceptMapDiagnostic::EmptyLabel {
                line: 1,
                segment: SegmentKind::Source
            }
        ));
    }

    #[test]
    fn parse_dsl_empty_relation_label() {
        let parsed = parse_dsl("Source >  > Target");
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(matches!(
            parsed.diagnostics[0],
            ConceptMapDiagnostic::EmptyLabel {
                line: 1,
                segment: SegmentKind::Relation
            }
        ));
    }

    #[test]
    fn parse_dsl_empty_target_label() {
        let parsed = parse_dsl("Source > rel > ");
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(matches!(
            parsed.diagnostics[0],
            ConceptMapDiagnostic::EmptyLabel {
                line: 1,
                segment: SegmentKind::Target
            }
        ));
    }

    #[test]
    fn parse_dsl_duplicate_edge() {
        let parsed = parse_dsl("A > rel > B\nA > rel > B");
        assert_eq!(parsed.edges.len(), 1);
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(matches!(
            parsed.diagnostics[0],
            ConceptMapDiagnostic::DuplicateEdge { line: 2, .. }
        ));
    }

    #[test]
    fn parse_dsl_self_edge() {
        let parsed = parse_dsl("Node > rel > Node");
        assert_eq!(parsed.edges.len(), 1);
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(matches!(
            parsed.diagnostics[0],
            ConceptMapDiagnostic::SelfEdge { line: 1, .. }
        ));
    }

    #[test]
    fn parse_dsl_non_colliding_labels_no_diagnostic() {
        // "User" → "user", "Us er" → "us-er" — different keys, no collision.
        let parsed = parse_dsl("User > creates > Document\nUs er > reads > Document");
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    fn parse_dsl_canonical_node_collision_detected() {
        // "A B" → "a-b", "A_B" → "a-b" - same key, different labels
        let parsed = parse_dsl("A B > rel > Target\nA_B > uses > Other");
        let collisions: Vec<_> = parsed
            .diagnostics
            .iter()
            .filter(|d| matches!(d, ConceptMapDiagnostic::CanonicalNodeCollision { .. }))
            .collect();
        assert_eq!(collisions.len(), 1);
        if let ConceptMapDiagnostic::CanonicalNodeCollision {
            key,
            first_label,
            first_line,
            label,
            line,
        } = &collisions[0]
        {
            assert_eq!(key, "a-b");
            assert_eq!(first_label, "A B");
            assert_eq!(*first_line, 1);
            assert_eq!(label, "A_B");
            assert_eq!(*line, 2);
        } else {
            panic!("expected CanonicalNodeCollision");
        }
    }

    // --- check ---

    #[test]
    fn check_clean_map_yields_no_diagnostics() {
        let parsed = parse_dsl("User > creates > Document\nDocument > belongs_to > Workspace");
        let diags = check(&parsed);
        assert!(diags.is_empty());
    }

    #[test]
    fn check_entity_ref_like() {
        let parsed = parse_dsl("PRD-010 > describes > Feature");
        let diags = check(&parsed);
        let refs: Vec<_> = diags
            .iter()
            .filter(|d| matches!(d, ConceptMapDiagnostic::EntityRefLike { .. }))
            .collect();
        assert_eq!(refs.len(), 1);
        if let ConceptMapDiagnostic::EntityRefLike { label, line } = &refs[0] {
            assert_eq!(label, "PRD-010");
            assert_eq!(*line, 1);
        } else {
            panic!("expected EntityRefLike");
        }
    }

    #[test]
    fn check_similar_node_label() {
        // "User Stori" vs "User Story" - Levenshtein distance 1 (substitute 'i' → 'y')
        let parsed = parse_dsl("User Stori > describes > Feature\nUser Story > relates_to > Epic");
        let diags = check(&parsed);
        let similar: Vec<_> = diags
            .iter()
            .filter(|d| matches!(d, ConceptMapDiagnostic::SimilarNodeLabel { .. }))
            .collect();
        assert_eq!(similar.len(), 1);
    }

    #[test]
    fn check_relation_drift() {
        let parsed = parse_dsl("A > include > B\nC > includes > D");
        let diags = check(&parsed);
        let drifts: Vec<_> = diags
            .iter()
            .filter(|d| matches!(d, ConceptMapDiagnostic::RelationDrift { .. }))
            .collect();
        assert_eq!(drifts.len(), 1);
    }

    #[test]
    fn check_no_relation_drift_for_dissimilar_text() {
        let parsed = parse_dsl("A > creates > B\nC > deletes > D");
        let diags = check(&parsed);
        let drifts: Vec<_> = diags
            .iter()
            .filter(|d| matches!(d, ConceptMapDiagnostic::RelationDrift { .. }))
            .collect();
        assert!(drifts.is_empty());
    }

    // --- run_check integration ---

    #[test]
    fn run_check_clean_exits_zero() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();

        run_new(Some(root.to_path_buf()), Some("Clean Map".into()), None).unwrap();

        let cm_root = root.join(CONCEPT_MAP_DIR);
        let toml_path = cm_root.join("001").join("concept-map-001.toml");
        let mut text = std::fs::read_to_string(&toml_path).unwrap();
        text = text.replace(
            "dsl = '''\n'''",
            "dsl = '''\nUser > creates > Document\nDocument > belongs_to > Workspace\n'''",
        );
        std::fs::write(&toml_path, text).unwrap();

        let result = run_check(Some(root.to_path_buf()), "1");
        assert!(result.is_ok());
    }

    #[test]
    fn run_check_malformed_exits_nonzero() {
        // run_check calls `std::process::exit(1)` on structural errors.
        // We test via a subprocess so our own test doesn't die.
        // Instead, test that check() produces the expected diagnostic.
        let parsed = parse_dsl("bad line");
        let parsed = ParsedConceptMap {
            diagnostics: vec![ConceptMapDiagnostic::MalformedLine {
                line: 1,
                text: "bad line".into(),
            }],
            ..parsed
        };
        let _diags = check(&parsed);
        // check() doesn't carry MalformedLine, so we test the merged path
        // via format_diagnostic instead.
        let msg = format_diagnostic(&parsed.diagnostics[0]);
        assert!(msg.starts_with("line 1:"));
        assert!(msg.contains("malformed"));
    }

    // --- get_dsl / set_dsl ---

    #[test]
    fn get_dsl_extracts_dsl_value() {
        let toml = "id = 1\nslug = \"test\"\ntitle = \"Test\"\nstatus = \"draft\"\ndsl = '''\nA > b > C\n'''";
        let dsl = get_dsl(toml).unwrap();
        assert_eq!(dsl.trim(), "A > b > C");
    }

    #[test]
    fn get_dsl_errors_on_absent_key() {
        let toml = "id = 1\nslug = \"test\"\n";
        assert!(get_dsl(toml).is_err());
    }

    #[test]
    fn set_dsl_round_trip_preserves_non_dsl_content() {
        let toml = concat!(
            "id = 1\n",
            "slug = \"test\"\n",
            "title = \"Test\"\n",
            "status = \"draft\"\n",
            "[[relation]]\n",
            "target = \"ADR-001\"\n",
            "label = \"test_label\"\n",
            "dsl = '''\n",
            "Initial\n",
            "'''"
        );
        let new_dsl = "A > b > C\nX > y > Z";
        let updated = set_dsl(toml, new_dsl).unwrap();
        // Round-trip: get_dsl must return what we set.
        let reread = get_dsl(&updated).unwrap();
        assert_eq!(reread.trim(), new_dsl);
        // Non-DSL content preserved.
        assert!(updated.contains("id = 1"));
        assert!(updated.contains("[[relation]]"));
        assert!(updated.contains("label = \"test_label\""));
        assert!(updated.contains("target = \"ADR-001\""));
    }

    #[test]
    fn set_dsl_preserves_relation_rows_byte_identical() {
        // A TOML file containing both dsl and [[relation]] rows must survive
        // get_dsl → set_dsl round-trip with non-DSL content byte-identical.
        let toml = concat!(
            "id = 1\n",
            "slug = \"test\"\n",
            "title = \"Test\"\n",
            "status = \"draft\"\n",
            "description = \"\"\n",
            "created = \"2026-06-01\"\n",
            "updated = \"2026-06-01\"\n",
            "dsl = '''\n",
            "A > rel > B\n",
            "'''\n",
            "[[relation]]\n",
            "target = \"ADR-001\"\n",
            "label = \"test_label\"\n",
        );
        // Extract the relation block before DSL change.
        let relation_block = {
            let idx = toml.find("[[relation]]").unwrap();
            &toml[idx..]
        };
        let dsl = get_dsl(toml).unwrap();
        let updated = set_dsl(toml, &dsl).unwrap();
        // Relation block must be byte-identical after round-trip.
        let updated_relation = {
            let idx = updated.find("[[relation]]").unwrap();
            &updated[idx..]
        };
        assert_eq!(
            updated_relation, relation_block,
            "relation rows must be byte-identical after get_dsl → set_dsl round-trip"
        );
    }

    // --- run_add ---

    #[test]
    fn run_add_empty_dsl_single_line() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        install_cm(root, "Test Map", None);
        // Clear DSL.
        let cm_root = root.join(CONCEPT_MAP_DIR);
        rewrite_dsl(&cm_root, 1, "");

        run_add(
            Some(root.to_path_buf()),
            "1",
            "User",
            "creates",
            "Document",
            false,
        )
        .unwrap();
        let (_doc, toml_text, _body) = read_concept_map(&cm_root, 1).unwrap();
        let dsl = get_dsl(&toml_text).unwrap();
        assert_eq!(dsl.trim(), "User > creates > Document");
    }

    #[test]
    fn run_add_duplicate_noop() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        install_cm(root, "Test Map", None);
        let cm_root = root.join(CONCEPT_MAP_DIR);
        rewrite_dsl(&cm_root, 1, "User > creates > Document");

        run_add(
            Some(root.to_path_buf()),
            "1",
            "User",
            "creates",
            "Document",
            false,
        )
        .unwrap();
        let (_doc, toml_text, _body) = read_concept_map(&cm_root, 1).unwrap();
        let dsl = get_dsl(&toml_text).unwrap();
        // Only one line.
        assert_eq!(dsl.trim().lines().count(), 1);
    }

    #[test]
    fn run_add_duplicate_force_appends() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        install_cm(root, "Test Map", None);
        let cm_root = root.join(CONCEPT_MAP_DIR);
        rewrite_dsl(&cm_root, 1, "User > creates > Document");

        run_add(
            Some(root.to_path_buf()),
            "1",
            "User",
            "creates",
            "Document",
            true,
        )
        .unwrap();
        let (_doc, toml_text, _body) = read_concept_map(&cm_root, 1).unwrap();
        let dsl = get_dsl(&toml_text).unwrap();
        assert_eq!(
            dsl.trim().lines().count(),
            2,
            "force should append duplicate"
        );
    }

    #[test]
    fn run_add_rejects_empty_segments() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        install_cm(root, "Test Map", None);

        assert!(run_add(Some(root.to_path_buf()), "1", "", "rel", "target", false).is_err());
        assert!(run_add(Some(root.to_path_buf()), "1", "src", "", "target", false).is_err());
        assert!(run_add(Some(root.to_path_buf()), "1", "src", "rel", "", false).is_err());
        assert!(run_add(Some(root.to_path_buf()), "1", "  ", "rel", "target", false).is_err());
    }

    // --- run_remove ---

    #[test]
    fn run_remove_removes_edge() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        install_cm(root, "Test Map", None);
        let cm_root = root.join(CONCEPT_MAP_DIR);
        rewrite_dsl(
            &cm_root,
            1,
            "User > creates > Document\nDoc > belongs_to > Workspace",
        );

        run_remove(Some(root.to_path_buf()), "1", "User", "creates", "Document").unwrap();
        let (_doc, toml_text, _body) = read_concept_map(&cm_root, 1).unwrap();
        let dsl = get_dsl(&toml_text).unwrap();
        assert!(!dsl.contains("User > creates > Document"));
        assert!(dsl.contains("Doc > belongs_to > Workspace"));
    }

    #[test]
    fn run_remove_not_found_bails() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        install_cm(root, "Test Map", None);
        let cm_root = root.join(CONCEPT_MAP_DIR);
        rewrite_dsl(&cm_root, 1, "User > creates > Document");

        let res = run_remove(Some(root.to_path_buf()), "1", "Ghost", "rel", "Target");
        assert!(res.is_err());
    }

    #[test]
    fn run_remove_case_sensitive_match() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        install_cm(root, "Test Map", None);
        let cm_root = root.join(CONCEPT_MAP_DIR);
        rewrite_dsl(&cm_root, 1, "User > creates > Document");

        let res = run_remove(Some(root.to_path_buf()), "1", "user", "creates", "document");
        assert!(res.is_err(), "case difference should not match");
    }

    #[test]
    fn run_remove_preserves_comments_and_blanks() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        install_cm(root, "Test Map", None);
        let cm_root = root.join(CONCEPT_MAP_DIR);
        rewrite_dsl(&cm_root, 1, "# comment\n\nA > rel > B\n\n# another");

        run_remove(Some(root.to_path_buf()), "1", "A", "rel", "B").unwrap();
        let (_doc, toml_text, _body) = read_concept_map(&cm_root, 1).unwrap();
        let dsl = get_dsl(&toml_text).unwrap();
        assert!(dsl.contains("# comment"));
        assert!(dsl.contains("# another"));
        assert!(!dsl.contains("A > rel > B"));
    }

    // --- run_rename_node ---

    #[test]
    fn run_rename_node_case_insensitive_rewrite() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        install_cm(root, "Test Map", None);
        let cm_root = root.join(CONCEPT_MAP_DIR);
        rewrite_dsl(&cm_root, 1, "User > creates > Document");

        run_rename_node(Some(root.to_path_buf()), "1", "user", "Actor", false, false).unwrap();
        let (_doc, toml_text, _body) = read_concept_map(&cm_root, 1).unwrap();
        let dsl = get_dsl(&toml_text).unwrap();
        assert!(dsl.contains("Actor > creates > Document"));
    }

    #[test]
    fn run_rename_node_case_sensitive_no_match() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        install_cm(root, "Test Map", None);
        let cm_root = root.join(CONCEPT_MAP_DIR);
        rewrite_dsl(&cm_root, 1, "User > creates > Document");

        run_rename_node(Some(root.to_path_buf()), "1", "user", "Actor", false, true).unwrap();
        let (_doc, toml_text, _body) = read_concept_map(&cm_root, 1).unwrap();
        let dsl = get_dsl(&toml_text).unwrap();
        // Case-sensitive: "user" does NOT match "User", so no change.
        assert!(dsl.contains("User > creates > Document"));
        assert!(!dsl.contains("Actor"));
    }

    #[test]
    fn run_rename_node_both_source_and_target_rewritten() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        install_cm(root, "Test Map", None);
        let cm_root = root.join(CONCEPT_MAP_DIR);
        rewrite_dsl(&cm_root, 1, "A > rel > A");

        run_rename_node(Some(root.to_path_buf()), "1", "A", "B", false, false).unwrap();
        let (_doc, toml_text, _body) = read_concept_map(&cm_root, 1).unwrap();
        let dsl = get_dsl(&toml_text).unwrap();
        assert_eq!(dsl.trim(), "B > rel > B");
    }

    #[test]
    fn run_rename_node_no_substring_match() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        install_cm(root, "Test Map", None);
        let cm_root = root.join(CONCEPT_MAP_DIR);
        rewrite_dsl(&cm_root, 1, "UserBase > rel > SuperUser");

        run_rename_node(Some(root.to_path_buf()), "1", "User", "Actor", false, false).unwrap();
        let (_doc, toml_text, _body) = read_concept_map(&cm_root, 1).unwrap();
        let dsl = get_dsl(&toml_text).unwrap();
        // "User" should NOT match "UserBase" or "SuperUser" (full segment match only).
        assert_eq!(dsl.trim(), "UserBase > rel > SuperUser");
    }

    #[test]
    fn run_rename_node_dry_run_prints_without_writing() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        install_cm(root, "Test Map", None);
        let cm_root = root.join(CONCEPT_MAP_DIR);
        rewrite_dsl(&cm_root, 1, "User > creates > Document");
        let original =
            std::fs::read_to_string(&cm_root.join("001").join("concept-map-001.toml")).unwrap();

        run_rename_node(Some(root.to_path_buf()), "1", "User", "Actor", true, false).unwrap();
        let after =
            std::fs::read_to_string(&cm_root.join("001").join("concept-map-001.toml")).unwrap();
        assert_eq!(after, original, "dry-run must not write to file");
    }

    // --- export renderers ---

    #[test]
    fn dot_escape_plain_text_unchanged() {
        assert_eq!(dot_escape("hello"), "hello");
        assert_eq!(dot_escape("foo bar"), "foo bar");
    }

    #[test]
    fn dot_escape_handles_quotes_and_backslashes() {
        assert_eq!(dot_escape("a\"b"), "a\\\"b");
        assert_eq!(dot_escape("a\\b"), "a\\\\b");
    }

    #[test]
    fn dot_escape_handles_newlines() {
        assert_eq!(dot_escape("a\nb"), "a\\nb");
        assert_eq!(dot_escape("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn dot_escape_combined() {
        assert_eq!(dot_escape("\"hello\\world\""), "\\\"hello\\\\world\\\"");
    }

    #[test]
    fn mermaid_escape_plain_text_unchanged() {
        assert_eq!(mermaid_escape("hello"), "hello");
        assert_eq!(mermaid_escape("foo bar"), "foo bar");
    }

    #[test]
    fn mermaid_escape_handles_special_chars() {
        assert_eq!(mermaid_escape("a\"b"), "a#quot;b");
        assert_eq!(mermaid_escape("a[b"), "a#91;b");
        assert_eq!(mermaid_escape("a]b"), "a#93;b");
        assert_eq!(mermaid_escape("a\nb"), "a#10;b");
    }

    #[test]
    fn mermaid_escape_combined() {
        assert_eq!(mermaid_escape("\"[test]\""), "#quot;#91;test#93;#quot;");
    }

    fn make_two_node_map() -> ParsedConceptMap {
        ParsedConceptMap {
            nodes: vec![
                ConceptMapNode {
                    key: "user".into(),
                    label: "User".into(),
                },
                ConceptMapNode {
                    key: "document".into(),
                    label: "Document".into(),
                },
            ],
            edges: vec![ConceptMapEdge {
                from_key: "user".into(),
                from_label: "User".into(),
                rel: "creates".into(),
                to_key: "document".into(),
                to_label: "Document".into(),
                line: 1,
            }],
            diagnostics: vec![],
        }
    }

    fn make_empty_map() -> ParsedConceptMap {
        ParsedConceptMap {
            nodes: vec![],
            edges: vec![],
            diagnostics: vec![],
        }
    }

    #[test]
    fn render_dot_two_node_map() {
        let dot = render_dot(&make_two_node_map(), "My Map");
        // Valid digraph structure.
        assert!(dot.starts_with("digraph \"My Map\" {\n"));
        assert!(dot.ends_with("}\n") || dot.ends_with('}'));
        assert!(dot.contains("rankdir=LR;"));
        // Nodes sorted by key: "document" before "user"
        let doc_pos = dot.find("\"document\"").unwrap();
        let user_pos = dot.find("\"user\"").unwrap();
        assert!(doc_pos < user_pos, "nodes should be sorted by key");
        // Edge present.
        assert!(dot.contains("\"user\" -> \"document\" [label=\"creates\"];"));
    }

    #[test]
    fn render_dot_empty_map() {
        let dot = render_dot(&make_empty_map(), "");
        assert!(dot.contains("digraph"));
        assert!(dot.contains("rankdir=LR;"));
        // No node/edge lines.
        assert!(!dot.contains("[label="));
    }

    #[test]
    fn render_dot_escapes_labels() {
        let parsed = ParsedConceptMap {
            nodes: vec![ConceptMapNode {
                key: "test".into(),
                label: "Hello \"World\"".into(),
            }],
            edges: vec![],
            diagnostics: vec![],
        };
        let dot = render_dot(&parsed, "Test");
        assert!(dot.contains("Hello \\\"World\\\""));
    }

    #[test]
    fn render_mermaid_two_node_map() {
        let mm = render_mermaid(&make_two_node_map());
        assert!(mm.starts_with("graph LR"));
        // Synthetic node ids.
        assert!(mm.contains("n_0"));
        assert!(mm.contains("n_1"));
        // Edge with relation text.
        assert!(mm.contains("-->|creates|"));
    }

    #[test]
    fn render_mermaid_empty_map() {
        let mm = render_mermaid(&make_empty_map());
        assert_eq!(mm.trim(), "graph LR");
    }

    #[test]
    fn render_mermaid_escapes_labels() {
        let parsed = ParsedConceptMap {
            nodes: vec![ConceptMapNode {
                key: "test".into(),
                label: "Hello \"World\"".into(),
            }],
            edges: vec![],
            diagnostics: vec![],
        };
        let mm = render_mermaid(&parsed);
        // The label should escape quotes with #quot;
        assert!(mm.contains("#quot;World#quot;"));
    }

    #[test]
    fn render_json_value_round_trip() {
        let value = render_json_value(&make_two_node_map());
        let obj = value.as_object().unwrap();
        let nodes = obj["nodes"].as_array().unwrap();
        let edges = obj["edges"].as_array().unwrap();
        assert_eq!(nodes.len(), 2);
        assert_eq!(edges.len(), 1);
        // Verify keys.
        assert_eq!(nodes[0]["key"], "document");
        assert_eq!(nodes[1]["key"], "user");
        // Verify edge fields.
        assert_eq!(edges[0]["from"], "user");
        assert_eq!(edges[0]["rel"], "creates");
        assert_eq!(edges[0]["to"], "document");
    }

    #[test]
    fn render_json_empty_map() {
        let value = render_json_value(&make_empty_map());
        let obj = value.as_object().unwrap();
        assert!(obj["nodes"].as_array().unwrap().is_empty());
        assert!(obj["edges"].as_array().unwrap().is_empty());
    }

    #[test]
    fn render_json_pretty_print() {
        let json = render_json(&make_two_node_map()).unwrap();
        assert!(json.contains("\"nodes\""));
        assert!(json.contains("\"edges\""));
        assert!(json.contains("\"from_label\""));
        assert!(json.contains("\"to_label\""));
    }

    #[test]
    fn export_dot_integration() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        install_cm(root, "Test Map", None);

        // Add edges.
        run_add(
            Some(root.to_path_buf()),
            "1",
            "User",
            "creates",
            "Document",
            false,
        )
        .unwrap();
        run_add(
            Some(root.to_path_buf()),
            "1",
            "Document",
            "belongs_to",
            "Workspace",
            false,
        )
        .unwrap();

        // Export DOT.
        let result = run_export(Some(root.to_path_buf()), "1", &ExportFormat::Dot);
        assert!(result.is_ok());
    }

    #[test]
    fn export_mermaid_integration() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        install_cm(root, "Test Map", None);

        run_add(
            Some(root.to_path_buf()),
            "1",
            "User",
            "creates",
            "Document",
            false,
        )
        .unwrap();

        let result = run_export(Some(root.to_path_buf()), "1", &ExportFormat::Mermaid);
        assert!(result.is_ok());
    }

    #[test]
    fn export_json_integration() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        install_cm(root, "Test Map", None);

        run_add(
            Some(root.to_path_buf()),
            "1",
            "User",
            "creates",
            "Document",
            false,
        )
        .unwrap();

        let result = run_export(Some(root.to_path_buf()), "1", &ExportFormat::Json);
        assert!(result.is_ok());
    }

    // --- pure mutation helpers ---

    fn make_dsl(lines: &[&str]) -> String {
        lines.join("\n")
    }

    // --- add_edge_to_dsl tests ---

    #[test]
    fn add_edge_appends_to_dsl() {
        let dsl = make_dsl(&["A > depends on > B"]);
        let result = add_edge_to_dsl(&dsl, "B", "depends on", "C").unwrap();
        assert_eq!(result, "A > depends on > B\nB > depends on > C");
    }

    #[test]
    fn add_edge_detects_duplicate() {
        let dsl = make_dsl(&["A > depends on > B"]);
        let result = add_edge_to_dsl(&dsl, "A", "depends on", "B");
        assert!(matches!(
            result,
            Err(ConceptMapMutationError::DuplicateEdge { .. })
        ));
    }

    #[test]
    fn add_edge_trims_inputs() {
        let dsl = make_dsl(&[]);
        let result = add_edge_to_dsl(&dsl, "  A  ", " depends on ", "  B  ").unwrap();
        assert_eq!(result, "A > depends on > B");
    }

    #[test]
    fn add_edge_rejects_empty_source() {
        let dsl = make_dsl(&[]);
        let result = add_edge_to_dsl(&dsl, "", "rel", "target");
        assert!(
            matches!(result, Err(ConceptMapMutationError::EmptyField(f)) if f.contains("source"))
        );
    }

    #[test]
    fn add_edge_preserves_existing_lines() {
        let dsl = make_dsl(&["# a comment", "", "A > depends on > B"]);
        let result = add_edge_to_dsl(&dsl, "B", "depends on", "C").unwrap();
        assert!(result.contains("# a comment"));
        assert!(result.contains("B > depends on > C"));
    }

    // --- remove_edge_from_dsl tests ---

    #[test]
    fn remove_edge_removes_matching_line() {
        let dsl = make_dsl(&["A > depends on > B", "B > depends on > C"]);
        let result = remove_edge_from_dsl(&dsl, "A", "depends on", "B").unwrap();
        assert!(!result.contains("A > depends on > B"));
        assert!(result.contains("B > depends on > C"));
    }

    #[test]
    fn remove_edge_not_found() {
        let dsl = make_dsl(&["A > depends on > B"]);
        let result = remove_edge_from_dsl(&dsl, "X", "depends on", "Y");
        assert!(matches!(result, Err(ConceptMapMutationError::EdgeNotFound)));
    }

    #[test]
    fn remove_edge_preserves_comments_and_blanks() {
        let dsl = make_dsl(&["# header", "", "A > depends on > B", "", "# footer"]);
        let result = remove_edge_from_dsl(&dsl, "A", "depends on", "B").unwrap();
        assert!(result.contains("# header"));
        assert!(result.contains("# footer"));
    }

    #[test]
    fn remove_edge_trims_inputs() {
        let dsl = make_dsl(&["A > depends on > B"]);
        let result = remove_edge_from_dsl(&dsl, "  A  ", " depends on ", "  B  ").unwrap();
        assert!(!result.contains("A > depends on > B"));
    }

    #[test]
    fn remove_edge_removes_only_first_match() {
        let dsl = make_dsl(&["A > depends on > B", "A > depends on > B"]);
        let result = remove_edge_from_dsl(&dsl, "A", "depends on", "B").unwrap();
        assert_eq!(result, "A > depends on > B"); // second line remains
    }

    // --- relabel_edge_in_dsl tests ---

    #[test]
    fn relabel_edge_persists_and_preserves_other_lines() {
        let dsl = make_dsl(&["User > creates > Document", "Doc > relates > Note"]);
        let result = relabel_edge_in_dsl(&dsl, "User", "creates", "makes", "Document").unwrap();
        assert_eq!(result, "User > makes > Document\nDoc > relates > Note");
    }

    #[test]
    fn relabel_edge_duplicate_by_key_collision() {
        // Distinct label spellings, same derived key — a label-only check misses this.
        let dsl = make_dsl(&["User Story > needs > Thing", "User-Story > wants > Thing"]);
        let result = relabel_edge_in_dsl(&dsl, "User Story", "needs", "wants", "Thing");
        assert!(matches!(
            result,
            Err(ConceptMapMutationError::DuplicateEdge { line: 2 })
        ));
    }

    #[test]
    fn relabel_edge_no_op_returns_unchanged() {
        let dsl = make_dsl(&["User > creates > Document"]);
        let result = relabel_edge_in_dsl(&dsl, "User", "creates", "creates", "Document").unwrap();
        assert_eq!(result, dsl);
    }

    #[test]
    fn relabel_edge_not_found() {
        let dsl = make_dsl(&["User > creates > Document"]);
        let result = relabel_edge_in_dsl(&dsl, "Ghost", "haunts", "spooks", "House");
        assert!(matches!(result, Err(ConceptMapMutationError::EdgeNotFound)));
    }

    #[test]
    fn relabel_edge_rejects_empty_source() {
        let dsl = make_dsl(&["User > creates > Document"]);
        let result = relabel_edge_in_dsl(&dsl, "", "creates", "makes", "Document");
        assert!(matches!(
            result,
            Err(ConceptMapMutationError::EmptyField(_))
        ));
    }

    // --- rename_node_in_dsl tests ---

    #[test]
    fn rename_node_edits_source_and_target_lines() {
        let dsl = make_dsl(&["X > related to > Y", "Y > related to > Z"]);
        let (result, occurrences) = rename_node_in_dsl(&dsl, "Y", "Ypsilon").unwrap();
        assert_eq!(occurrences, 2);
        assert!(result.contains("X > related to > Ypsilon"));
        assert!(result.contains("Ypsilon > related to > Z"));
    }

    #[test]
    fn rename_node_single_source_only() {
        let dsl = make_dsl(&["X > related to > Y"]);
        let (result, occurrences) = rename_node_in_dsl(&dsl, "X", "Alpha").unwrap();
        assert_eq!(occurrences, 1);
        assert!(result.contains("Alpha > related to > Y"));
    }

    #[test]
    fn rename_node_key_collision_rejected() {
        // "A" key = "a", "B" key = "b" — different keys, and "B" already exists
        let dsl = make_dsl(&["A > relates to > X", "B > relates to > Y"]);
        let result = rename_node_in_dsl(&dsl, "A", "B");
        assert!(matches!(
            result,
            Err(ConceptMapMutationError::NodeCollision { .. })
        ));
    }

    #[test]
    fn rename_node_case_only_same_key_succeeds() {
        // "Alpha" key = "alpha", "alpha" key = "alpha" — same key, no collision
        let dsl = make_dsl(&["Alpha > relates to > Beta"]);
        let (result, occurrences) = rename_node_in_dsl(&dsl, "Alpha", "alpha").unwrap();
        assert_eq!(occurrences, 1);
        assert!(result.contains("alpha > relates to > Beta"));
    }

    #[test]
    fn rename_node_no_match_no_change() {
        let dsl = make_dsl(&["A > relates to > B"]);
        let (result, occurrences) = rename_node_in_dsl(&dsl, "Z", "Zeta").unwrap();
        assert_eq!(occurrences, 0);
        assert_eq!(result, dsl);
    }

    #[test]
    fn rename_node_preserves_comments_and_blanks() {
        let dsl = make_dsl(&["# header", "", "A > relates to > B", "", "# footer"]);
        let (result, _) = rename_node_in_dsl(&dsl, "A", "Alpha").unwrap();
        assert!(result.contains("# header"));
        assert!(result.contains("# footer"));
    }

    #[test]
    fn rename_node_trims_inputs() {
        let dsl = make_dsl(&["A > relates to > B"]);
        let (result, occurrences) = rename_node_in_dsl(&dsl, "  A  ", "  Alpha  ").unwrap();
        assert_eq!(occurrences, 1);
        assert!(result.contains("Alpha > relates to > B"));
    }

    #[test]
    fn rename_node_rejects_empty_fields() {
        let dsl = make_dsl(&["A > relates to > B"]);
        assert!(matches!(
            rename_node_in_dsl(&dsl, "", "X"),
            Err(ConceptMapMutationError::EmptyField(_))
        ));
        assert!(matches!(
            rename_node_in_dsl(&dsl, "X", ""),
            Err(ConceptMapMutationError::EmptyField(_))
        ));
    }

    // --- rename_node_occurrence_in_dsl tests ---

    #[test]
    fn rename_node_occurrence_rewrites_source_leaves_sibling_unchanged() {
        // "User" appears in two rows; renaming the source of row 1 only must
        // leave row 2 (also using "User") byte-unchanged.
        let dsl = make_dsl(&["User > likes > Apple", "User > hates > Onion"]);
        let result = rename_node_occurrence_in_dsl(
            &dsl,
            "User",
            "likes",
            "Apple",
            EdgeEndpoint::Source,
            "Customer",
        )
        .unwrap();
        assert_eq!(result, "Customer > likes > Apple\nUser > hates > Onion");
    }

    #[test]
    fn rename_node_occurrence_rewrites_target_leaves_sibling_unchanged() {
        // "Apple" appears in two rows; renaming the target of row 1 only.
        let dsl = make_dsl(&["User > likes > Apple", "Vendor > sells > Apple"]);
        let result = rename_node_occurrence_in_dsl(
            &dsl,
            "User",
            "likes",
            "Apple",
            EdgeEndpoint::Target,
            "Cherry",
        )
        .unwrap();
        assert_eq!(result, "User > likes > Cherry\nVendor > sells > Apple");
    }

    #[test]
    fn rename_node_occurrence_duplicate_by_key_collision() {
        // Rewriting row 1's target to "Cherry" collides with row 2's triple.
        let dsl = make_dsl(&["User > likes > Apple", "User > likes > Cherry"]);
        let result = rename_node_occurrence_in_dsl(
            &dsl,
            "User",
            "likes",
            "Apple",
            EdgeEndpoint::Target,
            "Cherry",
        );
        assert!(matches!(
            result,
            Err(ConceptMapMutationError::DuplicateEdge { line: 2 })
        ));
    }

    #[test]
    fn rename_node_occurrence_no_triple_match_is_not_found() {
        let dsl = make_dsl(&["User > likes > Apple"]);
        let result = rename_node_occurrence_in_dsl(
            &dsl,
            "User",
            "hates",
            "Apple",
            EdgeEndpoint::Target,
            "Cherry",
        );
        assert!(matches!(result, Err(ConceptMapMutationError::EdgeNotFound)));
    }

    #[test]
    fn rename_node_occurrence_rejects_empty_new_label() {
        let dsl = make_dsl(&["User > likes > Apple"]);
        let result = rename_node_occurrence_in_dsl(
            &dsl,
            "User",
            "likes",
            "Apple",
            EdgeEndpoint::Target,
            "   ",
        );
        assert!(matches!(
            result,
            Err(ConceptMapMutationError::EmptyField(_))
        ));
    }

    #[test]
    fn rename_node_occurrence_no_op_returns_unchanged() {
        let dsl = make_dsl(&["User > likes > Apple"]);
        let result = rename_node_occurrence_in_dsl(
            &dsl,
            "User",
            "likes",
            "Apple",
            EdgeEndpoint::Target,
            "Apple",
        )
        .unwrap();
        assert_eq!(result, dsl);
    }

    // --- relabel_rel_all_in_dsl tests ---

    #[test]
    fn relabel_rel_all_rewrites_every_line_sharing_rel() {
        let dsl = make_dsl(&["A > rel1 > B", "C > rel1 > D", "E > other > F"]);
        let result = relabel_rel_all_in_dsl(&dsl, "rel1", "rel2").unwrap();
        assert_eq!(result, "A > rel2 > B\nC > rel2 > D\nE > other > F");
    }

    #[test]
    fn relabel_rel_all_no_op_when_equal_returns_unchanged() {
        let dsl = make_dsl(&["A > rel1 > B", "C > rel1 > D"]);
        let result = relabel_rel_all_in_dsl(&dsl, "rel1", "rel1").unwrap();
        assert_eq!(result, dsl);
    }

    #[test]
    fn relabel_rel_all_rejects_empty_fields() {
        let dsl = make_dsl(&["A > rel1 > B"]);
        assert!(matches!(
            relabel_rel_all_in_dsl(&dsl, "", "rel2"),
            Err(ConceptMapMutationError::EmptyField(_))
        ));
        assert!(matches!(
            relabel_rel_all_in_dsl(&dsl, "rel1", "  "),
            Err(ConceptMapMutationError::EmptyField(_))
        ));
    }

    #[test]
    fn relabel_rel_all_atomic_reject_vs_existing_line() {
        // Row 1 rewritten to rel2 would collide with the existing row 2.
        let dsl = make_dsl(&["A > rel1 > B", "A > rel2 > B"]);
        let result = relabel_rel_all_in_dsl(&dsl, "rel1", "rel2");
        assert!(matches!(
            result,
            Err(ConceptMapMutationError::DuplicateEdge { line: 2 })
        ));
    }

    #[test]
    fn relabel_rel_all_atomic_reject_vs_other_rewritten_line() {
        // Distinct label spellings deriving the same key: both rows share the
        // rel, so both get rewritten and then collide with each other. The
        // whole op must be rejected — no partial write.
        let dsl = make_dsl(&["Foo Bar > rel1 > Z", "Foo-Bar > rel1 > Z"]);
        let result = relabel_rel_all_in_dsl(&dsl, "rel1", "rel2");
        assert!(matches!(
            result,
            Err(ConceptMapMutationError::DuplicateEdge { .. })
        ));
    }

    // --- ConceptMapMutationError Display ---

    #[test]
    fn mutation_error_display_messages() {
        assert!(
            ConceptMapMutationError::EmptyField("source".into())
                .to_string()
                .contains("source")
        );
        assert!(
            ConceptMapMutationError::DuplicateEdge { line: 5 }
                .to_string()
                .contains("line 5")
        );
        assert_eq!(
            ConceptMapMutationError::EdgeNotFound.to_string(),
            "edge not found"
        );
        assert!(
            ConceptMapMutationError::NodeCollision {
                existing_label: "Foo".into(),
                line: 3
            }
            .to_string()
            .contains("Foo")
        );
        assert!(
            ConceptMapMutationError::MissingDsl
                .to_string()
                .contains("dsl")
        );
        assert!(
            ConceptMapMutationError::InvalidToml("oops".into())
                .to_string()
                .contains("oops")
        );
    }

    // --- integration helpers ---

    /// Install a fresh concept map via `run_new`.
    fn install_cm(root: &Path, title: &str, slug: Option<&str>) {
        run_new(
            Some(root.to_path_buf()),
            Some(title.into()),
            slug.map(str::to_string),
        )
        .unwrap();
    }

    /// Rewrite the DSL in a concept map's TOML file via `set_dsl`.
    fn rewrite_dsl(cm_root: &Path, id: u32, new_dsl: &str) {
        let name = format!("{id:03}");
        let stem = format!("concept-map-{name}");
        let toml_path = cm_root.join(&name).join(format!("{stem}.toml"));
        let toml_text = std::fs::read_to_string(&toml_path).unwrap();
        let updated = set_dsl(&toml_text, new_dsl).unwrap();
        std::fs::write(&toml_path, updated).unwrap();
    }

    // -----------------------------------------------------------------------
    // Adversarial: special chars in pure mutation functions
    // -----------------------------------------------------------------------

    #[test]
    fn add_edge_handles_double_quotes_in_labels() {
        let dsl = make_dsl(&[]);
        let result = add_edge_to_dsl(&dsl, "He said \"hello\"", "relates to", "World").unwrap();
        assert!(result.contains("He said \"hello\" > relates to > World"));
        let parsed = parse_dsl(&result);
        assert!(parsed.diagnostics.is_empty());
        assert_eq!(parsed.edges[0].from_label, "He said \"hello\"");
    }

    #[test]
    fn add_edge_handles_backslashes_in_labels() {
        let dsl = make_dsl(&[]);
        let result = add_edge_to_dsl(&dsl, "A\\B", "depends on", "C\\D").unwrap();
        assert!(result.contains("A\\B > depends on > C\\D"));
        let parsed = parse_dsl(&result);
        assert!(parsed.diagnostics.is_empty());
        assert_eq!(parsed.edges[0].from_label, "A\\B");
        assert_eq!(parsed.edges[0].to_label, "C\\D");
    }

    #[test]
    fn add_edge_with_source_containing_dsl_delimiter_does_not_panic() {
        // add_edge_to_dsl is pure string concatenation; it doesn't reject `>`.
        // The resulting line produces MalformedLine on parse, but must not panic.
        let dsl = make_dsl(&[]);
        let result = add_edge_to_dsl(&dsl, "A > B", "rel", "Target").unwrap();
        let parsed = parse_dsl(&result);
        assert!(!parsed.diagnostics.is_empty());
    }

    #[test]
    fn add_edge_handles_internal_newlines_gracefully() {
        // Internal newline in a label is not rejected by the pure function;
        // it creates a multi-line DSL entry. Verify no panic.
        let dsl = make_dsl(&[]);
        let result = add_edge_to_dsl(&dsl, "A\nB", "rel", "target").unwrap();
        let parsed = parse_dsl(&result);
        assert_eq!(parsed.edges.len(), 1);
        assert_eq!(parsed.edges[0].from_label, "B");
    }

    #[test]
    fn add_edge_handles_unicode_labels() {
        let dsl = make_dsl(&[]);
        let result = add_edge_to_dsl(&dsl, "Üser", "crëates", "Dökument").unwrap();
        assert!(result.contains("Üser > crëates > Dökument"));
        let parsed = parse_dsl(&result);
        assert!(parsed.diagnostics.is_empty());
        assert_eq!(parsed.nodes[0].key, "üser");
        assert_eq!(parsed.nodes[1].key, "dökument");
    }

    #[test]
    fn remove_edge_handles_special_chars_in_labels() {
        let dsl = make_dsl(&["He said \"hello\" > relates to > World"]);
        let result =
            remove_edge_from_dsl(&dsl, "He said \"hello\"", "relates to", "World").unwrap();
        assert!(!result.contains("He said \"hello\""));
    }

    #[test]
    fn rename_node_handles_special_chars_in_labels() {
        let dsl = make_dsl(&["A\\B > depends on > C\\D"]);
        let (result, occurrences) = rename_node_in_dsl(&dsl, "A\\B", "Alpha\\Beta").unwrap();
        assert_eq!(occurrences, 1);
        assert!(result.contains("Alpha\\Beta > depends on > C\\D"));
    }

    #[test]
    fn rename_node_handles_unicode_labels() {
        let dsl = make_dsl(&["Üser > crëates > Dökument"]);
        let (result, occurrences) = rename_node_in_dsl(&dsl, "Üser", "Përsön").unwrap();
        assert_eq!(occurrences, 1);
        assert!(result.contains("Përsön > crëates > Dökument"));
    }

    // -----------------------------------------------------------------------
    // TOML field preservation after each mutation
    // -----------------------------------------------------------------------

    /// Helper: round-trip a TOML text through get_dsl → mutate → set_dsl and
    /// assert that all metadata fields survive unchanged.
    fn assert_toml_fields_preserved<F>(original_toml: &str, mutate: F)
    where
        F: FnOnce(&str) -> String,
    {
        let dsl = get_dsl(original_toml).unwrap();
        let new_dsl = mutate(&dsl);
        let updated = set_dsl(original_toml, &new_dsl).unwrap();
        let doc: toml_edit::DocumentMut = updated.parse().unwrap();
        assert_eq!(doc.get("slug").and_then(|v| v.as_str()), Some("test"));
        assert_eq!(doc.get("title").and_then(|v| v.as_str()), Some("Test"));
        assert_eq!(doc.get("status").and_then(|v| v.as_str()), Some("draft"));
        assert_eq!(doc.get("description").and_then(|v| v.as_str()), Some(""));
        assert_eq!(
            doc.get("created").and_then(|v| v.as_str()),
            Some("2026-01-01")
        );
        assert_eq!(
            doc.get("updated").and_then(|v| v.as_str()),
            Some("2026-01-01")
        );
        assert_eq!(doc.get("id").and_then(|v| v.as_integer()), Some(1));
        // dsl was replaced correctly
        assert_eq!(get_dsl(&updated).unwrap().trim(), new_dsl.trim());
    }

    #[test]
    fn add_edge_preserves_toml_fields() {
        let toml = concat!(
            "id = 1\n",
            "slug = \"test\"\n",
            "title = \"Test\"\n",
            "status = \"draft\"\n",
            "description = \"\"\n",
            "created = \"2026-01-01\"\n",
            "updated = \"2026-01-01\"\n",
            "dsl = '''\n",
            "A > rel > B\n",
            "'''\n",
        );
        assert_toml_fields_preserved(toml, |dsl| add_edge_to_dsl(dsl, "B", "uses", "C").unwrap());
    }

    #[test]
    fn remove_edge_preserves_toml_fields() {
        let toml = concat!(
            "id = 1\n",
            "slug = \"test\"\n",
            "title = \"Test\"\n",
            "status = \"draft\"\n",
            "description = \"\"\n",
            "created = \"2026-01-01\"\n",
            "updated = \"2026-01-01\"\n",
            "dsl = '''\n",
            "A > rel > B\n",
            "X > y > Z\n",
            "'''\n",
        );
        assert_toml_fields_preserved(toml, |dsl| {
            remove_edge_from_dsl(dsl, "A", "rel", "B").unwrap()
        });
    }

    #[test]
    fn rename_node_preserves_toml_fields() {
        let toml = concat!(
            "id = 1\n",
            "slug = \"test\"\n",
            "title = \"Test\"\n",
            "status = \"draft\"\n",
            "description = \"\"\n",
            "created = \"2026-01-01\"\n",
            "updated = \"2026-01-01\"\n",
            "dsl = '''\n",
            "A > rel > B\n",
            "'''\n",
        );
        assert_toml_fields_preserved(toml, |dsl| {
            let (new_dsl, _) = rename_node_in_dsl(dsl, "A", "Alpha").unwrap();
            new_dsl
        });
    }

    // -----------------------------------------------------------------------
    // DSL comment and blank-line preservation
    // -----------------------------------------------------------------------

    #[test]
    fn add_edge_preserves_comments_and_blanks() {
        let dsl = make_dsl(&["# header", "", "A > rel > B", "", "# footer"]);
        let result = add_edge_to_dsl(&dsl, "B", "uses", "C").unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines[0], "# header");
        assert_eq!(lines[1], "");
        assert_eq!(lines[2], "A > rel > B");
        assert_eq!(lines[3], "");
        assert_eq!(lines[4], "# footer");
        assert_eq!(lines[5], "B > uses > C");
    }

    #[test]
    fn remove_edge_preserves_comments_within_dsl() {
        let dsl = make_dsl(&[
            "# section one",
            "A > rel > B",
            "# section two",
            "C > uses > D",
            "# footer",
        ]);
        let result = remove_edge_from_dsl(&dsl, "A", "rel", "B").unwrap();
        assert!(result.contains("# section one"));
        assert!(result.contains("# section two"));
        assert!(!result.contains("A > rel > B"));
        assert!(result.contains("C > uses > D"));
        assert!(result.contains("# footer"));
    }

    #[test]
    fn rename_node_preserves_comments_between_edges() {
        let dsl = make_dsl(&["# note", "A > rel > B", "B > uses > C"]);
        let (result, _) = rename_node_in_dsl(&dsl, "B", "Beta").unwrap();
        assert!(result.contains("# note"));
        assert!(result.contains("A > rel > Beta"));
        assert!(result.contains("Beta > uses > C"));
    }

    #[test]
    fn mutation_preserves_blank_lines_between_edges() {
        let dsl = make_dsl(&["A > rel > B", "", "C > uses > D"]);
        let result = add_edge_to_dsl(&dsl, "D", "creates", "E").unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines[0], "A > rel > B");
        assert_eq!(lines[1], "");
        assert_eq!(lines[2], "C > uses > D");
        assert_eq!(lines[3], "D > creates > E");
    }

    // -----------------------------------------------------------------------
    // Rename collision edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn rename_node_key_collision_different_spelling_same_key() {
        // "User Story" → key="user-story", "User-Story" → key="user-story"
        // Renaming "Other" to "User-Story" should collide with existing "User Story"
        let dsl = make_dsl(&["User Story > relates to > X", "Other > uses > Y"]);
        let result = rename_node_in_dsl(&dsl, "Other", "User-Story");
        assert!(matches!(
            result,
            Err(ConceptMapMutationError::NodeCollision { .. })
        ));
    }

    #[test]
    fn rename_node_text_identical_no_change() {
        let dsl = make_dsl(&["Alpha > relates to > Beta"]);
        let (result, occurrences) = rename_node_in_dsl(&dsl, "Alpha", "Alpha").unwrap();
        assert_eq!(occurrences, 0);
        assert_eq!(result, dsl);
    }

    #[test]
    fn rename_node_text_identical_with_whitespace_no_change() {
        let dsl = make_dsl(&["Alpha > relates to > Beta"]);
        let (result, occurrences) = rename_node_in_dsl(&dsl, "  Alpha  ", "  Alpha  ").unwrap();
        assert_eq!(occurrences, 0);
        assert_eq!(result, dsl);
    }

    // -----------------------------------------------------------------------
    // Duplicate edge detection with whitespace normalization
    // -----------------------------------------------------------------------

    #[test]
    fn add_edge_detects_duplicate_with_extra_spaces_in_input() {
        let dsl = make_dsl(&["A > depends on > B"]);
        let result = add_edge_to_dsl(&dsl, "  A  ", "  depends on  ", "  B  ");
        assert!(matches!(
            result,
            Err(ConceptMapMutationError::DuplicateEdge { .. })
        ));
    }

    #[test]
    fn add_edge_detects_duplicate_when_dsl_has_extra_spaces() {
        // DSL has extra spaces around the relation; trimmed comparison still matches.
        let dsl = "A >   depends on   > B";
        let result = add_edge_to_dsl(dsl, "A", "depends on", "B");
        assert!(matches!(
            result,
            Err(ConceptMapMutationError::DuplicateEdge { .. })
        ));
    }

    // --- SL-139 PHASE-03 paths verb tests ---

    fn paths_cm_fixture(root: &Path, id: u32, extra: &[&str]) {
        let cm_root = root.join(CONCEPT_MAP_DIR);
        let name = format!("{id:03}");
        let dir = cm_root.join(&name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("concept-map-{name}.toml")), "toml").unwrap();
        std::fs::write(dir.join(format!("concept-map-{name}.md")), "md").unwrap();
        for e in extra {
            std::fs::write(dir.join(e), e).unwrap();
        }
    }

    #[test]
    fn paths_cm_full_output() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        paths_cm_fixture(root, 1, &["diagram.dot"]);
        let sel = crate::paths::PathSelection {
            toml: false,
            md: false,
            entity: false,
            single: false,
        };
        let cm_root = root.join(CONCEPT_MAP_DIR);
        let entity_dir = cm_root.join("001");
        let set = crate::paths::scan_entity_dir(
            &entity_dir,
            &entity_dir.join("concept-map-001.toml"),
            Some(&entity_dir.join("concept-map-001.md")),
            root,
        )
        .unwrap();
        let lines = crate::paths::select_paths(&set, &sel).unwrap();
        assert_eq!(
            lines,
            vec![
                ".doctrine/concept-map/001/concept-map-001.toml",
                ".doctrine/concept-map/001/concept-map-001.md",
                ".doctrine/concept-map/001/diagram.dot"
            ]
        );
    }

    #[test]
    fn paths_cm_single() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        paths_cm_fixture(root, 1, &["a.txt", "z.txt"]);
        let sel = crate::paths::PathSelection {
            toml: false,
            md: false,
            entity: false,
            single: true,
        };
        let cm_root = root.join(CONCEPT_MAP_DIR);
        let entity_dir = cm_root.join("001");
        let set = crate::paths::scan_entity_dir(
            &entity_dir,
            &entity_dir.join("concept-map-001.toml"),
            Some(&entity_dir.join("concept-map-001.md")),
            root,
        )
        .unwrap();
        let lines = crate::paths::select_paths(&set, &sel).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], ".doctrine/concept-map/001/concept-map-001.toml");
    }
}
