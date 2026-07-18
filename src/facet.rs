// SPDX-License-Identifier: GPL-3.0-only
//! `facet` — shared `EntityFacets` aggregation (SL-132, SL-133).
//!
//! Consolidates `risk` facet and `tags` into a pure data struct consumed by entity
//! display renderers. The `estimate`/`value` facet fields have been removed (they
//! were deleted at SL-222 PHASE-09; presence-key tripwires in `scan.rs` now detect
//! residue).
//!
//! **Leaf tier (ADR-001).** Pure data — imports only `risk` (leaf).

use crate::risk::RiskFacet;

/// Shared projection carrying risk facet and tags for a single entity.
/// Constructed by the shell (`run_show`, `scan_catalog`) from already-parsed
/// data; consumed by `format_show` and (in SL-133) `format_survey_row`.
///
#[derive(Debug, Clone, Default)]
pub(crate) struct EntityFacets {
    pub risk: Option<RiskFacet>,
    pub tags: Vec<String>,
}
