// SPDX-License-Identifier: GPL-3.0-only
//! The priority subsystem (SL-047) — the cross-kind work-priority adapter.
//!
//! Sits at the engine layer above `relation_graph` (ADR-001): it consumes
//! `relation_graph`'s `pub(crate)` all-kind scan seam ([`crate::relation_graph::
//! scan_entities`]) to build a THIRD cordage `Graph` (distinct from
//! `backlog_order`'s and `inspect`'s — they share only the `Projection` TYPE),
//! carrying the dep/seq overlays, per-node attributes, a consequence pre-pass tally,
//! and an `OrderSpec`. PHASE-02 adds the pure policy core: [`partition`] (the OQ-8
//! status-class table) and [`channels`] (eligibility / blockers / actionable /
//! consequence / order-key / dep-cycle synthesis derived over a `PriorityGraph`).
//! The CLI surface that consumes these lands PHASE-03.
pub(crate) mod channels;
pub(crate) mod graph;
pub(crate) mod partition;
