// SPDX-License-Identifier: GPL-3.0-only
//! The design §9.1 pure-engine suite — eight tests, named by SL-233 PHASE-02
//! EX-8, operating on values and injected [`DerivedDesignFacts`] only.
//!
//! No clock, disk, git, or rng is reachable from here, because none is reachable
//! from the module under test.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test code — the repo's panic-avoidance denials target production paths"
)]

use super::Stage;
use super::facts::DerivedDesignFacts;
use super::gate::{
    Condition, ReviewStanding, advance, can_advance, cumulative_conditions, regress,
};
use super::ids::{DesignId, Fingerprint};
use super::inquiry::{Disposition, InquiryLifecycle, InquiryMap, InquiryNode, Provenance};
use super::refusal::Refusal;
use super::runbook::RunbookStanding;
use super::submission::{Batch, Declaration, Sparse};

/// A validated run-local id, or a test failure naming the bad literal.
fn id(raw: &str) -> DesignId {
    DesignId::parse(raw).expect("test fixture id must be well-formed")
}

/// Facts in which every *claimed* cumulative condition up to `stage` holds, each
/// cleared against a distinct subject at a known fingerprint.
///
/// The derived conditions are filtered out rather than recorded here: recording
/// them would build a fact set that looks like clearance the gate does not read
/// (a claim on a derived condition is refused at admission, and
/// [`Condition::is_derived`] is why). Their absence is what a
/// [`ReviewStanding`] answers for.
fn cleared_through(stage: Stage) -> DerivedDesignFacts {
    let mut facts = DerivedDesignFacts::default();
    for (index, condition) in cumulative_conditions(stage)
        .into_iter()
        .filter(|condition| !condition.is_derived())
        .enumerate()
    {
        let subject = id(&format!("sec-{index}"));
        let fingerprint = Fingerprint::new(format!("sha256:{index}"));
        facts = facts.observe(subject.clone(), fingerprint.clone()).record(
            condition,
            subject,
            fingerprint,
        );
    }
    facts
}

#[test]
fn stage_gate_table_admits_only_legal_forward_moves() {
    let mut admitted = Vec::new();
    for from in Stage::ALL {
        for to in Stage::ALL {
            if can_advance(from, to) {
                admitted.push((from, to));
            }
        }
    }

    // Exhaustive over all 25 ordered pairs; the four adjacent forward moves of
    // design §5.4 and nothing else — no self-move, no skip, no backward move.
    assert_eq!(
        admitted,
        vec![
            (Stage::Exploring, Stage::Inquiring),
            (Stage::Inquiring, Stage::Drafting),
            (Stage::Drafting, Stage::Reviewing),
            (Stage::Reviewing, Stage::Locked),
        ]
    );

    // The verb rides the same table: a skip is refused even when every condition
    // in the run holds, so legality is not something clearance can buy.
    let facts = cleared_through(Stage::Locked);
    assert_eq!(
        advance(
            Stage::Exploring,
            Stage::Drafting,
            &facts,
            ReviewStanding::default(),
            None
        ),
        Err(Refusal::IllegalStageMove {
            from: Stage::Exploring,
            to: Stage::Drafting,
        })
    );
    assert_eq!(
        // SL-233 PHASE-16: this edge now also carries a runbook, so clearance
        // takes a discharged standing beside the conditions. A default standing
        // holds nothing outstanding — the "ritual done" case — which keeps this
        // assertion about the CONDITION table, which is what it is here to test.
        advance(
            Stage::Exploring,
            Stage::Inquiring,
            &facts,
            ReviewStanding::default(),
            Some(&RunbookStanding::default())
        ),
        Ok(Stage::Inquiring)
    );
}

#[test]
fn direct_regression_requires_a_recorded_reason() {
    assert_eq!(
        regress(Stage::Drafting, Stage::Exploring, ""),
        Err(Refusal::RegressionReasonMissing {
            from: Stage::Drafting,
            to: Stage::Exploring,
        })
    );
    // Whitespace is not a reason.
    assert_eq!(
        regress(Stage::Drafting, Stage::Exploring, "   \n\t"),
        Err(Refusal::RegressionReasonMissing {
            from: Stage::Drafting,
            to: Stage::Exploring,
        })
    );
    // A forward move is not a regression, and must not be laundered into one by
    // supplying a reason.
    assert_eq!(
        regress(Stage::Exploring, Stage::Drafting, "reason"),
        Err(Refusal::NotARegression {
            from: Stage::Exploring,
            to: Stage::Drafting,
        })
    );

    let recorded =
        regress(Stage::Drafting, Stage::Exploring, "the framing was wrong").expect("legal");
    assert_eq!(recorded.from(), Stage::Drafting);
    assert_eq!(recorded.to(), Stage::Exploring);
    assert_eq!(recorded.reason(), "the framing was wrong");

    // DEC-067's other half: returning forward inherits no clearance. Facts that
    // satisfy only the drafting boundary do not re-open drafting, because the
    // *cumulative* set is re-derived against current content.
    let mut partial = DerivedDesignFacts::default();
    for condition in [
        Condition::BlockingInquiriesDispositioned,
        Condition::UserAcceptsSufficiency,
    ] {
        let subject = id("sec-late");
        let fingerprint = Fingerprint::new("sha256:late");
        partial = partial
            .observe(subject.clone(), fingerprint.clone())
            .record(condition, subject, fingerprint);
    }
    // The runbook standing is CLEARED here on purpose. SL-233 PHASE-08 gave this
    // edge a runbook, and the gate fails closed on a missing standing *before*
    // it derives conditions — so passing `None` would stop the advance one check
    // earlier and this assertion would no longer be about cumulative conditions
    // at all. Clearing it keeps the subject the same assertion always had.
    let discharged = RunbookStanding::default();
    assert!(discharged.cleared(), "no outstanding required steps");
    assert_eq!(
        advance(
            Stage::Inquiring,
            Stage::Drafting,
            &partial,
            ReviewStanding::default(),
            Some(&discharged)
        ),
        Err(Refusal::GateNotCleared {
            from: Stage::Inquiring,
            to: Stage::Drafting,
            missing: vec![
                Condition::GoverningContextRecorded,
                Condition::InitialConcernsRecorded,
            ],
        })
    );
}

#[test]
fn changed_fingerprint_invalidates_only_affected_evidence() {
    let moved = id("sec-moved");
    let stable = id("sec-stable");
    let before = Fingerprint::new("sha256:aaa");
    let after = Fingerprint::new("sha256:bbb");
    let untouched = Fingerprint::new("sha256:ccc");

    let facts = DerivedDesignFacts::default()
        .observe(moved.clone(), before.clone())
        .observe(stable.clone(), untouched.clone())
        .record(Condition::RequiredSectionsExist, moved.clone(), before)
        .record(
            Condition::MaterialisationCurrent,
            stable.clone(),
            untouched.clone(),
        );

    assert_eq!(facts.live_evidence().count(), 2);
    assert!(facts.satisfies(Condition::RequiredSectionsExist));
    assert!(facts.satisfies(Condition::MaterialisationCurrent));

    // One subject's content moves. Only its evidence dies.
    let after_edit = facts.observe(moved.clone(), after);
    let live: Vec<&DesignId> = after_edit
        .live_evidence()
        .map(super::facts::Evidence::subject)
        .collect();
    assert_eq!(live, vec![&stable]);
    assert!(!after_edit.satisfies(Condition::RequiredSectionsExist));
    assert!(after_edit.satisfies(Condition::MaterialisationCurrent));

    // A subject the shell can no longer observe is treated as changed, not as
    // unchanged — absence is not proof the bytes still match.
    let unobservable =
        DerivedDesignFacts::default().record(Condition::MaterialisationCurrent, stable, untouched);
    assert_eq!(unobservable.live_evidence().count(), 0);
}

#[test]
fn parent_and_needs_cycles_are_refused() {
    let (a, b) = (id("inq-a"), id("inq-b"));

    // Parent relation: a → b → a.
    let mut map = InquiryMap::default();
    map.insert(InquiryNode::open(a.clone(), "a?", Provenance::UserDirected))
        .expect("root");
    map.insert(
        InquiryNode::open(b.clone(), "b?", Provenance::AgentProposed).with_parent(a.clone()),
    )
    .expect("child");
    assert_eq!(
        map.insert(
            InquiryNode::open(a.clone(), "a?", Provenance::UserDirected).with_parent(b.clone())
        ),
        // The refusal names the edge that *closes* the cycle, which is the one
        // reached last on the walk, not the one just submitted.
        Err(Refusal::CyclicEdge {
            from: b.clone(),
            to: a.clone(),
        })
    );
    // The refused insert left the map as it was.
    assert_eq!(map.get(&a).and_then(InquiryNode::parent), None);

    // `needs` relation: the same cycle through the other edge kind.
    let mut needs_map = InquiryMap::default();
    needs_map
        .insert(InquiryNode::open(a.clone(), "a?", Provenance::UserDirected))
        .expect("root");
    needs_map
        .insert(InquiryNode::open(b.clone(), "b?", Provenance::AgentProposed).needing(a.clone()))
        .expect("dependant");
    assert_eq!(
        needs_map.insert(
            InquiryNode::open(a.clone(), "a?", Provenance::UserDirected).needing(b.clone())
        ),
        Err(Refusal::CyclicEdge { from: b, to: a })
    );

    // A diamond is not a cycle. `needs` makes them routine, and a checker that
    // reports every re-reached node would refuse this.
    let (d, e, f, g) = (id("inq-d"), id("inq-e"), id("inq-f"), id("inq-g"));
    let mut diamond = InquiryMap::default();
    diamond
        .insert(InquiryNode::open(g.clone(), "g?", Provenance::UserDirected))
        .expect("sink");
    diamond
        .insert(InquiryNode::open(e.clone(), "e?", Provenance::UserDirected).needing(g.clone()))
        .expect("left");
    diamond
        .insert(InquiryNode::open(f.clone(), "f?", Provenance::UserDirected).needing(g))
        .expect("right");
    assert_eq!(
        diamond.insert(
            InquiryNode::open(d.clone(), "d?", Provenance::UserDirected)
                .needing(e)
                .needing(f)
        ),
        Ok(())
    );
    assert_eq!(diamond.len(), 4);
}

#[test]
fn blocked_is_derived_not_stored() {
    let (waiting, blocker) = (id("inq-waiting"), id("inq-blocker"));
    let mut map = InquiryMap::default();
    map.insert(InquiryNode::open(
        blocker.clone(),
        "settle me first",
        Provenance::UserDirected,
    ))
    .expect("blocker");
    map.insert(
        InquiryNode::open(waiting.clone(), "depends", Provenance::AgentProposed)
            .needing(blocker.clone()),
    )
    .expect("dependant");

    assert!(map.is_blocked(&waiting));
    assert!(!map.is_blocked(&blocker));
    let before = map.get(&waiting).expect("present").clone();

    // Settling the blocker unblocks the dependant without touching it. The
    // dependant's value is byte-identical across the change, which is what
    // proves `blocked` is not a field somebody has to remember to update.
    let settled = map
        .get(&blocker)
        .expect("present")
        .clone()
        .resolve(Disposition::Created {
            record: "DEC-999".to_owned(),
        });
    map.insert(settled).expect("resolve");

    assert!(!map.is_blocked(&waiting));
    assert_eq!(map.get(&waiting), Some(&before));
    assert_eq!(map.blocked().count(), 0);
}

#[test]
fn resolved_node_without_disposition_is_refused() {
    let node_id = id("inq-1");
    let node = InquiryNode::open(node_id.clone(), "why?", Provenance::UserDirected);

    // The lifecycle-only route cannot reach `resolved`.
    assert_eq!(
        node.clone().transition(InquiryLifecycle::Resolved),
        Err(Refusal::DispositionMissing { id: node_id })
    );

    // The other non-resolved lifecycles are reachable and carry no disposition.
    for lifecycle in [InquiryLifecycle::Deferred, InquiryLifecycle::Pruned] {
        let moved = node.clone().transition(lifecycle).expect("legal");
        assert_eq!(moved.lifecycle(), lifecycle);
        assert_eq!(moved.disposition(), None);
    }

    // Resolution goes through the disposition-carrying route, and keeps it.
    let disposition = Disposition::RetainedUnresolved {
        note: "parked pending SPEC-024".to_owned(),
    };
    let resolved = node.resolve(disposition.clone());
    assert_eq!(resolved.lifecycle(), InquiryLifecycle::Resolved);
    assert_eq!(resolved.disposition(), Some(&disposition));

    // Moving back off `resolved` drops the disposition rather than leaving a
    // stale one attached to a node that is no longer resolved.
    let reopened = resolved.transition(InquiryLifecycle::Open).expect("legal");
    assert_eq!(reopened.disposition(), None);
}

#[test]
fn sparse_omission_null_and_empty_collection_differ() {
    let prior = || Some("kept".to_owned());

    // Three spellings, three outcomes — the scalar case.
    assert_eq!(Sparse::Omitted.apply(prior()), Some("kept".to_owned()));
    assert_eq!(Sparse::<String>::Null.apply(prior()), None);
    assert_eq!(
        Sparse::Value("replaced".to_owned()).apply(prior()),
        Some("replaced".to_owned())
    );

    // The collection case: an empty `Value` clears, omission does not. These are
    // the two an `Option<Vec<_>>` model collapses.
    let existing = || vec![id("inq-1"), id("inq-2")];
    assert_eq!(Sparse::Omitted.apply_collection(existing()), existing());
    assert_eq!(
        Sparse::<Vec<DesignId>>::Value(Vec::new()).apply_collection(existing()),
        Vec::new()
    );
    assert_eq!(
        Sparse::<Vec<DesignId>>::Null.apply_collection(existing()),
        Vec::new()
    );
    assert_ne!(
        Sparse::Omitted.apply_collection(existing()),
        Sparse::<Vec<DesignId>>::Value(Vec::new()).apply_collection(existing())
    );

    // Omission is the default, which is what lets `#[serde(default)]` mean
    // "absent key" rather than "null value".
    assert!(Sparse::<String>::default().is_omitted());
    assert!(!Sparse::<String>::Null.is_omitted());
}

#[test]
fn unordered_batch_refuses_duplicate_subjects() {
    let subject = id("inq-1");
    let duplicated = Batch::of(vec![
        Declaration::about(subject.clone()).question(Sparse::Value("first".to_owned())),
        Declaration::about(subject.clone()).question(Sparse::Value("second".to_owned())),
    ]);
    assert_eq!(
        duplicated.validate(),
        Err(Refusal::DuplicateSubject { id: subject })
    );

    // The batch is unordered: the same declarations submitted in either order
    // validate to the same candidate, in the same sequence.
    let (one, two, three) = (id("inq-1"), id("inq-2"), id("inq-3"));
    let declarations = |order: [&DesignId; 3]| {
        Batch::of(
            order
                .into_iter()
                .map(|subject| Declaration::about(subject.clone()))
                .collect(),
        )
    };
    let forward = declarations([&one, &two, &three])
        .validate()
        .expect("valid");
    let reversed = declarations([&three, &two, &one])
        .validate()
        .expect("valid");
    assert_eq!(forward, reversed);
    assert_eq!(
        forward.keys().collect::<Vec<&DesignId>>(),
        vec![&one, &two, &three]
    );
}
