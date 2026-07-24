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
    let compiled = compile(selectors);

    let mut out = Conformance::default();
    let mut matched: BTreeSet<&str> = BTreeSet::new();

    for (path, events) in actual {
        match matched_selector(&compiled, path) {
            Some(sel) => {
                matched.insert(sel);
                out.conformant.push(Conformant {
                    path: path.clone(),
                    matched_selector: sel.to_string(),
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

/// The paths in `paths` matched by NO selector, returned in INPUT order (the
/// import belt's scope check, SL-180 PHASE-02). Shares the ONE match
/// implementation with [`compute`] — a literal selector is a degenerate glob,
/// an empty `selectors` leaves every path undeclared (nothing declares it).
pub(crate) fn undeclared_paths(selectors: &[String], paths: &[&str]) -> Vec<String> {
    let compiled = compile(selectors);
    paths
        .iter()
        .filter(|path| matched_selector(&compiled, path).is_none())
        .map(|path| (*path).to_string())
        .collect()
}

/// The human-facing refusal detail for an `undeclared-scope` verdict: a count
/// header plus one RUNNABLE remediation line per undeclared path. Pure (no fs /
/// git / clock) so both the MCP funnel (`dispatch_import`) and the CLI import arm
/// share ONE prescription (SL-224 PHASE-01). Each line is exactly the command that
/// declares the path — `doctrine slice selector add SL-NNN <path> --intent
/// design-target` — with the slice id as the REQUIRED leading positional (the CLI
/// misparses without it). Deliberately TOKEN-FREE: the `undeclared-scope` reason
/// token is carried separately (the MCP `reason` field / the CLI bail), and a
/// leaf→engine reference to `Refusal::token()` would invert the ADR-001 layering.
pub(crate) fn undeclared_detail(slice: u32, undeclared: &[String]) -> String {
    // House style: build via Vec<String> + join, never push_str(&format!) (the
    // `format_push_string` deny).
    let mut lines = vec![format!(
        "{} path(s) declared by no design-target selector:",
        undeclared.len()
    )];
    lines.extend(undeclared.iter().map(|path| {
        format!("  doctrine slice selector add SL-{slice:03} {path} --intent design-target")
    }));
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

/// Compile the selector strings to `(selector, Option<Pattern>)` pairs once. A
/// selector that fails to compile as a glob carries `None` and matches nothing
/// (never panics). Shared by [`compute`] and [`undeclared_paths`].
fn compile(selectors: &[String]) -> Vec<(String, Option<Pattern>)> {
    selectors
        .iter()
        .map(|s| (s.clone(), Pattern::new(s).ok()))
        .collect()
}

/// The FIRST compiled selector matching `path` (selector order), or `None` when
/// no selector matches. The single per-path match rule behind both cells.
fn matched_selector<'a>(compiled: &'a [(String, Option<Pattern>)], path: &str) -> Option<&'a str> {
    compiled
        .iter()
        .find(|(_, pat)| pat.as_ref().is_some_and(|p| glob_matches(p, path)))
        .map(|(sel, _)| sel.as_str())
}

// ---------------------------------------------------------------------------
// Selector health advisory (SL-190 PHASE-06) — the pure predicate. `slice
// selector doctor` (the shell in `crate::slice`) is its first consumer; SL-180's
// future selector gate consumes this SAME function (no parallel matcher, RV-214
// F-7 — it reuses `compute`'s `glob::Pattern` + `glob_matches` machinery).
// ---------------------------------------------------------------------------

/// The health-relevant reading of a selector's declared intent. Mirrors slice's
/// `SelectorIntent` as a leaf value (the shell maps it, exactly as it maps git
/// name-status letters to [`Status`]) so the pure predicate needn't import a
/// command-tier type (ADR-001). `ReadFence` = `scope-relevant` (L0): a loose read
/// fence is legitimate, so `Broad` is suppressed. `WillTouch` = `design-target`
/// (L1): breadth over-claims the work surface, so `Broad` applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectorScope {
    ReadFence,
    WillTouch,
}

/// A `design-target` selector matching more than `BROAD_SHARE_NUM`/`BROAD_SHARE_DEN`
/// of the tracked universe is flagged `Broad` (advisory). Integer ratio — `f64`
/// casts are denied (`cast_precision_loss`): `matched·DEN > universe·NUM` ⇔
/// `matched/universe > NUM/DEN`. Half the universe is the loose default.
const BROAD_SHARE_NUM: usize = 1;
const BROAD_SHARE_DEN: usize = 2;

/// One health finding about a single authored selector (SL-190 PHASE-06). Advisory
/// by default; `slice selector doctor --assert` turns any finding into a non-zero
/// exit. Carries the offending selector string (F-7 transparency).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SelectorFinding {
    /// The glob string does not compile as a `glob::Pattern` — it can never match.
    Uncompilable { selector: String, error: String },
    /// The selector compiles but matches no path in the tracked universe (stale).
    Unmatched { selector: String },
    /// Every path this selector matches is also matched by a strictly broader
    /// peer — it adds nothing (its match set is a proper subset of `subsumed_by`).
    Redundant {
        selector: String,
        subsumed_by: String,
    },
    /// A `design-target` selector matching a suspiciously large share of the
    /// universe — it likely over-claims the work surface. Suppressed for
    /// `ReadFence` intent (a loose read fence is legitimate, not nagged).
    Broad {
        selector: String,
        matched: usize,
        universe: usize,
    },
}

/// The set of `universe` paths matched by glob `sel` under the shared match policy
/// ([`glob_matches`]). `None` when `sel` does not compile. Reuses conformance's
/// exact glob machinery — no parallel matcher (RV-214 F-7).
fn matches_in(sel: &str, universe: &BTreeSet<String>) -> Option<BTreeSet<String>> {
    let pat = Pattern::new(sel).ok()?;
    Some(
        universe
            .iter()
            .filter(|p| glob_matches(&pat, p))
            .cloned()
            .collect(),
    )
}

/// Diagnose one authored selector against the tracked `universe` and its peer
/// selector strings `others`. PURE (std + the `glob` leaf only, like [`compute`]):
/// no git, disk, or registry. Findings, in order: `Uncompilable` (returned alone —
/// a broken glob can't be reasoned about further), else `Unmatched` (empty match
/// set), else any of `Redundant` (proper subset of a broader peer) / `Broad`
/// (design-target matching over half the universe; suppressed for `ReadFence`).
/// `others` is scanned in the given order, so the caller sorts for determinism.
pub(crate) fn diagnose_selector(
    sel: &str,
    scope: SelectorScope,
    universe: &BTreeSet<String>,
    others: &[&str],
) -> Vec<SelectorFinding> {
    let pat = match Pattern::new(sel) {
        Ok(p) => p,
        Err(e) => {
            return vec![SelectorFinding::Uncompilable {
                selector: sel.to_string(),
                error: e.to_string(),
            }];
        }
    };

    let matched: BTreeSet<String> = universe
        .iter()
        .filter(|p| glob_matches(&pat, p))
        .cloned()
        .collect();

    if matched.is_empty() {
        return vec![SelectorFinding::Unmatched {
            selector: sel.to_string(),
        }];
    }

    let mut findings = Vec::new();

    // Redundant: a peer whose match set is a PROPER superset of ours absorbs us.
    // Proper (⊃, not ⊇) so two equal globs don't mutually flag each other.
    if let Some(peer) = others
        .iter()
        .filter(|o| **o != sel)
        .filter_map(|o| matches_in(o, universe).map(|m| (*o, m)))
        .find(|(_, m)| m.len() > matched.len() && matched.is_subset(m))
        .map(|(o, _)| o)
    {
        findings.push(SelectorFinding::Redundant {
            selector: sel.to_string(),
            subsumed_by: peer.to_string(),
        });
    }

    // Broad: only `design-target` (WillTouch); a `scope-relevant` read fence is
    // exempt (a loose read fence is legitimate, not nagged).
    if scope == SelectorScope::WillTouch
        && !universe.is_empty()
        && matched.len() * BROAD_SHARE_DEN > universe.len() * BROAD_SHARE_NUM
    {
        findings.push(SelectorFinding::Broad {
            selector: sel.to_string(),
            matched: matched.len(),
            universe: universe.len(),
        });
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(s: &[Status]) -> Vec<Status> {
        s.to_vec()
    }

    fn universe(paths: &[&str]) -> BTreeSet<String> {
        paths.iter().map(|p| (*p).to_string()).collect()
    }

    // --- diagnose_selector: one test per finding (VT-1) ---

    #[test]
    fn diagnose_uncompilable_glob_is_flagged_alone() {
        let u = universe(&["src/a.rs"]);
        let f = diagnose_selector("src/[", SelectorScope::WillTouch, &u, &[]);
        assert!(
            matches!(f.as_slice(), [SelectorFinding::Uncompilable { selector, .. }] if selector == "src/["),
            "a broken glob is Uncompilable and nothing else: {f:?}"
        );
    }

    #[test]
    fn diagnose_unmatched_selector_is_flagged() {
        let u = universe(&["src/a.rs", "src/b.rs"]);
        let f = diagnose_selector("docs/*.md", SelectorScope::WillTouch, &u, &[]);
        assert_eq!(
            f,
            vec![SelectorFinding::Unmatched {
                selector: "docs/*.md".to_string(),
            }]
        );
    }

    #[test]
    fn diagnose_redundant_when_subsumed_by_a_broader_peer() {
        let u = universe(&["src/a.rs", "src/b.rs", "src/sub/c.rs"]);
        // The literal matches only itself; the peer `src/**` is a proper superset.
        let f = diagnose_selector(
            "src/a.rs",
            SelectorScope::WillTouch,
            &u,
            &["src/**", "src/a.rs"],
        );
        assert!(
            f.contains(&SelectorFinding::Redundant {
                selector: "src/a.rs".to_string(),
                subsumed_by: "src/**".to_string(),
            }),
            "a selector whose matches are a subset of another is redundant: {f:?}"
        );
    }

    #[test]
    fn diagnose_broad_design_target_matching_most_of_the_universe() {
        let u = universe(&["src/a.rs", "src/b.rs", "docs/x.md", "docs/y.md"]);
        // `**` matches all four → over half → broad for a design-target.
        let f = diagnose_selector("**", SelectorScope::WillTouch, &u, &[]);
        assert!(
            f.iter().any(|x| matches!(
                x,
                SelectorFinding::Broad {
                    matched: 4,
                    universe: 4,
                    ..
                }
            )),
            "a design-target matching the whole universe is broad: {f:?}"
        );
    }

    #[test]
    fn diagnose_broad_suppressed_for_scope_relevant_read_fence() {
        let u = universe(&["src/a.rs", "src/b.rs", "docs/x.md", "docs/y.md"]);
        // Same broad `**`, but a `scope-relevant` read fence is legitimate.
        let f = diagnose_selector("**", SelectorScope::ReadFence, &u, &[]);
        assert!(
            !f.iter().any(|x| matches!(x, SelectorFinding::Broad { .. })),
            "broad is suppressed for a scope-relevant read fence: {f:?}"
        );
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

    // --- VT-3: undeclared_paths (the import belt's scope predicate) ---

    #[test]
    fn undeclared_paths_returns_the_unmatched_subset_in_input_order() {
        let sel = vec!["src/**".to_string()];
        let paths = ["docs/b.md", "src/a.rs", "docs/a.md"];
        // Only the two docs paths are undeclared, in their input order.
        assert_eq!(
            undeclared_paths(&sel, &paths),
            vec!["docs/b.md".to_string(), "docs/a.md".to_string()]
        );
    }

    #[test]
    fn undeclared_paths_empty_selectors_leaves_every_path_undeclared() {
        let sel: Vec<String> = Vec::new();
        let paths = ["src/a.rs", "docs/b.md"];
        assert_eq!(
            undeclared_paths(&sel, &paths),
            vec!["src/a.rs".to_string(), "docs/b.md".to_string()]
        );
    }

    #[test]
    fn undeclared_paths_glob_selector_absorbs_its_matches() {
        let sel = vec!["src/**".to_string()];
        let paths = ["src/a.rs", "src/sub/b.rs"];
        assert!(undeclared_paths(&sel, &paths).is_empty());
    }

    #[test]
    fn undeclared_paths_literal_selector_matches_its_exact_path_only() {
        let sel = vec!["src/state.rs".to_string()];
        let paths = ["src/state.rs", "src/state_helper.rs"];
        // The literal absorbs its exact path; the near-miss stays undeclared.
        assert_eq!(
            undeclared_paths(&sel, &paths),
            vec!["src/state_helper.rs".to_string()]
        );
    }

    #[test]
    fn undeclared_detail_renders_a_runnable_selector_add_per_path() {
        let undeclared = vec!["docs/readme.md".to_string(), "src/f.rs".to_string()];
        let detail = undeclared_detail(224, &undeclared);
        // Count header names how many paths and is TOKEN-FREE (no reason token).
        assert!(
            detail.starts_with("2 path(s) declared by no design-target selector:"),
            "count header: {detail}"
        );
        assert!(
            !detail.contains("undeclared-scope"),
            "detail must not embed the reason token: {detail}"
        );
        // One RUNNABLE remediation per path, id present as SL-NNN (3-digit zero-pad).
        assert!(
            detail.contains(
                "doctrine slice selector add SL-224 docs/readme.md --intent design-target"
            )
        );
        assert!(
            detail.contains("doctrine slice selector add SL-224 src/f.rs --intent design-target")
        );
        // Exactly one remediation line per undeclared path (plus the header).
        assert_eq!(
            detail
                .lines()
                .filter(|l| l.contains("selector add"))
                .count(),
            2
        );
    }
}
