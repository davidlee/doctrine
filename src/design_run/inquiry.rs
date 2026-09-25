// SPDX-License-Identifier: GPL-3.0-only
//! The inquiry map — nodes, provenance, lifecycle, and the two edge kinds.
//!
//! A primary-parent tree gives a readable decomposition; a sparse `needs` set
//! captures the minimum non-tree dependency (DEC-061). Both are acyclic, and
//! both are checked — a cycle in either is a refusal, not a traversal that
//! happens to terminate.
//!
//! `blocked` is **derived, never stored** (DEC-060). There is no field to set and
//! no setter to call: [`InquiryMap::is_blocked`] answers from the `needs` edges
//! and the lifecycle of what they point at, so a stale blocker is unrepresentable
//! rather than merely unlikely.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::ids::{DesignId, Fingerprint};
use super::refusal::Refusal;

/// Where a node came from, kept first-class so a tidy map cannot launder
/// agent-proposed structure into user-directed intent (design R2/R12).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "provenance")]
pub(crate) enum Provenance {
    /// The user raised or pinned this line of inquiry.
    UserDirected,
    /// The agent proposed it while decomposing.
    AgentProposed,
    /// Seeded from a direct non-terminal shaping QUE (DEC-085).
    ShapingQuestion { record: String },
    /// Imported from a conventional `OQ-*` entry in the authored Open Questions
    /// section — an *unverified* proposal until the run establishes evidence
    /// (DEC-085, design R9).
    ///
    /// DEC-085 requires the whole triple — **label**, **location**, and content
    /// **fingerprint**. `label` is the entry's own (`OQ-1`), which the question
    /// text no longer carries because the parse strips it to find the headline;
    /// it is the only thing distinguishing two entries whose text is identical.
    /// `fingerprint` digests the **headline alone** — the exact bytes stored as
    /// `question` — never the continuation lines the importer deliberately
    /// leaves in the section body (PHASE-15 D8).
    ImportedProse {
        section: DesignId,
        line: u32,
        label: String,
        fingerprint: Fingerprint,
    },
}

impl Provenance {
    /// [`Provenance::UserDirected`]'s label.
    pub(crate) const USER_DIRECTED: &'static str = "user-directed";
    /// [`Provenance::AgentProposed`]'s label.
    pub(crate) const AGENT_PROPOSED: &'static str = "agent-proposed";
    /// [`Provenance::ShapingQuestion`]'s label.
    pub(crate) const SHAPING_QUESTION: &'static str = "shaping-question";
    /// [`Provenance::ImportedProse`]'s label.
    pub(crate) const IMPORTED_PROSE: &'static str = "imported-prose";

    /// The closed-vocabulary label this provenance renders as on a change row.
    /// Bounded at admission by [`super::bounds::DESIGN_STAGE_LABEL_BYTES`] — the
    /// longest member, `shaping-question`, is 16 B.
    pub(crate) const fn label(&self) -> &'static str {
        match self {
            Provenance::UserDirected => Self::USER_DIRECTED,
            Provenance::AgentProposed => Self::AGENT_PROPOSED,
            Provenance::ShapingQuestion { .. } => Self::SHAPING_QUESTION,
            Provenance::ImportedProse { .. } => Self::IMPORTED_PROSE,
        }
    }
}

/// A node's lifecycle (design §5.3). Orthogonal to [`super::Stage`] — collapsing
/// the two is exactly the accidental hierarchical state machine R3 forbids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum InquiryLifecycle {
    Open,
    Resolved,
    Deferred,
    Pruned,
}

impl InquiryLifecycle {
    /// The kebab token this lifecycle is spelled with everywhere (STD-001).
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            InquiryLifecycle::Open => "open",
            InquiryLifecycle::Resolved => "resolved",
            InquiryLifecycle::Deferred => "deferred",
            InquiryLifecycle::Pruned => "pruned",
        }
    }
}

/// How a resolved node was disposed (DEC-062).
///
/// Resolution *requires* one of these; there is no bare `Resolved`. Accepted
/// truth stays user-owned, so "we discussed it" is not a disposition — the
/// intentionally non-durable case is declared, not defaulted into.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "disposition")]
pub(crate) enum Disposition {
    /// A durable record was created for this inquiry.
    Created { record: String },
    /// An existing canonical record was adopted.
    Adopted { record: String },
    /// The outcome is explicitly retained as unresolved.
    RetainedUnresolved { note: String },
    /// The exchange is intentionally non-durable.
    NonDurable { note: String },
}

impl Disposition {
    /// Which of the four forms this is.
    pub(crate) const fn form(&self) -> DispositionForm {
        match self {
            Disposition::Created { .. } => DispositionForm::Create,
            Disposition::Adopted { .. } => DispositionForm::Adopt,
            Disposition::RetainedUnresolved { .. } => DispositionForm::RetainUnresolved,
            Disposition::NonDurable { .. } => DispositionForm::NonDurable,
        }
    }

    /// The canonical record this disposition names, for the two forms that name
    /// one. The note-bearing forms name none, and that is the whole point of
    /// their existing: a resolved node that produces no record is representable.
    pub(crate) fn record(&self) -> Option<&str> {
        match self {
            Disposition::Created { record } | Disposition::Adopted { record } => Some(record),
            Disposition::RetainedUnresolved { .. } | Disposition::NonDurable { .. } => None,
        }
    }
}

/// The closed four-member vocabulary [`Disposition`] ranges over (DEC-062).
///
/// Separate from [`Disposition`] because a *refusal* has to name the whole
/// vocabulary without holding a member of it — "resolve declared none of these
/// four" cannot be spelled from an instance. One owner for the four tokens
/// (STD-001), so the wire tag, the refusal text, the change-row label and the
/// admission arm cannot drift apart.
///
/// The spellings are chosen to fit
/// [`super::bounds::DESIGN_STAGE_LABEL_BYTES`], because the form rides a change
/// row as a [`super::change_log::ValueKind::Label`] term and a label is
/// **refused, never trimmed**. `retain-unresolved` would be 17 B and refuse
/// itself; the token is `unresolved`, and the compile-time assertion below is
/// what makes that a checked fact rather than a hope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DispositionForm {
    Create,
    Adopt,
    RetainUnresolved,
    NonDurable,
}

impl DispositionForm {
    /// Every form — the closed vocabulary, single-sourced so a refusal that lists
    /// them cannot miss one (STD-001).
    pub(crate) const ALL: [DispositionForm; 4] = [
        DispositionForm::Create,
        DispositionForm::Adopt,
        DispositionForm::RetainUnresolved,
        DispositionForm::NonDurable,
    ];

    /// The kebab token this form is spelled with everywhere — the wire tag, the
    /// change-row label, the refusal listing.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            DispositionForm::Create => "create",
            DispositionForm::Adopt => "adopt",
            DispositionForm::RetainUnresolved => "unresolved",
            DispositionForm::NonDurable => "non-durable",
        }
    }

    /// The four tokens as one comma-separated list, for a refusal that must name
    /// the whole vocabulary.
    pub(crate) fn vocabulary() -> String {
        DispositionForm::ALL
            .into_iter()
            .map(DispositionForm::as_str)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// The disposition label fits its admission bound, **proved rather than
/// asserted** (EX-16(a)): the form vocabulary is closed, so a fifth form — or a
/// re-spelling of one of these four — that outgrew the label bound stops the
/// build instead of refusing itself at runtime, in one branch, later.
const _: () =
    assert!(widest_form(&DispositionForm::ALL) <= super::bounds::DESIGN_STAGE_LABEL_BYTES);

/// The widest disposition label, at compile time.
const fn widest_form(rest: &[DispositionForm]) -> usize {
    match rest {
        [] => 0,
        [head, tail @ ..] => {
            let head = head.as_str().len();
            let tail = widest_form(tail);
            if head > tail { head } else { tail }
        }
    }
}

/// One inquiry-map node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct InquiryNode {
    id: DesignId,
    question: String,
    provenance: Provenance,
    lifecycle: InquiryLifecycle,
    disposition: Option<Disposition>,
    parent: Option<DesignId>,
    needs: BTreeSet<DesignId>,
    /// Whether this question blocks the run (`SL-264` sec-3).
    ///
    /// **A shape fact, not progress** — it is a member of [`NodeMaterial`], so a
    /// flip on a covered node re-faces the human through the coverage comparison.
    /// The judgement is a property of the node rather than of a free-standing act
    /// because the user reviews it *in* the map, not beside it.
    ///
    /// `None` means **unjudged**, readable by design rather than by accident: an
    /// omission is refused where a node is created, so only a node that predates
    /// this change can hold it, and its effective judgement still falls back to
    /// the stored legacy declaration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    blocking: Option<bool>,
    /// Creation order, assigned from the snapshot's own counter
    /// ([`super::snapshot::MapGroup::claim_seq`]).
    ///
    /// **Persisted model state, not container iteration** (projection-bounds
    /// sketch §(c)): the frontier's third rank key is "the order these nodes
    /// were raised in", and reading that off a map's iteration order would lose
    /// the determinism PHASE-02 EX-5 requires. It is pure — derived from
    /// snapshot state, never from a clock or an rng.
    #[serde(default)]
    seq: u64,
}

impl InquiryNode {
    /// A new open node, unsequenced. [`InquiryNode::sequenced`] places it in
    /// creation order; a node that never receives one sorts first, which is the
    /// honest reading of "raised before the counter existed".
    ///
    /// The judgement is a **parameter and not a builder**, so no creation path
    /// can omit it by construction (`SL-264` sec-3, `RV-386` `F-10`): a defaulted
    /// field would let a new door into the map be born unjudged, which is the
    /// omission the wire refuses. [`Option`] rather than `bool` because a node
    /// that predates the attribute is legitimately unjudged, and reconstruction
    /// ([`super::run`]'s `rebuild`) carries that state forward.
    pub(crate) fn open(
        id: DesignId,
        question: impl Into<String>,
        provenance: Provenance,
        blocking: Option<bool>,
    ) -> Self {
        InquiryNode {
            id,
            question: question.into(),
            provenance,
            lifecycle: InquiryLifecycle::Open,
            disposition: None,
            parent: None,
            needs: BTreeSet::new(),
            blocking,
            seq: 0,
        }
    }

    /// Place this node at `seq` in creation order.
    #[must_use]
    pub(crate) const fn sequenced(mut self, seq: u64) -> Self {
        self.seq = seq;
        self
    }

    /// Where this node sits in creation order.
    pub(crate) const fn seq(&self) -> u64 {
        self.seq
    }

    /// Set the primary parent.
    #[must_use]
    pub(crate) fn with_parent(mut self, parent: DesignId) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Add a `needs` dependency.
    #[must_use]
    pub(crate) fn needing(mut self, other: DesignId) -> Self {
        self.needs.insert(other);
        self
    }

    /// This node's id.
    pub(crate) const fn id(&self) -> &DesignId {
        &self.id
    }

    /// The concise question.
    pub(crate) fn question(&self) -> &str {
        &self.question
    }

    /// Where this node came from.
    pub(crate) const fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    /// Current lifecycle.
    pub(crate) const fn lifecycle(&self) -> InquiryLifecycle {
        self.lifecycle
    }

    /// The recorded disposition, if resolved.
    pub(crate) const fn disposition(&self) -> Option<&Disposition> {
        self.disposition.as_ref()
    }

    /// The primary parent, if any.
    pub(crate) const fn parent(&self) -> Option<&DesignId> {
        self.parent.as_ref()
    }

    /// The sparse `needs` set.
    pub(crate) const fn needs(&self) -> &BTreeSet<DesignId> {
        &self.needs
    }

    /// The stored blocking judgement, or `None` where the node predates it.
    pub(crate) const fn blocking(&self) -> Option<bool> {
        self.blocking
    }

    /// Whether this node's **effective** judgement is blocking (`SL-264` sec-3).
    ///
    /// Routes through [`judged_blocking`], the one expression of the judgement,
    /// so the derived read and the `ReviewedGraph` coverage cannot disagree about
    /// which nodes block (`RV-386` F-13). Pure, and the legacy set is **passed
    /// in** rather than fetched: this leaf module takes no dependence on the act
    /// that holds the set to answer the question (`ADR-001`).
    pub(crate) fn effective_blocking(&self, legacy: &BTreeSet<DesignId>) -> bool {
        judged_blocking(self.blocking, &self.id, legacy)
    }

    /// What this node is made of, for coverage purposes — everything except the
    /// two fields [`NodeMaterial`] excludes and the `id` the map keys on.
    fn material(&self) -> NodeMaterial {
        NodeMaterial {
            question: self.question.clone(),
            provenance: self.provenance.clone(),
            parent: self.parent.clone(),
            needs: self.needs.clone(),
            blocking: self.blocking,
            seq: self.seq,
        }
    }

    /// Move to `resolved`, which is possible only with a semantic disposition
    /// (DEC-062). The disposition is an argument rather than a settable field,
    /// so resolution without one does not compile at the call site and is
    /// refused at the data boundary.
    #[must_use]
    pub(crate) fn resolve(mut self, disposition: Disposition) -> Self {
        self.lifecycle = InquiryLifecycle::Resolved;
        self.disposition = Some(disposition);
        self
    }

    /// Move to a non-resolved lifecycle.
    ///
    /// Refuses `resolved` — that transition owns a disposition and belongs to
    /// [`InquiryNode::resolve`]. This is the check that makes
    /// "resolved without a disposition" unreachable through *either* route.
    pub(crate) fn transition(mut self, lifecycle: InquiryLifecycle) -> Result<Self, Refusal> {
        if lifecycle == InquiryLifecycle::Resolved {
            return Err(Refusal::DispositionMissing { id: self.id });
        }
        self.lifecycle = lifecycle;
        self.disposition = None;
        Ok(self)
    }
}

/// What an inquiry-map coverage compares, per node (DEC-121).
///
/// Persisted inside `ContentCoverage<NodeMaterial>`, so it is stored and compared
/// by `Eq` — and carries no digest of its own, which is the whole point of the
/// variant: nodes are mutated by pure code after any shell digest would have been
/// taken, so material is the only trustworthy thing to compare.
///
/// **Deliberately not `lifecycle` and not `disposition`.** What the user reviewed
/// is the set of questions and how they relate; a question later being answered
/// is progress *through* that graph rather than a change *to* it. Admitting
/// disposition here would expire the sufficiency acceptance on the next disposal
/// — precisely what the following stage does — while double-guarding a fact
/// `blocking-inquiries-dispositioned` already owns.
///
/// `id` is absent because it is the covered map's key, not part of what a node is
/// compared at. Nothing here is new state: every field is [`InquiryNode`]'s own.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NodeMaterial {
    question: String,
    provenance: Provenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parent: Option<DesignId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    needs: BTreeSet<DesignId>,
    /// The blocking judgement is material, deliberately (`SL-264` sec-3): the
    /// user reviews the blocking marks *within* the graph, so a flipped judgement
    /// is a change to what they were shown rather than progress through it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    blocking: Option<bool>,
    #[serde(default)]
    seq: u64,
}

impl NodeMaterial {
    /// Whether the node this material describes is **effectively** blocking, at
    /// the judgement it carried (`SL-264` sec-2, sec-3).
    ///
    /// The carried-map sibling of [`InquiryNode::effective_blocking`], reading the
    /// same judgement so a `ReviewedGraph` coverage compares the act's carried
    /// blocking membership against the run's current one without a second
    /// expression of *which nodes block* (`RV-386` F-13).
    pub(crate) fn effective_blocking(&self, id: &DesignId, legacy: &BTreeSet<DesignId>) -> bool {
        judged_blocking(self.blocking, id, legacy)
    }
}

/// The one **effective** judgement (`SL-264` sec-3): a node's own `Some(bool)`
/// when it holds one, else its membership in the stored legacy
/// `blocking-set-declared` set.
///
/// The fallback is **per node** (`RV-386` F-8): an unjudged node is blocking
/// exactly when the set the run recorded before this attribute existed named it,
/// so a partly-judged map keeps every unjudged blocker, and a node leaves the
/// fallback only by being judged itself.
///
/// One home for the judgement, so its two readers — [`InquiryNode`] and its
/// [`NodeMaterial`] — cannot disagree about which nodes block (`RV-386` F-13).
/// Pure, and the legacy set is passed in.
fn judged_blocking(judgement: Option<bool>, id: &DesignId, legacy: &BTreeSet<DesignId>) -> bool {
    judgement.unwrap_or_else(|| legacy.contains(id))
}

/// The **marks**: the ids in `materials` whose node is effectively blocking,
/// whatever its lifecycle (`SL-264` sec-3) — the set the user reviews.
///
/// The one builder of the set (`RV-389` F-4), over materials because that is the
/// representation both sides of a [`Coverage::ReviewedGraph`] comparison hold —
/// the act's carried covered map and the run's current full map — so a key
/// absent from the carried map reads as not blocking and a *new* blocking node is
/// the only thing an addition contributes. Each member is [`judged_blocking`]'s,
/// the judgement the gate's open-blocker read also takes, so the coverage and the
/// gate cannot disagree about which nodes block (`RV-386` F-13).
///
/// [`Coverage::ReviewedGraph`]: super::gate::Coverage::ReviewedGraph
pub(crate) fn blocking_marks<'a>(
    materials: &'a BTreeMap<DesignId, NodeMaterial>,
    legacy: &BTreeSet<DesignId>,
) -> BTreeSet<&'a DesignId> {
    materials
        .iter()
        .filter(|(id, material)| material.effective_blocking(id, legacy))
        .map(|(id, _)| id)
        .collect()
}

/// The inquiry map: nodes plus the two acyclic edge relations over them.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct InquiryMap {
    nodes: BTreeMap<DesignId, InquiryNode>,
}

impl InquiryMap {
    /// Insert a node, refusing an edge that names an unknown node or closes a
    /// cycle in *either* relation.
    pub(crate) fn insert(&mut self, node: InquiryNode) -> Result<(), Refusal> {
        for target in node.parent().into_iter().chain(node.needs()) {
            if !self.nodes.contains_key(target) && target != node.id() {
                return Err(Refusal::UnknownNode { id: target.clone() });
            }
        }
        let id = node.id().clone();
        let previous = self.nodes.insert(id.clone(), node);
        if let Some(closing) = self.first_cycle_edge(&id) {
            match previous {
                Some(displaced) => {
                    self.nodes.insert(id, displaced);
                }
                None => {
                    self.nodes.remove(&id);
                }
            }
            return Err(closing);
        }
        Ok(())
    }

    /// A node by id.
    pub(crate) fn get(&self, id: &DesignId) -> Option<&InquiryNode> {
        self.nodes.get(id)
    }

    /// What every node is currently *made of* — the observation an inquiry-map
    /// coverage is evaluated against, and the sibling of
    /// [`SectionGroup::fingerprints`](super::snapshot::SectionGroup::fingerprints)
    /// on the section side.
    ///
    /// Pure, and it has to be: `DerivedInput` is built before `apply` runs the
    /// batch, so a shell-supplied digest of this map would have been taken
    /// *before* the very mutations it is meant to observe.
    pub(crate) fn materials(&self) -> BTreeMap<DesignId, NodeMaterial> {
        self.nodes
            .iter()
            .map(|(id, node)| (id.clone(), node.material()))
            .collect()
    }

    /// Node count.
    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the map holds no nodes.
    #[expect(
        dead_code,
        reason = "SL-233: the §9.1 suite does not reach this; PHASE-03/04 are its first callers"
    )]
    pub(crate) fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// The `needs` targets of `id` that are **not yet settled** — `open` or
    /// `deferred`; `resolved` and `pruned` both settle a dependency.
    ///
    /// The **one** expression of *which dependencies still hold* (`SL-266`
    /// `EX-1`): [`Self::is_blocked`] is this read's non-emptiness and nothing
    /// else, and the envelope's `MapNode.blocked_by` is this read's output. One
    /// home, so the blocked mark, the count and the reader that names the
    /// blockers cannot disagree (`RV-386` F-13's argument, applied here).
    ///
    /// Total: an id the map does not hold, and a node with no `needs` edges,
    /// both yield no unsettled target rather than an error.
    pub(crate) fn unsettled_needs(&self, id: &DesignId) -> Vec<&DesignId> {
        let Some(node) = self.nodes.get(id) else {
            return Vec::new();
        };
        node.needs()
            .iter()
            .filter(|needed| {
                self.nodes.get(needed).is_some_and(|target| {
                    matches!(
                        target.lifecycle(),
                        InquiryLifecycle::Open | InquiryLifecycle::Deferred
                    )
                })
            })
            .collect()
    }

    /// Whether `id` is blocked — **derived**, never stored (DEC-060).
    ///
    /// A node is blocked when anything it needs is not yet settled; that is
    /// [`Self::unsettled_needs`] and there is no second rule here.
    pub(crate) fn is_blocked(&self, id: &DesignId) -> bool {
        !self.unsettled_needs(id).is_empty()
    }

    /// Every node, in id order.
    pub(crate) fn nodes(&self) -> impl Iterator<Item = &InquiryNode> {
        self.nodes.values()
    }

    /// How many nodes depend on `id` through a `needs` edge — the frontier's
    /// second rank key, because a node many others depend on is more
    /// consequential than a leaf.
    pub(crate) fn needs_in_degree(&self, id: &DesignId) -> usize {
        self.nodes
            .values()
            .filter(|node| node.needs().contains(id))
            .count()
    }

    /// Every node currently blocked.
    pub(crate) fn blocked(&self) -> impl Iterator<Item = &InquiryNode> {
        self.nodes
            .values()
            .filter(|node| self.is_blocked(node.id()))
    }

    /// The **open blockers**: the nodes whose effective judgement is blocking and
    /// whose lifecycle is not `Resolved` — the inquiries the run still owes a
    /// disposition (`SL-264` sec-3).
    ///
    /// `Resolved` is the only lifecycle that can carry a [`Disposition`]
    /// (DEC-062), so *has a disposition* and *is resolved* are one question.
    pub(crate) fn open_blockers<'a>(
        &'a self,
        legacy: &'a BTreeSet<DesignId>,
    ) -> impl Iterator<Item = &'a InquiryNode> {
        self.nodes.values().filter(move |node| {
            node.effective_blocking(legacy) && node.lifecycle() != InquiryLifecycle::Resolved
        })
    }

    /// The first cycle reachable from `start` through either relation, as the
    /// refusal naming the edge that closes it.
    fn first_cycle_edge(&self, start: &DesignId) -> Option<Refusal> {
        self.walk_for_cycle(start, &mut BTreeSet::new(), &mut BTreeSet::new(), &|node| {
            node.parent().into_iter().collect()
        })
        .or_else(|| {
            self.walk_for_cycle(start, &mut BTreeSet::new(), &mut BTreeSet::new(), &|node| {
                node.needs().iter().collect()
            })
        })
    }

    /// Depth-first walk of one relation looking for a back edge into the current
    /// path. Both relations use the same walk — a cycle is a cycle, and
    /// duplicating the traversal per edge kind is how one of them drifts.
    ///
    /// `path` and `settled` are distinct on purpose. A back edge into `path` is a
    /// cycle; re-reaching a node already `settled` is a diamond, which `needs`
    /// makes routine (`a → b`, `a → c`, `b → d`, `c → d`). Collapsing the two into
    /// one visited set reports every diamond as a cycle.
    fn walk_for_cycle<'a>(
        &'a self,
        at: &'a DesignId,
        path: &mut BTreeSet<&'a DesignId>,
        settled: &mut BTreeSet<&'a DesignId>,
        edges: &dyn Fn(&'a InquiryNode) -> Vec<&'a DesignId>,
    ) -> Option<Refusal> {
        if settled.contains(at) {
            return None;
        }
        path.insert(at);
        if let Some(node) = self.nodes.get(at) {
            for target in edges(node) {
                if path.contains(target) {
                    return Some(Refusal::CyclicEdge {
                        from: at.clone(),
                        to: target.clone(),
                    });
                }
                if let Some(found) = self.walk_for_cycle(target, path, settled, edges) {
                    return Some(found);
                }
            }
        }
        path.remove(at);
        settled.insert(at);
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{Disposition, InquiryLifecycle, InquiryMap, InquiryNode, Provenance};
    use crate::design_run::fixture::id;
    use crate::design_run::ids::DesignId;

    /// One map holding a `needs` target of every lifecycle, plus a node waiting
    /// on all four — the row set the one blocked derivation ranges over.
    ///
    /// Node ids are chosen so `BTreeSet`'s order is also a readable one:
    /// `deferred` sorts before `open`, so the settled/unsettled split is visible
    /// in the returned vector without a sort here.
    fn waiting_on_every_lifecycle() -> (InquiryMap, DesignId) {
        let settled = |record: &str| Disposition::Created {
            record: record.to_owned(),
        };
        let mut map = InquiryMap::default();
        let (open, deferred) = (id("inq-open"), id("inq-deferred"));
        let (resolved, pruned) = (id("inq-resolved"), id("inq-pruned"));
        for node in [
            InquiryNode::open(
                open.clone(),
                "still open?",
                Provenance::UserDirected,
                Some(false),
            ),
            InquiryNode::open(
                deferred.clone(),
                "later?",
                Provenance::AgentProposed,
                Some(false),
            )
            .transition(InquiryLifecycle::Deferred)
            .expect("deferred is not `resolved`"),
            InquiryNode::open(
                resolved.clone(),
                "answered?",
                Provenance::AgentProposed,
                Some(false),
            )
            .resolve(settled("DEC-999")),
            InquiryNode::open(
                pruned.clone(),
                "dropped?",
                Provenance::AgentProposed,
                Some(false),
            )
            .transition(InquiryLifecycle::Pruned)
            .expect("pruned is not `resolved`"),
        ] {
            map.insert(node).expect("a well-formed node inserts");
        }
        let waiting = id("inq-waiting");
        map.insert(
            InquiryNode::open(
                waiting.clone(),
                "waits on all four?",
                Provenance::AgentProposed,
                Some(true),
            )
            .needing(open)
            .needing(deferred)
            .needing(resolved)
            .needing(pruned),
        )
        .expect("a well-formed node inserts");
        (map, waiting)
    }

    /// `EX-1` — the derivation ranges over exactly `open` and `deferred`.
    ///
    /// The settled half is asserted too, and it is not decoration: a version
    /// that returned *every* target passes the unsettled half alone, and a
    /// version that returned only `open` passes neither.
    #[test]
    fn unsettled_needs_returns_open_and_deferred_targets() {
        let (map, waiting) = waiting_on_every_lifecycle();
        let unsettled: Vec<&str> = map
            .unsettled_needs(&waiting)
            .into_iter()
            .map(DesignId::as_str)
            .collect();
        assert_eq!(
            unsettled,
            ["inq-deferred", "inq-open"],
            "`resolved` and `pruned` settle a dependency; `open` and `deferred` do not"
        );
        // A node with no `needs` edges at all is unblocked for the same reason:
        // an empty edge set, not a second rule.
        assert!(map.unsettled_needs(&id("inq-open")).is_empty());
        // And the read is total: an id the map does not hold is not blocked.
        assert!(map.unsettled_needs(&id("inq-absent")).is_empty());
    }

    /// `EX-1` — `is_blocked` is exactly the non-emptiness of `unsettled_needs`,
    /// for every node, so no second blocked derivation survives.
    ///
    /// The equivalence is asserted over the whole map rather than at one node,
    /// because a second derivation would agree on one fixture and drift on the
    /// next. The last half is the behaviour the derivation must have: settling
    /// the last unsettled dependency unblocks the waiter without touching it.
    #[test]
    fn unsettled_needs_is_the_one_blocked_derivation() {
        let (mut map, waiting) = waiting_on_every_lifecycle();
        assert!(map.is_blocked(&waiting), "an open dependency blocks");
        for node in map.nodes() {
            assert_eq!(
                map.is_blocked(node.id()),
                !map.unsettled_needs(node.id()).is_empty(),
                "`is_blocked` is `unsettled_needs` non-emptiness at {}",
                node.id()
            );
        }

        let before = map.get(&waiting).expect("present").clone();
        for (raw, record) in [("inq-deferred", "DEC-998"), ("inq-open", "DEC-997")] {
            let settled =
                map.get(&id(raw))
                    .expect("present")
                    .clone()
                    .resolve(Disposition::Created {
                        record: record.to_owned(),
                    });
            map.insert(settled).expect("a well-formed node inserts");
        }
        assert!(!map.is_blocked(&waiting));
        assert!(map.unsettled_needs(&waiting).is_empty());
        assert_eq!(
            map.get(&waiting),
            Some(&before),
            "the waiter's value is byte-identical across the change — blocked is not a field"
        );
    }
}
