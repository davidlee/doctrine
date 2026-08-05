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
use design_run::attestation::{
    ActKind, AgentAct, AgentActKind, CoveredSet, IntentState, IntentSubject, RecoveryIntent,
    ReviewDisposition, ReviewPolicy, ReviewRef,
};
use design_run::change_log::ChangeEvent;
use design_run::gate::Condition;
use design_run::snapshot::{self, CheckpointGroup, DesignSnapshot};

/// The slice every fixture designs.
const SLICE: &str = "SL-233";
/// Its zero-padded directory name.
const SLICE_NUMBER: &str = "233";
/// The `RV` ledger's file stem, mirroring `review.rs`'s own — one copy here
/// because a binary-only crate gives an integration test no route to the const.
const LEDGER_STEM: &str = "review";
/// The canonical `RV` prefix, mirrored for the same reason.
const REVIEW_PREFIX: &str = "RV";
/// The env var naming the DEC-086 step to crash before, mirroring
/// `commands/design.rs`'s `ENV_DESIGN_FAULT`. Debug builds only, which is what
/// every `cargo test` run is.
const DESIGN_FAULT_ENV: &str = "DOCTRINE_DESIGN_FAULT";
/// The mint journal's file name, beside the snapshot — the same derivation
/// `state::design_journal_path` performs.
const JOURNAL_FILE: &str = "design-journal.toml";

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

/// A blocking finding, raised before the lock is attempted.
const FINDING: &str = "fnd-1";

/// The conditions of the three earlier boundaries, claimed through the generic
/// evidence route because this suite is about the *fourth* boundary.
const EARLIER: [Condition; 6] = [
    Condition::GoverningContextRecorded,
    Condition::InitialConcernsRecorded,
    Condition::BlockingInquiriesDispositioned,
    Condition::UserAcceptsSufficiency,
    Condition::DraftingReadinessAttested,
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
    ///
    /// **No longer injective, since SL-244 PHASE-05 `T9`.** DEC-126 folded
    /// `integrated-review-present` and `blocking-findings-disposed` into the one
    /// `review-disposition-attested` row, so two components now clear the same
    /// condition. The components stay four because each is still repaired by its
    /// own act — what collapsed is the *vocabulary*, not the conjunction.
    const fn condition(self) -> Condition {
        match self {
            Component::Attestations => Condition::SectionAttestationsCurrent,
            Component::Integrated | Component::FindingDisposition => {
                Condition::ReviewDispositionAttested
            }
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
        let (fixture, entry) = Fixture::at_the_reviewing_edge();
        fixture.apply(&entry);
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

    /// The same run one submission short of `reviewing`, with the entry payload
    /// **returned rather than applied**.
    ///
    /// Split out of [`Fixture::reviewing`] so a test can interrupt the very
    /// submission that mints the pass (`VA-1`). The payload pins the revision it
    /// was built at, which is exactly what a resumed submission re-presents: a
    /// crashed apply never persisted a snapshot, so the run is still there.
    fn at_the_reviewing_edge() -> (Fixture, String) {
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
        // SL-244 `T8` (`D2`) — both mechanisms live at once. The `evidence`
        // claims below are what clears the gate *today*; the acts are what will
        // clear it once `T10` flips the evaluator, and they land now so that
        // flip is a change of mechanism with no change of fixture.
        //
        // `exploring → inquiring` owes two conditions' acts and `ApplyRequest`
        // holds one checkpoint act, so it owes two submissions.
        fixture.apply(&fixture.payload(
            "governance",
            &json!({"checkpoint_act": design_act::checkpoint_act(
                ActKind::GovernanceConfirmed,
                "the governing artefacts are the ones found",
            )}),
        ));
        // DEC-121's two acts by two actors, in one submission — `T6`'s build
        // order: the declaration is constructed and fingerprinted before the act
        // that confirms it, so no caller computes a digest.
        //
        // The blocking set is **empty**, and that is a claim rather than a gap:
        // this suite declares no inquiries at all, because its subject is the
        // FOURTH boundary. An empty inquiry map is an observable fact — it stays
        // empty for the run's whole life, since `fnd-1` is a review finding and
        // findings are not nodes — so the coverage these two acts carry is still
        // current at the lock.
        fixture.apply(&fixture.payload(
            "graph",
            &json!({
                "agent_declaration": design_act::agent_declaration(
                    AgentAct::BlockingSetDeclared { blocking: BTreeSet::new() },
                    "nothing blocks: this run interrogates no questions",
                ),
                "checkpoint_act": design_act::checkpoint_act(
                    ActKind::GraphReviewed,
                    "the empty blocking set is right",
                ),
            }),
        ));
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
        // `inquiring → drafting` owes `user-accepts-sufficiency`.
        fixture.apply(&fixture.payload(
            "sufficiency",
            &json!({"checkpoint_act": design_act::checkpoint_act(
                ActKind::SufficiencyAccepted,
                "there is nothing outstanding to interrogate",
            )}),
        ));
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
        // SL-244 `T10` — `drafting → reviewing` also owes
        // `materialisation-current`, and the evaluator derives it from the
        // authored watermark rather than reading the claim `EARLIER` carries.
        // So the ladder materialises the design it has just drafted, which is
        // the act the condition has always named and which the incumbent scan
        // let a claim stand in for. Before `entry` is built, for the reason
        // spelled out below it.
        run(&fixture.root, &["design", "materialise", SLICE, "-p", "."]);
        // `drafting → reviewing` owes `drafting-readiness-attested`, which is an
        // AGENT act. Recorded before the entry payload is *built*, not merely
        // before it is applied: `payload` pins the revision it is built at, so a
        // submission between the two would leave `entry` stale — and the two
        // crash tests re-apply `entry` after a fault, which is where that would
        // surface.
        fixture.apply(&fixture.payload(
            "ready",
            &json!({"agent_declaration": design_act::agent_declaration(
                AgentAct::DraftingReady,
                "the three sections are drafted and ready to review",
            )}),
        ));
        let entry = fixture.payload(
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
        );
        (fixture, entry)
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
        // `Component::Integrated` declares nothing. Entry to `reviewing` mints the
        // pass (DEC-125), so its presence is guaranteed by construction and the
        // condition is falsified by staleness alone — see [`Fixture::stale_the_pass`].
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

    /// The two checkpoint acts the `reviewing → locked` crossing owes, each in
    /// its own submission — `ApplyRequest` holds one (SL-244 `T8`).
    ///
    /// Keyed by **condition** rather than by component, for the reason
    /// [`refuses_without`] is: since DEC-126's fold two components clear
    /// `review-disposition-attested`, so omitting either must withhold its act,
    /// or the component under test would be repaired by the other one's.
    ///
    /// Called at each lock site rather than folded into [`Fixture::lock_payload`]
    /// because *when* an act is given is load-bearing and invisible today.
    /// `DesignAccepted` carries `EverySection` coverage, so it must be given
    /// after the last section edit a test makes — and nothing reads these acts
    /// until `T10`, so a builder that hid the moment would hide the one thing
    /// worth seeing.
    fn record_lock_acts(&self, tag: &str, omit: Option<Component>) {
        let cleared = |condition: Condition| omit.map(Component::condition) != Some(condition);
        if cleared(Condition::ReviewDispositionAttested) {
            // The `Waived` arm, necessarily: `Conducted` requires the ledger to
            // be concluded, and no verb sets that marker until IMP-392, so a
            // conducted ladder is unreachable this phase (sheet `A3`).
            self.apply(&self.payload(
                &format!("{tag}-dispose"),
                &json!({"checkpoint_act": design_act::review_disposed(
                    "the pass is disposed of at the close of review",
                    ReviewDisposition::Waived {
                        reason: "no adversarial reviewer was engaged on this run".to_owned(),
                    },
                )}),
            ));
        }
        if cleared(Condition::UserAcceptanceAttested) {
            self.apply(&self.payload(
                &format!("{tag}-accept"),
                &json!({"checkpoint_act": design_act::checkpoint_act(
                    ActKind::DesignAccepted,
                    "User accepted the design at the close of review",
                )}),
            ));
        }
    }

    /// Edit a covered section so the minted pass no longer covers current
    /// content — the only way the currency lamp can now be falsified (F2).
    ///
    /// Its own submission, deliberately: an edit batched with the lock payload
    /// would race the attestations declared beside it, and the lock tests need
    /// `section-attestations-current` to stay cleared so the refusal isolates
    /// `integrated-review-present`. The lock payload re-attests afterwards.
    fn stale_the_pass(&self) {
        self.apply(&self.payload(
            "stale-the-pass",
            &json!({"declare": [{
                "subject": SECTION_B,
                "body": section_body(SECTION_B, "as revised after the pass opened"),
            }]}),
        ));
    }

    /// Leave `reviewing` and come back — the re-entry DEC-125 mints a fresh pass on.
    fn re_enter_reviewing(&self, tag: &str) {
        self.apply(&self.payload(
            &format!("{tag}-out"),
            &json!({"stage": {
                "to": Stage::Drafting.as_str(),
                "reason": "reopening the draft after the review pass",
            }}),
        ));
        // **No second `DraftingReady` here**, and the omission is measured
        // rather than forgotten (SL-244 `T8`). The re-crossing does evaluate
        // `drafting-readiness-attested` — `EdgeLocal` governs which edges ask a
        // row, not when an act expires — but the act given at the first crossing
        // still satisfies it: its `Artefact` coverage is inert by construction,
        // exactly as the incumbent evidence claim persists across this same
        // re-entry today. An act nothing owes and nothing asserts is the fixture
        // ceremony `F33` already refused once.
        self.apply(&self.payload(
            &format!("{tag}-in"),
            &json!({"stage": {"to": Stage::Reviewing.as_str()}}),
        ));
    }

    /// The `RV` entity dirs on disk, sorted — what the mint actually authored.
    ///
    /// Numeric names only: every entity dir has a `NNN-slug` alias symlink beside
    /// it, and `is_dir()` follows symlinks, so a naive listing double-counts.
    fn minted_reviews(&self) -> Vec<String> {
        let mut found: Vec<String> = std::fs::read_dir(self.root.join(common::REVIEW_DIR))
            .into_iter()
            .flatten()
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.chars().all(|byte| byte.is_ascii_digit()))
            .collect();
        found.sort();
        found
    }

    /// The `RV`s that carry an authored ledger — the subset of
    /// [`Fixture::minted_reviews`] that is a **record** rather than a claimed
    /// directory.
    ///
    /// The two differ exactly across DEC-086's tolerated window: a crash between
    /// the id claim and the id journal leaves a claimed, empty directory that no
    /// journal names and no ledger fills. "How many RVs exist" is a question
    /// about ledgers, so it is asked here rather than of the directory listing.
    fn authored_reviews(&self) -> Vec<String> {
        self.minted_reviews()
            .into_iter()
            .filter(|name| {
                self.root
                    .join(common::REVIEW_DIR)
                    .join(name)
                    .join(format!("{LEDGER_STEM}-{name}.toml"))
                    .is_file()
            })
            .collect()
    }

    /// The one journalled mint intent, which every recovery test expects to exist.
    fn intent(&self) -> RecoveryIntent {
        let journal: CheckpointGroup =
            match std::fs::read_to_string(self.snapshot.with_file_name(JOURNAL_FILE)) {
                Ok(text) => toml::from_str(&text).expect("the journal parses"),
                Err(_) => CheckpointGroup::default(),
            };
        assert_eq!(
            journal.intents.len(),
            1,
            "exactly one mint intent is journalled: {:?}",
            journal.intents
        );
        journal.intents.into_iter().next().expect("one intent")
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

    /// Apply a payload with a fault injected before DEC-086's `step`, expecting
    /// the process to die there.
    ///
    /// Out of process on purpose: a **crash** is only observable across a process
    /// boundary. An injected `Err` would unwind, run cleanup, and exercise the
    /// paths that already work — the checkpoint suite makes that argument at
    /// length, and this is the review arm of the same seam.
    fn crash_at(&self, body: &str, step: &str) {
        let out = common::doctrine_cmd(&self.root)
            .args(["design", "apply", SLICE, "-p", ".", "--input", body])
            .env(DESIGN_FAULT_ENV, step)
            .output()
            .expect("spawn doctrine");
        assert!(
            !out.status.success(),
            "the injected fault at `{step}` did not stop the run: {}",
            String::from_utf8_lossy(&out.stdout)
        );
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
    if missing == Component::Integrated {
        fixture.stale_the_pass();
    }
    // After the edit above, so `DesignAccepted`'s `EverySection` coverage is
    // taken over the content the lock is attempted on (SL-244 `T8`).
    fixture.record_lock_acts("lock", Some(missing));
    let stderr = fixture.refuse(&fixture.lock_payload("lock", Some(missing)));
    let token = missing.condition().as_str();
    assert!(
        stderr.contains(token),
        "the refusal must name `{token}`, got: {stderr}"
    );
    // Filtered by *condition*, not by component: since DEC-126's fold two
    // components share `review-disposition-attested`, and filtering by component
    // would assert that the token the refusal must name is also absent.
    for other in Component::ALL
        .into_iter()
        .filter(|component| component.condition() != missing.condition())
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
    fixture.record_lock_acts("lock", None);
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
    fixture.record_lock_acts("lock", Some(Component::Attestations));
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

    // The run-level pass stays mandatory, and an adversarial *section*
    // attestation is not a substitute for it: with the pass staled by an edit,
    // the mixed section review does not lock.
    fixture.stale_the_pass();
    fixture.record_lock_acts("no-integrated", Some(Component::Integrated));
    let stderr =
        fixture.refuse(&fixture.lock_payload("no-integrated", Some(Component::Integrated)));
    let token = Condition::ReviewDispositionAttested.as_str();
    assert!(
        stderr.contains(token),
        "an adversarial section attestation does not clear `{token}`, got: {stderr}"
    );

    // Re-entering `reviewing` opens a fresh pass over the edited content, and
    // the same mixed section review locks. Since DEC-125's mint, that — not a
    // declaration — is how the condition is cleared again.
    fixture.re_enter_reviewing("reopen");
    // After the re-entry, so the disposition is given over the pass it disposes
    // of — the re-entry minted a second one (SL-244 `T8`).
    fixture.record_lock_acts("lock", None);
    fixture.apply(&fixture.lock_payload("lock", None));
    assert_eq!(fixture.stage(), Stage::Locked);
}

// ── DEC-125: the run's review pass, minted on entry to `reviewing` ─────────

/// `VT-1` (a): entry to `reviewing` mints exactly one `RV`, and the run holds it
/// as its pass over the content that was current at entry.
#[test]
fn mints_a_review_pass_on_entry_to_reviewing() {
    let fixture = Fixture::reviewing();

    assert_eq!(
        fixture.minted_reviews(),
        vec!["001".to_owned()],
        "one pass, one RV"
    );
    let pass = fixture
        .read()
        .review
        .pass
        .expect("a run in `reviewing` holds a pass");
    assert_eq!(
        pass.review,
        ReviewRef::new("RV-001"),
        "the run names the RV"
    );
    assert!(
        pass.is_current(&fixture.read().sections.fingerprints()),
        "the pass covers the content it opened over"
    );

    // The RV is a real ledger against the slice, not a bare directory.
    let ledger = std::fs::read_to_string(
        fixture
            .root
            .join(common::REVIEW_DIR)
            .join("001/review-001.toml"),
    )
    .expect("the minted RV carries its ledger");
    assert!(ledger.contains(SLICE), "the RV reviews the slice: {ledger}");
}

/// `VT-1` (b) / `EX-4`: a later entry **replaces** the pass and never reopens the
/// old one. The prior `RV` stays on disk — an authored record is never rolled
/// back — but the run's pass names the new one, over the new content.
#[test]
fn re_entry_replaces_the_review_pass() {
    let fixture = Fixture::reviewing();
    let first = fixture
        .read()
        .review
        .pass
        .expect("the first entry minted a pass");

    fixture.stale_the_pass();
    fixture.re_enter_reviewing("second");

    assert_eq!(
        fixture.minted_reviews(),
        vec!["001".to_owned(), "002".to_owned()],
        "the re-entry mints a second RV and leaves the first authored"
    );
    let second = fixture
        .read()
        .review
        .pass
        .expect("the re-entry minted a pass");
    assert_eq!(
        second.review,
        ReviewRef::new("RV-002"),
        "replaced, not reopened"
    );
    assert_ne!(
        second.covered, first.covered,
        "opened over the edited content"
    );
    assert!(second.is_current(&fixture.read().sections.fingerprints()));
}

// ── VA-1: the mint is idempotent under interruption ────────────────────────
//
// Two crash points, mirroring the pair the checkpoint suite already runs over
// the knowledge arm — because the two windows make different promises. Before
// the id journal, DEC-086 promises only that no unnamed record exists; from the
// id journal onward it promises the exact reserved id. One test cannot carry
// both, and the second is the one that pins `review::materialise_review_at`.
//
// These live here rather than in `e2e_design_checkpoint.rs` (where the phase
// sheet put them) because the ladder to `reviewing` is this fixture's: minting
// there would have meant a second copy of it. The fault seam is two lines of
// env, so the cheap half moved instead of the expensive half — and the
// checkpoint suite stays byte-unchanged, which is the control that PHASE-04's
// journal rewrite did not move the wire form.

/// `VA-1` (a) — interrupted **between the id claim and the id journal**, the
/// resumed submission authors exactly one pass.
///
/// The tolerated window is asserted rather than skirted: a hard exit runs no
/// cleanup, so the dead claim's empty directory survives. That is exactly
/// DEC-086's "empty or partial reservation", and it is why the observation here
/// is the **ledger** and not the directory listing — what may never exist is an
/// authored RV nothing can name.
#[test]
fn a_mint_interrupted_before_its_id_journal_authors_one_pass() {
    let (fixture, entry) = Fixture::at_the_reviewing_edge();

    fixture.crash_at(&entry, "id-journal");

    let intent = fixture.intent();
    assert_eq!(
        intent.subject(),
        &IntentSubject::ReviewPass,
        "the journalled intent is the run's pass, not a checkpoint"
    );
    assert_eq!(
        intent.state(),
        IntentState::Journalled,
        "step 1 landed and step 3 did not"
    );
    assert_eq!(
        intent.reserved_record(),
        None,
        "no canonical id is journalled, so none was promised"
    );
    assert!(
        fixture.authored_reviews().is_empty(),
        "and no ledger exists that nothing can name: {:?}",
        fixture.authored_reviews()
    );

    // The retry resumes the same submission.
    fixture.apply(&entry);

    let authored = fixture.authored_reviews();
    assert_eq!(
        authored.len(),
        1,
        "exactly one RV is authored — the resumed mint did not author a second: {authored:?}"
    );
    assert_eq!(
        fixture.minted_reviews().len(),
        2,
        "the dead claim's directory outlives the crash, which is the tolerated \
         reservation and precisely why the ledger is the observation: {:?}",
        fixture.minted_reviews()
    );
    let pass = fixture
        .read()
        .review
        .pass
        .expect("the resumed mint recorded the pass");
    assert_eq!(
        pass.review,
        ReviewRef::new(format!("{REVIEW_PREFIX}-{}", authored[0])),
        "and the run names the RV that exists"
    );
    assert_eq!(
        fixture.intent().state(),
        IntentState::Complete,
        "with the journal closed out"
    );
}

/// `VA-1` (b) — interrupted **after** the id journal, the resume runs against the
/// exact reserved id: one RV, one reservation consumed.
///
/// This is the half that exercises `review::materialise_review_at` on the tree —
/// the resumed mint claims nothing and writes into the reservation the journal
/// names. The expected id is read back out of the journal FILE rather than
/// recomputed here: "an RV exists" and "the RV the crash promised exists" are
/// different claims, and only the second one is idempotency.
#[test]
fn a_mint_interrupted_after_its_id_journal_resumes_the_reserved_pass() {
    let (fixture, entry) = Fixture::at_the_reviewing_edge();

    fixture.crash_at(&entry, "record-materialise");

    let reserved = fixture
        .intent()
        .reserved_record()
        .expect("step 3 journalled the claimed canonical id")
        .to_owned();
    assert_eq!(
        fixture.intent().state(),
        IntentState::Reserved,
        "step 3 landed and step 4 did not"
    );
    assert!(
        fixture.authored_reviews().is_empty(),
        "no authored bytes yet: {:?}",
        fixture.authored_reviews()
    );

    fixture.apply(&entry);

    assert_eq!(
        fixture.minted_reviews(),
        vec!["001".to_owned()],
        "one reservation consumed — a fresh claim would have made a second"
    );
    assert_eq!(
        fixture.authored_reviews(),
        vec!["001".to_owned()],
        "and exactly one RV is authored, in it"
    );
    let pass = fixture
        .read()
        .review
        .pass
        .expect("the resumed mint recorded the pass");
    assert_eq!(
        pass.review,
        ReviewRef::new(reserved.clone()),
        "the run names the RESERVED id {reserved}, not a fresh one"
    );
    assert_eq!(
        fixture.intent().state(),
        IntentState::Complete,
        "with the journal closed out"
    );
}

// ── SL-244 T8: the ladder's acts are recorded, not merely submitted ────────

/// `T8`'s control for this suite — the acts the ladder adds are *recorded*, in
/// the shape the crossing they belong to will read after `T10`.
///
/// Necessary because `T8` is additive by design (`D2`): the incumbent evidence
/// scan clears the gate until `T10`, so nothing else here would fail if a
/// `checkpoint_act` key deserialised into nothing. The suite would stay green
/// and the fixture would be inert JSON — which is precisely the state `T10` is
/// supposed to be able to flip into without touching a fixture.
///
/// It asserts three things and deliberately not a fourth. The **act set**, by
/// kind, because a ladder that crosses four edges owes an act at each. The
/// **disposition arm**, because `Waived` is the only reachable one this phase
/// and a `Conducted` record here would be a silent deferral. And
/// `DesignAccepted`'s **coverage currency**, because that is the one placement
/// question this suite can get wrong today and not find out until `T10` — the
/// act is given after the last section edit or it is worthless. What it does not
/// assert is the whole record: the engine slots are `T5`/`T6`'s to pin, and the
/// unit suite already does so by payload.
#[test]
fn the_ladder_records_an_act_at_every_edge_it_crosses() {
    let fixture = Fixture::reviewing();
    // The edit is what gives the coverage assertion teeth. Recorded acts are
    // taken over the state at the moment they are given, so an acceptance given
    // *before* this edit would carry a covered map the edit invalidates — and
    // nothing today would say so. Swapping these two lines is the control.
    fixture.stale_the_pass();
    fixture.record_lock_acts("lock", None);
    let held = fixture.read();

    let acts: BTreeSet<ActKind> = held.acts.acts.iter().map(|act| act.act).collect();
    assert_eq!(
        acts,
        BTreeSet::from([
            ActKind::GovernanceConfirmed,
            ActKind::GraphReviewed,
            ActKind::SufficiencyAccepted,
            ActKind::ReviewDisposed,
            ActKind::DesignAccepted,
        ]),
        "one user act per condition, across all four crossings"
    );
    let declared: BTreeSet<AgentActKind> = held
        .declarations
        .declarations
        .iter()
        .map(|declaration| declaration.act.kind())
        .collect();
    assert_eq!(
        declared,
        BTreeSet::from([
            AgentActKind::BlockingSetDeclared,
            AgentActKind::DraftingReady,
        ]),
        "and the two the agent authors — the one a `GraphReviewed` confirms, and \
         the drafting judgement that stands alone"
    );

    let disposed = held
        .acts
        .acts
        .iter()
        .find(|act| act.act == ActKind::ReviewDisposed)
        .and_then(|act| act.disposition.as_ref())
        .expect("the disposition act carries its disposition");
    assert!(
        matches!(disposed.disposition, ReviewDisposition::Waived { .. }),
        "the reachable arm: `Conducted` needs a concluded ledger, and no verb \
         sets that marker until IMP-392 — got {:?}",
        disposed.disposition
    );
    assert_eq!(
        disposed.pass,
        held.review
            .pass
            .as_ref()
            .expect("a run in `reviewing` holds a pass")
            .review,
        "given over the pass the run is actually on, which the caller never names"
    );

    let accepted = held
        .acts
        .acts
        .iter()
        .find(|act| act.act == ActKind::DesignAccepted)
        .expect("the acceptance act is recorded");
    let Some(CoveredSet::Sections(ref covered)) = accepted.covered else {
        panic!(
            "the acceptance covers every section, got {:?}",
            accepted.covered
        );
    };
    assert!(
        covered.is_current(&held.sections.fingerprints()),
        "and covers the content it was given over — the placement `T10` will be \
         the first to check, asserted here where it is still cheap"
    );
}

// ── ISS-310: the required lanes are the run's, and the policy is mutable ──

/// The deliberate hole, end to end: sections reviewed only adversarially do not
/// clear the gate under `human-only`, and loosening the run's policy clears it
/// **without a single new attestation**.
///
/// Asserted rather than left implicit precisely because it is a hole by choice —
/// the fence is authority and visibility, not prohibition — so closing it later
/// is a visible break rather than a silent tightening. Both halves of that fence
/// are here: the change is refused unless it is presented as the user's, and an
/// accepted change names its old and new value in the log.
///
/// The lock attempts OMIT the attestations rather than re-declaring them, which
/// is the condition that makes this test about the lane: `lock_payload`'s default
/// reviewer is human, so a re-declaration would quietly repair the very state
/// under test.
#[test]
fn loosening_the_policy_clears_the_gate() {
    let fixture = Fixture::reviewing();

    // Every section reviewed — adversarially, and nothing else outstanding.
    let adversarial: Vec<Value> = ATTESTED
        .into_iter()
        .map(|(attestation, section)| {
            json!({"subject": attestation, "attests": section, "reviewer": "adversarial"})
        })
        .collect();
    fixture.apply(&fixture.payload("attest", &json!({"declare": adversarial})));

    // Once, ahead of both lock attempts: nothing between them edits a section,
    // so the acceptance act's `EverySection` coverage is still current at the
    // second (SL-244 `T8`).
    fixture.record_lock_acts("lock", Some(Component::Attestations));
    let token = Condition::SectionAttestationsCurrent.as_str();
    let stderr = fixture
        .refuse(&fixture.lock_payload("lock-under-human-only", Some(Component::Attestations)));
    assert!(
        stderr.contains(token),
        "a review in the wrong lane leaves `{token}` outstanding, got: {stderr}"
    );

    // Changing the policy is a user act. Without an acceptance it is refused, and
    // a refused change moves nothing — an agent cannot relax the rules as
    // housekeeping.
    let unaccepted = fixture.payload(
        "policy-unaccepted",
        &json!({"review_policy": {"policy": "adversarial-only"}}),
    );
    let stderr = fixture.refuse(&unaccepted);
    assert!(
        stderr.contains("acceptance"),
        "the refusal must name the missing acceptance, got: {stderr}"
    );
    assert_eq!(
        fixture.read().run.review_policy,
        ReviewPolicy::HumanOnly,
        "a refused change leaves the policy where it was"
    );

    // Presented as the user's, it lands — and it is legible after the fact.
    fixture.apply(&fixture.payload(
        "policy",
        &json!({"review_policy": {
            "policy": "adversarial-only",
            "acceptance": {"basis": "the adversarial reviewer reads for us on this run"},
        }}),
    ));
    assert_eq!(
        fixture.read().run.review_policy,
        ReviewPolicy::AdversarialOnly
    );
    let logged: Vec<String> = fixture
        .read()
        .change_log
        .since(0)
        .into_iter()
        .filter(|row| row.event == ChangeEvent::ReviewPolicyChanged)
        .flat_map(|row| row.terms.iter().map(|term| term.value().to_owned()))
        .collect();
    assert_eq!(
        logged,
        vec![
            ReviewPolicy::HumanOnly.as_str(),
            ReviewPolicy::AdversarialOnly.as_str()
        ],
        "one row, naming what the policy was and what it became"
    );

    // Re-declaring the policy already in force is not a change, and the log does
    // not report an act that did not happen.
    fixture.apply(&fixture.payload(
        "policy-again",
        &json!({"review_policy": {
            "policy": "adversarial-only",
            "acceptance": {"basis": "restating what is already true"},
        }}),
    ));
    let rows = fixture
        .read()
        .change_log
        .since(0)
        .into_iter()
        .filter(|row| row.event == ChangeEvent::ReviewPolicyChanged)
        .count();
    assert_eq!(rows, 1, "a no-op re-declaration emits no row");

    // And the attestations recorded before any of this now clear the gate.
    fixture.apply(&fixture.lock_payload("lock", Some(Component::Attestations)));
    assert_eq!(fixture.stage(), Stage::Locked);
}
