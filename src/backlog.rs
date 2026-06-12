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

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::backlog_order::{BacklogOrder, ItemId, OrderInput, Override, OverrideReason};

use crate::entity::{
    self, Artifact, Fileset, Inputs, Kind, LocalFs, MaterialiseRequest, ScaffoldCtx,
};
use crate::listing::{self, Format, ListArgs};
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
pub(crate) const ISSUE_KIND: Kind = Kind {
    dir: ".doctrine/backlog/issue",
    prefix: "ISS",
    scaffold: |c| backlog_scaffold(ItemKind::Issue, c),
};

/// The improvement kind: an enhancement to existing behaviour.
pub(crate) const IMPROVEMENT_KIND: Kind = Kind {
    dir: ".doctrine/backlog/improvement",
    prefix: "IMP",
    scaffold: |c| backlog_scaffold(ItemKind::Improvement, c),
};

/// The chore kind: maintenance with no user-visible behaviour change.
pub(crate) const CHORE_KIND: Kind = Kind {
    dir: ".doctrine/backlog/chore",
    prefix: "CHR",
    scaffold: |c| backlog_scaffold(ItemKind::Chore, c),
};

/// The risk kind: a tracked risk — the only kind carrying a `[facet]`.
pub(crate) const RISK_KIND: Kind = Kind {
    dir: ".doctrine/backlog/risk",
    prefix: "RSK",
    scaffold: |c| backlog_scaffold(ItemKind::Risk, c),
};

/// The idea kind: a speculative possibility, not yet committed work.
pub(crate) const IDEA_KIND: Kind = Kind {
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
    /// `Kind` so the prefix is never hardcoded twice. `pub(crate)` so the
    /// `backlog_order` adapter's `ItemId` orders by `(prefix, id)` — the
    /// canonical-id ascending tiebreak — without re-rendering a string per compare.
    pub(crate) const fn prefix(self) -> &'static str {
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
    /// `Kind` (single source). `pub(crate)` so the `backlog_order` adapter's
    /// `ItemId` renders through the same single source.
    pub(crate) fn canonical_id(self, id: u32) -> String {
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
    /// terminal` coupling (`edit`) and the hide-terminal `list` rule — reused by
    /// `is_hidden` as the SL-025 `backlog list` hide-set (no new predicate, design §5.3).
    const fn is_terminal(self) -> bool {
        matches!(self, Status::Resolved | Status::Closed)
    }
}

/// The `backlog list` known-status set (A-2) — the five `Status` variants, the
/// authority `--status` is validated against. Lockstep-guarded against the enum by
/// `backlog_statuses_matches_the_variants`. backlog has a CLOSED status enum, so a
/// *stored* status is always in-vocabulary — no drift marker is possible.
const BACKLOG_STATUSES: &[&str] = &["open", "triaged", "started", "resolved", "closed"];

/// The `backlog list` hide-set fed to `listing::retain` (design §5.3): the terminal
/// statuses drop from the default list. This is the stringly bridge over the typed
/// [`Status::is_terminal`] — the SAME predicate, no new terminal set. An out-of-vocab
/// token (impossible on a serde-validated item, but `retain` is stringly) is treated
/// as not-hidden. `--all` or any explicit `--status` overrides (handled in `retain`).
fn is_hidden(status: &str) -> bool {
    parse_enum::<Status>(status, "status").is_ok_and(Status::is_terminal)
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

/// A soft-sequence edge (PRD-009): this item runs `after` the predecessor `to`,
/// with an optional per-edge `rank` (default `0` — a plain soft edge; a non-zero
/// rank is a manual tie-break hint). A bare `{ to = "X" }` is rank 0.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AfterEdge {
    to: String,
    #[serde(default)]
    rank: i32,
}

/// A `triggers` rider (PRD-009 §5.7): the source `globs` this item watches, with
/// an optional free-text `note` (default `""` — globs-only). FIELD ONLY this
/// phase — the IMP-026 staleness mask is out of scope.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Trigger {
    #[serde(default)]
    globs: Vec<String>,
    #[serde(default)]
    note: String,
}

/// Outbound-only relations (ADR-004): a backlog item points OUT at the slices,
/// specs, and drift it touches, plus three item→item axes (PRD-009) — `needs`
/// (hard prerequisite, payload-free), `after` (soft manual sequence, per-edge
/// optional `rank`), and the `triggers` rider (watched source globs). The reverse
/// view is derived (deferred, PRD-011). Shared verbatim by the raw and validated
/// layers (no `"" -> None` seam), seeded empty so `#[serde(default)]` parses a
/// virgin item.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
struct Relationships {
    #[serde(default)]
    slices: Vec<String>,
    #[serde(default)]
    specs: Vec<String>,
    #[serde(default)]
    drift: Vec<String>,
    #[serde(default)]
    needs: Vec<String>,
    #[serde(default)]
    after: Vec<AfterEdge>,
    #[serde(default)]
    triggers: Vec<Trigger>,
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

/// The risk exposure score — `likelihood × impact` (1..=16) when BOTH axes are
/// assessed, else `0`. The within-level ordering fallback the `backlog_order`
/// adapter consumes (design §5.1 tier 3, VT-4): `0` is the baseline shared by
/// every non-risk item (a `None` facet) and every part-assessed risk alike —
/// assessment is all-or-nothing for ordering. Weights are Low=1 … Critical=4 (A3);
/// the product fits `u8`, no cast. The single derivation site — PHASE-03's
/// `project` reads it here, not a second copy (the PHASE-01 self-clearing dead-code
/// scope removed itself once `project` landed).
pub(crate) fn exposure(facet: Option<&RiskFacet>) -> u8 {
    const fn weight(level: RiskLevel) -> u8 {
        match level {
            RiskLevel::Low => 1,
            RiskLevel::Medium => 2,
            RiskLevel::High => 3,
            RiskLevel::Critical => 4,
        }
    }
    match facet.and_then(|f| f.likelihood.zip(f.impact)) {
        Some((l, i)) => weight(l) * weight(i),
        None => 0,
    }
}

// ---------------------------------------------------------------------------
// Pure: the ordering projection (BacklogItem -> the adapter's OrderInput)
// ---------------------------------------------------------------------------

/// A project-level drop (design §5.6 honest-record, the project half): an authored
/// `needs`/`after` ref that does not even `parse_ref` to a `(kind, id)` — a stale or
/// malformed token that can never become an `ItemId`, so it never reaches the adapter
/// (whose `Dangling` covers the parses-but-not-a-node case). Carries the dependent's
/// `ItemId` and the offending raw ref, so the shell names the drop loudly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AbsentDrop {
    /// The item that authored the bad ref.
    from: ItemId,
    /// The unparseable ref string verbatim.
    reference: String,
}

impl AbsentDrop {
    /// The dependent item that authored the bad ref.
    pub(crate) fn from(&self) -> ItemId {
        self.from
    }

    /// The unparseable ref string.
    pub(crate) fn reference(&self) -> &str {
        &self.reference
    }
}

/// Project the live (non-terminal) corpus into the adapter's inputs (design §5.4,
/// OQ-A "projection in backlog.rs"). PURE — no clock/disk; `items` is the already-read
/// corpus.
///
/// Node set = the **non-terminal** items (`!Status::is_terminal`) across all five
/// kinds (§5.6 — a terminal item cannot participate in a live ordering). For each
/// node, every authored `needs` ref and every `after` edge's `to` is resolved via
/// `parse_ref` to an `ItemId`; a ref that fails to parse is recorded as an
/// [`AbsentDrop`] (never silently dropped) and contributes no edge. Whether a *parsed*
/// `ItemId` is itself a live node is the **adapter's** call (a non-node endpoint
/// surfaces as a `Dangling` override) — `project` never pre-filters edges by node
/// membership, keeping the honest record total.
///
/// **A-distinct (DD4).** The adapter's `by_item`/`by_node` bimap silently corrupts on
/// a duplicate `ItemId` in the input slice. The corpus reads at most one item per
/// `(kind, id)`, but `project` closes the precondition at the boundary: it builds the
/// inputs keyed by `ItemId` (a `BTreeMap`), so the emitted `Vec<OrderInput>` carries
/// strictly distinct `ItemId`s regardless of a malformed corpus.
fn project(items: &[BacklogItem]) -> (Vec<OrderInput>, Vec<AbsentDrop>) {
    let mut inputs: BTreeMap<ItemId, OrderInput> = BTreeMap::new();
    let mut absent: Vec<AbsentDrop> = Vec::new();

    for item in items.iter().filter(|i| !i.status.is_terminal()) {
        let from = ItemId::new(item.kind, item.id);

        let mut resolve = |reference: &str| -> Option<ItemId> {
            if let Ok((kind, id)) = parse_ref(reference) {
                Some(ItemId::new(kind, id))
            } else {
                absent.push(AbsentDrop {
                    from,
                    reference: reference.to_string(),
                });
                None
            }
        };

        let needs: Vec<ItemId> = item
            .relationships
            .needs
            .iter()
            .filter_map(|r| resolve(r))
            .collect();
        let after: Vec<(ItemId, i32)> = item
            .relationships
            .after
            .iter()
            .filter_map(|e| resolve(&e.to).map(|to| (to, e.rank)))
            .collect();

        // A-distinct: the corpus is one row per `(kind, id)`, but key by `ItemId` so a
        // duplicate can never reach the adapter's bimap (DD4).
        inputs.insert(
            from,
            OrderInput::new(
                from,
                item.created.clone(),
                exposure(item.facet.as_ref()),
                needs,
                after,
            ),
        );
    }

    (inputs.into_values().collect(), absent)
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
    let trunk_ids = crate::git::trunk_entity_ids(&root, item_kind.kind().dir)?;
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
        &trunk_ids,
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

/// Resolve a backlog canonical-id prefix (`ISS`/`IMP`/`CHR`/`RSK`/`IDE`) back to its
/// [`ItemKind`] — the inverse of `ItemKind::prefix`, over the single `ItemKind::ALL`
/// source. `pub(crate)` so the SL-046 cross-kind dispatch (`relation_graph`) routes a
/// backlog prefix to [`relation_edges`] without a second prefix↔kind copy.
pub(crate) fn kind_from_prefix(prefix: &str) -> Option<ItemKind> {
    ItemKind::from_prefix(prefix)
}

/// A backlog item's authored outbound relations (SL-046 §5.2/§5.3): `slices` →
/// [`RelationLabel::Slices`], `specs` → [`RelationLabel::Specs`], and `drift` →
/// [`RelationLabel::Drift`]. `drift` is free-text with no `DRIFT` kind in `KINDS`, so
/// it is a TARGET-UNVALIDATED label (ADR-010 Decision 2): emitted so the data is
/// preserved, but its targets never resolve and surface as danglers at the scan
/// (PHASE-03), never edges. NEVER `needs`/`after`/`triggers` (the dep/sequence/mask
/// axes — SL-047). Reads via the existing `read_item` reader (no new TOML parse). An
/// empty axis emits nothing.
pub(crate) fn relation_edges(
    root: &Path,
    item_kind: ItemKind,
    id: u32,
) -> anyhow::Result<Vec<crate::relation::RelationEdge>> {
    use crate::relation::{RelationEdge, RelationLabel};
    let item = read_item(root, item_kind, id)?;
    let rel = &item.relationships;
    let mut edges = Vec::new();
    for (label, refs) in [
        (RelationLabel::Slices, &rel.slices),
        (RelationLabel::Specs, &rel.specs),
        (RelationLabel::Drift, &rel.drift),
    ] {
        edges.extend(refs.iter().map(|t| RelationEdge::new(label, t.clone())));
    }
    Ok(edges)
}

/// A backlog item's `needs`/`after` dependency-sequence edges plus its `promoted`
/// flag, for the cross-kind priority scan (SL-047 §5.2). Targets are the AUTHORED
/// ref strings verbatim (the priority adapter resolves them through its own
/// projection — resolve-only, like `relation_edges`'s targets); each `after` edge
/// carries its per-edge `rank`. `promoted` is `resolution == Resolution::Promoted`
/// — the typed authority (PRD-009 §5.5), a DISTINCT flag from status-terminal and
/// NOT the free-text `origin`. Reads via the existing `read_item` reader (no new
/// TOML parse). Only backlog authors `needs`/`after`; every other kind routes here
/// not at all, so non-backlog nodes carry none (DD-2, dormant until IMP-033).
pub(crate) struct DepSeq {
    pub(crate) needs: Vec<String>,
    pub(crate) after: Vec<(String, i32)>,
    pub(crate) promoted: bool,
}

/// Read one backlog item's [`DepSeq`] (the SL-047 priority adapter's dep/seq +
/// promoted seam).
pub(crate) fn dep_seq_for(root: &Path, item_kind: ItemKind, id: u32) -> anyhow::Result<DepSeq> {
    let item = read_item(root, item_kind, id)?;
    let after = item
        .relationships
        .after
        .iter()
        .map(|e| (e.to.clone(), e.rank))
        .collect();
    Ok(DepSeq {
        needs: item.relationships.needs.clone(),
        after,
        promoted: item.resolution == Some(Resolution::Promoted),
    })
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

/// Project a `BacklogItem` to its filterable fields (design §5.2). `canonical` is
/// the prefixed id (`ISS-007`) — the regex domain; `status` is the kebab string the
/// hide-set / `--status` filter match on; `tags` are the item's own.
fn key(i: &BacklogItem) -> listing::FilterFields {
    listing::FilterFields {
        canonical: i.kind.canonical_id(i.id),
        slug: i.slug.clone(),
        title: i.title.clone(),
        status: i.status.as_str().to_string(),
        tags: i.tags.clone(),
    }
}

/// Re-export of the spine's status validator, scoped to backlog so callers read
/// intent locally. Guards `--status` against [`BACKLOG_STATUSES`] (READ input only).
fn validate_statuses(given: &[String], known: &[&str]) -> anyhow::Result<()> {
    listing::validate_statuses(given, known)
}

/// One backlog item projected to its faithful JSON row (design §5.3 — backlog owns
/// its serde shape). `id` is the prefixed canonical id; `kind`/`status`/`resolution`
/// are the kebab strings (resolution `null` when absent). The risk facet and
/// relationships are list-irrelevant (they ride `show`), so the list row stays flat.
#[derive(Debug, Serialize)]
struct BacklogRow {
    id: String,
    kind: &'static str,
    status: &'static str,
    resolution: Option<&'static str>,
    slug: String,
    title: String,
}

/// The table columns `backlog list` can show (`--columns` tokens over
/// `R = BacklogItem` — extractors are non-capturing, SL-037 D5; the prefixed id
/// is materialised in the cell from the item's own kind+id). Declaration order is
/// what the unknown-column error lists.
const BL_COLUMNS: [listing::Column<BacklogItem>; 5] = [
    listing::Column {
        name: "id",
        header: "id",
        cell: |i| i.kind.canonical_id(i.id),
    },
    listing::Column {
        name: "kind",
        header: "kind",
        cell: |i| i.kind.as_str().to_string(),
    },
    listing::Column {
        name: "status",
        header: "status",
        cell: |i| i.status.as_str().to_string(),
    },
    listing::Column {
        name: "slug",
        header: "slug",
        cell: |i| i.slug.clone(),
    },
    listing::Column {
        name: "title",
        header: "title",
        cell: |i| i.title.clone(),
    },
];

/// The default visible set — slug-free (SL-037 D4); `--columns …,slug` reveals it.
const BL_DEFAULT: &[&str] = &["id", "kind", "status", "title"];

/// The `backlog list` output as a string — the compute half of `run_list`, on the
/// shared spine. `validate_statuses` guards `--status` (A-2); `listing::build`
/// resolves the filter + format; `retain` applies the shared substr/regex/status/
/// tag axes + the terminal hide-set ([`is_hidden`], reusing `Status::is_terminal`).
/// The kind-specific `--kind` filter (not a shared axis) is applied here. backlog
/// sorts by `(kind.ordinal, id)` — its variant ordering, never in `retain` (§5.3).
fn list_rows(root: &Path, kind: Option<ItemKind>, mut args: ListArgs) -> anyhow::Result<String> {
    validate_statuses(&args.status, BACKLOG_STATUSES)?;
    let columns = args.columns.take();
    let (filter, format) = listing::build(args)?;
    let mut items = listing::retain(read_all(root)?, &filter, is_hidden, key);
    items.retain(|i| kind.is_none_or(|k| i.kind == k));
    items.sort_by_key(|i| (i.kind.ordinal(), i.id));
    match format {
        Format::Table => {
            let sel = listing::select_columns(&BL_COLUMNS, BL_DEFAULT, columns.as_deref())?;
            Ok(listing::render_columns(&items, &sel))
        }
        Format::Json => listing::json_envelope("backlog", &json_rows(&items)),
    }
}

/// Faithful JSON rows (D7) — the prefixed id plus the flat list fields.
fn json_rows(items: &[BacklogItem]) -> Vec<BacklogRow> {
    items
        .iter()
        .map(|i| BacklogRow {
            id: i.kind.canonical_id(i.id),
            kind: i.kind.as_str(),
            status: i.status.as_str(),
            resolution: i.resolution.map(Resolution::as_str),
            slug: i.slug.clone(),
            title: i.title.clone(),
        })
        .collect()
}

/// `doctrine backlog list [--kind K] [-f SUBSTR] [-r RE] [-i] [-s S,…] [-t T] [-a]
/// [--format F | --json] [<SUBSTR>]` — the survey verb (PRD-009 REQ-050), on the
/// shared spine. Thin shell (§5.4): find the root, lower the args, print the rows
/// verbatim (`list_rows` carries `render_table`'s own trailing newline). `--kind`
/// is the one kind-specific axis; the positional `[SUBSTR]` is folded into the
/// shared substr by the caller (deprecated alias — `--filter` wins, A-7).
pub(crate) fn run_list(
    path: Option<PathBuf>,
    kind: Option<ItemKind>,
    args: ListArgs,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    write!(io::stdout(), "{}", list_rows(&root, kind, args)?)?;
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
    if !rel.slices.is_empty()
        || !rel.specs.is_empty()
        || !rel.drift.is_empty()
        || !rel.needs.is_empty()
        || !rel.after.is_empty()
        || !rel.triggers.is_empty()
    {
        parts.push("\nrelationships:\n".to_string());
        // the four string axes share the one loop; `after`/`triggers` carry payload
        // (per-edge rank, glob+note) and render bespoke below, in §5.2 key order.
        for (label, refs) in [
            ("slices", &rel.slices),
            ("specs", &rel.specs),
            ("drift", &rel.drift),
            ("needs", &rel.needs),
        ] {
            if !refs.is_empty() {
                parts.push(format!("  {label}: {}\n", refs.join(", ")));
            }
        }
        if !rel.after.is_empty() {
            let rendered = rel
                .after
                .iter()
                .map(|e| {
                    if e.rank == 0 {
                        e.to.clone()
                    } else {
                        format!("{} (rank {})", e.to, e.rank)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            parts.push(format!("  after: {rendered}\n"));
        }
        if !rel.triggers.is_empty() {
            let rendered = rel
                .triggers
                .iter()
                .map(|t| {
                    let globs = t.globs.join(", ");
                    if t.note.is_empty() {
                        format!("[{globs}]")
                    } else {
                        format!("[{globs}] {}", t.note)
                    }
                })
                .collect::<Vec<_>>()
                .join("; ");
            parts.push(format!("  triggers: {rendered}\n"));
        }
    }

    parts.concat()
}

/// `doctrine backlog show <ID>` — the inspect verb (PRD-009 REQ-051, §5.4). Thin
/// shell: find the root, `parse_ref` the id to its kind (prefix auto-detect), read
/// THAT item's single toml, render it to stdout. READ-ONLY — no mutation, no
/// cross-corpus scan (only the one item's file is opened); the render is pure over
/// the item's own state.
pub(crate) fn run_show(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (item_kind, id) = parse_ref(reference)?;
    let item = read_item(&root, item_kind, id)?;
    let out = match format {
        Format::Table => format_show(&item),
        Format::Json => show_json(&item)?,
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
}

/// Render the `Json` show: the item's faithful state under the shared `{kind, …}`
/// envelope (the `adr::show_json` precedent). The validated `BacklogItem`'s fields
/// are private and its closed enums render via `as_str`, so the JSON is projected by
/// hand here (not a derive): the flat identity, the optional resolution, the risk
/// `[facet]` (risk only), and the outbound relationships — the same data the table
/// reassembles, structured. Pure over the item's own state (no cross-corpus scan).
fn show_json(item: &BacklogItem) -> anyhow::Result<String> {
    let facet = item.facet.as_ref().map(|f| {
        serde_json::json!({
            "likelihood": f.likelihood.map(RiskLevel::as_str),
            "impact": f.impact.map(RiskLevel::as_str),
            "origin": f.origin,
            "controls": f.controls,
        })
    });
    let rel = &item.relationships;
    let value = serde_json::json!({
        "kind": "backlog",
        "backlog": {
            "id": item.kind.canonical_id(item.id),
            "kind": item.kind.as_str(),
            "slug": item.slug,
            "title": item.title,
            "status": item.status.as_str(),
            "resolution": item.resolution.map(Resolution::as_str),
            "created": item.created,
            "updated": item.updated,
            "tags": item.tags,
            "facet": facet,
            "relationships": {
                "slices": rel.slices,
                "specs": rel.specs,
                "drift": rel.drift,
                "needs": rel.needs,
                "after": rel.after,
                "triggers": rel.triggers,
            },
        },
    });
    serde_json::to_string_pretty(&value).context("failed to serialize backlog show JSON")
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

/// One outbound item→item relationship-axis append (PHASE-03 set verbs). `Needs`
/// carries the prereq refs (a string array); `After` carries one soft-sequence edge
/// `{ to, rank }` (the array-of-inline-tables axis, one `to` per invocation — OQ-C).
/// The refs are pre-validated by the shell before this is called.
enum RelEdit<'a> {
    /// Append these prereq refs to `[relationships].needs`.
    Needs(&'a [String]),
    /// Append one `{ to, rank }` edge to `[relationships].after`.
    After { to: &'a str, rank: i32 },
}

/// Edit-preserving append into one `[relationships]` array — the `set_backlog_status`
/// `toml_edit` precedent (mem.pattern.entity.edit-preserving-status-transition): mutate
/// the file in place so comments, inert tables, and unknown keys survive verbatim (the
/// file is never reserialised). Navigates `[relationships]` → the target array, pushes
/// each new entry, and writes once.
///
/// **F-1 refuse** (the `set_backlog_status` corruption hazard): if `[relationships]`
/// or the seeded target array is absent, this is a malformed (hand-edited) item — a
/// tail `insert` would land the array inside a trailing subtable. Refuse instead,
/// touching nothing. **Idempotent**: an entry already present (a `needs` ref already
/// listed, or an identical `{ to, rank }` edge) is not duplicated; if every entry is
/// already present the file is left byte-identical (no write, mtime holds).
fn append_relationship(
    root: &Path,
    item_kind: ItemKind,
    id: u32,
    edit: &RelEdit<'_>,
) -> anyhow::Result<()> {
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

    // F-1: `[relationships]` and the target axis array are scaffold-seeded; their
    // absence means a malformed item (a tail insert would corrupt a trailing subtable).
    let axis = match edit {
        RelEdit::Needs(_) => "needs",
        RelEdit::After { .. } => "after",
    };
    let array = doc
        .get_mut("relationships")
        .and_then(toml_edit::Item::as_table_mut)
        .and_then(|t| t.get_mut(axis))
        .and_then(toml_edit::Item::as_array_mut)
        .with_context(|| {
            format!(
                "malformed backlog item {name}: missing seeded `[relationships].{axis}` (regenerate via `backlog new`)"
            )
        })?;

    let mut changed = false;
    match edit {
        RelEdit::Needs(refs) => {
            for r in *refs {
                // idempotent: skip a ref already in the array.
                if array.iter().any(|v| v.as_str() == Some(r.as_str())) {
                    continue;
                }
                array.push(r.as_str());
                changed = true;
            }
        }
        RelEdit::After { to, rank } => {
            // idempotent: skip an identical `{ to, rank }` edge.
            let present = array.iter().any(|v| {
                v.as_inline_table().is_some_and(|t| {
                    t.get("to").and_then(toml_edit::Value::as_str) == Some(to)
                        && t.get("rank").and_then(toml_edit::Value::as_integer)
                            == Some(i64::from(*rank))
                })
            });
            if !present {
                let mut edge = toml_edit::InlineTable::new();
                edge.insert("to", (*to).into());
                edge.insert("rank", i64::from(*rank).into());
                array.push(edge);
                changed = true;
            }
        }
    }

    if !changed {
        return Ok(()); // every entry already present — write nothing (mtime holds).
    }
    std::fs::write(&path, doc.to_string())
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
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
// PHASE-03 set verbs: `backlog needs` / `backlog after` (the thin impure shells)
// ---------------------------------------------------------------------------

/// Validate that a backlog ref names an existing item — `parse_ref` then a read.
/// A bad prefix / non-numeric tail (`parse_ref` Err) or a missing file is a HARD
/// user error (`bail!` via the `?`), never a soft drop: a set verb must reject a
/// stale ref at author time (design §5.6 — the absent case is rejected here, so
/// `order` only ever defends against later staleness). Returns the resolved id.
fn require_item(root: &Path, reference: &str) -> anyhow::Result<(ItemKind, u32)> {
    let (kind, id) = parse_ref(reference)?;
    read_item(root, kind, id)?;
    Ok((kind, id))
}

/// Render a diagnosed `needs` cycle as a stable, sorted member list (`A, B, C`) for
/// the refuse/error message — `ItemId` canonical refs only (no `NodeId` internals, R1).
fn name_cycle(members: &std::collections::BTreeSet<ItemId>) -> String {
    members
        .iter()
        .map(|id| id.render())
        .collect::<Vec<_>>()
        .join(", ")
}

/// The `needs` set verb's pure refuse oracle (A-setcycle / DD2): would adding
/// `new_needs` to `ITEM` close a `needs` cycle? Injects the proposed edges into a
/// CLONE of the corpus, projects, builds, and asks the adapter's `dep_cycles` (the
/// single cycle oracle — no parallel impl). Returns the offending cycles (empty ⇒
/// safe to append). Pure over the read corpus + the proposed edges.
fn needs_would_cycle(
    items: &[BacklogItem],
    target: (ItemKind, u32),
    new_needs: &[String],
) -> anyhow::Result<Vec<std::collections::BTreeSet<ItemId>>> {
    let mut corpus: Vec<BacklogItem> = items.to_vec();
    if let Some(item) = corpus
        .iter_mut()
        .find(|i| i.kind == target.0 && i.id == target.1)
    {
        item.relationships.needs.extend_from_slice(new_needs);
    }
    let (inputs, _) = project(&corpus);
    Ok(BacklogOrder::build(&inputs)?.dep_cycles())
}

/// `doctrine backlog needs <ITEM> <PREREQ>…` — append hard prerequisites (PRD-009,
/// design §5.5). Thin shell: find the root, validate ITEM + every PREREQ exists
/// (a bad ref is a hard user error), then **build the dep graph including the
/// proposed edges and refuse on a closing cycle** (naming members; nothing written
/// — validate-then-build-then-write). Else append edit-in-place + confirm.
pub(crate) fn run_needs(
    path: Option<PathBuf>,
    reference: &str,
    prereqs: &[String],
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let target = require_item(&root, reference)?;
    for prereq in prereqs {
        require_item(&root, prereq)?;
    }

    // refuse a closing cycle BEFORE any write (the adapter is the single oracle).
    let items = read_all(&root)?;
    let cycles = needs_would_cycle(&items, target, prereqs)?;
    if let Some(cycle) = cycles.first() {
        anyhow::bail!(
            "`backlog needs` would close a dependency cycle: {} (nothing written)",
            name_cycle(cycle)
        );
    }

    append_relationship(&root, target.0, target.1, &RelEdit::Needs(prereqs))?;
    writeln!(
        io::stdout(),
        "{} needs {}",
        target.0.canonical_id(target.1),
        prereqs.join(", ")
    )?;
    Ok(())
}

/// `doctrine backlog after <ITEM> <TO> [--rank N]` — append ONE soft-sequence edge
/// (PRD-009, design §5.5). Thin shell: validate ITEM + the single TO exists, then
/// append `{ to, rank }` (rank optional, default 0). **Never** rejects a cycle — a
/// soft `after` cycle is surfaced (and an edge evicted) at `order` time (VT-6).
pub(crate) fn run_after(
    path: Option<PathBuf>,
    reference: &str,
    to: &str,
    rank: i32,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let target = require_item(&root, reference)?;
    require_item(&root, to)?;

    append_relationship(&root, target.0, target.1, &RelEdit::After { to, rank })?;
    let suffix = if rank == 0 {
        String::new()
    } else {
        format!(" (rank {rank})")
    };
    writeln!(
        io::stdout(),
        "{} after {to}{suffix}",
        target.0.canonical_id(target.1),
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// PHASE-03 read verb: `backlog order` (composed order + the honest-record block)
// ---------------------------------------------------------------------------

/// Name a `Dangling` endpoint loudly (A-classify / DD1 / design §5.6 E1): the
/// status/resolution vocabulary is supplied SHELL-side from the corpus, keeping the
/// adapter id-only (the R-C kill). `endpoint` is the adapter's `Dangling.from()` — the
/// missing endpoint. Looked up in `corpus`:
/// - **present but terminal** (`resolved`/`closed`) ⇒ `"<status>/<resolution>"`
///   (e.g. `closed/wont-do`) — the author judges staleness from the named resolution,
///   never a silent satisfied-claim;
/// - **not present** (a stale ref to a never-existed / since-deleted id) ⇒ `"absent"`.
///
/// (A present-but-NON-terminal endpoint cannot be `Dangling` — it would be a live
/// node — so that arm is unreachable; rendered defensively as `"absent"`.)
fn classify_dangling(corpus: &BTreeMap<ItemId, &BacklogItem>, endpoint: ItemId) -> String {
    match corpus.get(&endpoint) {
        Some(item) if item.status.is_terminal() => {
            let resolution = item.resolution.map_or("?", Resolution::as_str);
            format!("{}/{resolution}", item.status.as_str())
        }
        _ => "absent".to_string(),
    }
}

/// Render the honest-record `overrides:` block (design §5.6, R1): one terse line per
/// dropped edge — `<from> → <to> dropped (<why>)` — `ItemId` refs + reason words only
/// (no NodeId/ordering internals leak). Covers BOTH the project-level [`AbsentDrop`]s
/// (unparseable refs that never reached the adapter) and the adapter's `overrides()`
/// (soft-cycle evictions, contradictions, and the parses-but-not-a-node `Dangling`s,
/// each named with status+resolution). Empty when nothing was dropped (no block).
fn render_overrides(
    corpus: &BTreeMap<ItemId, &BacklogItem>,
    absent: &[AbsentDrop],
    overrides: &[Override],
) -> String {
    let mut lines: Vec<String> = Vec::new();

    // project-level drops: an unparseable ref never became an ItemId.
    for drop in absent {
        lines.push(format!(
            "  {} → {} dropped (dangling: {} absent)\n",
            drop.from().render(),
            drop.reference(),
            drop.reference(),
        ));
    }

    // adapter-level drops.
    for ov in overrides {
        let line = match ov.reason() {
            OverrideReason::SoftCycleEvicted => format!(
                "  {} → {} dropped (soft cycle)\n",
                ov.from().render(),
                ov.to().render()
            ),
            OverrideReason::Contradicted => format!(
                "  {} → {} dropped (contradicts a need)\n",
                ov.from().render(),
                ov.to().render()
            ),
            OverrideReason::Dangling => format!(
                "  {} → {} dropped (dangling: {} {})\n",
                ov.from().render(),
                ov.to().render(),
                ov.from().render(),
                classify_dangling(corpus, ov.from()),
            ),
        };
        lines.push(line);
    }

    if lines.is_empty() {
        return String::new();
    }
    let mut out = vec!["\noverrides:\n".to_string()];
    out.extend(lines);
    out.concat()
}

/// The `backlog order` output as a string — the compute half (PURE over the read
/// corpus). Projects the non-terminal node set, builds the adapter, and — UNLESS a
/// `needs` cycle is present — renders the composed order (the `list` column model,
/// rows in cordage `ordered()` order) followed by the honest-record `overrides:`
/// block. A `needs` **dep cycle is a hard error** (design §5.5 / EX-3): a returned
/// `anyhow::Error` naming the members → `main`'s error path (stderr, non-zero exit),
/// NO misleading order printed.
fn order_rows(root: &Path) -> anyhow::Result<String> {
    let items = read_all(root)?;
    let (inputs, absent) = project(&items);
    let order = BacklogOrder::build(&inputs)?;

    if let Some(cycle) = order.dep_cycles().first() {
        anyhow::bail!(
            "`backlog order` cannot compose: a `needs` dependency cycle — {} (resolve it, then re-run)",
            name_cycle(cycle)
        );
    }

    // a fast ItemId → item index for the order render and the dangling classifier.
    let corpus: BTreeMap<ItemId, &BacklogItem> = items
        .iter()
        .map(|i| (ItemId::new(i.kind, i.id), i))
        .collect();

    // the composed order — rows in cordage order (NOT (kind,id)); the `list` columns.
    let ordered: Vec<BacklogItem> = order
        .ordered()
        .iter()
        .filter_map(|id| corpus.get(id).map(|i| (*i).clone()))
        .collect();
    let sel = listing::select_columns(&BL_COLUMNS, BL_DEFAULT, None)?;
    let table = listing::render_columns(&ordered, &sel);

    let overrides = render_overrides(&corpus, &absent, &order.overrides());
    Ok(format!("{table}{overrides}"))
}

/// `doctrine backlog order` — the composed-order view (PRD-009, design §5.5). Thin
/// shell: find the root, compute, print. READ-only. A `needs` dep cycle returns an
/// error (the shell never prints a misleading order) — `main` surfaces it on stderr
/// with a non-zero exit.
pub(crate) fn run_order(path: Option<PathBuf>) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    write!(io::stdout(), "{}", order_rows(&root)?)?;
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
            &[],
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
        // the three PRD-009 item→item axes default to `[]` on a virgin item (VT-1).
        assert!(item.relationships.needs.is_empty());
        assert!(item.relationships.after.is_empty());
        assert!(item.relationships.triggers.is_empty());
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

    /// A backlog-NNN.toml fixture spec — the single source of the test fixture
    /// literal. `'a` (not `'static`) because `write_related` passes borrowed
    /// `slices`/`specs`. `facet`/`rels` absent → that block is omitted.
    struct Fixture<'a> {
        kind: ItemKind,
        id: u32,
        slug: &'a str,
        title: &'a str,
        status: &'a str,
        resolution: &'a str,
        tags: &'a [&'a str],
        facet: Option<FacetLit<'a>>,
        rels: Option<RelLit<'a>>,
    }

    struct FacetLit<'a> {
        likelihood: &'a str,
        impact: &'a str,
        origin: &'a str,
        controls: &'a [&'a str],
    }

    struct RelLit<'a> {
        slices: &'a [&'a str],
        specs: &'a [&'a str],
        needs: &'a [&'a str],
        after: &'a [AfterLit<'a>],
        triggers: &'a [TriggerLit<'a>],
    }

    struct AfterLit<'a> {
        to: &'a str,
        rank: i32,
    }

    struct TriggerLit<'a> {
        globs: &'a [&'a str],
        note: &'a str,
    }

    /// The sole list-literal quoting: `[] → ""`, `["a","b"] → "\"a\", \"b\""`.
    fn toml_list(xs: &[&str]) -> String {
        xs.iter()
            .map(|x| format!("\"{x}\""))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// `after` array-of-inline-tables literal: each edge `{ to = "X", rank = N }`.
    fn toml_after(xs: &[AfterLit<'_>]) -> String {
        xs.iter()
            .map(|e| format!("{{ to = \"{}\", rank = {} }}", e.to, e.rank))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// `triggers` array-of-inline-tables literal: each `{ globs = [...], note = "" }`.
    fn toml_triggers(xs: &[TriggerLit<'_>]) -> String {
        xs.iter()
            .map(|t| {
                format!(
                    "{{ globs = [{}], note = \"{}\" }}",
                    toml_list(t.globs),
                    t.note
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// The sole fixture TOML literal: core head + optional `[facet]` + optional
    /// `[relationships]`. Segments concatenate (each `""` when absent) rather than
    /// `push_str(&format!(..))`, honouring the repo string-build convention.
    fn render_fixture_toml(f: &Fixture<'_>) -> String {
        let head = format!(
            "id = {}\nslug = \"{}\"\ntitle = \"{}\"\nkind = \"{}\"\n\
             status = \"{}\"\nresolution = \"{}\"\n\
             created = \"2026-06-08\"\nupdated = \"2026-06-08\"\ntags = [{}]\n",
            f.id,
            f.slug,
            f.title,
            f.kind.as_str(),
            f.status,
            f.resolution,
            toml_list(f.tags),
        );
        let facet = f.facet.as_ref().map_or_else(String::new, |x| {
            format!(
                "\n[facet]\nlikelihood = \"{}\"\nimpact = \"{}\"\norigin = \"{}\"\ncontrols = [{}]\n",
                x.likelihood,
                x.impact,
                x.origin,
                toml_list(x.controls),
            )
        });
        let rels = f.rels.as_ref().map_or_else(String::new, |x| {
            format!(
                "\n[relationships]\nslices = [{}]\nspecs = [{}]\ndrift = []\n\
                 needs = [{}]\nafter = [{}]\ntriggers = [{}]\n",
                toml_list(x.slices),
                toml_list(x.specs),
                toml_list(x.needs),
                toml_after(x.after),
                toml_triggers(x.triggers),
            )
        });
        format!("{head}{facet}{rels}")
    }

    /// The sole path/dir/write: render the fixture and lay it under its kind tree.
    fn write_fixture(root: &Path, f: Fixture<'_>) {
        let name = format!("{:03}", f.id);
        let dir = root.join(f.kind.kind().dir).join(&name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(format!("backlog-{name}.toml")),
            render_fixture_toml(&f),
        )
        .unwrap();
    }

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
        write_fixture(
            root,
            Fixture {
                kind,
                id,
                slug,
                title,
                status,
                resolution,
                tags,
                facet: None,
                rels: None,
            },
        );
    }

    /// The first column (canonical id) of each rendered row, in render order.
    /// Skips the §5.5 header line; an empty `""` (suppressed header) → no ids.
    fn ids(out: &str) -> Vec<String> {
        out.lines()
            .skip(1)
            .map(|l| l.split_whitespace().next().unwrap().to_string())
            .collect()
    }

    /// A no-constraint `ListArgs` (the default `backlog list`).
    fn list_args() -> ListArgs {
        ListArgs::default()
    }

    // --- §5.5: the uniform table header (extends to backlog) ---

    #[test]
    fn backlog_list_emits_a_header_then_prefixed_ids() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_item(root, ItemKind::Issue, 1, "open", "", "a", "Alpha", &[]);

        let out = list_rows(root, None, list_args()).unwrap();
        let lines: Vec<&str> = out.lines().collect();
        // §5.5: rows present → a header row naming the columns, then the data.
        assert!(lines[0].starts_with("id"), "header row: {:?}", lines[0]);
        assert!(
            lines[0].contains("kind") && lines[0].contains("status"),
            "header names columns: {:?}",
            lines[0]
        );
        assert!(lines[1].starts_with("ISS-001"), "first data row prefixed");
    }

    #[test]
    fn backlog_list_empty_suppresses_the_header() {
        let dir = tempfile::tempdir().unwrap();
        // no items written → "" (header suppressed, §5.5 virgin-repo contract).
        assert_eq!(list_rows(dir.path(), None, list_args()).unwrap(), "");
    }

    // --- SL-037: the column model (default omits slug, --columns reveals) ---

    /// A `ListArgs` requesting an explicit column set (SL-037 `--columns`).
    fn columns_args(cols: &[&str]) -> ListArgs {
        ListArgs {
            columns: Some(cols.iter().map(|s| (*s).to_string()).collect()),
            ..Default::default()
        }
    }

    #[test]
    fn backlog_list_default_omits_slug() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_item(
            root,
            ItemKind::Issue,
            1,
            "open",
            "",
            "token-expiry",
            "Alpha",
            &[],
        );

        let out = list_rows(root, None, list_args()).unwrap();
        let header = out.lines().next().unwrap();
        // SL-037 D4: default visible set is [id, kind, status, title] — slug hidden.
        assert!(
            !header.contains("slug"),
            "default header omits slug: {header:?}"
        );
        assert!(
            !out.contains("token-expiry"),
            "slug value hidden by default: {out}"
        );
        assert!(
            header.contains("kind") && header.contains("status") && header.contains("title"),
            "default keeps id/kind/status/title: {header:?}"
        );
    }

    #[test]
    fn backlog_list_columns_reveals_and_orders_slug() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_item(
            root,
            ItemKind::Issue,
            1,
            "open",
            "",
            "token-expiry",
            "Alpha",
            &[],
        );

        // Reorder slug ahead of title and reveal it.
        let out = list_rows(root, None, columns_args(&["id", "slug", "title"])).unwrap();
        let header = out.lines().next().unwrap();
        assert_eq!(
            header.split_whitespace().collect::<Vec<_>>(),
            vec!["id", "slug", "title"]
        );
        assert!(
            out.contains("token-expiry"),
            "slug revealed by --columns: {out}"
        );
    }

    #[test]
    fn backlog_list_columns_unknown_errors_with_available_set() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_item(root, ItemKind::Issue, 1, "open", "", "a", "Alpha", &[]);

        let err = list_rows(root, None, columns_args(&["bogus"]))
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();
        assert!(
            err.contains("unknown column `bogus`"),
            "uniform error: {err}"
        );
        assert!(
            err.contains("id") && err.contains("slug"),
            "lists available set: {err}"
        );
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

        let out = list_rows(root, None, list_args()).unwrap();
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
            None,
            ListArgs {
                all: true,
                ..ListArgs::default()
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
            None,
            ListArgs {
                status: vec!["resolved".into()],
                ..ListArgs::default()
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
            Some(ItemKind::Issue),
            ListArgs {
                tags: vec!["security".to_string()],
                substr: Some("auth".to_string()),
                ..ListArgs::default()
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

        let out = list_rows(root, None, list_args()).unwrap();
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

        let out = list_rows(root, None, list_args()).unwrap();
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
        let out = list_rows(root, None, list_args()).unwrap();
        assert_eq!(
            out, "",
            "a virgin repo prints an empty table, never an error"
        );
    }

    // --- SL-025 EX-3: the shared spine — regexp / case / hide-set / json ---

    #[test]
    fn backlog_list_regexp_matches_canonical_id_case_insensitive() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_item(root, ItemKind::Issue, 1, "open", "", "a", "Alpha", &[]);
        write_item(root, ItemKind::Risk, 1, "open", "", "r", "Risky", &[]);

        // --regexp over the canonical-id domain, made case-insensitive (-i): the
        // lower-case `iss` matches `ISS-001` only.
        let out = list_rows(
            root,
            None,
            ListArgs {
                regexp: Some("iss-".into()),
                case_insensitive: true,
                ..ListArgs::default()
            },
        )
        .unwrap();
        assert_eq!(
            ids(&out),
            vec!["ISS-001"],
            "regexp on the prefixed id: {out}"
        );
    }

    #[test]
    fn backlog_list_json_is_one_envelope_with_prefixed_ids() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_item(root, ItemKind::Issue, 1, "open", "", "a", "Alpha", &["x"]);
        write_item(
            root,
            ItemKind::Issue,
            2,
            "resolved",
            "fixed",
            "b",
            "Bravo",
            &[],
        );

        let json = list_rows(
            root,
            None,
            ListArgs {
                json: true,
                ..ListArgs::default()
            },
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["kind"], "backlog");
        let rows = v["rows"].as_array().expect("rows is an array");
        // the resolved item is hidden by default → one row; the open one survives.
        assert_eq!(rows.len(), 1, "hide-set applies under json too: {json}");
        let row = rows.first().expect("the open row");
        assert_eq!(row["id"], "ISS-001");
        assert_eq!(row["kind"], "issue");
        assert_eq!(row["status"], "open");
        assert_eq!(row["resolution"], serde_json::Value::Null);
    }

    #[test]
    fn backlog_list_rejects_an_unknown_status_with_the_uniform_error() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_item(root, ItemKind::Issue, 1, "open", "", "a", "Alpha", &[]);
        let err = list_rows(
            root,
            None,
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

    /// Drift canary: the `BACKLOG_STATUSES` known-set must stay in lockstep with
    /// the `Status` enum's kebab serde (A-2).
    #[test]
    fn backlog_statuses_matches_the_variants() {
        let from_variants: Vec<&str> = [
            Status::Open,
            Status::Triaged,
            Status::Started,
            Status::Resolved,
            Status::Closed,
        ]
        .iter()
        .map(|s| s.as_str())
        .collect();
        assert_eq!(from_variants, BACKLOG_STATUSES.to_vec());
    }

    #[test]
    fn is_hidden_reuses_status_is_terminal() {
        // the hide-set IS Status::is_terminal over the stringly token (no new set).
        assert!(is_hidden("resolved"));
        assert!(is_hidden("closed"));
        assert!(!is_hidden("open"));
        assert!(!is_hidden("triaged"));
        assert!(!is_hidden("started"));
        // an out-of-vocab token is not hidden (retain is stringly; serde can't store it).
        assert!(!is_hidden("bogus"));
    }

    // --- SL-025 EX-4 / VT-3: backlog show --json ---

    #[test]
    fn backlog_show_json_is_faithful_item_state() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // a fully-assessed risk: facet + relationships + a terminal resolution.
        write_fixture(
            root,
            Fixture {
                kind: ItemKind::Risk,
                id: 1,
                slug: "leak",
                title: "Token leak",
                status: "resolved",
                resolution: "mitigated",
                tags: &["security"],
                facet: Some(FacetLit {
                    likelihood: "high",
                    impact: "critical",
                    origin: "audit",
                    controls: &["rotate"],
                }),
                rels: Some(RelLit {
                    slices: &["SL-020"],
                    specs: &[],
                    needs: &[],
                    after: &[],
                    triggers: &[],
                }),
            },
        );

        let item = read_item(root, ItemKind::Risk, 1).unwrap();
        let json = show_json(&item).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["kind"], "backlog");
        let b = &v["backlog"];
        assert_eq!(b["id"], "RSK-001");
        assert_eq!(b["status"], "resolved");
        assert_eq!(b["resolution"], "mitigated");
        assert_eq!(b["tags"][0], "security");
        assert_eq!(b["facet"]["likelihood"], "high");
        assert_eq!(b["facet"]["impact"], "critical");
        assert_eq!(b["relationships"]["slices"][0], "SL-020");
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

    // --- VT-1: the three PRD-009 item→item axes (needs / after / triggers) ---

    #[test]
    fn after_edge_round_trips_with_optional_rank() {
        // a ranked edge keeps its `rank`; a bare `{ to }` defaults to rank 0.
        let rel: Relationships =
            toml::from_str("after = [{ to = \"ISS-002\", rank = 2 }, { to = \"ISS-003\" }]\n")
                .unwrap();
        assert_eq!(
            rel.after,
            vec![
                AfterEdge {
                    to: "ISS-002".to_string(),
                    rank: 2,
                },
                AfterEdge {
                    to: "ISS-003".to_string(),
                    rank: 0,
                },
            ]
        );
    }

    #[test]
    fn trigger_round_trips_with_optional_note() {
        // a noted trigger keeps its `note`; a globs-only `{ globs }` defaults to "".
        let rel: Relationships = toml::from_str(
            "triggers = [{ globs = [\"src/x/**\"], note = \"watch x\" }, \
             { globs = [\"src/y/**\"] }]\n",
        )
        .unwrap();
        assert_eq!(
            rel.triggers,
            vec![
                Trigger {
                    globs: vec!["src/x/**".to_string()],
                    note: "watch x".to_string(),
                },
                Trigger {
                    globs: vec!["src/y/**".to_string()],
                    note: String::new(),
                },
            ]
        );
    }

    #[test]
    fn backlog_show_renders_all_three_item_axes() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // a populated item carrying all three PRD-009 outbound axes.
        write_fixture(
            root,
            Fixture {
                kind: ItemKind::Issue,
                id: 1,
                slug: "s",
                title: "T",
                status: "open",
                resolution: "",
                tags: &[],
                facet: None,
                rels: Some(RelLit {
                    slices: &[],
                    specs: &[],
                    needs: &["ISS-002"],
                    after: &[AfterLit {
                        to: "ISS-003",
                        rank: 2,
                    }],
                    triggers: &[TriggerLit {
                        globs: &["src/x/**"],
                        note: "watch x",
                    }],
                }),
            },
        );
        let item = read_item(root, ItemKind::Issue, 1).unwrap();

        // table seam: each axis renders, in fixed §5.2 order (needs/after/triggers);
        // a non-zero `after` rank annotates, the trigger note trails its globs.
        let out = format_show(&item);
        assert!(out.contains("needs: ISS-002"), "hard prereq axis: {out}");
        assert!(
            out.contains("after: ISS-003 (rank 2)"),
            "soft seq axis with rank: {out}"
        );
        assert!(
            out.contains("triggers: [src/x/**] watch x"),
            "triggers rider: {out}"
        );

        // JSON seam: needs is a string array; after/triggers are arrays of tables.
        let json = show_json(&item).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let rel = &v["backlog"]["relationships"];
        assert_eq!(rel["needs"][0], "ISS-002");
        assert_eq!(rel["after"][0]["to"], "ISS-003");
        assert_eq!(rel["after"][0]["rank"], 2);
        assert_eq!(rel["triggers"][0]["globs"][0], "src/x/**");
        assert_eq!(rel["triggers"][0]["note"], "watch x");
    }

    /// Overwrite a reserved risk item with an assessed `[facet]` — exercises the
    /// real read+validate path for a populated facet without the (PHASE-05) `edit`.
    fn write_assessed_risk(root: &Path, id: u32) {
        write_fixture(
            root,
            Fixture {
                kind: ItemKind::Risk,
                id,
                slug: "token-expiry",
                title: "Token expiry",
                status: "open",
                resolution: "",
                tags: &[],
                facet: Some(FacetLit {
                    likelihood: "high",
                    impact: "critical",
                    origin: "audit",
                    controls: &["rate-limit"],
                }),
                rels: Some(RelLit {
                    slices: &[],
                    specs: &[],
                    needs: &[],
                    after: &[],
                    triggers: &[],
                }),
            },
        );
    }

    /// Write an item carrying seeded OUTBOUND `slices`/`specs` relations directly.
    fn write_related(root: &Path, kind: ItemKind, id: u32, slices: &[&str], specs: &[&str]) {
        write_fixture(
            root,
            Fixture {
                kind,
                id,
                slug: "s",
                title: "T",
                status: "open",
                resolution: "",
                tags: &[],
                facet: None,
                rels: Some(RelLit {
                    slices,
                    specs,
                    needs: &[],
                    after: &[],
                    triggers: &[],
                }),
            },
        );
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

    // --- SL-039 VT-4: exposure = likelihood × impact, baseline otherwise ---

    fn facet(likelihood: Option<RiskLevel>, impact: Option<RiskLevel>) -> RiskFacet {
        RiskFacet {
            likelihood,
            impact,
            origin: None,
            controls: Vec::new(),
        }
    }

    #[test]
    fn exposure_scores_a_fully_assessed_risk() {
        use RiskLevel::{Critical, High, Low};
        assert_eq!(exposure(Some(&facet(Some(High), Some(Critical)))), 12);
        assert_eq!(exposure(Some(&facet(Some(Low), Some(Low)))), 1);
        assert_eq!(exposure(Some(&facet(Some(Critical), Some(Critical)))), 16);
    }

    #[test]
    fn exposure_is_baseline_when_unassessed_or_non_risk() {
        use RiskLevel::High;
        // one axis only → baseline.
        assert_eq!(exposure(Some(&facet(Some(High), None))), 0);
        assert_eq!(exposure(Some(&facet(None, Some(High)))), 0);
        // no axis → baseline.
        assert_eq!(exposure(Some(&facet(None, None))), 0);
        // non-risk item (no facet) → baseline.
        assert_eq!(exposure(None), 0);
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

    // --- PHASE-03 T1: the ordering projection (project) ---

    /// Seed one item carrying outbound `needs`/`after` axes (the `project` input).
    fn write_rel_item(
        root: &Path,
        kind: ItemKind,
        id: u32,
        status: &str,
        needs: &[&str],
        after: &[AfterLit<'_>],
    ) {
        write_fixture(
            root,
            Fixture {
                kind,
                id,
                slug: "s",
                title: "T",
                status,
                resolution: if matches!(status, "resolved" | "closed") {
                    "done"
                } else {
                    ""
                },
                tags: &[],
                facet: None,
                rels: Some(RelLit {
                    slices: &[],
                    specs: &[],
                    needs,
                    after,
                    triggers: &[],
                }),
            },
        );
    }

    /// The rendered canonical ids of a built order, in composed order.
    fn ordered_ids(inputs: &[OrderInput]) -> Vec<String> {
        BacklogOrder::build(inputs)
            .unwrap()
            .ordered()
            .iter()
            .map(|id| id.render())
            .collect()
    }

    #[test]
    fn project_keeps_non_terminal_nodes_only() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_rel_item(root, ItemKind::Issue, 1, "open", &[], &[]);
        write_rel_item(root, ItemKind::Issue, 2, "resolved", &[], &[]);
        write_rel_item(root, ItemKind::Issue, 3, "closed", &[], &[]);
        write_rel_item(root, ItemKind::Issue, 4, "started", &[], &[]);

        let (inputs, absent) = project(&read_all(root).unwrap());
        assert!(absent.is_empty());
        // only the two non-terminal items (open, started) survive as nodes.
        let mut ids = ordered_ids(&inputs);
        ids.sort();
        assert_eq!(ids, vec!["ISS-001", "ISS-004"]);
    }

    #[test]
    fn project_wires_a_hard_needs_edge_into_the_order() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // ISS-001 needs ISS-002 ⇒ B(002) must precede A(001).
        write_rel_item(root, ItemKind::Issue, 1, "open", &["ISS-002"], &[]);
        write_rel_item(root, ItemKind::Issue, 2, "open", &[], &[]);

        let (inputs, _) = project(&read_all(root).unwrap());
        assert_eq!(ordered_ids(&inputs), vec!["ISS-002", "ISS-001"]);
    }

    #[test]
    fn project_honours_a_cross_kind_after_edge_with_rank() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // a cross-kind soft edge: CHR-001 after RSK-001 ⇒ RSK-001 precedes CHR-001.
        write_rel_item(
            root,
            ItemKind::Chore,
            1,
            "open",
            &[],
            &[AfterLit {
                to: "RSK-001",
                rank: 3,
            }],
        );
        write_rel_item(root, ItemKind::Risk, 1, "open", &[], &[]);

        let (inputs, absent) = project(&read_all(root).unwrap());
        assert!(absent.is_empty(), "both endpoints are live nodes");
        assert_eq!(ordered_ids(&inputs), vec!["RSK-001", "CHR-001"]);
    }

    #[test]
    fn project_records_an_unparseable_ref_as_an_absent_drop() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // a stale/garbage ref that cannot even parse to (kind, id).
        write_rel_item(root, ItemKind::Issue, 1, "open", &["NOPE-1"], &[]);

        let (inputs, absent) = project(&read_all(root).unwrap());
        assert_eq!(
            absent.len(),
            1,
            "the unparseable ref is recorded, not silent"
        );
        assert_eq!(absent[0].from().render(), "ISS-001");
        assert_eq!(absent[0].reference(), "NOPE-1");
        // the node itself still orders (the bad edge just contributes nothing).
        assert_eq!(ordered_ids(&inputs), vec!["ISS-001"]);
    }

    #[test]
    fn project_emits_distinct_item_ids_one_row_per_item() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_rel_item(root, ItemKind::Issue, 1, "open", &[], &[]);
        write_rel_item(root, ItemKind::Risk, 1, "open", &[], &[]);

        let (inputs, _) = project(&read_all(root).unwrap());
        // A-distinct/DD4: the bimap precondition — strictly distinct ItemIds. ISS-001
        // and RSK-001 share a numeric id but differ by kind, so both survive as rows
        // and the build never overwrites a node (would panic/corrupt otherwise).
        assert_eq!(ordered_ids(&inputs).len(), 2);
        assert!(BacklogOrder::build(&inputs).is_ok());
    }

    // --- PHASE-03 T2: edit-preserving relationship-array append ---

    #[test]
    fn append_needs_preserves_comments_and_inert_tables() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        new_item(root, ItemKind::Issue, "Auth"); // real template: seeded [relationships]
        let path = issue_dir(root, "001").join("backlog-001.toml");

        // hand-add an inert table + a comment (the F-1 corruption hazard).
        let mut body = fs::read_to_string(&path).unwrap();
        body.push_str("\n# hand note — keep me\n[custom]\nkeep = \"yes\"\n");
        fs::write(&path, &body).unwrap();

        append_relationship(
            root,
            ItemKind::Issue,
            1,
            &RelEdit::Needs(&["ISS-002".to_string(), "RSK-001".to_string()]),
        )
        .unwrap();

        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("# hand note — keep me"), "comment survives");
        assert!(after.contains("[custom]"), "inert table survives");
        assert!(after.contains("keep = \"yes\""), "unknown key survives");

        // the reader sees both new prereqs on the live axis.
        let item = read_item(root, ItemKind::Issue, 1).unwrap();
        assert_eq!(item.relationships.needs, vec!["ISS-002", "RSK-001"]);
    }

    #[test]
    fn append_after_round_trips_to_and_rank() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        new_item(root, ItemKind::Issue, "Auth");

        append_relationship(
            root,
            ItemKind::Issue,
            1,
            &RelEdit::After {
                to: "ISS-002",
                rank: 5,
            },
        )
        .unwrap();

        let item = read_item(root, ItemKind::Issue, 1).unwrap();
        assert_eq!(
            item.relationships.after,
            vec![AfterEdge {
                to: "ISS-002".to_string(),
                rank: 5,
            }]
        );
    }

    #[test]
    fn append_relationship_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        new_item(root, ItemKind::Issue, "Auth");
        let path = issue_dir(root, "001").join("backlog-001.toml");

        append_relationship(
            root,
            ItemKind::Issue,
            1,
            &RelEdit::Needs(&["ISS-002".to_string()]),
        )
        .unwrap();
        let once = fs::read_to_string(&path).unwrap();

        // a second identical append is a no-op — byte-identical, never duplicated.
        append_relationship(
            root,
            ItemKind::Issue,
            1,
            &RelEdit::Needs(&["ISS-002".to_string()]),
        )
        .unwrap();
        assert_eq!(
            once,
            fs::read_to_string(&path).unwrap(),
            "idempotent append"
        );

        let item = read_item(root, ItemKind::Issue, 1).unwrap();
        assert_eq!(item.relationships.needs, vec!["ISS-002"], "not duplicated");
    }

    #[test]
    fn append_relationship_refuses_a_malformed_missing_array() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // hand-corrupted: a `[relationships]` table that omits the seeded `needs` array.
        let d = root.join(ItemKind::Issue.kind().dir).join("001");
        fs::create_dir_all(&d).unwrap();
        let malformed = "id = 1\nslug = \"a\"\ntitle = \"A\"\nkind = \"issue\"\n\
             status = \"open\"\nresolution = \"\"\ncreated = \"2026-06-08\"\n\
             updated = \"2026-06-08\"\ntags = []\n\n[relationships]\nslices = []\n";
        let path = d.join("backlog-001.toml");
        fs::write(&path, malformed).unwrap();

        let err = append_relationship(
            root,
            ItemKind::Issue,
            1,
            &RelEdit::Needs(&["ISS-002".to_string()]),
        );
        assert!(err.is_err(), "a missing seeded array is refused");
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            malformed,
            "the file is untouched on refuse"
        );
    }

    // --- PHASE-03 T3: `run_needs` shell (VT-5 set-refuse) ---

    #[test]
    fn run_needs_appends_a_validated_prereq() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        new_item(root, ItemKind::Issue, "Auth"); // ISS-001
        new_item(root, ItemKind::Issue, "Login"); // ISS-002

        run_needs(
            Some(root.to_path_buf()),
            "ISS-001",
            &["ISS-002".to_string()],
        )
        .unwrap();

        let item = read_item(root, ItemKind::Issue, 1).unwrap();
        assert_eq!(item.relationships.needs, vec!["ISS-002"]);
    }

    #[test]
    fn run_needs_rejects_a_missing_prereq_ref() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        new_item(root, ItemKind::Issue, "Auth"); // ISS-001 only
        let path = issue_dir(root, "001").join("backlog-001.toml");
        let before = fs::read_to_string(&path).unwrap();

        // ISS-099 does not exist — a hard user error, nothing written.
        let err = run_needs(
            Some(root.to_path_buf()),
            "ISS-001",
            &["ISS-099".to_string()],
        );
        assert!(
            err.is_err(),
            "a missing prereq ref is rejected at author time"
        );
        assert_eq!(
            before,
            fs::read_to_string(&path).unwrap(),
            "nothing written"
        );
    }

    #[test]
    fn run_needs_refuses_a_closing_cycle_naming_members_nothing_written() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // VT-5: seed A.needs=[B]; `needs B A` would close the {A,B} cycle.
        write_rel_item(root, ItemKind::Issue, 1, "open", &["ISS-002"], &[]); // A=001 needs B=002
        write_rel_item(root, ItemKind::Issue, 2, "open", &[], &[]); // B=002
        let path_b = issue_dir(root, "002").join("backlog-002.toml");
        let before_b = fs::read_to_string(&path_b).unwrap();

        let err = run_needs(
            Some(root.to_path_buf()),
            "ISS-002",
            &["ISS-001".to_string()],
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("cycle"), "the refuse names the failure: {msg}");
        assert!(
            msg.contains("ISS-001") && msg.contains("ISS-002"),
            "names members: {msg}"
        );

        // nothing written — B's file is byte-identical.
        assert_eq!(
            before_b,
            fs::read_to_string(&path_b).unwrap(),
            "nothing written on refuse"
        );
    }

    // --- PHASE-03 T4: `run_after` shell (soft — never rejects a cycle) ---

    #[test]
    fn run_after_appends_one_edge_with_default_rank_zero() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        new_item(root, ItemKind::Issue, "Auth"); // ISS-001
        new_item(root, ItemKind::Issue, "Login"); // ISS-002

        run_after(Some(root.to_path_buf()), "ISS-001", "ISS-002", 0).unwrap();

        let item = read_item(root, ItemKind::Issue, 1).unwrap();
        assert_eq!(
            item.relationships.after,
            vec![AfterEdge {
                to: "ISS-002".to_string(),
                rank: 0,
            }]
        );
    }

    #[test]
    fn run_after_never_rejects_a_soft_cycle() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // X.after=[Y] already; `after X Y`-style reciprocal would cycle — but `after`
        // is soft, so it is ACCEPTED (the eviction surfaces at order time, VT-6).
        write_rel_item(
            root,
            ItemKind::Issue,
            1,
            "open",
            &[],
            &[AfterLit {
                to: "ISS-002",
                rank: 1,
            }],
        );
        write_rel_item(root, ItemKind::Issue, 2, "open", &[], &[]);

        // close the reciprocal soft edge Y.after=[X] — must NOT be rejected.
        run_after(Some(root.to_path_buf()), "ISS-002", "ISS-001", 5).unwrap();
        let item = read_item(root, ItemKind::Issue, 2).unwrap();
        assert_eq!(
            item.relationships.after,
            vec![AfterEdge {
                to: "ISS-001".to_string(),
                rank: 5,
            }]
        );
    }

    // --- PHASE-03 T5: `order_rows` compute (the render half) ---

    /// The table portion of an `order` render (before the `overrides:` block) → its
    /// composed-order ids. Reuses [`ids`] over just the table half.
    fn order_ids(out: &str) -> Vec<String> {
        let table = out.split("\noverrides:").next().unwrap_or(out);
        ids(table)
    }

    #[test]
    fn order_rows_composes_a_hard_needs_order() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // ISS-001 needs ISS-002 ⇒ ISS-002 must precede ISS-001 in the order.
        write_rel_item(root, ItemKind::Issue, 1, "open", &["ISS-002"], &[]);
        write_rel_item(root, ItemKind::Issue, 2, "open", &[], &[]);

        let out = order_rows(root).unwrap();
        assert_eq!(
            order_ids(&out),
            vec!["ISS-002", "ISS-001"],
            "B precedes A: {out}"
        );
        assert!(!out.contains("overrides:"), "no drops, no block: {out}");
    }

    #[test]
    fn order_rows_hard_errors_on_a_needs_cycle_with_no_table() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // VT-5: a hand-seeded mutual needs cycle {A,B}.
        write_rel_item(root, ItemKind::Issue, 1, "open", &["ISS-002"], &[]);
        write_rel_item(root, ItemKind::Issue, 2, "open", &["ISS-001"], &[]);

        let err = order_rows(root).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("cycle"), "names the failure: {msg}");
        assert!(
            msg.contains("ISS-001") && msg.contains("ISS-002"),
            "names members: {msg}"
        );
    }

    #[test]
    fn order_rows_evicts_the_lower_rank_edge_of_a_soft_cycle() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // VT-6: X.after=[{to=Y,rank=1}], Y.after=[{to=X,rank=5}] ⇒ the strictly
        // lower-rank edge (X→Y, the edge X.after=[Y] flips to Y→X cordage… name by
        // ItemId) is evicted. The order is still produced; the eviction is recorded.
        write_rel_item(
            root,
            ItemKind::Issue,
            1,
            "open",
            &[],
            &[AfterLit {
                to: "ISS-002",
                rank: 1,
            }],
        );
        write_rel_item(
            root,
            ItemKind::Issue,
            2,
            "open",
            &[],
            &[AfterLit {
                to: "ISS-001",
                rank: 5,
            }],
        );

        let out = order_rows(root).unwrap();
        // both nodes still ordered (the cycle was linearized, not refused).
        let mut shown = order_ids(&out);
        shown.sort();
        assert_eq!(shown, vec!["ISS-001", "ISS-002"]);
        // exactly the soft-cycle eviction is recorded.
        assert!(
            out.contains("overrides:"),
            "the eviction is recorded: {out}"
        );
        assert!(out.contains("soft cycle"), "named a soft-cycle drop: {out}");
    }

    #[test]
    fn order_rows_records_terminal_and_absent_drops_with_status_and_resolution() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // VT-7: ISS-001 needs a terminal (CHR-001 closed/wont-do) AND an absent ref.
        write_rel_item(
            root,
            ItemKind::Issue,
            1,
            "open",
            &["CHR-001", "ISS-099"],
            &[],
        );
        // CHR-001 is terminal — closed with a wont-do resolution (abandoned).
        write_item(
            root,
            ItemKind::Chore,
            1,
            "closed",
            "wont-do",
            "drop-me",
            "Dropped chore",
            &[],
        );

        let out = order_rows(root).unwrap();
        // the live node still orders.
        assert_eq!(
            order_ids(&out),
            vec!["ISS-001"],
            "the live node survives: {out}"
        );
        assert!(out.contains("overrides:"));
        // the terminal dep is named with status+resolution (never silently satisfied).
        assert!(
            out.contains("CHR-001") && out.contains("closed/wont-do"),
            "terminal dep named status/resolution: {out}"
        );
        // the absent ref is named absent.
        assert!(
            out.contains("ISS-099") && out.contains("absent"),
            "absent ref named: {out}"
        );
    }
}
