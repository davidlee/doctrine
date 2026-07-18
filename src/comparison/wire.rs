// SPDX-License-Identifier: GPL-3.0-only
//! `comparison::wire` — the comparison-session wire model, schema v3
//! (SL-220 §1: anchor rows join the pairwise v2 wire of SL-213 PHASE-01;
//! v1 retired in place per design D1 — verified zero exposure, no release
//! ever shipped it). Parse accepts {2, 3} and writes 3 (SL-220 D2: every v2
//! file is a valid v3 document).
//!
//! Pure leaf tier (ADR-001): depends only on `crate::kinds` and
//! `crate::value` (the value-domain anchor payload mirrors
//! [`crate::value::validate`] exactly — single source, SL-220 §1) plus
//! serde/toml. No clock, disk, rng, or git — dates and uids are function
//! inputs; the command shell mints them.
//!
//! The serde model IS the wire model: it serializes 1:1 to the documented
//! session-file schema — top-level `schema`/`version`, a nested `[session]`
//! table, singular `[[judgement]]` / `[[tombstone]]` arrays-of-tables,
//! lowercase/kebab-case enum tokens. `frame` and `domain` stay `String`-typed
//! so unknown vocab in *future* files round-trips losslessly; `response`,
//! `rater`, `form` and `admission` are closed enums by design — an unknown
//! token fails parse.

use serde::{Deserialize, Serialize};

use crate::kinds;
use crate::value;

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
/// The estimate domain (SL-219 D1): settle-cost testimony — rows answer
/// "which is more work?" and compile (a later phase) to cost order
/// `c_winner > c_loser`, the same winner-is-greater convention as the value
/// domain, with cost as the currency.
pub(crate) const DOMAIN_ESTIMATE: &str = "estimate";
/// Value frame: "equal effort assumed" — the default framing.
pub(crate) const FRAME_EQUAL_EFFORT: &str = "equal-effort";
/// Priority frame: "under a binding capacity cutoff, which do you keep?".
pub(crate) const FRAME_PREFER_FIRST: &str = "prefer-first";
/// Estimate frame (SL-219 D5): "which is more work?" — the winner is the
/// COSTLIER item. `prefer-a` ⇒ edge `c_A > c_B`; `equal` ⇒ cost-equality;
/// `incomparable` ⇒ no constraint.
pub(crate) const FRAME_MORE_WORK: &str = "more-work";
/// Value anchor frame (SL-220 §1): an absolute magnitude claim on a single
/// subject. Never user-typed — `value set|pin` stamps it.
pub(crate) const FRAME_VALUE_ANCHOR: &str = "value-anchor";
/// Estimate anchor frame (SL-222 §1): a bounded settle-cost claim on a single
/// subject — carries `est_lower`/`est_upper` in place of `magnitude`. Never
/// user-typed — `estimate set|pin` stamps it.
pub(crate) const FRAME_COST_ANCHOR: &str = "cost-anchor";

/// Per-domain closed frame vocabulary (design D2). The frame implies the
/// domain at capture — users never type a domain; [`domain_for_frame`] is the
/// single derivation seam and this table its single source (STD-001).
pub(crate) const DOMAIN_FRAMES: &[(&str, &[&str])] = &[
    (DOMAIN_VALUE, &[FRAME_EQUAL_EFFORT, FRAME_VALUE_ANCHOR]),
    (DOMAIN_PRIORITY, &[FRAME_PREFER_FIRST]),
    (DOMAIN_ESTIMATE, &[FRAME_MORE_WORK, FRAME_COST_ANCHOR]),
];

/// The anchor-frame membership set (SL-220 §1): each domain's frame set names
/// AT MOST one member of this set, and `form = anchor ⇔ frame ∈ ANCHOR_FRAMES`
/// is the validation biconditional. Membership anchors on this set, never on
/// a name pattern (`cost-anchor` joins with Phase 2).
const ANCHOR_FRAMES: &[&str] = &[FRAME_VALUE_ANCHOR, FRAME_COST_ANCHOR];

/// The wire version this model WRITES (SL-220 D2). Parse accepts every member
/// of [`SUPPORTED_VERSIONS`] — every v2 file is a valid v3 document, so the
/// corpus is never rewritten.
pub(crate) const COMPARISON_VERSION: u32 = 3;
/// The versions parse accepts (SL-220 D2: a two-member set — v1 was never
/// released and stays a remedy-naming parse error).
pub(crate) const SUPPORTED_VERSIONS: &[u32] = &[2, 3];

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

/// The domain's single anchor frame, if it names one (SL-220 §1 / SL-222 §1).
/// `None` for domains with no anchor form yet (`priority`), which bars
/// `form = anchor` there entirely.
fn anchor_frame_for(domain: &str) -> Option<&'static str> {
    frames_for_domain(domain)?
        .iter()
        .copied()
        .find(|frame| ANCHOR_FRAMES.contains(frame))
}

/// Who rendered the judgement. Closed by design: an unknown rater token
/// fails parse (losslessness covers the frame/domain strings only).
/// `migrated` (SL-220 §1) marks a facet-import row — anchor form only, with
/// `observed_at` in place of `date` (the authorship date is honestly unknown).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RaterKind {
    Human,
    Agent,
    Migrated,
}

/// Row form. The verb exposes `order` only; `ratio` keeps capture lossless
/// for RFC-019 OQ-6. `anchor` (SL-220 §1) is a single-subject absolute claim
/// — `b`/`response` absent, per-domain payload columns instead. Closed:
/// unknown tokens fail parse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RowForm {
    Order,
    Ratio,
    Anchor,
}

/// How a claim row was admitted (SL-220 §1). Closed enum, sole variant `pin`:
/// mintable only by the gated `value pin` path (design §4) — admission is a
/// contract, not a free column (RV-275 F-5). Unknown tokens fail parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AdmissionKind {
    Pin,
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

/// One `[[judgement]]` row: a pairwise judgement (order/ratio) or a
/// single-subject anchor claim (SL-220 §1).
///
/// No `Eq`: `magnitude` is an `f64` column (parsed, uncompiled on pairwise
/// rows — RFC-019 OQ-6 stays open; pure order semantics per design D8 ignore
/// it). Field order is the v2 wire order with the v3 optionals appended, so a
/// v2-shaped row serializes byte-identically (SL-220 D2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Judgement {
    pub uid: String,
    /// Row sequence within the file; ordering key is
    /// `(ordering_date, session_uid, seq)`.
    pub seq: u32,
    pub a: String,
    /// The pair's second side — required for order/ratio, absent for anchor
    /// rows (SL-220 §1: an anchor claims a single subject).
    pub b: Option<String>,
    /// Required for order/ratio, absent for anchor rows.
    pub response: Option<Response>,
    pub domain: String,
    pub frame: String,
    pub form: RowForm,
    /// Ratio column on pairwise rows — carried losslessly, never compiled
    /// (design C1). On a value-domain ANCHOR row: the payload — the claimed
    /// absolute magnitude (SL-220 D1).
    pub magnitude: Option<f64>,
    /// Estimate anchor payload: lower bound (SL-222 §1). Present on
    /// cost-anchor rows; absent on all other forms/domains.
    pub est_lower: Option<f64>,
    /// Estimate anchor payload: upper bound (SL-222 §1). Present on
    /// cost-anchor rows; absent on all other forms/domains.
    pub est_upper: Option<f64>,
    /// Explicit supersession target: this row's uid replaces that row's
    /// testimony (design R2 — a durable act, not testimony).
    pub supersedes: Option<String>,
    /// Optional value lens — the IDE-035 seam.
    pub lens: Option<String>,
    pub rater: RaterKind,
    /// Optional rater identity.
    pub by: Option<String>,
    pub note: Option<String>,
    /// Asserted-at date — required on every row EXCEPT `rater = migrated`
    /// (SL-220 §1: a migrated facet's authorship date is honestly absent).
    pub date: Option<String>,
    /// Present iff `rater = migrated` (strict biconditional): the migration
    /// date (SL-220 §1).
    pub observed_at: Option<String>,
    /// Free-text evidence citation — e.g. `REQ-059`, or the source facet's
    /// provenance on migrated rows (SL-220 §1).
    pub basis: Option<String>,
    /// Pin admission (SL-220 §1) — see [`AdmissionKind`].
    pub admission: Option<AdmissionKind>,
}

impl Judgement {
    /// The total tier-1 ordering date (SL-220 §1): `date` when present, else
    /// `observed_at`. Total by the validation matrix — every capturable row
    /// carries exactly one of the two; a row that somehow carries neither
    /// still orders (deterministically first) rather than panicking.
    pub(crate) fn ordering_date(&self) -> &str {
        self.date
            .as_deref()
            .or(self.observed_at.as_deref())
            .unwrap_or_default()
    }
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
/// version outside [`SUPPORTED_VERSIONS`] with a remedy-naming message
/// (design D1, SL-220 D2); unknown frame/domain strings parse and round-trip.
pub(crate) fn parse(text: &str) -> anyhow::Result<ComparisonSession> {
    let s: ComparisonSession = toml::from_str(text)?;
    if s.schema != COMPARISON_SCHEMA {
        anyhow::bail!(
            "unrecognized comparison-session schema `{}` (expected `{COMPARISON_SCHEMA}`)",
            s.schema
        );
    }
    if !SUPPORTED_VERSIONS.contains(&s.version) {
        let supported: Vec<String> = SUPPORTED_VERSIONS.iter().map(u32::to_string).collect();
        anyhow::bail!(
            "unsupported comparison-session version {} (expected one of: {}) — \
             schema version 1 was never released; delete or recreate this session file",
            s.version,
            supported.join(", ")
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

/// Structural row validation (capture-time — parse stays lossless): non-empty
/// refs, closed per-domain frame vocabulary (design D2 — the frame table is
/// normative at capture), and the SL-220 §1 validation matrix — form/payload
/// exactness ([`validate_form`]) and rater/date/admission provenance
/// ([`validate_provenance`]). `response` values need no check — the closed
/// enum makes an invalid answer unrepresentable. Admissibility is separate
/// (needs kinds).
pub(crate) fn validate_judgement(j: &Judgement) -> anyhow::Result<()> {
    if j.a.is_empty() {
        anyhow::bail!("the subject ref is required — empty ref");
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
    validate_form(j)?;
    validate_provenance(j)
}

/// The form half of the SL-220 §1 matrix: `form = anchor` ⇔ the domain's
/// single anchor frame, with `b`/`response` absent and the domain's payload
/// set present EXACTLY; order/ratio rows require `b`/`response` and a
/// pairwise frame (v2 rows satisfy this by construction).
fn validate_form(j: &Judgement) -> anyhow::Result<()> {
    let anchor_frame = anchor_frame_for(&j.domain);
    match j.form {
        RowForm::Anchor => {
            if anchor_frame != Some(j.frame.as_str()) {
                let expected = anchor_frame.map_or_else(
                    || format!(" (domain `{}` has none)", j.domain),
                    |f| format!(" (`{f}`)"),
                );
                anyhow::bail!(
                    "form `anchor` requires the domain's anchor frame{expected} — got `{}`",
                    j.frame
                );
            }
            if j.b.is_some() {
                anyhow::bail!("an anchor row claims a single subject — `b` must be absent");
            }
            if j.response.is_some() {
                anyhow::bail!(
                    "an anchor row carries no pairwise response — `response` must be absent"
                );
            }
            validate_anchor_payload(j)
        }
        RowForm::Order | RowForm::Ratio => {
            if anchor_frame == Some(j.frame.as_str()) {
                anyhow::bail!(
                    "frame `{}` is the `{}` domain's anchor frame — pairwise rows use a \
                     pairwise frame",
                    j.frame,
                    j.domain
                );
            }
            if j.est_lower.is_some() || j.est_upper.is_some() {
                anyhow::bail!(
                    "a pairwise row carries no estimate payload — `est_lower`/`est_upper` \
                     must be absent"
                );
            }
            let Some(b) = j.b.as_deref() else {
                anyhow::bail!("both sides of the pair are required — `b` is absent");
            };
            if b.is_empty() {
                anyhow::bail!("both sides of the pair are required — empty ref");
            }
            if j.a == b {
                anyhow::bail!("cannot compare `{}` against itself", j.a);
            }
            if j.response.is_none() {
                anyhow::bail!("a pairwise row requires a `response`");
            }
            Ok(())
        }
    }
}

/// The per-domain anchor payload, EXACTLY (SL-220 D1 / SL-222 §1). Value:
/// `{magnitude}` — present and finite, mirroring [`value::validate`] (the
/// single source — no range policy smuggled in; negatives included); the
/// estimate payload fields MUST be absent (payload exactness cuts both ways).
/// Estimate: `{est_lower, est_upper}` — both present, finite, lower >= 0,
/// upper >= lower, mirroring [`estimate::validate`] (the single source, never
/// duplicated); `magnitude` MUST be absent.
fn validate_anchor_payload(j: &Judgement) -> anyhow::Result<()> {
    match j.domain.as_str() {
        DOMAIN_VALUE => {
            if j.est_lower.is_some() || j.est_upper.is_some() {
                anyhow::bail!(
                    "a value anchor carries no estimate payload — `est_lower`/`est_upper` \
                     must be absent"
                );
            }
            let Some(magnitude) = j.magnitude else {
                anyhow::bail!("a value anchor claims a magnitude — `magnitude` is required");
            };
            value::validate(&value::ValueFacet { value: magnitude })
        }
        DOMAIN_ESTIMATE => {
            if j.magnitude.is_some() {
                anyhow::bail!("a cost anchor carries no magnitude — `magnitude` must be absent");
            }
            let Some(lower) = j.est_lower else {
                anyhow::bail!("a cost anchor claims a lower bound — `est_lower` is required");
            };
            let Some(upper) = j.est_upper else {
                anyhow::bail!("a cost anchor claims an upper bound — `est_upper` is required");
            };
            let facet = crate::estimate::EstimateFacet { lower, upper };
            crate::estimate::validate(&facet)
        }
        _ => {
            // The frame biconditional bars anchor rows from domains without
            // an anchor frame, so this arm is unreachable for valid rows.
            anyhow::bail!(
                "anchor payload validation not implemented for domain `{}`",
                j.domain
            );
        }
    }
}

/// The provenance half of the SL-220 §1 matrix: `rater = migrated` ⇒ anchor
/// form, `date` absent, `observed_at` present; any other rater ⇒ `date`
/// present, `observed_at` absent (strict — every row carries exactly one of
/// the two). `admission = pin` ⇒ human anchor (RV-275 F-5: contradictory
/// provenance rejected at capture).
fn validate_provenance(j: &Judgement) -> anyhow::Result<()> {
    match j.rater {
        RaterKind::Migrated => {
            if !matches!(j.form, RowForm::Anchor) {
                anyhow::bail!("rater `migrated` is facet-import provenance — anchor rows only");
            }
            if j.date.is_some() {
                anyhow::bail!(
                    "a migrated row's authorship date is unknown — `date` must be absent \
                     (`observed_at` carries the migration date)"
                );
            }
            if j.observed_at.is_none() {
                anyhow::bail!(
                    "a migrated row records its migration date — `observed_at` is required"
                );
            }
        }
        RaterKind::Human | RaterKind::Agent => {
            if j.date.is_none() {
                anyhow::bail!("`date` is required (absent only on `rater = migrated` rows)");
            }
            if j.observed_at.is_some() {
                anyhow::bail!(
                    "`observed_at` is migration provenance — `rater = migrated` rows only"
                );
            }
        }
    }
    // Sole variant `pin` (closed enum): presence IS the pin claim.
    if j.admission.is_some()
        && !(matches!(j.form, RowForm::Anchor) && matches!(j.rater, RaterKind::Human))
    {
        anyhow::bail!(
            "`admission = \"pin\"` requires a human anchor row — contradictory provenance"
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

/// Estimate-pair admissibility (SL-219 D3): the admit set is the UNION of
/// `kinds::VALUE_BEARING` and `kinds::RECORD` — derived from those constants,
/// never a parallel hand-list. RSK is admitted, unlike the value domain:
/// settle-cost is comparable even where value is not. `Err` carries the
/// human-readable refusal naming the currency.
pub(crate) fn admissible_estimate_pair(kind_a: &str, kind_b: &str) -> Result<(), String> {
    admissible_estimate_kind(kind_a)?;
    admissible_estimate_kind(kind_b)
}

/// One side of the pair: a work (value-bearing) or knowledge-record kind —
/// everything that carries a settle-cost worth sizing.
fn admissible_estimate_kind(kind: &str) -> Result<(), String> {
    if kinds::VALUE_BEARING.contains(&kind) || kinds::RECORD.contains(&kind) {
        return Ok(());
    }
    Err(format!(
        "{kind} has no comparable settle-cost — estimate comparison admits work and record kinds"
    ))
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

/// Anchor supersession scope (SL-220 §4): an anchor claim may supersede only
/// another ANCHOR claim sharing its `(subject, domain, lens)` — with
/// `None == None` lens equality. A foreign subject, cross-domain, pairwise-row,
/// or cross-lens target is refused AT CAPTURE (never deferred to a
/// resolution-time finding). Pure over the two wire rows; the shell resolves
/// the target uid against the corpus and passes the row in. `Err` carries the
/// human-readable refusal naming the mismatch.
pub(crate) fn validate_anchor_supersedes(
    new: &Judgement,
    target: &Judgement,
) -> Result<(), String> {
    if !matches!(target.form, RowForm::Anchor) {
        return Err(format!(
            "--supersedes target `{}` is a pairwise row — an anchor claim supersedes only another anchor claim",
            target.uid
        ));
    }
    if new.a != target.a {
        return Err(format!(
            "--supersedes target `{}` claims `{}`, not `{}` — supersession stays within one subject",
            target.uid, target.a, new.a
        ));
    }
    if new.domain != target.domain {
        return Err(format!(
            "--supersedes target `{}` is domain `{}`, not `{}` — cross-domain supersession is refused",
            target.uid, target.domain, new.domain
        ));
    }
    if new.lens != target.lens {
        let show = |l: &Option<String>| l.clone().unwrap_or_else(|| "(none)".to_string());
        return Err(format!(
            "--supersedes target `{}` lens `{}` differs from `{}` — cross-lens supersession is refused",
            target.uid,
            show(&target.lens),
            show(&new.lens)
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

    /// A judgement with EVERY v2 optional set (magnitude, supersedes, lens,
    /// by, note) — the golden's maximal pairwise row.
    fn full_judgement() -> Judgement {
        Judgement {
            uid: "0197f3a2-6c2f-7d4e-8f5a-2b3c4d5e6f7a".to_string(),
            seq: 0,
            a: "SL-204".to_string(),
            b: Some("IMP-118".to_string()),
            response: Some(Response::PreferA),
            domain: DOMAIN_VALUE.to_string(),
            frame: FRAME_EQUAL_EFFORT.to_string(),
            form: RowForm::Order,
            magnitude: Some(2.5),
            est_lower: None,
            est_upper: None,
            supersedes: Some("0197f3a1-1111-7abc-8def-0a1b2c3d4e5f".to_string()),
            lens: Some("user-value".to_string()),
            rater: RaterKind::Agent,
            by: Some("david".to_string()),
            note: Some("auth unblocks the pilot".to_string()),
            date: Some("2026-07-10".to_string()),
            observed_at: None,
            basis: None,
            admission: None,
        }
    }

    /// An estimate-domain judgement (SL-219 D1/D5): `more-work` frame, order
    /// form — the winner is the costlier item.
    fn estimate_judgement() -> Judgement {
        Judgement {
            uid: "0197f3a2-8f52-7a71-9c8d-5e6f7a8b9c0d".to_string(),
            seq: 2,
            a: "SL-204".to_string(),
            b: Some("RSK-004".to_string()),
            response: Some(Response::PreferB),
            domain: DOMAIN_ESTIMATE.to_string(),
            frame: FRAME_MORE_WORK.to_string(),
            form: RowForm::Order,
            magnitude: None,
            est_lower: None,
            est_upper: None,
            supersedes: None,
            lens: None,
            rater: RaterKind::Human,
            by: None,
            note: None,
            date: Some("2026-07-10".to_string()),
            observed_at: None,
            basis: None,
            admission: None,
        }
    }

    /// A judgement with every optional absent.
    fn bare_judgement() -> Judgement {
        Judgement {
            uid: "0197f3a2-7e41-7f60-8b7c-4d5e6f7a8b9c".to_string(),
            seq: 1,
            a: "IMP-118".to_string(),
            b: Some("CHR-042".to_string()),
            response: Some(Response::PreferB),
            domain: DOMAIN_PRIORITY.to_string(),
            frame: FRAME_PREFER_FIRST.to_string(),
            form: RowForm::Order,
            magnitude: None,
            est_lower: None,
            est_upper: None,
            supersedes: None,
            lens: None,
            rater: RaterKind::Human,
            by: None,
            note: None,
            date: Some("2026-07-10".to_string()),
            observed_at: None,
            basis: None,
            admission: None,
        }
    }

    // ---- SL-220 §1 anchor-row fixtures (design's sample rows) ---------------

    /// The design's first sample row: a live human anchor claim, `basis` set.
    fn human_anchor() -> Judgement {
        Judgement {
            uid: "0197f3a2-9a63-7b82-8d9e-6f7a8b9c0d1e".to_string(),
            seq: 0,
            a: "SL-204".to_string(),
            b: None,
            response: None,
            domain: DOMAIN_VALUE.to_string(),
            frame: FRAME_VALUE_ANCHOR.to_string(),
            form: RowForm::Anchor,
            magnitude: Some(6.5),
            est_lower: None,
            est_upper: None,
            supersedes: None,
            lens: None,
            rater: RaterKind::Human,
            by: Some("david".to_string()),
            note: None,
            date: Some("2026-07-16".to_string()),
            observed_at: None,
            basis: Some("REQ-059".to_string()),
            admission: None,
        }
    }

    /// A pin: the human anchor plus `admission = "pin"` (design §1).
    fn pin_anchor() -> Judgement {
        Judgement {
            admission: Some(AdmissionKind::Pin),
            ..human_anchor()
        }
    }

    // ---- SL-222 §1 estimate anchor-row fixtures (design's sample rows) ------

    /// The design's live-human estimate anchor sample (SL-222 §1):
    /// `cost-anchor` frame, `est_lower`/`est_upper` in place of `magnitude`.
    fn human_estimate_anchor() -> Judgement {
        Judgement {
            uid: "0197f3a2-bc85-7da4-9fb0-8b9c0d1e2f3a".to_string(),
            seq: 0,
            a: "SL-221".to_string(),
            b: None,
            response: None,
            domain: DOMAIN_ESTIMATE.to_string(),
            frame: FRAME_COST_ANCHOR.to_string(),
            form: RowForm::Anchor,
            magnitude: None,
            est_lower: Some(2.0),
            est_upper: Some(8.0),
            supersedes: None,
            lens: None,
            rater: RaterKind::Human,
            by: Some("david".to_string()),
            note: None,
            date: Some("2026-07-17".to_string()),
            observed_at: None,
            basis: Some("ASM-014; assumes v3 wire reuse".to_string()),
            admission: None,
        }
    }

    /// A pin estimate anchor: `admission = "pin"` on an estimate anchor.
    fn pin_estimate_anchor() -> Judgement {
        Judgement {
            admission: Some(AdmissionKind::Pin),
            ..human_estimate_anchor()
        }
    }

    /// A migrated estimate anchor: `rater = migrated`, `observed_at` in
    /// place of `date`.
    fn migrated_estimate_anchor() -> Judgement {
        Judgement {
            uid: "0197f3a2-cd96-7eb5-9fc1-9b0c1d2e3f4b".to_string(),
            seq: 1,
            a: "IMP-118".to_string(),
            b: None,
            response: None,
            domain: DOMAIN_ESTIMATE.to_string(),
            frame: FRAME_COST_ANCHOR.to_string(),
            form: RowForm::Anchor,
            magnitude: None,
            est_lower: Some(1.0),
            est_upper: Some(3.0),
            supersedes: None,
            lens: None,
            rater: RaterKind::Migrated,
            by: None,
            note: None,
            date: None,
            observed_at: Some("2026-07-17".to_string()),
            basis: Some(
                "facet [estimate] .doctrine/backlog/imp-118.toml @ 9c01d2aa david 2026-06-28"
                    .to_string(),
            ),
            admission: None,
        }
    }

    /// A point estimate: lower == upper, the `-x` point form (SL-222 §1).
    fn point_estimate_anchor() -> Judgement {
        Judgement {
            uid: "0197f3a2-dea7-7fc6-9fd2-ac1d2e3f5c6d".to_string(),
            seq: 2,
            a: "PLAN-001".to_string(),
            b: None,
            response: None,
            domain: DOMAIN_ESTIMATE.to_string(),
            frame: FRAME_COST_ANCHOR.to_string(),
            form: RowForm::Anchor,
            magnitude: None,
            est_lower: Some(5.0),
            est_upper: Some(5.0),
            supersedes: None,
            lens: None,
            rater: RaterKind::Human,
            by: None,
            note: None,
            date: Some("2026-07-17".to_string()),
            observed_at: None,
            basis: None,
            admission: None,
        }
    }

    /// The design's second sample row: a migrated facet import —
    /// `observed_at` in place of `date`, provenance in `basis`.
    fn migrated_anchor() -> Judgement {
        Judgement {
            uid: "0197f3a2-ab74-7c93-9eaf-7a8b9c0d1e2f".to_string(),
            seq: 1,
            a: "IMP-118".to_string(),
            b: None,
            response: None,
            domain: DOMAIN_VALUE.to_string(),
            frame: FRAME_VALUE_ANCHOR.to_string(),
            form: RowForm::Anchor,
            magnitude: Some(3.0),
            est_lower: None,
            est_upper: None,
            supersedes: None,
            lens: None,
            rater: RaterKind::Migrated,
            by: None,
            note: None,
            date: None,
            observed_at: Some("2026-07-16".to_string()),
            basis: Some("facet [value] .doctrine/backlog/imp-118.toml @ 4a12e576".to_string()),
            admission: None,
        }
    }

    /// Pins the vocab constants to the documented schema strings (style
    /// precedent: `kinds::tests::groupings_match_documented_membership`).
    /// SL-220 §1: version 3 written, `value-anchor` in the value frame set.
    /// SL-222 §1: `cost-anchor` joins the estimate frame set and anchor set.
    #[test]
    fn vocab_matches_documented_schema() {
        assert_eq!(COMPARISON_SCHEMA, "doctrine.comparison-session");
        assert_eq!(COMPARISONS_DIR, "comparisons");
        assert_eq!(COMPARISON_VERSION, 3);
        assert_eq!(SUPPORTED_VERSIONS, &[2, 3]);
        assert_eq!(DOMAIN_VALUE, "value");
        assert_eq!(DOMAIN_PRIORITY, "priority");
        assert_eq!(DOMAIN_ESTIMATE, "estimate");
        assert_eq!(FRAME_MORE_WORK, "more-work");
        assert_eq!(FRAME_VALUE_ANCHOR, "value-anchor");
        assert_eq!(FRAME_COST_ANCHOR, "cost-anchor");
        assert_eq!(
            DOMAIN_FRAMES,
            &[
                ("value", &["equal-effort", "value-anchor"][..]),
                ("priority", &["prefer-first"][..]),
                ("estimate", &["more-work", "cost-anchor"][..])
            ]
        );
    }

    /// The per-domain frame table drives capture derivation both ways
    /// (design D2/S1): frame → domain is total over the closed vocab, and an
    /// unknown frame derives nothing. SL-219 D1: `more-work` derives the
    /// estimate domain. SL-220 §1: `value-anchor` derives value —
    /// `domain_for_frame` stays total over the grown table.
    /// SL-222 §1: `cost-anchor` derives the estimate domain.
    #[test]
    fn domain_for_frame_derives_from_the_table() {
        assert_eq!(domain_for_frame(FRAME_EQUAL_EFFORT), Some(DOMAIN_VALUE));
        assert_eq!(domain_for_frame(FRAME_PREFER_FIRST), Some(DOMAIN_PRIORITY));
        assert_eq!(domain_for_frame(FRAME_MORE_WORK), Some(DOMAIN_ESTIMATE));
        assert_eq!(domain_for_frame(FRAME_VALUE_ANCHOR), Some(DOMAIN_VALUE));
        assert_eq!(domain_for_frame(FRAME_COST_ANCHOR), Some(DOMAIN_ESTIMATE));
        assert_eq!(domain_for_frame("opportunity-cost"), None);
    }

    /// SL-220 §1 / SL-222 §1: the anchor-frame derivation is membership in
    /// `ANCHOR_FRAMES`, per domain — value names `value-anchor`; estimate
    /// names `cost-anchor`; priority (no anchor yet) names none; unknown
    /// domains none.
    #[test]
    fn anchor_frame_for_derives_from_the_membership_set() {
        assert_eq!(anchor_frame_for(DOMAIN_VALUE), Some(FRAME_VALUE_ANCHOR));
        assert_eq!(anchor_frame_for(DOMAIN_PRIORITY), None);
        assert_eq!(anchor_frame_for(DOMAIN_ESTIMATE), Some(FRAME_COST_ANCHOR));
        assert_eq!(anchor_frame_for("effort"), None);
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
            s.judgements[0].response = Some(response);
            let text = to_toml(&s).unwrap();
            assert!(
                text.contains(&format!("response = \"{token}\"")),
                "{token} on the wire:\n{text}"
            );
            assert_eq!(parse(&text).unwrap().judgements[0].response, Some(response));
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

    /// D1/SL-220 D2: ANY version outside the supported set {2, 3} is a parse
    /// error, not only v1. (Designed churn at SL-220: this test previously
    /// pinned v3 as rejected; the D2 version bump admits it.)
    #[test]
    fn parse_rejects_any_unsupported_version() {
        let mut s = full_session();
        s.version = 4;
        let text = to_toml(&s).unwrap();
        let err = parse(&text).unwrap_err().to_string();
        assert!(err.contains("version 4"), "err names the version: {err}");
        assert!(err.contains("2, 3"), "err names the supported set: {err}");
    }

    /// SL-220 D2: parse accepts BOTH members of the supported set — every v2
    /// file is a valid v3 document, no corpus rewrite.
    #[test]
    fn parse_accepts_v2_and_v3() {
        for version in [2, 3] {
            let mut s = full_session();
            s.version = version;
            let text = to_toml(&s).unwrap();
            assert_eq!(parse(&text).unwrap().version, version);
        }
    }

    /// SL-220 D2: minting writes the CURRENT version (3) — the shell never
    /// hand-sets the wire discriminators.
    #[test]
    fn session_of_one_stamps_version_3() {
        let s = session_of_one(full_session().session, human_anchor());
        assert_eq!(s.version, 3);
        assert!(to_toml(&s).unwrap().contains("version = 3\n"));
    }

    #[test]
    fn validate_accepts_value_equal_effort() {
        assert!(validate_judgement(&full_judgement()).is_ok());
    }

    #[test]
    fn validate_accepts_priority_prefer_first() {
        assert!(validate_judgement(&bare_judgement()).is_ok());
    }

    #[test]
    fn validate_accepts_estimate_more_work() {
        assert!(validate_judgement(&estimate_judgement()).is_ok());
    }

    /// SL-219 D1: `more-work` is estimate-only, and the estimate domain admits
    /// no other frame — the closed table scopes per domain, both directions.
    #[test]
    fn validate_rejects_more_work_outside_estimate_domain() {
        let mut j = full_judgement();
        j.frame = FRAME_MORE_WORK.to_string(); // domain stays `value`
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("not admissible"), "got: {err}");

        let mut j = estimate_judgement();
        j.frame = FRAME_EQUAL_EFFORT.to_string(); // domain stays `estimate`
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("not admissible"), "got: {err}");
    }

    /// SL-219 VT-1: an estimate-domain row rides the v2 wire losslessly —
    /// the documented tokens on the wire, and parse(to_toml(s)) == s (the
    /// existing round-trip idiom extended with an est-domain row).
    #[test]
    fn round_trip_preserves_estimate_domain_rows() {
        let mut s = full_session();
        s.judgements.push(estimate_judgement());
        let text = to_toml(&s).unwrap();
        assert!(
            text.contains(&format!("domain = \"{DOMAIN_ESTIMATE}\"")),
            "estimate domain on the wire:\n{text}"
        );
        assert!(
            text.contains(&format!("frame = \"{FRAME_MORE_WORK}\"")),
            "more-work frame on the wire:\n{text}"
        );
        assert_eq!(parse(&text).unwrap(), s);
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
        j.b = Some(j.a.clone());
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
        j.b = Some(String::new());
        assert!(validate_judgement(&j).is_err());
    }

    // ---- SL-220 §1: wire v3 battery (VT-1) -----------------------------------

    /// Byte-exact v3 anchor wire shape: the design's sample rows — a live
    /// human anchor (with `basis`), a pin (`admission = "pin"`), a migrated
    /// import (`observed_at` in place of `date`) — written at version 3.
    #[test]
    fn golden_shape_v3_anchor_rows() {
        let s = ComparisonSession {
            schema: COMPARISON_SCHEMA.to_string(),
            version: COMPARISON_VERSION,
            session: SessionHeader {
                uid: "0197f3a2-5b1e-7c3d-9e4f-1a2b3c4d5e6f".to_string(),
                date: "2026-07-16".to_string(),
                audience: None,
            },
            judgements: vec![human_anchor(), migrated_anchor(), pin_anchor()],
            tombstones: vec![],
        };
        let expected = "\
schema = \"doctrine.comparison-session\"
version = 3
tombstone = []

[session]
uid = \"0197f3a2-5b1e-7c3d-9e4f-1a2b3c4d5e6f\"
date = \"2026-07-16\"

[[judgement]]
uid = \"0197f3a2-9a63-7b82-8d9e-6f7a8b9c0d1e\"
seq = 0
a = \"SL-204\"
domain = \"value\"
frame = \"value-anchor\"
form = \"anchor\"
magnitude = 6.5
rater = \"human\"
by = \"david\"
date = \"2026-07-16\"
basis = \"REQ-059\"

[[judgement]]
uid = \"0197f3a2-ab74-7c93-9eaf-7a8b9c0d1e2f\"
seq = 1
a = \"IMP-118\"
domain = \"value\"
frame = \"value-anchor\"
form = \"anchor\"
magnitude = 3.0
rater = \"migrated\"
observed_at = \"2026-07-16\"
basis = \"facet [value] .doctrine/backlog/imp-118.toml @ 4a12e576\"

[[judgement]]
uid = \"0197f3a2-9a63-7b82-8d9e-6f7a8b9c0d1e\"
seq = 0
a = \"SL-204\"
domain = \"value\"
frame = \"value-anchor\"
form = \"anchor\"
magnitude = 6.5
rater = \"human\"
by = \"david\"
date = \"2026-07-16\"
basis = \"REQ-059\"
admission = \"pin\"
";
        assert_eq!(to_toml(&s).unwrap(), expected);
    }

    /// Losslessness over the v3 optionals: anchor rows (human, pin, migrated
    /// — `observed_at`/`basis`/`admission` set) alongside maximal and bare
    /// pairwise rows all survive parse(to_toml(s)) == s.
    #[test]
    fn round_trip_preserves_anchor_rows_and_new_optionals() {
        let mut s = full_session();
        s.judgements.push(bare_judgement());
        s.judgements.push(human_anchor());
        s.judgements.push(pin_anchor());
        s.judgements.push(migrated_anchor());
        let text = to_toml(&s).unwrap();
        assert_eq!(parse(&text).unwrap(), s);
    }

    /// An unknown admission token is a parse error — the enum is closed
    /// (`pin` is its sole variant).
    #[test]
    fn parse_rejects_unknown_admission_token() {
        let mut s = full_session();
        s.judgements = vec![pin_anchor()];
        let text = to_toml(&s)
            .unwrap()
            .replace("admission = \"pin\"", "admission = \"vouched\"");
        assert!(parse(&text).is_err());
    }

    /// An unknown rater token is a parse error — `migrated` joined the closed
    /// enum; arbitrary strings did not.
    #[test]
    fn parse_rejects_unknown_rater_token() {
        let mut s = full_session();
        s.judgements = vec![migrated_anchor()];
        let text = to_toml(&s)
            .unwrap()
            .replace("rater = \"migrated\"", "rater = \"imported\"");
        assert!(parse(&text).is_err());
    }

    // The validation matrix, every rule in BOTH directions (§8.1).

    #[test]
    fn validate_accepts_live_human_anchor() {
        assert!(validate_judgement(&human_anchor()).is_ok());
    }

    #[test]
    fn validate_accepts_pin() {
        assert!(validate_judgement(&pin_anchor()).is_ok());
    }

    #[test]
    fn validate_accepts_migrated_anchor() {
        assert!(validate_judgement(&migrated_anchor()).is_ok());
    }

    /// form=anchor ⇒ `b` absent — a present `b` is rejected naming the rule.
    #[test]
    fn validate_rejects_anchor_with_b() {
        let mut j = human_anchor();
        j.b = Some("IMP-118".to_string());
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("single subject"), "got: {err}");
    }

    /// form=anchor ⇒ `response` absent.
    #[test]
    fn validate_rejects_anchor_with_response() {
        let mut j = human_anchor();
        j.response = Some(Response::PreferA);
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("no pairwise response"), "got: {err}");
    }

    /// Value payload exactness: `magnitude` present…
    #[test]
    fn validate_rejects_anchor_without_magnitude() {
        let mut j = human_anchor();
        j.magnitude = None;
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("magnitude"), "got: {err}");
    }

    /// …and finite, mirroring `value::validate` exactly: NaN/±Inf rejected,
    /// negatives ACCEPTED (no range policy smuggled in).
    #[test]
    fn validate_anchor_magnitude_mirrors_value_validate() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut j = human_anchor();
            j.magnitude = Some(bad);
            let err = validate_judgement(&j).unwrap_err().to_string();
            assert!(
                err.contains("finite"),
                "{bad} rejected naming finite: {err}"
            );
        }
        let mut j = human_anchor();
        j.magnitude = Some(-2.5);
        assert!(validate_judgement(&j).is_ok(), "negatives are admissible");
    }

    /// form=anchor ⇔ the domain's anchor frame: a pairwise frame on an anchor
    /// row is rejected…
    #[test]
    fn validate_rejects_anchor_with_pairwise_frame() {
        let mut j = human_anchor();
        j.frame = FRAME_EQUAL_EFFORT.to_string();
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("anchor frame"), "got: {err}");
    }

    /// …an anchor frame on an order/ratio row is rejected…
    #[test]
    fn validate_rejects_order_row_with_anchor_frame() {
        for form in [RowForm::Order, RowForm::Ratio] {
            let mut j = full_judgement();
            j.frame = FRAME_VALUE_ANCHOR.to_string();
            j.form = form;
            let err = validate_judgement(&j).unwrap_err().to_string();
            assert!(err.contains("pairwise"), "got: {err}");
        }
    }

    /// …and a domain with NO anchor frame admits no anchor row at all.
    #[test]
    fn validate_rejects_anchor_in_domain_without_anchor_frame() {
        let mut j = human_anchor();
        j.domain = DOMAIN_PRIORITY.to_string();
        j.frame = FRAME_PREFER_FIRST.to_string();
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("has none"), "got: {err}");
    }

    /// form=order|ratio ⇒ `b` present ∧ `response` present, both directions
    /// of the presence rule (the accept direction is every v2 validate test).
    #[test]
    fn validate_rejects_pairwise_row_missing_b_or_response() {
        let mut j = full_judgement();
        j.b = None;
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("`b` is absent"), "got: {err}");

        let mut j = full_judgement();
        j.response = None;
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("requires a `response`"), "got: {err}");
    }

    /// rater=migrated ⇒ form=anchor: a migrated pairwise row is contradictory.
    #[test]
    fn validate_rejects_migrated_on_pairwise_row() {
        let mut j = full_judgement();
        j.rater = RaterKind::Migrated;
        j.date = None;
        j.observed_at = Some("2026-07-16".to_string());
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("anchor rows only"), "got: {err}");
    }

    /// The date/observed_at strict biconditional, all four violations:
    /// migrated with `date`, migrated without `observed_at`, live with
    /// `observed_at`, live without `date`.
    #[test]
    fn validate_enforces_date_observed_at_biconditional() {
        let mut j = migrated_anchor();
        j.date = Some("2026-07-16".to_string());
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("`date` must be absent"), "got: {err}");

        let mut j = migrated_anchor();
        j.observed_at = None;
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("`observed_at` is required"), "got: {err}");

        let mut j = human_anchor();
        j.observed_at = Some("2026-07-16".to_string());
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("migration provenance"), "got: {err}");

        let mut j = human_anchor();
        j.date = None;
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("`date` is required"), "got: {err}");

        // The live-row rules hold on pairwise rows too (every row carries
        // exactly one of the two).
        let mut j = full_judgement();
        j.date = None;
        assert!(validate_judgement(&j).is_err());
        let mut j = full_judgement();
        j.observed_at = Some("2026-07-16".to_string());
        assert!(validate_judgement(&j).is_err());
    }

    /// admission=pin ⇒ form=anchor ∧ rater=human, both violations.
    #[test]
    fn validate_rejects_pin_without_human_anchor() {
        let mut j = pin_anchor();
        j.rater = RaterKind::Agent;
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("human anchor"), "got: {err}");

        let mut j = full_judgement();
        j.admission = Some(AdmissionKind::Pin);
        j.rater = RaterKind::Human;
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("human anchor"), "got: {err}");
    }

    /// An anchor row with an empty subject ref is rejected.
    #[test]
    fn validate_rejects_anchor_with_empty_subject() {
        let mut j = human_anchor();
        j.a = String::new();
        assert!(validate_judgement(&j).is_err());
    }

    // ---- SL-222 §1: estimate anchor payload (VT-1) --------------------------

    /// Goldens: the four canonical estimate anchor rows — live human,
    /// pin, migrated, and point estimate (lower == upper).
    #[test]
    fn golden_shape_estimate_anchor_rows() {
        let s = ComparisonSession {
            schema: COMPARISON_SCHEMA.to_string(),
            version: COMPARISON_VERSION,
            session: SessionHeader {
                uid: "0197f3a2-5b1e-7c3d-9e4f-1a2b3c4d5e6f".to_string(),
                date: "2026-07-17".to_string(),
                audience: None,
            },
            judgements: vec![
                human_estimate_anchor(),
                pin_estimate_anchor(),
                migrated_estimate_anchor(),
                point_estimate_anchor(),
            ],
            tombstones: vec![],
        };
        let expected = r#"schema = "doctrine.comparison-session"
version = 3
tombstone = []

[session]
uid = "0197f3a2-5b1e-7c3d-9e4f-1a2b3c4d5e6f"
date = "2026-07-17"

[[judgement]]
uid = "0197f3a2-bc85-7da4-9fb0-8b9c0d1e2f3a"
seq = 0
a = "SL-221"
domain = "estimate"
frame = "cost-anchor"
form = "anchor"
est_lower = 2.0
est_upper = 8.0
rater = "human"
by = "david"
date = "2026-07-17"
basis = "ASM-014; assumes v3 wire reuse"

[[judgement]]
uid = "0197f3a2-bc85-7da4-9fb0-8b9c0d1e2f3a"
seq = 0
a = "SL-221"
domain = "estimate"
frame = "cost-anchor"
form = "anchor"
est_lower = 2.0
est_upper = 8.0
rater = "human"
by = "david"
date = "2026-07-17"
basis = "ASM-014; assumes v3 wire reuse"
admission = "pin"

[[judgement]]
uid = "0197f3a2-cd96-7eb5-9fc1-9b0c1d2e3f4b"
seq = 1
a = "IMP-118"
domain = "estimate"
frame = "cost-anchor"
form = "anchor"
est_lower = 1.0
est_upper = 3.0
rater = "migrated"
observed_at = "2026-07-17"
basis = "facet [estimate] .doctrine/backlog/imp-118.toml @ 9c01d2aa david 2026-06-28"

[[judgement]]
uid = "0197f3a2-dea7-7fc6-9fd2-ac1d2e3f5c6d"
seq = 2
a = "PLAN-001"
domain = "estimate"
frame = "cost-anchor"
form = "anchor"
est_lower = 5.0
est_upper = 5.0
rater = "human"
date = "2026-07-17"
"#;
        assert_eq!(to_toml(&s).unwrap(), expected);
    }

    /// Round-trip losslessness over the new optional columns: estimate anchor
    /// rows (human, pin, migrated, point) alongside existing pairwise and value
    /// anchor rows survive parse(to_toml(s)) == s.
    #[test]
    fn round_trip_preserves_estimate_anchor_est_lower_est_upper() {
        let mut s = full_session();
        s.judgements = vec![
            full_judgement(),
            bare_judgement(),
            human_anchor(),
            migrated_anchor(),
            pin_anchor(),
            human_estimate_anchor(),
            pin_estimate_anchor(),
            migrated_estimate_anchor(),
            point_estimate_anchor(),
        ];
        let text = to_toml(&s).unwrap();
        let parsed = parse(&text).unwrap();
        assert_eq!(parsed, s);
        // Verify the est_lower/est_upper values survive byte-identically.
        for j in &parsed.judgements {
            if j.frame == FRAME_COST_ANCHOR {
                assert!(
                    j.est_lower.is_some(),
                    "cost-anchor row must carry est_lower"
                );
                assert!(
                    j.est_upper.is_some(),
                    "cost-anchor row must carry est_upper"
                );
            }
        }
    }

    // Validation matrix — every arm in BOTH directions (§8.1).

    #[test]
    fn validate_accepts_human_estimate_anchor() {
        assert!(validate_judgement(&human_estimate_anchor()).is_ok());
    }

    #[test]
    fn validate_accepts_pin_estimate_anchor() {
        assert!(validate_judgement(&pin_estimate_anchor()).is_ok());
    }

    #[test]
    fn validate_accepts_migrated_estimate_anchor() {
        assert!(validate_judgement(&migrated_estimate_anchor()).is_ok());
    }

    #[test]
    fn validate_accepts_point_estimate_anchor() {
        assert!(validate_judgement(&point_estimate_anchor()).is_ok());
    }

    /// form=anchor ∧ domain=estimate ⇒ est_lower present ∧ est_upper present.
    #[test]
    fn validate_rejects_estimate_anchor_without_est_lower() {
        let mut j = human_estimate_anchor();
        j.est_lower = None;
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("est_lower"), "got: {err}");
    }

    #[test]
    fn validate_rejects_estimate_anchor_without_est_upper() {
        let mut j = human_estimate_anchor();
        j.est_upper = None;
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("est_upper"), "got: {err}");
    }

    /// Estimate payload mirroring `estimate::validate` exactly: finite,
    /// lower >= 0, upper >= lower.
    #[test]
    fn validate_estimate_anchor_est_pair_mirrors_estimate_validate() {
        // NAN lower
        let mut j = human_estimate_anchor();
        j.est_lower = Some(f64::NAN);
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("finite"), "nan lower: {err}");

        // INF upper
        let mut j = human_estimate_anchor();
        j.est_upper = Some(f64::INFINITY);
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("finite"), "inf upper: {err}");

        // Negative lower
        let mut j = human_estimate_anchor();
        j.est_lower = Some(-1.0);
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains(">= 0"), "negative lower: {err}");

        // upper < lower
        let mut j = human_estimate_anchor();
        j.est_lower = Some(8.0);
        j.est_upper = Some(2.0);
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("upper"), "upper < lower: {err}");
    }

    /// frame=cost-anchor ⇔ (form=anchor ∧ domain=estimate). A more-work frame
    /// on an estimate anchor row, or cost-anchor on a pairwise row, is rejected.
    #[test]
    fn validate_rejects_cost_anchor_on_pairwise_row() {
        let mut j = estimate_judgement();
        j.frame = FRAME_COST_ANCHOR.to_string();
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(
            err.contains("anchor frame") || err.contains("pairwise"),
            "got: {err}"
        );
    }

    #[test]
    fn validate_rejects_estimate_anchor_with_pairwise_frame() {
        let mut j = human_estimate_anchor();
        j.frame = FRAME_MORE_WORK.to_string();
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("anchor frame"), "got: {err}");
    }

    /// Pairwise rows (order/ratio) ⇒ est_lower/est_upper absent. A pairwise
    /// row with these fields set is refused.
    #[test]
    fn validate_rejects_pairwise_row_with_est_fields() {
        let mut j = full_judgement();
        j.est_lower = Some(2.0);
        j.est_upper = Some(8.0);
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(
            err.contains("est_lower") || err.contains("est_upper"),
            "extra est fields on pairwise: {err}"
        );
    }

    /// form=anchor ∧ domain=value ⇒ est_lower/est_upper ABSENT (payload
    /// exactness cuts both ways).
    #[test]
    fn validate_rejects_value_anchor_with_est_fields() {
        let mut j = human_anchor();
        j.est_lower = Some(2.0);
        j.est_upper = Some(8.0);
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(
            err.contains("must be absent"),
            "extra estimate fields on value anchor: {err}"
        );
    }

    /// form=anchor ∧ domain=estimate ⇒ magnitude absent.
    #[test]
    fn validate_rejects_estimate_anchor_with_magnitude() {
        let mut j = human_estimate_anchor();
        j.magnitude = Some(3.5);
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(
            err.contains("magnitude") && err.contains("must be absent"),
            "got: {err}"
        );
    }

    /// rater=migrated / admission=pin rules: unchanged, domain-agnostic.
    /// A pin on an estimate anchor is valid; migrated estimate anchor is valid.
    #[test]
    fn validate_pin_on_estimate_anchor_is_valid() {
        assert!(validate_judgement(&pin_estimate_anchor()).is_ok());
    }

    #[test]
    fn validate_rejects_agent_pin_on_estimate_anchor() {
        let mut j = pin_estimate_anchor();
        j.rater = RaterKind::Agent;
        let err = validate_judgement(&j).unwrap_err().to_string();
        assert!(err.contains("human anchor"), "got: {err}");
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

    // ---- SL-219: estimate-domain admissibility (VT-1) ------------------------

    /// RSK is admitted in the estimate domain (unlike the value domain — SL-219
    /// D3: settle-cost is comparable even where value is not), and record
    /// kinds size against work kinds.
    #[test]
    fn admissible_estimate_pair_admits_rsk_and_records() {
        assert!(admissible_estimate_pair(kinds::SL, kinds::RSK).is_ok());
        assert!(admissible_estimate_pair(kinds::QUE, kinds::SL).is_ok());
        assert!(admissible_estimate_pair(kinds::QUE, kinds::CON).is_ok());
    }

    /// SL-219 D3 census property: the estimate admit set IS the union
    /// `kinds::VALUE_BEARING` + `kinds::RECORD` — for every census kind,
    /// admission holds exactly when the derivation says so (no parallel list).
    #[test]
    fn estimate_admit_set_is_value_bearing_union_record() {
        for &kind in kinds::ALL_KINDS {
            let admitted = admissible_estimate_pair(kind, kind).is_ok();
            let expected = kinds::VALUE_BEARING.contains(&kind) || kinds::RECORD.contains(&kind);
            assert_eq!(
                admitted, expected,
                "{kind}: admitted iff value-bearing or record"
            );
        }
    }

    // ---- SL-220 §4: anchor supersession scope --------------------------------

    /// A same-subject, same-domain, same-lens (None==None) anchor supersedes an
    /// anchor — the sanctioned correction path.
    #[test]
    fn validate_anchor_supersedes_accepts_same_scope() {
        let target = human_anchor();
        let new = human_anchor();
        assert!(validate_anchor_supersedes(&new, &target).is_ok());
    }

    /// A pairwise target is refused — an anchor supersedes only an anchor.
    #[test]
    fn validate_anchor_supersedes_refuses_pairwise_target() {
        let err = validate_anchor_supersedes(&human_anchor(), &full_judgement()).unwrap_err();
        assert!(err.contains("pairwise row"), "got: {err}");
    }

    /// A foreign-subject target is refused, naming the subject mismatch.
    #[test]
    fn validate_anchor_supersedes_refuses_foreign_subject() {
        let mut target = human_anchor();
        target.a = "IMP-118".to_string();
        let err = validate_anchor_supersedes(&human_anchor(), &target).unwrap_err();
        assert!(err.contains("within one subject"), "got: {err}");
    }

    /// A cross-lens target is refused (None vs Some), naming the lens mismatch.
    #[test]
    fn validate_anchor_supersedes_refuses_cross_lens() {
        let mut target = human_anchor();
        target.lens = Some("user-value".to_string());
        let err = validate_anchor_supersedes(&human_anchor(), &target).unwrap_err();
        assert!(err.contains("cross-lens"), "got: {err}");
    }

    /// A cross-domain target is refused (the target somehow rode another
    /// domain), naming the domain mismatch — belt-and-braces beyond the
    /// per-domain anchor-frame gate.
    #[test]
    fn validate_anchor_supersedes_refuses_cross_domain() {
        let mut target = human_anchor();
        target.domain = DOMAIN_PRIORITY.to_string();
        let err = validate_anchor_supersedes(&human_anchor(), &target).unwrap_err();
        assert!(err.contains("cross-domain"), "got: {err}");
    }

    /// The refusal names the currency (settle-cost) and the offending kind.
    #[test]
    fn admissible_estimate_pair_refusal_names_the_currency() {
        let err = admissible_estimate_pair(kinds::REV, kinds::SL).unwrap_err();
        assert!(err.contains(kinds::REV), "names the kind: {err}");
        assert!(err.contains("settle-cost"), "names the currency: {err}");
        assert!(
            err.contains("work and record kinds"),
            "names the admit set: {err}"
        );
    }
}
