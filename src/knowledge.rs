// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine knowledge` — durable knowledge records (assumption / decision /
//! question / constraint), each a numeric directory under
//! `.doctrine/knowledge/<kind>/` holding a sister `record-NNN.toml` (structured,
//! queried metadata), a scaffolded `record-NNN.md` prose body, and an `NNN-slug`
//! symlink alias — the `backlog.rs` structural twin (design SL-059 §5).
//!
//! Four `RecordKind`s ride four `entity::Kind`s over the same kind-blind engine,
//! each its own tree + reservation namespace (`ASM-001` and `DEC-001` coexist —
//! the counters are independent). The subtypes diverge in their prefix, status
//! vocabulary, and the typed `[facet]` they carry.
//!
//! This module owns the *knowledge-specific* parts — the four `Kind`s, the
//! per-kind status vocabularies (data, not an enum), the typed facet enum-of-
//! structs, the shared `Evidence`, the three closed facet value-enums, the
//! three-layer tolerant parse (`RawRecordToml` + a kind-blind superset `RawFacet`
//! → `validate` dispatches on `record_kind` → the typed `RecordFacet`, with the
//! `""`/`[]` → absent seam), and the per-kind scaffold templates. The kind-
//! agnostic engine is `crate::entity` (unchanged — four new scaffold callers).
//!
//! All phases landed — every production symbol has a real consumer. The only
//! non-production code is the hand-emit render subtree below (VT-1's byte-stable
//! round-trip check), which is `#[cfg(test)]`-gated at each fn rather than masked
//! by a blanket module suppression (mem.pattern.lint.dead-code-expect-vs-cfg-test):
//! production writes go through `render_record_toml_seed` (template) +
//! `set_record_status` (`toml_edit`).

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::entity::{
    self, Artifact, Fileset, Inputs, Kind, LocalFs, MaterialiseRequest, ScaffoldCtx,
};
use crate::listing::{self, Format, ListArgs};
use crate::tomlfmt::toml_string;
// `toml_array_inner` is spliced only by the test-only hand-emit render subtree
// (production list-writes go via the template seed), so its import is `#[cfg(test)]`.
#[cfg(test)]
use crate::tomlfmt::toml_array_inner;

/// The toml/md file stem — shared by all four kinds (`record-NNN.toml`). Distinct
/// from each `Kind.prefix` (`ASM`/`DEC`/…) and from the per-kind tree dirs.
const RECORD_STEM: &str = "record";

// ---------------------------------------------------------------------------
// The discriminator + its four engine `Kind`s
// ---------------------------------------------------------------------------

/// Which knowledge record this is. Closed set; kebab serde (round-trips the
/// toml's `record_kind`) and `clap::ValueEnum` (the `knowledge new` positional,
/// PHASE-03). Selects the tree, prefix, status vocabulary, and scaffold. Fixed at
/// capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RecordKind {
    Assumption,
    Decision,
    Question,
    Constraint,
}

/// The assumption kind: a working belief held until validated. Own tree +
/// reservation namespace.
pub(crate) const ASSUMPTION_KIND: Kind = Kind {
    dir: ".doctrine/knowledge/assumption",
    prefix: "ASM",
    scaffold: |c| record_scaffold(RecordKind::Assumption, c),
};

/// The decision kind: a recorded choice and its rationale.
pub(crate) const DECISION_KIND: Kind = Kind {
    dir: ".doctrine/knowledge/decision",
    prefix: "DEC",
    scaffold: |c| record_scaffold(RecordKind::Decision, c),
};

/// The question kind: an open question whose answer shapes the work.
pub(crate) const QUESTION_KIND: Kind = Kind {
    dir: ".doctrine/knowledge/question",
    prefix: "QUE",
    scaffold: |c| record_scaffold(RecordKind::Question, c),
};

/// The constraint kind: a standing limit on the solution space.
pub(crate) const CONSTRAINT_KIND: Kind = Kind {
    dir: ".doctrine/knowledge/constraint",
    prefix: "CON",
    scaffold: |c| record_scaffold(RecordKind::Constraint, c),
};

impl RecordKind {
    /// The engine `Kind` for this record kind — the single source of its tree +
    /// prefix + scaffold.
    pub(crate) const fn kind(self) -> &'static Kind {
        match self {
            RecordKind::Assumption => &ASSUMPTION_KIND,
            RecordKind::Decision => &DECISION_KIND,
            RecordKind::Question => &QUESTION_KIND,
            RecordKind::Constraint => &CONSTRAINT_KIND,
        }
    }

    /// The canonical-id prefix (`ASM`/`DEC`/`QUE`/`CON`), read off the `Kind` so
    /// the prefix is never hardcoded twice.
    pub(crate) const fn prefix(self) -> &'static str {
        self.kind().prefix
    }

    /// The kebab `record_kind` string written to `record-NNN.toml` (matches the
    /// serde rename). Pure; the render mirror for the stored discriminator.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            RecordKind::Assumption => "assumption",
            RecordKind::Decision => "decision",
            RecordKind::Question => "question",
            RecordKind::Constraint => "constraint",
        }
    }

    /// The canonical ref for an id in this kind's namespace (`ASM-007`) — the
    /// print of `knowledge new` and the inverse of `from_prefix`.
    pub(crate) fn canonical_id(self, id: u32) -> String {
        format!("{}-{id:03}", self.prefix())
    }

    /// Resolve a canonical-id prefix back to its kind (`knowledge show <ID>`
    /// auto-detect, PHASE-03). Prefixes come from the `Kind`s — the single source;
    /// the kind set is `RecordKind::ALL` (one declaration, not a second copy).
    pub(crate) fn from_prefix(prefix: &str) -> Option<Self> {
        RecordKind::ALL.into_iter().find(|k| k.prefix() == prefix)
    }

    /// The seeded default status — the FIRST element of the kind's vocabulary (the
    /// seed convention, §5). The single source the scaffold template literal mirrors;
    /// the F-A2 seed-status anti-drift guard test pins the two together — its only
    /// caller (the template bakes the literal at runtime), hence `#[cfg(test)]`.
    #[cfg(test)]
    pub(crate) fn default_status(self) -> &'static str {
        statuses(self).first().copied().unwrap_or_default()
    }

    /// Every kind in DECLARATION order — the single source for the cross-kind
    /// `list` read (each tree in turn) and the prefix round-trip.
    pub(crate) const ALL: [RecordKind; 4] = [
        RecordKind::Assumption,
        RecordKind::Decision,
        RecordKind::Question,
        RecordKind::Constraint,
    ];
}

// ---------------------------------------------------------------------------
// Status vocabulary — data-driven (L1); hide-set distinct from the partition
// ---------------------------------------------------------------------------

/// The assumption status vocabulary; `held` is the seed (first element). `pub(crate)`
/// — read by the PHASE-02 priority-partition canaries.
pub(crate) const ASSUMPTION_STATUSES: &[&str] =
    &["held", "testing", "validated", "invalidated", "obsolete"];
/// The decision status vocabulary; `proposed` is the seed.
pub(crate) const DECISION_STATUSES: &[&str] = &["proposed", "accepted", "rejected", "superseded"];
/// The question status vocabulary; `open` is the seed.
pub(crate) const QUESTION_STATUSES: &[&str] = &["open", "answered", "obsolete"];
/// The constraint status vocabulary; `active` is the seed.
pub(crate) const CONSTRAINT_STATUSES: &[&str] = &["active", "waived", "superseded", "retired"];

/// The default-list HIDE-set (settled states only) — NOT the full vocab, and NOT
/// the priority partition's terminal set. Drives `listing::retain` (PHASE-03).
const ASSUMPTION_HIDDEN: &[&str] = &["validated", "invalidated", "obsolete"];
/// `accepted` deliberately stays visible (a live decision is not settled-away).
const DECISION_HIDDEN: &[&str] = &["rejected", "superseded"];
const QUESTION_HIDDEN: &[&str] = &["answered", "obsolete"];
const CONSTRAINT_HIDDEN: &[&str] = &["waived", "superseded", "retired"];

/// The kind's status vocabulary + known-set — the single source `default_status`,
/// the PHASE-02 partition, and the PHASE-03 `--status` validator read.
pub(crate) fn statuses(k: RecordKind) -> &'static [&'static str] {
    match k {
        RecordKind::Assumption => ASSUMPTION_STATUSES,
        RecordKind::Decision => DECISION_STATUSES,
        RecordKind::Question => QUESTION_STATUSES,
        RecordKind::Constraint => CONSTRAINT_STATUSES,
    }
}

/// Whether `status` is in the kind's default-list hide-set (a settled state). An
/// out-of-vocab token (impossible on a serde-validated item, but the predicate is
/// stringly) is treated as not-hidden. `--all` / explicit `--status` override in
/// `retain` (PHASE-03).
pub(crate) fn is_hidden(k: RecordKind, status: &str) -> bool {
    hidden(k).contains(&status)
}

/// The kind's hide-set — the private companion to `statuses`.
const fn hidden(k: RecordKind) -> &'static [&'static str] {
    match k {
        RecordKind::Assumption => ASSUMPTION_HIDDEN,
        RecordKind::Decision => DECISION_HIDDEN,
        RecordKind::Question => QUESTION_HIDDEN,
        RecordKind::Constraint => CONSTRAINT_HIDDEN,
    }
}

// ---------------------------------------------------------------------------
// Closed facet value-enums (kebab serde + an `as_str` render mirror + known-set)
// ---------------------------------------------------------------------------

/// An assumption's confidence level (assumption facet only). Closed set, kebab
/// serde; optional (the `"" -> None` seam — seeded empty until assessed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Confidence {
    Low,
    Medium,
    High,
}

impl Confidence {
    /// The kebab string for render (matches the serde rename). Pure.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Confidence::Low => "low",
            Confidence::Medium => "medium",
            Confidence::High => "high",
        }
    }

    /// The known-set — the drift-canary authority (VT-3). Lockstep with the
    /// variants (`confidence_known_set_matches_variants`), its only consumer.
    #[cfg(test)]
    pub(crate) const KNOWN: &'static [&'static str] = &["low", "medium", "high"];
}

/// The basis an assumption rests on (assumption facet only). Closed set, kebab
/// serde; optional.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Basis {
    Observation,
    PriorArt,
    DesignInference,
    ExternalSource,
    OperatorJudgement,
}

impl Basis {
    /// The kebab string for render (matches the serde rename). Pure.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Basis::Observation => "observation",
            Basis::PriorArt => "prior-art",
            Basis::DesignInference => "design-inference",
            Basis::ExternalSource => "external-source",
            Basis::OperatorJudgement => "operator-judgement",
        }
    }

    /// The known-set — the drift-canary authority (VT-3), its only consumer.
    #[cfg(test)]
    pub(crate) const KNOWN: &'static [&'static str] = &[
        "observation",
        "prior-art",
        "design-inference",
        "external-source",
        "operator-judgement",
    ];
}

/// Where a constraint originates (constraint facet only). Closed set, kebab serde;
/// optional.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ConstraintSource {
    Canon,
    Adr,
    External,
    Technical,
    Legal,
    Compatibility,
    Operator,
}

impl ConstraintSource {
    /// The kebab string for render (matches the serde rename). Pure.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            ConstraintSource::Canon => "canon",
            ConstraintSource::Adr => "adr",
            ConstraintSource::External => "external",
            ConstraintSource::Technical => "technical",
            ConstraintSource::Legal => "legal",
            ConstraintSource::Compatibility => "compatibility",
            ConstraintSource::Operator => "operator",
        }
    }

    /// The known-set — the drift-canary authority (VT-3), its only consumer.
    #[cfg(test)]
    pub(crate) const KNOWN: &'static [&'static str] = &[
        "canon",
        "adr",
        "external",
        "technical",
        "legal",
        "compatibility",
        "operator",
    ];
}

// ---------------------------------------------------------------------------
// The validated entity + its typed facet enum-of-structs (L2)
// ---------------------------------------------------------------------------

/// The validated knowledge record (design §5). `id/slug/title/status` are top-level
/// in the toml so the file also round-trips into the shared `meta::Meta`.
/// `record_kind` is stored AND implied by the tree dir — stored so one read yields
/// the entity without path inspection. The `[facet]` is kind-dispatched; the
/// `[evidence]` is shared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KnowledgeRecord {
    id: u32,
    slug: String,
    title: String,
    record_kind: RecordKind,
    status: String,
    created: String,
    updated: String,
    tags: Vec<String>,
    facet: RecordFacet,
    evidence: Evidence,
}

/// The typed facet, kind-dispatched (one variant per kind — no untyped bag). Built
/// by `validate` off the kind-blind `RawFacet` superset, so the wrong kind's fields
/// can never reach the wrong variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RecordFacet {
    Assumption(AssumptionFacet),
    Decision(DecisionFacet),
    Question(QuestionFacet),
    Constraint(ConstraintFacet),
}

/// The assumption facet — `confidence` is assumption-only (§9). Every optional
/// field is `""`/`[]` → absent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct AssumptionFacet {
    claim: Option<String>,
    confidence: Option<Confidence>,
    basis: Option<Basis>,
    validation_plan: Option<String>,
    validated_by: Option<String>,
    validated_on: Option<String>,
    invalidated_by: Option<String>,
    invalidated_on: Option<String>,
}

/// The decision facet (§9). `alternatives`/`consequences` are lists; every `…_by`
/// is free-text attribution; `…_on` is an unvalidated ISO date string.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DecisionFacet {
    context: Option<String>,
    choice: Option<String>,
    alternatives: Vec<String>,
    rationale: Option<String>,
    consequences: Vec<String>,
    decided_by: Option<String>,
    decided_on: Option<String>,
}

/// The question facet (§9).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct QuestionFacet {
    question: Option<String>,
    why_matters: Option<String>,
    answer: Option<String>,
    answered_by: Option<String>,
    answered_on: Option<String>,
}

/// The constraint facet (§9). `applies_to` is a list; `source` is the closed
/// `ConstraintSource` enum.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ConstraintFacet {
    statement: Option<String>,
    source: Option<ConstraintSource>,
    applies_to: Vec<String>,
    waiver_reason: Option<String>,
    waived_by: Option<String>,
    waived_on: Option<String>,
}

/// The shared evidence block (all four kinds, §9): free-text citations. Never the
/// queryable relation graph (D5) — three plain `Vec<String>`, `[]` default.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Evidence {
    supports: Vec<String>,
    contradicts: Vec<String>,
    notes: Vec<String>,
}

// ---------------------------------------------------------------------------
// Three-layer tolerant parse (the entity-model parse tier — §5)
// ---------------------------------------------------------------------------

/// The tolerant top layer. `status` stays `String` (validated against
/// `statuses(kind)` at the CLI seam, PHASE-03 — not here). `[facet]` is read as ONE
/// kind-blind superset `RawFacet` (every field across all four kinds, each
/// `#[serde(default)]`), so the read is kind-blind and `validate` is kind-aware.
/// `[evidence]` defaults empty.
#[derive(Debug, Deserialize)]
struct RawRecordToml {
    id: u32,
    slug: String,
    title: String,
    record_kind: RecordKind,
    status: String,
    created: String,
    updated: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    facet: RawFacet,
    #[serde(default)]
    evidence: RawEvidence,
}

/// The kind-blind facet superset (§5): every field of every kind's facet, all
/// `#[serde(default)]`, all raw `String`/`Vec<String>` (the `"" -> None` seam is a
/// `validate` pass, not a serde derive). `validate` reads only the fields its
/// `record_kind` owns and discards the rest.
#[derive(Debug, Default, Deserialize)]
struct RawFacet {
    // assumption
    #[serde(default)]
    claim: String,
    #[serde(default)]
    confidence: String,
    #[serde(default)]
    basis: String,
    #[serde(default)]
    validation_plan: String,
    #[serde(default)]
    validated_by: String,
    #[serde(default)]
    validated_on: String,
    #[serde(default)]
    invalidated_by: String,
    #[serde(default)]
    invalidated_on: String,
    // decision
    #[serde(default)]
    context: String,
    #[serde(default)]
    choice: String,
    #[serde(default)]
    alternatives: Vec<String>,
    #[serde(default)]
    rationale: String,
    #[serde(default)]
    consequences: Vec<String>,
    #[serde(default)]
    decided_by: String,
    #[serde(default)]
    decided_on: String,
    // question
    #[serde(default)]
    question: String,
    #[serde(default)]
    why_matters: String,
    #[serde(default)]
    answer: String,
    #[serde(default)]
    answered_by: String,
    #[serde(default)]
    answered_on: String,
    // constraint
    #[serde(default)]
    statement: String,
    #[serde(default)]
    source: String,
    #[serde(default)]
    applies_to: Vec<String>,
    #[serde(default)]
    waiver_reason: String,
    #[serde(default)]
    waived_by: String,
    #[serde(default)]
    waived_on: String,
}

/// The tolerant evidence layer — three free lists, `[]` default.
#[derive(Debug, Default, Deserialize)]
struct RawEvidence {
    #[serde(default)]
    supports: Vec<String>,
    #[serde(default)]
    contradicts: Vec<String>,
    #[serde(default)]
    notes: Vec<String>,
}

/// Parse a kebab token into its closed enum via the serde derive — the single
/// source of the variant↔string mapping (the `as_str` mirrors render only). Mirrors
/// `backlog::parse_enum`.
fn parse_enum<T: serde::de::DeserializeOwned>(token: &str, what: &str) -> anyhow::Result<T> {
    use serde::de::IntoDeserializer;
    let de: serde::de::value::StrDeserializer<'_, serde::de::value::Error> =
        token.into_deserializer();
    T::deserialize(de).map_err(|e| anyhow::anyhow!("invalid {what} `{token}`: {e}"))
}

/// The `"" -> None` seam for an optional closed enum: an empty token is absent; a
/// non-empty token parses to its variant (erroring on an unknown one).
fn optional_enum<T: serde::de::DeserializeOwned>(
    token: &str,
    what: &str,
) -> anyhow::Result<Option<T>> {
    if token.is_empty() {
        Ok(None)
    } else {
        parse_enum(token, what).map(Some)
    }
}

/// The `"" -> None` seam for an optional free-text field. Consumes the raw string.
fn optional_text(text: String) -> Option<String> {
    if text.is_empty() { None } else { Some(text) }
}

/// Validate a tolerant `RawRecordToml` into a typed [`KnowledgeRecord`] — the second
/// layer of the parse model. Dispatches the kind-blind `RawFacet` on `record_kind`
/// to the right typed [`RecordFacet`] variant, mapping every seeded `""`/`[]` to
/// absent, and validates the closed facet enums. Consumes the raw layer.
fn validate(raw: RawRecordToml) -> anyhow::Result<KnowledgeRecord> {
    let facet = validate_facet(raw.record_kind, raw.facet)?;
    let evidence = Evidence {
        supports: raw.evidence.supports,
        contradicts: raw.evidence.contradicts,
        notes: raw.evidence.notes,
    };
    Ok(KnowledgeRecord {
        id: raw.id,
        slug: raw.slug,
        title: raw.title,
        record_kind: raw.record_kind,
        status: raw.status,
        created: raw.created,
        updated: raw.updated,
        tags: raw.tags,
        facet,
        evidence,
    })
}

/// Dispatch the kind-blind `RawFacet` on `record_kind` to the typed variant — the
/// kind-aware half of "kind-blind read, kind-aware validate" (§5). Each arm reads
/// only the fields its kind owns through the `"" -> None` / `[]`-passthrough seams.
fn validate_facet(kind: RecordKind, raw: RawFacet) -> anyhow::Result<RecordFacet> {
    Ok(match kind {
        RecordKind::Assumption => RecordFacet::Assumption(AssumptionFacet {
            claim: optional_text(raw.claim),
            confidence: optional_enum(&raw.confidence, "confidence")?,
            basis: optional_enum(&raw.basis, "basis")?,
            validation_plan: optional_text(raw.validation_plan),
            validated_by: optional_text(raw.validated_by),
            validated_on: optional_text(raw.validated_on),
            invalidated_by: optional_text(raw.invalidated_by),
            invalidated_on: optional_text(raw.invalidated_on),
        }),
        RecordKind::Decision => RecordFacet::Decision(DecisionFacet {
            context: optional_text(raw.context),
            choice: optional_text(raw.choice),
            alternatives: raw.alternatives,
            rationale: optional_text(raw.rationale),
            consequences: raw.consequences,
            decided_by: optional_text(raw.decided_by),
            decided_on: optional_text(raw.decided_on),
        }),
        RecordKind::Question => RecordFacet::Question(QuestionFacet {
            question: optional_text(raw.question),
            why_matters: optional_text(raw.why_matters),
            answer: optional_text(raw.answer),
            answered_by: optional_text(raw.answered_by),
            answered_on: optional_text(raw.answered_on),
        }),
        RecordKind::Constraint => RecordFacet::Constraint(ConstraintFacet {
            statement: optional_text(raw.statement),
            source: optional_enum(&raw.source, "source")?,
            applies_to: raw.applies_to,
            waiver_reason: optional_text(raw.waiver_reason),
            waived_by: optional_text(raw.waived_by),
            waived_on: optional_text(raw.waived_on),
        }),
    })
}

// ---------------------------------------------------------------------------
// Pure: render (the byte-stable round-trip seam, the rec.rs hand-emit precedent)
//
// Test-only (`#[cfg(test)]`): this hand-emit backs VT-1's byte-stable round-trip
// proof and has no production caller — writes go through `render_record_toml_seed`
// (template) + `set_record_status` (toml_edit). Gated per-fn, not by a blanket
// module suppression, so a future genuinely-dead symbol still trips the lint.
// ---------------------------------------------------------------------------

/// Render a populated [`KnowledgeRecord`] to its `record-NNN.toml` text — the
/// byte-stable round-trip seam (VT-1). Hand-emitted in the F1 on-disk order
/// (top-level meta → `[facet]` → `[evidence]`, NO `[[relation]]`/`[relationships]`),
/// every spliced value through `toml_string`/`toml_array_inner` so a hostile value
/// can neither break the document nor inject a key
/// (mem.pattern.render.toml-splice-escape-user-values). A naive `toml::to_string`
/// would bypass that seam and reorder keys, so the emit is by hand — the same idiom
/// as `rec::render_rec_toml_populated`.
#[cfg(test)]
fn render_record_toml(record: &KnowledgeRecord) -> String {
    [
        String::from("schema = \"doctrine.knowledge\"\nversion = 1\n\n"),
        format!("id = {}\n", record.id),
        format!("slug = {}\n", toml_string(&record.slug)),
        format!("title = {}\n", toml_string(&record.title)),
        format!("record_kind = \"{}\"\n", record.record_kind.as_str()),
        format!("status = {}\n", toml_string(&record.status)),
        format!("created = {}\n", toml_string(&record.created)),
        format!("updated = {}\n", toml_string(&record.updated)),
        format!("tags = [{}]\n", toml_array_inner(&record.tags)),
        render_facet(&record.facet),
        render_evidence(&record.evidence),
    ]
    .concat()
}

/// One `key = "value"` text line for an optional field — `""` when absent (the
/// inverse of the `"" -> None` parse seam, so a round-trip is byte-stable). The
/// value rides `toml_string` for escaping. Closed enums map through this too —
/// `kind.map(Enum::as_str)` yields the same `Option<&str>`.
#[cfg(test)]
fn opt_text_line(key: &str, value: Option<&str>) -> String {
    format!("{key} = {}\n", toml_string(value.unwrap_or("")))
}

/// One `key = [..]` list line, escaped through `toml_array_inner`.
#[cfg(test)]
fn list_line(key: &str, xs: &[String]) -> String {
    format!("{key} = [{}]\n", toml_array_inner(xs))
}

/// Render the `[facet]` block for the populated round-trip, kind-dispatched in the
/// template's field order so the emit is byte-stable against the on-disk layout.
#[cfg(test)]
fn render_facet(facet: &RecordFacet) -> String {
    let mut out = String::from("\n[facet]\n");
    match facet {
        RecordFacet::Assumption(f) => {
            out.push_str(&opt_text_line("claim", f.claim.as_deref()));
            out.push_str(&opt_text_line(
                "confidence",
                f.confidence.map(Confidence::as_str),
            ));
            out.push_str(&opt_text_line("basis", f.basis.map(Basis::as_str)));
            out.push_str(&opt_text_line(
                "validation_plan",
                f.validation_plan.as_deref(),
            ));
            out.push_str(&opt_text_line("validated_by", f.validated_by.as_deref()));
            out.push_str(&opt_text_line("validated_on", f.validated_on.as_deref()));
            out.push_str(&opt_text_line(
                "invalidated_by",
                f.invalidated_by.as_deref(),
            ));
            out.push_str(&opt_text_line(
                "invalidated_on",
                f.invalidated_on.as_deref(),
            ));
        }
        RecordFacet::Decision(f) => {
            out.push_str(&opt_text_line("context", f.context.as_deref()));
            out.push_str(&opt_text_line("choice", f.choice.as_deref()));
            out.push_str(&list_line("alternatives", &f.alternatives));
            out.push_str(&opt_text_line("rationale", f.rationale.as_deref()));
            out.push_str(&list_line("consequences", &f.consequences));
            out.push_str(&opt_text_line("decided_by", f.decided_by.as_deref()));
            out.push_str(&opt_text_line("decided_on", f.decided_on.as_deref()));
        }
        RecordFacet::Question(f) => {
            out.push_str(&opt_text_line("question", f.question.as_deref()));
            out.push_str(&opt_text_line("why_matters", f.why_matters.as_deref()));
            out.push_str(&opt_text_line("answer", f.answer.as_deref()));
            out.push_str(&opt_text_line("answered_by", f.answered_by.as_deref()));
            out.push_str(&opt_text_line("answered_on", f.answered_on.as_deref()));
        }
        RecordFacet::Constraint(f) => {
            out.push_str(&opt_text_line("statement", f.statement.as_deref()));
            out.push_str(&opt_text_line(
                "source",
                f.source.map(ConstraintSource::as_str),
            ));
            out.push_str(&list_line("applies_to", &f.applies_to));
            out.push_str(&opt_text_line("waiver_reason", f.waiver_reason.as_deref()));
            out.push_str(&opt_text_line("waived_by", f.waived_by.as_deref()));
            out.push_str(&opt_text_line("waived_on", f.waived_on.as_deref()));
        }
    }
    out
}

/// Render the shared `[evidence]` block for the populated round-trip.
#[cfg(test)]
fn render_evidence(e: &Evidence) -> String {
    [
        String::from("\n[evidence]\n"),
        list_line("supports", &e.supports),
        list_line("contradicts", &e.contradicts),
        list_line("notes", &e.notes),
    ]
    .concat()
}

// ---------------------------------------------------------------------------
// Pure: scaffold (the seed-empty materialiser — the backlog precedent)
// ---------------------------------------------------------------------------

/// Render `record-<id>.toml` from the kind's embedded template by token
/// substitution — the seeded-empty capture form (every facet/evidence field empty,
/// `status` == `default_status(kind)` baked into the template literal). The
/// `id/slug/title/status` keys round-trip into `meta::Meta`; `{{kind}}` is implied
/// by the template (one per kind), not a token.
fn render_record_toml_seed(
    kind: RecordKind,
    id: u32,
    slug: &str,
    title: &str,
    date: &str,
) -> anyhow::Result<String> {
    let template = match kind {
        RecordKind::Assumption => "templates/knowledge-assumption.toml",
        RecordKind::Decision => "templates/knowledge-decision.toml",
        RecordKind::Question => "templates/knowledge-question.toml",
        RecordKind::Constraint => "templates/knowledge-constraint.toml",
    };
    Ok(crate::install::asset_text(template)?
        .replace("{{id}}", &id.to_string())
        .replace("{{slug}}", &toml_string(slug))
        .replace("{{title}}", &toml_string(title))
        .replace("{{date}}", date))
}

/// Render `record-<id>.md` from the embedded prose template: `{{ref}}` (the
/// canonical id) + `{{title}}`. No frontmatter — metadata lives in the sister toml.
fn render_record_md(canonical_id: &str, title: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/knowledge.md")?
        .replace("{{ref}}", canonical_id)
        .replace("{{title}}", title))
}

/// The knowledge fileset: sister TOML, prose body, and `<id>-<slug>` symlink, all
/// relative to the kind's tree root — structurally `backlog_scaffold`. The `kind`
/// decides the toml template (the per-kind facet + seed status); the md and symlink
/// are kind-uniform. Shared by all four `Kind`s via their scaffold closure.
fn record_scaffold(kind: RecordKind, ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
    let id = ctx.id;
    let name = format!("{id:03}");
    Ok(vec![
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/{RECORD_STEM}-{name}.toml")),
            body: render_record_toml_seed(kind, id, ctx.slug, ctx.title, ctx.date)?,
        },
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/{RECORD_STEM}-{name}.md")),
            body: render_record_md(ctx.canonical, ctx.title)?,
        },
        Artifact::Symlink {
            rel_path: PathBuf::from(format!("{name}-{}", ctx.slug)),
            target: name,
        },
    ])
}

// ---------------------------------------------------------------------------
// Prefix → kind resolution (FR-004) — the shared `show`/`status` auto-detect
// ---------------------------------------------------------------------------

/// Resolve a canonical record ref (`ASM-007` / `dec-3`) into its `(RecordKind, id)`
/// — the prefix auto-detect shared by `show` and `status` (FR-004, design §6).
/// Split on the LAST `-`, upper-case the prefix (`dec-3` is tolerated, mirroring
/// `backlog::parse_ref`), resolve it via [`RecordKind::from_prefix`], and parse the
/// numeric tail (`DEC-7` and `DEC-007` both yield 7). The four counters are
/// independent, so the prefix is load-bearing for disambiguation (`ASM-1` ≠ `DEC-1`).
/// An unknown prefix or a non-numeric tail is a hard error — never an implicit create.
fn resolve_ref(reference: &str) -> anyhow::Result<(RecordKind, u32)> {
    let (prefix, tail) = reference.rsplit_once('-').with_context(|| {
        format!("`{reference}` is not a canonical record ref (expected e.g. ASM-007)")
    })?;
    let kind = RecordKind::from_prefix(&prefix.to_uppercase()).with_context(|| {
        format!("unknown record prefix `{prefix}` in `{reference}` (expected ASM/DEC/QUE/CON)")
    })?;
    let id: u32 = tail
        .parse()
        .with_context(|| format!("`{tail}` is not a numeric id in `{reference}`"))?;
    Ok((kind, id))
}

/// The union of all four kinds' status vocabularies — the cross-kind `--status`
/// known-set for `knowledge list` (design §6: the validator admits any token that is
/// in-vocab for ANY kind, so `-s superseded` spans DEC + CON). De-duplicated, in a
/// stable `RecordKind::ALL` × vocab order.
fn union_statuses() -> Vec<&'static str> {
    let mut union: Vec<&'static str> = Vec::new();
    for kind in RecordKind::ALL {
        for &status in statuses(kind) {
            if !union.contains(&status) {
                union.push(status);
            }
        }
    }
    union
}

// ---------------------------------------------------------------------------
// Read: per-kind tree → validated records (total over a missing dir)
// ---------------------------------------------------------------------------

/// Read ONE record's `record-<NNN>.toml` into a validated [`KnowledgeRecord`] — the
/// single-id read shared by `read_kind`'s loop and `show` (DRY: one parse path). The
/// caller owns kind disambiguation (`resolve_ref`). A missing file is a hard error
/// (the id must already be reserved — `show` never implicitly creates), mirroring
/// `backlog::read_item`.
fn read_record(root: &Path, kind: RecordKind, id: u32) -> anyhow::Result<KnowledgeRecord> {
    let name = format!("{id:03}");
    let path = root
        .join(kind.kind().dir)
        .join(&name)
        .join(format!("{RECORD_STEM}-{name}.toml"));
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("record not found at {}", path.display()))?;
    let raw: RawRecordToml =
        toml::from_str(&text).with_context(|| format!("Failed to parse {}", path.display()))?;
    validate(raw)
}

/// Read every record under one kind's tree into validated [`KnowledgeRecord`]s. Rides
/// `entity::scan_ids` (numeric dirs only; a MISSING tree → empty set, the total-function
/// tolerance), then parses + `validate`s each `record-NNN.toml`. Mirrors
/// `backlog::read_kind`.
fn read_kind(root: &Path, kind: RecordKind) -> anyhow::Result<Vec<KnowledgeRecord>> {
    let tree = root.join(kind.kind().dir);
    let mut records = Vec::new();
    for id in entity::scan_ids(&tree)? {
        records.push(read_record(root, kind, id)?);
    }
    Ok(records)
}

/// Read every record across ALL FOUR trees (cross-kind), in `RecordKind::ALL` order —
/// the corpus `list` surveys. Mirrors `backlog::read_all`.
fn read_all(root: &Path) -> anyhow::Result<Vec<KnowledgeRecord>> {
    let mut records = Vec::new();
    for kind in RecordKind::ALL {
        records.extend(read_kind(root, kind)?);
    }
    Ok(records)
}

// ---------------------------------------------------------------------------
// `knowledge new` — reserve an id + scaffold the seeded record
// ---------------------------------------------------------------------------

/// `doctrine knowledge new <record_kind> [title] [--slug]` — allocate the next id in
/// the kind's namespace and scaffold the seeded record (default status, empty
/// `[facet]`, empty `[evidence]`). Thin shell (mirrors `backlog::run_new`): resolve
/// the root + title + slug, stamp today, mint above the trunk ids, print the canonical
/// id + dir.
pub(crate) fn run_new(
    path: Option<PathBuf>,
    record_kind: RecordKind,
    title: Option<String>,
    slug: Option<String>,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let title = crate::input::resolve_title(title)?;
    let slug = crate::input::resolve_slug(&title, slug)?;
    let date = crate::clock::today();
    let trunk_ids = crate::git::trunk_entity_ids(&root, record_kind.kind().dir)?;
    let out = entity::materialise(
        record_kind.kind(),
        &LocalFs,
        &root,
        &MaterialiseRequest::Fresh,
        &Inputs {
            slug: &slug,
            title: &title,
            date: &date,
        },
        &trunk_ids,
    )?;
    let id = out
        .eid
        .numeric_id()
        .context("knowledge kind must yield a numeric id")?;
    writeln!(
        io::stdout(),
        "Created {}: {}",
        record_kind.canonical_id(id),
        out.dir.display()
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// `knowledge show` — reassemble one record (table | json)
// ---------------------------------------------------------------------------

/// Render a [`KnowledgeRecord`] for `show` — a PURE fn of the record's OWN local state
/// ("cannot go stale"), so it reads no other file and surfaces no inbound refs (the
/// reverse view is the deferred registry surface's, ADR-004). House style: `Vec<String>`
/// parts each carrying their own newline, joined by `concat()` (the `backlog::format_show`
/// precedent — avoids the `push_str(&format!)` lint). The `[facet]` block is
/// kind-dispatched; each axis renders only when populated; the `[evidence]` block
/// renders only when any axis is non-empty.
fn format_show(record: &KnowledgeRecord) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!(
        "{} — {}\n",
        record.record_kind.canonical_id(record.id),
        record.title
    ));
    parts.push(format!(
        "{} · {} · {}\n",
        record.slug,
        record.record_kind.as_str(),
        record.status,
    ));
    parts.push(format!(
        "created {} · updated {}\n",
        record.created, record.updated
    ));
    if !record.tags.is_empty() {
        parts.push(format!("tags: {}\n", record.tags.join(", ")));
    }
    parts.push(format_facet(&record.facet));
    parts.push(format_evidence(&record.evidence));
    parts.concat()
}

/// One `  key: value` show line for an optional text/enum field — emitted only when
/// present (absent fields are silent, unlike the round-trip render which seeds `""`).
fn show_opt_line(key: &str, value: Option<&str>) -> String {
    match value {
        Some(v) => format!("  {key}: {v}\n"),
        None => String::new(),
    }
}

/// One `  key: a, b` show line for a list field — emitted only when non-empty.
fn show_list_line(key: &str, xs: &[String]) -> String {
    if xs.is_empty() {
        String::new()
    } else {
        format!("  {key}: {}\n", xs.join(", "))
    }
}

/// Render the kind-dispatched `[facet]` block for `show` — the populated axes only,
/// in template field order, under a `\n[facet]\n` header that appears only when the
/// facet carries at least one populated axis.
fn format_facet(facet: &RecordFacet) -> String {
    let body = match facet {
        RecordFacet::Assumption(f) => [
            show_opt_line("claim", f.claim.as_deref()),
            show_opt_line("confidence", f.confidence.map(Confidence::as_str)),
            show_opt_line("basis", f.basis.map(Basis::as_str)),
            show_opt_line("validation_plan", f.validation_plan.as_deref()),
            show_opt_line("validated_by", f.validated_by.as_deref()),
            show_opt_line("validated_on", f.validated_on.as_deref()),
            show_opt_line("invalidated_by", f.invalidated_by.as_deref()),
            show_opt_line("invalidated_on", f.invalidated_on.as_deref()),
        ]
        .concat(),
        RecordFacet::Decision(f) => [
            show_opt_line("context", f.context.as_deref()),
            show_opt_line("choice", f.choice.as_deref()),
            show_list_line("alternatives", &f.alternatives),
            show_opt_line("rationale", f.rationale.as_deref()),
            show_list_line("consequences", &f.consequences),
            show_opt_line("decided_by", f.decided_by.as_deref()),
            show_opt_line("decided_on", f.decided_on.as_deref()),
        ]
        .concat(),
        RecordFacet::Question(f) => [
            show_opt_line("question", f.question.as_deref()),
            show_opt_line("why_matters", f.why_matters.as_deref()),
            show_opt_line("answer", f.answer.as_deref()),
            show_opt_line("answered_by", f.answered_by.as_deref()),
            show_opt_line("answered_on", f.answered_on.as_deref()),
        ]
        .concat(),
        RecordFacet::Constraint(f) => [
            show_opt_line("statement", f.statement.as_deref()),
            show_opt_line("source", f.source.map(ConstraintSource::as_str)),
            show_list_line("applies_to", &f.applies_to),
            show_opt_line("waiver_reason", f.waiver_reason.as_deref()),
            show_opt_line("waived_by", f.waived_by.as_deref()),
            show_opt_line("waived_on", f.waived_on.as_deref()),
        ]
        .concat(),
    };
    if body.is_empty() {
        String::new()
    } else {
        format!("\n[facet]\n{body}")
    }
}

/// Render the shared `[evidence]` block for `show` — the populated axes only, under a
/// header that appears only when at least one axis is non-empty.
fn format_evidence(e: &Evidence) -> String {
    let body = [
        show_list_line("supports", &e.supports),
        show_list_line("contradicts", &e.contradicts),
        show_list_line("notes", &e.notes),
    ]
    .concat();
    if body.is_empty() {
        String::new()
    } else {
        format!("\n[evidence]\n{body}")
    }
}

/// Render the `Json` show: the record's faithful state under the shared `{kind, …}`
/// envelope (the `backlog::show_json` precedent). The validated record's fields are
/// private and its closed enums render via `as_str`, so the JSON is projected by hand
/// (not a derive): the flat identity, the kind-dispatched `[facet]`, and the shared
/// `[evidence]`. Pure over the record's own state (no cross-corpus scan). `serde_json`
/// sorts object keys.
fn show_json(record: &KnowledgeRecord) -> anyhow::Result<String> {
    let value = serde_json::json!({
        "kind": "knowledge",
        "knowledge": {
            "id": record.record_kind.canonical_id(record.id),
            "record_kind": record.record_kind.as_str(),
            "slug": record.slug,
            "title": record.title,
            "status": record.status,
            "created": record.created,
            "updated": record.updated,
            "tags": record.tags,
            "facet": facet_json(&record.facet),
            "evidence": {
                "supports": record.evidence.supports,
                "contradicts": record.evidence.contradicts,
                "notes": record.evidence.notes,
            },
        },
    });
    serde_json::to_string_pretty(&value).context("failed to serialize knowledge show JSON")
}

/// The kind-dispatched `[facet]` JSON object — every field present (optional fields as
/// `null`, lists as arrays), so the shape is stable per kind. Closed enums render via
/// `as_str`.
fn facet_json(facet: &RecordFacet) -> serde_json::Value {
    match facet {
        RecordFacet::Assumption(f) => serde_json::json!({
            "claim": f.claim,
            "confidence": f.confidence.map(Confidence::as_str),
            "basis": f.basis.map(Basis::as_str),
            "validation_plan": f.validation_plan,
            "validated_by": f.validated_by,
            "validated_on": f.validated_on,
            "invalidated_by": f.invalidated_by,
            "invalidated_on": f.invalidated_on,
        }),
        RecordFacet::Decision(f) => serde_json::json!({
            "context": f.context,
            "choice": f.choice,
            "alternatives": f.alternatives,
            "rationale": f.rationale,
            "consequences": f.consequences,
            "decided_by": f.decided_by,
            "decided_on": f.decided_on,
        }),
        RecordFacet::Question(f) => serde_json::json!({
            "question": f.question,
            "why_matters": f.why_matters,
            "answer": f.answer,
            "answered_by": f.answered_by,
            "answered_on": f.answered_on,
        }),
        RecordFacet::Constraint(f) => serde_json::json!({
            "statement": f.statement,
            "source": f.source.map(ConstraintSource::as_str),
            "applies_to": f.applies_to,
            "waiver_reason": f.waiver_reason,
            "waived_by": f.waived_by,
            "waived_on": f.waived_on,
        }),
    }
}

/// `doctrine knowledge show <ID> [--format table|json]` — the inspect verb (design §6).
/// Thin shell: find the root, `resolve_ref` the id to its kind (prefix auto-detect),
/// read THAT record's single toml, render it to stdout. READ-ONLY — no mutation, no
/// cross-corpus scan (only the one record's file is opened).
pub(crate) fn run_show(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (kind, id) = resolve_ref(reference)?;
    let record = read_record(&root, kind, id)?;
    let out = match format {
        Format::Table => format_show(&record),
        Format::Json => show_json(&record)?,
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// `knowledge list` — cross-kind survey on the shared spine
// ---------------------------------------------------------------------------

/// One record projected to its faithful JSON list row (the `backlog::BacklogRow`
/// precedent). `id` is the prefixed canonical id; `record_kind`/`status` are the
/// kebab/vocab strings. The facet + evidence are list-irrelevant (they ride `show`),
/// so the list row stays flat.
#[derive(Debug, Serialize)]
struct RecordRow {
    id: String,
    record_kind: &'static str,
    status: String,
    slug: String,
    title: String,
}

/// The table columns `knowledge list` can show (`--columns` tokens over
/// `R = KnowledgeRecord` — non-capturing extractors, the prefixed id materialised in
/// the cell from the record's own kind+id). Declaration order is what the
/// unknown-column error lists.
const KN_COLUMNS: [listing::Column<KnowledgeRecord>; 5] = [
    listing::Column {
        name: "id",
        header: "id",
        cell: |r| r.record_kind.canonical_id(r.id),
        paint: listing::ColumnPaint::Fixed(owo_colors::DynColors::Ansi(
            owo_colors::AnsiColors::Cyan,
        )),
    },
    listing::Column {
        name: "kind",
        header: "kind",
        cell: |r| r.record_kind.as_str().to_string(),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "status",
        header: "status",
        cell: |r| r.status.clone(),
        paint: listing::ColumnPaint::ByValue(|r| listing::status_hue(&r.status)),
    },
    listing::Column {
        name: "slug",
        header: "slug",
        cell: |r| r.slug.clone(),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "title",
        header: "title",
        cell: |r| r.title.clone(),
        paint: listing::ColumnPaint::Alternate([listing::TITLE_EVEN, listing::TITLE_ODD]),
    },
];

/// The default visible set — slug-free (the SL-037 D4 convention); `--columns …,slug`
/// reveals it.
const KN_DEFAULT: &[&str] = &["id", "kind", "status", "title"];

/// Validate a stringly `--status` set against the cross-kind union vocabulary, via the
/// shared `listing::validate_statuses` (the opt-in surface — each list surface MUST
/// call it itself, mem.pattern.listing.validate-statuses-is-opt-in).
fn validate_statuses(given: &[String]) -> anyhow::Result<()> {
    listing::validate_statuses(given, &union_statuses())
}

/// Project a record to its [`listing::FilterFields`] for the shared substr/regex/status/
/// tag axes — the `backlog::key` precedent.
fn key(r: &KnowledgeRecord) -> listing::FilterFields {
    listing::FilterFields {
        canonical: r.record_kind.canonical_id(r.id),
        slug: r.slug.clone(),
        title: r.title.clone(),
        status: r.status.clone(),
        tags: r.tags.clone(),
    }
}

/// Faithful JSON rows (the prefixed id plus the flat list fields).
fn json_rows(records: &[KnowledgeRecord]) -> Vec<RecordRow> {
    records
        .iter()
        .map(|r| RecordRow {
            id: r.record_kind.canonical_id(r.id),
            record_kind: r.record_kind.as_str(),
            status: r.status.clone(),
            slug: r.slug.clone(),
            title: r.title.clone(),
        })
        .collect()
}

/// The `knowledge list` compute half — cross-kind, on the shared spine. `validate_statuses`
/// guards `--status` against the union vocab; `listing::build` resolves the filter +
/// format. The hide-set is PER-ITEM (`is_hidden(kind, status)`, design §7), which the
/// status-keyed `listing::retain` closure cannot express — so the hide drop is applied
/// here (mirroring retain's reveal rule: `--all` OR any explicit `--status` reveals),
/// then `retain` runs the shared substr/regex/status/tag axes with a no-op hide closure.
/// Rows sort by `(kind ordinal, id)` — the cross-kind grouping (no `needs`/`after`
/// ordering for records). Pure over the read corpus.
fn list_rows(root: &Path, mut args: ListArgs) -> anyhow::Result<String> {
    validate_statuses(&args.status)?;
    let render = args.render;
    let columns = args.columns.take();
    // DRIFT: this reveal rule reproduces `listing::retain`'s status-keyed reveal
    // (`--all` OR any explicit `--status`); if retain's rule changes, change it here too.
    let reveal_hidden = args.all || !args.status.is_empty();
    let (filter, format) = listing::build(args)?;
    let corpus = read_all(root)?;
    // Per-item hide-set (design §7): drop a settled-state record unless revealed. The
    // status-keyed `retain` closure cannot see the kind, so this runs first.
    let visible: Vec<KnowledgeRecord> = corpus
        .into_iter()
        .filter(|r| reveal_hidden || !is_hidden(r.record_kind, &r.status))
        .collect();
    // `retain` runs the remaining shared axes; hide is already applied, so its closure
    // is a no-op (`|_| false`).
    let mut records = listing::retain(visible, &filter, |_| false, key);
    records.sort_by_key(|r| (kind_ordinal(r.record_kind), r.id));
    match format {
        Format::Table => {
            let sel = listing::select_columns(&KN_COLUMNS, KN_DEFAULT, columns.as_deref())?;
            Ok(listing::render_columns(&records, &sel, render))
        }
        Format::Json => listing::json_envelope("knowledge", &json_rows(&records)),
    }
}

/// The cross-kind sort ordinal for a `RecordKind` — `RecordKind::ALL` declaration order
/// (ASM, DEC, QUE, CON), so `list` groups by kind then id.
fn kind_ordinal(kind: RecordKind) -> usize {
    RecordKind::ALL
        .iter()
        .position(|&k| k == kind)
        .unwrap_or(usize::MAX)
}

/// `doctrine knowledge list [CommonListArgs]` — the cross-kind survey verb (design §6),
/// on the shared spine. Thin shell: find the root, lower the args, print the rows
/// verbatim (`render_columns` carries its own trailing newline).
pub(crate) fn run_list(path: Option<PathBuf>, args: ListArgs) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let out = list_rows(&root, args)?;
    write!(io::stdout(), "{out}")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// `knowledge status` — edit-preserving transition (no resolution coupling)
// ---------------------------------------------------------------------------

/// Edit-preserving status transition on one authored `record-NNN.toml` — the
/// `backlog::set_backlog_status` / `adr::set_adr_status` precedent: `toml_edit` mutates
/// the file in place, so the `[facet]`/`[evidence]` tables, hand-added comments, and
/// unknown keys all survive (the file is never reserialised). NO resolution coupling
/// (design §6) — only `status` + `updated` move. Carries the no-op guard (an unchanged
/// status writes nothing) and the malformed-file refuse (a missing seeded `status`/
/// `updated` would let a tail-`insert` land inside the trailing `[facet]` subtable —
/// silent corruption; refuse instead). The date is injected by the shell. A missing
/// record file errors (read fails) — never an implicit create.
fn set_record_status(
    root: &Path,
    kind: RecordKind,
    id: u32,
    status: &str,
    today: &str,
) -> anyhow::Result<()> {
    let name = format!("{id:03}");
    let path = root
        .join(kind.kind().dir)
        .join(&name)
        .join(format!("{RECORD_STEM}-{name}.toml"));
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("record not found at {}", path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    // No-op guard: unchanged status → write nothing (mtime holds).
    if doc.get("status").and_then(toml_edit::Item::as_str) == Some(status) {
        return Ok(());
    }

    // Refuse a malformed (hand-edited) record: `status`/`updated` are scaffold-seeded.
    // Their absence means a tail `insert` would append the key AFTER the trailing
    // `[facet]`/`[evidence]` header, landing it inside that subtable (silent corruption).
    let table = doc.as_table_mut();
    if !table.contains_key("status") || !table.contains_key("updated") {
        anyhow::bail!(
            "malformed record {name}: missing seeded `status`/`updated` (regenerate via `knowledge new`)"
        );
    }
    table.insert("status", toml_edit::value(status));
    table.insert("updated", toml_edit::value(today));
    std::fs::write(&path, doc.to_string())
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

/// `doctrine knowledge status <ID> <state>` — transition one record's status in place
/// (design §6). Thin shell: find the root, `resolve_ref` the id to its kind, validate
/// `<state>` ∈ `statuses(kind)` and **REFUSE a foreign-kind state** (FR-002: a DEC
/// state on an ASM is rejected), then `set_record_status` writes `status` + `updated`
/// (no resolution coupling). Prints the canonical id + the new state.
pub(crate) fn run_status(
    path: Option<PathBuf>,
    reference: &str,
    state: &str,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (kind, id) = resolve_ref(reference)?;
    let vocab = statuses(kind);
    if !vocab.contains(&state) {
        anyhow::bail!(
            "`{state}` is not a {} status (known: {})",
            kind.as_str(),
            vocab.join(", ")
        );
    }
    let today = crate::clock::today();
    set_record_status(&root, kind, id, state, &today)?;
    writeln!(io::stdout(), "{}: {state}", kind.canonical_id(id))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta::Meta;
    use std::collections::BTreeSet;
    use std::path::Path;

    fn ctx_for(kind: RecordKind) -> ScaffoldCtx<'static> {
        let canonical: &'static str = match kind {
            RecordKind::Assumption => "ASM-003",
            RecordKind::Decision => "DEC-003",
            RecordKind::Question => "QUE-003",
            RecordKind::Constraint => "CON-003",
        };
        ScaffoldCtx {
            id: 3,
            canonical,
            slug: "token-expiry",
            title: "Token expiry",
            date: "2026-06-08",
        }
    }

    // --- the discriminator helpers ---

    #[test]
    fn record_kind_from_prefix_round_trips_each_kind() {
        for kind in RecordKind::ALL {
            assert_eq!(RecordKind::from_prefix(kind.prefix()), Some(kind));
        }
        assert_eq!(RecordKind::from_prefix("REQ"), None);
        let prefixes: BTreeSet<&str> = RecordKind::ALL.iter().map(|k| k.prefix()).collect();
        assert_eq!(prefixes.len(), 4, "the four prefixes are distinct");
    }

    #[test]
    fn canonical_id_uses_the_kind_prefix() {
        assert_eq!(RecordKind::Assumption.canonical_id(7), "ASM-007");
        assert_eq!(RecordKind::Decision.canonical_id(12), "DEC-012");
    }

    // --- VT-4: per-kind status known-set + seed-status anti-drift (F-A2) ---

    #[test]
    fn default_status_is_the_first_vocab_element_per_kind() {
        assert_eq!(RecordKind::Assumption.default_status(), "held");
        assert_eq!(RecordKind::Decision.default_status(), "proposed");
        assert_eq!(RecordKind::Question.default_status(), "open");
        assert_eq!(RecordKind::Constraint.default_status(), "active");
        // the seed is exactly statuses(kind)[0] — one source, never a second copy.
        for kind in RecordKind::ALL {
            assert_eq!(Some(kind.default_status()), statuses(kind).first().copied());
        }
    }

    #[test]
    fn status_vocabularies_are_the_expected_known_sets() {
        assert_eq!(
            statuses(RecordKind::Assumption),
            ["held", "testing", "validated", "invalidated", "obsolete"]
        );
        assert_eq!(
            statuses(RecordKind::Decision),
            ["proposed", "accepted", "rejected", "superseded"]
        );
        assert_eq!(
            statuses(RecordKind::Question),
            ["open", "answered", "obsolete"]
        );
        assert_eq!(
            statuses(RecordKind::Constraint),
            ["active", "waived", "superseded", "retired"]
        );
    }

    #[test]
    fn hide_set_is_a_subset_of_the_vocab_and_excludes_the_seed() {
        for kind in RecordKind::ALL {
            let vocab: BTreeSet<&str> = statuses(kind).iter().copied().collect();
            for h in hidden(kind) {
                assert!(vocab.contains(h), "{kind:?}: hidden `{h}` is in-vocab");
                assert!(
                    !is_hidden(kind, kind.default_status()),
                    "{kind:?}: the seed is never hidden"
                );
            }
        }
        // F-A5 precursor: `accepted` (a live decision) is list-visible.
        assert!(!is_hidden(RecordKind::Decision, "accepted"));
        assert!(is_hidden(RecordKind::Decision, "superseded"));
    }

    // --- VT-3: three facet-enum drift canaries (variant set == known-set) ---

    #[test]
    fn confidence_known_set_matches_variants() {
        use clap::ValueEnum;
        let variants: BTreeSet<&str> = Confidence::value_variants()
            .iter()
            .map(|v| v.as_str())
            .collect();
        let known: BTreeSet<&str> = Confidence::KNOWN.iter().copied().collect();
        assert_eq!(variants, known);
    }

    #[test]
    fn basis_known_set_matches_variants() {
        use clap::ValueEnum;
        let variants: BTreeSet<&str> = Basis::value_variants().iter().map(|v| v.as_str()).collect();
        let known: BTreeSet<&str> = Basis::KNOWN.iter().copied().collect();
        assert_eq!(variants, known);
    }

    #[test]
    fn constraint_source_known_set_matches_variants() {
        use clap::ValueEnum;
        let variants: BTreeSet<&str> = ConstraintSource::value_variants()
            .iter()
            .map(|v| v.as_str())
            .collect();
        let known: BTreeSet<&str> = ConstraintSource::KNOWN.iter().copied().collect();
        assert_eq!(variants, known);
    }

    // --- VT-2: the "" / [] -> absent optional seam, per kind ---

    #[test]
    fn seeded_facet_maps_empty_to_absent_per_kind() {
        for kind in RecordKind::ALL {
            let seed = render_record_toml_seed(kind, 1, "s", "T", "2026-06-08").unwrap();
            let record = validate(toml::from_str::<RawRecordToml>(&seed).unwrap()).unwrap();
            // the seeded status is the kind's default (F-A2 — template literal == default_status).
            assert_eq!(
                record.status,
                kind.default_status(),
                "{kind:?}: seeded status"
            );
            // evidence lists default empty.
            assert!(record.evidence.supports.is_empty());
            assert!(record.evidence.contradicts.is_empty());
            assert!(record.evidence.notes.is_empty());
            // every optional facet field maps "" / [] -> absent.
            match &record.facet {
                RecordFacet::Assumption(f) => {
                    assert_eq!(
                        f,
                        &AssumptionFacet::default(),
                        "{kind:?}: empty facet absent"
                    );
                }
                RecordFacet::Decision(f) => {
                    assert_eq!(f, &DecisionFacet::default());
                }
                RecordFacet::Question(f) => {
                    assert_eq!(f, &QuestionFacet::default());
                }
                RecordFacet::Constraint(f) => {
                    assert_eq!(f, &ConstraintFacet::default());
                }
            }
        }
    }

    #[test]
    fn non_empty_facet_enums_parse_to_their_variants() {
        let assessed = "\
id = 1
slug = \"a\"
title = \"A\"
record_kind = \"assumption\"
status = \"testing\"
created = \"2026-06-08\"
updated = \"2026-06-08\"
tags = []

[facet]
claim = \"tokens expire in 1h\"
confidence = \"high\"
basis = \"observation\"
validation_plan = \"probe the IdP\"
validated_by = \"\"
validated_on = \"\"
invalidated_by = \"\"
invalidated_on = \"\"

[evidence]
supports = [\"DEC-005-C\"]
contradicts = []
notes = [\"see the audit\"]
";
        let record = validate(toml::from_str::<RawRecordToml>(assessed).unwrap()).unwrap();
        match record.facet {
            RecordFacet::Assumption(f) => {
                assert_eq!(f.claim.as_deref(), Some("tokens expire in 1h"));
                assert_eq!(f.confidence, Some(Confidence::High));
                assert_eq!(f.basis, Some(Basis::Observation));
                assert_eq!(f.validation_plan.as_deref(), Some("probe the IdP"));
                assert_eq!(f.validated_by, None);
            }
            _ => panic!("expected an assumption facet"),
        }
        assert_eq!(record.evidence.supports, vec!["DEC-005-C"]);
        assert_eq!(record.evidence.notes, vec!["see the audit"]);
    }

    #[test]
    fn validate_errors_on_an_unknown_facet_enum_token() {
        let body = "\
id = 1
slug = \"a\"
title = \"A\"
record_kind = \"assumption\"
status = \"held\"
created = \"2026-06-08\"
updated = \"2026-06-08\"
tags = []

[facet]
confidence = \"bogus\"
";
        let raw: RawRecordToml = toml::from_str(body).unwrap();
        assert!(
            validate(raw).is_err(),
            "an unknown confidence token is rejected"
        );
    }

    // --- VT-1: per-kind byte-stable round-trip (facet + evidence) ---

    /// A fully-populated record-NNN.toml per kind. Round-trips toml -> struct -> toml
    /// byte-stable: the hand-emit (`render_record_toml`) reproduces the on-disk
    /// layout exactly (F1 order, every field present, lists populated).
    fn populated_fixture(kind: RecordKind) -> String {
        let head = format!(
            "schema = \"doctrine.knowledge\"\nversion = 1\n\nid = 7\nslug = \"token-expiry\"\ntitle = \"Token expiry\"\nrecord_kind = \"{}\"\nstatus = {}\ncreated = \"2026-06-08\"\nupdated = \"2026-06-09\"\ntags = [\"auth\", \"security\"]\n",
            kind.as_str(),
            toml_string(kind.default_status()),
        );
        let facet = match kind {
            RecordKind::Assumption => {
                "\n[facet]\nclaim = \"tokens expire in 1h\"\nconfidence = \"high\"\nbasis = \"observation\"\nvalidation_plan = \"probe the IdP\"\nvalidated_by = \"david\"\nvalidated_on = \"2026-06-09\"\ninvalidated_by = \"\"\ninvalidated_on = \"\"\n"
            }
            RecordKind::Decision => {
                "\n[facet]\ncontext = \"the import seam\"\nchoice = \"git cherry\"\nalternatives = [\"--merged\", \"delta-emptiness\"]\nrationale = \"patch-id is sound\"\nconsequences = [\"slower scan\", \"correct\"]\ndecided_by = \"david\"\ndecided_on = \"2026-06-09\"\n"
            }
            RecordKind::Question => {
                "\n[facet]\nquestion = \"do we re-anchor B?\"\nwhy_matters = \"the delta corrupts otherwise\"\nanswer = \"yes, on a disjointness proof\"\nanswered_by = \"david\"\nanswered_on = \"2026-06-09\"\n"
            }
            RecordKind::Constraint => {
                "\n[facet]\nstatement = \"no disk in the pure layer\"\nsource = \"canon\"\napplies_to = [\"src/knowledge.rs\", \"src/backlog.rs\"]\nwaiver_reason = \"\"\nwaived_by = \"\"\nwaived_on = \"\"\n"
            }
        };
        let evidence = "\n[evidence]\nsupports = [\"ADR-001\"]\ncontradicts = []\nnotes = [\"see §5\", \"and §9\"]\n";
        format!("{head}{facet}{evidence}")
    }

    #[test]
    fn populated_record_round_trips_byte_stable_per_kind() {
        for kind in RecordKind::ALL {
            let original = populated_fixture(kind);
            let record = validate(toml::from_str::<RawRecordToml>(&original).unwrap()).unwrap();
            let rendered = render_record_toml(&record);
            assert_eq!(
                rendered, original,
                "{kind:?}: toml -> struct -> toml must be byte-stable"
            );
            // and the struct survives a second parse identically (idempotence).
            let reparsed = validate(toml::from_str::<RawRecordToml>(&rendered).unwrap()).unwrap();
            assert_eq!(
                reparsed, record,
                "{kind:?}: struct stable across the round-trip"
            );
        }
    }

    #[test]
    fn populated_record_round_trips_into_shared_meta() {
        let original = populated_fixture(RecordKind::Decision);
        let meta: Meta = toml::from_str(&original).unwrap();
        assert_eq!(
            meta,
            Meta {
                id: 7,
                slug: "token-expiry".to_string(),
                title: "Token expiry".to_string(),
                status: "proposed".to_string(),
            }
        );
    }

    // --- VT-5: scaffold materialises 2 files + symlink, F1 ordering pinned ---

    #[test]
    fn record_scaffold_lays_out_toml_md_symlink_per_kind() {
        for kind in RecordKind::ALL {
            let ctx = ctx_for(kind);
            let fileset = record_scaffold(kind, &ctx).unwrap();
            assert_eq!(fileset.len(), 3, "{kind:?}: toml + md + symlink");

            let toml_body = match &fileset[0] {
                Artifact::File { rel_path, body } => {
                    assert_eq!(rel_path, Path::new("003/record-003.toml"));
                    body
                }
                Artifact::Symlink { .. } => panic!("first artifact is the toml"),
            };
            // the stored discriminator and the seeded default status.
            assert!(toml_body.contains(&format!("record_kind = \"{}\"", kind.as_str())));
            assert!(
                toml_body.contains(&format!("status = \"{}\"", kind.default_status())),
                "{kind:?}: scaffolded status == default_status (F-A2)"
            );
            // F1 on-disk order: top-level meta -> [facet] -> [evidence]; no relations.
            let facet_at = toml_body.find("[facet]").expect("a [facet] block");
            let evidence_at = toml_body.find("[evidence]").expect("an [evidence] block");
            let tags_at = toml_body.find("tags = []").expect("seeded tags");
            assert!(tags_at < facet_at, "{kind:?}: meta before [facet]");
            assert!(
                facet_at < evidence_at,
                "{kind:?}: [facet] before [evidence]"
            );
            assert!(
                !toml_body.contains("[[relation]]") && !toml_body.contains("[relationships]"),
                "{kind:?}: Slice A seeds no relation block"
            );
            assert!(
                !toml_body.contains("{{"),
                "{kind:?}: no token survives render"
            );

            // the md carries the canonical ref; the symlink is the NNN-slug alias.
            assert!(matches!(
                &fileset[1],
                Artifact::File { rel_path, body }
                if rel_path == Path::new("003/record-003.md")
                    && body.contains(&format!("{}: Token expiry", ctx.canonical))
            ));
            assert!(matches!(
                &fileset[2],
                Artifact::Symlink { rel_path, target }
                if rel_path == Path::new("003-token-expiry") && target == "003"
            ));
        }
    }

    #[test]
    fn scaffold_escapes_hostile_title_and_slug() {
        let title = crate::tomlfmt::HOSTILE_TITLE;
        let slug = crate::tomlfmt::HOSTILE_SLUG;
        let body =
            render_record_toml_seed(RecordKind::Assumption, 7, slug, title, "2026-06-08").unwrap();
        let parsed: Meta = toml::from_str(&body).unwrap();
        assert_eq!(parsed.slug, slug);
        assert_eq!(parsed.title, title);
    }

    #[test]
    fn render_escapes_hostile_facet_values() {
        // a populated record carrying a quoted-literal breaker in a facet text field
        // round-trips through the hand-emit without breaking the document.
        let body = "\
id = 1
slug = \"s\"
title = \"T\"
record_kind = \"decision\"
status = \"proposed\"
created = \"2026-06-08\"
updated = \"2026-06-08\"
tags = []

[facet]
context = \"a\\\"b\"
choice = \"\"
alternatives = [\"x\\\"y\"]
rationale = \"\"
consequences = []
decided_by = \"\"
decided_on = \"\"

[evidence]
supports = []
contradicts = []
notes = []
";
        let record = validate(toml::from_str::<RawRecordToml>(body).unwrap()).unwrap();
        let rendered = render_record_toml(&record);
        // the rendered text re-parses to the same struct (escaping survived).
        let reparsed = validate(toml::from_str::<RawRecordToml>(&rendered).unwrap()).unwrap();
        assert_eq!(reparsed, record);
    }
}
