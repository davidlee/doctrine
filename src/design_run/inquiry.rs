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
    /// The closed-vocabulary label this provenance renders as on a change row.
    /// Bounded at admission by [`super::bounds::DESIGN_STAGE_LABEL_BYTES`] — the
    /// longest member, `shaping-question`, is 16 B.
    pub(crate) const fn label(&self) -> &'static str {
        match self {
            Provenance::UserDirected => "user-directed",
            Provenance::AgentProposed => "agent-proposed",
            Provenance::ShapingQuestion { .. } => "shaping-question",
            Provenance::ImportedProse { .. } => "imported-prose",
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
    pub(crate) fn open(id: DesignId, question: impl Into<String>, provenance: Provenance) -> Self {
        InquiryNode {
            id,
            question: question.into(),
            provenance,
            lifecycle: InquiryLifecycle::Open,
            disposition: None,
            parent: None,
            needs: BTreeSet::new(),
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

    /// What this node is made of, for coverage purposes — everything except the
    /// two fields [`NodeMaterial`] excludes and the `id` the map keys on.
    fn material(&self) -> NodeMaterial {
        NodeMaterial {
            question: self.question.clone(),
            provenance: self.provenance.clone(),
            parent: self.parent.clone(),
            needs: self.needs.clone(),
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
    #[serde(default)]
    seq: u64,
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

    /// Whether `id` is blocked — **derived**, never stored (DEC-060).
    ///
    /// A node is blocked when anything it needs is not yet settled. `pruned` and
    /// `resolved` both settle a dependency; `open` and `deferred` do not.
    pub(crate) fn is_blocked(&self, id: &DesignId) -> bool {
        let Some(node) = self.nodes.get(id) else {
            return false;
        };
        node.needs().iter().any(|needed| {
            self.nodes.get(needed).is_some_and(|target| {
                matches!(
                    target.lifecycle(),
                    InquiryLifecycle::Open | InquiryLifecycle::Deferred
                )
            })
        })
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
