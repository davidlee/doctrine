// SPDX-License-Identifier: GPL-3.0-only
//! Bounded delegation — one exported obligation, one attributed proposal back
//! (SL-233 PHASE-10, design §5.4, DEC-058/DEC-068).
//!
//! Its own module rather than a corner of [`super::attestation`] because it is a
//! separate state model (R3, DEC-065) *and* because it is the one state model that
//! stores a caller's [`Declaration`]s — so it depends on [`super::submission`],
//! which the review and recovery models do not. Keeping that edge in one file is
//! what stops it becoming a mutual dependency between two modules.
//!
//! # The two properties the shape carries
//!
//! **Proposal-only (DEC-068).** A [`Proposal`] holds the delegate's declarations
//! *verbatim and unapplied*. Nothing here applies them; [`super::run::apply`]
//! merges them into its batch only on the coordinator's `accept`, where they land
//! as the coordinator's own act through the one declaration engine. There is no
//! second route by which a delegate's bytes can reach the map, so "the
//! coordinator is sole writer" is a property of where the data sits rather than
//! of a check somebody has to remember.
//!
//! **Bound to what was assigned (DEC-066, applied to a subject with no
//! fingerprint).** A section attestation stays live while its subject's stored
//! fingerprint still matches. An inquiry node has no fingerprint — so a
//! delegation stores *the obligation itself*, as it stood when the assignment was
//! cut, and a proposal is stale exactly when the map no longer holds that node.
//! Comparing the value is pure, exact, and needs no digest the pure layer may not
//! compute.

use serde::{Deserialize, Serialize};

use super::ids::DesignId;
use super::inquiry::{InquiryMap, InquiryNode};
use super::submission::Declaration;

/// A delegated obligation's state (DEC-068).
///
/// Delegation is proposal-only: the coordinator is the sole writer, and a stale
/// proposal is refused rather than rebased. The vocabulary carries that — there
/// is no `applied` a delegate can reach on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DelegationState {
    /// Exported to a delegate, awaiting a proposal.
    Outstanding,
    /// A proposal came back and is awaiting the coordinator.
    Proposed,
    /// The coordinator accepted it.
    Accepted,
    /// The coordinator refused it, with a reason.
    Refused,
}

/// What a delegate proposed back: who did the work, what they concluded, and the
/// map changes they propose.
///
/// `by` is **attribution, not authentication**. `EX-1` asks for an attributed
/// proposal and v1 authenticates nothing anywhere — the lock acceptance carries
/// the same limit and says so out loud. It is stored exactly as supplied and never
/// re-derived from the coordinator's environment, because a value the coordinator
/// filled in would be the coordinator's claim wearing the delegate's name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Proposal {
    by: String,
    summary: String,
    /// The proposed map changes, **unapplied**. Held as the wire type the
    /// coordinator's own declarations arrive in, so acceptance can put them
    /// through the same engine rather than a second interpreter.
    #[serde(default, rename = "declare")]
    declarations: Vec<Declaration>,
}

impl Proposal {
    /// A proposal, exactly as the delegate submitted it.
    pub(crate) fn of(
        by: impl Into<String>,
        summary: impl Into<String>,
        declarations: Vec<Declaration>,
    ) -> Self {
        Proposal {
            by: by.into(),
            summary: summary.into(),
            declarations,
        }
    }

    /// Who says they did the work.
    pub(crate) fn by(&self) -> &str {
        &self.by
    }

    /// What they concluded — prose, never interpreted.
    pub(crate) fn summary(&self) -> &str {
        &self.summary
    }

    /// The map changes proposed and not yet applied.
    pub(crate) fn declarations(&self) -> &[Declaration] {
        &self.declarations
    }
}

/// One exported assignment and its life (DEC-068).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Delegation {
    id: DesignId,
    /// The obligation **as it stood when the assignment was cut**. This is the
    /// binding, not a convenience copy: it is what a proposal's currency is
    /// measured against, and it is what makes the exported assignment
    /// self-contained (`EX-1`) — the question travels with the delegation instead
    /// of being re-read out of a map that may have moved.
    assigned: InquiryNode,
    /// The revision the assignment was cut at.
    exported_at: u64,
    state: DelegationState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    proposal: Option<Proposal>,
    /// The coordinator's stated reason for refusing. Stored rather than left to
    /// the change log alone: the log is bounded and evicts, and *why the
    /// delegate's work was turned down* outlives a retention window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    refused_because: Option<String>,
}

impl Delegation {
    /// Cut an assignment for `assigned` at `exported_at`.
    pub(crate) const fn exported(id: DesignId, assigned: InquiryNode, exported_at: u64) -> Self {
        Delegation {
            id,
            assigned,
            exported_at,
            state: DelegationState::Outstanding,
            proposal: None,
            refused_because: None,
        }
    }

    /// This delegation's id.
    pub(crate) const fn id(&self) -> &DesignId {
        &self.id
    }

    /// The obligation it was cut for.
    pub(crate) const fn obligation(&self) -> &DesignId {
        self.assigned.id()
    }

    /// The obligation's question, as assigned.
    pub(crate) fn question(&self) -> &str {
        self.assigned.question()
    }

    /// The revision it was cut at.
    pub(crate) const fn exported_at(&self) -> u64 {
        self.exported_at
    }

    /// Where it stands.
    pub(crate) const fn state(&self) -> DelegationState {
        self.state
    }

    /// The proposal it holds, if one has come back. Present whatever the state:
    /// an accepted, refused or stale proposal is still the delegate's work and is
    /// still readable (`EX-3`, design §9.2).
    pub(crate) const fn proposal(&self) -> Option<&Proposal> {
        self.proposal.as_ref()
    }

    /// Why the coordinator refused it.
    pub(crate) fn refused_because(&self) -> Option<&str> {
        self.refused_because.as_deref()
    }

    /// Whether the obligation is no longer the one that was assigned.
    ///
    /// Exact equality against the whole stored node, and deliberately not a
    /// comparison of selected fields: *any* difference means the delegate worked
    /// an obligation the run no longer holds, and a hand-picked field set would be
    /// a list to forget to extend. An absent node is stale too — it cannot be the
    /// node that was assigned.
    pub(crate) fn is_stale(&self, map: &InquiryMap) -> bool {
        map.get(self.obligation()) != Some(&self.assigned)
    }

    /// Record the proposal that came back. Replaces any earlier one: a second
    /// proposal against one assignment is the delegate's revised answer, and
    /// keeping both would raise the question of which the coordinator accepts.
    #[must_use]
    pub(crate) fn proposed(mut self, proposal: Proposal) -> Self {
        self.proposal = Some(proposal);
        self.state = DelegationState::Proposed;
        self
    }

    /// The coordinator accepts. The proposal is kept, not consumed — `EX-1`'s
    /// attribution has to survive the crossing to be worth anything.
    #[must_use]
    pub(crate) const fn accepted(mut self) -> Self {
        self.state = DelegationState::Accepted;
        self
    }

    /// The coordinator refuses, on the record.
    #[must_use]
    pub(crate) fn refused(mut self, reason: impl Into<String>) -> Self {
        self.state = DelegationState::Refused;
        self.refused_because = Some(reason.into());
        self
    }
}

/// Every delegation the run holds, ordered by id so serialisation is
/// deterministic.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DelegationGroup {
    #[serde(default, rename = "delegation")]
    pub(crate) delegations: Vec<Delegation>,
}

impl DelegationGroup {
    /// The delegation with `id`, if the run holds one.
    pub(crate) fn find(&self, id: &str) -> Option<&Delegation> {
        self.delegations.iter().find(|held| held.id.as_str() == id)
    }

    /// The assignment for `obligation` that is still awaiting the coordinator, if
    /// there is one.
    ///
    /// What bounds "one bounded obligation": a second assignment may be cut once
    /// the first is disposed, and not while the delegate it was given to may still
    /// answer. Returns the delegation rather than a boolean because the refusal
    /// has to name which assignment holds the obligation.
    pub(crate) fn outstanding_for(&self, obligation: &DesignId) -> Option<&Delegation> {
        self.delegations.iter().find(|held| {
            held.obligation() == obligation
                && matches!(
                    held.state,
                    DelegationState::Outstanding | DelegationState::Proposed
                )
        })
    }

    /// Insert or replace a delegation, keeping the group id-ordered.
    pub(crate) fn upsert(&mut self, delegation: Delegation) {
        self.delegations.retain(|held| held.id != delegation.id);
        self.delegations.push(delegation);
        self.delegations.sort_by(|a, b| a.id.cmp(&b.id));
    }
}

#[cfg(test)]
mod tests {
    use super::super::inquiry::{InquiryLifecycle, Provenance};
    use super::*;

    /// A well-formed id, or a failure naming the bad literal.
    fn id(raw: &str) -> DesignId {
        DesignId::parse(raw).expect("test fixture id must be well-formed")
    }

    /// A map holding one open obligation.
    fn map_with_obligation() -> (InquiryMap, InquiryNode) {
        let node = InquiryNode::open(
            id("inq-1"),
            "does it need a transport?",
            Provenance::AgentProposed,
        )
        .sequenced(1);
        let mut map = InquiryMap::default();
        map.insert(node.clone()).expect("a fresh node inserts");
        (map, node)
    }

    #[test]
    fn a_proposal_is_stale_exactly_when_the_obligation_is_not_the_one_assigned() {
        let (mut map, node) = map_with_obligation();
        let delegation = Delegation::exported(id("dlg-1"), node.clone(), 7);
        assert!(
            !delegation.is_stale(&map),
            "an untouched obligation is not stale"
        );

        // Every axis of the node counts, because the binding is the node: a
        // lifecycle move, an edge change, and a rewritten question are all
        // "the obligation is not what you were given".
        let deferred = node
            .clone()
            .transition(InquiryLifecycle::Deferred)
            .expect("deferring an open node is lawful");
        map.insert(deferred).expect("replacing a node is lawful");
        assert!(delegation.is_stale(&map));

        let (mut map, node) = map_with_obligation();
        map.insert(
            InquiryNode::open(
                id("inq-1"),
                "a different question",
                Provenance::AgentProposed,
            )
            .sequenced(1),
        )
        .expect("replacing a node is lawful");
        assert!(
            Delegation::exported(id("dlg-1"), node, 7).is_stale(&map),
            "a rewritten question is not a material change in the delta \
             vocabulary, but it IS a different obligation"
        );
    }

    #[test]
    fn one_obligation_holds_one_outstanding_assignment_at_a_time() {
        let (_, node) = map_with_obligation();
        let mut group = DelegationGroup::default();
        group.upsert(Delegation::exported(id("dlg-1"), node.clone(), 7));
        assert!(group.outstanding_for(node.id()).is_some());
        assert_eq!(
            group.find("dlg-1").map(Delegation::state),
            Some(DelegationState::Outstanding),
            "a cut assignment awaits a proposal"
        );

        // A proposal is still awaiting the coordinator, so the obligation is
        // still spoken for.
        let proposed = group
            .find("dlg-1")
            .cloned()
            .expect("the group holds it")
            .proposed(Proposal::of("delegate", "here is what I found", Vec::new()));
        group.upsert(proposed);
        assert!(group.outstanding_for(node.id()).is_some());
        assert_eq!(
            group.find("dlg-1").map(Delegation::state),
            Some(DelegationState::Proposed)
        );
        assert_eq!(
            group
                .find("dlg-1")
                .and_then(Delegation::proposal)
                .map(Proposal::summary),
            Some("here is what I found"),
            "the conclusion is stored as submitted"
        );

        // Disposed either way, it is not.
        let accepted = group
            .find("dlg-1")
            .cloned()
            .expect("the group holds it")
            .accepted();
        group.upsert(accepted);
        assert!(group.outstanding_for(node.id()).is_none());

        let refused = group
            .find("dlg-1")
            .cloned()
            .expect("the group holds it")
            .refused("the summary does not answer the question");
        group.upsert(refused);
        assert!(group.outstanding_for(node.id()).is_none());
        assert_eq!(
            group.find("dlg-1").and_then(Delegation::refused_because),
            Some("the summary does not answer the question")
        );
        assert!(
            group.find("dlg-1").and_then(Delegation::proposal).is_some(),
            "a refused proposal is still readable"
        );
    }
}
