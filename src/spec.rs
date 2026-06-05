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
//! the render fns, the parse-layer structs, and `new`/`list`. The kind-agnostic
//! engine is `crate::entity` (unchanged — three new `Fresh` callers only, R6
//! gate); the shared metadata-list substrate is `crate::meta`, reused **additively**
//! — `spec list` rides `read_metas`/`render_table` with zero `meta.rs` edits.
//!
//! `req add` / `show` / `validate` are later phases; the parse structs they consume
//! (`Spec`, `Interaction`, `Source`, `SpecStatus`, `C4Level`) have no production
//! caller until then, so they ride the `cfg_attr(not(test), expect(dead_code, …))`
//! bridge (D-2 / memory `mem.pattern.lint.expect-not-allow`), which self-erases on
//! the first real caller. `Member` is exempt — `spec list` counts members through it.

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::entity::{
    self, Artifact, Fileset, Inputs, Kind, LocalFs, MaterialiseRequest, ScaffoldCtx,
};
use crate::meta;
use crate::requirement::{self, ReqKind};

/// The toml/md file stem — shared by both subtypes (`spec-NNN.toml`). Distinct
/// from each `Kind.prefix` (`PRD`/`SPEC`) and from the tree dirs below.
const SPEC_STEM: &str = "spec";

/// The product subtype: light identity, `members.toml`, no interactions. Own tree
/// + reservation namespace.
const PRODUCT_SPEC_KIND: Kind = Kind {
    dir: ".doctrine/spec/product",
    prefix: "PRD",
    scaffold: product_spec_scaffold,
};

/// The tech subtype: identity + flat fields, `members.toml` + `interactions.toml`.
/// Own tree + reservation namespace (ids independent of product's).
const TECH_SPEC_KIND: Kind = Kind {
    dir: ".doctrine/spec/tech",
    prefix: "SPEC",
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

    /// Human label for `spec list` section headers.
    const fn label(self) -> &'static str {
        match self {
            SpecSubtype::Product => "product",
            SpecSubtype::Tech => "tech",
        }
    }
}

/// A code anchor a tech spec governs (tech-only; `[[source]]`). Shape mirrors the
/// legacy canon `doc/spec-entity-spec.md` (D-3): the language + a code identifier,
/// with an optional finer module path.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "first prod caller PHASE-04 (spec show render); remove then"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct Source {
    pub(crate) language: String,
    pub(crate) identifier: String,
    #[serde(default)]
    pub(crate) module: Option<String>,
}

/// A spec's lifecycle status. Closed set, kebab serde; hand-edited, git is the
/// trail (no date stamps — §5.4).
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "first prod caller PHASE-04 (spec show render); remove then"
    )
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SpecStatus {
    Draft,
    Active,
    Deprecated,
    Superseded,
}

/// The C4 architectural level of a tech spec. Closed set (C6 ruling), kebab serde;
/// tech-only, optional.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "first prod caller PHASE-04 (spec show render); remove then"
    )
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum C4Level {
    Context,
    Container,
    Component,
    Code,
}

/// The spec identity parse layer. `title` keys the shared-`Meta` convention (C2).
/// `category` is deliberately OPEN vocabulary (`Option<String>`, C6); the tech flat
/// fields default to absent/empty for a product spec.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "first prod caller PHASE-04 (spec show render); remove then"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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
    #[serde(default)]
    pub(crate) responsibilities: Vec<String>,
    #[serde(default, rename = "source")]
    pub(crate) sources: Vec<Source>,
}

/// One membership row in a spec's `members.toml` — the spec→requirement edge with
/// its sticky label and advisory order. The FK is a plain canonical string
/// (`REQ-NNN`); integrity is `validate`'s job, not the type's.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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
/// (`SPEC-NNN`). Hand-authored in v1 (no verb — D-Q4).
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "first prod caller PHASE-05 (spec validate); remove then"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct Interaction {
    pub(crate) target: String,
    #[serde(rename = "type")]
    pub(crate) kind: String,
    #[serde(default)]
    pub(crate) notes: Option<String>,
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
        .replace("{{slug}}", slug)
        .replace("{{title}}", title))
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
            rel_path: PathBuf::from(format!("{name}/{SPEC_STEM}-{name}.toml")),
            body: render_spec_toml(subtype, id, ctx.slug, ctx.title)?,
        },
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/{SPEC_STEM}-{name}.md")),
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

// ---------------------------------------------------------------------------
// `#members` — list's derived column
// ---------------------------------------------------------------------------

/// Parse a spec's `members.toml` into its rows. A missing file → no members (a
/// spec always carries the seed, but tolerance keeps callers robust). Shared by
/// the `#members` column and `req add`'s label/order scan.
fn read_members(members_path: &Path) -> anyhow::Result<Vec<Member>> {
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
    std::fs::write(members_path, doc.to_string())
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
    let title = crate::input::resolve_title(title)?;
    let slug = crate::input::resolve_slug(&title, slug)?;
    let date = crate::clock::today();
    let out = entity::materialise(
        subtype.kind(),
        &LocalFs,
        &root,
        &MaterialiseRequest::Fresh,
        &Inputs {
            slug: &slug,
            title: &title,
            date: &date,
        },
    )?;
    let id = out
        .eid
        .numeric_id()
        .context("spec kind must yield a numeric id")?;
    writeln!(
        io::stdout(),
        "Created {}-{id:03}: {}",
        subtype.kind().prefix,
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
    let slug = crate::input::resolve_slug(&title, None)?;
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

/// `doctrine spec list [--status S]` — per-subtype blocks of `id status slug
/// #members`, sorted by id. Each block rides the shared `meta::render_table` (the
/// `#members` cell is derived in this module, exactly as `slice list` derives its
/// `phases` cell — additive, no `meta.rs` change). `--status` filters within each.
pub(crate) fn run_list(path: Option<PathBuf>, status: Option<&str>) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let mut out = io::stdout();
    for subtype in [SpecSubtype::Product, SpecSubtype::Tech] {
        write!(out, "{}", list_block(&root, subtype, status)?)?;
    }
    Ok(())
}

/// One subtype's `list` block as a string — the compute half of `run_list`,
/// extracted so it is unit-testable without stdout. Empty (no specs) → `""` (the
/// whole block, header included, is suppressed).
fn list_block(root: &Path, subtype: SpecSubtype, status: Option<&str>) -> anyhow::Result<String> {
    let tree = root.join(subtype.kind().dir);
    let metas = meta::sort_and_filter(meta::read_metas(&tree, SPEC_STEM)?, status);
    let mut rows = Vec::with_capacity(metas.len());
    for m in metas {
        let count = member_count(&tree.join(format!("{:03}", m.id)))?;
        rows.push((m, count));
    }
    Ok(format_spec_rows(subtype, &rows))
}

/// Render one subtype's spec rows: a label line, a header row, then `id status slug
/// #members` per spec, aligned via the shared `meta::render_table`. Empty input →
/// `""` (the block is omitted entirely). Pure.
fn format_spec_rows(subtype: SpecSubtype, rows: &[(meta::Meta, usize)]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let mut grid: Vec<Vec<String>> = vec![
        ["id", "status", "slug", "#members"]
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
    ];
    for (m, count) in rows {
        grid.push(vec![
            format!("{:03}", m.id),
            m.status.clone(),
            m.slug.clone(),
            count.to_string(),
        ]);
    }
    format!("{}\n{}", subtype.label(), meta::render_table(&grid))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta::Meta;
    use std::fs;

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
        )
        .unwrap()
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
        let m = meta::read_meta(&tree, SPEC_STEM, 1).unwrap();
        assert_eq!(
            m,
            Meta {
                id: 1,
                slug: "onboarding".to_string(),
                title: "Onboarding".to_string(),
                status: "draft".to_string(),
            }
        );
    }

    #[test]
    fn spec_list_rows_per_subtype_with_member_count() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fresh(root, SpecSubtype::Product, "onboarding", "Onboarding");
        fresh(root, SpecSubtype::Product, "billing", "Billing");

        // seeded specs → member count 0 on every row.
        let block = list_block(root, SpecSubtype::Product, None).unwrap();
        assert!(block.starts_with("product\n"));
        assert!(block.contains("id   status  slug"));
        assert!(block.contains("#members"));
        assert!(block.contains("001  draft   onboarding"));
        assert!(block.contains("002  draft   billing"));
        // both rows end in the 0 member count.
        for line in block.lines().filter(|l| l.starts_with("00")) {
            assert!(
                line.trim_end().ends_with('0'),
                "row ends in #members=0: {line}"
            );
        }

        // the tech block is empty (no tech specs) → suppressed.
        assert_eq!(list_block(root, SpecSubtype::Tech, None).unwrap(), "");
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

        let active = list_block(root, SpecSubtype::Product, Some("active")).unwrap();
        assert!(active.contains("002  active  billing"));
        assert!(!active.contains("onboarding"));
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

        // C2: the same toml deserialises into the shared Meta (the `title` proof).
        let m: Meta = toml::from_str(body).unwrap();
        assert_eq!(m.title, "CLI");
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

        // make the append target read-only: the label/order scan still reads it
        // (valid seed), the reserve succeeds, the final write fails → torn write.
        let members_path = root.join(".doctrine/spec/product/001/members.toml");
        let mut perms = fs::metadata(&members_path).unwrap().permissions();
        perms.set_mode(0o444);
        fs::set_permissions(&members_path, perms).unwrap();

        let err = run_req_add(
            Some(root.to_path_buf()),
            "PRD-001",
            Some("X".into()),
            ReqKind::Functional,
            None,
        );
        assert!(
            err.is_err(),
            "append must fail on the read-only members.toml"
        );

        // the reserved requirement is an orphan: present (uncommitted), no member row.
        assert!(
            root.join(".doctrine/requirement/001/requirement-001.toml")
                .is_file()
        );
        let members = read_members(&members_path).unwrap();
        assert!(members.is_empty(), "no partial member row written");
    }
}
