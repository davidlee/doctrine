// SPDX-License-Identifier: GPL-3.0-only
//! The payload contract, applied to a value (`SL-259` `PHASE-03`, `DEC-244`).
//!
//! [`super::payload_contract`] **describes** the wire — every key of every type
//! at every nesting level, pinned against a fully populated value's serde output
//! (`SL-251` `sec-8` pin 1). This module is the other half: it walks a submitted
//! JSON value against that description and refuses a key the description does
//! not hold.
//!
//! # Why a sibling and not a function in `payload_contract`
//!
//! Its neighbour is a *describing* module — it renders the contract and does
//! nothing to a run. Refusing a payload does something. The split follows
//! [`super::admission`]'s precedent: a self-contained pure check, homed beside
//! the owners it reads rather than inside either of them.
//!
//! Nothing is restated to get here. The tagging × payload table
//! ([`super::payload_contract::place`]) has one implementation and this is its
//! fourth consumer.
//!
//! # What this refuses, and what it leaves alone
//!
//! **Keys only.** Value shapes, closed vocabularies and required-key presence
//! are serde's and the pure core's; re-deciding them here would be the second
//! schema `DEC-244` exists to avoid.
//!
//! ADR-001 tier: **leaf, out-degree 0** — every `use` below is `super::…`.

use serde_json::Value;

#[cfg(test)]
use super::payload_contract::{CREATE_RECORD, closure_types};
use super::payload_contract::{
    Fields, KeyContract, MapKey, PAYLOAD, Placement, RETIRED_KEYS, RetiredKey, Tagging,
    TypeContract, TypeForm, VariantContract, VariantPayload, WireType, place, retired_from,
};
use super::refusal::Refusal;

/// Refuse any key of `payload` that the contract rooted at
/// [`super::payload_contract::PAYLOAD`] does not hold, at any nesting level.
///
/// # Where this runs, and why it needs nothing injected
///
/// A **leaf function of the value alone** — `EX-8`'s second seam, minus the
/// input that seam anticipated. The extern regions are the reason a walk might
/// have needed the command tier, and neither reaches the keys this walk judges:
/// [`MapKey::Extern`] keys are left to the seam that can resolve the region
/// (`EX-7`, and see [`walk_map`]), and [`super::payload_contract::TokenSource::Extern`]
/// describes a key's *value*, which a key walk does not read. So the tier above
/// calls this with the parsed JSON and nothing else.
///
/// # What a refusal is, and is not
///
/// A property of the request against the schema (`DEC-245`). Three neighbouring
/// faults belong to other layers and are deliberately passed over here, each by
/// name at its site: a value of the wrong **shape** (serde will say so), an
/// unknown variant **token** (likewise), and a key that is known but inert at the
/// subject's **state** (`DEC-246`, the state axis).
pub(crate) fn refuse_unknown_keys(payload: &Value) -> Result<(), Refusal> {
    refuse_unknown_keys_against(payload, RETIRED_KEYS)
}

/// [`refuse_unknown_keys`] against a given retired-key roster — the one seam a
/// test reaches with a roster of its own while [`RETIRED_KEYS`] is empty.
fn refuse_unknown_keys_against(payload: &Value, retired: &[RetiredKey]) -> Result<(), Refusal> {
    walk_type(payload, &PAYLOAD, retired, "")
}

/// A value against the type that describes it.
fn walk_type(
    value: &Value,
    contract: &'static TypeContract,
    retired: &[RetiredKey],
    at: &str,
) -> Result<(), Refusal> {
    match contract.form {
        TypeForm::Struct { keys, .. } => match value.as_object() {
            Some(object) => walk_keys(&Fields::plain(object), contract, keys, retired, at),
            // A struct target that is not an object is a shape fault, and serde
            // reports it a moment later in its own words. Classifying it here
            // would be this walk answering a question it was not asked.
            None => Ok(()),
        },
        TypeForm::Enum { tagging, variants } => {
            walk_enum(value, contract, tagging, variants, retired, at)
        }
    }
}

/// A key surface against the rows that describe it: **subset, not equality**.
///
/// The equality form of this is pin 2's [`super::tests`] descent, which runs
/// against a fully populated fixture. A real payload is sparse by design
/// (`Presence::Optional`, `Presence::Sparse`), so a missing key is not this
/// walk's business — serde refuses a missing *required* one.
fn walk_keys(
    fields: &Fields<'_>,
    owner: &'static TypeContract,
    keys: &'static [KeyContract],
    retired: &[RetiredKey],
    at: &str,
) -> Result<(), Refusal> {
    // `Fields::names` is ordered, so a payload with two unknown keys names the
    // same one on every run.
    for name in fields.names() {
        if !keys.iter().any(|row| row.key == name) {
            // A once-known key earns its remedy, not the admitted list (`DEC-278`).
            if let Some(row) = retired_from(retired, owner).find(|row| row.key == name) {
                return Err(Refusal::RetiredPayloadKey {
                    at: child(at, name),
                    type_name: owner.name.to_owned(),
                    key: name.to_owned(),
                    remedy: row.remedy.to_owned(),
                });
            }
            return Err(Refusal::UnknownPayloadKey {
                at: child(at, name),
                type_name: owner.name.to_owned(),
                key: name.to_owned(),
                admitted: keys.iter().map(|row| row.key.to_owned()).collect(),
            });
        }
    }
    for row in keys {
        if let Some(value) = fields.get(row.key) {
            walk_wire(value, row.ty, retired, &child(at, row.key))?;
        }
    }
    Ok(())
}

/// Select the variant, then walk what rides with it.
///
/// Selection reads [`place`] — `sec-2`'s tagging × payload table — and never
/// restates it. **No matching variant is not a refusal from here** (`A3`): an
/// unrecognised token is serde's complaint, and answering it with an
/// unknown-*key* refusal would be a refusal firing for the wrong reason.
fn walk_enum(
    value: &Value,
    contract: &'static TypeContract,
    tagging: Tagging,
    variants: &'static [VariantContract],
    retired: &[RetiredKey],
    at: &str,
) -> Result<(), Refusal> {
    match tagging {
        Tagging::Internal(_) | Tagging::External => {
            let Some(variant) = variants.iter().find(|variant| {
                place(contract.name, tagging, variant.payload, value)
                    .is_ok_and(|placement| placement.token() == variant.token)
            }) else {
                return Ok(());
            };
            let Ok(placement) = place(contract.name, tagging, variant.payload, value) else {
                return Ok(());
            };
            walk_payload(&placement, variant.payload, contract, retired, at)
        }
        // Untagged: the variant is a shape rather than a token, and no untagged
        // variant in the closure reaches a key surface (`EN-4`; pinned by
        // `no_untagged_variant_reaches_a_key_surface` below, so a future
        // untagged variant that does fails there rather than here). Walking each
        // arm is sound precisely because a shape a value does not have reads as
        // nothing to descend.
        //
        // The failure mode if that stops holding is a FALSE REFUSAL, not an
        // unwalked pass-through: with two untagged variants over key surfaces,
        // a value of one is walked against the other's key rows and every key it
        // does not share earns `UnknownPayloadKey` — `walk_map`'s "expensive
        // direction", and the inverse of the defect this walk exists to fix.
        Tagging::Untagged => variants.iter().try_for_each(|variant| {
            walk_payload(
                &Placement::Shape(value),
                variant.payload,
                contract,
                retired,
                at,
            )
        }),
    }
}

/// The four [`VariantPayload`] arms, **with no wildcard** — a new arm is a
/// compile error here, as it is in every other walk over this model.
fn walk_payload(
    placement: &Placement<'_>,
    payload: VariantPayload,
    owner: &'static TypeContract,
    retired: &[RetiredKey],
    at: &str,
) -> Result<(), Refusal> {
    match payload {
        VariantPayload::Absent => Ok(()),
        VariantPayload::Keys(keys) => match placement.fields() {
            // `Fields` excludes the tag, so an internally tagged variant's own
            // discriminant is never read as an undeclared key.
            Some(fields) => walk_keys(&fields, owner, keys, retired, at),
            None => Ok(()),
        },
        VariantPayload::Inlines(target) => match (placement.fields(), target.form) {
            (Some(fields), TypeForm::Struct { keys, .. }) => {
                walk_keys(&fields, target, keys, retired, at)
            }
            // An inlined *enum* has no key surface at this level — its own
            // tagging decides where its keys sit, and the closure holds no such
            // variant today (`Dispose::Create(CreateRecord)` is the only
            // `Inlines`). Left as an explicit arm rather than a wildcard.
            (Some(_), TypeForm::Enum { .. }) | (None, _) => Ok(()),
        },
        VariantPayload::Shape(shape) => match *placement {
            Placement::Shape(inner) => walk_wire(inner, *shape, retired, at),
            // `place` refuses to pair a tagged placement with a shape payload,
            // so selection cannot deliver one here.
            Placement::Bare(_) | Placement::Beside { .. } | Placement::Nested { .. } => Ok(()),
        },
    }
}

/// One key's value against the type declared for it. No wildcard arm.
fn walk_wire(value: &Value, ty: WireType, retired: &[RetiredKey], at: &str) -> Result<(), Refusal> {
    match ty {
        // Scalars carry no keys. Their vocabularies and bounds are the pure
        // core's (`A1`).
        WireType::Text
        | WireType::Integer
        | WireType::Boolean
        | WireType::Id(_)
        | WireType::Token(_) => Ok(()),
        WireType::Named(target) => walk_type(value, target, retired, at),
        WireType::Seq(inner) => match value.as_array() {
            Some(items) => items.iter().enumerate().try_for_each(|(index, item)| {
                walk_wire(item, *inner, retired, &format!("{at}[{index}]"))
            }),
            None => Ok(()),
        },
        WireType::Map { key, value: item } => walk_map(value, key, item, retired, at),
    }
}

/// A map's keys are **values, not struct rows**, and which kind of value is what
/// the contract's [`MapKey`] says (`EX-7`).
///
/// This is the arm a careless walk gets wrong in the expensive direction:
/// treating `CreateRecord.facet`'s keys as a closed inventory would refuse every
/// legitimate facet name — this slice's own defect, inverted into refusing input
/// the engine would have acted on.
fn walk_map(
    value: &Value,
    key: MapKey,
    item: &'static WireType,
    retired: &[RetiredKey],
    at: &str,
) -> Result<(), Refusal> {
    let Some(object) = value.as_object() else {
        return Ok(());
    };
    for (name, item_value) in object {
        match key {
            // Routed through the value walk rather than given a bespoke check,
            // so a `WireType` arm that ever has something to say about a scalar
            // says it about map keys too.
            MapKey::Of(key_ty) => {
                walk_wire(
                    &Value::String(name.clone()),
                    *key_ty,
                    retired,
                    &child(at, name),
                )?;
            }
            // Skipped by name, never by a wildcard: which keys are legal is
            // chosen by the *value* of a sibling field, from a vocabulary this
            // leaf has crate out-degree zero to (ADR-001). The seam that can
            // resolve the region already refuses an unknown one before an id is
            // reserved — `knowledge::plan_facet_edits`, via
            // `SelectorTable::unknown_keys`, which reads `Refused`.
            MapKey::Extern { .. } => {}
        }
        walk_wire(item_value, *item, retired, &child(at, name))?;
    }
    Ok(())
}

/// One step down the dotted path a refusal reports.
fn child(at: &str, key: &str) -> String {
    if at.is_empty() {
        key.to_owned()
    } else {
        format!("{at}.{key}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A well-formed minimum: the three [`super::super::submission::SubmissionEnvelope`]
    /// keys `PAYLOAD` flattens in, and nothing else.
    fn envelope() -> serde_json::Map<String, Value> {
        let Value::Object(object) = serde_json::json!({
            "run_uid": "run-1",
            "known_revision": 7,
            "submission_id": "sub-1",
        }) else {
            unreachable!("a json! object literal is an object")
        };
        object
    }

    /// The refusal a payload earns, as a caller reads it.
    fn refusal_of(payload: &Value) -> String {
        refuse_unknown_keys(payload)
            .expect_err("the payload carries a key the contract does not hold")
            .to_string()
    }

    /// `VT-1` — the case `deny_unknown_fields` structurally cannot reach
    /// (`ISS-333`): `ApplyRequest` carries `#[serde(flatten)]`, so serde admits
    /// and discards a misspelt top-level key in silence. It is also the one a
    /// caller hand-authors from scratch.
    ///
    /// The **reason** is pinned, not the exit: a refusal that fires for the
    /// wrong reason is a test passing for the wrong reason (`sec-8` leg 3).
    #[test]
    fn an_unknown_top_level_key_inside_the_flatten_envelope_is_refused() {
        let mut object = envelope();
        object.insert("stagge".to_owned(), serde_json::json!({ "to": "shaping" }));
        let reason = refusal_of(&Value::Object(object));

        assert!(
            reason.contains("stagge"),
            "the refusal names the offending key: {reason}"
        );
        assert!(
            reason.contains("ApplyRequest"),
            "the refusal names the type that admitted it: {reason}"
        );
        assert!(
            reason.contains("stage"),
            "the refusal names what was expected instead: {reason}"
        );
    }

    /// The control on the test above: the same payload without the misspelling
    /// is accepted, so the refusal is not firing on the envelope itself.
    #[test]
    fn a_payload_carrying_only_contract_keys_is_accepted() {
        let mut object = envelope();
        object.insert("stage".to_owned(), serde_json::json!({ "to": "shaping" }));
        assert_eq!(refuse_unknown_keys(&Value::Object(object)), Ok(()));
    }

    /// A payload carrying one `declare` row, whose `dispose` is `disposition`.
    fn declaring(disposition: Value) -> Value {
        let mut object = envelope();
        object.insert(
            "declare".to_owned(),
            serde_json::json!([{ "subject": "inq-1", "dispose": disposition }]),
        );
        Value::Object(object)
    }

    /// `VT-2` — `ISS-328`: `CreateRecord` sits two levels inside a `Declaration`
    /// that *does* carry `deny_unknown_fields`, and carries none itself, so an
    /// unknown key in a `form = "create"` disposition was dropped in silence
    /// with none of the structural excuse the flatten envelope has.
    #[test]
    fn an_unknown_key_nested_inside_create_record_is_refused() {
        let reason = refusal_of(&declaring(serde_json::json!({
            "form": "create",
            "kind": "issue",
            "title": "a record",
            "titel": "the misspelling",
        })));

        assert!(
            reason.contains("titel"),
            "the refusal names the offending key: {reason}"
        );
        assert!(
            reason.contains("CreateRecord"),
            "the refusal names the type that admitted it, not the enclosing one: {reason}"
        );
        assert!(
            reason.contains("declare[0].dispose.titel"),
            "the refusal locates it in the payload: {reason}"
        );
    }

    /// `VT-3` — the second place `deny_unknown_fields` cannot reach: an
    /// internally tagged variant buffers its content, and its payload keys sit
    /// *beside* the tag rather than under it.
    #[test]
    fn an_unknown_key_inside_an_internally_tagged_variant_is_refused() {
        let reason = refusal_of(&declaring(serde_json::json!({
            "form": "adopt",
            "record": "ISS-1",
            "recrod": "the misspelling",
        })));

        assert!(
            reason.contains("recrod"),
            "the refusal names the offending key: {reason}"
        );
        assert!(
            reason.contains("Dispose"),
            "the refusal names the type that admitted it: {reason}"
        );
        assert!(
            reason.contains("`record`"),
            "the refusal names what was expected instead: {reason}"
        );
    }

    /// The tag is not a row. Without [`Fields`] excluding it, `form` — which
    /// every internally tagged variant must carry — would read as the first
    /// undeclared key of every such variant, and this walk would refuse every
    /// well-formed disposition in the corpus.
    #[test]
    fn an_internally_tagged_variants_own_tag_is_not_an_unknown_key() {
        assert_eq!(
            refuse_unknown_keys(&declaring(serde_json::json!({
                "form": "adopt",
                "record": "ISS-1",
            }))),
            Ok(())
        );
    }

    /// `VT-5` — the negative control on `EX-7`. `CreateRecord.facet`'s legal
    /// keys are chosen by the *value* of its sibling `kind`, from a vocabulary
    /// this leaf has crate out-degree zero to. A walk that closed this map would
    /// refuse every legitimate facet name: this slice's own defect, inverted
    /// into refusing input the engine would have acted on.
    #[test]
    fn facet_map_keys_are_not_judged_as_struct_keys() {
        assert_eq!(
            refuse_unknown_keys(&declaring(serde_json::json!({
                "form": "create",
                "kind": "issue",
                "title": "a record",
                "facet": { "severity": "high", "tags": ["a", "b"] },
            }))),
            Ok(())
        );
    }

    /// A roster that retires `titel` from `CreateRecord` — a test-local stand-in
    /// for [`RETIRED_KEYS`], which retires nothing until `PHASE-05`.
    static ROSTER: &[RetiredKey] = &[RetiredKey {
        owner: &CREATE_RECORD,
        key: "titel",
        remedy: "send `title` instead",
    }];

    /// `create` disposition carrying `key` beside its required rows.
    fn creating_with(key: &str) -> Value {
        declaring(serde_json::json!({
            "form": "create",
            "kind": "issue",
            "title": "a record",
            key: "a value",
        }))
    }

    /// `SL-261` `VT-1` (`DEC-278`) — a once-admitted key is refused as retired,
    /// naming its owner and the remedy rather than listing admitted keys that
    /// never mention the replacement.
    #[test]
    fn retired_key_is_refused_with_its_remedy() {
        let refused = refuse_unknown_keys_against(&creating_with("titel"), ROSTER)
            .expect_err("a retired key is refused");
        assert!(
            matches!(refused, Refusal::RetiredPayloadKey { .. }),
            "refused as retired, not unknown: {refused:?}"
        );
        let reason = refused.to_string();
        for part in [
            "`titel`",
            "declare[0].dispose.titel",
            "CreateRecord",
            "send `title` instead",
        ] {
            assert!(reason.contains(part), "the refusal names {part}: {reason}");
        }
    }

    /// `SL-261` `VT-1` — the roster matches its owner by **identity**, not by
    /// key name: the same name under another type is simply unknown there.
    #[test]
    fn retired_key_matches_its_owner_only() {
        let mut object = envelope();
        object.insert("titel".to_owned(), Value::from("a value"));
        let refused = refuse_unknown_keys_against(&Value::Object(object), ROSTER)
            .expect_err("an unknown key is refused");
        assert!(
            matches!(refused, Refusal::UnknownPayloadKey { ref type_name, .. } if type_name == "ApplyRequest"),
            "a key retired from CreateRecord is unknown on ApplyRequest: {refused:?}"
        );
    }

    /// `SL-261` `VT-3` — a key no type ever admitted keeps the existing refusal
    /// verbatim, roster or no roster.
    #[test]
    fn unknown_key_refusal_is_unchanged_for_never_known_keys() {
        let payload = creating_with("colour");
        let with_roster = refuse_unknown_keys_against(&payload, ROSTER);
        assert!(
            matches!(with_roster, Err(Refusal::UnknownPayloadKey { .. })),
            "a never-known key is unknown: {with_roster:?}"
        );
        assert_eq!(with_roster, refuse_unknown_keys_against(&payload, &[]));
        assert_eq!(with_roster, refuse_unknown_keys(&payload));
    }

    /// Whether a wire type can reach a **key surface** — a described type's key
    /// rows — directly or through what [`walk_wire`] descends into.
    ///
    /// No wildcard, exactly as in `walk_wire`: a new [`WireType`] arm is a
    /// compile error here and has to say which side of this it falls on. Map
    /// keys recurse because `walk_map` routes them through `walk_wire` too,
    /// rather than giving a map key a bespoke check.
    fn reaches_a_key_surface(ty: WireType) -> bool {
        match ty {
            WireType::Text
            | WireType::Integer
            | WireType::Boolean
            | WireType::Id(_)
            | WireType::Token(_) => false,
            WireType::Named(_) => true,
            WireType::Seq(inner) => reaches_a_key_surface(*inner),
            WireType::Map { key, value } => {
                let keyed = match key {
                    MapKey::Of(key_ty) => reaches_a_key_surface(*key_ty),
                    // The extern region's keys are skipped by name in
                    // `walk_map`, so they reach nothing from here.
                    MapKey::Extern { .. } => false,
                };
                keyed || reaches_a_key_surface(*value)
            }
        }
    }

    /// [`reaches_a_key_surface`] answers for the shapes the pin below rejects —
    /// including the one the weaker `matches!(.., Shape(_))` form admitted
    /// (`RV-367` `F-4`): `Shape(Named(..))` is a legal inhabitant and it carries
    /// a full key surface.
    #[test]
    fn a_key_surface_is_reached_through_a_seq_or_a_map() {
        static NAMED: WireType = WireType::Named(&PAYLOAD);
        assert!(reaches_a_key_surface(NAMED));
        assert!(reaches_a_key_surface(WireType::Seq(&NAMED)));
        assert!(reaches_a_key_surface(WireType::Map {
            key: MapKey::Of(&WireType::Text),
            value: &NAMED,
        }));
        assert!(!reaches_a_key_surface(WireType::Text));
        assert!(!reaches_a_key_surface(WireType::Seq(&WireType::Text)));
        assert!(!reaches_a_key_surface(WireType::Map {
            key: MapKey::Of(&WireType::Text),
            value: &WireType::Boolean,
        }));
    }

    /// The claim [`walk_enum`]'s `Untagged` arm rests on: no untagged variant in
    /// the closure reaches a key surface, so walking a value against every arm
    /// cannot refuse a key a sibling arm does not admit. `WireFacetValue` is the
    /// only inhabitant today (`EN-4`), and the point of the pin is the day it is
    /// not.
    ///
    /// Asserting [`VariantPayload::Shape`] alone would be weaker than the claim
    /// it is cited for (`RV-367` `F-4`): a `Shape` holds a [`WireType`], and
    /// `WireType::Named` — a described type, keys and all — is one.
    #[test]
    fn no_untagged_variant_reaches_a_key_surface() {
        let mut untagged = 0;
        for contract in closure_types(&PAYLOAD) {
            let TypeForm::Enum {
                tagging: Tagging::Untagged,
                variants,
            } = contract.form
            else {
                continue;
            };
            untagged += 1;
            for variant in variants {
                let VariantPayload::Shape(shape) = variant.payload else {
                    panic!(
                        "{}'s untagged variant carries keys rather than a shape, \
                         so `walk_enum` would refuse a sibling variant's value",
                        contract.name
                    );
                };
                assert!(
                    !reaches_a_key_surface(*shape),
                    "{} carries an untagged variant whose shape reaches a key \
                     surface, so `walk_enum` would refuse a sibling variant's \
                     value for keys this one does not admit",
                    contract.name
                );
            }
        }
        assert!(
            untagged > 0,
            "the closure holds an untagged type, or this pin is vacuous"
        );
    }

    /// `A3` — an unrecognised variant token is serde's complaint, not this
    /// walk's. Answering it with an unknown-*key* refusal would be a refusal
    /// firing for the wrong reason, and the test that pins the right reason
    /// would still pass.
    #[test]
    fn an_unknown_variant_token_is_left_to_serde() {
        assert_eq!(
            refuse_unknown_keys(&declaring(serde_json::json!({
                "form": "invent",
                "record": "ISS-1",
            }))),
            Ok(())
        );
    }
}
