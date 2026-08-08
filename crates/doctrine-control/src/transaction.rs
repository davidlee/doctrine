// SPDX-License-Identifier: GPL-3.0-only
//! `transaction` — what one capsule transaction binds, as provisioning returns
//! it (SL-248 `sec-3`, `EX-1`…`EX-3`, `EX-7`).
//!
//! This unit is **vocabulary only**. It holds no filesystem, no subprocess and
//! no policy evaluation: `provision` computes every field and this type records
//! the result. `DEC-159` fixes the field set to what has a consumer *now* — the
//! nine fields below — and the two that go beyond that decision's own
//! enumeration ([`CapsuleTransaction::backend`] and [`AcceptedBase`] in place of
//! a bare oid) each have a consumer *in this slice*, named at the field.
//!
//! **Two obligations are documented here and not checked** — deliberately, in
//! the same words as each other, so a later slice inherits two stated
//! obligations rather than one stated and one silent. Both are at their fields.
//!
//! Layering (`ADR-001`): `transaction` is `engine`, out-edges `{backend,
//! config}` — [`AcceptedBase`], [`BackendId`], [`CapsulePlacement`] and
//! [`TransactionRoot`] from `backend`, [`ResourceBounds`] from `config`.

use std::path::{Component, Path};

use doctrine::interpretation::{InterpretationPolicy, PolicyHash};

use crate::backend::{AcceptedBase, BackendId, CapsulePlacement, TransactionRoot};
use crate::config::ResourceBounds;

/// Why a [`TransactionId`] was refused.
///
/// Structured and carrying the rejected text, the same posture as every other
/// refusal in this crate: an operator triaging one wants the value to fix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TransactionIdRefusal {
    /// The value is not a single path component (`EX-7`) — it is empty, it
    /// carries a separator, or it is `.` / `..` / a root.
    NotASinglePathComponent { value: String },
}

/// A transaction's identity, and the leaf its root directory is named by.
///
/// **Refuses at construction anything that is not a single path component**
/// (`EX-7`). The identity is joined onto the capsule root to name a directory,
/// so a value carrying a separator — or `..`, or the empty string — would let
/// the *identity* choose where the transaction's state lands. One rule covers
/// all of those: [`Path::components`] yields exactly one
/// [`Component::Normal`] whose text is the whole value.
///
/// Allocated by the caller and passed in, never minted here: the shell mints it
/// (`EX-7`), which is what keeps this type free of a clock and an entropy
/// source, and what makes a collision test writable — a test hands the same id
/// twice.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TransactionId(String);

impl TransactionId {
    /// The only constructor.
    ///
    /// # Errors
    ///
    /// [`TransactionIdRefusal::NotASinglePathComponent`] when `value` is not a
    /// single path component.
    pub(crate) fn try_new(value: String) -> Result<Self, TransactionIdRefusal> {
        let mut components = Path::new(&value).components();
        let single = matches!(
            (components.next(), components.next()),
            (Some(Component::Normal(only)), None) if only == value.as_str()
        );
        if single {
            Ok(Self(value))
        } else {
            Err(TransactionIdRefusal::NotASinglePathComponent { value })
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// A phase, by durable identity only.
///
/// Two fields and nothing more (`EX-3`). `RFC-027` is open and proposes
/// reshaping selectors, plans, phase gates and criteria; provisioning reads
/// neither `plan.toml` nor a phase sheet, so this slice's whole exposure to that
/// churn is these two fields.
///
/// `sec-3` names the field types `SliceId` and `PhaseNumber`. Neither type
/// exists in the workspace, and `sec-6` fixes the root library's export set — so
/// they are spelled here as the plain values they stand for. Minting a pair of
/// newtypes in this crate would be a second home for an identity the platform
/// owns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PhaseIdentity {
    /// The durable slice id, e.g. `SL-248`.
    pub(crate) slice: String,
    /// The immutable `PHASE-NN`, as its number.
    pub(crate) phase: u32,
}

/// One capsule transaction, as provisioning returns it.
///
/// Fields are `pub(crate)` rather than accessor-guarded because **nothing here
/// is validated by this type**: every field arrives already validated by the
/// unit that owns its rule — [`CapsulePlacement::try_new`] for the placement,
/// `sec-4`'s restriction algebra for the policy, `config`'s reader for the
/// bounds — and a second constructor here would be a second place for those
/// rules to drift. The one field with a rule of its own, [`TransactionId`],
/// carries it in its own constructor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CapsuleTransaction {
    /// Allocated trusted-side. Never chosen by a capsule.
    pub(crate) id: TransactionId,
    /// A durable reference to the phase this serves, and nothing more.
    pub(crate) phase: PhaseIdentity,
    /// The commit this capsule is provisioned from.
    ///
    /// **Documented obligation, not checked (1 of 2).** This records a
    /// trusted-side caller's assertion that the commit was resolved as
    /// accepted; `provision` does **not** verify it against the canonical
    /// accepted ref, because that check belongs to admission (`REQ-455`), an
    /// explicit Non-Goal of this slice. What *is* verified is the weaker and
    /// entirely local claim that the export bound at `/source` holds exactly
    /// this base (step 8).
    pub(crate) base: AcceptedBase,
    /// The admitted mechanism that created this capsule and must execute it.
    ///
    /// Recorded because provisioning itself calls `execute` (step 11), so the
    /// choice is made inside this slice: an observation that cannot be
    /// attributed to a backend cannot be attributed to an admission verdict
    /// either.
    ///
    /// **Documented obligation, not checked (2 of 2).** This records a
    /// trusted-side caller's assertion that the mechanism passed `sec-7`'s
    /// suite; `provision` does **not** verify it against the admission journal,
    /// because that check belongs to admission (`REQ-455`), an explicit
    /// Non-Goal of this slice. `backend verify` is what a caller runs to
    /// discharge it today, and nothing yet records that it did.
    pub(crate) backend: BackendId,
    /// The canonical hash of the base policy, as resolved from `base`.
    pub(crate) base_policy: PolicyHash,
    /// The policy actually in force: the base policy, or the result of applying
    /// a phase refinement to it (`sec-4`).
    pub(crate) policy: InterpretationPolicy,
    /// The canonical hash of `policy`. Equal to `base_policy` when no refinement
    /// was applied — recorded rather than derived, so the admission journal a
    /// later slice writes has both without re-hashing.
    pub(crate) policy_hash: PolicyHash,
    /// Where the capsule lives, and what is readable inside it (`sec-2`).
    pub(crate) placement: CapsulePlacement,
    /// The resource choices read from `[capsule]` (`sec-5`).
    pub(crate) bounds: ResourceBounds,
}

impl CapsuleTransaction {
    /// This transaction's own writable state, minted by `sec-3` step 9.
    ///
    /// Reached through the placement rather than stored twice: the placement is
    /// the type that was validated against the forbidden scopes, so a second
    /// copy of the root here could disagree with the one that passed.
    pub(crate) const fn root(&self) -> &TransactionRoot {
        self.placement.root()
    }
}

#[cfg(test)]
mod tests {
    use super::{PhaseIdentity, TransactionId, TransactionIdRefusal};

    #[test]
    fn a_transaction_id_carrying_a_path_separator_refuses_at_construction() {
        let refused = TransactionId::try_new("../other".to_owned());

        assert_eq!(
            refused,
            Err(TransactionIdRefusal::NotASinglePathComponent {
                value: "../other".to_owned()
            })
        );

        // The whole family the one rule covers, so the rule is not read as
        // "rejects `..`" — every one of these would otherwise let the identity
        // choose where the transaction's state lands.
        for value in ["", ".", "..", "/", "a/b", "/absolute", "trailing/"] {
            assert_eq!(
                TransactionId::try_new(value.to_owned()),
                Err(TransactionIdRefusal::NotASinglePathComponent {
                    value: value.to_owned()
                }),
                "{value:?} is not a single path component"
            );
        }
    }

    #[test]
    fn an_ordinary_minted_identity_is_accepted_and_names_its_own_leaf() {
        let accepted =
            TransactionId::try_new("tx-19fe1653d81-7712".to_owned()).expect("single component");

        assert_eq!(accepted.as_str(), "tx-19fe1653d81-7712");
    }

    #[test]
    fn a_phase_identity_carries_the_durable_slice_id_and_the_immutable_number() {
        let phase = PhaseIdentity {
            slice: "SL-248".to_owned(),
            phase: 6,
        };

        assert_eq!(phase.slice, "SL-248");
        assert_eq!(phase.phase, 6);
    }
}
