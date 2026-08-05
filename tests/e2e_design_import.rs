// SPDX-License-Identifier: GPL-3.0-only
//! SL-233 PHASE-11 — conservative import and semantic reconstruction, over the
//! built binary (DEC-057/DEC-084/DEC-085, design §5.4-5.5).
//!
//! The five test names are PHASE-11 `EX-7`'s contract and are immutable. Extra
//! coverage belonged in `tests/e2e_design_state.rs`, not here — with ONE later
//! exception, recorded because a header that misstates its own file is the
//! defect PHASE-15 F-6 was about.
//!
//! PHASE-15 `VT-4` names `tests/e2e_design_import.rs` explicitly as the home of
//! `an_imported_node_retains_its_source_label_and_fingerprint` (`EX-9`), and it
//! is right to: the criterion is about what import seats, and the legacy
//! fixture it needs lives here. Duplicating `LEGACY_DESIGN` into another suite
//! to honour the word "EXACTLY" in a landed phase's criterion would be parallel
//! implementation. `EX-7`'s five names are unchanged, so its substance — those
//! names are the contract — holds; this file now carries six tests, not five.
//!
//! # Why the fixture looks the way it does
//!
//! Import's input is a *legacy* `design.md`: no markers, so PHASE-13's
//! `document::parse` cannot read it (its row 2, `MarkerFreeAddition`, fires on
//! the whole document). This suite's fixture is therefore shaped by the corpus
//! survey recorded in the slice's `notes.md` § *Learned* — 228 real designs —
//! rather than by what a synthetic three-section document would make convenient:
//!
//! - **front matter between the `#` title and the first `##`** (an HTML comment
//!   and a status blockquote). Universal in the corpus, and the reason import
//!   splits at every ATX heading rather than at `##` alone.
//! - **a heading-like line inside a fenced block.** 33 of 228 real designs
//!   (14.5%) carry one; a fence-blind splitter shreds them.
//! - **a `##` immediately followed by a `###`**, seating a legal but
//!   contentless section. 87% of the corpus has at least one.
//! - **an `OQ-*` outside the Open Questions section**, which `EX-3` says is
//!   ordinary prose and must not become a node.
//! - **two byte-identical question texts**, which `EX-4` says never merge
//!   because only an explicit canonical citation does.
//!
//! Two assertions read the persisted snapshot as **text**, and both are about
//! what landed on disk — the `imported-prose` label crossing the wire, and the
//! caller-facing wording of a refusal. Everything about the *model* is asserted
//! through the model (`InquiryMap::nodes`, `Section::source_line`), because the
//! section bodies are stored verbatim: `OQ-9` and `QUE-177` are in the snapshot
//! text whatever import does, so a string search there cannot tell "is not a
//! node" from "is not in the file", and only the first is a criterion.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::{Path, PathBuf};
use std::process::Output;

mod common;

/// The pure model, from source — the CHR-014 idiom this crate uses everywhere.
#[path = "../src/design_run/mod.rs"]
#[allow(
    dead_code,
    unused_imports,
    reason = "the whole leaf tree is included; no single test exercises all of it"
)]
mod design_run;

use design_run::Stage;
use design_run::inquiry::Provenance;
use design_run::snapshot::{self, DesignSnapshot};

/// The slice every fixture designs.
const SLICE: &str = "SL-233";
/// Its zero-padded directory name.
const SLICE_NUMBER: &str = "233";
/// The authored document import reads, mirroring `slice.rs`'s `DESIGN_DOC`.
const DESIGN_DOC: &str = "design.md";

/// A legacy `design.md`: no markers, and every shape the corpus survey found
/// that a naive reader gets wrong. Byte-for-byte meaningful — do not reflow.
const LEGACY_DESIGN: &str = "\
# Design SL-233: Import fixture

<!-- Front matter: universal in the corpus, and unheaded. Splitting at `##`
     alone would orphan it. -->

> **Status: locked.** Accepted by the user on 2026-01-01, with attestations.

## 1. Design Problem

The problem this fixture states. It cites QUE-177 explicitly, which is the only
thing `EX-4` lets merge an imported node with an existing record.

## 2. Proposed Design

### 2.1 A subsection immediately below its parent

Its parent seats a legal, contentless section. Body prose mentions OQ-9 here,
outside the Open Questions section, where `EX-3` says it stays ordinary prose.

```text
## Not a heading — this line is inside a fenced block
```

## 3. Open Questions & Unknowns

- **OQ-1:** whether the reader tracks fenced code blocks.
- **OQ-2:** whether the reader tracks fenced code blocks.
";

/// Every ATX heading in [`LEGACY_DESIGN`] that must seat a section, in document
/// order. The fenced `## Not a heading` line is deliberately absent.
const EXPECTED_TITLES: [&str; 5] = [
    "Design SL-233: Import fixture",
    "1. Design Problem",
    "2. Proposed Design",
    "2.1 A subsection immediately below its parent",
    "3. Open Questions & Unknowns",
];

// ── fixture ───────────────────────────────────────────────────────────────

/// A throwaway tree carrying an authored legacy `design.md` and no run.
struct Fixture {
    _tmp: tempfile::TempDir,
    root: PathBuf,
}

impl Fixture {
    /// Plant the slice tree and the legacy document. No run is started.
    fn legacy() -> Fixture {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let slice_dir = root.join(common::SLICE_DIR).join(SLICE_NUMBER);
        std::fs::create_dir_all(&slice_dir).unwrap();
        std::fs::write(slice_dir.join(DESIGN_DOC), LEGACY_DESIGN).unwrap();
        Fixture { _tmp: tmp, root }
    }

    /// `design start --from-design`, expecting success; returns stdout.
    fn import(&self) -> String {
        ok(self.run_import())
    }

    fn run_import(&self) -> Output {
        spawn(
            &self.root,
            &["design", "start", SLICE, "--from-design", "-p", "."],
        )
    }

    /// The snapshot path, learned from `design start`'s own output so no test
    /// re-types a state path (STD-001).
    fn snapshot_path(&self, stdout: &str) -> PathBuf {
        stdout
            .lines()
            .find_map(|line| line.strip_prefix("snapshot "))
            .map(|path| self.root.join(path))
            .expect("`design start` names the snapshot path")
    }

    /// The run uid, from the same output.
    fn uid(stdout: &str) -> String {
        stdout
            .split_whitespace()
            .nth(1)
            .expect("`design start` names the run uid")
            .to_owned()
    }

    /// Import, then return the parsed snapshot and its raw text.
    fn imported(&self) -> (DesignSnapshot, String) {
        let stdout = self.import();
        let path = self.snapshot_path(&stdout);
        let text = std::fs::read_to_string(&path).expect("the snapshot was written");
        (snapshot::parse(&text).expect("the snapshot parses"), text)
    }
}

// ── the five behaviours ───────────────────────────────────────────────────

/// `EX-2` — every heading seats a section carrying a fingerprint, and the run
/// holds them as unreviewed prose.
#[test]
fn from_design_imports_sections_as_unreviewed_prose_with_fingerprint() {
    let fixture = Fixture::legacy();
    let (run, _) = fixture.imported();

    let titles: Vec<&str> = run
        .sections
        .document_order()
        .into_iter()
        .map(|section| section.title.as_str())
        .collect();
    assert_eq!(
        titles, EXPECTED_TITLES,
        "import seats one section per ATX heading, in document order, and never \
         splits inside a fenced block"
    );

    let sources: Vec<Option<usize>> = run
        .sections
        .document_order()
        .into_iter()
        .map(|section| section.source_line)
        .collect();
    assert_eq!(
        sources,
        [Some(1), Some(8), Some(13), Some(15), Some(24)],
        "each imported section carries the source location its heading stands \
         on, not merely its digest (EX-2)"
    );

    let fingerprints: std::collections::BTreeSet<&str> = run
        .sections
        .document_order()
        .into_iter()
        .map(|section| section.fingerprint.as_str())
        .collect();
    assert!(
        fingerprints.iter().all(|digest| !digest.is_empty()),
        "every imported section carries a fingerprint (EX-2)"
    );
    assert_eq!(
        fingerprints.len(),
        EXPECTED_TITLES.len(),
        "the fingerprints are distinct — bound to each section's own content, \
         not a shared constant (DEC-066)"
    );
    assert!(
        run.review.attestations.is_empty(),
        "imported sections hold no review clearance (EX-2): {:?}",
        run.review.attestations
    );
}

/// `EX-3` — `OQ-*` is recognised only inside the Open Questions section.
#[test]
fn oq_entries_enter_only_from_the_open_questions_section() {
    let fixture = Fixture::legacy();
    let (run, text) = fixture.imported();

    assert_eq!(
        run.map.inquiry.len(),
        2,
        "exactly the two Open Questions entries seed nodes — OQ-9 in §2.1 is \
         ordinary prose (EX-3)"
    );
    assert!(
        text.contains("imported-prose"),
        "a conventional OQ entry enters as an unverified imported-prose \
         proposal (EX-3, DEC-085)"
    );

    // WHERE each node came from, not whether a string appears. The section
    // bodies are stored verbatim, so `OQ-9` IS in the snapshot text — it is in
    // §2.1's prose, which is exactly where it belongs. A text search cannot tell
    // "OQ-9 is not a node" from "OQ-9 is not in the file", and only one of those
    // is EX-3.
    let sources: Vec<(&str, u32)> = run
        .map
        .inquiry
        .nodes()
        .map(|node| match node.provenance() {
            Provenance::ImportedProse { section, line, .. } => (section.as_str(), *line),
            other => panic!("an imported OQ entry carries imported-prose: {other:?}"),
        })
        .collect();
    assert_eq!(
        sources,
        vec![("sec-5", 26), ("sec-5", 27)],
        "the two nodes are the two entries of §3 Open Questions (lines 26 and \
         27, section sec-5). OQ-9 stands at line 17 in §2.1 and is not among \
         them — it is ordinary body prose (EX-3)"
    );
}

/// `EX-4` — identical text never merges; only an explicit citation does.
#[test]
fn text_similarity_never_merges_only_explicit_citation_does() {
    let fixture = Fixture::legacy();
    let (run, _) = fixture.imported();

    let questions: Vec<&str> = run
        .map
        .inquiry
        .nodes()
        .map(design_run::inquiry::InquiryNode::question)
        .collect();
    assert_eq!(
        questions.len(),
        2,
        "OQ-1 and OQ-2 remain two distinct nodes (EX-4, DEC-085)"
    );
    assert_eq!(
        questions.first(),
        questions.last(),
        "…and their question texts are byte-identical, which is precisely what \
         a text comparison would have collapsed"
    );

    // The other half of EX-4, as a boundary rather than as a string search: the
    // snapshot stores §1's prose verbatim, so `QUE-177` is in the file no matter
    // what import does, and asserting its presence asserts nothing. What is
    // testable is that the citation seeded NOTHING: a canonical id in prose is
    // not a merge, because §1 is not the Open Questions section and this tree
    // holds no knowledge corpus for a shaping QUE to come from.
    assert!(
        run.map
            .inquiry
            .nodes()
            .all(|node| !matches!(node.provenance(), Provenance::ShapingQuestion { .. })),
        "a QUE citation in ordinary prose never manufactures a durable node — \
         only a real linked non-terminal shaping QUE does (EX-3/EX-4)"
    );
}

/// `EX-6` — reconstruction after runtime loss is a NEW run, said plainly.
#[test]
fn reconstruction_after_snapshot_loss_issues_a_new_run_uid() {
    let fixture = Fixture::legacy();
    let first = fixture.import();
    let first_uid = Fixture::uid(&first);
    let path = fixture.snapshot_path(&first);

    // Runtime loss: the snapshot tier is gitignored and disposable by design.
    std::fs::remove_file(&path).expect("the snapshot is removable");

    // EX-5: plain resume must refuse rather than reconstruct missing history.
    let resumed = spawn(&fixture.root, &["design", "resume", SLICE, "-p", "."]);
    assert!(
        !resumed.status.success(),
        "plain resume never reconstructs after runtime loss (EX-5, DEC-084)"
    );
    assert!(
        String::from_utf8_lossy(&resumed.stderr).contains("--from-design"),
        "the refusal names the lawful path instead of leaving the caller stuck"
    );

    let second = fixture.import();
    assert_ne!(
        Fixture::uid(&second),
        first_uid,
        "reconstruction issues a new run uid (EX-6, DEC-057)"
    );
    let lowered = second.to_lowercase();
    assert!(
        lowered.contains("reconstruct"),
        "the distinction from exact resume is surfaced to the caller, not \
         buried (EX-6): {second}"
    );
}

/// `EX-5` / `VA-4` — the three collections that would encode procedural history,
/// enumerated. Asserting one and leaving two free is the failure this guards.
#[test]
fn import_manufactures_no_attestation_receipt_or_gate_clearance() {
    let fixture = Fixture::legacy();
    let (run, _) = fixture.imported();

    // Non-vacuity first: absence proves nothing if nothing was imported. The
    // fixture's prose says "Status: locked" and "with attestations" precisely so
    // a reader that believes documents about themselves is caught here.
    assert_eq!(
        run.sections.document_order().len(),
        EXPECTED_TITLES.len(),
        "the import actually happened — otherwise every assertion below is vacuous"
    );

    assert!(
        run.review.attestations.is_empty(),
        "import manufactures no attestation: {:?}",
        run.review.attestations
    );
    assert!(
        run.receipts.receipts.is_empty(),
        "import manufactures no submission receipt: {:?}",
        run.receipts.receipts
    );
    assert!(
        run.acts.acts.is_empty() && run.declarations.declarations.is_empty(),
        "import manufactures no recorded act: {:?} / {:?}",
        run.acts.acts,
        run.declarations.declarations
    );
    assert_eq!(
        run.run.stage,
        Stage::Exploring,
        "an imported run starts at the first stage; prose claiming `locked` \
         does not advance it (DEC-088)"
    );
}

// ── PHASE-15 EX-9 / VT-4 ──────────────────────────────────────────────────

/// `EX-9` — an `imported-prose` node retains DEC-085's whole triple: source
/// **label**, source **location**, and content **fingerprint**.
///
/// Asserted on the NODE, which is the gap RV-324 F-4 named.
/// `from_design_imports_sections_as_unreviewed_prose_with_fingerprint` asserts
/// fingerprints on SECTIONS and `oq_entries_enter_only_from_the_open_questions_section`
/// asserts only `(section, line)` on nodes, so a node could carry the section's
/// digest, a constant, or nothing at all and both would stay green.
///
/// The fingerprint's extent is the **headline** — the exact bytes stored as the
/// node's question (D8). Not the continuation lines, which the importer
/// deliberately leaves in the section body, and not the entry text past the
/// label, which in the bolded form carries same-line prose after the headline.
#[test]
fn an_imported_node_retains_its_source_label_and_fingerprint() {
    let fixture = Fixture::legacy();
    let (run, _) = fixture.imported();

    let imported: Vec<(&str, u32, &str, &str)> = run
        .map
        .inquiry
        .nodes()
        .map(|node| match node.provenance() {
            Provenance::ImportedProse {
                section,
                line,
                label,
                fingerprint,
            } => (
                section.as_str(),
                *line,
                label.as_str(),
                fingerprint.as_str(),
            ),
            other => panic!("an imported OQ entry carries imported-prose: {other:?}"),
        })
        .collect();

    let labels: Vec<&str> = imported.iter().map(|(_, _, label, _)| *label).collect();
    assert_eq!(
        labels,
        vec!["OQ-1", "OQ-2"],
        "each node keeps the label its entry was written under. `entry_text` \
         strips the prefix and the id to find the question, so without this the \
         label is dropped as surely as the fingerprint was — and the label is \
         the ONLY thing that tells these two nodes apart in the source document \
         (EX-9, DEC-085)"
    );

    let digests: Vec<&str> = imported
        .iter()
        .map(|(_, _, _, fingerprint)| *fingerprint)
        .collect();
    assert!(
        digests.iter().all(|digest| !digest.is_empty()),
        "every imported node carries a content fingerprint, not an empty \
         placeholder: {digests:?}"
    );

    // ANTI-THEATRE. A node could satisfy "carries a non-empty fingerprint" by
    // copying the digest of the section it was found in — which would be a
    // fingerprint of the wrong thing, and unfalsifiable by the assertion above.
    // The section digests are the positive control: they exist, they are
    // non-empty, and the node's must be none of them.
    let section_digests: std::collections::BTreeSet<&str> = run
        .sections
        .document_order()
        .into_iter()
        .map(|section| section.fingerprint.as_str())
        .collect();
    assert!(
        !section_digests.is_empty(),
        "positive control: the sections carry digests to be confused with"
    );
    assert!(
        digests
            .iter()
            .all(|digest| !section_digests.contains(digest)),
        "a node's fingerprint is of its OWN headline, not of the section it was \
         found in: {digests:?} vs sections {section_digests:?}"
    );

    // The fixture's two entries are byte-identical past their labels — the
    // property `text_similarity_never_merges_only_explicit_citation_does`
    // exists to exercise. So their headline digests MUST be equal: a
    // fingerprint that differed here would be hashing something other than the
    // headline (the label, or the line number), which is the defect this
    // criterion closes in the other direction.
    assert_eq!(
        digests.first(),
        digests.last(),
        "byte-identical headlines digest identically — the fingerprint is of \
         the headline alone, and the LABEL is what keeps the two nodes distinct"
    );

    // THE EXTENT, which is the whole of D8 and cannot be read off a structural
    // assertion. `**OQ-1:** whether the reader…` has a headline of
    // `whether the reader…` and an entry-text-past-the-label of
    // `:** whether the reader…`, so the two hash differently and only one of
    // them is what DEC-085 asks to be fingerprinted. D4 defaulted to the
    // second; D8 settled on the first, because the supported bold form carries
    // same-line prose after the headline that is NOT the question.
    let headline = "whether the reader tracks fenced code blocks.";
    assert_eq!(
        digests.first().copied(),
        Some(common::sha256(headline.as_bytes()).as_str()),
        "the fingerprint digests the headline — the exact bytes stored as the \
         node's question (D8)"
    );
    assert!(
        !digests.contains(&common::sha256(format!(":** {headline}").as_bytes()).as_str()),
        "…and NOT the entry text past the label, which is D4's superseded \
         default and is a strictly wider extent"
    );
    let questions: Vec<&str> = run
        .map
        .inquiry
        .nodes()
        .map(design_run::inquiry::InquiryNode::question)
        .collect();
    assert_eq!(
        questions,
        vec![headline, headline],
        "positive control on the literal above: it IS what the nodes store, so \
         the digest assertion is about the extent rather than about a string \
         this test invented"
    );

    let located: Vec<(&str, u32)> = imported
        .iter()
        .map(|(section, line, _, _)| (*section, *line))
        .collect();
    assert_eq!(
        located,
        vec![("sec-5", 26), ("sec-5", 27)],
        "the location half of the triple is unchanged by this addition"
    );
}

// ── helpers ───────────────────────────────────────────────────────────────

fn spawn(root: &Path, args: &[&str]) -> Output {
    common::doctrine_cmd(root)
        .args(args)
        .env_remove("DOCTRINE_DESIGN_FAULT")
        // A fixture repo has no remote, so the reservation reach degrades to
        // local. Declaring the opt-in keeps that a decision rather than a prompt.
        .env("DOCTRINE_RESERVATION_FALLBACK", "1")
        .output()
        .expect("spawn doctrine")
}

/// The digest form `git::sha256` writes — bare lowercase hex.
///
/// Assert success and return stdout.
fn ok(out: Output) -> String {
    assert!(
        out.status.success(),
        "doctrine failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}
