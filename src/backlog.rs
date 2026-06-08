// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine backlog` — lightweight work-intake items (issue / improvement /
//! chore / risk / idea), each a numeric directory under
//! `.doctrine/backlog/<kind>/` holding a sister `backlog-NNN.toml` (structured,
//! queried metadata) and a scaffolded `backlog-NNN.md` prose body, with an
//! `NNN-slug` symlink alias — the ADR/spec/requirement shape (design §5.1/§5.3).
//!
//! Five `ItemKind`s ride five `entity::Kind`s over the same kind-blind engine,
//! each its own tree + reservation namespace (`ISS-001` and `RSK-001` coexist —
//! the counters are independent). The subtypes diverge only in their prefix and
//! whether the scaffold seeds a risk `[facet]`.
//!
//! This module owns the *backlog-specific* parts — the five `Kind`s, their shared
//! scaffold, the render fns, and the three-layer parse model (`RawBacklogToml`
//! tolerant parse → validated `BacklogItem`, with the `"" -> None` validation
//! seam for the optional `resolution`/risk-level fields). The kind-agnostic engine
//! is `crate::entity` (unchanged — five new `Fresh` callers only, the R6 gate).
//!
//! All four verbs (`new`/`list`/`show`/`edit`) are wired into the CLI as of
//! PHASE-06; the only production-dead item left is `KIND_PRECEDENCE` — inert
//! until the PRD-011 multi-kind resolver consumes it. Its dead-code expectation
//! is scoped to that one const (below), NOT module-wide, so genuinely-dead code
//! introduced later still surfaces.

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::entity::{
    self, Artifact, Fileset, Inputs, Kind, LocalFs, MaterialiseRequest, ScaffoldCtx,
};
use crate::tomlfmt::toml_string;

/// The toml/md file stem — shared by all five kinds (`backlog-NNN.toml`). Distinct
/// from each `Kind.prefix` (`ISS`/`IMP`/…) and from the per-kind tree dirs.
const BACKLOG_STEM: &str = "backlog";

// ---------------------------------------------------------------------------
// The discriminator + its five engine `Kind`s
// ---------------------------------------------------------------------------

/// Which backlog item this is. Closed set; kebab serde (round-trips the toml's
/// `kind`) and `clap::ValueEnum` (the `backlog new` positional, PHASE-02). Selects
/// the tree, prefix, and scaffold fileset. Fixed at capture (PRD-009 §4 invariant).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ItemKind {
    Issue,
    Improvement,
    Chore,
    Risk,
    Idea,
}

/// The issue kind: a defect / problem to fix. Own tree + reservation namespace.
const ISSUE_KIND: Kind = Kind {
    dir: ".doctrine/backlog/issue",
    prefix: "ISS",
    scaffold: |c| backlog_scaffold(ItemKind::Issue, c),
};

/// The improvement kind: an enhancement to existing behaviour.
const IMPROVEMENT_KIND: Kind = Kind {
    dir: ".doctrine/backlog/improvement",
    prefix: "IMP",
    scaffold: |c| backlog_scaffold(ItemKind::Improvement, c),
};

/// The chore kind: maintenance with no user-visible behaviour change.
const CHORE_KIND: Kind = Kind {
    dir: ".doctrine/backlog/chore",
    prefix: "CHR",
    scaffold: |c| backlog_scaffold(ItemKind::Chore, c),
};

/// The risk kind: a tracked risk — the only kind carrying a `[facet]`.
const RISK_KIND: Kind = Kind {
    dir: ".doctrine/backlog/risk",
    prefix: "RSK",
    scaffold: |c| backlog_scaffold(ItemKind::Risk, c),
};

/// The idea kind: a speculative possibility, not yet committed work.
const IDEA_KIND: Kind = Kind {
    dir: ".doctrine/backlog/idea",
    prefix: "IDE",
    scaffold: |c| backlog_scaffold(ItemKind::Idea, c),
};

/// Boundary precedence for the future multi-kind resolver (PRD-009 §4): when one
/// capture could match several kinds, `risk` wins, then issue/improvement/chore/
/// idea. INERT in v1 — `new` always takes an explicit kind, so this is never
/// exercised; recorded so the order is canon when the resolver lands (PRD-011).
#[expect(
    dead_code,
    reason = "inert until the PRD-011 multi-kind resolver consumes it"
)]
const KIND_PRECEDENCE: [ItemKind; 5] = [
    ItemKind::Risk,
    ItemKind::Issue,
    ItemKind::Improvement,
    ItemKind::Chore,
    ItemKind::Idea,
];

impl ItemKind {
    /// The engine `Kind` for this item kind — the single source of its tree +
    /// prefix + scaffold.
    const fn kind(self) -> &'static Kind {
        match self {
            ItemKind::Issue => &ISSUE_KIND,
            ItemKind::Improvement => &IMPROVEMENT_KIND,
            ItemKind::Chore => &CHORE_KIND,
            ItemKind::Risk => &RISK_KIND,
            ItemKind::Idea => &IDEA_KIND,
        }
    }

    /// The canonical-id prefix (`ISS`/`IMP`/`CHR`/`RSK`/`IDE`), read off the
    /// `Kind` so the prefix is never hardcoded twice.
    const fn prefix(self) -> &'static str {
        self.kind().prefix
    }

    /// The kebab `kind` string written to `backlog-NNN.toml` (matches the serde
    /// rename). Pure; the render mirror for the stored `kind` field.
    const fn as_str(self) -> &'static str {
        match self {
            ItemKind::Issue => "issue",
            ItemKind::Improvement => "improvement",
            ItemKind::Chore => "chore",
            ItemKind::Risk => "risk",
            ItemKind::Idea => "idea",
        }
    }

    /// The canonical ref for an id in this kind's namespace (`ISS-007`) — the
    /// print of `backlog new` and the inverse of `from_prefix`. Prefix from the
    /// `Kind` (single source).
    fn canonical_id(self, id: u32) -> String {
        format!("{}-{id:03}", self.prefix())
    }

    /// Resolve a canonical-id prefix back to its kind (`backlog show <ID>`
    /// auto-detect, PHASE-04). Prefixes come from the `Kind`s — the single source;
    /// the kind set is `ItemKind::ALL` (one declaration, not a second copy).
    fn from_prefix(prefix: &str) -> Option<Self> {
        ItemKind::ALL.into_iter().find(|k| k.prefix() == prefix)
    }

    /// Whether this kind carries a risk `[facet]` (risk only). Selects the
    /// scaffold template and gates facet render.
    const fn has_facet(self) -> bool {
        matches!(self, ItemKind::Risk)
    }

    /// Every kind in DECLARATION order — the single source for the cross-kind
    /// `list` read (each tree in turn) and the `ordinal` grouping key.
    const ALL: [ItemKind; 5] = [
        ItemKind::Issue,
        ItemKind::Improvement,
        ItemKind::Chore,
        ItemKind::Risk,
        ItemKind::Idea,
    ];

    /// The kind's position in declaration order — the primary `list` sort key.
    /// A deterministic GROUPING (Issue…Idea), explicitly NOT a priority claim
    /// (R7; priority is PRD-011, deferred) and NOT `KIND_PRECEDENCE` (risk-first,
    /// the inert future-resolver order).
    const fn ordinal(self) -> usize {
        match self {
            ItemKind::Issue => 0,
            ItemKind::Improvement => 1,
            ItemKind::Chore => 2,
            ItemKind::Risk => 3,
            ItemKind::Idea => 4,
        }
    }
}

// ---------------------------------------------------------------------------
// Closed value enums (kebab serde + an `as_str` render mirror)
// ---------------------------------------------------------------------------

/// A backlog item's lifecycle status. Closed canon set, kebab serde; hand-settable
/// and ungated (slices/ADRs/specs ship this way). `status` is always seeded a real
/// value (`open`), so it serde-parses directly — never the `"" -> None` seam.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Status {
    Open,
    Triaged,
    Started,
    Resolved,
    Closed,
}

impl Status {
    /// The kebab string for render (matches the serde rename). Pure.
    const fn as_str(self) -> &'static str {
        match self {
            Status::Open => "open",
            Status::Triaged => "triaged",
            Status::Started => "started",
            Status::Resolved => "resolved",
            Status::Closed => "closed",
        }
    }

    /// Whether this status is terminal (`resolved`/`closed`). A **backlog-local**
    /// predicate — explicitly NOT `slice::is_terminal_status` (R4): backlog and
    /// slice lifecycles are independent vocabularies. Drives the `resolution ⟺
    /// terminal` coupling (`edit`, PHASE-05) and the hide-terminal `list` rule.
    const fn is_terminal(self) -> bool {
        matches!(self, Status::Resolved | Status::Closed)
    }
}

/// Why a terminal item was closed. One generic, kind-agnostic set (PRD-009): a
/// resolution is never a close *reason* hidden in a facet. Optional — present only
/// on a terminal item (the `"" -> None` seam).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Resolution {
    Fixed,
    Done,
    Mitigated,
    Accepted,
    Expired,
    Duplicate,
    WontDo,
    Obsolete,
    Promoted,
}

impl Resolution {
    /// The kebab string for render (matches the serde rename). Pure.
    const fn as_str(self) -> &'static str {
        match self {
            Resolution::Fixed => "fixed",
            Resolution::Done => "done",
            Resolution::Mitigated => "mitigated",
            Resolution::Accepted => "accepted",
            Resolution::Expired => "expired",
            Resolution::Duplicate => "duplicate",
            Resolution::WontDo => "wont-do",
            Resolution::Obsolete => "obsolete",
            Resolution::Promoted => "promoted",
        }
    }
}

/// A risk facet axis level. Closed set, kebab serde; tech of the risk `[facet]`,
/// optional (the `"" -> None` seam — seeded empty until assessed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    /// The kebab string for render (matches the serde rename). Pure.
    const fn as_str(self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        }
    }
}

// ---------------------------------------------------------------------------
// Three-layer parse model (the entity-model tolerant-parse tier — §5.3)
// ---------------------------------------------------------------------------

/// The tolerant parse layer. `resolution` and the risk levels are read as raw
/// `String` (they are seeded `""`, which is no enum variant — serde would reject
/// a direct `Option<Resolution>`), so the `"" -> None` mapping is a separate
/// `validate` pass, not a serde derive. `status`/`kind` carry real values and
/// parse to their enums directly. `#[serde(default)]` lets the seeded-empty
/// collections and the absent (non-risk) `[facet]` parse.
#[derive(Debug, Deserialize)]
struct RawBacklogToml {
    id: u32,
    slug: String,
    title: String,
    kind: ItemKind,
    status: Status,
    #[serde(default)]
    resolution: String,
    created: String,
    updated: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    facet: Option<RawRiskFacet>,
    #[serde(default)]
    relationships: Relationships,
}

/// The tolerant risk-facet layer: the two assessable axes as raw `String` (the
/// `"" -> None` seam), `origin` as raw `String` (empty → absent), `controls` a
/// free list.
#[derive(Debug, Deserialize)]
struct RawRiskFacet {
    #[serde(default)]
    likelihood: String,
    #[serde(default)]
    impact: String,
    #[serde(default)]
    origin: String,
    #[serde(default)]
    controls: Vec<String>,
}

/// The validated entity (design §5.2). `id/slug/title/status` are top-level in the
/// toml so the file also round-trips into the shared `meta::Meta`. `kind` is stored
/// AND implied by the tree dir — stored so one read yields the entity without path
/// inspection. The `"" -> None` optionals are resolved off the raw layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BacklogItem {
    id: u32,
    slug: String,
    title: String,
    kind: ItemKind,
    status: Status,
    resolution: Option<Resolution>,
    created: String,
    updated: String,
    tags: Vec<String>,
    facet: Option<RiskFacet>,
    relationships: Relationships,
}

/// The validated risk facet (risk only). Every axis typed — no untyped bag
/// (PRD-009 invariant). The assessable axes are optional until assessed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RiskFacet {
    likelihood: Option<RiskLevel>,
    impact: Option<RiskLevel>,
    origin: Option<String>,
    controls: Vec<String>,
}

/// Outbound-only relations (ADR-004): a backlog item points OUT at the slices,
/// specs, and drift it touches; the reverse view is derived (deferred, PRD-011).
/// Shared verbatim by the raw and validated layers (no `"" -> None` seam — these
/// are plain lists), seeded empty so `#[serde(default)]` parses a virgin item.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
struct Relationships {
    #[serde(default)]
    slices: Vec<String>,
    #[serde(default)]
    specs: Vec<String>,
    #[serde(default)]
    drift: Vec<String>,
}

/// Parse a kebab token into its closed enum via the serde derive — the single
/// source of the variant↔string mapping (the `as_str` mirrors render only).
/// Errors with serde's "unknown variant" message on a bad token (`what` names the
/// field for the message).
fn parse_enum<T: serde::de::DeserializeOwned>(token: &str, what: &str) -> anyhow::Result<T> {
    use serde::de::IntoDeserializer;
    let de: serde::de::value::StrDeserializer<'_, serde::de::value::Error> =
        token.into_deserializer();
    T::deserialize(de).map_err(|e| anyhow::anyhow!("invalid {what} `{token}`: {e}"))
}

/// The `"" -> None` seam for an optional closed enum: an empty token is absent; a
/// non-empty token parses to its variant (erroring on an unknown one).
fn optional_enum<T: serde::de::DeserializeOwned>(
    token: &str,
    what: &str,
) -> anyhow::Result<Option<T>> {
    if token.is_empty() {
        Ok(None)
    } else {
        parse_enum(token, what).map(Some)
    }
}

/// The `"" -> None` seam for an optional free-text field.
fn optional_text(text: String) -> Option<String> {
    if text.is_empty() { None } else { Some(text) }
}

/// Validate a tolerant `RawBacklogToml` into a typed `BacklogItem` — the second
/// layer of the parse model. Maps the seeded-`""` optionals to `None`, parses any
/// non-empty value to its enum (erroring on an unknown token), and validates the
/// risk facet when present. Consumes the raw layer (its owned strings move across).
fn validate(raw: RawBacklogToml) -> anyhow::Result<BacklogItem> {
    let resolution = optional_enum(&raw.resolution, "resolution")?;
    let facet = match raw.facet {
        Some(f) => Some(validate_facet(f)?),
        None => None,
    };
    Ok(BacklogItem {
        id: raw.id,
        slug: raw.slug,
        title: raw.title,
        kind: raw.kind,
        status: raw.status,
        resolution,
        created: raw.created,
        updated: raw.updated,
        tags: raw.tags,
        facet,
        relationships: raw.relationships,
    })
}

/// Validate a tolerant risk facet: the two axes through the `"" -> None` enum seam,
/// `origin` through the text seam, `controls` passed through.
fn validate_facet(raw: RawRiskFacet) -> anyhow::Result<RiskFacet> {
    Ok(RiskFacet {
        likelihood: optional_enum(&raw.likelihood, "likelihood")?,
        impact: optional_enum(&raw.impact, "impact")?,
        origin: optional_text(raw.origin),
        controls: raw.controls,
    })
}

// ---------------------------------------------------------------------------
// Pure: render, scaffold
// ---------------------------------------------------------------------------

/// Render `backlog-<id>.toml` from the kind's embedded template by token
/// substitution. Risk picks the `[facet]` template; the four plain kinds the
/// light one. The `id/slug/title/status` keys round-trip into `meta::Meta` (VT-2);
/// `{{kind}}` is the stored discriminator (also the tree dir).
fn render_backlog_toml(
    item_kind: ItemKind,
    id: u32,
    slug: &str,
    title: &str,
    date: &str,
) -> anyhow::Result<String> {
    let template = if item_kind.has_facet() {
        "templates/backlog-risk.toml"
    } else {
        "templates/backlog.toml"
    };
    Ok(crate::install::asset_text(template)?
        .replace("{{id}}", &id.to_string())
        .replace("{{slug}}", &toml_string(slug))
        .replace("{{title}}", &toml_string(title))
        .replace("{{kind}}", item_kind.as_str())
        .replace("{{date}}", date))
}

/// Render `backlog-<id>.md` from the embedded prose template: `{{ref}}` (the
/// canonical id, e.g. `ISS-007`) + `{{title}}`. No frontmatter — metadata lives in
/// the sister toml.
fn render_backlog_md(canonical_id: &str, title: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/backlog.md")?
        .replace("{{ref}}", canonical_id)
        .replace("{{title}}", title))
}

/// The backlog fileset: sister TOML, prose body, and `<id>-<slug>` symlink, all
/// relative to the kind's tree root — structurally `requirement_scaffold` (§5.6).
/// The `item_kind` decides only the toml template (risk vs plain); the md and
/// symlink are kind-uniform. Shared by all five `Kind`s via their scaffold closure.
fn backlog_scaffold(item_kind: ItemKind, ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
    let id = ctx.id;
    let name = format!("{id:03}");
    Ok(vec![
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/{BACKLOG_STEM}-{name}.toml")),
            body: render_backlog_toml(item_kind, id, ctx.slug, ctx.title, ctx.date)?,
        },
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/{BACKLOG_STEM}-{name}.md")),
            body: render_backlog_md(ctx.canonical, ctx.title)?,
        },
        Artifact::Symlink {
            rel_path: PathBuf::from(format!("{name}-{}", ctx.slug)),
            target: name,
        },
    ])
}

// ---------------------------------------------------------------------------
// CLI entry points (thin)
// ---------------------------------------------------------------------------

/// `doctrine backlog new <kind> "<title>" [--slug S]` — the capture verb (PRD-009
/// REQ-049). Thin shell (§5.4): resolve the title/slug, inject the clock, reserve
/// the next id in the kind's INDEPENDENT namespace via the shared `Fresh` engine
/// path (monotonic id + race-retry inherited; `ISS-001` and `RSK-001` coexist),
/// then print the canonical `XXX-NNN` id. A pure mirror of `adr`/`spec` `run_new`,
/// dispatching the `Kind` on `item_kind`. Touches disk via the engine only — the
/// engine is unchanged (the R6 gate).
pub(crate) fn run_new(
    path: Option<PathBuf>,
    item_kind: ItemKind,
    title: Option<String>,
    slug: Option<String>,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let title = crate::input::resolve_title(title)?;
    let slug = crate::input::resolve_slug(&title, slug)?;
    let date = crate::clock::today();
    let out = entity::materialise(
        item_kind.kind(),
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
        .context("backlog kind must yield a numeric id")?;
    writeln!(
        io::stdout(),
        "Created {}: {}",
        item_kind.canonical_id(id),
        out.dir.display()
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Read: per-kind tree → validated items (total over a missing dir)
// ---------------------------------------------------------------------------

/// Read every item under one kind's tree into validated `BacklogItem`s. Rides
/// `entity::scan_ids` (numeric dirs only; **a missing tree → empty set**, the C2
/// total-function tolerance), then parses + `validate`s each `backlog-NNN.toml`.
/// The full-entity sibling of `meta::read_metas` (which yields only the 4 list
/// keys, no `kind`/`resolution`); `meta.rs` stays untouched (R6/EX-3).
fn read_kind(root: &Path, item_kind: ItemKind) -> anyhow::Result<Vec<BacklogItem>> {
    let tree = root.join(item_kind.kind().dir);
    let mut items = Vec::new();
    for id in entity::scan_ids(&tree)? {
        items.push(read_item(root, item_kind, id)?);
    }
    Ok(items)
}

/// Read ONE item's `backlog-<NNN>.toml` into a validated `BacklogItem` — the
/// single-id read shared by `read_kind`'s loop and `show` (DRY: one parse path).
/// A missing file is a hard error (the id must already be reserved — `show` never
/// implicitly creates, §5.5); the caller owns kind disambiguation (`parse_ref`).
fn read_item(root: &Path, item_kind: ItemKind, id: u32) -> anyhow::Result<BacklogItem> {
    let name = format!("{id:03}");
    let path = root
        .join(item_kind.kind().dir)
        .join(&name)
        .join(format!("{BACKLOG_STEM}-{name}.toml"));
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("backlog item not found at {}", path.display()))?;
    let raw: RawBacklogToml =
        toml::from_str(&text).with_context(|| format!("Failed to parse {}", path.display()))?;
    validate(raw)
}

/// Read all five kinds' trees, merged (declaration order, pre-sort). Each absent
/// kind dir contributes the empty set, so a virgin repo reads to `[]`.
fn read_all(root: &Path) -> anyhow::Result<Vec<BacklogItem>> {
    let mut items = Vec::new();
    for item_kind in ItemKind::ALL {
        items.extend(read_kind(root, item_kind)?);
    }
    Ok(items)
}

// ---------------------------------------------------------------------------
// Pure: filter (the visibility matrix) + render
// ---------------------------------------------------------------------------

/// The `list` filter axes — bundled so the verb stays under the CLI arg ceiling
/// and the compute half is one testable argument. All axes AND together.
#[derive(Debug, Default)]
struct ListFilter {
    kind: Option<ItemKind>,
    status: Option<Status>,
    tag: Option<String>,
    substr: Option<String>,
    all: bool,
}

/// Filter merged items by the AND of every set axis, then apply visibility
/// (design §5.4 / D5): an explicit `--status` keeps exactly that state (revealing
/// a terminal one); otherwise `--all` keeps everything and the default hides
/// terminal (`resolved`/`closed` — promoted falls out by the terminal rule, no
/// special branch). Pure; consumes and returns the item vec.
fn select(items: Vec<BacklogItem>, f: &ListFilter) -> Vec<BacklogItem> {
    let substr = f.substr.as_ref().map(|s| s.to_lowercase());
    items
        .into_iter()
        .filter(|i| f.kind.is_none_or(|k| i.kind == k))
        .filter(|i| f.tag.as_ref().is_none_or(|t| i.tags.iter().any(|x| x == t)))
        .filter(|i| {
            substr
                .as_ref()
                .is_none_or(|s| i.title.to_lowercase().contains(s))
        })
        .filter(|i| match f.status {
            Some(s) => i.status == s,
            None => f.all || !i.status.is_terminal(),
        })
        .collect()
}

/// Render rows as `id  kind  status  slug  title` over `meta::render_table` (the
/// SL-009 ragged-grid path — additive, NOT the fixed 4-col `format_list`). The id
/// is the canonical `XXX-NNN` (kind-disambiguated). Empty rows → `""` (the virgin
/// empty-table path). Pure.
fn format_rows(items: &[BacklogItem]) -> String {
    let grid: Vec<Vec<String>> = items
        .iter()
        .map(|i| {
            vec![
                i.kind.canonical_id(i.id),
                i.kind.as_str().to_string(),
                i.status.as_str().to_string(),
                i.slug.clone(),
                i.title.clone(),
            ]
        })
        .collect();
    crate::meta::render_table(&grid)
}

/// The `backlog list` rows as a string — the compute half of `run_list`,
/// extracted (the `adr::list_rows` precedent) so tests assert the rendered output
/// without capturing stdout. Read all kinds → filter → sort kind-then-id → render.
fn list_rows(root: &Path, f: &ListFilter) -> anyhow::Result<String> {
    let mut items = select(read_all(root)?, f);
    items.sort_by_key(|i| (i.kind.ordinal(), i.id));
    Ok(format_rows(&items))
}

/// `doctrine backlog list [--kind K] [--status S] [--tag T] [--all] [<substr>]`
/// — the survey verb (PRD-009 REQ-050). Thin shell (§5.4): find the root, build
/// the filter, print the rows verbatim (`list_rows` carries `render_table`'s own
/// trailing newline — no extra). Reads disk via `read_all` only; the engine and
/// the shared `meta` path are untouched (R6/EX-3).
pub(crate) fn run_list(
    path: Option<PathBuf>,
    kind: Option<ItemKind>,
    status: Option<Status>,
    tag: Option<String>,
    all: bool,
    substr: Option<String>,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let filter = ListFilter {
        kind,
        status,
        tag,
        substr,
        all,
    };
    write!(io::stdout(), "{}", list_rows(&root, &filter)?)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Pure: id parse + show render
// ---------------------------------------------------------------------------

/// Parse a canonical ref (`ISS-007`) into its `(kind, id)` — the `show` auto-detect
/// (§5.5 / R3). Split on the LAST `-`, upper-case the prefix (`iss-7` is tolerated),
/// resolve it via `ItemKind::from_prefix`, and parse the numeric tail as `u32`
/// (`ISS-7` and `ISS-007` both yield 7). An unknown prefix or a non-numeric tail is
/// a hard error — never an implicit create. The five counters are independent, so
/// the prefix is load-bearing for disambiguation (`ISS-1` ≠ `RSK-1`).
///
/// Deliberately NOT shared with `spec::resolve_spec_ref`: that sibling does NOT
/// upper-case (spec refs are always canonical), whereas backlog tolerates case here.
fn parse_ref(reference: &str) -> anyhow::Result<(ItemKind, u32)> {
    let (prefix, tail) = reference.rsplit_once('-').with_context(|| {
        format!("`{reference}` is not a canonical backlog ref (expected e.g. ISS-007)")
    })?;
    let kind = ItemKind::from_prefix(&prefix.to_uppercase()).with_context(|| {
        format!("unknown backlog prefix `{prefix}` in `{reference}` (expected ISS/IMP/CHR/RSK/IDE)")
    })?;
    let id: u32 = tail
        .parse()
        .with_context(|| format!("`{tail}` is not a numeric id in `{reference}`"))?;
    Ok((kind, id))
}

/// Render a `BacklogItem` for `show` — a pure fn of the item's OWN local state
/// ("cannot go stale"), so it reads no other file and surfaces no inbound refs
/// (the reverse view is the deferred registry surface's, ADR-004). House style:
/// `Vec<String>` parts each carrying their own newline, joined by `concat()` (the
/// `spec::render`/`format_rows` precedent — avoids the `push_str(&format!)` lint).
/// The facet block is gated on `item.facet` (risk only); relationship axes and the
/// optional fields render only when populated.
fn format_show(item: &BacklogItem) -> String {
    let mut parts: Vec<String> = Vec::new();

    // identity + the flat fields (resolution shown only on a terminal item).
    parts.push(format!(
        "{} — {}\n",
        item.kind.canonical_id(item.id),
        item.title
    ));
    let resolution = match item.resolution {
        Some(r) => format!(" · {}", r.as_str()),
        None => String::new(),
    };
    parts.push(format!(
        "{} · {} · {}{resolution}\n",
        item.slug,
        item.kind.as_str(),
        item.status.as_str(),
    ));
    parts.push(format!(
        "created {} · updated {}\n",
        item.created, item.updated
    ));
    if !item.tags.is_empty() {
        parts.push(format!("tags: {}\n", item.tags.join(", ")));
    }

    // risk facet (gated on the kind carrying one); each axis only when assessed.
    if let Some(facet) = &item.facet {
        parts.push("\n[facet]\n".to_string());
        if let Some(likelihood) = facet.likelihood {
            parts.push(format!("  likelihood: {}\n", likelihood.as_str()));
        }
        if let Some(impact) = facet.impact {
            parts.push(format!("  impact: {}\n", impact.as_str()));
        }
        if let Some(origin) = &facet.origin {
            parts.push(format!("  origin: {origin}\n"));
        }
        if !facet.controls.is_empty() {
            parts.push(format!("  controls: {}\n", facet.controls.join(", ")));
        }
    }

    // outbound relations (§5.5) — each axis only when non-empty; inbound is the
    // deferred registry surface's, NOT computed here (D-PHASE04-2 / ADR-004).
    let rel = &item.relationships;
    if !rel.slices.is_empty() || !rel.specs.is_empty() || !rel.drift.is_empty() {
        parts.push("\nrelationships:\n".to_string());
        for (label, refs) in [
            ("slices", &rel.slices),
            ("specs", &rel.specs),
            ("drift", &rel.drift),
        ] {
            if !refs.is_empty() {
                parts.push(format!("  {label}: {}\n", refs.join(", ")));
            }
        }
    }

    parts.concat()
}

/// `doctrine backlog show <ID>` — the inspect verb (PRD-009 REQ-051, §5.4). Thin
/// shell: find the root, `parse_ref` the id to its kind (prefix auto-detect), read
/// THAT item's single toml, render it to stdout. READ-ONLY — no mutation, no
/// cross-corpus scan (only the one item's file is opened); the render is pure over
/// the item's own state.
pub(crate) fn run_show(path: Option<PathBuf>, reference: &str) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (item_kind, id) = parse_ref(reference)?;
    let item = read_item(&root, item_kind, id)?;
    write!(io::stdout(), "{}", format_show(&item))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Pure: the status ⟺ resolution coupling + impure: the edit-in-place transition
// ---------------------------------------------------------------------------

/// The `status ⟺ resolution` coupling (PRD-009 REQ-059 / §5.5) plus the D9
/// re-open clear — a PURE decision over the *target* state, returning the
/// resolution string to write. A terminal status REQUIRES a `--resolution`; a
/// non-terminal status FORBIDS one and AUTO-CLEARS any prior resolution to `""`
/// (D9 — re-opening is one command, and the `resolution ⟺ terminal` invariant
/// holds post-write). `--resolution promoted` by hand is accepted (the promote
/// bridge is deferred; v1 is ungated). No clock/disk — the shell stamps `updated`.
fn validate_transition(
    status: Status,
    resolution: Option<Resolution>,
) -> anyhow::Result<&'static str> {
    match (status.is_terminal(), resolution) {
        (true, Some(r)) => Ok(r.as_str()),
        (true, None) => anyhow::bail!(
            "a terminal status (`{}`) requires `--resolution`",
            status.as_str()
        ),
        (false, Some(r)) => anyhow::bail!(
            "a non-terminal status (`{}`) takes no `--resolution` (got `{}`)",
            status.as_str(),
            r.as_str()
        ),
        (false, None) => Ok(""),
    }
}

/// Edit-preserving status/resolution transition on one authored `backlog-NNN.toml`
/// — the `adr::set_adr_status` precedent: `toml_edit` mutates the file in place, so
/// the inert `[facet]`/`[relationships]` tables, hand-added comments, and unknown
/// keys all survive (the file is never reserialised). Resolves the coupling via
/// `validate_transition`, carries the I5 no-op guard (an unchanged status+resolution
/// writes nothing), and the F-1 refuse (a malformed item missing a seeded key is
/// rejected, never corrupted by a tail-`insert` into a trailing subtable). The date
/// is injected by the shell; returns the resolution string written (for its confirm
/// line). A missing item file errors (read fails) — never an implicit create.
fn set_backlog_status(
    root: &Path,
    item_kind: ItemKind,
    id: u32,
    status: Status,
    resolution: Option<Resolution>,
    today: &str,
) -> anyhow::Result<&'static str> {
    let resolution = validate_transition(status, resolution)?;
    let name = format!("{id:03}");
    let path = root
        .join(item_kind.kind().dir)
        .join(&name)
        .join(format!("{BACKLOG_STEM}-{name}.toml"));
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("backlog item not found at {}", path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    // I5 no-op guard: unchanged status AND resolution → write nothing (mtime holds).
    let unchanged = doc.get("status").and_then(toml_edit::Item::as_str) == Some(status.as_str())
        && doc.get("resolution").and_then(toml_edit::Item::as_str) == Some(resolution);
    if unchanged {
        return Ok(resolution);
    }

    // F-1: `status`/`resolution`/`updated` are scaffold-seeded — this verb edits in
    // place, never creates. Their absence means a malformed (hand-edited) item; a tail
    // `insert` would append the key *after* the trailing `[facet]`/`[relationships]`
    // header, landing it inside that subtable (silent corruption). Refuse instead.
    let table = doc.as_table_mut();
    if !table.contains_key("status")
        || !table.contains_key("resolution")
        || !table.contains_key("updated")
    {
        anyhow::bail!(
            "malformed backlog item {name}: missing seeded `status`/`resolution`/`updated` (regenerate via `backlog new`)"
        );
    }
    table.insert("status", toml_edit::value(status.as_str()));
    table.insert("resolution", toml_edit::value(resolution));
    table.insert("updated", toml_edit::value(today));
    std::fs::write(&path, doc.to_string())
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(resolution)
}

/// `doctrine backlog edit <ID> --status <s> [--resolution <r>]` — the transition
/// verb (PRD-009 REQ-057/REQ-059, §5.4). Thin shell: find the root, `parse_ref` the
/// id to its kind (prefix auto-detect), apply the coupled edit in place (clock
/// injected), print the new state. A missing id hard-errors (never implicit create).
pub(crate) fn run_edit(
    path: Option<PathBuf>,
    reference: &str,
    status: Status,
    resolution: Option<Resolution>,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (item_kind, id) = parse_ref(reference)?;
    let written = set_backlog_status(
        &root,
        item_kind,
        id,
        status,
        resolution,
        &crate::clock::today(),
    )?;
    let suffix = if written.is_empty() {
        String::new()
    } else {
        format!(" · {written}")
    };
    writeln!(
        io::stdout(),
        "Edited {}: {}{suffix}",
        item_kind.canonical_id(id),
        status.as_str()
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{self, Inputs, LocalFs, MaterialiseRequest};
    use crate::meta::Meta;
    use std::fs;
    use std::path::Path;

    fn ctx_for(item_kind: ItemKind) -> ScaffoldCtx<'static> {
        let canonical: &'static str = match item_kind {
            ItemKind::Issue => "ISS-003",
            ItemKind::Improvement => "IMP-003",
            ItemKind::Chore => "CHR-003",
            ItemKind::Risk => "RSK-003",
            ItemKind::Idea => "IDE-003",
        };
        ScaffoldCtx {
            id: 3,
            canonical,
            slug: "token-expiry",
            title: "Token expiry",
            date: "2026-06-08",
        }
    }

    fn fresh(root: &Path, item_kind: ItemKind, slug: &str, title: &str) -> entity::Materialised {
        entity::materialise(
            item_kind.kind(),
            &LocalFs,
            root,
            &MaterialiseRequest::Fresh,
            &Inputs {
                slug,
                title,
                date: "2026-06-08",
            },
        )
        .unwrap()
    }

    // --- VT-1: per-kind scaffold fileset ---

    #[test]
    fn backlog_scaffold_lays_out_toml_md_symlink() {
        for kind in ItemKind::ALL {
            let ctx = ctx_for(kind);
            let fileset = backlog_scaffold(kind, &ctx).unwrap();
            assert_eq!(fileset.len(), 3, "{kind:?}: toml + md + symlink");

            assert!(
                matches!(&fileset[0],
                    Artifact::File { rel_path, body }
                    if rel_path == Path::new("003/backlog-003.toml")
                        && body.contains(&format!("kind = \"{}\"", kind.as_str()))),
                "{kind:?}: toml at tree-relative path with the stored kind"
            );
            assert!(
                matches!(&fileset[1],
                    Artifact::File { rel_path, body }
                    if rel_path == Path::new("003/backlog-003.md")
                        && body.contains(&format!("{}: Token expiry", ctx.canonical))),
                "{kind:?}: md carries the canonical ref"
            );
            assert!(
                matches!(&fileset[2],
                    Artifact::Symlink { rel_path, target }
                    if rel_path == Path::new("003-token-expiry") && target == "003"),
                "{kind:?}: NNN-slug alias last"
            );

            // risk carries `[facet]`; the four plain kinds omit it.
            let toml_body = match &fileset[0] {
                Artifact::File { body, .. } => body,
                Artifact::Symlink { .. } => panic!("first artifact is the toml"),
            };
            assert_eq!(
                toml_body.contains("[facet]"),
                kind.has_facet(),
                "{kind:?}: [facet] iff risk"
            );
        }
    }

    // --- VT-3: every kind seeds the mutable keys (the edit-in-place precondition) ---

    #[test]
    fn all_five_kinds_seed_status_resolution_updated_tags() {
        for kind in ItemKind::ALL {
            let body = render_backlog_toml(kind, 1, "s", "T", "2026-06-08").unwrap();
            assert!(
                body.contains("status = \"open\""),
                "{kind:?}: status seeded"
            );
            assert!(
                body.contains("resolution = \"\""),
                "{kind:?}: resolution seeded"
            );
            assert!(
                body.contains("updated = \"2026-06-08\""),
                "{kind:?}: updated seeded"
            );
            assert!(body.contains("tags = []"), "{kind:?}: tags seeded");
            assert!(!body.contains("{{"), "{kind:?}: no token survives render");
        }
    }

    // --- VT-2: the shared-Meta + full-entity round-trip, and the "" -> None seam ---

    #[test]
    fn rendered_toml_round_trips_into_meta_and_backlog_item() {
        let body = render_backlog_toml(ItemKind::Issue, 7, "fast-boot", "Fast boot", "2026-06-08")
            .unwrap();

        // the four list fields parse into the shared meta::Meta (status is a String there).
        let meta: Meta = toml::from_str(&body).unwrap();
        assert_eq!(
            meta,
            Meta {
                id: 7,
                slug: "fast-boot".to_string(),
                title: "Fast boot".to_string(),
                status: "open".to_string(),
            }
        );

        // the full entity validates; the seeded resolution `""` maps to None.
        let item = validate(toml::from_str::<RawBacklogToml>(&body).unwrap()).unwrap();
        assert_eq!(item.kind, ItemKind::Issue);
        assert_eq!(item.status, Status::Open);
        assert_eq!(item.resolution, None);
        assert!(item.facet.is_none(), "a plain kind has no facet");
        assert_eq!(item.relationships, Relationships::default());
    }

    #[test]
    fn render_backlog_toml_escapes_hostile_title_and_slug() {
        // SL-024: quoted-literal breakers (`"`, `\`, newline) round-trip.
        let title = crate::tomlfmt::HOSTILE_TITLE;
        let slug = crate::tomlfmt::HOSTILE_SLUG;
        let body = render_backlog_toml(ItemKind::Issue, 7, slug, title, "2026-06-08").unwrap();
        let parsed: Meta = toml::from_str(&body).unwrap();
        assert_eq!(parsed.slug, slug);
        assert_eq!(parsed.title, title);
    }

    #[test]
    fn risk_facet_levels_map_empty_to_none_and_parse_non_empty() {
        // a seeded risk toml: every facet axis empty → None.
        let seeded = render_backlog_toml(ItemKind::Risk, 1, "r", "R", "2026-06-08").unwrap();
        let item = validate(toml::from_str::<RawBacklogToml>(&seeded).unwrap()).unwrap();
        let facet = item.facet.expect("risk carries a facet");
        assert_eq!(facet.likelihood, None);
        assert_eq!(facet.impact, None);
        assert_eq!(facet.origin, None);
        assert!(facet.controls.is_empty());

        // an assessed risk: non-empty axes parse to their levels.
        let assessed = "\
id = 1
slug = \"r\"
title = \"R\"
kind = \"risk\"
status = \"open\"
resolution = \"\"
created = \"2026-06-08\"
updated = \"2026-06-08\"
tags = []

[facet]
likelihood = \"high\"
impact = \"critical\"
origin = \"audit\"
controls = [\"rate-limit\"]

[relationships]
slices = [\"SL-020\"]
specs = []
drift = []
";
        let item = validate(toml::from_str::<RawBacklogToml>(assessed).unwrap()).unwrap();
        let facet = item.facet.unwrap();
        assert_eq!(facet.likelihood, Some(RiskLevel::High));
        assert_eq!(facet.impact, Some(RiskLevel::Critical));
        assert_eq!(facet.origin.as_deref(), Some("audit"));
        assert_eq!(facet.controls, vec!["rate-limit"]);
        assert_eq!(item.relationships.slices, vec!["SL-020"]);
    }

    #[test]
    fn validate_errors_on_an_unknown_enum_token() {
        let body = "\
id = 1
slug = \"s\"
title = \"T\"
kind = \"issue\"
status = \"open\"
resolution = \"bogus\"
created = \"2026-06-08\"
updated = \"2026-06-08\"
tags = []
";
        let raw: RawBacklogToml = toml::from_str(body).unwrap();
        assert!(
            validate(raw).is_err(),
            "an unknown resolution token is rejected"
        );
    }

    // --- the value mirrors + discriminator helpers ---

    #[test]
    fn status_is_terminal_is_backlog_local() {
        assert!(Status::Resolved.is_terminal());
        assert!(Status::Closed.is_terminal());
        assert!(!Status::Open.is_terminal());
        assert!(!Status::Triaged.is_terminal());
        assert!(!Status::Started.is_terminal());
    }

    #[test]
    fn item_kind_from_prefix_round_trips_each_kind() {
        for kind in ItemKind::ALL {
            assert_eq!(ItemKind::from_prefix(kind.prefix()), Some(kind));
        }
        assert_eq!(ItemKind::from_prefix("REQ"), None);
        // the five prefixes are distinct.
        let prefixes: std::collections::BTreeSet<&str> =
            ItemKind::ALL.iter().map(|k| k.prefix()).collect();
        assert_eq!(prefixes.len(), 5);
    }

    #[test]
    fn resolution_and_risk_level_render_mirror_serde() {
        assert_eq!(Resolution::WontDo.as_str(), "wont-do");
        assert_eq!(Resolution::Promoted.as_str(), "promoted");
        assert_eq!(RiskLevel::Critical.as_str(), "critical");
        // the mirror matches the parse direction.
        assert_eq!(
            parse_enum::<Resolution>("wont-do", "resolution").unwrap(),
            Resolution::WontDo
        );
    }

    // --- EX-1 / VT-1: materialise(Fresh) reserves per-kind, counters independent ---

    #[test]
    fn materialise_fresh_reserves_each_kind_in_its_own_namespace() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // issue and risk both start at 001 — independent reservation namespaces.
        let i1 = fresh(root, ItemKind::Issue, "auth", "Auth");
        let r1 = fresh(root, ItemKind::Risk, "expiry", "Expiry");
        assert_eq!(i1.eid.numeric_id(), Some(1));
        assert_eq!(r1.eid.numeric_id(), Some(1));

        assert!(
            root.join(".doctrine/backlog/issue/001/backlog-001.toml")
                .is_file()
        );
        assert!(
            root.join(".doctrine/backlog/issue/001/backlog-001.md")
                .is_file()
        );
        assert_eq!(
            fs::read_link(root.join(".doctrine/backlog/issue/001-auth")).unwrap(),
            Path::new("001")
        );

        // the risk item on disk carries the `[facet]`; the issue item does not.
        let risk_toml =
            fs::read_to_string(root.join(".doctrine/backlog/risk/001/backlog-001.toml")).unwrap();
        assert!(risk_toml.contains("[facet]"));
        let issue_toml =
            fs::read_to_string(root.join(".doctrine/backlog/issue/001/backlog-001.toml")).unwrap();
        assert!(!issue_toml.contains("[facet]"));

        // a second issue lands 002; the risk counter is untouched (separate dirs).
        let i2 = fresh(root, ItemKind::Issue, "login", "Login");
        assert_eq!(i2.eid.numeric_id(), Some(2));
        let r2 = fresh(root, ItemKind::Risk, "leak", "Leak");
        assert_eq!(r2.eid.numeric_id(), Some(2));

        // the materialised toml round-trips through validate end-to-end.
        let item = validate(toml::from_str::<RawBacklogToml>(&risk_toml).unwrap()).unwrap();
        assert_eq!(item.kind, ItemKind::Risk);
        assert_eq!(item.id, 1);
    }

    // --- PHASE-02: the `backlog new` verb (thin shell over the engine) ---

    /// Drive the real `new` verb with an explicit root (short-circuits detection)
    /// and an explicit title (avoids stdin).
    fn new_item(root: &Path, kind: ItemKind, title: &str) {
        run_new(
            Some(root.to_path_buf()),
            kind,
            Some(title.to_string()),
            None,
        )
        .unwrap();
    }

    fn issue_dir(root: &Path, id: &str) -> PathBuf {
        root.join(format!(".doctrine/backlog/issue/{id}"))
    }

    // --- VT-1: monotonic per kind ---

    #[test]
    fn backlog_new_reserves_monotonic_per_kind() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        new_item(root, ItemKind::Issue, "Auth");
        new_item(root, ItemKind::Issue, "Login");

        assert!(issue_dir(root, "001").join("backlog-001.toml").is_file());
        assert!(issue_dir(root, "001").join("backlog-001.md").is_file());
        assert_eq!(
            fs::read_link(root.join(".doctrine/backlog/issue/001-auth")).unwrap(),
            Path::new("001")
        );
        // a second `new` lands the next id (engine race-retry inherited).
        assert!(issue_dir(root, "002").join("backlog-002.toml").is_file());
    }

    // --- VT-1: the five counters are independent (separate dirs) ---

    #[test]
    fn backlog_new_counters_isolated_across_kinds() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // an issue and a risk both open at 001 — independent namespaces.
        new_item(root, ItemKind::Issue, "Auth");
        new_item(root, ItemKind::Risk, "Expiry");
        assert!(issue_dir(root, "001").join("backlog-001.toml").is_file());
        assert!(
            root.join(".doctrine/backlog/risk/001/backlog-001.toml")
                .is_file()
        );

        // a second issue advances to 002; the risk counter is untouched.
        new_item(root, ItemKind::Issue, "Login");
        assert!(issue_dir(root, "002").join("backlog-002.toml").is_file());
        assert!(
            !root.join(".doctrine/backlog/risk/002").exists(),
            "an issue create must not advance the risk counter"
        );
    }

    // --- VT-2: the kind-correct template seeds onto disk ---

    #[test]
    fn backlog_new_seeds_kind_template() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        new_item(root, ItemKind::Risk, "Token expiry");
        new_item(root, ItemKind::Issue, "Token expiry");

        // risk seeds `[facet]`; the plain issue does not. Both default `open`.
        let risk =
            fs::read_to_string(root.join(".doctrine/backlog/risk/001/backlog-001.toml")).unwrap();
        assert!(risk.contains("[facet]"), "risk seeds a facet");
        assert!(risk.contains("status = \"open\""), "status defaults open");

        let issue = fs::read_to_string(issue_dir(root, "001").join("backlog-001.toml")).unwrap();
        assert!(!issue.contains("[facet]"), "a plain kind has no facet");
        assert!(issue.contains("status = \"open\""));

        // the printed canonical id (`ISS-001`) matches the reserved dir: the item
        // validates and carries id 1 under the issue tree.
        let item = validate(toml::from_str::<RawBacklogToml>(&issue).unwrap()).unwrap();
        assert_eq!(item.kind, ItemKind::Issue);
        assert_eq!(item.id, 1);
        assert_eq!(ItemKind::Issue.canonical_id(item.id), "ISS-001");
    }

    // --- VT-3: the gitignore negation makes a created item git-addable (R5) ---

    #[test]
    fn created_backlog_item_is_git_addable() {
        fn git(root: &Path, args: &[&str]) -> std::process::Output {
            std::process::Command::new("git")
                .arg("-C")
                .arg(root)
                .args(args)
                .output()
                .expect("spawn git")
        }

        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        assert!(git(root, &["init", "-b", "main"]).status.success());
        // the dogfood blanket-ignore + the PHASE-02 backlog negation (no inline #).
        fs::write(
            root.join(".gitignore"),
            ".doctrine/*\n!.doctrine/backlog/\n",
        )
        .unwrap();

        new_item(root, ItemKind::Issue, "Auth");
        let item = ".doctrine/backlog/issue/001/backlog-001.toml";

        // `check-ignore -q` exits 1 when the path is NOT ignored — negation is live.
        assert_eq!(
            git(root, &["check-ignore", "-q", item]).status.code(),
            Some(1),
            "the negation must un-ignore the backlog item"
        );
        // and `git add` of the item succeeds (no "paths are ignored").
        let add = git(root, &["add", item]);
        assert!(
            add.status.success(),
            "git add failed: {}",
            String::from_utf8_lossy(&add.stderr)
        );
    }

    // --- PHASE-03: the `backlog list` survey (visibility / filter / order) ---

    /// Write a complete `backlog-NNN.toml` directly under a kind's tree — a true
    /// unit fixture (the `meta::tests::write_meta_toml` precedent) that lets a
    /// non-`open`/terminal status + a resolution be seeded without the (unbuilt,
    /// PHASE-05) `edit` verb. Exercises the real reader: `scan_ids` + `validate`.
    fn write_item(
        root: &Path,
        kind: ItemKind,
        id: u32,
        status: &str,
        resolution: &str,
        slug: &str,
        title: &str,
        tags: &[&str],
    ) {
        let tree = root.join(kind.kind().dir);
        let name = format!("{id:03}");
        let dir = tree.join(&name);
        fs::create_dir_all(&dir).unwrap();
        let tags_lit = tags
            .iter()
            .map(|t| format!("\"{t}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let body = format!(
            "id = {id}\nslug = \"{slug}\"\ntitle = \"{title}\"\nkind = \"{}\"\n\
             status = \"{status}\"\nresolution = \"{resolution}\"\n\
             created = \"2026-06-08\"\nupdated = \"2026-06-08\"\ntags = [{tags_lit}]\n",
            kind.as_str()
        );
        fs::write(dir.join(format!("backlog-{name}.toml")), body).unwrap();
    }

    /// The first column (canonical id) of each rendered row, in render order.
    fn ids(out: &str) -> Vec<String> {
        out.lines()
            .map(|l| l.split_whitespace().next().unwrap().to_string())
            .collect()
    }

    // --- VT-1: the visibility matrix ---

    #[test]
    fn backlog_list_default_hides_terminal() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_item(root, ItemKind::Issue, 1, "open", "", "a", "Alpha", &[]);
        write_item(root, ItemKind::Issue, 2, "triaged", "", "b", "Bravo", &[]);
        write_item(root, ItemKind::Issue, 3, "started", "", "c", "Charlie", &[]);
        write_item(
            root,
            ItemKind::Issue,
            4,
            "resolved",
            "fixed",
            "d",
            "Delta",
            &[],
        );
        write_item(root, ItemKind::Issue, 5, "closed", "done", "e", "Echo", &[]);

        let out = list_rows(root, &ListFilter::default()).unwrap();
        assert_eq!(
            ids(&out),
            vec!["ISS-001", "ISS-002", "ISS-003"],
            "default shows only the active states; resolved/closed hidden"
        );
    }

    #[test]
    fn backlog_list_all_and_explicit_status_reveal() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_item(root, ItemKind::Issue, 1, "open", "", "a", "Alpha", &[]);
        write_item(
            root,
            ItemKind::Issue,
            4,
            "resolved",
            "fixed",
            "d",
            "Delta",
            &[],
        );
        write_item(root, ItemKind::Issue, 5, "closed", "done", "e", "Echo", &[]);
        // a promoted item is terminal (status resolved, resolution=promoted) — it
        // must hide by default and reveal by the terminal rule, no special branch.
        write_item(
            root,
            ItemKind::Issue,
            6,
            "resolved",
            "promoted",
            "f",
            "Foxtrot",
            &[],
        );

        // --all reveals every state.
        let all = list_rows(
            root,
            &ListFilter {
                all: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            ids(&all),
            vec!["ISS-001", "ISS-004", "ISS-005", "ISS-006"],
            "--all shows active + terminal + promoted"
        );

        // an explicit --status resolved reveals exactly that terminal state
        // (open hidden, closed hidden; the promoted resolved item included).
        let resolved = list_rows(
            root,
            &ListFilter {
                status: Some(Status::Resolved),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            ids(&resolved),
            vec!["ISS-004", "ISS-006"],
            "--status resolved reveals the resolved (incl. promoted) items only"
        );
    }

    // --- VT-2: filters AND together; kind-then-id order ---

    #[test]
    fn backlog_list_filters_and_together() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_item(
            root,
            ItemKind::Issue,
            1,
            "open",
            "",
            "auth-bug",
            "Auth bug",
            &["security"],
        );
        write_item(
            root,
            ItemKind::Issue,
            2,
            "open",
            "",
            "login",
            "Login flow",
            &["ui"],
        );
        write_item(
            root,
            ItemKind::Risk,
            1,
            "open",
            "",
            "auth-risk",
            "Auth risk",
            &["security"],
        );

        // --kind issue AND --tag security AND substring "auth" → only ISS-001.
        let out = list_rows(
            root,
            &ListFilter {
                kind: Some(ItemKind::Issue),
                tag: Some("security".to_string()),
                substr: Some("auth".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            ids(&out),
            vec!["ISS-001"],
            "the axes intersect: ISS-002 lacks the tag/substr, RSK-001 is the wrong kind"
        );
    }

    #[test]
    fn backlog_list_kind_then_id_order() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // write out of order, across kinds, to prove the sort (not insertion order).
        write_item(root, ItemKind::Risk, 1, "open", "", "r", "R", &[]);
        write_item(root, ItemKind::Issue, 2, "open", "", "i2", "I2", &[]);
        write_item(root, ItemKind::Issue, 1, "open", "", "i1", "I1", &[]);
        write_item(root, ItemKind::Idea, 1, "open", "", "d", "D", &[]);
        write_item(root, ItemKind::Chore, 1, "open", "", "c", "C", &[]);

        let out = list_rows(root, &ListFilter::default()).unwrap();
        assert_eq!(
            ids(&out),
            vec!["ISS-001", "ISS-002", "CHR-001", "RSK-001", "IDE-001"],
            "kind declaration order (issue/improvement/chore/risk/idea) then ascending id"
        );
    }

    // --- VT-3: total-function reads (missing dir / virgin repo) ---

    #[test]
    fn backlog_list_missing_dir_is_empty_set() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // only the issue tree exists; the other four kind dirs are absent.
        write_item(root, ItemKind::Issue, 1, "open", "", "a", "Alpha", &[]);

        let out = list_rows(root, &ListFilter::default()).unwrap();
        assert_eq!(
            ids(&out),
            vec!["ISS-001"],
            "an absent kind dir contributes the empty set, never an error"
        );
    }

    #[test]
    fn backlog_list_virgin_repo_empty_table() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // no `.doctrine/backlog` at all — every kind reads empty.
        let out = list_rows(root, &ListFilter::default()).unwrap();
        assert_eq!(
            out, "",
            "a virgin repo prints an empty table, never an error"
        );
    }

    // --- PHASE-04: the `backlog show <ID>` inspect verb (id parse + render) ---

    // --- VT-2: id-parse tolerance + both hard-error modes ---

    #[test]
    fn backlog_show_id_parse_tolerance() {
        // `ISS-7` and `ISS-007` both parse to (Issue, 7); case is tolerated.
        assert_eq!(parse_ref("ISS-7").unwrap(), (ItemKind::Issue, 7));
        assert_eq!(parse_ref("ISS-007").unwrap(), (ItemKind::Issue, 7));
        assert_eq!(parse_ref("iss-7").unwrap(), (ItemKind::Issue, 7));
        // each prefix routes to its own kind — the counters are independent.
        assert_eq!(parse_ref("RSK-001").unwrap(), (ItemKind::Risk, 1));
        assert_eq!(parse_ref("IDE-12").unwrap(), (ItemKind::Idea, 12));
    }

    #[test]
    fn backlog_show_unknown_prefix_errors() {
        // an unknown prefix and a non-numeric tail each hard-error (never a create).
        assert!(parse_ref("REQ-001").is_err(), "unknown prefix rejected");
        assert!(parse_ref("ISS-abc").is_err(), "non-numeric tail rejected");
        assert!(
            parse_ref("nodash").is_err(),
            "a ref with no `-` is rejected"
        );
    }

    // --- VT-1: auto-detect kind from prefix; identity + facet + relations render ---

    #[test]
    fn backlog_show_auto_detects_kind_from_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // a plain issue and an assessed risk, both reserved id 1 (independent trees).
        new_item(root, ItemKind::Issue, "Auth bug");
        let issue = read_item(root, ItemKind::Issue, 1).unwrap();
        let issue_out = format_show(&issue);
        assert!(
            issue_out.starts_with("ISS-001 — Auth bug\n"),
            "identity line: {issue_out}"
        );
        assert!(
            issue_out.contains("· issue · open"),
            "flat field line carries kind + status: {issue_out}"
        );
        assert!(
            !issue_out.contains("[facet]"),
            "a plain kind shows no facet block: {issue_out}"
        );

        // an assessed risk (seeded directly) shows its facet axes.
        write_assessed_risk(root, 1);
        let risk = read_item(root, ItemKind::Risk, 1).unwrap();
        let risk_out = format_show(&risk);
        assert!(risk_out.starts_with("RSK-001 — Token expiry\n"));
        assert!(risk_out.contains("[facet]"), "risk shows the facet block");
        assert!(risk_out.contains("likelihood: high"));
        assert!(risk_out.contains("impact: critical"));
        assert!(risk_out.contains("controls: rate-limit"));
    }

    // --- VT-3: outbound relations render; inbound is NOT surfaced (ADR-004) ---

    #[test]
    fn backlog_show_renders_outbound_only() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // ISS-001 points OUT at SL-020; a *separate* item (ISS-002) points AT it.
        write_related(root, ItemKind::Issue, 1, &["SL-020"], &["PRD-009"]);
        write_related(root, ItemKind::Issue, 2, &[], &[]);

        let out = format_show(&read_item(root, ItemKind::Issue, 1).unwrap());
        assert!(out.contains("relationships:"), "the outbound seam renders");
        assert!(out.contains("slices: SL-020"), "outbound slice ref shown");
        assert!(out.contains("specs: PRD-009"), "outbound spec ref shown");

        // an item with no outbound relations renders no relationships block —
        // and the reverse view (who points AT it) is never computed here.
        let bare = format_show(&read_item(root, ItemKind::Issue, 2).unwrap());
        assert!(
            !bare.contains("relationships:"),
            "no outbound relations → no block (inbound never surfaced): {bare}"
        );
    }

    /// Overwrite a reserved risk item with an assessed `[facet]` — exercises the
    /// real read+validate path for a populated facet without the (PHASE-05) `edit`.
    fn write_assessed_risk(root: &Path, id: u32) {
        let name = format!("{id:03}");
        let dir = root.join(RISK_KIND.dir).join(&name);
        fs::create_dir_all(&dir).unwrap();
        let body = format!(
            "id = {id}\nslug = \"token-expiry\"\ntitle = \"Token expiry\"\nkind = \"risk\"\n\
             status = \"open\"\nresolution = \"\"\ncreated = \"2026-06-08\"\n\
             updated = \"2026-06-08\"\ntags = []\n\n[facet]\nlikelihood = \"high\"\n\
             impact = \"critical\"\norigin = \"audit\"\ncontrols = [\"rate-limit\"]\n\n\
             [relationships]\nslices = []\nspecs = []\ndrift = []\n"
        );
        fs::write(dir.join(format!("backlog-{name}.toml")), body).unwrap();
    }

    /// Write an item carrying seeded OUTBOUND `slices`/`specs` relations directly.
    fn write_related(root: &Path, kind: ItemKind, id: u32, slices: &[&str], specs: &[&str]) {
        let name = format!("{id:03}");
        let dir = root.join(kind.kind().dir).join(&name);
        fs::create_dir_all(&dir).unwrap();
        let lit = |xs: &[&str]| {
            xs.iter()
                .map(|x| format!("\"{x}\""))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let body = format!(
            "id = {id}\nslug = \"s\"\ntitle = \"T\"\nkind = \"{}\"\nstatus = \"open\"\n\
             resolution = \"\"\ncreated = \"2026-06-08\"\nupdated = \"2026-06-08\"\ntags = []\n\n\
             [relationships]\nslices = [{}]\nspecs = [{}]\ndrift = []\n",
            kind.as_str(),
            lit(slices),
            lit(specs),
        );
        fs::write(dir.join(format!("backlog-{name}.toml")), body).unwrap();
    }

    // --- PHASE-05: the `backlog edit` coupled transition ---

    /// Validate one item's on-disk state via the real reader. Panics if absent.
    fn read_back(root: &Path, kind: ItemKind, id: u32) -> BacklogItem {
        read_item(root, kind, id).unwrap()
    }

    // --- VT-1 / VT-2: the coupling + D9, as a pure decision ---

    #[test]
    fn validate_transition_couples_both_directions_and_d9_clears() {
        // a terminal status REQUIRES a resolution (both terminal states).
        assert!(validate_transition(Status::Resolved, None).is_err());
        assert!(validate_transition(Status::Closed, None).is_err());
        // terminal + resolution → that resolution's kebab string.
        assert_eq!(
            validate_transition(Status::Resolved, Some(Resolution::Fixed)).unwrap(),
            "fixed"
        );
        // a non-terminal status FORBIDS a resolution (rejected outright).
        assert!(validate_transition(Status::Started, Some(Resolution::Fixed)).is_err());
        assert!(validate_transition(Status::Open, Some(Resolution::Promoted)).is_err());
        // a non-terminal status with no resolution → D9 auto-clear to "".
        assert_eq!(validate_transition(Status::Open, None).unwrap(), "");
        assert_eq!(validate_transition(Status::Triaged, None).unwrap(), "");
    }

    // --- VT-3: edit-preserving (comments/unknowns survive); updated bumps ---

    #[test]
    fn backlog_edit_is_edit_preserving() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        new_item(root, ItemKind::Issue, "Auth"); // real template: [relationships] subtable
        let path = issue_dir(root, "001").join("backlog-001.toml");

        // hand-add an inert top-level table + a comment (the F-1 corruption hazard the
        // in-place edit must NOT disturb).
        let mut body = fs::read_to_string(&path).unwrap();
        body.push_str("\n# hand note — keep me\n[custom]\nkeep = \"yes\"\n");
        fs::write(&path, &body).unwrap();

        set_backlog_status(
            root,
            ItemKind::Issue,
            1,
            Status::Resolved,
            Some(Resolution::Fixed),
            "2026-07-01",
        )
        .unwrap();

        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("# hand note — keep me"), "comment survives");
        assert!(after.contains("[custom]"), "inert table survives verbatim");
        assert!(after.contains("keep = \"yes\""), "unknown key survives");
        assert!(
            after.contains("[relationships]"),
            "seeded subtable survives"
        );
        assert!(after.contains("status = \"resolved\""));
        assert!(after.contains("resolution = \"fixed\""));
        assert!(after.contains("updated = \"2026-07-01\""), "updated bumps");

        // and it still round-trips the reader.
        let item = read_back(root, ItemKind::Issue, 1);
        assert_eq!(item.status, Status::Resolved);
        assert_eq!(item.resolution, Some(Resolution::Fixed));
    }

    // --- VT-3: the no-op guard writes nothing ---

    #[test]
    fn backlog_edit_noop_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        new_item(root, ItemKind::Issue, "Auth"); // status open, resolution ""
        let path = issue_dir(root, "001").join("backlog-001.toml");
        let before = fs::read_to_string(&path).unwrap();
        let mtime_before = fs::metadata(&path).unwrap().modified().unwrap();

        // re-open an already-open item (status open, no resolution) → no-op.
        let written =
            set_backlog_status(root, ItemKind::Issue, 1, Status::Open, None, "2026-07-01").unwrap();
        assert_eq!(
            written, "",
            "the no-op still reports the resolved (empty) state"
        );

        assert_eq!(
            before,
            fs::read_to_string(&path).unwrap(),
            "a no-op writes nothing — content byte-identical"
        );
        assert_eq!(
            mtime_before,
            fs::metadata(&path).unwrap().modified().unwrap(),
            "a no-op leaves mtime untouched"
        );
    }

    // --- VT-3: a malformed item (missing a seeded key) is refused, not corrupted ---

    #[test]
    fn backlog_edit_refuses_malformed_missing_seeded_key() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // hand-corrupted: the seeded `resolution` key is gone.
        let d = root.join(ItemKind::Issue.kind().dir).join("001");
        fs::create_dir_all(&d).unwrap();
        let malformed = "id = 1\nslug = \"a\"\ntitle = \"A\"\nkind = \"issue\"\n\
             status = \"open\"\ncreated = \"2026-06-08\"\nupdated = \"2026-06-08\"\ntags = []\n";
        let path = d.join("backlog-001.toml");
        fs::write(&path, malformed).unwrap();

        let err = set_backlog_status(
            root,
            ItemKind::Issue,
            1,
            Status::Resolved,
            Some(Resolution::Fixed),
            "2026-07-01",
        );
        assert!(err.is_err(), "a missing seeded key is refused");
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            malformed,
            "the file is untouched — never tail-inserted into corruption"
        );
    }

    // --- VT-3: a missing id hard-errors, never an implicit create ---

    #[test]
    fn backlog_edit_missing_id_hard_errors() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let err = set_backlog_status(
            root,
            ItemKind::Issue,
            99,
            Status::Started,
            None,
            "2026-07-01",
        );
        assert!(err.is_err(), "editing a nonexistent id errors");
        assert!(
            !issue_dir(root, "099").exists(),
            "the failed edit creates nothing"
        );
    }

    // --- VT-2: re-open auto-clears the resolution (D9); promoted is ungated ---

    #[test]
    fn backlog_edit_reopen_auto_clears_resolution() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        new_item(root, ItemKind::Issue, "Auth");

        // resolve it.
        set_backlog_status(
            root,
            ItemKind::Issue,
            1,
            Status::Resolved,
            Some(Resolution::Fixed),
            "2026-07-01",
        )
        .unwrap();
        let resolved = read_back(root, ItemKind::Issue, 1);
        assert_eq!(resolved.status, Status::Resolved);
        assert_eq!(resolved.resolution, Some(Resolution::Fixed));

        // re-open (no --resolution) → D9 clears the resolution; the invariant holds.
        set_backlog_status(root, ItemKind::Issue, 1, Status::Open, None, "2026-07-02").unwrap();
        let reopened = read_back(root, ItemKind::Issue, 1);
        assert_eq!(reopened.status, Status::Open);
        assert_eq!(reopened.resolution, None, "D9: re-open clears resolution");

        // a `promoted` item is hand-re-openable (ungated — the OQ-003 escape hatch).
        set_backlog_status(
            root,
            ItemKind::Issue,
            1,
            Status::Closed,
            Some(Resolution::Promoted),
            "2026-07-03",
        )
        .unwrap();
        set_backlog_status(root, ItemKind::Issue, 1, Status::Open, None, "2026-07-04").unwrap();
        let after = read_back(root, ItemKind::Issue, 1);
        assert_eq!(after.status, Status::Open);
        assert_eq!(after.resolution, None, "a promoted item re-opens ungated");
    }

    // --- VT-5: non-canon status/resolution rejected at the clap ValueEnum boundary ---

    #[test]
    fn backlog_edit_rejects_noncanon_status_and_resolution() {
        use clap::ValueEnum;
        assert!(Status::from_str("bogus", false).is_err());
        assert!(Resolution::from_str("nope", false).is_err());
        // the canon tokens (kebab) still parse — the lifecycle stays otherwise ungated.
        assert_eq!(Status::from_str("started", false).unwrap(), Status::Started);
        assert_eq!(
            Resolution::from_str("wont-do", false).unwrap(),
            Resolution::WontDo
        );
    }

    // --- VT-1: coupling both directions + missing id, through the real shell ---

    #[test]
    fn run_edit_drives_the_coupled_transition() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        new_item(root, ItemKind::Risk, "Token leak");

        // a terminal status without a resolution is rejected through the shell.
        assert!(run_edit(Some(root.to_path_buf()), "RSK-001", Status::Resolved, None).is_err());
        // a valid terminal+resolution is accepted.
        run_edit(
            Some(root.to_path_buf()),
            "RSK-001",
            Status::Resolved,
            Some(Resolution::Mitigated),
        )
        .unwrap();
        let item = read_back(root, ItemKind::Risk, 1);
        assert_eq!(item.status, Status::Resolved);
        assert_eq!(item.resolution, Some(Resolution::Mitigated));

        // a missing id hard-errors through the shell (never an implicit create).
        assert!(run_edit(Some(root.to_path_buf()), "RSK-099", Status::Started, None).is_err());
    }
}
