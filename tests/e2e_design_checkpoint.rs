//! SL-233 PHASE-05 — the recoverable checkpoint, over the built binary
//! (DEC-083/DEC-086/DEC-088, design §5.5).
//!
//! Five behaviours, and they are black-box for a reason that is not stylistic: a
//! **crash** is only observable across a process boundary. An injected `Err`
//! unwinds, runs cleanup, and exercises exactly the paths that already work;
//! only a process that dies mid-protocol leaves the state DEC-086's ordering is
//! a claim about. So the fault is injected through the environment
//! (`DOCTRINE_DESIGN_FAULT=<step>`, debug builds only) and the assertion is made
//! against what survived on disk.
//!
//! The pure model is `#[path]`-included rather than imported, the CHR-014 idiom
//! this crate already uses: it is binary-only (no `src/lib.rs`), so an
//! integration test can only spawn the binary — and including the leaf means the
//! journal these tests read back is parsed by the same bytes the binary wrote it
//! with, rather than by a hand-rolled second reader.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::{Path, PathBuf};
use std::process::Output;

mod common;

/// The pure model, from source. `design_run` is a leaf with crate out-degree
/// zero, so it compiles standalone here exactly as it does in the binary.
#[path = "../src/design_run/mod.rs"]
#[allow(
    dead_code,
    unused_imports,
    reason = "the whole leaf tree is included; no single test exercises all of it"
)]
mod design_run;

use design_run::attestation::{IntentState, RecoveryIntent};
use design_run::ids::DesignId;
use design_run::inquiry::{Disposition, InquiryLifecycle};
use design_run::snapshot::{self, CheckpointGroup, DesignSnapshot};

/// The slice every fixture designs.
const SLICE: &str = "SL-233";
/// Its zero-padded directory name.
const SLICE_NUMBER: &str = "233";
/// The knowledge tree the fixtures create records under — one copy, here, because
/// `src/kinds/dirs.rs` names the `.doctrine/knowledge` root but not the per-kind
/// leaf, and a binary-only crate gives an integration test no other route to it.
const KNOWLEDGE_DIR: &str = ".doctrine/knowledge";
/// The record file stem, mirroring `knowledge.rs`'s `RECORD_STEM` for the same
/// reason.
const RECORD_STEM: &str = "record";
/// The provisional ref the FIRST validation pass stands in for a `create`,
/// before DEC-086 step 3 has claimed a real id — `design.rs`'s
/// `PROVISIONAL_RECORD_ID` of 0, rendered for the decision kind. One copy here
/// because a binary-only crate gives an integration test no route to the const.
const PROVISIONAL_RECORD: &str = "DEC-000";

// ── fixture ───────────────────────────────────────────────────────────────

/// A started design run in a throwaway tree.
struct Fixture {
    _tmp: tempfile::TempDir,
    root: PathBuf,
    /// Learned from `design start`'s own output, so no test re-types a state
    /// path (STD-001).
    snapshot: PathBuf,
    uid: String,
}

impl Fixture {
    /// A run with no authored document and one open inquiry node to dispose.
    fn start() -> Fixture {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::create_dir_all(root.join(common::SLICE_DIR).join(SLICE_NUMBER)).unwrap();
        let out = ok(spawn(&root, &["design", "start", SLICE, "-p", "."], None));
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
        Fixture {
            _tmp: tmp,
            root,
            snapshot,
            uid,
        }
    }

    /// The checkpoint journal, beside the snapshot — the path is *derived* from
    /// the snapshot's, exactly as `state::design_journal_path` derives it.
    fn journal_path(&self) -> PathBuf {
        self.snapshot.with_file_name("design-journal.toml")
    }

    /// The journal as the binary wrote it. Absent is empty, not an error.
    fn journal(&self) -> CheckpointGroup {
        match std::fs::read_to_string(self.journal_path()) {
            Ok(text) => toml::from_str(&text).expect("the journal parses"),
            Err(_) => CheckpointGroup::default(),
        }
    }

    /// The one journalled intent, which every test here expects to exist.
    fn intent(&self) -> RecoveryIntent {
        let journal = self.journal();
        assert_eq!(
            journal.intents.len(),
            1,
            "exactly one checkpoint intent is journalled: {:?}",
            journal.intents
        );
        journal.intents.into_iter().next().expect("one intent")
    }

    /// The parsed snapshot.
    fn read(&self) -> DesignSnapshot {
        snapshot::parse(&std::fs::read_to_string(&self.snapshot).unwrap()).unwrap()
    }

    /// The current revision.
    fn revision(&self) -> u64 {
        self.read().run.revision
    }

    /// A payload envelope asserting the current revision.
    fn envelope(&self, submission: &str) -> String {
        format!(
            "\"run_uid\":\"{}\",\"known_revision\":{},\"submission_id\":\"{submission}\"",
            self.uid,
            self.revision()
        )
    }

    /// Apply a payload, expecting success; returns stdout.
    fn apply(&self, body: &str) -> String {
        ok(self.run_apply(body, None))
    }

    /// Apply a payload, expecting refusal; returns stderr.
    fn refuse(&self, body: &str) -> String {
        let out = self.run_apply(body, None);
        assert!(
            !out.status.success(),
            "the apply unexpectedly succeeded: {}",
            String::from_utf8_lossy(&out.stdout)
        );
        String::from_utf8_lossy(&out.stderr).to_string()
    }

    /// Apply a payload with a fault injected before `step`, expecting the process
    /// to die there.
    fn crash_at(&self, body: &str, step: &str) {
        let out = self.run_apply(body, Some(step));
        assert!(
            !out.status.success(),
            "the injected fault at `{step}` did not stop the run: {}",
            String::from_utf8_lossy(&out.stdout)
        );
    }

    fn run_apply(&self, body: &str, fault: Option<&str>) -> Output {
        spawn(
            &self.root,
            &["design", "apply", SLICE, "-p", ".", "--input", body],
            fault,
        )
    }

    /// Declare one open inquiry node.
    fn raise(&self, node: &str, submission: &str) {
        self.apply(&format!(
            "{{{},\"declare\":[{{\"subject\":\"{node}\",\"question\":\"q\"}}]}}",
            self.envelope(submission)
        ));
    }

    /// Every authored knowledge record's `record-NNN.toml`, sorted — the
    /// observation "how many records exist, and which".
    fn records(&self) -> Vec<PathBuf> {
        let mut found = Vec::new();
        collect_records(&self.root.join(KNOWLEDGE_DIR), &mut found);
        found.sort();
        found
    }

    /// The disposition recorded for `node`, which must be resolved.
    fn disposition(&self, node: &str) -> Disposition {
        let run = self.read();
        let held = run
            .map
            .inquiry
            .get(&id(node))
            .unwrap_or_else(|| panic!("the run holds {node}"));
        assert_eq!(
            held.lifecycle(),
            InquiryLifecycle::Resolved,
            "{node} is resolved"
        );
        held.disposition()
            .expect("a resolved node carries a disposition")
            .clone()
    }

    /// The disposition recorded for `node`, or `None` when it carries none.
    ///
    /// The non-asserting sibling of [`Fixture::disposition`]: a refused
    /// checkpoint leaves its node unresolved, and "was nothing recorded?" is
    /// the observation, not a fixture failure.
    fn disposition_of(&self, node: &str) -> Option<Disposition> {
        self.read()
            .map
            .inquiry
            .get(&id(node))
            .and_then(|held| held.disposition().cloned())
    }
}

/// Walk one knowledge tree for `record-NNN.toml` files.
///
/// Symlinks are skipped, not followed: every record scaffold mints an
/// `NNN-slug -> NNN` alias, so a following walk counts each record twice and
/// would report every single-record tree as a duplicate.
fn collect_records(dir: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            collect_records(&path, found);
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(RECORD_STEM) && name.ends_with(".toml"))
        {
            found.push(path);
        }
    }
}

/// Run the built binary in `root`, optionally with a fault injected.
fn spawn(root: &Path, args: &[&str], fault: Option<&str>) -> Output {
    let mut command = common::doctrine_cmd(root);
    command
        .args(args)
        .env_remove("DOCTRINE_DESIGN_FAULT")
        // A fixture repo has no remote, so the reservation reach degrades to
        // local. Declaring the opt-in keeps that a decision rather than a prompt.
        .env("DOCTRINE_RESERVATION_FALLBACK", "1");
    if let Some(step) = fault {
        command.env("DOCTRINE_DESIGN_FAULT", step);
    }
    command.output().expect("spawn doctrine")
}

/// Assert success and return stdout.
fn ok(out: Output) -> String {
    assert!(
        out.status.success(),
        "doctrine failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// Parse a run-local id, where a malformed literal is a test bug.
fn id(raw: &str) -> DesignId {
    DesignId::parse(raw).expect("a well-formed run-local id")
}

/// A checkpoint declaration that asks Doctrine to CREATE a decision record.
fn create_checkpoint(fixture: &Fixture, submission: &str, checkpoint: &str, node: &str) -> String {
    format!(
        "{{{},\"declare\":[{{\"subject\":\"{checkpoint}\",\"disposes\":\"{node}\",\
         \"dispose\":{{\"form\":\"create\",\"kind\":\"decision\",\
         \"title\":\"Checkpointed decision\"}}}}]}}",
        fixture.envelope(submission)
    )
}

/// Put `declaration` into the run as an ACCEPTED PROPOSAL rather than as the
/// coordinator's own declaration — export an assignment against `obligation`,
/// have a delegate propose the declaration back, then accept.
///
/// The two acts are separate payloads because a proposal-bearing payload may
/// carry no other writer act (PHASE-10 `EX-2`). The acceptance carries the
/// delegation act alone, which is exactly why `plan_checkpoints`' walk over
/// `request.declare` cannot see what it brings in (`RV-324` F-1).
fn accept_proposing(fixture: &Fixture, obligation: &str, declaration: &str) -> Output {
    fixture.apply(&format!(
        "{{{},\"delegation\":{{\"act\":\"export\",\"id\":\"dlg-1\",\
         \"obligation\":\"{obligation}\"}}}}",
        fixture.envelope("export")
    ));
    fixture.apply(&format!(
        "{{{},\"delegation\":{{\"act\":\"propose\",\"id\":\"dlg-1\",\
         \"by\":\"delegate-session-7\",\"summary\":\"the checkpoint is ready\",\
         \"declare\":[{declaration}]}}}}",
        fixture.envelope("propose")
    ));
    fixture.run_apply(
        &format!(
            "{{{},\"delegation\":{{\"act\":\"accept\",\"id\":\"dlg-1\"}}}}",
            fixture.envelope("accept")
        ),
        None,
    )
}

/// The numeric directory name a canonical ref like `DEC-001` names.
fn numeric_dir(canonical: &str) -> String {
    canonical
        .rsplit_once('-')
        .map(|(_, tail)| tail.to_owned())
        .expect("a canonical ref carries a numeric tail")
}

// ── the five behaviours ───────────────────────────────────────────────────

/// EX-2 — a crash **before** DEC-086 step 3 leaves no unidentified authored
/// record. The reservation may exist (an empty claimed directory is exactly the
/// "empty or partial reservation" DEC-086 tolerates); what may not exist is an
/// authored record no journal names.
#[test]
fn crash_before_id_journal_leaves_no_unidentified_record() {
    let fixture = Fixture::start();
    fixture.raise("inq-1", "seed");
    let payload = create_checkpoint(&fixture, "sub-cp", "cp-1", "inq-1");

    fixture.crash_at(&payload, "id-journal");

    let intent = fixture.intent();
    assert_eq!(intent.submission(), "sub-cp", "keyed by the submission");
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
        fixture.records().is_empty(),
        "and no authored record exists that nothing can name: {:?}",
        fixture.records()
    );
    assert_eq!(fixture.revision(), 2, "the run did not advance");
}

/// EX-3 — from DEC-086 step 3 onward, recovery resumes against the **exact**
/// reserved canonical id, never a fresh one.
///
/// The expected id is read back out of the journal FILE rather than recomputed
/// here: "a record exists" and "a record exists at the id the crash promised"
/// are different claims, and only the second one is EX-3.
#[test]
fn post_journal_recovery_resumes_the_exact_reserved_id() {
    let fixture = Fixture::start();
    fixture.raise("inq-1", "seed");
    let payload = create_checkpoint(&fixture, "sub-cp", "cp-1", "inq-1");

    // Crash after the id is journalled and before its bytes are written.
    fixture.crash_at(&payload, "record-materialise");

    let journalled = fixture
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
        fixture.records().is_empty(),
        "no authored bytes yet: {:?}",
        fixture.records()
    );

    // The retry resumes the same submission.
    fixture.apply(&payload);

    let records = fixture.records();
    assert_eq!(
        records.len(),
        1,
        "exactly one record — a fresh claim would have made a second: {records:?}"
    );
    let expected = numeric_dir(&journalled);
    assert!(
        records[0].ends_with(format!("{expected}/{RECORD_STEM}-{expected}.toml")),
        "the resumed record sits at the RESERVED id {journalled}, not a fresh one: {:?}",
        records[0]
    );
    assert_eq!(
        fixture.disposition("inq-1"),
        Disposition::Created {
            record: journalled.clone()
        },
        "and the stored disposition names that same id"
    );
    assert_eq!(
        fixture.intent().state(),
        IntentState::Complete,
        "with the journal closed out"
    );
}

/// EX-4 — adopting an existing canonical record creates no duplicate and needs no
/// reservation, and the required legal `shapes` edge is applied when absent —
/// once, however many checkpoints adopt it.
#[test]
fn adopting_an_existing_record_creates_no_duplicate() {
    let fixture = Fixture::start();
    let minted = ok(spawn(
        &fixture.root,
        &["knowledge", "new", "decision", "Prior decision", "-p", "."],
        None,
    ));
    let record = minted
        .split_whitespace()
        .nth(1)
        .expect("`knowledge new` names the canonical id")
        .trim_end_matches(':')
        .to_owned();
    let before = fixture.records();
    assert_eq!(before.len(), 1, "one record exists to adopt");

    fixture.raise("inq-1", "seed-one");
    fixture.raise("inq-2", "seed-two");
    fixture.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"cp-1\",\"disposes\":\"inq-1\",\
         \"dispose\":{{\"form\":\"adopt\",\"record\":\"{record}\"}}}}]}}",
        fixture.envelope("adopt-one")
    ));

    assert_eq!(
        fixture.records(),
        before,
        "an adoption reserves nothing and creates nothing"
    );
    assert_eq!(
        fixture.disposition("inq-1"),
        Disposition::Adopted {
            record: record.clone()
        }
    );
    let toml = std::fs::read_to_string(&before[0]).unwrap();
    assert!(
        toml.contains("label = \"shapes\"") && toml.contains(&format!("target = \"{SLICE}\"")),
        "the required legal edge is applied when absent: {toml}"
    );

    // A second checkpoint adopting the same record adds no second edge — the
    // "when absent" half is the append seam's own idempotency, not a check here.
    fixture.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"cp-2\",\"disposes\":\"inq-2\",\
         \"dispose\":{{\"form\":\"adopt\",\"record\":\"{record}\"}}}}]}}",
        fixture.envelope("adopt-two")
    ));
    let after = std::fs::read_to_string(&before[0]).unwrap();
    assert_eq!(
        after.matches(&format!("target = \"{SLICE}\"")).count(),
        1,
        "exactly one shapes edge after two adoptions: {after}"
    );
    assert_eq!(fixture.records(), before, "and still no duplicate record");
}

/// EX-7 — the two dispositions that create no graph edge are first-class. A
/// resolved node that produced no record is representable, and neither form
/// touches the authored tier.
#[test]
fn retain_unresolved_and_non_durable_are_valid_dispositions() {
    let fixture = Fixture::start();
    fixture.raise("inq-1", "seed-one");
    fixture.raise("inq-2", "seed-two");

    fixture.apply(&format!(
        "{{{},\"declare\":[\
         {{\"subject\":\"cp-1\",\"disposes\":\"inq-1\",\
         \"dispose\":{{\"form\":\"unresolved\",\"note\":\"needs the spike first\"}}}},\
         {{\"subject\":\"cp-2\",\"disposes\":\"inq-2\",\
         \"dispose\":{{\"form\":\"non-durable\",\"note\":\"a naming aside\"}}}}]}}",
        fixture.envelope("notes")
    ));

    assert_eq!(
        fixture.disposition("inq-1"),
        Disposition::RetainedUnresolved {
            note: "needs the spike first".to_owned()
        }
    );
    assert_eq!(
        fixture.disposition("inq-2"),
        Disposition::NonDurable {
            note: "a naming aside".to_owned()
        }
    );
    assert!(
        fixture.records().is_empty(),
        "neither form writes an authored record: {:?}",
        fixture.records()
    );
}

/// EX-7 — resolving a node without any of the four dispositions is refused, and
/// the refusal names all four. "We discussed it" is not a disposition: the
/// intentionally non-durable case is declared, never defaulted into.
// ── PHASE-15 EX-1/EX-2/EX-3 · VT-1 ────────────────────────────────────────
//
// RV-324 F-1: an accepted proposal's declarations join the coordinator's batch
// in the PURE core (`run.rs`) but never reach the SHELL's effect protocol —
// `plan_checkpoints` walks `request.declare` alone. Both tests below drive the
// same intersection, which the ledger established was wholly untested: this
// suite contained zero `accept` before PHASE-15.

/// `EX-1` — a proposed `adopt` is validated exactly as the coordinator's own
/// is: `knowledge::adoptable`'s kind-and-status check runs against the real
/// record, so an adoption naming a record that does not exist is refused.
///
/// Before the fix the accept SUCCEEDED and the node was recorded as adopted
/// against a nonexistent record — the silent inference the whole checkpoint
/// protocol exists to prevent.
#[test]
fn an_accepted_proposals_adopt_is_validated_like_the_coordinators_own() {
    const ABSENT: &str = "DEC-999";

    // POSITIVE CONTROL, taken FIRST so the comparison is against a live check
    // rather than against a remembered one: the coordinator's own adoption of
    // the same absent record is refused today.
    let control = Fixture::start();
    control.raise("inq-1", "seed");
    let refused = control.refuse(&format!(
        "{{{},\"declare\":[{{\"subject\":\"cp-1\",\"disposes\":\"inq-1\",\
         \"dispose\":{{\"form\":\"adopt\",\"record\":\"{ABSENT}\"}}}}]}}",
        control.envelope("adopt-direct")
    ));
    assert!(
        refused.contains(ABSENT),
        "control: a coordinator's own adoption of an absent record is refused, \
         naming it: {refused}"
    );

    let fixture = Fixture::start();
    fixture.raise("inq-1", "seed");
    let before = fixture.records();
    let out = accept_proposing(
        &fixture,
        "inq-1",
        &format!(
            "{{\"subject\":\"cp-1\",\"disposes\":\"inq-1\",\
             \"dispose\":{{\"form\":\"adopt\",\"record\":\"{ABSENT}\"}}}}"
        ),
    );

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "an accepted proposal's adopt reaches the same kind-and-status check \
         as the coordinator's own (EX-1): {}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        stderr.contains(ABSENT),
        "…and the refusal names the record it could not adopt: {stderr}"
    );
    assert_eq!(
        fixture.disposition_of("inq-1"),
        None,
        "the node is NOT recorded as adopted against a record that does not \
         exist — the failure this closes was silent, not loud"
    );
    assert_eq!(
        fixture.records(),
        before,
        "and a refused acceptance authors nothing"
    );
}

/// `EX-1`/`EX-2` — a proposed `create` enters DEC-086's reserve-then-journal
/// protocol instead of being refused as `CheckpointRecordUnresolved`.
///
/// `EX-2` is F-1's unnamed third consequence and is asserted here rather than
/// assumed: the acceptance attestation is bound for a proposed checkpoint too,
/// because the binding rides the very loop `EX-1` widens.
#[test]
fn an_accepted_proposals_create_enters_the_reserve_then_journal_protocol() {
    let fixture = Fixture::start();
    fixture.raise("inq-1", "seed");
    assert!(
        fixture.records().is_empty(),
        "the tree starts with no authored records"
    );

    let out = accept_proposing(
        &fixture,
        "inq-1",
        "{\"subject\":\"cp-1\",\"disposes\":\"inq-1\",\
         \"dispose\":{\"form\":\"create\",\"kind\":\"decision\",\
         \"title\":\"Checkpointed decision\",\
         \"acceptance\":{\"basis\":\"the user said so\",\"turn\":\"t-1\"}}}",
    );
    assert!(
        out.status.success(),
        "an accepted proposal's create is planned and effected, not refused as \
         unresolved (EX-1): {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // DEC-086 steps 2-4: reserved, journalled, authored.
    let intent = fixture.intent();
    assert_eq!(
        intent.state(),
        IntentState::Complete,
        "the checkpoint ran the whole protocol: {intent:?}"
    );
    let records = fixture.records();
    assert_eq!(
        records.len(),
        1,
        "exactly one record was authored: {records:?}"
    );

    // Steps 5-6: the resolved canonical id is bound onto the declaration the
    // snapshot stores — EX-3. `bind_resolved` walked `request.declare`, where
    // an accepted proposal's declaration never appears, so before the fix this
    // could not have been reached even if planning had produced the record.
    let Some(Disposition::Created { record }) = fixture.disposition_of("inq-1") else {
        panic!(
            "the node carries a `created` disposition: {:?}",
            fixture.disposition_of("inq-1")
        );
    };
    assert_ne!(
        record, PROVISIONAL_RECORD,
        "the disposition names the REAL canonical id, not the provisional one \
         the first validation pass validated against"
    );
    assert_eq!(
        intent.reserved_record(),
        Some(record.as_str()),
        "and it is the id the journal reserved — the same record, not a second one"
    );

    // EX-2, verified rather than assumed: the acceptance attestation is bound
    // for a proposed checkpoint too. It is built inside `plan_checkpoints`'
    // loop, so it was skipped with everything else the loop never reached.
    assert!(
        intent.acceptance().is_some(),
        "the acceptance attestation is bound for an accepted proposal's \
         checkpoint (EX-2): {intent:?}"
    );
}

#[test]
fn resolved_node_without_disposition_is_refused_at_the_command() {
    let fixture = Fixture::start();
    fixture.raise("inq-1", "seed");
    let before = std::fs::read(&fixture.snapshot).unwrap();

    let error = fixture.refuse(&format!(
        "{{{},\"declare\":[{{\"subject\":\"cp-1\",\"disposes\":\"inq-1\"}}]}}",
        fixture.envelope("bare")
    ));

    for form in design_run::inquiry::DispositionForm::ALL {
        assert!(
            error.contains(form.as_str()),
            "the refusal names `{}`: {error}",
            form.as_str()
        );
    }
    assert_eq!(
        std::fs::read(&fixture.snapshot).unwrap(),
        before,
        "and the run did not advance"
    );
}
