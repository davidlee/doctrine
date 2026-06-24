// SPDX-License-Identifier: GPL-3.0-only
//! `config` — the `[priority]` section of `doctrine.toml` (SL-133 PHASE-03).
//!
//! Declares the project's priority scoring coefficients: per-kind weights,
//! per-tag coefficients, value/risk/consequence multipliers. Purely advisory —
//! `load` never errors, silently clamping every out-of-bounds coefficient to a
//! safe finite range so downstream products stay bounded (no NaN poison).
//! Contrast `dispatch_config`, which deliberately hard-errors on malformed input.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

/// Cap all coefficients so downstream products stay finite.
/// NaN / +/-inf clamp to the field-specific default; negatives → 0.0;
/// values above this → `COEFF_MAX`.
pub(crate) const COEFF_MAX: f64 = 1e9;

// ── sub-structs ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct Coefficients {
    #[serde(default = "default_value_coeff")]
    pub(crate) value: f64,
    #[serde(default = "default_risk_coeff")]
    pub(crate) risk: f64,
}

impl Default for Coefficients {
    fn default() -> Self {
        Self {
            value: 1.0,
            risk: 2.0,
        }
    }
}

fn default_value_coeff() -> f64 {
    1.0
}
fn default_risk_coeff() -> f64 {
    2.0
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct ConsequenceCoeffs {
    #[serde(default = "default_dep_coeff")]
    pub(crate) dep_coeff: f64,
    #[serde(default = "default_ref_coeff")]
    pub(crate) ref_coeff: f64,
}

impl Default for ConsequenceCoeffs {
    fn default() -> Self {
        Self {
            dep_coeff: 0.5,
            ref_coeff: 1.0,
        }
    }
}

fn default_dep_coeff() -> f64 {
    0.5
}
fn default_ref_coeff() -> f64 {
    1.0
}

// ── top-level config ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub(crate) struct PriorityConfig {
    #[serde(default)]
    pub(crate) coefficients: Coefficients,
    #[serde(default)]
    pub(crate) kind_weights: BTreeMap<String, f64>,
    #[serde(default)]
    pub(crate) tag_coefficients: BTreeMap<String, f64>,
    #[serde(default)]
    pub(crate) consequence: ConsequenceCoeffs,
}

// ── accessors ─────────────────────────────────────────────────────────────

impl PriorityConfig {
    /// Look up the weight for a given kind string; returns 1.0 when absent.
    pub(crate) fn kind_weight(&self, kind: &str) -> f64 {
        self.kind_weights.get(kind).copied().unwrap_or(1.0)
    }

    /// Look up the coefficient for a given tag string; returns 1.0 when absent.
    pub(crate) fn tag_coeff(&self, tag: &str) -> f64 {
        self.tag_coefficients.get(tag).copied().unwrap_or(1.0)
    }
}

// ── load (impure shell) ──────────────────────────────────────────────────

/// Read `<root>/doctrine.toml`, deserialise the `[priority]` section, and clamp
/// every coefficient to a safe finite range. NEVER errors — absent file, missing
/// section, and malformed values all silently fall back to defaults.
pub(crate) fn load(root: &Path) -> PriorityConfig {
    let Some(table) = read_priority_table(root) else {
        return PriorityConfig::default();
    };
    load_from_table(&table)
}

pub(crate) fn read_priority_table(root: &Path) -> Option<toml::Table> {
    let text = std::fs::read_to_string(root.join(".doctrine/doctrine.toml")).ok()?;
    let raw: toml::Value = text.parse().ok()?;
    raw.get("priority")?.as_table().cloned()
}

pub(crate) fn load_from_table(table: &toml::value::Table) -> PriorityConfig {
    let mut cfg = PriorityConfig::default();

    if let Some(t) = table.get("coefficients").and_then(|v| v.as_table()) {
        cfg.coefficients.value = f64_or(t, "value", 1.0);
        cfg.coefficients.risk = f64_or(t, "risk", 2.0);
    }
    if let Some(t) = table.get("consequence").and_then(|v| v.as_table()) {
        cfg.consequence.dep_coeff = f64_or(t, "dep_coeff", 0.5);
        cfg.consequence.ref_coeff = f64_or(t, "ref_coeff", 1.0);
    }
    if let Some(t) = table.get("kind_weights").and_then(|v| v.as_table()) {
        for (k, v) in t {
            if let Some(f) = f64_val(v) {
                cfg.kind_weights.insert(k.clone(), f);
            }
        }
    }
    if let Some(t) = table.get("tag_coefficients").and_then(|v| v.as_table()) {
        for (k, v) in t {
            if let Some(f) = f64_val(v) {
                cfg.tag_coefficients.insert(k.clone(), f);
            }
        }
    }

    clamp(cfg)
}

/// Extract an f64 from a TOML value, accepting integers (TOML `3` → 3.0).
/// Returns `None` for strings, booleans, arrays, and other non-numeric types.
#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "i64→f64 safe for TOML config coefficients (never near i64::MAX)"
)]
fn f64_val(v: &toml::Value) -> Option<f64> {
    v.as_float().or_else(|| v.as_integer().map(|i| i as f64))
}

fn f64_or(table: &toml::value::Table, key: &str, default: f64) -> f64 {
    table.get(key).and_then(f64_val).unwrap_or(default)
}

// ── clamping ──────────────────────────────────────────────────────────────

/// Clamp every coefficient in-place so downstream products stay finite.
/// NaN / inf → field default; negative → 0.0; > `COEFF_MAX` → `COEFF_MAX`.
/// `dep_coeff` is tighter: (0, 1].
fn clamp(mut cfg: PriorityConfig) -> PriorityConfig {
    // General coefficients: value, risk, ref_coeff
    cfg.coefficients.value = clamp_general(cfg.coefficients.value, 1.0);
    cfg.coefficients.risk = clamp_general(cfg.coefficients.risk, 2.0);
    cfg.consequence.ref_coeff = clamp_general(cfg.consequence.ref_coeff, 1.0);

    // dep_coeff: (0, 1]
    cfg.consequence.dep_coeff = clamp_dep(cfg.consequence.dep_coeff);

    // kind_weights and tag_coefficients: clamp each value
    for v in cfg.kind_weights.values_mut() {
        *v = clamp_general(*v, 1.0);
    }
    for v in cfg.tag_coefficients.values_mut() {
        *v = clamp_general(*v, 1.0);
    }

    cfg
}

/// General coefficient clamp: non-finite → fallback; negative → 0.0; > `COEFF_MAX` → `COEFF_MAX`.
pub(crate) fn clamp_general(value: f64, fallback: f64) -> f64 {
    if !value.is_finite() {
        return fallback;
    }
    if value < 0.0 {
        return 0.0;
    }
    if value > COEFF_MAX {
        return COEFF_MAX;
    }
    value
}

/// Dep-coeff clamp: non-finite → fallback (0.5); ≤ 0 → 0.0; > 1 → 1.0.
pub(crate) fn clamp_dep(value: f64) -> f64 {
    if !value.is_finite() {
        return 0.5;
    }
    if value <= 0.0 {
        return 0.0;
    }
    if value > 1.0 {
        return 1.0;
    }
    value
}

// ── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Write a `doctrine.toml` into `root` and call `load(root)`.
    fn load_from(body: &str) -> PriorityConfig {
        let dir = tempfile::tempdir().unwrap();
        let config_dir = dir.path().join(".doctrine");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("doctrine.toml"), body).unwrap();
        load(dir.path())
    }

    // ---- absent / missing ----

    #[test]
    fn missing_priority_section_is_defaults() {
        let cfg = load_from("[dispatch]\npreferred-subprocess-harness = \"pi\"\n");
        assert_eq!(cfg.coefficients.value, 1.0);
        assert_eq!(cfg.coefficients.risk, 2.0);
        assert_eq!(cfg.consequence.dep_coeff, 0.5);
        assert_eq!(cfg.consequence.ref_coeff, 1.0);
        assert!(cfg.kind_weights.is_empty());
        assert!(cfg.tag_coefficients.is_empty());
    }

    #[test]
    fn no_doctrine_toml_is_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = load(dir.path());
        assert_eq!(cfg.coefficients.value, 1.0);
        assert_eq!(cfg.coefficients.risk, 2.0);
    }

    // ---- partial section — per-field defaults ----

    #[test]
    fn partial_section_fills_defaults() {
        let cfg = load_from("[priority]\nkind_weights = { SL = 2.5 }\n");
        assert_eq!(cfg.coefficients.value, 1.0); // missing → default
        assert_eq!(cfg.coefficients.risk, 2.0); // missing → default
        assert_eq!(cfg.consequence.dep_coeff, 0.5); // missing → default
        assert_eq!(cfg.consequence.ref_coeff, 1.0); // missing → default
        assert_eq!(cfg.kind_weight("SL"), 2.5);
        assert_eq!(cfg.kind_weight("ADR"), 1.0); // absent → default
        assert!(cfg.tag_coefficients.is_empty());
    }

    // ---- unknown key ignored ----

    #[test]
    fn unknown_key_ignored() {
        let cfg = load_from("[priority]\ncoefficients = { value = 3.0, risk = 4.0, extra = 99 }\n");
        assert_eq!(cfg.coefficients.value, 3.0);
        assert_eq!(cfg.coefficients.risk, 4.0);
        // extra key is silently ignored by serde(ignore_unknown)
    }

    // ---- non-finite → default ----

    #[test]
    fn nan_coefficient_clamps_to_default() {
        let cfg = load_from("[priority]\ncoefficients = { value = nan, risk = nan }\n");
        assert_eq!(cfg.coefficients.value, 1.0);
        assert_eq!(cfg.coefficients.risk, 2.0);
    }

    #[test]
    fn inf_coefficient_clamps_to_default() {
        let cfg = load_from("[priority]\ncoefficients = { value = inf, risk = -inf }\n");
        assert_eq!(cfg.coefficients.value, 1.0);
        assert_eq!(cfg.coefficients.risk, 2.0);
    }

    // ---- negative → 0.0 ----

    #[test]
    fn negative_coefficient_clamps_to_zero() {
        let cfg = load_from("[priority]\ncoefficients = { value = -5.0, risk = -0.1 }\n");
        assert_eq!(cfg.coefficients.value, 0.0);
        assert_eq!(cfg.coefficients.risk, 0.0);
    }

    // ---- over COEFF_MAX → COEFF_MAX ----

    #[test]
    fn over_max_coefficient_clamps_to_max() {
        let body = format!(
            "[priority]\ncoefficients = {{ value = {max}, risk = {max} }}\n",
            max = COEFF_MAX + 1.0
        );
        let cfg = load_from(&body);
        assert_eq!(cfg.coefficients.value, COEFF_MAX);
        assert_eq!(cfg.coefficients.risk, COEFF_MAX);
    }

    // ---- dep_coeff: > 1 → 1.0 ----

    #[test]
    fn dep_coeff_over_one_clamps_to_one() {
        let cfg = load_from("[priority]\nconsequence = { dep_coeff = 5.0 }\n");
        assert_eq!(cfg.consequence.dep_coeff, 1.0);
    }

    // ---- dep_coeff: ≤ 0 → 0.0 ----

    #[test]
    fn dep_coeff_zero_or_negative_clamps_to_zero() {
        let cfg = load_from("[priority]\nconsequence = { dep_coeff = 0.0 }\n");
        assert_eq!(cfg.consequence.dep_coeff, 0.0);

        let cfg2 = load_from("[priority]\nconsequence = { dep_coeff = -0.5 }\n");
        assert_eq!(cfg2.consequence.dep_coeff, 0.0);
    }

    // ---- malformed value clamps and load does NOT error ----

    #[test]
    fn malformed_toml_in_priority_section_returns_defaults() {
        // A missing closing bracket — malformed TOML in the [priority] value.
        let cfg = load_from("[priority]\ncoefficients = { value = 3.0\n");
        assert_eq!(cfg.coefficients.value, 1.0); // default
    }

    #[test]
    fn non_numeric_value_clamps_returns_defaults() {
        // A string where a number was expected — per-field isolation: only the
        // offending field falls back to its default; the sibling field survives.
        let cfg = load_from("[priority]\ncoefficients = { value = \"abc\", risk = 4.0 }\n");
        assert_eq!(cfg.coefficients.value, 1.0); // wrong-type → field default
        assert_eq!(cfg.coefficients.risk, 4.0); // preserved — per-field isolation
    }

    // ---- kind_weight / tag_coeff absent key returns 1.0 ----

    #[test]
    fn kind_weight_absent_key_returns_default_one() {
        let cfg = PriorityConfig::default();
        assert_eq!(cfg.kind_weight("NONEXISTENT"), 1.0);
    }

    #[test]
    fn tag_coeff_absent_key_returns_default_one() {
        let cfg = PriorityConfig::default();
        assert_eq!(cfg.tag_coeff("nonexistent"), 1.0);
    }

    // ---- kind_weight / tag_coeff present key returns stored value ----

    #[test]
    fn kind_weight_present_key_returns_stored() {
        let cfg = load_from("[priority]\nkind_weights = { SL = 3.0, ADR = 1.5 }\n");
        assert_eq!(cfg.kind_weight("SL"), 3.0);
        assert_eq!(cfg.kind_weight("ADR"), 1.5);
    }

    #[test]
    fn tag_coeff_present_key_returns_stored() {
        let cfg = load_from("[priority]\ntag_coefficients = { \"area:risk\" = 2.0 }\n");
        assert_eq!(cfg.tag_coeff("area:risk"), 2.0);
    }
}
