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

/// One spec→parent decomposition edge: a spec's `parent` field (SL-022 §5.2,
/// SL-065 §4). `on_product` selects the required parent subtype — a product
/// subject's parent must be a product spec, a tech subject's a tech spec.
pub(crate) struct ParentEdge {
    /// Canonical ref of the spec the `parent` field lives on.
    pub(crate) spec: String,
    /// Canonical parent ref — expected to be the same subtype as the subject.
    pub(crate) parent: String,
    /// True when the subject is a product spec (selects the required parent subtype).
    pub(crate) on_product: bool,
}

/// One spec→capability descent edge: a tech spec's `descends_from` field
/// (SL-022 §5.2). `on_product` carries the subject's kind (see `ParentEdge`).
pub(crate) struct DescentEdge {
    /// Canonical ref of the spec the `descends_from` field lives on.
    pub(crate) spec: String,
    /// Canonical descent target — expected to be a product spec.
    pub(crate) target: String,
    /// True when the subject is a product spec (`descends_from` is tech-only).
    pub(crate) on_product: bool,
}

/// A hard finding born at scan time, before any pure check can run — the
/// `second_parent` parse-error classification (SL-022 §5.2, codex F1). `validate`
/// surfaces it (scope-filtered by `spec`) alongside the pure-check findings.
pub(crate) struct BuildFinding {
    /// Canonical ref of the spec the finding is about (known from the dir scan even
    /// when its `spec-NNN.toml` failed to parse).
    pub(crate) spec: String,
    /// The rendered hard-finding message.
    pub(crate) message: String,
}

/// A cache-independent snapshot of the corpus's ids + edges. Built fresh per
/// `spec validate` invocation. Only the sets a check consumes are materialised.
#[derive(Default)]
pub(crate) struct Registry {
    /// Canonical ids of every requirement in the tree (`REQ-NNN`).
    pub(crate) requirements: BTreeSet<String>,
    /// Canonical ids of every tech spec (`SPEC-NNN`) — interaction targets resolve
    /// against this set (tech-only).
    pub(crate) tech_specs: BTreeSet<String>,
    /// Canonical ids of every product spec (`PRD-NNN`) — descent targets resolve
    /// against this set, and a tech parent / interaction target landing here is an
    /// invalid kind rather than dangling (SL-022 §5.2).
    pub(crate) product_specs: BTreeSet<String>,
    /// Every membership edge across product **and** tech specs.
    pub(crate) members: Vec<MemberEdge>,
    /// Every outbound interaction edge (tech specs only).
    pub(crate) interactions: Vec<InteractionEdge>,
    /// Every outbound decomposition (`parent`) edge, both subtypes (a product
    /// carrying the tech-only field is harvested so the check can flag it).
    pub(crate) parents: Vec<ParentEdge>,
    /// Every outbound descent (`descends_from`) edge, both subtypes (as above).
    pub(crate) descents: Vec<DescentEdge>,
    /// Scan-time hard findings from parse-error classification (the `second_parent`
    /// carrier, SL-022 §5.2 codex F1) — populated only by the impure `build_registry`,
    /// so `registry.rs` stays a pure leaf (ADR-001).
    pub(crate) build_findings: Vec<BuildFinding>,
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

    /// HARD — interaction target that is not a valid tech spec. Split by kind
    /// (REQ-084, an intended contract move — PRD-012 §6): a target that is a product
    /// spec is *invalid kind* (interactions are tech→tech), any other unresolved
    /// target is *dangling*. Scoped when `scope` is `Some`.
    pub(crate) fn dangling_interaction_targets(&self, scope: Option<&str>) -> Vec<String> {
        self.interactions
            .iter()
            .filter(|e| scope.is_none_or(|s| e.spec == s))
            .filter(|e| !self.tech_specs.contains(&e.target))
            .map(|e| {
                if self.product_specs.contains(&e.target) {
                    format!(
                        "invalid interaction target: {} in {} is a product spec (must be tech)",
                        e.target, e.spec
                    )
                } else {
                    format!(
                        "dangling interaction target: {} in {} resolves to no spec",
                        e.target, e.spec
                    )
                }
            })
            .collect()
    }

    /// HARD — `descends_from` integrity (REQ-082, SL-022 §5.2). `descends_from` is
    /// a tech-only field, so on a product subject it is *invalid kind* (codex F5).
    /// On a tech subject the target must be a product spec: a tech target is
    /// *invalid kind*, an absent target is *dangling*. Scoped when `scope` is `Some`.
    pub(crate) fn descent_findings(&self, scope: Option<&str>) -> Vec<String> {
        let mut out = Vec::new();
        for e in self
            .descents
            .iter()
            .filter(|e| scope.is_none_or(|s| e.spec == s))
        {
            if e.on_product {
                out.push(format!(
                    "invalid descent: descends_from on product {} (tech-only field)",
                    e.spec
                ));
                continue;
            }
            if self.product_specs.contains(&e.target) {
                continue; // clean: tech descends from product
            }
            let msg = if self.tech_specs.contains(&e.target) {
                format!(
                    "invalid descent: {} descends_from {} which is a tech spec (must be product)",
                    e.spec, e.target
                )
            } else {
                format!(
                    "dangling descent: {} descends_from {} resolves to no product spec",
                    e.spec, e.target
                )
            };
            out.push(msg);
        }
        out
    }

    /// HARD — `parent` integrity (REQ-083, SL-065 §4). `parent` must resolve to a
    /// spec of the **same subtype** as the subject: `on_product` selects which set
    /// (`product_specs` vs `tech_specs`) the parent must land in. A cross-subtype
    /// parent is *invalid kind*, an absent parent is *dangling*. The self case
    /// (`parent == spec`) is excluded — it is `self_parent`'s to report. Scoped when
    /// `scope` is `Some`.
    pub(crate) fn parent_findings(&self, scope: Option<&str>) -> Vec<String> {
        let mut out = Vec::new();
        for e in self
            .parents
            .iter()
            .filter(|e| scope.is_none_or(|s| e.spec == s))
        {
            if e.spec == e.parent {
                continue; // self-loop — owned by self_parent
            }
            let (own_set, other_set, own_kind, other_kind) = if e.on_product {
                (&self.product_specs, &self.tech_specs, "product", "tech")
            } else {
                (&self.tech_specs, &self.product_specs, "tech", "product")
            };
            if own_set.contains(&e.parent) {
                continue; // clean: same-subtype parent
            }
            let msg = if other_set.contains(&e.parent) {
                format!(
                    "invalid parent: {} parent {} is a {other_kind} spec (must be {own_kind})",
                    e.spec, e.parent
                )
            } else {
                format!(
                    "dangling parent: {} in {} resolves to no {own_kind} spec",
                    e.parent, e.spec
                )
            };
            out.push(msg);
        }
        out
    }

    /// HARD — a spec naming itself as `parent` (the 1-cycle A→A). The **sole**
    /// reporter of the self case (REQ-087, §5.2): `parent_cycle` skips self-loops, so
    /// A→A yields exactly one finding total. Subtype-blind (SL-065 §4) — acyclicity
    /// is a property of the chain, not the family. Scoped when `scope` is `Some`.
    pub(crate) fn self_parent(&self, scope: Option<&str>) -> Vec<String> {
        self.parents
            .iter()
            .filter(|e| scope.is_none_or(|s| e.spec == s))
            .filter(|e| e.spec == e.parent)
            .map(|e| format!("self parent: {} names itself as parent", e.spec))
            .collect()
    }

    /// HARD — a cycle in the `parent` decomposition chain (REQ-087, §5.2). Walks the
    /// child→parent map from each node keeping an **ordered path** plus a
    /// first-seen index, recovers the cycle SLICE on revisit (`path[first_idx..]` —
    /// the ring only, not a tail that fed it), and emits **one** finding per cycle —
    /// only when the start node is the slice's least id (codex F3, correct dedup even
    /// for a tail feeding a ring). Self-loops are skipped (owned by `self_parent`);
    /// the walk terminates at a root (no parent edge) or a dangling parent.
    /// Subtype-blind (SL-065 §4): the chain spans both families. When `scope` is
    /// `Some`, only cycles whose slice contains that node are kept.
    pub(crate) fn parent_cycle(&self, scope: Option<&str>) -> Vec<String> {
        // Ephemeral child→parent inversion — built here, never persisted (storage
        // rule). Skip self-loops (self_parent's). Subtype-blind: spans both families.
        // A cross-subtype ring still yields a cycle finding — but each of its edges is
        // already invalid-kind, so it cannot forge a *spurious additional* cycle that
        // matters (design §4). In practice decomposition chains are within-family.
        let mut parent_of: BTreeMap<&str, &str> = BTreeMap::new();
        for e in &self.parents {
            if e.spec == e.parent {
                continue;
            }
            parent_of.insert(&e.spec, &e.parent);
        }
        let mut out = Vec::new();
        for &start in parent_of.keys() {
            let mut path: Vec<&str> = Vec::new();
            let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
            let mut node = start;
            loop {
                if let Some(&first) = seen.get(node) {
                    // Revisited a node on the path: the cycle is the slice from its
                    // first sighting. Emit once, owned by the slice's least id.
                    let slice = path.get(first..).unwrap_or_default();
                    let least = slice.iter().min().copied().unwrap_or(node);
                    let in_scope = scope.is_none_or(|s| slice.contains(&s));
                    if start == least && in_scope {
                        out.push(format!("parent cycle: {}", slice.join(" -> ")));
                    }
                    break;
                }
                match parent_of.get(node) {
                    Some(&next) => {
                        seen.insert(node, path.len());
                        path.push(node);
                        node = next;
                    }
                    None => break, // root or dangling parent — not a cycle
                }
            }
        }
        out
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
        findings.extend(self.descent_findings(scope));
        findings.extend(self.parent_findings(scope));
        findings.extend(self.self_parent(scope));
        findings.extend(self.parent_cycle(scope));
        findings.extend(self.duplicate_labels(scope));
        findings.extend(
            self.build_findings
                .iter()
                .filter(|f| scope.is_none_or(|s| f.spec == s))
                .map(|f| f.message.clone()),
        );
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

    fn descent(spec: &str, target: &str, on_product: bool) -> DescentEdge {
        DescentEdge {
            spec: spec.to_string(),
            target: target.to_string(),
            on_product,
        }
    }

    fn parent_edge(spec: &str, parent: &str, on_product: bool) -> ParentEdge {
        ParentEdge {
            spec: spec.to_string(),
            parent: parent.to_string(),
            on_product,
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
            // PRD-001 appears as a member spec below, so declare it a known kind —
            // keeps clean() symmetric across families. A future parent-edge baseline
            // pushing a product edge here won't draw a spurious invalid-kind finding.
            product_specs: ids(&["PRD-001"]),
            members: vec![
                member("PRD-001", "REQ-001", "FR-001"),
                member("SPEC-001", "REQ-002", "FR-001"),
            ],
            interactions: vec![interaction("SPEC-001", "SPEC-001")],
            ..Default::default()
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
    fn product_interaction_target_is_invalid_kind_not_dangling() {
        // REQ-084 contract move (PRD-012 §6): an interaction pointing at a product
        // spec is now *invalid kind*, not dangling. The product ref must be in
        // `product_specs` for the kind to be known; a target in neither set is
        // *dangling*. The two messages must be distinct.
        let mut r = clean();
        r.product_specs = ids(&["PRD-001"]);
        r.interactions.push(interaction("SPEC-001", "PRD-001")); // product target
        r.interactions.push(interaction("SPEC-001", "SPEC-404")); // neither set
        let found = r.dangling_interaction_targets(None);
        assert_eq!(found.len(), 2);
        let invalid = found.iter().find(|f| f.contains("PRD-001")).unwrap();
        let dangling = found.iter().find(|f| f.contains("SPEC-404")).unwrap();
        assert!(invalid.contains("invalid") && invalid.contains("product"));
        assert!(dangling.contains("dangling"));
        assert_ne!(invalid, dangling);
    }

    #[test]
    fn descent_clean_tech_to_product_yields_no_finding() {
        let mut r = clean();
        r.product_specs = ids(&["PRD-001"]);
        r.descents.push(descent("SPEC-001", "PRD-001", false));
        assert!(r.descent_findings(None).is_empty());
    }

    #[test]
    fn descent_dangling_target_is_flagged() {
        let mut r = clean();
        r.descents.push(descent("SPEC-001", "PRD-404", false));
        let found = r.descent_findings(None);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("dangling") && found[0].contains("PRD-404"));
    }

    #[test]
    fn descent_to_tech_target_is_invalid_kind() {
        // descends_from must resolve to a PRODUCT spec; a tech target is wrong kind.
        let mut r = clean();
        r.tech_specs = ids(&["SPEC-001", "SPEC-002"]);
        r.descents.push(descent("SPEC-001", "SPEC-002", false));
        let found = r.descent_findings(None);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("invalid") && found[0].contains("SPEC-002"));
    }

    #[test]
    fn descent_on_product_subject_is_invalid_kind() {
        // codex F5: descends_from is tech-only; on a product subject it is invalid.
        let mut r = clean();
        r.product_specs = ids(&["PRD-001", "PRD-002"]);
        r.descents.push(descent("PRD-001", "PRD-002", true));
        let found = r.descent_findings(None);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("invalid") && found[0].contains("PRD-001"));
    }

    #[test]
    fn parent_clean_tech_to_tech_yields_no_finding() {
        let mut r = clean();
        r.tech_specs = ids(&["SPEC-001", "SPEC-002"]);
        r.parents.push(parent_edge("SPEC-001", "SPEC-002", false));
        assert!(r.parent_findings(None).is_empty());
    }

    #[test]
    fn parent_dangling_target_is_flagged() {
        let mut r = clean();
        r.parents.push(parent_edge("SPEC-001", "SPEC-404", false));
        let found = r.parent_findings(None);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("dangling") && found[0].contains("SPEC-404"));
    }

    #[test]
    fn parent_to_product_target_is_invalid_kind() {
        // A parent must be a tech spec; a product parent is wrong kind (symmetry).
        let mut r = clean();
        r.product_specs = ids(&["PRD-001"]);
        r.parents.push(parent_edge("SPEC-001", "PRD-001", false));
        let found = r.parent_findings(None);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("invalid") && found[0].contains("PRD-001"));
    }

    #[test]
    fn parent_product_to_tech_is_invalid_kind() {
        // SL-065 §4: parent is symmetric same-subtype. A product subject whose parent
        // is a TECH spec is invalid-kind ("must be product") — the mirror of the
        // tech→product case, replacing the old "tech-only field" reject.
        let mut r = clean();
        r.product_specs = ids(&["PRD-001"]);
        r.parents.push(parent_edge("PRD-001", "SPEC-001", true));
        let found = r.parent_findings(None);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("invalid") && found[0].contains("must be product"));
    }

    #[test]
    fn parent_clean_product_to_product_yields_no_finding() {
        // SL-065 §4: product → product decomposition is now valid.
        let mut r = clean();
        r.product_specs = ids(&["PRD-001", "PRD-002"]);
        r.parents.push(parent_edge("PRD-001", "PRD-002", true));
        assert!(r.parent_findings(None).is_empty());
    }

    #[test]
    fn parent_product_dangling_target_is_flagged() {
        let mut r = clean();
        r.product_specs = ids(&["PRD-001"]);
        r.parents.push(parent_edge("PRD-001", "PRD-404", true));
        let found = r.parent_findings(None);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("dangling") && found[0].contains("PRD-404"));
    }

    #[test]
    fn self_parent_reports_product_a_to_a_once() {
        // SL-065 §4: acyclicity is subtype-blind — self_parent reports a product A→A.
        let mut r = clean();
        r.product_specs = ids(&["PRD-001"]);
        r.parents.push(parent_edge("PRD-001", "PRD-001", true));
        let found = r.self_parent(None);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("PRD-001"));
    }

    #[test]
    fn parent_cycle_product_two_node_reports_once() {
        // SL-065 §4: subtype-blind parent_cycle catches a product ring.
        let mut r = clean();
        r.product_specs = ids(&["PRD-001", "PRD-002"]);
        r.parents.push(parent_edge("PRD-001", "PRD-002", true));
        r.parents.push(parent_edge("PRD-002", "PRD-001", true));
        assert_eq!(r.parent_cycle(None).len(), 1);
    }

    #[test]
    fn parent_cycle_mixed_family_ring_is_still_reported() {
        // SL-065 §4: a cross-family ring (PRD→SPEC→PRD) is subtype-blind — the cycle
        // still fires even though each edge is independently invalid-kind. Pins the
        // load-bearing invariant against a future dedup tightening that suppresses it.
        let mut r = clean();
        r.product_specs = ids(&["PRD-001"]);
        r.tech_specs = ids(&["SPEC-001"]);
        r.parents.push(parent_edge("PRD-001", "SPEC-001", true));
        r.parents.push(parent_edge("SPEC-001", "PRD-001", false));
        assert_eq!(r.parent_cycle(None).len(), 1);
    }

    #[test]
    fn parent_self_case_is_excluded_owned_by_self_parent() {
        // The self-loop A→A is self_parent's to report (PHASE-03); parent_findings
        // emits nothing for it.
        let mut r = clean();
        r.tech_specs = ids(&["SPEC-001"]);
        r.parents.push(parent_edge("SPEC-001", "SPEC-001", false));
        assert!(r.parent_findings(None).is_empty());
    }

    #[test]
    fn self_parent_reports_a_to_a_once() {
        let mut r = clean();
        r.tech_specs = ids(&["SPEC-001"]);
        r.parents.push(parent_edge("SPEC-001", "SPEC-001", false));
        let found = r.self_parent(None);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("SPEC-001"));
    }

    #[test]
    fn self_loop_yields_exactly_one_finding_across_both_checks() {
        // VT-4: A→A is a self-parent AND a 1-cycle; only self_parent reports it.
        let mut r = clean();
        r.tech_specs = ids(&["SPEC-001"]);
        r.parents.push(parent_edge("SPEC-001", "SPEC-001", false));
        assert_eq!(r.self_parent(None).len(), 1);
        assert!(r.parent_cycle(None).is_empty());
    }

    #[test]
    fn parent_cycle_two_node_reports_once() {
        let mut r = clean();
        r.tech_specs = ids(&["SPEC-001", "SPEC-002"]);
        r.parents.push(parent_edge("SPEC-001", "SPEC-002", false));
        r.parents.push(parent_edge("SPEC-002", "SPEC-001", false));
        assert_eq!(r.parent_cycle(None).len(), 1);
    }

    #[test]
    fn parent_cycle_three_node_reports_once() {
        let mut r = clean();
        r.tech_specs = ids(&["SPEC-001", "SPEC-002", "SPEC-003"]);
        r.parents.push(parent_edge("SPEC-001", "SPEC-002", false));
        r.parents.push(parent_edge("SPEC-002", "SPEC-003", false));
        r.parents.push(parent_edge("SPEC-003", "SPEC-001", false));
        assert_eq!(r.parent_cycle(None).len(), 1);
    }

    #[test]
    fn parent_cycle_tail_feeding_a_ring_reports_the_ring_once() {
        // codex F3: T → A → B → A. The cycle slice is {A,B}; T is not in it, so its
        // walk never emits. Exactly one finding, naming the ring not the tail.
        let mut r = clean();
        r.tech_specs = ids(&["SPEC-001", "SPEC-002", "SPEC-009"]);
        r.parents.push(parent_edge("SPEC-009", "SPEC-001", false)); // T → A
        r.parents.push(parent_edge("SPEC-001", "SPEC-002", false)); // A → B
        r.parents.push(parent_edge("SPEC-002", "SPEC-001", false)); // B → A
        let found = r.parent_cycle(None);
        assert_eq!(found.len(), 1);
        assert!(!found[0].contains("SPEC-009"), "tail T must not be named");
    }

    #[test]
    fn parent_cycle_clean_chain_to_root_yields_no_finding() {
        let mut r = clean();
        r.tech_specs = ids(&["SPEC-001", "SPEC-002", "SPEC-003"]);
        r.parents.push(parent_edge("SPEC-001", "SPEC-002", false));
        r.parents.push(parent_edge("SPEC-002", "SPEC-003", false)); // SPEC-003 is root
        assert!(r.parent_cycle(None).is_empty());
    }

    #[test]
    fn parent_cycle_scoped_to_a_member_node_reports_it() {
        // A scoped run of a spec IN the ring still sees the cycle; a spec outside
        // (the tail T) does not.
        let mut r = clean();
        r.tech_specs = ids(&["SPEC-001", "SPEC-002", "SPEC-009"]);
        r.parents.push(parent_edge("SPEC-009", "SPEC-001", false)); // tail T → A
        r.parents.push(parent_edge("SPEC-001", "SPEC-002", false)); // A → B
        r.parents.push(parent_edge("SPEC-002", "SPEC-001", false)); // B → A
        assert_eq!(r.parent_cycle(Some("SPEC-001")).len(), 1);
        assert!(r.parent_cycle(Some("SPEC-009")).is_empty());
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
