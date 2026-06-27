// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine spec` — product and technical specifications, the aggregate roots a
//! requirement is woven into.
//!
//! A spec is a numeric directory under `.doctrine/spec/<subtype>/` holding a
//! sister `spec-NNN.toml` (structured identity), a scaffolded `spec-NNN.md` prose
//! body, a `members.toml` (the requirements it members, seeded empty), and — for
//! the tech subtype only — an `interactions.toml` (outbound spec→spec edges,
//! seeded empty), with an `NNN-slug` symlink alias. Two subtypes ride two
//! `entity::Kind`s over the same kind-blind engine, each its own tree + reservation
//! namespace (design §5.1): product (`spec/product`, `PRD`) and tech (`spec/tech`,
//! `SPEC`). The subtypes diverge only in their scaffold fileset (product 3 content
//! files, tech 4) and the tech-only flat fields — D-Q5.
//!
//! This module owns the *spec-specific* parts — the two `Kind`s, their scaffolds,
//! the render fns, the parse-layer structs, and `new`/`list`/`req add`/`show`. The
//! kind-agnostic engine is `crate::entity` (unchanged — three new `Fresh` callers
//! only, R6 gate); the shared metadata-list substrate is `crate::meta`, reused
//! **additively** — `spec list` rides `read_metas` and the relocated
//! `listing::render_table` (SL-025) with zero `meta.rs` edits.
//!
//! `spec show` (PHASE-04) is the pure local reassembly that reads every parse
//! struct (`Spec`, `Member`, `Source`, `SpecStatus`, `C4Level`, `Interaction`) —
//! the last of the D-2 `dead_code` bridges erased. The only remaining later phase
//! is `validate` (PHASE-05), which reuses these readers + `requirement::load`.

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::entity::{self, Artifact, Fileset, Inputs, Kind, MaterialiseRequest, ScaffoldCtx};
use crate::listing::{self, Format, ListArgs};
use crate::meta::{self, Meta};
use crate::registry::{
    BuildFinding, DescentEdge, InteractionEdge, MemberEdge, ParentEdge, Registry,
};
use crate::requirement::{self, ReqKind, ReqStatus, Requirement};
use crate::tomlfmt::toml_string;

use std::str::FromStr;

use clap::Subcommand;

// ---------------------------------------------------------------------------
// CLI enums & dispatch (PHASE-03 relocation from main.rs)
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
pub(crate) enum SpecReqCommand {
    /// Reserve a requirement and append it to a spec as a labelled member.
    Add {
        /// Canonical spec ref: `PRD-NNN` (product) or `SPEC-NNN` (tech).
        spec_ref: String,

        /// Requirement title (prompted for if omitted).
        title: Option<String>,

        /// Requirement kind: functional | quality.
        #[arg(long)]
        kind: ReqKind,

        /// Explicit membership label (default: next free FR-/NF- for the kind).
        #[arg(long)]
        label: Option<String>,

        /// Explicit slug (default: derived from the title, bounded to a safe length).
        #[arg(long)]
        slug: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Transition a requirement's authored status (free any→any, edit-preserving).
    Status {
        /// Canonical requirement ref: `REQ-NNN` (by id only — no slug derivation).
        req_ref: String,

        /// Target status: pending | in-progress | active | deprecated | retired |
        /// superseded.
        #[arg(long)]
        to: ReqStatus,

        /// Operator note (accepted for v1; not yet stored on the requirement).
        #[arg(long)]
        note: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// List a spec's requirement members — authored roster (id, label, kind,
    /// status).
    List {
        /// Canonical spec ref: PRD-NNN | SPEC-NNN.
        spec_ref: String,

        #[command(flatten)]
        list: crate::CommonListArgs,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub(crate) enum SpecInteractionsCommand {
    /// Add an interaction edge from a tech spec to another tech spec.
    Add {
        /// Canonical spec ref: `SPEC-NNN`.
        spec_ref: String,
        /// Target canonical spec ref: `SPEC-NNN`.
        target: String,
        /// Free-text edge kind (calls / extends / uses / …).
        #[arg(long)]
        r#type: String,
        /// Optional notes.
        #[arg(long)]
        notes: Option<String>,
        /// Explicit project root.
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
    /// Remove interaction edges to a target from a tech spec.
    Remove {
        /// Canonical spec ref: `SPEC-NNN`.
        spec_ref: String,
        /// Target canonical spec ref: `SPEC-NNN`.
        target: String,
        /// Explicit project root.
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub(crate) enum SpecCommand {
    /// Allocate the next id in the subtype's namespace and scaffold a new spec.
    New {
        /// Spec subtype: product | tech.
        subtype: SpecSubtype,

        /// Spec title (prompted for if omitted).
        title: Option<String>,

        /// Explicit slug (default: derived from the title).
        #[arg(long)]
        slug: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// List specs per subtype: id, status, slug, #members.
    List {
        #[command(flatten)]
        list: crate::CommonListArgs,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Reassemble a spec into its readable whole and print it to stdout.
    Show {
        /// Canonical spec ref: `PRD-NNN` (product) or `SPEC-NNN` (tech).
        spec_ref: String,

        /// Output format (table | json).
        #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
        format: Format,

        /// Shorthand for `--format json`.
        #[arg(long)]
        json: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Check FK integrity across the corpus (or one spec): dangling member /
    /// interaction FKs, duplicate labels, and (corpus-wide) orphan requirements.
    Validate {
        /// Canonical spec ref to scope the check to (`PRD-NNN` / `SPEC-NNN`);
        /// omitted → the whole corpus (the only mode that checks for orphans).
        spec_ref: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Operate on a spec's requirements (membership).
    Req {
        #[command(subcommand)]
        command: SpecReqCommand,
    },

    /// Print the file paths of each spec entity directory.
    Paths {
        /// Spec reference(s) — `PRD-NNN` (product) or `SPEC-NNN` (tech).
        refs: Vec<String>,

        /// Show only the identity TOML file.
        #[arg(short = 't', long)]
        toml: bool,
        /// Show only the identity Markdown body.
        #[arg(short = 'm', long)]
        md: bool,
        /// Show the identity TOML + Markdown (equivalent to -t -m).
        #[arg(short = 'e', long)]
        entity: bool,
        /// Return only the first (primary) path per ref.
        #[arg(short = 's', long)]
        single: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Add or remove tech-spec interaction edges.
    Interactions {
        #[command(subcommand)]
        command: SpecInteractionsCommand,
    },

    /// Edit spec descent/parent scalar fields.
    Edit {
        #[arg(long, group = "edit-action")]
        descends_from: Option<String>,
        #[arg(long, group = "edit-action", conflicts_with = "descends_from")]
        clear_descends_from: bool,
        #[arg(long, group = "edit-action")]
        parent: Option<String>,
        #[arg(long, group = "edit-action", conflicts_with = "parent")]
        clear_parent: bool,
        spec_ref: String,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

pub(crate) fn dispatch(cmd: SpecCommand, color: bool) -> anyhow::Result<()> {
    match cmd {
        SpecCommand::New {
            subtype,
            title,
            slug,
            path,
        } => run_new(path, subtype, title, slug),
        SpecCommand::List { list, path } => run_list(path, list.into_list_args(color)),
        SpecCommand::Show {
            spec_ref,
            format,
            json,
            path,
        } => run_show(path, &spec_ref, if json { Format::Json } else { format }),
        SpecCommand::Validate { spec_ref, path } => run_validate(path, spec_ref.as_deref()),
        SpecCommand::Req { command } => match command {
            SpecReqCommand::Add {
                spec_ref,
                title,
                kind,
                label,
                slug,
                path,
            } => run_req_add(path, &spec_ref, title, kind, label, slug),
            SpecReqCommand::Status {
                req_ref,
                to,
                note,
                path,
            } => run_req_status(path, &req_ref, to, note),
            SpecReqCommand::List {
                spec_ref,
                list,
                path,
            } => run_req_list(path, &spec_ref, list.into_list_args(color)),
        },
        SpecCommand::Paths {
            refs,
            toml,
            md,
            entity,
            single,
            path,
        } => run_paths(
            path,
            &refs,
            &crate::paths::PathSelection {
                toml,
                md,
                entity,
                single,
            },
        ),
        SpecCommand::Interactions { command } => match command {
            SpecInteractionsCommand::Add {
                spec_ref,
                target,
                r#type,
                notes,
                path,
            } => run_interaction_add(path, &spec_ref, &target, &r#type, notes.as_deref()),
            SpecInteractionsCommand::Remove {
                spec_ref,
                target,
                path,
            } => run_interaction_remove(path, &spec_ref, &target),
        },
        SpecCommand::Edit {
            spec_ref,
            descends_from,
            clear_descends_from,
            parent,
            clear_parent,
            path,
        } => run_edit(
            path,
            &spec_ref,
            descends_from.as_deref(),
            clear_descends_from,
            parent.as_deref(),
            clear_parent,
        ),
    }
}

// ---------------------------------------------------------------------------
// `spec paths` — file paths for each spec entity directory
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// `spec edit` — edit descends_from / parent scalar fields
// ---------------------------------------------------------------------------

/// `doctrine spec edit <spec-ref> [--descends-from <PRD-NNN> | --clear-descends-from]
/// [--parent <ref> | --clear-parent]` — validate and set/clear spec descent/parent scalars
/// edit-preserving via `toml_edit`.
fn run_edit(
    path: Option<PathBuf>,
    spec_ref: &str,
    descends_from: Option<&str>,
    clear_descends_from: bool,
    parent: Option<&str>,
    clear_parent: bool,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (subtype, id) = resolve_spec_ref(spec_ref)?;
    let spec_dir = root.join(subtype.kind().dir).join(format!("{id:03}"));
    anyhow::ensure!(
        spec_dir.is_dir(),
        "no {} spec {spec_ref} at {}",
        subtype.label(),
        spec_dir.display()
    );

    // Read+parse spec TOML into DocumentMut ONCE.
    let toml_path = entity::id_path(&root, subtype.kind(), id, entity::Ext::Toml);
    let text = std::fs::read_to_string(&toml_path)
        .with_context(|| format!("Failed to read {}", toml_path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", toml_path.display()))?;

    // --- Validate ALL fields BEFORE any write ---

    let canonical_source = subtype.canonical_id(id);

    // descends_from set
    let canonical_descends_from: Option<String> = if let Some(df) = descends_from {
        anyhow::ensure!(
            subtype == SpecSubtype::Tech,
            "descends_from is a tech-only field; {canonical_source} is a product spec"
        );
        let (target_subtype, target_id) = resolve_spec_ref(df)?;
        let canonical = target_subtype.canonical_id(target_id);
        let rule = crate::relation::lookup(
            subtype.kind(),
            crate::relation::RelationLabel::DescendsFrom,
            None,
        )
        .context("descends_from is not a legal relation for this source")?;
        crate::relation::check_target_kind(rule, subtype.kind(), target_subtype.kind().prefix)?;
        let target_dir = root
            .join(target_subtype.kind().dir)
            .join(format!("{target_id:03}"));
        anyhow::ensure!(
            target_dir.is_dir(),
            "target spec {canonical} not found at {}",
            target_dir.display()
        );
        Some(canonical)
    } else {
        None
    };

    // clear_descends_from
    if clear_descends_from {
        anyhow::ensure!(
            subtype == SpecSubtype::Tech,
            "descends_from is a tech-only field; {canonical_source} is a product spec"
        );
    }

    // parent set
    let canonical_parent: Option<String> = if let Some(p) = parent {
        let (target_subtype, target_id) = resolve_spec_ref(p)?;
        let canonical = target_subtype.canonical_id(target_id);
        anyhow::ensure!(
            target_subtype == subtype,
            "parent target {canonical} must be the same subtype as {canonical_source} ({}), got {}",
            subtype.label(),
            target_subtype.label()
        );
        anyhow::ensure!(
            canonical != canonical_source,
            "{canonical_source} names itself as parent"
        );
        // Check kind via relation::lookup; if None (product parent undeclared),
        // inline-check target is PRD.
        if let Some(rule) =
            crate::relation::lookup(subtype.kind(), crate::relation::RelationLabel::Parent, None)
        {
            crate::relation::check_target_kind(rule, subtype.kind(), target_subtype.kind().prefix)?;
        } else {
            anyhow::ensure!(
                target_subtype == SpecSubtype::Product,
                "parent target for a product spec must be a product spec (PRD-NNN), got {canonical}"
            );
        }
        let target_dir = root
            .join(target_subtype.kind().dir)
            .join(format!("{target_id:03}"));
        anyhow::ensure!(
            target_dir.is_dir(),
            "target spec {canonical} not found at {}",
            target_dir.display()
        );

        // Acyclicity check: scan same-subtype parent values, build parent_map,
        // add edge source→target, walk from target — if source is reached,
        // it's a cycle.
        let tree = root.join(subtype.kind().dir);
        let ids = crate::entity::scan_ids(&tree)?;
        let mut parent_map: std::collections::BTreeMap<String, Option<String>> =
            std::collections::BTreeMap::new();
        for spec_id in &ids {
            if *spec_id == id {
                parent_map.insert(canonical_source.clone(), Some(canonical.clone()));
            } else {
                let spec_canon = subtype.canonical_id(*spec_id);
                if let Ok((spec, _, _)) = read_spec(subtype, &root, *spec_id) {
                    let parent_val = spec.parent.map(|raw| canonicalize_spec_ref(&raw));
                    parent_map.insert(spec_canon, parent_val);
                }
                // Skip unparseable specs — they can't participate in a cycle.
            }
        }
        let mut visited: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        let mut current = canonical.clone();
        loop {
            if !visited.insert(current.clone()) {
                anyhow::bail!(
                    "setting parent of {canonical_source} to {canonical} would create a cycle"
                );
            }
            if current == canonical_source {
                anyhow::bail!(
                    "setting parent of {canonical_source} to {canonical} would create a cycle"
                );
            }
            match parent_map.get(&current) {
                Some(Some(n)) => current = n.clone(),
                _ => break,
            }
        }

        Some(canonical)
    } else {
        None
    };

    // --- Apply changes ---
    let mut changed = false;

    if let Some(ref v) = canonical_descends_from
        && crate::dep_seq::apply_scalar(&mut doc, "descends_from", Some(v.as_str()))
    {
        changed = true;
        writeln!(io::stdout(), "Set descends_from = {v}")?;
    }
    if clear_descends_from && crate::dep_seq::apply_scalar(&mut doc, "descends_from", None) {
        changed = true;
        writeln!(io::stdout(), "Cleared descends_from")?;
    }
    if let Some(ref v) = canonical_parent
        && crate::dep_seq::apply_scalar(&mut doc, "parent", Some(v.as_str()))
    {
        changed = true;
        writeln!(io::stdout(), "Set parent = {v}")?;
    }
    if clear_parent && crate::dep_seq::apply_scalar(&mut doc, "parent", None) {
        changed = true;
        writeln!(io::stdout(), "Cleared parent")?;
    }

    if changed {
        crate::fsutil::write_atomic(&toml_path, doc.to_string().as_bytes())
            .with_context(|| format!("Failed to write {}", toml_path.display()))?;
    } else {
        writeln!(io::stdout(), "No changes (values already set)")?;
    }

    Ok(())
}

/// `doctrine spec paths <ref>…` — resolve each ref to its entity directory and
/// print the root-relative paths according to the selection.
fn run_paths(
    path: Option<PathBuf>,
    refs: &[String],
    sel: &crate::paths::PathSelection,
) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let mut all_lines: Vec<String> = Vec::new();
    for r in refs {
        let (subtype, id) = resolve_spec_ref(r)?;
        let name = format!("{id:03}");
        let entity_dir = root.join(subtype.kind().dir).join(&name);
        let toml_name = format!("{SPEC_STEM}-{name}.toml");
        let md_name = format!("{SPEC_STEM}-{name}.md");
        let set = crate::paths::scan_entity_dir(
            &entity_dir,
            &entity_dir.join(&toml_name),
            Some(&entity_dir.join(&md_name)),
            &root,
        )?;
        let lines = crate::paths::select_paths(&set, sel)?;
        all_lines.extend(lines);
    }
    write!(io::stdout(), "{}", all_lines.join("\n"))?;
    Ok(())
}

/// The toml/md file stem — shared by both subtypes (`spec-NNN.toml`). Distinct
/// from each `Kind.prefix` (`PRD`/`SPEC`) and from the tree dirs below.
pub(crate) const SPEC_STEM: &str = "spec";

/// The product subtype: light identity, `members.toml`, no interactions. Own tree
/// + reservation namespace.
pub(crate) const PRODUCT_SPEC_KIND: Kind = Kind {
    dir: ".doctrine/spec/product",
    prefix: crate::kinds::PRD,
    stem: "spec",
    scaffold: product_spec_scaffold,
};

/// The tech subtype: identity + flat fields, `members.toml` + `interactions.toml`.
/// Own tree + reservation namespace (ids independent of product's).
pub(crate) const TECH_SPEC_KIND: Kind = Kind {
    dir: ".doctrine/spec/tech",
    prefix: crate::kinds::SPEC,
    stem: "spec",
    scaffold: tech_spec_scaffold,
};

// ---------------------------------------------------------------------------
// Parse layer (entity-model tolerant-parse tier — §5.3)
// ---------------------------------------------------------------------------

/// Which spec this is. Closed set; kebab serde (round-trips the identity toml's
/// `kind`) and `clap::ValueEnum` (it is the `spec new` positional). Selects the
/// tree, prefix, and scaffold fileset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SpecSubtype {
    Product,
    Tech,
}

impl SpecSubtype {
    /// The engine `Kind` for this subtype — the single source of its tree + prefix.
    const fn kind(self) -> &'static Kind {
        match self {
            SpecSubtype::Product => &PRODUCT_SPEC_KIND,
            SpecSubtype::Tech => &TECH_SPEC_KIND,
        }
    }

    /// Embedded identity-toml template path for this subtype.
    const fn toml_template(self) -> &'static str {
        match self {
            SpecSubtype::Product => "templates/spec-product.toml",
            SpecSubtype::Tech => "templates/spec-tech.toml",
        }
    }

    /// Embedded prose template path for this subtype.
    const fn md_template(self) -> &'static str {
        match self {
            SpecSubtype::Product => "templates/spec-product.md",
            SpecSubtype::Tech => "templates/spec-tech.md",
        }
    }

    /// The canonical ref for an id in this subtype's namespace (`PRD-007` /
    /// `SPEC-012`) — the inverse of `resolve_spec_ref`, prefix from the `Kind`
    /// (single source). Used by `spec new`'s print and the registry scan.
    fn canonical_id(self, id: u32) -> String {
        listing::canonical_id(self.kind().prefix, id)
    }

    /// Human label for `spec list` section headers.
    const fn label(self) -> &'static str {
        match self {
            SpecSubtype::Product => "product",
            SpecSubtype::Tech => "tech",
        }
    }
}

/// A code anchor a tech spec governs (tech-only; `[[source]]`). Shape mirrors the
/// legacy canon `doc/spec-entity-spec.md`; superseded by SPEC-004: the language + a code identifier,
/// with an optional finer module path. Read by `spec show` render (PHASE-04).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct Source {
    pub(crate) language: String,
    pub(crate) identifier: String,
    #[serde(default)]
    pub(crate) module: Option<String>,
}

/// A spec's lifecycle status. Closed set, kebab serde; hand-edited, git is the
/// trail (no date stamps — §5.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SpecStatus {
    Draft,
    Active,
    Deprecated,
    Superseded,
}

impl SpecStatus {
    /// The kebab string for `spec show` render (matches the serde rename). Pure.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            SpecStatus::Draft => "draft",
            SpecStatus::Active => "active",
            SpecStatus::Deprecated => "deprecated",
            SpecStatus::Superseded => "superseded",
        }
    }
}

/// The C4 architectural level of a tech spec. Closed set (C6 ruling), kebab serde;
/// tech-only, optional.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum C4Level {
    Context,
    Container,
    Component,
    Code,
}

impl C4Level {
    /// The kebab string for `spec show` render (matches the serde rename). Pure.
    const fn as_str(self) -> &'static str {
        match self {
            C4Level::Context => "context",
            C4Level::Container => "container",
            C4Level::Component => "component",
            C4Level::Code => "code",
        }
    }
}

/// The product altitude of a product spec. Closed set, kebab serde; product-only,
/// optional. Mirror of `C4Level` (domain≈context, capability≈container,
/// feature≈component, story≈code). Advisory — no rank-adjacency enforced (SL-065 D2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProductLevel {
    Domain,
    Capability,
    Feature,
    Story,
}

impl ProductLevel {
    /// The kebab string for `spec show` render (matches the serde rename). Pure.
    const fn as_str(self) -> &'static str {
        match self {
            ProductLevel::Domain => "domain",
            ProductLevel::Capability => "capability",
            ProductLevel::Feature => "feature",
            ProductLevel::Story => "story",
        }
    }
}

/// The spec identity parse layer. `title` keys the shared-`Meta` convention (C2).
/// `category` is deliberately OPEN vocabulary (`Option<String>`, C6); the tech flat
/// fields default to absent/empty for a product spec. Read by `spec show` render.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct Spec {
    pub(crate) id: u32,
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) status: SpecStatus,
    pub(crate) kind: SpecSubtype,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    #[serde(default)]
    pub(crate) category: Option<String>,
    #[serde(default)]
    pub(crate) c4_level: Option<C4Level>,
    /// Product altitude (`domain|capability|feature|story`). Product-only,
    /// optional; absent on a tech or unlabelled product spec. Advisory tag — only
    /// `parent` is FK-validated (SL-065 D5). Mirror of `c4_level`.
    #[serde(default)]
    pub(crate) product_level: Option<ProductLevel>,
    #[serde(default)]
    pub(crate) responsibilities: Vec<String>,
    #[serde(default, rename = "source")]
    pub(crate) sources: Vec<Source>,
    /// Cross-family descent to the product capability this spec realises
    /// (`PRD-NNN`). Single-valued outbound (ADR-004 §1); absent on a
    /// product or unfilled tech spec. Integrity is `validate`'s job (SL-022 §5.2).
    #[serde(default)]
    pub(crate) descends_from: Option<String>,
    /// Single decomposition parent (`SPEC-NNN` or `PRD-NNN`). Single-valued
    /// outbound; the reciprocal children view is derived, never stored (§5.2).
    #[serde(default)]
    pub(crate) parent: Option<String>,
}

/// One membership row in a spec's `members.toml` — the spec→requirement edge with
/// its sticky label and advisory order. The FK is a plain canonical string
/// (`REQ-NNN`); integrity is `validate`'s job, not the type's.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct Member {
    pub(crate) requirement: String,
    pub(crate) label: String,
    pub(crate) order: u32,
}

/// A spec's `members.toml` document: the `[[member]]` array. Seeded empty by
/// `spec new`, so `#[serde(default)]` lets the comment-only seed parse to zero rows.
#[derive(Debug, Default, Deserialize)]
struct MembersDoc {
    #[serde(default)]
    member: Vec<Member>,
}

/// One outbound spec→spec edge in a tech spec's `interactions.toml`. `type` is
/// free-text per the relation schema (not an enum); the `target` FK is canonical
/// (`SPEC-NNN`). Hand-authored in v1 (no verb — D-Q4). First prod caller is `spec
/// show` render (PHASE-04 — render shows outbound interactions), not validate.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct Interaction {
    pub(crate) target: String,
    #[serde(rename = "type")]
    pub(crate) kind: String,
    #[serde(default)]
    pub(crate) notes: Option<String>,
}

/// A tech spec's `interactions.toml` document: the `[[edge]]` array (the seed's
/// array key — NOT `[[interaction]]`). Seeded empty; `#[serde(default)]` lets the
/// comment-only seed parse to zero edges.
#[derive(Debug, Default, Deserialize)]
struct InteractionsDoc {
    #[serde(default)]
    edge: Vec<Interaction>,
}

// ---------------------------------------------------------------------------
// Pure: render, scaffold
// ---------------------------------------------------------------------------

/// Render `spec-<id>.toml` from the subtype's embedded template by token
/// substitution. The `id/slug/title/status` keys round-trip into `meta::Meta`
/// (VT-2). No date fields (§5.4).
fn render_spec_toml(
    subtype: SpecSubtype,
    id: u32,
    slug: &str,
    title: &str,
) -> anyhow::Result<String> {
    Ok(crate::install::asset_text(subtype.toml_template())?
        .replace("{{id}}", &id.to_string())
        .replace("{{slug}}", &toml_string(slug))
        .replace("{{title}}", &toml_string(title)))
}

/// Render `spec-<id>.md` from the subtype's embedded prose template: `{{ref}}` (the
/// canonical id, e.g. `PRD-007`) + `{{title}}`. Metadata lives in the sister toml.
fn render_spec_md(subtype: SpecSubtype, canonical_id: &str, title: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text(subtype.md_template())?
        .replace("{{ref}}", canonical_id)
        .replace("{{title}}", title))
}

/// The seeded-empty `members.toml` body (comment-only; parses to zero members).
fn members_seed() -> anyhow::Result<String> {
    crate::install::asset_text("templates/members.toml")
}

/// The seeded-empty `interactions.toml` body (tech only; comment-only).
fn interactions_seed() -> anyhow::Result<String> {
    crate::install::asset_text("templates/interactions.toml")
}

/// The product fileset: `spec-NNN.toml`, `spec-NNN.md`, seeded `members.toml`, and
/// the `NNN-slug` symlink. No `interactions.toml` (absent, not empty — §5.4).
fn product_spec_scaffold(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
    spec_scaffold(SpecSubtype::Product, ctx)
}

/// The tech fileset: the product set plus a seeded `interactions.toml`.
fn tech_spec_scaffold(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
    spec_scaffold(SpecSubtype::Tech, ctx)
}

/// Shared scaffold body — the subtype decides the toml/md template and whether an
/// `interactions.toml` is emitted. The `NNN-slug` alias is last, mirroring
/// adr/requirement (the alias is universal across numeric entities; §5.1 lists the
/// *content* files only — F-2).
fn spec_scaffold(subtype: SpecSubtype, ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
    let id = ctx.id;
    let name = format!("{id:03}");
    let mut fileset = vec![
        Artifact::File {
            rel_path: entity::rel_path(subtype.kind(), id, entity::Ext::Toml),
            body: render_spec_toml(subtype, id, ctx.slug, ctx.title)?,
        },
        Artifact::File {
            rel_path: entity::rel_path(subtype.kind(), id, entity::Ext::Md),
            body: render_spec_md(subtype, ctx.canonical, ctx.title)?,
        },
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/members.toml")),
            body: members_seed()?,
        },
    ];
    if subtype == SpecSubtype::Tech {
        fileset.push(Artifact::File {
            rel_path: PathBuf::from(format!("{name}/interactions.toml")),
            body: interactions_seed()?,
        });
    }
    fileset.push(Artifact::Symlink {
        rel_path: PathBuf::from(format!("{name}-{}", ctx.slug)),
        target: name,
    });
    Ok(fileset)
}

/// Reassemble a spec into its readable whole (design §5.4) — the PURE compose half
/// of `spec show`. Takes already-parsed inputs and returns the document `String`;
/// touches no disk (the shell does all I/O — §8 purity thesis). Members are
/// stable-sorted by their advisory `order` here (gaps/dups cosmetic — EX-2). An
/// empty `interactions` slice omits that block entirely, covering a product spec's
/// absent file and a tech spec with zero edges uniformly (VT-3). The spec's own
/// prose body is emitted **verbatim** — never structurally parsed (D8 / storage
/// rule); per-requirement fields come from the structured toml, not their prose.
pub(crate) fn render(
    spec: &Spec,
    prose_body: &str,
    members: &[(Member, Requirement)],
    req_bodies: &[Option<String>],
    interactions: &[Interaction],
) -> String {
    let canonical_ref = spec.kind.canonical_id(spec.id);
    // House style: collect pre-formatted pieces (each carrying its own newlines)
    // and `concat()` — avoids the `push_str(&format!(…))` lint and stays pure.
    let mut parts: Vec<String> = Vec::new();

    // identity + flat fields. The identity is NOT an H1 — the verbatim prose body
    // below carries the spec's own `# <ref>: <title>` heading, so a synthetic H1
    // here would double it. This line is the authoritative structured identity
    // (title/status from the toml, which can drift from the prose H1).
    parts.push(format!("`{canonical_ref}` — {}\n", spec.title));
    parts.push(format!(
        "{} · {} · {}\n",
        spec.slug,
        spec.status.as_str(),
        spec.kind.label(),
    ));
    if !spec.tags.is_empty() {
        parts.push(format!("tags: {}\n", spec.tags.join(", ")));
    }
    if let Some(category) = &spec.category {
        parts.push(format!("category: {category}\n"));
    }
    // Post-category altitude + outbound spine (SL-022 §5.2, SL-065 §5): branch on
    // subtype so each family renders only its own axes, Some-gated. Tech output is
    // byte-identical to pre-SL-065. A product spec's `c4_level` (and a tech spec's
    // `product_level`) is an at-rest tag that falls outside its branch and is not
    // rendered (SL-065 D5/F1). Children are derived, never rendered (ADR-004 §3).
    match spec.kind {
        SpecSubtype::Tech => {
            if let Some(level) = spec.c4_level {
                parts.push(format!("c4 level: {}\n", level.as_str()));
            }
            if let Some(d) = &spec.descends_from {
                parts.push(format!("descends from: {d}\n"));
            }
            if let Some(p) = &spec.parent {
                parts.push(format!("parent: {p}\n"));
            }
        }
        SpecSubtype::Product => {
            if let Some(level) = spec.product_level {
                parts.push(format!("product level: {}\n", level.as_str()));
            }
            if let Some(p) = &spec.parent {
                parts.push(format!("parent: {p}\n"));
            }
        }
    }
    if !spec.responsibilities.is_empty() {
        parts.push("responsibilities:\n".to_string());
        for r in &spec.responsibilities {
            parts.push(format!("  - {r}\n"));
        }
    }
    if !spec.sources.is_empty() {
        parts.push("sources:\n".to_string());
        for s in &spec.sources {
            let module = match &s.module {
                Some(m) => format!(" ({m})"),
                None => String::new(),
            };
            parts.push(format!("  - {} {}{module}\n", s.language, s.identifier));
        }
    }

    // prose body, verbatim.
    parts.push("\n".to_string());
    parts.push(prose_body.to_string());
    if !prose_body.ends_with('\n') {
        parts.push("\n".to_string());
    }

    // Requirements — each member in advisory `order`, its requirement read by FK.
    parts.push("\n## Requirements\n".to_string());
    let mut indexed: Vec<(usize, &(Member, Requirement))> = members.iter().enumerate().collect();
    indexed.sort_by_key(|(_, (m, _))| m.order);
    for (i, (member, req)) in indexed {
        let req_ref = requirement::canonical_id(req.id);
        parts.push(format!(
            "\n### {} ({req_ref}) — {}\n\n",
            member.label, req.title
        ));
        parts.push(format!(
            "{} · {} · {}\n",
            req.slug,
            req.kind.as_str(),
            req.status.as_str(),
        ));
        if !req.tags.is_empty() {
            parts.push(format!("tags: {}\n", req.tags.join(", ")));
        }
        // "statement" is the structured `description` (D-P4-1): the storage rule
        // forbids parsing the requirement's prose; absent → no line.
        if let Some(statement) = &req.description {
            parts.push(format!("\n{statement}\n"));
        }
        if !req.acceptance_criteria.is_empty() {
            parts.push("\nacceptance criteria:\n".to_string());
            for c in &req.acceptance_criteria {
                parts.push(format!("  - {c}\n"));
            }
        }
        // Prose body (IMP-058): render the requirement's `.md` body verbatim when
        // authored; omit when scaffold (no noise).
        if let Some(body) = req_bodies.get(i).and_then(|b| b.as_ref()) {
            parts.push("\n".to_string());
            parts.push(body.clone());
            if !body.ends_with('\n') {
                parts.push("\n".to_string());
            }
        }
    }

    // outbound interactions (tech only; omitted when empty — VT-3).
    if !interactions.is_empty() {
        parts.push("\n## Interactions\n\n".to_string());
        for i in interactions {
            let notes = match &i.notes {
                Some(n) => format!(": {n}"),
                None => String::new(),
            };
            parts.push(format!("- {} — {}{notes}\n", i.target, i.kind));
        }
    }

    parts.concat()
}

// ---------------------------------------------------------------------------
// `#members` — list's derived column
// ---------------------------------------------------------------------------

/// Parse a spec's `members.toml` into its rows. A missing file → no members (a
/// spec always carries the seed, but tolerance keeps callers robust). Shared by
/// the `#members` column and `req add`'s label/order scan.
pub(crate) fn read_members(members_path: &Path) -> anyhow::Result<Vec<Member>> {
    let text = match std::fs::read_to_string(members_path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => {
            return Err(e).with_context(|| format!("Failed to read {}", members_path.display()));
        }
    };
    let doc: MembersDoc = toml::from_str(&text)
        .with_context(|| format!("Failed to parse {}", members_path.display()))?;
    Ok(doc.member)
}

/// Count the members of the spec rooted at `spec_dir` (`0` for the seeded-empty
/// file). The `#members` list column.
fn member_count(spec_dir: &Path) -> anyhow::Result<usize> {
    Ok(read_members(&spec_dir.join("members.toml"))?.len())
}

/// Parse a tech spec's `interactions.toml` into its outbound edges. A missing file
/// → no interactions (a product spec has none — absent, not empty; §5.4), so a
/// product spec and a tech spec with zero edges both yield `[]`. Read-only; mirrors
/// `read_members`.
pub(crate) fn read_interactions(interactions_path: &Path) -> anyhow::Result<Vec<Interaction>> {
    let text = match std::fs::read_to_string(interactions_path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => {
            return Err(e)
                .with_context(|| format!("Failed to read {}", interactions_path.display()));
        }
    };
    let doc: InteractionsDoc = toml::from_str(&text)
        .with_context(|| format!("Failed to parse {}", interactions_path.display()))?;
    Ok(doc.edge)
}

/// Read a spec's both tiers in one call — parse `spec-NNN.toml` into a [`Spec`]
/// and read `spec-NNN.md` as its prose body. Mirrors [`read_slice`]'s `(parsed,
/// raw_toml, prose_body)` signature. Errors on a missing file or malformed TOML.
pub(crate) fn read_spec(
    subtype: SpecSubtype,
    root: &Path,
    id: u32,
) -> anyhow::Result<(Spec, String, String)> {
    let toml_path = entity::id_path(root, subtype.kind(), id, entity::Ext::Toml);
    let toml_text = std::fs::read_to_string(&toml_path)
        .with_context(|| format!("Failed to read {}", toml_path.display()))?;
    let spec: Spec = toml::from_str(&toml_text)
        .with_context(|| format!("Failed to parse {}", toml_path.display()))?;
    let md_path = entity::id_path(root, subtype.kind(), id, entity::Ext::Md);
    let prose_body = std::fs::read_to_string(&md_path)
        .with_context(|| format!("Failed to read {}", md_path.display()))?;
    Ok((spec, toml_text, prose_body))
}

/// A spec's authored outbound relations (SL-046 §5.2/§5.3): the `Meta` lineage
/// Options `descends_from` → [`RelationLabel::DescendsFrom`] and `parent` →
/// [`RelationLabel::Parent`] (tech-only, absent on a product → emit nothing), the
/// `members.toml` rows → [`RelationLabel::Members`], and the tech-spec
/// `interactions.toml` `[[edge]]` rows → [`RelationLabel::Interactions`] (target =
/// the edge `target`; the per-edge free-text `type` is a SINGLE relation class here
/// and re-read from the source at render — C2/D2, never encoded in the label).
/// Members + interactions read via the existing `read_members`/`read_interactions`
/// readers; the spec toml itself is read via [`read_spec`] (IMP-037 extraction).
/// Ordering: lineage, members, interactions — each in authored order.
pub(crate) fn relation_edges(
    subtype: SpecSubtype,
    root: &Path,
    id: u32,
) -> anyhow::Result<Vec<crate::relation::RelationEdge>> {
    use crate::relation::{RelationEdge, RelationLabel};
    let (spec, spec_text, _prose_body) = read_spec(subtype, root, id)?;
    let name = format!("{id:03}");
    let spec_dir = root.join(subtype.kind().dir).join(&name);

    let mut edges = Vec::new();
    if let Some(d) = &spec.descends_from {
        edges.push(RelationEdge::new(RelationLabel::DescendsFrom, d.clone()));
    }
    if let Some(p) = &spec.parent {
        edges.push(RelationEdge::new(RelationLabel::Parent, p.clone()));
    }
    for m in read_members(&spec_dir.join("members.toml"))? {
        edges.push(RelationEdge::new(RelationLabel::Members, m.requirement));
    }
    for i in read_interactions(&spec_dir.join("interactions.toml"))? {
        edges.push(RelationEdge::new(RelationLabel::Interactions, i.target));
    }
    // SL-048 PHASE-04 (the cut): the NEW tier-1 axes (`governed_by`, `consumes`) live
    // in the uniform `[[relation]]` block, read generically. They sit AFTER the typed
    // lineage/members/interactions edges in canonical RELATION_RULES order (X1 merge
    // order — for a spec source the tier-1 labels follow the typed ones). No corpus
    // spec authors them yet, so this is additive: the emitted edge set is unchanged
    // until a `governed_by`/`consumes` row is authored (PHASE-05 `link`).
    edges.extend(crate::relation::tier1_edges(subtype.kind(), &spec_text)?);
    Ok(edges)
}

/// The per-edge free-text `type` of a tech spec's outbound `interactions` edges,
/// keyed by target ref (SL-046 §5.3 / C2 — the `inspect` render re-reads the type
/// from the SOURCE at render time; it is NOT carried in `InspectView`). Returns an
/// empty map for a product spec (no `interactions.toml`) or a tech spec with no
/// edges. Re-uses the existing `read_interactions` reader — no new TOML parse. A
/// duplicate target keeps the LAST authored type (the map is target-keyed; a spec
/// authoring two interactions to the same target is degenerate, §5.5).
pub(crate) fn interaction_types(
    root: &Path,
    id: u32,
) -> anyhow::Result<std::collections::BTreeMap<String, String>> {
    let name = format!("{id:03}");
    let dir = root.join(TECH_SPEC_KIND.dir).join(&name);
    let mut by_target = std::collections::BTreeMap::new();
    for i in read_interactions(&dir.join("interactions.toml"))? {
        by_target.insert(i.target, i.kind);
    }
    Ok(by_target)
}

// ---------------------------------------------------------------------------
// `req add` — resolver, label/order, the edit-preserving member append
// ---------------------------------------------------------------------------

/// Parse a canonical `<spec-ref>` (`PRD-NNN` / `SPEC-NNN`) into its subtype +
/// numeric id. The prefix is REQUIRED (C4): a bare numeric is ambiguous across
/// the two independent reservation namespaces, so it is rejected. Prefixes are
/// derived from the two `Kind`s — the single source — never hardcoded here.
fn resolve_spec_ref(spec_ref: &str) -> anyhow::Result<(SpecSubtype, u32)> {
    let (prefix, num) = spec_ref.rsplit_once('-').with_context(|| {
        format!("`{spec_ref}` is not a canonical spec ref (expected PRD-NNN or SPEC-NNN)")
    })?;
    let subtype = [SpecSubtype::Product, SpecSubtype::Tech]
        .into_iter()
        .find(|s| s.kind().prefix == prefix)
        .with_context(|| {
            format!("unknown spec prefix `{prefix}` in `{spec_ref}` (expected PRD or SPEC)")
        })?;
    let id: u32 = num
        .parse()
        .with_context(|| format!("`{num}` is not a numeric id in `{spec_ref}`"))?;
    Ok((subtype, id))
}

/// Canonicalise a stored spec ref for the registry, leaving an unparseable ref
/// as-is so the integrity check (`validate`) can flag it as dangling rather than
/// the scan swallowing it. The single canonicalisation path for every outbound
/// spec→spec ref harvested into the registry (interactions, parents, descents).
fn canonicalize_spec_ref(raw: &str) -> String {
    resolve_spec_ref(raw).map_or_else(|_| raw.to_string(), |(s, n)| s.canonical_id(n))
}

/// One spec→requirement membership, surfaced for the downstream coverage scan: the
/// sticky `label` carried verbatim (F-A8 — `member_reqs` is its only public-ish
/// source; `read_members` is private) and the `requirement` FK in canonical form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MemberReq {
    pub(crate) label: String,
    pub(crate) requirement: String,
}

/// The spec-fan seam: resolve `<spec-ref>`'s members, ordered by advisory `order`,
/// with every requirement FK canonicalised. The canonicalisation is load-bearing —
/// the downstream scan matches evidence by exact string against canonical `REQ-NNN`
/// keys, so a raw `REQ-1` would render observed=none, a read that lies (BLOCKER E2).
/// Mirrors `run_req_add`'s spec-dir resolution; the private `resolve_spec_ref` /
/// `read_members` stay private, with this their only `pub(crate)` exit.
pub(crate) fn member_reqs(root: &Path, spec_ref: &str) -> anyhow::Result<Vec<MemberReq>> {
    let (subtype, spec_id) = resolve_spec_ref(spec_ref)?;
    let spec_dir = root.join(subtype.kind().dir).join(format!("{spec_id:03}"));
    anyhow::ensure!(
        spec_dir.is_dir(),
        "no {} spec {spec_ref} at {}",
        subtype.label(),
        spec_dir.display()
    );
    let mut members = read_members(&spec_dir.join("members.toml"))?;
    members.sort_by_key(|m| m.order);
    Ok(members
        .into_iter()
        .map(|m| MemberReq {
            label: m.label,
            requirement: requirement::canonicalize_fk(&m.requirement),
        })
        .collect())
}

/// The source line enclosing `byte`, without its trailing newline. Used to attribute
/// a parse error to the offending key.
fn enclosing_line(src: &str, byte: usize) -> &str {
    let byte = byte.min(src.len());
    let start = src
        .get(..byte)
        .and_then(|s| s.rfind('\n'))
        .map_or(0, |i| i + 1);
    let end = src
        .get(byte..)
        .and_then(|s| s.find('\n'))
        .map_or(src.len(), |i| byte + i);
    src.get(start..end).unwrap_or("")
}

/// Classify a `Spec` parse error as a `second_parent` violation (SL-022 §5.2/§5.3,
/// codex F1/F2): a duplicate `parent` key or an array-valued `parent` — both ways of
/// declaring more than one parent for the scalar field. Attribution rides the error
/// **span**: the parser has already ignored comments, so a freshly-scaffolded spec's
/// commented `# parent = …` example can never be the span (the F2 guarantee is
/// structural, not a heuristic). The shape is then confirmed by message text
/// (toml-version-fragile, R2 — pinned by `second_parent_classifier_*` tests). Any
/// other parse error returns `false` and propagates as `Failed to parse` — a
/// degraded message, never a silent pass.
fn is_second_parent(err: &toml::de::Error, src: &str) -> bool {
    let Some(span) = err.span() else {
        return false;
    };
    let on_parent_key = enclosing_line(src, span.start)
        .trim_start()
        .split('=')
        .next()
        .map(str::trim)
        == Some("parent");
    if !on_parent_key {
        return false;
    }
    let msg = err.message();
    msg.contains("duplicate key") || msg.contains("invalid type: sequence")
}

/// The membership-label prefix for a requirement kind: `FR` (functional) / `NF`
/// (quality). The label is membership state, not requirement state (§5.3), so it
/// lives spec-side.
fn label_prefix(kind: ReqKind) -> &'static str {
    match kind {
        ReqKind::Functional => "FR",
        ReqKind::Quality => "NF",
    }
}

/// Next free `<prefix>-NNN` label for `kind` among existing members (max + 1,
/// zero-padded, first is 001). Labels of the other kind are ignored. Racy under
/// concurrent `req add` (TOCTOU); the P5 uniqueness lint is the backstop (§5.4).
fn next_label(members: &[Member], kind: ReqKind) -> String {
    let prefix = label_prefix(kind);
    let max = members
        .iter()
        .filter_map(|m| {
            m.label
                .strip_prefix(prefix)?
                .strip_prefix('-')?
                .parse::<u32>()
                .ok()
        })
        .max()
        .unwrap_or(0);
    format!("{prefix}-{:03}", max + 1)
}

/// Next `order` for a new member: max existing + 1 (empty → 1). Advisory sort key.
fn next_order(members: &[Member]) -> u32 {
    members.iter().map(|m| m.order).max().unwrap_or(0) + 1
}

/// Edit-preserving append of one `[[member]]` row to a spec's `members.toml`
/// (§5.4 step 4). A `toml_edit` array-of-tables `push` — never a serde
/// reserialize — so the seeded comment, hand-added comments, and unknown keys
/// survive; pushing a table is header-safe (unlike a trailing key insert).
/// Mirrors `adr::set_adr_status`'s parse → mutate → write shape.
fn append_member(
    members_path: &Path,
    requirement_fk: &str,
    label: &str,
    order: u32,
) -> anyhow::Result<()> {
    let text = std::fs::read_to_string(members_path)
        .with_context(|| format!("Failed to read {}", members_path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", members_path.display()))?;
    let members = doc
        .entry("member")
        .or_insert(toml_edit::Item::ArrayOfTables(
            toml_edit::ArrayOfTables::new(),
        ))
        .as_array_of_tables_mut()
        .context("`member` is not an array of tables")?;
    let mut row = toml_edit::Table::new();
    row.insert("requirement", toml_edit::value(requirement_fk));
    row.insert("label", toml_edit::value(label));
    row.insert("order", toml_edit::value(i64::from(order)));
    members.push(row);
    crate::fsutil::write_atomic(members_path, doc.to_string().as_bytes())
        .with_context(|| format!("Failed to write {}", members_path.display()))
}

// ---------------------------------------------------------------------------
// CLI entry points (thin)
// ---------------------------------------------------------------------------

/// `doctrine spec new <product|tech> "<title>" [--slug S]` — allocate the next id
/// in the subtype's namespace and scaffold its fileset. Pure mirror of
/// `adr run_new`, dispatching the `Kind` on `subtype`. Prints `PRD-NNN`/`SPEC-NNN`.
pub(crate) fn run_new(
    path: Option<PathBuf>,
    subtype: SpecSubtype,
    title: Option<String>,
    slug: Option<String>,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let trunk_ids = crate::git::trunk_entity_ids(&root, subtype.kind().dir)?;
    let (backend, mut reserved) =
        crate::reserve::backend(&root, subtype.kind().prefix, crate::install::prompt_confirm)?;
    let title = crate::input::resolve_title(title)?;
    let slug = crate::input::resolve_slug(&title, slug)?;
    let date = crate::clock::today();
    let out = entity::materialise(
        subtype.kind(),
        &*backend,
        &root,
        &MaterialiseRequest::Fresh,
        &Inputs {
            slug: &slug,
            title: &title,
            date: &date,
        },
        &trunk_ids,
        &mut reserved,
    )?;
    let id = out
        .eid
        .numeric_id()
        .context("spec kind must yield a numeric id")?;
    writeln!(
        io::stdout(),
        "Created {}: {}",
        subtype.canonical_id(id),
        out.dir.display()
    )?;
    Ok(())
}

/// `doctrine spec req add <spec-ref> "<title>" --kind <functional|quality>
/// [--label …]` — the two-tree write (§5.4). Resolve the spec (canonical ref,
/// C4); reserve a `REQ-NNN`; overwrite its seeded kind (D-1); append a membership
/// row to the spec's `members.toml`. NOT transactional by design (C5): an append
/// failure after the reserve leaves an orphan requirement (uncommitted, operator-
/// cleaned; P5 `validate` flags it hard). Pure label/order compute precedes any
/// write so the torn-write window is as tight as possible.
pub(crate) fn run_req_add(
    path: Option<PathBuf>,
    spec_ref: &str,
    title: Option<String>,
    kind: ReqKind,
    label: Option<String>,
    slug: Option<String>,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (subtype, spec_id) = resolve_spec_ref(spec_ref)?;
    let spec_dir = root.join(subtype.kind().dir).join(format!("{spec_id:03}"));
    anyhow::ensure!(
        spec_dir.is_dir(),
        "no {} spec {spec_ref} at {}",
        subtype.label(),
        spec_dir.display()
    );

    // Pure compute before any write — keeps the torn-write window minimal.
    let members_path = spec_dir.join("members.toml");
    let members = read_members(&members_path)?;
    let label = match label {
        Some(l) => l,
        None => next_label(&members, kind),
    };
    let order = next_order(&members);

    // Step 2 (§5.4): reserve the requirement — H2-atomic, collision-proof.
    let title = crate::input::resolve_title(title)?;
    let slug = crate::input::resolve_slug(&title, slug)?;
    let date = crate::clock::today();
    let reserved = requirement::reserve(&root, &slug, &title, &date)?;
    let req_id = reserved
        .eid
        .numeric_id()
        .context("requirement kind must yield a numeric id")?;
    let fk = requirement::canonical_id(req_id);

    // D-1: overwrite the template-seeded kind now that we know it.
    requirement::set_kind(&root, req_id, kind)?;

    // Step 4 (§5.4): append the membership row — the orphan window (C5).
    append_member(&members_path, &fk, &label, order)?;

    writeln!(io::stdout(), "Added {label} ({fk}) to {spec_ref}")?;
    Ok(())
}

/// `doctrine spec req status <REQ-NNN> --to <state> [--note <text>]` — the single
/// authored-`status` write seam (SL-044 B·P1, design §5.2). The thin impure shell:
/// find the root, resolve the requirement **by id only** (`REQ-NNN`, no slug/title
/// derivation — dodges the ISS-004 unescaped-slug abort), and delegate the
/// edit-preserving FREE any→any transition to `requirement::set_status`. The `--to`
/// `ReqStatus` is a closed clap `ValueEnum`, so an out-of-vocab value is rejected
/// before the verb runs; an unknown id surfaces as a read failure.
///
/// `--note` is operator prose with no structural home on the requirement entity in
/// v1 (material prose routes to the future IDE-003 vehicle); it is accepted and
/// intentionally not spliced into the TOML — preferring no invented field over a
/// speculative one.
pub(crate) fn run_req_status(
    path: Option<PathBuf>,
    req_ref: &str,
    to: ReqStatus,
    _note: Option<String>,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let id = requirement::id_from_fk(req_ref)?;
    requirement::set_status(&root, id, to)?;
    writeln!(io::stdout(), "Set {} status to {}", req_ref, to.as_str())?;
    Ok(())
}

/// `doctrine spec show <spec-ref>` — reassemble a spec into its readable whole and
/// write it to stdout (design §5.4). The impure shell: resolve the canonical ref
/// (C4), read the spec's own toml + prose body + members + (tech) interactions,
/// resolve each member's requirement by FK, then hand the parsed data to the pure
/// `render`. READ-ONLY: no write, no mutation, and **no cross-corpus scan** — only
/// this spec's dir and the requirement dirs reached by FK are opened (EX-2).
/// Ephemeral stdout, no `*.rendered.md` (D9).
pub(crate) fn run_show(
    path: Option<PathBuf>,
    spec_ref: &str,
    format: Format,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (subtype, spec_id) = resolve_spec_ref(spec_ref)?;
    let name = format!("{spec_id:03}");
    let spec_dir = root.join(subtype.kind().dir).join(&name);
    anyhow::ensure!(
        spec_dir.is_dir(),
        "no {} spec {spec_ref} at {}",
        subtype.label(),
        spec_dir.display()
    );

    // Read both tiers in one call (IMP-037 — read_spec extraction).
    let (spec, _spec_text, prose_body) = read_spec(subtype, &root, spec_id)?;

    // Resolve members → their requirement entities by canonical FK, reading both
    // TOML and MD tiers (IMP-058 — requirement prose visible). Only the membered
    // requirement dirs are touched — no whole-tree scan (EX-2).
    let members = read_members(&spec_dir.join("members.toml"))?;
    let mut resolved = Vec::with_capacity(members.len());
    let mut req_bodies: Vec<Option<String>> = Vec::with_capacity(members.len());
    for member in members {
        let (req, prose) = requirement::load_with_prose(&root, &member.requirement)?;
        req_bodies.push(prose);
        resolved.push((member, req));
    }

    let interactions = read_interactions(&spec_dir.join("interactions.toml"))?;

    let out = match format {
        Format::Table => render(&spec, &prose_body, &resolved, &req_bodies, &interactions),
        Format::Json => show_json(&spec, &prose_body, &resolved, &req_bodies, &interactions)?,
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
}

/// Render the `Json` show: the spec's faithful toml-as-data (`Spec`) plus the prose
/// body, its members (each edge with its resolved requirement's structured fields),
/// and its outbound interactions, under the shared `{kind, …}` envelope (the
/// `adr::show_json` precedent — toml-as-data is faithful, D7). Members keep advisory
/// `order`; the requirement is projected by hand (its struct stays Deserialize-only,
/// so render-faithful fields are spliced here, not via a derive). EX-2: still no
/// cross-corpus scan — only the data already read by `run_show` is serialised.
fn show_json(
    spec: &Spec,
    body: &str,
    members: &[(Member, Requirement)],
    req_bodies: &[Option<String>],
    interactions: &[Interaction],
) -> anyhow::Result<String> {
    let member_rows: Vec<serde_json::Value> = members
        .iter()
        .enumerate()
        .map(|(i, (m, req))| {
            let mut req_obj = serde_json::json!({
                "id": requirement::canonical_id(req.id),
                "slug": req.slug,
                "title": req.title,
                "kind": req.kind.as_str(),
                "status": req.status.as_str(),
                "tags": req.tags,
            });
            if let Some(prose) = req_bodies.get(i).and_then(|b| b.as_ref())
                && let Some(obj) = req_obj.as_object_mut()
            {
                obj.insert("body".to_string(), serde_json::Value::String(prose.clone()));
            }
            serde_json::json!({
                "label": m.label,
                "order": m.order,
                "requirement": req_obj,
            })
        })
        .collect();
    let value = serde_json::json!({
        "kind": "spec",
        "spec": spec,
        "id": spec.kind.canonical_id(spec.id),
        "body": body,
        "members": member_rows,
        "interactions": interactions,
    });
    serde_json::to_string_pretty(&value).context("failed to serialize spec show JSON")
}

/// Scan the three trees into a `Registry` (design §5.6) — the impure half of
/// `validate`, cache-independent and built fresh per invocation. Requirement ids
/// and tech-spec ids are stored canonical (the check-site needs no FK parsing);
/// member edges are collected from **both** subtypes (products member requirements
/// too), interaction edges from tech only (products have no `interactions.toml`).
pub(crate) fn build_registry(root: &Path) -> anyhow::Result<Registry> {
    let mut reg = Registry::default();

    for id in entity::scan_ids(&requirement::tree_root(root))? {
        reg.requirements.insert(requirement::canonical_id(id));
    }

    for subtype in [SpecSubtype::Product, SpecSubtype::Tech] {
        let tree = root.join(subtype.kind().dir);
        let on_product = subtype == SpecSubtype::Product;
        for id in entity::scan_ids(&tree)? {
            let spec_ref = subtype.canonical_id(id);
            let dir = tree.join(format!("{id:03}"));

            // Parse the spec itself to harvest its outbound relational fields. This
            // is a NEW per-spec read (Charge I) — `build_registry` parsed no spec
            // before SL-022, so a malformed `spec-NNN.toml` now surfaces here where
            // it was invisible to `validate`. BOTH arms harvest BOTH tech-only
            // fields so a product carrying one is seen, not dropped (codex F5b); the
            // `on_product` flag lets the check turn it into an invalid-kind finding.
            let spec_toml = entity::id_path(root, subtype.kind(), id, entity::Ext::Toml);
            let spec_text = std::fs::read_to_string(&spec_toml)
                .with_context(|| format!("Failed to read {}", spec_toml.display()))?;
            // Classify a `parent` duplicate-key / array parse error into a named
            // `second_parent` hard finding (carried, not propagated) before the `?`;
            // any other parse error still fails the build (Charge I error surface).
            let spec: Spec = match toml::from_str::<Spec>(&spec_text) {
                Ok(s) => s,
                Err(e) if is_second_parent(&e, &spec_text) => {
                    reg.build_findings.push(BuildFinding {
                        spec: spec_ref.clone(),
                        message: format!("second parent: {spec_ref} declares more than one parent"),
                    });
                    continue;
                }
                Err(e) => {
                    return Err(anyhow::Error::new(e)
                        .context(format!("Failed to parse {}", spec_toml.display())));
                }
            };
            if let Some(target) = &spec.descends_from {
                reg.descents.push(DescentEdge {
                    spec: spec_ref.clone(),
                    target: canonicalize_spec_ref(target),
                    on_product,
                });
            }
            if let Some(parent) = &spec.parent {
                reg.parents.push(ParentEdge {
                    spec: spec_ref.clone(),
                    parent: canonicalize_spec_ref(parent),
                    on_product,
                });
            }

            for m in read_members(&dir.join("members.toml"))? {
                reg.members.push(MemberEdge {
                    spec: spec_ref.clone(),
                    requirement: requirement::canonicalize_fk(&m.requirement),
                    label: m.label,
                });
            }
            if subtype == SpecSubtype::Tech {
                reg.tech_specs.insert(spec_ref.clone());
                for e in read_interactions(&dir.join("interactions.toml"))? {
                    reg.interactions.push(InteractionEdge {
                        spec: spec_ref.clone(),
                        target: canonicalize_spec_ref(&e.target),
                    });
                }
            } else {
                reg.product_specs.insert(spec_ref.clone());
            }
        }
    }
    Ok(reg)
}

/// Convenience entry point for the doctor (SL-168, #3 `SpecFk`): build a
/// `Registry` and run all spec-FK integrity checks, returning plain strings for
/// the `from_lines` adapter. Lives here, in the command tier, so the `registry`
/// leaf stays pure (out=0, ADR-001) — the composition is the orchestrator's, not
/// the index's (RV-185 F-1).
pub(crate) fn spec_fk_findings(root: &Path) -> Vec<String> {
    let Ok(reg) = build_registry(root) else {
        return vec![];
    };
    reg.validate(None)
}

/// `doctrine spec validate [<spec-ref>]` — the FK-integrity pass (§5.4). Whole-
/// corpus by default; a canonical `<spec-ref>` scopes it to that spec's outbound
/// FKs + label uniqueness (the corpus-only orphan check is suppressed). Prints each
/// hard finding to stdout and exits non-zero (via `bail!`) if any; a clean run
/// prints a one-line all-clear and exits zero. Read-only — pure over parsed facets.
pub(crate) fn run_validate(path: Option<PathBuf>, spec_ref: Option<&str>) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;

    // A scoped ref must name an existing spec, and the scope key is its canonical
    // form (the registry stores edges keyed by canonical ref).
    let scope = match spec_ref {
        Some(r) => {
            let (subtype, id) = resolve_spec_ref(r)?;
            let dir = root.join(subtype.kind().dir).join(format!("{id:03}"));
            anyhow::ensure!(
                dir.is_dir(),
                "no {} spec {r} at {}",
                subtype.label(),
                dir.display()
            );
            Some(subtype.canonical_id(id))
        }
        None => None,
    };

    let registry = build_registry(&root)?;
    let findings = registry.validate(scope.as_deref());

    let target = scope.as_deref().unwrap_or("corpus");
    if findings.is_empty() {
        writeln!(io::stdout(), "validate: {target} clean")?;
        return Ok(());
    }

    let mut lines = Vec::with_capacity(findings.len() + 1);
    for f in &findings {
        lines.push(format!("  {f}\n"));
    }
    write!(io::stdout(), "{}", lines.concat())?;
    anyhow::bail!("validate: {} hard finding(s) in {target}", findings.len())
}

/// The `spec list` known-status set (A-2) — the four `SpecStatus` variants, the
/// authority `--status` is validated against. Lockstep-guarded against the enum by
/// a drift-canary test (`spec_statuses_matches_the_variants`). spec has a CLOSED
/// status enum, so a *stored* status is always in-vocabulary — no drift marker is
/// possible (unlike slice's stringly status; design §5.5 vocabulary-drift).
pub(crate) const SPEC_STATUSES: &[&str] = &["draft", "active", "deprecated", "superseded"];

/// The `spec list` hide-set (design §5.3): a `superseded` spec no longer governs,
/// so it drops from the default list. `--all` or any explicit `--status` reveals it
/// (handled in `listing::retain`). A presentation predicate fed only to `retain` —
/// distinct from any lifecycle semantics.
fn is_hidden(status: &str) -> bool {
    status == "superseded"
}

/// Re-export of the spine's status validator, scoped to spec so callers read intent
/// locally. Guards `--status` against [`SPEC_STATUSES`] (READ/filter input only).
fn validate_statuses(given: &[String], known: &[&str]) -> anyhow::Result<()> {
    listing::validate_statuses(given, known)
}

/// Project a spec `Meta` to its filterable fields (design §5.2). `canonical` is the
/// prefixed id (`PRD-007` / `SPEC-012`) — the regex domain; spec's identity toml
/// carries no `[tags]` on the `Meta` read path, so the tag axis is empty here.
fn key(subtype: SpecSubtype, m: &Meta) -> listing::FilterFields {
    listing::FilterFields {
        canonical: subtype.canonical_id(m.id),
        slug: m.slug.clone(),
        title: m.title.clone(),
        status: m.status.clone(),
        tags: m.tags.clone(),
    }
}

/// One subtype's retained, sorted spec rows joined with their `#members` count —
/// the variant-axis join (the `slice list` phase-rollup precedent). The shared
/// `retain` filters the `Meta`s; the member-count read runs only for survivors,
/// after the filter. Sorted by id (ordering is spec's, not `retain`'s — §5.3).
fn subtype_rows(
    root: &Path,
    subtype: SpecSubtype,
    filter: &listing::Filter,
) -> anyhow::Result<Vec<(Meta, usize)>> {
    let tree = root.join(subtype.kind().dir);
    let mut metas = listing::retain(
        meta::read_metas(&tree, SPEC_STEM, subtype.kind().prefix)?,
        filter,
        is_hidden,
        |m| key(subtype, m),
    );
    metas.sort_by_key(|m| m.id);
    let mut rows = Vec::with_capacity(metas.len());
    for m in metas {
        let count = member_count(&tree.join(format!("{:03}", m.id)))?;
        rows.push((m, count));
    }
    Ok(rows)
}

/// One spec projected to its faithful JSON row (design §5.3 — spec owns its serde
/// shape). `id` is the prefixed canonical id (so product/tech ids never collide in
/// the single cross-subtype envelope, A-8); `subtype` labels each row in lieu of
/// two envelopes; `members` is the structured COUNT, not a rendered cell.
#[derive(Debug, Serialize)]
struct SpecRow {
    id: String,
    subtype: &'static str,
    status: String,
    slug: String,
    tags: Vec<String>,
    members: usize,
}

/// One spec pre-materialised for the table (SL-037 §4) — spec is the one kind
/// whose table and JSON rows do NOT coincide (A3): the table needs `title`
/// (absent from `SpecRow`) and a rendered `#members` cell, and its prefixed id is
/// *subtype-dependent* (`PRD`/`SPEC`). The id is resolved into the row PER BLOCK
/// where `subtype` is in scope (D5), so every extractor stays a trivial
/// non-capturing `fn(&SpecListRow)->String` — no captured subtype, no
/// `Box<dyn Fn>`. Table-only — NOT `Serialize` (the JSON path is `SpecRow`, D2).
struct SpecListRow {
    id: String,
    status: String,
    slug: String,
    title: String,
    tags: Vec<String>,
    members: usize,
}

/// The table columns a spec block can show (`--columns` tokens over
/// `R = SpecListRow`). Extractors are non-capturing (D5); the subtype-prefixed id
/// is already materialised in the row. `members`' header is `#members` while its
/// selector name is `members` — the one place header ≠ name in spec (the `#` is
/// shell-hostile as a token, design §4). Declaration order is what the
/// unknown-column error lists.
const SPEC_COLUMNS: [listing::Column<SpecListRow>; 6] = [
    listing::Column {
        name: "id",
        header: "id",
        cell: |r| r.id.clone(),
        paint: listing::ColumnPaint::Fixed(owo_colors::DynColors::Ansi(
            owo_colors::AnsiColors::Cyan,
        )),
    },
    listing::Column {
        name: "status",
        header: "status",
        cell: |r| r.status.clone(),
        paint: listing::ColumnPaint::ByValue(|r| listing::status_hue(&r.status)),
    },
    listing::Column {
        name: "tags",
        header: "tags",
        cell: |r| r.tags.join(", "),
        paint: listing::ColumnPaint::PerToken {
            split: |r| r.tags.clone(),
            render: listing::paint_tag,
        },
    },
    listing::Column {
        name: "slug",
        header: "slug",
        cell: |r| r.slug.clone(),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "title",
        header: "title",
        cell: |r| r.title.clone(),
        paint: listing::ColumnPaint::Alternate([listing::TITLE_EVEN, listing::TITLE_ODD]),
    },
    listing::Column {
        name: "members",
        header: "#members",
        cell: |r| r.members.to_string(),
        paint: listing::ColumnPaint::None,
    },
];

/// The default visible set — the D4 slug→title swap: the spec table GAINS `title`
/// and DROPS `slug` from the default; `--columns …,slug` still reveals it.
const SPEC_DEFAULT: &[&str] = &["id", "status", "title", "members"];

/// Materialise one subtype's `(Meta, count)` rows into table rows, resolving the
/// subtype-dependent prefixed id (`PRD`/`SPEC`) HERE where `subtype` is in scope —
/// so the column extractors never capture it (D5). Mirrors governance's `gov_rows`.
fn spec_list_rows(subtype: SpecSubtype, rows: &[(Meta, usize)]) -> Vec<SpecListRow> {
    rows.iter()
        .map(|(m, count)| SpecListRow {
            id: subtype.canonical_id(m.id),
            status: m.status.clone(),
            slug: m.slug.clone(),
            title: m.title.clone(),
            tags: m.tags.clone(),
            members: *count,
        })
        .collect()
}

/// `doctrine spec list` — the survey verb, on the shared spine (design §5.4). The
/// compute half is [`list_rows`]; this is the thin shell that writes it.
pub(crate) fn run_list(path: Option<PathBuf>, args: ListArgs) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    write!(io::stdout(), "{}", list_rows(&root, args)?)?;
    Ok(())
}

/// The `spec list` output as a string — the compute half of `run_list`. Rides the
/// shared spine: `validate_statuses` guards `--status` (A-2), `listing::build`
/// resolves the filter + format. `Table` emits per-subtype labelled blocks
/// (product then tech), each `id status slug #members`, the `#members` derived per
/// row (spec's variant axis). `Json` emits a SINGLE `{kind:"spec", rows:[…]}`
/// envelope spanning BOTH subtypes, each row carrying a `subtype` field (A-8) — not
/// two envelopes. Empty → `""` (§5.5).
pub(crate) fn list_rows(root: &Path, mut args: ListArgs) -> anyhow::Result<String> {
    validate_statuses(&args.status, SPEC_STATUSES)?;
    let render = args.render;
    let columns = args.columns.take();
    let (filter, format) = listing::build(args)?;
    match format {
        Format::Table => {
            // Pre-collect blocks so we can scan all rows for any_tagged (the
            // effective default depends on the cross-subtype union, R3).
            let mut blocks_data: Vec<(SpecSubtype, Vec<SpecListRow>)> = Vec::new();
            for subtype in [SpecSubtype::Product, SpecSubtype::Tech] {
                let block_rows = spec_list_rows(subtype, &subtype_rows(root, subtype, &filter)?);
                if block_rows.is_empty() {
                    continue;
                }
                blocks_data.push((subtype, block_rows));
            }
            let any_tagged = blocks_data
                .iter()
                .flat_map(|(_, rows)| rows.iter())
                .any(|r| !r.tags.is_empty());
            let effective_default = listing::default_with_tags(SPEC_DEFAULT, any_tagged);
            let sel =
                listing::select_columns(&SPEC_COLUMNS, &effective_default, columns.as_deref())?;
            let mut blocks = Vec::new();
            for (subtype, block_rows) in blocks_data {
                blocks.push(format!(
                    "{}\n{}",
                    subtype.label(),
                    listing::render_columns(&block_rows, &sel, render)
                ));
            }
            Ok(blocks.concat())
        }
        Format::Json => {
            let mut rows = Vec::new();
            for subtype in [SpecSubtype::Product, SpecSubtype::Tech] {
                for (m, count) in subtype_rows(root, subtype, &filter)? {
                    rows.push(SpecRow {
                        id: subtype.canonical_id(m.id),
                        subtype: subtype.label(),
                        status: m.status,
                        slug: m.slug,
                        tags: m.tags.clone(),
                        members: count,
                    });
                }
            }
            listing::json_envelope("spec", &rows)
        }
    }
}

// ---------------------------------------------------------------------------
// `spec req list` — the authored-only requirement roster (design §5.1/§5.2/§5.4)
// ---------------------------------------------------------------------------

/// One requirement membered by a spec, pre-materialised for the table (mirrors
/// [`SpecListRow`], SL-037 §4). **Authored-only (INV-3):** every cell comes from
/// an authored file — `id` is the canonical FK, `label` the sticky membership
/// label (`FR-`/`NF-`), `kind`/`status` the requirement's own authored fields.
/// There is deliberately **no observed/verdict column** — the roster never scans
/// (no `coverage` import). On a dangling member FK the `kind`/`status` cells hold
/// the inline load-error note instead (E5, degrade-and-continue). Table-only —
/// NOT `Serialize` (the JSON path is [`ReqJsonRow`], mirroring spec's D2 split).
struct ReqListRow {
    id: String,
    label: String,
    kind: String,
    status: String,
    prose: String,
}

/// One roster entry projected to its faithful JSON row (mirrors [`SpecRow`]).
/// The roster's JSON contract is lighter than coverage's: `id`/`label`/`kind`/
/// `status` for a resolved member; a dangling member drops `kind`/`status` and
/// surfaces `load_error` instead, so the corpus-health signal is machine-visible
/// (`dangling: true`) rather than silently absent.
#[derive(Debug, Serialize)]
struct ReqJsonRow {
    id: String,
    label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    dangling: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    load_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prose: Option<bool>,
}

/// The table columns a roster row can show (`--columns` tokens over
/// `R = ReqListRow`). Non-capturing extractors (SL-037 D5). Declaration order is
/// what the unknown-column error lists. Mirrors [`SPEC_COLUMNS`].
const REQ_COLUMNS: [listing::Column<ReqListRow>; 5] = [
    listing::Column {
        name: "id",
        header: "id",
        cell: |r| r.id.clone(),
        paint: listing::ColumnPaint::Fixed(owo_colors::DynColors::Ansi(
            owo_colors::AnsiColors::Cyan,
        )),
    },
    listing::Column {
        name: "label",
        header: "label",
        cell: |r| r.label.clone(),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "kind",
        header: "kind",
        cell: |r| r.kind.clone(),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "status",
        header: "status",
        cell: |r| r.status.clone(),
        paint: listing::ColumnPaint::ByValue(|r| listing::status_hue(&r.status)),
    },
    listing::Column {
        name: "prose",
        header: "prose",
        cell: |r| r.prose.clone(),
        paint: listing::ColumnPaint::None,
    },
];

/// The default visible set — every authored column (`id label kind status`).
const REQ_DEFAULT: &[&str] = &["id", "label", "kind", "status", "prose"];

/// Resolve a spec's membered requirements into rows, degrading a dangling member
/// FK to an error-bearing row (E5). `member_reqs` (PHASE-02) supplies the ordered,
/// FK-canonicalised members; each is loaded for its authored `kind`/`status`. A
/// load failure does NOT abort the roster — the offending row carries the inline
/// error in place of those cells and the walk continues (symmetric with the
/// coverage scan's dangling tolerance). Returns each row paired with the loaded
/// `Requirement` (when resolvable) so the caller can project its filter fields
/// without a second read.
fn req_rows(root: &Path, spec_ref: &str) -> anyhow::Result<Vec<(ReqListRow, Option<Requirement>)>> {
    let members = member_reqs(root, spec_ref)?;
    let mut rows = Vec::with_capacity(members.len());
    for m in members {
        match requirement::load_with_prose(root, &m.requirement) {
            Ok((req, prose)) => {
                let prose_indicator = if prose.is_some() {
                    "\u{2713}"
                } else {
                    "\u{2014}"
                };
                let row = ReqListRow {
                    id: m.requirement.clone(),
                    label: m.label.clone(),
                    kind: req.kind.as_str().to_string(),
                    status: req.status.as_str().to_string(),
                    prose: prose_indicator.to_string(),
                };
                rows.push((row, Some(req)));
            }
            Err(e) => {
                // Degrade-and-continue (E5): the inline load-error replaces the
                // authored cells rather than aborting the whole roster.
                let note = format!("<load error: {e}>");
                let row = ReqListRow {
                    id: m.requirement.clone(),
                    label: m.label.clone(),
                    kind: note.clone(),
                    status: note,
                    prose: "\u{2014}".to_string(),
                };
                rows.push((row, None));
            }
        }
    }
    Ok(rows)
}

/// Project a resolved roster row to its filterable fields (design §5.2). The
/// requirement's authored `slug`/`title`/`tags` come from the loaded entity; the
/// canonical FK is the regex domain's leading field (mirrors spec's [`key`]).
fn req_key(id: &str, req: &Requirement) -> listing::FilterFields {
    listing::FilterFields {
        canonical: id.to_string(),
        slug: req.slug.clone(),
        title: req.title.clone(),
        status: req.status.as_str().to_string(),
        tags: req.tags.clone(),
    }
}

/// The `spec req list` output as a string — the compute half of [`run_req_list`],
/// factored pure-ish so it is unit-testable without a CLI (mirrors [`list_rows`]).
/// Rides the shared spine: `listing::build` resolves filter + format, `retain`
/// applies `--status/--filter/--tag/--all` (E3). **Authored-only (INV-3):** no
/// scan, no observed column. A dangling member FK is rendered as a degraded row
/// and is **always kept** — its authored fields are unreadable, so the filter is
/// moot, and dropping it would hide a corpus-health signal (E5). `Table` reuses
/// `select_columns`/`render_columns` UNCHANGED (A5); `Json` emits a faithful
/// `{kind:"requirement", rows:[…]}` envelope. Empty → `""` (§5.5).
fn req_list_rows(root: &Path, spec_ref: &str, mut args: ListArgs) -> anyhow::Result<String> {
    // F4/SL-025 parity: validate `--status` against the requirement known-set
    // before filtering, exactly as `list_rows` does against `SPEC_STATUSES` — a
    // bogus status errors here rather than silently emptying the roster (RV-005 F-1).
    validate_statuses(&args.status, requirement::REQ_STATUSES)?;
    let render = args.render;
    let columns = args.columns.take();
    let (filter, format) = listing::build(args)?;
    let rows = req_rows(root, spec_ref)?;
    // Filter the resolved rows through the shared spine ONCE (reusing `retain`,
    // not a parallel filter); a dangling row is kept unconditionally — it has no
    // authored fields for `--status`/`--filter` to speak to, and silencing the
    // corpus-health signal would be a read that lies (E5). Indices keep both sets
    // in their original member order (`member_reqs` ordering, which `retain`
    // preserves) when re-interleaved.
    let resolved: Vec<(usize, &ReqListRow, &Requirement)> = rows
        .iter()
        .enumerate()
        .filter_map(|(i, (row, req))| req.as_ref().map(|r| (i, row, r)))
        .collect();
    let kept_resolved: std::collections::BTreeSet<usize> =
        listing::retain(resolved, &filter, is_hidden, |(_, row, req)| {
            req_key(&row.id, req)
        })
        .into_iter()
        .map(|(i, _, _)| i)
        .collect();
    let kept: Vec<(ReqListRow, Option<Requirement>)> = rows
        .into_iter()
        .enumerate()
        .filter(|(i, (_, req))| req.is_none() || kept_resolved.contains(i))
        .map(|(_, pair)| pair)
        .collect();
    match format {
        Format::Table => {
            let sel = listing::select_columns(&REQ_COLUMNS, REQ_DEFAULT, columns.as_deref())?;
            let table_rows: Vec<ReqListRow> = kept.into_iter().map(|(row, _)| row).collect();
            Ok(listing::render_columns(&table_rows, &sel, render))
        }
        Format::Json => {
            let json_rows: Vec<ReqJsonRow> = kept
                .into_iter()
                .map(|(row, req)| match req {
                    Some(ref req) => {
                        let prose_val = Some(row.prose == "\u{2713}");
                        ReqJsonRow {
                            id: row.id,
                            label: row.label,
                            kind: Some(row.kind),
                            status: Some(row.status),
                            tags: req.tags.clone(),
                            dangling: false,
                            load_error: None,
                            prose: prose_val,
                        }
                    }
                    None => ReqJsonRow {
                        id: row.id,
                        label: row.label,
                        kind: None,
                        status: None,
                        tags: vec![],
                        dangling: true,
                        // `kind` held the load-error note for the table row.
                        load_error: Some(row.kind),
                        prose: None,
                    },
                })
                .collect();
            listing::json_envelope("requirement", &json_rows)
        }
    }
}

/// `doctrine spec req list <SPEC>` — the authored requirement roster (design
/// §5.4). The compute half is [`req_list_rows`]; this is the thin shell that
/// resolves the root and writes it.
pub(crate) fn run_req_list(
    path: Option<PathBuf>,
    spec_ref: &str,
    args: ListArgs,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    write!(io::stdout(), "{}", req_list_rows(&root, spec_ref, args)?)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// `spec interactions add` / `spec interactions remove` (SL-153 PHASE-03)
// ---------------------------------------------------------------------------

/// Remove all `[[edge]]` rows from a held interactions.toml `DocumentMut` whose
/// canonical `target` matches `canonical_target`. Returns how many rows were
/// removed. Preserves comments and inert rows. Idempotent (returns 0 if no match).
fn remove_interaction_edges(doc: &mut toml_edit::DocumentMut, canonical_target: &str) -> usize {
    let Some(edges) = doc.get_mut("edge").and_then(|v| v.as_array_of_tables_mut()) else {
        return 0;
    };
    let mut indices: Vec<usize> = Vec::new();
    for (i, row) in edges.iter().enumerate() {
        if let Some(target_val) = row.get("target").and_then(|v| v.as_str()) {
            let row_canon = canonicalize_spec_ref(target_val);
            if row_canon == canonical_target {
                indices.push(i);
            }
        }
    }
    for i in indices.iter().rev() {
        edges.remove(*i);
    }
    indices.len()
}

/// `doctrine spec interactions add <SPEC> <SPEC> --type <text> [--notes <text>]` —
/// append an outbound `[[edge]]` row to a tech spec's `interactions.toml`.
fn run_interaction_add(
    path: Option<PathBuf>,
    spec_ref: &str,
    target: &str,
    kind: &str,
    notes: Option<&str>,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;

    // Resolve the source spec — must be Tech (interactions are tech-only).
    let (subtype, spec_id) = resolve_spec_ref(spec_ref)?;
    anyhow::ensure!(
        subtype == SpecSubtype::Tech,
        "interactions are tech-spec-only; {spec_ref} is a product spec"
    );
    let spec_dir = root.join(subtype.kind().dir).join(format!("{spec_id:03}"));
    anyhow::ensure!(
        spec_dir.is_dir(),
        "no {} spec {spec_ref} at {}",
        subtype.label(),
        spec_dir.display()
    );

    // Resolve the target — must be a tech spec that exists.
    let (target_subtype, target_id) = resolve_spec_ref(target)?;
    anyhow::ensure!(
        target_subtype == SpecSubtype::Tech,
        "interaction target must be a tech spec; {target} is a {}",
        target_subtype.label()
    );
    let target_dir = root
        .join(target_subtype.kind().dir)
        .join(format!("{target_id:03}"));
    anyhow::ensure!(
        target_dir.is_dir(),
        "target spec {target} not found at {}",
        target_dir.display()
    );
    let canonical_target = target_subtype.canonical_id(target_id);

    // Read interactions.toml into DocumentMut.
    let interactions_path = spec_dir.join("interactions.toml");
    let text = std::fs::read_to_string(&interactions_path)
        .with_context(|| format!("Failed to read {}", interactions_path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", interactions_path.display()))?;

    // Canonical dup-check: iterate existing [[edge]] rows, canonicalize each
    // `target`, if match found → no-op informational.
    if let Some(edges) = doc.get("edge").and_then(|v| v.as_array_of_tables()) {
        for row in edges {
            if let Some(existing_target) = row.get("target").and_then(|v| v.as_str())
                && canonicalize_spec_ref(existing_target) == canonical_target
            {
                writeln!(
                    io::stdout(),
                    "edge to {target} already present; remove + add to change its type"
                )?;
                return Ok(());
            }
        }
    }

    // Append new [[edge]] table.
    let edges = doc
        .entry("edge")
        .or_insert(toml_edit::Item::ArrayOfTables(
            toml_edit::ArrayOfTables::new(),
        ))
        .as_array_of_tables_mut()
        .context("`edge` is not an array of tables")?;
    let mut row = toml_edit::Table::new();
    row.insert("target", toml_edit::value(target));
    row.insert("type", toml_edit::value(kind));
    if let Some(n) = notes {
        row.insert("notes", toml_edit::value(n));
    }
    edges.push(row);

    crate::fsutil::write_atomic(&interactions_path, doc.to_string().as_bytes())
        .with_context(|| format!("Failed to write {}", interactions_path.display()))?;

    writeln!(
        io::stdout(),
        "Added interaction {spec_ref} → {canonical_target} ({kind})"
    )?;
    Ok(())
}

/// `doctrine spec interactions remove <SPEC> <SPEC>` — remove all outbound
/// `[[edge]]` rows to a target from a tech spec's `interactions.toml`.
fn run_interaction_remove(
    path: Option<PathBuf>,
    spec_ref: &str,
    target: &str,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;

    // Resolve the source spec — must be Tech.
    let (subtype, spec_id) = resolve_spec_ref(spec_ref)?;
    anyhow::ensure!(
        subtype == SpecSubtype::Tech,
        "interactions are tech-spec-only; {spec_ref} is a product spec"
    );
    let spec_dir = root.join(subtype.kind().dir).join(format!("{spec_id:03}"));
    anyhow::ensure!(
        spec_dir.is_dir(),
        "no {} spec {spec_ref} at {}",
        subtype.label(),
        spec_dir.display()
    );

    // Canonicalize the target for comparison.
    let canonical_target = canonicalize_spec_ref(target);

    // Read interactions.toml into DocumentMut.
    let interactions_path = spec_dir.join("interactions.toml");
    let text = std::fs::read_to_string(&interactions_path)
        .with_context(|| format!("Failed to read {}", interactions_path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", interactions_path.display()))?;

    // Remove matching edges.
    let count = remove_interaction_edges(&mut doc, &canonical_target);
    if count > 0 {
        crate::fsutil::write_atomic(&interactions_path, doc.to_string().as_bytes())
            .with_context(|| format!("Failed to write {}", interactions_path.display()))?;
        writeln!(
            io::stdout(),
            "Removed {count} interaction(s) via {spec_ref} → {canonical_target}"
        )?;
    } else {
        writeln!(
            io::stdout(),
            "No interaction edge to {canonical_target} found in {spec_ref}"
        )?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::LocalFs;
    use crate::requirement::ReqStatus;
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;

    /// A no-constraint `ListArgs` (the default `spec list`).
    fn list_args() -> ListArgs {
        ListArgs::default()
    }

    fn fresh(root: &Path, subtype: SpecSubtype, slug: &str, title: &str) -> entity::Materialised {
        entity::materialise(
            subtype.kind(),
            &LocalFs,
            root,
            &MaterialiseRequest::Fresh,
            &Inputs {
                slug,
                title,
                date: "2026-06-05",
            },
            &[],
            &mut entity::local_reserved(),
        )
        .unwrap()
    }

    // --- PHASE-05: the registry scan (build_registry) ---

    /// The impure scan reaches all three trees: requirement ids (canonical),
    /// tech-spec ids (canonical, tech only), member edges from BOTH subtypes, and
    /// interaction edges from tech only. The pure checks are unit-tested in
    /// `registry.rs`; this covers the disk→`Registry` half.
    #[test]
    fn build_registry_scans_all_three_trees() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
        fresh(root, SpecSubtype::Tech, "store", "Store"); // SPEC-002
        fresh(root, SpecSubtype::Product, "login", "Login"); // PRD-001
        for slug in ["a", "b", "c"] {
            requirement::reserve(root, slug, slug, "2026-06-05").unwrap(); // REQ-001..003
        }
        // A tech member and a PRODUCT member — both must be collected. REQ-003 is
        // left unmembered (an orphan, for the checks' benefit, not asserted here).
        append_member(
            &root.join(".doctrine/spec/tech/001/members.toml"),
            "REQ-001",
            "FR-001",
            1,
        )
        .unwrap();
        append_member(
            &root.join(".doctrine/spec/product/001/members.toml"),
            "REQ-002",
            "FR-001",
            1,
        )
        .unwrap();
        // A hand-authored interaction (no verb in v1 — D-Q4): SPEC-001 → SPEC-002.
        let ix = root.join(".doctrine/spec/tech/001/interactions.toml");
        let mut s = fs::read_to_string(&ix).unwrap();
        s.push_str("\n[[edge]]\ntarget = \"SPEC-002\"\ntype = \"calls\"\n");
        fs::write(&ix, s).unwrap();

        let reg = build_registry(root).unwrap();

        let want_reqs: BTreeSet<String> = ["REQ-001", "REQ-002", "REQ-003"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(reg.requirements, want_reqs);
        let want_techs: BTreeSet<String> = ["SPEC-001", "SPEC-002"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(reg.tech_specs, want_techs); // products excluded from the set
        assert_eq!(reg.members.len(), 2, "members from both subtypes");
        assert!(
            reg.members
                .iter()
                .any(|m| m.spec == "PRD-001" && m.requirement == "REQ-002"),
            "the product member edge is collected"
        );
        assert_eq!(reg.interactions.len(), 1, "tech-only interaction edge");
        assert!(
            reg.interactions
                .iter()
                .any(|e| e.spec == "SPEC-001" && e.target == "SPEC-002")
        );
    }

    // --- SL-046 VT-1/VT-3: the relation_edges accessor ---

    /// VT-1: a tech spec's outbound relations — lineage Options, members, and
    /// interactions — surface with the right labels via the show-path readers.
    #[test]
    fn relation_edges_tech_lineage_members_interactions() {
        use crate::relation::RelationLabel;
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
        // Hand-author lineage Options onto the toml (no verb in v1).
        let toml_path = root.join(".doctrine/spec/tech/001/spec-001.toml");
        let mut t = fs::read_to_string(&toml_path).unwrap();
        t.push_str("descends_from = \"PRD-005\"\nparent = \"SPEC-000\"\n");
        fs::write(&toml_path, t).unwrap();
        append_member(
            &root.join(".doctrine/spec/tech/001/members.toml"),
            "REQ-009",
            "FR-001",
            1,
        )
        .unwrap();
        let ix = root.join(".doctrine/spec/tech/001/interactions.toml");
        let mut s = fs::read_to_string(&ix).unwrap();
        s.push_str("\n[[edge]]\ntarget = \"SPEC-002\"\ntype = \"calls\"\nnotes = \"sync\"\n");
        fs::write(&ix, s).unwrap();

        let edges = relation_edges(SpecSubtype::Tech, root, 1).unwrap();
        let got: Vec<(RelationLabel, &str)> =
            edges.iter().map(|e| (e.label, e.target.as_str())).collect();
        assert_eq!(
            got,
            vec![
                (RelationLabel::DescendsFrom, "PRD-005"),
                (RelationLabel::Parent, "SPEC-000"),
                (RelationLabel::Members, "REQ-009"),
                (RelationLabel::Interactions, "SPEC-002"),
            ]
        );
    }

    /// VT-3: the per-edge free-text `type` is NOT carried on the `RelationEdge`
    /// (single `Interactions` class), but it round-trips from the SOURCE
    /// `Interaction` struct for re-read at render (C2/D2).
    #[test]
    fn interactions_free_text_type_round_trips_from_source() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
        let ix = root.join(".doctrine/spec/tech/001/interactions.toml");
        let mut s = fs::read_to_string(&ix).unwrap();
        s.push_str("\n[[edge]]\ntarget = \"SPEC-002\"\ntype = \"depends-on\"\nnotes = \"n\"\n");
        fs::write(&ix, s).unwrap();

        // The accessor collapses the type into a single class label.
        let edges = relation_edges(SpecSubtype::Tech, root, 1).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].label, crate::relation::RelationLabel::Interactions);
        assert_eq!(edges[0].target, "SPEC-002");

        // The free-text type is recoverable from the source for the PHASE-04 render.
        let src = read_interactions(&ix).unwrap();
        assert_eq!(src.len(), 1);
        assert_eq!(
            src[0].kind, "depends-on",
            "free-text type survives at source"
        );
    }

    // --- VT-1: per-subtype scaffold filesets ---

    #[test]
    fn product_spec_scaffold_is_light_3_files() {
        let ctx = ScaffoldCtx {
            id: 7,
            canonical: "PRD-007",
            slug: "fast-onboarding",
            title: "Fast onboarding",
            date: "2026-06-05",
        };
        let fileset = product_spec_scaffold(&ctx).unwrap();
        // 3 content files + the alias symlink; NO interactions.toml.
        assert_eq!(fileset.len(), 4);
        assert!(matches!(&fileset[0],
            Artifact::File { rel_path, body }
            if rel_path == Path::new("007/spec-007.toml") && body.contains("kind = \"product\"")));
        assert!(matches!(&fileset[1],
            Artifact::File { rel_path, body }
            if rel_path == Path::new("007/spec-007.md") && body.contains("PRD-007: Fast onboarding")));
        assert!(matches!(&fileset[2],
            Artifact::File { rel_path, .. } if rel_path == Path::new("007/members.toml")));
        assert!(matches!(&fileset[3],
            Artifact::Symlink { rel_path, target }
            if rel_path == Path::new("007-fast-onboarding") && target == "007"));
        assert!(
            !fileset.iter().any(|a| matches!(a,
                Artifact::File { rel_path, .. } if rel_path == Path::new("007/interactions.toml"))),
            "product has no interactions.toml"
        );
    }

    #[test]
    fn tech_spec_scaffold_has_members_and_interactions() {
        let ctx = ScaffoldCtx {
            id: 3,
            canonical: "SPEC-003",
            slug: "cli",
            title: "CLI",
            date: "2026-06-05",
        };
        let fileset = tech_spec_scaffold(&ctx).unwrap();
        // 4 content files (+ interactions.toml) + the alias symlink.
        assert_eq!(fileset.len(), 5);
        let has = |p: &str| {
            fileset
                .iter()
                .any(|a| matches!(a, Artifact::File { rel_path, .. } if rel_path == Path::new(p)))
        };
        assert!(has("003/spec-003.toml"));
        assert!(has("003/spec-003.md"));
        assert!(has("003/members.toml"));
        assert!(has("003/interactions.toml"));
        // the tech toml carries kind=tech and the flat-field scaffolding.
        let toml_body = match &fileset[0] {
            Artifact::File { body, .. } => body,
            _ => panic!("first artifact is the toml"),
        };
        assert!(toml_body.contains("kind = \"tech\""));
        assert!(toml_body.contains("responsibilities = []"));
    }

    #[test]
    fn materialise_fresh_writes_each_subtype_in_its_own_namespace() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        let p1 = fresh(root, SpecSubtype::Product, "onboarding", "Onboarding");
        let t1 = fresh(root, SpecSubtype::Tech, "cli", "CLI");
        // independent namespaces — both start at 001.
        assert_eq!(p1.eid.numeric_id(), Some(1));
        assert_eq!(t1.eid.numeric_id(), Some(1));

        assert!(
            root.join(".doctrine/spec/product/001/spec-001.toml")
                .is_file()
        );
        assert!(
            root.join(".doctrine/spec/product/001/members.toml")
                .is_file()
        );
        assert!(
            !root
                .join(".doctrine/spec/product/001/interactions.toml")
                .exists()
        );
        assert!(
            root.join(".doctrine/spec/tech/001/interactions.toml")
                .is_file()
        );
        assert_eq!(
            fs::read_link(root.join(".doctrine/spec/product/001-onboarding")).unwrap(),
            Path::new("001")
        );

        // a second product lands 002; tech is unaffected (separate reservation).
        let p2 = fresh(root, SpecSubtype::Product, "billing", "Billing");
        assert_eq!(p2.eid.numeric_id(), Some(2));
        let md = fs::read_to_string(root.join(".doctrine/spec/tech/001/spec-001.md")).unwrap();
        assert!(md.contains("SPEC-001: CLI"));
    }

    // --- VT-2: shared Meta round-trip + the member-count column ---

    #[test]
    fn spec_list_meta_parses_scaffolded_spec_toml() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Product, "onboarding", "Onboarding");

        // C2: the scaffolded identity toml round-trips through the SHARED reader.
        let tree = root.join(".doctrine/spec/product");
        let m = meta::read_meta(&tree, SPEC_STEM, 1, SpecSubtype::Product.kind().prefix).unwrap();
        assert_eq!(
            m,
            Meta {
                id: 1,
                slug: "onboarding".to_string(),
                title: "Onboarding".to_string(),
                status: "draft".to_string(),
                tags: vec![],
            }
        );
    }

    #[test]
    fn render_spec_toml_escapes_hostile_title_and_slug() {
        // SL-024 (inquisition Charge 1): spec has no existing direct render test —
        // call `render_spec_toml` DIRECTLY (the disk path via `fresh` would
        // false-red at `<id>-<slug>` symlink creation, the wrong stratum). A title
        // / explicit slug carrying the quoted-literal breakers must round-trip.
        let title = crate::tomlfmt::HOSTILE_TITLE;
        let slug = crate::tomlfmt::HOSTILE_SLUG;
        let body = render_spec_toml(SpecSubtype::Product, 7, slug, title).unwrap();
        let parsed: Meta = toml::from_str(&body).unwrap();
        assert_eq!(parsed.slug, slug);
        assert_eq!(parsed.title, title);
    }

    #[test]
    fn spec_list_rows_per_subtype_with_member_count() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Product, "onboarding", "Onboarding");
        fresh(root, SpecSubtype::Product, "billing", "Billing");

        // seeded specs → member count 0 on every row. Prefixed ids, per-subtype
        // labelled block (the product block; the tech block is suppressed empty).
        // The default table is [id, status, title, members] (D4 slug→title swap):
        // it shows the human title, NOT the slug, plus the `#members` header.
        let out = list_rows(root, list_args()).unwrap();
        assert!(out.starts_with("product\n"), "product block leads: {out}");
        assert!(out.contains("#members"));
        assert!(out.contains("PRD-001 │ draft  │ Onboarding"), "{out}");
        assert!(out.contains("PRD-002 │ draft  │ Billing"), "{out}");
        // slug is dropped from the default set (still reachable via --columns).
        assert!(!out.contains("onboarding"), "slug hidden by default: {out}");
        assert!(!out.contains("billing"), "slug hidden by default: {out}");
        // both data rows end in the 0 member count.
        for line in out.lines().filter(|l| l.starts_with("PRD-")) {
            assert!(
                line.trim_end().ends_with('0'),
                "row ends in #members=0: {line}"
            );
        }

        // no tech specs → the tech block is suppressed entirely (no "tech" label).
        assert!(
            !out.contains("tech\n"),
            "empty tech block suppressed: {out}"
        );
    }

    #[test]
    fn list_rows_columns_selects_orders_and_reveals_slug() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Product, "onboarding", "Onboarding");

        // --columns id,slug reveals slug (hidden by default) and drops the rest.
        let out = list_rows(
            root,
            ListArgs {
                columns: Some(vec!["id".into(), "slug".into()]),
                ..ListArgs::default()
            },
        )
        .unwrap();
        assert!(out.starts_with("product\n"), "block label preserved: {out}");
        assert!(out.contains("id"));
        assert!(out.contains("slug"));
        assert!(out.contains("PRD-001 │ onboarding"), "{out}");
        // unselected columns are gone (title/status/#members).
        assert!(!out.contains("#members"), "members dropped: {out}");
        assert!(!out.contains("Onboarding"), "title dropped: {out}");
    }

    #[test]
    fn list_rows_unknown_column_is_the_uniform_error_listing_available() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Product, "onboarding", "Onboarding");

        let err = list_rows(
            root,
            ListArgs {
                columns: Some(vec!["bogus".into()]),
                ..ListArgs::default()
            },
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("bogus"), "names the bad column: {msg}");
        // the available set is listed, including the `#members` token name `members`.
        assert!(msg.contains("members"), "lists the available set: {msg}");
        assert!(msg.contains("title"), "lists the available set: {msg}");
    }

    #[test]
    fn list_rows_prefixed_ids_are_correct_per_subtype() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Product, "onboarding", "Onboarding"); // PRD-001
        fresh(root, SpecSubtype::Tech, "cli", "CLI"); // SPEC-001

        let out = list_rows(root, list_args()).unwrap();
        // product → PRD prefix, tech → SPEC prefix, resolved per block.
        assert!(out.contains("PRD-001"), "product id prefixed PRD: {out}");
        assert!(out.contains("SPEC-001"), "tech id prefixed SPEC: {out}");
    }

    /// Write a spec's identity toml directly at an explicit id under the subtype's
    /// tree (creating the dir), bypassing the monotonic `fresh` allocator so the
    /// fixture's creation order can differ from id order. No members.toml — the
    /// member count reads 0 (read_members tolerates absence). Only the spine-read
    /// fields are written.
    fn spec_at(root: &Path, subtype: SpecSubtype, id: u32, status: &str, slug: &str, title: &str) {
        let name = format!("{id:03}");
        let dir = root.join(subtype.kind().dir).join(&name);
        fs::create_dir_all(&dir).unwrap();
        let toml = format!(
            "id = {id}\nslug = \"{slug}\"\ntitle = \"{title}\"\nstatus = \"{status}\"\ncreated = \"2026-06-04\"\nupdated = \"2026-06-04\"\n"
        );
        let spec_toml = entity::id_path(root, subtype.kind(), id, entity::Ext::Toml);
        fs::write(&spec_toml, toml).unwrap();
    }

    #[test]
    fn list_rows_orders_by_id_within_each_subtype_block_regardless_of_creation_order() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // Product subtype, created OUT of id order: 003, 001, 002.
        spec_at(root, SpecSubtype::Product, 3, "draft", "pg", "ProductGamma");
        spec_at(root, SpecSubtype::Product, 1, "draft", "pa", "ProductAlpha");
        spec_at(root, SpecSubtype::Product, 2, "draft", "pb", "ProductBeta");
        // Tech subtype, also out of order: 002 then 001.
        spec_at(root, SpecSubtype::Tech, 2, "draft", "tb", "TechBeta");
        spec_at(root, SpecSubtype::Tech, 1, "draft", "ta", "TechAlpha");

        let out = list_rows(root, list_args()).unwrap();
        let off = |id: &str| {
            out.find(id)
                .unwrap_or_else(|| panic!("{id} present: {out}"))
        };
        // product block leads, ascending ids within it.
        assert!(
            off("PRD-001") < off("PRD-002") && off("PRD-002") < off("PRD-003"),
            "product rows ascend by id: {out}"
        );
        // tech block ascends by id.
        assert!(
            off("SPEC-001") < off("SPEC-002"),
            "tech rows ascend by id: {out}"
        );
        // the whole product block precedes the whole tech block.
        assert!(
            off("PRD-003") < off("SPEC-001"),
            "the product block precedes the tech block: {out}"
        );
    }

    #[test]
    fn member_count_reads_appended_rows() {
        // prove the column is live, not hardcoded 0: a hand-appended member counts.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "cli", "CLI");
        let spec_dir = root.join(".doctrine/spec/tech/001");
        let members = spec_dir.join("members.toml");
        let appended = format!(
            "{}\n[[member]]\nrequirement = \"REQ-001\"\nlabel = \"FR-001\"\norder = 1\n",
            fs::read_to_string(&members).unwrap()
        );
        fs::write(&members, appended).unwrap();
        assert_eq!(member_count(&spec_dir).unwrap(), 1);
    }

    /// Append a raw `[[member]]` row to a spec's seeded `members.toml`, preserving the
    /// seed (mirrors `member_count_reads_appended_rows`'s hand-edit style).
    fn append_raw_member(spec_dir: &Path, requirement: &str, label: &str, order: u32) {
        let members = spec_dir.join("members.toml");
        let appended = format!(
            "{}\n[[member]]\nrequirement = \"{requirement}\"\nlabel = \"{label}\"\norder = {order}\n",
            fs::read_to_string(&members).unwrap()
        );
        fs::write(&members, appended).unwrap();
    }

    /// VT-1 (BLOCKER E2): a non-canonical member FK (`REQ-1`) is canonicalised to
    /// `REQ-001` on the way out — the downstream scan keys on canonical ids, so a raw
    /// FK would silently read observed=none.
    #[test]
    fn member_reqs_canonicalises_the_requirement_fk() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "cli", "CLI"); // SPEC-001
        append_raw_member(&root.join(".doctrine/spec/tech/001"), "REQ-1", "FR-001", 1);

        let reqs = member_reqs(root, "SPEC-001").unwrap();
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].requirement, "REQ-001");
    }

    /// VT-2: out-of-order members come back sorted by advisory `order`, each `label`
    /// carried verbatim (F-A8).
    #[test]
    fn member_reqs_sorts_by_order_and_carries_labels_verbatim() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "cli", "CLI"); // SPEC-001
        let spec_dir = root.join(".doctrine/spec/tech/001");
        // appended out of order: order 2 first, then order 1.
        append_raw_member(&spec_dir, "REQ-002", "FR-002", 2);
        append_raw_member(&spec_dir, "REQ-001", "FR-001", 1);

        let reqs = member_reqs(root, "SPEC-001").unwrap();
        let labels: Vec<&str> = reqs.iter().map(|m| m.label.as_str()).collect();
        let fks: Vec<&str> = reqs.iter().map(|m| m.requirement.as_str()).collect();
        assert_eq!(
            labels,
            ["FR-001", "FR-002"],
            "sorted by order; labels verbatim"
        );
        assert_eq!(fks, ["REQ-001", "REQ-002"]);
    }

    #[test]
    fn list_status_filter_selects_within_a_subtype() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Product, "onboarding", "Onboarding");
        fresh(root, SpecSubtype::Product, "billing", "Billing");
        // flip 002 to active by hand (status is hand-edited in v1).
        let p = root.join(".doctrine/spec/product/002/spec-002.toml");
        let flipped = fs::read_to_string(&p)
            .unwrap()
            .replace("status = \"draft\"", "status = \"active\"");
        fs::write(&p, flipped).unwrap();

        let active = list_rows(
            root,
            ListArgs {
                status: vec!["active".into()],
                ..ListArgs::default()
            },
        )
        .unwrap();
        // default table shows the title (D4 swap); the status filter still
        // selects within the subtype.
        assert!(active.contains("PRD-002 │ active │ Billing"), "{active}");
        assert!(!active.contains("Onboarding"));
    }

    /// Flip a spec's authored `status` on disk (no status verb in v1).
    fn flip_status(root: &Path, subtype: SpecSubtype, id: u32, to: &str) {
        let p = root
            .join(subtype.kind().dir)
            .join(format!("{id:03}"))
            .join(format!("spec-{id:03}.toml"));
        let flipped = fs::read_to_string(&p)
            .unwrap()
            .replace("status = \"draft\"", &format!("status = \"{to}\""));
        fs::write(&p, flipped).unwrap();
    }

    // --- SL-025 EX-1: hide-set {superseded}, prefixed ids, shared flags ---

    #[test]
    fn spec_list_hides_superseded_by_default_and_all_reveals() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Product, "onboarding", "Onboarding");
        fresh(root, SpecSubtype::Product, "billing", "Billing");
        flip_status(root, SpecSubtype::Product, 2, "superseded");

        // default: the superseded spec drops from the list.
        let def = list_rows(root, list_args()).unwrap();
        assert!(def.contains("PRD-001"), "{def}");
        assert!(
            !def.contains("PRD-002"),
            "superseded hidden by default: {def}"
        );

        // --all reveals it.
        let all = list_rows(
            root,
            ListArgs {
                all: true,
                ..ListArgs::default()
            },
        )
        .unwrap();
        assert!(all.contains("PRD-002"), "--all reveals superseded: {all}");

        // an explicit --status superseded also reveals it.
        let explicit = list_rows(
            root,
            ListArgs {
                status: vec!["superseded".into()],
                ..ListArgs::default()
            },
        )
        .unwrap();
        assert!(explicit.contains("PRD-002"), "{explicit}");
        assert!(!explicit.contains("PRD-001"), "{explicit}");
    }

    #[test]
    fn spec_list_filter_matches_slug_and_title_regexp_matches_canonical() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Product, "onboarding", "Onboarding");
        fresh(root, SpecSubtype::Tech, "cli", "CLI");

        // --filter (substr on slug+title) selects the onboarding product spec.
        let by_substr = list_rows(
            root,
            ListArgs {
                substr: Some("onboard".into()),
                ..ListArgs::default()
            },
        )
        .unwrap();
        assert!(by_substr.contains("PRD-001"), "{by_substr}");
        assert!(!by_substr.contains("SPEC-001"), "{by_substr}");

        // --regexp on the canonical id domain selects the tech spec by its prefix.
        let by_regex = list_rows(
            root,
            ListArgs {
                regexp: Some("^SPEC-".into()),
                ..ListArgs::default()
            },
        )
        .unwrap();
        assert!(by_regex.contains("SPEC-001"), "{by_regex}");
        assert!(!by_regex.contains("PRD-001"), "{by_regex}");
    }

    // --- SL-025 EX-1 / A-8: a SINGLE json envelope, subtype per row ---

    #[test]
    fn spec_list_json_is_one_envelope_with_subtype_per_row() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Product, "onboarding", "Onboarding");
        fresh(root, SpecSubtype::Tech, "cli", "CLI");

        let json = list_rows(
            root,
            ListArgs {
                json: true,
                ..ListArgs::default()
            },
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["kind"], "spec", "single envelope keyed `spec`");
        let rows = v["rows"].as_array().expect("rows is an array");
        assert_eq!(rows.len(), 2, "both subtypes in ONE envelope: {json}");

        // each row carries its subtype + the prefixed id + a NUMERIC member count.
        let prd = rows
            .iter()
            .find(|r| r["id"] == "PRD-001")
            .expect("the product row");
        assert_eq!(prd["subtype"], "product");
        assert_eq!(prd["status"], "draft");
        assert_eq!(prd["members"], 0);
        let spec = rows
            .iter()
            .find(|r| r["id"] == "SPEC-001")
            .expect("the tech row");
        assert_eq!(spec["subtype"], "tech");
    }

    // --- SL-025 A-2: --status is validated against the spec known-set ---

    #[test]
    fn spec_list_rejects_an_unknown_status_with_the_uniform_error() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Product, "onboarding", "Onboarding");
        let err = list_rows(
            root,
            ListArgs {
                status: vec!["bogus".into()],
                ..ListArgs::default()
            },
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("bogus"),
            "names the bad value: {err}"
        );
    }

    /// Drift canary: the `SPEC_STATUSES` known-set must stay in lockstep with the
    /// `SpecStatus` enum's kebab serde — adding a variant without the const (or vice
    /// versa) breaks the read-filter coherence (A-2).
    #[test]
    fn spec_statuses_matches_the_variants() {
        let from_variants: Vec<&str> = [
            SpecStatus::Draft,
            SpecStatus::Active,
            SpecStatus::Deprecated,
            SpecStatus::Superseded,
        ]
        .iter()
        .map(|s| s.as_str())
        .collect();
        assert_eq!(from_variants, SPEC_STATUSES.to_vec());
    }

    // --- SL-025 EX-2 / VT-3: spec show --json ---

    #[test]
    fn spec_show_json_is_faithful_toml_as_data_plus_body_and_members() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "cli", "CLI");
        run_req_add(
            Some(root.to_path_buf()),
            "SPEC-001",
            Some("Route subcommands".into()),
            ReqKind::Functional,
            None,
            None,
        )
        .unwrap();

        let spec_dir = root.join(".doctrine/spec/tech/001");
        let spec_toml = spec_dir.join("spec-001.toml");
        let spec: Spec = toml::from_str(&fs::read_to_string(&spec_toml).unwrap()).unwrap();
        let body = fs::read_to_string(spec_dir.join("spec-001.md")).unwrap();
        let members = read_members(&spec_dir.join("members.toml")).unwrap();
        let mut resolved = Vec::new();
        for m in members {
            let req = requirement::load(root, &m.requirement).unwrap();
            resolved.push((m, req));
        }
        let interactions = read_interactions(&spec_dir.join("interactions.toml")).unwrap();

        let json = show_json(&spec, &body, &resolved, &[], &interactions).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["kind"], "spec");
        assert_eq!(v["id"], "SPEC-001");
        assert_eq!(v["spec"]["title"], "CLI");
        assert_eq!(v["spec"]["status"], "draft");
        assert_eq!(v["body"], body, "the prose body is verbatim");
        let mrows = v["members"].as_array().expect("members array");
        assert_eq!(mrows.len(), 1, "the one membered requirement");
        assert_eq!(mrows[0]["label"], "FR-001");
        assert_eq!(mrows[0]["requirement"]["id"], "REQ-001");
        assert_eq!(mrows[0]["requirement"]["title"], "Route subcommands");
    }

    // --- VT-2: the parse structs + tag/source round-trips ---

    #[test]
    fn tags_and_description_round_trip_on_spec() {
        // a rich tech spec toml round-trips into Spec: tags, the open category, the
        // closed c4_level, responsibilities, and the [[source]] anchors (D-3 shape).
        let body = "\
id = 3
slug = \"cli\"
title = \"CLI\"
status = \"active\"
kind = \"tech\"
tags = [\"infra\", \"surface\"]
category = \"cli\"
c4_level = \"container\"
responsibilities = [\"route subcommands\"]

[[source]]
language = \"rust\"
identifier = \"doctrine/cli\"
module = \"doctrine::cli\"
";
        let spec: Spec = toml::from_str(body).unwrap();
        assert_eq!(spec.kind, SpecSubtype::Tech);
        assert_eq!(spec.status, SpecStatus::Active);
        assert_eq!(spec.tags, vec!["infra", "surface"]);
        assert_eq!(spec.category.as_deref(), Some("cli"));
        assert_eq!(spec.c4_level, Some(C4Level::Container));
        assert_eq!(spec.responsibilities, vec!["route subcommands"]);
        assert_eq!(spec.sources.len(), 1);
        assert_eq!(spec.sources[0].language, "rust");
        assert_eq!(spec.sources[0].module.as_deref(), Some("doctrine::cli"));
        // the spine fields are absent here → None (the at-rest default, VT-1).
        assert_eq!(spec.descends_from, None);
        assert_eq!(spec.parent, None);

        // C2: the same toml deserialises into the shared Meta (the `title` proof).
        let m: Meta = toml::from_str(body).unwrap();
        assert_eq!(m.title, "CLI");
    }

    // --- SL-065 PHASE-02 / VT-1: ProductLevel parse round-trip ---

    #[test]
    fn product_level_kebab_round_trips_every_variant() {
        // each variant ↔ its kebab string, both directions (as_str + serde parse).
        for (variant, kebab) in [
            (ProductLevel::Domain, "domain"),
            (ProductLevel::Capability, "capability"),
            (ProductLevel::Feature, "feature"),
            (ProductLevel::Story, "story"),
        ] {
            assert_eq!(variant.as_str(), kebab);
            let body = format!(
                "id = 1\nslug = \"x\"\ntitle = \"X\"\nstatus = \"draft\"\nkind = \"product\"\nproduct_level = \"{kebab}\"\n"
            );
            let spec: Spec = toml::from_str(&body).unwrap();
            assert_eq!(spec.product_level, Some(variant));
        }
    }

    #[test]
    fn product_level_rejects_unknown_variant_at_parse() {
        // an out-of-set value is a parse error (closed enum, serde-enforced).
        let body = "id = 1\nslug = \"x\"\ntitle = \"X\"\nstatus = \"draft\"\nkind = \"product\"\nproduct_level = \"epic\"\n";
        assert!(toml::from_str::<Spec>(body).is_err());
    }

    #[test]
    fn product_level_absent_defaults_to_none() {
        // #[serde(default)]: an unlabelled product spec parses with product_level None.
        let body = "id = 1\nslug = \"x\"\ntitle = \"X\"\nstatus = \"draft\"\nkind = \"product\"\n";
        let spec: Spec = toml::from_str(body).unwrap();
        assert_eq!(spec.product_level, None);
    }

    // --- PHASE-03 (SL-022) T6: pin the second_parent error classifier (R2) ---
    // The match rides `toml::de::Error::{span,message}` (toml 0.8) and is version-
    // fragile by construction; these tests are the canary if a toml bump shifts it.

    const SPEC_BASE: &str =
        "id = 1\nslug = \"x\"\ntitle = \"X\"\nstatus = \"draft\"\nkind = \"tech\"\ntags = []\n";

    fn classify(doc: &str) -> bool {
        let err = toml::from_str::<Spec>(doc).unwrap_err();
        is_second_parent(&err, doc)
    }

    #[test]
    fn second_parent_classifier_matches_duplicate_parent() {
        assert!(classify(&format!(
            "{SPEC_BASE}parent = \"SPEC-001\"\nparent = \"SPEC-002\"\n"
        )));
    }

    #[test]
    fn second_parent_classifier_matches_array_parent() {
        assert!(classify(&format!(
            "{SPEC_BASE}parent = [\"SPEC-001\", \"SPEC-002\"]\n"
        )));
    }

    #[test]
    fn second_parent_classifier_ignores_unrelated_parse_errors() {
        // A scalar wrong-type that is not a multi-parent attempt → falls through.
        assert!(!classify(&format!("{SPEC_BASE}parent = 5\n")));
        // A duplicate of a different key → not attributed to `parent`.
        assert!(!classify(&format!(
            "{SPEC_BASE}category = \"a\"\ncategory = \"b\"\n"
        )));
        // An array given to a different string field → span is not the parent line.
        assert!(!classify(
            "id = 1\nslug = []\ntitle = \"X\"\nstatus = \"draft\"\nkind = \"tech\"\ntags = []\n"
        ));
    }

    #[test]
    fn product_spec_toml_defaults_tech_flat_fields() {
        // the light product identity parses; tech flat fields default empty/absent.
        let body = "\
id = 1
slug = \"onboarding\"
title = \"Onboarding\"
status = \"draft\"
kind = \"product\"
tags = []
";
        let spec: Spec = toml::from_str(body).unwrap();
        assert_eq!(spec.kind, SpecSubtype::Product);
        assert_eq!(spec.category, None);
        assert_eq!(spec.c4_level, None);
        assert!(spec.responsibilities.is_empty());
        assert!(spec.sources.is_empty());
        assert_eq!(spec.descends_from, None);
        assert_eq!(spec.parent, None);
    }

    #[test]
    fn tech_spec_parses_descent_and_parent_when_present() {
        // the two outbound spine fields (VT-1): present → Some, stored verbatim.
        let body = "\
id = 1
slug = \"cli\"
title = \"CLI\"
status = \"active\"
kind = \"tech\"
descends_from = \"PRD-001\"
parent = \"SPEC-002\"
";
        let spec: Spec = toml::from_str(body).unwrap();
        assert_eq!(spec.descends_from.as_deref(), Some("PRD-001"));
        assert_eq!(spec.parent.as_deref(), Some("SPEC-002"));
    }

    #[test]
    fn member_and_interaction_parse_layer_round_trips() {
        // the edge parse structs (consumed in P3/P5) parse their row shapes now.
        let m: Member =
            toml::from_str("requirement = \"REQ-007\"\nlabel = \"FR-001\"\norder = 2\n").unwrap();
        assert_eq!(m.requirement, "REQ-007");
        assert_eq!(m.label, "FR-001");
        assert_eq!(m.order, 2);

        let i: Interaction =
            toml::from_str("target = \"SPEC-002\"\ntype = \"uses\"\nnotes = \"x\"\n").unwrap();
        assert_eq!(i.target, "SPEC-002");
        assert_eq!(i.kind, "uses"); // `type` → kind
        assert_eq!(i.notes.as_deref(), Some("x"));
    }

    #[test]
    fn seeded_members_toml_parses_to_zero() {
        // the comment-only template is valid toml and yields no members.
        let doc: MembersDoc = toml::from_str(&members_seed().unwrap()).unwrap();
        assert!(doc.member.is_empty());
    }

    // --- PHASE-03 VT-2: the canonical-ref resolver + label/order ---

    #[test]
    fn req_add_resolver_rejects_bare_numeric() {
        assert_eq!(
            resolve_spec_ref("PRD-7").unwrap(),
            (SpecSubtype::Product, 7)
        );
        assert_eq!(
            resolve_spec_ref("SPEC-012").unwrap(),
            (SpecSubtype::Tech, 12)
        );
        // bare numeric is ambiguous across the two namespaces → rejected (C4).
        assert!(resolve_spec_ref("7").is_err());
        // wrong/unknown prefix, and a non-numeric tail.
        assert!(resolve_spec_ref("REQ-1").is_err());
        assert!(resolve_spec_ref("PRD-x").is_err());
    }

    #[test]
    fn next_label_and_order_fill_per_kind_independently() {
        let members = vec![
            Member {
                requirement: "REQ-001".into(),
                label: "FR-001".into(),
                order: 1,
            },
            Member {
                requirement: "REQ-002".into(),
                label: "FR-002".into(),
                order: 2,
            },
            Member {
                requirement: "REQ-003".into(),
                label: "NF-001".into(),
                order: 3,
            },
        ];
        assert_eq!(next_label(&members, ReqKind::Functional), "FR-003");
        assert_eq!(next_label(&members, ReqKind::Quality), "NF-002");
        assert_eq!(next_label(&[], ReqKind::Functional), "FR-001");
        assert_eq!(next_order(&members), 4);
        assert_eq!(next_order(&[]), 1);
    }

    // --- PHASE-03 VT-1 / VT-2 / VT-3: the two-tree write end-to-end ---

    #[test]
    fn spec_req_add_reserves_requirement_and_appends_member() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Product, "onboarding", "Onboarding");

        run_req_add(
            Some(root.to_path_buf()),
            "PRD-001",
            Some("User can sign up".into()),
            ReqKind::Functional,
            None,
            None,
        )
        .unwrap();

        // a requirement was reserved in its own tree, kind overwritten (D-1).
        let req_toml = root.join(".doctrine/requirement/001/requirement-001.toml");
        assert!(req_toml.is_file());
        assert!(
            fs::read_to_string(&req_toml)
                .unwrap()
                .contains("kind = \"functional\"")
        );

        // the membership row carries FK + auto label + order.
        let members = read_members(&root.join(".doctrine/spec/product/001/members.toml")).unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].requirement, "REQ-001");
        assert_eq!(members[0].label, "FR-001");
        assert_eq!(members[0].order, 1);
    }

    #[test]
    fn spec_req_add_is_edit_preserving() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Product, "onboarding", "Onboarding");

        // hand-edit the seeded members.toml: a comment + an unknown top-level key.
        let members_path = root.join(".doctrine/spec/product/001/members.toml");
        let seeded = fs::read_to_string(&members_path).unwrap();
        fs::write(
            &members_path,
            format!("{seeded}\n# hand-added note\nschema_hint = \"survives\"\n"),
        )
        .unwrap();

        run_req_add(
            Some(root.to_path_buf()),
            "PRD-001",
            Some("X".into()),
            ReqKind::Functional,
            None,
            None,
        )
        .unwrap();

        let after = fs::read_to_string(&members_path).unwrap();
        // comment + unknown key survive the append (toml_edit, not reserialize) …
        assert!(after.contains("# hand-added note"));
        assert!(after.contains("schema_hint = \"survives\""));
        // … and the new row is present.
        assert!(after.contains("[[member]]"));
        assert!(after.contains("requirement = \"REQ-001\""));
    }

    #[test]
    fn spec_req_add_auto_labels_fr_then_nf_by_kind() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "cli", "CLI");
        let add = |title: &str, kind: ReqKind, label: Option<&str>| {
            run_req_add(
                Some(root.to_path_buf()),
                "SPEC-001",
                Some(title.into()),
                kind,
                label.map(str::to_string),
                None,
            )
            .unwrap();
        };
        add("route subcommands", ReqKind::Functional, None); // FR-001
        add("parse flags", ReqKind::Functional, None); // FR-002
        add("fast startup", ReqKind::Quality, None); // NF-001
        add("explicit", ReqKind::Functional, Some("FR-099")); // override honoured

        let members = read_members(&root.join(".doctrine/spec/tech/001/members.toml")).unwrap();
        let labels: Vec<&str> = members.iter().map(|m| m.label.as_str()).collect();
        assert_eq!(labels, vec!["FR-001", "FR-002", "NF-001", "FR-099"]);
        // each reserved a distinct REQ-NNN in order.
        let fks: Vec<&str> = members.iter().map(|m| m.requirement.as_str()).collect();
        assert_eq!(fks, vec!["REQ-001", "REQ-002", "REQ-003", "REQ-004"]);
        // D-1: the quality requirement's kind was overwritten off the functional seed.
        let q = fs::read_to_string(root.join(".doctrine/requirement/003/requirement-003.toml"))
            .unwrap();
        assert!(q.contains("kind = \"quality\""));
    }

    #[test]
    #[cfg(unix)]
    fn spec_req_add_orphan_on_append_failure_left_uncommitted() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Product, "onboarding", "Onboarding");

        // Make the append target's DIRECTORY read-only. The label/order scan still
        // reads members.toml (valid seed) and the requirement reserve (a different
        // dir) succeeds, but the member append's temp+rename in this dir fails → no
        // member row is written. NB: since SL-113 the append goes through
        // `write_atomic`, which renames over a read-only *file* fine (rename keys on
        // *directory* write perm) — so the old 0o444-on-the-file trick no longer
        // induces failure; the directory must be made unwritable instead.
        let members_path = root.join(".doctrine/spec/product/001/members.toml");
        let members_dir = root.join(".doctrine/spec/product/001");
        let mut perms = fs::metadata(&members_dir).unwrap().permissions();
        perms.set_mode(0o555);
        fs::set_permissions(&members_dir, perms).unwrap();

        let err = run_req_add(
            Some(root.to_path_buf()),
            "PRD-001",
            Some("X".into()),
            ReqKind::Functional,
            None,
            None,
        );

        // Restore writability so the tempdir can be reaped on drop.
        let mut perms = fs::metadata(&members_dir).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&members_dir, perms).unwrap();

        assert!(
            err.is_err(),
            "append must fail when its directory is read-only"
        );

        // the reserved requirement is an orphan: present (uncommitted), no member row.
        assert!(
            root.join(".doctrine/requirement/001/requirement-001.toml")
                .is_file()
        );
        let members = read_members(&members_path).unwrap();
        assert!(members.is_empty(), "no partial member row written");
    }

    // --- PHASE-01: read_spec reader ---

    #[test]
    fn read_spec_returns_parsed_toml_and_prose_body() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let mat = fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
        let id = mat.eid.numeric_id().unwrap();
        let (spec, raw_toml, prose) = read_spec(SpecSubtype::Tech, root, id).unwrap();

        // Parsed fields from TOML.
        assert_eq!(spec.id, id);
        assert_eq!(spec.slug, "auth");
        assert_eq!(spec.title, "Auth");
        assert_eq!(spec.status, SpecStatus::Draft);
        assert_eq!(spec.kind, SpecSubtype::Tech);

        // Raw TOML is parse-able (round-trips).
        assert!(raw_toml.contains("Auth"));
        assert!(raw_toml.contains("draft"));
        toml::from_str::<Spec>(&raw_toml).unwrap();

        // Prose body is the scaffolded markdown.
        assert!(prose.contains("# SPEC-001: Auth"));
    }

    #[test]
    fn read_spec_missing_toml_errors() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // Never created the spec — root is empty.
        let err = read_spec(SpecSubtype::Tech, root, 1).unwrap_err();
        assert!(
            err.to_string().contains("Failed to read"),
            "missing spec errors with 'Failed to read'"
        );
    }

    // --- PHASE-04: the pure render compose fn (VT-1 / VT-3) ---

    /// A `Requirement` fixture for the pure render tests — `description` doubles as
    /// the rendered "statement" (D-P4-1).
    fn req(id: u32, title: &str, kind: ReqKind) -> Requirement {
        Requirement {
            id,
            title: title.to_string(),
            slug: title.to_lowercase().replace(' ', "-"),
            status: ReqStatus::Active,
            kind,
            description: Some(format!("{title} statement")),
            tags: Vec::new(),
            acceptance_criteria: Vec::new(),
        }
    }

    fn member(fk: &str, label: &str, order: u32) -> Member {
        Member {
            requirement: fk.to_string(),
            label: label.to_string(),
            order,
        }
    }

    fn tech_spec(id: u32) -> Spec {
        Spec {
            id,
            slug: "cli".to_string(),
            title: "CLI".to_string(),
            status: SpecStatus::Active,
            kind: SpecSubtype::Tech,
            tags: Vec::new(),
            category: None,
            c4_level: None,
            product_level: None,
            responsibilities: Vec::new(),
            sources: Vec::new(),
            descends_from: None,
            parent: None,
        }
    }

    #[test]
    fn render_reassembles_members_in_order() {
        let spec = tech_spec(7);
        // input order 3,1,2 — render must sort by advisory `order`.
        let members = vec![
            (
                member("REQ-003", "FR-003", 3),
                req(3, "Third", ReqKind::Functional),
            ),
            (
                member("REQ-001", "FR-001", 1),
                req(1, "First", ReqKind::Functional),
            ),
            (
                member("REQ-002", "NF-001", 2),
                req(2, "Second", ReqKind::Quality),
            ),
        ];
        let out = render(&spec, "## Body\n\nverbatim prose\n", &members, &[], &[]);

        // structured identity (single non-H1 line) + prose body verbatim.
        assert!(out.starts_with("`SPEC-007` — CLI\n"));
        assert!(out.contains("cli · active · tech"));
        assert!(out.contains("## Body"));
        assert!(out.contains("verbatim prose"));
        // render emits no H1 of its own — the sole H1 (when present) is the prose's.
        // This `## Body` fixture has none, so the total is zero (no synthetic dup).
        assert_eq!(
            out.matches("\n# ").count() + usize::from(out.starts_with("# ")),
            0
        );

        // headings sorted by order; FK derived from req.id; shape per §5.4.
        let h1 = out.find("### FR-001 (REQ-001) — First").unwrap();
        let h2 = out.find("### NF-001 (REQ-002) — Second").unwrap();
        let h3 = out.find("### FR-003 (REQ-003) — Third").unwrap();
        assert!(
            h1 < h2 && h2 < h3,
            "members render sorted by order, not input order"
        );
        // the per-requirement facet line + statement (from description).
        assert!(out.contains("first · functional · active"));
        assert!(out.contains("First statement"));
        // no interactions block when the slice is empty.
        assert!(!out.contains("## Interactions"));
    }

    #[test]
    fn render_includes_tech_flat_fields_and_requirement_facets() {
        let spec = Spec {
            tags: vec!["infra".to_string()],
            category: Some("cli".to_string()),
            c4_level: Some(C4Level::Container),
            responsibilities: vec!["route subcommands".to_string()],
            sources: vec![Source {
                language: "rust".to_string(),
                identifier: "doctrine/cli".to_string(),
                module: Some("doctrine::cli".to_string()),
            }],
            ..tech_spec(1)
        };
        let mut r = req(1, "Route", ReqKind::Functional);
        r.tags = vec!["core".to_string()];
        r.acceptance_criteria = vec!["dispatch works".to_string()];
        let members = vec![(member("REQ-001", "FR-001", 1), r)];

        let out = render(&spec, "## Overview\n", &members, &[], &[]);
        // every tech flat field renders (un-deads Spec/SpecStatus/C4Level/Source).
        assert!(out.contains("tags: infra"));
        assert!(out.contains("category: cli"));
        assert!(out.contains("c4 level: container"));
        assert!(out.contains("  - route subcommands"));
        assert!(out.contains("  - rust doctrine/cli (doctrine::cli)"));
        // requirement facets: tags, statement, acceptance criteria.
        assert!(out.contains("tags: core"));
        assert!(out.contains("Route statement"));
        assert!(out.contains("  - dispatch works"));
    }

    #[test]
    fn render_omits_statement_line_when_description_absent() {
        let spec = tech_spec(1);
        let mut r = req(1, "Bare", ReqKind::Functional);
        r.description = None; // no statement (D-P4-1: absent → no line)
        let members = vec![(member("REQ-001", "FR-001", 1), r)];
        let out = render(&spec, "p\n", &members, &[], &[]);
        assert!(out.contains("### FR-001 (REQ-001) — Bare"));
        assert!(!out.contains("statement"));
    }

    #[test]
    fn render_emits_outbound_interactions_for_tech_omits_when_empty() {
        let spec = tech_spec(1);
        let edges = vec![
            Interaction {
                target: "SPEC-002".to_string(),
                kind: "uses".to_string(),
                notes: Some("calls boot".to_string()),
            },
            Interaction {
                target: "SPEC-003".to_string(),
                kind: "extends".to_string(),
                notes: None,
            },
        ];
        let with = render(&spec, "p\n", &[], &[], &edges);
        assert!(with.contains("## Interactions"));
        assert!(with.contains("- SPEC-002 — uses: calls boot"));
        assert!(with.contains("- SPEC-003 — extends\n"));

        // empty (product spec or a tech spec with zero edges) → block omitted.
        let without = render(&spec, "p\n", &[], &[], &[]);
        assert!(!without.contains("## Interactions"));
    }

    #[test]
    fn render_emits_descent_and_parent_for_tech_in_order() {
        // VT-2: tech emits both lines, ordered c4 → descends → parent → resp → sources.
        let spec = Spec {
            c4_level: Some(C4Level::Component),
            descends_from: Some("PRD-001".to_string()),
            parent: Some("SPEC-002".to_string()),
            responsibilities: vec!["route".to_string()],
            ..tech_spec(1)
        };
        let out = render(&spec, "p\n", &[], &[], &[]);
        assert!(out.contains("descends from: PRD-001\n"));
        assert!(out.contains("parent: SPEC-002\n"));
        // no derived children line ever (ADR-004 §3, outbound-only).
        assert!(!out.contains("children"));
        // strict order: c4 < descends < parent < responsibilities < sources(absent).
        let c4 = out.find("c4 level:").unwrap();
        let descends = out.find("descends from:").unwrap();
        let parent = out.find("parent:").unwrap();
        let resp = out.find("responsibilities:").unwrap();
        assert!(c4 < descends && descends < parent && parent < resp);
    }

    #[test]
    fn render_omits_descent_and_parent_when_none_and_for_product() {
        // VT-2: tech with both None → neither line.
        let tech = tech_spec(1);
        let out = render(&tech, "p\n", &[], &[], &[]);
        assert!(!out.contains("descends from:"));
        assert!(!out.contains("\nparent:"));

        // product subject carrying an (invalid, at-rest) descends_from → the product
        // arm never renders descends_from; parent absent → no parent line either.
        let product = Spec {
            kind: SpecSubtype::Product,
            descends_from: Some("PRD-001".to_string()),
            parent: None,
            ..tech_spec(1)
        };
        let pout = render(&product, "p\n", &[], &[], &[]);
        assert!(!pout.contains("descends from:"));
        assert!(!pout.contains("parent:"));
    }

    #[test]
    fn render_emits_product_level_and_parent_for_product_in_order() {
        // VT-2: product subject emits `product level:` then `parent:`, in that order.
        let spec = Spec {
            kind: SpecSubtype::Product,
            product_level: Some(ProductLevel::Capability),
            parent: Some("PRD-003".to_string()),
            ..tech_spec(1)
        };
        let out = render(&spec, "p\n", &[], &[], &[]);
        assert!(out.contains("product level: capability\n"));
        assert!(out.contains("parent: PRD-003\n"));
        // reciprocal children are derived, never rendered (ADR-004 §3).
        assert!(!out.contains("children"));
        let level = out.find("product level:").unwrap();
        let parent = out.find("parent:").unwrap();
        assert!(level < parent);
    }

    #[test]
    fn render_omits_c4_level_on_a_product_spec() {
        // design §5 F1: a product spec illegitimately carrying c4_level no longer
        // renders `c4 level:` — it falls outside the tech branch.
        let spec = Spec {
            kind: SpecSubtype::Product,
            c4_level: Some(C4Level::Container),
            ..tech_spec(1)
        };
        let out = render(&spec, "p\n", &[], &[], &[]);
        assert!(!out.contains("c4 level:"));
    }

    // --- FIX 2: build_registry canonicalizes non-canonical author-supplied FKs ---

    /// Non-canonical FKs in hand-authored `members.toml` and `interactions.toml`
    /// must be canonicalized by `build_registry` so that `Registry::validate` can
    /// resolve them against the canonical id sets. Genuinely unresolvable junk must
    /// still be stored verbatim (and flagged dangling by validate).
    #[test]
    fn build_registry_canonicalizes_member_and_interaction_fks() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "a", "Spec A"); // SPEC-001
        fresh(root, SpecSubtype::Tech, "b", "Spec B"); // SPEC-002
        requirement::reserve(root, "x", "X", "2026-06-05").unwrap(); // REQ-001

        // Hand-author a non-canonical member FK ("REQ-1" instead of "REQ-001").
        let members_path = root.join(".doctrine/spec/tech/001/members.toml");
        append_member(&members_path, "REQ-1", "FR-001", 1).unwrap();

        // Hand-author a non-canonical interaction target ("SPEC-2" instead of "SPEC-002").
        let ix_path = root.join(".doctrine/spec/tech/001/interactions.toml");
        let seeded = fs::read_to_string(&ix_path).unwrap();
        fs::write(
            &ix_path,
            format!("{seeded}\n[[edge]]\ntarget = \"SPEC-2\"\ntype = \"calls\"\n"),
        )
        .unwrap();

        let reg = build_registry(root).unwrap();

        // Both edges must be stored in canonical form after registry build.
        let member_edge = reg
            .members
            .iter()
            .find(|m| m.spec == "SPEC-001")
            .expect("member edge for SPEC-001");
        assert_eq!(
            member_edge.requirement, "REQ-001",
            "non-canonical REQ-1 must be canonicalized to REQ-001"
        );

        let ix_edge = reg
            .interactions
            .iter()
            .find(|e| e.spec == "SPEC-001")
            .expect("interaction edge for SPEC-001");
        assert_eq!(
            ix_edge.target, "SPEC-002",
            "non-canonical SPEC-2 must be canonicalized to SPEC-002"
        );

        // validate must report no findings — the corpus is internally consistent.
        let findings = reg.validate(None);
        assert!(
            findings.is_empty(),
            "non-canonical-but-valid FKs must not produce dangling findings: {findings:?}"
        );
    }

    // --- PHASE-02 (SL-022) Layer C: build_registry harvests the relational spine ---

    /// Append fields to a scaffolded `spec-NNN.toml` (flat keys at top level).
    fn append_spec_fields(path: &Path, lines: &str) {
        let seeded = fs::read_to_string(path).unwrap();
        fs::write(path, format!("{seeded}\n{lines}\n")).unwrap();
    }

    #[test]
    fn build_registry_harvests_product_set_and_relational_edges() {
        // VT-4 Layer C(i): a well-formed corpus → product ids + parent/descent edges
        // with the right `on_product` flag.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
        fresh(root, SpecSubtype::Tech, "store", "Store"); // SPEC-002
        fresh(root, SpecSubtype::Product, "login", "Login"); // PRD-001
        append_spec_fields(
            &root.join(".doctrine/spec/tech/001/spec-001.toml"),
            "descends_from = \"PRD-001\"\nparent = \"SPEC-002\"",
        );

        let reg = build_registry(root).unwrap();

        assert!(
            reg.product_specs.contains("PRD-001"),
            "product id is collected into product_specs"
        );
        let descent = reg
            .descents
            .iter()
            .find(|e| e.spec == "SPEC-001")
            .expect("descent edge for SPEC-001");
        assert_eq!(descent.target, "PRD-001");
        assert!(!descent.on_product, "tech subject → on_product false");
        let parent = reg
            .parents
            .iter()
            .find(|e| e.spec == "SPEC-001")
            .expect("parent edge for SPEC-001");
        assert_eq!(parent.parent, "SPEC-002");
        assert!(!parent.on_product);

        // The corpus is internally consistent (tech→product descent, tech parent).
        assert!(
            reg.validate(None).is_empty(),
            "well-formed spine produces no findings: {:?}",
            reg.validate(None)
        );
    }

    #[test]
    fn build_registry_surfaces_a_malformed_spec_toml() {
        // VT-4 Layer C(iv): the new per-spec parse (Charge I) widens the error
        // surface — a malformed `spec-NNN.toml`, invisible to `validate` before
        // SL-022, now fails the build.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
        let spec_toml = root.join(".doctrine/spec/tech/001/spec-001.toml");
        fs::write(&spec_toml, "this is not = = valid toml").unwrap();

        let result = build_registry(root);
        assert!(result.is_err(), "malformed spec toml must fail the build");
        let err = result.err().unwrap();
        assert!(
            err.to_string().contains("Failed to parse"),
            "malformed spec toml surfaces as a parse error: {err}"
        );
    }

    // --- PHASE-03 (SL-022) Layer C: second_parent end-to-end (VT-2 / VT-3) ---

    /// Assert a single-tech-spec corpus carrying `parent_lines` surfaces the named
    /// second-parent finding through `validate` AND a non-zero `run_validate` exit
    /// (REQ-087 AC1 + AC3, proven end-to-end — not at `toml::from_str` level).
    fn assert_second_parent_end_to_end(parent_lines: &str) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
        append_spec_fields(
            &root.join(".doctrine/spec/tech/001/spec-001.toml"),
            parent_lines,
        );

        let reg = build_registry(root).unwrap();
        let findings = reg.validate(None);
        assert!(
            findings
                .iter()
                .any(|f| f.contains("second parent") && f.contains("SPEC-001")),
            "validate surfaces the named second-parent finding: {findings:?}"
        );
        assert!(
            run_validate(Some(root.to_path_buf()), None).is_err(),
            "run_validate exits non-zero on a second-parent corpus"
        );
    }

    #[test]
    fn second_parent_duplicate_key_surfaces_end_to_end() {
        // VT-2: a duplicate `parent` key → carried finding + non-zero exit.
        assert_second_parent_end_to_end("parent = \"SPEC-002\"\nparent = \"SPEC-003\"");
    }

    #[test]
    fn second_parent_array_value_surfaces_end_to_end() {
        // VT-2: an array-valued `parent` → carried finding + non-zero exit.
        assert_second_parent_end_to_end("parent = [\"SPEC-002\", \"SPEC-003\"]");
    }

    #[test]
    fn scaffold_commented_parent_does_not_trip_second_parent() {
        // VT-3 (codex F2 regression): a freshly-scaffolded tech spec ships a commented
        // `# parent = …` example. Classifying the PARSE error (not scanning raw text)
        // means the comment can never trip the finding, and the corpus validates clean.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001, commented # parent

        let reg = build_registry(root).unwrap();
        assert!(
            reg.build_findings.is_empty(),
            "commented # parent is not a finding"
        );
        assert!(
            reg.validate(None).is_empty(),
            "a clean scaffolded corpus has no findings: {:?}",
            reg.validate(None)
        );
    }

    // --- SL-022 PHASE-04: cross-cutting validation sweep (VT-1) ---
    //
    // Each hard violation, proven NON-ZERO end-to-end through `run_validate` (the
    // function backing `doctrine spec validate`) over a minimal crafted corpus —
    // the integration the Layer A pure-check tests (registry.rs) bypass. One corpus
    // per violation (not a mega-corpus): VT-1 reads "each crafted hard violation →
    // non-zero", so per-violation granularity proves each INDEPENDENTLY trips the
    // bail and attributes the exit to the right check. Second-parent is already
    // proven end-to-end above (`second_parent_*_surfaces_end_to_end`) — referenced
    // here so the matrix reads complete, not re-proven. A clean corpus → zero closes
    // the sweep.

    /// Absolute path to a spec's `spec-NNN.toml`, via the production tree convention.
    fn spec_toml(root: &Path, subtype: SpecSubtype, id: u32) -> PathBuf {
        root.join(subtype.kind().dir)
            .join(format!("{id:03}"))
            .join(format!("spec-{id:03}.toml"))
    }

    /// Append a hand-authored interaction `[[edge]]` to a tech spec (no producer
    /// verb in v1 — mirrors the `build_registry_scans_all_three_trees` fixture).
    fn append_interaction(root: &Path, tech_id: u32, target: &str) {
        let p = root
            .join(SpecSubtype::Tech.kind().dir)
            .join(format!("{tech_id:03}"))
            .join("interactions.toml");
        let seeded = fs::read_to_string(&p).unwrap();
        fs::write(
            &p,
            format!("{seeded}\n[[edge]]\ntarget = \"{target}\"\ntype = \"uses\"\n"),
        )
        .unwrap();
    }

    /// Build a corpus via `build`, then assert `run_validate` exits NON-ZERO and the
    /// surfaced finding names `expect_substr` — proving the intended check fired, not
    /// merely that some error did.
    fn assert_validate_flags(build: impl Fn(&Path), expect_substr: &str) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        build(root);
        let findings = build_registry(root).unwrap().validate(None);
        assert!(
            findings.iter().any(|f| f.contains(expect_substr)),
            "expected a finding containing {expect_substr:?}, got {findings:?}"
        );
        assert!(
            run_validate(Some(root.to_path_buf()), None).is_err(),
            "run_validate exits non-zero on a {expect_substr:?} corpus"
        );
    }

    #[test]
    fn sweep_descent_dangling() {
        assert_validate_flags(
            |root| {
                fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
                append_spec_fields(
                    &spec_toml(root, SpecSubtype::Tech, 1),
                    "descends_from = \"PRD-099\"", // no such product
                );
            },
            "dangling descent:",
        );
    }

    #[test]
    fn sweep_descent_invalid_kind_tech_target() {
        assert_validate_flags(
            |root| {
                fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
                fresh(root, SpecSubtype::Tech, "store", "Store"); // SPEC-002
                append_spec_fields(
                    &spec_toml(root, SpecSubtype::Tech, 1),
                    "descends_from = \"SPEC-002\"", // a tech spec, must be product
                );
            },
            "which is a tech spec (must be product)",
        );
    }

    #[test]
    fn sweep_descent_on_product_subject() {
        assert_validate_flags(
            |root| {
                fresh(root, SpecSubtype::Product, "login", "Login"); // PRD-001
                append_spec_fields(
                    &spec_toml(root, SpecSubtype::Product, 1),
                    "descends_from = \"PRD-002\"", // tech-only field on a product
                );
            },
            "invalid descent: descends_from on product",
        );
    }

    #[test]
    fn sweep_parent_dangling() {
        assert_validate_flags(
            |root| {
                fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
                append_spec_fields(
                    &spec_toml(root, SpecSubtype::Tech, 1),
                    "parent = \"SPEC-099\"", // no such tech spec
                );
            },
            "dangling parent:",
        );
    }

    #[test]
    fn sweep_parent_invalid_kind_product_target() {
        assert_validate_flags(
            |root| {
                fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
                fresh(root, SpecSubtype::Product, "login", "Login"); // PRD-001
                append_spec_fields(
                    &spec_toml(root, SpecSubtype::Tech, 1),
                    "parent = \"PRD-001\"", // a product spec, must be tech
                );
            },
            "is a product spec (must be tech)",
        );
    }

    #[test]
    fn sweep_parent_product_to_tech_is_invalid_kind() {
        // SL-065 §4: parent is now symmetric same-subtype. A product subject whose
        // parent resolves to a TECH spec is invalid-kind (mirror of the tech→product
        // case), no longer rejected as a "tech-only field".
        assert_validate_flags(
            |root| {
                fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
                fresh(root, SpecSubtype::Product, "login", "Login"); // PRD-001
                append_spec_fields(
                    &spec_toml(root, SpecSubtype::Product, 1),
                    "parent = \"SPEC-001\"", // a tech spec, must be product
                );
            },
            "is a tech spec (must be product)",
        );
    }

    #[test]
    fn sweep_self_parent() {
        assert_validate_flags(
            |root| {
                fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
                append_spec_fields(
                    &spec_toml(root, SpecSubtype::Tech, 1),
                    "parent = \"SPEC-001\"", // A → A
                );
            },
            "names itself as parent",
        );
    }

    #[test]
    fn sweep_parent_cycle() {
        assert_validate_flags(
            |root| {
                fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
                fresh(root, SpecSubtype::Tech, "store", "Store"); // SPEC-002
                append_spec_fields(
                    &spec_toml(root, SpecSubtype::Tech, 1),
                    "parent = \"SPEC-002\"",
                );
                append_spec_fields(
                    &spec_toml(root, SpecSubtype::Tech, 2),
                    "parent = \"SPEC-001\"",
                );
            },
            "parent cycle:",
        );
    }

    #[test]
    fn sweep_parent_product_to_product_is_clean() {
        // SL-065 §4: a product spec may now decompose into another product spec.
        // The well-formed PRD→PRD spine produces no finding and exits zero.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Product, "login", "Login"); // PRD-001
        fresh(root, SpecSubtype::Product, "accounts", "Accounts"); // PRD-002 (root)
        append_spec_fields(
            &spec_toml(root, SpecSubtype::Product, 1),
            "parent = \"PRD-002\"",
        );
        assert!(
            build_registry(root).unwrap().validate(None).is_empty(),
            "a well-formed product spine produces no findings"
        );
        assert!(
            run_validate(Some(root.to_path_buf()), None).is_ok(),
            "run_validate exits zero on a clean product spine"
        );
    }

    #[test]
    fn sweep_parent_product_dangling() {
        // SL-065 §4: a product parent that resolves to no product spec is dangling.
        assert_validate_flags(
            |root| {
                fresh(root, SpecSubtype::Product, "login", "Login"); // PRD-001
                append_spec_fields(
                    &spec_toml(root, SpecSubtype::Product, 1),
                    "parent = \"PRD-099\"", // no such product spec
                );
            },
            "dangling parent:",
        );
    }

    #[test]
    fn sweep_self_parent_product() {
        // SL-065 §4: acyclicity is subtype-blind — a product naming itself is caught.
        assert_validate_flags(
            |root| {
                fresh(root, SpecSubtype::Product, "login", "Login"); // PRD-001
                append_spec_fields(
                    &spec_toml(root, SpecSubtype::Product, 1),
                    "parent = \"PRD-001\"", // A → A
                );
            },
            "names itself as parent",
        );
    }

    #[test]
    fn sweep_parent_cycle_product() {
        // SL-065 §4: a multi-hop product cycle (PRD-001 → PRD-002 → PRD-001) is caught
        // by the now subtype-blind parent_cycle walk.
        assert_validate_flags(
            |root| {
                fresh(root, SpecSubtype::Product, "login", "Login"); // PRD-001
                fresh(root, SpecSubtype::Product, "accounts", "Accounts"); // PRD-002
                append_spec_fields(
                    &spec_toml(root, SpecSubtype::Product, 1),
                    "parent = \"PRD-002\"",
                );
                append_spec_fields(
                    &spec_toml(root, SpecSubtype::Product, 2),
                    "parent = \"PRD-001\"",
                );
            },
            "parent cycle:",
        );
    }

    #[test]
    fn sweep_interaction_dangling() {
        assert_validate_flags(
            |root| {
                fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
                append_interaction(root, 1, "SPEC-099"); // no such tech spec
            },
            "dangling interaction target:",
        );
    }

    #[test]
    fn sweep_interaction_invalid_kind_product_target() {
        assert_validate_flags(
            |root| {
                fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
                fresh(root, SpecSubtype::Product, "login", "Login"); // PRD-001
                append_interaction(root, 1, "PRD-001"); // a product spec, must be tech
            },
            "is a product spec (must be tech)",
        );
    }

    #[test]
    fn sweep_clean_corpus_exits_zero() {
        // VT-1 closing case: a well-formed spine — tech descends_from a product,
        // tech parent a tech root, a valid tech→tech interaction — exits ZERO.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
        fresh(root, SpecSubtype::Tech, "store", "Store"); // SPEC-002 (root)
        fresh(root, SpecSubtype::Product, "login", "Login"); // PRD-001
        append_spec_fields(
            &spec_toml(root, SpecSubtype::Tech, 1),
            "descends_from = \"PRD-001\"\nparent = \"SPEC-002\"",
        );
        append_interaction(root, 1, "SPEC-002");

        assert!(
            build_registry(root).unwrap().validate(None).is_empty(),
            "a well-formed spine produces no findings"
        );
        assert!(
            run_validate(Some(root.to_path_buf()), None).is_ok(),
            "run_validate exits zero on a clean corpus"
        );
    }

    // --- PHASE-04: read_interactions (the [[edge]] reader) ---

    #[test]
    fn read_interactions_parses_edges_and_tolerates_absence() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "cli", "CLI");
        let ipath = root.join(".doctrine/spec/tech/001/interactions.toml");
        // seeded-empty → zero edges.
        assert!(read_interactions(&ipath).unwrap().is_empty());
        // a hand-authored [[edge]] parses.
        let seeded = fs::read_to_string(&ipath).unwrap();
        fs::write(
            &ipath,
            format!("{seeded}\n[[edge]]\ntarget = \"SPEC-002\"\ntype = \"uses\"\nnotes = \"x\"\n"),
        )
        .unwrap();
        let edges = read_interactions(&ipath).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target, "SPEC-002");
        assert_eq!(edges[0].kind, "uses");
        // a product spec has no interactions.toml → absent, not empty → [].
        fresh(root, SpecSubtype::Product, "onb", "Onboarding");
        assert!(
            read_interactions(&root.join(".doctrine/spec/product/001/interactions.toml"))
                .unwrap()
                .is_empty()
        );
    }

    // --- PHASE-04 VT-2: show is pure (no write, no mutation) ---

    /// Snapshot every file body + symlink target under a tree into a sorted map —
    /// equality catches content mutation AND any added/removed path.
    fn snapshot_tree(root: &Path) -> BTreeMap<PathBuf, String> {
        let mut map = BTreeMap::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).unwrap() {
                let entry = entry.unwrap();
                let p = entry.path();
                let ft = entry.file_type().unwrap();
                if ft.is_symlink() {
                    map.insert(
                        p.clone(),
                        format!("symlink->{}", fs::read_link(&p).unwrap().display()),
                    );
                } else if ft.is_dir() {
                    stack.push(p);
                } else {
                    map.insert(p.clone(), fs::read_to_string(&p).unwrap_or_default());
                }
            }
        }
        map
    }

    #[test]
    fn render_is_pure_no_write() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "cli", "CLI");
        run_req_add(
            Some(root.to_path_buf()),
            "SPEC-001",
            Some("Route".into()),
            ReqKind::Functional,
            None,
            None,
        )
        .unwrap();

        let before = snapshot_tree(&root.join(".doctrine"));
        run_show(Some(root.to_path_buf()), "SPEC-001", Format::Table).unwrap();
        let after = snapshot_tree(&root.join(".doctrine"));

        assert_eq!(before, after, "spec show mutates nothing on disk");
        // no `*.rendered.md` materialised (D9 — ephemeral v1).
        assert!(
            !after
                .keys()
                .any(|p| p.to_string_lossy().ends_with(".rendered.md")),
            "no rendered file written"
        );
    }

    // --- PHASE-04: `spec req list` — the authored-only requirement roster ---

    /// Mint a real requirement on disk and member it onto a spec at `order`. Uses
    /// `reserve` (the producer's first step) + `set_kind`/`set_status` to vary the
    /// authored fields, then `append_raw_member` for the membership row — no
    /// stdout (unlike `run_req_add`), so the roster reads back from authored files.
    fn member_a_requirement(
        root: &Path,
        spec_dir: &Path,
        slug: &str,
        title: &str,
        kind: ReqKind,
        status: ReqStatus,
        label: &str,
        order: u32,
    ) -> String {
        let reserved = requirement::reserve(root, slug, title, "2026-06-05").unwrap();
        let id = reserved.eid.numeric_id().unwrap();
        requirement::set_kind(root, id, kind).unwrap();
        requirement::set_status(root, id, status).unwrap();
        let fk = requirement::canonical_id(id);
        append_raw_member(spec_dir, &fk, label, order);
        fk
    }

    /// VT-1 (authored-only, INV-3): the roster carries only authored columns
    /// (`id label kind status`) — no observed/verdict field — and the module
    /// imports no `coverage` symbol (asserted by construction: this file has no
    /// such `use`). The rendered surface shows the authored `kind`/`status`, never
    /// an observed verdict token.
    #[test]
    fn req_list_is_authored_only_no_observed_column() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "cli", "CLI"); // SPEC-001
        let spec_dir = root.join(".doctrine/spec/tech/001");
        member_a_requirement(
            root,
            &spec_dir,
            "route",
            "Route",
            ReqKind::Functional,
            ReqStatus::Active,
            "FR-001",
            1,
        );

        let out = req_list_rows(root, "SPEC-001", ListArgs::default()).unwrap();
        // The four authored columns head the table; the membership label and the
        // requirement's authored kind/status are present.
        assert!(
            out.starts_with("id"),
            "authored columns head the table: {out}"
        );
        assert!(out.contains("label"));
        assert!(out.contains("kind"));
        assert!(out.contains("status"));
        assert!(out.contains("REQ-001"), "the canonical FK: {out}");
        assert!(out.contains("FR-001"), "the membership label: {out}");
        assert!(out.contains("functional"), "authored kind: {out}");
        assert!(out.contains("active"), "authored status: {out}");
        // No observed/verdict vocabulary leaks in (the roster never scans).
        for forbidden in ["observed", "verdict", "coverage", "verified"] {
            assert!(
                !out.contains(forbidden),
                "no observed/verdict column (`{forbidden}`): {out}"
            );
        }
    }

    /// VT-2 (dangling tolerance, E5): a member whose FK points at an absent
    /// requirement dir does NOT abort the roster — the row is rendered with an
    /// inline load-error in place of kind/status, and the result is `Ok`.
    #[test]
    fn req_list_dangling_member_degrades_and_continues() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "cli", "CLI"); // SPEC-001
        let spec_dir = root.join(".doctrine/spec/tech/001");
        // one resolvable member …
        member_a_requirement(
            root,
            &spec_dir,
            "route",
            "Route",
            ReqKind::Functional,
            ReqStatus::Active,
            "FR-001",
            1,
        );
        // … and one dangling member (no REQ-099 dir exists).
        append_raw_member(&spec_dir, "REQ-099", "FR-099", 2);

        let out = req_list_rows(root, "SPEC-001", ListArgs::default()).unwrap();
        // The resolvable row is intact …
        assert!(out.contains("REQ-001"), "resolved row present: {out}");
        assert!(out.contains("functional"));
        // … and the dangling row is present with an inline load-error, not dropped.
        assert!(out.contains("REQ-099"), "dangling row present: {out}");
        assert!(out.contains("FR-099"), "dangling label present: {out}");
        assert!(out.contains("load error"), "inline load-error note: {out}");

        // The JSON surface flags the dangling row machine-visibly.
        let json = req_list_rows(
            root,
            "SPEC-001",
            ListArgs {
                json: true,
                ..ListArgs::default()
            },
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["kind"], "requirement");
        let rows = v["rows"].as_array().unwrap();
        let dangling = rows.iter().find(|r| r["id"] == "REQ-099").unwrap();
        assert_eq!(dangling["dangling"], true);
        assert!(
            dangling["load_error"].is_string(),
            "load_error surfaced: {json}"
        );
        assert!(
            dangling.get("kind").is_none(),
            "no kind on a dangling row: {json}"
        );
    }

    /// A dangling row survives a `--status` filter that its authored siblings would
    /// fail — it carries no authored status for the filter to speak to, so dropping
    /// it would silence a corpus-health signal (E5).
    #[test]
    fn req_list_status_filter_never_drops_a_dangling_row() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "cli", "CLI"); // SPEC-001
        let spec_dir = root.join(".doctrine/spec/tech/001");
        member_a_requirement(
            root,
            &spec_dir,
            "route",
            "Route",
            ReqKind::Functional,
            ReqStatus::Active,
            "FR-001",
            1,
        );
        append_raw_member(&spec_dir, "REQ-099", "FR-099", 2);

        // filter to `pending` — the resolved `active` row drops out, the dangling
        // row stays.
        let out = req_list_rows(
            root,
            "SPEC-001",
            ListArgs {
                status: vec!["pending".into()],
                ..ListArgs::default()
            },
        )
        .unwrap();
        assert!(!out.contains("REQ-001"), "active row filtered out: {out}");
        assert!(out.contains("REQ-099"), "dangling row retained: {out}");
    }

    /// The thin shell `run_req_list` resolves the root and writes the compute
    /// half (smoke — exercises the CLI entry point ahead of its main.rs wiring, so
    /// the `dead_code` suppression rides only the non-test gate build).
    #[test]
    fn run_req_list_writes_the_roster() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "cli", "CLI"); // SPEC-001
        member_a_requirement(
            root,
            &root.join(".doctrine/spec/tech/001"),
            "route",
            "Route",
            ReqKind::Functional,
            ReqStatus::Active,
            "FR-001",
            1,
        );
        run_req_list(Some(root.to_path_buf()), "SPEC-001", ListArgs::default()).unwrap();
    }

    /// VT-3: `--status` filters the resolved roster and `--columns` projects /
    /// reorders; an unknown column is the SL-037 declaration-order error propagated
    /// from `select_columns`.
    #[test]
    fn req_list_status_and_columns_honoured_unknown_column_errors() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "cli", "CLI"); // SPEC-001
        let spec_dir = root.join(".doctrine/spec/tech/001");
        member_a_requirement(
            root,
            &spec_dir,
            "route",
            "Route",
            ReqKind::Functional,
            ReqStatus::Active,
            "FR-001",
            1,
        );
        member_a_requirement(
            root,
            &spec_dir,
            "store",
            "Store",
            ReqKind::Quality,
            ReqStatus::Pending,
            "NF-001",
            2,
        );

        // --status active keeps only REQ-001.
        let filtered = req_list_rows(
            root,
            "SPEC-001",
            ListArgs {
                status: vec!["active".into()],
                ..ListArgs::default()
            },
        )
        .unwrap();
        assert!(filtered.contains("REQ-001"), "active kept: {filtered}");
        assert!(!filtered.contains("REQ-002"), "pending dropped: {filtered}");

        // --columns id,label projects + orders; kind/status are dropped.
        let projected = req_list_rows(
            root,
            "SPEC-001",
            ListArgs {
                columns: Some(vec!["id".into(), "label".into()]),
                ..ListArgs::default()
            },
        )
        .unwrap();
        assert!(projected.contains("REQ-001"));
        assert!(projected.contains("FR-001"));
        assert!(
            !projected.contains("functional"),
            "kind column dropped: {projected}"
        );

        // unknown column → the uniform SL-037 error listing the available tokens.
        let err = req_list_rows(
            root,
            "SPEC-001",
            ListArgs {
                columns: Some(vec!["bogus".into()]),
                ..ListArgs::default()
            },
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("bogus"), "names the bad column: {msg}");
        assert!(msg.contains("id"), "lists the available set: {msg}");
        assert!(msg.contains("status"), "lists the available set: {msg}");
    }

    /// F4/SL-025 parity (RV-005 F-1): `spec req list --status` validates the
    /// requested value against the requirement known-set, mirroring `spec list`
    /// (`spec_list_rejects_an_unknown_status_…`) — a bogus status errors naming
    /// the value, never a silently-empty roster.
    #[test]
    fn req_list_rejects_an_unknown_status_with_the_uniform_error() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "cli", "CLI"); // SPEC-001
        member_a_requirement(
            root,
            &root.join(".doctrine/spec/tech/001"),
            "route",
            "Route",
            ReqKind::Functional,
            ReqStatus::Active,
            "FR-001",
            1,
        );
        let err = req_list_rows(
            root,
            "SPEC-001",
            ListArgs {
                status: vec!["bogus".into()],
                ..ListArgs::default()
            },
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("bogus"),
            "names the bad value: {err}"
        );
    }

    // --- PHASE-04 paths verb golden tests ---

    /// Scaffold one spec entity dir with identity files + optional extras.
    fn spec_fixture(root: &Path, subtype: SpecSubtype, id: u32, extra: &[&str]) {
        let name = format!("{id:03}");
        let dir = root.join(subtype.kind().dir).join(&name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(format!("{SPEC_STEM}-{name}.toml")), "toml").unwrap();
        fs::write(dir.join(format!("{SPEC_STEM}-{name}.md")), "md").unwrap();
        for e in extra {
            fs::write(dir.join(e), e).unwrap();
        }
    }

    #[test]
    fn paths_full_shows_toml_md_and_members_in_canonical_order() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        spec_fixture(
            root,
            SpecSubtype::Tech,
            1,
            &["members.toml", "interactions.toml"],
        );
        let sel = crate::paths::PathSelection {
            toml: false,
            md: false,
            entity: false,
            single: false,
        };
        let entity_dir = root.join(SpecSubtype::Tech.kind().dir).join("001");
        let identity_toml = entity_dir.join("spec-001.toml");
        let identity_md = entity_dir.join("spec-001.md");
        let set =
            crate::paths::scan_entity_dir(&entity_dir, &identity_toml, Some(&identity_md), root)
                .unwrap();
        let lines = crate::paths::select_paths(&set, &sel).unwrap();
        let output = lines.join("\n");
        assert!(output.contains(".doctrine/spec/tech/001/spec-001.toml"));
        assert!(output.contains(".doctrine/spec/tech/001/spec-001.md"));
        assert!(output.contains(".doctrine/spec/tech/001/interactions.toml"));
        assert!(output.contains(".doctrine/spec/tech/001/members.toml"));
    }

    #[test]
    fn paths_single_truncates_to_first() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        spec_fixture(root, SpecSubtype::Product, 1, &["members.toml"]);
        let sel = crate::paths::PathSelection {
            toml: false,
            md: false,
            entity: false,
            single: true,
        };
        let entity_dir = root.join(SpecSubtype::Product.kind().dir).join("001");
        let identity_toml = entity_dir.join("spec-001.toml");
        let identity_md = entity_dir.join("spec-001.md");
        let set =
            crate::paths::scan_entity_dir(&entity_dir, &identity_toml, Some(&identity_md), root)
                .unwrap();
        let lines = crate::paths::select_paths(&set, &sel).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], ".doctrine/spec/product/001/spec-001.toml");
    }

    #[test]
    fn paths_toml_only() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        spec_fixture(root, SpecSubtype::Tech, 2, &["members.toml"]);
        let sel = crate::paths::PathSelection {
            toml: true,
            md: false,
            entity: false,
            single: false,
        };
        let entity_dir = root.join(SpecSubtype::Tech.kind().dir).join("002");
        let identity_toml = entity_dir.join("spec-002.toml");
        let identity_md = entity_dir.join("spec-002.md");
        let set =
            crate::paths::scan_entity_dir(&entity_dir, &identity_toml, Some(&identity_md), root)
                .unwrap();
        let lines = crate::paths::select_paths(&set, &sel).unwrap();
        assert_eq!(lines, vec![".doctrine/spec/tech/002/spec-002.toml"]);
    }

    #[test]
    fn paths_md_only() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        spec_fixture(root, SpecSubtype::Product, 3, &[]);
        let sel = crate::paths::PathSelection {
            toml: false,
            md: true,
            entity: false,
            single: false,
        };
        let entity_dir = root.join(SpecSubtype::Product.kind().dir).join("003");
        let identity_toml = entity_dir.join("spec-003.toml");
        let identity_md = entity_dir.join("spec-003.md");
        let set =
            crate::paths::scan_entity_dir(&entity_dir, &identity_toml, Some(&identity_md), root)
                .unwrap();
        let lines = crate::paths::select_paths(&set, &sel).unwrap();
        assert_eq!(lines, vec![".doctrine/spec/product/003/spec-003.md"]);
    }

    #[test]
    fn paths_entity_gives_toml_and_md() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        spec_fixture(
            root,
            SpecSubtype::Tech,
            4,
            &["members.toml", "interactions.toml"],
        );
        let sel = crate::paths::PathSelection {
            toml: false,
            md: false,
            entity: true,
            single: false,
        };
        let entity_dir = root.join(SpecSubtype::Tech.kind().dir).join("004");
        let identity_toml = entity_dir.join("spec-004.toml");
        let identity_md = entity_dir.join("spec-004.md");
        let set =
            crate::paths::scan_entity_dir(&entity_dir, &identity_toml, Some(&identity_md), root)
                .unwrap();
        let lines = crate::paths::select_paths(&set, &sel).unwrap();
        assert_eq!(
            lines,
            vec![
                ".doctrine/spec/tech/004/spec-004.toml",
                ".doctrine/spec/tech/004/spec-004.md"
            ]
        );
    }

    #[test]
    fn paths_invalid_ref_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        spec_fixture(root, SpecSubtype::Tech, 1, &[]);
        // PRD-99999 doesn't exist.
        let (_, id) = resolve_spec_ref("PRD-99999").unwrap();
        let entity_dir = root
            .join(SpecSubtype::Product.kind().dir)
            .join(format!("{id:03}"));
        let identity_toml = entity_dir.join(format!("spec-{id:03}.toml"));
        let identity_md = entity_dir.join(format!("spec-{id:03}.md"));
        let scan =
            crate::paths::scan_entity_dir(&entity_dir, &identity_toml, Some(&identity_md), root);
        assert!(scan.is_err());
    }

    #[test]
    fn paths_multi_ref_splat_preserves_order() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        spec_fixture(root, SpecSubtype::Tech, 1, &[]);
        spec_fixture(root, SpecSubtype::Product, 1, &[]);
        let sel = crate::paths::PathSelection {
            toml: false,
            md: false,
            entity: false,
            single: false,
        };
        let mut all_lines: Vec<String> = Vec::new();
        for (subtype, n) in [(SpecSubtype::Tech, "001"), (SpecSubtype::Product, "001")] {
            let entity_dir = root.join(subtype.kind().dir).join(n);
            let toml_name = format!("{SPEC_STEM}-{n}.toml");
            let md_name = format!("{SPEC_STEM}-{n}.md");
            let set = crate::paths::scan_entity_dir(
                &entity_dir,
                &entity_dir.join(&toml_name),
                Some(&entity_dir.join(&md_name)),
                root,
            )
            .unwrap();
            all_lines.extend(crate::paths::select_paths(&set, &sel).unwrap());
        }
        assert_eq!(all_lines.len(), 4);
        assert!(all_lines[0].contains("tech/001/spec-001.toml"));
        assert!(all_lines[2].contains("product/001/spec-001.toml"));
    }

    // --- SL-153 PHASE-03: spec interactions add/remove ---

    /// VT-1: adding an interaction edge to a tech spec appends it and show reflects it.
    #[test]
    fn interaction_add_appends_edge() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
        fresh(root, SpecSubtype::Tech, "store", "Store"); // SPEC-002

        run_interaction_add(
            Some(root.to_path_buf()),
            "SPEC-001",
            "SPEC-002",
            "uses",
            Some("calls boot"),
        )
        .unwrap();

        let edges =
            read_interactions(&root.join(".doctrine/spec/tech/001/interactions.toml")).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target, "SPEC-002");
        assert_eq!(edges[0].kind, "uses");
        assert_eq!(edges[0].notes.as_deref(), Some("calls boot"));

        // show reflects the new edge.
        run_show(Some(root.to_path_buf()), "SPEC-001", Format::Table).unwrap();
    }

    /// VT-2: adding an interaction to a product spec errors.
    #[test]
    fn interaction_add_product_errors() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Product, "onboarding", "Onboarding"); // PRD-001
        fresh(root, SpecSubtype::Tech, "store", "Store"); // SPEC-001

        let err = run_interaction_add(
            Some(root.to_path_buf()),
            "PRD-001",
            "SPEC-001",
            "uses",
            None,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("tech-spec-only"),
            "product spec source errors: {err}"
        );
    }

    /// VT-3: adding an interaction to a nonexistent target spec errors.
    #[test]
    fn interaction_add_nonexistent_target_errors() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001

        let err = run_interaction_add(
            Some(root.to_path_buf()),
            "SPEC-001",
            "SPEC-999",
            "uses",
            None,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("not found"),
            "nonexistent target errors: {err}"
        );
    }

    /// VT-4: adding an interaction targeting a product spec (PRD) errors.
    #[test]
    fn interaction_add_non_spec_target_errors() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
        fresh(root, SpecSubtype::Product, "onboarding", "Onboarding"); // PRD-001

        let err = run_interaction_add(
            Some(root.to_path_buf()),
            "SPEC-001",
            "PRD-001",
            "uses",
            None,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("must be a tech spec"),
            "PRD target errors: {err}"
        );
    }

    /// VT-5: adding the same target twice via `run_interaction_add` is idempotent —
    /// a second add is a no-op that prints an informational message.
    #[test]
    fn interaction_add_idempotent_on_target() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
        fresh(root, SpecSubtype::Tech, "store", "Store"); // SPEC-002

        run_interaction_add(
            Some(root.to_path_buf()),
            "SPEC-001",
            "SPEC-002",
            "uses",
            None,
        )
        .unwrap();

        // Second add to same target → no-op (informational, not error).
        run_interaction_add(
            Some(root.to_path_buf()),
            "SPEC-001",
            "SPEC-002",
            "calls",
            None,
        )
        .unwrap();

        // Only one edge remains, and the original type is preserved (idempotent).
        let edges =
            read_interactions(&root.join(".doctrine/spec/tech/001/interactions.toml")).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].kind, "uses", "first write wins");
    }

    /// VT-6: removing an interaction edge clears it; show no longer reflects it.
    #[test]
    fn interaction_remove_clears_edge() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
        fresh(root, SpecSubtype::Tech, "store", "Store"); // SPEC-002

        run_interaction_add(
            Some(root.to_path_buf()),
            "SPEC-001",
            "SPEC-002",
            "uses",
            None,
        )
        .unwrap();

        run_interaction_remove(Some(root.to_path_buf()), "SPEC-001", "SPEC-002").unwrap();

        let edges =
            read_interactions(&root.join(".doctrine/spec/tech/001/interactions.toml")).unwrap();
        assert!(edges.is_empty(), "edge removed");
    }

    /// VT-7: removing an absent interaction is a no-op (count 0).
    #[test]
    fn interaction_remove_absent_noop() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001

        run_interaction_remove(Some(root.to_path_buf()), "SPEC-001", "SPEC-002").unwrap();

        // file still seeded-empty comment-only.
        let text =
            fs::read_to_string(root.join(".doctrine/spec/tech/001/interactions.toml")).unwrap();
        assert!(
            text.contains("Seeded empty"),
            "seeded comment survives no-op remove"
        );
    }

    /// VT-8: a non-canonical target in the toml (bare `SPEC-2`) is canonicalised
    /// by the dup-check, so adding `SPEC-002` is a no-op, and removing `SPEC-002`
    /// clears the edge.
    #[test]
    fn interaction_canonical_dup_match() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Tech, "auth", "Auth"); // SPEC-001
        fresh(root, SpecSubtype::Tech, "store", "Store"); // SPEC-002

        // Hand-write a non-canonical `target = "SPEC-2"` into interactions.toml.
        let ipath = root.join(".doctrine/spec/tech/001/interactions.toml");
        let seeded = fs::read_to_string(&ipath).unwrap();
        fs::write(
            &ipath,
            format!("{seeded}\n[[edge]]\ntarget = \"SPEC-2\"\ntype = \"calls\"\n"),
        )
        .unwrap();

        // Adding SPEC-002 → no-op (canonical dup-check matches SPEC-2).
        run_interaction_add(
            Some(root.to_path_buf()),
            "SPEC-001",
            "SPEC-002",
            "uses",
            None,
        )
        .unwrap();

        let edges = read_interactions(&ipath).unwrap();
        assert_eq!(edges.len(), 1, "dup-check prevented second edge");
        assert_eq!(edges[0].kind, "calls", "original type preserved");

        // Removing SPEC-002 clears the non-canonical SPEC-2 edge.
        run_interaction_remove(Some(root.to_path_buf()), "SPEC-001", "SPEC-002").unwrap();

        let edges2 = read_interactions(&ipath).unwrap();
        assert!(edges2.is_empty(), "non-canonical edge removed");
    }
}
