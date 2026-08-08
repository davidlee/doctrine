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

// ---------------------------------------------------------------------------
// The restriction algebra.
// ---------------------------------------------------------------------------

/// Why a phase contract may not refine a base policy the way it asks to.
///
/// A refinement states the refined policy **in full**, in the same schema, and
/// goes through the same [`parse`]. A delta document was rejected because
/// "remove a project verification row" and "reorder project verification" have
/// no spelling in an additions-only document — an author who dropped a check
/// would be silently granted the removal.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RestrictionRefusal {
    /// The two documents describe different schema versions.
    #[error("refinement schema {refinement} does not match base schema {base}")]
    SchemaMismatch { base: u64, refinement: u64 },
    /// The refinement drops a forbidden executable the base names.
    #[error("refinement removes the forbidden executable `{entry}`")]
    ForbiddenEntryRemoved { entry: String },
    /// The refinement drops an interpreted path the base names.
    #[error("refinement removes the interpreted path `{entry}`")]
    InterpretedPathRemoved { entry: String },
    /// A base verification row is gone.
    #[error("refinement removes base verification row {index}")]
    VerificationRowRemoved { index: usize },
    /// The base rows all survive in order, but the additions are interleaved
    /// rather than appended. `index` is the first refinement position the base
    /// subsequence does not consume.
    #[error(
        "refinement inserts a row at position {index}; additions must come after the base sequence"
    )]
    VerificationRowInserted { index: usize },
    /// The base rows all survive but have moved relative to each other.
    /// `index` is the first position at which the two sequences differ.
    #[error("refinement reorders base verification at position {index}")]
    VerificationRowReordered { index: usize },
    /// The exhaustive fallthrough of the ordered classification.
    ///
    /// **No input reaches it today**, and that is a property of the order
    /// rather than an accident: once removal is diagnosed over the *multiset*
    /// of rows, any refinement that still holds every base row either carries
    /// the base as a subsequence (→ `Inserted`) or does not (→ `Reordered`). A
    /// replacement is a removal plus an insertion, and the removal is caught
    /// first — `refinement_replacing_a_project_verification_row_refuses` is the
    /// test that pins that. The variant is kept because `sec-4`'s classification
    /// names four cases and "otherwise" must have a name.
    #[error("refinement replaces base verification row {index}")]
    VerificationRowReplaced { index: usize },
}

/// Narrow `base` by `refinement`, or refuse (PURE).
///
/// The four rules are evaluated **in order**, so an earlier axis wins when a
/// refinement violates several. Rules 2 and 3 are superset checks in the same
/// direction: both lists name things the trusted plan refuses to run or treats
/// as hostile, so a superset is strictly narrower.
///
/// # Errors
///
/// Returns the [`RestrictionRefusal`] naming the edit to undo. Rule 4's failure
/// is diagnosed rather than reported as one opaque mismatch, because the cases
/// have different fixes: **prefix decides acceptance, subsequence decides which
/// refusal** (`RV-346` `F-17`).
pub fn restrict(
    base: &InterpretationPolicy,
    refinement: &InterpretationPolicy,
) -> Result<InterpretationPolicy, RestrictionRefusal> {
    if base.schema != refinement.schema {
        return Err(RestrictionRefusal::SchemaMismatch {
            base: base.schema,
            refinement: refinement.schema,
        });
    }
    if let Some(entry) = dropped(
        &base.forbidden_executables,
        &refinement.forbidden_executables,
    ) {
        return Err(RestrictionRefusal::ForbiddenEntryRemoved {
            entry: entry.0.clone(),
        });
    }
    if let Some(entry) = dropped(&base.interpreted_paths, &refinement.interpreted_paths) {
        return Err(RestrictionRefusal::InterpretedPathRemoved {
            entry: entry.0.clone(),
        });
    }
    if !is_prefix(&base.verification, &refinement.verification) {
        return Err(diagnose(&base.verification, &refinement.verification));
    }
    Ok(refinement.clone())
}

/// The first `base` entry the `refinement` does not carry, if any.
fn dropped<'a, T: PartialEq>(base: &'a [T], refinement: &[T]) -> Option<&'a T> {
    base.iter().find(|entry| !refinement.contains(entry))
}

/// Whether `base` is a prefix of `refinement`, row by row.
fn is_prefix(base: &[VerificationRow], refinement: &[VerificationRow]) -> bool {
    base.len() <= refinement.len() && base.iter().zip(refinement).all(|(b, r)| b == r)
}

/// Classify a rule-4 failure. Ordered, and the order is what makes it
/// deterministic — a refinement that both removes a row and reorders the rest
/// reports the removal.
fn diagnose(base: &[VerificationRow], refinement: &[VerificationRow]) -> RestrictionRefusal {
    if let Some(index) = first_deficient(base, refinement) {
        return RestrictionRefusal::VerificationRowRemoved { index };
    }
    if let Some(index) = first_unconsumed(base, refinement) {
        return RestrictionRefusal::VerificationRowInserted { index };
    }
    // Nothing is missing and the base is not a subsequence, so the rows moved
    // relative to each other. `VerificationRowReplaced` is *not* constructed
    // here — see its doc comment for why no input reaches it.
    RestrictionRefusal::VerificationRowReordered {
        index: first_difference(base, refinement),
    }
}

/// The first base position whose row occurs more often up to that point in the
/// base than it does in the whole refinement — i.e. the first row the
/// refinement has genuinely dropped. Multiset-aware, so a duplicated check
/// collapsed to one is a removal rather than a reordering.
fn first_deficient(base: &[VerificationRow], refinement: &[VerificationRow]) -> Option<usize> {
    base.iter().enumerate().position(|(index, row)| {
        let needed = base.iter().take(index + 1).filter(|r| *r == row).count();
        let available = refinement.iter().filter(|r| *r == row).count();
        needed > available
    })
}

/// Greedily match `base` into `refinement` as a subsequence. `Some(index)` —
/// the first refinement position the match skipped — when every base row is
/// consumed in order; `None` when it is not a subsequence at all.
fn first_unconsumed(base: &[VerificationRow], refinement: &[VerificationRow]) -> Option<usize> {
    let mut skipped = None;
    let mut remaining = base.iter();
    let mut wanted = remaining.next();
    for (index, row) in refinement.iter().enumerate() {
        match wanted {
            Some(expected) if expected == row => wanted = remaining.next(),
            _ => skipped = skipped.or(Some(index)),
        }
    }
    if wanted.is_some() { None } else { skipped }
}

/// The first position at which the two sequences differ, or the shorter one's
/// length when one is a prefix of the other.
fn first_difference(base: &[VerificationRow], refinement: &[VerificationRow]) -> usize {
    base.iter()
        .zip(refinement)
        .position(|(b, r)| b != r)
        .unwrap_or_else(|| base.len().min(refinement.len()))
}

// ---------------------------------------------------------------------------
// The canonical hash.
// ---------------------------------------------------------------------------

/// The domain prefix. Keeps a future v2 encoding from colliding with a v1 one
/// over the same bytes.
const HASH_DOMAIN: &[u8] = b"doctrine.interpretation.v1";

/// The canonical hash of a policy. A newtype with no constructor but
/// [`canonical_hash`], so a hash cannot be fabricated from anything except a
/// validated policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PolicyHash([u8; 32]);

/// SHA-256 over a deterministic encoding of the **typed value** (PURE).
///
/// Never over source text and never over a re-serialized document: either would
/// let a formatting choice — quoting style, key order, whitespace, integer
/// spelling — re-enter a value whose entire purpose is to be stable across
/// them.
///
/// Length prefixes are what make the encoding injective. Plain concatenation
/// lets `["ab", "c"]` and `["a", "bc"]` hash identically, and those are two
/// different forbidden-executable sets — one of which forbids an executable the
/// other permits.
#[must_use]
pub fn canonical_hash(policy: &InterpretationPolicy) -> PolicyHash {
    use sha2::{Digest as _, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(HASH_DOMAIN);
    hasher.update(policy.schema.to_le_bytes());

    hasher.update(count(policy.forbidden_executables.len()));
    for entry in &policy.forbidden_executables {
        hash_str(&mut hasher, &entry.0);
    }

    hasher.update(count(policy.interpreted_paths.len()));
    for entry in &policy.interpreted_paths {
        hash_str(&mut hasher, &entry.0);
    }

    hasher.update(count(policy.verification.len()));
    for row in &policy.verification {
        hasher.update(count(row.argv.len()));
        for arg in &row.argv {
            hash_str(&mut hasher, arg);
        }
    }

    PolicyHash(hasher.finalize().into())
}

/// A length or count as little-endian `u64`.
///
/// `usize as u64` is `clippy::as_conversions`, and `unwrap`/`expect`/`panic`
/// are equally denied, so the conversion is total by saturation. On every
/// supported target `usize` is at most 64 bits and the fallback is unreachable;
/// were it ever reachable, saturating is the safe direction — it cannot make
/// two distinct policies collide without first exhausting memory.
fn count(n: usize) -> [u8; 8] {
    u64::try_from(n).unwrap_or(u64::MAX).to_le_bytes()
}

/// Length-prefixed UTF-8 bytes.
fn hash_str(hasher: &mut sha2::Sha256, s: &str) {
    use sha2::Digest as _;
    hasher.update(count(s.len()));
    hasher.update(s.as_bytes());
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

    /// A block with the given list bodies and verification sequence, each
    /// spliced verbatim so a test can state the exact TOML it means.
    fn block(forbidden: &str, paths: &str, verification: &str) -> String {
        document(&format!(
            "[interpretation]\n\
             schema = 1\n\
             trusted_side_forbidden_executables = [{forbidden}]\n\
             interpreted_paths = [{paths}]\n\n\
             {verification}\n"
        ))
    }

    /// One verification row per `argv` body.
    fn rows(argvs: &[&str]) -> String {
        argvs
            .iter()
            .map(|argv| format!("[[interpretation.verification]]\nargv = [{argv}]\n\n"))
            .collect()
    }

    /// The smallest well-formed block: both lists explicitly empty, one row.
    fn minimal() -> String {
        block("", "", &rows(&["\"just\", \"validate\""]))
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

    // ── per-field validation ────────────────────────────────────────────

    /// The refusal a single forbidden-executable entry earns.
    fn executable_refusal(entry: &str) -> PolicyRefusal {
        parse(&block(&format!("\"{entry}\""), "", &rows(&["\"just\""])))
            .expect_err("the entry is invalid")
    }

    /// The refusal a single interpreted-path entry earns.
    fn path_refusal(entry: &str) -> PolicyRefusal {
        parse(&block("", &format!("\"{entry}\""), &rows(&["\"just\""])))
            .expect_err("the entry is invalid")
    }

    #[test]
    fn executable_with_a_slash_refuses() {
        assert_eq!(
            executable_refusal("/usr/bin/node"),
            PolicyRefusal::InvalidExecutable {
                entry: "/usr/bin/node".to_owned(),
                reason: ExecutableFault::Slash,
            },
            "the list names basenames; a path would silently forbid nothing"
        );
        assert!(matches!(
            executable_refusal("bin/node"),
            PolicyRefusal::InvalidExecutable {
                reason: ExecutableFault::Slash,
                ..
            }
        ));
    }

    #[test]
    fn executable_with_whitespace_refuses() {
        for entry in ["node ", " node", "no de", "node\t"] {
            assert!(
                matches!(
                    executable_refusal(entry),
                    PolicyRefusal::InvalidExecutable {
                        reason: ExecutableFault::Whitespace,
                        ..
                    }
                ),
                "`{entry}` must refuse for whitespace"
            );
        }
    }

    #[test]
    fn executable_that_is_dot_or_dotdot_refuses() {
        for entry in [".", ".."] {
            assert!(
                matches!(
                    executable_refusal(entry),
                    PolicyRefusal::InvalidExecutable {
                        reason: ExecutableFault::DotOrDotDot,
                        ..
                    }
                ),
                "`{entry}` must refuse as a directory reference"
            );
        }
    }

    #[test]
    fn executable_that_is_empty_refuses() {
        assert!(matches!(
            executable_refusal(""),
            PolicyRefusal::InvalidExecutable {
                reason: ExecutableFault::Empty,
                ..
            }
        ));
    }

    #[test]
    fn absolute_path_pattern_refuses() {
        assert_eq!(
            path_refusal("/etc/passwd"),
            PolicyRefusal::InvalidPathPattern {
                entry: "/etc/passwd".to_owned(),
                reason: PathFault::Absolute,
            },
            "patterns are repository-relative; an absolute one names a path \
             outside the tree it is meant to classify"
        );
    }

    #[test]
    fn backslash_path_pattern_refuses() {
        assert!(matches!(
            path_refusal("scripts\\\\run.sh"),
            PolicyRefusal::InvalidPathPattern {
                reason: PathFault::Backslash,
                ..
            }
        ));
    }

    #[test]
    fn nul_path_pattern_refuses() {
        assert!(matches!(
            path_refusal("scripts\\u0000run.sh"),
            PolicyRefusal::InvalidPathPattern {
                reason: PathFault::Nul,
                ..
            }
        ));
    }

    #[test]
    fn lexical_dotdot_component_refuses() {
        for entry in ["../outside", "scripts/../../etc", "a/../b"] {
            assert!(
                matches!(
                    path_refusal(entry),
                    PolicyRefusal::InvalidPathPattern {
                        reason: PathFault::DotDotComponent,
                        ..
                    }
                ),
                "`{entry}` must refuse for its `..` component"
            );
        }
        assert!(
            parse(&block("", "\"..hidden\"", &rows(&["\"just\""]))).is_ok(),
            "the rule is a `..` *component*, not the two characters anywhere"
        );
    }

    #[test]
    fn empty_path_pattern_refuses() {
        assert!(matches!(
            path_refusal(""),
            PolicyRefusal::InvalidPathPattern {
                reason: PathFault::Empty,
                ..
            }
        ));
    }

    #[test]
    fn a_list_entry_that_is_not_a_string_refuses() {
        let text = block("1", "", &rows(&["\"just\""]));
        assert_eq!(
            parse(&text),
            Err(PolicyRefusal::EntryMalformed {
                key: KEY_FORBIDDEN_EXECUTABLES.to_owned(),
                index: 0,
                found: "integer",
            })
        );
    }

    #[test]
    fn a_set_valued_key_that_is_not_an_array_refuses() {
        let text = minimal().replace(
            "trusted_side_forbidden_executables = []",
            "trusted_side_forbidden_executables = \"node\"",
        );
        assert_eq!(
            parse(&text),
            Err(PolicyRefusal::ListMalformed {
                key: KEY_FORBIDDEN_EXECUTABLES.to_owned(),
                found: "string",
            })
        );
    }

    // ── duplicate rejection, which precedes sorting ─────────────────────

    #[test]
    fn duplicate_forbidden_executable_refuses() {
        let text = block("\"node\", \"deno\", \"node\"", "", &rows(&["\"just\""]));
        assert_eq!(
            parse(&text),
            Err(PolicyRefusal::DuplicateEntry {
                field: KEY_FORBIDDEN_EXECUTABLES.to_owned(),
                entry: "node".to_owned(),
            }),
            "a set type would have absorbed this silently — REQ-449 criterion 2 \
             sorts *after* duplicate detection, which only means anything if \
             detection can refuse"
        );
    }

    #[test]
    fn duplicate_interpreted_path_refuses() {
        let text = block("", "\"*.sh\", \"*.py\", \"*.sh\"", &rows(&["\"just\""]));
        assert_eq!(
            parse(&text),
            Err(PolicyRefusal::DuplicateEntry {
                field: KEY_INTERPRETED_PATHS.to_owned(),
                entry: "*.sh".to_owned(),
            })
        );
    }

    #[test]
    fn a_duplicate_is_refused_even_when_the_entries_are_also_invalid() {
        let text = block("\"a b\", \"a b\"", "", &rows(&["\"just\""]));
        assert_eq!(
            parse(&text),
            Err(PolicyRefusal::DuplicateEntry {
                field: KEY_FORBIDDEN_EXECUTABLES.to_owned(),
                entry: "a b".to_owned(),
            }),
            "duplicate rejection runs over the raw entries, before validation \
             and before the sort"
        );
    }

    // ── argv rules ──────────────────────────────────────────────────────

    #[test]
    fn empty_argv_row_refuses() {
        let text = block("", "", &rows(&["\"just\"", ""]));
        assert_eq!(parse(&text), Err(PolicyRefusal::EmptyArgv { row: 1 }));
    }

    #[test]
    fn empty_argument_refuses() {
        let text = block("", "", &rows(&["\"just\", \"\", \"validate\""]));
        assert_eq!(
            parse(&text),
            Err(PolicyRefusal::EmptyArgument { row: 0, index: 1 }),
            "TOML strings are UTF-8 by construction, so non-emptiness is the \
             only executable half of SPEC-030's rule"
        );
    }

    // ── normalization ───────────────────────────────────────────────────

    #[test]
    fn set_valued_lists_sort_by_raw_utf8_bytes() {
        // `Z` is 0x5A and `a` is 0x61, so byte order and any case-folding order
        // disagree — a payload both orders accept would prove nothing.
        let text = block(
            "\"a\", \"Z\", \"B\"",
            "\"zed/*\", \"Alpha/*\", \"_under\"",
            &rows(&["\"just\""]),
        );
        let policy = parse(&text).expect("valid");
        assert_eq!(
            policy.forbidden_executables,
            vec![
                ExecutableName("B".to_owned()),
                ExecutableName("Z".to_owned()),
                ExecutableName("a".to_owned()),
            ]
        );
        assert_eq!(
            policy.interpreted_paths,
            vec![
                PathPattern("Alpha/*".to_owned()),
                PathPattern("_under".to_owned()),
                PathPattern("zed/*".to_owned()),
            ]
        );
    }

    #[test]
    fn verification_row_and_argument_order_are_preserved() {
        let text = block(
            "",
            "",
            &rows(&["\"z\", \"a\", \"m\"", "\"cargo\", \"test\"", "\"a\""]),
        );
        let policy = parse(&text).expect("valid");
        assert_eq!(
            policy.verification,
            vec![
                VerificationRow {
                    argv: vec!["z".to_owned(), "a".to_owned(), "m".to_owned()]
                },
                VerificationRow {
                    argv: vec!["cargo".to_owned(), "test".to_owned()]
                },
                VerificationRow {
                    argv: vec!["a".to_owned()]
                },
            ],
            "the sequence is a sequence of checks to run, not a set — sorting \
             it would reorder the operator's verification"
        );
    }

    // ── the canonical hash ──────────────────────────────────────────────

    /// The hash of a document that must parse.
    fn hash_of(text: &str) -> PolicyHash {
        canonical_hash(&parse(text).expect("valid"))
    }

    #[test]
    fn canonical_hash_is_stable_across_key_order_and_whitespace() {
        let ordered = block("\"node\"", "\"*.sh\"", &rows(&["\"just\", \"validate\""]));
        let jumbled = document(
            "[interpretation]\n\n\
             # the operator's own note, which is not part of the value\n\
             interpreted_paths   =   [ \"*.sh\" ]\n\
             trusted_side_forbidden_executables = [\n  \"node\",\n]\n\
             schema=1\n\n\
             [[interpretation.verification]]\n\
             argv = [\n  \"just\",\n  \"validate\",\n]\n\n",
        );
        assert_eq!(
            hash_of(&ordered),
            hash_of(&jumbled),
            "quoting, key order, comments and whitespace are formatting — the \
             hash is over the typed value, whose entire purpose is to be stable \
             across them"
        );
    }

    #[test]
    fn canonical_hash_distinguishes_split_boundaries_in_adjacent_entries() {
        let ab_c = block("\"ab\", \"c\"", "", &rows(&["\"just\""]));
        let a_bc = block("\"a\", \"bc\"", "", &rows(&["\"just\""]));
        assert_ne!(
            hash_of(&ab_c),
            hash_of(&a_bc),
            "without length prefixes these two concatenate to the same bytes, \
             and they are different forbidden sets — one forbids an executable \
             the other permits"
        );

        let one_arg = block("", "", &rows(&["\"ab\", \"c\""]));
        let other = block("", "", &rows(&["\"a\", \"bc\""]));
        assert_ne!(hash_of(&one_arg), hash_of(&other));
    }

    #[test]
    fn canonical_hash_is_not_computed_over_source_text() {
        let text = block("\"node\"", "\"*.sh\"", &rows(&["\"just\""]));
        let reformatted = text.replace('\n', "\r\n").replace("= [", "=[");
        assert_eq!(
            hash_of(&text),
            hash_of(&reformatted),
            "a hash over source text, or over a re-serialized document, would \
             let a formatting choice re-enter the value"
        );
    }

    #[test]
    fn the_hash_separates_the_fields_it_covers() {
        // The same three strings, moved between the two lists. A concatenation
        // without per-field counts would hash these identically.
        let left = block("\"a\"", "\"b\"", &rows(&["\"just\""]));
        let right = block("\"b\"", "\"a\"", &rows(&["\"just\""]));
        assert_ne!(hash_of(&left), hash_of(&right));
    }

    #[test]
    fn the_hash_follows_verification_order() {
        let one = block("", "", &rows(&["\"a\"", "\"b\""]));
        let two = block("", "", &rows(&["\"b\"", "\"a\""]));
        assert_ne!(
            hash_of(&one),
            hash_of(&two),
            "row order is semantic — it is the order the checks run in"
        );
    }

    // ── the restriction algebra ─────────────────────────────────────────

    /// A policy, always through [`parse`] — invariant 3 is what lets `restrict`
    /// assume normalization, and a hand-built fixture would assert over a value
    /// that cannot exist in production.
    fn policy(forbidden: &str, paths: &str, argvs: &[&str]) -> InterpretationPolicy {
        parse(&block(forbidden, paths, &rows(argvs))).expect("valid")
    }

    /// Verification rows spelled as single-argument commands, which is all the
    /// restriction tests need to distinguish rows from one another.
    fn checks(names: &[&str]) -> Vec<String> {
        names.iter().map(|n| format!("\"{n}\"")).collect()
    }

    /// A policy whose verification sequence is `names`, one row each.
    fn with_checks(names: &[&str]) -> InterpretationPolicy {
        let owned = checks(names);
        let argvs: Vec<&str> = owned.iter().map(String::as_str).collect();
        policy("", "", &argvs)
    }

    #[test]
    fn refinement_may_add_forbidden_entries() {
        let base = policy("\"node\"", "\"*.sh\"", &["\"just\""]);
        let refinement = policy("\"deno\", \"node\"", "\"*.py\", \"*.sh\"", &["\"just\""]);
        assert_eq!(
            restrict(&base, &refinement),
            Ok(refinement.clone()),
            "both lists name things the trusted plan refuses to run or treats \
             as hostile, so a superset is strictly narrower"
        );
    }

    #[test]
    fn refinement_may_append_verification_rows() {
        let base = with_checks(&["a"]);
        let refinement = with_checks(&["a", "b"]);
        assert_eq!(restrict(&base, &refinement), Ok(refinement.clone()));
    }

    #[test]
    fn restrict_is_identity_on_its_own_base() {
        let base = policy("\"node\"", "\"*.sh\"", &["\"just\", \"validate\""]);
        assert_eq!(restrict(&base, &base), Ok(base.clone()));
    }

    #[test]
    fn refinement_removing_a_forbidden_entry_refuses() {
        let base = policy("\"deno\", \"node\"", "", &["\"just\""]);
        let refinement = policy("\"node\"", "", &["\"just\""]);
        assert_eq!(
            restrict(&base, &refinement),
            Err(RestrictionRefusal::ForbiddenEntryRemoved {
                entry: "deno".to_owned()
            })
        );
    }

    #[test]
    fn refinement_removing_an_interpreted_path_refuses() {
        let base = policy("", "\"*.py\", \"*.sh\"", &["\"just\""]);
        let refinement = policy("", "\"*.sh\"", &["\"just\""]);
        assert_eq!(
            restrict(&base, &refinement),
            Err(RestrictionRefusal::InterpretedPathRemoved {
                entry: "*.py".to_owned()
            })
        );
    }

    #[test]
    fn refinement_removing_a_project_verification_row_refuses() {
        let base = with_checks(&["a", "b"]);
        let refinement = with_checks(&["a"]);
        assert_eq!(
            restrict(&base, &refinement),
            Err(RestrictionRefusal::VerificationRowRemoved { index: 1 }),
            "stated in full, dropping a check *is* the refusal — which is why \
             the refinement document is not a delta"
        );
    }

    #[test]
    fn refinement_reordering_project_verification_refuses() {
        let base = with_checks(&["a", "b", "c"]);
        let refinement = with_checks(&["a", "c", "b"]);
        assert_eq!(
            restrict(&base, &refinement),
            Err(RestrictionRefusal::VerificationRowReordered { index: 1 })
        );
    }

    #[test]
    fn refinement_swapping_two_project_rows_refuses_as_reordered() {
        let base = with_checks(&["a", "b"]);
        let refinement = with_checks(&["b", "a"]);
        assert_eq!(
            restrict(&base, &refinement),
            Err(RestrictionRefusal::VerificationRowReordered { index: 0 })
        );
    }

    #[test]
    fn refinement_replacing_a_project_verification_row_refuses() {
        let base = with_checks(&["a", "b"]);
        let refinement = with_checks(&["a", "c"]);
        assert_eq!(
            restrict(&base, &refinement),
            Err(RestrictionRefusal::VerificationRowRemoved { index: 1 }),
            "a replacement is a removal plus an insertion, and the ordered \
             classification catches the removal first — see the note on \
             VerificationRowReplaced"
        );
    }

    #[test]
    fn refinement_inserting_a_row_before_a_project_row_refuses_as_inserted() {
        let base = with_checks(&["a"]);
        let refinement = with_checks(&["x", "a"]);
        assert_eq!(
            restrict(&base, &refinement),
            Err(RestrictionRefusal::VerificationRowInserted { index: 0 }),
            "the base rows keep their relative order, so this is neither a \
             reordering nor a replacement (RV-346 F-17)"
        );
    }

    #[test]
    fn refinement_inserting_a_row_between_project_rows_refuses_as_inserted() {
        let base = with_checks(&["a", "b"]);
        let refinement = with_checks(&["a", "x", "b"]);
        assert_eq!(
            restrict(&base, &refinement),
            Err(RestrictionRefusal::VerificationRowInserted { index: 1 })
        );
    }

    #[test]
    fn the_diagnosis_is_classified_in_the_stated_order() {
        // Removes `b` *and* reorders what is left. Both descriptions are true;
        // the order is what makes the answer deterministic.
        let base = with_checks(&["a", "b", "c"]);
        let refinement = with_checks(&["c", "a"]);
        assert_eq!(
            restrict(&base, &refinement),
            Err(RestrictionRefusal::VerificationRowRemoved { index: 1 }),
            "removal is diagnosed before reordering"
        );
    }

    #[test]
    fn refinement_with_a_different_schema_refuses() {
        // The one fixture not built through `parse`, and deliberately so: today
        // `parse` accepts exactly INTERPRETATION_SCHEMA, so two parsed policies
        // always agree and this rule cannot be reached through a document. It
        // guards the v2 in which `parse` accepts more than one version.
        let base = with_checks(&["a"]);
        let mut refinement = base.clone();
        refinement.schema = INTERPRETATION_SCHEMA + 1;
        assert_eq!(
            restrict(&base, &refinement),
            Err(RestrictionRefusal::SchemaMismatch {
                base: INTERPRETATION_SCHEMA,
                refinement: INTERPRETATION_SCHEMA + 1,
            })
        );
    }

    #[test]
    fn a_removal_is_diagnosed_before_a_verification_change() {
        let base = policy("\"node\"", "", &["\"a\""]);
        let refinement = policy("", "", &["\"b\""]);
        assert_eq!(
            restrict(&base, &refinement),
            Err(RestrictionRefusal::ForbiddenEntryRemoved {
                entry: "node".to_owned()
            }),
            "the rules are evaluated in order, so the earlier axis wins"
        );
    }
}
