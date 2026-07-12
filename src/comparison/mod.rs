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
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SL-217 PHASE-01 stages the query predicates one phase ahead of their consumer (priority::elicit, PHASE-02); until then every item is test-only (mem.pattern.lint.dead-code-staged-ahead-cfg-test)"
    )
)]
mod query;
mod resolve;
mod store;
mod wire;

pub(crate) use compile::*;
pub(crate) use project::*;
// `query` is not re-exported yet: its first non-test consumer is SL-217
// PHASE-02 (`priority::elicit`), which adds `pub(crate) use query::*;` — the
// same self-clearing staging as the PHASE-01..04 note above.
pub(crate) use resolve::*;
pub(crate) use store::*;
pub(crate) use wire::*;
