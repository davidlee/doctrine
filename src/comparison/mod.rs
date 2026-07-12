// SPDX-License-Identifier: GPL-3.0-only
//! `comparison` — the pairwise comparison layer (SL-213).
//!
//! Directory module mirroring the `priority/` style; tiers map one-to-one
//! onto submodules (SL-213 design §1). `wire` is the schema wire model;
//! `resolve` is tier 1 (row validity), `compile` is tier 2 (constraint
//! compilation), `project` is tier 3 (placement & gauge), `store` is the one
//! impure seam (disk load + full-pipeline composition).
//!
//! The wire vocabulary is re-exported so `crate::comparison::X` paths compile
//! unchanged (behaviour-preservation gate). PHASE-05 wires `store` to the
//! priority build shell — every re-export now has a real, non-test consumer,
//! so the PHASE-01..04 self-clearing `dead_code` suppressions have retired
//! themselves (`mem.pattern.lint.dead-code-expect-vs-cfg-test`).

mod compile;
mod project;
mod query;
mod resolve;
mod store;
mod wire;

pub(crate) use compile::*;
pub(crate) use project::*;
// SL-217 PHASE-02 retires the staged-ahead `dead_code` gate: `priority::elicit`
// is `query`'s first non-test consumer. `hypothetical_yield` stays the lone
// exception — a thin signed-count wrapper over `hypothetical_outcome`, which is
// what elicit actually needs (D13 impact wants the pair sets) — so it carries
// its own per-item gate at its definition.
pub(crate) use query::*;
pub(crate) use resolve::*;
pub(crate) use store::*;
pub(crate) use wire::*;
