// SPDX-License-Identifier: GPL-3.0-only
//! `config` — the `[capsule]` table of `.doctrine/doctrine.toml`, its vocabulary
//! and its two-stage reader (SL-248 `sec-5`, `EX-3`…`EX-13`).
//!
//! This is the one place the operator's configuration enters the slice. The
//! table is **new** rather than a corner of an incumbent one (`DEC-158`):
//! `[interpretation]` is hashed into the work contract, so an operational edit
//! to a capacity threshold would move the canonical hash; `[dispatch]` is the
//! worktree arm's configuration, which this slice sits beside rather than
//! replaces.
//!
//! Keys are **kebab-case** (`EX-4`), which puts this table with `[dispatch]` and
//! `[reservation]`. `[interpretation]`'s `snake_case` is not a house style —
//! `SPEC-030` fixes those spellings verbatim — and `[capsule]` has no such
//! constraint. This amends `sec-2`, whose sample was written `readable_roots` /
//! `closure_roots` / `closure_resolver`.
//!
//! **The reader splits where the table touches the host** (`EX-6`), which is the
//! project's pure/imperative rule applied at its one crossing here: every list
//! rule, every bound and the multiplier are decided from text alone by
//! [`parse_capsule_config`] and are testable without a filesystem; only the
//! capsule root's default reads the environment, in [`root_capsule_config`].
//!
//! **Serde here, a hand walk in `sec-4`, and that is not inconsistency**
//! (`EX-5`). `REQ-449` demands six *distinguishable* refusals, which serde would
//! collapse into a formatted string for four of them. `[capsule]` needs only
//! that a mistyped key be refused rather than silently defaulted —
//! `execution-timeout-secconds` quietly restoring a 300s bound on a project that
//! configured 900 is the hazard — and `#[serde(deny_unknown_fields)]` supplies
//! exactly that while naming the key. Two readers, two requirements, one of them
//! met by a derive.
//!
//! Layering (`ADR-001`): `config` is `leaf`, out-edges `{host}` — the one import
//! is [`HostFacts`], for the root resolution.
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "staged ahead of PHASE-06's provision consumer (PHASE-03 D5); \
                  PHASE-06 deletes this line when `provision` lands"
    )
)]

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;

use crate::host::HostFacts;

// ---------------------------------------------------------------------------
// Named constants (`STD-001`) — no key spelling, default or unit is a literal
// appearing twice.
// ---------------------------------------------------------------------------

/// Bytes in a mebibyte. Sizes are configured in mebibytes *spelled in the key*
/// rather than as suffixed strings: a `"4GiB"` spelling would need a parser, a
/// refusal vocabulary for malformed units, and tests for both, to buy nothing
/// this slice's callers need.
pub(crate) const BYTES_PER_MIB: u64 = 1_048_576;

/// The spike's directly measured `SIGTERM`→`SIGKILL` window
/// (`capsule/sandbox.sh:68`). A window between two signals, not a bound that
/// ends work, which is why it may default at all (`EX-8`).
pub(crate) const EXECUTION_KILL_GRACE_SECONDS_DEFAULT: u64 = 5;

/// Whole-tree expected size — the quantity the spike's 4.4 GiB peak actually
/// measured, which is why this default survives `EX-7`'s argument against the
/// other two. `POL-002` facet 1 is why it is not simply the figure this
/// repository sets for itself.
pub(crate) const EXPECTED_CAPSULE_SIZE_MIB_DEFAULT: u64 = 4096;

/// Warn below twice the expected capsule size (`REQ-461` criterion 2).
pub(crate) const CAPACITY_WARN_MULTIPLIER_DEFAULT: u32 = 2;

/// The subdirectory of the platform data directory that Doctrine owns.
pub(crate) const CAPSULE_ROOT_LEAF: &str = "doctrine/capsules";
/// The XDG data-directory variable, read first.
pub(crate) const XDG_DATA_HOME: &str = "XDG_DATA_HOME";
/// The home variable, read second and held to the *same* test.
pub(crate) const HOME: &str = "HOME";
/// What `HOME` is joined with when it supplies the data directory.
pub(crate) const HOME_DATA_LEAF: &str = ".local/share";

/// One `KEY_*` per `[capsule]` key. These are the strings the refusals name, so
/// an operator reading a refusal is reading the key they must edit.
pub(crate) const KEY_ROOT: &str = "root";
pub(crate) const KEY_READABLE_ROOTS: &str = "readable-roots";
pub(crate) const KEY_CLOSURE_ROOTS: &str = "closure-roots";
pub(crate) const KEY_CLOSURE_RESOLVER: &str = "closure-resolver";
pub(crate) const KEY_EXPECTED_CAPSULE_SIZE_MIB: &str = "expected-capsule-size-mib";
pub(crate) const KEY_CAPACITY_WARN_MULTIPLIER: &str = "capacity-warn-multiplier";
pub(crate) const KEY_EXECUTION_TIMEOUT_SECONDS: &str = "execution-timeout-seconds";
pub(crate) const KEY_FILE_SIZE_CAP_MIB: &str = "file-size-cap-mib";
pub(crate) const KEY_EXECUTION_KILL_GRACE_SECONDS: &str = "execution-kill-grace-seconds";

// ---------------------------------------------------------------------------
// Vocabulary
// ---------------------------------------------------------------------------

/// A byte quantity. Defined **here** (`EX-3`) because this module owns the
/// arithmetic that can overflow, and `sec-2`'s `file_size_cap` and `disk_used`
/// are this type — which is what makes `backend → config` the edge `sec-6`'s
/// unit table records, and what PHASE-04 depends on this phase for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ByteCount(u64);

impl ByteCount {
    pub(crate) const fn from_bytes(bytes: u64) -> Self {
        Self(bytes)
    }

    /// A mebibyte figure converted once, at construction, through **checked**
    /// arithmetic (`EX-9`). `None` above the conversion ceiling; the caller
    /// turns that into `BoundOverflows { key }` rather than wrapping.
    pub(crate) const fn from_mib(mib: u64) -> Option<Self> {
        match mib.checked_mul(BYTES_PER_MIB) {
            Some(bytes) => Some(Self(bytes)),
            None => None,
        }
    }

    pub(crate) const fn as_u64(self) -> u64 {
        self.0
    }

    /// The warning threshold's multiplication, which **saturates** at
    /// `u64::MAX` where the *conversion* above refuses (`EX-9`). The two are
    /// deliberately different: a configured size that cannot be represented is
    /// an operator error, while a threshold beyond the largest possible disk is
    /// simply a threshold nothing reaches.
    pub(crate) fn saturating_mul_u32(self, factor: u32) -> Self {
        Self(self.0.saturating_mul(u64::from(factor)))
    }
}

/// A non-empty argument vector.
///
/// The design uses `Argv` for `Execution.argv` and for the closure resolver but
/// never defines it and assigns it to no unit (`F-2`). It lands here because
/// `sec-6`'s table already records `backend → config`, so this is the only
/// placement that adds no edge the table lacks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Argv(Vec<String>);

impl Argv {
    /// `None` for the empty vector — the case `EmptyResolverArgv` refuses.
    pub(crate) fn try_new(words: Vec<String>) -> Option<Self> {
        (!words.is_empty()).then_some(Self(words))
    }

    pub(crate) fn as_slice(&self) -> &[String] {
        &self.0
    }
}

/// The resource choices `sec-3`'s transaction binds and `sec-2`'s `Execution`
/// carries. Converted once, at construction, into the typed units below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResourceBounds {
    timeout: Duration,
    kill_grace: Duration,
    file_size_cap: ByteCount,
}

impl ResourceBounds {
    pub(crate) const fn timeout(&self) -> Duration {
        self.timeout
    }
    pub(crate) const fn kill_grace(&self) -> Duration {
        self.kill_grace
    }
    pub(crate) const fn file_size_cap(&self) -> ByteCount {
        self.file_size_cap
    }
}

/// The advisory capacity policy `capacity::assess_capacity` reads.
///
/// It holds the expected size and the multiplier and **nothing else** — in
/// particular no reserved figure, which is where `REQ-461`'s "no
/// pre-reservation" is structural rather than merely unimplemented (`EX-20`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CapacityPolicy {
    expected_capsule_size: ByteCount,
    warn_multiplier: u32,
}

impl CapacityPolicy {
    pub(crate) const fn new(expected_capsule_size: ByteCount, warn_multiplier: u32) -> Self {
        Self {
            expected_capsule_size,
            warn_multiplier,
        }
    }
    pub(crate) const fn expected_capsule_size(&self) -> ByteCount {
        self.expected_capsule_size
    }
    pub(crate) const fn warn_multiplier(&self) -> u32 {
        self.warn_multiplier
    }
}

/// Every refusal this reader can produce, each carrying the key or path it is
/// about (`EX-11`).
///
/// The first three of `sec-2` invariant 9's probes are discharged here because
/// this is where the lists are read; its filesystem probes — an entry that does
/// not exist, a `closure-roots` entry that is not realised — need a disk and
/// stay at `sec-3` step 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConfigRefusal {
    /// Not valid TOML, a wrong value type, or an unknown key. The `detail` is
    /// serde's own message, which names the offending key.
    MalformedTable { detail: String },
    /// `readable-roots` and `closure-roots` both empty (`sec-2`).
    NoReadableInputs,
    /// `closure-roots` non-empty with no `closure-resolver`.
    ClosureRootsWithoutResolver,
    /// `closure-resolver = []`.
    EmptyResolverArgv,
    /// A configured `root` that is not absolute.
    RelativeCapsuleRoot { path: PathBuf },
    /// No `root`, and no platform data directory resolves.
    UnresolvableCapsuleRoot,
    /// Any bound or size configured as `0`.
    ZeroBound { key: &'static str },
    /// A required key with no measurement to default it from (`EX-7`).
    BoundMissing { key: &'static str },
    /// A mebibyte figure above the conversion ceiling (`EX-9`).
    BoundOverflows { key: &'static str },
    /// `capacity-warn-multiplier ≤ 1`, at which the warning region is empty and
    /// `REQ-461` criterion 1 could never be met (`EX-10`).
    WarnMultiplierNotAboveOne { found: u32 },
}

impl ConfigRefusal {
    /// The `[capsule]` keys this refusal is about, structured rather than
    /// formatted — an operator triaging a refusal wants the key to edit, and a
    /// sentence is neither machine-readable nor greppable.
    ///
    /// `MalformedTable` names none: serde has already named the offending key
    /// inside `detail`, and this reader does not re-parse its own error text.
    pub(crate) fn keys(&self) -> &[&'static str] {
        match self {
            Self::MalformedTable { .. } => &[],
            Self::NoReadableInputs => &[KEY_READABLE_ROOTS, KEY_CLOSURE_ROOTS],
            Self::ClosureRootsWithoutResolver => &[KEY_CLOSURE_ROOTS, KEY_CLOSURE_RESOLVER],
            Self::EmptyResolverArgv => &[KEY_CLOSURE_RESOLVER],
            Self::RelativeCapsuleRoot { .. } | Self::UnresolvableCapsuleRoot => &[KEY_ROOT],
            Self::ZeroBound { key } | Self::BoundMissing { key } | Self::BoundOverflows { key } => {
                std::slice::from_ref(key)
            }
            Self::WarnMultiplierNotAboveOne { .. } => &[KEY_CAPACITY_WARN_MULTIPLIER],
        }
    }
}

// ---------------------------------------------------------------------------
// The validated configuration — private fields, one fallible constructor each
// ---------------------------------------------------------------------------

/// The parts of the configuration that do not depend on the host, validated.
///
/// Private fields for `EX-6`'s reason: a public literal would let a caller
/// assemble a configuration that never went through the readable-list rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UnrootedCapsuleConfig {
    configured_root: Option<PathBuf>,
    readable_roots: Vec<PathBuf>,
    closure_roots: Vec<PathBuf>,
    closure_resolver: Option<Argv>,
    capacity: CapacityPolicy,
    bounds: ResourceBounds,
}

impl UnrootedCapsuleConfig {
    pub(crate) fn readable_roots(&self) -> &[PathBuf] {
        &self.readable_roots
    }
    pub(crate) fn closure_roots(&self) -> &[PathBuf] {
        &self.closure_roots
    }
    pub(crate) const fn closure_resolver(&self) -> Option<&Argv> {
        self.closure_resolver.as_ref()
    }
    pub(crate) const fn capacity(&self) -> &CapacityPolicy {
        &self.capacity
    }
    pub(crate) const fn bounds(&self) -> &ResourceBounds {
        &self.bounds
    }
}

/// The validated `[capsule]` table, with the capsule root resolved.
///
/// The root is absolute and outside the repository by construction; there is no
/// arrangement of missing configuration that produces a relative one
/// (`sec-5` invariant 3, `EX-12`). It is what populates `sec-2`'s
/// `ForbiddenScopes::capsule_root` in PHASE-04.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CapsuleConfig {
    root: PathBuf,
    readable_roots: Vec<PathBuf>,
    closure_roots: Vec<PathBuf>,
    closure_resolver: Option<Argv>,
    capacity: CapacityPolicy,
    bounds: ResourceBounds,
}

impl CapsuleConfig {
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }
    pub(crate) fn readable_roots(&self) -> &[PathBuf] {
        &self.readable_roots
    }
    pub(crate) fn closure_roots(&self) -> &[PathBuf] {
        &self.closure_roots
    }
    pub(crate) const fn closure_resolver(&self) -> Option<&Argv> {
        self.closure_resolver.as_ref()
    }
    pub(crate) const fn capacity(&self) -> &CapacityPolicy {
        &self.capacity
    }
    pub(crate) const fn bounds(&self) -> &ResourceBounds {
        &self.bounds
    }
}

// ---------------------------------------------------------------------------
// The serde projection
// ---------------------------------------------------------------------------

/// The outer shape projecting just `[capsule]` out of a `doctrine.toml` body.
///
/// **Tolerant, deliberately**: no `deny_unknown_fields` here, because the
/// document it reads also carries `[dispatch]`, `[reservation]`,
/// `[interpretation]` and whatever else. The denial belongs on the inner table
/// and only there. `#[serde(flatten)]` is not used anywhere in this reader — it
/// would silently disable `deny_unknown_fields`, which is the whole of `EX-5`.
#[derive(Debug, Default, Deserialize)]
struct CapsuleDoc {
    #[serde(default)]
    capsule: RawCapsule,
}

/// The `[capsule]` table exactly as written, before any rule is applied.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields, default)]
struct RawCapsule {
    root: Option<PathBuf>,
    readable_roots: Vec<PathBuf>,
    closure_roots: Vec<PathBuf>,
    closure_resolver: Option<Vec<String>>,
    expected_capsule_size_mib: Option<u64>,
    capacity_warn_multiplier: Option<u32>,
    execution_timeout_seconds: Option<u64>,
    file_size_cap_mib: Option<u64>,
    execution_kill_grace_seconds: Option<u64>,
}

// ---------------------------------------------------------------------------
// Stage 1: PURE
// ---------------------------------------------------------------------------

/// A seconds-valued key. `default = None` means **required** — there is no
/// measurement to default it from (`EX-7`), so absent refuses naming the key.
/// Zero always refuses, defaulted or not.
fn seconds_bound(
    value: Option<u64>,
    key: &'static str,
    default: Option<u64>,
) -> Result<Duration, ConfigRefusal> {
    let seconds = value
        .or(default)
        .ok_or(ConfigRefusal::BoundMissing { key })?;
    if seconds == 0 {
        return Err(ConfigRefusal::ZeroBound { key });
    }
    Ok(Duration::from_secs(seconds))
}

/// A mebibyte-valued key, converted **once** into bytes through checked
/// arithmetic. Same `default = None` convention as [`seconds_bound`].
fn mib_bound(
    value: Option<u64>,
    key: &'static str,
    default: Option<u64>,
) -> Result<ByteCount, ConfigRefusal> {
    let mib = value
        .or(default)
        .ok_or(ConfigRefusal::BoundMissing { key })?;
    if mib == 0 {
        return Err(ConfigRefusal::ZeroBound { key });
    }
    ByteCount::from_mib(mib).ok_or(ConfigRefusal::BoundOverflows { key })
}

/// **PURE.** Project and validate everything that does not need the host
/// (`EX-6`): the list rules, every bound, and the multiplier floor.
pub(crate) fn parse_capsule_config(text: &str) -> Result<UnrootedCapsuleConfig, ConfigRefusal> {
    let doc: CapsuleDoc = toml::from_str(text).map_err(|error| ConfigRefusal::MalformedTable {
        detail: error.to_string(),
    })?;
    validate(doc.capsule)
}

fn validate(raw: RawCapsule) -> Result<UnrootedCapsuleConfig, ConfigRefusal> {
    if let Some(root) = raw.root.as_deref().filter(|root| !root.is_absolute()) {
        return Err(ConfigRefusal::RelativeCapsuleRoot {
            path: root.to_path_buf(),
        });
    }

    if raw.readable_roots.is_empty() && raw.closure_roots.is_empty() {
        return Err(ConfigRefusal::NoReadableInputs);
    }

    let closure_resolver = match raw.closure_resolver {
        None if !raw.closure_roots.is_empty() => {
            return Err(ConfigRefusal::ClosureRootsWithoutResolver);
        }
        None => None,
        Some(words) => Some(Argv::try_new(words).ok_or(ConfigRefusal::EmptyResolverArgv)?),
    };

    let bounds = ResourceBounds {
        timeout: seconds_bound(
            raw.execution_timeout_seconds,
            KEY_EXECUTION_TIMEOUT_SECONDS,
            None,
        )?,
        file_size_cap: mib_bound(raw.file_size_cap_mib, KEY_FILE_SIZE_CAP_MIB, None)?,
        kill_grace: seconds_bound(
            raw.execution_kill_grace_seconds,
            KEY_EXECUTION_KILL_GRACE_SECONDS,
            Some(EXECUTION_KILL_GRACE_SECONDS_DEFAULT),
        )?,
    };

    let expected_capsule_size = mib_bound(
        raw.expected_capsule_size_mib,
        KEY_EXPECTED_CAPSULE_SIZE_MIB,
        Some(EXPECTED_CAPSULE_SIZE_MIB_DEFAULT),
    )?;
    let warn_multiplier = raw
        .capacity_warn_multiplier
        .unwrap_or(CAPACITY_WARN_MULTIPLIER_DEFAULT);
    if warn_multiplier <= 1 {
        return Err(ConfigRefusal::WarnMultiplierNotAboveOne {
            found: warn_multiplier,
        });
    }

    Ok(UnrootedCapsuleConfig {
        configured_root: raw.root,
        readable_roots: raw.readable_roots,
        closure_roots: raw.closure_roots,
        closure_resolver,
        capacity: CapacityPolicy::new(expected_capsule_size, warn_multiplier),
        bounds,
    })
}

// ---------------------------------------------------------------------------
// Stage 2: the one host read
// ---------------------------------------------------------------------------

/// A data-directory variable, held to all three conditions: set, non-empty, and
/// **absolute**. An unusable value is skipped, never joined (`EX-12`,
/// `RV-346` `F-16`) — joining an empty `HOME` is what would otherwise root
/// capsules at `.local/share/doctrine/capsules` under the working directory,
/// which on an ordinary invocation is the repository, and that is precisely the
/// outcome `DEC-158` rules out.
///
/// `var_os`, through [`HostFacts`]: this repository bans `std::env::var`
/// (`clippy.toml` `disallowed-methods`, precedent `src/tty.rs:41`), and going
/// through the trait is what makes all sixteen combinations testable without
/// mutating the process environment.
fn usable_data_dir(host: &dyn HostFacts, name: &str) -> Option<PathBuf> {
    let value = host.env_var(name)?;
    if value.is_empty() {
        return None;
    }
    let path = PathBuf::from(value);
    path.is_absolute().then_some(path)
}

/// Resolve the capsule root: configured, else `XDG_DATA_HOME`, else `HOME`
/// joined with `.local/share`, else **refuse** (`EX-12`).
///
/// The last arm is a refusal and not a guess. A capsule root silently landing in
/// a temporary directory is how large live work ends up somewhere a reboot
/// removes; and no platform-directory crate is added, which would buy macOS and
/// Windows arms for a slice whose backend is Linux against the standing
/// zero-new-compiled-crates posture (`EX-13`).
fn resolve_capsule_root(
    configured: Option<PathBuf>,
    host: &dyn HostFacts,
) -> Result<PathBuf, ConfigRefusal> {
    // A configured root reached here has already been held to `is_absolute` by
    // `validate`, so it wins outright.
    if let Some(root) = configured {
        return Ok(root);
    }
    if let Some(data_dir) = usable_data_dir(host, XDG_DATA_HOME) {
        return Ok(data_dir.join(CAPSULE_ROOT_LEAF));
    }
    if let Some(home) = usable_data_dir(host, HOME) {
        return Ok(home.join(HOME_DATA_LEAF).join(CAPSULE_ROOT_LEAF));
    }
    Err(ConfigRefusal::UnresolvableCapsuleRoot)
}

/// Resolve the capsule root — the only part of this table that needs the host
/// (`EX-6`).
pub(crate) fn root_capsule_config(
    parsed: UnrootedCapsuleConfig,
    host: &dyn HostFacts,
) -> Result<CapsuleConfig, ConfigRefusal> {
    let root = resolve_capsule_root(parsed.configured_root, host)?;
    Ok(CapsuleConfig {
        root,
        readable_roots: parsed.readable_roots,
        closure_roots: parsed.closure_roots,
        closure_resolver: parsed.closure_resolver,
        capacity: parsed.capacity,
        bounds: parsed.bounds,
    })
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use super::{
        Argv, ByteCount, CAPACITY_WARN_MULTIPLIER_DEFAULT, CapsuleConfig, ConfigRefusal,
        EXECUTION_KILL_GRACE_SECONDS_DEFAULT, EXPECTED_CAPSULE_SIZE_MIB_DEFAULT, HOME,
        KEY_CAPACITY_WARN_MULTIPLIER, KEY_CLOSURE_RESOLVER, KEY_CLOSURE_ROOTS,
        KEY_EXECUTION_KILL_GRACE_SECONDS, KEY_EXECUTION_TIMEOUT_SECONDS,
        KEY_EXPECTED_CAPSULE_SIZE_MIB, KEY_FILE_SIZE_CAP_MIB, KEY_READABLE_ROOTS, KEY_ROOT,
        UnrootedCapsuleConfig, XDG_DATA_HOME, parse_capsule_config, root_capsule_config,
    };
    use crate::host::fixture::FixtureHost;

    /// The two required keys, present and plausible, so a fixture exercising
    /// some *other* rule is not accidentally refused by these.
    const REQUIRED_BOUNDS: &str = "execution-timeout-seconds = 900\nfile-size-cap-mib = 512";
    /// One declared readable input, so the readable-list rule is satisfied.
    const READABLE: &str = r#"readable-roots = ["/bin/sh"]"#;
    /// The largest mebibyte figure that converts without overflowing `u64`.
    ///
    /// Derived from [`super::BYTES_PER_MIB`] rather than restated, and spelled
    /// as a shift rather than `u64::MAX / BYTES_PER_MIB` because
    /// `clippy::integer_division` is `deny` (`D6`). A mebibyte is a power of
    /// two, so the shift is the division, exactly.
    const MAX_CONVERTIBLE_MIB: u64 = u64::MAX >> super::BYTES_PER_MIB.trailing_zeros();

    fn parse(body: &str) -> Result<UnrootedCapsuleConfig, ConfigRefusal> {
        parse_capsule_config(&format!("[capsule]\n{body}\n"))
    }

    /// A body that satisfies every rule, plus `extra`.
    fn valid(extra: &str) -> String {
        format!("{READABLE}\n{REQUIRED_BOUNDS}\n{extra}\n")
    }

    fn parsed_valid(extra: &str) -> UnrootedCapsuleConfig {
        parse(&valid(extra)).expect("the valid fixture must parse")
    }

    fn refuse(body: &str) -> ConfigRefusal {
        parse(body).expect_err("this fixture must refuse")
    }

    // ── T4 / VT-4: the list and resolver rules ─────────────────────────────

    /// `deny_unknown_fields` is the whole point: a mistyped key must refuse
    /// rather than silently restore a default the project configured away.
    ///
    /// Asserts *containment* of the offending key, never an exact serde
    /// message — `toml`/serde message text is version-fragile (`R4`).
    #[test]
    fn unknown_key_refuses_naming_the_key() {
        const MISTYPED: &str = "execution-timeout-secconds";

        let refusal = refuse(&valid(&format!("{MISTYPED} = 900")));
        let ConfigRefusal::MalformedTable { detail } = refusal else {
            panic!("expected MalformedTable, got {refusal:?}");
        };
        assert!(
            detail.contains(MISTYPED),
            "the refusal must name the offending key; got {detail}"
        );
    }

    #[test]
    fn both_readable_lists_empty_refuses() {
        assert_eq!(refuse(REQUIRED_BOUNDS), ConfigRefusal::NoReadableInputs);
        assert_eq!(
            refuse(REQUIRED_BOUNDS).keys(),
            [KEY_READABLE_ROOTS, KEY_CLOSURE_ROOTS]
        );
    }

    #[test]
    fn closure_roots_without_a_resolver_refuses_naming_both_keys() {
        let refusal = refuse(&format!(
            "{REQUIRED_BOUNDS}\nclosure-roots = [\"/nix/store/thing\"]"
        ));
        assert_eq!(refusal, ConfigRefusal::ClosureRootsWithoutResolver);
        assert_eq!(refusal.keys(), [KEY_CLOSURE_ROOTS, KEY_CLOSURE_RESOLVER]);
    }

    #[test]
    fn empty_resolver_argv_refuses() {
        let refusal = refuse(&valid("closure-resolver = []"));
        assert_eq!(refusal, ConfigRefusal::EmptyResolverArgv);
        assert_eq!(refusal.keys(), [KEY_CLOSURE_RESOLVER]);
    }

    #[test]
    fn relative_configured_root_refuses() {
        const RELATIVE_ROOT: &str = "capsules";

        let refusal = refuse(&valid(&format!(r#"root = "{RELATIVE_ROOT}""#)));
        assert_eq!(
            refusal,
            ConfigRefusal::RelativeCapsuleRoot {
                path: PathBuf::from(RELATIVE_ROOT)
            }
        );
        assert_eq!(refusal.keys(), [KEY_ROOT]);
    }

    // ── T5 / VT-5: the bounds and the conversion ceiling ───────────────────

    #[test]
    fn zero_valued_bound_refuses_naming_the_key() {
        for key in [
            KEY_EXECUTION_TIMEOUT_SECONDS,
            KEY_FILE_SIZE_CAP_MIB,
            KEY_EXECUTION_KILL_GRACE_SECONDS,
            KEY_EXPECTED_CAPSULE_SIZE_MIB,
        ] {
            // Overriding a key already in REQUIRED_BOUNDS would be a duplicate
            // key, so build the body from the lists plus the two required keys
            // with this one zeroed.
            let body = format!(
                "{READABLE}\nexecution-timeout-seconds = {}\nfile-size-cap-mib = {}\n{}",
                if key == KEY_EXECUTION_TIMEOUT_SECONDS {
                    0
                } else {
                    900
                },
                if key == KEY_FILE_SIZE_CAP_MIB { 0 } else { 512 },
                if key == KEY_EXECUTION_KILL_GRACE_SECONDS || key == KEY_EXPECTED_CAPSULE_SIZE_MIB {
                    format!("{key} = 0")
                } else {
                    String::new()
                }
            );
            let refusal = refuse(&body);
            assert_eq!(refusal, ConfigRefusal::ZeroBound { key }, "for {key}");
            assert_eq!(refusal.keys(), [key], "for {key}");
        }
    }

    #[test]
    fn absent_execution_timeout_refuses_naming_the_key() {
        let refusal = refuse(&format!("{READABLE}\nfile-size-cap-mib = 512"));
        assert_eq!(
            refusal,
            ConfigRefusal::BoundMissing {
                key: KEY_EXECUTION_TIMEOUT_SECONDS
            }
        );
        assert_eq!(refusal.keys(), [KEY_EXECUTION_TIMEOUT_SECONDS]);
    }

    #[test]
    fn absent_file_size_cap_refuses_naming_the_key() {
        let refusal = refuse(&format!("{READABLE}\nexecution-timeout-seconds = 900"));
        assert_eq!(
            refusal,
            ConfigRefusal::BoundMissing {
                key: KEY_FILE_SIZE_CAP_MIB
            }
        );
        assert_eq!(refusal.keys(), [KEY_FILE_SIZE_CAP_MIB]);
    }

    #[test]
    fn a_mebibyte_figure_above_the_conversion_ceiling_refuses_naming_the_key() {
        let over = MAX_CONVERTIBLE_MIB + 1;
        let refusal = refuse(&format!(
            "{READABLE}\nexecution-timeout-seconds = 900\nfile-size-cap-mib = {over}"
        ));
        assert_eq!(
            refusal,
            ConfigRefusal::BoundOverflows {
                key: KEY_FILE_SIZE_CAP_MIB
            }
        );
        assert_eq!(refusal.keys(), [KEY_FILE_SIZE_CAP_MIB]);

        // The advisory size converts through the same ceiling.
        let refusal = refuse(&valid(&format!("{KEY_EXPECTED_CAPSULE_SIZE_MIB} = {over}")));
        assert_eq!(
            refusal,
            ConfigRefusal::BoundOverflows {
                key: KEY_EXPECTED_CAPSULE_SIZE_MIB
            }
        );
    }

    /// The other side of the boundary, so the check is not off by one.
    #[test]
    fn the_largest_convertible_mebibyte_figure_is_accepted() {
        let parsed = parse(&format!(
            "{READABLE}\nexecution-timeout-seconds = 900\nfile-size-cap-mib = {MAX_CONVERTIBLE_MIB}"
        ))
        .expect("the ceiling itself must convert");
        assert_eq!(
            parsed.bounds().file_size_cap(),
            ByteCount::from_mib(MAX_CONVERTIBLE_MIB).expect("the ceiling converts by definition")
        );
    }

    #[test]
    fn sizes_in_the_key_unit_convert_once_into_bytes_and_seconds() {
        const TIMEOUT_SECONDS: u64 = 900;
        const FILE_SIZE_CAP_MIB: u64 = 512;
        const GRACE_SECONDS: u64 = 7;
        const EXPECTED_MIB: u64 = 8192;

        let parsed = parse(&format!(
            "{READABLE}\n\
             execution-timeout-seconds = {TIMEOUT_SECONDS}\n\
             file-size-cap-mib = {FILE_SIZE_CAP_MIB}\n\
             execution-kill-grace-seconds = {GRACE_SECONDS}\n\
             expected-capsule-size-mib = {EXPECTED_MIB}"
        ))
        .expect("the fixture must parse");

        assert_eq!(
            parsed.bounds().timeout(),
            Duration::from_secs(TIMEOUT_SECONDS)
        );
        assert_eq!(
            parsed.bounds().kill_grace(),
            Duration::from_secs(GRACE_SECONDS)
        );
        assert_eq!(
            parsed.bounds().file_size_cap().as_u64(),
            FILE_SIZE_CAP_MIB * super::BYTES_PER_MIB
        );
        assert_eq!(
            parsed.capacity().expected_capsule_size().as_u64(),
            EXPECTED_MIB * super::BYTES_PER_MIB
        );
    }

    // ── T6 / VT-6: the multiplier floor and the defaults ───────────────────

    #[test]
    fn warn_multiplier_of_one_refuses_because_the_warning_region_would_be_empty() {
        let refusal = refuse(&valid(&format!("{KEY_CAPACITY_WARN_MULTIPLIER} = 1")));
        assert_eq!(
            refusal,
            ConfigRefusal::WarnMultiplierNotAboveOne { found: 1 }
        );
        assert_eq!(refusal.keys(), [KEY_CAPACITY_WARN_MULTIPLIER]);
    }

    /// Zero refuses as `WarnMultiplierNotAboveOne`, **not** `ZeroBound`: the
    /// multiplier is not a bound, and the empty-warning-region argument is the
    /// one that applies at every value at or below 1.
    #[test]
    fn warn_multiplier_of_zero_refuses() {
        assert_eq!(
            refuse(&valid(&format!("{KEY_CAPACITY_WARN_MULTIPLIER} = 0"))),
            ConfigRefusal::WarnMultiplierNotAboveOne { found: 0 }
        );
    }

    /// The grace and the two capacity keys only — the two enforced bounds have
    /// no default to take (`EX-7`).
    ///
    /// Also proves the outer document is **tolerant**: `[dispatch]` here must be
    /// ignored, while an unknown key *inside* `[capsule]` refuses.
    #[test]
    fn absent_optional_keys_take_the_named_default_constants() {
        const RESOLVER: &str = "nix-store";
        let parsed = parse_capsule_config(&format!(
            "[dispatch]\nsomething = true\n\n\
             [capsule]\n{READABLE}\nclosure-roots = [\"/nix/store/thing\"]\n\
             closure-resolver = [\"{RESOLVER}\", \"--query\"]\n{REQUIRED_BOUNDS}\n"
        ))
        .expect("an unrelated table must be ignored, not refused");

        assert_eq!(
            parsed.bounds().kill_grace(),
            Duration::from_secs(EXECUTION_KILL_GRACE_SECONDS_DEFAULT)
        );
        assert_eq!(
            parsed.capacity().expected_capsule_size(),
            ByteCount::from_mib(EXPECTED_CAPSULE_SIZE_MIB_DEFAULT).expect("the default converts")
        );
        assert_eq!(
            parsed.capacity().warn_multiplier(),
            CAPACITY_WARN_MULTIPLIER_DEFAULT
        );
        assert_eq!(parsed.readable_roots(), [PathBuf::from("/bin/sh")]);
        assert_eq!(parsed.closure_roots(), [PathBuf::from("/nix/store/thing")]);
        assert_eq!(
            parsed
                .closure_resolver()
                .map(Argv::as_slice)
                .and_then(<[String]>::first)
                .map(String::as_str),
            Some(RESOLVER)
        );
    }

    // ── T7 / VT-7: capsule-root resolution ─────────────────────────────────

    const ABSOLUTE_XDG: &str = "/xdg/data";
    const ABSOLUTE_HOME: &str = "/home/agent";
    const CONFIGURED_ROOT: &str = "/var/lib/doctrine/capsules";
    const RELATIVE_VALUE: &str = "relative/data";
    const EMPTY_VALUE: &str = "";

    fn host_with(xdg: Option<&str>, home: Option<&str>) -> FixtureHost {
        let mut host = FixtureHost::new();
        if let Some(value) = xdg {
            host = host.with_env(XDG_DATA_HOME, value);
        }
        if let Some(value) = home {
            host = host.with_env(HOME, value);
        }
        host
    }

    fn root_of(extra: &str, host: &FixtureHost) -> Result<CapsuleConfig, ConfigRefusal> {
        root_capsule_config(parsed_valid(extra), host)
    }

    /// The configured root wins outright — and every other part of the parsed
    /// configuration survives rooting unchanged.
    #[test]
    fn configured_root_wins_over_the_environment() {
        let host = host_with(Some(ABSOLUTE_XDG), Some(ABSOLUTE_HOME));
        let extra = format!(
            "root = \"{CONFIGURED_ROOT}\"\nclosure-roots = [\"/nix/store/thing\"]\n\
             closure-resolver = [\"nix-store\"]"
        );
        let config = root_of(&extra, &host).expect("a configured absolute root resolves");

        assert_eq!(config.root(), Path::new(CONFIGURED_ROOT));
        assert_eq!(config.readable_roots(), [PathBuf::from("/bin/sh")]);
        assert_eq!(config.closure_roots(), [PathBuf::from("/nix/store/thing")]);
        assert!(config.closure_resolver().is_some());
        assert_eq!(config.bounds().timeout(), Duration::from_secs(900));
        assert_eq!(
            config.capacity().warn_multiplier(),
            CAPACITY_WARN_MULTIPLIER_DEFAULT
        );
    }

    #[test]
    fn xdg_data_home_is_used_when_set_and_absolute() {
        let host = host_with(Some(ABSOLUTE_XDG), Some(ABSOLUTE_HOME));
        let config = root_of("", &host).expect("an absolute XDG_DATA_HOME resolves");
        assert_eq!(
            config.root(),
            Path::new(ABSOLUTE_XDG).join(super::CAPSULE_ROOT_LEAF)
        );
    }

    #[test]
    fn home_supplies_the_default_when_xdg_data_home_is_unset() {
        let host = host_with(None, Some(ABSOLUTE_HOME));
        let config = root_of("", &host).expect("an absolute HOME resolves");
        assert_eq!(
            config.root(),
            Path::new(ABSOLUTE_HOME)
                .join(super::HOME_DATA_LEAF)
                .join(super::CAPSULE_ROOT_LEAF)
        );
    }

    /// Skipped, never joined: the arm falls through to `HOME`.
    #[test]
    fn relative_xdg_data_home_is_ignored_rather_than_joined() {
        let host = host_with(Some(RELATIVE_VALUE), Some(ABSOLUTE_HOME));
        let config = root_of("", &host).expect("a usable HOME still resolves");
        assert_eq!(
            config.root(),
            Path::new(ABSOLUTE_HOME)
                .join(super::HOME_DATA_LEAF)
                .join(super::CAPSULE_ROOT_LEAF)
        );
    }

    /// `HOME` is held to the *same* test as `XDG_DATA_HOME` — the asymmetry
    /// `RV-346` `F-16` found.
    #[test]
    fn relative_home_is_ignored_rather_than_joined() {
        let host = host_with(None, Some(RELATIVE_VALUE));
        assert_eq!(
            root_of("", &host).expect_err("a relative HOME must not be joined"),
            ConfigRefusal::UnresolvableCapsuleRoot
        );
    }

    /// The case that would otherwise root capsules under the working directory,
    /// which on an ordinary invocation is the repository.
    #[test]
    fn empty_home_is_ignored_rather_than_joined() {
        let host = host_with(None, Some(EMPTY_VALUE));
        assert_eq!(
            root_of("", &host).expect_err("an empty HOME must not be joined"),
            ConfigRefusal::UnresolvableCapsuleRoot
        );
    }

    /// A **property over all sixteen combinations** of the two variables ×
    /// {unset, empty, relative, absolute}, not a further example: whatever the
    /// environment, resolution either refuses or yields an absolute root. There
    /// is no arrangement of missing configuration that produces a relative one
    /// (`sec-5` invariant 3).
    #[test]
    fn no_resolution_path_yields_a_relative_root() {
        let states = [None, Some(EMPTY_VALUE), Some(RELATIVE_VALUE), Some("/abs")];
        let mut resolved = 0_u32;
        let mut refused = 0_u32;

        for xdg in states {
            for home in states {
                let host = host_with(xdg, home);
                match root_of("", &host) {
                    Ok(config) => {
                        assert!(
                            config.root().is_absolute(),
                            "xdg={xdg:?} home={home:?} produced {}",
                            config.root().display()
                        );
                        resolved += 1;
                    }
                    Err(refusal) => {
                        assert_eq!(refusal, ConfigRefusal::UnresolvableCapsuleRoot);
                        refused += 1;
                    }
                }
            }
        }

        // 4×4: an absolute value in either variable resolves — 4 + 4 − 1 = 7.
        assert_eq!((resolved, refused), (7, 9));
    }

    #[test]
    fn neither_variable_usable_refuses_naming_the_config_key() {
        let host = host_with(Some(RELATIVE_VALUE), Some(EMPTY_VALUE));
        let refusal = root_of("", &host).expect_err("nothing usable must refuse");
        assert_eq!(refusal, ConfigRefusal::UnresolvableCapsuleRoot);
        assert_eq!(refusal.keys(), [KEY_ROOT]);
    }
}
