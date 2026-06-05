// SPDX-License-Identifier: GPL-3.0-only
//! The relation-index seed — the pure FK-integrity index `spec validate` checks
//! against (design §5.6).
//!
//! This module is deliberately minimal: a `Registry` is a snapshot of the corpus's
//! id sets + edge lists, and the checks are pure functions over it (no disk, no
//! clock). `spec::build_registry` is the impure scan that populates one; the verb
//! `spec::run_validate` consumes the checks. Kept at the shape the four v1 checks
//! force — no generic edge framework, no cache (the relation-index *cache* is
//! deferred; this is the cache-independent seed).
//!
//! Ids are stored **canonical** (`"REQ-007"`, `"SPEC-012"`): FKs are stored
//! canonical too, so every check is a direct string-set membership test — no
//! FK→numeric parsing at the check site, and the tech-only interaction rule falls
//! out for free (a `PRD-*` target is simply absent from `tech_specs`).

use std::collections::{BTreeMap, BTreeSet};

/// One spec→requirement membership edge: a row of some spec's `members.toml`.
pub(crate) struct MemberEdge {
    /// Canonical ref of the spec the row lives in (`PRD-NNN` / `SPEC-NNN`).
    pub(crate) spec: String,
    /// Canonical requirement FK the row points at (`REQ-NNN`).
    pub(crate) requirement: String,
    /// The membership label (`FR-NNN` / `NF-NNN`), unique within a spec.
    pub(crate) label: String,
}

/// One spec→spec outbound edge: a row of a tech spec's `interactions.toml`.
pub(crate) struct InteractionEdge {
    /// Canonical ref of the tech spec the edge originates from (`SPEC-NNN`).
    pub(crate) spec: String,
    /// Canonical target ref the edge points at — expected to be a tech spec.
    pub(crate) target: String,
}

/// A cache-independent snapshot of the corpus's ids + edges. Built fresh per
/// `spec validate` invocation. Only the sets a check consumes are materialised:
/// there is no product id set (no check resolves against one — §5.4).
#[derive(Default)]
pub(crate) struct Registry {
    /// Canonical ids of every requirement in the tree (`REQ-NNN`).
    pub(crate) requirements: BTreeSet<String>,
    /// Canonical ids of every tech spec (`SPEC-NNN`) — interaction targets resolve
    /// against this set (tech-only).
    pub(crate) tech_specs: BTreeSet<String>,
    /// Every membership edge across product **and** tech specs.
    pub(crate) members: Vec<MemberEdge>,
    /// Every outbound interaction edge (tech specs only).
    pub(crate) interactions: Vec<InteractionEdge>,
}

impl Registry {
    /// HARD — member FK that resolves to no requirement (dangling FK). Restricted
    /// to one spec when `scope` is `Some` (the scoped-validate outbound check).
    pub(crate) fn dangling_member_fks(&self, scope: Option<&str>) -> Vec<String> {
        self.members
            .iter()
            .filter(|m| scope.is_none_or(|s| m.spec == s))
            .filter(|m| !self.requirements.contains(&m.requirement))
            .map(|m| {
                format!(
                    "dangling member FK: {} ({}) in {} resolves to no requirement",
                    m.label, m.requirement, m.spec
                )
            })
            .collect()
    }

    /// HARD — interaction target that resolves to no tech spec (dangling FK). A
    /// non-tech target (`PRD-*`) is absent from `tech_specs`, so the tech-only rule
    /// is enforced by the same membership test. Scoped when `scope` is `Some`.
    pub(crate) fn dangling_interaction_targets(&self, scope: Option<&str>) -> Vec<String> {
        self.interactions
            .iter()
            .filter(|e| scope.is_none_or(|s| e.spec == s))
            .filter(|e| !self.tech_specs.contains(&e.target))
            .map(|e| {
                format!(
                    "dangling interaction target: {} in {} resolves to no tech spec",
                    e.target, e.spec
                )
            })
            .collect()
    }

    /// HARD — a membership label used more than once within a single spec. Grouped
    /// per spec (`BTreeMap` for deterministic ordering). Scoped when `scope` is
    /// `Some` — duplicate detection is intra-spec, so a scoped run is complete.
    pub(crate) fn duplicate_labels(&self, scope: Option<&str>) -> Vec<String> {
        let mut counts: BTreeMap<&str, BTreeMap<&str, u32>> = BTreeMap::new();
        for m in self
            .members
            .iter()
            .filter(|m| scope.is_none_or(|s| m.spec == s))
        {
            *counts
                .entry(&m.spec)
                .or_default()
                .entry(&m.label)
                .or_default() += 1;
        }
        let mut out = Vec::new();
        for (spec, labels) in counts {
            for (label, n) in labels {
                if n > 1 {
                    out.push(format!(
                        "duplicate label {label} in {spec} ({n} members share it)"
                    ));
                }
            }
        }
        out
    }

    /// Run every check appropriate to `scope` and return all findings, in a stable
    /// order. The three outbound / intra-spec checks always run (restricted to
    /// `scope` when `Some`); the orphan check is corpus-level and runs ONLY on a
    /// whole-corpus pass — a requirement's membership is unknowable from one spec
    /// (§5.4), so a scoped run suppresses it.
    pub(crate) fn validate(&self, scope: Option<&str>) -> Vec<String> {
        let mut findings = Vec::new();
        findings.extend(self.dangling_member_fks(scope));
        findings.extend(self.dangling_interaction_targets(scope));
        findings.extend(self.duplicate_labels(scope));
        if scope.is_none() {
            findings.extend(self.orphan_requirements());
        }
        findings
    }

    /// HARD, **corpus only** — a requirement membered by no spec. Every requirement
    /// is born membered (§5.4), so an orphan is evidence of a torn two-tree write,
    /// not benign drift. Membership is unknowable from a single spec, so this never
    /// runs scoped (the caller suppresses it).
    pub(crate) fn orphan_requirements(&self) -> Vec<String> {
        let membered: BTreeSet<&str> = self
            .members
            .iter()
            .map(|m| m.requirement.as_str())
            .collect();
        self.requirements
            .iter()
            .filter(|r| !membered.contains(r.as_str()))
            .map(|r| format!("orphan requirement: {r} is membered by no spec (torn write?)"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(spec: &str, requirement: &str, label: &str) -> MemberEdge {
        MemberEdge {
            spec: spec.to_string(),
            requirement: requirement.to_string(),
            label: label.to_string(),
        }
    }

    fn interaction(spec: &str, target: &str) -> InteractionEdge {
        InteractionEdge {
            spec: spec.to_string(),
            target: target.to_string(),
        }
    }

    fn ids(refs: &[&str]) -> BTreeSet<String> {
        refs.iter().map(|s| (*s).to_string()).collect()
    }

    /// A clean corpus: every FK resolves, labels unique, no orphan.
    fn clean() -> Registry {
        Registry {
            requirements: ids(&["REQ-001", "REQ-002"]),
            tech_specs: ids(&["SPEC-001"]),
            members: vec![
                member("PRD-001", "REQ-001", "FR-001"),
                member("SPEC-001", "REQ-002", "FR-001"),
            ],
            interactions: vec![interaction("SPEC-001", "SPEC-001")],
        }
    }

    #[test]
    fn clean_corpus_yields_no_findings() {
        let r = clean();
        assert!(r.dangling_member_fks(None).is_empty());
        assert!(r.dangling_interaction_targets(None).is_empty());
        assert!(r.duplicate_labels(None).is_empty());
        assert!(r.orphan_requirements().is_empty());
    }

    #[test]
    fn dangling_member_fk_is_flagged() {
        let mut r = clean();
        r.members.push(member("PRD-001", "REQ-404", "FR-002"));
        let found = r.dangling_member_fks(None);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("REQ-404"));
    }

    #[test]
    fn dangling_interaction_target_is_flagged() {
        let mut r = clean();
        r.interactions.push(interaction("SPEC-001", "SPEC-404"));
        let found = r.dangling_interaction_targets(None);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("SPEC-404"));
    }

    #[test]
    fn non_tech_interaction_target_is_flagged_tech_only() {
        // A product ref as an interaction target is absent from `tech_specs`.
        let mut r = clean();
        r.requirements = ids(&["REQ-001", "REQ-002"]);
        r.interactions.push(interaction("SPEC-001", "PRD-001"));
        let found = r.dangling_interaction_targets(None);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("PRD-001"));
    }

    #[test]
    fn duplicate_label_within_a_spec_is_flagged() {
        let mut r = clean();
        // SPEC-001 already has FR-001; add a second member reusing it.
        r.requirements.insert("REQ-003".to_string());
        r.members.push(member("SPEC-001", "REQ-003", "FR-001"));
        let found = r.duplicate_labels(None);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("FR-001") && found[0].contains("SPEC-001"));
    }

    #[test]
    fn same_label_across_different_specs_is_not_duplicate() {
        // clean() already has FR-001 in both PRD-001 and SPEC-001 — intra-spec only.
        assert!(clean().duplicate_labels(None).is_empty());
    }

    #[test]
    fn orphan_requirement_is_flagged_corpus() {
        let mut r = clean();
        r.requirements.insert("REQ-009".to_string()); // membered by nobody
        let found = r.orphan_requirements();
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("REQ-009"));
    }

    #[test]
    fn validate_runs_orphan_only_on_a_corpus_pass() {
        let mut r = clean();
        r.requirements.insert("REQ-009".to_string()); // orphan
        // corpus pass sees the orphan …
        assert_eq!(r.validate(None).len(), 1);
        // … a scoped pass of a clean spec suppresses the corpus orphan check.
        assert!(r.validate(Some("SPEC-001")).is_empty());
    }

    #[test]
    fn scoped_checks_only_the_named_spec() {
        let mut r = clean();
        // A dangling FK in PRD-001; a scoped run of SPEC-001 must not see it.
        r.members.push(member("PRD-001", "REQ-404", "FR-002"));
        assert!(r.dangling_member_fks(Some("SPEC-001")).is_empty());
        assert_eq!(r.dangling_member_fks(Some("PRD-001")).len(), 1);
        assert_eq!(r.dangling_member_fks(None).len(), 1);
    }
}
