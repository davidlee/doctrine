//! SL-233 PHASE-13/PHASE-14 — the authored document Doctrine writes and reads
//! back, over the built binary: the marker grammar's round trip, document order
//! on both the write and the read side, the five §5.5 refusals, and the
//! deprecated `slice design` shim.
//!
//! Every behaviour here is end-to-end against `design materialise` and
//! `design adopt`, because the property under test is
//! `parse(materialise(S)) == S` and only the shell owns both halves.
//!
//! The fixture below is this binary's own. Integration tests are separate
//! compilation units and this crate is **binary-only** (no `[lib]`), so a
//! fixture cannot be imported from a sibling test binary; the pure model is
//! `#[path]`-included instead, which is the CHR-014 idiom the suite already uses.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

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

use design_run::snapshot::{self, DesignSnapshot};

/// The slice every fixture designs.
const SLICE: &str = "SL-233";
/// Its zero-padded directory name.
const SLICE_NUMBER: &str = "233";
/// The authored design document's file name.
const DESIGN_DOC: &str = "design.md";

/// The stable-section marker `materialise` writes — the one place this suite
/// spells that grammar (the constants are private to a binary-only crate, so one
/// copy is the floor the layout imposes).
fn section_marker(section: &str) -> String {
    format!("<!-- doctrine:section {section} -->")
}

/// The section ids the document names, **in document order** — read off the
/// marker lines, which is the only order a reader of `design.md` can observe.
///
/// Recognition is the grammar's own, not a second hand-written spelling of it:
/// two recognisers kept in agreement is one more than this suite needs, and a
/// looser one here would count an unescapable lookalike as a boundary.
fn marker_order(document: &str) -> Vec<String> {
    document
        .lines()
        .filter_map(|line| design_run::document::marker(line).map(|id| id.as_str().to_owned()))
        .collect()
}

// ── fixture ───────────────────────────────────────────────────────────────

/// A started design run in a throwaway tree.
struct Fixture {
    _tmp: tempfile::TempDir,
    root: PathBuf,
    /// Learned from `design start`'s own output, so no test re-types the state
    /// path.
    snapshot: PathBuf,
    uid: String,
}

impl Fixture {
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

    /// The authored document's exact bytes.
    fn document(&self) -> String {
        std::fs::read_to_string(self.doc()).unwrap()
    }

    fn read(&self) -> DesignSnapshot {
        snapshot::parse(&std::fs::read_to_string(&self.snapshot).unwrap()).unwrap()
    }

    /// A payload carrying the current revision and `submission`, plus `body`'s
    /// top-level keys merged in.
    fn payload(&self, submission: &str, body: &Value) -> String {
        let mut object = json!({
            "run_uid": self.uid,
            "known_revision": self.read().run.revision,
            "submission_id": submission,
        });
        let map = object.as_object_mut().unwrap();
        for (key, value) in body.as_object().unwrap() {
            map.insert(key.clone(), value.clone());
        }
        object.to_string()
    }

    /// Apply a payload, expecting success.
    fn apply(&self, submission: &str, body: &Value) -> String {
        let payload = self.payload(submission, body);
        run(
            &self.root,
            &["design", "apply", SLICE, "-p", ".", "--input", &payload],
        )
    }

    /// Declare one section.
    fn declare(&self, submission: &str, id: &str, body: &str) {
        self.apply(
            submission,
            &json!({ "declare": [{ "subject": id, "body": body }] }),
        );
    }

    fn materialise(&self) {
        run(&self.root, &["design", "materialise", SLICE, "-p", "."]);
    }

    /// Replace the authored document with bytes a human left behind.
    fn write_doc(&self, text: &str) {
        std::fs::write(self.doc(), text).unwrap();
    }

    /// `design adopt`, expecting success; returns stdout.
    fn adopt(&self, extra: &[&str]) -> String {
        let mut args = vec!["design", "adopt", SLICE, "-p", "."];
        args.extend_from_slice(extra);
        run(&self.root, &args)
    }

    /// `design adopt`, expecting refusal; returns stderr.
    fn refuse_adopt(&self, extra: &[&str]) -> String {
        let mut args = vec!["design", "adopt", SLICE, "-p", "."];
        args.extend_from_slice(extra);
        fail(&self.root, &args)
    }

    /// Force a divergence that moves no section body and read the parser's own
    /// section decomposition out of `adopt --dry-run` (`SL-261` `PHASE-05`).
    ///
    /// After `materialise` the document is aligned, so `adopt` short-circuits to
    /// the no-op before parsing. One blank line prepended is a **whitespace-only
    /// head**: the parser accepts it and `materialise` drops it, so the report
    /// has no `section_fingerprint_changed` row and its `unchanged` line lists
    /// every section the run holds. Held fingerprints are the digests of the
    /// declared bodies, so `unchanged` means every body parsed back byte for
    /// byte.
    fn parser_readout(&self) -> String {
        let document = self.document();
        self.write_doc(&format!("\n{document}"));
        let report = self.adopt(&["--dry-run"]);
        assert!(
            report.contains("head: "),
            "the probe forced a whitespace head: {report}"
        );
        report
    }
}

/// The uniform block affix `materialise` emits — the ONE place this suite
/// spells it (the constants are private to a binary-only crate). For each
/// section: the marker line, a newline, the body verbatim, then ONE newline,
/// blocks concatenated with no separator.
fn framed(sections: &[(&str, &str)]) -> String {
    sections
        .iter()
        .map(|(id, body)| format!("{}\n{body}\n", section_marker(id)))
        .collect()
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

// ── EX-7 / VT-1 — the generated round trip ────────────────────────────────

/// A **product** of hazard families, not a list. The families are the ones the
/// escaping answer claims to survive, and the product is what stops the row that
/// matters from being the row nobody thought of:
///
/// - **marker lookalikes** at every colon count the scheme distinguishes, plus
///   one whose id is illegal under the grammar (so it is *not* shaped, in both
///   directions) and one carrying trailing spaces (which a formatter would
///   normalise INTO a marker — the sharpest edge in the grammar);
/// - **indentation** 0…4 spaces, straddling CommonMark's three-space limit;
/// - **fence state**, because escaping is lexical and deliberately does not
///   consult Markdown block structure;
/// - **adversarial whitespace** at the body's end: none, trailing spaces, a
///   terminal blank line, and no terminal newline at all.
fn hazard_corpus() -> Vec<String> {
    const LOOKALIKES: [&str; 5] = [
        "<!-- doctrine:section sec-1 -->",
        "<!-- doctrine::section sec-1 -->",
        "<!-- doctrine:::section sec-2 -->",
        "<!-- doctrine:section not a valid id -->",
        "<!-- doctrine:section sec-1 -->   ",
    ];
    const INDENTS: [&str; 5] = ["", " ", "  ", "   ", "    "];
    const TRAILERS: [&str; 4] = ["\n", "  ", "\n\n", ""];

    let mut bodies = Vec::new();
    for lookalike in LOOKALIKES {
        for indent in INDENTS {
            for fenced in [false, true] {
                for trailer in TRAILERS {
                    let line = format!("{indent}{lookalike}");
                    let middle = if fenced {
                        format!("```\n{line}\n```")
                    } else {
                        line
                    };
                    bodies.push(format!("## Heading\n\nordinary prose\n{middle}{trailer}"));
                }
            }
        }
    }
    bodies
}

/// EX-7 — for every generated hazardous body, materialise-then-parse recovers
/// the identical section map.
///
/// The oracle is the `adopt --dry-run` report: its `unchanged` line carries
/// every declared id, which holds only if every marker resolved to its own
/// region and every body came back byte for byte (the held fingerprints ARE the
/// declared bodies' digests). Byte equality, never equality modulo whitespace —
/// a body that came back with whitespace moved reports a changed fingerprint.
#[test]
fn marker_round_trip_survives_lookalike_and_adversarial_bodies() {
    let corpus = hazard_corpus();
    assert!(
        corpus.len() > 100,
        "the product must actually be a product: only {} bodies",
        corpus.len()
    );

    let fixture = Fixture::start();
    let declared: Vec<(String, String)> = corpus
        .into_iter()
        .enumerate()
        .map(|(index, body)| (format!("sec-{}", index + 1), body))
        .collect();

    let declarations: Vec<Value> = declared
        .iter()
        .map(|(id, body)| json!({ "subject": id, "body": body }))
        .collect();
    fixture.apply("seed", &json!({ "declare": declarations }));
    fixture.materialise();

    let document = fixture.document();
    let report = fixture.parser_readout();
    assert!(
        !report.contains("section_fingerprint_changed"),
        "no body moved: {report}"
    );
    let expected: std::collections::BTreeSet<&str> =
        declared.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(
        common::unchanged_ids(&report),
        expected,
        "every declared body parsed back exactly: {report}"
    );

    // The document holds exactly the declared markers and no lookalike was
    // promoted into one — the other half of "the identical section map".
    assert_eq!(
        marker_order(&document).len(),
        declared.len(),
        "every marker in the document is a real section boundary"
    );
}

// ── EX-6 / VT-1 — document order is not id order ──────────────────────────

/// EX-6 — materialise emits sections in DOCUMENT order (`seq`), not in the id
/// order `SectionGroup` keeps for serialisation determinism.
///
/// The fixture is arranged so the two orders DIFFER: `sec-2` is declared first,
/// but `sec-11` sorts first lexically (`'1' < '2'`). A fixture where the orders
/// coincide passes against the defect — declaring `sec-10` under id ordering
/// silently moves existing prose, which is the whole harm.
#[test]
fn materialise_emits_document_order_not_id_order() {
    let fixture = Fixture::start();
    fixture.declare("first", "sec-2", "## Declared first\n\nprose\n");
    fixture.declare("second", "sec-11", "## Declared second\n\nprose\n");
    fixture.materialise();

    let document = fixture.document();
    assert_eq!(
        marker_order(&document),
        vec!["sec-2".to_owned(), "sec-11".to_owned()],
        "the document is in declaration order, not id order:\n{document}"
    );
    assert!(
        document.find(&section_marker("sec-2")) < document.find(&section_marker("sec-11")),
        "and the earlier-declared section's prose comes first"
    );
}

// ── EX-1/EX-5 / VT-1 — the crossing preserves the run ─────────────────────

/// EX-1 — a hand edit re-adopted into the SAME live run: the run uid, the
/// inquiry map, the cursor and the submission receipts all survive the crossing
/// (design.md:290), and the adopted PROSE reaches the snapshot.
///
/// The prose assertion is the one that reds at head: the crossing must seat the
/// document's parsed bodies, not merely record a digest for bytes the snapshot
/// does not hold — the same defect
/// `readopted_hand_edit_survives_a_subsequent_materialise` sees from the authored
/// side.
///
/// Fragment (prompt) receipts have no `apply` wire form in v1, so the receipt
/// ledger asserted here is the submission one plus the fragment group's own
/// value; asserting the group unchanged is what "preserves prompt receipts"
/// can honestly mean at this surface.
#[test]
fn hand_edit_then_readopt_preserves_run_map_cursor_receipts() {
    let fixture = Fixture::start();
    fixture.apply(
        "seed",
        &json!({
            "declare": [
                { "subject": "sec-1", "body": "## Draft\n\nas Doctrine wrote it\n" },
                { "subject": "inq-1", "question": "what is the boundary?" },
            ],
            "traversal": { "cursor": "inq-1" },
        }),
    );
    fixture.materialise();

    let before = fixture.read();
    let edited_body = "## Draft\n\na human rewrote this by hand\n";
    let document = framed(&[("sec-1", edited_body)]);
    fixture.write_doc(&document);

    fixture.adopt(&["--expect", &common::sha256(document.as_bytes())]);

    let after = fixture.read();
    assert_eq!(after.run.uid, before.run.uid, "the run uid is the same run");
    assert_eq!(
        after.map.inquiry, before.map.inquiry,
        "the inquiry map crossed intact"
    );
    assert_eq!(after.map.cursor, before.map.cursor, "and so did the cursor");
    assert_eq!(
        after.fragments, before.fragments,
        "and the prompt-fragment receipts"
    );
    for receipt in ["seed"] {
        assert!(
            after.receipts.find(receipt).is_some(),
            "receipt `{receipt}` survived the crossing"
        );
    }

    // And the crossing ADOPTED the prose, rather than recording a digest for
    // bytes the snapshot does not hold.
    assert_eq!(
        after.sections.find(&id("sec-1")).unwrap().body,
        edited_body,
        "the hand-edited body is what the run now holds"
    );
}

// ── EX-3 / VT-1 — the §5.5 refusals, each by its own reason ───────────────

/// EX-3 — a missing, a duplicated and an unknown marker are each refused BY
/// NAME, from three distinct documents.
///
/// Three documents and three assertions on purpose (VA-2): one `is_err()` over
/// three wrong inputs is one test wearing three names. Each fixture is its own
/// run, because a refusal is sticky by design and a shared run would let the
/// first refusal explain the second.
#[test]
fn readopt_refuses_missing_duplicate_unknown_markers() {
    let held = [
        ("sec-1", "## One\n\none's prose\n"),
        ("sec-2", "## Two\n\ntwo's prose\n"),
    ];

    // (a) MISSING — `sec-2`'s marker line is deleted and its prose kept, so the
    // orphan merges into `sec-1`'s region. The refusal names the CAUSE (the
    // absent marker), not the symptom (sec-1's moved fingerprint).
    let fixture = two_section_run(&held);
    let merged = "## One\n\none's prose\n## Two\n\ntwo's prose\n";
    let document = framed(&[("sec-1", merged)]);
    fixture.write_doc(&document);
    let error = fixture.refuse_adopt(&["--expect", &common::sha256(document.as_bytes())]);
    assert!(
        error.contains("sec-2 has no marker in the document"),
        "the missing marker is named by its own refusal: {error}"
    );

    // (b) DUPLICATE — one id marks two regions.
    let fixture = two_section_run(&held);
    let document = framed(&[
        ("sec-1", "## One\n\none's prose\n"),
        ("sec-2", "## Two\n\ntwo's prose\n"),
        ("sec-1", "## One again\n\npasted\n"),
    ]);
    fixture.write_doc(&document);
    let error = fixture.refuse_adopt(&["--expect", &common::sha256(document.as_bytes())]);
    assert!(
        error.contains("sec-1 is marked twice"),
        "the duplicated id is named by its own refusal: {error}"
    );

    // (c) UNKNOWN — a legal marker naming a section the run does not hold. The
    // document's marker set is compared against the run, so an invented marker
    // is refused rather than silently ignored.
    let fixture = two_section_run(&held);
    let document = framed(&[
        ("sec-1", "## One\n\none's prose\n"),
        ("sec-2", "## Two\n\ntwo's prose\n"),
        ("sec-9", "## Nine\n\ninvented in prose\n"),
    ]);
    fixture.write_doc(&document);
    let error = fixture.refuse_adopt(&["--expect", &common::sha256(document.as_bytes())]);
    assert!(
        error.contains("sec-9, which this run does not hold"),
        "the unknown marker is named by its own refusal: {error}"
    );
}

/// EX-3 — a marker-free addition and a structural deletion are each refused BY
/// NAME.
///
/// Split from the three above because these two classify the HEAD and a REGION
/// rather than the marker set, and because the advice differs: one says
/// "declare the new section through the run", the other says "restore the prose
/// you deleted".
#[test]
fn readopt_refuses_marker_free_addition_and_structural_deletion() {
    let held = [
        ("sec-1", "## One\n\none's prose\n"),
        ("sec-2", "## Two\n\ntwo's prose\n"),
    ];

    // (a) MARKER-FREE ADDITION — prose typed in before the first marker.
    let fixture = two_section_run(&held);
    let document = format!("A preamble nobody declared.\n\n{}", framed(&held));
    fixture.write_doc(&document);
    let error = fixture.refuse_adopt(&["--expect", &common::sha256(document.as_bytes())]);
    assert!(
        error.contains("before its first section marker"),
        "the addition is refused as an addition: {error}"
    );

    // A leading BLANK line is what a formatter produces, so it is not an
    // addition — the same document with whitespace in the head is adopted.
    let fixture = two_section_run(&held);
    let document = format!("  \n\n{}", framed(&held));
    fixture.write_doc(&document);
    fixture.adopt(&["--expect", &common::sha256(document.as_bytes())]);

    // (b) STRUCTURAL DELETION — a marker kept, its prose emptied.
    let fixture = two_section_run(&held);
    let document = format!(
        "{}\n{}",
        section_marker("sec-1"),
        framed(&[("sec-2", "## Two\n\ntwo's prose\n")])
    );
    fixture.write_doc(&document);
    let error = fixture.refuse_adopt(&["--expect", &common::sha256(document.as_bytes())]);
    assert!(
        error.contains("region is empty"),
        "the emptied section is refused as a deletion, not as a bad map: {error}"
    );
}

// ── EX-7 / VT-1 — document order is authoritative on re-adopt ─────────────

/// EX-7 — hand-REORDERING sections is a supported edit: re-adoption takes
/// document order as authoritative and renumbers `seq` to the marker sequence,
/// so the next materialise emits the order the human left behind.
///
/// The fixture separates document order from id order deliberately: `sec-2` is
/// declared FIRST and `sec-11` sorts first lexically (`'1' < '2'`), so the
/// reorder swaps document order WITHOUT coinciding with id order. The opposite
/// pairing makes the two orders agree and passes against the defect.
#[test]
fn readopt_takes_document_order_as_authoritative() {
    let fixture = Fixture::start();
    let two = "## Two\n\ntwo's prose\n";
    let eleven = "## Eleven\n\neleven's prose\n";
    fixture.declare("first", "sec-2", two);
    fixture.declare("second", "sec-11", eleven);
    fixture.materialise();
    assert_eq!(
        marker_order(&fixture.document()),
        vec!["sec-2".to_owned(), "sec-11".to_owned()],
        "the run starts in declaration order"
    );

    // The human swaps the two blocks and nothing else.
    let reordered = framed(&[("sec-11", eleven), ("sec-2", two)]);
    fixture.write_doc(&reordered);
    fixture.adopt(&["--expect", &common::sha256(reordered.as_bytes())]);

    // The run adopted the order, so re-rendering reproduces it rather than
    // reverting to the order the run happened to be declared in.
    fixture.materialise();
    assert_eq!(
        marker_order(&fixture.document()),
        vec!["sec-11".to_owned(), "sec-2".to_owned()],
        "document order is authoritative:\n{}",
        fixture.document()
    );
}

// ── shared fixtures ───────────────────────────────────────────────────────

/// A run holding `sections` in declaration order — declared, not yet rendered.
fn declared_run(sections: &[(&str, &str)]) -> Fixture {
    let fixture = Fixture::start();
    for (index, (id, body)) in sections.iter().enumerate() {
        fixture.declare(&format!("declare-{index}"), id, body);
    }
    fixture
}

/// A run holding `sections` in declaration order, materialised, watermark
/// baselined — the state a human then edits by hand.
fn two_section_run(sections: &[(&str, &str)]) -> Fixture {
    let fixture = declared_run(sections);
    fixture.materialise();
    fixture
}

/// Parse a run-local id in a test, where a malformed literal is a test bug.
fn id(raw: &str) -> design_run::ids::DesignId {
    design_run::ids::DesignId::parse(raw).expect("a well-formed run-local id")
}
