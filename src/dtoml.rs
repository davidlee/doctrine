// SPDX-License-Identifier: GPL-3.0-only
//! `dtoml` — THE single shared `doctrine.toml` reader (SL-057 PHASE-02, design D2).
//!
//! One parser owns the whole `doctrine.toml` shape so the file is read once and
//! split into its sub-configs: the `[conduct]` table ([`crate::conduct`]) and the
//! `[verification]` table ([`crate::verify`]). Both fields `#[serde(default)]`, so
//! an absent table parses to its sub-config's default (tolerant — the conduct
//! precedent). Every other top-level key is ignored.
//!
//! **Pure leaf (ADR-001).** The file *read* lives in the shell; [`parse`] takes
//! owned text only.

use serde::Deserialize;

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
}

/// Parse a project `doctrine.toml` body into its sub-configs (PURE). The shell
/// owns the file read; this is the ONLY `doctrine.toml` parser.
pub(crate) fn parse(text: &str) -> anyhow::Result<DoctrineToml> {
    Ok(toml::from_str(text)?)
}
