// SPDX-License-Identifier: GPL-3.0-only
//! SL-233 PHASE-12 — review choreography and the `reviewing → locked` gate, over
//! the built binary (design §9.2).
//!
//! Seven behaviours, and the shape of the first four is the point: the lock
//! predicate is a **conjunction** of four independent components (design §5.4),
//! and one `lock_gate_refuses` test cannot prove that. Each refusal test
//! therefore builds a run in which **exactly one** component is absent and the
//! other three are present, and asserts that the refusal names that component's
//! condition *and none of the other three* — so a gate that read the wrong
//! component, or read only one of them, fails here rather than passing on a
//! plausible-looking refusal.
//!
//! The pure model is `#[path]`-included rather than imported: this crate is
//! **binary-only** (no `[lib]`), which is the CHR-014 idiom the rest of the
//! design suite already uses. It means the condition tokens these tests assert
//! against are the exact bytes the binary compiles, not kebab strings re-typed
//! here (STD-001).

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
use design_run::change_log::ChangeEvent;
use design_run::gate::Condition;
use design_run::snapshot::{self, DesignSnapshot};

/// The slice every fixture designs.
const SLICE: &str = "SL-233";
/// Its zero-padded directory name.
const SLICE_NUMBER: &str = "233";

// ── the run under test ────────────────────────────────────────────────────

/// The two sections review choreography is exercised on: one is edited, the
/// other must be seen to survive that edit (EX-1's `only`).
const SECTION_A: &str = "sec-a";
const SECTION_B: &str = "sec-b";
/// A third section that is never edited. Every earlier-boundary clearance is
/// claimed against it, so an edit in a test invalidates review evidence
/// *without* also dropping the six conditions that got the run to `reviewing` —
/// otherwise a refusal could not be attributed to the component under test.
const SECTION_SPINE: &str = "sec-spine";

/// Section id → the attestation that reviews it.
const ATTESTED: [(&str, &str); 3] = [
    ("att-a", SECTION_A),
    ("att-b", SECTION_B),
    ("att-spine", SECTION_SPINE),
];

/// The integrated adversarial pass (EX-2 — mandatory in v1).
const INTEGRATED: &str = "int-1";
/// A blocking finding, raised before the lock is attempted.
const FINDING: &str = "fnd-1";

/// The conditions of the three earlier boundaries, claimed through the generic
/// evidence route because this suite is about the *fourth* boundary.
const EARLIER: [Condition; 6] = [
    Condition::GoverningContextRecorded,
    Condition::InitialConcernsRecorded,
    Condition::BlockingInquiriesDispositioned,
    Condition::UserAcceptsSufficiency,
    Condition::RequiredSectionsExist,
    Condition::MaterialisationCurrent,
];

/// One component of the `reviewing → locked` conjunction (design §5.4).
///
/// Each member owns the condition it clears, so a test names a component and the
/// condition token it must (and must not) see is derived rather than re-typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Component {
    /// A current attestation for every section (DEC-073).
    Attestations,
    /// The integrated adversarial pass.
    Integrated,
    /// Every blocking finding disposed.
    FindingDisposition,
    /// A current, content-bound user acceptance (DEC-088).
    Acceptance,
}

impl Component {
    /// All four — the conjunction, single-sourced.
    const ALL: [Component; 4] = [
        Component::Attestations,
        Component::Integrated,
        Component::FindingDisposition,
        Component::Acceptance,
    ];

    /// The gate condition this component clears.
    const fn condition(self) -> Condition {
        match self {
            Component::Attestations => Condition::SectionAttestationsCurrent,
            Component::Integrated => Condition::IntegratedReviewPresent,
            Component::FindingDisposition => Condition::BlockingFindingsDisposed,
            Component::Acceptance => Condition::UserAcceptanceAttested,
        }
    }
}

// ── fixture ───────────────────────────────────────────────────────────────

/// A started design run in a throwaway tree, driven to `reviewing`.
struct Fixture {
    _tmp: tempfile::TempDir,
    root: PathBuf,
    /// Learned from `design start`'s own output, so no test re-types the state
    /// path (STD-001).
    snapshot: PathBuf,
    uid: String,
}

impl Fixture {
    /// A run in `reviewing` holding three sections, the six earlier clearances,
    /// and one **undisposed blocking finding** — the state every lock test
    /// starts from.
    fn reviewing() -> Fixture {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::create_dir_all(root.join(common::SLICE_DIR).join(SLICE_NUMBER)).unwrap();
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

        let sections: Vec<Value> = [SECTION_A, SECTION_B, SECTION_SPINE]
            .into_iter()
            .map(|id| json!({"subject": id, "body": section_body(id, "as first drafted")}))
            .collect();
        // Every forward edge carries a runbook — `exploring` since SL-233
        // PHASE-16 (`EX-8`), the rest since PHASE-08 — so the seed discharges
        // each edge's list as it reaches it. Not fixture ceremony: it is the
        // guard every caller of these edges now meets. This suite's subject is
        // the LOCK gate, so the discharges are setup and the lock assertions
        // below remain the only thing under test.
        for step in runbook_fixture::EXPLORING_STEPS {
            fixture.apply(&fixture.payload(step, &runbook_fixture::discharge_body(step)));
        }
        fixture.apply(&fixture.payload(
            "draft",
            &json!({
                "declare": sections,
                "evidence": claimed(),
                "stage": {"to": Stage::Inquiring.as_str()},
            }),
        ));
        for step in runbook_fixture::INQUIRING_STEPS {
            fixture.apply(&fixture.payload(
                &runbook_fixture::discharge_label(step),
                &runbook_fixture::discharge_body(step),
            ));
        }
        fixture.apply(&fixture.payload(
            "to-drafting",
            &json!({"stage": {"to": Stage::Drafting.as_str()}}),
        ));
        for step in runbook_fixture::DRAFTING_STEPS {
            fixture.apply(&fixture.payload(
                &runbook_fixture::discharge_label(step),
                &runbook_fixture::discharge_body(step),
            ));
        }
        fixture.apply(&fixture.payload(
            "to-reviewing",
            &json!({
                "declare": [{
                    "subject": FINDING,
                    "concerns": SECTION_A,
                    "summary": "the eviction ladder is asserted rather than measured",
                    "blocking": true,
                }],
                "stage": {"to": Stage::Reviewing.as_str()},
            }),
        ));
        // The lock edge's own runbook, discharged here for the same reason the
        // other two are: this suite's subject is which of the FOUR lock
        // components a refusal names. An outstanding runbook refuses first and
        // would mask every one of those assertions with the same message.
        for step in runbook_fixture::REVIEWING_STEPS {
            fixture.apply(&fixture.payload(
                &runbook_fixture::discharge_label(step),
                &runbook_fixture::discharge_body(step),
            ));
        }
        fixture
    }

    /// The lock submission carrying every component **except** `omit`.
    ///
    /// One owner for "the other three are present": a refusal test names only
    /// what it removes, so the three it keeps cannot silently rot into two.
    fn lock_payload(&self, submission: &str, omit: Option<Component>) -> String {
        let present = |component: Component| omit != Some(component);
        let mut declare: Vec<Value> = Vec::new();
        if present(Component::Attestations) {
            for (attestation, section) in ATTESTED {
                declare.push(json!({"subject": attestation, "attests": section}));
            }
        }
        if present(Component::Integrated) {
            declare.push(json!({"subject": INTEGRATED}));
        }
        if present(Component::FindingDisposition) {
            declare.push(json!({
                "subject": FINDING,
                "resolution": "accepted — the ladder's evidence is disclosed as synthetic",
            }));
        }
        let mut body = json!({"declare": declare, "stage": {"to": Stage::Locked.as_str()}});
        if present(Component::Acceptance) {
            body.as_object_mut().unwrap().insert(
                "acceptance".to_owned(),
                json!({"basis": "User accepted the design at the close of review"}),
            );
        }
        self.payload(submission, &body)
    }

    /// The parsed snapshot.
    fn read(&self) -> DesignSnapshot {
        snapshot::parse(&std::fs::read_to_string(&self.snapshot).unwrap()).unwrap()
    }

    /// The run's current stage.
    fn stage(&self) -> Stage {
        self.read().run.stage
    }

    /// The current revision.
    fn revision(&self) -> u64 {
        self.read().run.revision
    }

    /// Every attestation the change log has reported as invalidated.
    fn invalidated(&self) -> BTreeSet<String> {
        self.read()
            .change_log
            .since(0)
            .into_iter()
            .filter(|row| row.event == ChangeEvent::ReviewInvalidated)
            .filter_map(|row| row.subject.as_ref().map(|id| id.as_str().to_owned()))
            .collect()
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

    /// Apply a payload, expecting success.
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

/// A section body, which must open with the section's own heading.
fn section_body(id: &str, note: &str) -> String {
    format!("## {id}\n\n{id} {note}.\n")
}

/// The clearances the fixture claims through the generic evidence route, each
/// bound to the never-edited spine section.
fn claimed() -> Vec<Value> {
    EARLIER
        .into_iter()
        .map(|condition| json!({"condition": condition.as_str(), "subject": SECTION_SPINE}))
        .collect()
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

/// The conjunction, proved for one component: build a lock attempt in which
/// `missing` is the ONLY absent component, and hold the refusal to naming
/// exactly that condition.
fn refuses_without(missing: Component) {
    let fixture = Fixture::reviewing();
    let stderr = fixture.refuse(&fixture.lock_payload("lock", Some(missing)));
    let token = missing.condition().as_str();
    assert!(
        stderr.contains(token),
        "the refusal must name `{token}`, got: {stderr}"
    );
    for other in Component::ALL
        .into_iter()
        .filter(|component| *component != missing)
    {
        let held = other.condition().as_str();
        assert!(
            !stderr.contains(held),
            "`{held}` was present and must not be reported outstanding, got: {stderr}"
        );
    }
    assert_eq!(
        fixture.stage(),
        Stage::Reviewing,
        "a refused lock leaves the run where it was"
    );
}

// ── the four independent refusals ─────────────────────────────────────────

#[test]
fn lock_refuses_without_current_section_attestations() {
    refuses_without(Component::Attestations);
}

#[test]
fn lock_refuses_without_an_integrated_review() {
    refuses_without(Component::Integrated);
}

#[test]
fn lock_refuses_with_an_undisposed_blocking_finding() {
    refuses_without(Component::FindingDisposition);
}

#[test]
fn lock_refuses_without_a_user_acceptance_attestation() {
    refuses_without(Component::Acceptance);
}

// ── and the admission ─────────────────────────────────────────────────────

#[test]
fn lock_admits_with_all_four_present() {
    let fixture = Fixture::reviewing();
    let stdout = fixture.apply(&fixture.lock_payload("lock", None));
    assert_eq!(fixture.stage(), Stage::Locked);

    // EX-5 / design R12: what the lock rests on is disclosed where the lock is
    // taken. v1 trusts a cooperative agent, and the surface says so rather than
    // leaving the residual risk in `design.md`.
    for claim in ["auditable", "not authenticated"] {
        assert!(
            stdout.contains(claim),
            "the lock must disclose `{claim}`, got: {stdout}"
        );
    }
}

// ── DEC-066: a section edit invalidates only that section's clearance ──────

#[test]
fn section_edit_invalidates_only_that_sections_attestation() {
    let fixture = Fixture::reviewing();
    let attestations: Vec<Value> = ATTESTED
        .into_iter()
        .map(|(attestation, section)| json!({"subject": attestation, "attests": section}))
        .collect();
    fixture.apply(&fixture.payload("attest", &json!({"declare": attestations})));
    assert!(
        fixture.invalidated().is_empty(),
        "nothing is invalidated before the edit"
    );

    fixture.apply(&fixture.payload(
        "edit-a",
        &json!({"declare": [{
            "subject": SECTION_A,
            "body": section_body(SECTION_A, "as revised after review"),
        }]}),
    ));

    let invalidated = fixture.invalidated();
    assert!(
        invalidated.contains("att-a"),
        "the edited section's clearance is gone, got: {invalidated:?}"
    );
    assert!(
        !invalidated.contains("att-b"),
        "a sibling section's clearance survives, got: {invalidated:?}"
    );

    // The gate consequence of `only`: two of three attestations are still live,
    // and that is still not "current section attestations".
    let stderr = fixture.refuse(&fixture.lock_payload("lock", Some(Component::Attestations)));
    let token = Condition::SectionAttestationsCurrent.as_str();
    assert!(
        stderr.contains(token),
        "a surviving sibling attestation does not clear `{token}`, got: {stderr}"
    );
}

// ── DEC-074: the v1 choreography ──────────────────────────────────────────

#[test]
fn integrated_adversarial_pass_is_mandatory_section_adversarial_is_opt_in() {
    let fixture = Fixture::reviewing();

    // Section-level adversarial review is opt-in per section: one section takes
    // it, the others keep the v1 human default, and the mixture is admitted.
    let mixed: Vec<Value> = ATTESTED
        .into_iter()
        .map(|(attestation, section)| {
            if section == SECTION_A {
                json!({"subject": attestation, "attests": section, "reviewer": "adversarial"})
            } else {
                json!({"subject": attestation, "attests": section})
            }
        })
        .collect();
    fixture.apply(&fixture.payload("attest", &json!({"declare": mixed})));

    // The integrated pass stays mandatory, and an adversarial *section*
    // attestation is not a substitute for it.
    let stderr =
        fixture.refuse(&fixture.lock_payload("no-integrated", Some(Component::Integrated)));
    let token = Condition::IntegratedReviewPresent.as_str();
    assert!(
        stderr.contains(token),
        "an adversarial section attestation does not clear `{token}`, got: {stderr}"
    );

    // With the integrated pass recorded, the same mixed section review locks.
    fixture.apply(&fixture.lock_payload("lock", None));
    assert_eq!(fixture.stage(), Stage::Locked);
}
