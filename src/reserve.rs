// SPDX-License-Identifier: GPL-3.0-only
//! `reserve` — claim-backend selection for fresh-id allocation (SL-148).
//!
//! [`backend`] is the single seam that resolves which [`Claim`](crate::entity::Claim)
//! backend a Fresh-allocating materialise site uses. v1 always returns
//! [`LocalFs`](crate::entity::LocalFs); the `GitRef` remote-reservation backend and the
//! `[reservation]` reach config arrive in PHASE-03. Routing the ~11 Fresh call sites
//! through one helper — rather than a literal `&LocalFs` at each — is what lets the
//! second backend drop in behind a single signature (design §5.2, F-3).

use std::path::Path;

use crate::entity::{Claim, LocalFs};

/// Resolve the claim backend for a fresh-id allocation under `root`, for the kind
/// whose canonical id-space is `prefix` (`SL` / `ASM` / … — the reservation ref
/// segment, F-V7, never the shared file-stem). v1 hardcodes [`LocalFs`]; reach
/// selection and the `GitRef` backend land in PHASE-03.
#[expect(
    unused_variables,
    reason = "root/prefix select the GitRef backend in PHASE-03 (SL-148)"
)]
#[expect(
    clippy::unnecessary_wraps,
    reason = "PHASE-03 returns Err on a reach/config/transport failure (SL-148)"
)]
pub(crate) fn backend(root: &Path, prefix: &str) -> anyhow::Result<Box<dyn Claim>> {
    Ok(Box::new(LocalFs))
}
