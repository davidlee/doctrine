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
//! `dep_seq::set_authored_status` (`toml_edit`).

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::dtoml;
use crate::entity::{self, Artifact, Fileset, Inputs, Kind, MaterialiseRequest, ScaffoldCtx};
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
    prefix: crate::kinds::ASM,
    stem: "record",
    scaffold: |c| record_scaffold(RecordKind::Assumption, c),
};

/// The decision kind: a recorded choice and its rationale.
pub(crate) const DECISION_KIND: Kind = Kind {
    dir: ".doctrine/knowledge/decision",
    prefix: crate::kinds::DEC,
    stem: "record",
    scaffold: |c| record_scaffold(RecordKind::Decision, c),
};

/// The question kind: an open question whose answer shapes the work.
pub(crate) const QUESTION_KIND: Kind = Kind {
    dir: ".doctrine/knowledge/question",
    prefix: crate::kinds::QUE,
    stem: "record",
    scaffold: |c| record_scaffold(RecordKind::Question, c),
};

/// The constraint kind: a standing limit on the solution space.
pub(crate) const CONSTRAINT_KIND: Kind = Kind {
    dir: ".doctrine/knowledge/constraint",
    prefix: crate::kinds::CON,
    stem: "record",
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
        listing::canonical_id(self.prefix(), id)
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

    /// Whether `status` is a terminal status for this record kind (D2). An
    /// already-terminal record is not status-flipped during supersession — an
    /// already-`validated` assumption stays `validated`; an `open` question becomes
    /// `obsolete`. Delegates to the per-kind terminal set; an out-of-vocab token is
    /// conservatively treated as terminal (decline to flip unknown status).
    pub(crate) fn is_terminal(self, status: &str) -> bool {
        terminal(self).contains(&status)
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

/// The per-kind terminal status sets (D2, SL-097 PHASE-01) — distinct from the
/// hide-set: `accepted` (decision) is terminal but not hidden. Each is a subset of
/// the kind's status vocabulary. An already-terminal record is not flipped during
/// supersession.
const ASSUMPTION_TERMINAL: &[&str] = &["validated", "invalidated", "obsolete"];
const DECISION_TERMINAL: &[&str] = &["accepted", "rejected", "superseded"];
const QUESTION_TERMINAL: &[&str] = &["answered", "obsolete"];
const CONSTRAINT_TERMINAL: &[&str] = &["waived", "superseded", "retired"];

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

/// The kind's terminal set — the supersession guard (D2, SL-097 PHASE-01).
/// An already-terminal record is not status-flipped during supersession.
const fn terminal(k: RecordKind) -> &'static [&'static str] {
    match k {
        RecordKind::Assumption => ASSUMPTION_TERMINAL,
        RecordKind::Decision => DECISION_TERMINAL,
        RecordKind::Question => QUESTION_TERMINAL,
        RecordKind::Constraint => CONSTRAINT_TERMINAL,
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
    tier1: Vec<crate::relation::RelationEdge>,
    /// Prose body read from the sibling `record-NNN.md`.
    pub(crate) body: String,
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
        tier1: Vec::new(),
        // Filled by `read_record` from the sibling .md; empty otherwise.
        body: String::new(),
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
// (template) + `dep_seq::set_authored_status` (toml_edit). Gated per-fn, not by a blanket
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
    let raw: RawRecordToml = dtoml::parse_entity_toml(&text, kind.prefix(), id)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    let mut record = validate(raw)?;
    record.tier1 = crate::relation::tier1_edges(kind.kind(), &text)?;
    let md_path = root
        .join(kind.kind().dir)
        .join(&name)
        .join(format!("{RECORD_STEM}-{name}.md"));
    record.body = std::fs::read_to_string(&md_path)
        .with_context(|| format!("Failed to read {}", md_path.display()))?;
    Ok(record)
}

/// The kind-module accessor for relation edges (SL-096 PHASE-01): read one record
/// and return its tier-1 relation edges. Delegates to [`read_record`].
pub(crate) fn relation_edges(
    root: &Path,
    kind: RecordKind,
    id: u32,
) -> anyhow::Result<Vec<crate::relation::RelationEdge>> {
    let record = read_record(root, kind, id)?;
    Ok(record.tier1)
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
    let trunk_ids = crate::git::trunk_entity_ids(&root, record_kind.kind().dir)?;
    let (backend, mut reserved) = crate::reserve::backend(
        &root,
        record_kind.kind().prefix,
        crate::install::prompt_confirm,
    )?;
    let title = crate::input::resolve_title(title)?;
    let slug = crate::input::resolve_slug(&title, slug)?;
    let date = crate::clock::today();
    let out = entity::materialise(
        record_kind.kind(),
        &*backend,
        &root,
        &MaterialiseRequest::Fresh,
        &Inputs {
            slug: &slug,
            title: &title,
            date: &date,
        },
        &trunk_ids,
        &mut reserved,
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
// `knowledge show` / `knowledge inspect` — reassemble one record (table | json)
// ---------------------------------------------------------------------------

/// Render the metadata portion of a [`KnowledgeRecord`] — a PURE fn of the record's
/// OWN local state ("cannot go stale"), shared by `format_show` and `format_inspect`.
fn format_metadata(record: &KnowledgeRecord) -> Vec<String> {
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
    // shapes, spawns, governed_by axes
    for label in [
        crate::relation::RelationLabel::Shapes,
        crate::relation::RelationLabel::Spawns,
        crate::relation::RelationLabel::GovernedBy,
    ] {
        let targets = crate::relation::targets_for(&record.tier1, label);
        if !targets.is_empty() {
            let targets_str = targets.join(", ");
            parts.push(format!("{}: [{}]\n", label.name(), targets_str));
        }
    }
    parts
}

/// Render a [`KnowledgeRecord`] for `show` — metadata + prose body.
fn format_show(record: &KnowledgeRecord) -> String {
    let mut parts = format_metadata(record);
    parts.push(format!("\n{}", record.body));
    parts.concat()
}

/// Render a [`KnowledgeRecord`] for `inspect` — metadata only, no prose body.
fn format_inspect(record: &KnowledgeRecord) -> String {
    format_metadata(record).concat()
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

/// Render the `Json` for show (`with_body=true`) or inspect (`with_body=false`).
/// The shared `{kind, …}` envelope (the `backlog::show_json` precedent). The validated
/// record's fields are private and its closed enums render via `as_str`, so the JSON is
/// projected by hand (not a derive): the flat identity, the kind-dispatched `[facet]`,
/// and the shared `[evidence]`. Pure over the record's own state (no cross-corpus
/// scan). `serde_json` sorts object keys.
fn show_json(record: &KnowledgeRecord, with_body: bool) -> anyhow::Result<String> {
    let mut inner = serde_json::Map::new();
    inner.insert(
        "id".into(),
        serde_json::json!(record.record_kind.canonical_id(record.id)),
    );
    inner.insert(
        "record_kind".into(),
        serde_json::json!(record.record_kind.as_str()),
    );
    inner.insert("slug".into(), serde_json::json!(record.slug));
    inner.insert("title".into(), serde_json::json!(record.title));
    inner.insert("status".into(), serde_json::json!(record.status));
    inner.insert("created".into(), serde_json::json!(record.created));
    inner.insert("updated".into(), serde_json::json!(record.updated));
    inner.insert("tags".into(), serde_json::json!(record.tags));
    if with_body {
        inner.insert("body".into(), serde_json::json!(record.body));
    }
    inner.insert("facet".into(), serde_json::json!(facet_json(&record.facet)));
    inner.insert(
        "evidence".into(),
        serde_json::json!({
            "supports": record.evidence.supports,
            "contradicts": record.evidence.contradicts,
            "notes": record.evidence.notes,
        }),
    );
    inner.insert("relationships".into(), serde_json::json!({
        "shapes": crate::relation::targets_for(&record.tier1, crate::relation::RelationLabel::Shapes),
        "spawns": crate::relation::targets_for(&record.tier1, crate::relation::RelationLabel::Spawns),
        "governed_by": crate::relation::targets_for(&record.tier1, crate::relation::RelationLabel::GovernedBy),
    }));
    let value = serde_json::json!({
        "kind": "knowledge",
        "knowledge": inner,
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

/// Shared shell: root-find → resolve → read → render. The `format_table` fn and
/// `with_body` flag select the table renderer and whether JSON includes the prose body.
fn run_show_inspect(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
    format_table: fn(&KnowledgeRecord) -> String,
    with_body: bool,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (kind, id) = resolve_ref(reference)?;
    let record = read_record(&root, kind, id)?;
    let out = match format {
        Format::Table => format_table(&record),
        Format::Json => show_json(&record, with_body)?,
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
}

/// `doctrine knowledge show <ID> [--format table|json]` — metadata + prose body.
/// Thin shell: find the root, `resolve_ref` the id to its kind (prefix auto-detect),
/// read THAT record's single toml, render it to stdout. READ-ONLY — no mutation, no
/// cross-corpus scan (only the one record's file is opened).
pub(crate) fn run_show(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
) -> anyhow::Result<()> {
    run_show_inspect(path, reference, format, format_show, true)
}

/// `doctrine knowledge inspect <ID> [--format table|json]` — metadata only, no prose
/// body. Thin shell: same read path as `show`, rendered via `format_inspect`.
pub(crate) fn run_inspect(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
) -> anyhow::Result<()> {
    run_show_inspect(path, reference, format, format_inspect, false)
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

/// `doctrine knowledge status <ID> <state>` — transition one record's status in place
/// (design §6). Thin shell: find the root, `resolve_ref` the id to its kind, validate
/// `<state>` ∈ `statuses(kind)` and **REFUSE a foreign-kind state** (FR-002: a DEC
/// state on an ASM is rejected), then the shared `dep_seq::set_authored_status` writes
/// `status` + `updated` (no resolution coupling). Prints the canonical id + the new state.
pub(crate) fn run_status(
    path: Option<PathBuf>,
    reference: &str,
    state: &str,
    color: bool,
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
    let name = format!("{id:03}");
    let record_path = root
        .join(kind.kind().dir)
        .join(&name)
        .join(format!("{RECORD_STEM}-{name}.toml"));
    let hint = format!(
        "malformed record {name}: missing seeded `status`/`updated` \
         — restore the missing keys and retry; the file is left untouched"
    );
    crate::dep_seq::set_authored_status(
        &record_path,
        &[("status", state), ("updated", &today)],
        &hint,
    )?;
    writeln!(
        io::stdout(),
        "{}: {}",
        kind.canonical_id(id),
        crate::listing::status_colored(state, color)
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// `knowledge paths` — file paths for each knowledge record entity directory
// ---------------------------------------------------------------------------

/// `doctrine knowledge paths <ref>…` — resolve each ref to its entity directory
/// and print the root-relative paths according to the selection.
fn run_paths(
    path: Option<PathBuf>,
    refs: &[String],
    sel: &crate::paths::PathSelection,
) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let mut all_lines: Vec<String> = Vec::new();
    for r in refs {
        let (kind, id) = resolve_ref(r)?;
        let name = format!("{id:03}");
        let entity_dir = root.join(kind.kind().dir).join(&name);
        let toml_name = format!("{RECORD_STEM}-{name}.toml");
        let md_name = format!("{RECORD_STEM}-{name}.md");
        let set = crate::paths::scan_entity_dir(
            &entity_dir,
            &entity_dir.join(&toml_name),
            Some(&entity_dir.join(&md_name)),
            &root,
        )?;
        let lines = crate::paths::select_paths(&set, sel)?;
        all_lines.extend(lines);
    }
    write!(io::stdout(), "{}", all_lines.join("\n"))?;
    Ok(())
}

// ── CLI dispatch ───────────────────────────────────────────────────────────

use crate::CommonListArgs;
use clap::Subcommand;

#[derive(Subcommand)]
pub(crate) enum KnowledgeCommand {
    /// Create a new knowledge record (assumption / decision / question / constraint).
    New {
        kind: RecordKind,
        title: Option<String>,
        #[arg(long)]
        slug: Option<String>,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
    /// List knowledge records.
    List {
        #[command(flatten)]
        list: CommonListArgs,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
    /// Show one knowledge record (metadata + prose body).
    Show {
        #[command(flatten)]
        common: crate::CommonShowArgs,
    },
    /// Inspect one knowledge record's metadata only (no prose body).
    Inspect {
        #[command(flatten)]
        common: crate::CommonShowArgs,
    },
    /// Set a knowledge record's status.
    Status {
        id: String,
        state: String,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Print the file paths of each knowledge record entity directory.
    Paths {
        /// Knowledge record reference(s) — `ASM-007`, `DEC-012`, etc.
        refs: Vec<String>,

        /// Show only the identity TOML file.
        #[arg(short = 't', long)]
        toml: bool,
        /// Show only the identity Markdown body.
        #[arg(short = 'm', long)]
        md: bool,
        /// Show the identity TOML + Markdown (equivalent to -t -m).
        #[arg(short = 'e', long)]
        entity: bool,
        /// Return only the first (primary) path per ref.
        #[arg(short = 's', long)]
        single: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

pub(crate) fn dispatch(cmd: KnowledgeCommand, color: bool) -> anyhow::Result<()> {
    match cmd {
        KnowledgeCommand::New {
            kind,
            title,
            slug,
            path,
        } => run_new(path, kind, title, slug),
        KnowledgeCommand::List { list, path } => run_list(path, list.into_list_args(color)),
        KnowledgeCommand::Show { common } => {
            let format = if common.json {
                Format::Json
            } else {
                common.format
            };
            run_show(common.path, &common.id, format)
        }
        KnowledgeCommand::Inspect { common } => {
            let format = if common.json {
                Format::Json
            } else {
                common.format
            };
            run_inspect(common.path, &common.id, format)
        }
        KnowledgeCommand::Status { id, state, path } => run_status(path, &id, &state, color),
        KnowledgeCommand::Paths {
            refs,
            toml,
            md,
            entity,
            single,
            path,
        } => run_paths(
            path,
            &refs,
            &crate::paths::PathSelection {
                toml,
                md,
                entity,
                single,
            },
        ),
    }
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

    // --- VT-1/VT-2/VT-3/VT-4: per-kind terminal predicate (D2, SL-097) ---

    #[test]
    fn is_terminal_returns_correct_per_kind() {
        // Assumption (VT-1)
        assert!(!RecordKind::Assumption.is_terminal("held"));
        assert!(!RecordKind::Assumption.is_terminal("testing"));
        assert!(RecordKind::Assumption.is_terminal("validated"));
        assert!(RecordKind::Assumption.is_terminal("invalidated"));
        assert!(RecordKind::Assumption.is_terminal("obsolete"));
        // Decision (VT-2)
        assert!(!RecordKind::Decision.is_terminal("proposed"));
        assert!(RecordKind::Decision.is_terminal("accepted"));
        assert!(RecordKind::Decision.is_terminal("rejected"));
        assert!(RecordKind::Decision.is_terminal("superseded"));
        // Question (VT-3)
        assert!(!RecordKind::Question.is_terminal("open"));
        assert!(RecordKind::Question.is_terminal("answered"));
        assert!(RecordKind::Question.is_terminal("obsolete"));
        // Constraint (VT-4)
        assert!(!RecordKind::Constraint.is_terminal("active"));
        assert!(RecordKind::Constraint.is_terminal("waived"));
        assert!(RecordKind::Constraint.is_terminal("superseded"));
        assert!(RecordKind::Constraint.is_terminal("retired"));
    }

    #[test]
    fn terminal_set_is_subset_of_the_vocab_and_excludes_the_seed() {
        for kind in RecordKind::ALL {
            let vocab: BTreeSet<&str> = statuses(kind).iter().copied().collect();
            for t in terminal(kind) {
                assert!(vocab.contains(t), "{kind:?}: terminal `{t}` is in-vocab");
            }
            assert!(
                !kind.is_terminal(kind.default_status()),
                "{kind:?}: the seed is never terminal"
            );
        }
        // `accepted` is terminal (D2) but not hidden (F-A5 precursor).
        assert!(RecordKind::Decision.is_terminal("accepted"));
        assert!(!is_hidden(RecordKind::Decision, "accepted"));
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
                tags: vec!["auth".to_string(), "security".to_string()],
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
            // F1 on-disk order: meta -> [facet] -> [evidence] -> [relationships].
            let facet_at = toml_body.find("[facet]").expect("a [facet] block");
            let evidence_at = toml_body.find("[evidence]").expect("an [evidence] block");
            let tags_at = toml_body.find("tags = []").expect("seeded tags");
            let relationships_at = toml_body
                .find("[relationships]")
                .expect("a [relationships] block");
            assert!(tags_at < facet_at, "{kind:?}: meta before [facet]");
            assert!(
                facet_at < evidence_at,
                "{kind:?}: [facet] before [evidence]"
            );
            assert!(
                evidence_at < relationships_at,
                "{kind:?}: [evidence] before [relationships]"
            );
            assert!(
                !toml_body.contains("[[relation]]"),
                "{kind:?}: Slice A seeds no [[relation]] block"
            );
            assert!(
                toml_body.contains("supersedes    = []"),
                "{kind:?}: seeded supersedes"
            );
            assert!(
                toml_body.contains("superseded_by = []"),
                "{kind:?}: seeded superseded_by"
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

    // --- VT-6: tier1 relation edges via read_record (SL-096 PHASE-01) ---

    fn seed_record(root: &Path, kind: RecordKind, id: u32, body: &str) {
        let name = format!("{id:03}");
        let dir = root.join(kind.kind().dir).join(&name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("record-{name}.toml")), body).unwrap();
        std::fs::write(
            dir.join(format!("record-{name}.md")),
            format!("# {}: Test\n", kind.canonical_id(id)),
        )
        .unwrap();
    }

    #[test]
    fn record_without_relation_block_has_empty_tier1() {
        let root = std::env::temp_dir().join("doctrine-sl096-pt1-empty");
        let _ = std::fs::remove_dir_all(&root);
        let record = "\
schema = \"doctrine.knowledge\"
version = 1

id = 1
slug = \"test\"
title = \"Test\"
record_kind = \"assumption\"
status = \"held\"
created = \"2026-06-08\"
updated = \"2026-06-08\"
tags = []

[facet]

[evidence]
supports = []
contradicts = []
notes = []
";
        seed_record(&root, RecordKind::Assumption, 1, record);
        let r = read_record(&root, RecordKind::Assumption, 1).unwrap();
        assert!(r.tier1.is_empty(), "no [[relation]] block → empty tier1");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn record_with_authored_relation_rows_populates_tier1() {
        let root = std::env::temp_dir().join("doctrine-sl096-pt1-auth");
        let _ = std::fs::remove_dir_all(&root);
        let record = "\
schema = \"doctrine.knowledge\"
version = 1

id = 1
slug = \"test\"
title = \"Test\"
record_kind = \"assumption\"
status = \"held\"
created = \"2026-06-08\"
updated = \"2026-06-08\"
tags = []

[facet]

[evidence]
supports = []
contradicts = []
notes = []

[[relation]]
label = \"shapes\"
target = \"SL-001\"

[[relation]]
label = \"spawns\"
target = \"ISS-001\"

[[relation]]
label = \"governed_by\"
target = \"ADR-001\"
";
        seed_record(&root, RecordKind::Assumption, 1, record);
        let r = read_record(&root, RecordKind::Assumption, 1).unwrap();
        assert_eq!(r.tier1.len(), 3);
        assert_eq!(r.tier1[0].label, crate::relation::RelationLabel::Shapes);
        assert_eq!(r.tier1[0].target, "SL-001");
        assert_eq!(r.tier1[1].label, crate::relation::RelationLabel::Spawns);
        assert_eq!(r.tier1[1].target, "ISS-001");
        assert_eq!(r.tier1[2].label, crate::relation::RelationLabel::GovernedBy);
        assert_eq!(r.tier1[2].target, "ADR-001");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn record_with_illegal_label_excludes_illegal_from_tier1() {
        let root = std::env::temp_dir().join("doctrine-sl096-pt1-illegal");
        let _ = std::fs::remove_dir_all(&root);
        let record = "\
schema = \"doctrine.knowledge\"
version = 1

id = 1
slug = \"test\"
title = \"Test\"
record_kind = \"assumption\"
status = \"held\"
created = \"2026-06-08\"
updated = \"2026-06-08\"
tags = []

[facet]

[evidence]
supports = []
contradicts = []
notes = []

[[relation]]
label = \"supersedes\"
target = \"SL-001\"

[[relation]]
label = \"shapes\"
target = \"PRD-001\"
";
        seed_record(&root, RecordKind::Assumption, 1, record);
        let r = read_record(&root, RecordKind::Assumption, 1).unwrap();
        assert_eq!(
            r.tier1.len(),
            2,
            "supersedes now has a RECORD rule (LifecycleOnly), shapes is Writable — both in tier1"
        );
        assert_eq!(r.tier1[0].label, crate::relation::RelationLabel::Supersedes);
        assert_eq!(r.tier1[0].target, "SL-001");
        assert_eq!(r.tier1[1].label, crate::relation::RelationLabel::Shapes);
        assert_eq!(r.tier1[1].target, "PRD-001");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn record_with_unknown_label_excludes_unknown_from_tier1() {
        let root = std::env::temp_dir().join("doctrine-sl096-pt1-unknown");
        let _ = std::fs::remove_dir_all(&root);
        let record = "\
schema = \"doctrine.knowledge\"
version = 1

id = 1
slug = \"test\"
title = \"Test\"
record_kind = \"assumption\"
status = \"held\"
created = \"2026-06-08\"
updated = \"2026-06-08\"
tags = []

[facet]

[evidence]
supports = []
contradicts = []
notes = []

[[relation]]
label = \"nonsense\"
target = \"X\"

[[relation]]
label = \"governed_by\"
target = \"ADR-001\"
";
        seed_record(&root, RecordKind::Assumption, 1, record);
        let r = read_record(&root, RecordKind::Assumption, 1).unwrap();
        assert_eq!(
            r.tier1.len(),
            1,
            "unknown nonsense label excluded, governed_by survives"
        );
        assert_eq!(r.tier1[0].label, crate::relation::RelationLabel::GovernedBy);
        assert_eq!(r.tier1[0].target, "ADR-001");
        let _ = std::fs::remove_dir_all(&root);
    }

    // --- PHASE-04 paths verb golden tests ---

    /// Scaffold one knowledge record entity dir with identity files + optional extras.
    fn record_fixture(root: &Path, kind: RecordKind, id: u32, extra: &[&str]) {
        let name = format!("{id:03}");
        let dir = root.join(kind.kind().dir).join(&name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{RECORD_STEM}-{name}.toml")), "toml").unwrap();
        std::fs::write(dir.join(format!("{RECORD_STEM}-{name}.md")), "md").unwrap();
        for e in extra {
            std::fs::write(dir.join(e), e).unwrap();
        }
    }

    #[test]
    fn paths_full_shows_toml_md_and_extras_in_canonical_order() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        record_fixture(root, RecordKind::Assumption, 1, &["notes.md", "z.log"]);
        let sel = crate::paths::PathSelection {
            toml: false,
            md: false,
            entity: false,
            single: false,
        };
        let entity_dir = root.join(RecordKind::Assumption.kind().dir).join("001");
        let identity_toml = entity_dir.join("record-001.toml");
        let identity_md = entity_dir.join("record-001.md");
        let set =
            crate::paths::scan_entity_dir(&entity_dir, &identity_toml, Some(&identity_md), root)
                .unwrap();
        let lines = crate::paths::select_paths(&set, &sel).unwrap();
        let output = lines.join("\n");
        assert!(output.contains(".doctrine/knowledge/assumption/001/record-001.toml"));
        assert!(output.contains(".doctrine/knowledge/assumption/001/record-001.md"));
        assert!(output.contains(".doctrine/knowledge/assumption/001/notes.md"));
        assert!(output.contains(".doctrine/knowledge/assumption/001/z.log"));
    }

    #[test]
    fn paths_single_truncates_to_first() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        record_fixture(root, RecordKind::Decision, 1, &["notes.md"]);
        let sel = crate::paths::PathSelection {
            toml: false,
            md: false,
            entity: false,
            single: true,
        };
        let entity_dir = root.join(RecordKind::Decision.kind().dir).join("001");
        let identity_toml = entity_dir.join("record-001.toml");
        let identity_md = entity_dir.join("record-001.md");
        let set =
            crate::paths::scan_entity_dir(&entity_dir, &identity_toml, Some(&identity_md), root)
                .unwrap();
        let lines = crate::paths::select_paths(&set, &sel).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], ".doctrine/knowledge/decision/001/record-001.toml");
    }

    #[test]
    fn paths_toml_only() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        record_fixture(root, RecordKind::Question, 2, &["notes.md"]);
        let sel = crate::paths::PathSelection {
            toml: true,
            md: false,
            entity: false,
            single: false,
        };
        let entity_dir = root.join(RecordKind::Question.kind().dir).join("002");
        let identity_toml = entity_dir.join("record-002.toml");
        let identity_md = entity_dir.join("record-002.md");
        let set =
            crate::paths::scan_entity_dir(&entity_dir, &identity_toml, Some(&identity_md), root)
                .unwrap();
        let lines = crate::paths::select_paths(&set, &sel).unwrap();
        assert_eq!(
            lines,
            vec![".doctrine/knowledge/question/002/record-002.toml"]
        );
    }

    #[test]
    fn paths_md_only() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        record_fixture(root, RecordKind::Constraint, 3, &[]);
        let sel = crate::paths::PathSelection {
            toml: false,
            md: true,
            entity: false,
            single: false,
        };
        let entity_dir = root.join(RecordKind::Constraint.kind().dir).join("003");
        let identity_toml = entity_dir.join("record-003.toml");
        let identity_md = entity_dir.join("record-003.md");
        let set =
            crate::paths::scan_entity_dir(&entity_dir, &identity_toml, Some(&identity_md), root)
                .unwrap();
        let lines = crate::paths::select_paths(&set, &sel).unwrap();
        assert_eq!(
            lines,
            vec![".doctrine/knowledge/constraint/003/record-003.md"]
        );
    }

    #[test]
    fn paths_entity_gives_toml_and_md() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        record_fixture(root, RecordKind::Assumption, 4, &["extra.txt"]);
        let sel = crate::paths::PathSelection {
            toml: false,
            md: false,
            entity: true,
            single: false,
        };
        let entity_dir = root.join(RecordKind::Assumption.kind().dir).join("004");
        let identity_toml = entity_dir.join("record-004.toml");
        let identity_md = entity_dir.join("record-004.md");
        let set =
            crate::paths::scan_entity_dir(&entity_dir, &identity_toml, Some(&identity_md), root)
                .unwrap();
        let lines = crate::paths::select_paths(&set, &sel).unwrap();
        assert_eq!(
            lines,
            vec![
                ".doctrine/knowledge/assumption/004/record-004.toml",
                ".doctrine/knowledge/assumption/004/record-004.md"
            ]
        );
    }

    #[test]
    fn paths_invalid_ref_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        record_fixture(root, RecordKind::Assumption, 1, &[]);
        let (_, id) = resolve_ref("ASM-99999").unwrap();
        let entity_dir = root
            .join(RecordKind::Assumption.kind().dir)
            .join(format!("{id:03}"));
        let identity_toml = entity_dir.join(format!("record-{id:03}.toml"));
        let identity_md = entity_dir.join(format!("record-{id:03}.md"));
        let scan =
            crate::paths::scan_entity_dir(&entity_dir, &identity_toml, Some(&identity_md), root);
        assert!(scan.is_err());
    }

    #[test]
    fn paths_multi_ref_splat_preserves_order() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        record_fixture(root, RecordKind::Assumption, 1, &[]);
        record_fixture(root, RecordKind::Decision, 1, &[]);
        let sel = crate::paths::PathSelection {
            toml: false,
            md: false,
            entity: false,
            single: false,
        };
        let mut all_lines: Vec<String> = Vec::new();
        for (kind, n) in [
            (RecordKind::Assumption, "001"),
            (RecordKind::Decision, "001"),
        ] {
            let entity_dir = root.join(kind.kind().dir).join(n);
            let toml_name = format!("{RECORD_STEM}-{n}.toml");
            let md_name = format!("{RECORD_STEM}-{n}.md");
            let set = crate::paths::scan_entity_dir(
                &entity_dir,
                &entity_dir.join(&toml_name),
                Some(&entity_dir.join(&md_name)),
                root,
            )
            .unwrap();
            all_lines.extend(crate::paths::select_paths(&set, &sel).unwrap());
        }
        assert_eq!(all_lines.len(), 4);
        assert!(all_lines[0].contains("assumption/001/record-001.toml"));
        assert!(all_lines[2].contains("decision/001/record-001.toml"));
    }

    // --- VT-7 (SL-158 D3): estimate round-trip on a record ---
    // `[estimate]` on a knowledge record TOML is silently tolerated by
    // `RawRecordToml` (no `deny_unknown_fields`), so the parse succeeds and
    // `estimate::parse_optional` reads the bounds back clean. The full validate
    // pass ignores the table — table ignored, not rejected.

    #[test]
    fn estimate_roundtrip_on_record() {
        let toml = "schema = \"doctrine.knowledge\"\n\
                     version = 1\n\
                     id = 1\n\
                     slug = \"test\"\n\
                     title = \"Test\"\n\
                     record_kind = \"assumption\"\n\
                     status = \"held\"\n\
                     created = \"2026-01-01\"\n\
                     updated = \"2026-01-01\"\n\
                     tags = []\n\
                     [facet]\n\
                     claim = \"x\"\n\
                     [evidence]\n\
                     [estimate]\n\
                     lower = 3.0\n\
                     upper = 3.0\n";
        // parse_entity_toml tolerates the unknown [estimate] table (no deny_unknown_fields).
        let raw: RawRecordToml = crate::dtoml::parse_entity_toml(toml, "ASM", 1).unwrap();
        assert_eq!(raw.id, 1);
        assert_eq!(raw.record_kind, RecordKind::Assumption);
        assert_eq!(raw.title, "Test");

        // Extract and parse the [estimate] sub-table via the pure estimate path.
        let full: toml::Table = toml.parse().unwrap();
        let est_table = full.get("estimate").and_then(|v| v.as_table());
        let facet = crate::estimate::parse_optional(est_table)
            .unwrap()
            .expect("estimate should be present");
        assert_eq!(facet.lower, 3.0);
        assert_eq!(facet.upper, 3.0);

        // Full validate is clean — [estimate] is ignored, not rejected.
        let record = validate(raw).unwrap();
        assert_eq!(record.title, "Test");
        assert_eq!(record.record_kind, RecordKind::Assumption);
    }
}
