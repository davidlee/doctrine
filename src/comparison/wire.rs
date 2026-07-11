// SPDX-License-Identifier: GPL-3.0-only
//! `comparison::wire` — the pairwise comparison-session wire model, schema v2
//! (SL-213 PHASE-01; v1 retired in place per design D1 — verified zero
//! exposure, no release ever shipped it).
//!
//! Pure leaf tier (ADR-001): depends only on `crate::kinds` plus serde/toml.
//! No clock, disk, rng, or git — dates and uids are function inputs; the
//! command shell mints them.
//!
//! The serde model IS the wire model: it serializes 1:1 to the documented
//! session-file schema — top-level `schema`/`version`, a nested `[session]`
//! table, singular `[[judgement]]` / `[[tombstone]]` arrays-of-tables,
//! lowercase/kebab-case enum tokens. `frame` and `domain` stay `String`-typed
//! so unknown vocab in *future* files round-trips losslessly; `response`,
//! `rater` and `form` are closed enums by design — an unknown token fails
//! parse.

use serde::{Deserialize, Serialize};

use crate::kinds;

/// The `schema` discriminator every session file carries; checked on parse.
pub(crate) const COMPARISON_SCHEMA: &str = "doctrine.comparison-session";
/// Session-file directory under `.doctrine/` (the shell joins the root).
pub(crate) const COMPARISONS_DIR: &str = "comparisons";
/// The value domain: rows compile to `v_winner > v_loser` (Phase B+).
pub(crate) const DOMAIN_VALUE: &str = "value";
/// The priority domain (design D2): capacity-cutoff testimony — value-oriented
/// but cost-confounded, so never compiled to a value constraint; inert until a
/// consumer with a cost model exists.
pub(crate) const DOMAIN_PRIORITY: &str = "priority";
/// Value frame: "equal effort assumed" — the default framing.
pub(crate) const FRAME_EQUAL_EFFORT: &str = "equal-effort";
/// Priority frame: "under a binding capacity cutoff, which do you keep?".
pub(crate) const FRAME_PREFER_FIRST: &str = "prefer-first";

/// Per-domain closed frame vocabulary (design D2). The frame implies the
/// domain at capture — users never type a domain; [`domain_for_frame`] is the
/// single derivation seam and this table its single source (STD-001).
pub(crate) const DOMAIN_FRAMES: &[(&str, &[&str])] = &[
    (DOMAIN_VALUE, &[FRAME_EQUAL_EFFORT]),
    (DOMAIN_PRIORITY, &[FRAME_PREFER_FIRST]),
];

/// The only wire version this model reads or writes (design D1: `version ≠ 2`
/// is a parse error — v1 was never released).
pub(crate) const COMPARISON_VERSION: u32 = 2;

/// The domain a frame implies at capture (design S1: `--frame prefer-first`
/// derives `domain = priority` silently).
pub(crate) fn domain_for_frame(frame: &str) -> Option<&'static str> {
    DOMAIN_FRAMES
        .iter()
        .find(|(_, frames)| frames.contains(&frame))
        .map(|(domain, _)| *domain)
}

/// The closed frame set for a domain, if the domain is known.
fn frames_for_domain(domain: &str) -> Option<&'static [&'static str]> {
    DOMAIN_FRAMES
        .iter()
        .find(|(d, _)| *d == domain)
        .map(|(_, frames)| *frames)
}

/// Who rendered the judgement. Closed by design: an unknown rater token
/// fails parse (losslessness covers the frame/domain strings only).
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RaterKind {
    Human,
    Agent,
}

/// Row form. The verb exposes `order` only; `ratio` keeps capture lossless
/// for RFC-019 OQ-6. Closed: unknown tokens fail parse.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RowForm {
    Order,
    Ratio,
}

/// The elicited answer (design D1/S1): one of the two sides preferred, an
/// exact-equality statement, or a considered "these don't compare" —
/// `incomparable` is valid evidence that compiles to zero constraint.
/// Closed: unknown tokens fail parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Response {
    PreferA,
    PreferB,
    Equal,
    Incomparable,
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

/// One `[[judgement]]` row: a single pairwise judgement.
///
/// No `Eq`: `magnitude` is an `f64` column (parsed, uncompiled — RFC-019 OQ-6
/// stays open; pure order semantics per design D8 ignore it).
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Judgement {
    pub uid: String,
    /// Row sequence within the file; ordering key is `(date, session_uid, seq)`.
    pub seq: u32,
    pub a: String,
    pub b: String,
    pub response: Response,
    pub domain: String,
    pub frame: String,
    pub form: RowForm,
    /// Ratio column — carried losslessly, never compiled (design C1).
    pub magnitude: Option<f64>,
    /// Explicit supersession target: this row's uid replaces that row's
    /// testimony (design R2 — a durable act, not testimony).
    pub supersedes: Option<String>,
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

/// The file model — serializes 1:1 to the documented schema. No `Eq`
/// (contains [`Judgement`]).
#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

/// Parse a session-file body. Rejects a wrong `schema` discriminator or any
/// version other than [`COMPARISON_VERSION`] with a remedy-naming message
/// (design D1); unknown frame/domain strings parse and round-trip.
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
            "unsupported comparison-session version {} (expected {COMPARISON_VERSION}) — \
             schema version 1 was never released; delete or recreate this session file",
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
/// no tombstones (ad-hoc capture mints one file per invocation). Stamps the
/// current [`COMPARISON_SCHEMA`] + version so the shell never hand-sets the
/// wire discriminators.
pub(crate) fn session_of_one(session: SessionHeader, judgement: Judgement) -> ComparisonSession {
    ComparisonSession {
        schema: COMPARISON_SCHEMA.to_string(),
        version: COMPARISON_VERSION,
        session,
        judgements: vec![judgement],
        tombstones: Vec::new(),
    }
}

/// Structural row validation: a ≠ b, non-empty refs, closed per-domain frame
/// vocabulary (design D2 — the frame table is normative at capture; parse-level
/// losslessness is separate). `response` needs no check — the closed enum
/// makes an invalid answer unrepresentable. Admissibility is separate (needs
/// kinds).
pub(crate) fn validate_judgement(j: &Judgement) -> anyhow::Result<()> {
    if j.a.is_empty() || j.b.is_empty() {
        anyhow::bail!("both sides of the pair are required — empty ref");
    }
    if j.a == j.b {
        anyhow::bail!("cannot compare `{}` against itself", j.a);
    }
    let Some(frames) = frames_for_domain(&j.domain) else {
        let domains: Vec<&str> = DOMAIN_FRAMES.iter().map(|(d, _)| *d).collect();
        anyhow::bail!(
            "unknown domain `{}` (expected one of: {})",
            j.domain,
            domains.join(", ")
        );
    };
    if !frames.contains(&j.frame.as_str()) {
        anyhow::bail!(
            "frame `{}` is not admissible in domain `{}` (expected one of: {})",
            j.frame,
            j.domain,
            frames.join(", ")
        );
    }
    Ok(())
}

/// Pair admissibility over already-resolved kinds (pure; the kind lookup
/// happens in the shell). The admit set is `kinds::VALUE_BEARING` minus
/// `kinds::RSK` — derived from those constants, never a parallel list. The
/// `priority` domain reuses this set initially (design D2 — widened only when
/// a consumer exists to justify it). `Err` carries the human-readable refusal.
pub(crate) fn admissible_value_pair(kind_a: &str, kind_b: &str) -> Result<(), String> {
    admissible_value_kind(kind_a)?;
    admissible_value_kind(kind_b)
}

/// One side of the pair: value-bearing, and not a risk (risk carries
/// exposure on its own facet, not comparable value).
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
            version: 2,
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

    /// A judgement with EVERY optional set (magnitude, supersedes, lens, by,
    /// note) — the golden's maximal row.
    fn full_judgement() -> Judgement {
        Judgement {
            uid: "0197f3a2-6c2f-7d4e-8f5a-2b3c4d5e6f7a".to_string(),
            seq: 0,
            a: "SL-204".to_string(),
            b: "IMP-118".to_string(),
            response: Response::PreferA,
            domain: DOMAIN_VALUE.to_string(),
            frame: FRAME_EQUAL_EFFORT.to_string(),
            form: RowForm::Order,
            magnitude: Some(2.5),
            supersedes: Some("0197f3a1-1111-7abc-8def-0a1b2c3d4e5f".to_string()),
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
            response: Response::PreferB,
            domain: DOMAIN_PRIORITY.to_string(),
            frame: FRAME_PREFER_FIRST.to_string(),
            form: RowForm::Order,
            magnitude: None,
            supersedes: None,
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
        assert_eq!(COMPARISON_VERSION, 2);
        assert_eq!(DOMAIN_VALUE, "value");
        assert_eq!(DOMAIN_PRIORITY, "priority");
        assert_eq!(
            DOMAIN_FRAMES,
            &[
                ("value", &["equal-effort"][..]),
                ("priority", &["prefer-first"][..])
            ]
        );
    }

    /// The per-domain frame table drives capture derivation both ways
    /// (design D2/S1): frame → domain is total over the closed vocab, and an
    /// unknown frame derives nothing.
    #[test]
    fn domain_for_frame_derives_from_the_table() {
        assert_eq!(domain_for_frame(FRAME_EQUAL_EFFORT), Some(DOMAIN_VALUE));
        assert_eq!(domain_for_frame(FRAME_PREFER_FIRST), Some(DOMAIN_PRIORITY));
        assert_eq!(domain_for_frame("opportunity-cost"), None);
    }

    /// Byte-exact wire shape: nested `[session]`, singular
    /// `[[judgement]]`/`[[tombstone]]`, kebab-case response token, v2 columns
    /// (`magnitude`, `supersedes`), fixed uids/dates.
    #[test]
    fn golden_shape_matches_documented_schema() {
        let expected = "\
schema = \"doctrine.comparison-session\"
version = 2

[session]
uid = \"0197f3a2-5b1e-7c3d-9e4f-1a2b3c4d5e6f\"
date = \"2026-07-10\"
audience = \"stakeholder\"

[[judgement]]
uid = \"0197f3a2-6c2f-7d4e-8f5a-2b3c4d5e6f7a\"
seq = 0
a = \"SL-204\"
b = \"IMP-118\"
response = \"prefer-a\"
domain = \"value\"
frame = \"equal-effort\"
form = \"order\"
magnitude = 2.5
supersedes = \"0197f3a1-1111-7abc-8def-0a1b2c3d4e5f\"
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

    /// Every `Response` variant serializes to its documented kebab-case token
    /// and parses back (VT-1: the response vocabulary is the wire contract).
    #[test]
    fn response_vocabulary_round_trips() {
        let cases = [
            (Response::PreferA, "prefer-a"),
            (Response::PreferB, "prefer-b"),
            (Response::Equal, "equal"),
            (Response::Incomparable, "incomparable"),
        ];
        for (response, token) in cases {
            let mut s = full_session();
            s.judgements[0].response = response;
            let text = to_toml(&s).unwrap();
            assert!(
                text.contains(&format!("response = \"{token}\"")),
                "{token} on the wire:\n{text}"
            );
            assert_eq!(parse(&text).unwrap().judgements[0].response, response);
        }
    }

    /// An unknown response token is a parse error — the enum is closed.
    #[test]
    fn parse_rejects_unknown_response_token() {
        let text = to_toml(&full_session())
            .unwrap()
            .replace("response = \"prefer-a\"", "response = \"prefer-c\"");
        assert!(parse(&text).is_err());
    }

    /// Losslessness: a row with ALL optionals set (incl. magnitude +
    /// supersedes) and a row with optionals absent both survive
    /// parse(to_toml(s)) == s.
    #[test]
    fn round_trip_preserves_all_fields() {
        let mut s = full_session();
        s.judgements.push(bare_judgement());
        let text = to_toml(&s).unwrap();
        assert_eq!(parse(&text).unwrap(), s);
    }

    /// Forward compatibility: an unknown frame string parses and round-trips
    /// verbatim — losslessness applies to the frame/domain STRINGS; the closed
    /// table is enforced by [`validate_judgement`] at capture, not by parse.
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

    /// D1: version 1 is rejected with a message naming the remedy — v1 was
    /// never released; the stray file is deleted or recreated, never migrated.
    #[test]
    fn parse_rejects_version_1_naming_remedy() {
        let mut s = full_session();
        s.version = 1;
        let text = to_toml(&s).unwrap();
        let err = parse(&text).unwrap_err().to_string();
        assert!(err.contains("never released"), "remedy named: {err}");
        assert!(err.contains("delete or recreate"), "remedy named: {err}");
    }

    /// D1: ANY version ≠ 2 is a parse error, not only v1.
    #[test]
    fn parse_rejects_any_non_v2_version() {
        let mut s = full_session();
        s.version = 3;
        let text = to_toml(&s).unwrap();
        let err = parse(&text).unwrap_err().to_string();
        assert!(err.contains("version 3"), "err names the version: {err}");
    }

    #[test]
    fn validate_accepts_value_equal_effort() {
        assert!(validate_judgement(&full_judgement()).is_ok());
    }

    #[test]
    fn validate_accepts_priority_prefer_first() {
        assert!(validate_judgement(&bare_judgement()).is_ok());
    }

    /// D2: the frame table is per-domain — each frame is inadmissible in the
    /// other domain (VT-1 frame admissibility, both directions).
    #[test]
    fn validate_rejects_cross_domain_frames() {
        let mut j = full_judgement();
        j.frame = FRAME_PREFER_FIRST.to_string(); // domain stays `value`
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("not admissible"), "got: {err}");

        let mut j = bare_judgement();
        j.frame = FRAME_EQUAL_EFFORT.to_string(); // domain stays `priority`
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("not admissible"), "got: {err}");
    }

    #[test]
    fn validate_rejects_self_pair() {
        let mut j = full_judgement();
        j.b = j.a.clone();
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

    /// Pins the admit set to the `kinds::` constants: it IS `VALUE_BEARING`
    /// minus `RSK` — for every census kind, admission holds exactly when the
    /// derivation says so.
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
