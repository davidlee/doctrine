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

use design_run::ids::DesignId;
use design_run::legacy;
use design_run::section;

/// The authored document each slice's design lives in.
const DESIGN_DOC: &str = "design.md";

/// Below this many designs read, the oracle cannot have swept the corpus — the
/// survey found 228, and a vacuous pass is the failure this guards (the same
/// positive-control discipline the absence criteria needed).
const CORPUS_FLOOR: usize = 200;

#[test]
fn every_authored_design_in_this_repo_imports_losslessly() {
    let root = common::repo_root().join(common::SLICE_DIR);
    let Ok(entries) = std::fs::read_dir(&root) else {
        return;
    };

    let id = DesignId::parse("sec-1").expect("a well-formed section id");
    let mut swept = 0_usize;
    let mut refused = Vec::new();
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
        let regions = match legacy::read(&text) {
            Ok(regions) => regions,
            Err(refusal) => {
                refused.push((path, refusal));
                continue;
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
            "{}: a real design has at least its `#` title",
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
        swept = swept.saturating_add(1);
    }

    assert!(
        swept >= CORPUS_FLOOR,
        "the oracle read only {swept} designs — below {CORPUS_FLOOR}, so it \
         cannot have swept the corpus and a pass would be vacuous"
    );
    assert!(
        refused.is_empty(),
        "import refuses a design this repo authored: {refused:?}"
    );
}
