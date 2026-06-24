// SPDX-License-Identifier: GPL-3.0-only
//! `conformance` — the pure set-algebra core (ADR-001 leaf, SL-147 design D6).
//! Given the slice's `design-target` selectors and an `actual` map of touched
//! paths (each carrying its ordered per-phase A|M|D events, folded over the
//! recorded registry's git name-status diffs), it partitions the touched paths
//! into three cells: `conformant` (matched by a design-target selector — each
//! carrying the selector that matched it, F-7 transparency), `undeclared` (no
//! selector matched — each carrying its `net()` display verb, the highest-signal
//! cell), and `undelivered` (declared selectors that no actual path matched).
//!
//! Pure: std + the shared `globmatch` leaf only. Git, the registry, and selector
//! reads live in the conformance shell (`crate::slice`); the `actual` map is
//! built there and passed in.

use std::collections::{BTreeMap, BTreeSet};

use glob::Pattern;

use crate::globmatch::glob_matches;

/// A single git name-status event for a path within one phase's diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Status {
    Added,
    Modified,
    Deleted,
}

/// The displayed verb for a path, derived from its ordered event set by [`net`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Verb {
    Added,
    Modified,
    Removed,
}

impl Verb {
    /// One-character marker for rendering (`A`/`M`/`D`).
    pub(crate) fn marker(self) -> char {
        match self {
            Verb::Added => 'A',
            Verb::Modified => 'M',
            Verb::Removed => 'D',
        }
    }
}

/// Derive the display verb from a path's ordered event set (design D6, F-3).
/// A path may be touched across phases (A then M, or A then D); the fold keeps
/// the ordered events and this pure rule collapses them for display only:
/// a trailing `Deleted` ⇒ `Removed`; otherwise contains `Added` ⇒ `Added`;
/// otherwise `Modified`. An empty event set is `Modified` (presence with no
/// classifiable event — never reached in practice, but total).
pub(crate) fn net(events: &[Status]) -> Verb {
    match events.last() {
        Some(Status::Deleted) => Verb::Removed,
        _ if events.contains(&Status::Added) => Verb::Added,
        _ => Verb::Modified,
    }
}

/// One conformant path with the design-target selector string that matched it
/// (F-7: a broad declaration is visible in the output).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Conformant {
    pub(crate) path: String,
    pub(crate) matched_selector: String,
}

/// One undeclared path with its `net()` display verb (highest-signal cell).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Undeclared {
    pub(crate) path: String,
    pub(crate) verb: Verb,
}

/// The three-cell partition of the touched paths against the design-target
/// selectors (design D6).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Conformance {
    /// Touched paths matched by some design-target selector — each carrying the
    /// matched selector string.
    pub(crate) conformant: Vec<Conformant>,
    /// Touched paths matched by no selector — each carrying its `net()` verb.
    pub(crate) undeclared: Vec<Undeclared>,
    /// Design-target selectors that matched no touched path.
    pub(crate) undelivered: Vec<String>,
}

/// Partition `actual` (the folded per-path event map) against `selectors` (the
/// design-target selector strings) per design D6. Pure; the `actual` map is
/// built in the shell from git reads. A literal path selector is a degenerate
/// glob (matches itself, no tree resolution). A path matched by several
/// selectors is reported once against the FIRST selector (selector order).
/// Iteration is over a `BTreeMap`, so output ordering is deterministic.
pub(crate) fn compute(selectors: &[String], actual: &BTreeMap<String, Vec<Status>>) -> Conformance {
    let compiled: Vec<(String, Option<Pattern>)> = selectors
        .iter()
        .map(|s| (s.clone(), Pattern::new(s).ok()))
        .collect();

    let mut out = Conformance::default();
    let mut matched: BTreeSet<&str> = BTreeSet::new();

    for (path, events) in actual {
        match compiled
            .iter()
            .find(|(_, pat)| pat.as_ref().is_some_and(|p| glob_matches(p, path)))
        {
            Some((sel, _)) => {
                matched.insert(sel.as_str());
                out.conformant.push(Conformant {
                    path: path.clone(),
                    matched_selector: sel.clone(),
                });
            }
            None => out.undeclared.push(Undeclared {
                path: path.clone(),
                verb: net(events),
            }),
        }
    }

    out.undelivered = compiled
        .iter()
        .filter(|(sel, _)| !matched.contains(sel.as_str()))
        .map(|(sel, _)| sel.clone())
        .collect();

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(s: &[Status]) -> Vec<Status> {
        s.to_vec()
    }

    // --- net() over the four canonical orderings (F-3) ---

    #[test]
    fn net_added_then_modified_is_added() {
        assert_eq!(net(&ev(&[Status::Added, Status::Modified])), Verb::Added);
    }

    #[test]
    fn net_added_then_deleted_is_removed() {
        assert_eq!(net(&ev(&[Status::Added, Status::Deleted])), Verb::Removed);
    }

    #[test]
    fn net_modified_then_modified_is_modified() {
        assert_eq!(
            net(&ev(&[Status::Modified, Status::Modified])),
            Verb::Modified
        );
    }

    #[test]
    fn net_modified_deleted_added_is_added() {
        // trailing event is Added (not Deleted) and the set contains Added.
        assert_eq!(
            net(&ev(&[Status::Modified, Status::Deleted, Status::Added])),
            Verb::Added
        );
    }

    // --- algebra cells ---

    fn actual(pairs: &[(&str, &[Status])]) -> BTreeMap<String, Vec<Status>> {
        pairs
            .iter()
            .map(|(p, s)| ((*p).to_string(), s.to_vec()))
            .collect()
    }

    #[test]
    fn conformant_paths_carry_the_matched_selector() {
        let sel = vec!["src/*.rs".to_string()];
        let a = actual(&[("src/state.rs", &[Status::Modified])]);
        let c = compute(&sel, &a);
        assert_eq!(
            c.conformant,
            vec![Conformant {
                path: "src/state.rs".to_string(),
                matched_selector: "src/*.rs".to_string(),
            }]
        );
        assert!(c.undeclared.is_empty());
        assert!(c.undelivered.is_empty());
    }

    #[test]
    fn undeclared_path_carries_its_net_verb() {
        let sel = vec!["src/state.rs".to_string()];
        let a = actual(&[("docs/readme.md", &[Status::Added, Status::Deleted])]);
        let c = compute(&sel, &a);
        assert!(c.conformant.is_empty());
        assert_eq!(
            c.undeclared,
            vec![Undeclared {
                path: "docs/readme.md".to_string(),
                verb: Verb::Removed,
            }]
        );
        // the unmatched selector is undelivered.
        assert_eq!(c.undelivered, vec!["src/state.rs".to_string()]);
    }

    #[test]
    fn literal_selector_matches_exact_path_only() {
        let sel = vec!["src/state.rs".to_string()];
        let a = actual(&[
            ("src/state.rs", &[Status::Modified]),
            ("src/state_helper.rs", &[Status::Modified]),
        ]);
        let c = compute(&sel, &a);
        assert_eq!(c.conformant.len(), 1);
        assert_eq!(c.conformant[0].path, "src/state.rs");
        assert_eq!(c.undeclared.len(), 1);
        assert_eq!(c.undeclared[0].path, "src/state_helper.rs");
    }

    #[test]
    fn glob_selector_absorbs_multiple_paths_each_reporting_the_selector() {
        let sel = vec!["src/**".to_string()];
        let a = actual(&[
            ("src/a.rs", &[Status::Added]),
            ("src/sub/b.rs", &[Status::Modified]),
        ]);
        let c = compute(&sel, &a);
        assert_eq!(c.conformant.len(), 2);
        assert!(c.conformant.iter().all(|x| x.matched_selector == "src/**"));
        assert!(c.undeclared.is_empty());
        assert!(c.undelivered.is_empty());
    }

    #[test]
    fn first_matching_selector_wins_and_others_stay_undelivered() {
        let sel = vec!["src/**".to_string(), "src/state.rs".to_string()];
        let a = actual(&[("src/state.rs", &[Status::Modified])]);
        let c = compute(&sel, &a);
        assert_eq!(c.conformant[0].matched_selector, "src/**");
        // the more specific selector matched nothing of its own → undelivered.
        assert_eq!(c.undelivered, vec!["src/state.rs".to_string()]);
    }
}
