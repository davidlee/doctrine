// SPDX-License-Identifier: GPL-3.0-only
//! The act payloads a design-run e2e ladder submits (SL-244 `T8`).
//!
//! **Why not in `tests/design_fixture/`.** That module is the *tree* bootstrap,
//! and crates that only drive the CLI opt into it (`e2e_claude_install`). These
//! builders are over the pure model's wire types, so putting them there would
//! drag the `design_run` leaf into crates with no use for it — the same argument
//! its own doc makes for staying out of `tests/common/`
//! (`mem.pattern.tests.shared-helper-placement`).
//!
//! **Precondition.** An including crate must also declare the `#[path]`-included
//! `mod design_run;` — these resolve `crate::design_run::…`. A crate that
//! forgets fails to compile, so the coupling cannot rot silently.
//!
//! **Built from the wire types, never a JSON literal.** Neither [`ActKind`] nor
//! [`AgentAct`] carries an `as_str`, and re-typing a serde-derived kebab token in
//! a test is the drift STD-001 forbids. `serde_json::to_value` over the real type
//! takes the token from the same source the binary compiles, and exercises
//! `Serialize` for free.

#![allow(
    dead_code,
    reason = "shared builders: not every includer submits every act"
)]

use serde_json::Value;

use crate::design_run::attestation::{ActKind, AgentAct, ReviewDisposition};
use crate::design_run::submission::{
    AcceptanceDeclaration, AgentActDeclaration, CheckpointActDeclaration,
};

/// A checkpoint act as an `ApplyRequest::checkpoint_act` field.
pub(crate) fn checkpoint_act(act: ActKind, basis: &str) -> Value {
    declared(act, basis, None)
}

/// The one checkpoint act that disposes of the run's current pass.
///
/// The act kind is fixed rather than a parameter: the correspondence admits a
/// disposition on [`ActKind::ReviewDisposed`] and nowhere else, so binding the
/// two here means a ladder cannot build the pair that admission exists to refuse.
/// A record deliberately mismatched is [`super::design_run::admission`]'s to
/// test, by payload, in the unit suite.
pub(crate) fn review_disposed(basis: &str, disposition: ReviewDisposition) -> Value {
    declared(ActKind::ReviewDisposed, basis, Some(disposition))
}

/// An agent declaration as an `ApplyRequest::agent_declaration` field.
pub(crate) fn agent_declaration(act: AgentAct, basis: &str) -> Value {
    serde_json::to_value(AgentActDeclaration {
        act,
        basis: basis.to_owned(),
        turn: None,
    })
    .unwrap()
}

fn declared(act: ActKind, basis: &str, disposition: Option<ReviewDisposition>) -> Value {
    serde_json::to_value(CheckpointActDeclaration {
        act,
        acceptance: AcceptanceDeclaration {
            basis: basis.to_owned(),
            turn: None,
        },
        disposition,
    })
    .unwrap()
}
