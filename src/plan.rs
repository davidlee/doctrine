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
/// and `objective` seed the disposable phase sheet. Criteria/verification/link
/// fields exist in the file but are not consumed until a tracking consumer
/// graduates them (D5/Q2), so they are not modelled here.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct PlanPhase {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub objective: String,
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
        let text = format!(r#"
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
        "#);
        let plan = Plan::parse(&text).unwrap();
        let ids: Vec<&str> = plan.phases.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, vec!["PHASE-01", "PHASE-02"]);
        assert_eq!(plan.phases[0].objective, "do a");
        // an absent objective defaults to empty, not an error
        assert_eq!(plan.phases[1].objective, "");
    }

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
