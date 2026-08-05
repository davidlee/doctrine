// SPDX-License-Identifier: GPL-3.0-only
//! SL-233 PHASE-10 — the bounded delegation seam, over the built binary
//! (design §5.4, §9.2, DEC-058/DEC-068).
//!
//! Three behaviours, and the phase's exit criterion names them: a stale proposal
//! is refused and left inspectable, a delegated worker cannot advance the run, and
//! an accepted proposal keeps its attribution.
//!
//! **What is deliberately absent.** There is no second process anywhere in this
//! file. The "session boundary" a delegation crosses is fiction the protocol does
//! not need enacted: the delegate's proposal is an ordinary `design apply`, and
//! v1 defines no spawn transport and no write broker (`EX-4`, scope Non-Goals).
//! A test that spawned a worker would be testing a transport this phase refuses
//! to have.
//!
//! The pure model is `#[path]`-included rather than imported: this crate is
//! **binary-only** (no `[lib]`), which is the CHR-014 idiom the rest of the design
//! suite already uses. It means the state tokens these tests read are the exact
//! bytes the binary compiles, not strings re-typed here (STD-001).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

mod common;
/// The wire-shaped act payloads a ladder submits (SL-244 `T8`), shared with the
/// other design suites that cross an edge.
mod design_act;
/// Opted into for [`design_fixture::seed_slice_record`] alone — SL-244 `T8`'s
/// governance act projects the slice's own edge set, and one seeder shared with
/// the other design suites beats three copies.
mod design_fixture;
mod runbook_fixture;

/// The pure model, from source. `design_run` is a leaf with crate out-degree
/// zero, so it compiles standalone here exactly as it does in the binary.
#[path = "../src/design_run/mod.rs"]
#[allow(
    dead_code,
    unused_imports,
    reason = "the whole leaf tree is included; no single test exercises all of it"
)]
mod design_run;

use design_run::Stage;
use design_run::attestation::{ActKind, AgentAct, AgentActKind, ReviewPolicy};
use design_run::delegation::{Delegation, DelegationState};
use design_run::ids::DesignId;
use design_run::inquiry::DispositionForm;
use design_run::snapshot::{self, DesignSnapshot};
use design_run::submission::ApplyRequest;
use design_run::traversal::Posture;

/// The slice every fixture designs.
const SLICE: &str = "SL-233";
/// Its zero-padded directory name.
const SLICE_NUMBER: &str = "233";

// ── the run under test ────────────────────────────────────────────────────

/// The one bounded obligation that gets delegated.
const OBLIGATION: &str = "inq-1";
/// The delegation exported against it.
const DELEGATION: &str = "dlg-1";
/// The map change the delegate proposes — a child of the obligation it worked.
const PROPOSED_NODE: &str = "inq-2";
/// The checkpoint the coordinator disposes [`OBLIGATION`] through.
const CHECKPOINT: &str = "cp-1";
/// A section nothing in this suite edits, so the coordinator's positive-control
/// stage move stays reachable no matter what a test does to the inquiry map. It
/// held every claimed clearance until SL-244 `T13` retired the last of them; the
/// acts that cleared the gate in their place bind to the run, not to a section.
const SECTION_SPINE: &str = "sec-spine";

/// Who the proposal says did the work. Stored verbatim: `EX-1` asks for an
/// *attributed* proposal, and v1 authenticates nothing (A3).
const DELEGATE: &str = "delegate-session-7";
/// What the proposal says. Prose, and never interpreted by Doctrine.
const PROPOSAL_SUMMARY: &str =
    "the obligation splits: the transport question is separable from the authority question";
/// Why the coordinator turns it down — the one thing a refusal has to carry.
const REFUSAL_REASON: &str = "the obligation was deferred; re-export it if it returns";

/// Every writer act a proposal-bearing payload may not also carry (`EX-2`) — one
/// row per key [`ApplyRequest::WRITER_ACTS`] checks, and that equality is itself
/// asserted by [`the_writer_act_table_covers_every_key_writer_act_checks`].
///
/// A table rather than six hand-written cases: the guard is about the *class* of
/// act. The enumeration alone does not make a gap visible, though — it read as
/// exhaustive while covering four of six (`RV-324` F-6), so the tie to the guard's
/// own vocabulary is what carries the claim, not this comment.
///
/// The bodies need only be well-formed enough to deserialize: the guard fires on a
/// field's *presence*, ahead of anything that would validate it.
const WRITER_ACTS: [(&str, fn() -> Value); 9] = [
    ("stage", || json!({"to": Stage::Drafting.as_str()})),
    ("acceptance", || json!({"basis": "the delegate says so"})),
    (
        "declare",
        || json!([{"subject": PROPOSED_NODE, "question": "declared, not proposed"}]),
    ),
    (
        "adopt_authored",
        || json!({"fingerprint": "0000", "sections": {}}),
    ),
    ("traversal", || json!({"posture": Posture::Depth})),
    (
        "discharge",
        || json!({"step": "explore.scope", "outcome": "attested"}),
    ),
    ("review_policy", || {
        json!({
            "policy": ReviewPolicy::AdversarialOnly.as_str(),
            "acceptance": {"basis": "the delegate says the lanes should change"},
        })
    }),
    // The two act rows go through the shared builders rather than a second
    // spelling of them. Written out here at `T4`, four tasks before
    // `design_act` existed, and re-pointed at `T14`'s `VA-2` sweep — the
    // builders make the same claim (from the wire types, never a re-typed
    // serde token, STD-001) in one place.
    ("checkpoint_act", || {
        design_act::checkpoint_act(
            ActKind::GraphReviewed,
            "the delegate says the graph is steered",
        )
    }),
    ("agent_declaration", || {
        design_act::agent_declaration(
            AgentAct::DraftingReady,
            "the delegate says the draft is ready",
        )
    }),
];

// ── fixture ───────────────────────────────────────────────────────────────

/// A started design run in a throwaway tree, driven to `inquiring`.
struct Fixture {
    _tmp: tempfile::TempDir,
    root: PathBuf,
    /// Learned from `design start`'s own output, so no test re-types the state
    /// path (STD-001).
    snapshot: PathBuf,
    uid: String,
}

impl Fixture {
    /// A run in `inquiring` holding the obligation, the spine section, and the
    /// acts that cleared the two boundaries below it — the state every delegation
    /// test starts from.
    fn inquiring() -> Fixture {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        // SL-244 `T8`: the `governance-confirmed` act projects the slice's own
        // outbound edge set, so the tree needs an authored record to project
        // from — a slice directory with nothing in it is an UNOBSERVABLE fact,
        // which reads as changed and refuses the act.
        design_fixture::seed_slice_record(&root, SLICE_NUMBER);
        let out = run(&root, &["design", "start", SLICE, "-p", "."]);
        let uid = out
            .split_whitespace()
            .nth(1)
            .expect("`design start` names the run uid")
            .to_owned();
        let snapshot = out
            .lines()
            .find_map(|line| line.strip_prefix("snapshot "))
            .map(|path| root.join(path))
            .expect("`design start` names the snapshot path");
        let fixture = Fixture {
            _tmp: tmp,
            root,
            snapshot,
            uid,
        };

        // The edge out of `exploring` carries a runbook since SL-233 PHASE-16
        // (`EX-8`), so the seed discharges it before staging. Not fixture
        // ceremony — it is the guard every caller of this edge now meets.
        for step in runbook_fixture::EXPLORING_STEPS {
            fixture.apply(&fixture.payload(
                &runbook_fixture::discharge_label(step),
                &runbook_fixture::discharge_body(step),
            ));
        }
        // The map is declared in its own submission, ahead of the acts, because
        // `initial-concerns-recorded` binds to the inquiry map: an act recorded
        // before `OBLIGATION` existed would carry a covered map the very next
        // submission invalidates. Declare, then attest over what was declared.
        fixture.apply(&fixture.payload(
            "seed-map",
            &json!({"declare": [
                {"subject": SECTION_SPINE, "body": spine_body()},
                {"subject": OBLIGATION, "question": "does delegation need a transport in v1?"},
            ]}),
        ));
        fixture.apply(&fixture.payload(
            "governance",
            &json!({
                "checkpoint_act": design_act::checkpoint_act(
                    ActKind::GovernanceConfirmed,
                    "the governing artefacts are the ones found",
                ),
            }),
        ));
        // DEC-121's two acts by two actors, in one submission — which is what
        // `T6`'s build order buys: the declaration is constructed and
        // fingerprinted before the act that confirms it, so no caller computes a
        // digest.
        fixture.apply(&fixture.payload(
            "graph",
            &json!({
                "agent_declaration": design_act::agent_declaration(
                    AgentAct::BlockingSetDeclared {
                        blocking: [DesignId::parse(OBLIGATION).unwrap()].into(),
                    },
                    "the obligation blocks drafting until it is settled",
                ),
                "checkpoint_act": design_act::checkpoint_act(
                    ActKind::GraphReviewed,
                    "the blocking set is right",
                ),
            }),
        ));
        fixture
            .apply(&fixture.payload("seed", &json!({"stage": {"to": Stage::Inquiring.as_str()}})));
        fixture
    }

    /// Export the obligation as an assignment; returns the emitted assignment.
    fn export(&self) -> String {
        self.apply(&self.payload(
            "export",
            &json!({"delegation": {"act": "export", "id": DELEGATION, "obligation": OBLIGATION}}),
        ))
    }

    /// The delegate's proposal: its attribution, its prose result, and the one map
    /// change it proposes.
    fn proposal(&self) -> Value {
        json!({"delegation": {
            "act": "propose",
            "id": DELEGATION,
            "by": DELEGATE,
            "summary": PROPOSAL_SUMMARY,
            "declare": [{
                "subject": PROPOSED_NODE,
                "question": "is the write broker separable from the authority model?",
                "parent": OBLIGATION,
            }],
        }})
    }

    /// The parsed snapshot.
    fn read(&self) -> DesignSnapshot {
        snapshot::parse(&std::fs::read_to_string(&self.snapshot).unwrap()).unwrap()
    }

    /// The delegation the suite works, or a failure naming what the run holds.
    fn delegation(&self) -> Delegation {
        self.read()
            .delegation
            .find(DELEGATION)
            .cloned()
            .expect("the run holds the exported delegation")
    }

    /// The run's current stage.
    fn stage(&self) -> Stage {
        self.read().run.stage
    }

    /// The current revision.
    fn revision(&self) -> u64 {
        self.read().run.revision
    }

    /// Whether the inquiry map holds `id` — how "the proposal has not been
    /// applied" is observed.
    fn holds_node(&self, id: &str) -> bool {
        self.read()
            .map
            .inquiry
            .nodes()
            .any(|node| node.id().as_str() == id)
    }

    /// A payload carrying the current revision and `submission`, plus `body`'s
    /// top-level keys merged in.
    fn payload(&self, submission: &str, body: &Value) -> String {
        let mut object = json!({
            "run_uid": self.uid,
            "known_revision": self.revision(),
            "submission_id": submission,
        });
        let map = object.as_object_mut().unwrap();
        for (key, value) in body.as_object().unwrap() {
            map.insert(key.clone(), value.clone());
        }
        object.to_string()
    }

    /// Apply a payload, expecting success; returns stdout.
    fn apply(&self, body: &str) -> String {
        run(
            &self.root,
            &["design", "apply", SLICE, "-p", ".", "--input", body],
        )
    }

    /// Apply a payload, expecting refusal; returns stderr.
    fn refuse(&self, body: &str) -> String {
        fail(
            &self.root,
            &["design", "apply", SLICE, "-p", ".", "--input", body],
        )
    }
}

/// The spine section's body, which must open with its own heading.
fn spine_body() -> String {
    format!("## {SECTION_SPINE}\n\nThe section no test in this suite edits.\n")
}

/// Run the built binary, expecting success; returns stdout.
fn run(root: &Path, args: &[&str]) -> String {
    let out = common::doctrine_cmd(root)
        .args(args)
        .output()
        .expect("spawn doctrine");
    assert!(
        out.status.success(),
        "doctrine {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// Run the built binary, expecting failure; returns stderr.
fn fail(root: &Path, args: &[&str]) -> String {
    let out = common::doctrine_cmd(root)
        .args(args)
        .output()
        .expect("spawn doctrine");
    assert!(
        !out.status.success(),
        "doctrine {args:?} unexpectedly succeeded: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    String::from_utf8_lossy(&out.stderr).to_string()
}

// ── EX-3: refused as stale, and still there to read ───────────────────────

#[test]
fn stale_proposal_is_refused_and_left_inspectable() {
    let fixture = Fixture::inquiring();
    fixture.export();
    fixture.apply(&fixture.payload("propose", &fixture.proposal()));

    // The coordinator moves the very obligation the assignment was cut from.
    // Obligation-scoped, not run-scoped: unrelated coordinator activity must not
    // invalidate an outstanding proposal, or delegation is unusable.
    fixture.apply(&fixture.payload(
        "defer-the-obligation",
        &json!({"declare": [{"subject": OBLIGATION, "lifecycle": "deferred"}]}),
    ));
    let before = fixture.revision();

    let stderr = fixture.refuse(&fixture.payload(
        "accept",
        &json!({"delegation": {"act": "accept", "id": DELEGATION}}),
    ));
    assert!(
        stderr.contains(DELEGATION) && stderr.contains("stale"),
        "the refusal must name the delegation and say stale, got: {stderr}"
    );

    // VA-2, first half: the run state is unchanged. Not merely "the stage did not
    // move" — the proposal's own map change must not have landed either, which is
    // what `refused, never silently rebased` rules out.
    assert_eq!(
        before,
        fixture.revision(),
        "a refused acceptance writes nothing"
    );
    assert!(
        !fixture.holds_node(PROPOSED_NODE),
        "the proposed node must not be applied by a refused acceptance"
    );

    // VA-2, second half: the proposal is still there to read. A refusal that
    // dropped it would satisfy `refused` and lose the delegate's work — the
    // failure mode `left inspectable` was written against.
    let delegation = fixture.delegation();
    let proposal = delegation
        .proposal()
        .expect("the refused proposal is still readable");
    assert_eq!(delegation.state(), DelegationState::Proposed);
    assert_eq!(proposal.by(), DELEGATE);
    assert_eq!(proposal.summary(), PROPOSAL_SUMMARY);

    // The stale acceptance is a refusal, not a disposition: the assignment is
    // still awaiting the coordinator, who disposes it explicitly and on the
    // record. `refused` is the coordinator's act — never the name of a stale
    // acceptance, which writes nothing at all.
    fixture.apply(&fixture.payload(
        "refuse",
        &json!({"delegation": {
            "act": "refuse",
            "id": DELEGATION,
            "reason": REFUSAL_REASON,
        }}),
    ));
    let delegation = fixture.delegation();
    assert_eq!(delegation.state(), DelegationState::Refused);
    assert_eq!(delegation.refused_because(), Some(REFUSAL_REASON));
    assert!(
        delegation.proposal().is_some(),
        "disposing a proposal does not discard it"
    );
    assert!(
        !fixture.holds_node(PROPOSED_NODE),
        "and refusing it certainly does not apply it"
    );
}

// ── EX-2: the coordinator keeps global transition authority ────────────────

/// The table below claims to cover the *class* of writer act, so the claim is
/// asserted rather than trusted (`RV-324` F-6): it covered four of the six keys
/// `writer_act` checks, and the two it missed — `adopt_authored` and `traversal` —
/// were unrefused in test while the comment read as though they were not.
///
/// Tied to [`ApplyRequest::WRITER_ACTS`], which is the guard itself, so a seventh
/// act fails here instead of quietly widening the class.
#[test]
fn the_writer_act_table_covers_every_key_writer_act_checks() {
    let tabled: BTreeSet<&str> = WRITER_ACTS.iter().map(|(act, _)| *act).collect();
    let guarded: BTreeSet<&str> = ApplyRequest::WRITER_ACTS
        .iter()
        .map(|(act, _)| *act)
        .collect();
    assert_eq!(
        tabled, guarded,
        "every writer act the guard checks needs a row here, or its refusal is \
         untested while the comment claims otherwise"
    );
}

#[test]
fn delegated_worker_cannot_advance_the_run() {
    let fixture = Fixture::inquiring();
    fixture.export();

    // A proposal may not ride any writer act into the run. The channel cannot
    // express an advance, whoever is holding it — there is no worker identity to
    // check, and inventing one would be authority laundering (design R2/R12).
    for (act, body) in WRITER_ACTS {
        let mut payload = fixture.proposal();
        payload
            .as_object_mut()
            .unwrap()
            .insert(act.to_owned(), body());
        let stderr = fixture.refuse(&fixture.payload(&format!("advance-via-{act}"), &payload));
        assert!(
            stderr.contains(act),
            "the refusal must name `{act}` as the writer act it found, got: {stderr}"
        );
        assert_eq!(
            fixture.stage(),
            Stage::Inquiring,
            "a refused proposal leaves the run where it was"
        );
    }

    // A lawful proposal records and applies nothing: the map change is *proposed*.
    fixture.apply(&fixture.payload("propose", &fixture.proposal()));
    assert_eq!(fixture.delegation().state(), DelegationState::Proposed);
    assert!(
        !fixture.holds_node(PROPOSED_NODE),
        "a recorded proposal has changed no map state"
    );

    // The positive control, and it is what makes the refusals above mean
    // something: the same stage move, submitted alone, is the coordinator's to
    // make and lands.
    // The edge out of `inquiring` carries a runbook since SL-233 PHASE-08, so
    // the coordinator discharges it before its own advance lands. The positive
    // control is still the advance, not the discharges.
    for step in runbook_fixture::INQUIRING_STEPS {
        fixture.apply(&fixture.payload(
            &runbook_fixture::discharge_label(step),
            &runbook_fixture::discharge_body(step),
        ));
    }
    // SL-244 `T10` — the other thing this crossing owes once the evaluator
    // derives it: `blocking-inquiries-dispositioned`. The fixture declares
    // `OBLIGATION` blocking, and the incumbent scan cleared that condition on a
    // claim; the evaluator reads the map. So the coordinator disposes the
    // question it delegated before it advances past it — which is the
    // condition's whole point, and was previously assertable only as a claim.
    //
    // `unresolved` rather than a record-bearing form: the proposal above is
    // recorded and deliberately unapplied, so *retained, unresolved, with the
    // reason stated* is the true disposition here. Disposing moves no material —
    // `NodeMaterial` excludes lifecycle and disposition on purpose (PHASE-02
    // `EX-2`) — so the two inquiry-map coverages given above stay current.
    fixture.apply(&fixture.payload(
        "dispose-the-obligation",
        &json!({"declare": [{
            "subject": CHECKPOINT,
            "disposes": OBLIGATION,
            "dispose": {
                "form": DispositionForm::RetainUnresolved.as_str(),
                "note": "the delegate's proposal stands unapplied; drafting proceeds without it",
            },
        }]}),
    ));
    // SL-244 `T8` — what this crossing will owe after `T10`:
    // `user-accepts-sufficiency`. Recorded here rather than in the fixture
    // because it belongs to *this* edge, and because the recorded proposal above
    // deliberately changed no map state, so the act's inquiry-map coverage is
    // still current when it is given.
    fixture.apply(&fixture.payload(
        "sufficiency",
        &json!({
            "checkpoint_act": design_act::checkpoint_act(
                ActKind::SufficiencyAccepted,
                "the obligation is understood well enough to draft",
            ),
        }),
    ));
    fixture.apply(&fixture.payload(
        "coordinator-advances",
        &json!({"stage": {"to": Stage::Drafting.as_str()}}),
    ));
    assert_eq!(fixture.stage(), Stage::Drafting);
}

// ── EX-1: accepted back into the coordinating run, still attributed ────────

#[test]
fn accepted_proposal_retains_its_attribution() {
    let fixture = Fixture::inquiring();

    // The assignment is self-contained: everything a fresh session needs to work
    // the obligation without reading the coordinator's state.
    let assignment = fixture.export();
    for part in [DELEGATION, OBLIGATION, &fixture.uid, "propose"] {
        assert!(
            assignment.contains(part),
            "the exported assignment must carry `{part}`, got: {assignment}"
        );
    }

    fixture.apply(&fixture.payload("propose", &fixture.proposal()));
    fixture.apply(&fixture.payload(
        "accept",
        &json!({"delegation": {"act": "accept", "id": DELEGATION}}),
    ));

    // Accepted *into the run*: the proposed map change is now the coordinator's.
    assert!(
        fixture.holds_node(PROPOSED_NODE),
        "acceptance lands the proposed map change"
    );

    // And the attribution survives the crossing verbatim. The coordinator accepts
    // the proposal; it does not become the author of it.
    let delegation = fixture.delegation();
    assert_eq!(delegation.state(), DelegationState::Accepted);
    let proposal = delegation
        .proposal()
        .expect("an accepted delegation still holds the proposal it accepted");
    assert_eq!(proposal.by(), DELEGATE);
    assert_eq!(proposal.summary(), PROPOSAL_SUMMARY);

    // Later coordinator activity does not rewrite it either.
    fixture.apply(&fixture.payload(
        "carry-on",
        &json!({"declare": [{"subject": "inq-3", "question": "and then?"}]}),
    ));
    assert_eq!(
        fixture
            .delegation()
            .proposal()
            .map(|held| held.by().to_owned()),
        Some(DELEGATE.to_owned()),
        "attribution is stored, not re-derived"
    );
}

// ── SL-244 T8: the ladder's acts are recorded, not merely submitted ────────

/// `T8`'s own control — the acts the fixture adds are *recorded*, so a payload
/// key that silently deserialised into nothing cannot read as done.
///
/// Necessary because `T8` is additive by design (`D2`): the incumbent evidence
/// scan is what clears the gate until `T10`, so nothing else in this suite would
/// fail if the acts went nowhere. The suite would stay green and the fixture
/// would be inert JSON — which is precisely the state `T10` is supposed to be
/// able to flip into without touching a fixture.
///
/// It asserts the acts by **kind**, not the whole record: the record's engine
/// slots are `T5`/`T6`'s to pin, and unit tests already do so by payload.
#[test]
fn the_ladder_records_the_acts_its_crossings_will_owe() {
    let fixture = Fixture::inquiring();
    let held = fixture.read();

    let acts: BTreeSet<ActKind> = held.acts.acts.iter().map(|act| act.act).collect();
    assert_eq!(
        acts,
        BTreeSet::from([ActKind::GovernanceConfirmed, ActKind::GraphReviewed]),
        "the exploring → inquiring crossing owes both of its conditions' acts"
    );

    let declared: Vec<AgentActKind> = held
        .declarations
        .declarations
        .iter()
        .map(|declaration| declaration.act.kind())
        .collect();
    assert_eq!(
        declared,
        vec![AgentActKind::BlockingSetDeclared],
        "and the declaration the user's `GraphReviewed` confirms"
    );
    assert!(
        held.declarations.declarations.iter().all(|declaration| {
            held.acts
                .acts
                .iter()
                .any(|act| act.confirms.as_ref() == Some(&declaration.fingerprint))
        }),
        "the confirmation link is live: the act carries the digest of the record the \
         engine wrote in the same submission, with no caller-computed digest anywhere"
    );
}
