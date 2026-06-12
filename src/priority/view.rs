// SPDX-License-Identifier: GPL-3.0-only
//! The priority VIEW layer (SL-047 §5.4) — the structured reasons that are the
//! render SOURCE OF TRUTH (REQ-072 AC3).
//!
//! These types carry the COMPUTED classification of each surfaced node: its
//! actionability, its consequence, its direct blockers, and a `Vec<ReasonKind>` of
//! the structured reasons behind the verdict. The human table and the `--json`
//! output ([`super::render`]) are produced *from* these types — never recomputed in
//! the renderer. A reason is built ONCE, here (or in the surface shell that fills
//! these rows from the pure [`super::channels`] signals), so the two render targets
//! cannot drift.
//!
//! Pure data: no clock, RNG, or disk. The surface shell ([`super::surface`]) reads
//! the graph + titles and fills these rows; the renderer only formats them.

use super::partition::StatusClass;
use crate::backlog_order::OverrideReason;

/// One structured reason behind a node's classification (design §5.4). The render
/// SOURCE OF TRUTH — every human line and `--json` reason field is produced from a
/// `ReasonKind`, never recomputed (REQ-072 AC3). Refs are canonical `KIND-NNN`
/// strings (the opaque cordage ids never escape — re-mapped in the surface shell).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReasonKind {
    /// The node's eligibility verdict: its authored status + the class it landed in
    /// (`Workable` ⇒ eligible; `Terminal`/`Unrecognised` ⇒ not). `status` is `None`
    /// for the status-less REC kind.
    Eligibility {
        status: Option<String>,
        class: StatusClass,
    },
    /// The node is blocked by these (non-terminal) prerequisites (direct, or the
    /// transitive chain for `explain`/`--transitive`).
    BlockedBy { items: Vec<String> },
    /// The node is blocking these dependents (direct, or transitive).
    Blocking { items: Vec<String> },
    /// The node's consequence tally — the inbound work/lineage reference count.
    Consequence { inbound: u32 },
    /// The node's order-key contributors: its dep-topology level and its seq rank
    /// (the soft-sequence tiebreak), `None` when no `after` edge constrains it.
    OrderContrib {
        dep_level: u32,
        seq_rank: Option<i32>,
    },
    /// A soft `after` edge cordage evicted to linearize — the honest record
    /// (`from → to`, with the cordage reason re-expressed in the shared vocabulary).
    EvictedEdge {
        from: String,
        to: String,
        reason: OverrideReason,
    },
    /// The node sits in a diagnosed dep cycle — its order degraded to the fallback
    /// rather than a false topological order (REQ-076 / F2).
    CycleDegraded { nodes: Vec<String> },
    /// No structured reason applies (e.g. a promoted backlog item excluded by its own
    /// reason carries this where eligibility is moot) — the explicit empty signal.
    /// Part of the design §5.4 reason vocabulary; no surface emits it in v1 (every
    /// classification carries a concrete reason), but it renders (`render::reason_*`)
    /// so the vocabulary is complete and a future reason can adopt it.
    #[expect(
        dead_code,
        reason = "design §5.4 reason vocabulary completeness; no v1 surface emits Fallback \
                  (every classification carries a concrete reason) but it renders"
    )]
    Fallback,
}

/// Whether an eligible node is ready to start now, or held by a blocker (design
/// §5.4). EVERY survey row is eligible; the variant splits actionable from blocked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Actionability {
    /// Eligible AND unblocked — ready to start.
    Actionable,
    /// Eligible but held by at least one non-terminal direct blocker.
    Blocked,
}

impl Actionability {
    /// The badge word for a row (`""` for actionable, `"BLOCKED"` for blocked) — the
    /// single source so the human render stays consistent.
    pub(crate) fn badge(self) -> &'static str {
        match self {
            Actionability::Actionable => "",
            Actionability::Blocked => "BLOCKED",
        }
    }

    /// The JSON token for the actionability axis.
    pub(crate) fn token(self) -> &'static str {
        match self {
            Actionability::Actionable => "actionable",
            Actionability::Blocked => "blocked",
        }
    }
}

/// One `survey` row (design §5.4) — an eligible node with its importance signals and
/// structured reasons. The set is all eligible nodes (terminal excluded unless
/// `--all`); both [`Actionability`] variants appear (the divergence feature — a
/// blocked-but-workable item still leads importance order, D10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SurveyRow {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) kind: String,
    pub(crate) status: String,
    pub(crate) act: Actionability,
    pub(crate) consequence: u32,
    /// Direct blockers (canonical refs) — empty for an actionable row.
    pub(crate) blockers: Vec<String>,
    pub(crate) reasons: Vec<ReasonKind>,
}

/// One `next` row (design §5.4) — an ACTIONABLE node only (blocked items are absent,
/// the divergence feature). Ordered by `order_key` (D9). Carries its blocking set
/// (what it unblocks) for the advisory display; blockers is empty by construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NextRow {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) kind: String,
    pub(crate) status: String,
    pub(crate) act: Actionability,
    pub(crate) reasons: Vec<ReasonKind>,
    pub(crate) blockers: Vec<String>,
    pub(crate) blocking: Vec<String>,
}

/// The `blockers <ID>` result (design §5.4 / REQ-073) — the node's direct (or
/// `--transitive`) blocked-by set and blocking set, in canonical refs. Display depth
/// (`transitive`) is a presentation flag carried for the renderer; it NEVER reorders
/// (both lists are canonical-id sorted regardless).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockersView {
    pub(crate) id: String,
    pub(crate) transitive: bool,
    pub(crate) blocked_by: Vec<String>,
    pub(crate) blocking: Vec<String>,
}

/// The `inspect` actionability block (design §5.4 / SL-046 D1) — appended below the
/// relation view at the command layer. Carries the eligible/actionable flags, the
/// direct blockers + blocking, and the consequence; rendered as a trailing block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ActionabilityBlock {
    pub(crate) eligible: bool,
    pub(crate) actionable: bool,
    pub(crate) blockers: Vec<String>,
    pub(crate) blocking: Vec<String>,
    pub(crate) consequence: u32,
}

/// The `explain <ID>` result (design §5.4 / D11) — always walked to root: the
/// eligibility reason, the transitive blocker chain, the order-key contributors, the
/// evicted seq edges, and the consequence. Each field is a structured reason (or a
/// list of them) so the renderer only formats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Explanation {
    pub(crate) id: String,
    pub(crate) eligibility: ReasonKind,
    pub(crate) blocker_chain: Vec<ReasonKind>,
    pub(crate) order_contrib: ReasonKind,
    pub(crate) evictions: Vec<ReasonKind>,
    pub(crate) consequence: ReasonKind,
}
