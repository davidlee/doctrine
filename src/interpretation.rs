// SPDX-License-Identifier: GPL-3.0-only
//! The `[interpretation]` policy — a **strict** typed projection of one table
//! out of a deliberately tolerant `doctrine.toml` (SL-248 `sec-4`, `REQ-449`).
//!
//! `src/reserve.rs:78-97` is the shape: a tolerant outer document projecting a
//! single table, a pure `parse_*(text)`, and a thin loader elsewhere that
//! supplies the text. This module differs from that precedent in exactly one
//! respect — **the projected table is strict inside**. `[dispatch]`,
//! `[capsule]` and `[reservation]` are simply not looked at; a key inside
//! `[interpretation]` that this module does not know is a refusal.
//!
//! **Pure throughout.** Nothing here reads a file, a clock, or a repository —
//! `out=0` in `.doctrine/adr/001/layering.toml` is an assertion the layering
//! gate checks, not a comment. The text arrives from the trusted side, read as
//! a blob at the contracted base OID.
//!
//! **The table is walked, not deserialized.** `REQ-449` criterion 1 requires
//! distinguishable refusals, and a `#[derive(Deserialize)]` with
//! `deny_unknown_fields` produces a formatted *string* for the interesting
//! cases — turning those back into typed refusals means pattern-matching a
//! dependency's error message, which breaks silently when `toml` rewords
//! itself, in exactly the code whose job is to refuse precisely. A derive would
//! also collapse `SPEC-030`'s distinction between an explicitly empty list and
//! an omitted one, which `#[serde(default)]` cannot express.
//!
//! **The walk is recursive.** Each `[[interpretation.verification]]` row is
//! walked against its own known key set, for the same reason the table is
//! (`RV-346` `F-11`): strictness that stops at the first level is a strict
//! outer table wrapping a tolerant inner one.

// ---------------------------------------------------------------------------
// The schema, and the key vocabulary it fixes (STD-001 — named once, here).
// ---------------------------------------------------------------------------

/// The v1 schema version. The only accepted value.
pub const INTERPRETATION_SCHEMA: u64 = 1;

/// The single table this module projects out of a `doctrine.toml` body.
const BLOCK: &str = "interpretation";

const KEY_SCHEMA: &str = "schema";
const KEY_FORBIDDEN_EXECUTABLES: &str = "trusted_side_forbidden_executables";
const KEY_INTERPRETED_PATHS: &str = "interpreted_paths";
const KEY_VERIFICATION: &str = "verification";
const KEY_ARGV: &str = "argv";

/// Every key `[interpretation]` may carry. A key outside this set is
/// [`PolicyRefusal::UnknownKey`]; a key in it and absent from the document is
/// [`PolicyRefusal::KeyMissing`]. All four are required — `SPEC-030` says the
/// two lists may be *explicitly* empty and that omission is not equivalent to
/// emptiness, so neither defaults.
const KNOWN_KEYS: &[&str] = &[
    KEY_SCHEMA,
    KEY_FORBIDDEN_EXECUTABLES,
    KEY_INTERPRETED_PATHS,
    KEY_VERIFICATION,
];

/// Every key a `[[interpretation.verification]]` row may carry — `argv`, and
/// nothing else.
const KNOWN_ROW_KEYS: &[&str] = &[KEY_ARGV];

// ---------------------------------------------------------------------------
// The typed value.
// ---------------------------------------------------------------------------

/// A normalized, validated interpretation policy.
///
/// Construction is through [`parse`] alone — there is no public constructor and
/// no public field mutation, so an unnormalized value of this type cannot
/// exist. That is what lets `canonical_hash` and `restrict` assume
/// normalization rather than re-checking it.
// No `dead_code` staging attribute, and that is measured rather than assumed:
// the derived `Clone`/`PartialEq` impls read every field, so the fields are live
// from the moment the type exists — `canonical_hash` and `restrict` are not the
// first readers. (`mem.pattern.lint.dead-code-staged-ahead-cfg-test`'s
// `cfg_attr(not(test), expect(dead_code))` was tried here first and reported
// `unfulfilled_lint_expectations`.)
#[derive(Debug, Clone, PartialEq, Eq)]
#[expect(
    clippy::module_name_repetitions,
    reason = "`sec-4` and `EX-1` name this type, and it is read as \
              `interpretation::InterpretationPolicy` only inside this crate — \
              `doctrine-control` reaches it through the lib target, where the \
              module qualifier is what disambiguates it from the several other \
              `Policy` types in the workspace. Item-level, never module-blanket"
)]
pub struct InterpretationPolicy {
    schema: u64,
    /// Byte-sorted, after duplicate rejection.
    forbidden_executables: Vec<ExecutableName>,
    /// Byte-sorted, after duplicate rejection.
    interpreted_paths: Vec<PathPattern>,
    /// Order preserved. Non-empty.
    verification: Vec<VerificationRow>,
}

/// A normalized executable basename: non-empty, no slash, no whitespace, and
/// neither `.` nor `..`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExecutableName(String);

/// A normalized repository-relative gitignore-style pattern: non-empty, not
/// absolute, no backslash, no NUL, and no lexical `..` component.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PathPattern(String);

/// One verification row. Argument order preserved; every argument non-empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationRow {
    argv: Vec<String>,
}

// ---------------------------------------------------------------------------
// The refusals.
// ---------------------------------------------------------------------------

/// Why a document is not a valid interpretation policy.
///
/// Every variant is a *typed* classification reached by walking the document.
/// The single exception is [`PolicyRefusal::NotToml`], which carries `toml`'s
/// own message because a document that is not TOML never reaches the block —
/// it is the one case this module does not classify, rather than one it
/// classifies by matching a message.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PolicyRefusal {
    /// The document is not valid TOML.
    #[error("not valid TOML: {0}")]
    NotToml(String),
    /// There is no `[interpretation]` table.
    #[error("the [{BLOCK}] block is missing")]
    BlockMissing,
    /// `interpretation` is present but is not a table.
    #[error("[{BLOCK}] must be a table, found {found}")]
    BlockMalformed { found: &'static str },
    /// A required key is absent. Omission is never emptiness.
    #[error("[{BLOCK}] is missing the required key `{key}`")]
    KeyMissing { key: String },
    /// A key outside [`KNOWN_KEYS`] is present.
    #[error("[{BLOCK}] carries the unknown key `{key}`")]
    UnknownKey { key: String },
    /// `schema` is absent from the accepted set (currently the single value
    /// [`INTERPRETATION_SCHEMA`]).
    #[error("[{BLOCK}] schema version `{found}` is not supported")]
    UnsupportedSchema { found: String },
    /// A set-valued key is not an array.
    #[error("[{BLOCK}] `{key}` must be an array, found {found}")]
    ListMalformed { key: String, found: &'static str },
    /// An entry of a set-valued key is not a string.
    #[error("[{BLOCK}] `{key}` entry {index} must be a string, found {found}")]
    EntryMalformed {
        key: String,
        index: usize,
        found: &'static str,
    },
    /// An executable name violates its normalization rule.
    #[error("[{BLOCK}] forbidden executable `{entry}` is invalid: {reason}")]
    InvalidExecutable {
        entry: String,
        reason: ExecutableFault,
    },
    /// A path pattern violates its normalization rule.
    #[error("[{BLOCK}] interpreted path `{entry}` is invalid: {reason}")]
    InvalidPathPattern { entry: String, reason: PathFault },
    /// Two entries of a set-valued key are equal. Rejected explicitly, before
    /// sorting — a `BTreeSet` would make the sort free and the duplicate
    /// invisible.
    #[error("[{BLOCK}] `{field}` carries `{entry}` twice")]
    DuplicateEntry { field: String, entry: String },
    /// The verification sequence is empty.
    #[error("[{BLOCK}] the verification sequence is empty")]
    EmptyVerificationSequence,
    /// A verification row is not a table, or its `argv` is not an array of
    /// strings.
    #[error("[{BLOCK}] verification row {row} is malformed")]
    MalformedVerificationRow { row: usize },
    /// A verification row is missing a required key.
    #[error("[{BLOCK}] verification row {row} is missing the required key `{key}`")]
    RowKeyMissing { row: usize, key: String },
    /// A verification row carries a key outside [`KNOWN_ROW_KEYS`].
    #[error("[{BLOCK}] verification row {row} carries the unknown key `{key}`")]
    UnknownRowKey { row: usize, key: String },
    /// A verification row's `argv` is empty.
    #[error("[{BLOCK}] verification row {row} has an empty argv")]
    EmptyArgv { row: usize },
    /// A verification row carries an empty argument.
    #[error("[{BLOCK}] verification row {row} argument {index} is empty")]
    EmptyArgument { row: usize, index: usize },
}

/// How an executable basename fails its normalization rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ExecutableFault {
    /// The empty string.
    #[error("it is empty")]
    Empty,
    /// Contains `/` — a basename, not a path.
    #[error("it contains a slash")]
    Slash,
    /// Contains whitespace.
    #[error("it contains whitespace")]
    Whitespace,
    /// Is `.` or `..`.
    #[error("it is a directory reference")]
    DotOrDotDot,
}

/// How a path pattern fails its normalization rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PathFault {
    /// The empty string.
    #[error("it is empty")]
    Empty,
    /// Starts with `/` — patterns are repository-relative.
    #[error("it is absolute")]
    Absolute,
    /// Contains `\`.
    #[error("it contains a backslash")]
    Backslash,
    /// Contains a NUL byte.
    #[error("it contains a NUL")]
    Nul,
    /// Carries a lexical `..` component.
    #[error("it carries a `..` component")]
    DotDotComponent,
}

// ---------------------------------------------------------------------------
// The walk.
// ---------------------------------------------------------------------------

/// Project, walk, validate and normalize the `[interpretation]` block of a
/// `doctrine.toml` body (PURE).
///
/// # Errors
///
/// Returns the [`PolicyRefusal`] classifying the first violation found. The
/// walk is ordered — document shape, then the key set, then per-key values —
/// so the refusal is deterministic for a given document.
pub fn parse(text: &str) -> Result<InterpretationPolicy, PolicyRefusal> {
    let doc = text
        .parse::<toml::Table>()
        .map_err(|e| PolicyRefusal::NotToml(e.to_string()))?;

    let Some(value) = doc.get(BLOCK) else {
        return Err(PolicyRefusal::BlockMissing);
    };
    let block = value.as_table().ok_or(PolicyRefusal::BlockMalformed {
        found: value.type_str(),
    })?;

    // Unknown before missing: a typo'd key would otherwise be reported as the
    // absence of the key it was trying to be, which names the wrong edit.
    for key in block.keys() {
        if !KNOWN_KEYS.contains(&key.as_str()) {
            return Err(PolicyRefusal::UnknownKey { key: key.clone() });
        }
    }

    let schema = parse_schema(require(block, KEY_SCHEMA)?)?;
    let forbidden_executables = parse_set(
        require(block, KEY_FORBIDDEN_EXECUTABLES)?,
        KEY_FORBIDDEN_EXECUTABLES,
    )
    .and_then(|raw| normalize_set(raw, KEY_FORBIDDEN_EXECUTABLES, executable_name))?;
    let interpreted_paths = parse_set(
        require(block, KEY_INTERPRETED_PATHS)?,
        KEY_INTERPRETED_PATHS,
    )
    .and_then(|raw| normalize_set(raw, KEY_INTERPRETED_PATHS, path_pattern))?;
    let verification = parse_verification(require(block, KEY_VERIFICATION)?)?;

    Ok(InterpretationPolicy {
        schema,
        forbidden_executables,
        interpreted_paths,
        verification,
    })
}

/// A required key's value, or [`PolicyRefusal::KeyMissing`] naming it.
fn require<'a>(block: &'a toml::Table, key: &str) -> Result<&'a toml::Value, PolicyRefusal> {
    block.get(key).ok_or_else(|| PolicyRefusal::KeyMissing {
        key: key.to_owned(),
    })
}

/// `schema` must be an integer equal to [`INTERPRETATION_SCHEMA`]. Every other
/// spelling — a string, a float, a different version — is the same refusal,
/// naming what was found.
fn parse_schema(value: &toml::Value) -> Result<u64, PolicyRefusal> {
    let found = value
        .as_integer()
        .and_then(|i| u64::try_from(i).ok())
        .filter(|v| *v == INTERPRETATION_SCHEMA);
    found.ok_or_else(|| PolicyRefusal::UnsupportedSchema {
        found: render(value),
    })
}

/// The raw strings of a set-valued key, in document order.
fn parse_set(value: &toml::Value, key: &str) -> Result<Vec<String>, PolicyRefusal> {
    let array = value.as_array().ok_or(PolicyRefusal::ListMalformed {
        key: key.to_owned(),
        found: value.type_str(),
    })?;
    array
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            entry
                .as_str()
                .map(str::to_owned)
                .ok_or(PolicyRefusal::EntryMalformed {
                    key: key.to_owned(),
                    index,
                    found: entry.type_str(),
                })
        })
        .collect()
}

/// Validate each entry, reject duplicates, then sort by raw UTF-8 bytes.
///
/// The three steps are in that order and the order is the requirement:
/// `REQ-449` criterion 2 sorts *after* duplicate detection, and a set type
/// would have silently absorbed the duplicate instead of refusing it.
fn normalize_set<T, F>(raw: Vec<String>, field: &str, validate: F) -> Result<Vec<T>, PolicyRefusal>
where
    T: Ord,
    F: Fn(String) -> Result<T, PolicyRefusal>,
{
    let mut seen: Vec<&String> = Vec::with_capacity(raw.len());
    for entry in &raw {
        if seen.contains(&entry) {
            return Err(PolicyRefusal::DuplicateEntry {
                field: field.to_owned(),
                entry: entry.clone(),
            });
        }
        seen.push(entry);
    }

    let mut out = raw
        .into_iter()
        .map(validate)
        .collect::<Result<Vec<T>, PolicyRefusal>>()?;
    out.sort();
    Ok(out)
}

/// Non-empty, no slash, no whitespace, not `.` or `..`.
fn executable_name(entry: String) -> Result<ExecutableName, PolicyRefusal> {
    let fault = if entry.is_empty() {
        Some(ExecutableFault::Empty)
    } else if entry.contains('/') {
        Some(ExecutableFault::Slash)
    } else if entry.chars().any(char::is_whitespace) {
        Some(ExecutableFault::Whitespace)
    } else if entry == "." || entry == ".." {
        Some(ExecutableFault::DotOrDotDot)
    } else {
        None
    };
    match fault {
        Some(reason) => Err(PolicyRefusal::InvalidExecutable { entry, reason }),
        None => Ok(ExecutableName(entry)),
    }
}

/// Non-empty, not absolute, no backslash, no NUL, no lexical `..` component.
fn path_pattern(entry: String) -> Result<PathPattern, PolicyRefusal> {
    let fault = if entry.is_empty() {
        Some(PathFault::Empty)
    } else if entry.starts_with('/') {
        Some(PathFault::Absolute)
    } else if entry.contains('\\') {
        Some(PathFault::Backslash)
    } else if entry.contains('\0') {
        Some(PathFault::Nul)
    } else if entry.split('/').any(|c| c == "..") {
        Some(PathFault::DotDotComponent)
    } else {
        None
    };
    match fault {
        Some(reason) => Err(PolicyRefusal::InvalidPathPattern { entry, reason }),
        None => Ok(PathPattern(entry)),
    }
}

/// Walk the verification sequence — a non-empty array of tables, each walked
/// against [`KNOWN_ROW_KEYS`] in its own right. Row order is preserved.
fn parse_verification(value: &toml::Value) -> Result<Vec<VerificationRow>, PolicyRefusal> {
    let rows = value.as_array().ok_or(PolicyRefusal::ListMalformed {
        key: KEY_VERIFICATION.to_owned(),
        found: value.type_str(),
    })?;
    if rows.is_empty() {
        return Err(PolicyRefusal::EmptyVerificationSequence);
    }
    rows.iter()
        .enumerate()
        .map(|(row, entry)| parse_verification_row(row, entry))
        .collect()
}

/// One row: a table carrying `argv` and nothing else, whose `argv` is a
/// non-empty array of non-empty strings.
fn parse_verification_row(
    row: usize,
    entry: &toml::Value,
) -> Result<VerificationRow, PolicyRefusal> {
    let table = entry
        .as_table()
        .ok_or(PolicyRefusal::MalformedVerificationRow { row })?;

    for key in table.keys() {
        if !KNOWN_ROW_KEYS.contains(&key.as_str()) {
            return Err(PolicyRefusal::UnknownRowKey {
                row,
                key: key.clone(),
            });
        }
    }

    let argv = table
        .get(KEY_ARGV)
        .ok_or(PolicyRefusal::RowKeyMissing {
            row,
            key: KEY_ARGV.to_owned(),
        })?
        .as_array()
        .ok_or(PolicyRefusal::MalformedVerificationRow { row })?;

    if argv.is_empty() {
        return Err(PolicyRefusal::EmptyArgv { row });
    }

    let argv = argv
        .iter()
        .enumerate()
        .map(|(index, arg)| {
            let arg = arg
                .as_str()
                .ok_or(PolicyRefusal::MalformedVerificationRow { row })?;
            if arg.is_empty() {
                return Err(PolicyRefusal::EmptyArgument { row, index });
            }
            Ok(arg.to_owned())
        })
        .collect::<Result<Vec<String>, PolicyRefusal>>()?;

    Ok(VerificationRow { argv })
}

/// A document value rendered for a refusal message — its own TOML spelling for
/// scalars, its type otherwise. Never parsed back; this is diagnosis only.
fn render(value: &toml::Value) -> String {
    value
        .as_str()
        .map(str::to_owned)
        .or_else(|| value.as_integer().map(|i| i.to_string()))
        .or_else(|| value.as_float().map(|f| f.to_string()))
        .or_else(|| value.as_bool().map(|b| b.to_string()))
        .unwrap_or_else(|| value.type_str().to_owned())
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// A document whose `[interpretation]` block is `body`, wrapped in the
    /// unrelated tables a real `doctrine.toml` carries — the tolerant half of
    /// the contract, asserted by every test that uses this helper.
    fn document(body: &str) -> String {
        format!(
            "[dispatch]\narm = \"claude\"\n\n\
             [capsule]\nroot = \"/var/tmp\"\n\n\
             {body}\n\
             [reservation]\nreach = \"local\"\n"
        )
    }

    /// The smallest well-formed block.
    fn minimal() -> String {
        document(
            "[interpretation]\n\
             schema = 1\n\
             trusted_side_forbidden_executables = []\n\
             interpreted_paths = []\n\n\
             [[interpretation.verification]]\n\
             argv = [\"just\", \"validate\"]\n\n",
        )
    }

    // ── the table walk ──────────────────────────────────────────────────

    #[test]
    fn a_well_formed_block_parses_out_of_a_document_full_of_other_tables() {
        let policy = parse(&minimal()).expect("the minimal block is valid");
        assert_eq!(policy.schema, INTERPRETATION_SCHEMA);
        assert_eq!(policy.verification.len(), 1);
    }

    #[test]
    fn missing_interpretation_block_refuses() {
        let text = document("");
        assert_eq!(parse(&text), Err(PolicyRefusal::BlockMissing));
    }

    #[test]
    fn missing_required_key_refuses_naming_the_key() {
        let text = document(
            "[interpretation]\n\
             schema = 1\n\
             interpreted_paths = []\n\n\
             [[interpretation.verification]]\n\
             argv = [\"just\"]\n\n",
        );
        assert_eq!(
            parse(&text),
            Err(PolicyRefusal::KeyMissing {
                key: KEY_FORBIDDEN_EXECUTABLES.to_owned()
            })
        );
    }

    #[test]
    fn unknown_key_refuses_naming_the_key() {
        let text = document(
            "[interpretation]\n\
             schema = 1\n\
             trusted_side_forbidden_executables = []\n\
             interpreted_paths = []\n\
             interpreted_path = []\n\n\
             [[interpretation.verification]]\n\
             argv = [\"just\"]\n\n",
        );
        assert_eq!(
            parse(&text),
            Err(PolicyRefusal::UnknownKey {
                key: "interpreted_path".to_owned()
            })
        );
    }

    #[test]
    fn unsupported_schema_version_refuses_naming_the_version() {
        let text = minimal().replace("schema = 1", "schema = 2");
        assert_eq!(
            parse(&text),
            Err(PolicyRefusal::UnsupportedSchema {
                found: "2".to_owned()
            })
        );
    }

    #[test]
    fn a_schema_that_is_not_an_integer_refuses_as_unsupported() {
        let text = minimal().replace("schema = 1", "schema = \"1\"");
        assert_eq!(
            parse(&text),
            Err(PolicyRefusal::UnsupportedSchema {
                found: "1".to_owned()
            })
        );
    }

    #[test]
    fn empty_verification_sequence_refuses() {
        let text = document(
            "[interpretation]\n\
             schema = 1\n\
             trusted_side_forbidden_executables = []\n\
             interpreted_paths = []\n\
             verification = []\n\n",
        );
        assert_eq!(parse(&text), Err(PolicyRefusal::EmptyVerificationSequence));
    }

    #[test]
    fn explicitly_empty_list_is_accepted_and_is_not_the_same_as_omission() {
        let explicit = parse(&minimal()).expect("an explicitly empty list is valid");
        assert!(explicit.forbidden_executables.is_empty());

        let omitted = minimal().replace("trusted_side_forbidden_executables = []\n", "");
        assert_eq!(
            parse(&omitted),
            Err(PolicyRefusal::KeyMissing {
                key: KEY_FORBIDDEN_EXECUTABLES.to_owned()
            }),
            "omission must refuse where an explicit [] is accepted — the \
             distinction a #[serde(default)] derive would collapse"
        );
    }

    #[test]
    fn a_block_that_is_not_a_table_refuses() {
        // Top-level, not via `document` — a bare key after a table header would
        // belong to that table, and the fixture would prove nothing.
        let text = "interpretation = 1\n\n[dispatch]\narm = \"claude\"\n";
        assert_eq!(
            parse(&text),
            Err(PolicyRefusal::BlockMalformed { found: "integer" })
        );
    }

    #[test]
    fn a_document_that_is_not_toml_refuses_without_classifying_it() {
        let refusal = parse("this is not toml{").expect_err("not a document");
        assert!(matches!(refusal, PolicyRefusal::NotToml(_)));
    }

    // ── the recursive row walk (RV-346 F-11) ────────────────────────────

    #[test]
    fn unknown_key_inside_a_verification_row_refuses_naming_the_row_and_the_key() {
        let text = minimal().replace(
            "argv = [\"just\", \"validate\"]",
            "argv = [\"just\", \"validate\"]\nextra = true",
        );
        assert_eq!(
            parse(&text),
            Err(PolicyRefusal::UnknownRowKey {
                row: 0,
                key: "extra".to_owned()
            })
        );
    }

    #[test]
    fn verification_row_missing_argv_refuses_naming_the_row() {
        let text = minimal().replace("argv = [\"just\", \"validate\"]", "");
        assert_eq!(
            parse(&text),
            Err(PolicyRefusal::RowKeyMissing {
                row: 0,
                key: KEY_ARGV.to_owned()
            })
        );
    }

    #[test]
    fn verification_row_that_is_not_a_table_refuses() {
        let text = document(
            "[interpretation]\n\
             schema = 1\n\
             trusted_side_forbidden_executables = []\n\
             interpreted_paths = []\n\
             verification = [\"just validate\"]\n\n",
        );
        assert_eq!(
            parse(&text),
            Err(PolicyRefusal::MalformedVerificationRow { row: 0 })
        );
    }

    #[test]
    fn argv_that_is_not_an_array_of_strings_refuses() {
        let not_an_array = minimal().replace("argv = [\"just\", \"validate\"]", "argv = \"just\"");
        assert_eq!(
            parse(&not_an_array),
            Err(PolicyRefusal::MalformedVerificationRow { row: 0 })
        );

        let not_strings = minimal().replace("argv = [\"just\", \"validate\"]", "argv = [1, 2]");
        assert_eq!(
            parse(&not_strings),
            Err(PolicyRefusal::MalformedVerificationRow { row: 0 })
        );
    }

    #[test]
    fn the_row_walk_reaches_rows_beyond_the_first() {
        let text = minimal().replace(
            "argv = [\"just\", \"validate\"]",
            "argv = [\"just\", \"validate\"]\n\n\
             [[interpretation.verification]]\nargv = [\"cargo\"]\nextra = 1",
        );
        assert_eq!(
            parse(&text),
            Err(PolicyRefusal::UnknownRowKey {
                row: 1,
                key: "extra".to_owned()
            }),
            "a walk that stops at the first row is a strict outer sequence \
             wrapping a tolerant inner one"
        );
    }
}
