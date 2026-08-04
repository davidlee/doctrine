// SPDX-License-Identifier: GPL-3.0-only
//! `design_run` — the pure run model for CLI-managed design runs (SL-233).
//!
//! ADR-001 tier: **leaf** (`.doctrine/adr/001/layering.toml`). Crate out-degree
//! is **zero**: std plus the serde *derive* only, following the `funnel_machine`
//! and `boundary` precedent (a data-shape derive is neither IO nor domain
//! knowledge). No clock, rng, git, or filesystem anywhere beneath this module —
//! the shell gathers facts and passes them in as [`facts::DerivedDesignFacts`]
//! (AGENTS.md pure/imperative split).
//!
//! # Two rules this module tree is laid out to satisfy
//!
//! Both come from `.doctrine/slice/233/sketches/projection-bounds.md` rev 8,
//! and PHASE-03/PHASE-04 implement against them. They are recorded here because
//! the *module tree* is what makes the first one enforceable, and the tree is
//! created by PHASE-02.
//!
//! **The layer rule.** A constant's prefix declares the layer it binds, and a
//! layer may bind only its own artefacts. `ENVELOPE_*` bounds the budgeted
//! rendering and nothing else. `DESIGN_*` bounds a value at *admission* —
//! exceeding it is a refusal, never a trim. Identity and closed vocabularies
//! are bounded at admission, never at emission; only gracefully degrading prose
//! may be elided.
//!
//! **The provenance rule.** Every bound states what derives it. A bound that
//! cannot be derived is removed, not guessed.
//!
//! # The privacy precondition — read before adding a submodule
//!
//! The layer boundary is enforced by Rust privacy: the `ENVELOPE_*` constants
//! PHASE-04 introduces are **private to the rendering submodule**, so a storage
//! or admission path cannot reference what it cannot name. Rust privacy is
//! module-scoped and **descendants can see an ancestor's private items**, so
//! that guarantee holds only while the tree keeps rendering a *sibling* of
//! storage and admission:
//!
//! - no constants live at this module root;
//! - the future `render` submodule is never an ancestor of a storage or
//!   admission submodule. [`submission`] and [`inquiry`] (admission) and the
//!   PHASE-03 snapshot wire (storage) are siblings of it by construction.
//!
//! Placing `ENVELOPE_*` at this root, or nesting storage beneath `render`, makes
//! the compile error silently stop firing while every test still passes.

// This module is staged ahead of its consumer: PHASE-03 (persistence) and
// PHASE-04 (command surface) land the first non-test callers. Self-clearing —
// when nothing here is dead the expectation goes unfulfilled and forces its own
// removal.
//
// Scoped to `not(test)` on purpose, and the scoping is the whole point
// (`mem.pattern.lint.dead-code-expect-vs-cfg-test`). `expect` is fulfilled per
// *compilation*. Under `cargo clippy`/`cargo build` every item here is dead, so
// one module-level gate is honest and there is nothing it could mask that is not
// already expected. Under `cargo test` this gate is stripped, which is what keeps
// "you added an item no test exercises" a hard error rather than something a
// blanket swallows (`mem.pattern.lint.dead-code-blanket-masks-siblings`).
//
// The handful of items the §9.1 suite genuinely does not reach carry their own
// UNCONDITIONAL per-item `expect` at their definition, and `traversal` carries a
// file-level one because EX-8 fixes the suite at exactly eight tests and that
// module has no test consumer at all. Those are the narrowest gates that compile,
// not a convenience.
//
// `attestation` used to carry a file-level gate too. PHASE-10 narrowed it to
// per-item, which is what the module-wide form was always standing in for: most
// of that module went live at PHASE-12, and a blanket over a mostly-live module
// masks its siblings rather than disclosing what is unreached.
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SL-233 PHASE-03/04 land the first non-test callers"
    )
)]

pub(crate) mod admission;
pub(crate) mod attestation;
pub(crate) mod bounds;
pub(crate) mod change_log;
pub(crate) mod delegation;
pub(crate) mod document;
pub(crate) mod facts;
pub(crate) mod gate;
pub(crate) mod ids;
pub(crate) mod inquiry;
pub(crate) mod legacy;
pub(crate) mod prompt;
pub(crate) mod refusal;
// `render` is a SIBLING of the storage and admission modules above and below,
// never an ancestor of one — see the privacy precondition in this module's doc.
pub(crate) mod render;
pub(crate) mod run;
pub(crate) mod runbook;
pub(crate) mod section;
pub(crate) mod snapshot;
pub(crate) mod submission;
pub(crate) mod traversal;

#[cfg(test)]
mod fixture;
#[cfg(test)]
mod tests;

/// The canonical read model, re-exported **by name** (EX-2).
///
/// By name and not by glob, on purpose: a glob would re-export whatever the
/// rendering subtree happens to make `pub(crate)` next, which is precisely how a
/// bound leaks out of the layer that owns it. The type crosses; the caps do not.
///
/// It is *declared* inside [`render::envelope`] rather than here because the
/// projection that builds it must name the private `ENVELOPE_*` caps, and only a
/// descendant of [`render`] can. Assembling it at this root would mean widening
/// them — see the privacy precondition above.
pub(crate) use render::envelope::TurnEnvelope;

use serde::{Deserialize, Serialize};

/// The five coarse stages of a design run (design §5.4).
///
/// Landmarks, not an exhaustive FSM — inquiry lifecycle, cursor/posture, review,
/// delegation, and recovery are separate state models (DEC-065, EX-4). The
/// declaration order is the forward order, and [`gate::Advance`] is the sole
/// authority on which moves between them are legal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Stage {
    Exploring,
    Inquiring,
    Drafting,
    Reviewing,
    Locked,
}

impl Stage {
    /// Every stage in forward order — the closed vocabulary, single-sourced so
    /// an exhaustive table test cannot silently miss a new variant (STD-001).
    pub(crate) const ALL: [Stage; 5] = [
        Stage::Exploring,
        Stage::Inquiring,
        Stage::Drafting,
        Stage::Reviewing,
        Stage::Locked,
    ];

    /// The kebab token this stage is spelled with everywhere — the snapshot
    /// value, the refusal text, the rendered label (STD-001).
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Stage::Exploring => "exploring",
            Stage::Inquiring => "inquiring",
            Stage::Drafting => "drafting",
            Stage::Reviewing => "reviewing",
            Stage::Locked => "locked",
        }
    }
}

/// The widest stage label, at compile time.
const fn widest_stage(rest: &[Stage]) -> usize {
    match rest {
        [] => 0,
        [head, tail @ ..] => {
            let head = head.as_str().len();
            let tail = widest_stage(tail);
            if head > tail { head } else { tail }
        }
    }
}

/// The provenance of [`bounds::DESIGN_STAGE_LABEL_BYTES`], **proved rather than
/// asserted** (EX-16(a)): the stage vocabulary is closed, its longest member is
/// `exploring` at 9 B, and the bound leaves room for one more without moving the
/// rendered-row arithmetic.
const _: () = assert!(widest_stage(&Stage::ALL) <= bounds::DESIGN_STAGE_LABEL_BYTES);
