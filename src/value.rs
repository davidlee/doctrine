// SPDX-License-Identifier: GPL-3.0-only
//! `value` — the optional value facet (SL-101, SPEC-020 §3).
//!
//! A single finite `f64` magnitude parsed from an entity `[value]` TOML table.
//! The facet is kind-agnostic and optional — an entity may carry it, or not.
//!
//! **Pure engine tier (ADR-001).** No clock / disk / rng / git here.
//! `ValueConfig` is parsed by the shell from `doctrine.toml`; `resolve_unit`
//! is pure over owned config. File reads live in the command shell.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub(crate) const DEFAULT_VALUE_UNIT: &str = "magic_beans";

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Project-wide value config, parsed from `doctrine.toml [value]`.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
pub(crate) struct ValueConfig {
    #[serde(default)]
    pub unit: Option<String>,
}

/// Resolve the value unit. Pure over config — the file read is the shell's job.
/// Empty string falls back to default.
pub(crate) fn resolve_unit(cfg: &ValueConfig) -> String {
    match &cfg.unit {
        Some(unit) if !unit.is_empty() => unit.clone(),
        _ => DEFAULT_VALUE_UNIT.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Facet — the normalised value
// ---------------------------------------------------------------------------

/// The normalised value facet — a single finite f64 magnitude.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct ValueFacet {
    pub value: f64,
}

/// Deserialisation target before normalisation. `value` is a raw TOML value so
/// integers and floats both arrive, and non-finite values are caught.
#[derive(Debug, Clone, Deserialize)]
struct ValueRaw {
    value: Option<toml::Value>,
    /// `#[serde(flatten)]` on a `BTreeMap` collects every key NOT matching
    /// `value` — the Rust field name `_extra` is never a TOML key.
    /// This is the forward-compatibility mechanism (NF-003).
    #[serde(flatten)]
    _extra: BTreeMap<String, toml::Value>,
}

impl<'de> Deserialize<'de> for ValueFacet {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = ValueRaw::deserialize(d)?;
        normalise(raw).map_err(serde::de::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// Parse
// ---------------------------------------------------------------------------

/// Parse an optional `[value]` table. Returns `Ok(None)` absent,
/// `Ok(Some(facet))` present+valid, `Err(_)` malformed. Bakes in validation —
/// callers never hold an invalid facet.
pub(crate) fn parse_optional(
    table: Option<&toml::value::Table>,
) -> anyhow::Result<Option<ValueFacet>> {
    let Some(table) = table else {
        return Ok(None);
    };
    let raw: ValueRaw = toml::from_str(&toml::to_string(table)?)?;
    let facet = normalise(raw)?;
    Ok(Some(facet))
}

// ---------------------------------------------------------------------------
// Normalise
// ---------------------------------------------------------------------------

/// Normalise raw values to finite f64. Rejects missing values and non-finite
/// values. Pure — callers deserialise then call this.
fn normalise(raw: ValueRaw) -> anyhow::Result<ValueFacet> {
    let value = raw
        .value
        .ok_or_else(|| anyhow::anyhow!("value: value is required"))?;

    let facet = ValueFacet {
        value: toml_to_f64(&value, "value")?,
    };
    validate(&facet)?;
    Ok(facet)
}

/// Validate a present value. Pure. Sentence-case error.
pub(crate) fn validate(facet: &ValueFacet) -> anyhow::Result<()> {
    if !facet.value.is_finite() {
        anyhow::bail!("value: value must be finite");
    }
    Ok(())
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
        _ => anyhow::bail!("value: {name} must be a number"),
    };
    if !f.is_finite() {
        anyhow::bail!("value: {name} must be finite");
    }
    Ok(f)
}

/// Render a value facet line for `slice show`.
/// Output: `"value: {magnitude} {unit}"`
pub(crate) fn format_value_normal(facet: &ValueFacet, unit: &str) -> String {
    debug_assert!(!unit.is_empty());
    format!("value: {:.1} {}", facet.value, unit)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn table_from(s: &str) -> toml::value::Table {
        s.parse::<toml::Table>().unwrap()
    }

    #[test]
    fn v1_absent() {
        let result = parse_optional(None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn v2_integer_value() {
        let t = table_from("value=5");
        let facet = parse_optional(Some(&t)).unwrap().unwrap();
        assert_eq!(facet.value, 5.0);
    }

    #[test]
    fn v3_float_value() {
        let t = table_from("value=3.5");
        let facet = parse_optional(Some(&t)).unwrap().unwrap();
        assert_eq!(facet.value, 3.5);
    }

    #[test]
    fn v4_missing_value() {
        let t = table_from("unit=\"beans\"");
        let err = parse_optional(Some(&t)).unwrap_err().to_string();
        assert!(err.contains("value: value is required"), "got: {err}");
    }

    #[test]
    fn v5_nan_value() {
        let t = table_from("value=nan");
        let err = parse_optional(Some(&t)).unwrap_err().to_string();
        assert!(err.contains("value: value must be finite"), "got: {err}");
    }

    #[test]
    fn v5a_negative_finite() {
        let t = table_from("value=-5");
        let facet = parse_optional(Some(&t)).unwrap().unwrap();
        assert_eq!(facet.value, -5.0);
        // No range constraint — validate must also accept negative finite
        assert!(validate(&facet).is_ok());
    }

    #[test]
    fn v6_resolve_unit_default() {
        assert_eq!(resolve_unit(&ValueConfig::default()), "magic_beans");
    }

    #[test]
    fn v7_unknown_keys_tolerated() {
        let t = table_from("value=5\ncurrency=\"USD\"\nsource=\"guess\"");
        let facet = parse_optional(Some(&t)).unwrap().unwrap();
        assert_eq!(facet.value, 5.0);
        let serialised = toml::to_string(&facet).unwrap();
        assert!(
            !serialised.contains("currency"),
            "extra key leaked: {serialised}"
        );
        assert!(
            !serialised.contains("source"),
            "extra key leaked: {serialised}"
        );
    }

    #[test]
    fn custom_deserialize_valid() {
        let t = table_from("value=3");
        let s = toml::to_string(&t).unwrap();
        let facet: ValueFacet = toml::from_str(&s).unwrap();
        assert_eq!(facet.value, 3.0);
    }

    #[test]
    fn custom_deserialize_missing_field() {
        let t = table_from("unit=\"beans\"");
        let s = toml::to_string(&t).unwrap();
        let err = toml::from_str::<ValueFacet>(&s).unwrap_err();
        assert!(err.to_string().contains("value is required"));
    }

    #[test]
    fn custom_deserialize_unknown_keys() {
        let t = table_from("value=3\ncurrency=\"USD\"\nextra=42");
        let s = toml::to_string(&t).unwrap();
        let facet: ValueFacet = toml::from_str(&s).unwrap();
        assert_eq!(facet.value, 3.0);
        let s2 = toml::to_string(&facet).unwrap();
        assert!(!s2.contains("currency"));
        assert!(!s2.contains("extra"));
    }

    #[test]
    fn value_raw_absorbs_unknown_keys() {
        let t = table_from("value=1\nfoo=\"bar\"\nbaz=99");
        let raw: ValueRaw = toml::from_str(&toml::to_string(&t).unwrap()).unwrap();
        assert!(raw.value.is_some());
        assert!(raw._extra.contains_key("foo"));
        assert!(raw._extra.contains_key("baz"));
    }

    #[test]
    fn validate_accepts_finite() {
        assert!(validate(&ValueFacet { value: 3.5 }).is_ok());
    }

    #[test]
    fn validate_rejects_infinity() {
        let err = validate(&ValueFacet {
            value: f64::INFINITY,
        })
        .unwrap_err()
        .to_string();
        assert!(err.contains("finite"), "got: {}", err);
    }

    #[test]
    fn validate_rejects_nan() {
        let err = validate(&ValueFacet { value: f64::NAN })
            .unwrap_err()
            .to_string();
        assert!(err.contains("finite"), "got: {}", err);
    }
}
