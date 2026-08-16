// SPDX-License-Identifier: GPL-3.0-only
//! The wire-payload contract model, and the barrier that keeps it honest
//! (SL-251 `sec-2`, `sec-3`, `sec-4`).
//!
//! A **describing** module, and a sibling of [`super::artifact`] for that
//! reason: `artifact` renders the published stage machine, this one describes
//! the submission payload. Neither *does* anything to a run. It is deliberately
//! not under [`super::render`], which is keyed to a turn snapshot — the contract
//! is a property of the types, not of any one run's state (`sec-7`).
//!
//! ADR-001 tier: **leaf, out-degree 0**. Every `use` below is `super::…`; this
//! file names `crate::` nowhere. The one vocabulary the leaf cannot see —
//! `knowledge::RecordKind` and its facet fields — is declared as an
//! [`ExternRegion`] and supplied from above (`sec-3`), never imported.
//!
//! # What lands here, and what does not
//!
//! The model ([`TypeContract`] and friends), the extern-region seam, the
//! [`payload_variants!`] exhaustiveness barrier over the closure's fourteen
//! enums, and the two addresses the generated document is published at. The
//! `PAYLOAD` table itself, the renderer, and the CLI surface are later phases;
//! nothing here reads a table that does not yet exist.

use std::collections::BTreeSet;

#[cfg(test)]
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};

use super::Stage;
#[cfg(test)]
use super::attestation::ReviewRef;
use super::attestation::{ActKind, AgentAct, ReviewDisposition, ReviewPolicy, Reviewer};
#[cfg(test)]
use super::fixture::id;
#[cfg(test)]
use super::ids::Fingerprint;
use super::ids::IdKind;
use super::inquiry::{InquiryLifecycle, Provenance};
#[cfg(test)]
use super::submission::CreateRecord;
use super::submission::{DelegationAct, DischargeClaim, Dispose, WireFacetValue};
use super::traversal::{Authority, Posture};

// ---------------------------------------------------------------------------
// The exhaustiveness barrier (sec-4)
// ---------------------------------------------------------------------------

/// The token vocabulary of one closure enum, **pinned to the enum itself** by a
/// dead exhaustive match (design `sec-4`, `DEC-229`).
///
/// `SL-244`'s [`condition_vocabulary!`](super::gate) owns the type it describes:
/// it emits the enum, its `ALL`, its `as_str` and its contracts from one list,
/// so an omission is unspellable. That is not available here. `Stage`,
/// `Dispose`, `AgentAct` and the rest **already exist**, carry data on their
/// variants, and sit under the behaviour-preservation gate as shared machinery;
/// regenerating their definitions would be the largest and riskiest change in a
/// slice scoped as a documentation surface.
///
/// The insight that removes the need: `condition_vocabulary!`'s real guarantee
/// is not that it writes the type — it is that an omission makes a *generated
/// match* non-exhaustive, which is a build failure. That guarantee can be had
/// without owning the definition, and this macro takes exactly it. Not one
/// existing type definition moves.
///
/// It pins both directions at compile time:
///
/// - a variant added to the enum and left out of an invocation makes `_pin`'s
///   match non-exhaustive — build failure;
/// - a variant named in an invocation that does not exist on the enum is an
///   unresolved path — build failure.
///
/// It also supplies `TYPE_NAME` by `stringify!` on the identifier the invocation
/// names, so a rename is a build failure at the match rather than a stale
/// literal every other pin agrees with (`sec-2`, *Naming*).
///
/// What it does **not** pin is that the tokens equal serde's actual renames —
/// that is `sec-8` pin 4's per-variant round trip, `tokens_match_serde_renames`
/// below. The barrier is what makes that round trip *sufficient*: the match is
/// total over variants, so the test cannot silently lose a case.
///
/// **What generation costs** (`R3`, and `condition_vocabulary!` measured it):
/// clippy does not lint tokens a macro body wrote. So the generated region is
/// held to a token array and a match with no bodies — there is nothing in it
/// that could hold a bug. The tokens themselves are written by the caller, where
/// lints do fire.
///
/// # The three arms
///
/// ```text
/// Stage via Stage::as_str { Exploring, … }          // the type names its tokens
/// AgentAct { DraftingReady = "drafting-ready", … }  // named here for the first time
/// WireFacetValue untagged { List, Text }            // no tokens on the wire at all
/// ```
///
/// **`via`** consumes an existing authority rather than retyping it (`STD-001`:
/// when a constant for a value already exists, use it). `$auth` is called as
/// `$auth($ty::$variant)` in a `const` initialiser, so it takes a `const fn`
/// with a **`self`** receiver over a **fieldless** variant. Two of `sec-4`'s six
/// authority-bearing enums cannot meet that — see [`payload_variants`]'s
/// invocation block and `PHASE-01/EX-9` — and take the naming arm with a
/// per-variant pin against their authority in `tokens_match_serde_renames`.
///
/// **`untagged`** exists because `WireFacetValue` is `#[serde(untagged)]` and
/// contributes no token at all: its `VARIANTS` is empty by construction and the
/// invocation buys the match barrier alone. Spelling it with an empty token list
/// on the naming arm would have claimed a token that is not on the wire.
macro_rules! payload_variants {
    // The type already owns its token vocabulary; take it (STD-001).
    ($ty:ident via $auth:path { $($variant:ident),+ $(,)? }) => {
        impl $ty {
            /// Every token this enum is spelled with on the wire, from the
            /// type's own authority — generated, so it cannot lose a variant.
            pub(crate) const VARIANTS: &[&str] = &[$( $auth($ty::$variant) ),+];

            /// The Rust type name a refusal cites, from the identifier this
            /// invocation names (`sec-2`, *Naming*).
            pub(crate) const TYPE_NAME: &str = stringify!($ty);

            /// The barrier. Never called; its exhaustiveness is the whole point.
            const fn _pin(&self) {
                match self { $( $ty::$variant { .. } => {} ),+ }
            }
        }
    };

    // No authority exists, so this invocation is where the vocabulary is named
    // for the first time — which STD-001 permits and in fact wants.
    ($ty:ident { $($variant:ident = $token:literal),+ $(,)? }) => {
        impl $ty {
            /// Every token this enum is spelled with on the wire.
            pub(crate) const VARIANTS: &[&str] = &[$( $token ),+];

            /// The Rust type name a refusal cites, from the identifier this
            /// invocation names (`sec-2`, *Naming*).
            pub(crate) const TYPE_NAME: &str = stringify!($ty);

            /// The barrier. Never called; its exhaustiveness is the whole point.
            const fn _pin(&self) {
                match self { $( $ty::$variant { .. } => {} ),+ }
            }
        }
    };

    // Untagged: no token reaches the wire, so there is no vocabulary to name.
    ($ty:ident untagged { $($variant:ident),+ $(,)? }) => {
        impl $ty {
            /// Empty, and truthfully so: an untagged enum puts no token on the
            /// wire, and printing the Rust variant name would say something
            /// false. Its shapes are `sec-8` pin 2's, not pin 4's.
            pub(crate) const VARIANTS: &[&str] = &[];

            /// The Rust type name a refusal cites, from the identifier this
            /// invocation names (`sec-2`, *Naming*).
            pub(crate) const TYPE_NAME: &str = stringify!($ty);

            /// The barrier. Never called; its exhaustiveness is the whole point.
            const fn _pin(&self) {
                match self { $( $ty::$variant { .. } => {} ),+ }
            }
        }
    };
}

// --- Group 1: the type names its own tokens; take them (sec-4, STD-001). -----

payload_variants! { Stage via Stage::as_str { Exploring, Inquiring, Drafting, Reviewing, Locked } }

payload_variants! {
    ActKind via ActKind::as_str {
        GovernanceConfirmed,
        GraphReviewed,
        BlockingSetDeclared,
        SufficiencyAccepted,
        DraftingReady,
        SectionReviewed,
        ReviewDisposed,
        DesignAccepted,
    }
}

payload_variants! {
    ReviewPolicy via ReviewPolicy::as_str {
        HumanOnly,
        AdversarialOnly,
        HumanThenAdversarial,
        AdversarialThenHuman,
    }
}

payload_variants! {
    InquiryLifecycle via InquiryLifecycle::as_str { Open, Resolved, Deferred, Pruned }
}

// --- Group 2: no authority reachable from a const; name the vocabulary here. -
//
// `Provenance` and `ReviewDisposition` are the two `sec-4` lists under `via` and
// `PHASE-01/EX-9` moves here. The reason is mechanical, not stylistic: **every**
// variant of both carries a field, so `Provenance::ShapingQuestion { record:
// String::new() }.label()` in a `const` initialiser drops a temporary (E0493),
// and two of the four are not constructible from this leaf at all —
// `ImportedProse` needs a `DesignId` (private field, no const constructor) and
// `Conducted` a `ReviewRef` (private tuple field, non-const `new`). The single
// source is recovered at test time rather than abandoned: the per-variant walk
// in `tokens_match_serde_renames` asserts each token below equals that variant's
// own `label()` / `arm()`, and the barrier makes that walk total over variants.

payload_variants! {
    Provenance {
        UserDirected    = "user-directed",
        AgentProposed   = "agent-proposed",
        ShapingQuestion = "shaping-question",
        ImportedProse   = "imported-prose",
    }
}

payload_variants! {
    ReviewDisposition {
        Conducted = "conducted",
        Waived    = "waived",
    }
}

payload_variants! {
    DischargeClaim {
        Attested = "attested",
        Skipped  = "skipped",
    }
}

payload_variants! {
    DelegationAct {
        Export  = "export",
        Propose = "propose",
        Accept  = "accept",
        Refuse  = "refuse",
    }
}

payload_variants! {
    AgentAct {
        BlockingSetDeclared = "blocking-set-declared",
        DraftingReady       = "drafting-ready",
    }
}

payload_variants! {
    Reviewer {
        Human       = "human",
        Adversarial = "adversarial",
    }
}

payload_variants! {
    Posture {
        Breadth = "breadth",
        Depth   = "depth",
    }
}

payload_variants! {
    Authority {
        AgentProposed = "agent-proposed",
        UserPinned    = "user-pinned",
        UserLocked    = "user-locked",
    }
}

// `Dispose` names its own four rather than borrowing `DispositionForm::as_str`
// (`inquiry.rs:163`), whose tokens coincide exactly. That is a **different
// type** whose vocabulary merely agrees, and the agreement is unenforced —
// `DispositionForm::RetainUnresolved => "unresolved"` is a hand-written arm and
// not the `rename_all` derivation, which is the standing evidence. Consuming it
// would assert an equivalence nothing holds, and a later divergence would then
// be invisible rather than merely undetected (`sec-4`).
payload_variants! {
    Dispose {
        Create      = "create",
        Adopt       = "adopt",
        Unresolved  = "unresolved",
        NonDurable  = "non-durable",
    }
}

// --- Group 3: untagged; no vocabulary anywhere to take (sec-4). --------------

payload_variants! { WireFacetValue untagged { List, Text } }

// ---------------------------------------------------------------------------
// The contract model (sec-2)
// ---------------------------------------------------------------------------

/// One wire key, wherever it appears in the closure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KeyContract {
    /// The key as a caller spells it.
    pub(crate) key: &'static str,
    /// What may be sent under it.
    pub(crate) ty: WireType,
    /// Whether it may be omitted, and what omission means.
    pub(crate) presence: Presence,
}

/// A struct or enum on the wire, named so a refusal and the contract agree.
///
/// `name` carries the **Rust** type name — `Declaration`, `AgentAct`,
/// `TraversalDeclaration` — because a refusal already names these types and a
/// second vocabulary would leave the caller bridging them (`sec-2`, *Naming*).
/// Nothing here retypes an identity the compiler owns: the enums take theirs
/// from [`payload_variants!`]'s `TYPE_NAME`, the structs from the key-set pin
/// that already holds both the value and its contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TypeContract {
    /// The Rust type name a refusal cites.
    pub(crate) name: &'static str,
    /// Whether it is a struct or an enum, and the rest follows from that.
    pub(crate) form: TypeForm,
}

/// What kind of thing a [`TypeContract`] describes.
///
/// `tagging` sits inside `Enum` and `unknown_keys` inside `Struct`, both for the
/// same reason: a field whose only job is to be meaningless on half its
/// inhabitants is a slot for a wrong answer rather than a fact. A struct has no
/// tagging, and `Stage` — a fieldless externally tagged enum — has no key
/// surface to deny on (`sec-2`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TypeForm {
    /// A struct: a fixed key set, and a disclosure about the keys outside it.
    Struct {
        /// What happens to a key this contract does not list.
        unknown_keys: UnknownKeys,
        /// The keys it does list.
        keys: &'static [KeyContract],
    },
    /// An enum: how the discriminant reaches the wire, and the arms.
    Enum {
        /// Where the token sits, or that there is none.
        tagging: Tagging,
        /// The arms, in declaration order.
        variants: &'static [VariantContract],
    },
}

/// What may be sent under one key — the recursion the closure is walked by.
///
/// Every slice stays `&'static`, and that is a consequence rather than a wish:
/// nothing injected is ever reached through [`WireType::Named`]. The two
/// externally-sourced regions arrive through [`TokenSource::Extern`] and
/// [`MapKey::Extern`] instead (`sec-3`), so the closure edge remains a const
/// reference and there is nothing to borrow-or-own about. An intermediate draft
/// made every row slice here copy-on-write for that reason and `sec-7` records
/// the removal — reintroducing a borrowed-or-owned row fails `PHASE-01/VA-2`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WireType {
    /// A JSON string with no closed vocabulary.
    Text,
    /// A JSON number, integral.
    Integer,
    /// A JSON boolean.
    Boolean,
    /// A run-local id, and **which kinds this key admits** — a bare `id` is the
    /// same omission this slice exists to remove. `declare` admits five of the
    /// eight, `AdoptAuthored.sections` admits `sec-` alone.
    Id(&'static [IdKind]),
    /// The closure edge: another described type.
    Named(&'static TypeContract),
    /// A plain string drawn from a closed set, with **no Rust type name a caller
    /// could ever see** — what a facet field's `FieldShape::Closed` actually is
    /// on the wire. `Named` pointing at an invented enum would say something
    /// false about the payload.
    Token(TokenSource),
    /// A JSON array of one element type.
    Seq(&'static WireType),
    /// A JSON object used as a map. The key is a description, not an
    /// afterthought: the closure's two maps are keyed by a section id and by a
    /// vocabulary a sibling field selects.
    Map {
        /// What the keys may be.
        key: MapKey,
        /// What the values may be.
        value: &'static WireType,
    },
}

/// Where a [`WireType::Token`]'s closed vocabulary comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TokenSource {
    /// Named here, because this tier can see it.
    Fixed(&'static [&'static str]),
    /// Supplied by a region this tier cannot import (`sec-3`).
    Extern(ExternRegion),
}

/// What a map's keys may be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MapKey {
    /// Keys are values of this wire type — `AdoptAuthored.sections` is keyed by
    /// section id, and "map of text" would have lost that.
    Of(&'static WireType),
    /// Keys are supplied by a region this tier cannot import, and *which* keys
    /// are legal is chosen by the value of a sibling field named here.
    /// `CreateRecord.facet`'s keys are the facet fields of the kind in `kind`.
    ///
    /// The only place in this model where one key's contract depends on another
    /// key's **value** — and that dependency is real: it is the whole of what
    /// makes `facet` discoverable rather than an open bag.
    Extern {
        /// Whose vocabulary this is.
        region: ExternRegion,
        /// The sibling key whose value selects the key set. A plain field name,
        /// not a type from another tier, so it costs ADR-001 nothing.
        selector: &'static str,
    },
}

/// One enum variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VariantContract {
    /// The token a caller writes. `None` only under [`Tagging::Untagged`], where
    /// serde emits no token at all and printing the Rust variant name would say
    /// something false.
    pub(crate) token: Option<&'static str>,
    /// What rides with it.
    pub(crate) payload: VariantPayload,
}

/// What a variant carries. **Where those keys sit on the wire is a function of
/// [`Tagging`] and the payload together, and is derived rather than stored** —
/// which is what keeps the two from disagreeing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VariantPayload {
    /// A unit variant.
    Absent,
    /// A struct variant's own keys.
    Keys(&'static [KeyContract]),
    /// A newtype variant over a named type, whose keys arrive in this variant's
    /// place — `Dispose::Create(CreateRecord)`. A rendering has to be able to
    /// *name* `CreateRecord` rather than silently re-listing its keys.
    Inlines(&'static TypeContract),
    /// An untagged variant, which is a shape rather than a set of keys —
    /// `WireFacetValue`'s `[text]` and `text`.
    Shape(&'static WireType),
}

/// How an enum's discriminant reaches the wire.
///
/// Three sibling act types spell theirs three different ways —
/// `DelegationAct` internally on `tag = "act"`, `AgentAct` externally, and
/// `CheckpointActDeclaration.act` as a bare `ActKind`. A contract stating field
/// types alone would have said nothing useful about any of them.
///
/// There is no `Bare` arm. *Bare* is what `External` + [`VariantPayload::Absent`]
/// already produces for every variant of `Stage`, `ActKind`, `Reviewer` and the
/// rest; storing it beside the payload would give a `Bare` claim something to
/// contradict. It survives as a word the **renderer** prints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tagging {
    /// `{ "act": "export", … }` — the token is the value at this key, and a
    /// payload's keys sit **beside** it.
    Internal(&'static str),
    /// `{ "blocking-set-declared": { … } }` for a variant with a payload, and a
    /// **bare string** `"drafting-ready"` for one without. `AgentAct` is mixed
    /// and that asymmetry is per-variant, never per-type.
    External,
    /// Discriminated by JSON shape alone, with no tag anywhere.
    Untagged,
}

/// Whether a key may be omitted, and what omission means. **Three states, not
/// two.**
///
/// `Sparse<T>` is the run's editing idiom and it is invisible from a type
/// signature: omitting `parent` persists the existing value, sending `null`
/// clears it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Presence {
    /// It must be sent.
    Required,
    /// Absent means absent.
    Optional,
    /// Absent means *persist*; `null` means *clear*.
    Sparse,
}

/// What happens to a key a struct's contract does not list — the honest
/// disclosure of `ISS-333`.
///
/// **Two states, and an earlier draft's third was false.** A `StoredThenFlagged`
/// arm — accepted, written, reported afterwards by `doctor` — describes
/// behaviour this codebase does not have: an unrecognised facet key on a
/// `create` is **refused** before an id is reserved. A bool would also have been
/// the wrong type here: `denies_unknown: false` reads two ways, *accepted and
/// stored* or *accepted and thrown away*, and only one of them is true.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnknownKeys {
    /// `deny_unknown_fields` — a misspelt key is a refusal.
    Refused,
    /// `serde(flatten)` forbids `deny_unknown_fields`, and eight more wire
    /// structs simply carry no attribute — the key is accepted and discarded in
    /// silence, and a *correct* submission prints no change row either, so the
    /// caller has no observable that separates *landed* from *discarded*.
    SilentlyDropped,
}

// ---------------------------------------------------------------------------
// The extern seam (sec-3)
// ---------------------------------------------------------------------------

/// A region of the closure owned by a tier this one cannot import. Closed, so
/// naming a region and supplying one are the same act.
///
/// Closed and not `Extern(&'static str)`: the precedent this borrows from —
/// [`super::prompt`]'s `contract_block`, keyed by a closed `Condition` rather
/// than by a string — is safe *because* of the closure, and a string-keyed
/// supply map would be the precedent minus the thing that makes it work. One
/// member today; the cardinality is not the point, the closure is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ExternRegion {
    /// `knowledge::RecordKind` and, per kind, `facet_fields`. **One** region and
    /// not two: `kind`'s admissible tokens and `facet`'s admissible keys are
    /// both functions of `RecordKind`, and the second is indexed by the first —
    /// two lists would be free to disagree about which kinds exist.
    KnowledgeRecord,
}

impl ExternRegion {
    /// Every region — the closed set, single-sourced.
    pub(crate) const ALL: [ExternRegion; 1] = [ExternRegion::KnowledgeRecord];

    /// The source this region names, rendered. **One spelling** (STD-001): the
    /// builder above this tier reads it rather than retyping it, and so does
    /// [`SelectorTable::source`].
    pub(crate) const fn label(self) -> &'static str {
        match self {
            ExternRegion::KnowledgeRecord => "knowledge::RecordKind",
        }
    }
}

/// Key sets chosen by the value of a sibling field.
///
/// **Not a type on the wire and not a tagged union**: `kind` and `facet` are two
/// ordinary keys of the same `CreateRecord` object. A draft that pressed this
/// into an enum-form [`TypeContract`] was wrong twice over — no [`Tagging`]
/// value is true of it (no tag key, no nesting under the token, and the rows do
/// carry tokens), and [`TypeContract::name`] is the Rust type name a refusal
/// cites while nothing here is a type a refusal ever names.
///
/// `Vec` rather than `&'static`, and **only here**: these rows are assembled at
/// run time by the command tier from `RecordKind::ALL` and `facet_fields`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectorTable {
    /// Where a caller can go and read the region. One spelling (STD-001) —
    /// [`ExternRegion::label`]'s.
    pub(crate) source: &'static str,
    /// Each admissible value of the selecting field, with the keys it opens.
    pub(crate) rows: Vec<SelectedKeys>,
    /// `Refused` — an unknown facet key is rejected before the mint, not stored.
    pub(crate) unknown_keys: UnknownKeys,
}

/// One selector value and the keys it opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectedKeys {
    /// The selecting field's value — a `RecordKind` token.
    pub(crate) token: &'static str,
    /// The keys legal when the sibling field holds `token`.
    pub(crate) keys: Vec<KeyContract>,
}

/// What the tier above supplies for every region this leaf declares.
///
/// **A region nobody supplies is made unspellable rather than tested**: one
/// field per region plus [`ExternContracts::region`]'s exhaustive match means
/// adding a region to [`ExternRegion`] without supplying it is a
/// non-exhaustive-match error, and there is no string key left to typo. This
/// seam is deliberately stricter than `contract_block`, which tolerates an
/// absent body — an absent extern contract renders the region as nothing,
/// silently, which is the discoverability failure itself rather than a degraded
/// report of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExternContracts {
    /// [`ExternRegion::KnowledgeRecord`]'s supply.
    pub(crate) knowledge_record: SelectorTable,
}

impl ExternContracts {
    /// The table supplied for `region`.
    ///
    /// An exhaustive match **with no wildcard arm** — that is the barrier, and
    /// the reason `sec-8` lists this among the things that need no test.
    pub(crate) const fn region(&self, region: ExternRegion) -> &SelectorTable {
        match region {
            ExternRegion::KnowledgeRecord => &self.knowledge_record,
        }
    }
}

// ---------------------------------------------------------------------------
// The closure, described (sec-3)
// ---------------------------------------------------------------------------
//
// `sec-3` bounds the closure at **twelve struct types** and fourteen enums, and
// every one of the twelve structs is declared in [`super::submission`].
//
// Eleven of the twelve carry a [`TypeContract`] of their own below —
// [`PAYLOAD`] itself being `ApplyRequest`'s. `SubmissionEnvelope` carries none,
// and that is not an omission: `#[serde(flatten)]` renders its three keys at the
// root, so it participates through `PAYLOAD`'s composition and has no key
// surface of its own to describe. Its pin is the disjoint union in the §9.1
// suite rather than a contract here.
//
// That split is also why the `unknown_keys` census reads 3 + 8 below while
// `sec-2` reads 3 + 9: three closure structs carry `deny_unknown_fields`
// (`Declaration`, `CheckpointActDeclaration`, `AgentActDeclaration`) and the
// other **nine** do not, but the ninth of those nine is `SubmissionEnvelope`,
// which has no contract to state it on. Eight contracts say `SilentlyDropped`;
// the envelope's keys inherit the root's, which is the same answer.
//
// **Every row below is a claim about the wire, and the claims are pinned rather
// than trusted.** `sec-8` pin 1 holds each struct contract's key set against a
// fully populated value's serde output and its `name` against the Rust type's
// own; pin 4 holds the enum vocabularies against serde's renames. A variant's
// `token` is therefore written here as a literal on purpose — it is the
// contract's claim, checked against `VARIANTS` and serde, not a second spelling
// of an authority already in scope. The type names are not written twice at all:
// the enums take theirs from [`payload_variants!`]'s `TYPE_NAME` (STD-001).
//
// **Written out rather than generated**, and the eight all-bare enums are where
// that costs most visibly. A second macro over them would have to be handed the
// same token list [`payload_variants!`] is already handed, in the same file —
// which is duplication introduced in the name of removing it — and `const` has no
// way to map `VARIANTS` into a `&'static [VariantContract]` without one. So the
// rows stay explicit, where clippy lints them and a reviewer can read them.

// --- The enums (sec-3): fourteen, each `Tagging` read off `sec-4`'s barrier. --

/// `Stage`, externally tagged with no payload anywhere — a bare string.
pub(crate) static STAGE: TypeContract = TypeContract {
    name: Stage::TYPE_NAME,
    form: TypeForm::Enum {
        tagging: Tagging::External,
        variants: &[
            VariantContract {
                token: Some("exploring"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("inquiring"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("drafting"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("reviewing"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("locked"),
                payload: VariantPayload::Absent,
            },
        ],
    },
};

/// `ActKind` — the eight checkpoint acts, each a bare string.
pub(crate) static ACT_KIND: TypeContract = TypeContract {
    name: ActKind::TYPE_NAME,
    form: TypeForm::Enum {
        tagging: Tagging::External,
        variants: &[
            VariantContract {
                token: Some("governance-confirmed"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("graph-reviewed"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("blocking-set-declared"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("sufficiency-accepted"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("drafting-ready"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("section-reviewed"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("review-disposed"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("design-accepted"),
                payload: VariantPayload::Absent,
            },
        ],
    },
};

/// `ReviewPolicy` — which reviewer lanes a run requires.
pub(crate) static REVIEW_POLICY: TypeContract = TypeContract {
    name: ReviewPolicy::TYPE_NAME,
    form: TypeForm::Enum {
        tagging: Tagging::External,
        variants: &[
            VariantContract {
                token: Some("human-only"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("adversarial-only"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("human-then-adversarial"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("adversarial-then-human"),
                payload: VariantPayload::Absent,
            },
        ],
    },
};

/// `InquiryLifecycle` — a node's non-resolving lifecycle states.
pub(crate) static INQUIRY_LIFECYCLE: TypeContract = TypeContract {
    name: InquiryLifecycle::TYPE_NAME,
    form: TypeForm::Enum {
        tagging: Tagging::External,
        variants: &[
            VariantContract {
                token: Some("open"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("resolved"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("deferred"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("pruned"),
                payload: VariantPayload::Absent,
            },
        ],
    },
};

/// `Reviewer` — who reviewed.
pub(crate) static REVIEWER: TypeContract = TypeContract {
    name: Reviewer::TYPE_NAME,
    form: TypeForm::Enum {
        tagging: Tagging::External,
        variants: &[
            VariantContract {
                token: Some("human"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("adversarial"),
                payload: VariantPayload::Absent,
            },
        ],
    },
};

/// `Posture` — how the map is being walked.
pub(crate) static POSTURE: TypeContract = TypeContract {
    name: Posture::TYPE_NAME,
    form: TypeForm::Enum {
        tagging: Tagging::External,
        variants: &[
            VariantContract {
                token: Some("breadth"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("depth"),
                payload: VariantPayload::Absent,
            },
        ],
    },
};

/// `Authority` — on whose authority a traversal moved.
pub(crate) static AUTHORITY: TypeContract = TypeContract {
    name: Authority::TYPE_NAME,
    form: TypeForm::Enum {
        tagging: Tagging::External,
        variants: &[
            VariantContract {
                token: Some("agent-proposed"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("user-pinned"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("user-locked"),
                payload: VariantPayload::Absent,
            },
        ],
    },
};

/// `DischargeClaim` — what a caller may claim about one runbook step.
pub(crate) static DISCHARGE_CLAIM: TypeContract = TypeContract {
    name: DischargeClaim::TYPE_NAME,
    form: TypeForm::Enum {
        tagging: Tagging::External,
        variants: &[
            VariantContract {
                token: Some("attested"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("skipped"),
                payload: VariantPayload::Absent,
            },
        ],
    },
};

/// `Provenance` — where a node came from, internally tagged so a variant's own
/// keys sit **beside** the tag rather than under it.
pub(crate) static PROVENANCE: TypeContract = TypeContract {
    name: Provenance::TYPE_NAME,
    form: TypeForm::Enum {
        tagging: Tagging::Internal("provenance"),
        variants: &[
            VariantContract {
                token: Some("user-directed"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("agent-proposed"),
                payload: VariantPayload::Absent,
            },
            VariantContract {
                token: Some("shaping-question"),
                payload: VariantPayload::Keys(&[KeyContract {
                    key: "record",
                    ty: WireType::Text,
                    presence: Presence::Required,
                }]),
            },
            VariantContract {
                token: Some("imported-prose"),
                payload: VariantPayload::Keys(&[
                    KeyContract {
                        key: "section",
                        ty: WireType::Id(&[IdKind::Section]),
                        presence: Presence::Required,
                    },
                    KeyContract {
                        key: "line",
                        ty: WireType::Integer,
                        presence: Presence::Required,
                    },
                    KeyContract {
                        key: "label",
                        ty: WireType::Text,
                        presence: Presence::Required,
                    },
                    KeyContract {
                        key: "fingerprint",
                        ty: WireType::Text,
                        presence: Presence::Required,
                    },
                ]),
            },
        ],
    },
};

/// `ReviewDisposition` — externally tagged, so each variant's keys arrive nested
/// under its token.
pub(crate) static REVIEW_DISPOSITION: TypeContract = TypeContract {
    name: ReviewDisposition::TYPE_NAME,
    form: TypeForm::Enum {
        tagging: Tagging::External,
        variants: &[
            VariantContract {
                token: Some("conducted"),
                payload: VariantPayload::Keys(&[KeyContract {
                    key: "review",
                    ty: WireType::Text,
                    presence: Presence::Required,
                }]),
            },
            VariantContract {
                token: Some("waived"),
                payload: VariantPayload::Keys(&[KeyContract {
                    key: "reason",
                    ty: WireType::Text,
                    presence: Presence::Required,
                }]),
            },
        ],
    },
};

/// `AgentAct` — the live **mixed** case: externally tagged, one variant nesting
/// under its token and the other a bare string. The asymmetry is per-variant,
/// which is why it is read off `payload` rather than off `tagging`.
pub(crate) static AGENT_ACT: TypeContract = TypeContract {
    name: AgentAct::TYPE_NAME,
    form: TypeForm::Enum {
        tagging: Tagging::External,
        variants: &[
            VariantContract {
                token: Some("blocking-set-declared"),
                payload: VariantPayload::Keys(&[KeyContract {
                    key: "blocking",
                    ty: WireType::Seq(&WireType::Id(&[IdKind::Inquiry])),
                    presence: Presence::Required,
                }]),
            },
            VariantContract {
                token: Some("drafting-ready"),
                payload: VariantPayload::Absent,
            },
        ],
    },
};

/// `DelegationAct` — internally tagged on `act`, so each variant's keys sit
/// beside the tag. `propose`'s `declare` is `#[serde(default)]` and therefore
/// omissible (`PHASE-02/EX-11`).
pub(crate) static DELEGATION_ACT: TypeContract = TypeContract {
    name: DelegationAct::TYPE_NAME,
    form: TypeForm::Enum {
        tagging: Tagging::Internal("act"),
        variants: &[
            VariantContract {
                token: Some("export"),
                payload: VariantPayload::Keys(&[
                    KeyContract {
                        key: "id",
                        ty: WireType::Id(&[IdKind::Delegation]),
                        presence: Presence::Required,
                    },
                    KeyContract {
                        key: "obligation",
                        ty: WireType::Id(&[IdKind::Inquiry]),
                        presence: Presence::Required,
                    },
                ]),
            },
            VariantContract {
                token: Some("propose"),
                payload: VariantPayload::Keys(&[
                    KeyContract {
                        key: "id",
                        ty: WireType::Id(&[IdKind::Delegation]),
                        presence: Presence::Required,
                    },
                    KeyContract {
                        key: "by",
                        ty: WireType::Text,
                        presence: Presence::Required,
                    },
                    KeyContract {
                        key: "summary",
                        ty: WireType::Text,
                        presence: Presence::Required,
                    },
                    KeyContract {
                        key: "declare",
                        ty: WireType::Seq(&WireType::Named(&DECLARATION)),
                        presence: Presence::Optional,
                    },
                ]),
            },
            VariantContract {
                token: Some("accept"),
                payload: VariantPayload::Keys(&[KeyContract {
                    key: "id",
                    ty: WireType::Id(&[IdKind::Delegation]),
                    presence: Presence::Required,
                }]),
            },
            VariantContract {
                token: Some("refuse"),
                payload: VariantPayload::Keys(&[
                    KeyContract {
                        key: "id",
                        ty: WireType::Id(&[IdKind::Delegation]),
                        presence: Presence::Required,
                    },
                    KeyContract {
                        key: "reason",
                        ty: WireType::Text,
                        presence: Presence::Required,
                    },
                ]),
            },
        ],
    },
};

/// `Dispose` — internally tagged on `form`. `create` **inlines** `CreateRecord`
/// by name rather than re-listing its keys, which is what lets a rendering say
/// which type arrived.
pub(crate) static DISPOSE: TypeContract = TypeContract {
    name: Dispose::TYPE_NAME,
    form: TypeForm::Enum {
        tagging: Tagging::Internal("form"),
        variants: &[
            VariantContract {
                token: Some("create"),
                payload: VariantPayload::Inlines(&CREATE_RECORD),
            },
            VariantContract {
                token: Some("adopt"),
                payload: VariantPayload::Keys(&[KeyContract {
                    key: "record",
                    ty: WireType::Text,
                    presence: Presence::Required,
                }]),
            },
            VariantContract {
                token: Some("unresolved"),
                payload: VariantPayload::Keys(&[KeyContract {
                    key: "note",
                    ty: WireType::Text,
                    presence: Presence::Required,
                }]),
            },
            VariantContract {
                token: Some("non-durable"),
                payload: VariantPayload::Keys(&[KeyContract {
                    key: "note",
                    ty: WireType::Text,
                    presence: Presence::Required,
                }]),
            },
        ],
    },
};

/// `WireFacetValue` — untagged, so each variant is a *shape* and carries no
/// token at all.
pub(crate) static WIRE_FACET_VALUE: TypeContract = TypeContract {
    name: WireFacetValue::TYPE_NAME,
    form: TypeForm::Enum {
        tagging: Tagging::Untagged,
        variants: &[
            VariantContract {
                token: None,
                payload: VariantPayload::Shape(&WireType::Seq(&WireType::Text)),
            },
            VariantContract {
                token: None,
                payload: VariantPayload::Shape(&WireType::Text),
            },
        ],
    },
};

// --- The structs (sec-3): eleven contracts, leaves first. --------------------

/// `AcceptanceDeclaration` — the user-acceptance half a caller may supply.
pub(crate) static ACCEPTANCE_DECLARATION: TypeContract = TypeContract {
    name: "AcceptanceDeclaration",
    form: TypeForm::Struct {
        unknown_keys: UnknownKeys::SilentlyDropped,
        keys: &[
            KeyContract {
                key: "basis",
                ty: WireType::Text,
                presence: Presence::Required,
            },
            KeyContract {
                key: "turn",
                ty: WireType::Text,
                presence: Presence::Optional,
            },
        ],
    },
};

/// `StageDeclaration` — a declared stage move.
pub(crate) static STAGE_DECLARATION: TypeContract = TypeContract {
    name: "StageDeclaration",
    form: TypeForm::Struct {
        unknown_keys: UnknownKeys::SilentlyDropped,
        keys: &[
            KeyContract {
                key: "to",
                ty: WireType::Named(&STAGE),
                presence: Presence::Required,
            },
            KeyContract {
                key: "reason",
                ty: WireType::Text,
                presence: Presence::Optional,
            },
        ],
    },
};

/// `DischargeDeclaration` — one runbook discharge.
///
/// `outcome` is `Named(&DISCHARGE_CLAIM)` and not a `Token`: a Rust enum a
/// refusal can name is a named type, and the exemplar's `Token(Fixed(…))` row
/// exists to reach that arm of the model, not to describe this key.
pub(crate) static DISCHARGE_DECLARATION: TypeContract = TypeContract {
    name: "DischargeDeclaration",
    form: TypeForm::Struct {
        unknown_keys: UnknownKeys::SilentlyDropped,
        keys: &[
            KeyContract {
                key: "step",
                ty: WireType::Text,
                presence: Presence::Required,
            },
            KeyContract {
                key: "outcome",
                ty: WireType::Named(&DISCHARGE_CLAIM),
                presence: Presence::Required,
            },
            KeyContract {
                key: "reason",
                ty: WireType::Text,
                presence: Presence::Optional,
            },
        ],
    },
};

/// `AdoptAuthored` — the sole lawful crossing of an authored-watermark
/// divergence.
///
/// `sections` is `EX-4`'s second row: a map keyed by **section id alone**, which
/// "map of text" would have lost. It is `#[serde(default)]` and so omissible
/// (`PHASE-02/EX-11`).
pub(crate) static ADOPT_AUTHORED: TypeContract = TypeContract {
    name: "AdoptAuthored",
    form: TypeForm::Struct {
        unknown_keys: UnknownKeys::SilentlyDropped,
        keys: &[
            KeyContract {
                key: "fingerprint",
                ty: WireType::Text,
                presence: Presence::Required,
            },
            KeyContract {
                key: "sections",
                ty: WireType::Map {
                    key: MapKey::Of(&WireType::Id(&[IdKind::Section])),
                    value: &WireType::Text,
                },
                presence: Presence::Optional,
            },
        ],
    },
};

/// `TraversalDeclaration` — the two `Sparse<T>` keys of the closure's second
/// sparse-bearing struct: omitting `pin` persists it, `null` clears it.
pub(crate) static TRAVERSAL_DECLARATION: TypeContract = TypeContract {
    name: "TraversalDeclaration",
    form: TypeForm::Struct {
        unknown_keys: UnknownKeys::SilentlyDropped,
        keys: &[
            KeyContract {
                key: "pin",
                ty: WireType::Id(&[IdKind::Inquiry]),
                presence: Presence::Sparse,
            },
            KeyContract {
                key: "cursor",
                ty: WireType::Id(&[IdKind::Inquiry]),
                presence: Presence::Sparse,
            },
            KeyContract {
                key: "posture",
                ty: WireType::Named(&POSTURE),
                presence: Presence::Optional,
            },
            KeyContract {
                key: "authority",
                ty: WireType::Named(&AUTHORITY),
                presence: Presence::Optional,
            },
        ],
    },
};

/// `CreateRecord` — what a `create` disposition asks Doctrine to materialise,
/// and the closure's **one externally-sourced region** (`sec-3`, `EX-9`).
///
/// `kind` is a `Token` drawn from `knowledge::RecordKind`, and `facet` a `Map`
/// whose admissible keys are chosen by the value of the sibling `kind`. Neither
/// is a `Named` edge, and the difference is not cosmetic: there is no Rust type
/// a refusal could cite for either, so a `Named` pointing at an invented type
/// would state something false about the wire. The vocabulary itself arrives
/// through [`ExternRegion`], never by import — this file is ADR-001 leaf tier.
pub(crate) static CREATE_RECORD: TypeContract = TypeContract {
    name: "CreateRecord",
    form: TypeForm::Struct {
        unknown_keys: UnknownKeys::SilentlyDropped,
        keys: &[
            KeyContract {
                key: "kind",
                ty: WireType::Token(TokenSource::Extern(ExternRegion::KnowledgeRecord)),
                presence: Presence::Required,
            },
            KeyContract {
                key: "title",
                ty: WireType::Text,
                presence: Presence::Required,
            },
            KeyContract {
                key: "slug",
                ty: WireType::Text,
                presence: Presence::Optional,
            },
            KeyContract {
                key: "body",
                ty: WireType::Text,
                presence: Presence::Optional,
            },
            KeyContract {
                key: "facet",
                ty: WireType::Map {
                    key: MapKey::Extern {
                        region: ExternRegion::KnowledgeRecord,
                        selector: "kind",
                    },
                    value: &WireType::Named(&WIRE_FACET_VALUE),
                },
                presence: Presence::Optional,
            },
            KeyContract {
                key: "acceptance",
                ty: WireType::Named(&ACCEPTANCE_DECLARATION),
                presence: Presence::Optional,
            },
        ],
    },
};

/// `ReviewPolicyDeclaration` — a change to the run's review policy.
///
/// `acceptance` is **required** here and optional everywhere else it appears:
/// the policy is mutable on purpose, and what the design buys is visibility
/// rather than prohibition.
pub(crate) static REVIEW_POLICY_DECLARATION: TypeContract = TypeContract {
    name: "ReviewPolicyDeclaration",
    form: TypeForm::Struct {
        unknown_keys: UnknownKeys::SilentlyDropped,
        keys: &[
            KeyContract {
                key: "policy",
                ty: WireType::Named(&REVIEW_POLICY),
                presence: Presence::Required,
            },
            KeyContract {
                key: "acceptance",
                ty: WireType::Named(&ACCEPTANCE_DECLARATION),
                presence: Presence::Required,
            },
        ],
    },
};

/// `CheckpointActDeclaration` — a user act at a checkpoint. One of the three
/// closure structs carrying `deny_unknown_fields`, so a misspelt key is a
/// refusal rather than a key serde swallows.
pub(crate) static CHECKPOINT_ACT_DECLARATION: TypeContract = TypeContract {
    name: "CheckpointActDeclaration",
    form: TypeForm::Struct {
        unknown_keys: UnknownKeys::Refused,
        keys: &[
            KeyContract {
                key: "act",
                ty: WireType::Named(&ACT_KIND),
                presence: Presence::Required,
            },
            KeyContract {
                key: "acceptance",
                ty: WireType::Named(&ACCEPTANCE_DECLARATION),
                presence: Presence::Required,
            },
            KeyContract {
                key: "disposition",
                ty: WireType::Named(&REVIEW_DISPOSITION),
                presence: Presence::Optional,
            },
        ],
    },
};

/// `AgentActDeclaration` — an agent's declaration about its own work. The second
/// of the three structs that refuse unknown keys.
pub(crate) static AGENT_ACT_DECLARATION: TypeContract = TypeContract {
    name: "AgentActDeclaration",
    form: TypeForm::Struct {
        unknown_keys: UnknownKeys::Refused,
        keys: &[
            KeyContract {
                key: "act",
                ty: WireType::Named(&AGENT_ACT),
                presence: Presence::Required,
            },
            KeyContract {
                key: "basis",
                ty: WireType::Text,
                presence: Presence::Required,
            },
            KeyContract {
                key: "turn",
                ty: WireType::Text,
                presence: Presence::Optional,
            },
        ],
    },
};

/// `Declaration` — the flat per-subject struct, and the third that refuses
/// unknown keys.
///
/// `subject` carries `EX-4`'s five-kind row: `super::run::declare` routes on the
/// subject's **kind** and admits `inq-`, `sec-`, `att-`, `fnd-` and `cp-`,
/// refusing the other three of `IdKind`'s eight. The set lives here rather than
/// on `ApplyRequest.declare`, which is a sequence of these and carries no id of
/// its own (`PHASE-02/EX-11`).
///
/// `resolved_record` is **not** a row, and no exception list says so: it carries
/// `#[serde(skip)]`, which puts it outside the wire by construction (`EX-6`).
pub(crate) static DECLARATION: TypeContract = TypeContract {
    name: "Declaration",
    form: TypeForm::Struct {
        unknown_keys: UnknownKeys::Refused,
        keys: &[
            KeyContract {
                key: "subject",
                ty: WireType::Id(&[
                    IdKind::Inquiry,
                    IdKind::Section,
                    IdKind::Attestation,
                    IdKind::Finding,
                    IdKind::Checkpoint,
                ]),
                presence: Presence::Required,
            },
            KeyContract {
                key: "question",
                ty: WireType::Text,
                presence: Presence::Sparse,
            },
            KeyContract {
                key: "needs",
                ty: WireType::Seq(&WireType::Id(&[IdKind::Inquiry])),
                presence: Presence::Sparse,
            },
            KeyContract {
                key: "parent",
                ty: WireType::Id(&[IdKind::Inquiry]),
                presence: Presence::Sparse,
            },
            KeyContract {
                key: "provenance",
                ty: WireType::Named(&PROVENANCE),
                presence: Presence::Optional,
            },
            KeyContract {
                key: "lifecycle",
                ty: WireType::Named(&INQUIRY_LIFECYCLE),
                presence: Presence::Optional,
            },
            KeyContract {
                key: "body",
                ty: WireType::Text,
                presence: Presence::Optional,
            },
            KeyContract {
                key: "attests",
                ty: WireType::Id(&[IdKind::Section]),
                presence: Presence::Optional,
            },
            KeyContract {
                key: "reviewer",
                ty: WireType::Named(&REVIEWER),
                presence: Presence::Optional,
            },
            KeyContract {
                key: "concerns",
                ty: WireType::Id(&[IdKind::Section]),
                presence: Presence::Optional,
            },
            KeyContract {
                key: "summary",
                ty: WireType::Text,
                presence: Presence::Optional,
            },
            KeyContract {
                key: "blocking",
                ty: WireType::Boolean,
                presence: Presence::Optional,
            },
            KeyContract {
                key: "resolution",
                ty: WireType::Text,
                presence: Presence::Optional,
            },
            KeyContract {
                key: "disposes",
                ty: WireType::Id(&[IdKind::Inquiry]),
                presence: Presence::Optional,
            },
            KeyContract {
                key: "dispose",
                ty: WireType::Named(&DISPOSE),
                presence: Presence::Optional,
            },
        ],
    },
};

/// The root: one `apply` payload, at one level (`EX-1`).
///
/// **Thirteen keys.** Three are `SubmissionEnvelope`'s, flattened here — which is
/// the whole of how that twelfth closure struct participates — and ten are the
/// act fields, in the order `ApplyRequest` declares them. Ten, not the nine of
/// [`super::submission::ApplyRequest::WRITER_ACTS`]: that list correctly omits
/// `delegation`, whose acts are not all writes, and reading the payload's key
/// count off it would reproduce the omission this contract exists to close.
///
/// `unknown_keys` is `SilentlyDropped` and cannot be anything else:
/// `#[serde(flatten)]` and `deny_unknown_fields` are mutually exclusive, so a
/// misspelt top-level key is discarded in silence (`ISS-333`).
pub(crate) const PAYLOAD: TypeContract = TypeContract {
    name: "ApplyRequest",
    form: TypeForm::Struct {
        unknown_keys: UnknownKeys::SilentlyDropped,
        keys: &[
            KeyContract {
                key: "run_uid",
                ty: WireType::Text,
                presence: Presence::Required,
            },
            KeyContract {
                key: "known_revision",
                ty: WireType::Integer,
                presence: Presence::Required,
            },
            KeyContract {
                key: "submission_id",
                ty: WireType::Text,
                presence: Presence::Required,
            },
            KeyContract {
                key: "adopt_authored",
                ty: WireType::Named(&ADOPT_AUTHORED),
                presence: Presence::Optional,
            },
            KeyContract {
                key: "traversal",
                ty: WireType::Named(&TRAVERSAL_DECLARATION),
                presence: Presence::Optional,
            },
            KeyContract {
                key: "stage",
                ty: WireType::Named(&STAGE_DECLARATION),
                presence: Presence::Optional,
            },
            KeyContract {
                key: "acceptance",
                ty: WireType::Named(&ACCEPTANCE_DECLARATION),
                presence: Presence::Optional,
            },
            KeyContract {
                key: "declare",
                ty: WireType::Seq(&WireType::Named(&DECLARATION)),
                presence: Presence::Optional,
            },
            KeyContract {
                key: "delegation",
                ty: WireType::Named(&DELEGATION_ACT),
                presence: Presence::Optional,
            },
            KeyContract {
                key: "discharge",
                ty: WireType::Named(&DISCHARGE_DECLARATION),
                presence: Presence::Optional,
            },
            KeyContract {
                key: "review_policy",
                ty: WireType::Named(&REVIEW_POLICY_DECLARATION),
                presence: Presence::Optional,
            },
            KeyContract {
                key: "checkpoint_act",
                ty: WireType::Named(&CHECKPOINT_ACT_DECLARATION),
                presence: Presence::Optional,
            },
            KeyContract {
                key: "agent_declaration",
                ty: WireType::Named(&AGENT_ACT_DECLARATION),
                presence: Presence::Optional,
            },
        ],
    },
};

// ---------------------------------------------------------------------------
// Where the contract is published (sec-6)
// ---------------------------------------------------------------------------

/// How an agent asks for the contract — the pointer a refusal and the run's
/// prompts cite. Spelled once (STD-001).
#[expect(
    dead_code,
    reason = "SL-251 PHASE-06/07 land the first readers — the renderer and the CLI surface"
)]
pub(crate) const PAYLOAD_CONTRACT_POINTER: &str = "doctrine design contract --format prompt";

/// The committed file the generated contract is pinned to, relative to the repo
/// root — [`super::artifact::ARTIFACT_PATH`]'s pattern. Spelled once (STD-001).
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SL-251: the golden and the regeneration test are the only readers; the path never ships"
    )
)]
pub(crate) const PAYLOAD_CONTRACT_PATH: &str = "install/design-payload-contract.md";

// ---------------------------------------------------------------------------
// One generator, three consumers (sec-5)
// ---------------------------------------------------------------------------
//
// `render_json`, `render_prompt` and `render_document` share the walk over
// [`PAYLOAD`] and differ only in emission — three entry points rather than one
// with a format parameter, because a JSON document, a line sequence and a
// Markdown page are genuinely different shapes and one function returning one of
// three unrelated things would be a `match` pretending to be an abstraction
// (`sec-5`).
//
// **The walk is a graph walk, not a recursion into the emission.**
// [`WireType::Named`] holds the whole [`TypeContract`], and the closure is a
// graph: `Declaration` is reached twice and `AcceptanceDeclaration` five times.
// So [`closure_types`] collects types **by name** into a flat table, each
// rendering emits each type once, and every `Named` edge is emitted as a *name*.
// A rendering that recursed instead would look right and be quadratically
// redundant, and a consumer could not tell the two `Declaration` inlinings are
// one type — which is the single thing a machine-readable contract exists to say
// (`fnd-10`).
//
// Everything here is pure. Nothing prints, writes, embeds or publishes.

/// The JSON document's schema name — [`super::render`]'s `TurnEnvelope`
/// precedent, so a consumer branches on a declared name rather than sniffing for
/// keys. Spelled once (STD-001).
pub(crate) const PAYLOAD_CONTRACT_SCHEMA: &str = "design-payload-contract";

/// The JSON document's schema version, on the same precedent.
pub(crate) const PAYLOAD_CONTRACT_VERSION: u64 = 1;

/// The wire scalars, and the word an id row is spelled with. **One spelling
/// each** (STD-001): both renderings below say the same thing about the same
/// wire, so neither may say it in its own words.
const TEXT: &str = "text";
const INTEGER: &str = "integer";
const BOOLEAN: &str = "boolean";
const ID: &str = "id";

/// A struct's disclosure about the keys its contract does not list. Spelled
/// once, and read by both renderings and the extern region's block.
const UNKNOWN_KEYS: &str = "unknown-keys";

/// What a block leads with: the root's, and every other type's.
const ROOT_LEAD: &str = "payload";
const TYPE_LEAD: &str = "type";

/// Every type reachable from `root`, in first-visit order, **each once**.
///
/// The `BTreeSet` of names is what makes this a graph walk: a type met a second
/// time contributes its name to an edge and nothing else. It is also why a cycle
/// could not hang the walk, though the closure has none (`sec-3`).
fn closure_types(root: &'static TypeContract) -> Vec<&'static TypeContract> {
    let mut seen = BTreeSet::new();
    let mut order = Vec::new();
    visit_type(root, &mut seen, &mut order);
    order
}

fn visit_type(
    contract: &'static TypeContract,
    seen: &mut BTreeSet<&'static str>,
    order: &mut Vec<&'static TypeContract>,
) {
    if !seen.insert(contract.name) {
        return;
    }
    order.push(contract);
    match contract.form {
        TypeForm::Struct { keys, .. } => {
            for key in keys {
                visit_wire(&key.ty, seen, order);
            }
        }
        TypeForm::Enum { variants, .. } => {
            for variant in variants {
                visit_payload(variant.payload, seen, order);
            }
        }
    }
}

fn visit_payload(
    payload: VariantPayload,
    seen: &mut BTreeSet<&'static str>,
    order: &mut Vec<&'static TypeContract>,
) {
    match payload {
        VariantPayload::Absent => {}
        VariantPayload::Keys(keys) => {
            for key in keys {
                visit_wire(&key.ty, seen, order);
            }
        }
        VariantPayload::Inlines(target) => visit_type(target, seen, order),
        VariantPayload::Shape(shape) => visit_wire(shape, seen, order),
    }
}

/// No wildcard arm, here or anywhere below: a new [`WireType`] must be a compile
/// error in every rendering rather than a silently unrendered edge.
fn visit_wire(
    ty: &'static WireType,
    seen: &mut BTreeSet<&'static str>,
    order: &mut Vec<&'static TypeContract>,
) {
    match *ty {
        WireType::Text
        | WireType::Integer
        | WireType::Boolean
        | WireType::Id(_)
        | WireType::Token(_) => {}
        WireType::Named(target) => visit_type(target, seen, order),
        WireType::Seq(inner) => visit_wire(inner, seen, order),
        WireType::Map { key, value } => {
            match key {
                // An extern map key is a leaf: its vocabulary arrives through
                // [`ExternContracts`], never through a closure edge (`sec-3`).
                MapKey::Of(inner) => visit_wire(inner, seen, order),
                MapKey::Extern { .. } => {}
            }
            visit_wire(value, seen, order);
        }
    }
}

// --- The model's own vocabulary, spelled once each (STD-001) -----------------

/// How a key's presence is spelled. **Three states, not two** — `sparse` is not
/// `optional` (`sec-2`).
const fn presence_token(presence: Presence) -> &'static str {
    match presence {
        Presence::Required => "required",
        Presence::Optional => "optional",
        Presence::Sparse => "sparse",
    }
}

/// How a struct's disclosure about unlisted keys is spelled.
const fn unknown_keys_token(unknown_keys: UnknownKeys) -> &'static str {
    match unknown_keys {
        UnknownKeys::Refused => "refused",
        UnknownKeys::SilentlyDropped => "silently-dropped",
    }
}

/// How a tagging style is spelled. The `Internal` tag key rides beside it rather
/// than in it, because the two renderings place it differently.
const fn tagging_token(tagging: Tagging) -> &'static str {
    match tagging {
        Tagging::Internal(_) => "internal",
        Tagging::External => "external",
        Tagging::Untagged => "untagged",
    }
}

// --- `--format json`: bespoke, and a flat table (sec-5) ----------------------

/// A JSON object from a fixed set of pairs — the shape every emission below is
/// built out of, so no site hand-rolls a `Map`.
fn object<const N: usize>(pairs: [(&str, Value); N]) -> Value {
    let mut map = Map::new();
    for (key, value) in pairs {
        map.insert(key.to_owned(), value);
    }
    Value::Object(map)
}

/// One wire type as a JSON **type expression**. A [`WireType::Named`] edge is
/// the target's *name*, which is the whole of why the table is flat.
fn json_wire(ty: &WireType) -> Value {
    match *ty {
        WireType::Text => Value::String(TEXT.to_owned()),
        WireType::Integer => Value::String(INTEGER.to_owned()),
        WireType::Boolean => Value::String(BOOLEAN.to_owned()),
        // The admissible prefixes, never a bare `id`: which kinds a key admits
        // is the omission this slice exists to remove (`WireType::Id`'s doc).
        WireType::Id(kinds) => object([(
            ID,
            Value::Array(
                kinds
                    .iter()
                    .map(|kind| Value::String(kind.prefix().to_owned()))
                    .collect(),
            ),
        )]),
        WireType::Named(target) => Value::String(target.name.to_owned()),
        WireType::Token(TokenSource::Fixed(tokens)) => object([(
            "token",
            object([(
                "fixed",
                Value::Array(
                    tokens
                        .iter()
                        .map(|token| Value::String((*token).to_owned()))
                        .collect(),
                ),
            )]),
        )]),
        WireType::Token(TokenSource::Extern(region)) => object([(
            "token",
            object([("extern", Value::String(region.label().to_owned()))]),
        )]),
        WireType::Seq(inner) => object([("seq", json_wire(inner))]),
        WireType::Map { key, value } => object([(
            "map",
            object([("key", json_map_key(key)), ("value", json_wire(value))]),
        )]),
    }
}

/// A map's key description. The extern arm **must** carry `region` and
/// `selector`: `facet`'s admissible keys are chosen by the value of the sibling
/// `kind`, and a rendering that dropped the selector would leave a caller a
/// promise it cannot act on (`sec-8` pin 9).
fn json_map_key(key: MapKey) -> Value {
    match key {
        MapKey::Of(inner) => json_wire(inner),
        MapKey::Extern { region, selector } => object([
            ("region", Value::String(region.label().to_owned())),
            ("selector", Value::String(selector.to_owned())),
        ]),
    }
}

fn json_key(row: &KeyContract) -> Value {
    object([
        ("key", Value::String(row.key.to_owned())),
        ("type", json_wire(&row.ty)),
        (
            "presence",
            Value::String(presence_token(row.presence).to_owned()),
        ),
    ])
}

/// Tagging carried **as tagging style** rather than encoded structurally — one
/// of the three things `sec-5` spent the JSON's budget on.
fn json_tagging(tagging: Tagging) -> Value {
    match tagging {
        Tagging::Internal(tag) => object([(tagging_token(tagging), Value::String(tag.to_owned()))]),
        Tagging::External | Tagging::Untagged => Value::String(tagging_token(tagging).to_owned()),
    }
}

/// Where a variant's payload sits, **derived from the tagging and the payload
/// together** (`sec-2`'s table). Every pair is named: a new [`Tagging`] or
/// [`VariantPayload`] arm is a compile error here rather than a wrong answer.
fn json_payload(tagging: Tagging, payload: VariantPayload) -> Option<(&'static str, Value)> {
    match (tagging, payload) {
        // The row that bit: externally tagged with no payload is the bare JSON
        // string, not `{"drafting-ready":{}}` (`sec-2`, `fnd-12`).
        (Tagging::External, VariantPayload::Absent) => Some(("bare-string", Value::Bool(true))),
        // Internally tagged with no payload is the tag alone; untagged with no
        // payload carries nothing to describe.
        (Tagging::Internal(_) | Tagging::Untagged, VariantPayload::Absent) => None,
        (
            Tagging::Internal(_) | Tagging::External | Tagging::Untagged,
            VariantPayload::Keys(rows),
        ) => Some(("payload", Value::Array(rows.iter().map(json_key).collect()))),
        (
            Tagging::Internal(_) | Tagging::External | Tagging::Untagged,
            VariantPayload::Inlines(target),
        ) => Some(("inlines", Value::String(target.name.to_owned()))),
        (
            Tagging::Internal(_) | Tagging::External | Tagging::Untagged,
            VariantPayload::Shape(shape),
        ) => Some(("shape", json_wire(shape))),
    }
}

fn json_variant(tagging: Tagging, variant: &VariantContract) -> Value {
    let mut map = Map::new();
    if let Some(token) = variant.token {
        map.insert("token".to_owned(), Value::String(token.to_owned()));
    }
    if let Some((key, value)) = json_payload(tagging, variant.payload) {
        map.insert(key.to_owned(), value);
    }
    Value::Object(map)
}

fn json_type(contract: &TypeContract) -> Value {
    match contract.form {
        TypeForm::Struct { unknown_keys, keys } => object([
            (
                UNKNOWN_KEYS,
                Value::String(unknown_keys_token(unknown_keys).to_owned()),
            ),
            ("struct", Value::Array(keys.iter().map(json_key).collect())),
        ]),
        TypeForm::Enum { tagging, variants } => object([
            ("tagging", json_tagging(tagging)),
            (
                "enum",
                Value::Array(
                    variants
                        .iter()
                        .map(|variant| json_variant(tagging, variant))
                        .collect(),
                ),
            ),
        ]),
    }
}

/// One externally-supplied region: the tokens the selecting key admits, and the
/// keys each token opens.
///
/// It rides the document as its own member rather than being inlined at both
/// sites that reference the region (`CreateRecord.kind`'s token source and
/// `facet`'s map key), which would state the same table twice (STD-001).
fn json_region(table: &SelectorTable) -> Value {
    object([
        ("source", Value::String(table.source.to_owned())),
        (
            UNKNOWN_KEYS,
            Value::String(unknown_keys_token(table.unknown_keys).to_owned()),
        ),
        (
            "selects",
            Value::Array(
                table
                    .rows
                    .iter()
                    .map(|row| {
                        object([
                            ("token", Value::String(row.token.to_owned())),
                            (
                                "keys",
                                Value::Array(row.keys.iter().map(json_key).collect()),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

/// The contract as one bespoke JSON document — `{schema, version, root, types}`,
/// plus the externally supplied regions.
///
/// **Bespoke, and not the derived `Serialize` of [`PAYLOAD`]**: `Named` holds
/// the whole target, so a derived rendering inlines `AcceptanceDeclaration` five
/// times and leaves a consumer unable to tell one type from two (`sec-5`,
/// `fnd-10`). Not JSON Schema either — it cannot say
/// [`UnknownKeys::SilentlyDropped`], [`Presence::Sparse`]'s omit-persists /
/// null-clears, or serde tagging style *as* tagging style, which are the three
/// things the contract's budget was spent on.
///
/// `serde_json::Map` is a `BTreeMap` here (no `preserve_order` feature), so the
/// object key order is deterministic and a golden can rest on it.
pub(crate) fn render_json(extern_contracts: &ExternContracts) -> String {
    let mut types = Map::new();
    for contract in closure_types(&PAYLOAD) {
        types.insert(contract.name.to_owned(), json_type(contract));
    }

    let mut regions = Map::new();
    for region in ExternRegion::ALL {
        regions.insert(
            region.label().to_owned(),
            json_region(extern_contracts.region(region)),
        );
    }

    let document = object([
        ("schema", Value::String(PAYLOAD_CONTRACT_SCHEMA.to_owned())),
        ("version", Value::from(PAYLOAD_CONTRACT_VERSION)),
        ("root", Value::String(PAYLOAD.name.to_owned())),
        ("types", Value::Object(types)),
        ("extern", Value::Object(regions)),
    ]);

    // No `expect`: a document built from `Map` and `Value` has no unserialisable
    // inhabitant, and clippy denies both `unwrap` and `expect` in production.
    serde_json::to_string_pretty(&document).unwrap_or_default()
}

// --- `--format prompt`: `contract_block`'s line shape (sec-5) ---------------

/// The marker for an externally tagged unit variant: it is the JSON string
/// itself, never an object. Spelled once — a caller who wraps it has the object
/// discarded in silence, which is one of the two failures that cost this
/// slice's own design run a round trip (`sec-1`, `fnd-12`).
const BARE_STRING: &str = "BARE STRING";

/// What the token column holds where the wire carries no token at all. An
/// untagged variant is a *shape*, and printing its Rust variant name would say
/// something false (`sec-2`).
const NO_TOKEN: &str = "‹no token›";

/// What a selector token that opens no keys renders as. An empty key list emits
/// no rows at all, and no rows is indistinguishable from a row that failed to
/// render — so the emptiness is stated.
const NO_KEYS: &str = "‹opens no keys›";

/// The column gap. Two spaces everywhere, so the reading is one shape.
const GAP: &str = "  ";

/// The widest a type column pads to.
///
/// A closed token list runs to a hundred characters, and padding every sibling
/// row out to meet it costs a block more than the alignment buys. A wider type
/// overflows its own column instead of widening the block — which keeps the
/// vocabulary on one greppable line, the reading `sec-5` prefers to wrapping it
/// mid-list.
const TYPE_COLUMN_CAP: usize = 40;

/// The parenthetical for a [`Presence`] — **one fixed string per variant**, and
/// never per key, so no two rows can word the same fact differently (`EX-9`).
const fn presence_note(presence: Presence) -> &'static str {
    match presence {
        // Required and Optional mean what they say and need no gloss.
        Presence::Required | Presence::Optional => "",
        Presence::Sparse => "(omit persists · null clears)",
    }
}

/// The parenthetical for an [`UnknownKeys`] — one fixed string per variant, on
/// the same rule.
///
/// **No id, ever.** An earlier draft rendered the root's as
/// `(ISS-333 — …)`, which would ship a doctrine-development issue id into every
/// installed client, where it is absent or is that client's own unrelated
/// record. The behaviour is rendered; the citation stays in the source comment
/// on [`UnknownKeys::SilentlyDropped`] (`sec-5`, `POL-002`).
const fn unknown_keys_note(unknown_keys: UnknownKeys) -> &'static str {
    match unknown_keys {
        UnknownKeys::Refused => "(a misspelt key is refused)",
        UnknownKeys::SilentlyDropped => "(a misspelt key is discarded, exit 0)",
    }
}

/// One wire type, as an agent reads it.
fn prompt_wire(ty: &WireType) -> String {
    match *ty {
        WireType::Text => TEXT.to_owned(),
        WireType::Integer => INTEGER.to_owned(),
        WireType::Boolean => BOOLEAN.to_owned(),
        // Which kinds the key admits, never a bare `id` — that is the same
        // omission this slice exists to remove ([`WireType::Id`]'s own doc).
        WireType::Id(kinds) => {
            let prefixes: Vec<&str> = kinds.iter().map(|kind| kind.prefix()).collect();
            format!("{ID}({})", prefixes.join("|"))
        }
        WireType::Named(target) => target.name.to_owned(),
        WireType::Token(TokenSource::Fixed(tokens)) => format!("one of: {}", tokens.join(" | ")),
        WireType::Token(TokenSource::Extern(region)) => region.label().to_owned(),
        WireType::Seq(inner) => format!("[{}]", prompt_wire(inner)),
        WireType::Map { key, value } => {
            format!("{{{}: {}}}", prompt_map_key(key), prompt_wire(value))
        }
    }
}

fn prompt_map_key(key: MapKey) -> String {
    match key {
        MapKey::Of(inner) => prompt_wire(inner),
        // The one place in the model where a key's contract depends on another
        // key's *value*, and the selector is the whole of what makes it
        // discoverable rather than an open bag.
        MapKey::Extern { region, selector } => {
            format!("{} chosen by {selector}", region.label())
        }
    }
}

/// One key row: name, type, presence, and the presence's fixed parenthetical.
fn key_line(row: &KeyContract, key_width: usize, type_width: usize) -> String {
    let line = format!(
        "{key:<key_width$}{GAP}{ty:<type_width$}{GAP}{presence}{GAP} {note}",
        key = row.key,
        ty = prompt_wire(&row.ty),
        presence = presence_token(row.presence),
        note = presence_note(row.presence),
    );
    line.trim_end().to_owned()
}

/// The widest key name and rendered type across a key list — column widths are
/// computed per block, so a wide row in one block does not pad every other, and
/// the type column stops at [`TYPE_COLUMN_CAP`].
fn key_widths<'a>(rows: impl Iterator<Item = &'a KeyContract>) -> (usize, usize) {
    let (key, ty) = rows.fold((0, 0), |(key, ty), row| {
        (
            key.max(row.key.chars().count()),
            ty.max(prompt_wire(&row.ty).chars().count()),
        )
    });
    (key, ty.min(TYPE_COLUMN_CAP))
}

/// A struct block: the header, then one line per key in declaration order —
/// field order is the commitment, as it is for `contract_block`'s rows.
///
/// `lead` is `payload` for the root and `type` for the rest, which is the only
/// difference between the two (`sec-5`).
fn struct_block(
    lead: &str,
    contract: &TypeContract,
    unknown_keys: UnknownKeys,
    keys: &[KeyContract],
) -> Vec<String> {
    let mut lines = vec![format!(
        "{lead} {name}{GAP}{UNKNOWN_KEYS}: {token}{GAP} {note}",
        name = contract.name,
        token = unknown_keys_token(unknown_keys),
        note = unknown_keys_note(unknown_keys),
    )];
    let (key_width, type_width) = key_widths(keys.iter());
    lines.extend(
        keys.iter()
            .map(|row| format!("{GAP}{}", key_line(row, key_width, type_width))),
    );
    lines
}

/// Whether this enum is **bare**: externally tagged with every payload
/// [`VariantPayload::Absent`], so every variant is a plain string on the wire.
///
/// Derived here and stored nowhere. Holding a `Tagging::Bare` beside the
/// variants would give that claim something to contradict, so `bare` survives as
/// a word the renderer prints (`sec-2`, `EX-5`).
fn renders_bare(tagging: Tagging, variants: &[VariantContract]) -> bool {
    let every_payload_absent = variants.iter().all(|variant| match variant.payload {
        VariantPayload::Absent => true,
        VariantPayload::Keys(_) | VariantPayload::Inlines(_) | VariantPayload::Shape(_) => false,
    });
    match tagging {
        Tagging::External => every_payload_absent,
        Tagging::Internal(_) | Tagging::Untagged => false,
    }
}

/// The enum header's tagging clause, and what that tagging *means* for where a
/// payload lands.
fn tagging_clause(tagging: Tagging, bare: bool) -> String {
    let style = tagging_token(tagging);
    let claim = match tagging {
        Tagging::Internal(tag) => {
            format!("{style}({tag:?}){GAP} variant keys sit BESIDE {tag:?}")
        }
        // The header carries the claim for the whole type, so the rows below it
        // need not repeat it once per variant.
        Tagging::External if bare => "bare".to_owned(),
        Tagging::External => format!("{style}{GAP} payload nests UNDER the token"),
        Tagging::Untagged => format!("{style}{GAP} discriminated by JSON shape alone"),
    };
    format!("tagging: {claim}")
}

/// A variant's payload, rendered **where the tagging and the payload together
/// put it** — never where the tagging alone would suggest (`sec-2`'s table).
///
/// Every pair is named. A new [`Tagging`] or [`VariantPayload`] arm is a compile
/// error here rather than a plausible, wrong line.
fn variant_payload_lines(
    tagging: Tagging,
    payload: VariantPayload,
    bare: bool,
    key_width: usize,
    type_width: usize,
) -> Vec<String> {
    let keys = |rows: &[KeyContract]| -> Vec<String> {
        rows.iter()
            .map(|row| key_line(row, key_width, type_width))
            .collect()
    };
    match (tagging, payload) {
        // The row that bit: external + no payload is the bare string itself,
        // marked per variant *because the asymmetry is per variant*. On an enum
        // whose every variant is bare the header has already said so, and
        // repeating it on each row would say nothing the reader does not have.
        (Tagging::External, VariantPayload::Absent) if bare => Vec::new(),
        (Tagging::External, VariantPayload::Absent) => {
            vec![format!("— a {BARE_STRING}, not an object")]
        }
        // Internally tagged with no payload is the tag alone; an untagged unit
        // variant is JSON `null`.
        (Tagging::Internal(_), VariantPayload::Absent) => Vec::new(),
        (Tagging::Untagged, VariantPayload::Absent) => vec!["null".to_owned()],
        (Tagging::Internal(_) | Tagging::Untagged, VariantPayload::Keys(rows)) => keys(rows),
        (Tagging::External, VariantPayload::Keys(rows)) => nested(keys(rows)),
        (Tagging::Internal(_) | Tagging::Untagged, VariantPayload::Inlines(target)) => {
            vec![inlined(target)]
        }
        (Tagging::External, VariantPayload::Inlines(target)) => nested(vec![inlined(target)]),
        (
            Tagging::Internal(_) | Tagging::External | Tagging::Untagged,
            VariantPayload::Shape(shape),
        ) => {
            vec![prompt_wire(shape)]
        }
    }
}

/// The name of the type a variant inlines — never a re-listing of its keys,
/// which is what lets a rendering say *which type arrived* (`fnd-14`).
fn inlined(target: &TypeContract) -> String {
    format!("→ {}'s keys, inlined", target.name)
}

/// Braces around a payload that arrives nested under its token.
fn nested(rows: Vec<String>) -> Vec<String> {
    let last = rows.len().saturating_sub(1);
    rows.into_iter()
        .enumerate()
        .map(|(at, row)| {
            let open = if at == 0 { "{ " } else { "  " };
            let close = if at == last { " }" } else { "" };
            format!("{open}{row}{close}")
        })
        .collect()
}

/// An enum block: the header, then one line per variant, with a multi-key
/// payload indented under its variant line rather than wrapped (`fnd-17`).
fn enum_block(
    contract: &TypeContract,
    tagging: Tagging,
    variants: &[VariantContract],
) -> Vec<String> {
    let bare = renders_bare(tagging, variants);
    let mut lines = vec![format!(
        "enum {name}{GAP}{clause}",
        name = contract.name,
        clause = tagging_clause(tagging, bare),
    )];

    let token_width = variants
        .iter()
        .map(|variant| variant.token.unwrap_or(NO_TOKEN).chars().count())
        .max()
        .unwrap_or_default();
    let (key_width, type_width) =
        key_widths(variants.iter().flat_map(|variant| match variant.payload {
            VariantPayload::Keys(rows) => rows.iter(),
            VariantPayload::Absent | VariantPayload::Inlines(_) | VariantPayload::Shape(_) => {
                [].iter()
            }
        }));

    for variant in variants {
        let token = variant.token.unwrap_or(NO_TOKEN);
        let payload = variant_payload_lines(tagging, variant.payload, bare, key_width, type_width);
        if payload.is_empty() {
            lines.push(format!("{GAP}{token}"));
            continue;
        }
        for (at, row) in payload.into_iter().enumerate() {
            let column = if at == 0 { token } else { "" };
            lines.push(
                format!("{GAP}{column:<token_width$}{GAP}{row}")
                    .trim_end()
                    .to_owned(),
            );
        }
    }
    lines
}

/// What an extern block's rows are — one fixed sentence for every region, on
/// the same rule as the parentheticals: a per-region gloss is a per-region
/// wording, and there is nothing region-specific to say (`EX-9`'s discipline).
const REGION_INTRO: &str =
    "Each token below is one admissible value; the rows under it are the keys it opens.";

/// The block for one externally supplied region: each admissible token, and the
/// keys it opens.
///
/// This is the one part of the contract a caller cannot recover any other way,
/// and the only part that is wholly undocumented today (`sec-5`).
fn region_block(table: &SelectorTable) -> Vec<String> {
    let mut lines = vec![
        format!(
            "extern {source}{GAP}{UNKNOWN_KEYS}: {token}{GAP} {note}",
            source = table.source,
            token = unknown_keys_token(table.unknown_keys),
            note = unknown_keys_note(table.unknown_keys),
        ),
        format!("{GAP}{REGION_INTRO}"),
    ];

    let token_width = table
        .rows
        .iter()
        .map(|row| row.token.chars().count())
        .max()
        .unwrap_or_default();
    let (key_width, type_width) = key_widths(table.rows.iter().flat_map(|row| row.keys.iter()));

    for row in &table.rows {
        if row.keys.is_empty() {
            lines.push(format!(
                "{GAP}{token:<token_width$}{GAP}{NO_KEYS}",
                token = row.token
            ));
            continue;
        }
        for (at, key) in row.keys.iter().enumerate() {
            let column = if at == 0 { row.token } else { "" };
            lines.push(
                format!(
                    "{GAP}{column:<token_width$}{GAP}{}",
                    key_line(key, key_width, type_width)
                )
                .trim_end()
                .to_owned(),
            );
        }
    }
    lines
}

/// The contract as the lines an agent reads — one block per described type,
/// then one per externally supplied region.
///
/// Blocks come in reading order rather than walk order: the root first, then the
/// remaining structs, then the enums, then the regions. The *contents* of each
/// are the closure's own declaration order, because field order is the
/// commitment (`sec-5`).
pub(crate) fn render_prompt(extern_contracts: &ExternContracts) -> Vec<String> {
    // The root's block leads, wherever the walk met it — asked for by name
    // rather than assumed from the walk's first entry.
    let mut root = Vec::new();
    let mut structs = Vec::new();
    let mut enums = Vec::new();

    for contract in closure_types(&PAYLOAD) {
        match contract.form {
            TypeForm::Struct { unknown_keys, keys } if contract.name == PAYLOAD.name => {
                root = struct_block(ROOT_LEAD, contract, unknown_keys, keys);
            }
            TypeForm::Struct { unknown_keys, keys } => {
                structs.push(struct_block(TYPE_LEAD, contract, unknown_keys, keys));
            }
            TypeForm::Enum { tagging, variants } => {
                enums.push(enum_block(contract, tagging, variants));
            }
        }
    }

    let mut lines = Vec::new();
    for block in std::iter::once(root).chain(structs).chain(enums).chain(
        ExternRegion::ALL
            .into_iter()
            .map(|region| region_block(extern_contracts.region(region))),
    ) {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.extend(block);
    }
    lines
}

// --- The published document (sec-5) -----------------------------------------

/// The document's provenance banner and intro.
///
/// It states **provenance, not an edit procedure**, on `ARTIFACT_HEADER`'s
/// pattern — change-the-table-and-re-render wording is correct in this
/// repository and useless in a client project, where the file is a read-only
/// artefact of an installed binary. And it cites **no id**: the constraint the
/// rendering is held to applies to the banner first.
const DOCUMENT_HEADER: &str = "\
<!-- GENERATED — rendered from the wire-payload contract your installed
     `doctrine` binary parses with, and pinned to it by test. Not hand-editable,
     and not overridable: an edited copy would describe a payload the binary
     does not accept. -->

# Design run — the apply payload contract

Every key a design-run submission may carry, what may be sent under it, whether
it may be omitted and what omission means, and what happens to a key this
contract does not list. Where a variant's payload sits is a function of the
enum's tagging and that variant's payload together, so the rendering states it
per variant rather than per type.
";

/// The fence the rendering sits in, opened and closed.
const FENCE_OPEN: &str = "\n```text\n";
const FENCE_CLOSE: &str = "```\n";

/// The contract as one Markdown page.
///
/// **Not a third emission of the rows**: a banner, an intro, and
/// [`render_prompt`]'s lines inside a fence. Two renderings that restate each
/// other is what `STD-001` forbids, and the rows have one producer.
pub(crate) fn render_document(extern_contracts: &ExternContracts) -> String {
    let mut out = String::from(DOCUMENT_HEADER);
    out.push_str(FENCE_OPEN);
    for line in render_prompt(extern_contracts) {
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str(FENCE_CLOSE);
    out
}

// ---------------------------------------------------------------------------
// The test-time seam (sec-8 pins 2, 3 and 4)
// ---------------------------------------------------------------------------
//
// Everything from here to `mod tests` is `#[cfg(test)]` and ships in nothing.
// It sits at **module level** rather than inside `mod tests` because the §9.1
// suite ([`super::tests`]) is where pin 2's recursive descent and its coverage
// equality live, and a sibling module cannot see a private `mod tests`. Pin 4's
// per-variant samples are the only inputs that reach most `VariantContract`
// sites, so the coverage union is unbuildable without them (`PHASE-03/F-2`).
//
// **Under `cfg(test)` the crate's blanket `expect(dead_code)` is off**
// (`super`'s attribute is `cfg_attr(not(test), …)`), so anything here that no
// test consumes is a `warnings = "deny"` build error rather than a quiet
// addition. That is the intended pressure.

/// Where one enum variant's token and its keys sit on the wire — `sec-2`'s
/// tagging × payload table, resolved against a value.
///
/// The table has **one** implementation ([`place`]) and three consumers: pin 4's
/// [`serde_token`], pin 2's variant selection, and pin 3's variant removal
/// probe. None of the three restates it (`PHASE-03/EX-2`, `EX-6`).
#[cfg(test)]
#[derive(Debug)]
pub(super) enum Placement<'a> {
    /// [`Tagging::External`] + [`VariantPayload::Absent`] — the JSON string is
    /// the token, and nothing rides with it.
    Bare(&'a str),
    /// [`Tagging::Internal`] — the token is the value at `tag`, and the
    /// payload's keys sit **beside** it in the same object.
    Beside {
        /// The token serde actually wrote.
        token: &'a str,
        /// The tag key, which is pin 4's and not a payload row.
        tag: &'static str,
        /// The whole object, tag included.
        object: &'a Map<String, Value>,
    },
    /// [`Tagging::External`] + a payload — the token is the sole object key and
    /// the payload is the value under it.
    Nested {
        /// The sole key.
        token: &'a str,
        /// What it wraps.
        value: &'a Value,
    },
    /// [`Tagging::Untagged`] — no token anywhere; the variant is a shape.
    Shape(&'a Value),
}

/// The key surface a variant's payload arrived on, wherever [`Placement`] put
/// it — a struct's own object, an internally tagged object minus its tag, or an
/// externally tagged wrapper's contents.
///
/// One reading for all three is what lets pin 2's key-set equality and pin 3's
/// removal probe be written once each rather than once per tagging.
#[cfg(test)]
pub(super) struct Fields<'a> {
    object: Option<&'a Map<String, Value>>,
    tag: Option<&'a str>,
}

#[cfg(test)]
impl<'a> Fields<'a> {
    /// A plain object with no tag in it — a struct target's own keys.
    pub(super) const fn plain(object: &'a Map<String, Value>) -> Fields<'a> {
        Fields {
            object: Some(object),
            tag: None,
        }
    }

    /// The payload keys, tag excluded.
    pub(super) fn names(&self) -> BTreeSet<&'a str> {
        self.object.map_or_else(BTreeSet::new, |object| {
            object
                .keys()
                .map(String::as_str)
                .filter(|key| Some(*key) != self.tag)
                .collect()
        })
    }

    /// The value under one payload key. The tag is not a row, so it reads as
    /// absent (`sec-8`, *The tag is not a row*).
    pub(super) fn get(&self, key: &str) -> Option<&'a Value> {
        if Some(key) == self.tag {
            return None;
        }
        self.object.and_then(|object| object.get(key))
    }
}

#[cfg(test)]
impl<'a> Placement<'a> {
    /// The token serde wrote, or `None` where the placement carries none.
    pub(super) fn token(&self) -> Option<&'a str> {
        match *self {
            Placement::Bare(token)
            | Placement::Beside { token, .. }
            | Placement::Nested { token, .. } => Some(token),
            Placement::Shape(_) => None,
        }
    }

    /// The payload's key surface, or `None` where there is none to read — an
    /// untagged shape is a value rather than a set of keys.
    pub(super) fn fields(&self) -> Option<Fields<'a>> {
        match *self {
            Placement::Bare(_) => Some(Fields {
                object: None,
                tag: None,
            }),
            Placement::Beside { tag, object, .. } => Some(Fields {
                object: Some(object),
                tag: Some(tag),
            }),
            Placement::Nested { value, .. } => Some(Fields {
                object: value.as_object(),
                tag: None,
            }),
            Placement::Shape(_) => None,
        }
    }

    /// The whole value again with one payload key removed, rewrapped exactly
    /// where it came from — pin 3's variant removal probe rebuilds its subject
    /// through this rather than restating where the keys sat.
    pub(super) fn without(&self, key: &str) -> Option<Value> {
        match *self {
            Placement::Bare(_) | Placement::Shape(_) => None,
            Placement::Beside { object, .. } => {
                let mut object = object.clone();
                object.remove(key);
                Some(Value::Object(object))
            }
            Placement::Nested { token, value } => {
                let mut inner = value.as_object()?.clone();
                inner.remove(key);
                let mut wrapper = Map::new();
                wrapper.insert(token.to_owned(), Value::Object(inner));
                Some(Value::Object(wrapper))
            }
        }
    }
}

/// `sec-2`'s tagging × payload table, written **once**.
///
/// A `Result` rather than a panic because pin 2's descent has to be able to try
/// a candidate variant and be told *no*: under a tagged enum the selection is
/// "the variant whose declared token is the one this placement extracted", and
/// an inapplicable candidate must fail rather than abort the walk.
///
/// The two `_` here are the table's own *any* rows (design `sec-8` pin 4's
/// extraction table: `Internal(tag)` × any, `Untagged` × any) and not a
/// wildcard over the model — pin 2's own matches, which are what `EX-1` binds,
/// name every arm.
#[cfg(test)]
pub(super) fn place<'a>(
    type_name: &str,
    tagging: Tagging,
    payload: VariantPayload,
    value: &'a Value,
) -> Result<Placement<'a>, String> {
    match (tagging, payload) {
        (Tagging::Internal(tag), _) => {
            let token = value
                .get(tag)
                .and_then(Value::as_str)
                .ok_or_else(|| format!("{type_name}: no string at the tag {tag:?} in {value}"))?;
            let object = value.as_object().ok_or_else(|| {
                format!("{type_name}: an internally tagged variant is an object, got {value}")
            })?;
            Ok(Placement::Beside { token, tag, object })
        }
        (Tagging::External, VariantPayload::Absent) => {
            value.as_str().map(Placement::Bare).ok_or_else(|| {
                format!(
                    "{type_name}: an external absent-payload variant is a bare string, got {value}"
                )
            })
        }
        (Tagging::External, VariantPayload::Keys(_) | VariantPayload::Inlines(_)) => {
            let object = value.as_object().ok_or_else(|| {
                format!(
                    "{type_name}: an external payload variant nests under its token, got {value}"
                )
            })?;
            if object.len() != 1 {
                return Err(format!(
                    "{type_name}: an externally tagged payload has exactly one key, got {value}"
                ));
            }
            let (token, inner) = object
                .iter()
                .next()
                .ok_or_else(|| format!("{type_name}: an object of length one has a key"))?;
            Ok(Placement::Nested {
                token,
                value: inner,
            })
        }
        (Tagging::External, VariantPayload::Shape(_)) => {
            Err(format!("{type_name}: {value} carries no token to read"))
        }
        (Tagging::Untagged, _) => Ok(Placement::Shape(value)),
    }
}

/// The rows a key list declares [`Presence::Required`]. Shared by pin 3's two
/// removal probes — the struct one in `super::tests` and the variant one below
/// — so requiredness has one reading (STD-001).
#[cfg(test)]
pub(super) fn required_keys(rows: &'static [KeyContract]) -> BTreeSet<&'static str> {
    rows.iter()
        .filter(|row| row.presence == Presence::Required)
        .map(|row| row.key)
        .collect()
}

/// The rows a **variant** declares required, read off the real
/// [`VariantContract`] — never off a sample's own `payload` field, which is a
/// placement claim and not a contract (`PHASE-03/EX-11`).
///
/// An exhaustive match with no wildcard arm, like every other walk over the
/// model in this slice.
#[cfg(test)]
fn required_variant_keys(payload: VariantPayload) -> BTreeSet<&'static str> {
    match payload {
        // A unit variant has no keys; an untagged shape has a value rather than
        // keys, so neither can require one.
        VariantPayload::Absent | VariantPayload::Shape(_) => BTreeSet::new(),
        VariantPayload::Keys(rows) => required_keys(rows),
        VariantPayload::Inlines(target) => match target.form {
            TypeForm::Struct { keys, .. } => required_keys(keys),
            TypeForm::Enum { .. } => {
                panic!("{}: an inlining variant names a struct", target.name)
            }
        },
    }
}

/// One variant of one closure enum, as the contract claims it.
///
/// Deliberately carries **no token literal**: the token is what the walk
/// derives from serde, and `VARIANTS` is what it is compared against. A
/// third spelling here would be the drift the macro exists to remove.
#[cfg(test)]
pub(super) struct VariantSample {
    /// Where this variant's payload sits, as the contract claims it. With the
    /// enum's [`Tagging`] this decides where the token sits — and it is read
    /// *per variant*, never per type: `AgentAct::DraftingReady` is the live
    /// mixed case and a per-type reading fails on a correct table.
    ///
    /// **A placement claim, not a contract.** It is fed to [`place`] and to
    /// nothing else: `CARRIES_KEYS` is an empty-row stand-in, so a walk that
    /// read this as a variant's rows would assert nothing and pass on any input
    /// (`PHASE-03/EX-11`). Rows come from [`EnumClaim::contract`].
    pub(super) payload: VariantPayload,
    /// A sample of this variant, serialised.
    pub(super) value: Value,
    /// This variant's token according to the type's **own** authority, where
    /// one exists. `PHASE-01/EX-9`: `Provenance` and `ReviewDisposition`
    /// cannot take the macro's `via` arm, so the single source is recovered
    /// here instead — the walk is total over variants by the barrier, so
    /// this cannot silently lose a case.
    pub(super) authority: Option<&'static str>,
    /// The read path back into the Rust type. A [`Value`] has lost its type, and
    /// pin 3's removal probe needs one to `from_value` into.
    pub(super) parse: fn(Value) -> Result<(), String>,
}

/// One closure enum's claim.
#[cfg(test)]
pub(super) struct EnumClaim {
    pub(super) type_name: &'static str,
    pub(super) variants: &'static [&'static str],
    pub(super) tagging: Tagging,
    /// The **real** contract for this enum — [`STAGE`], [`DISPOSE`], … Pin 2's
    /// descent and pin 3's variant probe read their rows from here
    /// (`PHASE-03/EX-11`).
    pub(super) contract: &'static TypeContract,
    pub(super) samples: Vec<VariantSample>,
}

#[cfg(test)]
fn json(value: &impl Serialize) -> Value {
    serde_json::to_value(value).expect("a closure value serialises")
}

/// The read path for one concrete closure type, as a plain function pointer.
#[cfg(test)]
fn parse_of<T: DeserializeOwned>() -> fn(Value) -> Result<(), String> {
    |value| {
        serde_json::from_value::<T>(value)
            .map(|_| ())
            .map_err(|refusal| refusal.to_string())
    }
}

#[cfg(test)]
fn sample<T: Serialize + DeserializeOwned>(payload: VariantPayload, value: &T) -> VariantSample {
    VariantSample {
        payload,
        value: json(value),
        authority: None,
        parse: parse_of::<T>(),
    }
}

/// A [`Provenance`] sample, pinned to `Provenance::label` (EX-9).
#[cfg(test)]
fn provenance(payload: VariantPayload, value: Provenance) -> VariantSample {
    VariantSample {
        payload,
        authority: Some(value.label()),
        value: json(&value),
        parse: parse_of::<Provenance>(),
    }
}

/// A [`ReviewDisposition`] sample, pinned to `ReviewDisposition::arm` (EX-9).
#[cfg(test)]
fn disposition(payload: VariantPayload, value: ReviewDisposition) -> VariantSample {
    VariantSample {
        payload,
        authority: Some(value.arm()),
        value: json(&value),
        parse: parse_of::<ReviewDisposition>(),
    }
}

/// A `Keys` payload. The rows are irrelevant to pin 4 — only *that* the
/// variant carries a payload is, since that is what decides where the token
/// sits. The rows themselves are `sec-8` pin 1's, in PHASE-02.
#[cfg(test)]
const CARRIES_KEYS: VariantPayload = VariantPayload::Keys(&[]);

/// Every closure enum, its declared vocabulary, its **real** contract, and one
/// sample per variant.
///
/// Named rather than inlined into the assertion because `PHASE-03`'s `VT-2`,
/// `VT-4` and the coverage union all consume the same samples.
#[cfg(test)]
pub(super) fn claims() -> Vec<EnumClaim> {
    vec![
        EnumClaim {
            type_name: Stage::TYPE_NAME,
            variants: Stage::VARIANTS,
            tagging: Tagging::External,
            contract: &STAGE,
            samples: vec![
                sample(VariantPayload::Absent, &Stage::Exploring),
                sample(VariantPayload::Absent, &Stage::Inquiring),
                sample(VariantPayload::Absent, &Stage::Drafting),
                sample(VariantPayload::Absent, &Stage::Reviewing),
                sample(VariantPayload::Absent, &Stage::Locked),
            ],
        },
        EnumClaim {
            type_name: ActKind::TYPE_NAME,
            variants: ActKind::VARIANTS,
            tagging: Tagging::External,
            contract: &ACT_KIND,
            samples: ActKind::ALL
                .iter()
                .map(|act| sample(VariantPayload::Absent, act))
                .collect(),
        },
        EnumClaim {
            type_name: ReviewPolicy::TYPE_NAME,
            variants: ReviewPolicy::VARIANTS,
            tagging: Tagging::External,
            contract: &REVIEW_POLICY,
            samples: ReviewPolicy::ALL
                .iter()
                .map(|policy| sample(VariantPayload::Absent, policy))
                .collect(),
        },
        EnumClaim {
            type_name: InquiryLifecycle::TYPE_NAME,
            variants: InquiryLifecycle::VARIANTS,
            tagging: Tagging::External,
            contract: &INQUIRY_LIFECYCLE,
            samples: vec![
                sample(VariantPayload::Absent, &InquiryLifecycle::Open),
                sample(VariantPayload::Absent, &InquiryLifecycle::Resolved),
                sample(VariantPayload::Absent, &InquiryLifecycle::Deferred),
                sample(VariantPayload::Absent, &InquiryLifecycle::Pruned),
            ],
        },
        EnumClaim {
            type_name: Provenance::TYPE_NAME,
            variants: Provenance::VARIANTS,
            tagging: Tagging::Internal("provenance"),
            contract: &PROVENANCE,
            samples: vec![
                provenance(VariantPayload::Absent, Provenance::UserDirected),
                provenance(VariantPayload::Absent, Provenance::AgentProposed),
                provenance(
                    CARRIES_KEYS,
                    Provenance::ShapingQuestion {
                        record: "QUE-001".to_owned(),
                    },
                ),
                provenance(
                    CARRIES_KEYS,
                    Provenance::ImportedProse {
                        section: id("sec-1"),
                        line: 12,
                        label: "OQ-1".to_owned(),
                        fingerprint: Fingerprint::new("sha256:seed"),
                    },
                ),
            ],
        },
        EnumClaim {
            type_name: ReviewDisposition::TYPE_NAME,
            variants: ReviewDisposition::VARIANTS,
            tagging: Tagging::External,
            contract: &REVIEW_DISPOSITION,
            samples: vec![
                disposition(
                    CARRIES_KEYS,
                    ReviewDisposition::Conducted {
                        review: ReviewRef::new("RV-001"),
                    },
                ),
                disposition(
                    CARRIES_KEYS,
                    ReviewDisposition::Waived {
                        reason: "out of budget".to_owned(),
                    },
                ),
            ],
        },
        EnumClaim {
            type_name: DischargeClaim::TYPE_NAME,
            variants: DischargeClaim::VARIANTS,
            tagging: Tagging::External,
            contract: &DISCHARGE_CLAIM,
            samples: vec![
                sample(VariantPayload::Absent, &DischargeClaim::Attested),
                sample(VariantPayload::Absent, &DischargeClaim::Skipped),
            ],
        },
        EnumClaim {
            type_name: DelegationAct::TYPE_NAME,
            variants: DelegationAct::VARIANTS,
            tagging: Tagging::Internal("act"),
            contract: &DELEGATION_ACT,
            samples: vec![
                sample(
                    CARRIES_KEYS,
                    &DelegationAct::Export {
                        id: id("dlg-1"),
                        obligation: id("inq-1"),
                    },
                ),
                sample(
                    CARRIES_KEYS,
                    &DelegationAct::Propose {
                        id: id("dlg-1"),
                        by: "a delegate".to_owned(),
                        summary: "what it concluded".to_owned(),
                        declare: Vec::new(),
                    },
                ),
                sample(CARRIES_KEYS, &DelegationAct::Accept { id: id("dlg-1") }),
                sample(
                    CARRIES_KEYS,
                    &DelegationAct::Refuse {
                        id: id("dlg-1"),
                        reason: "not the obligation cut".to_owned(),
                    },
                ),
            ],
        },
        EnumClaim {
            type_name: AgentAct::TYPE_NAME,
            variants: AgentAct::VARIANTS,
            tagging: Tagging::External,
            contract: &AGENT_ACT,
            samples: vec![
                sample(
                    CARRIES_KEYS,
                    &AgentAct::BlockingSetDeclared {
                        blocking: BTreeSet::from([id("inq-1")]),
                    },
                ),
                // The mixed case: externally tagged, and a *bare string*.
                sample(VariantPayload::Absent, &AgentAct::DraftingReady),
            ],
        },
        EnumClaim {
            type_name: Reviewer::TYPE_NAME,
            variants: Reviewer::VARIANTS,
            tagging: Tagging::External,
            contract: &REVIEWER,
            samples: vec![
                sample(VariantPayload::Absent, &Reviewer::Human),
                sample(VariantPayload::Absent, &Reviewer::Adversarial),
            ],
        },
        EnumClaim {
            type_name: Posture::TYPE_NAME,
            variants: Posture::VARIANTS,
            tagging: Tagging::External,
            contract: &POSTURE,
            samples: vec![
                sample(VariantPayload::Absent, &Posture::Breadth),
                sample(VariantPayload::Absent, &Posture::Depth),
            ],
        },
        EnumClaim {
            type_name: Authority::TYPE_NAME,
            variants: Authority::VARIANTS,
            tagging: Tagging::External,
            contract: &AUTHORITY,
            samples: vec![
                sample(VariantPayload::Absent, &Authority::AgentProposed),
                sample(VariantPayload::Absent, &Authority::UserPinned),
                sample(VariantPayload::Absent, &Authority::UserLocked),
            ],
        },
        EnumClaim {
            type_name: Dispose::TYPE_NAME,
            variants: Dispose::VARIANTS,
            tagging: Tagging::Internal("form"),
            contract: &DISPOSE,
            samples: vec![
                // `&CREATE_RECORD` and the real fixture, not the exemplar
                // contract and a hand-thinned record: the exemplar reference
                // was gratuitous (its placement class is identical) and both
                // were traps a walk could read as a declaration
                // (`PHASE-03/EX-11`, STD-001).
                sample(
                    VariantPayload::Inlines(&CREATE_RECORD),
                    &Dispose::Create(CreateRecord::fully_populated()),
                ),
                sample(
                    CARRIES_KEYS,
                    &Dispose::Adopt {
                        record: "DEC-001".to_owned(),
                    },
                ),
                sample(
                    CARRIES_KEYS,
                    &Dispose::Unresolved {
                        note: "still open".to_owned(),
                    },
                ),
                sample(
                    CARRIES_KEYS,
                    &Dispose::NonDurable {
                        note: "discussed only".to_owned(),
                    },
                ),
            ],
        },
        // Untagged: no token reaches the wire, so it contributes nothing to
        // pin 4. Its two shapes are pin 2's, in PHASE-03.
        EnumClaim {
            type_name: WireFacetValue::TYPE_NAME,
            variants: WireFacetValue::VARIANTS,
            tagging: Tagging::Untagged,
            contract: &WIRE_FACET_VALUE,
            samples: Vec::new(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // VT-1 — the tokens the contract declares are the tokens serde writes
    // (sec-8 pin 4), and EX-9's authority pin rides the same walk.
    // -----------------------------------------------------------------------

    /// The token serde actually wrote, read through the contract's **own**
    /// per-variant claim (`sec-8` pin 4).
    ///
    /// A thin wrapper over [`place`], which is where the tagging × payload table
    /// now lives — one implementation, three consumers (`PHASE-03/D1`). The
    /// refusal messages are `place`'s and are unchanged.
    fn serde_token(type_name: &str, tagging: Tagging, sample: &VariantSample) -> String {
        let value = &sample.value;
        place(type_name, tagging, sample.payload, value)
            .unwrap_or_else(|fault| panic!("{fault}"))
            .token()
            .unwrap_or_else(|| panic!("{type_name}: {value} carries no token to read"))
            .to_owned()
    }

    /// `sec-8` pin 4 — every token the contract declares is the token serde
    /// writes, per variant, through the tagging the contract itself claims.
    ///
    /// The set comparison is sufficient *because* `payload_variants!` makes the
    /// walk total: a variant added to an enum and left out of its invocation is
    /// a build failure, so this test cannot silently lose a case.
    #[test]
    fn tokens_match_serde_renames() {
        for claim in claims() {
            if matches!(claim.tagging, Tagging::Untagged) {
                assert!(
                    claim.variants.is_empty(),
                    "{}: an untagged enum puts no token on the wire",
                    claim.type_name
                );
                assert!(
                    claim.samples.is_empty(),
                    "{}: an untagged enum contributes nothing to pin 4",
                    claim.type_name
                );
                continue;
            }

            let declared: BTreeSet<&str> = claim.variants.iter().copied().collect();
            assert_eq!(
                declared.len(),
                claim.variants.len(),
                "{}: two variants declare the same token",
                claim.type_name
            );
            assert_eq!(
                claim.samples.len(),
                claim.variants.len(),
                "{}: one sample per declared variant",
                claim.type_name
            );

            let serialised: BTreeSet<String> = claim
                .samples
                .iter()
                .map(|sample| serde_token(claim.type_name, claim.tagging, sample))
                .collect();
            let declared_owned: BTreeSet<String> =
                declared.iter().map(|token| (*token).to_owned()).collect();
            assert_eq!(
                serialised, declared_owned,
                "{}: the declared vocabulary is not serde's",
                claim.type_name
            );

            // EX-9 — where the type owns an authority the macro could not reach
            // from a `const`, the authority is pinned here instead.
            let attested: BTreeSet<&str> =
                claim.samples.iter().filter_map(|s| s.authority).collect();
            if !attested.is_empty() {
                assert_eq!(
                    attested, declared,
                    "{}: the named tokens are not the type's own authority",
                    claim.type_name
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // VT-4 — pin 3's variant arm (sec-8, "Where the variant payloads are
    // pinned"). Here rather than in `super::tests` because the probe needs a
    // *type* to deserialise into, and pin 4's samples are the only place one
    // sits beside each variant.
    // -----------------------------------------------------------------------

    /// `sec-8` pin 3, over the variant payloads — removing one key from a
    /// variant's sample refuses **exactly** the rows that variant declares
    /// [`Presence::Required`].
    ///
    /// **Requiredness only.** Key sets and wire types are pin 2's descent's, and
    /// restating them here is the duplication this slice exists to remove
    /// (`PHASE-03/EX-6`). Where the keys sit is [`place`]'s — the tag is skipped
    /// under [`Tagging::Internal`] and the wrapped object is the subject under
    /// [`Tagging::External`] — and the rows are read off the real
    /// [`VariantContract`], never off the sample's own [`VariantPayload`]
    /// (`PHASE-03/EX-11`).
    ///
    /// The presence half is not decoration: without it the equality could be
    /// satisfied by a sample that simply omits a required key, since a key that
    /// is not there cannot be removed.
    #[test]
    fn the_removal_probe_over_variant_payloads_refuses_exactly_the_required_rows() {
        for claim in claims() {
            let TypeForm::Enum { tagging, variants } = claim.contract.form else {
                panic!(
                    "{}: a closure enum is described by an enum form",
                    claim.type_name
                );
            };

            for sample in &claim.samples {
                let token = serde_token(claim.type_name, claim.tagging, sample);
                let variant = variants
                    .iter()
                    .find(|variant| variant.token == Some(token.as_str()))
                    .unwrap_or_else(|| {
                        panic!("{}: no variant declares {token:?}", claim.type_name)
                    });

                let placement = place(claim.contract.name, tagging, variant.payload, &sample.value)
                    .unwrap_or_else(|fault| panic!("{fault}"));
                let Some(fields) = placement.fields() else {
                    continue;
                };

                let required = required_variant_keys(variant.payload);
                for key in &required {
                    assert!(
                        fields.get(key).is_some(),
                        "{}::{token}: the sample omits the required row `{key}`",
                        claim.type_name
                    );
                }

                let mut refusing = BTreeSet::new();
                for key in fields.names() {
                    let Some(candidate) = placement.without(key) else {
                        continue;
                    };
                    if (sample.parse)(candidate).is_err() {
                        refusing.insert(key);
                    }
                }
                assert_eq!(
                    refusing, required,
                    "{}::{token}: the keys whose removal refuses are not the declared \
                     `Presence::Required` rows",
                    claim.type_name
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // The model's own requirements (sec-2, "What the model has to be able to
    // say") — an exemplar contract expressing all four shapes, walked with no
    // wildcard arm so a new model arm is a build failure here.
    // -----------------------------------------------------------------------

    /// Shape 4's second half, and the `MapKey::Extern` inhabitant.
    static EXEMPLAR_CREATE_RECORD: TypeContract = TypeContract {
        name: "CreateRecord",
        form: TypeForm::Struct {
            unknown_keys: UnknownKeys::SilentlyDropped,
            keys: &[
                KeyContract {
                    key: "kind",
                    ty: WireType::Token(TokenSource::Extern(ExternRegion::KnowledgeRecord)),
                    presence: Presence::Required,
                },
                KeyContract {
                    key: "facet",
                    ty: WireType::Map {
                        key: MapKey::Extern {
                            region: ExternRegion::KnowledgeRecord,
                            selector: "kind",
                        },
                        value: &WireType::Named(&EXEMPLAR_WIRE_FACET_VALUE),
                    },
                    presence: Presence::Optional,
                },
            ],
        },
    };

    /// Shape 1 — an untagged variant is a *shape*, with no token at all.
    static EXEMPLAR_WIRE_FACET_VALUE: TypeContract = TypeContract {
        name: "WireFacetValue",
        form: TypeForm::Enum {
            tagging: Tagging::Untagged,
            variants: &[
                VariantContract {
                    token: None,
                    payload: VariantPayload::Shape(&WireType::Seq(&WireType::Text)),
                },
                VariantContract {
                    token: None,
                    payload: VariantPayload::Shape(&WireType::Text),
                },
            ],
        },
    };

    /// Shape 2 — a variant that inlines another *named* type.
    static EXEMPLAR_DISPOSE: TypeContract = TypeContract {
        name: "Dispose",
        form: TypeForm::Enum {
            tagging: Tagging::Internal("form"),
            variants: &[
                VariantContract {
                    token: Some("create"),
                    payload: VariantPayload::Inlines(&EXEMPLAR_CREATE_RECORD),
                },
                VariantContract {
                    token: Some("adopt"),
                    payload: VariantPayload::Keys(&[KeyContract {
                        key: "record",
                        ty: WireType::Text,
                        presence: Presence::Required,
                    }]),
                },
            ],
        },
    };

    /// Shape 3 — an externally tagged enum whose two variants *disagree*: one
    /// nests under its token, the other is a bare string.
    static EXEMPLAR_AGENT_ACT: TypeContract = TypeContract {
        name: "AgentAct",
        form: TypeForm::Enum {
            tagging: Tagging::External,
            variants: &[
                VariantContract {
                    token: Some("blocking-set-declared"),
                    payload: VariantPayload::Keys(&[KeyContract {
                        key: "blocking",
                        ty: WireType::Seq(&WireType::Id(&[IdKind::Inquiry])),
                        presence: Presence::Required,
                    }]),
                },
                VariantContract {
                    token: Some("drafting-ready"),
                    payload: VariantPayload::Absent,
                },
            ],
        },
    };

    /// The exemplar root, reaching every arm of the model.
    static EXEMPLAR_ROOT: TypeContract = TypeContract {
        name: "ApplyRequest",
        form: TypeForm::Struct {
            unknown_keys: UnknownKeys::Refused,
            keys: &[
                KeyContract {
                    key: "run_uid",
                    ty: WireType::Text,
                    presence: Presence::Required,
                },
                KeyContract {
                    key: "known_revision",
                    ty: WireType::Integer,
                    presence: Presence::Required,
                },
                KeyContract {
                    key: "locked",
                    ty: WireType::Boolean,
                    presence: Presence::Optional,
                },
                KeyContract {
                    key: "parent",
                    ty: WireType::Id(&[IdKind::Inquiry]),
                    presence: Presence::Sparse,
                },
                KeyContract {
                    key: "sections",
                    ty: WireType::Map {
                        key: MapKey::Of(&WireType::Id(&[IdKind::Section])),
                        value: &WireType::Text,
                    },
                    presence: Presence::Optional,
                },
                KeyContract {
                    key: "outcome",
                    ty: WireType::Token(TokenSource::Fixed(&["attested", "skipped"])),
                    presence: Presence::Optional,
                },
                KeyContract {
                    key: "dispose",
                    ty: WireType::Named(&EXEMPLAR_DISPOSE),
                    presence: Presence::Optional,
                },
                KeyContract {
                    key: "act",
                    ty: WireType::Named(&EXEMPLAR_AGENT_ACT),
                    presence: Presence::Optional,
                },
            ],
        },
    };

    /// Walk a contract, **reading every field of every node**, and record the
    /// shapes it used. No wildcard arm anywhere below: a new arm on any model
    /// type is a build failure here, which is what makes the census a claim
    /// about the model rather than about this exemplar.
    fn census_type(contract: &'static TypeContract, seen: &mut BTreeSet<String>) {
        seen.insert(format!("type:{}", contract.name));
        match contract.form {
            TypeForm::Struct { unknown_keys, keys } => {
                seen.insert(format!("unknown-keys:{unknown_keys:?}"));
                for key in keys {
                    census_key(key, seen);
                }
            }
            TypeForm::Enum { tagging, variants } => {
                seen.insert(match tagging {
                    Tagging::Internal(tag) => format!("tagging:internal:{tag}"),
                    Tagging::External => "tagging:external".to_owned(),
                    Tagging::Untagged => "tagging:untagged".to_owned(),
                });
                for variant in variants {
                    seen.insert(match variant.token {
                        Some(token) => format!("token:{token}"),
                        None => "token:none".to_owned(),
                    });
                    census_payload(variant.payload, seen);
                }
            }
        }
    }

    fn census_payload(payload: VariantPayload, seen: &mut BTreeSet<String>) {
        match payload {
            VariantPayload::Absent => {
                seen.insert("payload:absent".to_owned());
            }
            VariantPayload::Keys(keys) => {
                seen.insert("payload:keys".to_owned());
                for key in keys {
                    census_key(key, seen);
                }
            }
            VariantPayload::Inlines(target) => {
                seen.insert(format!("payload:inlines:{}", target.name));
                census_type(target, seen);
            }
            VariantPayload::Shape(shape) => {
                seen.insert("payload:shape".to_owned());
                census_wire(shape, seen);
            }
        }
    }

    fn census_key(key: &'static KeyContract, seen: &mut BTreeSet<String>) {
        seen.insert(format!("key:{}", key.key));
        seen.insert(format!("presence:{:?}", key.presence));
        census_wire(&key.ty, seen);
    }

    fn census_wire(ty: &'static WireType, seen: &mut BTreeSet<String>) {
        match *ty {
            WireType::Text => {
                seen.insert("wire:text".to_owned());
            }
            WireType::Integer => {
                seen.insert("wire:integer".to_owned());
            }
            WireType::Boolean => {
                seen.insert("wire:boolean".to_owned());
            }
            WireType::Id(kinds) => {
                for kind in kinds {
                    seen.insert(format!("wire:id:{}", kind.prefix()));
                }
            }
            WireType::Named(target) => {
                seen.insert(format!("wire:named:{}", target.name));
                census_type(target, seen);
            }
            WireType::Token(TokenSource::Fixed(tokens)) => {
                seen.insert(format!("wire:token:fixed:{}", tokens.join(",")));
            }
            WireType::Token(TokenSource::Extern(region)) => {
                seen.insert(format!("wire:token:extern:{}", region.label()));
            }
            WireType::Seq(inner) => {
                seen.insert("wire:seq".to_owned());
                census_wire(inner, seen);
            }
            WireType::Map { key, value } => {
                seen.insert("wire:map".to_owned());
                match key {
                    MapKey::Of(inner) => {
                        seen.insert("map-key:of".to_owned());
                        census_wire(inner, seen);
                    }
                    MapKey::Extern { region, selector } => {
                        seen.insert(format!("map-key:extern:{}:{selector}", region.label()));
                    }
                }
                census_wire(value, seen);
            }
        }
    }

    /// `sec-2` § *What the model has to be able to say* lists four shapes the
    /// closure forced on the model, each one a shape a caller gets wrong
    /// unaided. A spelling that cannot state all four is not a different
    /// spelling, it is a worse one — so this asserts it can state them.
    #[test]
    fn the_model_states_the_four_shapes_the_closure_forced() {
        let mut seen = BTreeSet::new();
        census_type(&EXEMPLAR_ROOT, &mut seen);

        // 1. An untagged variant — a shape, with no token at all.
        assert!(seen.contains("tagging:untagged"), "{seen:?}");
        assert!(seen.contains("token:none"), "{seen:?}");
        assert!(seen.contains("payload:shape"), "{seen:?}");

        // 2. A variant that inlines another *named* type, by name.
        assert!(seen.contains("payload:inlines:CreateRecord"), "{seen:?}");

        // 3. An externally tagged enum whose variants disagree — read off the
        //    one type, not off the whole census, since the claim is about a
        //    single enum carrying both readings.
        let TypeForm::Enum { tagging, variants } = EXEMPLAR_AGENT_ACT.form else {
            panic!("the mixed exemplar is an enum");
        };
        assert_eq!(tagging, Tagging::External);
        assert_eq!(
            variants
                .iter()
                .filter(|v| matches!(v.payload, VariantPayload::Absent))
                .count(),
            1,
            "one arm is a bare string"
        );
        assert_eq!(
            variants
                .iter()
                .filter(|v| matches!(v.payload, VariantPayload::Keys(_)))
                .count(),
            1,
            "the other nests under its token"
        );

        // 4. A map whose keys are not free — chosen by a *sibling* field's value.
        assert!(
            seen.contains("map-key:extern:knowledge::RecordKind:kind"),
            "{seen:?}"
        );
    }

    /// The model's own arms, each reached by the exemplar. This is what makes
    /// the census above a total statement rather than a sample: every arm of
    /// every model type is exercised, so a walk that compiles has walked all of
    /// them.
    #[test]
    fn the_exemplar_reaches_every_arm_of_the_model() {
        let mut seen = BTreeSet::new();
        census_type(&EXEMPLAR_ROOT, &mut seen);

        for expected in [
            // WireType, eight arms.
            "wire:text",
            "wire:integer",
            "wire:boolean",
            "wire:id:inq-",
            "wire:named:Dispose",
            "wire:token:fixed:attested,skipped",
            "wire:token:extern:knowledge::RecordKind",
            "wire:seq",
            "wire:map",
            // MapKey, both.
            "map-key:of",
            "map-key:extern:knowledge::RecordKind:kind",
            // Presence, all three.
            "presence:Required",
            "presence:Optional",
            "presence:Sparse",
            // UnknownKeys, both — and, being struct-only, `TypeForm::Struct`.
            "unknown-keys:Refused",
            "unknown-keys:SilentlyDropped",
            // Tagging, all three — and, being enum-only, `TypeForm::Enum`.
            "tagging:internal:form",
            "tagging:external",
            "tagging:untagged",
            // VariantPayload, all four.
            "payload:absent",
            "payload:keys",
            "payload:inlines:CreateRecord",
            "payload:shape",
        ] {
            assert!(
                seen.contains(expected),
                "the exemplar never reached {expected}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // The extern seam (sec-3)
    // -----------------------------------------------------------------------

    /// A leaf-local [`ExternContracts`], generalising the one-row fixture in
    /// `every_extern_region_resolves_to_its_own_supply` below (`PHASE-05/D3`).
    ///
    /// **Two rows, and the second has none of its own keys.** `concept`
    /// legitimately opens no facet fields in the real table
    /// (`knowledge.rs`'s `CONCEPT_FACET_FIELDS`), so the empty-key path runs in
    /// production on every invocation and a `for` loop that silently emits
    /// nothing is the failure it hides. Built here rather than imported from the
    /// command tier: this file names `crate::` nowhere, in production and in
    /// tests alike (ADR-001).
    fn extern_fixture() -> ExternContracts {
        ExternContracts {
            knowledge_record: SelectorTable {
                source: ExternRegion::KnowledgeRecord.label(),
                rows: vec![
                    SelectedKeys {
                        token: "assumption",
                        keys: vec![
                            KeyContract {
                                key: "claim",
                                ty: WireType::Text,
                                presence: Presence::Optional,
                            },
                            KeyContract {
                                key: "confidence",
                                ty: WireType::Token(TokenSource::Fixed(&["low", "medium", "high"])),
                                presence: Presence::Optional,
                            },
                        ],
                    },
                    // The zero-key row: a real shape, not a degenerate one.
                    SelectedKeys {
                        token: "concept",
                        keys: Vec::new(),
                    },
                ],
                unknown_keys: UnknownKeys::Refused,
            },
        }
    }

    /// Every string in `value` that sits in a **wire-type position** and is
    /// shaped like a type name — the rendered document's own edge relation,
    /// read back out of the JSON rather than out of [`PAYLOAD`].
    ///
    /// Reading it out of the output is the point: `VT-1` is a claim about what a
    /// consumer receives, and a walk over the table would pass over a renderer
    /// that emitted nothing at all.
    fn named_edges(value: &Value, into: &mut BTreeSet<String>) {
        match value {
            Value::Object(map) => {
                for (key, child) in map {
                    if matches!(key.as_str(), "type" | "seq" | "value" | "shape" | "inlines")
                        && let Some(name) = child.as_str()
                        && name.starts_with(|first: char| first.is_ascii_uppercase())
                    {
                        into.insert(name.to_owned());
                    }
                    named_edges(child, into);
                }
            }
            Value::Array(items) => {
                for item in items {
                    named_edges(item, into);
                }
            }
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
        }
    }

    /// The parsed `types` table of a rendered JSON document.
    fn types_of(document: &Value) -> Map<String, Value> {
        document
            .get("types")
            .and_then(Value::as_object)
            .expect("the rendered contract carries a `types` table")
            .clone()
    }

    /// `sec-8` pin 9 (`VT-1`, first half) — the JSON is a **flat** table, every
    /// `Named` edge lands in it as a name, and every entry is reachable from the
    /// root.
    ///
    /// The count and the multi-parent assertion are not decoration. "Every edge
    /// resolves" is vacuously true over an empty table and "no type is inlined"
    /// is vacuously true over a closure that never meets one type twice, so the
    /// verdict is asserted beside its evidence: twenty-five types, and
    /// `AcceptanceDeclaration` reached from more than one parent.
    #[test]
    fn the_json_is_a_flat_table_every_edge_lands_in() {
        let rendered = render_json(&extern_fixture());
        let document: Value = serde_json::from_str(&rendered).expect("the rendering is JSON");

        // 1. The envelope: schema, version, root.
        assert_eq!(
            document.get("schema").and_then(Value::as_str),
            Some(PAYLOAD_CONTRACT_SCHEMA)
        );
        assert_eq!(
            document.get("version").and_then(Value::as_u64),
            Some(PAYLOAD_CONTRACT_VERSION)
        );
        assert_eq!(
            document.get("root").and_then(Value::as_str),
            Some(PAYLOAD.name)
        );
        assert_eq!(PAYLOAD.name, "ApplyRequest");

        // 2. The evidence count — eleven struct contracts and fourteen enums.
        //    `SubmissionEnvelope` is in the closure and has no contract: its
        //    three keys are flattened into the root (`fnd-16`).
        let types = types_of(&document);
        assert_eq!(
            types.len(),
            25,
            "the closure is twenty-five described types"
        );

        // 3. Every name in a type position is a key of `types` — the half that
        //    catches an edge pointing at nothing.
        let mut edges = BTreeSet::new();
        for entry in types.values() {
            named_edges(entry, &mut edges);
        }
        let described: BTreeSet<String> = types.keys().cloned().collect();
        let dangling: BTreeSet<&String> = edges.difference(&described).collect();
        assert!(
            dangling.is_empty(),
            "these edges name a type the table does not describe: {dangling:?}"
        );

        // 4. Every key of `types` is reachable from `root` — the half that
        //    catches a type the walk dropped, or one it invented.
        let mut reached = BTreeSet::from([PAYLOAD.name.to_owned()]);
        let mut frontier = vec![PAYLOAD.name.to_owned()];
        while let Some(name) = frontier.pop() {
            let Some(entry) = types.get(&name) else {
                continue;
            };
            let mut out = BTreeSet::new();
            named_edges(entry, &mut out);
            for target in out {
                if reached.insert(target.clone()) {
                    frontier.push(target);
                }
            }
        }
        assert_eq!(
            reached, described,
            "the table and the closure reachable from the root disagree"
        );

        // 5. Nothing is inlined — and the fixture can tell: `AcceptanceDeclaration`
        //    is reached from more than one parent, so "emitted once" has
        //    something to be wrong about.
        let parents: Vec<&String> = types
            .iter()
            .filter(|(_, entry)| {
                let mut out = BTreeSet::new();
                named_edges(entry, &mut out);
                out.contains("AcceptanceDeclaration")
            })
            .map(|(name, _)| name)
            .collect();
        assert!(
            parents.len() > 1,
            "the closure must reach AcceptanceDeclaration more than once, got {parents:?}"
        );
        assert_eq!(
            types
                .keys()
                .filter(|name| name.as_str() == "AcceptanceDeclaration")
                .count(),
            1,
            "a type reached twice is still described once"
        );

        // 6. `EX-3`'s three expressibility claims, each read out of the output:
        //    a silently-dropped disclosure, a sparse presence, and tagging
        //    carried *as* tagging rather than encoded structurally.
        assert_eq!(
            types
                .get("ApplyRequest")
                .and_then(|entry| entry.get("unknown-keys"))
                .and_then(Value::as_str),
            Some("silently-dropped")
        );
        assert!(
            rendered.contains("\"presence\": \"sparse\""),
            "Sparse is a third presence, not a boolean required"
        );
        assert_eq!(
            types
                .get("Dispose")
                .and_then(|entry| entry.get("tagging"))
                .and_then(|tagging| tagging.get("internal"))
                .and_then(Value::as_str),
            Some("form")
        );
    }

    /// `sec-8` pin 9 (`VT-1`, second half) — `CreateRecord.facet`'s map key
    /// names `kind` as its selector, and `kind` is a sibling key **of the same
    /// type**.
    ///
    /// Both sides are read out of the *rendered* document rather than out of
    /// [`CREATE_RECORD`], because the pin is that the promise a consumer
    /// receives is actionable. `sec-8` records that no other pin reads this
    /// claim: a mis-typed selector is otherwise a promise nothing can act on.
    #[test]
    fn the_facet_maps_selector_names_a_sibling_key_of_the_same_type() {
        let document: Value =
            serde_json::from_str(&render_json(&extern_fixture())).expect("the rendering is JSON");
        let types = types_of(&document);
        let rows = types
            .get("CreateRecord")
            .and_then(|entry| entry.get("struct"))
            .and_then(Value::as_array)
            .expect("CreateRecord is described as a struct")
            .clone();

        let facet = rows
            .iter()
            .find(|row| row.get("key").and_then(Value::as_str) == Some("facet"))
            .expect("CreateRecord carries a facet row");
        let selector = facet
            .get("type")
            .and_then(|ty| ty.get("map"))
            .and_then(|map| map.get("key"))
            .and_then(|key| key.get("selector"))
            .and_then(Value::as_str)
            .expect("the facet map's key names its selector");
        assert_eq!(selector, "kind");

        assert_eq!(
            facet
                .get("type")
                .and_then(|ty| ty.get("map"))
                .and_then(|map| map.get("key"))
                .and_then(|key| key.get("region"))
                .and_then(Value::as_str),
            Some(ExternRegion::KnowledgeRecord.label()),
            "the map key names the region it is supplied from"
        );

        let siblings: BTreeSet<&str> = rows
            .iter()
            .filter_map(|row| row.get("key").and_then(Value::as_str))
            .collect();
        assert!(
            siblings.contains(selector),
            "the selector {selector:?} is a key of the same type, got {siblings:?}"
        );
    }

    /// One block of the prompt rendering: its header and the rows under it, up
    /// to the blank line that ends it.
    fn block(lines: &[String], header: &str) -> Vec<String> {
        lines
            .iter()
            .skip_while(|line| !line.starts_with(header))
            .take_while(|line| !line.is_empty())
            .cloned()
            .collect()
    }

    /// `EX-4` — the root's header line stays inside the 100 columns the rest of
    /// the corpus holds to. The measured sample's ran to 104, and `sec-5` says
    /// which way to fix it: shorten the parenthetical, not the disclosure.
    #[test]
    fn the_root_header_line_fits_the_corpus_width() {
        let lines = render_prompt(&extern_fixture());
        let header = lines.first().expect("the rendering opens with the root");
        assert!(
            header.starts_with("payload ApplyRequest"),
            "the root block comes first, got {header:?}"
        );
        assert!(
            header.contains(unknown_keys_token(UnknownKeys::SilentlyDropped)),
            "the root still discloses that a misspelt key is dropped: {header:?}"
        );
        assert!(
            header.chars().count() <= 100,
            "the root header runs to {} columns: {header:?}",
            header.chars().count()
        );
    }

    /// `EX-9` — a parenthetical is one fixed string per [`Presence`] /
    /// [`UnknownKeys`] variant, never per key, so two rows cannot word the same
    /// fact differently.
    ///
    /// Asserted as a *set size* over the whole render rather than by comparing
    /// against a literal: a per-key parenthetical would still contain the right
    /// words and would fail here for the right reason.
    #[test]
    fn each_presence_parenthetical_is_one_fixed_string() {
        let lines = render_prompt(&extern_fixture());

        // The *trailing* parenthetical: `id(inq-|sec-)` is a rendered type, not
        // a gloss, and a leading-paren reading would collect those instead.
        let parenthetical = |line: &String| -> Option<String> {
            let at = line.rfind('(')?;
            let tail = line.get(at..)?;
            tail.ends_with(')').then(|| tail.to_owned())
        };

        let sparse: BTreeSet<String> = lines
            .iter()
            .filter(|line| line.contains(presence_token(Presence::Sparse)))
            .filter_map(parenthetical)
            .collect();
        assert_eq!(
            sparse.len(),
            1,
            "every sparse row carries the same parenthetical, got {sparse:?}"
        );

        let disclosures: BTreeSet<String> = lines
            .iter()
            .filter(|line| line.contains("unknown-keys:"))
            .filter_map(parenthetical)
            .collect();
        assert_eq!(
            disclosures.len(),
            2,
            "one fixed string per UnknownKeys variant, got {disclosures:?}"
        );
    }

    /// `EX-8` / `VT-3` — the three semantic rules `sec-5` names, each asserted
    /// on the marker rather than on the column layout. Layout is `PHASE-06`'s
    /// golden; what is pinned here is what the rendering *says*.
    ///
    /// These are the two failures that cost this slice's own design run a round
    /// trip (`sec-1`): a payload rendered where the tagging does not put it, and
    /// `AgentAct::DraftingReady` — an externally tagged unit variant — rendered
    /// as an object rather than as the bare string it is.
    #[test]
    fn a_variants_payload_is_rendered_where_the_tagging_puts_it() {
        let lines = render_prompt(&extern_fixture());

        // 1. Internally tagged: the variant's keys sit BESIDE the tag, and an
        //    inlining variant names the type it inlines rather than re-listing
        //    its keys.
        let dispose = block(&lines, "enum Dispose ");
        let dispose_text = dispose.join("\n");
        assert!(
            dispose_text.contains(r#"tagging: internal("form")"#),
            "{dispose_text}"
        );
        assert!(dispose_text.contains("BESIDE"), "{dispose_text}");
        assert!(
            dispose_text.contains("→ CreateRecord's keys, inlined"),
            "{dispose_text}"
        );
        assert!(
            !dispose_text.contains("title"),
            "the inlined type is named, never re-listed: {dispose_text}"
        );

        // 2. Externally tagged, and *mixed*: one variant nests under its token,
        //    the other is the bare string `"drafting-ready"` — not
        //    `{"drafting-ready":{}}`, which is discarded in silence.
        let agent_act = block(&lines, "enum AgentAct ");
        let agent_text = agent_act.join("\n");
        assert!(agent_text.contains("tagging: external"), "{agent_text}");
        assert!(agent_text.contains("UNDER the token"), "{agent_text}");
        let nests = agent_act
            .iter()
            .find(|line| line.contains("blocking-set-declared"))
            .expect("the nesting variant renders");
        assert!(nests.contains('{') && nests.contains('}'), "{nests}");
        let bare = agent_act
            .iter()
            .find(|line| line.contains("drafting-ready"))
            .expect("the bare-string variant renders");
        assert!(bare.contains(BARE_STRING), "{bare}");
        assert!(
            !bare.contains('{'),
            "a bare string is not an object: {bare}"
        );

        // 3. Untagged: the token column is struck out, and no Rust variant name
        //    — which is not on the wire — appears anywhere in the rendering.
        let untagged = block(&lines, "enum WireFacetValue ");
        let untagged_text = untagged.join("\n");
        assert!(
            untagged_text.contains("tagging: untagged"),
            "{untagged_text}"
        );
        assert_eq!(
            untagged
                .iter()
                .filter(|line| line.contains(NO_TOKEN))
                .count(),
            2,
            "both shapes render with no token: {untagged_text}"
        );
        let whole = lines.join("\n");
        for rust_only in ["WireFacetValue::", "List", "Text"] {
            assert!(
                !whole.contains(rust_only),
                "{rust_only} is a Rust variant name and is not on the wire"
            );
        }

        // 4. `EX-5` — `bare` is a word this renderer prints, derived from every
        //    payload being Absent. `Stage` is stored as `External` and has no
        //    `Tagging::Bare` to read.
        let TypeForm::Enum { tagging, .. } = STAGE.form else {
            panic!("Stage is described as an enum");
        };
        assert_eq!(
            tagging,
            Tagging::External,
            "bare is derived from the model, never stored beside the variants"
        );
        let stage = block(&lines, "enum Stage ");
        let stage_text = stage.join("\n");
        assert!(stage_text.contains("tagging: bare"), "{stage_text}");
        assert!(!stage_text.contains("tagging: external"), "{stage_text}");
    }

    /// The zero-key row renders as a row that *says* it opens no keys, and the
    /// block survives it.
    ///
    /// `concept` legitimately opens no facet fields in the real table, so this
    /// path runs in production on every invocation: a `for` loop over an empty
    /// key list emits nothing, and nothing is indistinguishable from a row that
    /// failed to render (`PHASE-04/F-2`).
    #[test]
    fn a_token_that_opens_no_keys_renders_a_row_that_says_so() {
        let lines = render_prompt(&extern_fixture());
        let region = block(&lines, "extern ");
        let region_text = region.join("\n");

        let empty = region
            .iter()
            .find(|line| line.contains("concept"))
            .expect("the zero-key row renders at all");
        assert!(empty.contains(NO_KEYS), "{empty}");

        // The row before it is intact — an empty key list did not swallow the
        // rest of the block.
        assert!(region_text.contains("assumption"), "{region_text}");
        assert!(region_text.contains("confidence"), "{region_text}");
        assert!(
            region_text.contains("one of: low | medium | high"),
            "{region_text}"
        );
        assert!(
            region_text.contains(ExternRegion::KnowledgeRecord.label()),
            "the block names the region it supplies: {region_text}"
        );
    }

    /// `sec-8` pin 10 (`VT-2`) — **no rendered surface cites a repo-private
    /// id.**
    ///
    /// The detector is `artifact.rs`'s, reached at module level rather than
    /// reimplemented here: one predicate and one fourteen-prefix list, so a
    /// second copy would be a description free to drift (STD-001). This is a
    /// sibling-module path, not a `crate::` one — `payload_contract` names
    /// `crate::` nowhere, in tests as in production (ADR-001).
    ///
    /// **The positive control comes first, and the non-vacuity assertion with
    /// it.** A probe whose passing observation is an absence cannot tell "not
    /// there" from "I never looked": `!cites_a_repo_private_id(&document)`
    /// passes just as happily over the empty string.
    #[test]
    fn no_rendered_surface_cites_a_repo_private_id() {
        use super::super::artifact::cites_a_repo_private_id;

        // 1. The control: the sample's own root parenthetical, which is exactly
        //    what must not ship.
        assert!(
            cites_a_repo_private_id("(ISS-333 — a misspelt key is discarded)"),
            "the detector fires on a known citation, or the assertions below are unfalsified"
        );

        // 2. Non-vacuity: the surfaces under test are real documents.
        let contracts = extern_fixture();
        let document = render_document(&contracts);
        let prompt = render_prompt(&contracts).join("\n");
        let json = render_json(&contracts);
        for (surface, rendered) in [
            ("document", &document),
            ("prompt", &prompt),
            ("json", &json),
        ] {
            assert!(
                rendered.len() > 1000 && rendered.contains(PAYLOAD.name),
                "the {surface} rendering is {} bytes and must be a whole contract",
                rendered.len()
            );
        }

        // 3. The whole published document, and 4. both renderings it is built
        //    from — pin 10 covers all three surfaces.
        assert!(
            !cites_a_repo_private_id(&document),
            "the published document cites a per-repo sequential id"
        );
        assert!(
            !cites_a_repo_private_id(&prompt),
            "the prompt rendering cites a per-repo sequential id"
        );
        assert!(
            !cites_a_repo_private_id(&json),
            "the json rendering cites a per-repo sequential id"
        );
    }

    /// The published document is a banner, an intro and the prompt rendering in
    /// a fence — not a third emission of the rows (STD-001).
    #[test]
    fn the_document_carries_the_prompt_rendering_and_adds_no_second_table() {
        let contracts = extern_fixture();
        let document = render_document(&contracts);
        let lines = render_prompt(&contracts);

        assert!(
            document.starts_with("<!-- GENERATED"),
            "the document opens with the provenance banner"
        );
        assert!(document.contains(&lines.join("\n")), "{document}");
        assert_eq!(
            document.matches("payload ApplyRequest").count(),
            1,
            "the rows are rendered once"
        );
    }

    /// `PHASE-06`'s golden rests on this, and a `HashMap` slipped into the walk
    /// would make it flap there — a phase later, on someone else's time.
    #[test]
    fn both_renderings_are_deterministic() {
        let contracts = extern_fixture();
        assert_eq!(render_json(&contracts), render_json(&contracts));
        assert_eq!(render_prompt(&contracts), render_prompt(&contracts));
        assert_eq!(render_document(&contracts), render_document(&contracts));
    }

    /// Every region resolves to a supply, and the supply names the region's own
    /// spelling rather than a second one (STD-001).
    ///
    /// The barrier itself needs no test — `ExternContracts::region`'s match has
    /// no wildcard arm, so a region nobody supplies does not compile. What this
    /// checks is that the resolution is the *right* one and that a table's rows
    /// are readable, which is what PHASE-04 will populate.
    #[test]
    fn every_extern_region_resolves_to_its_own_supply() {
        let contracts = ExternContracts {
            knowledge_record: SelectorTable {
                source: ExternRegion::KnowledgeRecord.label(),
                rows: vec![SelectedKeys {
                    token: "assumption",
                    keys: vec![KeyContract {
                        key: "confidence",
                        ty: WireType::Token(TokenSource::Fixed(&["low", "medium", "high"])),
                        presence: Presence::Optional,
                    }],
                }],
                unknown_keys: UnknownKeys::Refused,
            },
        };

        for region in ExternRegion::ALL {
            let table = contracts.region(region);
            assert_eq!(table.source, region.label());
            assert_eq!(table.unknown_keys, UnknownKeys::Refused);
            let row = table.rows.first().expect("a supplied region has rows");
            assert_eq!(row.token, "assumption");
            let key = row.keys.first().expect("a row opens keys");
            assert_eq!(key.key, "confidence");
            assert_eq!(key.presence, Presence::Optional);
            assert_eq!(
                key.ty,
                WireType::Token(TokenSource::Fixed(&["low", "medium", "high"]))
            );
        }
    }
}
