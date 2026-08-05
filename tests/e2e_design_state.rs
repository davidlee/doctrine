//! SL-233 PHASE-03 — persistence and the authored watermark, over the built
//! binary (design §9.2).
//!
//! Twenty-three black-box behaviours. The two that cannot be black-box — the
//! ones that must inject a closure into the pre-write window — live beside their
//! seam in `src/commands/design.rs` under EX-17, because this crate is
//! **binary-only** (no `[lib]`, no `src/lib.rs`) and an integration test can only
//! spawn the built binary.
//!
//! For the same reason the pure model is `#[path]`-included below rather than
//! imported: it is the CHR-014 idiom this repo already uses for
//! `tests/common/mod.rs`, and it means the bounds these tests assert against are
//! the exact bytes the binary compiles rather than numbers re-typed here.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

mod common;
/// The wire-shaped act payloads a ladder submits (SL-244 `T8`), shared with the
/// other design suites that cross an edge.
mod design_act;
/// Opted into for [`design_fixture::seed_slice_record`] alone — SL-244 `T7`'s
/// governance act projects the slice's own edge set, and one seeder shared with
/// the other design suites beats four copies.
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

use design_run::attestation::{ActKind, AgentAct, ReviewDisposition};
use design_run::bounds::{DESIGN_ID_BYTES, DESIGN_STAGE_LABEL_BYTES};
use design_run::change_log::{ChangeEvent, ChangeRow, PayloadTerm, ValueKind};
use design_run::ids::{DesignId, IdKind};
use design_run::render::{ELISION_MARKER_UNDER_TEST, ENVELOPE_PAYLOAD_BYTES_UNDER_TEST};
use design_run::snapshot::{self, DesignSnapshot};

/// The slice every fixture designs.
const SLICE: &str = "SL-233";
/// Its zero-padded directory name.
const SLICE_NUMBER: &str = "233";

// ── fixture ───────────────────────────────────────────────────────────────
//
// STD-001 inside the fixture (RV-321 F-4). Three protocol spellings used to be
// hand-copied across five test bodies; each now has ONE owner here.
//
// `.doctrine/slice` comes from `crate::kinds::SLICE_DIR` through `common`, so
// there is no copy at all. The authored document's name and the stable-section
// marker grammar are private to `src/commands/design.rs` in a **binary-only**
// crate, so an integration test cannot name them: one copy is the floor the
// layout imposes, and these two helpers are that one copy. Claiming zero would
// be the over-claim RV-321 F-3 is about.

/// The authored design document's file name.
const DESIGN_DOC: &str = "design.md";

/// The stable-section marker `materialise` writes and re-adoption addresses —
/// the single place this suite spells that grammar.
fn section_marker(section: &str) -> String {
    format!("<!-- doctrine:section {section} -->")
}

/// The uniform block affix EX-3 mandates, and the ONE place this suite spells
/// it: for each section, the marker line, a newline, the body **verbatim**,
/// then ONE newline — blocks concatenated with **no separator**.
///
/// Uniform on purpose. The affix an interior block carries is the affix the last
/// block carries, so the end-of-document case disappears instead of needing a
/// special case, and remove-exactly-one inverts append-exactly-one at every
/// position.
fn framed(sections: &[(&str, &str)]) -> String {
    sections
        .iter()
        .map(|(id, body)| format!("{}\n{body}\n", section_marker(id)))
        .collect()
}

/// A hand-authored document holding one marker-addressed section, exactly as a
/// human would leave it after editing outside the run — the one-section instance
/// of [`framed`], never a second spelling of the framing.
fn authored_document(section: &str, body: &str) -> String {
    framed(&[(section, body)])
}

/// A started design run in a throwaway tree.
struct Fixture {
    _tmp: tempfile::TempDir,
    root: PathBuf,
    /// Learned from `design start`'s own output, so no test re-types the state
    /// path (STD-001 / EX-8: the path has exactly one source, `design_snapshot_path`).
    snapshot: PathBuf,
    uid: String,
}

impl Fixture {
    /// A run with no authored document — the cold start.
    fn start() -> Fixture {
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
        Fixture {
            _tmp: tmp,
            root,
            snapshot,
            uid,
        }
    }

    /// The authored design document.
    fn doc(&self) -> PathBuf {
        self.root
            .join(common::SLICE_DIR)
            .join(SLICE_NUMBER)
            .join(DESIGN_DOC)
    }

    /// The parsed snapshot.
    fn read(&self) -> DesignSnapshot {
        snapshot::parse(&std::fs::read_to_string(&self.snapshot).unwrap()).unwrap()
    }

    /// The snapshot's exact bytes.
    fn bytes(&self) -> Vec<u8> {
        std::fs::read(&self.snapshot).unwrap()
    }

    /// The current revision.
    fn revision(&self) -> u64 {
        self.read().run.revision
    }

    /// A payload envelope asserting the current revision (EX-9).
    fn envelope(&self, submission: &str) -> String {
        self.envelope_at(self.revision(), submission)
    }

    /// A payload envelope asserting `revision`.
    fn envelope_at(&self, revision: u64, submission: &str) -> String {
        format!(
            "\"run_uid\":\"{}\",\"known_revision\":{revision},\"submission_id\":\"{submission}\"",
            self.uid
        )
    }

    /// A payload carrying the current revision and `submission`, plus `body`'s
    /// top-level keys merged in — the JSON-built spelling, for payloads whose
    /// prose carries newlines, tabs or quotes that a hand-escaped literal
    /// cannot state honestly.
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

    /// One `apply` carrying only the envelope: a lawful revision that produces
    /// no material rows, which is exactly the case an inferred change-log floor
    /// gets wrong.
    fn empty_apply(&self, submission: &str) {
        self.apply(&format!("{{{}}}", self.envelope(submission)));
    }

    /// `design show`, budgeted.
    fn show(&self, extra: &[&str]) -> String {
        let mut args = vec!["design", "show", SLICE, "-p", "."];
        args.extend_from_slice(extra);
        run(&self.root, &args)
    }
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

/// Run the built binary with one extra env var set, expecting failure; returns
/// stderr. The var is spelled literally, as `DOCTRINE_DESIGN_FAULT` is in
/// `e2e_design_checkpoint`: an e2e drives the binary as a black box, so the env
/// surface is part of the contract under test rather than a shared constant.
fn fail_with_env(root: &Path, args: &[&str], key: &str, value: &OsStr) -> String {
    let out = common::doctrine_cmd(root)
        .args(args)
        .env(key, value)
        .output()
        .expect("spawn doctrine");
    assert!(
        !out.status.success(),
        "doctrine {args:?} unexpectedly succeeded: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    String::from_utf8_lossy(&out.stderr).to_string()
}

/// A run holding one section, materialised, with the watermark baselined.
///
/// The argument is the section's **heading**, because since EX-13(b) that is
/// what a body is: the title is derived from the body's first non-blank line,
/// so there is no way to declare a section without declaring its heading.
fn materialised(title: &str) -> Fixture {
    let fixture = Fixture::start();
    fixture.apply(&fixture.payload(
        "seed",
        &json!({ "declare": [{ "subject": "sec-1", "body": format!("## {title}\n") }] }),
    ));
    run(&fixture.root, &["design", "materialise", SLICE, "-p", "."]);
    fixture
}

// ── DEC-092 rule 1/2/3, the five that stay black-box (EX-17(b)) ───────────

/// §9.2 — an authored hand-edit landing between two applies is refused by the
/// entry watermark check on the next ordinary mutating verb, not discovered
/// later at materialise.
#[test]
fn hand_edit_between_applies_is_refused_at_entry() {
    let fixture = materialised("first draft");
    std::fs::write(fixture.doc(), b"a human rewrote this\n").unwrap();

    let before = fixture.bytes();
    let error = fixture.refuse(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-1\",\"question\":\"q\"}}]}}",
        fixture.envelope("after-edit")
    ));
    assert!(error.contains("edited outside this run"), "{error}");
    assert_eq!(fixture.bytes(), before, "the run did not advance");
}

/// §9.2 — an edit landing after the final comparison and before the atomic
/// rename is detected by the entry check on the NEXT mutating verb rather than
/// silently accepted. The guarantee is delayed detection, never silent
/// acceptance: the first invocation completes, and the second refuses.
#[test]
fn edit_after_final_comparison_is_caught_by_next_entry_check() {
    let fixture = materialised("first draft");
    let baseline = fixture.revision();

    // An apply that completes: its pre-write comparison passed and the rename
    // landed.
    fixture.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-1\",\"question\":\"q\"}}]}}",
        fixture.envelope("landed")
    ));
    assert_eq!(fixture.revision(), baseline + 1, "the first apply landed");

    // The edit lands after that comparison — outside any window this
    // invocation could have closed.
    std::fs::write(fixture.doc(), b"landed after the comparison\n").unwrap();

    let error = fixture.refuse(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-2\",\"question\":\"q\"}}]}}",
        fixture.envelope("next")
    ));
    assert!(error.contains("edited outside this run"), "{error}");
    assert_eq!(fixture.revision(), baseline + 1, "and nothing advanced");
}

/// §9.2 / `VT-2` — `materialise` re-baselines only to bytes demonstrably on disk.
///
/// Two windows, and this asserts only the one that is closed. A write landing
/// between the pre-write re-check and the atomic rename is **destroyed** by the
/// rename, so nothing survives for any in-process check to observe; that
/// lost-update window is consciously tolerated for v0.0.1 and is deliberately not
/// asserted here. A write landing *after* the rename is observable, and before
/// this fix it was certified: the watermark was derived from the bytes Doctrine
/// **rendered** rather than the bytes `design.md` **holds**, so the next entry
/// check saw an aligned document and the divergence became permanently invisible.
/// That is `RV-324` F-2's self-certification half.
///
/// Its sibling above is the apply-path twin — same class of window, but `apply`
/// never re-derives the watermark from its own output, so delayed detection
/// already worked there.
#[test]
fn a_foreign_write_after_materialises_rename_is_not_certified() {
    let fixture = materialised("first draft");
    let baseline = fixture.revision();

    let foreign_bytes = b"## Foreign\n\nwritten by another hand\n";
    let foreign = fixture.root.join("foreign.md");
    std::fs::write(&foreign, foreign_bytes).unwrap();

    let error = fail_with_env(
        &fixture.root,
        &["design", "materialise", SLICE, "-p", "."],
        "DOCTRINE_DESIGN_EDIT",
        foreign.as_os_str(),
    );
    assert!(
        error.contains("changed again"),
        "the refusal must say the document moved under it, got: {error}"
    );
    assert_eq!(
        fixture.revision(),
        baseline,
        "the snapshot must not advance on bytes Doctrine did not write"
    );
    assert_eq!(
        std::fs::read(fixture.doc()).unwrap(),
        foreign_bytes,
        "and the other writer's bytes stand — this path overwrites nothing further"
    );

    // The half that was permanently invisible before: because the watermark never
    // moved to Doctrine's render, the ordinary entry check now sees the divergence.
    // Self-certification had defeated DEC-092's next-entry promise outright.
    let next = fixture.refuse(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-9\",\"question\":\"q\"}}]}}",
        fixture.envelope("next")
    ));
    assert!(next.contains("edited outside this run"), "{next}");
}

/// §9.2 — a valid `adopt_authored` crosses divergence where an ordinary `apply`
/// is refused, and it alone re-baselines the watermark.
///
/// The hand-written body opens with its own heading because since PHASE-14 EX-6
/// the title derivation runs at ADOPTION too: a headless body is now refused at
/// this door exactly as it always was at declare, so a fixture without one would
/// be asserting the watermark crossing against prose the door no longer admits.
#[test]
fn adopt_authored_crosses_divergence_and_rebaselines_alone() {
    let fixture = materialised("first draft");
    let hand_written = "## First draft\n\nhand written prose";
    let foreign = authored_document("sec-1", hand_written);
    std::fs::write(fixture.doc(), &foreign).unwrap();
    let doc_digest = common::sha256(foreign.as_bytes());
    let section_digest = common::sha256(hand_written.as_bytes());

    // Ordinary mutation is refused at exactly this divergence.
    let error = fixture.refuse(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-1\",\"question\":\"q\"}}]}}",
        fixture.envelope("ordinary")
    ));
    assert!(error.contains("edited outside this run"), "{error}");

    // The re-adoption crosses it — and alone re-baselines.
    fixture.apply(&format!(
        "{{{},\"adopt_authored\":{{\"fingerprint\":\"{doc_digest}\",\
         \"sections\":{{\"sec-1\":\"{section_digest}\"}}}}}}",
        fixture.envelope("adopt")
    ));
    let after = fixture.read();
    assert_eq!(
        after.authored.watermark.as_ref().map(|f| f.as_str()),
        Some(doc_digest.as_str()),
        "the watermark re-baselined onto the adopted bytes"
    );
    assert_eq!(
        after
            .sections
            .find(&id("sec-1"))
            .unwrap()
            .fingerprint
            .as_str(),
        section_digest,
        "and the section's fingerprint moved with it"
    );
}

/// §9.2 / VA-5 — an `adopt_authored` with a stale declared fingerprint, or one
/// failing marker validation, changes NEITHER runtime clearance NOR the
/// watermark. Asserted by comparing to the pre-call values, not merely by the
/// call erroring: F-20 was raised precisely because a guard can be present and
/// still self-refuse.
#[test]
fn stale_or_invalid_adoption_changes_neither_clearance_nor_watermark() {
    let fixture = materialised("first draft");
    let foreign = authored_document("sec-1", "hand written prose");
    std::fs::write(fixture.doc(), &foreign).unwrap();
    let doc_digest = common::sha256(foreign.as_bytes());
    let section_digest = common::sha256(b"hand written prose");

    let before = fixture.read();
    let watermark_before = before.authored.watermark.clone();

    // (a) stale declared fingerprint.
    let stale = fixture.refuse(&format!(
        "{{{},\"adopt_authored\":{{\"fingerprint\":\"0000\",\
         \"sections\":{{\"sec-1\":\"{section_digest}\"}}}}}}",
        fixture.envelope("stale")
    ));
    assert!(stale.contains("declares fingerprint"), "{stale}");

    // (b) a marker map that is not complete and exact.
    let invalid = fixture.refuse(&format!(
        "{{{},\"adopt_authored\":{{\"fingerprint\":\"{doc_digest}\",\
         \"sections\":{{\"sec-9\":\"{section_digest}\"}}}}}}",
        fixture.envelope("invalid")
    ));
    assert!(invalid.contains("marker map"), "{invalid}");

    let after = fixture.read();
    assert_eq!(
        after.authored.watermark, watermark_before,
        "the watermark is UNCHANGED against its pre-call value"
    );
    // `T11` retired the gate-clearance witness with `Evidence` (`EX-11`), so the
    // claim is made on the whole snapshot instead — which is strictly stronger
    // than the one field it replaces: a refusal must leave `prior` untouched
    // entire, not merely leave one group untouched.
    assert_eq!(after, before, "and the refusal moved nothing at all");
}

/// §9.2 — an absent `design.md` before first materialisation is COLD rather than
/// divergent; absent after it is divergent. Without the `materialised` flag
/// those two are the same observation.
#[test]
fn absent_design_md_is_cold_before_first_materialise_divergent_after() {
    let fixture = Fixture::start();
    assert!(!fixture.doc().exists());

    // Cold: an ordinary apply proceeds against an absent document.
    fixture.apply(&fixture.payload(
        "cold",
        &json!({ "declare": [{ "subject": "sec-1", "body": "## Draft\n" }] }),
    ));
    run(&fixture.root, &["design", "materialise", SLICE, "-p", "."]);
    assert!(fixture.doc().exists());

    // Divergent: the same absence, after Doctrine has left bytes there.
    std::fs::remove_file(fixture.doc()).unwrap();
    let error = fixture.refuse(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-1\",\"question\":\"q\"}}]}}",
        fixture.envelope("after")
    ));
    assert!(error.contains("edited outside this run"), "{error}");
    assert!(
        error.contains("absent"),
        "the refusal says what it read: {error}"
    );
}

// ── persistence (EX-1..EX-4, EX-9, EX-10) ─────────────────────────────────

/// EX-1 — an unknown schema version is refused at parse with a message that
/// names the supported set and a remedy.
#[test]
fn unknown_schema_version_is_refused_with_a_useful_message() {
    let fixture = Fixture::start();
    let text = std::fs::read_to_string(&fixture.snapshot).unwrap();
    std::fs::write(
        &fixture.snapshot,
        text.replace("version = 1", "version = 99"),
    )
    .unwrap();

    let error = fail(&fixture.root, &["design", "show", SLICE, "-p", "."]);
    assert!(
        error.contains("unsupported design-run snapshot version 99"),
        "{error}"
    );
    assert!(
        error.contains("expected one of: 1"),
        "names the supported set: {error}"
    );
    assert!(error.contains("design start"), "names a remedy: {error}");
}

/// EX-1 — the wire model round-trips deterministically: parse then serialise
/// reproduces the stored bytes exactly.
#[test]
fn snapshot_toml_round_trips_deterministically() {
    let fixture = materialised("first draft");
    fixture.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-1\",\"question\":\"why\"}}]}}",
        fixture.envelope("one")
    ));
    let stored = std::fs::read_to_string(&fixture.snapshot).unwrap();
    let parsed = snapshot::parse(&stored).unwrap();
    let reserialised = snapshot::to_toml(&parsed).unwrap();
    assert_eq!(reserialised, stored, "byte-identical round trip");
    assert_eq!(
        snapshot::to_toml(&snapshot::parse(&reserialised).unwrap()).unwrap(),
        stored,
        "and stable under a second pass"
    );
}

/// EX-2 — a stale `known_revision` is refused with a conflict report naming both
/// revisions, and the run does not advance.
#[test]
fn stale_known_revision_is_refused_with_a_conflict_report() {
    let fixture = Fixture::start();
    fixture.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-1\",\"question\":\"q\"}}]}}",
        fixture.envelope("one")
    ));
    fixture.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-2\",\"question\":\"q\"}}]}}",
        fixture.envelope("two")
    ));
    let before = fixture.bytes();

    let error = fixture.refuse(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-3\",\"question\":\"q\"}}]}}",
        fixture.envelope_at(2, "stale")
    ));
    assert!(error.contains("conflict"), "{error}");
    assert!(
        error.contains("known_revision 2"),
        "names what was asserted: {error}"
    );
    assert!(error.contains("revision 3"), "and what is current: {error}");
    assert_eq!(fixture.bytes(), before, "the run did not advance");
}

/// EX-2 — a candidate that fails validation leaves the snapshot byte-identical.
/// The whole candidate is validated before any mutation (DEC-063), so a refusal
/// cannot leave half a batch behind.
#[test]
fn validation_failure_leaves_the_snapshot_byte_identical() {
    let fixture = Fixture::start();
    let before = fixture.bytes();

    // The second declaration names an unknown parent — the first is lawful, so a
    // half-applied batch would be visible.
    let error = fixture.refuse(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-1\",\"question\":\"q\"}},\
         {{\"subject\":\"inq-2\",\"question\":\"q\",\"parent\":\"inq-404\"}}]}}",
        fixture.envelope("invalid")
    ));
    assert!(error.contains("unknown node"), "{error}");
    assert_eq!(fixture.bytes(), before, "byte-identical after a refusal");
}

/// EX-3 — submission idempotency: an identical retry RESUMES without advancing
/// the run, and a changed payload under a reused id is refused.
#[test]
fn reused_submission_id_with_changed_payload_is_refused() {
    let fixture = Fixture::start();
    let first = format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-1\",\"question\":\"q\"}}]}}",
        fixture.envelope("sub-1")
    );
    fixture.apply(&first);
    let landed = fixture.bytes();

    // The retry resumes: same id, same bytes, no advance.
    let resumed = fixture.apply(&first);
    assert!(resumed.contains("resumed submission sub-1"), "{resumed}");
    assert_eq!(fixture.bytes(), landed, "a retry does not advance the run");

    // Different bytes under the same id is a different submission wearing a
    // used name.
    let error = fixture.refuse(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-2\",\"question\":\"q\"}}]}}",
        fixture.envelope_at(1, "sub-1")
    ));
    assert!(
        error.contains("already applied with different bytes"),
        "{error}"
    );
    assert_eq!(fixture.bytes(), landed, "and still no advance");
}

/// EX-4 — receipt eviction is bounded, but can never remove the latest receipt
/// or one an outstanding delegation still references.
#[test]
fn eviction_preserves_latest_and_outstanding_delegation_receipts() {
    let fixture = Fixture::start();
    fixture.empty_apply("pinned");

    // Pin the early receipt the way an outstanding delegation does. The
    // delegation *verb* is a later phase; the stored shape it produces is this
    // one, and eviction is what is under test.
    let text = std::fs::read_to_string(&fixture.snapshot).unwrap();
    std::fs::write(
        &fixture.snapshot,
        text.replace(
            "submission = \"pinned\"",
            "submission = \"pinned\"\ndelegation = \"dlg-1\"\n\
             delegation_state = \"outstanding\"",
        ),
    )
    .unwrap();

    for step in 0..40 {
        fixture.empty_apply(&format!("s{step}"));
    }

    let receipts = fixture.read().receipts;
    let held: BTreeSet<&str> = receipts
        .receipts
        .iter()
        .map(|receipt| receipt.submission.as_str())
        .collect();
    assert!(
        held.contains("pinned"),
        "an outstanding delegation pins its receipt"
    );
    assert!(held.contains("s39"), "the latest receipt always survives");
    assert!(
        !held.contains("s0"),
        "unpinned receipts below the window are evicted"
    );
    assert!(
        receipts.receipts.len() < 41,
        "history is bounded: {} receipts",
        receipts.receipts.len()
    );
}

// ── the change log (EX-13) ────────────────────────────────────────────────

/// EX-13 — `change_log_floor` is recorded EXPLICITLY, never inferred from the
/// oldest surviving row. Inference breaks exactly here: the intervening
/// revisions produced no material rows, so the oldest surviving row says
/// nothing about how far back the log's coverage reaches.
#[test]
fn change_log_floor_is_recorded_not_inferred() {
    let fixture = Fixture::start();
    // An early material revision, then a long QUIET span, then a late material
    // one. The early rows fall out of the window; the late ones survive well
    // above it — so the oldest surviving row says nothing about where coverage
    // begins, which is exactly where inference breaks.
    fixture.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-early\",\"question\":\"q\"}}]}}",
        fixture.envelope("material")
    ));
    for step in 0..40 {
        fixture.empty_apply(&format!("quiet{step}"));
    }
    fixture.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-late\",\"question\":\"q\"}}]}}",
        fixture.envelope("late")
    ));

    let log = fixture.read().change_log;
    let oldest_row = log
        .rows
        .iter()
        .map(|row| row.revision)
        .min()
        .expect("the late revision's rows survive");
    assert!(
        log.floor > 1,
        "the floor advanced with the window: {}",
        log.floor
    );
    assert!(
        log.floor < oldest_row,
        "the RECORDED floor ({}) reaches further back than the oldest surviving row \
         ({oldest_row}) — an inferred floor could only ever say {oldest_row}",
        log.floor
    );

    // A caller inside the recorded coverage gets a COMPLETE answer. Under an
    // inferred floor the same call would report UNAVAILABLE — reporting "I
    // cannot tell you" about a range the log does in fact cover.
    let complete = fixture.show(&["--known-revision", &log.floor.to_string()]);
    assert!(!complete.contains("UNAVAILABLE"), "{complete}");
    assert!(
        complete.contains("inq-late"),
        "and it names what changed: {complete}"
    );
}

/// EX-13 — a `known_revision` below the floor renders as UNAVAILABLE with the
/// floor named, never as empty. "Nothing changed" and "I cannot tell you what
/// changed" are opposite facts (design R2).
#[test]
fn known_revision_below_floor_is_unavailable_not_empty() {
    let fixture = Fixture::start();
    for step in 0..40 {
        fixture.empty_apply(&format!("q{step}"));
    }
    let floor = fixture.read().change_log.floor;
    assert!(floor > 1);

    let unavailable = fixture.show(&["--known-revision", "1"]);
    assert!(unavailable.contains("UNAVAILABLE"), "{unavailable}");
    assert!(
        unavailable.contains(&format!("from {floor} onward")),
        "names the floor: {unavailable}"
    );

    let empty = fixture.show(&["--known-revision", &floor.to_string()]);
    assert!(empty.contains("changes: none since"), "{empty}");
    assert!(
        !empty.contains("UNAVAILABLE"),
        "an empty delta must never render like an unavailable one: {empty}"
    );
}

/// EX-13 — the within-revision `index` is assigned in validated-candidate
/// serialisation order, NEVER submission order. One apply is an unordered batch
/// (DEC-063), so the same declaration set submitted either way must produce the
/// same indices.
#[test]
fn within_revision_index_is_candidate_order_not_submission_order() {
    let forward = Fixture::start();
    forward.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-a\",\"question\":\"q\"}},\
         {{\"subject\":\"inq-b\",\"question\":\"q\"}},\
         {{\"subject\":\"inq-c\",\"question\":\"q\"}}]}}",
        forward.envelope("one")
    ));
    let reversed = Fixture::start();
    reversed.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-c\",\"question\":\"q\"}},\
         {{\"subject\":\"inq-b\",\"question\":\"q\"}},\
         {{\"subject\":\"inq-a\",\"question\":\"q\"}}]}}",
        reversed.envelope("one")
    ));

    let indices = |fixture: &Fixture| -> Vec<(u32, String)> {
        fixture
            .read()
            .change_log
            .rows
            .iter()
            .map(|row| {
                (
                    row.index,
                    row.subject
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_default(),
                )
            })
            .collect()
    };
    assert_eq!(
        indices(&forward),
        indices(&reversed),
        "the index is a function of the declaration SET, not of how it was submitted"
    );
    assert_eq!(
        indices(&forward),
        vec![
            (0, "inq-a".to_owned()),
            (1, "inq-b".to_owned()),
            (2, "inq-c".to_owned())
        ]
    );
}

// ── the five behavioural gaps (EX-14) ─────────────────────────────────────

/// The two section bodies [`every_event_fixture`] declares, in order. Named
/// because a test downstream asserts the digest of the second one, and a
/// re-typed literal there is a digest of bytes nothing declared.
const FIRST_SECTION_BODY: &str = "## Draft one\n";
/// The redraft that moves the fingerprint.
const REVISED_SECTION_BODY: &str = "## Draft two\n";
/// A third draft, declared **after** the run's acceptance, so the
/// `EverySection`-covered `design-accepted` act stops binding and the run owes
/// an `act_invalidated` row (SL-244 `T11`, `EX-11`).
const REACCEPTED_SECTION_BODY: &str = "## Draft three\n";

/// Why [`every_event_fixture`] waives its review pass. Named because the test
/// that reads it asserts the stored row carries this reason **whole**, and a
/// re-typed literal there would be asserting against bytes nothing declared.
const WAIVER_REASON: &str = "the adversarial pass is not worth its cost on a run this size, and the sections have been \
     read by the person accepting them";

/// Drive the run through every material event kind, and return the fixture.
fn every_event_fixture() -> Fixture {
    let fixture = Fixture::start();
    let section = |body: &str, submission: &str| {
        fixture.payload(
            submission,
            &json!({ "declare": [{ "subject": "sec-1", "body": body }] }),
        )
    };
    // SL-244 `T7`/`T8`: the `governance-confirmed` act below projects the
    // slice's own outbound edge set, so the tree needs an authored record to
    // project from — a slice directory with nothing in it is an UNOBSERVABLE
    // fact, which reads as changed and refuses the act. Seeded here rather than
    // in `Fixture::start` because this is the only ladder in the suite that
    // records the act, and the other 140-odd tests have no business acquiring a
    // slice record they never read.
    design_fixture::seed_slice_record(&fixture.root, SLICE_NUMBER);
    // step_discharged — first, while the run still stands at `exploring`, which
    // is the only stage whose outbound edge carries a runbook. The whole
    // sequence, not just the first step: the same edge's guard (`EX-8`) blocks
    // the stage move below until every required step is discharged.
    for step in runbook_fixture::EXPLORING_STEPS {
        fixture.apply(&fixture.payload(step, &runbook_fixture::discharge_body(step)));
    }
    // section_created
    fixture.apply(&section(FIRST_SECTION_BODY, "sec"));
    // node_created ×3
    fixture.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-1\",\"question\":\"q\"}},\
         {{\"subject\":\"inq-2\",\"question\":\"q\"}},\
         {{\"subject\":\"inq-3\",\"question\":\"q\"}}]}}",
        fixture.envelope("nodes")
    ));
    // node_reparented + needs_added
    fixture.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-2\",\"parent\":\"inq-1\"}},\
         {{\"subject\":\"inq-3\",\"needs\":[\"inq-1\"]}}]}}",
        fixture.envelope("edges")
    ));
    // needs_removed
    fixture.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-3\",\"needs\":[]}}]}}",
        fixture.envelope("unneed")
    ));
    // review_attested
    fixture.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"att-1\",\"attests\":\"sec-1\"}}]}}",
        fixture.envelope("attest")
    ));
    // SL-244 `T10` — the crossing below owes `governing-context-recorded` and
    // `initial-concerns-recorded`, and the evaluator derives both from the acts
    // rather than reading the claims above. Two submissions because
    // `ApplyRequest` holds one checkpoint act at a time and this crossing owes
    // two; the pair that does fit in one is the declaration and the act that
    // confirms it. An EMPTY blocking set, which is the truth here — this fixture
    // interrogates none of the three questions it declares — and it keeps the
    // next crossing's condition out of a fixture whose subject is change rows.
    fixture.apply(&fixture.payload(
        "governance",
        &json!({"checkpoint_act": design_act::checkpoint_act(
            ActKind::GovernanceConfirmed,
            "the governing artefacts are the ones found",
        )}),
    ));
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
    // stage_moved (forward)
    fixture.apply(&format!(
        "{{{},\"stage\":{{\"to\":\"inquiring\"}}}}",
        fixture.envelope("advance")
    ));
    // section_fingerprint_changed + evidence_invalidated + review_invalidated
    fixture.apply(&section(REVISED_SECTION_BODY, "revise"));
    // node_lifecycle
    fixture.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-2\",\"lifecycle\":\"deferred\"}}]}}",
        fixture.envelope("defer")
    ));
    // checkpoint_disposed. EX-12: `dispose` is the only spelling, so the record
    // this names is one Doctrine mints through DEC-086 rather than one the
    // payload asserts already exists.
    fixture.apply(&fixture.payload(
        "dispose",
        &json!({ "declare": [{
            "subject": "cp-1",
            "disposes": "inq-1",
            "dispose": { "form": "create", "kind": "decision", "title": "Checkpointed decision" },
        }] }),
    ));
    // The PHASE-12 review vocabulary: finding_raised, finding_disposed,
    // acceptance_attested. No stage move — this fixture is about the rows each
    // declaration persists, not about the lock gate, which
    // `tests/e2e_design_review.rs` owns. `integrated_review_recorded` left this
    // vocabulary with SL-244's retirement of the `int-` declaration: the run's
    // review pass is minted on entry to `reviewing`, not declared, and the
    // stage move it rides is already covered above.
    fixture.apply(&fixture.payload(
        "review",
        &json!({ "declare": [
            { "subject": "fnd-1", "concerns": "sec-1", "summary": "a concern" },
        ] }),
    ));
    fixture.apply(&fixture.payload(
        "dispose-finding",
        &json!({
            "declare": [{ "subject": "fnd-1", "resolution": "accepted" }],
            "acceptance": { "basis": "the user accepted it" },
        }),
    ));
    // The SL-244 review-policy act: `review_policy_changed`. It rides an
    // acceptance because changing which reviewer lanes a run requires is a user
    // judgement, and it names a policy the run does NOT hold — re-declaring the
    // one in force changes nothing and emits no row, so a fixture that declared
    // the default would leave this member of the vocabulary unexercised while
    // looking as though it covered it.
    fixture.apply(&fixture.payload(
        "review-policy",
        &json!({ "review_policy": {
            "policy": "adversarial-only",
            "acceptance": { "basis": "the adversarial reviewer reads for us here" },
        } }),
    ));
    // The PHASE-10 delegation vocabulary: obligation_delegated, proposal_recorded,
    // proposal_accepted, proposal_refused. Two assignments, because accepted and
    // refused are alternative dispositions of one proposal and no assignment can
    // be both — and against two different obligations, because one obligation
    // holds one outstanding assignment at a time.
    for (delegation, obligation) in [("dlg-1", "inq-2"), ("dlg-2", "inq-3")] {
        fixture.apply(&fixture.payload(
            &format!("export-{delegation}"),
            &json!({ "delegation": {
                "act": "export",
                "id": delegation,
                "obligation": obligation,
            } }),
        ));
        fixture.apply(&fixture.payload(
            &format!("propose-{delegation}"),
            &json!({ "delegation": {
                "act": "propose",
                "id": delegation,
                "by": "a delegate",
                "summary": "what I found",
            } }),
        ));
    }
    // act_invalidated. The acceptance above recorded a `design-accepted` act
    // covering every section, so moving one retires it — the row `T11`
    // re-sourced from the evidence feed, exercised by the death of a real act
    // rather than by a claim expiring. It has to come after the acceptance:
    // before it there is no `EverySection`-covered act to invalidate, which is
    // why the earlier `revise` alone does not produce this row.
    fixture.apply(&section(REACCEPTED_SECTION_BODY, "redraft"));
    fixture.apply(&fixture.payload(
        "accept-dlg-1",
        &json!({ "delegation": { "act": "accept", "id": "dlg-1" } }),
    ));
    fixture.apply(&fixture.payload(
        "refuse-dlg-2",
        &json!({ "delegation": {
            "act": "refuse",
            "id": "dlg-2",
            "reason": "the summary does not answer the question",
        } }),
    ));
    // review_disposed (SL-244 `T12`, `EX-13`). A disposition binds to the run's
    // current review pass, and a pass exists only from `reviewing` onwards — so
    // this member of the vocabulary cannot be reached without the two crossings
    // below, and the fixture's earlier "no stage move" scope note is narrowed
    // rather than abandoned: what stays another suite's is the LOCK gate, which
    // this ladder still never attempts.
    //
    // The `Waived` arm, deliberately: it is the arm whose reason `EX-13` says
    // must be legible in the log, and it is answered entirely at admission, so
    // this fixture acquires no `RV` ledger to observe.
    for step in runbook_fixture::INQUIRING_STEPS {
        fixture.apply(&fixture.payload(
            &runbook_fixture::discharge_label(step),
            &runbook_fixture::discharge_body(step),
        ));
    }
    fixture.apply(&fixture.payload(
        "sufficiency",
        &json!({"checkpoint_act": design_act::checkpoint_act(
            ActKind::SufficiencyAccepted,
            "there is nothing outstanding to interrogate",
        )}),
    ));
    fixture.apply(&format!(
        "{{{},\"stage\":{{\"to\":\"drafting\"}}}}",
        fixture.envelope("to-drafting")
    ));
    for step in runbook_fixture::DRAFTING_STEPS {
        fixture.apply(&fixture.payload(
            &runbook_fixture::discharge_label(step),
            &runbook_fixture::discharge_body(step),
        ));
    }
    // `drafting → reviewing` owes `materialisation-current`, which the evaluator
    // derives from the authored watermark — so the ladder materialises the
    // sections it has drafted rather than claiming the condition.
    run(&fixture.root, &["design", "materialise", SLICE, "-p", "."]);
    fixture.apply(&fixture.payload(
        "ready",
        &json!({"agent_declaration": design_act::agent_declaration(
            AgentAct::DraftingReady,
            "the section is drafted and ready to review",
        )}),
    ));
    fixture.apply(&format!(
        "{{{},\"stage\":{{\"to\":\"reviewing\"}}}}",
        fixture.envelope("to-reviewing")
    ));
    fixture.apply(&fixture.payload(
        "waive",
        &json!({"checkpoint_act": design_act::review_disposed(
            "the user waives the integrated pass",
            ReviewDisposition::Waived { reason: WAIVER_REASON.to_owned() },
        )}),
    ));
    fixture
}

/// The canonical id [`every_event_fixture`]'s `create` disposition mints — the
/// first decision in a tree that holds no other. Guarded, not assumed: the one
/// test that names it asserts that the literal is present in the snapshot
/// before doctoring it.
const MINTED_RECORD: &str = "DEC-001";

/// EX-14 — every member of the closed vocabulary persists a change row. The
/// vocabulary is enumerated programmatically, so a new event kind nobody wired
/// fails here rather than being quietly absent.
#[test]
fn every_material_event_kind_persists_a_change_row() {
    let fixture = every_event_fixture();
    let seen: BTreeSet<ChangeEvent> = fixture
        .read()
        .change_log
        .rows
        .iter()
        .map(|row| row.event)
        .collect();
    let missing: Vec<&str> = ChangeEvent::ALL
        .into_iter()
        .filter(|event| !seen.contains(event))
        .map(ChangeEvent::as_str)
        .collect();
    assert!(
        missing.is_empty(),
        "no change row persisted for: {missing:?}"
    );
}

/// SL-244 `EX-13` — the disposition row is legible on its own. It is subject to
/// the act it reports, names the arm taken, and on the waiving arm carries the
/// reason the pass was declined.
///
/// All three, because each alone is the fact the snapshot already held: a row
/// that only said *a disposition happened* would leave the user's choice
/// readable exclusively from state, which is the visibility half `sec-4` makes
/// both arms defensible by.
#[test]
fn a_waived_disposition_row_names_its_arm_and_carries_its_reason() {
    let log = every_event_fixture().read().change_log;
    let row = log
        .rows
        .iter()
        .find(|row| row.event == ChangeEvent::ReviewDisposed)
        .expect("waiving the pass persists a disposition row");

    assert_eq!(
        row.subject.as_ref().map(DesignId::to_string),
        Some(format!(
            "{}{}",
            IdKind::CheckpointAct.prefix(),
            ActKind::ReviewDisposed.as_str()
        )),
        "the row names the act record it reports"
    );
    let terms: Vec<(&str, &str)> = row
        .terms
        .iter()
        .map(|term| (term.key().as_str(), term.value()))
        .collect();
    assert_eq!(
        terms,
        vec![("disposition", "waived"), ("reason", WAIVER_REASON)],
        "the arm, then the reason — in the event's declared term order"
    );
}

/// EX-13 — retention is bounded by `CHANGE_LOG_REVISIONS`: the oldest revisions
/// are evicted and the floor advances with them.
#[test]
fn retention_evicts_oldest_revisions_and_advances_the_floor() {
    let fixture = Fixture::start();
    for step in 0..40 {
        fixture.apply(&format!(
            "{{{},\"declare\":[{{\"subject\":\"inq-{step}\",\"question\":\"q\"}}]}}",
            fixture.envelope(&format!("s{step}"))
        ));
    }
    let log = fixture.read().change_log;
    let oldest = log.rows.iter().map(|row| row.revision).min().unwrap();
    assert!(
        log.floor > 1 && oldest >= log.floor,
        "floor {} advanced and no row survives below it (oldest {oldest})",
        log.floor
    );
    let span = log
        .rows
        .iter()
        .map(|row| row.revision)
        .collect::<BTreeSet<_>>();
    assert!(
        u64::try_from(span.len()).unwrap() <= design_run::bounds::CHANGE_LOG_REVISIONS,
        "at most CHANGE_LOG_REVISIONS revisions are retained: {}",
        span.len()
    );
}

/// EX-14(a) / VA-7 — the STORED row keeps full fidelity: the whole regression
/// reason exactly as accepted, and the whole digest. A render-side cap leaking
/// into the store would show up here as a shortened value.
#[test]
fn stored_change_row_keeps_full_reason_and_fingerprint() {
    let fixture = every_event_fixture();
    let reason = "the review found the section incoherent, so the run regresses to redraft it \
                  before any further attestation is recorded against prose nobody stands behind";
    fixture.apply(&format!(
        "{{{},\"stage\":{{\"to\":\"exploring\",\"reason\":\"{reason}\"}}}}",
        fixture.envelope("regress")
    ));

    let log = fixture.read().change_log;
    let stored_reason = log
        .rows
        .iter()
        .filter(|row| row.event == ChangeEvent::StageMoved)
        .flat_map(|row| row.terms.iter())
        .find(|term| term.kind() == ValueKind::Prose)
        .expect("the regression row stores its reason");
    assert_eq!(
        stored_reason.value(),
        reason,
        "byte-identical to the full input after a round trip"
    );

    let stored_digest = log
        .rows
        .iter()
        .filter(|row| row.event == ChangeEvent::SectionFingerprintChanged)
        .flat_map(|row| row.terms.iter())
        .find(|term| term.key() == design_run::change_log::PayloadKey::New)
        .expect("a fingerprint-changed row exists");
    assert_eq!(
        stored_digest.value(),
        common::sha256(REVISED_SECTION_BODY.as_bytes()),
        "the whole digest, not an abbreviation"
    );
}

/// EX-14(c) / VA-7 — the mechanical containment check. Every member of the
/// closed vocabulary, rendered with EVERY scalar saturated at the bound its
/// value kind carries, fits the payload budget.
///
/// Saturation is the point: a hand-picked example proves nothing, and a check
/// that truncated its own output to the cap could never fail. This one can —
/// identity and closed-vocabulary terms render WHOLE, so the only slack is what
/// the admission bounds guarantee.
#[test]
fn rendered_payload_fits_its_cap_for_every_event_kind() {
    for event in ChangeEvent::ALL {
        let terms: Vec<PayloadTerm> = event
            .payload_terms()
            .iter()
            .map(|(key, kind)| match kind {
                ValueKind::Token => PayloadTerm::token(*key, "x".repeat(DESIGN_ID_BYTES))
                    .expect("a saturated identity term is exactly at its admission bound"),
                ValueKind::Label => PayloadTerm::label(*key, "x".repeat(DESIGN_STAGE_LABEL_BYTES))
                    .expect("a saturated label term is exactly at its admission bound"),
                ValueKind::Digest => PayloadTerm::digest(*key, "a".repeat(64)),
                ValueKind::Prose => PayloadTerm::prose(*key, "z".repeat(5000)),
            })
            .collect();
        let row = ChangeRow {
            revision: u64::MAX,
            index: u32::MAX,
            event,
            subject: Some(id(&format!("inq-{}", "y".repeat(DESIGN_ID_BYTES - 4)))),
            terms,
        };
        let payload = design_run::render::change_row::render_payload(&row);
        assert!(
            payload.len() <= ENVELOPE_PAYLOAD_BYTES_UNDER_TEST,
            "{} renders {} bytes of payload, over the budget: {payload}",
            event.as_str(),
            payload.len()
        );
    }
}

/// EX-14(c) — an elided reason carries an EXPLICIT marker, so a reader can tell
/// a short reason from a shortened one.
#[test]
fn elided_reason_carries_an_explicit_marker() {
    let fixture = every_event_fixture();
    let reason = "x".repeat(400);
    fixture.apply(&format!(
        "{{{},\"stage\":{{\"to\":\"exploring\",\"reason\":\"{reason}\"}}}}",
        fixture.envelope("regress")
    ));
    let rendered = fixture.show(&[]);
    assert!(
        rendered.contains(ELISION_MARKER_UNDER_TEST),
        "the rendered reason is marked as elided: {rendered}"
    );
    assert!(
        !rendered.contains(&reason),
        "and the whole reason is not in the budgeted rendering"
    );
}

// ── the layer rule, structurally (EX-15/EX-16) ────────────────────────────

/// VA-8(1) — two ids identical for `DESIGN_ID_BYTES` − 1 bytes and differing
/// only after must render DISTINGUISHABLY. Two short ids would prove nothing:
/// the defect only appears at the bound, which is why identity is bounded at
/// admission and never truncated at emission.
#[test]
fn distinct_ids_sharing_a_long_prefix_render_distinguishably() {
    let shared = "a".repeat(DESIGN_ID_BYTES - "inq-".len() - 1);
    let first = format!("inq-{shared}b");
    let second = format!("inq-{shared}c");
    assert_eq!(first.len(), DESIGN_ID_BYTES);
    assert_eq!(&first[..first.len() - 1], &second[..second.len() - 1]);

    let fixture = Fixture::start();
    fixture.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"{first}\",\"question\":\"q\"}},\
         {{\"subject\":\"{second}\",\"question\":\"q\"}}]}}",
        fixture.envelope("both")
    ));

    let rendered: Vec<String> = fixture
        .show(&["--known-revision", "1"])
        .lines()
        .filter(|line| line.contains("node_created"))
        .map(ToString::to_string)
        .collect();
    assert_eq!(rendered.len(), 2, "both rows rendered: {rendered:?}");
    assert_ne!(rendered[0], rendered[1], "and they are distinguishable");
    assert!(rendered.iter().any(|line| line.contains(&first)));
    assert!(rendered.iter().any(|line| line.contains(&second)));

    // The same must hold for an id carried in the PAYLOAD, not just in the
    // subject column: `needs_added` names both endpoints, and both are identity.
    let revision = fixture.revision();
    fixture.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"{first}\",\"needs\":[\"{second}\"]}}]}}",
        fixture.envelope("edge")
    ));
    let edge = fixture.show(&["--known-revision", &revision.to_string()]);
    assert!(
        edge.contains(&format!("from={first} to={second}")),
        "both endpoint ids render whole in the payload: {edge}"
    );
}

/// VA-9(1) — a stored reason is byte-identical at ANY length, far past every
/// constant in the design, while the rendered row still fits its budget with the
/// elision marker present. Both halves are the point: full fidelity in the
/// store, bounded in the projection, from one input.
#[test]
fn stored_reason_is_byte_identical_at_any_length() {
    let fixture = every_event_fixture();
    // Well past ENVELOPE_REASON_BYTES (240) and past the deleted 2048 — there is
    // no bound of any kind on a stored regression reason (EX-16(b)).
    let reason = "w".repeat(3000);
    fixture.apply(&format!(
        "{{{},\"stage\":{{\"to\":\"exploring\",\"reason\":\"{reason}\"}}}}",
        fixture.envelope("regress")
    ));

    let log = fixture.read().change_log;
    let stored = log
        .rows
        .iter()
        // Scoped to the stage move, and it has to be: PHASE-10's delegation rows
        // carry prose of their own (attribution, a refusal reason), so "the first
        // prose term in the log" stopped being a synonym for "the regression
        // reason" the moment the vocabulary grew. The premise was unstated, not
        // wrong; it is stated now.
        .filter(|row| row.event == ChangeEvent::StageMoved)
        .flat_map(|row| row.terms.iter())
        .find(|term| term.kind() == ValueKind::Prose)
        .expect("the reason is stored");
    assert_eq!(stored.value().len(), 3000, "stored whole");
    assert_eq!(
        stored.value(),
        reason,
        "and byte-identical after a round trip"
    );

    let row = log
        .rows
        .iter()
        .find(|row| {
            row.event == ChangeEvent::StageMoved
                && row.terms.iter().any(|term| term.kind() == ValueKind::Prose)
        })
        .expect("the regression row, which is the one carrying a reason");
    let payload = design_run::render::change_row::render_payload(row);
    assert!(
        payload.len() <= ENVELOPE_PAYLOAD_BYTES_UNDER_TEST,
        "the rendered payload still fits: {} bytes",
        payload.len()
    );
    assert!(
        payload.contains(ELISION_MARKER_UNDER_TEST),
        "with the elision marker present: {payload}"
    );
}

/// VA-9(2) — EVERY run-local id kind refuses a value one byte over
/// `DESIGN_ID_BYTES`. The kinds are enumerated programmatically, because it is
/// the unenumerated kind that carries the defect.
///
/// The criterion's prose names five kinds — node, section, gate, inquiry,
/// attestation. `IdKind::ALL` holds `Inquiry` (node and inquiry are one thing),
/// `Section`, `Checkpoint`, `Attestation`, and — since PHASE-12 — `Integrated`
/// and `Finding`. The loop is what keeps this honest: it enumerates the
/// vocabulary rather than a count, so a kind added later is covered without this
/// sentence being right. A *gate* id is a
/// `gate::Condition` — a closed vocabulary bounded by construction, with no
/// caller-supplied text to admit — so enumerating `IdKind::ALL` is the correct
/// and stronger reading of the obligation.
#[test]
fn every_run_local_id_kind_refuses_an_over_bound_value() {
    let fixture = Fixture::start();
    for kind in IdKind::ALL {
        let over = format!(
            "{}{}",
            kind.prefix(),
            "n".repeat(DESIGN_ID_BYTES + 1 - kind.prefix().len())
        );
        assert_eq!(over.len(), DESIGN_ID_BYTES + 1);
        assert!(
            DesignId::parse(&over).is_err(),
            "the one validating constructor refuses {over}"
        );
        let error = fixture.refuse(&format!(
            "{{{},\"declare\":[{{\"subject\":\"{over}\"}}]}}",
            fixture.envelope("over")
        ));
        assert!(
            error.contains("admission bound"),
            "and so does the admission path for {}: {error}",
            kind.prefix()
        );
    }
    // A refusal, never a trim: nothing landed under a shortened name.
    assert_eq!(fixture.read().map.inquiry.len(), 0);
}

/// Whether `$type` implements `$bound`, answered by the **compiler** rather than
/// by reading source text.
///
/// The inherent associated const on `Probe<T>` applies only when the bound holds;
/// when it does not, name resolution falls through to the blanket trait impl.
/// So the value flips the moment the impl exists — including when it is produced
/// by a `#[derive]`, which is exactly the bypass class a spelling scan misses.
macro_rules! implements {
    ($type:ty, $($bound:tt)+) => {{
        struct Probe<T: ?Sized>(std::marker::PhantomData<T>);
        #[allow(dead_code)]
        trait Absent {
            const IMPLEMENTED: bool = false;
        }
        impl<T: ?Sized> Absent for T {}
        #[allow(dead_code)]
        impl<T: ?Sized + $($bound)+> Probe<T> {
            const IMPLEMENTED: bool = true;
        }
        <Probe<$type>>::IMPLEMENTED
    }};
}

/// VA-9(3) / EX-11(c) — the id type cannot be built by any route that skips
/// [`DesignId::parse`]. This is what makes admission universal rather than
/// remembered: with ids rendered whole, one path accepting a 33-byte id breaks
/// the row premise while every other test still passes.
///
/// **There is no source-text scan here, deliberately.** RV-321 F-3 established
/// why the previous shape could not discharge this: counting the literal
/// `DesignId {` is the grep-as-test VA-9(3) explicitly withdrew, and it stayed
/// green for `#[derive(Default)]` and for a hand-written `Deserialize`. The
/// replacement is not a second grep — it is the three routes themselves:
///
/// 1. **the struct literal** — the inner field is private, so `DesignId { .. }`
///    outside `ids.rs` does not *compile*. That guarantee is the compiler's and
///    no runtime assertion can observe it; it is named here so a later reader
///    does not mistake its absence for an omission.
/// 2. **an inbound impl** — `Default` and `From<String>` each manufacture a
///    `DesignId` without naming the field. Both are asserted absent, and the
///    assertion is a type-system query, so adding either derive turns it red.
/// 3. **deserialization** — asserted below to route through `parse`, by feeding
///    values `parse` refuses and observing the refusal.
#[test]
fn no_id_is_constructed_outside_the_validating_constructor() {
    assert!(
        !implements!(DesignId, Default),
        "`DesignId: Default` would mint an unvalidated empty id with no field access \
         and no call to `parse`"
    );
    assert!(
        !implements!(DesignId, From<String>),
        "an infallible `From<String>` is a raw constructor wearing a conversion's name; \
         the only inbound conversion is the fallible `TryFrom<String>`"
    );
    assert!(
        implements!(DesignId, TryFrom<String, Error = design_run::refusal::Refusal>),
        "and the one inbound conversion that does exist is the fallible one, so serde's \
         `try_from` cannot be pointed at a lossy sibling"
    );

    // Route 3, both polarities: what `parse` refuses, deserialization refuses.
    for refused in [
        "x".repeat(DESIGN_ID_BYTES + 1), // over the admission bound
        format!("inq-{}", "x".repeat(DESIGN_ID_BYTES)), // over it, with a good prefix
        "inq-".to_owned(),               // empty body after the prefix
        "nope-1".to_owned(),             // unknown prefix
    ] {
        assert!(
            DesignId::parse(&refused).is_err(),
            "`parse` refuses {refused}"
        );
        assert!(
            serde_json::from_str::<DesignId>(&format!("\"{refused}\"")).is_err(),
            "and so does the deserialization route for {refused} — a derived or \
             hand-written `Deserialize` that skipped `parse` would accept it"
        );
    }
    let lawful = "inq-1";
    assert_eq!(
        serde_json::from_str::<DesignId>(&format!("\"{lawful}\""))
            .expect("a well-formed id deserialises")
            .as_str(),
        lawful,
        "the route is validating, not merely refusing"
    );
}

/// EX-11(a) / VA-7(1) — an over-bound identity or label cannot reach a payload
/// term by EITHER route, and the deserialization leg is the one that matters:
/// it is the route that re-entered without re-validating, so a fix guarding only
/// the constructors would leave the whole finding open.
#[test]
fn over_bound_payload_terms_are_refused_at_construction_and_on_the_wire() {
    // (a) the constructors.
    let over_id = "x".repeat(DESIGN_ID_BYTES + 1);
    let over_label = "y".repeat(DESIGN_STAGE_LABEL_BYTES + 1);
    assert!(
        PayloadTerm::token(design_run::change_log::PayloadKey::Node, over_id.clone()).is_err(),
        "an identity term one byte over `DESIGN_ID_BYTES` is refused, never trimmed"
    );
    assert!(
        PayloadTerm::label(
            design_run::change_log::PayloadKey::Disposition,
            over_label.clone()
        )
        .is_err(),
        "and so is a label term one byte over `DESIGN_STAGE_LABEL_BYTES`"
    );
    assert!(
        PayloadTerm::token(
            design_run::change_log::PayloadKey::Node,
            "x".repeat(DESIGN_ID_BYTES)
        )
        .is_ok(),
        "exactly at the bound is admitted — this is a bound, not an off-by-one"
    );

    // (b) the wire. A hand-edited snapshot claiming an over-bound value is a
    // token is refused at parse, before any rendering can emit it.
    let fixture = every_event_fixture();
    let stored = std::fs::read_to_string(&fixture.snapshot).unwrap();
    let doctored = stored.replace(
        &format!("value = \"{MINTED_RECORD}\""),
        &format!("value = \"{over_id}\""),
    );
    assert_ne!(
        doctored, stored,
        "the fixture stores a token term to doctor"
    );
    std::fs::write(&fixture.snapshot, &doctored).unwrap();

    let error = fail(&fixture.root, &["design", "show", SLICE, "-p", "."]);
    assert!(
        error.contains("admission bound"),
        "the deserialization path re-validates rather than re-entering: {error}"
    );
    assert!(
        !error.contains(ELISION_MARKER_UNDER_TEST),
        "and it REFUSES rather than trimming the value to fit"
    );
}

// ── PHASE-06: one wire, one route (EX-12, EX-13(b), EX-14) ────────────────
//
// Three defects that let the snapshot and the wire disagree in silence: a
// second, unvalidated route to `Disposition::Adopted`; a `title` field kept in
// agreement with the prose by nothing at all; and a payload whose surplus keys
// were swallowed rather than refused.

/// The fragment each PHASE-06 derivation refusal renders. Named once, because
/// a table of expectations that re-types prose per row is a table of typos.
const BODY_EMPTY: &str = "empty body";
/// [`Refusal::SectionBodyHeadingMissing`]'s rendering.
const HEADING_MISSING: &str = "ATX heading";
/// [`Refusal::SectionTitleEmpty`]'s rendering.
const TITLE_EMPTY: &str = "heading has no text";

/// EX-13(b) — a section's title is DERIVED from its own body, so `title` is no
/// longer a wire field. A payload that still declares one is refused as an
/// unknown key rather than accepted beside a `body` it may contradict.
#[test]
fn declare_refuses_removed_title_wire_field() {
    let fixture = Fixture::start();
    let before = fixture.bytes();

    let error = fixture.refuse(&fixture.payload(
        "out-of-band-title",
        &json!({ "declare": [{
            "subject": "sec-1",
            "title": "Declared out of band",
            "body": "## Derived from the body\n\nprose\n",
        }] }),
    ));
    assert!(
        error.contains("unknown field") && error.contains("title"),
        "the removed `title` key is refused by name: {error}"
    );
    assert_eq!(fixture.bytes(), before, "and the run did not advance");
}

/// EX-12 / EX-14 — the pre-effect annotation pair is off the wire, and
/// `deny_unknown_fields` is what refuses it. Each payload carries a spelling
/// that would land it on its own, so none of these can pass because the
/// declaration was incomplete.
#[test]
fn declare_refuses_removed_annotation_wire_fields() {
    let fixture = Fixture::start();
    fixture.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-1\",\"question\":\"q\"}},\
         {{\"subject\":\"inq-2\",\"question\":\"q\"}}]}}",
        fixture.envelope("seed")
    ));
    let before = fixture.bytes();

    // (a) `adopt_record` beside a valid `dispose`: without the disposition this
    // would refuse as an incomplete checkpoint and pass for the wrong reason.
    let error = fixture.refuse(&fixture.payload(
        "adopt-record",
        &json!({ "declare": [{
            "subject": "cp-1",
            "disposes": "inq-1",
            "adopt_record": true,
            "dispose": { "form": "non-durable", "note": "a naming aside" },
        }] }),
    ));
    assert!(
        error.contains("unknown field") && error.contains("adopt_record"),
        "`adopt_record` is refused by name: {error}"
    );

    // (b) `record` riding a declaration that is otherwise complete — the key was
    // top-level on every subject kind, so a section swallowed it too.
    let error = fixture.refuse(&fixture.payload(
        "record-on-a-section",
        &json!({ "declare": [{
            "subject": "sec-1",
            "body": "## A section\n\nprose\n",
            "record": "DEC-001",
        }] }),
    ));
    assert!(
        error.contains("unknown field") && error.contains("record"),
        "`record` is refused by name: {error}"
    );

    // (c) `record` beside a valid `dispose` — the same key, on the subject kind
    // it was invented for.
    let error = fixture.refuse(&fixture.payload(
        "record-and-dispose",
        &json!({ "declare": [{
            "subject": "cp-2",
            "disposes": "inq-2",
            "record": "DEC-001",
            "dispose": { "form": "non-durable", "note": "a naming aside" },
        }] }),
    ));
    assert!(
        error.contains("unknown field") && error.contains("record"),
        "and it is refused as an unknown key, not as an ambiguity: {error}"
    );

    assert_eq!(fixture.bytes(), before, "no payload advanced the run");
}

/// EX-12 — `Disposition::Adopted` has exactly ONE route. The annotation pair
/// used to reach it (and `Disposition::Created`) without passing the adoption
/// validation `dispose` goes through, which is the whole defect.
#[test]
fn checkpoint_annotation_spelling_is_refused() {
    let fixture = Fixture::start();
    fixture.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-1\",\"question\":\"q\"}},\
         {{\"subject\":\"inq-2\",\"question\":\"q\"}}]}}",
        fixture.envelope("seed")
    ));

    // The arm that reached `Adopted`.
    fixture.refuse(&fixture.payload(
        "annotated-adopt",
        &json!({ "declare": [{
            "subject": "cp-1", "disposes": "inq-1",
            "record": "DEC-001", "adopt_record": true,
        }] }),
    ));
    // The arm that reached `Created`.
    fixture.refuse(&fixture.payload(
        "annotated-create",
        &json!({ "declare": [{
            "subject": "cp-2", "disposes": "inq-2", "record": "ASM-001",
        }] }),
    ));

    let after = fixture.read();
    for node in ["inq-1", "inq-2"] {
        assert!(
            after
                .map
                .inquiry
                .get(&id(node))
                .unwrap()
                .disposition()
                .is_none(),
            "{node} was not disposed by a spelling that no longer exists"
        );
    }
}

/// EX-13(b) — the derivation, over the whole input domain rather than over a
/// list of forms someone thought of.
///
/// **The rows are the marker-grammar sketch's recorded probe output**, not a
/// recollection of CommonMark: `.doctrine/slice/233/sketches/marker-grammar.md`
/// § *The derivation, stated totally* and § *The oracle this rule owes its shape
/// to* classify seven hashes, `#hashtag`, a four-space indent and the closing
/// sequence by measurement against `prettier@3.9.6`. That probe is not re-run
/// here — no formatter is installed in this tree, and nothing below asserts
/// anything about one.
#[test]
fn section_title_derivation_covers_every_heading_form() {
    // (body, Ok(derived title) | Err(the refusal's rendering fragment))
    let rows: Vec<(&str, Result<&str, &str>)> = vec![
        // Arm 4: the ordinary heading, one through six hashes.
        ("## Title\n", Ok("Title")),
        ("# T\n", Ok("T")),
        ("## T\n", Ok("T")),
        ("### T\n", Ok("T")),
        ("#### T\n", Ok("T")),
        ("##### T\n", Ok("T")),
        ("###### T\n", Ok("T")),
        // The closing sequence is framing, not content.
        ("## Title ##\n", Ok("Title")),
        // …and dropping it must not stop at one pass: the cascade.
        ("## ###\n", Err(TITLE_EMPTY)),
        ("## # # #\n", Err(TITLE_EMPTY)),
        // A tab is a delimiter, exactly as a space is.
        ("#\tT\n", Ok("T")),
        // Up to three spaces of indent is still a heading; four is not.
        ("   ## Title\n", Ok("Title")),
        ("    ## Title\n", Err(HEADING_MISSING)),
        // Arm 2: shapes that are not ATX heading lines at all.
        ("####### T\n", Err(HEADING_MISSING)),
        ("#hashtag\n", Err(HEADING_MISSING)),
        ("T\n===\n", Err(HEADING_MISSING)),
        ("```\n## Title\n```\n", Err(HEADING_MISSING)),
        // Arm 1: no non-blank line at all.
        ("", Err(BODY_EMPTY)),
        ("\n", Err(BODY_EMPTY)),
        ("   \n\t\n", Err(BODY_EMPTY)),
        // Arm 3: a heading line whose extracted text is empty.
        ("##\n", Err(TITLE_EMPTY)),
        // *f* is the first NON-BLANK line, not the first line.
        ("\n\n## Late\n", Ok("Late")),
        // Every other line, heading or not, stays ordinary body content.
        ("## First\n\n## Second\n", Ok("First")),
    ];

    let fixture = Fixture::start();
    for (index, (body, expected)) in rows.iter().enumerate() {
        let subject = format!("sec-{}", index + 1);
        let payload = fixture.payload(
            &format!("row-{index}"),
            &json!({ "declare": [{ "subject": subject, "body": body }] }),
        );
        match expected {
            Ok(title) => {
                fixture.apply(&payload);
                let section = fixture.read();
                let section = section.sections.find(&id(&subject)).unwrap();
                assert_eq!(&section.title, title, "derived from {body:?}");
                assert_eq!(
                    &section.body, body,
                    "and the body is stored whole, headings and all"
                );
            }
            Err(fragment) => {
                let error = fixture.refuse(&payload);
                assert!(
                    error.contains(fragment),
                    "{body:?} is refused, naming `{fragment}`: {error}"
                );
            }
        }
    }
}

/// A **product** corpus of section bodies — indents × hash runs × delimiters ×
/// contents × trailers, plus first lines that are not headings at all for arms
/// 1 and 2. A generated corpus rather than a hand-written list, because three
/// successive hand-written tables of "heading forms someone thought of" each
/// shipped a defect the next one found (the sketch's § *The oracle*).
fn generated_bodies() -> Vec<String> {
    const LEADING: [&str; 3] = ["", "\n", "  \n\t\n"];
    const INDENTS: [&str; 5] = ["", " ", "  ", "   ", "    "];
    const HASHES: [&str; 8] = ["", "#", "##", "###", "####", "#####", "######", "#######"];
    const DELIMITERS: [&str; 4] = ["", " ", "\t", "  "];
    const CONTENTS: [&str; 9] = [
        "Title", "a  b", "#", "# #", "###", "#hashtag", "T ##", "", "*em*",
    ];
    const TRAILERS: [&str; 6] = ["", " ", " ##", "##", " #  ", "\t###"];

    let mut bodies = Vec::new();
    for leading in LEADING {
        for indent in INDENTS {
            for hashes in HASHES {
                for delimiter in DELIMITERS {
                    for content in CONTENTS {
                        for trailer in TRAILERS {
                            bodies.push(format!(
                                "{leading}{indent}{hashes}{delimiter}{content}{trailer}\n\
                                 ordinary body content\n"
                            ));
                        }
                    }
                }
            }
        }
    }
    // Arms 1 and 2: first lines that are not ATX heading lines.
    for body in [
        "Setext\n===\n",
        "```\n## Fenced\n```\n",
        "prose first\n\n## Late\n",
        "",
        "\n",
        "   \n\t\n",
    ] {
        bodies.push(body.to_owned());
    }
    bodies
}

/// EX-13(b) — re-emitting a derived title as a heading and re-deriving yields
/// the SAME title, over a generated corpus.
///
/// This is the property the rule owes its shape to. It is asserted here rather
/// than reasoned about because reading is what produced the three tables that
/// were each wrong: the cascade (`## # # #` → `# #` → `#` → nothing) is a fixed
/// point failure no enumeration of "forms" exposes.
#[test]
fn section_title_derivation_is_idempotent_over_generated_bodies() {
    let subject = id("sec-1");
    let mut derived = 0_usize;
    let mut divergences: Vec<String> = Vec::new();

    for body in generated_bodies() {
        let Ok(title) = design_run::section::derive_title(&subject, &body) else {
            continue;
        };
        derived += 1;
        let reemitted = format!("## {title}\n");
        match design_run::section::derive_title(&subject, &reemitted) {
            Ok(again) if again == title => {}
            other => divergences.push(format!("{body:?} → {title:?} → {other:?}")),
        }
    }

    assert!(
        derived > 1_000,
        "the corpus must actually exercise arm 4: only {derived} bodies derived a title"
    );
    assert!(
        divergences.is_empty(),
        "{} of {derived} derived titles are not fixed points; first ten:\n{}",
        divergences.len(),
        divergences
            .iter()
            .take(10)
            .cloned()
            .collect::<Vec<String>>()
            .join("\n")
    );
}

/// EX-13(b) — internal whitespace runs survive the derivation: `# a  b` derives
/// `a  b`, double space intact.
///
/// This test pins an ABSENCE. Collapsing internal whitespace closed a real
/// family of formatter divergences and was measured doing so, but its only
/// justification was one formatter's behaviour and the stability claim that
/// rested on has been withdrawn. A rule whose derivation is retracted does not
/// get kept because it is already written.
#[test]
fn section_title_keeps_internal_whitespace_runs() {
    let subject = id("sec-1");
    assert_eq!(
        design_run::section::derive_title(&subject, "# a  b\n").unwrap(),
        "a  b"
    );
}

/// EX-12 / EX-4 — an adoption names a record the corpus holds, and the check
/// runs BEFORE anything is recorded. With one spelling there is one place that
/// can be true; with two, the second one skipped the check entirely.
#[test]
fn checkpoint_adopt_validates_target_before_recording() {
    let fixture = Fixture::start();
    fixture.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"inq-1\",\"question\":\"q\"}},\
         {{\"subject\":\"inq-2\",\"question\":\"q\"}}]}}",
        fixture.envelope("seed")
    ));
    let before = fixture.bytes();

    // (a) the surviving spelling validates its target.
    let error = fixture.refuse(&fixture.payload(
        "adopt-a-ghost",
        &json!({ "declare": [{
            "subject": "cp-1", "disposes": "inq-1",
            "dispose": { "form": "adopt", "record": "DEC-404" },
        }] }),
    ));
    assert!(
        error.contains("names no record this project holds"),
        "the adoption target is resolved against the corpus: {error}"
    );

    // (b) and there is no second spelling that reaches `Adopted` around it.
    fixture.refuse(&fixture.payload(
        "annotate-a-ghost",
        &json!({ "declare": [{
            "subject": "cp-2", "disposes": "inq-2",
            "record": "DEC-404", "adopt_record": true,
        }] }),
    ));

    assert_eq!(
        fixture.bytes(),
        before,
        "nothing was recorded before the target was validated"
    );
}

/// EX-13(b) — a retitle is an edit to the body's own heading, so it moves the
/// section's fingerprint and DEC-066 invalidates the attestation bound to the
/// prose that no longer exists. Under an independently declared `title` a
/// retitle moved nothing at all, and prior review survived a change to the
/// section's most visible claim.
#[test]
fn retitle_moves_the_section_fingerprint_and_invalidates_attestation() {
    let fixture = Fixture::start();
    let section = |body: &str, submission: &str| {
        fixture.payload(
            submission,
            &json!({ "declare": [{ "subject": "sec-1", "body": body }] }),
        )
    };
    fixture.apply(&section(
        "## Old title\n\nprose that does not change\n",
        "draft",
    ));
    let before = fixture.read();
    let before = before.sections.find(&id("sec-1")).unwrap();
    assert_eq!(before.title, "Old title", "the title came from the heading");
    let old_fingerprint = before.fingerprint.clone();

    fixture.apply(&format!(
        "{{{},\"declare\":[{{\"subject\":\"att-1\",\"attests\":\"sec-1\"}}]}}",
        fixture.envelope("attest")
    ));
    assert!(
        fixture.show(&[]).contains("review=current"),
        "the attestation is live against the reviewed bytes"
    );

    // The retitle: the heading changes, the prose does not.
    fixture.apply(&section(
        "## New title\n\nprose that does not change\n",
        "retitle",
    ));

    let after = fixture.read();
    let after = after.sections.find(&id("sec-1")).unwrap();
    assert_eq!(
        after.id,
        id("sec-1"),
        "identity is invariant under a retitle"
    );
    assert_eq!(
        after.title, "New title",
        "and the title moved with the prose"
    );
    assert_ne!(
        after.fingerprint, old_fingerprint,
        "the heading is inside the fingerprint, so the fingerprint moved"
    );
    assert!(
        fixture.show(&[]).contains("review=outstanding"),
        "and the attestation bound to the old bytes is no longer live"
    );
}

// ── PHASE-13 EX-3 / EX-4 / EX-5(c) — the document's framing and its edges ──

/// EX-3 — a section body survives materialise-then-parse BYTE FOR BYTE,
/// terminal whitespace and all, at section counts 1, 2 **and** 3.
///
/// The count matters more than the bodies do. `render_document` used to build
/// each block as marker + `\n` + body + `\n` and then `blocks.join("\n")`, so an
/// INTERIOR block carried two framing newlines and the last carried one — a
/// defect invisible at n=1 and living entirely in the interior/last distinction.
/// The oracle is byte equality over the WHOLE document, never equality modulo
/// whitespace, because equality modulo whitespace passes against the very defect
/// this test exists to close.
#[test]
fn section_body_round_trips_byte_exactly_including_terminal_whitespace() {
    // One body ends in a BLANK LINE, one in TRAILING SPACES, one is ordinary.
    // Each opens with its own heading, because since EX-13(b) that is what a
    // body is.
    let corpus = [
        ("sec-1", "## One\n\ninterior prose\n\n"),
        ("sec-2", "## Two\n\ntrailing spaces  "),
        ("sec-3", "## Three\n\nlast prose\n"),
    ];

    let mut runs: Vec<(Fixture, &[(&str, &str)])> = Vec::new();
    for count in 1..=corpus.len() {
        let declared = corpus.get(..count).expect("a prefix of the corpus");
        let fixture = Fixture::start();
        let declarations: Vec<Value> = declared
            .iter()
            .map(|(id, body)| json!({ "subject": id, "body": body }))
            .collect();
        fixture.apply(&fixture.payload("seed", &json!({ "declare": declarations })));
        run(&fixture.root, &["design", "materialise", SLICE, "-p", "."]);
        runs.push((fixture, declared));
    }

    // (a) The WRITE side: the document is the uniform affix, byte for byte.
    for (fixture, declared) in &runs {
        assert_eq!(
            std::fs::read_to_string(fixture.doc()).unwrap(),
            framed(declared),
            "{} section(s): the emitted document is the uniform affix",
            declared.len()
        );
    }

    // (b) The READ side, proved with digests of the ORIGINAL declared bytes: a
    // re-adoption validates only if parse recovered every body exactly.
    for (fixture, declared) in &runs {
        let document = std::fs::read(fixture.doc()).unwrap();
        let sections: serde_json::Map<String, Value> = declared
            .iter()
            .map(|(id, body)| ((*id).to_owned(), json!(common::sha256(body.as_bytes()))))
            .collect();
        fixture.apply(&fixture.payload(
            "round-trip",
            &json!({ "adopt_authored": {
                "fingerprint": common::sha256(&document),
                "sections": Value::Object(sections),
            }}),
        ));
    }
}

/// EX-4 — `CarriageReturnInDocument` is the document boundary, and DECLARATION
/// ADMISSION enforces the same one.
///
/// A `\r` accepted at declare and refused only at parse is the split-door
/// defect: the body would reach title derivation and materialise through a door
/// the document boundary has already closed, and the refusal would arrive
/// against bytes the caller no longer recognises as theirs.
#[test]
fn declare_refuses_carriage_return_on_the_document_boundary() {
    let fixture = Fixture::start();
    let before = fixture.bytes();

    let error = fixture.refuse(&fixture.payload(
        "crlf",
        &json!({ "declare": [{ "subject": "sec-1", "body": "## Title\r\n\r\nprose\r\n" }] }),
    ));

    assert!(
        error.contains("carriage return"),
        "the refusal names the boundary: {error}"
    );
    assert!(
        error.contains("dos2unix"),
        "and the one-command fix: {error}"
    );
    assert_eq!(fixture.bytes(), before, "and no section was seated");
}

/// EX-6(a) — a run written before `seq` existed materialises in the id order it
/// already had, so the migration is a no-op **by construction** rather than by a
/// migration pass someone has to get right.
///
/// The fixture ages the snapshot by removing the field such a run never carried:
/// `serde(default)` then reads every `seq` as 0, and a STABLE sort over equal
/// keys is the identity on the id-ordered group.
#[test]
fn sections_without_a_stored_seq_keep_their_existing_id_order() {
    let fixture = Fixture::start();
    for (index, id) in ["sec-2", "sec-11"].into_iter().enumerate() {
        fixture.apply(&fixture.payload(
            &format!("declare-{index}"),
            &json!({ "declare": [{ "subject": id, "body": format!("## {id}\n\nprose\n") }] }),
        ));
    }

    let aged: String = std::fs::read_to_string(&fixture.snapshot)
        .unwrap()
        .lines()
        .filter(|line| !line.starts_with("seq = "))
        .map(|line| format!("{line}\n"))
        .collect();
    assert!(
        !aged.contains("\nseq = "),
        "the aged snapshot carries no section seq"
    );
    std::fs::write(&fixture.snapshot, aged).unwrap();

    run(&fixture.root, &["design", "materialise", SLICE, "-p", "."]);
    let document = std::fs::read_to_string(fixture.doc()).unwrap();
    assert!(
        document.find(&section_marker("sec-11")) < document.find(&section_marker("sec-2")),
        "with no seq to order by, the group's existing id order stands:\n{document}"
    );
}

/// EX-5(c) — ids that are distinct but share a long common prefix resolve to
/// their OWN regions. Two fixtures, both from the sketch's answer (d).
///
/// `sec-11` is declared FIRST so it is the earlier region in document order:
/// under prefix matching the later, shorter `sec-1` would be answered by
/// `sec-11`'s region, and a fixture that declared the shorter one first would
/// pass against exactly that defect. The second pair differs only in byte 32,
/// which is the adversarial case rather than the ordinary one.
#[test]
fn sections_sharing_a_long_common_prefix_resolve_distinctly() {
    let long_a = format!("sec-{}0", "a".repeat(27));
    let long_b = format!("sec-{}1", "a".repeat(27));
    assert_eq!(
        long_a.len(),
        DESIGN_ID_BYTES,
        "the adversarial pair is at the bound"
    );

    let corpus = [
        ("sec-11", "## Eleven\n\neleven's prose\n"),
        ("sec-1", "## One\n\none's prose\n"),
        (long_a.as_str(), "## Long A\n\nlong a's prose\n"),
        (long_b.as_str(), "## Long B\n\nlong b's prose\n"),
    ];

    let fixture = Fixture::start();
    for (index, (id, body)) in corpus.iter().enumerate() {
        fixture.apply(&fixture.payload(
            &format!("declare-{index}"),
            &json!({ "declare": [{ "subject": id, "body": body }] }),
        ));
    }
    run(&fixture.root, &["design", "materialise", SLICE, "-p", "."]);

    let document = std::fs::read(fixture.doc()).unwrap();
    let sections: serde_json::Map<String, Value> = corpus
        .iter()
        .map(|(id, body)| ((*id).to_owned(), json!(common::sha256(body.as_bytes()))))
        .collect();
    fixture.apply(&fixture.payload(
        "resolve",
        &json!({ "adopt_authored": {
            "fingerprint": common::sha256(&document),
            "sections": Value::Object(sections),
        }}),
    ));

    // Byte equality on the whole id is what resolves them; a prefix match would
    // have handed one region's prose to two ids.
    let after = fixture.read();
    for (id, body) in corpus {
        assert_eq!(
            after
                .sections
                .find(&self::id(id))
                .expect("every declared section survives")
                .fingerprint
                .as_str(),
            common::sha256(body.as_bytes()),
            "{id} kept its own region"
        );
    }
}

/// EX-5 — a hand edit that has been RE-ADOPTED survives the next materialise.
///
/// The oracle is the BYTES of `design.md` after the second materialise, and it
/// is deliberately neither the exit code nor the fingerprint (VA-6). The
/// fingerprint matching is PRECISELY the condition that holds while the body is
/// reverted — `adopt_authored` records the digest of bytes the snapshot does not
/// hold — so a fingerprint assertion passes against the very defect this test
/// exists to close, and the watermark then certifies the reverted prose.
#[test]
fn readopted_hand_edit_survives_a_subsequent_materialise() {
    let fixture = materialised("first draft");

    let edited = "## First draft\n\na human rewrote this by hand\n";
    let document = authored_document("sec-1", edited);
    std::fs::write(fixture.doc(), &document).unwrap();

    fixture.apply(&format!(
        "{{{},\"adopt_authored\":{{\"fingerprint\":\"{}\",\
         \"sections\":{{\"sec-1\":\"{}\"}}}}}}",
        fixture.envelope("adopt"),
        common::sha256(document.as_bytes()),
        common::sha256(edited.as_bytes()),
    ));

    run(&fixture.root, &["design", "materialise", SLICE, "-p", "."]);

    assert_eq!(
        std::fs::read_to_string(fixture.doc()).unwrap(),
        document,
        "the second materialise re-emitted the ADOPTED bytes, not the pre-edit body"
    );
}

/// EX-6 — the title derivation runs at ADOPTION, by the same procedure the
/// declare path runs.
///
/// Two halves, because "the same procedure" is a claim about agreement in both
/// directions: a body whose heading is underivable is refused at adoption with
/// the SAME refusal declare gives it, and a derivable one seats the SAME title.
/// Without the adoption arm a hand-edited document seats a section whose title
/// is underivable — the partiality re-entering through the other door.
#[test]
fn adoption_derives_the_title_by_the_same_procedure_as_declare() {
    // (a) The refusal. `derive_title`'s arm 2: the first non-blank line is not
    // an ATX heading line.
    let fixture = materialised("first draft");
    let headless = "a human deleted the heading\n";
    let document = authored_document("sec-1", headless);
    std::fs::write(fixture.doc(), &document).unwrap();

    // The declare door is probed on its OWN run: rule 1 entry-refuses ordinary
    // mutation against a diverged document, so a declare submitted into the
    // edited fixture would never reach the derivation at all.
    let declaring = Fixture::start();
    let declared = declaring.refuse(&declaring.payload(
        "declare-headless",
        &json!({ "declare": [{ "subject": "sec-2", "body": headless }] }),
    ));
    let adopted = fixture.refuse(&format!(
        "{{{},\"adopt_authored\":{{\"fingerprint\":\"{}\",\
         \"sections\":{{\"sec-1\":\"{}\"}}}}}}",
        fixture.envelope("adopt-headless"),
        common::sha256(document.as_bytes()),
        common::sha256(headless.as_bytes()),
    ));
    for (door, error) in [("declare", &declared), ("adopt", &adopted)] {
        assert!(
            error.contains("does not begin with an ATX heading"),
            "the {door} door refuses an underivable heading by the same name: {error}"
        );
    }

    // (b) The agreement. A derivable heading seats the SAME title through both
    // doors — one procedure, not two that happen to agree on the refusal.
    let body = "## Retitled by hand ###\n\nprose\n";
    let document = authored_document("sec-1", body);
    std::fs::write(fixture.doc(), &document).unwrap();
    fixture.apply(&format!(
        "{{{},\"adopt_authored\":{{\"fingerprint\":\"{}\",\
         \"sections\":{{\"sec-1\":\"{}\"}}}}}}",
        fixture.envelope("adopt-titled"),
        common::sha256(document.as_bytes()),
        common::sha256(body.as_bytes()),
    ));

    declaring.apply(&declaring.payload(
        "declare-titled",
        &json!({ "declare": [{ "subject": "sec-1", "body": body }] }),
    ));

    assert_eq!(
        fixture.read().sections.find(&id("sec-1")).unwrap().title,
        declaring
            .read()
            .sections
            .find(&id("sec-1"))
            .unwrap()
            .title
            .clone(),
        "both doors derive one title from one procedure"
    );
    assert_eq!(
        fixture.read().sections.find(&id("sec-1")).unwrap().title,
        "Retitled by hand",
        "and it is the derivation's own answer, closing sequence dropped"
    );
}

/// Parse a run-local id in a test, where a malformed literal is a test bug.
fn id(raw: &str) -> DesignId {
    DesignId::parse(raw).expect("a well-formed run-local id")
}
