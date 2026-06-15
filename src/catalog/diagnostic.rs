// SPDX-License-Identifier: GPL-3.0-only
//! Structured diagnostics for the entity corpus catalog (SL-071 PHASE-03).
//! Plumbed now so a follow-up error-tolerant walk only needs to fill them;
//! this phase only generates diagnostics from edge target classification.

use std::path::PathBuf;

use super::scan::EntityKey;

/// One finding from corpus scanning — never a panic or fail-fast.
/// Collectable so consumers filter by severity or suppress known noise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CatalogDiagnostic {
    /// The entity directory (or TOML) that sourced this finding.
    pub(crate) file: PathBuf,
    /// The entity that produced the finding, if one is implicated.
    pub(crate) entity_key: Option<EntityKey>,
    /// The field or section that produced the finding (e.g. a `[[relation]]` label).
    pub(crate) field: Option<String>,
    /// Human-readable description of the finding.
    pub(crate) message: String,
    pub(crate) severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Severity {
    /// A hard integrity violation (malformed TOML, duplicate id). Not produced
    /// this phase — the fail-fast `scan_entities` bails before we reach here.
    /// Plumbed for the follow-up error-tolerant walk.
    #[expect(dead_code, reason = "plumbed for follow-up error-tolerant walk")]
    Error,
    /// A dangling canonical ref — the ref parses but the target entity is absent.
    Warning,
    /// An unvalidated free-text target (not a canonical ref at all).
    Info,
}
