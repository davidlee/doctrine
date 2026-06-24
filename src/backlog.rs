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

use crate::dtoml;

use crate::backlog_order::{BacklogOrder, ItemId, OrderInput, Override, OverrideReason};
use crate::tag::{self, normalize_tag};
// SL-060 PHASE-02: the dep/sequence schema + the strict edit-preserving append now
// live in the shared `dep_seq` leaf. Backlog uses the leaf TYPE (`AfterEdge`) and the
// leaf `RelEdit`/`append` write seam; its own `read_item`/`dep_seq_for` (the one-parse
// `promoted` projection) stay backlog-local.
use crate::dep_seq::{self, AfterEdge, RelEdit};

use crate::entity::{self, Artifact, Fileset, Inputs, Kind, MaterialiseRequest, ScaffoldCtx};
use crate::listing::{self, Format, ListArgs};
use crate::tomlfmt::toml_string;

use crate::risk;
use crate::risk::{RiskFacet, RiskLevel, exposure, validate_facet};

use clap::Subcommand;

// ---------------------------------------------------------------------------
// CLI enum & dispatch (PHASE-03 relocation from main.rs)
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
pub(crate) enum BacklogCommand {
    /// Allocate the next id in the kind's namespace and scaffold a new item.
    New {
        /// Item kind: issue | improvement | chore | risk | idea.
        kind: ItemKind,

        /// Item title (prompted for if omitted).
        title: Option<String>,

        /// Explicit slug (default: derived from the title).
        #[arg(long)]
        slug: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Survey items across all kinds; filters AND together. Hides terminal
    /// (resolved/closed) by default — `--all` or an explicit `--status` reveals.
    List {
        /// Only this kind.
        #[arg(long)]
        kind: Option<ItemKind>,

        /// Row order: `sequence` (the composed `needs`/`after` work order, default) or
        /// `id` (the classic kind-then-id grouping).
        #[arg(long = "by", value_enum, default_value_t = OrderBy::Sequence)]
        by: OrderBy,

        #[command(flatten)]
        list: crate::CommonListArgs,

        /// Title substring filter (DEPRECATED alias of `--filter`; `--filter` wins).
        substr: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Reassemble one item by id (`ISS-007`) — kind auto-detected from the prefix.
    Show {
        #[command(flatten)]
        common: crate::CommonShowArgs,
    },

    /// Inspect one item's metadata only (no prose body) — kind auto-detected from
    /// the prefix.
    Inspect {
        #[command(flatten)]
        common: crate::CommonShowArgs,
    },

    /// Transition one item's status (and resolution) in place — kind auto-detected
    /// from the prefix. Coupling holds: a terminal status requires a resolution, a
    /// non-terminal forbids one (re-opening auto-clears it).
    Edit {
        /// Canonical item ref (e.g. ISS-007); the prefix selects the kind.
        id: String,

        /// The target status (open | triaged | started | resolved | closed).
        #[arg(long)]
        status: Status,

        /// The resolution (required by a terminal status, forbidden otherwise).
        #[arg(long)]
        resolution: Option<Resolution>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Append hard prerequisites to an item's `needs` axis — kind auto-detected from
    /// the prefix. Validates every ref exists, then refuses a closing dependency
    /// cycle (naming the members; nothing written).
    Needs {
        /// The dependent item ref (e.g. ISS-007); the prefix selects the kind.
        #[arg(value_name = "DEPENDENT")]
        id: String,

        /// One or more prerequisite refs the item must wait on.
        #[arg(required = true, value_name = "PREREQUISITE")]
        prereqs: Vec<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Append one soft-sequence edge to an item's `after` axis — kind auto-detected
    /// from the prefix. Validates the target exists; never rejects a cycle (a soft
    /// preference, surfaced and evicted at `order` time).
    After {
        /// The item ref that should run after the target (e.g. ISS-007).
        #[arg(value_name = "DEPENDENT")]
        id: String,

        /// The predecessor ref this item should follow.
        /// Required unless --prune is set.
        #[arg(required_unless_present = "prune", value_name = "PREDECESSOR")]
        to: Option<String>,

        /// Per-edge rank (a manual tie-break hint; default 0).
        #[arg(long, default_value_t = 0)]
        rank: i32,

        /// Remove matching after edges instead of appending.
        #[arg(long, conflicts_with = "prune")]
        remove: bool,

        /// Drop every dangling after edge from the source item.
        #[arg(long, conflicts_with = "remove")]
        prune: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Add and/or remove tags on an item — kind auto-detected from the prefix. Tags
    /// are lowercased and validated `[a-z0-9_:-]` (colon namespacing, e.g.
    /// `area:backlog`); the stored set is sorted. At least one add or remove required.
    Tag {
        /// Canonical item ref (e.g. ISS-007); the prefix selects the kind.
        id: String,

        /// Tags to add (positional, repeatable).
        tags: Vec<String>,

        /// Tags to remove, repeatable (`-d security -d area:backlog`).
        #[arg(long = "remove", short = 'd')]
        remove: Vec<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Print the file paths of each backlog item's entity directory.
    Paths {
        /// Backlog item reference(s) — `ISS-007`, `IMP-003`, etc.
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
}

pub(crate) fn dispatch(cmd: BacklogCommand, color: bool) -> anyhow::Result<()> {
    match cmd {
        BacklogCommand::New {
            kind,
            title,
            slug,
            path,
        } => run_new(path, kind, title, slug),
        BacklogCommand::List {
            kind,
            by,
            mut list,
            substr,
            path,
        } => {
            // A-7: the positional `[SUBSTR]` is a DEPRECATED alias of `--filter`;
            // `--filter` WINS when both are given (the positional folds in only
            // when `--filter` is absent). Documented precedence, not an error.
            if list.filter.is_none() {
                list.filter = substr;
            }
            run_list(path, kind, by, list.into_list_args(color))
        }
        BacklogCommand::Show { common } => {
            let format = if common.json {
                Format::Json
            } else {
                common.format
            };
            run_show(common.path, &common.id, format)
        }
        BacklogCommand::Inspect { common } => {
            let format = if common.json {
                Format::Json
            } else {
                common.format
            };
            run_inspect(common.path, &common.id, format)
        }
        BacklogCommand::Edit {
            id,
            status,
            resolution,
            path,
        } => run_edit(path, &id, status, resolution),
        BacklogCommand::Needs { id, prereqs, path } => run_needs(path, &id, &prereqs),
        BacklogCommand::After {
            id,
            to,
            rank,
            remove,
            prune,
            path,
        } => run_after(path, &id, to.as_deref(), rank, remove, prune),
        BacklogCommand::Tag {
            id,
            tags,
            remove,
            path,
        } => run_tag(path, &id, &tags, &remove),
        BacklogCommand::Paths {
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
    }
}

// ---------------------------------------------------------------------------

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
    prefix: crate::kinds::ISS,
    stem: "backlog",
    scaffold: |c| backlog_scaffold(ItemKind::Issue, c),
};

/// The improvement kind: an enhancement to existing behaviour.
pub(crate) const IMPROVEMENT_KIND: Kind = Kind {
    dir: ".doctrine/backlog/improvement",
    prefix: crate::kinds::IMP,
    stem: "backlog",
    scaffold: |c| backlog_scaffold(ItemKind::Improvement, c),
};

/// The chore kind: maintenance with no user-visible behaviour change.
pub(crate) const CHORE_KIND: Kind = Kind {
    dir: ".doctrine/backlog/chore",
    prefix: crate::kinds::CHR,
    stem: "backlog",
    scaffold: |c| backlog_scaffold(ItemKind::Chore, c),
};

/// The risk kind: a tracked risk — the only kind carrying a `[facet]`.
pub(crate) const RISK_KIND: Kind = Kind {
    dir: ".doctrine/backlog/risk",
    prefix: crate::kinds::RSK,
    stem: "backlog",
    scaffold: |c| backlog_scaffold(ItemKind::Risk, c),
};

/// The idea kind: a speculative possibility, not yet committed work.
pub(crate) const IDEA_KIND: Kind = Kind {
    dir: ".doctrine/backlog/idea",
    prefix: crate::kinds::IDE,
    stem: "backlog",
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
    /// prefix + scaffold. `pub(crate)` so the lazyspec loader composes a backlog
    /// item's tree dir + tier-1 edge vocabulary off the single source (SL-026).
    pub(crate) const fn kind(self) -> &'static Kind {
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
    pub(crate) const fn as_str(self) -> &'static str {
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
        listing::canonical_id(self.prefix(), id)
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
    pub(crate) const ALL: [ItemKind; 5] = [
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
    pub(crate) const fn as_str(self) -> &'static str {
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
    pub(crate) const fn is_terminal(self) -> bool {
        matches!(self, Status::Resolved | Status::Closed)
    }
}

/// The `backlog list` known-status set (A-2) — the five `Status` variants, the
/// authority `--status` is validated against. Lockstep-guarded against the enum by
/// `backlog_statuses_matches_the_variants`. backlog has a CLOSED status enum, so a
/// *stored* status is always in-vocabulary — no drift marker is possible.
pub(crate) const BACKLOG_STATUSES: &[&str] = &["open", "triaged", "started", "resolved", "closed"];

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
    facet: Option<risk::RawRiskFacet>,
    #[serde(default)]
    relationships: Relationships,
}

/// The validated entity (design §5.2). `id/slug/title/status` are top-level in the
/// toml so the file also round-trips into the shared `meta::Meta`. `kind` is stored
/// AND implied by the tree dir — stored so one read yields the entity without path
/// inspection. The `"" -> None` optionals are resolved off the raw layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BacklogItem {
    pub(crate) id: u32,
    slug: String,
    pub(crate) title: String,
    pub(crate) kind: ItemKind,
    pub(crate) status: Status,
    resolution: Option<Resolution>,
    created: String,
    updated: String,
    tags: Vec<String>,
    facet: Option<RiskFacet>,
    relationships: Relationships,
    /// SL-048 PHASE-04: the migrated tier-1 cross-kind edges (`slices`/`specs`/
    /// `drift`) read generically from the `[[relation]]` block in canonical order.
    /// Populated by [`read_item`] from the raw TOML text; the `validate` test seam
    /// leaves it empty (those tests assert the typed dep/sequence axes, not tier-1).
    tier1: Vec<crate::relation::RelationEdge>,
    /// Prose body read from the sibling `backlog-NNN.md`.
    pub(crate) body: String,
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

/// The item→item dependency / sequence / mask axes (PRD-009) — `needs` (hard
/// prerequisite, payload-free), `after` (soft manual sequence, per-edge optional
/// `rank`), and the `triggers` rider (watched source globs). Shared verbatim by the
/// raw and validated layers (no `"" -> None` seam), seeded empty so `#[serde(default)]`
/// parses a virgin item.
///
/// SL-048 PHASE-04 (the cut): the tier-1 cross-kind axes (`slices`/`specs`/`drift`)
/// migrated to uniform `[[relation]]` rows (read via `relation::read_block` →
/// `BacklogItem::tier1`), so they are NO LONGER typed fields here. The dep/sequence/
/// mask axes (SL-047) carry per-edge payloads and stay typed.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
struct Relationships {
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
        // Filled by `read_item` from the raw TOML text (read_block); empty otherwise.
        tier1: Vec::new(),
        // Filled by `read_item` from the sibling .md; empty otherwise.
        body: String::new(),
    })
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
    let (backend, mut reserved) = crate::reserve::backend(
        &root,
        item_kind.kind().prefix,
        crate::install::prompt_confirm,
    )?;
    let out = entity::materialise(
        item_kind.kind(),
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
    let raw: RawBacklogToml = dtoml::parse_entity_toml(&text, item_kind.prefix(), id)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    let mut item = validate(raw)?;
    // SL-048 PHASE-04: the migrated tier-1 axes (slices/specs/drift) come from the
    // `[[relation]]` block, read generically in canonical order.
    item.tier1 = crate::relation::tier1_edges(item_kind.kind(), &text)?;
    let md_path = root
        .join(item_kind.kind().dir)
        .join(&name)
        .join(format!("{BACKLOG_STEM}-{name}.md"));
    item.body = std::fs::read_to_string(&md_path)
        .with_context(|| format!("Failed to read {}", md_path.display()))?;
    Ok(item)
}

/// Resolve a backlog canonical-id prefix (`ISS`/`IMP`/`CHR`/`RSK`/`IDE`) back to its
/// [`ItemKind`] — the inverse of `ItemKind::prefix`, over the single `ItemKind::ALL`
/// source. `pub(crate)` so the SL-046 cross-kind dispatch (`relation_graph`) routes a
/// backlog prefix to [`relation_edges`] without a second prefix↔kind copy.
pub(crate) fn kind_from_prefix(prefix: &str) -> Option<ItemKind> {
    ItemKind::from_prefix(prefix)
}

/// A backlog item's authored outbound relations (SL-046 §5.2/§5.3): `slices` →
/// [`RelationLabel::Slices`], `references` (role-refined, SL-149) →
/// [`RelationLabel::References`], and `drift` → [`RelationLabel::Drift`]. `drift` is
/// free-text with no `DRIFT` kind in `KINDS`, so
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
    // SL-048 PHASE-04 (the cut): the tier-1 axes (slices/specs/drift) now live in the
    // uniform `[[relation]]` block, read generically into `item.tier1` in canonical
    // [`RELATION_RULES`] order (X1). Backlog has no other tier-1 edges; the typed
    // needs/after/triggers axes are NOT outbound relation edges (the SL-047 dep seam).
    let item = read_item(root, item_kind, id)?;
    Ok(item.tier1)
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
pub(crate) fn read_all(root: &Path) -> anyhow::Result<Vec<BacklogItem>> {
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
    /// The item's own tags — projected UNCONDITIONALLY (flat, never visibility-gated);
    /// an untagged item emits `[]` (SL-067 PHASE-01, EX-4).
    tags: Vec<String>,
}

/// The table columns `backlog list` can show (`--columns` tokens over
/// `R = BacklogItem` — extractors are non-capturing, SL-037 D5; the prefixed id
/// is materialised in the cell from the item's own kind+id). Declaration order is
/// what the unknown-column error lists.
const BL_COLUMNS: [listing::Column<BacklogItem>; 6] = [
    listing::Column {
        name: "id",
        header: "id",
        cell: |i| i.kind.canonical_id(i.id),
        paint: listing::ColumnPaint::Fixed(owo_colors::DynColors::Ansi(
            owo_colors::AnsiColors::Cyan,
        )),
    },
    listing::Column {
        name: "kind",
        header: "kind",
        cell: |i| i.kind.as_str().to_string(),
        paint: listing::ColumnPaint::ByValue(|i| listing::backlog_kind_hue(i.kind.as_str())),
    },
    listing::Column {
        name: "status",
        header: "status",
        cell: |i| i.status.as_str().to_string(),
        paint: listing::ColumnPaint::ByValue(|i| listing::status_hue(i.status.as_str())),
    },
    listing::Column {
        name: "slug",
        header: "slug",
        cell: |i| i.slug.clone(),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "tags",
        header: "tags",
        // `cell` (plain) and `split` (coloured) MUST agree byte-for-byte stripped of
        // ANSI: both project the item's tags joined by `", "`.
        cell: |i| i.tags.join(", "),
        paint: listing::ColumnPaint::PerToken {
            split: |i| i.tags.clone(),
            render: listing::paint_tag,
        },
    },
    listing::Column {
        name: "title",
        header: "title",
        cell: |i| i.title.clone(),
        paint: listing::ColumnPaint::Alternate([listing::TITLE_EVEN, listing::TITLE_ODD]),
    },
];

/// The default visible set — slug-free (SL-037 D4); `--columns …,slug` reveals it.
const BL_DEFAULT: &[&str] = &["id", "kind", "status", "title"];

/// How `backlog list` orders its rows (SL-051, the folded-in `order` axis). The
/// default `Sequence` composes the cordage `needs`/`after` work order over the live
/// corpus; `Id` is the classic `(kind.ordinal, id)` grouping.
#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
pub(crate) enum OrderBy {
    /// The composed `needs`/`after` work order (default).
    #[default]
    Sequence,
    /// The classic `(kind.ordinal, id)` grouping.
    Id,
}

/// The composed ordering over the live corpus plus its honest-record diagnostic
/// (SL-051 — the folded-in `order` view). The two outcomes are distinct variants so
/// the illegal mixes — a degrade carrying composed positions, a clean compose carrying
/// a warning — are unrepresentable. `footer` (the `render_overrides` honest-record
/// block, `""` when nothing was dropped) rides both.
enum Ordering {
    /// A clean compose: `pos` maps each composed item to its sequence position.
    Composed {
        pos: BTreeMap<ItemId, usize>,
        footer: String,
    },
    /// A `needs` cycle forced the classic id-sort fallback; `warning` is the stderr
    /// advisory naming the cycle.
    Degraded { footer: String, warning: String },
}

/// Compose the `needs`/`after` work order over the live corpus (SL-051 — the former
/// `order_rows` compute, folded into `list`). PURE over the read corpus. Projects the
/// non-terminal node set, builds the adapter, and renders the honest-record `footer`.
/// On a `needs` dependency cycle, `build` still succeeds; `compose` returns
/// `Degraded` (carrying the cycle `warning`), so `list_rows` falls back to the classic
/// id sort and emits the advisory to stderr (no misleading order, never a non-zero
/// exit — SL-051 §4.4). Borrows `corpus` (then `list_rows` MOVES it into `retain`).
fn compose(corpus: &[BacklogItem]) -> anyhow::Result<Ordering> {
    let (inputs, absent) = project(corpus);
    let order = BacklogOrder::build(&inputs)?;
    let cmap: BTreeMap<ItemId, &BacklogItem> = corpus
        .iter()
        .map(|i| (ItemId::new(i.kind, i.id), i))
        .collect();
    let footer = render_overrides(&cmap, &absent, &order.overrides());
    if let Some(cycle) = order.dep_cycles().first() {
        return Ok(Ordering::Degraded {
            footer,
            warning: format!(
                "backlog list: `needs` dependency cycle — {} — ordering by id (resolve, then re-run)",
                name_cycle(cycle)
            ),
        });
    }
    let pos = order
        .ordered()
        .iter()
        .enumerate()
        .map(|(i, id)| (*id, i))
        .collect();
    Ok(Ordering::Composed { pos, footer })
}

/// The `list_rows` output split — the two destination streams named so the `run_list`
/// shell cannot transpose them (a swap would misroute the cycle warning into stdout
/// and corrupt the goldens — SL-051 §4.3). The type, not a doc-comment, is the guard.
struct ListOutput {
    stdout: String,
    stderr: String,
}

/// The `backlog list` output — the compute half of `run_list`, on the shared spine.
/// Returns a [`ListOutput`]: `stdout` carries the rendered rows (plus the honest-record
/// `footer` in table mode); `stderr` carries the cycle `warning` (and, under `--json`,
/// the advisory `footer`).
///
/// `validate_statuses` guards `--status` (A-2); `listing::build` resolves the filter +
/// format; `retain` applies the shared substr/regex/status/tag axes + the terminal
/// hide-set ([`is_hidden`], reusing `Status::is_terminal`); the kind-specific `--kind`
/// filter (not a shared axis) is applied here. The corpus is read ONCE: `--by
/// sequence` composes the work order over the FULL non-terminal corpus (`compose`
/// borrows first), then `retain` MOVES the corpus and the surviving rows tail by
/// `(usize::MAX, kind.ordinal, id)` for off-sequence items. `--by id` (or a
/// cycle-degrade) skips the graph and sorts by `(kind.ordinal, id)` (§5.3). Membership
/// is EXACTLY `retain ∩ --kind` either way (A-2 invariant) — the ordering never filters.
fn list_rows(
    root: &Path,
    kind: Option<ItemKind>,
    by: OrderBy,
    mut args: ListArgs,
) -> anyhow::Result<ListOutput> {
    validate_statuses(&args.status, BACKLOG_STATUSES)?;
    let render = args.render;
    let columns = args.columns.take();
    let (filter, format) = listing::build(args)?;
    let corpus = read_all(root)?;
    let ordering = match by {
        OrderBy::Sequence => Some(compose(&corpus)?),
        OrderBy::Id => None,
    };
    let mut items = listing::retain(corpus, &filter, is_hidden, key);
    items.retain(|i| kind.is_none_or(|k| i.kind == k));
    // Dynamic tags-column visibility (D2): the column shows iff the FINAL displayed set
    // (post-retain ∩ post-`--kind`) carries at least one tagged row. Computed once, on
    // the visible rows, and reused across any `--by id` layout (uniform).
    let any_tagged = items.iter().any(|i| !i.tags.is_empty());
    // Only a clean `Composed` sorts by sequence; a `Degraded` cycle and `--by id`
    // both fall to the classic `(kind.ordinal, id)`. Off-sequence rows tail via the
    // `usize::MAX` sentinel.
    match &ordering {
        Some(Ordering::Composed { pos, .. }) => items.sort_by_key(|i| {
            (
                pos.get(&ItemId::new(i.kind, i.id))
                    .copied()
                    .unwrap_or(usize::MAX),
                i.kind.ordinal(),
                i.id,
            )
        }),
        _ => items.sort_by_key(|i| (i.kind.ordinal(), i.id)),
    }
    let (footer, warning) = match &ordering {
        Some(Ordering::Composed { footer, .. }) => (footer.as_str(), ""),
        Some(Ordering::Degraded { footer, warning }) => (footer.as_str(), warning.as_str()),
        None => ("", ""),
    };
    match format {
        Format::Table => {
            // Build the effective default LOCALLY (never mutate the `BL_DEFAULT` const):
            // splice `"tags"` before `"title"` IFF a visible row is tagged. With
            // `--columns` given, `select_columns` ignores `default` entirely (the user's
            // order wins verbatim — tags shown iff requested, even all-empty).
            let effective_default: Vec<&str> = if any_tagged {
                BL_DEFAULT
                    .iter()
                    .flat_map(|&c| {
                        if c == "title" {
                            vec!["tags", "title"]
                        } else {
                            vec![c]
                        }
                    })
                    .collect()
            } else {
                BL_DEFAULT.to_vec()
            };
            let sel = listing::select_columns(&BL_COLUMNS, &effective_default, columns.as_deref())?;
            let table = listing::render_columns(&items, &sel, render);
            // Table: rows + footer to stdout; the cycle warning to stderr.
            Ok(ListOutput {
                stdout: format!("{table}{footer}"),
                stderr: warning.to_string(),
            })
        }
        Format::Json => {
            // JSON: the envelope (rows in composed sequence) to stdout; the warning
            // and the advisory footer to stderr (the honest-record stays out of the
            // envelope — no listing.rs change).
            let envelope = listing::json_envelope("backlog", &json_rows(&items))?;
            Ok(ListOutput {
                stdout: envelope,
                stderr: format!("{warning}{footer}"),
            })
        }
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
            tags: i.tags.clone(),
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
    by: OrderBy,
    args: ListArgs,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let ListOutput { stdout, stderr } = list_rows(&root, kind, by, args)?;
    write!(io::stdout(), "{stdout}")?;
    if !stderr.is_empty() {
        write!(io::stderr(), "{stderr}")?;
    }
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

/// Render a `BacklogItem` for `show` or `inspect` — a pure fn of the item's OWN
/// local state ("cannot go stale"), so it reads no other file and surfaces no
/// inbound refs (the reverse view is the deferred registry surface's, ADR-004).
/// House style: `Vec<String>` parts each carrying their own newline, joined by
/// `concat()` (the `spec::render`/`format_rows` precedent — avoids the
/// `push_str(&format!)` lint). The facet block is gated on `item.facet` (risk
/// only); relationship axes and the optional fields render only when populated.
fn format_metadata(item: &BacklogItem) -> Vec<String> {
    use crate::relation::{RelationLabel, Role, targets_for, targets_for_role};
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
    // SL-048 PHASE-04: the tier-1 axes (slices/drift) come from `item.tier1` (read via
    // `read_block`), the dep/sequence axes stay typed.
    let rel = &item.relationships;
    let slices = targets_for(&item.tier1, RelationLabel::Slices);
    let drift = targets_for(&item.tier1, RelationLabel::Drift);
    // SL-149: the `references` label by role. A backlog item only ever authors
    // `concerns` (implements/scoped_from are SL-only), so the other buckets stay empty;
    // each axis renders only when non-empty. PHASE-05's migration rewrote the old
    // backlog `specs`→canon edges onto `references(concerns)`.
    let ref_concerns = targets_for_role(&item.tier1, RelationLabel::References, Role::Concerns);
    let ref_implements = targets_for_role(&item.tier1, RelationLabel::References, Role::Implements);
    let ref_scoped_from =
        targets_for_role(&item.tier1, RelationLabel::References, Role::ScopedFrom);
    if !slices.is_empty()
        || !drift.is_empty()
        || !rel.needs.is_empty()
        || !rel.after.is_empty()
        || !rel.triggers.is_empty()
        || !ref_implements.is_empty()
        || !ref_scoped_from.is_empty()
        || !ref_concerns.is_empty()
    {
        parts.push("\nrelationships:\n".to_string());
        // the four string axes share the one loop; `after`/`triggers` carry payload
        // (per-edge rank, glob+note) and render bespoke below, in §5.2 key order.
        for (label, refs) in [
            ("slices", &slices),
            ("drift", &drift),
            ("needs", &rel.needs),
            ("references(implements)", &ref_implements),
            ("references(scoped_from)", &ref_scoped_from),
            ("references(concerns)", &ref_concerns),
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

    parts
}

/// Render a `BacklogItem` for `show` — metadata + prose body.
fn format_show(item: &BacklogItem) -> String {
    let mut parts = format_metadata(item);
    parts.push(format!("\n{}", item.body));
    parts.concat()
}

/// Render a `BacklogItem` for `inspect` — metadata only, no prose body.
fn format_inspect(item: &BacklogItem) -> String {
    format_metadata(item).concat()
}

/// `doctrine backlog show <ID>` — reassemble metadata + prose body (PRD-009 REQ-051, §5.4). Thin
/// shell: find the root, `parse_ref` the id to its kind (prefix auto-detect), read
/// THAT item's single toml, render it to stdout. READ-ONLY — no mutation, no
/// cross-corpus scan (only the one item's file is opened); the render is pure over
/// the item's own state.
/// Shared shell: root-find → parse → read → render. The `format_table` fn and
/// `with_body` flag select the table renderer and whether JSON includes the prose body.
fn run_show_inspect(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
    format_table: fn(&BacklogItem) -> String,
    with_body: bool,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (item_kind, id) = parse_ref(reference)?;
    let item = read_item(&root, item_kind, id)?;
    let out = match format {
        Format::Table => format_table(&item),
        Format::Json => show_json(&item, with_body)?,
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
}

/// `doctrine backlog show <ID>` — reassemble metadata + prose body (PRD-009 REQ-051, §5.4).
pub(crate) fn run_show(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
) -> anyhow::Result<()> {
    run_show_inspect(path, reference, format, format_show, true)
}

/// Render the `Json` for show (`with_body=true`) or inspect (`with_body=false`).
/// The shared `{kind, …}` envelope (the `adr::show_json` precedent). The validated
/// `BacklogItem`'s fields are private and its closed enums render via `as_str`, so the
/// JSON is projected by hand (not a derive): the flat identity, the optional resolution,
/// the risk `[facet]` (risk only), and the outbound relationships — the same data the
/// table reassembles, structured. Pure over the item's own state (no cross-corpus scan).
fn show_json(item: &BacklogItem, with_body: bool) -> anyhow::Result<String> {
    use crate::relation::{RelationLabel, Role, targets_for, targets_for_role};
    let facet = item.facet.as_ref().map(|f| {
        serde_json::json!({
            "likelihood": f.likelihood.map(RiskLevel::as_str),
            "impact": f.impact.map(RiskLevel::as_str),
            "origin": f.origin,
            "controls": f.controls,
        })
    });
    let rel = &item.relationships;
    let mut inner = serde_json::Map::new();
    inner.insert(
        "id".into(),
        serde_json::json!(item.kind.canonical_id(item.id)),
    );
    inner.insert("kind".into(), serde_json::json!(item.kind.as_str()));
    inner.insert("slug".into(), serde_json::json!(item.slug));
    inner.insert("title".into(), serde_json::json!(item.title));
    inner.insert("status".into(), serde_json::json!(item.status.as_str()));
    inner.insert(
        "resolution".into(),
        serde_json::json!(item.resolution.map(Resolution::as_str)),
    );
    inner.insert("created".into(), serde_json::json!(item.created));
    inner.insert("updated".into(), serde_json::json!(item.updated));
    inner.insert("tags".into(), serde_json::json!(item.tags));
    if with_body {
        inner.insert("body".into(), serde_json::json!(item.body));
    }
    inner.insert("facet".into(), serde_json::json!(facet));
    inner.insert("relationships".into(), serde_json::json!({
        "slices": targets_for(&item.tier1, RelationLabel::Slices),
        "drift": targets_for(&item.tier1, RelationLabel::Drift),
        "needs": rel.needs,
        "after": rel.after,
        "triggers": rel.triggers,
        "references": {
            "implements": targets_for_role(&item.tier1, RelationLabel::References, Role::Implements),
            "scoped_from": targets_for_role(&item.tier1, RelationLabel::References, Role::ScopedFrom),
            "concerns": targets_for_role(&item.tier1, RelationLabel::References, Role::Concerns),
        },
    }));
    let value = serde_json::json!({
        "kind": "backlog",
        "backlog": inner,
    });
    serde_json::to_string_pretty(&value).context("failed to serialize backlog show JSON")
}

/// `doctrine backlog inspect <ID> [--format table|json]` — metadata-only (no prose
/// body). Thin shell: same read path as `show`, rendered via `format_inspect`.
pub(crate) fn run_inspect(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
) -> anyhow::Result<()> {
    run_show_inspect(path, reference, format, format_inspect, false)
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
    // Gate in the shell: the status⟺resolution coupling + D9 reopen clear. Keep it
    // here, BEFORE the delegated write; the resolution string is still returned to the
    // caller's confirm line.
    let resolution = validate_transition(status, resolution)?;
    let name = format!("{id:03}");
    let path = root
        .join(item_kind.kind().dir)
        .join(&name)
        .join(format!("{BACKLOG_STEM}-{name}.toml"));
    // Delegate the write-core (no-op guard + F-1 refuse + edit-preserving insert) to
    // the shared authored-TOML seam. The three managed pairs prove the longest shape.
    // Hint preserved verbatim (EX-4 rewording is scoped to gov + requirement).
    let hint = format!(
        "malformed backlog item {name}: missing seeded `status`/`resolution`/`updated` — restore the missing keys and retry; the file is left untouched"
    );
    dep_seq::set_authored_status(
        &path,
        &[
            ("status", status.as_str()),
            ("resolution", resolution),
            ("updated", today),
        ],
        &hint,
    )?;
    Ok(resolution)
}

/// One outbound item→item relationship-axis append (PHASE-03 set verbs). Resolves
/// the item's `backlog-NNN.toml` path and delegates to the shared `dep_seq::append`
/// write seam (SL-060 PHASE-02 lift) — the strict edit-preserving `toml_edit` append
/// that refuses (F-1, non-destructively) on a missing seeded array. Backlog keeps
/// this thin wrapper so its callers stay path-blind (root + kind + id), while the
/// schema + write body are shared with the future slice consumer. The `RelEdit`
/// variants are the leaf's, re-imported above.
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
    dep_seq::append(&path, edit)
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
/// `list`'s sequence compose only ever defends against later staleness). Returns the
/// resolved id.
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
        crate::integrity::ensure_ref_resolves(&root, prereq)
            .with_context(|| format!("prerequisite `{prereq}` does not resolve"))?;
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

/// `doctrine backlog after <ITEM> <TO> [--rank N] [--remove] [--prune]` — append
/// or remove ONE soft-sequence edge (PRD-009, design §5.5). **Never** rejects a
/// cycle — a soft `after` cycle is surfaced (and an edge evicted) when `list`
/// composes the sequence (VT-6).
pub(crate) fn run_after(
    path: Option<PathBuf>,
    reference: &str,
    to: Option<&str>,
    rank: i32,
    remove: bool,
    prune: bool,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let target = require_item(&root, reference)?;

    if prune {
        let name = format!("{:03}", target.1);
        let item_path = root
            .join(target.0.kind().dir)
            .join(&name)
            .join(format!("{BACKLOG_STEM}-{name}.toml"));
        let ds = dep_seq::read(&item_path)?;

        let mut dropped: Vec<(String, i32, String)> = Vec::new();
        let mut to_drop: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

        for edge in &ds.after {
            let is_dangling = match crate::integrity::parse_canonical_ref(&edge.to) {
                Ok((kref, tid)) => {
                    let target_path =
                        crate::entity::id_path(&root, kref.kind, tid, crate::entity::Ext::Toml);
                    if target_path.exists() {
                        let body = std::fs::read_to_string(&target_path).unwrap_or_default();
                        let val: toml::Value = match toml::from_str(&body) {
                            Ok(v) => v,
                            Err(_) => toml::Value::Table(toml::Table::new()),
                        };
                        let status = val.get("status").and_then(|s| s.as_str()).unwrap_or("");
                        status == "resolved" || status == "closed"
                    } else {
                        true
                    }
                }
                Err(_) => true,
            };

            if is_dangling {
                let reason = match crate::integrity::parse_canonical_ref(&edge.to) {
                    Ok((kref2, tid2)) => {
                        let target_path = crate::entity::id_path(
                            &root,
                            kref2.kind,
                            tid2,
                            crate::entity::Ext::Toml,
                        );
                        if target_path.exists() {
                            let body = std::fs::read_to_string(&target_path).unwrap_or_default();
                            let val: toml::Value = match toml::from_str(&body) {
                                Ok(v) => v,
                                Err(_) => toml::Value::Table(toml::Table::new()),
                            };
                            let status = val.get("status").and_then(|s| s.as_str()).unwrap_or("");
                            let resolution =
                                val.get("resolution").and_then(|s| s.as_str()).unwrap_or("");
                            if resolution.is_empty() {
                                status.to_string()
                            } else {
                                format!("{status}/{resolution}")
                            }
                        } else {
                            "absent".to_string()
                        }
                    }
                    Err(_) => "(unparseable)".to_string(),
                };
                dropped.push((edge.to.clone(), edge.rank, reason));
                to_drop.insert(edge.to.clone());
            }
        }

        if dropped.is_empty() {
            writeln!(
                io::stdout(),
                "{}: nothing to prune",
                target.0.canonical_id(target.1)
            )?;
            return Ok(());
        }

        for target_id in &to_drop {
            let _ = dep_seq::remove(&item_path, target_id, None)?;
        }

        for (target_id, r, reason) in &dropped {
            writeln!(
                io::stdout(),
                "{} after {target_id} (rank {r}) dropped (dangling: {reason})",
                target.0.canonical_id(target.1),
            )?;
        }
        return Ok(());
    }

    if remove {
        let to = to.ok_or_else(|| anyhow::anyhow!("--remove requires a target"))?;
        require_item(&root, to)?;
        let name = format!("{:03}", target.1);
        let item_path = root
            .join(target.0.kind().dir)
            .join(&name)
            .join(format!("{BACKLOG_STEM}-{name}.toml"));
        let ceiling = if rank == 0 { None } else { Some(rank) };
        let removed = dep_seq::remove(&item_path, to, ceiling)?;
        if removed == 0 {
            anyhow::bail!(
                "{} has no after edge to {to}",
                target.0.canonical_id(target.1)
            );
        }
        writeln!(
            io::stdout(),
            "{} after {to} removed ({} edge{})",
            target.0.canonical_id(target.1),
            removed,
            if removed == 1 { "" } else { "s" }
        )?;
        return Ok(());
    }

    // Original append path
    let to = to.ok_or_else(|| anyhow::anyhow!("after requires a target"))?;
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
// SL-067 PHASE-01: the `backlog tag` verb
// ---------------------------------------------------------------------------

/// `doctrine backlog tag <ID> [TAGS]… [--remove/-d <TAGS>…]` — the tag-edit verb
/// (SL-067 PHASE-01, §4.1/§4.3). Thin impure shell: find the root, `parse_ref` +
/// `require_item` (a missing id hard-errors, never an implicit create), normalise
/// the adds/removes through the WRITE chokepoint
/// [`crate::tag::normalize_tag`], reject an
/// add∩remove overlap (a user error), then apply the edit-preserving set-replace
/// in place (clock injected) and print the post-state. At least one add OR remove is
/// required (clap enforces neither alone, so the shell does — EX-1).
pub(crate) fn run_tag(
    path: Option<PathBuf>,
    reference: &str,
    adds: &[String],
    removes: &[String],
) -> anyhow::Result<()> {
    if adds.is_empty() && removes.is_empty() {
        anyhow::bail!("`backlog tag` needs at least one tag to add or remove (--remove/-d)");
    }
    let add_set: std::collections::BTreeSet<String> = adds
        .iter()
        .map(|t| normalize_tag(t))
        .collect::<anyhow::Result<_>>()?;
    let remove_set: std::collections::BTreeSet<String> = removes
        .iter()
        .map(|t| normalize_tag(t))
        .collect::<anyhow::Result<_>>()?;
    // A tag in BOTH add and remove (after normalisation) is contradictory — reject
    // rather than silently letting the remove win (user error, §4.1).
    let overlap: Vec<&String> = add_set.intersection(&remove_set).collect();
    if let Some(first) = overlap.first() {
        anyhow::bail!("tag `{first}` is in both add and remove (pick one)");
    }

    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (item_kind, id) = require_item(&root, reference)?;
    let name = format!("{id:03}");
    let item_path = root
        .join(item_kind.kind().dir)
        .join(&name)
        .join(format!("{BACKLOG_STEM}-{name}.toml"));

    let text = std::fs::read_to_string(&item_path)
        .with_context(|| format!("backlog item not found at {}", item_path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", item_path.display()))?;
    let changed = tag::apply_tags_set(&mut doc, &add_set, &remove_set, &crate::clock::today())?;
    if changed {
        crate::fsutil::write_atomic(&item_path, doc.to_string().as_bytes())
            .with_context(|| format!("Failed to write {}", item_path.display()))?;
    }

    // Print the post-state (the resulting tag set, sorted) — re-derived from the doc
    // so it is faithful whether or not a write occurred.
    let final_tags: Vec<String> = doc
        .as_table()
        .get("tags")
        .and_then(toml_edit::Item::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let listed = if final_tags.is_empty() {
        "(none)".to_string()
    } else {
        final_tags.join(", ")
    };
    writeln!(
        io::stdout(),
        "Tagged {}: {listed}",
        item_kind.canonical_id(id),
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// `backlog paths` — file paths for each backlog item's entity directory
// ---------------------------------------------------------------------------

/// `doctrine backlog paths <ref>…` — resolve each ref to its entity directory and
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
        let (item_kind, id) = parse_ref(r)?;
        let name = format!("{id:03}");
        let entity_dir = root.join(item_kind.kind().dir).join(&name);
        let toml_name = format!("{BACKLOG_STEM}-{name}.toml");
        let md_name = format!("{BACKLOG_STEM}-{name}.md");
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

// ---------------------------------------------------------------------------
// The honest-record block (SL-051: folded into `backlog list --by sequence`)
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

    // adapter-level drops. Suppress Dangling overrides whose from-endpoint is
    // terminal (IDE-019): a stale dep on resolved/closed work is noise; only
    // truly-absent endpoints matter in the default view.
    for ov in overrides {
        if ov.reason() == OverrideReason::Dangling
            && corpus
                .get(&ov.from())
                .is_some_and(|item| item.status.is_terminal())
        {
            continue;
        }
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

// ---------------------------------------------------------------------------
// Test support (SL-026 PHASE-02)
// ---------------------------------------------------------------------------

/// Promoted backlog fixture builder — the SINGLE source of the backlog-NNN.toml
/// fixture literal (SL-027 DRY'd it; re-rolling it re-opens the closed ISS-001
/// debt). Promoted to `pub(crate)` in place so a later golden-corpus phase can
/// seed the same TOML via `crate::backlog::test_support::write_fixture(...)`
/// without dragging `ItemKind`/`RiskFacet`/the borrowed `Fixture` across a module
/// boundary. The backlog suite drives it through these items unchanged.
#[cfg(test)]
pub(crate) mod test_support {
    use super::ItemKind;
    use std::fs;
    use std::path::Path;

    /// A backlog-NNN.toml fixture spec — the single source of the test fixture
    /// literal. `'a` (not `'static`) because `write_related` passes borrowed
    /// `slices`/`specs`. `facet`/`rels` absent → that block is omitted.
    pub(crate) struct Fixture<'a> {
        pub(crate) kind: ItemKind,
        pub(crate) id: u32,
        pub(crate) slug: &'a str,
        pub(crate) title: &'a str,
        pub(crate) status: &'a str,
        pub(crate) resolution: &'a str,
        pub(crate) tags: &'a [&'a str],
        pub(crate) facet: Option<FacetLit<'a>>,
        pub(crate) rels: Option<RelLit<'a>>,
    }

    pub(crate) struct FacetLit<'a> {
        pub(crate) likelihood: &'a str,
        pub(crate) impact: &'a str,
        pub(crate) origin: &'a str,
        pub(crate) controls: &'a [&'a str],
    }

    pub(crate) struct RelLit<'a> {
        pub(crate) slices: &'a [&'a str],
        pub(crate) specs: &'a [&'a str],
        pub(crate) needs: &'a [&'a str],
        pub(crate) after: &'a [AfterLit<'a>],
        pub(crate) triggers: &'a [TriggerLit<'a>],
    }

    pub(crate) struct AfterLit<'a> {
        pub(crate) to: &'a str,
        pub(crate) rank: i32,
    }

    pub(crate) struct TriggerLit<'a> {
        pub(crate) globs: &'a [&'a str],
        pub(crate) note: &'a str,
    }

    /// The sole list-literal quoting: `[] → ""`, `["a","b"] → "\"a\", \"b\""`.
    pub(crate) fn toml_list(xs: &[&str]) -> String {
        xs.iter()
            .map(|x| format!("\"{x}\""))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// `after` array-of-inline-tables literal: each edge `{ to = "X", rank = N }`.
    pub(crate) fn toml_after(xs: &[AfterLit<'_>]) -> String {
        xs.iter()
            .map(|e| format!("{{ to = \"{}\", rank = {} }}", e.to, e.rank))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// `triggers` array-of-inline-tables literal: each `{ globs = [...], note = "" }`.
    pub(crate) fn toml_triggers(xs: &[TriggerLit<'_>]) -> String {
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
    pub(crate) fn render_fixture_toml(f: &Fixture<'_>) -> String {
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
        // SL-048 PHASE-04 (the cut): the migrated tier-1 axes (slices/specs/drift) are
        // emitted as `[[relation]]` rows AFTER the typed `[relationships]` table (F1 —
        // typed tables precede all arrays-of-tables). The dep/sequence axes stay typed.
        let rels = f.rels.as_ref().map_or_else(String::new, |x| {
            format!(
                "\n[relationships]\nneeds = [{}]\nafter = [{}]\ntriggers = [{}]\n",
                toml_list(x.needs),
                toml_after(x.after),
                toml_triggers(x.triggers),
            )
        });
        let mut relation_rows = String::new();
        if let Some(x) = f.rels.as_ref() {
            for s in x.slices {
                relation_rows.push_str(&format!(
                    "\n[[relation]]\nlabel = \"slices\"\ntarget = \"{s}\"\n"
                ));
            }
            // SL-149: a backlog item's spec-targeting edges are references(concerns)
            // (implements/scoped_from are SL-only). The `specs` fixture field name is
            // retained as a convenience; the authored row is references(concerns).
            for s in x.specs {
                relation_rows.push_str(&format!(
                    "\n[[relation]]\nlabel = \"references\"\nrole = \"concerns\"\ntarget = \"{s}\"\n"
                ));
            }
        }
        format!("{head}{facet}{rels}{relation_rows}")
    }

    /// The sole path/dir/write: render the fixture and lay it under its kind tree.
    pub(crate) fn write_fixture(root: &Path, f: Fixture<'_>) {
        let name = format!("{:03}", f.id);
        let dir = root.join(f.kind.kind().dir).join(&name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(format!("backlog-{name}.toml")),
            render_fixture_toml(&f),
        )
        .unwrap();
        fs::write(
            dir.join(format!("backlog-{name}.md")),
            format!("# {}: {}\n", f.kind.canonical_id(f.id), f.title),
        )
        .unwrap();
    }
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
            &mut entity::local_reserved(),
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
                tags: vec![],
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
        // SL-048: `slices` is no longer a typed `[relationships]` field — it migrated
        // to `[[relation]]` (read by `read_item` via `read_block`, not `validate`). A
        // stray `[relationships].slices` key in the fixture is now simply ignored on
        // parse. The tier-1 read seam is covered by the show/relation_edges tests.
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
    fn resolution_render_mirror_serde() {
        assert_eq!(Resolution::WontDo.as_str(), "wont-do");
        assert_eq!(Resolution::Promoted.as_str(), "promoted");
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

    // SL-026 PHASE-02: the fixture builder (`Fixture`/`write_fixture`/
    // `render_fixture_toml` + the `*Lit` literals + `toml_*` helpers) was promoted
    // to the `pub(crate) mod test_support` submodule above, so a later golden-corpus
    // phase can seed the same backlog TOML through `crate::backlog::test_support::*`
    // (it stays the SOLE source of the fixture literal — re-rolling it re-opens the
    // closed ISS-001 debt). The backlog suite drives it via this glob import.
    use super::test_support::*;

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

    /// Drive `list_rows` in the classic `--by id` mode and return just stdout — the
    /// shape every pre-SL-051 list test expected (membership / filter / column
    /// behaviour, asserted against the `(kind.ordinal, id)` grouping). The composed
    /// `--by sequence` default is exercised separately (VT-1 / VT-2).
    fn list_id(root: &Path, kind: Option<ItemKind>, args: ListArgs) -> anyhow::Result<String> {
        list_rows(root, kind, OrderBy::Id, args).map(|o| o.stdout)
    }

    // --- §5.5: the uniform table header (extends to backlog) ---

    #[test]
    fn backlog_list_emits_a_header_then_prefixed_ids() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_item(root, ItemKind::Issue, 1, "open", "", "a", "Alpha", &[]);

        let out = list_id(root, None, list_args()).unwrap();
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
        assert_eq!(list_id(dir.path(), None, list_args()).unwrap(), "");
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

        let out = list_id(root, None, list_args()).unwrap();
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
        let out = list_id(root, None, columns_args(&["id", "slug", "title"])).unwrap();
        let header = out.lines().next().unwrap();
        assert_eq!(
            header.split_whitespace().collect::<Vec<_>>(),
            vec!["id", "│", "slug", "│", "title"]
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

        let err = list_id(root, None, columns_args(&["bogus"]))
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

    // --- SL-067 PHASE-02: the dynamic `tags` column (D2) ---

    /// VT-4: an UNTAGGED corpus shows NO `tags` column (the golden-corpus invariant —
    /// untagged lists stay byte-identical to pre-SL-067).
    #[test]
    fn backlog_list_untagged_corpus_hides_tags_column() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_item(root, ItemKind::Issue, 1, "open", "", "a", "Alpha", &[]);

        let out = list_id(root, None, list_args()).unwrap();
        let header = out.lines().next().unwrap();
        assert!(
            !header.contains("tags"),
            "untagged corpus omits the tags column: {header:?}"
        );
    }

    /// VT-4: ≥1 tagged row → the `tags` column appears, spliced before `title`.
    #[test]
    fn backlog_list_tagged_corpus_shows_tags_before_title() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_item(root, ItemKind::Issue, 1, "open", "", "a", "Alpha", &["cli"]);

        let out = list_id(root, None, list_args()).unwrap();
        let header = out.lines().next().unwrap();
        assert_eq!(
            header.split_whitespace().collect::<Vec<_>>(),
            vec!["id", "│", "kind", "│", "status", "│", "tags", "│", "title"],
            "tags spliced before title: {header:?}"
        );
        assert!(out.contains("cli"), "the tag value renders: {out}");
    }

    /// VT-4: `--columns id,tags` FORCES the tags column even when every row is empty
    /// (the explicit request honoured verbatim — the dynamic default is bypassed).
    #[test]
    fn backlog_list_columns_forces_tags_even_when_all_empty() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_item(root, ItemKind::Issue, 1, "open", "", "a", "Alpha", &[]);

        let out = list_id(root, None, columns_args(&["id", "tags"])).unwrap();
        let header = out.lines().next().unwrap();
        assert_eq!(
            header.split_whitespace().collect::<Vec<_>>(),
            vec!["id", "│", "tags"],
            "explicit --columns shows tags despite all-empty: {header:?}"
        );
    }

    /// VT-4: `--columns` omitting tags HIDES it despite tagged rows (the explicit set
    /// wins; the dynamic default never overrides an explicit request).
    #[test]
    fn backlog_list_columns_omitting_tags_hides_it_despite_tagged_rows() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_item(root, ItemKind::Issue, 1, "open", "", "a", "Alpha", &["cli"]);

        let out = list_id(root, None, columns_args(&["id", "title"])).unwrap();
        let header = out.lines().next().unwrap();
        assert!(
            !header.contains("tags"),
            "explicit columns omitting tags hides it: {header:?}"
        );
    }

    /// VT-4: a tagged item FILTERED OUT by `--kind` leaves no tagged row in the visible
    /// set → no tags column (the visibility keys on the FINAL displayed set, post-kind).
    #[test]
    fn backlog_list_tagged_row_filtered_by_kind_hides_tags_column() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // The only tagged item is an Issue; we list Improvements → it is filtered out.
        write_item(root, ItemKind::Issue, 1, "open", "", "a", "Alpha", &["cli"]);
        write_item(
            root,
            ItemKind::Improvement,
            1,
            "open",
            "",
            "b",
            "Bravo",
            &[],
        );

        let out = list_id(root, Some(ItemKind::Improvement), list_args()).unwrap();
        let header = out.lines().next().unwrap();
        assert!(
            !header.contains("tags"),
            "no visible tagged row after --kind → no tags column: {header:?}"
        );
    }

    /// VT-1 (backlog wiring smoke): under colour the tagged cell carries ANSI and
    /// stripping it reproduces the plain render — the PerToken column is wired and
    /// byte-clean-coupled end to end.
    #[test]
    fn backlog_list_tags_column_colour_strips_to_plain() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_item(
            root,
            ItemKind::Issue,
            1,
            "open",
            "",
            "a",
            "Alpha",
            &["cli:command", "security"],
        );

        let plain = list_id(root, None, list_args()).unwrap();
        let coloured = list_id(
            root,
            None,
            ListArgs {
                render: listing::RenderOpts {
                    color: true,
                    term_width: None,
                },
                ..Default::default()
            },
        )
        .unwrap();
        assert!(
            coloured.contains('\u{1b}'),
            "the tagged cell carries ANSI under colour"
        );
        assert_eq!(
            crate::listing::strip_ansi(&coloured),
            plain,
            "stripping the coloured backlog render reproduces the plain layout"
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

        let out = list_id(root, None, list_args()).unwrap();
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
        let all = list_id(
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
        let resolved = list_id(
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
        let out = list_id(
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

        let out = list_id(root, None, list_args()).unwrap();
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

        let out = list_id(root, None, list_args()).unwrap();
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
        let out = list_id(root, None, list_args()).unwrap();
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
        let out = list_id(
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

        let json = list_id(
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
        let err = list_id(
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
        let json = show_json(&item, true).unwrap();
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

    #[test]
    fn backlog_show_json_groups_references_by_role_and_keeps_legacy_keys() {
        // SL-149 PHASE-04b: a backlog item authoring a `concerns` reference edge (plus a
        // legacy `slices` edge) → the JSON carries a `references` object grouped by role
        // (only `concerns` populated — implements/scoped_from are SL-only), and the
        // legacy `slices` key still carries its data unchanged (additive).
        use crate::relation::{RelationEdge, RelationLabel, Role};
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        new_item(root, ItemKind::Improvement, "Token expiry");
        let mut item = read_item(root, ItemKind::Improvement, 1).unwrap();
        item.tier1 = vec![
            RelationEdge::new(RelationLabel::Slices, "SL-020".into()),
            RelationEdge::with_role(
                RelationLabel::References,
                Some(Role::Concerns),
                "SPEC-018".into(),
            ),
        ];
        let json = show_json(&item, true).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let rel = &v["backlog"]["relationships"];
        assert_eq!(
            rel["references"]["concerns"],
            serde_json::json!(["SPEC-018"])
        );
        assert_eq!(rel["references"]["implements"], serde_json::json!([]));
        assert_eq!(rel["references"]["scoped_from"], serde_json::json!([]));
        // legacy key unchanged
        assert_eq!(rel["slices"], serde_json::json!(["SL-020"]));
    }

    #[test]
    fn backlog_show_json_references_object_carries_concerns() {
        // SL-149 PHASE-05: a backlog item authors `references(concerns)` (the migration
        // target of the old backlog `specs`→canon edge); the legacy `specs` key is gone.
        // implements/scoped_from stay empty (SL-only roles).
        use crate::relation::{RelationEdge, RelationLabel, Role};
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        new_item(root, ItemKind::Improvement, "Concerns only");
        let mut item = read_item(root, ItemKind::Improvement, 1).unwrap();
        item.tier1 = vec![
            RelationEdge::new(RelationLabel::Slices, "SL-007".into()),
            RelationEdge::with_role(
                RelationLabel::References,
                Some(Role::Concerns),
                "SPEC-018".into(),
            ),
        ];
        let json = show_json(&item, true).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let rel = &v["backlog"]["relationships"];
        assert_eq!(rel["slices"], serde_json::json!(["SL-007"]));
        assert!(rel.get("specs").is_none(), "legacy specs key removed");
        assert_eq!(rel["references"]["implements"], serde_json::json!([]));
        assert_eq!(rel["references"]["scoped_from"], serde_json::json!([]));
        assert_eq!(
            rel["references"]["concerns"],
            serde_json::json!(["SPEC-018"])
        );
    }

    #[test]
    fn backlog_inspect_json_omits_body() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_fixture(
            root,
            Fixture {
                kind: ItemKind::Improvement,
                id: 1,
                slug: "token",
                title: "Token expiry",
                status: "open",
                resolution: "",
                tags: &[],
                facet: None,
                rels: None,
            },
        );
        let item = read_item(root, ItemKind::Improvement, 1).unwrap();
        let json = show_json(&item, false).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(
            v["backlog"].get("body").is_none(),
            "inspect JSON must not include body"
        );
    }

    // --- PHASE-04: the `backlog show <ID>` verb (id parse + render) ---

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
        assert!(
            out.contains("references(concerns): PRD-009"),
            "outbound spec ref shown as references(concerns)"
        );

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
        let json = show_json(&item, true).unwrap();
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
    fn run_needs_accepts_cross_kind_slice_prereq() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        new_item(root, ItemKind::Issue, "Auth"); // ISS-001
        // Create a minimal slice entity dir so ensure_ref_resolves passes.
        let sl_dir = root.join(".doctrine/slice/001");
        fs::create_dir_all(&sl_dir).unwrap();

        run_needs(Some(root.to_path_buf()), "ISS-001", &["SL-001".to_string()]).unwrap();

        let item = read_item(root, ItemKind::Issue, 1).unwrap();
        assert_eq!(item.relationships.needs, vec!["SL-001"]);
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

        run_after(
            Some(root.to_path_buf()),
            "ISS-001",
            Some("ISS-002"),
            0,
            false,
            false,
        )
        .unwrap();

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
        run_after(
            Some(root.to_path_buf()),
            "ISS-002",
            Some("ISS-001"),
            5,
            false,
            false,
        )
        .unwrap();
        let item = read_item(root, ItemKind::Issue, 2).unwrap();
        assert_eq!(
            item.relationships.after,
            vec![AfterEdge {
                to: "ISS-001".to_string(),
                rank: 5,
            }]
        );
    }

    // --- SL-067 PHASE-01: the `backlog tag` verb + the normalise/filter folds ---

    fn item_path(root: &Path, kind: ItemKind, id: u32) -> PathBuf {
        let name = format!("{id:03}");
        root.join(kind.kind().dir)
            .join(&name)
            .join(format!("backlog-{name}.toml"))
    }

    fn s(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|x| (*x).to_string()).collect()
    }

    /// VT-1: round-trip e2e — add surfaces via the (folded) `-t` filter, remove drops it.
    #[test]
    fn run_tag_round_trips_add_filter_remove() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        new_item(root, ItemKind::Issue, "Auth"); // ISS-001, tags = []

        run_tag(Some(root.to_path_buf()), "ISS-001", &s(&["a", "b"]), &[]).unwrap();
        assert_eq!(
            read_item(root, ItemKind::Issue, 1).unwrap().tags,
            s(&["a", "b"])
        );

        // surfaces under the tag filter (input folded → exact-match the store).
        let json = list_id(
            root,
            None,
            ListArgs {
                json: true,
                tags: s(&["a"]),
                ..ListArgs::default()
            },
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            v["rows"].as_array().unwrap().len(),
            1,
            "tag filter matches: {json}"
        );

        run_tag(Some(root.to_path_buf()), "ISS-001", &[], &s(&["a"])).unwrap();
        assert_eq!(read_item(root, ItemKind::Issue, 1).unwrap().tags, s(&["b"]));
    }

    /// VT-2: normalisation — case-fold, charset reject naming the token, colon accepted.
    #[test]
    fn run_tag_normalises_and_rejects_bad_charset() {
        assert_eq!(normalize_tag("Security").unwrap(), "security");
        assert_eq!(normalize_tag("  Area:Backlog ").unwrap(), "area:backlog");
        // colon namespacing accepted; underscore/hyphen/digits accepted.
        assert_eq!(normalize_tag("a_b-1:c").unwrap(), "a_b-1:c");

        for bad in ["a b", "a@b"] {
            let err = normalize_tag(bad).unwrap_err().to_string();
            assert!(
                err.contains(bad),
                "the reject names the offending token: {err}"
            );
        }
        assert!(
            normalize_tag("   ").is_err(),
            "empty-after-trim is rejected"
        );

        // The verb routes adds through the chokepoint — a bad add hard-errors.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        new_item(root, ItemKind::Issue, "Auth");
        let path = item_path(root, ItemKind::Issue, 1);
        let before = fs::read_to_string(&path).unwrap();
        assert!(run_tag(Some(root.to_path_buf()), "ISS-001", &s(&["a@b"]), &[]).is_err());
        assert_eq!(
            before,
            fs::read_to_string(&path).unwrap(),
            "rejected before any write"
        );

        // A `Security` add lands lowercased.
        run_tag(Some(root.to_path_buf()), "ISS-001", &s(&["Security"]), &[]).unwrap();
        assert_eq!(
            read_item(root, ItemKind::Issue, 1).unwrap().tags,
            s(&["security"])
        );
    }

    /// VT-3: idempotency — re-add present / remove absent are no-ops (mtime unchanged,
    /// proven against an UNSORTED hand store); add∩remove overlap is rejected.
    #[test]
    fn run_tag_idempotent_no_op_holds_mtime_on_unsorted_store() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // Hand-author an UNSORTED store: tags = ["b", "a"]. The set is already {a,b}.
        fs::create_dir_all(item_path(root, ItemKind::Issue, 1).parent().unwrap()).unwrap();
        let toml = "id = 1\nslug = \"a\"\ntitle = \"A\"\nkind = \"issue\"\n\
             status = \"open\"\nresolution = \"\"\ncreated = \"2026-06-08\"\n\
             updated = \"2026-06-08\"\ntags = [\"b\", \"a\"]\n";
        let path = item_path(root, ItemKind::Issue, 1);
        fs::write(&path, toml).unwrap();
        // .md companion now required by read_item (ISS-050).
        fs::write(
            path.parent().unwrap().join(format!("backlog-{:03}.md", 1)),
            "# ISS-001: A\n",
        )
        .unwrap();
        let before = fs::read_to_string(&path).unwrap();
        let mtime0 = fs::metadata(&path).unwrap().modified().unwrap();

        // Re-add a present tag (set already {a,b}) — set-compare no-op, NO write+stamp.
        run_tag(Some(root.to_path_buf()), "ISS-001", &s(&["a"]), &[]).unwrap();
        assert_eq!(
            before,
            fs::read_to_string(&path).unwrap(),
            "no-op: content held"
        );
        assert_eq!(
            mtime0,
            fs::metadata(&path).unwrap().modified().unwrap(),
            "mtime held"
        );

        // Remove an absent tag — also a no-op.
        run_tag(Some(root.to_path_buf()), "ISS-001", &[], &s(&["zzz"])).unwrap();
        assert_eq!(
            before,
            fs::read_to_string(&path).unwrap(),
            "remove-absent no-op"
        );

        // add∩remove overlap (after normalisation) is rejected, nothing written.
        let err = run_tag(Some(root.to_path_buf()), "ISS-001", &s(&["X"]), &s(&["x"]));
        assert!(err.is_err(), "an add∩remove overlap is rejected");
        assert_eq!(
            before,
            fs::read_to_string(&path).unwrap(),
            "nothing written on reject"
        );
    }

    /// VT-3 (no-input): neither an add nor a remove is a hard error.
    #[test]
    fn run_tag_requires_at_least_one_edit() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        new_item(root, ItemKind::Issue, "Auth");
        assert!(run_tag(Some(root.to_path_buf()), "ISS-001", &[], &[]).is_err());
    }

    /// VT-4: `list --json` — untagged emits `[]`, tagged emits its array unconditionally.
    #[test]
    fn run_tag_json_projects_tags_unconditionally() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_item(root, ItemKind::Issue, 1, "open", "", "a", "Alpha", &[]); // untagged
        write_item(
            root,
            ItemKind::Issue,
            2,
            "open",
            "",
            "b",
            "Bravo",
            &["security"],
        );

        let json = list_id(
            root,
            None,
            ListArgs {
                json: true,
                ..ListArgs::default()
            },
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let rows = v["rows"].as_array().unwrap();
        let by_id = |id: &str| {
            rows.iter()
                .find(|r| r["id"] == id)
                .unwrap_or_else(|| panic!("row {id}"))
        };
        // untagged → empty array (present, not omitted, never gated).
        assert_eq!(by_id("ISS-001")["tags"], serde_json::json!([]));
        assert_eq!(by_id("ISS-002")["tags"], serde_json::json!(["security"]));
    }

    /// VT-5: edit-preserving — a comment / inert table / unknown key survive; `updated`
    /// stamped; unrelated keys untouched. And an F-1 missing-`tags` file is refused
    /// byte-unchanged.
    #[test]
    fn run_tag_is_edit_preserving_and_refuses_missing_tags() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(item_path(root, ItemKind::Issue, 1).parent().unwrap()).unwrap();
        let path = item_path(root, ItemKind::Issue, 1);
        // A hand comment, an inert `[relationships]` table, an unknown key.
        let toml = "# keep me\nid = 1\nslug = \"a\"\ntitle = \"A\"\nkind = \"issue\"\n\
             status = \"open\"\nresolution = \"\"\ncreated = \"2026-06-08\"\n\
             updated = \"2026-06-08\"\ntags = []\nunknown = \"survives\"\n\
             \n[relationships]\nneeds = []\n";
        fs::write(&path, toml).unwrap();
        // .md companion now required by read_item (ISS-050).
        fs::write(
            path.parent().unwrap().join(format!("backlog-{:03}.md", 1)),
            "# ISS-001: A\n",
        )
        .unwrap();

        run_tag(Some(root.to_path_buf()), "ISS-001", &s(&["security"]), &[]).unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("# keep me"), "comment survives: {after}");
        assert!(
            after.contains("unknown = \"survives\""),
            "unknown key survives"
        );
        assert!(after.contains("[relationships]"), "inert table survives");
        assert!(
            after.contains("tags = [\"security\"]"),
            "tag written: {after}"
        );
        assert!(
            !after.contains("updated = \"2026-06-08\""),
            "updated stamped"
        );

        // Self-heal: a file with NO `tags` key gets tags = ["x"] seeded (SL-136).
        fs::create_dir_all(item_path(root, ItemKind::Issue, 2).parent().unwrap()).unwrap();
        let path2 = item_path(root, ItemKind::Issue, 2);
        let no_tags = "id = 2\nslug = \"b\"\ntitle = \"B\"\nkind = \"issue\"\n\
             status = \"open\"\nresolution = \"\"\ncreated = \"2026-06-08\"\n\
             updated = \"2026-06-08\"\n";
        fs::write(&path2, no_tags).unwrap();
        fs::write(
            path2.parent().unwrap().join(format!("backlog-{:03}.md", 2)),
            "# ISS-002: B\n",
        )
        .unwrap();
        run_tag(Some(root.to_path_buf()), "ISS-002", &s(&["x"]), &[]).unwrap();
        let after2 = fs::read_to_string(&path2).unwrap();
        assert!(
            after2.contains("tags = [\"x\"]"),
            "self-heal seeds tags and writes: {after2}"
        );
    }

    /// EX-5: the filter fold is LENIENT — a mixed-case / surrounding-space input never
    /// errors and round-trips the store; a no-match input succeeds silently.
    #[test]
    fn filter_fold_is_lenient_and_distinct_from_write_normalise() {
        assert_eq!(tag::fold_filter_tag("  Security "), "security");
        // The lenient fold accepts what the write chokepoint rejects (no bail).
        assert_eq!(tag::fold_filter_tag("a b"), "a b");

        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_item(
            root,
            ItemKind::Issue,
            1,
            "open",
            "",
            "a",
            "Alpha",
            &["security"],
        );
        // `-t Security` (mixed case) folds to `security` and matches the store.
        let hit = list_id(
            root,
            None,
            ListArgs {
                json: true,
                tags: s(&["Security"]),
                ..ListArgs::default()
            },
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&hit).unwrap();
        assert_eq!(
            v["rows"].as_array().unwrap().len(),
            1,
            "case-folded filter hits"
        );
        // A no-match input succeeds silently (zero rows, no error).
        let miss = list_id(
            root,
            None,
            ListArgs {
                json: true,
                tags: s(&["nomatch at all"]),
                ..ListArgs::default()
            },
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&miss).unwrap();
        assert_eq!(
            v["rows"].as_array().unwrap().len(),
            0,
            "no-match filter is silent"
        );
    }

    // --- SL-051: the composed `backlog list --by sequence` (the folded-in order) ---

    /// Drive `list_rows` in the default `--by sequence` mode, returning `(stdout,
    /// stderr)` — the SL-051 tuple shape (rows + footer on stdout, the cycle advisory
    /// on stderr).
    fn list_seq(root: &Path, args: ListArgs) -> (String, String) {
        let out = list_rows(root, None, OrderBy::Sequence, args).unwrap();
        (out.stdout, out.stderr)
    }

    /// The composed-order ids from a `--by sequence` stdout (before the `overrides:`
    /// honest-record footer). Reuses [`ids`] over just the table half.
    fn seq_ids(out: &str) -> Vec<String> {
        let table = out.split("\noverrides:").next().unwrap_or(out);
        ids(table)
    }

    #[test]
    fn list_sequence_composes_a_hard_needs_order() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // ISS-001 needs ISS-002 ⇒ ISS-002 must precede ISS-001 in the order.
        write_rel_item(root, ItemKind::Issue, 1, "open", &["ISS-002"], &[]);
        write_rel_item(root, ItemKind::Issue, 2, "open", &[], &[]);

        let (out, err) = list_seq(root, list_args());
        assert_eq!(
            seq_ids(&out),
            vec!["ISS-002", "ISS-001"],
            "B precedes A: {out}"
        );
        assert!(!out.contains("overrides:"), "no drops, no footer: {out}");
        assert!(err.is_empty(), "no advisory on a clean compose: {err:?}");
    }

    // --- VT-1: --by sequence vs --by id share membership; differ on order ---

    #[test]
    fn list_sequence_and_id_share_membership_differ_on_order() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // ISS-001 needs ISS-002 — the sequence flips them; the id sort does not.
        write_rel_item(root, ItemKind::Issue, 1, "open", &["ISS-002"], &[]);
        write_rel_item(root, ItemKind::Issue, 2, "open", &[], &[]);
        write_rel_item(root, ItemKind::Issue, 3, "open", &[], &[]);

        let (seq, _) = list_seq(root, list_args());
        let by_id = list_id(root, None, list_args()).unwrap();

        // default sequence: the prerequisite (ISS-002) precedes its dependent (ISS-001);
        // the unconstrained ISS-003 sits where the tie-break (id asc) places it — the
        // key point is 002-before-001, which the plain id sort would NOT produce.
        let seq_order = seq_ids(&seq);
        let pos = |id: &str| seq_order.iter().position(|x| x == id).unwrap();
        assert!(
            pos("ISS-002") < pos("ISS-001"),
            "needs flips 002 ahead of its dependent 001: {seq}"
        );
        // classic id sort: ascending id, unaffected by the dependency.
        assert_eq!(
            ids(&by_id),
            vec!["ISS-001", "ISS-002", "ISS-003"],
            "--by id is plain ascending: {by_id}"
        );
        // A-2: the two orderings are PERMUTATIONS — identical membership sets.
        let mut a = seq_ids(&seq);
        let mut b = ids(&by_id);
        a.sort();
        b.sort();
        assert_eq!(a, b, "sequence and id list the same items, reordered");
    }

    // --- VT-2: a `needs` cycle degrades to the id sort with a stderr advisory ---

    #[test]
    fn compose_degrades_on_a_needs_cycle() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // a mutual needs cycle {ISS-001, ISS-002}.
        write_rel_item(root, ItemKind::Issue, 1, "open", &["ISS-002"], &[]);
        write_rel_item(root, ItemKind::Issue, 2, "open", &["ISS-001"], &[]);

        let corpus = read_all(root).unwrap();
        let Ordering::Degraded { warning, .. } = compose(&corpus).unwrap() else {
            panic!("a needs cycle degrades to Ordering::Degraded");
        };
        assert!(warning.contains("cycle"), "names the failure: {warning}");
        assert!(
            warning.contains("ISS-001") && warning.contains("ISS-002"),
            "names members: {warning}"
        );

        // end to end: `list --by sequence` falls back to the id sort, EXITS 0 (no
        // error), and routes the advisory to stderr — never an empty / misleading list.
        let (out, err) = list_seq(root, list_args());
        assert_eq!(
            ids(&out),
            vec!["ISS-001", "ISS-002"],
            "degrade falls back to the id sort, never empty: {out}"
        );
        assert!(err.contains("cycle"), "the advisory is on stderr: {err}");
    }

    #[test]
    fn list_sequence_evicts_the_lower_rank_edge_of_a_soft_cycle() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // VT-6: X.after=[{to=Y,rank=1}], Y.after=[{to=X,rank=5}] ⇒ the strictly
        // lower-rank edge is evicted. The order is still produced; the eviction is
        // recorded in the stdout footer.
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

        let (out, _) = list_seq(root, list_args());
        // both nodes still ordered (the cycle was linearized, not refused).
        let mut shown = seq_ids(&out);
        shown.sort();
        assert_eq!(shown, vec!["ISS-001", "ISS-002"]);
        // exactly the soft-cycle eviction is recorded in the footer.
        assert!(
            out.contains("overrides:"),
            "the eviction is recorded: {out}"
        );
        assert!(out.contains("soft cycle"), "named a soft-cycle drop: {out}");
    }

    #[test]
    fn list_sequence_records_terminal_and_absent_drops_with_status_and_resolution() {
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

        let (out, _) = list_seq(root, list_args());
        // the live node still orders.
        assert_eq!(
            seq_ids(&out),
            vec!["ISS-001"],
            "the live node survives: {out}"
        );
        assert!(out.contains("overrides:"));
        // IDE-019: the terminal dep is suppressed by default (stale; no action
        // needed on resolved work). Only the truly-absent ref surfaces.
        assert!(
            !out.contains("CHR-001"),
            "terminal dep suppressed by default: {out}"
        );
        // the absent ref is still named absent.
        assert!(
            out.contains("ISS-099") && out.contains("absent"),
            "absent ref named: {out}"
        );
    }

    // --- PHASE-04 paths verb golden tests ---

    /// Scaffold one backlog item and write an extra file into its entity dir.
    fn backlog_fixture(root: &Path, item_kind: ItemKind, id: u32, extra: &[&str]) {
        let name = format!("{id:03}");
        let dir = root.join(item_kind.kind().dir).join(&name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(format!("{BACKLOG_STEM}-{name}.toml")), "toml").unwrap();
        fs::write(dir.join(format!("{BACKLOG_STEM}-{name}.md")), "md").unwrap();
        for e in extra {
            fs::write(dir.join(e), e).unwrap();
        }
    }

    #[test]
    fn paths_full_shows_toml_md_and_extras_in_canonical_order() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        backlog_fixture(root, ItemKind::Issue, 1, &["notes.md", "z.log"]);
        let sel = crate::paths::PathSelection {
            toml: false,
            md: false,
            entity: false,
            single: false,
        };
        let entity_dir = root.join(ItemKind::Issue.kind().dir).join("001");
        let identity_toml = entity_dir.join("backlog-001.toml");
        let identity_md = entity_dir.join("backlog-001.md");
        let set =
            crate::paths::scan_entity_dir(&entity_dir, &identity_toml, Some(&identity_md), root)
                .unwrap();
        let lines = crate::paths::select_paths(&set, &sel).unwrap();
        let output = lines.join("\n");
        assert!(output.contains(".doctrine/backlog/issue/001/backlog-001.toml"));
        assert!(output.contains(".doctrine/backlog/issue/001/backlog-001.md"));
        assert!(output.contains(".doctrine/backlog/issue/001/notes.md"));
        assert!(output.contains(".doctrine/backlog/issue/001/z.log"));
    }

    #[test]
    fn paths_single_truncates_to_first() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        backlog_fixture(root, ItemKind::Issue, 1, &["notes.md"]);
        let sel = crate::paths::PathSelection {
            toml: false,
            md: false,
            entity: false,
            single: true,
        };
        let entity_dir = root.join(ItemKind::Issue.kind().dir).join("001");
        let identity_toml = entity_dir.join("backlog-001.toml");
        let identity_md = entity_dir.join("backlog-001.md");
        let set =
            crate::paths::scan_entity_dir(&entity_dir, &identity_toml, Some(&identity_md), root)
                .unwrap();
        let lines = crate::paths::select_paths(&set, &sel).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], ".doctrine/backlog/issue/001/backlog-001.toml");
    }

    #[test]
    fn paths_toml_only() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        backlog_fixture(root, ItemKind::Chore, 2, &["notes.md"]);
        let sel = crate::paths::PathSelection {
            toml: true,
            md: false,
            entity: false,
            single: false,
        };
        let entity_dir = root.join(ItemKind::Chore.kind().dir).join("002");
        let identity_toml = entity_dir.join("backlog-002.toml");
        let identity_md = entity_dir.join("backlog-002.md");
        let set =
            crate::paths::scan_entity_dir(&entity_dir, &identity_toml, Some(&identity_md), root)
                .unwrap();
        let lines = crate::paths::select_paths(&set, &sel).unwrap();
        assert_eq!(lines, vec![".doctrine/backlog/chore/002/backlog-002.toml"]);
    }

    #[test]
    fn paths_md_only() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        backlog_fixture(root, ItemKind::Risk, 3, &[]);
        let sel = crate::paths::PathSelection {
            toml: false,
            md: true,
            entity: false,
            single: false,
        };
        let entity_dir = root.join(ItemKind::Risk.kind().dir).join("003");
        let identity_toml = entity_dir.join("backlog-003.toml");
        let identity_md = entity_dir.join("backlog-003.md");
        let set =
            crate::paths::scan_entity_dir(&entity_dir, &identity_toml, Some(&identity_md), root)
                .unwrap();
        let lines = crate::paths::select_paths(&set, &sel).unwrap();
        assert_eq!(lines, vec![".doctrine/backlog/risk/003/backlog-003.md"]);
    }

    #[test]
    fn paths_entity_gives_toml_and_md() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        backlog_fixture(root, ItemKind::Idea, 4, &["extra.txt"]);
        let sel = crate::paths::PathSelection {
            toml: false,
            md: false,
            entity: true,
            single: false,
        };
        let entity_dir = root.join(ItemKind::Idea.kind().dir).join("004");
        let identity_toml = entity_dir.join("backlog-004.toml");
        let identity_md = entity_dir.join("backlog-004.md");
        let set =
            crate::paths::scan_entity_dir(&entity_dir, &identity_toml, Some(&identity_md), root)
                .unwrap();
        let lines = crate::paths::select_paths(&set, &sel).unwrap();
        assert_eq!(
            lines,
            vec![
                ".doctrine/backlog/idea/004/backlog-004.toml",
                ".doctrine/backlog/idea/004/backlog-004.md"
            ]
        );
    }

    #[test]
    fn paths_invalid_ref_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        backlog_fixture(root, ItemKind::Issue, 1, &[]);
        let result = parse_ref("ISS-99999");
        assert!(result.is_ok()); // parses fine, but entity dir doesn't exist
        let entity_dir = root.join(ItemKind::Issue.kind().dir).join("99999");
        let identity_toml = entity_dir.join("backlog-99999.toml");
        let identity_md = entity_dir.join("backlog-99999.md");
        let scan =
            crate::paths::scan_entity_dir(&entity_dir, &identity_toml, Some(&identity_md), root);
        assert!(scan.is_err());
    }

    #[test]
    fn paths_multi_ref_splat_preserves_order() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        backlog_fixture(root, ItemKind::Issue, 1, &[]);
        backlog_fixture(root, ItemKind::Improvement, 1, &[]);
        let sel = crate::paths::PathSelection {
            toml: false,
            md: false,
            entity: false,
            single: false,
        };
        let mut all_lines: Vec<String> = Vec::new();
        for (kind, n) in [(ItemKind::Issue, "001"), (ItemKind::Improvement, "001")] {
            let entity_dir = root.join(kind.kind().dir).join(n);
            let toml_name = format!("{BACKLOG_STEM}-{n}.toml");
            let md_name = format!("{BACKLOG_STEM}-{n}.md");
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
        assert!(all_lines[0].contains("issue/001/backlog-001.toml"));
        assert!(all_lines[2].contains("improvement/001/backlog-001.toml"));
    }
}
