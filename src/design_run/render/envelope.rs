// SPDX-License-Identifier: GPL-3.0-only
//! [`TurnEnvelope`] — the canonical read model, and the budgeted projection that
//! builds it (SL-233 PHASE-04, projection-bounds sketch rev 8 §§(b)(c)(e)(g)).
//!
//! # Home (ADR-001), and why it is *here* rather than one level up
//!
//! This module is a **descendant of [`super`]**, which is the whole reason it
//! can be written at all: the `ENVELOPE_*` caps are private to the rendering
//! module, and Rust privacy lets a descendant see an ancestor's private items.
//! Assembling the projection anywhere else — at the `design_run` root, or in the
//! command shell — would mean widening those constants, and the widening is
//! exactly what EX-16(c) forbids. So the rule reads forward as well as backward:
//! *the layer that may name a bound is the layer that applies it*, and the shell
//! receives an already-bounded envelope rather than bounds to apply itself.
//!
//! Nothing here reads a clock, an rng, a disk or git. The projection is a pure
//! function of the snapshot plus the caller's declared baseline.
//!
//! # One type, three renderings (DEC-064)
//!
//! [`prompt`], [`status`] and [`resume`] are three *renderings* of one
//! [`TurnEnvelope`]; `json` is the same value through serde in the shell. There
//! is deliberately no second model: three models is how "what the agent sees"
//! and "what the human sees" start disagreeing about what the run is.
//!
//! **Only the prompt rendering is budgeted** (sketch § *The budgeted
//! rendering*). `json`'s framing overhead differs and `status` is for a human at
//! a terminal, so neither is what R1 is about.
//!
//! # The ceiling is enforced, not predicted
//!
//! Per-field caps alone do not bound the whole: individually legal fields can
//! collectively exceed the budget. So [`project`] renders, measures, and evicts
//! one entry at a time along a fixed ladder until it fits — and if the *no-drop
//! set alone* exceeds the ceiling it refuses rather than emitting a quietly
//! malformed envelope. That terminal state is the only irreducible one, by
//! construction, because every bounded list is on the ladder.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::super::attestation::ActKind;
use super::super::ids::DesignId;
use super::super::inquiry::{Disposition, InquiryLifecycle, InquiryNode};
use super::super::refusal::Refusal;
use super::super::snapshot::DesignSnapshot;
use super::super::traversal::{Authority, Posture};
use super::{
    ENVELOPE_ACTIVE_PATH_DEPTH, ENVELOPE_BLOCKERS, ENVELOPE_CHANGE_ROWS,
    ENVELOPE_DECLARATION_EXAMPLE_BYTES, ENVELOPE_DURABLE_RECORDS, ENVELOPE_FRONTIER_NODES,
    ENVELOPE_LABEL_BYTES, ENVELOPE_NORMAL_BUDGET_BYTES, ENVELOPE_QUESTION_BYTES,
    ENVELOPE_REASON_BYTES, ENVELOPE_SECTION_ROWS, FIELD_SEPARATOR, change_row, elide,
};

/// The schema discriminator the JSON rendering carries, so a reader can tell a
/// turn envelope from the snapshot it projects.
const TURN_ENVELOPE_SCHEMA: &str = "doctrine.design-turn";
/// The turn envelope's wire version.
const TURN_ENVELOPE_VERSION: u32 = 1;

/// The worked next-mutation example — the contract in one line a caller can copy.
///
/// In the **no-drop set**: half a contract is worse than none. It is therefore
/// never elided, and the compile-time assertion below is the refusal EX-4's
/// no-truncate rule calls for, moved to build time — an oversized example is an
/// authoring defect in this constant, not something a projection should clip.
const DECLARATION_EXAMPLE: &str = concat!(
    r#"{"run_uid":"<uid>","known_revision":<n>,"submission_id":"<unique>","#,
    r#""declare":[{"subject":"inq-2","question":"...","parent":"inq-1"}],"#,
    r#""traversal":{"pin":"inq-2","posture":"depth","authority":"user-pinned"}}"#,
    "  (omit a key to persist it, send null to clear a scalar, [] to clear a collection)"
);
const _: () = assert!(DECLARATION_EXAMPLE.len() <= ENVELOPE_DECLARATION_EXAMPLE_BYTES);

/// How much of the run a projection carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Detail {
    /// The budgeted rendering: every list capped, the ceiling enforced.
    Normal,
    /// `show --full`: the caps lift and the projection may scale with the run.
    Full,
}

impl Detail {
    /// The cap to apply to a list — [`usize::MAX`] under `--full`, which is what
    /// makes "normal is a subsequence of full" the *same code path* rather than
    /// two implementations that could disagree.
    const fn cap(self, normal: usize) -> usize {
        match self {
            Detail::Normal => normal,
            Detail::Full => usize::MAX,
        }
    }

    /// The byte cap to apply to a prose scalar. `--full` may scale, but it still
    /// never inlines authored prose — it cites sections, and the caller has the
    /// file.
    const fn prose(self, normal: usize) -> usize {
        self.cap(normal)
    }
}

// ── the model ─────────────────────────────────────────────────────────────

/// Run identity and position — never dropped, never truncated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RunLine {
    pub(crate) uid: String,
    pub(crate) slice: u32,
    pub(crate) revision: u64,
    pub(crate) stage: &'static str,
    /// The reviewer lanes this run requires (DEC-073). Rendered so no reader has
    /// to fetch the snapshot to learn which lanes a section owes — the visibility
    /// half of the fence around a policy that is deliberately mutable.
    pub(crate) review_policy: &'static str,
    pub(crate) watermark: Option<String>,
    pub(crate) materialised: bool,
    pub(crate) change_log_floor: u64,
    pub(crate) receipt_floor: u64,
    pub(crate) posture: &'static str,
    pub(crate) posture_authority: &'static str,
    pub(crate) cursor: Option<String>,
    /// The cursor names a node that is no longer open, so candidates come from
    /// its nearest open ancestor. Surfaced, never silently repaired.
    pub(crate) cursor_stale: bool,
}

/// The global totals (sketch §(g)) — **in the no-drop set**, and the repair for
/// design R2's falsified first form.
///
/// An omitted count measures what a *cap* discarded and says nothing about what
/// *selection* discarded: a 500-node map with a leaf cursor yields a handful of
/// candidates, so `frontier_omitted` is 0 and `truncated` is false on a huge
/// run. These integers are what make "this run is small" and "this run was
/// narrowed to look small" different observations.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub(crate) struct GlobalTotals {
    pub(crate) nodes: usize,
    pub(crate) open: usize,
    pub(crate) resolved: usize,
    pub(crate) deferred: usize,
    pub(crate) pruned: usize,
    pub(crate) blocked: usize,
    /// Open nodes outside the frontier's *candidate* set — the number the first
    /// draft's R2 claim needed and never had.
    pub(crate) open_outside_frontier: usize,
    pub(crate) sections: usize,
    pub(crate) sections_outstanding_review: usize,
    /// Material changes since the baseline, distinct from the rows rendered.
    pub(crate) changes_since_baseline: usize,
}

/// One active-path entry, root → cursor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct PathEntry {
    pub(crate) id: String,
    pub(crate) question: String,
}

/// One nearby-frontier candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FrontierEntry {
    pub(crate) id: String,
    pub(crate) question: String,
    pub(crate) kinship: &'static str,
    pub(crate) needs_in_degree: usize,
    pub(crate) provenance: &'static str,
}

/// The pinned slot — its own field, never a frontier entry, and it renders with
/// whatever lifecycle and blocked state the node currently has.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct PinnedSlot {
    pub(crate) id: String,
    pub(crate) question: String,
    pub(crate) lifecycle: &'static str,
    pub(crate) blocked: bool,
    pub(crate) authority: &'static str,
}

/// One blocker: a node whose `needs` are not yet settled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BlockerEntry {
    pub(crate) id: String,
    pub(crate) reason: String,
    pub(crate) needs_in_degree: usize,
}

/// One recorded act, and whether it still binds to what it was given over.
///
/// This is what `resume`'s *evidence references* re-sourced to at `T11` — the
/// same substitution `EX-11` makes for the change log's invalidation feed, for
/// the same reason: under DEC-120 the recorded acts **are** the run's evidence,
/// so the field SL-233 scope §4 names keeps its meaning rather than its store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ActRow {
    pub(crate) act: &'static str,
    /// Whether the act's coverage still matches current content.
    pub(crate) current: bool,
}

/// One section / review-state row.
///
/// **`clearances` retired at `T11`** (`EX-11`) with the evidence store it read,
/// and is deliberately *not* rebuilt from the acts. Of the nine conditions
/// exactly one is per-section, and `review_outstanding` on this same row already
/// reports it — so a second per-section list would restate one bit beside
/// itself. Its removal takes the uncapped list out of the envelope's byte
/// budget, which is the one competitor `sec-2` named.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct SectionRow {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) fingerprint: String,
    pub(crate) review_outstanding: bool,
}

/// One linked durable record, and the inquiry whose disposition linked it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct DurableRef {
    pub(crate) record: String,
    pub(crate) node: String,
    pub(crate) form: &'static str,
}

/// The material-change delta.
///
/// **Unavailable and empty are opposite facts** (design R2): "nothing changed"
/// and "I cannot tell you what changed" must never render identically, so they
/// are different variants rather than an empty list twice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "delta")]
pub(crate) enum ChangeDelta {
    Unavailable {
        floor: u64,
        known_revision: u64,
    },
    Since {
        known_revision: u64,
        rows: Vec<String>,
    },
}

/// Exact omitted counts, per bounded field. A ladder eviction and a cap eviction
/// are indistinguishable here on purpose: the count is exact either way.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub(crate) struct Omitted {
    pub(crate) active_path: usize,
    pub(crate) frontier: usize,
    pub(crate) blockers: usize,
    pub(crate) changes: usize,
    pub(crate) durable_records: usize,
    pub(crate) sections: usize,
}

impl Omitted {
    /// Whether any field dropped anything at all — the single `truncated` flag.
    const fn any(&self) -> bool {
        self.active_path > 0
            || self.frontier > 0
            || self.blockers > 0
            || self.changes > 0
            || self.durable_records > 0
            || self.sections > 0
    }
}

/// The canonical read model: everything one turn needs, and nothing that scales
/// with the run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct TurnEnvelope {
    pub(crate) schema: &'static str,
    pub(crate) version: u32,
    pub(crate) detail: Detail,
    pub(crate) run: RunLine,
    pub(crate) totals: GlobalTotals,
    pub(crate) next_obligation: Option<String>,
    pub(crate) pinned: Option<PinnedSlot>,
    pub(crate) active_path: Vec<PathEntry>,
    pub(crate) frontier: Vec<FrontierEntry>,
    pub(crate) blockers: Vec<BlockerEntry>,
    pub(crate) sections: Vec<SectionRow>,
    /// The recorded acts — `resume`'s evidence references (SL-233 scope §4).
    pub(crate) acts: Vec<ActRow>,
    pub(crate) durable_records: Vec<DurableRef>,
    pub(crate) changes: ChangeDelta,
    pub(crate) declaration_example: &'static str,
    pub(crate) omitted: Omitted,
    pub(crate) truncated: bool,
}

// ── projection ────────────────────────────────────────────────────────────

/// Project one turn, bounded.
///
/// `known_revision` is the caller's declared baseline; the delta is the
/// half-open range `(known_revision, current]`.
pub(crate) fn project(
    run: &DesignSnapshot,
    known_revision: u64,
    detail: Detail,
) -> Result<TurnEnvelope, Refusal> {
    project_within(run, known_revision, detail, ENVELOPE_NORMAL_BUDGET_BYTES)
}

/// [`project`], against an explicit ceiling.
///
/// The parameter exists so the eviction ladder and its terminal rule are
/// *reachable in a test* without retuning the shipped constant: a bound whose
/// enforcement has never been observed firing is a bound nobody has checked.
pub(crate) fn project_within(
    run: &DesignSnapshot,
    known_revision: u64,
    detail: Detail,
    budget: usize,
) -> Result<TurnEnvelope, Refusal> {
    let mut envelope = assemble(run, known_revision, detail);
    if detail == Detail::Full {
        return Ok(envelope);
    }
    while rendered_bytes(&envelope) > budget {
        if !evict_one(&mut envelope) {
            // The terminal rule: every bounded list is on the ladder, so the one
            // irreducible state left is the no-drop set alone exceeding the
            // ceiling. Refuse rather than emit a quietly malformed envelope.
            return Err(Refusal::EnvelopeIrreducible {
                budget,
                rendered: rendered_bytes(&envelope),
            });
        }
    }
    envelope.truncated = envelope.omitted.any();
    Ok(envelope)
}

/// The rendered size of the budgeted rendering, measured rather than predicted.
fn rendered_bytes(envelope: &TurnEnvelope) -> usize {
    prompt(envelope).iter().map(|line| line.len() + 1).sum()
}

/// The eviction ladder (sketch §(b)): drop **one entry at a time**, from each
/// list's own stated drop end, in this fixed field order.
///
/// The active path is rung 6 rather than excluded. "Costs the most to lose" is
/// an argument for ordering it last, not for removing it — excluding it left the
/// overflow rule *partial*, with a state that had no defined outcome.
fn evict_one(envelope: &mut TurnEnvelope) -> bool {
    // 1. section rows — drop end is the latest.
    if envelope.sections.pop().is_some() {
        envelope.omitted.sections += 1;
        return true;
    }
    // 2. durable records — drop end is the least recently linked.
    if envelope.durable_records.pop().is_some() {
        envelope.omitted.durable_records += 1;
        return true;
    }
    // 3. change delta — drop end is the OLDEST, and rows render oldest-first.
    if let ChangeDelta::Since { rows, .. } = &mut envelope.changes
        && !rows.is_empty()
    {
        rows.remove(0);
        envelope.omitted.changes += 1;
        return true;
    }
    // 4. blockers — drop end is the least consequential.
    if envelope.blockers.pop().is_some() {
        envelope.omitted.blockers += 1;
        return true;
    }
    // 5. frontier — drop end is the lowest-ranked.
    if envelope.frontier.pop().is_some() {
        envelope.omitted.frontier += 1;
        return true;
    }
    // 6. active path — drop end is the ROOT end; the cursor end is retained.
    if envelope.active_path.is_empty() {
        return false;
    }
    envelope.active_path.remove(0);
    envelope.omitted.active_path += 1;
    true
}

/// Build the envelope with every per-field cap applied, before the ladder runs.
fn assemble(run: &DesignSnapshot, known_revision: u64, detail: Detail) -> TurnEnvelope {
    let (cursor, cursor_stale) = effective_cursor(run);
    let candidates = frontier_candidates(run, cursor.as_ref());

    let (active_path, path_omitted) = active_path(run, cursor.as_ref(), detail);
    let (frontier, frontier_omitted) = frontier(run, &candidates, detail);
    let (blockers, blockers_omitted) = blockers(run, detail);
    let (sections, sections_omitted) = sections(run, detail);
    let (durable_records, records_omitted) = durable_records(run, detail);
    let (changes, changes_omitted, total_changes) = changes(run, known_revision, detail);

    let omitted = Omitted {
        active_path: path_omitted,
        frontier: frontier_omitted,
        blockers: blockers_omitted,
        changes: changes_omitted,
        durable_records: records_omitted,
        sections: sections_omitted,
    };

    let totals = totals(run, &candidates, total_changes);

    TurnEnvelope {
        schema: TURN_ENVELOPE_SCHEMA,
        version: TURN_ENVELOPE_VERSION,
        detail,
        run: RunLine {
            uid: run.run.uid.clone(),
            slice: run.run.slice,
            revision: run.run.revision,
            stage: run.run.stage.as_str(),
            review_policy: run.run.review_policy.as_str(),
            watermark: run
                .authored
                .watermark
                .as_ref()
                .map(|f| f.as_str().to_owned()),
            materialised: run.authored.materialised,
            change_log_floor: run.change_log.floor,
            receipt_floor: run.receipts.floor,
            posture: posture_label(run.map.posture.posture()),
            posture_authority: authority_label(run.map.posture.authority()),
            cursor: cursor.as_ref().map(DesignId::to_string),
            cursor_stale,
        },
        totals,
        next_obligation: run
            .run
            .next_obligation
            .as_ref()
            .map(|text| elide(text, detail.prose(ENVELOPE_REASON_BYTES))),
        pinned: pinned(run, detail),
        active_path,
        frontier,
        blockers,
        sections,
        acts: acts(run),
        durable_records,
        changes,
        declaration_example: DECLARATION_EXAMPLE,
        truncated: omitted.any(),
        omitted,
    }
}

// ── (c) frontier selection ────────────────────────────────────────────────

/// The kinship classes, in rank order. **The table IS the eligibility rule**: a
/// node is a candidate if and only if it matches a class here, and classes are
/// tested in ascending rank so every candidate has exactly one rank.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kinship {
    Child,
    Sibling,
    NeedsNeighbour,
    Grandchild,
    AncestorSibling,
    Parent,
    GrandparentOrNibling,
}

impl Kinship {
    /// The rank, with posture applied: **breadth swaps children and siblings**,
    /// and nothing else. That swap is the concrete content of design §5.3's
    /// adaptive traversal — posture is load-bearing, not decoration.
    const fn rank(self, posture: Posture) -> u8 {
        match (self, posture) {
            (Kinship::Child, Posture::Breadth) | (Kinship::Sibling, Posture::Depth) => 1,
            (Kinship::Child, Posture::Depth) | (Kinship::Sibling, Posture::Breadth) => 0,
            (Kinship::NeedsNeighbour, _) => 2,
            (Kinship::Grandchild, _) => 3,
            (Kinship::AncestorSibling, _) => 4,
            (Kinship::Parent, _) => 5,
            (Kinship::GrandparentOrNibling, _) => 6,
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Kinship::Child => "child",
            Kinship::Sibling => "sibling",
            Kinship::NeedsNeighbour => "needs-neighbour",
            Kinship::Grandchild => "grandchild",
            Kinship::AncestorSibling => "ancestor-sibling",
            Kinship::Parent => "parent",
            Kinship::GrandparentOrNibling => "grandparent-or-nibling",
        }
    }
}

/// The cursor the projection actually reasons from, plus whether the declared
/// one was stale.
///
/// A cursor on a pruned or resolved node is **stale**: candidates come from its
/// nearest open ancestor, and the envelope says so rather than quietly acting as
/// though the caller had moved it.
fn effective_cursor(run: &DesignSnapshot) -> (Option<DesignId>, bool) {
    let map = &run.map.inquiry;
    let Some(at) = run.map.cursor.at() else {
        return (None, false);
    };
    let Some(node) = map.get(at) else {
        return (None, true);
    };
    if node.lifecycle() == InquiryLifecycle::Open {
        return (Some(at.clone()), false);
    }
    let mut walk = node.parent().cloned();
    while let Some(id) = walk {
        let Some(ancestor) = map.get(&id) else { break };
        if ancestor.lifecycle() == InquiryLifecycle::Open {
            return (Some(id), true);
        }
        walk = ancestor.parent().cloned();
    }
    (None, true)
}

/// Every eligible candidate, with its single kinship rank.
///
/// Candidates are `open`, not derived-`blocked`, not the cursor, and not the
/// pinned node — the pin has its own always-present slot, so including it here
/// would render it twice.
fn frontier_candidates(
    run: &DesignSnapshot,
    cursor: Option<&DesignId>,
) -> BTreeMap<DesignId, Kinship> {
    let map = &run.map.inquiry;
    let pinned = run.map.pin.as_ref().map(|pin| pin.at.clone());
    let mut candidates: BTreeMap<DesignId, Kinship> = BTreeMap::new();

    let eligible = |node: &InquiryNode| {
        node.lifecycle() == InquiryLifecycle::Open
            && !map.is_blocked(node.id())
            && Some(node.id()) != cursor
            && pinned.as_ref() != Some(node.id())
    };

    let Some(cursor) = cursor else {
        // No cursor: candidates are the roots' own children — here, the
        // parentless open nodes, which is what a map with no chosen focus
        // offers.
        for node in map.nodes().filter(|node| node.parent().is_none()) {
            if eligible(node) {
                candidates.insert(node.id().clone(), Kinship::Child);
            }
        }
        return candidates;
    };

    let parent = map.get(cursor).and_then(|node| node.parent().cloned());
    let grandparent = parent
        .as_ref()
        .and_then(|id| map.get(id))
        .and_then(|node| node.parent().cloned());
    let children: BTreeSet<DesignId> = map
        .nodes()
        .filter(|node| node.parent() == Some(cursor))
        .map(|node| node.id().clone())
        .collect();
    let siblings: BTreeSet<DesignId> = match parent.as_ref() {
        Some(parent) => map
            .nodes()
            .filter(|node| node.parent() == Some(parent) && node.id() != cursor)
            .map(|node| node.id().clone())
            .collect(),
        None => BTreeSet::new(),
    };

    // Classes are tested in ascending rank; the first match wins, so overlap
    // (a child that is also a `needs` neighbour) resolves to the lowest rank
    // instead of having no defined answer.
    let mut claim = |id: &DesignId, kinship: Kinship| {
        let Some(node) = map.get(id) else { return };
        if eligible(node) {
            candidates.entry(id.clone()).or_insert(kinship);
        }
    };

    for id in &children {
        claim(id, Kinship::Child);
    }
    for id in &siblings {
        claim(id, Kinship::Sibling);
    }
    if let Some(node) = map.get(cursor) {
        for id in node.needs() {
            claim(id, Kinship::NeedsNeighbour);
        }
    }
    for node in map.nodes().filter(|node| node.needs().contains(cursor)) {
        claim(node.id(), Kinship::NeedsNeighbour);
    }
    for child in &children {
        for node in map.nodes().filter(|node| node.parent() == Some(child)) {
            claim(node.id(), Kinship::Grandchild);
        }
    }
    for id in nearest_ancestor_siblings(run, cursor) {
        claim(&id, Kinship::AncestorSibling);
    }
    if let Some(parent) = parent.as_ref() {
        claim(parent, Kinship::Parent);
    }
    if let Some(grandparent) = grandparent.as_ref() {
        claim(grandparent, Kinship::GrandparentOrNibling);
    }
    for sibling in &siblings {
        for node in map.nodes().filter(|node| node.parent() == Some(sibling)) {
            claim(node.id(), Kinship::GrandparentOrNibling);
        }
    }
    candidates
}

/// The siblings of the **nearest** ancestor that has any open ones — where
/// traversal goes next once this arm is exhausted.
fn nearest_ancestor_siblings(run: &DesignSnapshot, cursor: &DesignId) -> Vec<DesignId> {
    let map = &run.map.inquiry;
    let mut at = map.get(cursor).and_then(|node| node.parent().cloned());
    while let Some(ancestor) = at {
        let Some(node) = map.get(&ancestor) else {
            break;
        };
        let above = node.parent().cloned();
        if let Some(above_id) = above.as_ref() {
            let found: Vec<DesignId> = map
                .nodes()
                .filter(|sibling| {
                    sibling.parent() == Some(above_id)
                        && sibling.id() != &ancestor
                        && sibling.lifecycle() == InquiryLifecycle::Open
                })
                .map(|sibling| sibling.id().clone())
                .collect();
            if !found.is_empty() {
                return found;
            }
        }
        at = above;
    }
    Vec::new()
}

/// The frontier, ranked and capped.
///
/// The sort key is lexicographic over persisted values only: kinship rank, then
/// `needs` in-degree descending, then creation order, then id — total, because
/// ids are unique.
fn frontier(
    run: &DesignSnapshot,
    candidates: &BTreeMap<DesignId, Kinship>,
    detail: Detail,
) -> (Vec<FrontierEntry>, usize) {
    let map = &run.map.inquiry;
    let posture = run.map.posture.posture();
    let mut ranked: Vec<(u8, std::cmp::Reverse<usize>, u64, DesignId, Kinship)> = candidates
        .iter()
        .filter_map(|(id, kinship)| {
            let node = map.get(id)?;
            Some((
                kinship.rank(posture),
                std::cmp::Reverse(map.needs_in_degree(id)),
                node.seq(),
                id.clone(),
                *kinship,
            ))
        })
        .collect();
    ranked.sort();

    let cap = detail.cap(ENVELOPE_FRONTIER_NODES);
    let omitted = ranked.len().saturating_sub(cap);
    let entries = ranked
        .into_iter()
        .take(cap)
        .filter_map(|(_, needs, _, id, kinship)| {
            let node = map.get(&id)?;
            Some(FrontierEntry {
                id: id.to_string(),
                question: elide(node.question(), detail.prose(ENVELOPE_QUESTION_BYTES)),
                kinship: kinship.label(),
                needs_in_degree: needs.0,
                provenance: node.provenance().label(),
            })
        })
        .collect();
    (entries, omitted)
}

/// The pinned slot, with the node's *current* state whatever it is.
fn pinned(run: &DesignSnapshot, detail: Detail) -> Option<PinnedSlot> {
    let pin = run.map.pin.as_ref()?;
    let node = run.map.inquiry.get(&pin.at)?;
    Some(PinnedSlot {
        id: pin.at.to_string(),
        question: elide(node.question(), detail.prose(ENVELOPE_QUESTION_BYTES)),
        lifecycle: node.lifecycle().as_str(),
        blocked: run.map.inquiry.is_blocked(&pin.at),
        authority: authority_label(pin.authority),
    })
}

/// The active path, root → cursor, retained from the **cursor** end.
fn active_path(
    run: &DesignSnapshot,
    cursor: Option<&DesignId>,
    detail: Detail,
) -> (Vec<PathEntry>, usize) {
    let map = &run.map.inquiry;
    let mut chain: Vec<DesignId> = Vec::new();
    let mut at = cursor.cloned();
    while let Some(id) = at {
        if chain.contains(&id) {
            break;
        }
        let Some(node) = map.get(&id) else { break };
        at = node.parent().cloned();
        chain.push(id);
    }
    chain.reverse();

    let cap = detail.cap(ENVELOPE_ACTIVE_PATH_DEPTH);
    let omitted = chain.len().saturating_sub(cap);
    let entries = chain
        .into_iter()
        .skip(omitted)
        .filter_map(|id| {
            let node = map.get(&id)?;
            Some(PathEntry {
                id: id.to_string(),
                question: elide(node.question(), detail.prose(ENVELOPE_QUESTION_BYTES)),
            })
        })
        .collect();
    (entries, omitted)
}

/// Blockers, most consequential first.
fn blockers(run: &DesignSnapshot, detail: Detail) -> (Vec<BlockerEntry>, usize) {
    let map = &run.map.inquiry;
    let mut ranked: Vec<(std::cmp::Reverse<usize>, u64, DesignId)> = map
        .blocked()
        .map(|node| {
            (
                std::cmp::Reverse(map.needs_in_degree(node.id())),
                node.seq(),
                node.id().clone(),
            )
        })
        .collect();
    ranked.sort();

    let cap = detail.cap(ENVELOPE_BLOCKERS);
    let omitted = ranked.len().saturating_sub(cap);
    let entries = ranked
        .into_iter()
        .take(cap)
        .filter_map(|(degree, _, id)| {
            let node = map.get(&id)?;
            let unsettled: Vec<String> = node
                .needs()
                .iter()
                .filter(|need| {
                    map.get(need).is_some_and(|target| {
                        matches!(
                            target.lifecycle(),
                            InquiryLifecycle::Open | InquiryLifecycle::Deferred
                        )
                    })
                })
                .map(DesignId::to_string)
                .collect();
            Some(BlockerEntry {
                id: id.to_string(),
                reason: elide(
                    &format!("needs {}", unsettled.join(", ")),
                    detail.prose(ENVELOPE_REASON_BYTES),
                ),
                needs_in_degree: degree.0,
            })
        })
        .collect();
    (entries, omitted)
}

/// The acts this run has on record, and whether each still binds.
///
/// **Uncapped, and structurally bounded rather than budgeted.** Both act groups
/// replace by kind — [`CheckpointActGroup::record`] retains-then-pushes, and an
/// agent declaration displaces its predecessor — so this list can never exceed
/// [`ActKind::ALL`]'s eight members and needs no entry on the eviction ladder.
/// That is the difference from the `clearances` list it replaces, which grew
/// with the run and was the byte-budget competitor `sec-2` named: `EX-11` did
/// not trade one uncapped list for another.
///
/// [`CheckpointActGroup::record`]: super::super::snapshot::CheckpointActGroup::record
fn acts(run: &DesignSnapshot) -> Vec<ActRow> {
    let live = super::super::run::live_acts(run);
    let mut rows: Vec<ActRow> = run
        .acts
        .acts
        .iter()
        .map(|held| (held.act, &held.id))
        .chain(
            run.declarations
                .declarations
                .iter()
                .map(|held| (ActKind::from(held.act.kind()), &held.id)),
        )
        .map(|(act, id)| ActRow {
            act: act.as_str(),
            current: live.contains(&(act, id.clone())),
        })
        .collect();
    rows.sort_by_key(|row| row.act);
    rows
}

/// Section / review-state rows: outstanding review first, then section order.
fn sections(run: &DesignSnapshot, detail: Detail) -> (Vec<SectionRow>, usize) {
    let mut ranked: Vec<(bool, DesignId)> = run
        .sections
        .sections
        .iter()
        .map(|section| (!review_outstanding(run, &section.id), section.id.clone()))
        .collect();
    ranked.sort();

    let cap = detail.cap(ENVELOPE_SECTION_ROWS);
    let omitted = ranked.len().saturating_sub(cap);
    let rows = ranked
        .into_iter()
        .take(cap)
        .filter_map(|(settled, id)| {
            let section = run.sections.find(&id)?;
            Some(SectionRow {
                id: id.to_string(),
                title: elide(&section.title, detail.prose(ENVELOPE_LABEL_BYTES)),
                // An abbreviation with a stated collision budget, not a
                // truncation: the stored row keeps the whole digest.
                fingerprint: super::abbreviate_digest(section.fingerprint.as_str()),
                review_outstanding: !settled,
            })
        })
        .collect();
    (rows, omitted)
}

/// Whether a section still owes a reviewer lane under the run's policy.
///
/// The same derivation the gate reads ([`DesignSnapshot::missing_lanes`]), not a
/// second spelling of it: this is the surface that would otherwise render as
/// settled a section the gate refuses, which is ISS-310's defect one step further
/// out. Sharing the home makes the two agree by construction.
fn review_outstanding(run: &DesignSnapshot, id: &DesignId) -> bool {
    let Some(section) = run.sections.find(id) else {
        return true;
    };
    !run.missing_lanes(id, &section.fingerprint).is_empty()
}

/// Linked durable records, most recently linked first.
fn durable_records(run: &DesignSnapshot, detail: Detail) -> (Vec<DurableRef>, usize) {
    let mut ranked: Vec<(std::cmp::Reverse<u64>, DesignId, String, &'static str)> = run
        .map
        .inquiry
        .nodes()
        .filter_map(|node| {
            let (record, form) = match node.disposition()? {
                Disposition::Created { record } => (record.clone(), "create"),
                Disposition::Adopted { record } => (record.clone(), "adopt"),
                Disposition::RetainedUnresolved { .. } | Disposition::NonDurable { .. } => {
                    return None;
                }
            };
            Some((
                std::cmp::Reverse(node.seq()),
                node.id().clone(),
                record,
                form,
            ))
        })
        .collect();
    ranked.sort();

    let cap = detail.cap(ENVELOPE_DURABLE_RECORDS);
    let omitted = ranked.len().saturating_sub(cap);
    let refs = ranked
        .into_iter()
        .take(cap)
        .map(|(_, node, record, form)| DurableRef {
            record,
            node: node.to_string(),
            form,
        })
        .collect();
    (refs, omitted)
}

/// The material-change delta, newest retained.
///
/// Returns the projection, the omitted count, and the *total* material changes
/// since the baseline — the last being a global total, so it reports what the
/// delta could not.
fn changes(
    run: &DesignSnapshot,
    known_revision: u64,
    detail: Detail,
) -> (ChangeDelta, usize, usize) {
    let log = &run.change_log;
    if !log.covers(known_revision) {
        return (
            ChangeDelta::Unavailable {
                floor: log.floor,
                known_revision,
            },
            0,
            0,
        );
    }
    let all = log.since(known_revision);
    let total = all.len();
    let cap = detail.cap(ENVELOPE_CHANGE_ROWS);
    let omitted = total.saturating_sub(cap);
    // Retained from the NEWEST end, rendered oldest-first: the cut is by
    // `(revision, index)` descending, the reading order is not.
    let rows = all
        .into_iter()
        .skip(omitted)
        .map(|row| match detail {
            Detail::Normal => change_row::render(row),
            Detail::Full => change_row::render_full(row),
        })
        .collect();
    (
        ChangeDelta::Since {
            known_revision,
            rows,
        },
        omitted,
        total,
    )
}

/// The global totals.
fn totals(
    run: &DesignSnapshot,
    candidates: &BTreeMap<DesignId, Kinship>,
    changes_since_baseline: usize,
) -> GlobalTotals {
    let map = &run.map.inquiry;
    let mut totals = GlobalTotals {
        nodes: map.len(),
        changes_since_baseline,
        sections: run.sections.sections.len(),
        ..GlobalTotals::default()
    };
    for node in map.nodes() {
        match node.lifecycle() {
            InquiryLifecycle::Open => totals.open += 1,
            InquiryLifecycle::Resolved => totals.resolved += 1,
            InquiryLifecycle::Deferred => totals.deferred += 1,
            InquiryLifecycle::Pruned => totals.pruned += 1,
        }
        if map.is_blocked(node.id()) {
            totals.blocked += 1;
        }
        if node.lifecycle() == InquiryLifecycle::Open && !candidates.contains_key(node.id()) {
            totals.open_outside_frontier += 1;
        }
    }
    totals.sections_outstanding_review = run
        .sections
        .sections
        .iter()
        .filter(|section| review_outstanding(run, &section.id))
        .count();
    totals
}

const fn posture_label(posture: Posture) -> &'static str {
    match posture {
        Posture::Breadth => "breadth",
        Posture::Depth => "depth",
    }
}

const fn authority_label(authority: Authority) -> &'static str {
    match authority {
        Authority::AgentProposed => "agent-proposed",
        Authority::UserPinned => "user-pinned",
        Authority::UserLocked => "user-locked",
    }
}

// ── renderings (DEC-064: one model, three renderings) ─────────────────────

/// The budgeted rendering — `design show --format prompt`, the projection that
/// enters an agent's context and the only one R1 is about.
pub(crate) fn prompt(envelope: &TurnEnvelope) -> Vec<String> {
    let run = &envelope.run;
    let mut lines = vec![
        format!(
            "run {} revision {} stage {}",
            run.uid, run.revision, run.stage
        ),
        format!(
            "watermark {} materialised {}",
            run.watermark.as_deref().unwrap_or("absent"),
            run.materialised
        ),
        format!(
            "change_log_floor {} receipt_floor {}",
            run.change_log_floor, run.receipt_floor
        ),
        format!(
            "posture {} ({}) cursor {}{}",
            run.posture,
            run.posture_authority,
            run.cursor.as_deref().unwrap_or("unset"),
            if run.cursor_stale { " STALE" } else { "" }
        ),
        totals_line(&envelope.totals),
    ];
    if let Some(obligation) = envelope.next_obligation.as_ref() {
        lines.push(format!("next_obligation {obligation}"));
    }
    if let Some(pin) = envelope.pinned.as_ref() {
        lines.push(format!(
            "pinned {} lifecycle={} blocked={} authority={} — {}",
            pin.id, pin.lifecycle, pin.blocked, pin.authority, pin.question
        ));
    }
    if !envelope.active_path.is_empty() || envelope.omitted.active_path > 0 {
        let path: Vec<&str> = envelope
            .active_path
            .iter()
            .map(|entry| entry.id.as_str())
            .collect();
        lines.push(format!(
            "active_path {}{}",
            path.join(" > "),
            more(envelope.omitted.active_path)
        ));
    }
    lines.push(format!("frontier{}", more(envelope.omitted.frontier)));
    for entry in &envelope.frontier {
        lines.push(format!(
            "  {} kinship={} needs_in_degree={} provenance={} — {}",
            entry.id, entry.kinship, entry.needs_in_degree, entry.provenance, entry.question
        ));
    }
    lines.push(format!("blockers{}", more(envelope.omitted.blockers)));
    for entry in &envelope.blockers {
        lines.push(format!("  {} — {}", entry.id, entry.reason));
    }
    lines.push(format!("sections{}", more(envelope.omitted.sections)));
    for row in &envelope.sections {
        lines.push(format!(
            "  {} fingerprint={} review={}{}",
            row.id,
            row.fingerprint,
            if row.review_outstanding {
                "outstanding"
            } else {
                "current"
            },
            if row.title.is_empty() {
                String::new()
            } else {
                format!(" — {}", row.title)
            }
        ));
    }
    lines.push(format!("records{}", more(envelope.omitted.durable_records)));
    for entry in &envelope.durable_records {
        lines.push(format!(
            "  {} via {} ({})",
            entry.record, entry.node, entry.form
        ));
    }
    lines.extend(delta_lines(&envelope.changes, envelope.omitted.changes));
    lines.push(format!("truncated {}", envelope.truncated));
    lines.push(format!("declare {}", envelope.declaration_example));
    lines
}

/// The human status rendering — same model, terminal-shaped, unbudgeted.
pub(crate) fn status(envelope: &TurnEnvelope) -> Vec<String> {
    let run = &envelope.run;
    let totals = &envelope.totals;
    let mut lines = vec![
        format!("design run {} for slice {:03}", run.uid, run.slice),
        format!("  stage        {} (revision {})", run.stage, run.revision),
        format!(
            "  traversal    {} posture, {}; cursor {}",
            run.posture,
            run.posture_authority,
            run.cursor.as_deref().unwrap_or("unset")
        ),
        format!(
            "  inquiry      {} nodes — {} open, {} resolved, {} deferred, {} pruned, {} blocked",
            totals.nodes,
            totals.open,
            totals.resolved,
            totals.deferred,
            totals.pruned,
            totals.blocked
        ),
        format!(
            "  sections     {} ({} with outstanding review)",
            totals.sections, totals.sections_outstanding_review
        ),
        format!(
            "  changes      {} since the declared baseline",
            totals.changes_since_baseline
        ),
    ];
    if let Some(obligation) = envelope.next_obligation.as_ref() {
        lines.push(format!("  next         {obligation}"));
    }
    if envelope.truncated {
        lines.push("  (this projection is bounded; `design show --full` widens it)".to_owned());
    }
    lines
}

/// The compact resume projection — scope §4's seven fields, in that order, from
/// the same envelope. `resume` is a *rendering*, not a second model.
pub(crate) fn resume(envelope: &TurnEnvelope) -> Vec<String> {
    let path: Vec<&str> = envelope
        .active_path
        .iter()
        .map(|entry| entry.id.as_str())
        .collect();
    let mut lines = vec![
        format!(
            "run {} revision {} stage {}",
            envelope.run.uid, envelope.run.revision, envelope.run.stage
        ),
        format!(
            "active_path {}{}",
            if path.is_empty() {
                "none".to_owned()
            } else {
                path.join(" > ")
            },
            more(envelope.omitted.active_path)
        ),
    ];

    lines.push("accepted_decisions".to_owned());
    lines.extend(record_lines(envelope, "DEC-"));
    lines.push("open_questions".to_owned());
    for entry in &envelope.frontier {
        lines.push(format!("  {} — {}", entry.id, entry.question));
    }
    lines.push("assumptions".to_owned());
    lines.extend(record_lines(envelope, "ASM-"));
    // Re-sourced from the evidence store to the act set at `T11`, keeping the
    // field SL-233 scope §4 names. `current` is the point of the row: an act
    // whose material has moved is still on the record and no longer binds, and a
    // resuming agent that could not tell those apart would re-do work or skip it.
    lines.push("evidence_references".to_owned());
    for row in &envelope.acts {
        lines.push(format!(
            "  {} — {}",
            row.act,
            if row.current { "current" } else { "stale" }
        ));
    }
    lines.push("blockers".to_owned());
    for entry in &envelope.blockers {
        lines.push(format!("  {} — {}", entry.id, entry.reason));
    }
    lines.push(format!(
        "next_obligation {}",
        envelope
            .next_obligation
            .as_deref()
            .unwrap_or("none recorded")
    ));
    lines
}

/// The linked records whose canonical reference carries `prefix`.
fn record_lines(envelope: &TurnEnvelope, prefix: &str) -> Vec<String> {
    envelope
        .durable_records
        .iter()
        .filter(|entry| entry.record.starts_with(prefix))
        .map(|entry| format!("  {} via {} ({})", entry.record, entry.node, entry.form))
        .collect()
}

/// The totals block — in the no-drop set, so it is one line that always renders.
fn totals_line(totals: &GlobalTotals) -> String {
    [
        format!("totals nodes={}", totals.nodes),
        format!("open={}", totals.open),
        format!("resolved={}", totals.resolved),
        format!("deferred={}", totals.deferred),
        format!("pruned={}", totals.pruned),
        format!("blocked={}", totals.blocked),
        format!("open_outside_frontier={}", totals.open_outside_frontier),
        format!("sections={}", totals.sections),
        format!(
            "sections_outstanding_review={}",
            totals.sections_outstanding_review
        ),
        format!("changes_since_baseline={}", totals.changes_since_baseline),
    ]
    .join(FIELD_SEPARATOR)
}

/// The delta block. Unavailable, empty and non-empty are three different
/// renderings, because they are three different facts.
fn delta_lines(delta: &ChangeDelta, omitted: usize) -> Vec<String> {
    match delta {
        ChangeDelta::Unavailable {
            floor,
            known_revision,
        } => vec![super::delta_unavailable_line(*floor, *known_revision)],
        ChangeDelta::Since {
            known_revision,
            rows,
        } if rows.is_empty() && omitted == 0 => vec![super::delta_none_line(*known_revision)],
        ChangeDelta::Since {
            known_revision,
            rows,
        } => {
            let mut lines = vec![format!(
                "{}{}",
                super::delta_since_line(*known_revision),
                more(omitted)
            )];
            lines.extend(rows.iter().cloned());
            lines
        }
    }
}

/// The omitted-count marker. **No drop is ever silent**: every bounded field
/// that omitted anything carries its exact count.
fn more(omitted: usize) -> String {
    if omitted == 0 {
        String::new()
    } else {
        format!(" (+{omitted} more)")
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test code — the repo's panic-avoidance denials target production paths"
)]
mod tests {
    use super::{Detail, ENVELOPE_NORMAL_BUDGET_BYTES, project, project_within, rendered_bytes};
    use crate::design_run::attestation::{ReviewPolicy, Reviewer};
    use crate::design_run::fixture::{attest, run_holding};
    use crate::design_run::ids::DesignId;
    use crate::design_run::inquiry::{InquiryNode, Provenance};
    use crate::design_run::refusal::Refusal;
    use crate::design_run::snapshot::DesignSnapshot;

    /// A run with `count` open nodes hanging off one root.
    fn wide_run(count: u32) -> DesignSnapshot {
        let mut run = DesignSnapshot::new("dr-test", 233, None);
        let root = DesignId::parse("inq-root").unwrap();
        let seq = run.map.claim_seq();
        run.map
            .inquiry
            .insert(
                InquiryNode::open(root.clone(), "the root question", Provenance::UserDirected)
                    .sequenced(seq),
            )
            .unwrap();
        for index in 0..count {
            let id = DesignId::parse(&format!("inq-{index}")).unwrap();
            let seq = run.map.claim_seq();
            run.map
                .inquiry
                .insert(
                    InquiryNode::open(id, format!("question {index}"), Provenance::AgentProposed)
                        .sequenced(seq)
                        .with_parent(root.clone()),
                )
                .unwrap();
        }
        run.map.cursor = crate::design_run::traversal::Cursor::placed(
            root,
            crate::design_run::traversal::Authority::UserPinned,
        );
        run
    }

    /// The gate and the envelope answer *is this section reviewed* from the same
    /// derivation, so they cannot disagree by construction rather than by two
    /// edits that happen to match.
    ///
    /// Repairing only the gate would leave it refusing a section the envelope
    /// simultaneously renders as settled — ISS-310's defect one surface further
    /// out. The adversarial attestation is held and stays held throughout: under
    /// `HumanOnly` it is visible without being sufficient, and loosening the
    /// policy settles the row without a single new attestation.
    #[test]
    fn gate_and_envelope_agree_under_one_policy() {
        let mut run = run_holding(&[("sec-a", "sha256:a")]);
        attest(&mut run, "att-a", "sec-a", Reviewer::Adversarial);

        let settled = |run: &DesignSnapshot| {
            let envelope = project(run, 0, Detail::Normal).expect("the fixture run projects");
            let row = envelope
                .sections
                .iter()
                .find(|row| row.id == "sec-a")
                .expect("the envelope renders the section it holds")
                .clone();
            // The policy in force rides the run line, so a reader who sees an
            // outstanding row can tell WHICH lane is owed without fetching the
            // snapshot — visibility is the fence around a mutable policy.
            assert_eq!(envelope.run.review_policy, run.run.review_policy.as_str());
            !row.review_outstanding
        };

        assert_eq!(run.run.review_policy, ReviewPolicy::HumanOnly);
        assert!(!run.sections_unreviewed().is_empty(), "the gate refuses");
        assert!(!settled(&run), "and the envelope agrees it is outstanding");

        run.run.review_policy = ReviewPolicy::AdversarialOnly;
        assert!(
            run.sections_unreviewed().is_empty(),
            "the gate is satisfied by the attestation already recorded"
        );
        assert!(
            settled(&run),
            "and the envelope agrees, on the same reading"
        );

        // The attestation was never touched: what moved is the requirement.
        assert_eq!(run.review.attestations.len(), 1);
    }

    /// The ladder fires and the counts are exact: against a ceiling too small for
    /// the assembled envelope, entries are evicted one at a time and every drop
    /// is reported.
    #[test]
    fn the_eviction_ladder_fires_and_counts_every_drop() {
        let run = wide_run(40);
        let roomy = project_within(&run, 0, Detail::Normal, ENVELOPE_NORMAL_BUDGET_BYTES).unwrap();
        assert_eq!(roomy.frontier.len(), super::ENVELOPE_FRONTIER_NODES);
        assert!(
            rendered_bytes(&roomy) <= ENVELOPE_NORMAL_BUDGET_BYTES,
            "a well-formed run is nowhere near the ceiling"
        );

        // A ceiling below the assembled size forces the ladder.
        let tight = rendered_bytes(&roomy) - 200;
        let cut = project_within(&run, 0, Detail::Normal, tight).unwrap();
        assert!(rendered_bytes(&cut) <= tight, "the ceiling is enforced");
        assert!(cut.truncated, "and the drop is not silent");
        assert!(
            cut.omitted.frontier > roomy.omitted.frontier,
            "the ladder reached the frontier and counted what it dropped"
        );
    }

    /// The terminal rule: when the no-drop set alone exceeds the ceiling, the
    /// projection REFUSES rather than emitting a quietly malformed envelope.
    #[test]
    fn the_no_drop_set_alone_over_the_ceiling_is_refused() {
        let run = wide_run(40);
        let refused = project_within(&run, 0, Detail::Normal, 1).unwrap_err();
        assert!(
            matches!(refused, Refusal::EnvelopeIrreducible { .. }),
            "{refused:?}"
        );
    }
}
