// SPDX-License-Identifier: GPL-3.0-only
//! `boundary` — the per-phase code-boundary row, a `leaf` shared by both the
//! committed dispatch run-ledger (`crate::ledger`) and the gitignored recorded
//! source-delta registry (`crate::state`).
//!
//! Extracted to its own leaf (SL-147 PHASE-02) so the engine-tier registry can
//! consume the row type without depending on the whole `ledger` module for a
//! single struct (cohesion). Pure: std + serde only, no clock/disk/git.

use serde::{Deserialize, Serialize};

/// One phase's code boundary (design §4.3): `code_end_oid` is the worker code
/// commit *before* the knowledge record commit; an empty-code phase has
/// `code_start_oid == code_end_oid`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct BoundaryRow {
    /// The `PHASE-NN` id this boundary belongs to.
    pub phase: String,
    /// HEAD before the phase's code landed.
    pub code_start_oid: String,
    /// The phase's cumulative code tip (pre-knowledge-record).
    pub code_end_oid: String,
}
