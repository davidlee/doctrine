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

use super::Stage;
use super::attestation::{ActKind, AgentAct, ReviewDisposition, ReviewPolicy, Reviewer};
use super::ids::IdKind;
use super::inquiry::{InquiryLifecycle, Provenance};
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
#[expect(
    dead_code,
    reason = "SL-251 PHASE-06/07 land the first readers — the generator and its golden test"
)]
pub(crate) const PAYLOAD_CONTRACT_PATH: &str = "install/design-payload-contract.md";

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use serde::Serialize;
    use serde_json::Value;

    use super::super::attestation::ReviewRef;
    use super::super::fixture::id;
    use super::super::ids::Fingerprint;
    use super::super::submission::CreateRecord;
    use super::*;

    // -----------------------------------------------------------------------
    // VT-1 — the tokens the contract declares are the tokens serde writes
    // (sec-8 pin 4), and EX-9's authority pin rides the same walk.
    // -----------------------------------------------------------------------

    /// One variant of one closure enum, as the contract claims it.
    ///
    /// Deliberately carries **no token literal**: the token is what the walk
    /// derives from serde, and `VARIANTS` is what it is compared against. A
    /// third spelling here would be the drift the macro exists to remove.
    struct VariantSample {
        /// The payload shape the contract claims. With the enum's [`Tagging`]
        /// this decides where the token sits — and it is read *per variant*,
        /// never per type: `AgentAct::DraftingReady` is the live mixed case and
        /// a per-type reading fails on a correct table.
        payload: VariantPayload,
        /// A sample of this variant, serialised.
        value: Value,
        /// This variant's token according to the type's **own** authority, where
        /// one exists. `PHASE-01/EX-9`: `Provenance` and `ReviewDisposition`
        /// cannot take the macro's `via` arm, so the single source is recovered
        /// here instead — the walk is total over variants by the barrier, so
        /// this cannot silently lose a case.
        authority: Option<&'static str>,
    }

    /// One closure enum's claim.
    struct EnumClaim {
        type_name: &'static str,
        variants: &'static [&'static str],
        tagging: Tagging,
        samples: Vec<VariantSample>,
    }

    fn json(value: &impl Serialize) -> Value {
        serde_json::to_value(value).expect("a closure value serialises")
    }

    fn sample(payload: VariantPayload, value: &impl Serialize) -> VariantSample {
        VariantSample {
            payload,
            value: json(value),
            authority: None,
        }
    }

    /// A [`Provenance`] sample, pinned to `Provenance::label` (EX-9).
    fn provenance(payload: VariantPayload, value: Provenance) -> VariantSample {
        VariantSample {
            payload,
            authority: Some(value.label()),
            value: json(&value),
        }
    }

    /// A [`ReviewDisposition`] sample, pinned to `ReviewDisposition::arm` (EX-9).
    fn disposition(payload: VariantPayload, value: ReviewDisposition) -> VariantSample {
        VariantSample {
            payload,
            authority: Some(value.arm()),
            value: json(&value),
        }
    }

    /// A `Keys` payload. The rows are irrelevant to pin 4 — only *that* the
    /// variant carries a payload is, since that is what decides where the token
    /// sits. The rows themselves are `sec-8` pin 1's, in PHASE-02.
    const CARRIES_KEYS: VariantPayload = VariantPayload::Keys(&[]);

    /// A sample `CreateRecord`, for `Dispose::Create`'s inlining variant.
    fn create_record() -> CreateRecord {
        CreateRecord {
            kind: "decision".to_owned(),
            title: "A sample record".to_owned(),
            slug: None,
            body: None,
            facet: BTreeMap::new(),
            acceptance: None,
        }
    }

    /// Every closure enum, its declared vocabulary, and one sample per variant.
    ///
    /// Named rather than inlined into the assertion because `PHASE-03`'s
    /// `VT-4` consumes the same samples.
    fn claims() -> Vec<EnumClaim> {
        vec![
            EnumClaim {
                type_name: Stage::TYPE_NAME,
                variants: Stage::VARIANTS,
                tagging: Tagging::External,
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
                samples: ActKind::ALL
                    .iter()
                    .map(|act| sample(VariantPayload::Absent, act))
                    .collect(),
            },
            EnumClaim {
                type_name: ReviewPolicy::TYPE_NAME,
                variants: ReviewPolicy::VARIANTS,
                tagging: Tagging::External,
                samples: ReviewPolicy::ALL
                    .iter()
                    .map(|policy| sample(VariantPayload::Absent, policy))
                    .collect(),
            },
            EnumClaim {
                type_name: InquiryLifecycle::TYPE_NAME,
                variants: InquiryLifecycle::VARIANTS,
                tagging: Tagging::External,
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
                samples: vec![
                    sample(VariantPayload::Absent, &DischargeClaim::Attested),
                    sample(VariantPayload::Absent, &DischargeClaim::Skipped),
                ],
            },
            EnumClaim {
                type_name: DelegationAct::TYPE_NAME,
                variants: DelegationAct::VARIANTS,
                tagging: Tagging::Internal("act"),
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
                samples: vec![
                    sample(VariantPayload::Absent, &Reviewer::Human),
                    sample(VariantPayload::Absent, &Reviewer::Adversarial),
                ],
            },
            EnumClaim {
                type_name: Posture::TYPE_NAME,
                variants: Posture::VARIANTS,
                tagging: Tagging::External,
                samples: vec![
                    sample(VariantPayload::Absent, &Posture::Breadth),
                    sample(VariantPayload::Absent, &Posture::Depth),
                ],
            },
            EnumClaim {
                type_name: Authority::TYPE_NAME,
                variants: Authority::VARIANTS,
                tagging: Tagging::External,
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
                samples: vec![
                    sample(
                        VariantPayload::Inlines(&EXEMPLAR_CREATE_RECORD),
                        &Dispose::Create(create_record()),
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
                samples: Vec::new(),
            },
        ]
    }

    /// The token serde actually wrote, read through the contract's **own**
    /// per-variant claim (`sec-8` pin 4).
    fn serde_token(type_name: &str, tagging: Tagging, sample: &VariantSample) -> String {
        let value = &sample.value;
        match (tagging, sample.payload) {
            (Tagging::Internal(tag), _) => value
                .get(tag)
                .and_then(Value::as_str)
                .unwrap_or_else(|| panic!("{type_name}: no string at the tag {tag:?} in {value}"))
                .to_owned(),
            (Tagging::External, VariantPayload::Absent) => value
                .as_str()
                .unwrap_or_else(|| {
                    panic!("{type_name}: an external absent-payload variant is a bare string, got {value}")
                })
                .to_owned(),
            (Tagging::External, VariantPayload::Keys(_) | VariantPayload::Inlines(_)) => {
                let object = value.as_object().unwrap_or_else(|| {
                    panic!("{type_name}: an external payload variant nests under its token, got {value}")
                });
                assert_eq!(
                    object.len(),
                    1,
                    "{type_name}: an externally tagged payload has exactly one key, got {value}"
                );
                object
                    .keys()
                    .next()
                    .expect("an object of length one has a key")
                    .clone()
            }
            (Tagging::External, VariantPayload::Shape(_)) | (Tagging::Untagged, _) => {
                panic!("{type_name}: {value} carries no token to read")
            }
        }
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
