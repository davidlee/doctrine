// SPDX-License-Identifier: GPL-3.0-only
//! Admission and storage bounds (SL-233 PHASE-03, projection-bounds sketch rev 8 §(a)).
//!
//! Every constant here binds a value at **admission** — the moment it is created
//! or accepted — or bounds what *storage* retains. Exceeding an admission bound
//! is a **refusal**, never a trim: each one bounds identity or a closed
//! vocabulary, and a truncated identity is a *wrong* identity rather than a
//! shorter one.
//!
//! The emission bounds (`ENVELOPE_*`) deliberately do **not** live here. They are
//! private to [`super::render`] so that a storage or admission path cannot
//! reference what it cannot name (EX-16(c)); the reverse direction is fine and
//! intended — `render` names these for its row arithmetic.
//!
//! **The provenance rule (EX-16(a)).** Every bound below states what derives it.
//! A bound that cannot be derived is removed, not guessed — which is why there is
//! deliberately **no bound of any kind on a stored regression reason**: rev 6's
//! `CHANGE_REASON_INPUT_BYTES` = 2048 was never derived, and the snapshot already
//! stores unbounded prose in section bodies, so bounding that one field would be
//! special-casing without a rationale. The projection is safe regardless — the
//! rendered reason elides however long the stored one is.
//!
//! This module imports nothing on purpose: it is `#[path]`-included by
//! `tests/e2e_design_state.rs` (the CHR-014 idiom for a binary-only crate), so
//! the bounds a test asserts against are the same bytes the binary compiles.

/// Bytes of any **run-local id** at creation: inquiry/node, section, checkpoint,
/// attestation — and the gate ids and canonical record refs that ride the same
/// slot in a change payload.
///
/// Derivation: the longest identifier this vocabulary must express is a gate id,
/// which is a [`super::gate::Condition`] token — `blocking-inquiries-dispositioned`
/// at exactly 32 B. Canonical entity refs (`SL-233`, `DEC-083`) are ≤ 9 B and fit
/// trivially. 32 is therefore the smallest bound that admits the widest member of
/// the closed vocabulary the id slot must carry, and the rendered-row arithmetic
/// in the sketch is computed from it (subject id term = 32 B).
pub(crate) const DESIGN_ID_BYTES: usize = 32;

/// Bytes of a **stage label** at admission.
///
/// Derivation: the stage vocabulary is closed ([`super::Stage::ALL`]) and its
/// longest member is `exploring` at 9 B. 16 is the next power of two above it,
/// leaving room for one further stage name without moving the row arithmetic.
pub(crate) const DESIGN_STAGE_LABEL_BYTES: usize = 16;

/// Bytes of a **change-event name** at admission.
///
/// Derivation: the event vocabulary is closed
/// ([`super::change_log::ChangeEvent::ALL`]) and its longest member is
/// `section_fingerprint_changed` at 27 B. 32 is the next power of two above it.
pub(crate) const DESIGN_EVENT_NAME_BYTES: usize = 32;

/// Past **revisions** the snapshot's change log retains.
///
/// Derivation: a storage bound, deliberately a different constant from any
/// projection cardinality. It is the window within which Doctrine can still
/// answer *what changed* — and therefore, by the same argument, the window
/// within which a submission receipt can still be resumed honestly. A
/// `known_revision` or a submission built below the retained window is refused
/// as expired rather than silently treated as new, because outside the window
/// "nothing changed" and "I cannot tell you what changed" are indistinguishable.
/// 32 revisions is a full working session's worth of applies at a cost of a few
/// KiB in gitignored runtime state.
pub(crate) const CHANGE_LOG_REVISIONS: u64 = 32;
