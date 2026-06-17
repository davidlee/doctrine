// SPDX-License-Identifier: GPL-3.0-only
//! Structured diagnostics for the entity corpus catalog (SL-071 PHASE-03).
//! Plumbed now so a follow-up error-tolerant walk only needs to fill them;
//! this phase only generates diagnostics from edge target classification.

use std::path::PathBuf;

use super::hydrate::CatalogKey;

/// One finding from corpus scanning — never a panic or fail-fast.
/// Collectable so consumers filter by severity or suppress known noise.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct CatalogDiagnostic {
    /// The entity directory (or TOML) that sourced this finding.
    pub(crate) file: PathBuf,
    /// The entity that produced the finding, if one is implicated.
    pub(crate) entity_key: Option<CatalogKey>,
    /// The field or section that produced the finding (e.g. a `[[relation]]` label).
    pub(crate) field: Option<String>,
    /// Human-readable description of the finding.
    pub(crate) message: String,
    pub(crate) severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub(crate) enum Severity {
    /// A hard integrity violation (malformed TOML, duplicate id).
    /// Produced by memory scan for uid/dirname mismatches and malformed toml.
    Error,
    /// A dangling canonical ref — the ref parses but the target entity is absent.
    Warning,
    /// An unvalidated free-text target (not a canonical ref at all).
    Info,
}
