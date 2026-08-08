// SPDX-License-Identifier: GPL-3.0-only
//! `provision` — the thirteen steps that turn a request into a capsule
//! transaction (SL-248 `sec-3`, `EX-4`…`EX-16`).
//!
//! **The two scratch areas are named, throughout, by what they are**
//! (`EX-5`) — transient and retained, never *isolated* and *shared*, because
//! bare *shared* collides head-on with `REQ-450` criterion 1's vocabulary,
//! whose whole claim is that two transactions share no such state:
//!
//! - **transient** — `/tmp` inside the capsule. The profile's own anonymous
//!   `--tmpfs`, one `execute` long, not counted in `Observation::disk_used`, and
//!   no placement-level delta reaches it. It is not a declared writable entry.
//! - **retained** — `/capsule/tmp`, on host disk beneath the transaction root.
//!   It lives for the whole transaction, is counted in `disk_used`, and is the
//!   **only** declared writable entry in the placement.
//!
//! **Purity (`EX-6`, `VA-3`).** Steps 1, 4, 5 and 6 are pure given
//! [`HostFacts`] and are free functions taking owned inputs: [`capsule_config`],
//! [`resolved_policy`] and [`admit_resolver`]. Steps 2, 3 and 7–12 are impure
//! and live in the thin outer part below. Nothing in the pure four reads a
//! clock, an entropy source, `git` or the disk.
//!
//! Layering (`ADR-001`): `provision` is `engine`, out-edges `{transaction,
//! backend, capacity, config, host}`. `backend::bubblewrap` is a submodule of
//! `backend` and the layering gate maps it to that unit, so the closure seam
//! adds no sixth edge.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use doctrine::interpretation::{
    self, InterpretationPolicy, PolicyRefusal, RestrictionRefusal,
};
use doctrine::{DOCTRINE_TOML, read_path_at};

use crate::backend::bubblewrap::{
    ClosureQuery, ProfileRefusal, SpawnedClosureQuery, profile_owned_host_path, readable_set,
};
use crate::backend::{
    AcceptedBase, BackendError, CAPSULE_GIT_IDENTITY_EMAIL, CAPSULE_GIT_IDENTITY_NAME,
    CapsuleBackend, CapsuleEnv, CapsulePlacement, CapsuleStdio, EXPORT_DIRECTORY_LEAF, Execution,
    ForbiddenScopes, INNER_AGENT, INNER_CAPSULE, INNER_SOURCE, InnerPath, MountedPath,
    NetworkPosture, PlacementParts, PlacementRefusal, SourceExport, Termination, TransactionRoot,
};
use crate::capacity::{CapacityReport, assess_capacity};
use crate::config::{
    Argv, ByteCount, CapsuleConfig, ConfigRefusal, ResourceBounds, parse_capsule_config,
    root_capsule_config,
};
use crate::host::{CapacityUnknown, HostFacts};
use crate::transaction::{CapsuleTransaction, PhaseIdentity, TransactionId};

// ---------------------------------------------------------------------------
// Named constants (`STD-001`) — the on-disk layout, spelled once
// ---------------------------------------------------------------------------

/// The subdirectory of the capsule root that per-transaction roots live under:
/// `<capsule_root>/tx/<id>/` (`EX-4`).
const TRANSACTION_DIRECTORY_LEAF: &str = "tx";
/// The prefix of a temporary export build directory, `export/.building-<id>/`
/// (`EX-11`). Dot-led so it never collides with an oid-named published export.
const EXPORT_BUILD_PREFIX: &str = ".building-";
/// The single ref a published export carries. Validation asserts there is no
/// other, and that this one names the contracted base.
const EXPORT_BASE_REF: &str = "refs/heads/base";
/// The alternates file whose **absence** an export is validated for: its
/// presence would bind host object state the export exists to replace
/// (`EX-10`).
const EXPORT_ALTERNATES_PATH: &str = "objects/info/alternates";
/// The clone's working tree, beneath the retained capsule state.
const CAPSULE_REPOSITORY_LEAF: &str = "repo";
/// Where a capsule writes its results.
const CAPSULE_OUTPUT_LEAF: &str = "out";
/// The **retained** scratch area (`EX-5`) — host disk, whole transaction,
/// counted in `disk_used`, and the only declared writable entry. Never the
/// transient `/tmp`, which is the profile's own mount.
const CAPSULE_RETAINED_TMP_LEAF: &str = "tmp";

/// The trusted-side Git the export build drives.
const GIT_EXECUTABLE: &str = "git";
/// `git config` keys the capsule identity is read back from (step 12).
const GIT_IDENTITY_NAME_KEY: &str = "user.name";
const GIT_IDENTITY_EMAIL_KEY: &str = "user.email";
/// The regular expression step 12's fourth execution matches. Anchored, so a
/// key like `user.nameother` cannot answer for `user.name`.
const GIT_IDENTITY_PATTERN: &str = r"^user\.(name|email)$";

/// What `git rev-parse --is-bare-repository` prints for a bare repository.
const GIT_TRUE: &str = "true";

// ---------------------------------------------------------------------------
// T2: the request and the refusal vocabulary
// ---------------------------------------------------------------------------

/// Everything [`provision`] is asked for.
///
/// [`ProvisionRequest::id`] is a **request field allocated by the caller**
/// (`EX-7`), not something `provision` mints. That is the project's
/// pure/imperative rule — the impure value is read in the outer shell and passed
/// in, exactly as the date and uid inputs already are — and it is also what
/// makes the collision tests writable at all: with no allocator named in the
/// signature the collision branch is unreachable through the stated API
/// (`RV-346` `F-9`/`F-14`). A test hands the same id twice; no entropy is
/// involved.
///
/// It lives here rather than in `transaction` (`D1`): it is `provision`'s input,
/// not part of what a transaction binds.
#[derive(Debug, Clone)]
pub(crate) struct ProvisionRequest {
    /// The repository the base is resolved from and the export is fetched from.
    pub(crate) repository_root: PathBuf,
    pub(crate) base: AcceptedBase,
    pub(crate) phase: PhaseIdentity,
    /// Allocated by the caller (`EX-7`).
    pub(crate) id: TransactionId,
    /// An optional phase-refinement document, by path (`sec-4`).
    pub(crate) refinement: Option<PathBuf>,
    pub(crate) network: NetworkPosture,
}

/// Which of the three-execution clone (plus step 12's read-back) a refusal is
/// about.
///
/// Named steps rather than an index, because "refuses **at that step**" is the
/// whole point of issuing three executions instead of one aggregate status
/// (`EX-14`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CloneStep {
    /// `git clone --no-hardlinks --quiet -c user.name=… -c user.email=… -- /source /capsule/repo`
    Clone,
    /// `git -C /capsule/repo switch --detach --quiet <base>`
    Detach,
    /// `git -C /capsule/repo remote remove origin`
    RemoveOrigin,
    /// Step 12's fourth execution, reading the pinned identity back.
    ReadIdentity,
}

/// Why a candidate export was refused rather than adopted (`EX-10`).
///
/// One variant per condition, so the mutation battery can move one rule and see
/// exactly one refusal move with it. Nothing here repairs: a partially-built or
/// widened export binds more host object state than `DEC-157` permits, and
/// repairing one in place is how a shared artefact silently acquires a second
/// writer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExportFault {
    /// Not a real directory — most sharply, a **symlink**, which is `sec-2`
    /// `F-1`'s escape arriving by another route
    /// (`mem.fact.capsule.bwrap-ro-bind-dereferences-source` is why it matters:
    /// a read-only bind dereferences its source).
    NotARealDirectory,
    /// Not a bare repository.
    NotBare,
    /// An `objects/info/alternates` file is present.
    AlternatesPresent,
    /// The base is not held as a complete object closure — a partially fetched
    /// export.
    BaseNotAWholeClosure,
    /// A ref other than the single one naming the base.
    RefsOtherThanBase { found: usize },
    /// The single ref does not name the contracted base: an export of some
    /// *other* base, sitting where this base's export belongs.
    RefDoesNotNameBase { found: String },
    /// Git itself could not be run, or answered unusably.
    Unreadable { detail: String },
}

/// Why provisioning refused.
///
/// A plain data enum that **wraps** the refusals of the units it composes rather
/// than re-classifying them (`A5`). Wrapping is what keeps a variant those units
/// can no longer produce costless here, and what keeps the recommendation to
/// delete one live at reconciliation instead of forcing this module to carry a
/// match arm for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProvisionRefusal {
    /// Step 1 — the `[capsule]` table.
    Config(ConfigRefusal),
    /// Step 1 — `.doctrine/doctrine.toml` could not be read from the working
    /// tree at all.
    ConfigUnreadable { path: PathBuf, detail: String },
    /// Step 3 — capacity below one expected capsule size.
    Capacity(CapacityReport),
    /// Step 4 — the blob at the base could not be read.
    BaseDocumentUnreadable { detail: String },
    /// Step 4 — the contracted base carries no `.doctrine/doctrine.toml`.
    BaseDocumentAbsent { path: String },
    /// Step 4 — the base document's `[interpretation]` block.
    BasePolicy(PolicyRefusal),
    /// Step 5 — the refinement document could not be read.
    RefinementUnreadable { path: PathBuf, detail: String },
    /// Step 5 — the refinement document's `[interpretation]` block.
    RefinementPolicy(PolicyRefusal),
    /// Step 5 — the refinement widens the base policy on some axis.
    Restriction(RestrictionRefusal),
    /// Step 6 — the closure resolver's basename is on the policy's
    /// `trusted_side_forbidden_executables`.
    ForbiddenResolver { executable: String },
    /// Steps 2 and 7 — the declared readable inputs.
    Profile(ProfileRefusal),
    /// Step 8 — a candidate export that is not adoptable.
    Export { path: PathBuf, fault: ExportFault },
    /// Step 8 — building the temporary export failed.
    ExportBuildFailed { path: PathBuf, detail: String },
    /// Steps 8 and 9 — a directory this call had to create exclusively already
    /// exists, or could not be created. **The exclusive create is what
    /// establishes ownership**, so this is a refusal and never a retry.
    DirectoryNotExclusivelyCreated { path: PathBuf, detail: String },
    /// Step 10 — the placement.
    Placement(PlacementRefusal),
    /// Step 10 — a path used as an inner destination is not absolute.
    /// Fail-closed rather than dropped: a silently omitted readable entry is a
    /// capsule missing an input it was told it had.
    InnerDestinationNotAbsolute { path: PathBuf },
    /// Steps 11 and 12 — the confinement mechanism itself failed. A capsule's
    /// own nonzero exit is not this; it is [`ProvisionRefusal::CloneFailed`].
    Backend(BackendError),
    /// Steps 11 and 12 — an execution did not exit zero, named by **which**
    /// one.
    CloneFailed {
        step: CloneStep,
        termination: Termination,
    },
    /// Step 12 — the clone's config does not carry the pinned capsule identity.
    /// A git that stopped persisting `-c` would otherwise restore the resolver
    /// stall silently.
    IdentityNotPersisted { read_back: String },
    /// Unreachable by construction: every argv this module builds is a literal
    /// of at least three words, and [`Argv::try_new`] refuses only an empty
    /// one. Fail-closed rather than a panic, which is denied anyway.
    EmptyExecutionArgv { step: CloneStep },
}

impl ProvisionRefusal {
    /// The paths this refusal is about, structured rather than formatted — the
    /// same posture every refusal in this crate takes, and the reason the
    /// `provision` verb can name what to fix without parsing a sentence.
    ///
    /// Owned, where the wrapped refusals return `&[PathBuf]`: this enum
    /// aggregates over sources that do not all store a `PathBuf`
    /// ([`CapacityReport::capsule_root`] answers `&Path`), and one allocation on
    /// a refusal path is cheaper than an accessor minted in another unit purely
    /// to satisfy a return type.
    pub(crate) fn paths(&self) -> Vec<PathBuf> {
        match self {
            Self::Config(_)
            | Self::BaseDocumentUnreadable { .. }
            | Self::BaseDocumentAbsent { .. }
            | Self::BasePolicy(_)
            | Self::RefinementPolicy(_)
            | Self::Restriction(_)
            | Self::ForbiddenResolver { .. }
            | Self::Backend(_)
            | Self::CloneFailed { .. }
            | Self::IdentityNotPersisted { .. }
            | Self::EmptyExecutionArgv { .. } => Vec::new(),
            Self::ConfigUnreadable { path, .. }
            | Self::RefinementUnreadable { path, .. }
            | Self::Export { path, .. }
            | Self::ExportBuildFailed { path, .. }
            | Self::InnerDestinationNotAbsolute { path }
            | Self::DirectoryNotExclusivelyCreated { path, .. } => vec![path.clone()],
            Self::Capacity(report) => vec![report.capsule_root().to_path_buf()],
            Self::Profile(refusal) => refusal.paths().to_vec(),
            Self::Placement(refusal) => refusal.paths().to_vec(),
        }
    }

    /// The `[capsule]` keys an operator must edit, delegated to the wrapped
    /// refusal wherever one owns the answer.
    pub(crate) fn keys(&self) -> &[&'static str] {
        match self {
            Self::Config(refusal) => refusal.keys(),
            Self::Profile(refusal) => refusal.keys(),
            Self::Capacity(
                CapacityReport::Warn { key, .. }
                | CapacityReport::Refuse { key, .. }
                | CapacityReport::Report { key, .. },
            ) => std::slice::from_ref(key),
            _ => &[],
        }
    }
}

// ---------------------------------------------------------------------------
// T3: the local `git` wrapper
// ---------------------------------------------------------------------------

/// What one trusted-side `git` invocation produced.
#[derive(Debug, Clone, PartialEq, Eq)]
struct GitRun {
    /// `None` when the process was terminated by a signal.
    status: Option<i32>,
    stdout: String,
}

impl GitRun {
    fn succeeded(&self) -> bool {
        self.status == Some(0)
    }
}

/// Run `git` trusted-side, with **an explicit per-command refspec at every call
/// site** and never a `git remote add` or a `git config` write.
///
/// **The road not taken.** The root package already has
/// `git::fetch_refspec` (`src/git.rs:2718`) — eleven lines: one `git fetch`
/// with an explicit refspec plus error formatting. Reusing it would mean
/// widening a **published** crate's permanent public API, paying `sec-9` `R7`'s
/// minimal-surface cost at PHASE-01, five phases before the need appears, on a
/// function that is `pub(crate)` and in the other crate. The slice owner ruled
/// for the local wrapper (`EN-3`); this comment is that alternative recorded at
/// the seam rather than only in the design. What the wrapper *does* inherit is
/// that function's one real discipline: the refspec is a per-command argument,
/// so nothing here mutates `.git/config`.
///
/// **This trusted-side `git` is governed, not a new hazard.** `SPEC-030`
/// (`spec-030.md:80`) places Doctrine-owned Git operations under the ingestion
/// contract rather than under the `trusted_side_forbidden_executables` check
/// that step 6 applies to the *project-supplied* closure resolver. The two are
/// different populations and are governed by different rules on purpose.
fn run_git(args: &[&str]) -> Result<GitRun, String> {
    let output = Command::new(GIT_EXECUTABLE)
        .args(args)
        .output()
        .map_err(|error| error.to_string())?;
    Ok(GitRun {
        status: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
    })
}

// ---------------------------------------------------------------------------
// Exclusive creation and the token it mints
// ---------------------------------------------------------------------------

/// Proof that **this call** created the directory it names (`EX-13`).
///
/// Minted only by [`create_exclusively`], never from a [`TransactionId`]:
/// exclusivity is a fact the filesystem establishes, and the id's entropy only
/// makes the refusing case rare rather than impossible. Rollback removes a path
/// only on one of these, which is what keeps a retry with a colliding id from
/// deleting another transaction's work — the automated deletion of live work
/// `REQ-461` criterion 3 and `DEC-133`/`DEC-137` forbid.
///
/// Private to this module and carrying no public constructor: nothing outside
/// `provision` can mint one, and therefore nothing outside `provision` can ask
/// for a removal (`EX-16`, `VA-2`).
#[derive(Debug)]
struct CreationToken {
    path: PathBuf,
}

/// `mkdir` semantics that **fail when the path exists** — never a recursive
/// create-if-missing (`EX-11`, `EX-13`).
///
/// [`std::fs::create_dir`] is exactly `mkdir(2)`: it refuses `EEXIST` and it
/// creates no parents. The parent containers (`export/`, `tx/`) are shared and
/// created separately; the *keyed leaf* is what this establishes ownership of.
fn create_exclusively(path: &Path) -> Result<CreationToken, ProvisionRefusal> {
    std::fs::create_dir(path).map_err(|error| ProvisionRefusal::DirectoryNotExclusivelyCreated {
        path: path.to_path_buf(),
        detail: error.to_string(),
    })?;
    Ok(CreationToken {
        path: path.to_path_buf(),
    })
}

/// Remove the directory this call created, **and only that path** (`EX-16`).
///
/// Takes the token by value, so a caller cannot remove the same path twice, and
/// there is no route to this function without one. A failure to remove is not
/// reported: the refusal that triggered the rollback is the one worth reporting,
/// and a leftover directory is a smaller harm than a masked cause.
///
/// **Not a product-side delete capability.** `DEC-156` names the hazard
/// precisely — a delete primitive introduced here for tidiness is what a later
/// slice reaches for — so this is private, token-guarded and has no
/// `pub(crate)` caller.
fn roll_back(token: CreationToken) {
    let _removed = std::fs::remove_dir_all(&token.path);
}

/// Create a shared container directory (`export/`, `tx/`, the capsule root
/// itself). These are **not** owned by any one transaction, so they are
/// create-if-missing and mint no token.
fn ensure_container(path: &Path) -> Result<(), ProvisionRefusal> {
    std::fs::create_dir_all(path).map_err(|error| {
        ProvisionRefusal::DirectoryNotExclusivelyCreated {
            path: path.to_path_buf(),
            detail: error.to_string(),
        }
    })
}

/// Existence that does **not** follow a symlink.
///
/// [`Path::exists`] dereferences, so a symlink to a valid export would read as
/// absent once its target went away, and — worse here — a dangling symlink at
/// `export/<oid>` would read as absent and be published *over*. `symlink_metadata`
/// asks about the entry itself.
fn entry_present(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok()
}

// ---------------------------------------------------------------------------
// T4: export validation, publish-or-adopt
// ---------------------------------------------------------------------------

/// The **one** validation rule, run against both what is adopted and what is
/// published (`EX-12`).
///
/// Returns adopt-or-refuse and **never repairs**. The five conditions, in
/// order:
///
/// 1. a real directory — never a symlink;
/// 2. bare;
/// 3. no `objects/info/alternates`;
/// 4. the base held as a complete object closure;
/// 5. no ref other than the one naming the base — which is also the pairing
///    check: the single ref must name *this* base, so an export of some other
///    base sitting at this base's path refuses before the clone rather than
///    producing a capsule whose contracted base is a fiction (invariant 3).
///
/// Conditions 4 and 5 are separate tests for a reason: 4 catches a partially
/// fetched export of the right base, 5 catches a complete export of the wrong
/// one, and a single combined check would make neither failure diagnosable.
fn validate_export(path: &Path, base: &AcceptedBase) -> Result<(), ProvisionRefusal> {
    let fault = export_fault(path, base);
    match fault {
        Ok(()) => Ok(()),
        Err(fault) => Err(ProvisionRefusal::Export {
            path: path.to_path_buf(),
            fault,
        }),
    }
}

fn export_fault(path: &Path, base: &AcceptedBase) -> Result<(), ExportFault> {
    // 1. A real directory, never a symlink.
    let metadata = std::fs::symlink_metadata(path).map_err(|error| ExportFault::Unreadable {
        detail: error.to_string(),
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ExportFault::NotARealDirectory);
    }

    let repository = path.to_string_lossy().into_owned();

    // 2. Bare.
    let bare = git_in(&repository, &["rev-parse", "--is-bare-repository"])?;
    if !bare.succeeded() || bare.stdout.trim() != GIT_TRUE {
        return Err(ExportFault::NotBare);
    }

    // 3. No alternates: an alternates file binds host object state the export
    //    exists to replace.
    if entry_present(&path.join(EXPORT_ALTERNATES_PATH)) {
        return Err(ExportFault::AlternatesPresent);
    }

    // 4. The base as a complete object closure. `rev-list --quiet --objects`
    //    walks every reachable object and exits nonzero on the first that is
    //    missing, which a `cat-file -e` on the commit alone would not
    //    (`mem.pattern.tooling.git-cat-file-e-exit-masked-use-ls-tree`).
    let closure = git_in(&repository, &["rev-list", "--quiet", "--objects", base.as_str()])?;
    if !closure.succeeded() {
        return Err(ExportFault::BaseNotAWholeClosure);
    }

    // 5. No ref other than the one naming the base.
    let refs = git_in(&repository, &["for-each-ref", "--format=%(objectname)"])?;
    if !refs.succeeded() {
        return Err(ExportFault::Unreadable {
            detail: "for-each-ref".to_owned(),
        });
    }
    let named: Vec<&str> = refs.stdout.lines().map(str::trim).filter(|line| !line.is_empty()).collect();
    if named.len() != 1 {
        return Err(ExportFault::RefsOtherThanBase { found: named.len() });
    }
    let Some(only) = named.first() else {
        return Err(ExportFault::RefsOtherThanBase { found: 0 });
    };
    if *only != base.as_str() {
        return Err(ExportFault::RefDoesNotNameBase {
            found: (*only).to_owned(),
        });
    }
    Ok(())
}

/// `git -C <repository> …`, with the fault mapped into the export vocabulary.
fn git_in(repository: &str, args: &[&str]) -> Result<GitRun, ExportFault> {
    let mut argv = vec!["-C", repository];
    argv.extend_from_slice(args);
    run_git(&argv).map_err(|detail| ExportFault::Unreadable { detail })
}

/// Step 8, in full: adopt a valid published export, or build, validate, publish
/// and — when another builder won the race — adopt the winner.
///
/// Returns the published export's path.
fn publish_or_adopt_export(
    capsule_root: &Path,
    repository_root: &Path,
    base: &AcceptedBase,
    id: &TransactionId,
) -> Result<PathBuf, ProvisionRefusal> {
    let export_directory = capsule_root.join(EXPORT_DIRECTORY_LEAF);
    ensure_container(&export_directory)?;
    let published = export_directory.join(base.as_str());

    // 8.1 — adopt, if a valid export already exists. Anything else refuses
    // rather than being repaired.
    if entry_present(&published) {
        validate_export(&published, base)?;
        return Ok(published);
    }

    build_and_publish_export(&export_directory, &published, repository_root, base, id)
}

/// Steps 8.2 to 8.5 — the half that builds.
///
/// Split from [`publish_or_adopt_export`] so the loser branch is reachable
/// deterministically: a test publishes an export and then calls this directly,
/// which drives the rename against an already-occupied path without threads.
/// `A2` confirms `EEXIST` on both filesystems in play; a flaky concurrency test
/// would be worse than none.
fn build_and_publish_export(
    export_directory: &Path,
    published: &Path,
    repository_root: &Path,
    base: &AcceptedBase,
    id: &TransactionId,
) -> Result<PathBuf, ProvisionRefusal> {
    // 8.2 — build in a temporary directory this call exclusively created. The
    // name is *derived* from the id but ownership does not rest on it: this
    // runs before step 9, so nothing has yet established that this call owns the
    // id, collision-resistant is not collision-free, and a retry with the same
    // id is not even improbable (`RV-346` `F-8`). The exclusive create is what
    // establishes ownership, and it establishes it here.
    let temporary = export_directory.join(format!("{EXPORT_BUILD_PREFIX}{}", id.as_str()));
    let token = create_exclusively(&temporary)?;

    // 8.3 — validated by the same function as 8.1 (`EX-12`).
    let built = build_export(&temporary, repository_root, base)
        .and_then(|()| validate_export(&temporary, base));
    if let Err(refusal) = built {
        roll_back(token);
        return Err(refusal);
    }

    // 8.4 — publish by no-replace rename, so the loser of a concurrent race
    // gets `EEXIST` rather than replacing a live export other capsules already
    // have bound.
    match rustix::fs::renameat_with(
        rustix::fs::CWD,
        &temporary,
        rustix::fs::CWD,
        published,
        rustix::fs::RenameFlags::NOREPLACE,
    ) {
        Ok(()) => Ok(published.to_path_buf()),
        Err(errno) if errno == rustix::io::Errno::EXIST => {
            // 8.5 — the loser adopts the winner by re-running 8.1's check
            // against the published path, and removes **only its own**
            // temporary directory: never the published export, never another
            // builder's temporary.
            let adopted = validate_export(published, base);
            roll_back(token);
            adopted.map(|()| published.to_path_buf())
        }
        Err(errno) => {
            roll_back(token);
            Err(ProvisionRefusal::ExportBuildFailed {
                path: published.to_path_buf(),
                detail: errno.to_string(),
            })
        }
    }
}

/// `git init --bare` then a fetch of the base with an **explicit per-command
/// refspec** — never a `git remote add`, never a `git config` write.
fn build_export(
    temporary: &Path,
    repository_root: &Path,
    base: &AcceptedBase,
) -> Result<(), ProvisionRefusal> {
    let failed = |detail: String| ProvisionRefusal::ExportBuildFailed {
        path: temporary.to_path_buf(),
        detail,
    };
    let target = temporary.to_string_lossy().into_owned();
    let source = repository_root.to_string_lossy().into_owned();
    let refspec = format!("+{}:{EXPORT_BASE_REF}", base.as_str());

    let initialised = run_git(&["init", "--bare", "--quiet", &target]).map_err(failed)?;
    if !initialised.succeeded() {
        return Err(failed("git init --bare".to_owned()));
    }
    let fetched = run_git(&["-C", &target, "fetch", "--no-tags", &source, &refspec]).map_err(failed)?;
    if !fetched.succeeded() {
        return Err(failed(format!("git fetch {refspec}")));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// The pure four: steps 1, 4, 5 and 6 (`EX-6`, `VA-3`)
// ---------------------------------------------------------------------------

/// **PURE given [`HostFacts`].** Step 1's rule half: project and validate the
/// `[capsule]` table, then resolve the capsule root — the one part that needs
/// the host.
fn capsule_config(text: &str, host: &dyn HostFacts) -> Result<CapsuleConfig, ConfigRefusal> {
    root_capsule_config(parse_capsule_config(text)?, host)
}

/// Step 1, whole: read `[capsule]` from the working tree and apply the pure
/// rule to it.
///
/// `pub(crate)` for one caller: the `provision` verb, which needs
/// `bounds().kill_grace()` to construct the backend it then hands to
/// [`provision`] (`D3`). That is a **second** read of the same file in one
/// invocation — the cost of `EX-6`'s three fixed parameters, paid on a small
/// local file, and cheaper than moving the grace onto `Execution` and
/// cascading through every construction site for no observable difference.
pub(crate) fn host_capsule_config(
    repository_root: &Path,
    host: &dyn HostFacts,
) -> Result<CapsuleConfig, ProvisionRefusal> {
    let document = read_working_tree_document(repository_root)?;
    capsule_config(&document, host).map_err(ProvisionRefusal::Config)
}

/// **PURE.** Steps 4 and 5: the base policy as resolved from the contracted
/// base, and the policy actually in force after any refinement.
///
/// Step 5 goes through `sec-4`'s monotonic restriction algebra, so any widening
/// refuses; this module re-implements none of it (`EN-2`).
fn resolved_policy(
    base_document: &str,
    refinement_document: Option<&str>,
) -> Result<(InterpretationPolicy, InterpretationPolicy), ProvisionRefusal> {
    let base = interpretation::parse(base_document).map_err(ProvisionRefusal::BasePolicy)?;
    let Some(refinement_document) = refinement_document else {
        let policy = base.clone();
        return Ok((base, policy));
    };
    let refinement =
        interpretation::parse(refinement_document).map_err(ProvisionRefusal::RefinementPolicy)?;
    let policy =
        interpretation::restrict(&base, &refinement).map_err(ProvisionRefusal::Restriction)?;
    Ok((base, policy))
}

/// **PURE.** Step 6: admit the closure resolver **against the policy just
/// bound**.
///
/// The ordering is load-bearing (`EX-9`). This runs *after* step 5, because a
/// phase refinement may add a forbidden entry and the entry it adds must bind
/// the transaction it was supplied for. A check against the base policy alone
/// would let a refinement forbid an executable that this very provisioning had
/// already run.
///
/// The basename derivation is `interpretation::forbids`', not this module's:
/// a second spelling of *what counts as the executable's name* is a second
/// place for a denial to be routed around.
fn admit_resolver(
    policy: &InterpretationPolicy,
    resolver: Option<&Argv>,
) -> Result<(), ProvisionRefusal> {
    let Some(resolver) = resolver else {
        return Ok(());
    };
    let Some(executable) = resolver.as_slice().first() else {
        return Ok(());
    };
    if interpretation::forbids(policy, executable) {
        return Err(ProvisionRefusal::ForbiddenResolver {
            executable: executable.clone(),
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// T6: the thirteen steps
// ---------------------------------------------------------------------------

/// Provision a capsule transaction, or refuse.
///
/// The signature is exactly `EX-6`'s three parameters. The closure query is
/// injected through [`provision_with_query`] (`D2`) rather than by widening
/// this one: tests call the inner function, the resolver stays injectable, and
/// the public shape does not move.
///
/// # Errors
///
/// Returns the [`ProvisionRefusal`] naming the step that refused. **A refused
/// provision leaves no transaction and removes nothing it did not create**
/// (invariant 6): either a [`CapsuleTransaction`] comes back, or the root this
/// call exclusively created is gone and nothing else changed.
pub(crate) fn provision(
    request: &ProvisionRequest,
    host: &dyn HostFacts,
    backend: &dyn CapsuleBackend,
) -> Result<CapsuleTransaction, ProvisionRefusal> {
    provision_with_query(request, host, backend, &SpawnedClosureQuery)
}

fn provision_with_query(
    request: &ProvisionRequest,
    host: &dyn HostFacts,
    backend: &dyn CapsuleBackend,
    query: &dyn ClosureQuery,
) -> Result<CapsuleTransaction, ProvisionRefusal> {
    // Step 1 — read `[capsule]` from the working tree, then apply the pure rule.
    let config = host_capsule_config(&request.repository_root, host)?;
    ensure_container(config.root())?;

    // Step 3 — probe capacity against the capsule root. (Step 2's existence
    // probes are discharged by `readable_set` at step 7; see the module's
    // Findings note in the phase sheet.)
    let report = CapacityReport::of(
        assess_capacity(
            host.available_bytes(config.root()).map(ByteCount::from_bytes),
            config.capacity(),
        ),
        config.root().to_path_buf(),
    );
    if report.refuses() {
        return Err(ProvisionRefusal::Capacity(report));
    }
    emit_capacity(&report);

    // Steps 4 and 5 — the only read of the contracted base's document, and the
    // refinement applied to it.
    let base_document = read_base_document(&request.repository_root, &request.base)?;
    let refinement_document = read_refinement_document(request.refinement.as_deref())?;
    let (base_policy, policy) =
        resolved_policy(&base_document, refinement_document.as_deref())?;

    // Step 6 — admission, after step 5 and before any resolver invocation.
    admit_resolver(&policy, config.closure_resolver())?;

    // Steps 2 and 7 — the declared entries are probed and the closures expanded
    // by one call into the profile's seam.
    let readable = readable_set(&config, host, query).map_err(ProvisionRefusal::Profile)?;

    // Step 8 — publish or adopt the per-base export.
    let export = publish_or_adopt_export(
        config.root(),
        &request.repository_root,
        &request.base,
        &request.id,
    )?;

    // Step 9 — own the transaction root exclusively.
    let transaction_directory = config.root().join(TRANSACTION_DIRECTORY_LEAF);
    ensure_container(&transaction_directory)?;
    let root_path = transaction_directory.join(request.id.as_str());
    let token = create_exclusively(&root_path)?;
    let root = TransactionRoot::new(root_path);

    match finish(request, backend, &config, &root, &export, &readable, &policy, &base_policy) {
        Ok(transaction) => Ok(transaction),
        Err(refusal) => {
            roll_back(token);
            Err(refusal)
        }
    }
}

/// Steps 9's layout through 13, as one fallible unit so the caller can roll the
/// root back on any of them and on nothing before them.
#[expect(
    clippy::too_many_arguments,
    reason = "every value here was computed by an earlier numbered step and is \
              consumed by a later one; bundling them into a struct would mint a \
              type whose only meaning is 'the arguments of this function', which \
              is the shape `STD-002` and the design's own vocabulary rules \
              exist to avoid. `provision`'s own signature is unaffected (`EX-6`)"
)]
fn finish(
    request: &ProvisionRequest,
    backend: &dyn CapsuleBackend,
    config: &CapsuleConfig,
    root: &TransactionRoot,
    export: &Path,
    readable: &[PathBuf],
    policy: &InterpretationPolicy,
    base_policy: &InterpretationPolicy,
) -> Result<CapsuleTransaction, ProvisionRefusal> {
    // Step 9's layout: `capsule/{out,tmp}` and `agent/` beneath the root, on
    // real disk and never tmpfs (`EX-4`). `capsule/repo` is deliberately absent
    // — the clone creates it, **inside**, and no working tree is materialised
    // trusted-side.
    let capsule_state = profile_owned_host_path(root, INNER_CAPSULE);
    let agent_home = profile_owned_host_path(root, INNER_AGENT);
    let retained_scratch = capsule_state.join(CAPSULE_RETAINED_TMP_LEAF);
    for directory in [
        &capsule_state,
        &agent_home,
        &capsule_state.join(CAPSULE_OUTPUT_LEAF),
        &retained_scratch,
    ] {
        ensure_container(directory)?;
    }

    // Step 10 — assemble the placement.
    let placement = assemble_placement(
        request,
        config,
        root,
        export,
        readable,
        &retained_scratch,
    )?;

    // Steps 11 and 12 — the clone, inside.
    clone_inside(backend, &placement, config.bounds(), &request.base)?;

    // Step 13 — return the transaction.
    Ok(CapsuleTransaction {
        id: request.id.clone(),
        phase: request.phase.clone(),
        base: request.base.clone(),
        backend: backend.id(),
        base_policy: interpretation::canonical_hash(base_policy),
        policy_hash: interpretation::canonical_hash(policy),
        policy: policy.clone(),
        placement,
        bounds: config.bounds().clone(),
    })
}

/// Step 10 — every bound readable path read-only at its resolved host path, the
/// export read-only at the reserved `/source`, the **retained** scratch as the
/// one declared writable entry, working directory `/capsule`, network posture as
/// requested.
///
/// `/capsule` and `/agent` themselves are profile-owned mounts derived from the
/// [`TransactionRoot`] (`sec-2`), so they are not declared entries here — a
/// declared entry may not even *name* a reserved inner destination.
fn assemble_placement(
    request: &ProvisionRequest,
    config: &CapsuleConfig,
    root: &TransactionRoot,
    export: &Path,
    readable: &[PathBuf],
    retained_scratch: &Path,
) -> Result<CapsulePlacement, ProvisionRefusal> {
    // A resolved readable input appears inside at its own host path. That is
    // what makes the derived inner `PATH` (`sec-2`) mean anything: an
    // executable found at `/nix/store/…/bin` outside is at the same place
    // inside, so a resolver's answer needs no second translation table.
    let mut readable_entries = Vec::with_capacity(readable.len());
    for path in readable {
        readable_entries.push(MountedPath::new(path.clone(), inner_path(path)?));
    }

    // Exactly one declared writable entry: the **retained** scratch area
    // (`EX-5`). `/capsule` and `/agent` are profile-owned mounts derived from
    // the root, and `/tmp` is the profile's own tmpfs — a declared entry may not
    // even name them.
    let writable = vec![MountedPath::new(
        retained_scratch.to_path_buf(),
        inner_path(Path::new(&format!(
            "{INNER_CAPSULE}/{CAPSULE_RETAINED_TMP_LEAF}"
        )))?,
    )];

    let working_directory = inner_path(Path::new(INNER_CAPSULE))?;

    let scopes = ForbiddenScopes::new(
        request.repository_root.clone(),
        request.repository_root.join(CONTROL_PLANE_STATE_LEAF),
        config.root().to_path_buf(),
        Vec::new(),
    );

    CapsulePlacement::try_new(
        PlacementParts {
            root: root.clone(),
            source: SourceExport::new(export.to_path_buf(), request.base.clone()),
            writable,
            readable: readable_entries,
            working_directory,
            network: request.network,
            accepted_base: request.base.clone(),
        },
        &scopes,
    )
    .map_err(ProvisionRefusal::Placement)
}

/// The control plane's own state directory, beneath the repository root. Named
/// as a forbidden scope so a declared readable entry can never reach it.
const CONTROL_PLANE_STATE_LEAF: &str = ".doctrine";

/// An inner destination, or a refusal.
///
/// [`InnerPath::try_new`] refuses a relative or empty path. For the two literal
/// constants this is unreachable; for a resolved readable input it is
/// unreachable too, since `readable_set` resolves. It is still a refusal rather
/// than a drop: a silently omitted entry is a capsule missing an input it was
/// told it had, which is exactly the failure `sec-2`'s `F-4` measured.
fn inner_path(path: &Path) -> Result<InnerPath, ProvisionRefusal> {
    InnerPath::try_new(path.to_path_buf()).ok_or_else(|| {
        ProvisionRefusal::InnerDestinationNotAbsolute {
            path: path.to_path_buf(),
        }
    })
}

/// Steps 11 and 12 — **four** executions, not three.
///
/// `EX-14` names the three clone executions; step 12 (`design.md:1398-1401`) is
/// a fourth that reads the pinned identity back. Each is checked and the first
/// non-zero exit refuses, naming **which** step — three executions over one
/// aggregate status is what makes that possible.
///
/// The alternatives were considered and rejected: `sh -c` reintroduces the
/// quoting hazard on a security boundary; running 2 and 3 trusted-side would put
/// trusted Git in a capsule-authored repository, which `SPEC-030` forbids
/// outright; and a mounted helper script is the right answer for a conditional
/// *sequence*, not for three unconditional commands.
fn clone_inside(
    backend: &dyn CapsuleBackend,
    placement: &CapsulePlacement,
    bounds: &ResourceBounds,
    base: &AcceptedBase,
) -> Result<(), ProvisionRefusal> {
    let repository = format!("{INNER_CAPSULE}/{CAPSULE_REPOSITORY_LEAF}");

    // `--no-hardlinks` is a **mitigation, not tuning** (`EX-15`): a local clone
    // hardlinks object files by default, so a hostile capsule corrupting a
    // shared object would corrupt its source. The read-only binding makes the
    // write fail rather than corrupt; the flag is what survives someone making
    // the export writable.
    //
    // Identity is pinned by `git clone -c`, which takes effect after init and
    // before the fetch, so it covers the clone's own reflog writes. Configured
    // afterwards, git guesses an identity, resolves the hostname, and inside
    // `--unshare-all` that is a DNS query for an unshared UTS name that blocks
    // ~3.9s per ident-needing operation against 40ms pinned
    // (`mem.pattern.sandbox.git-ident-unset-dns-stall`).
    let clone = vec![
        GIT_EXECUTABLE.to_owned(),
        "clone".to_owned(),
        "--no-hardlinks".to_owned(),
        "--quiet".to_owned(),
        "-c".to_owned(),
        format!("{GIT_IDENTITY_NAME_KEY}={CAPSULE_GIT_IDENTITY_NAME}"),
        "-c".to_owned(),
        format!("{GIT_IDENTITY_EMAIL_KEY}={CAPSULE_GIT_IDENTITY_EMAIL}"),
        "--".to_owned(),
        INNER_SOURCE.to_owned(),
        repository.clone(),
    ];
    run_inside(backend, placement, bounds, CloneStep::Clone, clone)?;

    let detach = vec![
        GIT_EXECUTABLE.to_owned(),
        "-C".to_owned(),
        repository.clone(),
        "switch".to_owned(),
        "--detach".to_owned(),
        "--quiet".to_owned(),
        base.as_str().to_owned(),
    ];
    run_inside(backend, placement, bounds, CloneStep::Detach, detach)?;

    // No remotes: the capsule has nowhere to push. Harvest, when a later slice
    // builds it, is a control-plane pull from the capsule, never a
    // capsule-initiated write outward.
    let remove_origin = vec![
        GIT_EXECUTABLE.to_owned(),
        "-C".to_owned(),
        repository.clone(),
        "remote".to_owned(),
        "remove".to_owned(),
        "origin".to_owned(),
    ];
    run_inside(backend, placement, bounds, CloneStep::RemoveOrigin, remove_origin)?;

    // Step 12 — assert the identity **persisted**, by reading it back from
    // inside. This is not a trusted-side read of `<root>/capsule/repo/config`:
    // that would be a trusted touch of a capsule-authored repository, which
    // `SPEC-030` forbids, and a different assertion besides.
    let read_back = vec![
        GIT_EXECUTABLE.to_owned(),
        "-C".to_owned(),
        repository,
        "config".to_owned(),
        "--get-regexp".to_owned(),
        GIT_IDENTITY_PATTERN.to_owned(),
    ];
    let observed = run_inside(backend, placement, bounds, CloneStep::ReadIdentity, read_back)?;
    assert_identity_persisted(&observed)
}

/// One `backend.execute`, checked.
fn run_inside(
    backend: &dyn CapsuleBackend,
    placement: &CapsulePlacement,
    bounds: &ResourceBounds,
    step: CloneStep,
    words: Vec<String>,
) -> Result<String, ProvisionRefusal> {
    let Some(argv) = Argv::try_new(words) else {
        return Err(ProvisionRefusal::EmptyExecutionArgv { step });
    };
    let execution = Execution::new(
        argv,
        CapsuleEnv::complete(),
        bounds.timeout(),
        bounds.file_size_cap(),
        CapsuleStdio::EmptyInputCapturedOutput,
    );
    let observation = backend
        .execute(placement, &execution)
        .map_err(ProvisionRefusal::Backend)?;
    if observation.termination == (Termination::Exited { code: 0 }) {
        Ok(String::from_utf8_lossy(&observation.stdout).into_owned())
    } else {
        Err(ProvisionRefusal::CloneFailed {
            step,
            termination: observation.termination,
        })
    }
}

/// `git config --get-regexp` prints `<key> <value>` per line. Both the capsule
/// name and the capsule email must be present with **exactly** the trusted-side
/// constants: a git that stopped persisting `-c` would otherwise restore the
/// resolver stall silently.
fn assert_identity_persisted(read_back: &str) -> Result<(), ProvisionRefusal> {
    let mut name = false;
    let mut email = false;
    for line in read_back.lines() {
        if let Some(value) = line.strip_prefix(&format!("{GIT_IDENTITY_NAME_KEY} ")) {
            name |= value.trim_end() == CAPSULE_GIT_IDENTITY_NAME;
        }
        if let Some(value) = line.strip_prefix(&format!("{GIT_IDENTITY_EMAIL_KEY} ")) {
            email |= value.trim_end() == CAPSULE_GIT_IDENTITY_EMAIL;
        }
    }
    if name && email {
        Ok(())
    } else {
        Err(ProvisionRefusal::IdentityNotPersisted {
            read_back: read_back.to_owned(),
        })
    }
}

// ---------------------------------------------------------------------------
// The impure reads
// ---------------------------------------------------------------------------

/// Step 1's impure half: the `[capsule]` table is read from the **working
/// tree**, because it is the operator's host configuration, not the contract.
fn read_working_tree_document(repository_root: &Path) -> Result<String, ProvisionRefusal> {
    let path = repository_root.join(DOCTRINE_TOML);
    std::fs::read_to_string(&path).map_err(|error| ProvisionRefusal::ConfigUnreadable {
        path,
        detail: error.to_string(),
    })
}

/// Step 4's impure half — **the only** read of `.doctrine/doctrine.toml` from
/// the contracted base, `DEC-136`'s read-once invariant made physical.
///
/// Through `read_path_at`, which is `git cat-file -p <base>:<path>`: the blob is
/// read from the object store and **no working tree is materialised**.
fn read_base_document(
    repository_root: &Path,
    base: &AcceptedBase,
) -> Result<String, ProvisionRefusal> {
    match read_path_at(repository_root, base.as_str(), DOCTRINE_TOML) {
        Ok(Some(text)) => Ok(text),
        Ok(None) => Err(ProvisionRefusal::BaseDocumentAbsent {
            path: DOCTRINE_TOML.to_owned(),
        }),
        Err(error) => Err(ProvisionRefusal::BaseDocumentUnreadable {
            detail: error.to_string(),
        }),
    }
}

/// Step 5's impure half: the refinement arrives as its own document, by path.
fn read_refinement_document(
    refinement: Option<&Path>,
) -> Result<Option<String>, ProvisionRefusal> {
    let Some(path) = refinement else {
        return Ok(None);
    };
    std::fs::read_to_string(path)
        .map(Some)
        .map_err(|error| ProvisionRefusal::RefinementUnreadable {
            path: path.to_path_buf(),
            detail: error.to_string(),
        })
}

/// Emit a **non-refusing** capacity report (`REQ-461` criterion 1).
///
/// The report is emitted here rather than at the verb because `EX-6` fixes
/// `provision` at three parameters and `EX-1` fixes the transaction at nine
/// fields, so a non-refusing report has no return channel — and dropping it
/// would leave `REQ-461`'s conspicuous warning unimplemented while
/// `CapacityReport::Warn` looked constructed. Structured `key=value`, never a
/// sentence: an operator triaging a stalled queue greps it.
///
/// `clippy::print_stderr` is denied and this is not that lint's shape: the
/// handle is locked once and written through [`std::io::Write`], which is what
/// the ban's reason string asks for.
fn emit_capacity(report: &CapacityReport) {
    let line = match report {
        CapacityReport::Proceed { .. } => return,
        CapacityReport::Warn {
            available_bytes,
            expected_bytes,
            threshold_bytes,
            capsule_root,
            key,
        } => format!(
            "capacity=low capsule-root={} available-bytes={available_bytes} \
             expected-bytes={expected_bytes} threshold-bytes={threshold_bytes} key={key}",
            capsule_root.display()
        ),
        CapacityReport::Report {
            reason,
            capsule_root,
            key,
        } => format!(
            "capacity=unknown capsule-root={} reason={} key={key}",
            capsule_root.display(),
            unknown_reason(*reason)
        ),
        CapacityReport::Refuse { .. } => return,
    };
    let mut stderr = std::io::stderr().lock();
    let _written = writeln!(stderr, "{line}");
}

/// Why the probe was unusable, as a stable token rather than a `Debug` render:
/// `Debug` output is not a contract and an operator's grep would break the first
/// time a variant gained a field.
const REASON_PROBE_FAILED: &str = "probe-failed";
const REASON_UNUSABLE_FIGURE: &str = "unusable-figure";
const REASON_FIGURE_OVERFLOWS: &str = "figure-overflows";

fn unknown_reason(reason: CapacityUnknown) -> String {
    match reason {
        CapacityUnknown::ProbeFailed { errno } => format!("{REASON_PROBE_FAILED}:{errno}"),
        CapacityUnknown::UnusableFigure => REASON_UNUSABLE_FIGURE.to_owned(),
        CapacityUnknown::FigureOverflows => REASON_FIGURE_OVERFLOWS.to_owned(),
    }
}
