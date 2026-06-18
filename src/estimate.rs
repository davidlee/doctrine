// SPDX-License-Identifier: GPL-3.0-only
//! `estimate` — the optional estimation facet (SL-101, SPEC-020 §3).
#![allow(
    dead_code,
    reason = "lifted in PHASE-03 when dtoml.rs wires config imports"
)]
//!
//! A bounded human-attention-burden claim: two finite `f64` bounds (`lower`/`upper`),
//! parsed from an entity `[estimate]` TOML table. The facet is kind-agnostic and
//! optional — an entity may carry it, or not.
//!
//! **Pure engine tier (ADR-001).** No clock / disk / rng / git here.
//! `EstimationConfig` is parsed by the shell from `doctrine.toml`; `resolve_unit`
//! and `resolve_confidence` are pure over owned config. File reads live in the
//! command shell.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};

pub(crate) mod display;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub(crate) const DEFAULT_ESTIMATION_UNIT: &str = "espresso_shots";
pub(crate) const DEFAULT_LOWER_CONFIDENCE: f64 = 0.1;
pub(crate) const DEFAULT_UPPER_CONFIDENCE: f64 = 0.9;

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Project-wide estimation config, parsed from `doctrine.toml [estimation]`.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
pub(crate) struct EstimationConfig {
    #[serde(default)]
    pub unit: Option<String>,
    /// Default confidence bounds for display / Monte Carlo / downstream use.
    /// Stored as fractions in [0.0, 1.0]; validated: finite, in range, lower < upper.
    /// No runtime effect in this slice — purely informational until consumed.
    #[serde(default)]
    pub lower_confidence: Option<f64>,
    #[serde(default)]
    pub upper_confidence: Option<f64>,
}

/// Resolve the estimation unit. Pure over config — the file read is the shell's
/// job. Empty string falls back to default.
pub(crate) fn resolve_unit(cfg: &EstimationConfig) -> String {
    match &cfg.unit {
        Some(u) if !u.is_empty() => u.clone(),
        _ => DEFAULT_ESTIMATION_UNIT.to_string(),
    }
}

/// Resolve the default confidence bounds. Pure. Each bound falls back to its
/// default when absent; validated: finite, in [0.0, 1.0], lower < upper.
pub(crate) fn resolve_confidence(cfg: &EstimationConfig) -> anyhow::Result<(f64, f64)> {
    let lower = cfg.lower_confidence.unwrap_or(DEFAULT_LOWER_CONFIDENCE);
    let upper = cfg.upper_confidence.unwrap_or(DEFAULT_UPPER_CONFIDENCE);

    if !lower.is_finite() {
        anyhow::bail!("lower_confidence must be finite");
    }
    if !upper.is_finite() {
        anyhow::bail!("upper_confidence must be finite");
    }
    if !(0.0..=1.0).contains(&lower) {
        anyhow::bail!("lower_confidence must be in [0.0, 1.0]");
    }
    if !(0.0..=1.0).contains(&upper) {
        anyhow::bail!("upper_confidence must be in [0.0, 1.0]");
    }
    if lower >= upper {
        anyhow::bail!("upper_confidence must be > lower_confidence");
    }

    Ok((lower, upper))
}

// ---------------------------------------------------------------------------
// Facet — the normalised estimate
// ---------------------------------------------------------------------------

/// The normalised estimation facet — two finite f64 bounds.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct EstimateFacet {
    pub lower: f64,
    pub upper: f64,
}

/// Deserialisation target before normalisation. `lower`/`upper` are raw TOML
/// values so integers and floats both arrive, and non-finite values are caught.
#[derive(Debug, Clone, Deserialize)]
struct EstimateRaw {
    lower: Option<toml::Value>,
    upper: Option<toml::Value>,
    /// `#[serde(flatten)]` on a `BTreeMap` collects every key NOT matching
    /// `lower`/`upper` — the Rust field name `_extra` is never a TOML key.
    /// This is the forward-compatibility mechanism (NF-003).
    #[serde(flatten)]
    _extra: BTreeMap<String, toml::Value>,
}

impl<'de> Deserialize<'de> for EstimateFacet {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = EstimateRaw::deserialize(d)?;
        normalise(raw).map_err(serde::de::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// Parse
// ---------------------------------------------------------------------------

/// Parse an optional `[estimate]` table. Returns `Ok(None)` absent,
/// `Ok(Some(facet))` present+valid, `Err(_)` malformed. Bakes in validation —
/// callers never hold an invalid facet.
pub(crate) fn parse_optional(
    table: Option<&toml::value::Table>,
) -> anyhow::Result<Option<EstimateFacet>> {
    let Some(table) = table else {
        return Ok(None);
    };
    let raw: EstimateRaw = toml::from_str(&toml::to_string(table)?)?;
    let facet = normalise(raw)?;
    Ok(Some(facet))
}

// ---------------------------------------------------------------------------
// Normalise
// ---------------------------------------------------------------------------

/// Normalise raw values to finite f64. Rejects missing bounds and non-finite
/// values. Pure — callers deserialise then call this.
fn normalise(raw: EstimateRaw) -> anyhow::Result<EstimateFacet> {
    let lower_val = raw
        .lower
        .ok_or_else(|| anyhow::anyhow!("estimate: lower is required"))?;
    let upper_val = raw
        .upper
        .ok_or_else(|| anyhow::anyhow!("estimate: upper is required"))?;

    let lower = toml_to_f64(&lower_val, "lower")?;
    let upper = toml_to_f64(&upper_val, "upper")?;

    let facet = EstimateFacet { lower, upper };
    validate(&facet)?;
    Ok(facet)
}

/// Convert a `toml::Value` to `f64`. Accepts `Integer` and `Float`; rejects
/// everything else as a parse-time type error.
fn toml_to_f64(value: &toml::Value, name: &str) -> anyhow::Result<f64> {
    let f = match value {
        #[expect(
            clippy::cast_precision_loss,
            clippy::as_conversions,
            reason = "integer <= 2^53 fits exactly in f64"
        )]
        toml::Value::Integer(i) => *i as f64,
        toml::Value::Float(f) => *f,
        _ => anyhow::bail!("estimate: {name} must be a number"),
    };
    if !f.is_finite() {
        anyhow::bail!("estimate: {name} must be finite");
    }
    Ok(f)
}

// ---------------------------------------------------------------------------
// Validate
// ---------------------------------------------------------------------------

/// Validate a present estimate. Pure. Violations produce sentence-case errors.
pub(crate) fn validate(facet: &EstimateFacet) -> anyhow::Result<()> {
    if facet.lower < 0.0 {
        anyhow::bail!("estimate: lower must be >= 0");
    }
    if facet.upper < facet.lower {
        anyhow::bail!(
            "estimate: upper must be >= lower (got lower={}, upper={})",
            facet.lower,
            facet.upper
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: build a TOML table from a string like "lower=2\nupper=8"
    fn table_from(s: &str) -> toml::value::Table {
        s.parse::<toml::Table>().unwrap()
    }

    // ---- E1: absent table ----
    #[test]
    fn e1_absent() {
        let result = parse_optional(None).unwrap();
        assert!(result.is_none());
    }

    // ---- E2: integer bounds ----
    #[test]
    fn e2_integer_bounds() {
        let t = table_from("lower=2\nupper=8");
        let facet = parse_optional(Some(&t)).unwrap().unwrap();
        assert_eq!(facet.lower, 2.0);
        assert_eq!(facet.upper, 8.0);
    }

    // ---- E3: float bounds ----
    #[test]
    fn e3_float_bounds() {
        let t = table_from("lower=2.5\nupper=8.0");
        let facet = parse_optional(Some(&t)).unwrap().unwrap();
        assert_eq!(facet.lower, 2.5);
        assert_eq!(facet.upper, 8.0);
    }

    // ---- E4: zero-width estimate ----
    #[test]
    fn e4_zero_width() {
        let t = table_from("lower=2\nupper=2");
        let facet = parse_optional(Some(&t)).unwrap().unwrap();
        assert_eq!(facet.lower, 2.0);
        assert_eq!(facet.upper, 2.0);
    }

    // ---- E5: missing lower ----
    #[test]
    fn e5_missing_lower() {
        let t = table_from("upper=8");
        let err = parse_optional(Some(&t)).unwrap_err().to_string();
        assert!(err.contains("lower is required"), "got: {}", err);
    }

    // ---- E6: missing upper ----
    #[test]
    fn e6_missing_upper() {
        let t = table_from("lower=2");
        let err = parse_optional(Some(&t)).unwrap_err().to_string();
        assert!(err.contains("upper is required"), "got: {}", err);
    }

    // ---- E7: lower = nan ----
    #[test]
    fn e7_nan_lower() {
        let t = table_from("lower=nan\nupper=8");
        let err = parse_optional(Some(&t)).unwrap_err().to_string();
        assert!(err.contains("lower must be finite"), "got: {}", err);
    }

    // ---- E8: lower = -inf ----
    #[test]
    fn e8_neg_inf_lower() {
        let t = table_from("lower=-inf\nupper=8");
        let err = parse_optional(Some(&t)).unwrap_err().to_string();
        assert!(err.contains("lower must be finite"), "got: {}", err);
    }

    // ---- E9: lower = inf ----
    #[test]
    fn e9_inf_lower() {
        let t = table_from("lower=inf\nupper=8");
        let err = parse_optional(Some(&t)).unwrap_err().to_string();
        assert!(err.contains("lower must be finite"), "got: {}", err);
    }

    // ---- E10: lower negative ----
    #[test]
    fn e10_negative_lower() {
        let t = table_from("lower=-1\nupper=8");
        let err = parse_optional(Some(&t)).unwrap_err().to_string();
        assert!(err.contains("lower must be >= 0"), "got: {}", err);
    }

    // ---- E11: upper < lower ----
    #[test]
    fn e11_upper_lt_lower() {
        let t = table_from("lower=5\nupper=2");
        let err = parse_optional(Some(&t)).unwrap_err().to_string();
        assert!(err.contains("upper must be >= lower"), "got: {}", err);
    }

    // ---- E12: resolve_unit default ----
    #[test]
    fn e12_resolve_unit_default() {
        let unit = resolve_unit(&EstimationConfig::default());
        assert_eq!(unit, "espresso_shots");
    }

    // ---- E13: resolve_unit custom ----
    #[test]
    fn e13_resolve_unit_custom() {
        let cfg = EstimationConfig {
            unit: Some("story_points".into()),
            ..Default::default()
        };
        let unit = resolve_unit(&cfg);
        assert_eq!(unit, "story_points");
    }

    // ---- E14: resolve_unit empty string ----
    #[test]
    fn e14_resolve_unit_empty() {
        let cfg = EstimationConfig {
            unit: Some(String::new()),
            ..Default::default()
        };
        let unit = resolve_unit(&cfg);
        assert_eq!(unit, "espresso_shots");
    }

    // ---- E15: resolve_confidence default ----
    #[test]
    fn e15_resolve_confidence_default() {
        let (l, u) = resolve_confidence(&EstimationConfig::default()).unwrap();
        assert_eq!(l, 0.1);
        assert_eq!(u, 0.9);
    }

    // ---- E15a: resolve_confidence custom ----
    #[test]
    fn e15a_resolve_confidence_custom() {
        let cfg = EstimationConfig {
            lower_confidence: Some(0.2),
            upper_confidence: Some(0.8),
            ..Default::default()
        };
        let (l, u) = resolve_confidence(&cfg).unwrap();
        assert_eq!(l, 0.2);
        assert_eq!(u, 0.8);
    }

    // ---- E15b: resolve_confidence nan ----
    #[test]
    fn e15b_resolve_confidence_nan() {
        let cfg = EstimationConfig {
            lower_confidence: Some(f64::NAN),
            ..Default::default()
        };
        let err = resolve_confidence(&cfg).unwrap_err().to_string();
        assert!(
            err.contains("lower_confidence must be finite"),
            "got: {}",
            err
        );
    }

    // ---- E15c: resolve_confidence upper <= lower ----
    #[test]
    fn e15c_resolve_confidence_upper_le_lower() {
        let cfg = EstimationConfig {
            lower_confidence: Some(0.5),
            upper_confidence: Some(0.3),
            ..Default::default()
        };
        let err = resolve_confidence(&cfg).unwrap_err().to_string();
        assert!(
            err.contains("upper_confidence must be > lower_confidence"),
            "got: {}",
            err
        );
    }

    // ---- E15d: resolve_confidence lower out of range ----
    #[test]
    fn e15d_resolve_confidence_lower_out_of_range() {
        let cfg = EstimationConfig {
            lower_confidence: Some(-0.1),
            ..Default::default()
        };
        let err = resolve_confidence(&cfg).unwrap_err().to_string();
        assert!(
            err.contains("lower_confidence must be in [0.0, 1.0]"),
            "got: {}",
            err
        );
    }

    // ---- E19: unknown keys tolerated (NF-003) ----
    #[test]
    fn e19_unknown_keys_tolerated() {
        let t = table_from("lower=2\nupper=8\nmode=\"pert\"");
        let facet = parse_optional(Some(&t)).unwrap().unwrap();
        assert_eq!(facet.lower, 2.0);
        assert_eq!(facet.upper, 8.0);
        // Serialised form must NOT contain the extra key
        let serialised = toml::to_string(&facet).unwrap();
        assert!(
            !serialised.contains("mode"),
            "extra key leaked: {}",
            serialised
        );
    }

    // ---- Extra: direct validate tests ----
    #[test]
    fn validate_accepts_valid() {
        let f = EstimateFacet {
            lower: 1.0,
            upper: 5.0,
        };
        assert!(validate(&f).is_ok());
    }

    #[test]
    fn validate_rejects_negative_lower() {
        let f = EstimateFacet {
            lower: -1.0,
            upper: 5.0,
        };
        let err = validate(&f).unwrap_err().to_string();
        assert!(err.contains("lower must be >= 0"));
    }

    #[test]
    fn validate_rejects_upper_lt_lower() {
        let f = EstimateFacet {
            lower: 5.0,
            upper: 2.0,
        };
        let err = validate(&f).unwrap_err().to_string();
        assert!(err.contains("upper must be >= lower"));
    }

    // ---- Custom Deserialize direct ----
    #[test]
    fn custom_deserialize_valid() {
        let t = table_from("lower=3\nupper=7");
        let s = toml::to_string(&t).unwrap();
        let facet: EstimateFacet = toml::from_str(&s).unwrap();
        assert_eq!(facet.lower, 3.0);
        assert_eq!(facet.upper, 7.0);
    }

    #[test]
    fn custom_deserialize_missing_field() {
        let t = table_from("lower=3");
        let s = toml::to_string(&t).unwrap();
        let err = toml::from_str::<EstimateFacet>(&s).unwrap_err();
        assert!(err.to_string().contains("upper is required"));
    }

    #[test]
    fn custom_deserialize_unknown_keys() {
        let t = table_from("lower=3\nupper=7\nmode=\"pert\"\nextra=42");
        let s = toml::to_string(&t).unwrap();
        let facet: EstimateFacet = toml::from_str(&s).unwrap();
        assert_eq!(facet.lower, 3.0);
        assert_eq!(facet.upper, 7.0);
        // Serialise round-trip: extra keys dropped
        let s2 = toml::to_string(&facet).unwrap();
        assert!(!s2.contains("mode"));
        assert!(!s2.contains("extra"));
    }

    #[test]
    fn estimate_raw_absorbs_unknown_keys() {
        let t = table_from("lower=1\nupper=2\nfoo=\"bar\"\nbaz=99");
        let raw: EstimateRaw = toml::from_str(&toml::to_string(&t).unwrap()).unwrap();
        assert!(raw.lower.is_some());
        assert!(raw.upper.is_some());
        assert!(raw._extra.contains_key("foo"));
        assert!(raw._extra.contains_key("baz"));
    }
}
