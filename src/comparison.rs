// SPDX-License-Identifier: GPL-3.0-only
//! `comparison` — the pairwise comparison-session wire model (SL-210 PHASE-01).
//!
//! Pure engine tier (ADR-001): depends only on `crate::kinds` (leaf) plus
//! serde/toml. No clock, disk, rng, or git — dates and uids are function
//! inputs; the command shell (PHASE-02) mints them.
//!
//! The serde model IS the wire model: it serializes 1:1 to the documented
//! session-file schema (SL-210 design § Session-file schema) — top-level
//! `schema`/`version`, a nested `[session]` table, singular `[[judgement]]` /
//! `[[tombstone]]` arrays-of-tables, lowercase enum tokens. `frame` and
//! `domain` stay `String`-typed so unknown vocab in *future* files
//! round-trips losslessly; `rater`/`form` are closed enums by design — an
//! unknown token fails parse.

use serde::{Deserialize, Serialize};

use crate::kinds;

/// The `schema` discriminator every session file carries; checked on parse.
pub(crate) const COMPARISON_SCHEMA: &str = "doctrine.comparison-session";
/// Session-file directory under `.doctrine/` (the shell joins the root).
pub(crate) const COMPARISONS_DIR: &str = "comparisons";
/// The only comparison domain the verb writes at ship (design D8).
pub(crate) const DOMAIN_VALUE: &str = "value";
/// Value frame: "equal effort assumed" — the default framing.
pub(crate) const FRAME_EQUAL_EFFORT: &str = "equal-effort";
/// Value frame: "prefer whichever ships first".
pub(crate) const FRAME_PREFER_FIRST: &str = "prefer-first";
/// The closed frame vocab for the value domain (design D7).
pub(crate) const VALUE_FRAMES: &[&str] = &[FRAME_EQUAL_EFFORT, FRAME_PREFER_FIRST];

/// The only wire version this model reads or writes.
const COMPARISON_VERSION: u32 = 1;

/// Who rendered the judgement. Closed by design: an unknown rater token
/// fails parse (losslessness covers the frame/domain strings only).
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RaterKind {
    Human,
    Agent,
}

/// Row form. The verb exposes `order` only at ship; `ratio` keeps capture
/// lossless for RFC-019 OQ-6 (design D8). Closed: unknown tokens fail parse.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RowForm {
    Order,
    Ratio,
}

/// The `[session]` header table.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SessionHeader {
    pub uid: String,
    pub date: String,
    /// Optional audience tag — the OQ-1/T4 per-audience surfacing contract field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,
}

/// One `[[judgement]]` row: a single pairwise preference.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Judgement {
    pub uid: String,
    /// Row sequence within the file; ordering key is `(date, session_uid, seq)`.
    pub seq: u32,
    pub a: String,
    pub b: String,
    /// Must equal `a` or `b` ([`validate_judgement`]).
    pub preferred: String,
    pub domain: String,
    pub frame: String,
    pub form: RowForm,
    /// Optional value lens — the IDE-035 seam.
    pub lens: Option<String>,
    pub rater: RaterKind,
    /// Optional rater identity.
    pub by: Option<String>,
    pub note: Option<String>,
    pub date: String,
}

/// One `[[tombstone]]` row: an append-only withdrawal of a judgement row,
/// referenced by uid (file-order-independent).
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Tombstone {
    pub uid: String,
    pub seq: u32,
    /// The withdrawn judgement row's uid.
    pub target: String,
    pub date: String,
    pub note: Option<String>,
}

/// The file model — serializes 1:1 to the documented schema.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ComparisonSession {
    /// [`COMPARISON_SCHEMA`], checked on parse.
    pub schema: String,
    pub version: u32,
    pub session: SessionHeader,
    #[serde(default, rename = "judgement")]
    pub judgements: Vec<Judgement>,
    #[serde(default, rename = "tombstone")]
    pub tombstones: Vec<Tombstone>,
}

/// Parse a session-file body. Rejects a wrong `schema` discriminator or an
/// unsupported `version`; unknown frame/domain strings parse and round-trip.
pub(crate) fn parse(text: &str) -> anyhow::Result<ComparisonSession> {
    let s: ComparisonSession = toml::from_str(text)?;
    if s.schema != COMPARISON_SCHEMA {
        anyhow::bail!(
            "unrecognized comparison-session schema `{}` (expected `{COMPARISON_SCHEMA}`)",
            s.schema
        );
    }
    if s.version != COMPARISON_VERSION {
        anyhow::bail!(
            "unsupported comparison-session version {} (expected {COMPARISON_VERSION})",
            s.version
        );
    }
    Ok(s)
}

/// Serialize to a session-file body (serde-escaped — no raw splicing).
pub(crate) fn to_toml(s: &ComparisonSession) -> anyhow::Result<String> {
    Ok(toml::to_string(s)?)
}

/// Build a session-of-one: a fresh session carrying exactly one judgement and
/// no tombstones (design D2 — ad-hoc capture mints one file per invocation).
/// Stamps the current [`COMPARISON_SCHEMA`] + version so the shell never
/// hand-sets the wire discriminators.
pub(crate) fn session_of_one(session: SessionHeader, judgement: Judgement) -> ComparisonSession {
    ComparisonSession {
        schema: COMPARISON_SCHEMA.to_string(),
        version: COMPARISON_VERSION,
        session,
        judgements: vec![judgement],
        tombstones: Vec::new(),
    }
}

/// Structural row validation: preferred ∈ {a,b}, a ≠ b, non-empty refs,
/// closed domain/frame vocab. Admissibility is separate (needs kinds).
pub(crate) fn validate_judgement(j: &Judgement) -> anyhow::Result<()> {
    if j.a.is_empty() || j.b.is_empty() {
        anyhow::bail!("both sides of the pair are required — empty ref");
    }
    if j.a == j.b {
        anyhow::bail!("cannot compare `{}` against itself", j.a);
    }
    if j.preferred != j.a && j.preferred != j.b {
        anyhow::bail!(
            "preferred `{}` is not a side of the pair ({} / {})",
            j.preferred,
            j.a,
            j.b
        );
    }
    if j.domain != DOMAIN_VALUE {
        anyhow::bail!(
            "unknown domain `{}` — this verb writes `{DOMAIN_VALUE}` only",
            j.domain
        );
    }
    if !VALUE_FRAMES.contains(&j.frame.as_str()) {
        anyhow::bail!(
            "unknown value frame `{}` (expected one of: {})",
            j.frame,
            VALUE_FRAMES.join(", ")
        );
    }
    Ok(())
}

/// Value-domain admissibility over already-resolved kinds (pure; the kind
/// lookup happens in the shell). The admit set is `kinds::VALUE_BEARING`
/// minus `kinds::RSK` (design D6) — derived from those constants, never a
/// parallel list. `Err` carries the human-readable refusal.
pub(crate) fn admissible_value_pair(kind_a: &str, kind_b: &str) -> Result<(), String> {
    admissible_value_kind(kind_a)?;
    admissible_value_kind(kind_b)
}

/// One side of the pair: value-bearing, and not a risk (risk carries
/// exposure on its own facet, not comparable value — design D6).
fn admissible_value_kind(kind: &str) -> Result<(), String> {
    if kind == kinds::RSK {
        return Err(format!(
            "{kind} is excluded from value comparison — risk carries exposure, not value"
        ));
    }
    if !kinds::VALUE_BEARING.contains(&kind) {
        return Err(format!(
            "{kind} is not value-bearing — value comparison admits value-bearing kinds only"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_session() -> ComparisonSession {
        ComparisonSession {
            schema: COMPARISON_SCHEMA.to_string(),
            version: 1,
            session: SessionHeader {
                uid: "0197f3a2-5b1e-7c3d-9e4f-1a2b3c4d5e6f".to_string(),
                date: "2026-07-10".to_string(),
                audience: Some("stakeholder".to_string()),
            },
            judgements: vec![full_judgement()],
            tombstones: vec![Tombstone {
                uid: "0197f3a4-7d30-7e5f-9a6b-3c4d5e6f7a8b".to_string(),
                seq: 1,
                target: "0197f3a2-6c2f-7d4e-8f5a-2b3c4d5e6f7a".to_string(),
                date: "2026-07-10".to_string(),
                note: Some("wrong way round".to_string()),
            }],
        }
    }

    fn full_judgement() -> Judgement {
        Judgement {
            uid: "0197f3a2-6c2f-7d4e-8f5a-2b3c4d5e6f7a".to_string(),
            seq: 0,
            a: "SL-204".to_string(),
            b: "IMP-118".to_string(),
            preferred: "SL-204".to_string(),
            domain: DOMAIN_VALUE.to_string(),
            frame: FRAME_EQUAL_EFFORT.to_string(),
            form: RowForm::Order,
            lens: Some("user-value".to_string()),
            rater: RaterKind::Agent,
            by: Some("david".to_string()),
            note: Some("auth unblocks the pilot".to_string()),
            date: "2026-07-10".to_string(),
        }
    }

    /// A judgement with every optional absent.
    fn bare_judgement() -> Judgement {
        Judgement {
            uid: "0197f3a2-7e41-7f60-8b7c-4d5e6f7a8b9c".to_string(),
            seq: 1,
            a: "IMP-118".to_string(),
            b: "CHR-042".to_string(),
            preferred: "CHR-042".to_string(),
            domain: DOMAIN_VALUE.to_string(),
            frame: FRAME_PREFER_FIRST.to_string(),
            form: RowForm::Order,
            lens: None,
            rater: RaterKind::Human,
            by: None,
            note: None,
            date: "2026-07-10".to_string(),
        }
    }

    /// Pins the vocab constants to the documented schema strings (style
    /// precedent: `kinds::tests::groupings_match_documented_membership`).
    #[test]
    fn vocab_matches_documented_schema() {
        assert_eq!(COMPARISON_SCHEMA, "doctrine.comparison-session");
        assert_eq!(COMPARISONS_DIR, "comparisons");
        assert_eq!(DOMAIN_VALUE, "value");
        assert_eq!(VALUE_FRAMES, &["equal-effort", "prefer-first"]);
    }

    /// Byte-exact wire shape (RV-262 F-1): nested `[session]`, singular
    /// `[[judgement]]`/`[[tombstone]]`, lowercase enum tokens, fixed uids/dates.
    #[test]
    fn golden_shape_matches_documented_schema() {
        let expected = "\
schema = \"doctrine.comparison-session\"
version = 1

[session]
uid = \"0197f3a2-5b1e-7c3d-9e4f-1a2b3c4d5e6f\"
date = \"2026-07-10\"
audience = \"stakeholder\"

[[judgement]]
uid = \"0197f3a2-6c2f-7d4e-8f5a-2b3c4d5e6f7a\"
seq = 0
a = \"SL-204\"
b = \"IMP-118\"
preferred = \"SL-204\"
domain = \"value\"
frame = \"equal-effort\"
form = \"order\"
lens = \"user-value\"
rater = \"agent\"
by = \"david\"
note = \"auth unblocks the pilot\"
date = \"2026-07-10\"

[[tombstone]]
uid = \"0197f3a4-7d30-7e5f-9a6b-3c4d5e6f7a8b\"
seq = 1
target = \"0197f3a2-6c2f-7d4e-8f5a-2b3c4d5e6f7a\"
date = \"2026-07-10\"
note = \"wrong way round\"
";
        assert_eq!(to_toml(&full_session()).unwrap(), expected);
    }

    /// Losslessness: a row with ALL optionals set and a row with optionals
    /// absent both survive parse(to_toml(s)) == s.
    #[test]
    fn round_trip_preserves_all_fields() {
        let mut s = full_session();
        s.judgements.push(bare_judgement());
        let text = to_toml(&s).unwrap();
        assert_eq!(parse(&text).unwrap(), s);
    }

    /// Forward compatibility: an unknown frame string parses and round-trips
    /// verbatim — losslessness applies to the frame/domain STRINGS.
    #[test]
    fn parse_preserves_unknown_frame_rows() {
        let mut s = full_session();
        s.judgements[0].frame = "opportunity-cost".to_string();
        let text = to_toml(&s).unwrap();
        let parsed = parse(&text).unwrap();
        assert_eq!(parsed.judgements[0].frame, "opportunity-cost");
        assert_eq!(parsed, s);
    }

    #[test]
    fn parse_rejects_wrong_schema() {
        let mut s = full_session();
        s.schema = "doctrine.plan".to_string();
        let text = to_toml(&s).unwrap();
        let err = parse(&text).unwrap_err().to_string();
        assert!(
            err.contains(COMPARISON_SCHEMA),
            "err names expected schema: {err}"
        );
    }

    #[test]
    fn parse_rejects_wrong_version() {
        let mut s = full_session();
        s.version = 2;
        let text = to_toml(&s).unwrap();
        let err = parse(&text).unwrap_err().to_string();
        assert!(err.contains("version"), "err names version: {err}");
    }

    #[test]
    fn validate_accepts_well_formed_row() {
        assert!(validate_judgement(&full_judgement()).is_ok());
    }

    #[test]
    fn validate_rejects_preferred_outside_pair() {
        let mut j = full_judgement();
        j.preferred = "ISS-001".to_string();
        assert!(validate_judgement(&j).is_err());
    }

    #[test]
    fn validate_rejects_self_pair() {
        let mut j = full_judgement();
        j.b = j.a.clone();
        j.preferred = j.a.clone();
        assert!(validate_judgement(&j).is_err());
    }

    #[test]
    fn validate_rejects_unknown_frame() {
        let mut j = full_judgement();
        j.frame = "opportunity-cost".to_string();
        assert!(validate_judgement(&j).is_err());
    }

    #[test]
    fn validate_rejects_unknown_domain() {
        let mut j = full_judgement();
        j.domain = "effort".to_string();
        assert!(validate_judgement(&j).is_err());
    }

    #[test]
    fn validate_rejects_empty_refs() {
        let mut j = full_judgement();
        j.a = String::new();
        assert!(validate_judgement(&j).is_err());
        let mut j = full_judgement();
        j.b = String::new();
        assert!(validate_judgement(&j).is_err());
    }

    #[test]
    fn admissible_value_pair_admits_cross_kind_work() {
        assert!(admissible_value_pair(kinds::SL, kinds::IMP).is_ok());
    }

    #[test]
    fn admissible_value_pair_refuses_record() {
        assert!(admissible_value_pair(kinds::QUE, kinds::SL).is_err());
    }

    #[test]
    fn admissible_value_pair_refuses_rsk() {
        assert!(admissible_value_pair(kinds::SL, kinds::RSK).is_err());
    }

    /// Pins design D6 to the `kinds::` constants: the admit set IS
    /// `VALUE_BEARING` minus `RSK` — for every census kind, admission holds
    /// exactly when the derivation says so.
    #[test]
    fn admit_set_is_value_bearing_minus_rsk() {
        for &kind in kinds::ALL_KINDS {
            let admitted = admissible_value_pair(kind, kind).is_ok();
            let expected = kinds::VALUE_BEARING.contains(&kind) && kind != kinds::RSK;
            assert_eq!(
                admitted, expected,
                "{kind}: admitted iff value-bearing minus RSK"
            );
        }
    }
}
