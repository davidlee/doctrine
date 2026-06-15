// SPDX-License-Identifier: GPL-3.0-only
//! Entity corpus catalog — the single source of truth for scanning and
//! hydrating the authored entity corpus (SL-071). Engine-tier (ADR-001):
//! depends on leaf modules + kind modules, never on command modules.
//!
//! - `scan` — the KINDS-driven corpus walk (re-homed from `relation_graph`)
//! - `hydrate` — richer catalog types (PHASE-03)
//! - `graph` — presentation-neutral graph projection (PHASE-04)
//! - `diagnostic` — structured diagnostics (PHASE-03)

pub(crate) mod scan;
