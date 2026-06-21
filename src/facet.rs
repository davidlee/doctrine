// SPDX-License-Identifier: GPL-3.0-only
//! `facet` — shared `EntityFacets` aggregation (SL-132, SL-133).
//!
//! Wraps already-parsed facet fields (`estimate`, `value`) into a pure data struct
//! consumed by entity display renderers. No parsing, no disk I/O — both input
//! paths (`SliceDoc` serde + `scan::read_facets`) predate this module.
//!
//! **Leaf tier (ADR-001).** Pure data — imports only `estimate` + `value` (both leaf).

use crate::estimate::EstimateFacet;
use crate::value::ValueFacet;

/// Shared projection carrying optional estimate and value facets for a single entity.
/// Constructed by the shell (`run_show`, `scan_catalog`) from already-parsed data;
/// consumed by `format_show` and (in SL-133) `format_survey_row`.
///
/// Extended in later slices: risk (SL-133), tags (SL-136).
#[derive(Debug, Clone, Default)]
pub(crate) struct EntityFacets {
    pub estimate: Option<EstimateFacet>,
    pub value: Option<ValueFacet>,
}
