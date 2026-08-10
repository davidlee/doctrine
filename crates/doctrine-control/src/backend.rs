// SPDX-License-Identifier: GPL-3.0-only
//! `backend` — the confinement contract, its closed vocabularies, and the one
//! validating [`CapsulePlacement`] constructor (SL-248 `sec-2`, `EX-1`…`EX-16`).
//!
//! This unit is where a confinement mechanism is *described* rather than
//! implemented. [`CapsuleBackend`] is deliberately smaller than a launch verb —
//! no work contract, no notification, no harvest, no result publication
//! (`EX-2`). `DEC-156` requires the contract to exist at all because `REQ-459`'s
//! process-teardown and resource-observation criteria are claims about this
//! primitive and have nowhere else to attach, and because a suite driving a
//! particular sandbox tool directly would certify a path production never takes.
//!
//! **The mount set is the confinement**, so an unvalidated placement is an
//! unconfined capsule with a confined shape. That is why [`CapsulePlacement`]
//! has private fields and exactly one constructor (`EX-3`): there is no route to
//! a placement that skipped the reserved-destination, inner-collision,
//! forbidden-scope and filesystem-root rules.
//!
//! **Overlap is bidirectional and the descendant half is the half that matters**
//! (`EX-6`). An earlier draft refused only equality and ancestry, on the
//! reasoning that provisioning computes every path in a placement and therefore
//! computes only its own. `RV-346` `F-10` refuted it: a declared readable entry
//! and closure-resolver output are provisioning-computed in that sense too, and
//! they can name anything. `repository/.git/config`, `credentials/token` and a
//! sibling transaction under the capsule root all pass an equal-or-ancestor test
//! and each is exactly what its scope exists to deny.
//!
//! **The mirror defect is the reason every refusal here is written with the
//! lawful case it must not capture** (`EX-9`). `RV-346` `F-25` found that a
//! validator refusing everything beneath the capsule root refuses the design's
//! only lawful source placement — every conformance row would have failed before
//! running, for a reason unrelated to any property under test. A refusal rule is
//! half a specification until the lawful case is asserted positively.
//!
//! **The two carve-outs are typed fields rather than entries in the vectors**
//! (`EX-7`), and they are evaluated independently and never composed: a source
//! beneath the transaction root is refused, and a readable entry naming the
//! export directory is refused. The vectors stay carve-out-free, so no declared
//! entry of any kind is ever admitted beneath a forbidden scope.
//!
//! **What this unit does not do.** No pure test here stands in for one of the
//! executed-only invariants — read-only attachment, descriptor closure above the
//! standard streams, a computed environment, parent-owned standard streams and
//! established process credentials are properties of a *running* capsule rather
//! than of an argv (`EX-16`). Their homes are `sec-7`'s executed rows. There is
//! no argv in this file at all; `DEC-156` is explicit that shape assertions are
//! necessary and never sufficient. Nor does this unit restate a property count
//! as a numeral (`EX-15`): `sec-7`'s Table A is the single source.
//!
//! Layering (`ADR-001`): `backend` is `leaf`, out-edges `{config, host}` — this
//! file imports [`Argv`] and [`ByteCount`], and the `bubblewrap` submodule
//! imports `crate::host`. A backend *profile* lands as a submodule of this unit,
//! which the layering gate maps to `backend` itself, so a profile adds no **row**
//! — but it does add that edge, which `sec-6`'s unit table does not record
//! (`bubblewrap.rs`'s `F-2`; `notes.md` item 27, closed here).

// `pub(crate)` because PHASE-06's `provision` calls two of its items directly —
// `readable_set` (`D3`, the seam the profile publishes) and
// `profile_owned_host_path` (so `<root>/capsule` has one spelling). The
// layering gate maps a submodule to its parent unit, so this adds no edge.
pub(crate) mod bubblewrap;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::config::{Argv, ByteCount};

// ---------------------------------------------------------------------------
// Named constants (`STD-001`) — the inner layout is named here and nowhere else
// ---------------------------------------------------------------------------

/// The filesystem root, which no resolved host path in a placement may be.
pub(crate) const FILESYSTEM_ROOT: &str = "/";

/// The kernel's process table, mounted by the profile.
pub(crate) const INNER_PROC: &str = "/proc";
/// The minimal device set, mounted by the profile.
pub(crate) const INNER_DEV: &str = "/dev";
/// Scratch space, mounted by the profile.
pub(crate) const INNER_TMP: &str = "/tmp";
/// Where the contracted source export appears. Shadowing it would substitute
/// the export, which is why it is reserved rather than merely conventional.
pub(crate) const INNER_SOURCE: &str = "/source";
/// The capsule's own writable state.
pub(crate) const INNER_CAPSULE: &str = "/capsule";
/// The agent home, and the value of [`CapsuleEnvVar::Home`].
pub(crate) const INNER_AGENT: &str = "/agent";

/// The inner destinations a declared entry may never *name* (`EX-5`).
///
/// **This is the single home for the inner layout** (`D9`, `STD-001`): a
/// backend profile imports these rather than spelling them again. Reserved
/// means naming the destination — an entry *beneath* one of these is the
/// ordinary case and is admitted.
pub(crate) const RESERVED_INNER_DESTINATIONS: &[&str] = &[
    INNER_PROC,
    INNER_DEV,
    INNER_TMP,
    INNER_SOURCE,
    INNER_CAPSULE,
    INNER_AGENT,
];

/// The subdirectory of the capsule root that per-base exports live under.
pub(crate) const EXPORT_DIRECTORY_LEAF: &str = "export";

/// [`CapsuleEnvVar::Term`]'s value — a terminal with no capabilities, so no
/// capsule command negotiates one.
pub(crate) const TERM_VALUE: &str = "dumb";
/// The capsule's own git identity, never the host's (`sec-3`). Author and
/// committer are the same identity, so the value is named once.
pub(crate) const CAPSULE_GIT_IDENTITY_NAME: &str = "Doctrine Capsule";
/// The capsule's git email. `.invalid` is reserved by RFC 2606 and resolves
/// nowhere, which is the point: a capsule commit is not correspondence.
pub(crate) const CAPSULE_GIT_IDENTITY_EMAIL: &str = "capsule@doctrine.invalid";

// ---------------------------------------------------------------------------
// The vocabulary of paths
// ---------------------------------------------------------------------------

/// A destination inside a capsule: **absolute and non-empty by construction**.
///
/// There is no sentinel and no `Option` here, which is how
/// [`CapsulePlacement::working_directory`] has no inherit value (`VA-1`): an
/// inherited working directory would make the capsule's view depend on
/// trusted-side state. The empty path is not absolute, so non-emptiness follows
/// from the one condition rather than being a second check that could drift.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct InnerPath(PathBuf);

impl InnerPath {
    /// `None` for a relative or empty path. The only constructor.
    pub(crate) fn try_new(path: PathBuf) -> Option<Self> {
        path.is_absolute().then_some(Self(path))
    }

    pub(crate) fn as_path(&self) -> &Path {
        &self.0
    }
}

/// One host path made visible at one inner destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MountedPath {
    /// **Already fully resolved** when it reaches this type (`EX-8`,
    /// invariant 3). Resolution precedes validation, so what is validated is
    /// what is bound; and a backend chooses no path of its own (invariant 1),
    /// so every host path here was computed by provisioning.
    host: PathBuf,
    inner: InnerPath,
}

impl MountedPath {
    pub(crate) const fn new(host: PathBuf, inner: InnerPath) -> Self {
        Self { host, inner }
    }

    pub(crate) fn host(&self) -> &Path {
        &self.host
    }

    pub(crate) const fn inner(&self) -> &InnerPath {
        &self.inner
    }
}

/// Whether a capsule may reach the network. Stated, never defaulted silently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NetworkPosture {
    Denied,
    Permitted,
}

/// The host regions a capsule may never reach, named by the trusted side.
///
/// A parameter to [`CapsulePlacement::try_new`] rather than knowledge the
/// placement type holds, because a confinement type cannot know where a
/// particular host keeps its canonical repository (`EX-3`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ForbiddenScopes {
    canonical_repository: PathBuf,
    control_plane_state: PathBuf,
    capsule_root: PathBuf,
    credentials: Vec<PathBuf>,
}

impl ForbiddenScopes {
    /// The only constructor. `credentials` is the list the operator extends —
    /// the git config, the ssh directory, the harness credential store, and
    /// anything else that host keeps.
    pub(crate) const fn new(
        canonical_repository: PathBuf,
        control_plane_state: PathBuf,
        capsule_root: PathBuf,
        credentials: Vec<PathBuf>,
    ) -> Self {
        Self {
            canonical_repository,
            control_plane_state,
            capsule_root,
            credentials,
        }
    }

    /// The capsule root, which is also where the export directory hangs.
    pub(crate) fn capsule_root(&self) -> &Path {
        &self.capsule_root
    }

    /// Every named region, as one sequence — the thing overlap is tested
    /// against. Order is not significant: any overlap refuses.
    pub(crate) fn members(&self) -> impl Iterator<Item = &Path> + '_ {
        [
            self.canonical_repository.as_path(),
            self.control_plane_state.as_path(),
            self.capsule_root.as_path(),
        ]
        .into_iter()
        .chain(self.credentials.iter().map(PathBuf::as_path))
    }
}

// ---------------------------------------------------------------------------
// The two typed carve-outs
// ---------------------------------------------------------------------------

/// The base a placement's source export is permitted to be an export **of**.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AcceptedBase(String);

impl AcceptedBase {
    pub(crate) const fn new(oid: String) -> Self {
        Self(oid)
    }

    /// The oid, for the two places PHASE-06 needs it as text: the `export/<oid>`
    /// leaf, and the refspec/`switch --detach` arguments. Read-only — there is
    /// still exactly one constructor.
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// This placement's own writable state, minted by `sec-3` step 9.
///
/// Defined here rather than with the transaction because `sec-6`'s unit table
/// records the edge `transaction → backend`, and [`CapsulePlacement`] carries
/// this type; defining it above `backend` would close a two-node cycle that this
/// tree's zero tangle baseline rejects outright (`D1`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TransactionRoot(PathBuf);

impl TransactionRoot {
    pub(crate) const fn new(path: PathBuf) -> Self {
        Self(path)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }
}

/// The per-base source export, minted by `sec-3` step 8's publish-or-adopt
/// protocol, carrying the base identity it is an export of.
///
/// It is its own field rather than an element of a vector precisely so that the
/// vectors stay carve-out-free (`EX-7`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceExport {
    host: PathBuf,
    base: AcceptedBase,
}

impl SourceExport {
    pub(crate) const fn new(host: PathBuf, base: AcceptedBase) -> Self {
        Self { host, base }
    }

    pub(crate) fn host(&self) -> &Path {
        &self.host
    }

    pub(crate) const fn base(&self) -> &AcceptedBase {
        &self.base
    }
}

// ---------------------------------------------------------------------------
// Placement: the unvalidated input, the refusals, and the one constructor
// ---------------------------------------------------------------------------

/// Everything [`CapsulePlacement::try_new`] is asked to validate.
///
/// Public where the placement is private: this is the *request*, and it is
/// worthless without the constructor that turns it into a placement.
#[derive(Debug, Clone)]
pub(crate) struct PlacementParts {
    pub(crate) root: TransactionRoot,
    pub(crate) source: SourceExport,
    pub(crate) writable: Vec<MountedPath>,
    pub(crate) readable: Vec<MountedPath>,
    pub(crate) working_directory: InnerPath,
    pub(crate) network: NetworkPosture,
    /// What `source` is permitted to be an export of. Checked and discarded:
    /// the placement carries the export, and the export carries its identity.
    pub(crate) accepted_base: AcceptedBase,
}

/// Why a placement was refused, each variant carrying the path or inner
/// destination it is about.
///
/// Structured, never formatted — the same posture as `config`'s
/// `ConfigRefusal::keys`: an operator triaging a refusal wants the path to fix,
/// and a sentence is neither machine-readable nor greppable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PlacementRefusal {
    /// No readable entry at all. A capsule with no readable input has no
    /// executable, so this refuses rather than running unconfined (`D4`,
    /// invariant 9). An empty `writable` vector is lawful: `root` is a typed
    /// field and supplies the capsule's writable state.
    NoReadableEntries,
    /// A reserved inner destination supplied as an ordinary declared entry.
    ReservedInnerDestination { inner: PathBuf },
    /// Two declared entries whose inner paths are equal, or one an ancestor of
    /// the other — so mount order can never decide what is visible. One rule
    /// and therefore one variant (`D3`), exercised from both directions.
    InnerPathCollision { inner: [PathBuf; 2] },
    /// A resolved host path overlapping a [`ForbiddenScopes`] member in either
    /// direction, outside the two typed carve-outs.
    ForbiddenScopeOverlap { path: PathBuf },
    /// A source that does not descend from the export directory — including one
    /// beneath this placement's own transaction root, which the *other*
    /// carve-out would otherwise seem to admit.
    SourceOutsideExportDirectory { path: PathBuf },
    /// A well-typed source export under the export directory carrying the wrong
    /// base identity: a sibling export, refused here and not only later.
    SourceBaseMismatch {
        path: PathBuf,
        carried: AcceptedBase,
    },
    /// A resolved host path that is the filesystem root.
    FilesystemRoot { path: PathBuf },
}

impl PlacementRefusal {
    /// The paths this refusal is about, structured rather than formatted.
    ///
    /// [`PlacementRefusal::NoReadableEntries`] names none: it is about an
    /// absence, and there is no path to point at.
    pub(crate) fn paths(&self) -> &[PathBuf] {
        match self {
            Self::NoReadableEntries => &[],
            Self::ReservedInnerDestination { inner } => std::slice::from_ref(inner),
            Self::InnerPathCollision { inner } => inner.as_slice(),
            Self::ForbiddenScopeOverlap { path }
            | Self::SourceOutsideExportDirectory { path }
            | Self::SourceBaseMismatch { path, .. }
            | Self::FilesystemRoot { path } => std::slice::from_ref(path),
        }
    }
}

/// Where a capsule's state lives on the host and what may be read into it.
///
/// **Every field is private and the only constructor validates** (`EX-3`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CapsulePlacement {
    root: TransactionRoot,
    source: SourceExport,
    writable: Vec<MountedPath>,
    readable: Vec<MountedPath>,
    working_directory: InnerPath,
    network: NetworkPosture,
}

/// True when either path lies on the other's root-ward chain.
///
/// Both arguments are already resolved, so this is a **component** comparison
/// and not a textual prefix test: `/var/lib/doctrine-other` is not under
/// `/var/lib/doctrine`. One implementation, and the only one in this unit
/// (`EX-6`).
fn overlaps(a: &Path, b: &Path) -> bool {
    a == b || a.starts_with(b) || b.starts_with(a)
}

fn is_filesystem_root(path: &Path) -> bool {
    path == Path::new(FILESYSTEM_ROOT)
}

/// The inner-destination rules, over the declared entries of both vectors.
fn check_inner_destinations(
    readable: &[MountedPath],
    writable: &[MountedPath],
) -> Result<(), PlacementRefusal> {
    let entries: Vec<&MountedPath> = readable.iter().chain(writable.iter()).collect();

    for entry in &entries {
        let inner = entry.inner().as_path();
        if RESERVED_INNER_DESTINATIONS
            .iter()
            .any(|reserved| Path::new(reserved) == inner)
        {
            return Err(PlacementRefusal::ReservedInnerDestination {
                inner: inner.to_path_buf(),
            });
        }
    }

    for (index, first) in entries.iter().enumerate() {
        for second in entries.iter().skip(index + 1) {
            let (first, second) = (first.inner().as_path(), second.inner().as_path());
            if overlaps(first, second) {
                return Err(PlacementRefusal::InnerPathCollision {
                    inner: [first.to_path_buf(), second.to_path_buf()],
                });
            }
        }
    }

    Ok(())
}

/// The source carve-out: descent from the export directory **and** a matching
/// base identity. Both halves, or a sibling export passes vacuously.
fn check_source_export(
    source: &SourceExport,
    accepted: &AcceptedBase,
    scopes: &ForbiddenScopes,
) -> Result<(), PlacementRefusal> {
    let export_directory = scopes.capsule_root().join(EXPORT_DIRECTORY_LEAF);
    let host = source.host();

    if host == export_directory || !host.starts_with(&export_directory) {
        return Err(PlacementRefusal::SourceOutsideExportDirectory {
            path: host.to_path_buf(),
        });
    }
    if source.base() != accepted {
        return Err(PlacementRefusal::SourceBaseMismatch {
            path: host.to_path_buf(),
            carried: source.base().clone(),
        });
    }
    Ok(())
}

/// One declared entry against the host-path rules.
///
/// `licensed` is this placement's own transaction root for a **writable** entry
/// and `None` for a readable one — the carve-out is writable-only. The
/// filesystem-root rule is tested before the general overlap rule so that the
/// specific diagnosis wins: `/` is an ancestor of every scope, so an entry there
/// would otherwise be reported as an ordinary overlap and the more precise
/// refusal would be unreachable.
fn check_declared_entry(
    entry: &MountedPath,
    scopes: &ForbiddenScopes,
    licensed: Option<&Path>,
) -> Result<(), PlacementRefusal> {
    let host = entry.host();

    if is_filesystem_root(host) {
        return Err(PlacementRefusal::FilesystemRoot {
            path: host.to_path_buf(),
        });
    }
    if licensed.is_some_and(|root| host.starts_with(root)) {
        return Ok(());
    }
    for member in scopes.members() {
        if overlaps(host, member) {
            return Err(PlacementRefusal::ForbiddenScopeOverlap {
                path: host.to_path_buf(),
            });
        }
    }
    Ok(())
}

impl CapsulePlacement {
    /// The one constructor (`EX-3`).
    ///
    /// The check order is inner-path rules → carve-outs → the host-path walk
    /// (`D5`). The carve-outs are evaluated first and mark what they license,
    /// and they are **never composed**: a source beneath the transaction root
    /// fails the source rule, and a readable entry naming the export directory
    /// is an ordinary overlap. That order is what lets a writable entry under
    /// this placement's own root be admitted by the same test that refuses a
    /// sibling transaction.
    pub(crate) fn try_new(
        parts: PlacementParts,
        scopes: &ForbiddenScopes,
    ) -> Result<Self, PlacementRefusal> {
        if parts.readable.is_empty() {
            return Err(PlacementRefusal::NoReadableEntries);
        }

        check_inner_destinations(&parts.readable, &parts.writable)?;
        check_source_export(&parts.source, &parts.accepted_base, scopes)?;

        if is_filesystem_root(parts.root.path()) {
            return Err(PlacementRefusal::FilesystemRoot {
                path: parts.root.path().to_path_buf(),
            });
        }

        for entry in &parts.readable {
            check_declared_entry(entry, scopes, None)?;
        }
        for entry in &parts.writable {
            check_declared_entry(entry, scopes, Some(parts.root.path()))?;
        }

        Ok(Self {
            root: parts.root,
            source: parts.source,
            writable: parts.writable,
            readable: parts.readable,
            working_directory: parts.working_directory,
            network: parts.network,
        })
    }

    pub(crate) const fn root(&self) -> &TransactionRoot {
        &self.root
    }

    pub(crate) const fn source(&self) -> &SourceExport {
        &self.source
    }

    pub(crate) fn writable(&self) -> &[MountedPath] {
        &self.writable
    }

    pub(crate) fn readable(&self) -> &[MountedPath] {
        &self.readable
    }

    pub(crate) const fn working_directory(&self) -> &InnerPath {
        &self.working_directory
    }

    pub(crate) const fn network(&self) -> NetworkPosture {
        self.network
    }
}

// ---------------------------------------------------------------------------
// The two closed vocabularies, and one run
// ---------------------------------------------------------------------------

/// The environment a capsule may be given, at this slice's altitude.
///
/// A `BTreeMap<String, String>` was the first shape and is rejected: a
/// `--clearenv`-style flag stops *inheritance* only, so an open map is a second
/// unguarded route for exactly the credentials the mount set denies — a caller
/// passing a token by value would satisfy every mount assertion in `sec-7`.
/// Credential denial cannot be claimed to rest on absence from the mount set
/// while another public field carries values in (`EX-11`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CapsuleEnvVar {
    /// Derived from the bound readable paths, never the host's. Its value is
    /// composed by the backend profile at execute time from the placement it is
    /// given, which is why it is the one variant with no fixed value here
    /// (`D6`).
    Path,
    /// Always the inner agent home.
    Home,
    /// Always [`TERM_VALUE`].
    Term,
    GitAuthorName,
    GitAuthorEmail,
    GitCommitterName,
    GitCommitterEmail,
}

impl CapsuleEnvVar {
    /// The whole vocabulary, in one place, for the callers that need to name it
    /// as data. The exhaustive `match` in [`CapsuleEnvVar::fixed_value`] is what
    /// makes widening the enum a compile error rather than a silent gap here.
    pub(crate) const ALL: &'static [Self] = &[
        Self::Path,
        Self::Home,
        Self::Term,
        Self::GitAuthorName,
        Self::GitAuthorEmail,
        Self::GitCommitterName,
        Self::GitCommitterEmail,
    ];

    /// The trusted-side value this variable always takes, or `None` when the
    /// value is derived from the placement at execute time.
    ///
    /// Every value here is a constant of this unit: none is caller-supplied
    /// text, and there is no route by which it could become any (`EX-11`).
    pub(crate) const fn fixed_value(self) -> Option<&'static str> {
        match self {
            Self::Path => None,
            Self::Home => Some(INNER_AGENT),
            Self::Term => Some(TERM_VALUE),
            Self::GitAuthorName | Self::GitCommitterName => Some(CAPSULE_GIT_IDENTITY_NAME),
            Self::GitAuthorEmail | Self::GitCommitterEmail => Some(CAPSULE_GIT_IDENTITY_EMAIL),
        }
    }

    /// The environment variable name this variant is set under.
    ///
    /// Here rather than in the profile, so a variable's **name** and its
    /// **value** are single-sourced in the same place (`STD-001`; `notes.md`
    /// item 34(b), closed in SL-248 PHASE-06). Same exhaustive-match discipline
    /// as [`CapsuleEnvVar::fixed_value`]: widening the enum is a compile error
    /// here rather than a silent gap.
    ///
    /// `PATH` has a *second* role — the host variable a profile reads through
    /// `HostFacts::env_var` to derive the capsule's own — and the profile's
    /// `HOST_PATH_VARIABLE` is defined as `CapsuleEnvVar::Path.name()`, so both
    /// roles have one spelling and it is this one.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Path => "PATH",
            Self::Home => "HOME",
            Self::Term => "TERM",
            Self::GitAuthorName => "GIT_AUTHOR_NAME",
            Self::GitAuthorEmail => "GIT_AUTHOR_EMAIL",
            Self::GitCommitterName => "GIT_COMMITTER_NAME",
            Self::GitCommitterEmail => "GIT_COMMITTER_EMAIL",
        }
    }
}

/// The environment of one run: an ordered set of variables and **no text**.
///
/// **Not the whole of a capsule's environment.** `bwrap` writes `PWD` into the
/// child's exec block itself, after `--clearenv` and after every `--setenv`,
/// naming the directory `--chdir` moved it to. A conforming capsule's environment
/// is therefore this set **plus `PWD`** — an entry doctrine never declared and
/// cannot suppress. Row 11 states its equality over that larger set and admits the
/// entry by **whole value, never by name**, because without `--chdir` the same
/// name would carry a host path. Deliberately not fixed by adding `PWD` here,
/// which would change production `--setenv` output to make a test tidier
/// (`RV-352` / `notes.md` item 128).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CapsuleEnv(BTreeSet<CapsuleEnvVar>);

impl CapsuleEnv {
    /// The only route in, and it carries variants (`D6`).
    pub(crate) const fn new(vars: BTreeSet<CapsuleEnvVar>) -> Self {
        Self(vars)
    }

    /// The whole vocabulary — what an ordinary capsule run is given.
    pub(crate) fn complete() -> Self {
        Self::new(CapsuleEnvVar::ALL.iter().copied().collect())
    }

    pub(crate) fn vars(&self) -> impl Iterator<Item = CapsuleEnvVar> + '_ {
        self.0.iter().copied()
    }
}

/// The standard-stream posture a capsule may be given.
///
/// One variant, deliberately, and the same argument as the environment one
/// channel across: a field typed as a raw descriptor, or an `Inherit` variant,
/// would make handing a capsule a live trusted-side descriptor the ordinary case
/// rather than an unrepresentable one (`EX-12`). Widening this is a later
/// slice's decision and must arrive with its own governed rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CapsuleStdio {
    /// Descriptor 0 is an empty source: reads return EOF immediately.
    /// Descriptors 1 and 2 are **one-way** endpoints the parent created and
    /// reads. One-way rather than merely parent-created, because a socket pair
    /// would satisfy *the parent made it* while remaining readable from inside:
    /// the **capsule** could `read(2)` descriptor 1 and receive whatever the
    /// trusted side put there — an inbound channel nothing here declares. A
    /// capture pipe's write end answers that read with `EBADF`. (Row 12 measures
    /// that direction. An earlier wording justified the rule by a socket pair
    /// "carrying bytes back into the trusted side", which is the *specified*
    /// behaviour of a capture endpoint rather than the hazard — corrected at
    /// reconcile, `RV-352` / `notes.md` item 129.)
    /// `/dev/null` is deliberately not named: the property is
    /// *yields no bytes*, not *is `/dev/null`*.
    EmptyInputCapturedOutput,
}

/// One run inside a capsule.
///
/// Private fields and one constructor, so `argv` can only be a typed
/// [`Argv`] — never a shell string, because a string would put quoting on a
/// security boundary — and `env`/`stdio` can only be the closed vocabularies
/// above (`EX-10`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Execution {
    argv: Argv,
    env: CapsuleEnv,
    timeout: Duration,
    file_size_cap: ByteCount,
    stdio: CapsuleStdio,
}

impl Execution {
    /// The only constructor. There is no route to an [`Execution`] with an
    /// empty argument vector: [`Argv`] itself refuses one (invariant 9).
    pub(crate) const fn new(
        argv: Argv,
        env: CapsuleEnv,
        timeout: Duration,
        file_size_cap: ByteCount,
        stdio: CapsuleStdio,
    ) -> Self {
        Self {
            argv,
            env,
            timeout,
            file_size_cap,
            stdio,
        }
    }

    pub(crate) const fn argv(&self) -> &Argv {
        &self.argv
    }

    pub(crate) const fn env(&self) -> &CapsuleEnv {
        &self.env
    }

    pub(crate) const fn timeout(&self) -> Duration {
        self.timeout
    }

    pub(crate) const fn file_size_cap(&self) -> ByteCount {
        self.file_size_cap
    }

    pub(crate) const fn stdio(&self) -> CapsuleStdio {
        self.stdio
    }
}

// ---------------------------------------------------------------------------
// What a backend reports, and the contract itself
// ---------------------------------------------------------------------------

/// How a run ended, as distinguishable states rather than a status code the
/// caller has to classify — *the runner refused* and *the runner never ran*
/// otherwise read identically (`EX-13`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Termination {
    Exited {
        code: i32,
    },
    Signalled {
        signal: i32,
    },
    TimedOut,
    FileSizeExceeded,
    /// The command could not be executed at all — a missing binary, or a
    /// shebang interpreter outside the readable set.
    NotExecutable,
}

/// What the trusted parent observed.
///
/// Every field is observed by the **parent**; nothing here is reported by the
/// capsule (`REQ-448` criterion 3: worker output is evidence, never authority).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Observation {
    pub(crate) termination: Termination,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
    /// Bytes resident beneath the transaction root after the run.
    pub(crate) disk_used: ByteCount,
}

/// Whether this host can run this backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Availability {
    Available,
    /// `POL-002` facet 3: what was missing, and what would satisfy it.
    Unavailable {
        missing: String,
        remedy: String,
    },
}

/// A failure of the **backend**, never a capsule's own nonzero exit (`EX-14`).
///
/// A capsule that exits 1 is `Ok(Observation { termination: Exited { code: 1 },
/// .. })`. Collapsing the two would make a working capsule reporting failure
/// indistinguishable from a broken confinement mechanism.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BackendError {
    /// The backend cannot run on this host at all.
    Unavailable { id: BackendId },
    /// The confinement mechanism itself failed — it could not be started, or it
    /// died before the capsule's command was reached.
    MechanismFailed { detail: String },
}

/// A backend's stable identity, recorded in an admission verdict.
///
/// A `&'static str` rather than a `String`, so it cannot be built from runtime
/// text and identity is stable by construction; and no `Display` impl, so it
/// cannot drift into a rendering (`D7`). Not a closed enum, because the contract
/// must bind mechanisms nobody has written yet (`DEC-156`) — a profile mints its
/// own constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct BackendId(&'static str);

impl BackendId {
    pub(crate) const fn new(id: &'static str) -> Self {
        Self(id)
    }

    pub(crate) const fn as_str(self) -> &'static str {
        self.0
    }
}

/// One confinement mechanism, admitted by passing the property suite.
///
/// Every method is total with respect to the host: an unavailable backend
/// reports that it is unavailable rather than failing at execution time. The
/// method set is deliberately this small (`EX-1`, `EX-2`) — and dyn-compatible,
/// because provisioning holds one of these behind a reference.
pub(crate) trait CapsuleBackend {
    /// The stable identity recorded in an admission verdict. Never a display
    /// string.
    fn id(&self) -> BackendId;

    /// Whether this host can run this backend, and what is missing when it
    /// cannot.
    fn availability(&self) -> Availability;

    /// Make the placement's readable paths readable and its writable paths
    /// writable inside a capsule, run `execution` there, and return what the
    /// trusted parent observed.
    fn execute(
        &self,
        placement: &CapsulePlacement,
        execution: &Execution,
    ) -> Result<Observation, BackendError>;
}

/// The shared backend double.
///
/// Lives here, `pub(crate)` and `#[cfg(test)]`, on `host::fixture`'s precedent
/// (`D4`): PHASE-06's `provision` tests need a backend that answers *per call*,
/// and a second double in `provision` would be a second definition of what a
/// backend does — the parallel implementation the project rules forbid.
///
/// It asserts **no** confinement property whatsoever: it runs nothing, isolates
/// nothing, and observes nothing. Every property of a running capsule is
/// `sec-7`'s, executed.
#[cfg(test)]
pub(crate) mod fixture {
    use std::cell::RefCell;
    use std::collections::VecDeque;

    use super::{
        Availability, BackendError, BackendId, ByteCount, CapsuleBackend, CapsulePlacement,
        Execution, Observation, Termination,
    };

    pub(crate) const WITNESS_ID: BackendId = BackendId::new("witness");

    /// One canned outcome per call, then a fallback.
    ///
    /// The queue is what makes `sec-3`'s four executions testable at all: a
    /// single canned outcome cannot distinguish *the clone failed* from *the
    /// detach failed*, and `EX-14`'s whole claim is that provisioning names
    /// which.
    #[derive(Debug)]
    pub(crate) struct WitnessBackend {
        availability: Availability,
        script: RefCell<VecDeque<Result<Observation, BackendError>>>,
        fallback: Result<Observation, BackendError>,
        calls: RefCell<Vec<Execution>>,
    }

    impl WitnessBackend {
        /// Every call answers `outcome`.
        pub(crate) fn always(outcome: Result<Observation, BackendError>) -> Self {
            Self::scripted(Vec::new(), outcome)
        }

        /// The first calls answer from `script`, in order; every later call
        /// answers `fallback`.
        pub(crate) fn scripted(
            script: Vec<Result<Observation, BackendError>>,
            fallback: Result<Observation, BackendError>,
        ) -> Self {
            Self {
                availability: Availability::Available,
                script: RefCell::new(script.into()),
                fallback,
                calls: RefCell::new(Vec::new()),
            }
        }

        pub(crate) fn reporting(mut self, availability: Availability) -> Self {
            self.availability = availability;
            self
        }

        /// Every execution this backend was asked for, in order — the evidence
        /// that the argv provisioning built is the argv that ran.
        pub(crate) fn calls(&self) -> Vec<Execution> {
            self.calls.borrow().clone()
        }
    }

    impl CapsuleBackend for WitnessBackend {
        fn id(&self) -> BackendId {
            WITNESS_ID
        }

        fn availability(&self) -> Availability {
            self.availability.clone()
        }

        fn execute(
            &self,
            _placement: &CapsulePlacement,
            execution: &Execution,
        ) -> Result<Observation, BackendError> {
            self.calls.borrow_mut().push(execution.clone());
            self.script
                .borrow_mut()
                .pop_front()
                .unwrap_or_else(|| self.fallback.clone())
        }
    }

    /// A capsule that exited with `code`, having printed `stdout`.
    pub(crate) fn exited(code: i32, stdout: &str) -> Result<Observation, BackendError> {
        Ok(Observation {
            termination: Termination::Exited { code },
            stdout: stdout.as_bytes().to_vec(),
            stderr: Vec::new(),
            disk_used: ByteCount::from_bytes(0),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use super::fixture::{WITNESS_ID, WitnessBackend};
    use super::{
        AcceptedBase, Availability, BackendError, BackendId, CapsuleBackend, CapsuleEnv,
        CapsuleEnvVar, CapsulePlacement, CapsuleStdio, Execution, ForbiddenScopes, InnerPath,
        MountedPath, NetworkPosture, Observation, PlacementParts, PlacementRefusal, SourceExport,
        Termination, TransactionRoot, overlaps,
    };
    use crate::config::{Argv, ByteCount, parse_capsule_config, root_capsule_config};
    use crate::host::HostFacts;
    use crate::host::fixture::FixtureHost;

    // ── Fixture geometry ───────────────────────────────────────────────────
    //
    // `root` is overloaded, so the names here keep the four apart: the
    // *capsule root* (a forbidden scope), the *transaction root* (this
    // placement's writable carve-out), a *declared root* (an ordinary readable
    // or writable entry, which is what the `declared_root_*` titles are about),
    // and the *filesystem root*.

    const CANONICAL_REPOSITORY: &str = "/srv/repo";
    const CONTROL_PLANE_STATE: &str = "/srv/repo/.doctrine";
    const CAPSULE_ROOT: &str = "/var/lib/doctrine";
    const CREDENTIALS: &str = "/home/agent/.ssh";

    const BASE_OID: &str = "1f0e3dad99908345f7439f8ffabdffc4";
    const OTHER_BASE_OID: &str = "9b74c9897bac770ffc029102a200c5de";

    const TRANSACTION_ROOT: &str = "/var/lib/doctrine/tx/0001";
    const SIBLING_TRANSACTION_ROOT: &str = "/var/lib/doctrine/tx/0002";
    const EXPORT_DIRECTORY: &str = "/var/lib/doctrine/export";
    const LAWFUL_EXPORT: &str = "/var/lib/doctrine/export/1f0e3dad99908345f7439f8ffabdffc4";
    const SIBLING_EXPORT: &str = "/var/lib/doctrine/export/9b74c9897bac770ffc029102a200c5de";

    const LAWFUL_HOST: &str = "/nix/store/bash";
    const LAWFUL_INNER: &str = "/nix/store/bash";

    fn scopes() -> ForbiddenScopes {
        ForbiddenScopes::new(
            PathBuf::from(CANONICAL_REPOSITORY),
            PathBuf::from(CONTROL_PLANE_STATE),
            PathBuf::from(CAPSULE_ROOT),
            vec![PathBuf::from(CREDENTIALS)],
        )
    }

    fn inner(path: &str) -> InnerPath {
        InnerPath::try_new(PathBuf::from(path)).expect("fixture inner paths are absolute")
    }

    fn mount(host: &str, at: &str) -> MountedPath {
        MountedPath::new(PathBuf::from(host), inner(at))
    }

    /// A placement request that satisfies every rule. Each test perturbs one
    /// thing, so a refusal is attributable to the perturbation.
    fn lawful_parts() -> PlacementParts {
        PlacementParts {
            root: TransactionRoot::new(PathBuf::from(TRANSACTION_ROOT)),
            source: SourceExport::new(
                PathBuf::from(LAWFUL_EXPORT),
                AcceptedBase::new(BASE_OID.to_owned()),
            ),
            writable: Vec::new(),
            readable: vec![mount(LAWFUL_HOST, LAWFUL_INNER)],
            working_directory: inner(super::INNER_CAPSULE),
            network: NetworkPosture::Denied,
            accepted_base: AcceptedBase::new(BASE_OID.to_owned()),
        }
    }

    fn validate(parts: PlacementParts) -> Result<CapsulePlacement, PlacementRefusal> {
        CapsulePlacement::try_new(parts, &scopes())
    }

    /// The lawful request, plus one further readable entry.
    fn with_readable(host: &str, at: &str) -> Result<CapsulePlacement, PlacementRefusal> {
        let mut parts = lawful_parts();
        parts.readable.push(mount(host, at));
        validate(parts)
    }

    /// The lawful request, plus one writable entry.
    fn with_writable(host: &str, at: &str) -> Result<CapsulePlacement, PlacementRefusal> {
        let mut parts = lawful_parts();
        parts.writable.push(mount(host, at));
        validate(parts)
    }

    fn refusal(outcome: Result<CapsulePlacement, PlacementRefusal>) -> PlacementRefusal {
        outcome.expect_err("this placement must refuse")
    }

    // ── T3: overlap is a component comparison ──────────────────────────────

    /// `A3` measured rather than assumed: `Path::starts_with` compares
    /// components, so a sibling whose *name* extends a scope's name is not
    /// under it. Written before the validator consumed `overlaps`.
    #[test]
    fn overlap_is_bidirectional_over_components_not_text() {
        let scope = Path::new("/var/lib/doctrine");

        assert!(overlaps(scope, scope), "equality overlaps");
        assert!(
            overlaps(Path::new("/var/lib/doctrine/tx/0001"), scope),
            "a descendant overlaps — the half `F-10` found missing"
        );
        assert!(
            overlaps(Path::new("/var/lib"), scope),
            "an ancestor overlaps"
        );
        assert!(
            !overlaps(Path::new("/var/lib/doctrine-other"), scope),
            "a component comparison, not a textual prefix test"
        );
        assert!(
            !overlaps(Path::new("/nix/store/bash"), scope),
            "an unrelated path does not overlap"
        );
    }

    // ── T4 / VT-1: a declared entry resolving into a forbidden scope ───────
    //
    // The `RV-346` `F-1` class, ordered first because it was found by
    // execution rather than by reading. Each test pairs the refusal with the
    // near-miss it must not capture (`EX-9`).

    const DECLARED_INNER: &str = "/work";

    fn refuses_at(host: &str) -> PlacementRefusal {
        refusal(with_readable(host, DECLARED_INNER))
    }

    fn admits_at(host: &str) {
        assert!(
            with_readable(host, DECLARED_INNER).is_ok(),
            "{host} names no forbidden scope and must be admitted"
        );
    }

    #[test]
    fn declared_root_that_resolves_into_the_canonical_repository_refuses() {
        assert_eq!(
            refuses_at("/srv/repo/src"),
            PlacementRefusal::ForbiddenScopeOverlap {
                path: PathBuf::from("/srv/repo/src")
            }
        );
        // The refusal names the path, structured rather than formatted.
        assert_eq!(
            refuses_at("/srv/repo/src").paths(),
            [PathBuf::from("/srv/repo/src")]
        );
        admits_at("/srv/elsewhere/src");
    }

    #[test]
    fn declared_root_that_resolves_into_the_credential_scope_refuses() {
        assert_eq!(
            refuses_at(CREDENTIALS),
            PlacementRefusal::ForbiddenScopeOverlap {
                path: PathBuf::from(CREDENTIALS)
            }
        );
        admits_at("/home/agent/work");
    }

    #[test]
    fn declared_root_that_resolves_into_the_capsule_root_refuses() {
        assert_eq!(
            refuses_at("/var/lib/doctrine/somewhere"),
            PlacementRefusal::ForbiddenScopeOverlap {
                path: PathBuf::from("/var/lib/doctrine/somewhere")
            }
        );
        admits_at("/var/lib/other");
    }

    /// The filesystem root gets its **own** refusal rather than being reported
    /// as an ordinary overlap: `/` is an ancestor of every scope, so a general
    /// overlap test would swallow it and the specific diagnosis would be
    /// unreachable. Deleting the rule turns this assertion red rather than
    /// leaving it green on the other rule's answer.
    #[test]
    fn declared_root_that_resolves_to_the_filesystem_root_refuses() {
        assert_eq!(
            refuses_at(super::FILESYSTEM_ROOT),
            PlacementRefusal::FilesystemRoot {
                path: PathBuf::from(super::FILESYSTEM_ROOT)
            }
        );
        admits_at("/nix");
    }

    /// The capsule root a placement is validated against is the **resolved**
    /// one the reader produced, not a literal restated here (`sec-5`'s
    /// alignment). Drives it through the real two-stage reader.
    fn resolved_capsule_root() -> PathBuf {
        let parsed = parse_capsule_config(&format!(
            "[capsule]\nroot = \"{CAPSULE_ROOT}\"\nreadable-roots = [\"/bin/sh\"]\n\
             execution-timeout-seconds = 900\nfile-size-cap-mib = 512\n"
        ))
        .expect("the fixture table must parse");
        root_capsule_config(parsed, &FixtureHost::new())
            .expect("a configured absolute root resolves")
            .root()
            .to_path_buf()
    }

    fn scopes_from_resolved_root() -> ForbiddenScopes {
        ForbiddenScopes::new(
            PathBuf::from(CANONICAL_REPOSITORY),
            PathBuf::from(CONTROL_PLANE_STATE),
            resolved_capsule_root(),
            vec![PathBuf::from(CREDENTIALS)],
        )
    }

    #[test]
    fn the_resolved_root_is_the_forbidden_scope_a_placement_is_validated_against() {
        let resolved = resolved_capsule_root();
        assert_eq!(
            resolved,
            PathBuf::from(CAPSULE_ROOT),
            "the reader resolves the configured root; this phase re-does no resolution"
        );

        let scopes = scopes_from_resolved_root();
        let mut parts = lawful_parts();
        parts
            .readable
            .push(mount("/var/lib/doctrine/somewhere", DECLARED_INNER));
        assert_eq!(
            CapsulePlacement::try_new(parts, &scopes)
                .expect_err("an entry under the resolved capsule root must refuse"),
            PlacementRefusal::ForbiddenScopeOverlap {
                path: PathBuf::from("/var/lib/doctrine/somewhere")
            }
        );

        // The pair: the lawful placement is still admitted against exactly the
        // same resolved scope.
        assert!(CapsulePlacement::try_new(lawful_parts(), &scopes).is_ok());
    }

    // ── T5 / VT-2: descendant mutants, and the writable-only carve-out ─────
    //
    // The `F-10` class: each of the first three passes an equal-or-ancestor
    // test and is exactly what its scope exists to deny.

    #[test]
    fn a_file_inside_the_canonical_repository_refuses() {
        const OBJECT_STORE: &str = "/srv/repo/.git/config";

        assert_eq!(
            refuses_at(OBJECT_STORE),
            PlacementRefusal::ForbiddenScopeOverlap {
                path: PathBuf::from(OBJECT_STORE)
            }
        );
        admits_at("/srv/elsewhere/.git/config");
    }

    /// Scoped so that **only** the control-plane region can catch it: the
    /// repository scope is moved aside, or this test would be green on the
    /// previous rule's answer.
    #[test]
    fn a_file_inside_the_control_plane_state_refuses() {
        const CONTROL_PLANE_FILE: &str = "/srv/repo/.doctrine/state/boot.md";
        let scopes = ForbiddenScopes::new(
            PathBuf::from("/srv/other-repo"),
            PathBuf::from(CONTROL_PLANE_STATE),
            PathBuf::from(CAPSULE_ROOT),
            vec![PathBuf::from(CREDENTIALS)],
        );

        let mut parts = lawful_parts();
        parts
            .readable
            .push(mount(CONTROL_PLANE_FILE, DECLARED_INNER));
        assert_eq!(
            CapsulePlacement::try_new(parts, &scopes)
                .expect_err("a file inside the control plane must refuse"),
            PlacementRefusal::ForbiddenScopeOverlap {
                path: PathBuf::from(CONTROL_PLANE_FILE)
            }
        );

        let mut lawful = lawful_parts();
        lawful
            .readable
            .push(mount("/srv/repo/README.md", DECLARED_INNER));
        assert!(
            CapsulePlacement::try_new(lawful, &scopes).is_ok(),
            "a repository file outside the control plane is not caught by this scope"
        );
    }

    #[test]
    fn a_file_inside_a_credential_location_refuses() {
        const KEY: &str = "/home/agent/.ssh/id_ed25519";

        assert_eq!(
            refuses_at(KEY),
            PlacementRefusal::ForbiddenScopeOverlap {
                path: PathBuf::from(KEY)
            }
        );
        admits_at("/home/agent/.cache/agent");
    }

    #[test]
    fn a_sibling_transaction_root_under_the_capsule_root_refuses() {
        let sibling = format!("{SIBLING_TRANSACTION_ROOT}/capsule");
        assert_eq!(
            refusal(with_writable(&sibling, DECLARED_INNER)),
            PlacementRefusal::ForbiddenScopeOverlap {
                path: PathBuf::from(&sibling)
            }
        );

        // The pair, and the whole point of the carve-out: the *same* shape
        // under this placement's own transaction root is admitted.
        assert!(
            with_writable(&format!("{TRANSACTION_ROOT}/capsule"), DECLARED_INNER).is_ok(),
            "this placement's own transaction root is its writable state"
        );
    }

    /// The same rule, with the capsule root driven out of the reader rather
    /// than restated — `sec-5`'s alignment. Not a duplicate of the test above:
    /// that one builds [`ForbiddenScopes`] directly.
    #[test]
    fn a_sibling_transaction_under_the_resolved_root_is_refused_by_the_placement() {
        let scopes = scopes_from_resolved_root();
        let sibling = format!("{SIBLING_TRANSACTION_ROOT}/capsule");

        let mut parts = lawful_parts();
        parts.writable.push(mount(&sibling, DECLARED_INNER));
        assert_eq!(
            CapsulePlacement::try_new(parts, &scopes)
                .expect_err("a sibling transaction must refuse"),
            PlacementRefusal::ForbiddenScopeOverlap {
                path: PathBuf::from(&sibling)
            }
        );

        let mut own = lawful_parts();
        own.writable
            .push(mount(&format!("{TRANSACTION_ROOT}/agent"), DECLARED_INNER));
        assert!(CapsulePlacement::try_new(own, &scopes).is_ok());
    }

    /// Asserted **positively**, so a fix that refuses everything under the
    /// capsule root cannot pass (`F-25`).
    #[test]
    fn a_writable_entry_under_this_placements_own_transaction_root_is_admitted() {
        let own = format!("{TRANSACTION_ROOT}/capsule");
        let placement =
            with_writable(&own, DECLARED_INNER).expect("this placement's own state is writable");

        assert_eq!(
            placement.writable().first().map(MountedPath::host),
            Some(Path::new(&own))
        );
        assert_eq!(placement.root().path(), Path::new(TRANSACTION_ROOT));
    }

    /// The carve-out is **writable-only**. A validator that admits both this
    /// and the test above is wrong.
    #[test]
    fn a_readable_entry_under_this_placements_own_transaction_root_refuses() {
        let own = format!("{TRANSACTION_ROOT}/capsule");
        assert_eq!(
            refusal(with_readable(&own, DECLARED_INNER)),
            PlacementRefusal::ForbiddenScopeOverlap {
                path: PathBuf::from(&own)
            }
        );
        assert!(with_writable(&own, DECLARED_INNER).is_ok());
    }

    // ── T6 / VT-3: the source-export carve-out ─────────────────────────────

    /// **The positive control whose absence let `F-25` stand.** The lawful
    /// export lives under the capsule root, which is a forbidden scope, so a
    /// validator that refuses everything beneath the capsule root — which this
    /// design specified until round 4 — reds here and nowhere else.
    #[test]
    fn the_lawful_source_export_for_this_base_is_admitted() {
        let placement =
            validate(lawful_parts()).expect("the design's only lawful source placement");

        assert_eq!(placement.source().host(), Path::new(LAWFUL_EXPORT));
        assert!(
            Path::new(LAWFUL_EXPORT).starts_with(CAPSULE_ROOT),
            "the lawful export is beneath a forbidden scope — that is the point"
        );
    }

    #[test]
    fn a_source_export_of_a_different_base_refuses() {
        let mut parts = lawful_parts();
        parts.source = SourceExport::new(
            PathBuf::from(SIBLING_EXPORT),
            AcceptedBase::new(OTHER_BASE_OID.to_owned()),
        );
        assert_eq!(
            refusal(validate(parts)),
            PlacementRefusal::SourceBaseMismatch {
                path: PathBuf::from(SIBLING_EXPORT),
                carried: AcceptedBase::new(OTHER_BASE_OID.to_owned()),
            }
        );

        // The pair: the identity is what refuses, not the path. The same
        // sibling export is lawful for a placement that accepts that base.
        let mut accepted = lawful_parts();
        accepted.source = SourceExport::new(
            PathBuf::from(SIBLING_EXPORT),
            AcceptedBase::new(OTHER_BASE_OID.to_owned()),
        );
        accepted.accepted_base = AcceptedBase::new(OTHER_BASE_OID.to_owned());
        assert!(validate(accepted).is_ok());
    }

    /// Including a source beneath this placement's **own transaction root**,
    /// which the *other* carve-out would otherwise seem to admit. The two are
    /// never composed into a third (`EX-7`).
    #[test]
    fn a_source_outside_the_export_directory_refuses() {
        for outside in [
            format!("{TRANSACTION_ROOT}/export/{BASE_OID}"),
            "/srv/elsewhere/export".to_owned(),
            EXPORT_DIRECTORY.to_owned(),
        ] {
            let mut parts = lawful_parts();
            parts.source = SourceExport::new(
                PathBuf::from(&outside),
                AcceptedBase::new(BASE_OID.to_owned()),
            );
            assert_eq!(
                refusal(validate(parts)),
                PlacementRefusal::SourceOutsideExportDirectory {
                    path: PathBuf::from(&outside)
                },
                "for {outside}"
            );
        }

        assert!(
            validate(lawful_parts()).is_ok(),
            "the export directory's own descendant is the lawful case"
        );
    }

    /// The export reaches a capsule as `source` and never as a declared
    /// readable entry — the other face of *the carve-outs are never composed*.
    #[test]
    fn a_readable_entry_naming_the_export_directory_refuses() {
        assert_eq!(
            refuses_at(LAWFUL_EXPORT),
            PlacementRefusal::ForbiddenScopeOverlap {
                path: PathBuf::from(LAWFUL_EXPORT)
            }
        );
        assert_eq!(
            refuses_at(EXPORT_DIRECTORY),
            PlacementRefusal::ForbiddenScopeOverlap {
                path: PathBuf::from(EXPORT_DIRECTORY)
            }
        );

        // The pair: the very same path, arriving as `source`, is admitted.
        assert!(validate(lawful_parts()).is_ok());
    }

    /// `A3`'s control — it fails if `overlaps` is a textual prefix test.
    #[test]
    fn a_sibling_directory_whose_name_extends_a_forbidden_scope_is_admitted() {
        admits_at("/var/lib/doctrine-other/thing");
        assert_eq!(
            refuses_at("/var/lib/doctrine/thing"),
            PlacementRefusal::ForbiddenScopeOverlap {
                path: PathBuf::from("/var/lib/doctrine/thing")
            },
            "the control's own control: the real descendant still refuses"
        );
    }

    // ── T7 / VT-4: inner destinations ──────────────────────────────────────

    /// The lawful request plus several further readable entries.
    fn with_readables(entries: &[(&str, &str)]) -> Result<CapsulePlacement, PlacementRefusal> {
        let mut parts = lawful_parts();
        for (host, at) in entries {
            parts.readable.push(mount(host, at));
        }
        validate(parts)
    }

    /// One case per reserved path, driven from the constants rather than
    /// spelled again (`D9`).
    #[test]
    fn reserved_inner_destination_supplied_as_a_readable_entry_refuses() {
        for reserved in super::RESERVED_INNER_DESTINATIONS {
            assert_eq!(
                refusal(with_readable("/nix/store/thing", reserved)),
                PlacementRefusal::ReservedInnerDestination {
                    inner: PathBuf::from(reserved)
                },
                "for {reserved}"
            );
        }

        // The pair: an ordinary destination of the same shape is admitted.
        assert!(with_readable("/nix/store/thing", "/opt").is_ok());
    }

    #[test]
    fn two_entries_with_the_same_inner_path_refuse() {
        assert_eq!(
            refusal(with_readables(&[
                ("/nix/store/one", DECLARED_INNER),
                ("/nix/store/two", DECLARED_INNER),
            ])),
            PlacementRefusal::InnerPathCollision {
                inner: [PathBuf::from(DECLARED_INNER), PathBuf::from(DECLARED_INNER)]
            }
        );

        assert!(
            with_readables(&[("/nix/store/one", "/work-a"), ("/nix/store/two", "/work-b")]).is_ok(),
            "distinct inner destinations do not collide"
        );
    }

    /// Same rule, one variant (`D3`): collision is `overlaps` over inner paths,
    /// so mount order can never decide what is visible.
    #[test]
    fn an_inner_path_that_is_an_ancestor_of_another_refuses() {
        let collision = refusal(with_readables(&[
            ("/nix/store/one", DECLARED_INNER),
            ("/nix/store/two", "/work/sub"),
        ]));
        assert_eq!(
            collision,
            PlacementRefusal::InnerPathCollision {
                inner: [PathBuf::from(DECLARED_INNER), PathBuf::from("/work/sub")]
            }
        );
        assert_eq!(
            collision.paths(),
            [PathBuf::from(DECLARED_INNER), PathBuf::from("/work/sub")],
            "both sides of the collision, structured rather than formatted"
        );

        // Order-independent: the same pair declared the other way round.
        assert!(
            with_readables(&[
                ("/nix/store/two", "/work/sub"),
                ("/nix/store/one", DECLARED_INNER),
            ])
            .is_err()
        );
        assert!(with_readables(&[("/nix/store/one", "/work-other/sub")]).is_ok());
    }

    /// What is validated is what is bound (`EX-8`, invariant 3). The declared
    /// path here resolves *out of* a forbidden scope, so the two answers
    /// differ: binding the declared path would refuse.
    #[test]
    fn the_bound_host_path_is_the_resolved_path_not_the_declared_one() {
        const DECLARED: &str = "/srv/repo/toolchain";
        const RESOLVED: &str = "/nix/store/toolchain";

        let host = FixtureHost::new().with_resolution(DECLARED, RESOLVED);
        let resolved = host
            .resolve(Path::new(DECLARED))
            .expect("the fixture resolves this path");
        assert_eq!(resolved, PathBuf::from(RESOLVED));

        let mut parts = lawful_parts();
        parts
            .readable
            .push(MountedPath::new(resolved, inner("/tools")));
        let placement = validate(parts).expect("the resolved path names no forbidden scope");
        assert_eq!(
            placement.readable().last().map(MountedPath::host),
            Some(Path::new(RESOLVED))
        );

        // The discriminating half: the declared path is inside the canonical
        // repository and refuses. Resolution precedes validation for a reason.
        assert_eq!(
            refusal(with_readable(DECLARED, "/tools")),
            PlacementRefusal::ForbiddenScopeOverlap {
                path: PathBuf::from(DECLARED)
            }
        );
    }

    /// Reserved means **naming** the destination, never descending from it —
    /// the lawful case the reserved rule must not capture.
    #[test]
    fn a_readable_entry_beneath_a_profile_owned_mount_is_admitted() {
        let placement = with_readable("/var/cache/agent", "/tmp/cache")
            .expect("an entry beneath a profile-owned mount is the ordinary case");
        assert_eq!(
            placement.readable().last().map(MountedPath::inner),
            Some(&inner("/tmp/cache"))
        );

        assert!(
            with_readable("/var/cache/agent", super::INNER_TMP).is_err(),
            "naming the reserved destination itself still refuses"
        );
    }

    // ── T8 / VT-5: the two closed vocabularies ─────────────────────────────

    const A_TIMEOUT: Duration = Duration::from_secs(900);

    fn execution() -> Execution {
        Execution::new(
            Argv::try_new(vec![
                "/bin/sh".to_owned(),
                "-c".to_owned(),
                "true".to_owned(),
            ])
            .expect("a non-empty argument vector"),
            CapsuleEnv::complete(),
            A_TIMEOUT,
            ByteCount::from_mib(512).expect("a representable cap"),
            CapsuleStdio::EmptyInputCapturedOutput,
        )
    }

    #[test]
    fn capsule_env_is_a_closed_vocabulary_with_trusted_side_values() {
        for var in CapsuleEnvVar::ALL.iter().copied() {
            // Exhaustive by construction: widening the vocabulary is a compile
            // error *in this test*, not a silently uncovered variant.
            let expected = match var {
                CapsuleEnvVar::Path => None,
                CapsuleEnvVar::Home => Some(super::INNER_AGENT),
                CapsuleEnvVar::Term => Some(super::TERM_VALUE),
                CapsuleEnvVar::GitAuthorName | CapsuleEnvVar::GitCommitterName => {
                    Some(super::CAPSULE_GIT_IDENTITY_NAME)
                }
                CapsuleEnvVar::GitAuthorEmail | CapsuleEnvVar::GitCommitterEmail => {
                    Some(super::CAPSULE_GIT_IDENTITY_EMAIL)
                }
            };
            assert_eq!(var.fixed_value(), expected, "for {var:?}");
        }

        assert_eq!(CapsuleEnvVar::Term.fixed_value(), Some("dumb"));
        assert_eq!(
            CapsuleEnvVar::Home.fixed_value(),
            Some(super::INNER_AGENT),
            "the agent home is the reserved inner destination, named once"
        );
        assert_eq!(
            CapsuleEnvVar::Path.fixed_value(),
            None,
            "the inner PATH is derived from the bound paths by the profile"
        );
        assert_eq!(
            CapsuleEnv::complete().vars().count(),
            CapsuleEnvVar::ALL.len()
        );
    }

    /// **A compile-time witness, not a runtime proof** (`D8`). A `#[test]`
    /// cannot demonstrate a negative existential over a type's public surface,
    /// so what carries the claim is the exhaustive `match` above — which stops
    /// compiling the moment the vocabulary widens — together with the
    /// constructor-shape assertion below, which stops holding the moment a text
    /// route is added.
    ///
    /// **What this does not cover:** inheritance. `--clearenv`-style flags stop
    /// *inheritance* only, and this test says nothing about what a backend
    /// passes through from the trusted-side process. That is invariant 13, and
    /// its home is `sec-7` row 11, executed.
    #[test]
    fn no_public_route_carries_caller_supplied_environment_text() {
        let requested: BTreeSet<CapsuleEnvVar> = [CapsuleEnvVar::Term, CapsuleEnvVar::Home]
            .into_iter()
            .collect();
        let env = CapsuleEnv::new(requested.clone());

        // The only route in carries variants; what comes back out is exactly
        // the variants that went in, with no text anywhere on the path.
        assert_eq!(env.vars().collect::<BTreeSet<_>>(), requested);
        assert_eq!(execution().env(), &CapsuleEnv::complete());
    }

    #[test]
    fn capsule_stdio_is_a_closed_vocabulary_with_parent_owned_endpoints() {
        // Exhaustive: an `Inherit` variant or a raw-descriptor payload is a
        // compile error here.
        let described = match CapsuleStdio::EmptyInputCapturedOutput {
            CapsuleStdio::EmptyInputCapturedOutput => "empty input, captured output",
        };
        assert_eq!(described, "empty input, captured output");
        assert_eq!(execution().stdio(), CapsuleStdio::EmptyInputCapturedOutput);
    }

    /// The counterpart witness to the environment's text route (`D8`).
    ///
    /// **What this does not cover:** descriptors already open across `exec`.
    /// A closed enum makes an *inherited* descriptor unrepresentable in the
    /// contract; it observes nothing about a running capsule. That is
    /// invariant 12, and its home is `sec-7` row 10, executed.
    #[test]
    fn no_public_route_supplies_a_raw_descriptor_to_execution() {
        let execution = execution();

        // The only constructor takes the closed vocabulary and nothing else;
        // every field round-trips as given.
        assert_eq!(execution.stdio(), CapsuleStdio::EmptyInputCapturedOutput);
        assert_eq!(execution.timeout(), A_TIMEOUT);
        assert_eq!(
            execution.file_size_cap(),
            ByteCount::from_mib(512).expect("a representable cap")
        );
        assert_eq!(
            execution.argv().as_slice().first().map(String::as_str),
            Some("/bin/sh")
        );
    }

    // ── T9 / VT-6: the default is refusal ──────────────────────────────────

    #[test]
    fn empty_mount_vector_refuses_rather_than_running_unconfined() {
        let mut parts = lawful_parts();
        parts.readable.clear();
        assert_eq!(
            refusal(validate(parts)),
            PlacementRefusal::NoReadableEntries
        );
        assert!(
            refusal(validate({
                let mut parts = lawful_parts();
                parts.readable.clear();
                parts
            }))
            .paths()
            .is_empty()
        );

        // The pair: the minimal lawful placement — one readable entry, and an
        // empty `writable` vector, which is lawful because `root` is a typed
        // field and supplies the capsule's writable state.
        let mut minimal = lawful_parts();
        minimal.network = NetworkPosture::Permitted;
        let placement = validate(minimal).expect("one readable entry is enough to run");
        assert!(placement.writable().is_empty());
        assert_eq!(placement.readable().len(), 1);
        assert_eq!(placement.network(), NetworkPosture::Permitted);
        assert_eq!(
            placement.working_directory(),
            &inner(super::INNER_CAPSULE),
            "the working directory is stated, never inherited"
        );
    }

    /// Invariant 9, at the other entry point. The refusal lives in `Argv`
    /// itself, and `Execution` is constructible only from an `Argv`, so an
    /// empty argument vector has no route to a run.
    #[test]
    fn empty_argv_refuses() {
        assert!(Argv::try_new(Vec::new()).is_none());
        assert!(Argv::try_new(vec![String::new()]).is_some());
        assert_eq!(execution().argv().as_slice().len(), 3);
    }

    // ── T10: the contract itself ───────────────────────────────────────────

    /// `EX-14`: a capsule's own nonzero exit is **data**, carried in `Ok`.
    /// Collapsing it into `BackendError` would make a working capsule reporting
    /// failure indistinguishable from a broken confinement mechanism.
    #[test]
    fn a_capsule_exit_is_observation_data_and_a_backend_failure_is_an_error() {
        let placement = validate(lawful_parts()).expect("the lawful placement");
        let execution = execution();

        let exited = WitnessBackend::always(Ok(Observation {
            termination: Termination::Exited { code: 1 },
            stdout: Vec::new(),
            stderr: b"boom".to_vec(),
            disk_used: ByteCount::from_bytes(4096),
        }));
        // Held behind a reference: `provision` will hold one of these, so the
        // trait must stay dyn-compatible.
        let backend: &dyn CapsuleBackend = &exited;
        assert_eq!(backend.id().as_str(), WITNESS_ID.as_str());
        assert_eq!(backend.availability(), Availability::Available);
        let observation = backend
            .execute(&placement, &execution)
            .expect("a nonzero capsule exit is not a backend failure");
        assert_eq!(observation.termination, Termination::Exited { code: 1 });
        assert_eq!(observation.stderr, b"boom");
        assert_eq!(observation.disk_used, ByteCount::from_bytes(4096));

        let broken = WitnessBackend::always(Err(BackendError::MechanismFailed {
            detail: "could not start".to_owned(),
        }))
        .reporting(Availability::Unavailable {
            missing: "bwrap".to_owned(),
            remedy: "install bubblewrap".to_owned(),
        });
        let backend: &dyn CapsuleBackend = &broken;
        assert_eq!(
            backend.availability(),
            Availability::Unavailable {
                missing: "bwrap".to_owned(),
                remedy: "install bubblewrap".to_owned(),
            },
            "an unavailable backend says what is missing and what would satisfy it"
        );
        assert_eq!(
            backend.execute(&placement, &execution),
            Err(BackendError::MechanismFailed {
                detail: "could not start".to_owned()
            })
        );
        assert_eq!(
            BackendError::Unavailable { id: WITNESS_ID },
            BackendError::Unavailable {
                id: BackendId::new("witness")
            }
        );

        // Every termination state is distinguishable — *the runner refused* and
        // *the runner never ran* must not read identically.
        let states = [
            Termination::Exited { code: 0 },
            Termination::Signalled { signal: 9 },
            Termination::TimedOut,
            Termination::FileSizeExceeded,
            Termination::NotExecutable,
        ];
        for (i, left) in states.iter().enumerate() {
            for (j, right) in states.iter().enumerate() {
                assert_eq!(i == j, left == right);
            }
        }
    }
}
