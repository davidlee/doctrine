// SPDX-License-Identifier: GPL-3.0-only
//! `dtoml` — THE single shared `doctrine.toml` reader (SL-057 PHASE-02, design D2).
//!
//! One parser owns the whole `doctrine.toml` shape so the file is read once and
//! split into its sub-configs: the `[conduct]` table ([`crate::conduct`]) and the
//! `[verification]` table ([`crate::verify`]), plus the `[estimation]` and
//! `[value]` tables. Every field is `#[serde(default)]`, so an absent table
//! parses to its sub-config's default (tolerant — the conduct precedent). Every
//! other top-level key is ignored.
//!
//! **Layering (ADR-001).** [`parse`] is the pure leaf — owned text in, no IO.
//! [`load_doctrine_toml`] is the one thin impure shell seam co-located here (read
//! file → `parse` → absent ⇒ default), so every consumer shares a single reader.

use anyhow::Context;
use serde::Deserialize;
use std::path::Path;

/// The outer `doctrine.toml` shape — the union of every read sub-config. Absent
/// tables fall to their sub-config defaults (`#[serde(default)]`); unknown
/// top-level keys are ignored (tolerant parse).
#[derive(Debug, Default, Deserialize)]
pub(crate) struct DoctrineToml {
    /// The `[conduct]` table — LIVE (consumed by [`crate::conduct::parse`]).
    #[serde(default)]
    pub(crate) conduct: crate::conduct::ConductConfig,
    /// The `[verification]` table — consumed by the verifier + the record handler
    /// (SL-057 PHASE-05) through the shared `coverage_store::load_config` reader.
    #[serde(default)]
    pub(crate) verification: crate::verify::VerificationConfig,
    /// The `[estimation]` table — project-wide display/default unit + confidence
    /// bounds for estimation facets. The unit is resolved by the catalog shell
    /// (`scan_catalog`) into the top-level `Units` block (SL-103 PHASE-02).
    #[serde(default)]
    pub(crate) estimation: crate::estimate::EstimationConfig,
    /// The `[value]` table — project-wide display/default unit for value facets.
    /// The unit is resolved by the catalog shell into `Units` (SL-103 PHASE-02).
    #[serde(default)]
    pub(crate) value: crate::value::ValueConfig,
    /// The `[dispatch]` table — consumed by the dispatch orchestrator to select
    /// the spawn arm (SL-108 design D3 / IMP-101).
    #[serde(default)]
    pub(crate) dispatch: crate::dispatch_config::DispatchConfig,
    /// The `[install]` table — parameterises the printed post-install plugin /
    /// npx-skills instructions (SL-152 PHASE-06).
    #[serde(default)]
    pub(crate) install: crate::install_config::InstallConfig,
}

/// Parse a project `doctrine.toml` body into its sub-configs (PURE). The shell
/// owns the file read; this is the ONLY `doctrine.toml` parser.
pub(crate) fn parse(text: &str) -> anyhow::Result<DoctrineToml> {
    // Design §3.3: confidence bounds are "purely informational until consumed" —
    // no runtime effect in this slice. We deliberately do NOT eagerly validate
    // [estimation] here: parse() is the shared reader for conduct, verification,
    // and coverage_store config, so propagating confidence validation would
    // couple those unrelated reads to estimation-config validity. Consumers that
    // need the bounds call `estimate::resolve_confidence` themselves.
    let doc: DoctrineToml = toml::from_str(text)?;
    Ok(doc)
}

/// Parse an entity TOML with canonical-id error context.
///
/// Wraps `toml::from_str`. On parse failure, injects the entity's canonical
/// id so the user sees which entity is broken. The raw `toml` error already
/// describes *what* went wrong.
///
/// Pure leaf (ADR-001): owned text in, no IO, no config dependency.
pub(crate) fn parse_entity_toml<T: serde::de::DeserializeOwned>(
    text: &str,
    prefix: &str,
    id: u32,
) -> anyhow::Result<T> {
    toml::from_str(text).with_context(|| format!("{prefix}-{id:03}: TOML parse failed"))
}

/// The project config filename — lives under `.doctrine/`, the single
/// canonical home for project-local config (ISS-055).
pub(crate) const DOCTRINE_TOML: &str = ".doctrine/doctrine.toml";

/// Read the raw `doctrine.toml` body at `root` (IMPURE shell seam) — `None` when
/// the file is absent (a genuine read error still surfaces). The single file-read
/// seam shared by [`load_doctrine_toml`] and any consumer that projects its own
/// section out-of-band of [`DoctrineToml`] (SL-148 `reserve`: keeps `[reservation]`
/// parsing inside the engine-tier consumer so no `leaf → engine` import is forced).
pub(crate) fn read_doctrine_toml_text(root: &Path) -> anyhow::Result<Option<String>> {
    let path = root.join(DOCTRINE_TOML);
    match std::fs::read_to_string(&path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("Failed to read {}", path.display())),
    }
}

/// Read + parse the project `doctrine.toml` (IMPURE shell seam, ADR-001).
/// Absent file -> `DoctrineToml::default()`; present -> tolerant [`parse`];
/// genuinely malformed TOML errors with context. The single reader shared by
/// the close-integration gate, the sync handler, the `deliver-to` verb, and
/// `load_conduct`.
pub(crate) fn load_doctrine_toml(root: &Path) -> anyhow::Result<DoctrineToml> {
    match read_doctrine_toml_text(root)? {
        Some(text) => parse(&text)
            .with_context(|| format!("Failed to parse {}", root.join(DOCTRINE_TOML).display())),
        None => Ok(DoctrineToml::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_tables_yield_defaults() {
        let doc = parse("").unwrap();
        assert_eq!(doc.conduct, crate::conduct::ConductConfig::default());
        assert_eq!(
            doc.verification,
            crate::verify::VerificationConfig::default()
        );
        assert_eq!(doc.estimation, crate::estimate::EstimationConfig::default());
        assert_eq!(doc.value, crate::value::ValueConfig::default());
        assert_eq!(
            doc.dispatch,
            crate::dispatch_config::DispatchConfig::default()
        );
        assert_eq!(doc.install, crate::install_config::InstallConfig::default());
    }

    #[test]
    fn install_table_roundtrip() {
        // SL-152 PHASE-06: the [install] repo survives the full DoctrineToml parse;
        // absent → default davidlee/doctrine.
        let doc = parse("[install]\nrepo = \"acme/doctrine\"\n").unwrap();
        assert_eq!(doc.install.repo, "acme/doctrine");
        let doc2 = parse("").unwrap();
        assert_eq!(doc2.install.repo, "davidlee/doctrine");
    }

    #[test]
    fn estimation_and_value_tables_parse() {
        let doc = parse("[estimation]\nunit=\"x\"\n[value]\nunit=\"y\"").unwrap();
        assert_eq!(doc.estimation.unit.as_deref(), Some("x"));
        assert_eq!(doc.value.unit.as_deref(), Some("y"));
    }

    #[test]
    fn dispatch_table_roundtrip() {
        // The full round-trip through the shared dtoml::parse — not just the
        // DispatchConfig unit tests. Prove a populated [dispatch] survives the
        // outer TOML deserialize, and that a missing key within [dispatch]
        // defaults to codex.
        let doc = parse("[dispatch]\npreferred-subprocess-harness = \"pi\"\n").unwrap();
        use crate::dispatch_config::SubprocessHarness;
        assert_eq!(
            doc.dispatch.preferred_subprocess_harness,
            SubprocessHarness::Pi
        );
        // [dispatch] present but key absent → default (codex)
        let doc2 = parse("[dispatch]\n").unwrap();
        assert_eq!(
            doc2.dispatch.preferred_subprocess_harness,
            SubprocessHarness::Codex
        );
    }

    #[test]
    fn dispatch_deliver_to_roundtrip() {
        // VT-3: deliver-to survives the full DoctrineToml parse.
        let doc = parse("[dispatch]\ndeliver-to = \"refs/heads/release\"\n").unwrap();
        assert_eq!(doc.dispatch.deliver_to, "refs/heads/release");
        let doc2 = parse("[dispatch]\n").unwrap();
        assert_eq!(doc2.dispatch.deliver_to, "refs/heads/main");
    }

    #[test]
    fn dispatch_table_combined_keys() {
        // SL-117: prove both dispatch keys survive the full dtoml::parse round-trip.
        let doc =
            parse("[dispatch]\npreferred-subprocess-harness = \"pi\"\nclaude-force-subprocess-dispatch = true\n")
                .unwrap();
        use crate::dispatch_config::SubprocessHarness;
        assert_eq!(
            doc.dispatch.preferred_subprocess_harness,
            SubprocessHarness::Pi
        );
        assert!(doc.dispatch.claude_force_subprocess_dispatch);
    }

    // RV-085 F-1 regression: a malformed [estimation] confidence config must NOT
    // fail the shared config read. parse() is the reader for conduct, verification,
    // and coverage_store; coupling those to estimation validity violates design §3.3
    // ("no runtime effect in this slice"). Confidence validation belongs to the
    // consumer that needs the bounds, not to every doctrine.toml read.
    #[test]
    fn malformed_estimation_confidence_does_not_block_config_read() {
        let doc = parse("[estimation]\nlower_confidence=0.5\nupper_confidence=0.3\n[conduct]\n")
            .expect("malformed estimation confidence must not block the shared config read");
        // estimation table is still parsed (tolerated); only eager validation is gone
        assert_eq!(doc.estimation.lower_confidence, Some(0.5));
        assert_eq!(doc.estimation.upper_confidence, Some(0.3));
    }
}
