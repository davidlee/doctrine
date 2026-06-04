//! Pure memory schema + parse core (SL-005 PHASE-02).
//!
//! Two layers, the doctrine `Meta` pattern widened (slice.rs:99):
//! `RawMemoryToml` is the tolerant read — it fills defaults for an absent nested
//! block and preserves *top-level* unknown keys in `extra` — and `Memory` is the
//! validated projection (`schema_version == 1`, closed vocab, a non-empty
//! workspace, a shape-checked uid/key). No disk, no clock: the uid and the date
//! are inputs minted in the shell (PHASE-04); this layer only validates shapes.
//!
//! PHASE-02 ships the pure core with no binary consumer yet — `record` (PHASE-04)
//! and `show`/`list` (PHASE-05) wire it in. So the non-test build sees the whole
//! module as dead; the tests exercise it. The expectation comes off as each
//! consumer lands.
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "pure memory core; consumers wired by record (PHASE-04) and show/list (PHASE-05)"
    )
)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use crate::entity::{Artifact, Fileset};

/// Workspace coordinate carried on every memory; hardcoded `"default"` in v1 (no
/// flag — design § 5.3 / interop constraint 6). Read back by `list`/`show`.
const WORKSPACE: &str = "default";

/// The only schema version v1 emits and accepts (validated `== 1` on read).
const SCHEMA_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// Closed vocabularies (memory-spec § Memory types / Lifecycle status).
// Provisional membership; the model does not depend on the exact set. Parsed in
// the validation layer (not via serde on the raw struct) so a bad member is a
// validated error with our wording, and the raw layer still parses — keeping the
// `extra` round-trip available even for an otherwise-invalid file.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MemoryType {
    Concept,
    Fact,
    Pattern,
    Signpost,
    System,
    Thread,
}

impl MemoryType {
    fn parse(s: &str) -> Result<Self> {
        Ok(match s {
            "concept" => Self::Concept,
            "fact" => Self::Fact,
            "pattern" => Self::Pattern,
            "signpost" => Self::Signpost,
            "system" => Self::System,
            "thread" => Self::Thread,
            other => bail!("unknown memory_type {other:?}"),
        })
    }

    /// The stored kebab token (inverse of `parse`) — the `{{type}}` substitution.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Concept => "concept",
            Self::Fact => "fact",
            Self::Pattern => "pattern",
            Self::Signpost => "signpost",
            Self::System => "system",
            Self::Thread => "thread",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Status {
    Active,
    Draft,
    Superseded,
    Retracted,
    Archived,
    Quarantined,
}

impl Status {
    fn parse(s: &str) -> Result<Self> {
        Ok(match s {
            "active" => Self::Active,
            "draft" => Self::Draft,
            "superseded" => Self::Superseded,
            "retracted" => Self::Retracted,
            "archived" => Self::Archived,
            "quarantined" => Self::Quarantined,
            other => bail!("unknown status {other:?}"),
        })
    }

    /// The stored kebab token (inverse of `parse`) — the `{{status}}` substitution.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Draft => "draft",
            Self::Superseded => "superseded",
            Self::Retracted => "retracted",
            Self::Archived => "archived",
            Self::Quarantined => "quarantined",
        }
    }
}

// ---------------------------------------------------------------------------
// Identity & key shape validators (memory-spec § Identity :266-272).
// Hand-rolled byte scans — no `regex`/`uuid` parse needed to check a string, and
// they keep the strict lint surface (`as_conversions`, `indexing_slicing`) clean.
// ---------------------------------------------------------------------------

/// `^mem_[0-9a-f]{32}$` — the stored uid shape (lowercase simple-form UUID).
/// Uppercase / hyphenated forms are rejected, not normalized (design § 5.6).
fn is_uid(s: &str) -> bool {
    s.strip_prefix("mem_").is_some_and(|hex| {
        hex.len() == 32 && hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
    })
}

/// One key segment: `[a-z0-9]+(-[a-z0-9]+)*` — lowercase-alnum runs joined by
/// single internal hyphens, no leading/trailing/double hyphen, non-empty.
fn valid_segment(seg: &str) -> bool {
    seg.split('-')
        .all(|run| !run.is_empty() && run.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'z')))
}

/// A full `memory_key`: `mem.` + 1–6 further segments (2–7 dot-segments total,
/// the literal `mem` counted), each a valid segment (memory-spec :271-272).
fn validate_key(key: &str) -> Result<()> {
    let segs: Vec<&str> = key.split('.').collect();
    if !(2..=7).contains(&segs.len()) {
        bail!(
            "memory_key must have 2-7 dot-segments, got {}: {key:?}",
            segs.len()
        );
    }
    if segs.first() != Some(&"mem") {
        bail!("memory_key must start with 'mem.': {key:?}");
    }
    for seg in &segs {
        if !valid_segment(seg) {
            bail!("invalid memory_key segment {seg:?} in {key:?}");
        }
    }
    Ok(())
}

/// Normalize a `--key` argument: a shorthand without the `mem.` prefix
/// (`pattern.cli.skinny`) gains it (`mem.pattern.cli.skinny`); the result is then
/// segment-validated. Distinct from `MemoryRef::parse`, which validates an
/// already-full key for `show`; both share `validate_key`.
pub(crate) fn normalize_key(input: &str) -> Result<String> {
    let key = if input.starts_with("mem.") {
        input.to_owned()
    } else {
        format!("mem.{input}")
    };
    validate_key(&key)?;
    Ok(key)
}

/// Free-form tags (memory-spec § Schema): trimmed, lowercased, non-empty,
/// deduplicated (first-seen order). No `mem.*` segment grammar (review #9).
pub(crate) fn validate_tags(tags: &[String]) -> Result<Vec<String>> {
    let mut out: Vec<String> = Vec::new();
    for raw in tags {
        let tag = raw.trim().to_lowercase();
        if tag.is_empty() {
            bail!("tag must not be empty/blank");
        }
        if !out.iter().any(|seen| seen == &tag) {
            out.push(tag);
        }
    }
    Ok(out)
}

/// A validated `show <uid|key>` argument. Parsing rejects any path-traversal
/// shape *before* a future `safe_join` (codex-MAJOR-3), then classifies as a uid
/// or a full key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MemoryRef {
    Uid(String),
    Key(String),
}

impl MemoryRef {
    pub(crate) fn parse(arg: &str) -> Result<MemoryRef> {
        if arg.is_empty() {
            bail!("empty memory reference");
        }
        // Reject traversal / separators at the boundary — hostile input never
        // reaches the read-path join unvalidated.
        if arg.contains('/') || arg.contains('\\') || arg.contains('\0') {
            bail!("memory reference must not contain a path separator: {arg:?}");
        }
        if arg.contains("..") {
            bail!("memory reference must not contain '..': {arg:?}");
        }
        if is_uid(arg) {
            return Ok(MemoryRef::Uid(arg.to_owned()));
        }
        if validate_key(arg).is_ok() {
            return Ok(MemoryRef::Key(arg.to_owned()));
        }
        bail!("not a valid memory uid or key: {arg:?}");
    }
}

// ---------------------------------------------------------------------------
// Raw layer — tolerant parse. Every nested block is `#[serde(default)]` so a
// deleted `[block]` fills defaults; a single top-level `#[serde(flatten)] extra`
// preserves unknown *top-level* keys (the round-trip seam — unknown keys inside a
// nested block are dropped, no nested `extra`, design § 5.3). All raw structs
// derive `Serialize` for that round-trip.
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct RawMemoryToml {
    #[serde(default)]
    memory_uid: String,
    #[serde(default)]
    memory_key: Option<String>,
    #[serde(default)]
    schema_version: u32,
    #[serde(default)]
    memory_type: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    created: String,
    #[serde(default)]
    updated: String,
    #[serde(default)]
    scope: RawScope,
    #[serde(default)]
    git: RawGit,
    #[serde(default)]
    review: RawReview,
    #[serde(default)]
    trust: RawTrust,
    #[serde(default)]
    ranking: RawRanking,
    #[serde(default, rename = "relation")]
    relations: Vec<RawRelation>,
    #[serde(default, rename = "source")]
    sources: Vec<RawSource>,
    #[serde(flatten)]
    extra: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct RawScope {
    #[serde(default)]
    paths: Vec<String>,
    #[serde(default)]
    globs: Vec<String>,
    #[serde(default)]
    commands: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    workspace: String,
    #[serde(default)]
    repo: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct RawReview {
    #[serde(default)]
    verification_state: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct RawTrust {
    #[serde(default)]
    trust_level: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct RawRanking {
    #[serde(default)]
    severity: String,
    #[serde(default)]
    weight: i64,
}

// Blocks carried for shape-faithful parse (so they are consumed, not leaked into
// `extra`) but not read by any v1 verb: git anchoring is SL-007, relation/source
// resolution is the SL-008 registry. Modelled fieldless — serde ignores their
// keys, v1 stores nothing from them.
#[derive(Debug, Default, Deserialize, Serialize)]
struct RawGit {}

#[derive(Debug, Default, Deserialize, Serialize)]
struct RawRelation {}

#[derive(Debug, Default, Deserialize, Serialize)]
struct RawSource {}

// ---------------------------------------------------------------------------
// Validated layer.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Scope {
    pub(crate) paths: Vec<String>,
    pub(crate) globs: Vec<String>,
    pub(crate) commands: Vec<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) workspace: String,
    pub(crate) repo: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Memory {
    pub(crate) uid: String,
    pub(crate) key: Option<String>,
    pub(crate) kind: MemoryType,
    pub(crate) status: Status,
    pub(crate) title: String,
    pub(crate) summary: String,
    pub(crate) created: String,
    pub(crate) updated: String,
    pub(crate) scope: Scope,
    pub(crate) verification_state: String,
    pub(crate) trust_level: String,
    pub(crate) severity: String,
    pub(crate) weight: i64,
}

impl TryFrom<RawMemoryToml> for Memory {
    type Error = anyhow::Error;

    fn try_from(raw: RawMemoryToml) -> Result<Self> {
        let RawMemoryToml {
            memory_uid,
            memory_key,
            schema_version,
            memory_type: type_raw,
            status: status_raw,
            title,
            summary,
            created,
            updated,
            scope,
            review,
            trust,
            ranking,
            ..
        } = raw;

        if schema_version != 1 {
            bail!("unsupported schema_version {schema_version} (v1 accepts only 1)");
        }
        if !is_uid(&memory_uid) {
            bail!("memory_uid {memory_uid:?} is not a valid mem_<32 hex> uid");
        }
        let memory_type = MemoryType::parse(&type_raw)?;
        let status = Status::parse(&status_raw)?;
        let key = match memory_key {
            Some(k) => {
                validate_key(&k)?;
                Some(k)
            }
            None => None,
        };

        let workspace = scope.workspace.trim().to_owned();
        if workspace.is_empty() {
            bail!("scope.workspace must be non-empty (interop constraint 6)");
        }
        let tags = validate_tags(&scope.tags)?;

        Ok(Memory {
            uid: memory_uid,
            key,
            kind: memory_type,
            status,
            title,
            summary,
            created,
            updated,
            scope: Scope {
                paths: scope.paths,
                globs: scope.globs,
                commands: scope.commands,
                tags,
                workspace,
                repo: scope.repo,
            },
            verification_state: review.verification_state,
            trust_level: trust.trust_level,
            severity: ranking.severity,
            weight: ranking.weight,
        })
    }
}

impl Memory {
    /// Parse a `memory.toml` body into a validated `Memory` (the two layers
    /// composed). The read path for `show`/`list` (PHASE-05).
    pub(crate) fn parse(text: &str) -> Result<Memory> {
        let raw: RawMemoryToml = toml::from_str(text)
            .map_err(|e| anyhow::anyhow!("failed to parse memory.toml: {e}"))?;
        Memory::try_from(raw)
    }
}

// ---------------------------------------------------------------------------
// Pure render + the memory fileset (PHASE-03).
//
// Memory's render needs more than the engine's uniform `ScaffoldCtx`
// (eid/slug/title/date) carries — `type`/`status`/`summary`/`key`/`tags` — so it
// does not ride `Kind.scaffold: fn(&ScaffoldCtx)`. Instead the shell mints the uid
// (PHASE-04), the record fields are known up front, and `memory_scaffold` renders
// the whole `Fileset` eagerly; PHASE-04's `materialise_named` claims `items/<uid>/`
// and hands this fileset to the existing transactional `write_fileset` (seam A).
// No clock, no disk here — values in, fileset out.
// ---------------------------------------------------------------------------

/// Render `memory.toml` from the embedded template. The `memory_key` line is
/// present iff `key` is `Some` (an empty `memory_key = ""` would fail
/// `validate_key` on read); `tags` becomes a TOML array literal; `workspace` and
/// `schema_version` are the hardcoded v1 constants.
#[expect(
    clippy::too_many_arguments,
    reason = "flat record fields; a Draft struct lands with run_record (PHASE-04)"
)]
fn render_memory_toml(
    uid: &str,
    key: Option<&str>,
    memory_type: MemoryType,
    status: Status,
    title: &str,
    summary: &str,
    date: &str,
    tags: &[String],
) -> Result<String> {
    let key_line = match key {
        Some(k) => format!("memory_key = \"{k}\"\n"),
        None => String::new(),
    };
    let tags_lit = tags
        .iter()
        .map(|t| format!("\"{t}\""))
        .collect::<Vec<_>>()
        .join(", ");
    Ok(crate::install::asset_text("templates/memory.toml")?
        .replace("{{uid}}", uid)
        .replace("{{key_line}}", &key_line)
        .replace("{{schema_version}}", &SCHEMA_VERSION.to_string())
        .replace("{{type}}", memory_type.as_str())
        .replace("{{status}}", status.as_str())
        .replace("{{title}}", title)
        .replace("{{summary}}", summary)
        .replace("{{date}}", date)
        .replace("{{tags}}", &tags_lit)
        .replace("{{workspace}}", WORKSPACE))
}

/// Render `memory.md` — the tool-authored body: title + summary only (design § 5.2).
fn render_memory_md(title: &str, summary: &str) -> Result<String> {
    Ok(crate::install::asset_text("templates/memory.md")?
        .replace("{{title}}", title)
        .replace("{{summary}}", summary))
}

/// The memory fileset, relative to the `items/` tree root: `<uid>/memory.toml`,
/// `<uid>/memory.md`, and — iff `key` is given — a `<key> -> <uid>` symlink sibling
/// to the uid dir, carried *in the fileset* so PHASE-04's `write_fileset`
/// transaction covers it (a pre-existing alias fails the whole record, design § 5.5).
#[expect(
    clippy::too_many_arguments,
    reason = "flat record fields; a Draft struct lands with run_record (PHASE-04)"
)]
pub(crate) fn memory_scaffold(
    uid: &str,
    key: Option<&str>,
    memory_type: MemoryType,
    status: Status,
    title: &str,
    summary: &str,
    date: &str,
    tags: &[String],
) -> Result<Fileset> {
    let mut fileset = vec![
        Artifact::File {
            rel_path: PathBuf::from(format!("{uid}/memory.toml")),
            body: render_memory_toml(uid, key, memory_type, status, title, summary, date, tags)?,
        },
        Artifact::File {
            rel_path: PathBuf::from(format!("{uid}/memory.md")),
            body: render_memory_md(title, summary)?,
        },
    ];
    if let Some(k) = key {
        fileset.push(Artifact::Symlink {
            rel_path: PathBuf::from(k),
            target: uid.to_string(),
        });
    }
    Ok(fileset)
}

#[cfg(test)]
mod tests {
    use super::*;

    // A valid uid for fixtures (32 lowercase hex after `mem_`).
    const UID: &str = "mem_018f3a1b2c3d4e5f60718293a4b5c6d7";

    fn full_toml() -> String {
        format!(
            r#"
memory_uid = "{UID}"
memory_key = "mem.pattern.cli.skinny"
schema_version = 1
memory_type = "pattern"
status = "active"
title = "Skinny CLI"
summary = "CLI delegates to domain logic."
created = "2026-06-04"
updated = "2026-06-04"

[scope]
paths = ["src/main.rs"]
globs = ["src/**/*.rs"]
commands = ["doctrine slice"]
tags = ["cli", "architecture"]
workspace = "default"
repo = "github.com/davidlee/doctrine"

[git]
anchor_kind = "none"

[review]
verification_state = "unverified"

[trust]
trust_level = "medium"

[ranking]
severity = "high"
weight = 8

[[relation]]
rel = "supersedes"
to = "mem_018e000000000000000000000000000b"

[[source]]
kind = "code"
ref = "src/main.rs"
"#
        )
    }

    // -- EX-2/EX-4/EX-5: the validated projection ---------------------------

    #[test]
    fn parses_a_full_memory_toml_reading_every_carried_field() {
        let m = Memory::parse(&full_toml()).unwrap();
        assert_eq!(m.uid, UID);
        assert_eq!(m.key.as_deref(), Some("mem.pattern.cli.skinny"));
        assert_eq!(m.kind, MemoryType::Pattern);
        assert_eq!(m.status, Status::Active);
        assert_eq!(m.title, "Skinny CLI");
        assert_eq!(m.summary, "CLI delegates to domain logic.");
        assert_eq!(m.created, "2026-06-04");
        assert_eq!(m.updated, "2026-06-04");
        assert_eq!(m.scope.paths, ["src/main.rs"]);
        assert_eq!(m.scope.globs, ["src/**/*.rs"]);
        assert_eq!(m.scope.commands, ["doctrine slice"]);
        assert_eq!(m.scope.tags, ["cli", "architecture"]);
        assert_eq!(m.scope.workspace, "default");
        assert_eq!(m.scope.repo, "github.com/davidlee/doctrine");
        assert_eq!(m.verification_state, "unverified");
        assert_eq!(m.trust_level, "medium");
        assert_eq!(m.severity, "high");
        assert_eq!(m.weight, 8);
    }

    #[test]
    fn key_is_optional() {
        let toml = full_toml().replace("memory_key = \"mem.pattern.cli.skinny\"\n", "");
        let m = Memory::parse(&toml).unwrap();
        assert_eq!(m.key, None);
    }

    // -- VT-1: round-trip / extra scope ------------------------------------

    #[test]
    fn top_level_unknown_key_is_preserved_in_extra() {
        // A top-level key must sit before the first [table] or TOML nests it.
        let toml = full_toml().replace(
            "updated = \"2026-06-04\"\n",
            "updated = \"2026-06-04\"\nmystery_top = \"keep me\"\n",
        );
        let raw: RawMemoryToml = toml::from_str(&toml).unwrap();
        assert!(raw.extra.contains_key("mystery_top"));
        // and it survives a serialize round-trip
        let back = toml::to_string(&raw).unwrap();
        assert!(back.contains("mystery_top"));
    }

    #[test]
    fn nested_block_unknown_key_is_not_preserved() {
        let toml = full_toml().replace("[scope]\n", "[scope]\nmystery_nested = 1\n");
        let raw: RawMemoryToml = toml::from_str(&toml).unwrap();
        assert!(!raw.extra.contains_key("mystery_nested"));
        let back = toml::to_string(&raw).unwrap();
        assert!(!back.contains("mystery_nested"));
    }

    #[test]
    fn a_deleted_nested_block_fills_defaults() {
        // Drop [ranking] entirely — it must default, not fail to parse.
        let toml = full_toml().replace("[ranking]\nseverity = \"high\"\nweight = 8\n", "");
        let m = Memory::parse(&toml).unwrap();
        assert_eq!(m.severity, "");
        assert_eq!(m.weight, 0);
    }

    // -- VT-2: closed vocab + schema_version --------------------------------

    #[test]
    fn unknown_memory_type_is_an_error() {
        let toml = full_toml().replace("memory_type = \"pattern\"", "memory_type = \"bogus\"");
        assert!(Memory::parse(&toml).is_err());
    }

    #[test]
    fn unknown_status_is_an_error() {
        let toml = full_toml().replace("status = \"active\"", "status = \"bogus\"");
        assert!(Memory::parse(&toml).is_err());
    }

    #[test]
    fn schema_version_other_than_one_is_an_error() {
        let toml = full_toml().replace("schema_version = 1", "schema_version = 2");
        assert!(Memory::parse(&toml).is_err());
    }

    #[test]
    fn missing_schema_version_is_an_error() {
        let toml = full_toml().replace("schema_version = 1\n", "");
        assert!(Memory::parse(&toml).is_err());
    }

    // -- EX-5 workspace -----------------------------------------------------

    #[test]
    fn empty_workspace_is_rejected() {
        let toml = full_toml().replace("workspace = \"default\"", "workspace = \"\"");
        assert!(Memory::parse(&toml).is_err());
    }

    #[test]
    fn missing_scope_block_rejects_for_empty_workspace() {
        let toml = full_toml().replace(
            "[scope]\npaths = [\"src/main.rs\"]\nglobs = [\"src/**/*.rs\"]\ncommands = [\"doctrine slice\"]\ntags = [\"cli\", \"architecture\"]\nworkspace = \"default\"\nrepo = \"github.com/davidlee/doctrine\"\n",
            "",
        );
        assert!(Memory::parse(&toml).is_err());
    }

    #[test]
    fn invalid_uid_is_rejected() {
        let toml = full_toml().replace(UID, "mem_NOTHEX");
        assert!(Memory::parse(&toml).is_err());
    }

    // -- VT-3: MemoryRef ----------------------------------------------------

    #[test]
    fn memory_ref_accepts_a_valid_uid() {
        assert_eq!(
            MemoryRef::parse(UID).unwrap(),
            MemoryRef::Uid(UID.to_owned())
        );
    }

    #[test]
    fn memory_ref_accepts_a_valid_key() {
        assert_eq!(
            MemoryRef::parse("mem.pattern.cli.skinny").unwrap(),
            MemoryRef::Key("mem.pattern.cli.skinny".to_owned())
        );
    }

    #[test]
    fn memory_ref_rejects_traversal_and_separators() {
        for hostile in ["../x", "a/b", "/abs", "..", "mem..x", "a\\b", "mem.\0x"] {
            assert!(
                MemoryRef::parse(hostile).is_err(),
                "should reject {hostile:?}"
            );
        }
    }

    #[test]
    fn memory_ref_rejects_malformed_uid_shapes() {
        for bad in [
            "mem_018F3A00000000000000000000000A",  // uppercase
            "mem_018f3a",                          // too short
            "mem_018f3a00000000000000000000000aa", // too long
            "018f3a00000000000000000000000000",    // no prefix
        ] {
            // not a uid; and not a valid key either -> error
            assert!(MemoryRef::parse(bad).is_err(), "should reject {bad:?}");
        }
    }

    // -- VT-4: key normalize + tags -----------------------------------------

    #[test]
    fn normalize_key_table() {
        assert_eq!(
            normalize_key("pattern.cli.skinny").unwrap(),
            "mem.pattern.cli.skinny"
        );
        assert_eq!(
            normalize_key("mem.pattern.cli.skinny").unwrap(),
            "mem.pattern.cli.skinny"
        );
        assert_eq!(normalize_key("a.b").unwrap(), "mem.a.b");
        assert_eq!(
            normalize_key("multi-word.cli").unwrap(),
            "mem.multi-word.cli"
        );
    }

    #[test]
    fn normalize_key_rejects_bad_segments() {
        for bad in [
            "Bad.Case", // uppercase
            "a..b",     // empty segment
            "-lead.x",  // leading hyphen
            "trail-.x", // trailing hyphen
            "a--b.x",   // double hyphen
        ] {
            assert!(normalize_key(bad).is_err(), "should reject {bad:?}");
        }
    }

    #[test]
    fn key_segment_count_bounds() {
        // 7 segments incl mem -> ok; 8 -> error.
        assert!(normalize_key("mem.a.b.c.d.e.f").is_ok());
        assert!(normalize_key("mem.a.b.c.d.e.f.g").is_err());
    }

    #[test]
    fn validate_tags_trims_lowercases_dedups() {
        let input = vec![
            "  CLI ".to_owned(),
            "Architecture".to_owned(),
            "cli".to_owned(),
        ];
        assert_eq!(validate_tags(&input).unwrap(), ["cli", "architecture"]);
    }

    #[test]
    fn validate_tags_rejects_blank() {
        assert!(validate_tags(&["  ".to_owned()]).is_err());
    }

    // -- PHASE-03: render + scaffold ----------------------------------------

    fn tags(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|s| (*s).to_owned()).collect()
    }

    // VT-1: token substitution — parses, carries workspace + schema_version, no
    // leftover tokens; every rendered field reads back.
    #[test]
    fn render_memory_toml_substitutes_and_parses() {
        let body = render_memory_toml(
            UID,
            Some("mem.pattern.cli.skinny"),
            MemoryType::Pattern,
            Status::Active,
            "Skinny CLI",
            "CLI delegates to domain logic.",
            "2026-06-04",
            &tags(&["cli", "architecture"]),
        )
        .unwrap();
        assert!(!body.contains("{{"), "no leftover tokens: {body}");
        assert!(body.contains("workspace = \"default\""));
        assert!(body.contains("schema_version = 1"));

        let m = Memory::parse(&body).unwrap();
        assert_eq!(m.uid, UID);
        assert_eq!(m.key.as_deref(), Some("mem.pattern.cli.skinny"));
        assert_eq!(m.kind, MemoryType::Pattern);
        assert_eq!(m.status, Status::Active);
        assert_eq!(m.title, "Skinny CLI");
        assert_eq!(m.summary, "CLI delegates to domain logic.");
        assert_eq!(m.scope.tags, ["cli", "architecture"]);
        assert_eq!(m.scope.workspace, "default");
    }

    #[test]
    fn render_memory_toml_omits_the_key_line_when_absent() {
        let body = render_memory_toml(
            UID,
            None,
            MemoryType::Fact,
            Status::Draft,
            "T",
            "",
            "2026-06-04",
            &[],
        )
        .unwrap();
        assert!(!body.contains("memory_key"), "no empty key line: {body}");
        assert_eq!(Memory::parse(&body).unwrap().key, None);
    }

    // VT-3: the rendered toml round-trips into Memory with every defaulted block
    // present.
    #[test]
    fn rendered_toml_round_trips_with_defaulted_blocks() {
        let body = render_memory_toml(
            UID,
            None,
            MemoryType::System,
            Status::Active,
            "T",
            "S",
            "2026-06-04",
            &[],
        )
        .unwrap();
        let m = Memory::parse(&body).unwrap();
        assert_eq!(m.verification_state, "unverified");
        assert_eq!(m.trust_level, "medium");
        assert_eq!(m.severity, "none");
        assert_eq!(m.weight, 0);
        assert_eq!(m.scope.workspace, "default");
        assert!(m.scope.tags.is_empty());
    }

    #[test]
    fn render_memory_md_is_title_and_summary() {
        let body = render_memory_md("My Title", "My summary.").unwrap();
        assert!(body.contains("# My Title"));
        assert!(body.contains("My summary."));
        assert!(!body.contains("{{"));
    }

    // VT-2: fileset shape — 2 artifacts without a key.
    #[test]
    fn memory_scaffold_is_two_files_without_a_key() {
        let fileset = memory_scaffold(
            UID,
            None,
            MemoryType::Pattern,
            Status::Active,
            "T",
            "S",
            "2026-06-04",
            &[],
        )
        .unwrap();
        assert_eq!(fileset.len(), 2);
        assert!(matches!(&fileset[0],
            Artifact::File { rel_path, .. }
            if rel_path == std::path::Path::new(&format!("{UID}/memory.toml"))));
        assert!(matches!(&fileset[1],
            Artifact::File { rel_path, .. }
            if rel_path == std::path::Path::new(&format!("{UID}/memory.md"))));
    }

    // VT-2: with a key — a third artifact, an Artifact::Symlink whose target is the
    // uid (the alias rides the fileset so PHASE-04's write_fileset transaction
    // covers it).
    #[test]
    fn memory_scaffold_adds_a_key_symlink_targeting_the_uid() {
        let fileset = memory_scaffold(
            UID,
            Some("mem.pattern.cli.skinny"),
            MemoryType::Pattern,
            Status::Active,
            "T",
            "S",
            "2026-06-04",
            &[],
        )
        .unwrap();
        assert_eq!(fileset.len(), 3);
        assert!(matches!(&fileset[2],
            Artifact::Symlink { rel_path, target }
            if rel_path == std::path::Path::new("mem.pattern.cli.skinny") && target == UID));
    }
}
