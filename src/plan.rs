// SPDX-License-Identifier: GPL-3.0-only
//! `plan` — the authored implementation-plan read model.
//!
//! An engine-tier leaf: `Plan`/`PlanPhase` and the pure `Plan::parse`
//! validator, lifted out of `crate::slice` (SL-016) so the runtime `state`
//! layer can depend on a neutral home instead of reaching *up* into the
//! slice-CLI module. Pure — no clock, disk, or git here; disk IO
//! (`read_plan`) stays in the slice shell and calls `Plan::parse`.

use serde::Deserialize;

use anyhow::{Context, bail};

/// The authored implementation plan, read from `plan.toml`. Only the ordered
/// phase list is consumed in v1 (phase materialisation, slice-004 §5.2); the
/// specs/requirements link tables exist in the file but are empty (no registry
/// yet) and are not modelled. The first relational *read* model — no shared
/// `Meta` (slice-003 Non-Goal).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct Plan {
    #[serde(default)]
    pub phases: Vec<PlanPhase>,
}

/// One authored phase row. `id` is the canonical `PHASE-NN` join key; `name`
/// and `objective` seed the disposable phase sheet. The entrance/exit/verification
/// criteria are lifted into the model (SL-170 PHASE-01) so the VT existence/shape
/// gate (PHASE-03 `vtgate`) and downstream IDE-008 can read them; every added
/// field is `#[serde(default)]`, so legacy plans without them round-trip to
/// defaulted empties (the behaviour-preservation gate, design §3).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct PlanPhase {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub objective: String,
    #[serde(default)]
    pub entrance_criteria: Vec<Criterion>,
    #[serde(default)]
    pub exit_criteria: Vec<Criterion>,
    #[serde(default)]
    pub verification: Vec<VerificationCriterion>,
}

/// An authored entrance (`EN-`) or exit (`EX-`) criterion. `id` is the immutable
/// doc-local handle; `text` is the prose body (defaulted empty if omitted).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct Criterion {
    pub id: String,
    #[serde(default)]
    pub text: String,
}

/// An authored verification criterion. Mode (VT test / VA agent / VH human) stays
/// encoded in the `id` prefix — there is no separate mode field. `expects` is the
/// free-text expectation (untouched, heterogeneous by design); the P2 structured
/// fields (`test_file` / `keywords` / `patterns`) are the machine-checkable mandate
/// the PHASE-03 gate reads, and `waived` / `waived_reason` are the recorded escape
/// valve. All but `id` default, so legacy `{ id, expects }` rows parse unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct VerificationCriterion {
    pub id: String,
    #[serde(default)]
    pub expects: String,
    #[serde(default)]
    pub test_file: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub patterns: Vec<String>,
    #[serde(default)]
    pub waived: bool,
    #[serde(default)]
    pub waived_reason: Option<String>,
}

impl Plan {
    /// Parse and validate a `plan.toml` body. Rejects a plan whose phase ids
    /// are not unique — a duplicate would alias two phases onto one tracking
    /// file (finding 6). Per-id well-formedness (`PHASE-<digits>`) is enforced
    /// at the filesystem boundary by `state::phase_stem` (slice-004 §9), where
    /// an id becomes a filename.
    pub(crate) fn parse(text: &str) -> anyhow::Result<Plan> {
        // serde renames the TOML `[[phase]]` array to the `phases` field.
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            phase: Vec<PlanPhase>,
        }
        let raw: Raw = toml::from_str(text).context("Failed to parse plan.toml")?;
        let mut seen = std::collections::BTreeSet::new();
        for ph in &raw.phase {
            if !seen.insert(ph.id.as_str()) {
                bail!("Duplicate phase id {} in plan", ph.id);
            }
        }
        Ok(Plan { phases: raw.phase })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::SCHEMA_PLAN_OVERVIEW;

    #[test]
    fn plan_parse_reads_ordered_phases() {
        let text = format!(
            r#"
            schema = "{SCHEMA_PLAN_OVERVIEW}"
            version = 1
            slice = "SL-004"
            [[phase]]
            id = "PHASE-01"
            name = "First"
            objective = "do a"
            [[phase]]
            id = "PHASE-02"
            name = "Second"
        "#
        );
        let plan = Plan::parse(&text).unwrap();
        let ids: Vec<&str> = plan.phases.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, vec!["PHASE-01", "PHASE-02"]);
        assert_eq!(plan.phases[0].objective, "do a");
        // an absent objective defaults to empty, not an error
        assert_eq!(plan.phases[1].objective, "");
    }

    #[test]
    fn plan_parse_lifts_criteria_and_p2_fields() {
        let text = format!(
            r#"
            schema = "{SCHEMA_PLAN_OVERVIEW}"
            version = 1
            slice = "SL-170"
            [[phase]]
            id = "PHASE-01"
            name = "Lift"
            objective = "do it"
            entrance_criteria = [
              {{ id = "EN-1", text = "design locked" }},
            ]
            exit_criteria = [
              {{ id = "EX-1", text = "fields parse" }},
            ]
            verification = [
              {{ id = "VT-1", expects = "round-trip", test_file = "src/plan.rs", keywords = ["entrance_criteria", "verification"], patterns = ["^\\s*pub"], waived = false }},
              {{ id = "VT-2", expects = "behaviour-preserved", waived = true, waived_reason = "covered by existing suite" }},
            ]
        "#
        );
        let plan = Plan::parse(&text).unwrap();
        let ph = &plan.phases[0];
        assert_eq!(ph.entrance_criteria.len(), 1);
        assert_eq!(ph.entrance_criteria[0].id, "EN-1");
        assert_eq!(ph.entrance_criteria[0].text, "design locked");
        assert_eq!(ph.exit_criteria[0].id, "EX-1");
        assert_eq!(ph.exit_criteria[0].text, "fields parse");
        assert_eq!(ph.verification.len(), 2);
        let vt1 = &ph.verification[0];
        assert_eq!(vt1.id, "VT-1");
        assert_eq!(vt1.expects, "round-trip");
        assert_eq!(vt1.test_file.as_deref(), Some("src/plan.rs"));
        assert_eq!(vt1.keywords, vec!["entrance_criteria", "verification"]);
        assert_eq!(vt1.patterns, vec!["^\\s*pub"]);
        assert!(!vt1.waived);
        assert_eq!(vt1.waived_reason, None);
        let vt2 = &ph.verification[1];
        assert!(vt2.waived);
        assert_eq!(
            vt2.waived_reason.as_deref(),
            Some("covered by existing suite")
        );
    }

    #[test]
    fn plan_parse_defaults_structured_fields_on_legacy_rows() {
        let text = format!(
            r#"
            schema = "{SCHEMA_PLAN_OVERVIEW}"
            version = 1
            slice = "SL-016"
            [[phase]]
            id = "PHASE-01"
            name = "Legacy"
            verification = [
              {{ id = "VT-1", expects = "full suite green" }},
            ]
        "#
        );
        let plan = Plan::parse(&text).unwrap();
        let ph = &plan.phases[0];
        assert!(ph.entrance_criteria.is_empty());
        assert!(ph.exit_criteria.is_empty());
        let vt = &ph.verification[0];
        assert_eq!(vt.expects, "full suite green");
        assert_eq!(vt.test_file, None);
        assert!(vt.keywords.is_empty());
        assert!(vt.patterns.is_empty());
        assert!(!vt.waived);
        assert_eq!(vt.waived_reason, None);
    }
}

// ---------------------------------------------------------------------------
// VT shape check — `doctrine check plan <id>` (IMP-209)
// ---------------------------------------------------------------------------

/// A finding from [`check_vt_shape`] — one per flagged verification criterion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VtShapeFinding {
    pub phase_id: String,
    pub vt_id: String,
    pub problem: VtShapeProblem,
}

/// The reason a VT row was flagged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum VtShapeProblem {
    /// `test_file` is `None` — nothing to grep; the VT is UNCHECKABLE at runtime.
    BareTestFile,
    /// `test_file` is set but `keywords` is empty — vacuous pass at runtime.
    BareKeywords,
    /// `waived` is true but `waived_reason` is `None` — opaque waiver.
    MissingWaiverReason,
}

/// Check every VT-mode row in the plan for structured mandate completeness.
///
/// Returns findings for:
/// - non-waived VTs without `test_file` (`BareTestFile`)
/// - non-waived VTs with `test_file` but empty `keywords` (`BareKeywords` —
///   vacuous pass)
/// - waived VTs without `waived_reason` (`MissingWaiverReason` — opaque)
///
/// VA/VH rows are skipped entirely.
pub(crate) fn check_vt_shape(plan: &Plan) -> Vec<VtShapeFinding> {
    let mut findings = Vec::new();
    for ph in &plan.phases {
        for vt in &ph.verification {
            // Skip non-VT rows.
            if !vt.id.starts_with("VT-") {
                continue;
            }
            if vt.waived {
                if vt.waived_reason.is_none() {
                    findings.push(VtShapeFinding {
                        phase_id: ph.id.clone(),
                        vt_id: vt.id.clone(),
                        problem: VtShapeProblem::MissingWaiverReason,
                    });
                }
                continue;
            }
            // Non-waived VT — must have a structured mandate.
            if vt.test_file.is_none() {
                findings.push(VtShapeFinding {
                    phase_id: ph.id.clone(),
                    vt_id: vt.id.clone(),
                    problem: VtShapeProblem::BareTestFile,
                });
            } else if vt.keywords.is_empty() {
                findings.push(VtShapeFinding {
                    phase_id: ph.id.clone(),
                    vt_id: vt.id.clone(),
                    problem: VtShapeProblem::BareKeywords,
                });
            }
        }
    }
    findings
}

#[cfg(test)]
mod vt_shape_tests {
    use super::*;

    #[test]
    fn bare_test_file_flag() {
        let text = r#"
            [[phase]]
            id = "PHASE-01"
            verification = [
              { id = "VT-1", expects = "just prose" },
            ]
        "#;
        let plan = Plan::parse(text).unwrap();
        let findings = check_vt_shape(&plan);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].vt_id, "VT-1");
        assert_eq!(findings[0].phase_id, "PHASE-01");
        assert_eq!(findings[0].problem, VtShapeProblem::BareTestFile);
    }

    #[test]
    fn bare_keywords_flag() {
        let text = r#"
            [[phase]]
            id = "PHASE-01"
            verification = [
              { id = "VT-1", test_file = "src/foo.rs", keywords = [] },
            ]
        "#;
        let plan = Plan::parse(text).unwrap();
        let findings = check_vt_shape(&plan);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].problem, VtShapeProblem::BareKeywords);
    }

    #[test]
    fn structured_vt_passes() {
        let text = r#"
            [[phase]]
            id = "PHASE-01"
            verification = [
              { id = "VT-1", test_file = "src/foo.rs", keywords = ["fn"] },
            ]
        "#;
        let plan = Plan::parse(text).unwrap();
        let findings = check_vt_shape(&plan);
        assert!(findings.is_empty());
    }

    #[test]
    fn waived_with_reason_passes() {
        let text = r#"
            [[phase]]
            id = "PHASE-01"
            verification = [
              { id = "VT-1", waived = true, waived_reason = "manual only" },
            ]
        "#;
        let plan = Plan::parse(text).unwrap();
        let findings = check_vt_shape(&plan);
        assert!(findings.is_empty());
    }

    #[test]
    fn waived_without_reason_warns() {
        let text = r#"
            [[phase]]
            id = "PHASE-01"
            verification = [
              { id = "VT-1", waived = true },
            ]
        "#;
        let plan = Plan::parse(text).unwrap();
        let findings = check_vt_shape(&plan);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].problem, VtShapeProblem::MissingWaiverReason);
    }

    #[test]
    fn va_rows_skipped() {
        let text = r#"
            [[phase]]
            id = "PHASE-01"
            verification = [
              { id = "VA-1", expects = "agent reviews" },
              { id = "VH-1", expects = "human signs off" },
            ]
        "#;
        let plan = Plan::parse(text).unwrap();
        let findings = check_vt_shape(&plan);
        assert!(findings.is_empty());
    }

    #[test]
    fn multi_phase_multi_finding() {
        let text = r#"
            [[phase]]
            id = "PHASE-01"
            verification = [
              { id = "VT-1", test_file = "src/foo.rs", keywords = ["fn"] },
              { id = "VT-2", expects = "bare prose" },
            ]
            [[phase]]
            id = "PHASE-02"
            verification = [
              { id = "VT-3", test_file = "src/bar.rs", keywords = [] },
              { id = "VT-4", waived = true },
            ]
        "#;
        let plan = Plan::parse(text).unwrap();
        let findings = check_vt_shape(&plan);
        assert_eq!(findings.len(), 3);
        assert_eq!(findings[0].vt_id, "VT-2");
        assert_eq!(findings[0].problem, VtShapeProblem::BareTestFile);
        assert_eq!(findings[1].vt_id, "VT-3");
        assert_eq!(findings[1].problem, VtShapeProblem::BareKeywords);
        assert_eq!(findings[2].vt_id, "VT-4");
        assert_eq!(findings[2].problem, VtShapeProblem::MissingWaiverReason);
    }

    #[test]
    fn empty_phases_no_findings() {
        let text = "";
        let plan = Plan::parse(text).unwrap();
        assert!(plan.phases.is_empty());
        let findings = check_vt_shape(&plan);
        assert!(findings.is_empty());
    }

    #[test]
    fn waived_overrides_bare_shape() {
        // A waived VT without test_file should NOT report BareTestFile — the
        // waiver short-circuits; only MissingWaiverReason if reason is absent.
        let text = r#"
            [[phase]]
            id = "PHASE-01"
            verification = [
              { id = "VT-1", waived = true, waived_reason = "N/A" },
            ]
        "#;
        let plan = Plan::parse(text).unwrap();
        let findings = check_vt_shape(&plan);
        assert!(findings.is_empty());
    }

    #[test]
    fn waived_overrides_bare_shape_no_reason() {
        let text = r#"
            [[phase]]
            id = "PHASE-01"
            verification = [
              { id = "VT-1", waived = true },
            ]
        "#;
        let plan = Plan::parse(text).unwrap();
        let findings = check_vt_shape(&plan);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].problem, VtShapeProblem::MissingWaiverReason);
        // NOT BareTestFile — waiver short-circuits.
    }
}

#[cfg(test)]
mod tests_rejects {
    use super::*;

    #[test]
    fn plan_parse_rejects_duplicate_phase_ids() {
        let text = r#"
            [[phase]]
            id = "PHASE-01"
            [[phase]]
            id = "PHASE-01"
        "#;
        let err = Plan::parse(text).unwrap_err();
        assert!(err.to_string().contains("Duplicate phase id PHASE-01"));
    }
}
