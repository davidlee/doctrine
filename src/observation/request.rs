// SPDX-License-Identifier: GPL-3.0-only
//! Caller-request adaptation for observation capture (IMP-332, design §3.1/§3.3).
//!
//! Both capture adapters accept a caller-supplied request; neither may own the
//! parsing, because they sit in the SAME tier and cannot import one another
//! (ADR-001 severs the `mcp_server → commands` back edge). So the shape lives
//! here, below both.
//!
//! Separate from `wire` on purpose. `wire` is the pure typed domain — envelopes,
//! facets, validation, canonical TOML serialization — and carries no
//! external-format dependency beyond `serde` derive. Request adaptation is where
//! `serde_json` and the CLI's flag grammar belong, so keeping them out of `wire`
//! keeps the domain module free of wire-format concerns.
//!
//! No clock, RNG, disk, environment, terminal, or MCP imports.

use serde::Deserialize;
use serde_json::{Value, json};

use crate::observation::wire::{self, EscapeContext, Facets};

/// The complete caller-supplied friction request (design §3.3).
///
/// One shape for both adapters: the MCP tool receives it as the JSON-RPC
/// `arguments` object, and the CLI's `--input` reads the identical object from a
/// file or stdin. `enrich` defaults to true — absent means "enrich", matching
/// the MCP surface and the CLI's opt-out `--no-enrich`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FrictionRequest {
    /// Caller-supplied UUID. Absent means the adapter generates one.
    #[serde(default)]
    pub(crate) uid: Option<String>,
    /// The friction summary. Required and non-empty; the wire validator is what
    /// enforces non-empty, so this type stays a pure shape.
    pub(crate) summary: String,
    /// Optional detail.
    #[serde(default)]
    pub(crate) detail: Option<String>,
    /// Optional explicit facets, merged over automatic enrichment.
    #[serde(default)]
    pub(crate) facets: Option<Facets>,
    /// Whether to apply automatic enrichment. Absent means yes.
    #[serde(default = "enrich_default")]
    pub(crate) enrich: bool,
}

/// Absent `enrich` means "enrich" on both surfaces.
fn enrich_default() -> bool {
    true
}

impl FrictionRequest {
    /// Parse a complete request from JSON text (the CLI `--input` payload).
    ///
    /// Returns the escaped serde message on failure — the caller chose the text,
    /// so it is untrusted and cannot be allowed to forge a second line.
    pub(crate) fn from_json(raw: &str) -> Result<Self, String> {
        serde_json::from_str::<Value>(raw)
            .map_err(|e| escape(&e.to_string()))
            .and_then(Self::from_value)
    }

    /// Parse a complete request from an already-decoded JSON value.
    ///
    /// Facet `schema_version` is defaulted first, so a caller need not repeat it
    /// in every group here any more than over MCP.
    pub(crate) fn from_value(mut raw: Value) -> Result<Self, String> {
        if let Some(facets) = raw.get_mut("facets") {
            *facets = default_facet_schema_versions(facets.take());
        }
        serde_json::from_value(raw).map_err(|e| escape(&e.to_string()))
    }
}

/// Parse caller-supplied explicit facets from a raw JSON value.
///
/// `None` and `null` both mean "no explicit facets" — absent is not an error.
/// Every other shape is deserialized against the real [`Facets`] types, whose
/// `deny_unknown_fields` is what rejects an unknown group or field. The
/// validator is the struct definition, so this cannot drift from it.
pub(crate) fn parse_explicit_facets(raw: Option<&Value>) -> Result<Option<Facets>, String> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    if raw.is_null() {
        return Ok(None);
    }
    let facets: Facets = serde_json::from_value(default_facet_schema_versions(raw.clone()))
        .map_err(|e| escape(&e.to_string()))?;
    Ok(Some(facets))
}

/// Default each supplied facet group's `schema_version` to the current wire
/// version, so a caller need not repeat it in every group.
///
/// Only ever fills an ABSENT key — an explicit `schema_version` (including a
/// wrong one, which the wire then rejects) is left exactly as sent.
pub(crate) fn default_facet_schema_versions(mut raw: Value) -> Value {
    if let Some(groups) = raw.as_object_mut() {
        for group in groups.values_mut() {
            if let Some(obj) = group.as_object_mut() {
                obj.entry("schema_version")
                    .or_insert_with(|| json!(wire::SCHEMA_VERSION));
            }
        }
    }
    raw
}

/// The one wording for a `--facet` argument that is not `group.field=value`
/// (STD-001) — raised in exactly one place so the diagnostic cannot drift.
pub(crate) const FACET_SHAPE: &str = "expected `group.field=value`";

/// Build explicit [`Facets`] from repeatable dotted `group.field=value` pairs
/// (the CLI `--facet` grammar, design §3.1).
///
/// Values are taken as **strings**. Of the 30 caller-settable facet fields, 26
/// are strings; the four that are not (`usage.{total,prompt,completion}_tokens`
/// and `correlation.related_observations`) are the machine-emitted ones, and a
/// program producing them has a complete request to send through `--input`
/// anyway. So this grammar stays unambiguous — no quoting rules, no inference —
/// and a `--facet` attempt at a typed field earns serde's own type error naming
/// the field and what it expected.
///
/// Unknown groups and unknown fields are rejected the same way, by
/// `deny_unknown_fields` on the facet types rather than by a list here that
/// could fall out of step with them.
///
/// An empty slice yields `None`, not an empty `Facets`: "the caller supplied no
/// facets" and "the caller supplied an empty facet set" are the same fact, and
/// the merge treats `None` as "nothing to shadow".
pub(crate) fn facets_from_dotted(pairs: &[String]) -> Result<Option<Facets>, String> {
    if pairs.is_empty() {
        return Ok(None);
    }

    let mut root = serde_json::Map::new();
    for pair in pairs {
        let (group, field, value) = split_pair(pair)?;
        root.entry(group.to_owned())
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .ok_or_else(|| format!("`{}`: {FACET_SHAPE}", escape(pair)))?
            .insert(field.to_owned(), json!(value));
    }

    parse_explicit_facets(Some(&Value::Object(root))).map_err(|aggregate| blame(pairs, aggregate))
}

/// Split one `group.field=value` argument, refusing every degenerate shape.
///
/// Only the FIRST `=` and the first `.` before it are separators, so a value may
/// itself contain both — `execution.command=doctrine a=b.c` is one command
/// string, not a parse error.
fn split_pair(pair: &str) -> Result<(&str, &str, &str), String> {
    let shape = || format!("`{}`: {FACET_SHAPE}", escape(pair));
    let (key, value) = pair.split_once('=').ok_or_else(shape)?;
    let (group, field) = key.split_once('.').ok_or_else(shape)?;
    if group.is_empty() || field.is_empty() {
        return Err(shape());
    }
    Ok((group, field, value))
}

/// Name the argument responsible for a facet-parse failure.
///
/// `serde_json::from_value` reports an unknown key by name but a **type**
/// mismatch only as `invalid type: string "500", expected u64` — no field path.
/// That is precisely the failure a caller reaching for one of the four
/// non-string fields will hit, so the aggregate message alone would leave them
/// without the one fact they need. Re-parsing each argument alone is what
/// recovers it: whichever fails in isolation is the offender.
///
/// Only ever runs on the failure path, so the happy path stays a single parse.
/// Falls back to the aggregate message if no single argument reproduces it.
fn blame(pairs: &[String], aggregate: String) -> String {
    for pair in pairs {
        let Ok((group, field, value)) = split_pair(pair) else {
            continue;
        };
        let one = json!({ group: { field: value } });
        if let Err(reason) = parse_explicit_facets(Some(&one)) {
            return format!("`{}`: {reason}{}", escape(pair), TYPED_FIELD_HINT);
        }
    }
    aggregate
}

/// Where to go when a facet field will not take a string (STD-001).
///
/// `--facet` values are strings by construction, so the four non-string fields
/// are unreachable from it. This names the way through rather than leaving the
/// caller to infer one from a serde type error.
const TYPED_FIELD_HINT: &str =
    " (--facet values are strings; send a non-string field in a complete request via --input)";

/// Escape caller-chosen text before it reaches a diagnostic.
///
/// Every error in this module embeds something the caller wrote — a flag, a
/// serde message echoing a field name. The refusal occupies one logical line, so
/// content must not be able to forge a second.
fn escape(s: &str) -> String {
    wire::escape_hostile(s, EscapeContext::Inline)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::wire::Origin;

    // ── Dotted `--facet` grammar ──────────────────────────────────────────

    #[test]
    fn dotted_pairs_populate_the_named_group_and_field() {
        let facets = facets_from_dotted(&["execution.harness=claude".to_owned()])
            .expect("valid pair parses")
            .expect("a pair yields facets");
        let execution = facets.execution.expect("execution group present");
        assert_eq!(execution.harness.as_deref(), Some("claude"));
    }

    #[test]
    fn repeated_pairs_accumulate_across_and_within_groups() {
        let facets = facets_from_dotted(&[
            "execution.harness=claude".to_owned(),
            "execution.model=opus".to_owned(),
            "work_context.slice=SL-231".to_owned(),
        ])
        .expect("valid pairs parse")
        .expect("pairs yield facets");

        let execution = facets.execution.expect("execution group present");
        assert_eq!(execution.harness.as_deref(), Some("claude"));
        assert_eq!(execution.model.as_deref(), Some("opus"));
        assert_eq!(
            facets
                .work_context
                .expect("work_context group present")
                .slice
                .as_deref(),
            Some("SL-231")
        );
    }

    #[test]
    fn no_pairs_yields_no_facets() {
        assert!(
            facets_from_dotted(&[]).expect("empty parses").is_none(),
            "an empty pair list must mean `no explicit facets`, not an empty set"
        );
    }

    #[test]
    fn a_value_may_itself_contain_equals_and_dots() {
        let facets = facets_from_dotted(&["execution.command=doctrine a=b.c".to_owned()])
            .expect("only the FIRST `=` and first `.` are separators")
            .expect("a pair yields facets");
        assert_eq!(
            facets
                .execution
                .expect("execution group present")
                .command
                .as_deref(),
            Some("doctrine a=b.c")
        );
    }

    #[test]
    fn unknown_group_is_refused_by_the_facet_types() {
        let err = facets_from_dotted(&["nonsuch.field=v".to_owned()])
            .expect_err("an unknown group must not be silently dropped");
        assert!(
            err.contains("nonsuch"),
            "the refusal must name the offending group: {err}"
        );
    }

    #[test]
    fn unknown_field_within_a_real_group_is_refused() {
        let err = facets_from_dotted(&["execution.nonsuch=v".to_owned()])
            .expect_err("an unknown field must not be silently dropped");
        assert!(
            err.contains("nonsuch"),
            "the refusal must name the offending field: {err}"
        );
    }

    #[test]
    fn a_pair_without_an_equals_is_refused() {
        let err = facets_from_dotted(&["execution.harness".to_owned()])
            .expect_err("a pair with no value is not `group.field=value`");
        assert!(err.contains(FACET_SHAPE), "{err}");
    }

    #[test]
    fn a_key_without_a_dot_is_refused() {
        let err = facets_from_dotted(&["harness=claude".to_owned()])
            .expect_err("an ungrouped field is not addressable");
        assert!(err.contains(FACET_SHAPE), "{err}");
    }

    #[test]
    fn an_empty_group_or_field_is_refused() {
        for pair in [".harness=claude", "execution.=claude", ".=v"] {
            let err = facets_from_dotted(&[pair.to_owned()])
                .expect_err("an empty group or field addresses nothing");
            assert!(err.contains(FACET_SHAPE), "{pair}: {err}");
        }
    }

    #[test]
    fn a_typed_field_earns_a_type_error_naming_it() {
        let err = facets_from_dotted(&["usage.total_tokens=500".to_owned()])
            .expect_err("a u64 field is not reachable from the string-valued grammar");
        assert!(
            err.contains("usage.total_tokens=500"),
            "serde reports a type mismatch with NO field path, so the refusal must name the \
             offending argument itself: {err}"
        );
        assert!(
            err.contains("--input"),
            "and must name the way through, not leave it to be inferred: {err}"
        );
    }

    #[test]
    fn the_offending_argument_is_named_even_among_valid_ones() {
        let err = facets_from_dotted(&[
            "execution.harness=claude".to_owned(),
            "usage.total_tokens=500".to_owned(),
            "work_context.slice=SL-231".to_owned(),
        ])
        .expect_err("one bad argument fails the set");
        assert!(
            err.contains("usage.total_tokens=500"),
            "the refusal must blame the offender, not the first or last argument: {err}"
        );
        assert!(
            !err.contains("execution.harness"),
            "and must not implicate a valid one: {err}"
        );
    }

    // ── Origin is derived, never caller-supplied (RV-318 F-2) ─────────────

    #[test]
    fn caller_supplied_origin_cannot_ride_the_dotted_grammar() {
        let facets = facets_from_dotted(&[
            "execution.harness=claude".to_owned(),
            "execution.harness_origin=automatic".to_owned(),
        ])
        .expect("the origin field deserializes")
        .expect("pairs yield facets");

        let merged = wire::merge_explicit_facets(Facets::default(), Some(facets));
        assert_eq!(
            merged
                .execution
                .expect("execution group present")
                .harness_origin,
            Some(Origin::Explicit),
            "origin is derived from reaching the merge, never read from the caller (RV-318 F-2)"
        );
    }

    // ── Complete request (`--input`) ──────────────────────────────────────

    #[test]
    fn a_minimal_request_needs_only_a_summary_and_enriches_by_default() {
        let req = FrictionRequest::from_json(r#"{"summary":"it hurt"}"#).expect("minimal request");
        assert_eq!(req.summary, "it hurt");
        assert!(req.uid.is_none());
        assert!(req.detail.is_none());
        assert!(req.facets.is_none());
        assert!(req.enrich, "absent `enrich` must mean enrich");
    }

    #[test]
    fn a_complete_request_carries_every_section_three_three_field() {
        let req = FrictionRequest::from_json(
            r#"{"uid":"u","summary":"s","detail":"d",
                "facets":{"execution":{"harness":"claude"}},"enrich":false}"#,
        )
        .expect("complete request");
        assert_eq!(req.uid.as_deref(), Some("u"));
        assert_eq!(req.detail.as_deref(), Some("d"));
        assert!(!req.enrich);
        assert_eq!(
            req.facets
                .expect("facets present")
                .execution
                .expect("execution group present")
                .harness
                .as_deref(),
            Some("claude")
        );
    }

    #[test]
    fn a_request_omitting_summary_is_refused() {
        let err = FrictionRequest::from_json(r#"{"detail":"d"}"#)
            .expect_err("summary is the one required field");
        assert!(err.contains("summary"), "{err}");
    }

    #[test]
    fn an_unknown_request_key_is_refused_rather_than_ignored() {
        let err = FrictionRequest::from_json(r#"{"summary":"s","path":"/etc"}"#)
            .expect_err("a key past the request contract must not be silently dropped");
        assert!(err.contains("path"), "{err}");
    }

    #[test]
    fn malformed_json_is_refused_with_an_escaped_message() {
        let err = FrictionRequest::from_json("{not json").expect_err("malformed input is refused");
        assert!(!err.contains('\n'), "a refusal is one line: {err:?}");
    }

    #[test]
    fn a_request_need_not_repeat_the_facet_schema_version() {
        let req = FrictionRequest::from_json(
            r#"{"summary":"s","facets":{"execution":{"harness":"claude"}}}"#,
        )
        .expect("schema_version is defaulted per group");
        assert_eq!(
            req.facets
                .expect("facets present")
                .execution
                .expect("execution group present")
                .schema_version,
            wire::SCHEMA_VERSION
        );
    }

    // ── parse_explicit_facets ─────────────────────────────────────────────

    #[test]
    fn absent_and_null_facets_both_mean_none() {
        assert!(
            parse_explicit_facets(None)
                .expect("absent parses")
                .is_none()
        );
        assert!(
            parse_explicit_facets(Some(&Value::Null))
                .expect("null parses")
                .is_none()
        );
    }
}
