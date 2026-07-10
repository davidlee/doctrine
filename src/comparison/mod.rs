// SPDX-License-Identifier: GPL-3.0-only
//! `comparison` — the pairwise comparison layer (SL-213).
//!
//! Directory module mirroring the `priority/` style; tiers map one-to-one
//! onto submodules (SL-213 design §1). `wire` is the schema wire model; the
//! resolution / compilation / projection tiers land in later phases.
//!
//! The wire vocabulary is re-exported so `crate::comparison::X` paths compile
//! unchanged (behaviour-preservation gate).

mod wire;

pub(crate) use wire::*;
