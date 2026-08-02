// SPDX-License-Identifier: GPL-3.0-only
//! The closed process-fragment catalogue (SL-233 PHASE-07, DEC-077).
//!
//! Four coarse process fragments ship beneath `install/design-prompts/`. This
//! module is the *whole* selection mechanism: one closed enum, one variant per
//! file, at most one file chosen. Deliberately **not** a second prompt cascade —
//! it forgoes selector algebra, `replaces` validation, seal integrity and user
//! overrides, all of which the hymn cascade has and this store does not want
//! (design.md §7). A fragment's bytes are bound by digest at emission instead.
//!
//! Leaf tier, out-degree zero: this module names an asset *key* and never reads
//! it. The shell resolves the key against the embedded corpus and digests the
//! bytes, because a leaf may not touch the filesystem (AGENTS.md pure/imperative
//! split).

use super::Stage;

/// Where the fragment store lives inside the `install/` embed root. Single
/// source (STD-001) — the four keys derive from it rather than repeating it.
///
/// `pub(super)` because [`super::runbook`] shares it: a runbook is the
/// structured sibling of the prose it guards, in the same store, so spelling
/// the directory twice would be two sources for one address.
pub(super) const STORE: &str = "design-prompts";

/// Binds a fragment's name to the digest of the bytes a caller holds, in both
/// directions: the `name@digest` header `design resume` emits and the receipt a
/// caller declares back. One separator, one source (STD-001).
const RECEIPT_SEPARATOR: char = '@';

/// One coarse process fragment: an intra-design obligation, not a lifecycle
/// stage. The distinction is why these are a closed store rather than four
/// additional `stage/*` hymns — `KNOWN_STAGE_LABELS` is an *enforced lifecycle*
/// vocabulary, and inquiry/drafting/reviewing/delegation are obligations within
/// the design stage (design.md §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Fragment {
    Inquiry,
    Drafting,
    Reviewing,
    Delegation,
}

impl Fragment {
    /// Every fragment, single-sourced so an exhaustive table test cannot
    /// silently miss a new variant (STD-001, the `Stage::ALL` precedent).
    pub(crate) const ALL: [Fragment; 4] = [
        Fragment::Inquiry,
        Fragment::Drafting,
        Fragment::Reviewing,
        Fragment::Delegation,
    ];

    /// The fragment's stable name — what a receipt identifies it by, and the
    /// stem of its file. Identity, so it is bounded at admission and never
    /// elided at emission (the layer rule).
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Fragment::Inquiry => "inquiry",
            Fragment::Drafting => "drafting",
            Fragment::Reviewing => "reviewing",
            Fragment::Delegation => "delegation",
        }
    }

    /// The embedded asset key the shell resolves. One variant, one file — the
    /// "at most one file" bound is structural here, not a runtime check.
    pub(crate) fn asset_key(self) -> String {
        format!("{STORE}/{}.md", self.name())
    }

    /// The fragment a name identifies, if any. Closed vocabulary: an unknown
    /// name is `None`, never a fallback to some default fragment.
    pub(crate) fn parse(name: &str) -> Option<Self> {
        Fragment::ALL
            .into_iter()
            .find(|fragment| fragment.name() == name)
    }

    /// This fragment bound to `digest` — the exact form [`Fragment::parse_receipt`]
    /// accepts, so emission and admission cannot drift apart. The separator is
    /// spelled once, here and in the parser, and nowhere in the shell.
    pub(crate) fn receipt(self, digest: &str) -> String {
        format!("{}{RECEIPT_SEPARATOR}{digest}", self.name())
    }

    /// The fragment and held digest a receipt declaration names, if it is one.
    ///
    /// The wire form is `name@digest` — the same string `design resume` emits, so
    /// a caller echoes back what it was given rather than composing a second
    /// grammar (STD-001). Closed like [`Fragment::parse`]: an unknown name is
    /// `None`, and so is a bare name with no digest, because a receipt that binds
    /// no bytes is not a receipt.
    pub(crate) fn parse_receipt(declared: &str) -> Option<(Self, &str)> {
        let (name, digest) = declared.split_once(RECEIPT_SEPARATOR)?;
        let fragment = Fragment::parse(name)?;
        (!digest.is_empty()).then_some((fragment, digest))
    }

    /// The obligation a run's stage implies, when the stage implies one.
    ///
    /// Exhaustive over [`Stage`] by construction. `Locked` yields `None` —
    /// a locked run has no next obligation, and emitting process guidance for
    /// one would be inventing work. `Delegation` is deliberately unreachable
    /// here: delegation is a *separate* state model from the stage machine
    /// (DEC-065), so it is selected by delegation state rather than by stage.
    pub(crate) const fn for_stage(stage: Stage) -> Option<Self> {
        match stage {
            // Exploring and inquiring are both question-shaping work.
            Stage::Exploring | Stage::Inquiring => Some(Fragment::Inquiry),
            Stage::Drafting => Some(Fragment::Drafting),
            Stage::Reviewing => Some(Fragment::Reviewing),
            Stage::Locked => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_fragment_has_a_distinct_name_and_asset_key_under_the_store() {
        let mut names: Vec<&str> = Fragment::ALL.iter().map(|f| f.name()).collect();
        let count = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), count, "fragment names are distinct");

        for fragment in Fragment::ALL {
            assert_eq!(
                fragment.asset_key(),
                format!("design-prompts/{}.md", fragment.name()),
                "asset key sits under the store and is derived from the name"
            );
        }
    }

    #[test]
    fn a_name_round_trips_and_an_unknown_name_selects_nothing() {
        for fragment in Fragment::ALL {
            assert_eq!(
                Fragment::parse(fragment.name()),
                Some(fragment),
                "{} round-trips",
                fragment.name()
            );
        }
        // Closed vocabulary: no fallback, no fuzzy match.
        assert_eq!(Fragment::parse("Inquiry"), None, "matching is exact");
        assert_eq!(
            Fragment::parse("inquiry.md"),
            None,
            "the name is not the file"
        );
        assert_eq!(Fragment::parse(""), None);
        assert_eq!(Fragment::parse("nonesuch"), None);
    }

    #[test]
    fn every_emitted_receipt_round_trips_through_the_parser() {
        // The property that matters: what emission writes, admission reads back —
        // for every variant, so a new fragment cannot ship an unparseable receipt.
        for fragment in Fragment::ALL {
            let emitted = fragment.receipt("abc123");
            assert_eq!(
                Fragment::parse_receipt(&emitted),
                Some((fragment, "abc123")),
                "{emitted} round-trips"
            );
        }
    }

    #[test]
    fn a_receipt_binds_a_known_name_to_a_nonempty_digest_and_nothing_else() {
        assert_eq!(
            Fragment::parse_receipt("inquiry@abc123"),
            Some((Fragment::Inquiry, "abc123")),
            "the wire form is name@digest"
        );
        // A digest is opaque to this grammar — only the first separator splits,
        // so a digest is never silently truncated.
        assert_eq!(
            Fragment::parse_receipt("drafting@a@b"),
            Some((Fragment::Drafting, "a@b"))
        );

        // A receipt that binds no bytes is not a receipt: the bare name is the
        // OLD `--known-fragment` spelling and must not read as a current hold.
        assert_eq!(
            Fragment::parse_receipt("inquiry"),
            None,
            "no digest, no bind"
        );
        assert_eq!(Fragment::parse_receipt("inquiry@"), None, "empty digest");
        assert_eq!(Fragment::parse_receipt("@abc123"), None, "no name");
        // Closed vocabulary, as `parse` — an unknown name never binds.
        assert_eq!(Fragment::parse_receipt("nonesuch@abc123"), None);
        assert_eq!(
            Fragment::parse_receipt("Inquiry@abc123"),
            None,
            "exact match"
        );
        assert_eq!(Fragment::parse_receipt(""), None);
    }

    #[test]
    fn stage_selects_at_most_one_fragment_and_locked_selects_none() {
        // Exhaustive over the closed stage vocabulary: a new Stage variant
        // fails to compile in `for_stage` rather than silently selecting None.
        let selected: Vec<(Stage, Option<Fragment>)> = Stage::ALL
            .into_iter()
            .map(|stage| (stage, Fragment::for_stage(stage)))
            .collect();

        assert_eq!(
            selected,
            vec![
                (Stage::Exploring, Some(Fragment::Inquiry)),
                (Stage::Inquiring, Some(Fragment::Inquiry)),
                (Stage::Drafting, Some(Fragment::Drafting)),
                (Stage::Reviewing, Some(Fragment::Reviewing)),
                (Stage::Locked, None),
            ],
            "the stage→obligation mapping is closed and total"
        );
    }
}
