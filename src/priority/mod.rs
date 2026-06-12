// SPDX-License-Identifier: GPL-3.0-only
//! The priority subsystem (SL-047) — the cross-kind work-priority adapter.
//!
//! Sits at the engine layer above `relation_graph` (ADR-001): it consumes
//! `relation_graph`'s `pub(crate)` all-kind scan seam ([`crate::relation_graph::
//! scan_entities`]) to build a THIRD cordage `Graph` (distinct from
//! `backlog_order`'s and `inspect`'s — they share only the `Projection` TYPE),
//! carrying the dep/seq overlays, per-node attributes, a consequence pre-pass tally,
//! and an `OrderSpec`. No partition/channel POLICY yet — PHASE-01 stores the RAW
//! authored status; classification (workable/terminal) is PHASE-02.
pub(crate) mod graph;
