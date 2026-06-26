// SPDX-License-Identifier: GPL-3.0-only
//! `boundary` — the per-phase code-boundary row, a `leaf` shared by both the
//! committed dispatch run-ledger (`crate::ledger`) and the gitignored recorded
//! source-delta registry (`crate::state`).
//!
//! Extracted to its own leaf (SL-147 PHASE-02) so the engine-tier registry can
//! consume the row type without depending on the whole `ledger` module for a
//! single struct (cohesion). Pure: std + serde only, no clock/disk/git.

use serde::{Deserialize, Serialize};

/// The landing path that recorded a boundary row — the registry's
/// self-describing provenance discriminator (design §5.3, D12). It governs the
/// sticky merge in `state::record_source_delta`: the landing writers `Solo` and
/// `Funnel` are authoritative; `Manual` (the `record-delta` escape hatch) never
/// reclassifies an existing path; `Unknown` is the legacy default, only ever
/// *read* from a pre-provenance row — **live code never writes it**.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Provenance {
    /// Recorded by the solo phase-binding capture (`state::set_phase_status`).
    Solo,
    /// Recorded by the dispatch funnel (`dispatch::run_record_boundary`).
    Funnel,
    /// Recorded by the manual `record-delta` escape hatch (`slice::run_record_delta`).
    Manual,
    /// Read back from a legacy row written before provenance existed. The
    /// `#[serde(default)]` on the field below maps a missing key here; live code
    /// never constructs it as a written value.
    #[default]
    Unknown,
}

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
    /// The landing path that recorded this row (design §5.3). The whole
    /// back-compat story is `#[serde(default)]`: a legacy row with no
    /// `provenance` key reads as [`Provenance::Unknown`].
    #[serde(default)]
    pub provenance: Provenance,
}

#[cfg(test)]
mod tests {
    use super::*;

    // VT-1: a legacy row with no `provenance` key deserializes to Unknown (the
    // back-compat default), and a row carrying an explicit provenance round-trips.
    #[test]
    fn provenance_serde_defaults_to_unknown_and_round_trips() {
        let legacy = "phase = \"PHASE-01\"\ncode_start_oid = \"s\"\ncode_end_oid = \"e\"\n";
        let row: BoundaryRow = toml::from_str(legacy).expect("legacy row (no provenance) parses");
        assert_eq!(row.provenance, Provenance::Unknown, "missing key ⇒ Unknown");

        let funnel = BoundaryRow {
            phase: "PHASE-02".into(),
            code_start_oid: "a".into(),
            code_end_oid: "b".into(),
            provenance: Provenance::Funnel,
        };
        let text = toml::to_string(&funnel).expect("serialize");
        assert!(
            text.contains("provenance = \"funnel\""),
            "snake_case token: {text}"
        );
        assert_eq!(
            toml::from_str::<BoundaryRow>(&text).expect("round-trip"),
            funnel
        );
    }
}
