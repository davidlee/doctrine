// SPDX-License-Identifier: GPL-3.0-only
//! `risk` — the risk facet model (SL-133).
//!
//! The risk facet's closed-set axes (`RiskLevel`), the tolerant + validated parse
//! layers (`RawRiskFacet` → `RiskFacet`), the `exposure` pure function, and the
//! shared `"" -> None` parse seams (private copies — each module that parses enums
//! from a string owns its own; precedence: `knowledge.rs`).
//!
//! **Leaf tier (ADR-001).** Pure data & parse — imports nothing from the engine or
//! any command module.

use serde::Deserialize;

// ── the risk-level enum ──────────────────────────────────────────────────────

/// A risk facet axis level. Closed set, kebab serde; tech of the risk `[facet]`,
/// optional (the `"" -> None` seam — seeded empty until assessed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    /// The kebab string for render (matches the serde rename). Pure.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        }
    }
}

// ── tolerant parse layer ────────────────────────────────────────────────────

/// The tolerant risk-facet layer: the two assessable axes as raw `String` (the
/// `"" -> None` seam), `origin` as raw `String` (empty → absent), `controls` a
/// free list.
#[derive(Debug, Deserialize)]
pub(crate) struct RawRiskFacet {
    #[serde(default)]
    pub(crate) likelihood: String,
    #[serde(default)]
    pub(crate) impact: String,
    #[serde(default)]
    pub(crate) origin: String,
    #[serde(default)]
    pub(crate) controls: Vec<String>,
}

// ── validated layer ─────────────────────────────────────────────────────────

/// The validated risk facet (risk only). Every axis typed — no untyped bag
/// (PRD-009 invariant). The assessable axes are optional until assessed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RiskFacet {
    pub(crate) likelihood: Option<RiskLevel>,
    pub(crate) impact: Option<RiskLevel>,
    pub(crate) origin: Option<String>,
    pub(crate) controls: Vec<String>,
}

// ── `"" -> None` parse seams (private copies; cf. knowledge.rs) ─────────────

/// Parse a kebab token into its closed enum via the serde derive — the single
/// source of the variant↔string mapping (the `as_str` mirrors render only).
/// Errors with serde's "unknown variant" message on a bad token (`what` names the
/// field for the message).
fn parse_enum<T: serde::de::DeserializeOwned>(token: &str, what: &str) -> anyhow::Result<T> {
    use serde::de::IntoDeserializer;
    let de: serde::de::value::StrDeserializer<'_, serde::de::value::Error> =
        token.into_deserializer();
    T::deserialize(de).map_err(|e| anyhow::anyhow!("invalid {what} `{token}`: {e}"))
}

/// The `"" -> None` seam for an optional closed enum: an empty token is absent; a
/// non-empty token parses to its variant (erroring on an unknown one).
fn optional_enum<T: serde::de::DeserializeOwned>(
    token: &str,
    what: &str,
) -> anyhow::Result<Option<T>> {
    if token.is_empty() {
        Ok(None)
    } else {
        parse_enum(token, what).map(Some)
    }
}

/// The `"" -> None` seam for an optional free-text field. Consumes the raw string.
fn optional_text(text: String) -> Option<String> {
    if text.is_empty() { None } else { Some(text) }
}

// ── parse optional ────────────────────────────────────────────────────────

/// Parse an optional `[facet]` table. Returns `Ok(None)` absent,
/// `Ok(Some(facet))` present+valid, `Err(_)` malformed. Bakes in validation —
/// callers never hold an invalid facet.
pub(crate) fn parse_optional(
    table: Option<&toml::value::Table>,
) -> anyhow::Result<Option<RiskFacet>> {
    let Some(table) = table else {
        return Ok(None);
    };
    let raw: RawRiskFacet = toml::from_str(&toml::to_string(table)?)?;
    let facet = validate_facet(raw)?;
    Ok(Some(facet))
}

// ── validation ──────────────────────────────────────────────────────────────

/// Validate a tolerant risk facet: the two axes through the `"" -> None` enum seam,
/// `origin` through the text seam, `controls` passed through.
pub(crate) fn validate_facet(raw: RawRiskFacet) -> anyhow::Result<RiskFacet> {
    Ok(RiskFacet {
        likelihood: optional_enum(&raw.likelihood, "likelihood")?,
        impact: optional_enum(&raw.impact, "impact")?,
        origin: optional_text(raw.origin),
        controls: raw.controls,
    })
}

// ── exposure ────────────────────────────────────────────────────────────────

/// The risk exposure score — `likelihood × impact` (1..=16) when BOTH axes are
/// assessed, else `0`. The within-level ordering fallback the `backlog_order`
/// adapter consumes (design §5.1 tier 3, VT-4): `0` is the baseline shared by
/// every non-risk item (a `None` facet) and every part-assessed risk alike —
/// assessment is all-or-nothing for ordering. Weights are Low=1 … Critical=4 (A3);
/// the product fits `u8`, no cast. The single derivation site — PHASE-03's
/// `project` reads it here, not a second copy (the PHASE-01 self-clearing dead-code
/// scope removed itself once `project` landed).
pub(crate) fn exposure(facet: Option<&RiskFacet>) -> u8 {
    const fn weight(level: RiskLevel) -> u8 {
        match level {
            RiskLevel::Low => 1,
            RiskLevel::Medium => 2,
            RiskLevel::High => 3,
            RiskLevel::Critical => 4,
        }
    }
    match facet.and_then(|f| f.likelihood.zip(f.impact)) {
        Some((l, i)) => weight(l) * weight(i),
        None => 0,
    }
}

// ── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // --- level map (was in backlog.rs:2370) ---

    #[test]
    fn risk_levels_map_to_correct_kebab_strings() {
        assert_eq!(RiskLevel::Low.as_str(), "low");
        assert_eq!(RiskLevel::Medium.as_str(), "medium");
        assert_eq!(RiskLevel::High.as_str(), "high");
        assert_eq!(RiskLevel::Critical.as_str(), "critical");
    }

    // --- as_str mirror matches parse (was in backlog.rs:2459 resolution_and_risk_level_render_mirror_serde, risk half) ---

    #[test]
    fn risk_level_render_mirror_serde() {
        assert_eq!(RiskLevel::Critical.as_str(), "critical");
        // the mirror matches the parse direction.
        assert_eq!(
            parse_enum::<RiskLevel>("critical", "risk-level").unwrap(),
            RiskLevel::Critical
        );
    }

    // --- exposure test helper (was in backlog.rs:3587) + exposure tests (VT-1) ---

    fn facet(likelihood: Option<RiskLevel>, impact: Option<RiskLevel>) -> RiskFacet {
        RiskFacet {
            likelihood,
            impact,
            origin: None,
            controls: Vec::new(),
        }
    }

    #[test]
    fn exposure_scores_a_fully_assessed_risk() {
        use RiskLevel::{Critical, High, Low};
        assert_eq!(exposure(Some(&facet(Some(High), Some(Critical)))), 12);
        assert_eq!(exposure(Some(&facet(Some(Low), Some(Low)))), 1);
        assert_eq!(exposure(Some(&facet(Some(Critical), Some(Critical)))), 16);
    }

    #[test]
    fn exposure_is_baseline_when_unassessed_or_non_risk() {
        use RiskLevel::High;
        // one axis only → baseline.
        assert_eq!(exposure(Some(&facet(Some(High), None))), 0);
        assert_eq!(exposure(Some(&facet(None, Some(High)))), 0);
        // no axis → baseline.
        assert_eq!(exposure(Some(&facet(None, None))), 0);
        // non-risk item (no facet) → baseline.
        assert_eq!(exposure(None), 0);
    }

    // --- parse_optional ---

    fn facet_table_from(s: &str) -> toml::value::Table {
        s.parse::<toml::Table>().unwrap()
    }

    #[test]
    fn parse_optional_absent_is_none() {
        let result = parse_optional(None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn parse_optional_valid_facet_is_some() {
        let t = facet_table_from("likelihood = \"low\"\nimpact = \"medium\"");
        let facet = parse_optional(Some(&t)).unwrap().unwrap();
        assert_eq!(facet.likelihood, Some(RiskLevel::Low));
        assert_eq!(facet.impact, Some(RiskLevel::Medium));
        assert!(facet.origin.is_none());
        assert!(facet.controls.is_empty());
    }

    #[test]
    fn parse_optional_malformed_is_err() {
        let t = facet_table_from("likelihood = \"bogus\"\nimpact = \"high\"");
        let err = parse_optional(Some(&t)).unwrap_err().to_string();
        assert!(err.contains("invalid likelihood"), "got: {err}");
    }
}
