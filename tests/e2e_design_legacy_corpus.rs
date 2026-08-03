// SPDX-License-Identifier: GPL-3.0-only
//! SL-233 PHASE-11 — the legacy import reader against every design this repo has
//! authored (the T2 regression oracle).
//!
//! # Why the corpus and not another fixture
//!
//! Import's input is *legacy prose*, so the only honest oracle is prose somebody
//! actually wrote. A synthetic fixture proves what its author already believed —
//! and on this reader that was measurably not enough: the naive fence rule the
//! a-priori argument produced passes every hand-written case and misreads two
//! real designs. The corpus is what caught it.
//!
//! # Why it lives here and not beside the reader
//!
//! `design_run` is a pure leaf — no clock, disk, git or rng is reachable from
//! it, and its inline tests keep that true. Reading the authored tree is the
//! shell's kind of work, so the oracle sits on the integration side, where
//! `common::repo_root()` resolves the tree at RUNTIME (CHR-014: `env!` bakes the
//! compiling tree's path into a binary another worktree may reuse).
//!
//! It refuses **nothing**, and that is the sharpest evidence for the phase's
//! fence rule. The survey grounding the reader reported two designs with an
//! unbalanced fence; under `CommonMark`'s rule both are well-formed, and the two
//! sections a parity reading would have swallowed (`## 17. Open Questions`,
//! `## Remaining open questions`) seat normally. The parity analysis was what
//! was malformed, not the documents.
//!
//! # Why the corpus is partitioned (ISS-311)
//!
//! The corpus is a *mixed* population, and it migrates. `design materialise`
//! writes marker-led documents, and import turns legacy prose into one, so a
//! `design.md`'s **path says nothing about which reader owns it** — only its
//! bytes do. Reading class off the path was true exactly until the slice that
//! wrote this test shipped its writer, and then refused the writer's own output
//! as an unheaded preamble.
//!
//! So each document is routed by its first non-blank line, and each half gets
//! the reader whose contract it satisfies: unmarked prose to [`legacy::read`],
//! marked to [`design_run::document::parse`] with the `render` that inverts it.
//! That makes the oracle total over the corpus rather than single-class — the
//! managed half previously had no corpus-level oracle at all, only the inline
//! round-trip over synthetic bodies.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

mod common;

/// The pure model, from source — the CHR-014 idiom this crate uses everywhere.
#[path = "../src/design_run/mod.rs"]
#[allow(
    dead_code,
    unused_imports,
    reason = "the whole leaf tree is included; no single test exercises all of it"
)]
mod design_run;

use design_run::document;
use design_run::ids::DesignId;
use design_run::legacy;
use design_run::refusal::Refusal;
use design_run::section;
use std::path::{Path, PathBuf};

/// The authored document each slice's design lives in.
const DESIGN_DOC: &str = "design.md";

/// Below this many designs read, the oracle cannot have swept the corpus — the
/// survey found 228, and a vacuous pass is the failure this guards (the same
/// positive-control discipline the absence criteria needed).
///
/// Stated over **both** halves. Stating it over the legacy half alone would rot:
/// import drives that count monotonically toward zero, so a correct oracle would
/// eventually red with this message for a reason that has nothing to do with a
/// reader (ISS-311).
const CORPUS_FLOOR: usize = 200;

/// Below this many managed documents, the round-trip half is vacuous.
///
/// One is enough and does not rot in the other direction: a design becomes
/// managed and stays managed, so the count only grows. If it ever reaches zero
/// the writer has stopped emitting markers, and redding is the right answer.
const MANAGED_FLOOR: usize = 1;

/// Every document the corpus refused, and which reader refused it.
type Refusals = Vec<(PathBuf, Refusal)>;

/// Whether the document is the managed writer's output rather than legacy prose.
///
/// Decided on the first **non-blank** line, because that is where
/// [`design_run::document::parse`] itself draws the line: `render` emits a marker
/// at column 0, and a leading blank line is what a formatter produces, so
/// tolerating one here and refusing it there would disagree about the same
/// document.
fn is_managed(text: &str) -> bool {
    text.lines()
        .find(|line| !line.trim().is_empty())
        .is_some_and(|line| document::marker(line).is_some())
}

/// Import one legacy document: it decomposes, the regions are the document minus
/// a blank head, and every region opens with a heading that names a title.
fn sweep_legacy(path: &Path, text: &str, refused: &mut Refusals) -> bool {
    let id = DesignId::parse("sec-1").expect("a well-formed section id");
    let regions = match legacy::read(text) {
        Ok(regions) => regions,
        Err(refusal) => {
            refused.push((path.to_owned(), refusal));
            return false;
        }
    };

    let body: String = regions.iter().map(|region| region.body).collect();
    let head = text.get(..text.len().saturating_sub(body.len()));
    assert!(
        text.ends_with(&body) && head.unwrap_or_default().trim().is_empty(),
        "{}: the regions are not the document minus a blank head",
        path.display()
    );
    assert!(
        !regions.is_empty(),
        "{}: a real design has at least one titled heading",
        path.display()
    );
    for region in &regions {
        assert!(
            section::derive_title(&id, region.body).is_ok(),
            "{}: the region at line {} is not title-bearing",
            path.display(),
            region.line
        );
    }
    true
}

/// Read one managed document: it parses, `render` inverts that parse byte-exactly,
/// and every section it carries is title-bearing — the same three claims the
/// legacy half makes, through the reader that owns marked documents.
fn sweep_managed(path: &Path, text: &str, refused: &mut Refusals) -> bool {
    let sections = match document::parse(text, None) {
        Ok(sections) => sections,
        Err(refusal) => {
            refused.push((path.to_owned(), refusal));
            return false;
        }
    };

    assert!(
        !sections.is_empty(),
        "{}: a marked document carries at least one section",
        path.display()
    );
    let rendered = document::render(
        sections
            .iter()
            .map(|section| (&section.id, section.body.as_str())),
    );
    assert_eq!(
        rendered,
        text,
        "{}: render does not invert the parse of the authored document",
        path.display()
    );
    for section in &sections {
        assert!(
            section::derive_title(&section.id, &section.body).is_ok(),
            "{}: section {} is not title-bearing",
            path.display(),
            section.id.as_str()
        );
    }
    true
}

#[test]
fn every_authored_design_in_this_repo_reads_losslessly() {
    let root = common::repo_root().join(common::SLICE_DIR);
    let Ok(entries) = std::fs::read_dir(&root) else {
        return;
    };

    let mut legacy_swept = 0_usize;
    let mut managed_swept = 0_usize;
    let mut refused = Refusals::new();
    for entry in entries.flatten() {
        // Slug symlinks double-count every slice — the raw glob returns 453
        // paths for 228 real designs.
        if entry.file_type().is_ok_and(|kind| kind.is_symlink()) {
            continue;
        }
        let path = entry.path().join(DESIGN_DOC);
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let (swept, read) = if is_managed(&text) {
            (
                &mut managed_swept,
                sweep_managed(&path, &text, &mut refused),
            )
        } else {
            (&mut legacy_swept, sweep_legacy(&path, &text, &mut refused))
        };
        if read {
            *swept = swept.saturating_add(1);
        }
    }

    let swept = legacy_swept.saturating_add(managed_swept);
    assert!(
        swept >= CORPUS_FLOOR,
        "the oracle read only {swept} designs ({legacy_swept} legacy, \
         {managed_swept} managed) — below {CORPUS_FLOOR}, so it cannot have swept \
         the corpus and a pass would be vacuous"
    );
    assert!(
        managed_swept >= MANAGED_FLOOR,
        "the oracle read {managed_swept} managed designs — below {MANAGED_FLOOR}, \
         so the round-trip half is vacuous and either the writer stopped emitting \
         markers or the routing misclassified the documents it wrote"
    );
    assert!(
        refused.is_empty(),
        "a reader refuses a design this repo authored: {refused:?}"
    );
}
