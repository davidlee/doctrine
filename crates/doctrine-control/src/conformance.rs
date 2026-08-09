// SPDX-License-Identifier: GPL-3.0-only
//! The conformance suite — what a backend must prove before it is admitted
//! (`SL-248` `sec-7`, `DEC-156`, `DEC-160`).
//!
//! `DEC-160` gives this module exactly **two** callers:
//!
//! - `backend verify` (PHASE-10), for the on-host verdict, exiting nonzero with
//!   structured output when the backend is not admitted;
//! - a `#[cfg(test)]` module **in this file**, so `cargo test` runs the suite.
//!
//! The second caller is a unit test rather than a `tests/` file and that is
//! forced, not preferred: `doctrine-control` is bin-only (`sec-6`), a `tests/`
//! file cannot link one (`E0433`), and adding a lib target to rescue it fails
//! `E0603` until `verify` is promoted to `pub` — which would take the weakening
//! vocabulary public with it.
//!
//! ## The sealing assumption (`EX-17`), recorded rather than silently relied on
//!
//! [`PropertyRemoval`], [`AuthorityGrant`], [`HostPid`] and
//! [`ConformanceBackend`] are `pub(crate)` because **every backend lives in this
//! crate** (`sec-6`: `backend/bubblewrap.rs`, plural-ready) and the only callers
//! are `main.rs` and this module's own tests. That is an assumption, not a law.
//! If a backend ever ships from outside the crate, this vocabulary becomes
//! public API and needs sealing behind a newtype over a private enum, so that
//! nothing but the suite can ask a backend to weaken itself (`sec-9`).
//!
//! ## What this phase is
//!
//! PHASE-07 lands the pure core: the weakening vocabulary, classification, and
//! the row and admission verdict algebra. Nothing here executes a capsule. The
//! executing harness is PHASE-08's, tables A and B are PHASE-09's and
//! PHASE-10's. See [`tables`] for what that means for [`verify`]'s outcome
//! today.
//!
//! Layering (`ADR-001`): engine tier. `EX-1` records five out-edges —
//! `provision`, `transaction`, `backend`, `config`, `host` — of which this phase
//! carries three; `provision` and `transaction` arrive with PHASE-08's fixture
//! and freshness delta.
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SL-248 PHASE-07 lands the suite's pure vocabulary one phase ahead of the \
                  harness that runs it (PHASE-08) and the verb that calls it (PHASE-10), so \
                  under cfg(not(test)) the whole module is dead. The suppression is stripped \
                  under cfg(test), where `unused = deny` still requires every item to be \
                  named by a test."
    )
)]

use std::cell::{Cell, OnceCell, RefCell};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use std::os::unix::fs::PermissionsExt as _;
use std::os::unix::net::UnixStream;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use doctrine::DOCTRINE_TOML;
use doctrine::interpretation::PolicyHash;
use rustix::fs::{FsWord, OFlags};
use rustix::io::{FdFlags, fcntl_getfd, fcntl_setfd};

use crate::backend::bubblewrap::{
    BubblewrapBackend, WeakenedProfile, Weakening, mechanism_failed, profile_owned_host_path,
};
use crate::backend::{
    AcceptedBase, Availability, BackendError, BackendId, CapsuleBackend, CapsuleEnv, CapsuleEnvVar,
    CapsulePlacement, CapsuleStdio, EXPORT_DIRECTORY_LEAF, Execution, FILESYSTEM_ROOT,
    ForbiddenScopes, INNER_CAPSULE, MountedPath, NetworkPosture, Observation, PlacementParts,
    Termination, TransactionRoot,
};
use crate::config::{Argv, ByteCount};
use crate::host::HostFacts;
use crate::provision::{ProvisionRequest, provision};
use crate::transaction::{CapsuleTransaction, PhaseIdentity, TransactionId};

// ---------------------------------------------------------------------------
// Payload constants (`STD-001`, `EX-16`)
// ---------------------------------------------------------------------------

/// The token every payload prints before anything else, and the whole of stage
/// one of classification (`EX-10`).
const LIVENESS_MARKER: &str = "LIVE";

/// The interpreter every payload runs under. Payloads are `Argv` values holding
/// fixed constants — not compiled helpers, not embedded assets (`DEC-160`,
/// `EX-16`). They do invoke `/bin/sh -c`, which provisioning deliberately
/// avoids; `sec-2`'s rule exists to keep *caller-supplied* text off a security
/// boundary, and these payloads interpolate nothing except row B5's
/// trusted-side-observed pid.
const SHELL: &str = "/bin/sh";

/// What would satisfy a host that has no [`SHELL`] (`POL-002` facet 3).
const SHELL_REMEDY: &str = "install a POSIX shell at /bin/sh";

/// Where the kernel release is read from, for [`host_descriptor`].
const KERNEL_RELEASE: &str = "/proc/sys/kernel/osrelease";

/// The documented fallback when a host fact cannot be read.
const UNKNOWN_HOST_FACT: &str = "unknown";

// ---------------------------------------------------------------------------
// The weakening vocabulary (`EX-4`, `EX-5`, `EX-6`)
// ---------------------------------------------------------------------------

/// One property a control arm removes.
///
/// **Nine variants, ten removals** — [`PropertyRemoval::ResourceBound`] carries
/// two ([`Bound::FileSize`] and [`Bound::Wall`]) — and the two numbers are not
/// interchangeable: anything counting the *vocabulary* says nine, anything
/// counting what `execute_weakened` must handle says ten. An earlier draft of
/// `EX-4` said "ten variants" above a list of nine, which is why the count is
/// written here rather than left to be recomputed.
///
/// `SharedRoot` is **not** a member: it is a [`Delta`] variant (`EX-8`), because
/// it re-points a placement rather than removing a profile property. A grant is
/// not a member either — see [`AuthorityGrant`].
///
/// The vocabulary is **property-shaped, never flag-shaped**: what a removal
/// costs in flags is the backend's business, and a flag-shaped vocabulary would
/// be bubblewrap's and could not be asked of a second backend. Every variant
/// names the mechanism unique to it *and what it does not change*, because a
/// removal that moves two things names no mechanism when its row fails.
///
/// Closed and **complete at this phase** (`EX-5`): PHASE-08 implements all ten
/// removals at once against a finished enum, because a non-exhaustive match does
/// not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PropertyRemoval {
    /// Row 6. The capsule's working directory is not set, so the process starts
    /// wherever the mechanism leaves it. Changes no mount, no descriptor, no
    /// environment variable and no identity.
    WorkingDirectory,
    /// Row 7. The capsule's process tree is not torn down with its parent.
    /// Changes no mount and no environment variable — only what happens to
    /// descendants when the trusted side goes away.
    Teardown,
    /// Row 8's process half. The capsule shares the host's pid namespace instead
    /// of receiving its own. Changes no mount, no descriptor and no environment
    /// variable; it is the one removal expressed by enumeration rather than
    /// subtraction, because bubblewrap has no `--share-pid`.
    ProcessVisibility,
    /// Rows 7 and 8's resource half. One bound is not applied to the child.
    /// Changes no mount, no descriptor and no environment variable. **The one
    /// variant carrying two removals** — see [`Bound`].
    ResourceBound(Bound),
    /// Row 9. Every readable input and the source export become writable — the
    /// mechanism unique to input immutability, and the only removal that changes
    /// how an existing mount is *bound* rather than which mounts exist.
    InputsWritable,
    /// Row 10. The fixture's three decoy descriptors — readable, write-only and
    /// one end of a socket pair — are left inheritable across `exec` instead of
    /// close-on-exec. Changes no mount, no environment variable, no argv byte
    /// and nothing at or below descriptor 2 — the mechanism unique to descriptor
    /// closure *above* the standard streams.
    DescriptorsClosed,
    /// Row 11. The trusted-side environment is not cleared before the capsule's
    /// own is applied. Changes no mount and no descriptor.
    EnvCleared,
    /// Row 12. Descriptors 0, 1 and 2 are inherited from the trusted-side
    /// process instead of being attached to parent-owned endpoints. Disjoint
    /// from [`PropertyRemoval::DescriptorsClosed`] by descriptor number, which
    /// is what keeps the two independently controllable rather than one widening
    /// of the other.
    StdioOwned,
    /// Row 13. The capsule's declared identity is not applied — it runs as
    /// whatever uid and gid the user namespace maps by default, which is the
    /// trusted side's. Changes no mount, no descriptor and no environment
    /// variable.
    ///
    /// Replaces an earlier `CredentialsConfined`, which named four mechanisms at
    /// once and was measured not to fire on any of them (`EVD-014`, `RV-346`
    /// `F-32`). This one is measured to fire.
    MappedIdentity,
}

impl PropertyRemoval {
    /// All **ten** removals — nine variants, of which one carries two bounds.
    ///
    /// Named as data here for the same reason `CapsuleEnvVar::ALL` is: the
    /// exhaustive `match` PHASE-08 writes over this enum is what makes widening
    /// it a compile error rather than a silent gap in a hand-maintained list.
    pub(crate) const ALL: &'static [Self] = &[
        Self::WorkingDirectory,
        Self::Teardown,
        Self::ProcessVisibility,
        Self::ResourceBound(Bound::FileSize),
        Self::ResourceBound(Bound::Wall),
        Self::InputsWritable,
        Self::DescriptorsClosed,
        Self::EnvCleared,
        Self::StdioOwned,
        Self::MappedIdentity,
    ];
}

/// The two resource bounds [`PropertyRemoval::ResourceBound`] can drop. Each is
/// a removal in its own right; together they are why nine variants carry ten
/// removals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Bound {
    /// The child's file-size rlimit is not set.
    FileSize,
    /// The wall-clock kill wrapper is omitted.
    Wall,
}

/// One authority a control arm **grants**.
///
/// Separate from [`PropertyRemoval`] because the two move the capsule in
/// opposite directions, and a reader counting removals must not silently count a
/// grant among them (`F-31`'s lesson one level up). See [`Delta::Granted`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthorityGrant {
    /// Row 14. Every capability is added to the capsule's sets inside its own
    /// user namespace. Changes no mount, no descriptor, no environment variable
    /// and no identity — the mechanism unique to capability confinement.
    AllCapabilities,
}

/// A pid as the trusted parent sees it, in the **host's** own pid namespace.
///
/// A newtype because the whole point of row B5 is that this number means
/// something different inside a capsule than it does outside one: under the
/// probe arm the observer has its own namespace, in which the number names
/// nothing, so the assertion holds for the reason the property claims rather
/// than for an arithmetic one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HostPid(pub(crate) i32);

/// The admission instrumentation, deliberately **not** on `CapsuleBackend`
/// (invariant 6).
///
/// Production uses `CapsuleBackend` and it carries no way to weaken anything. A
/// backend seeking admission implements this second trait as well, and
/// implementing it is not a favour: without controls the suite proves nothing,
/// so `DEC-156`'s discipline *is* this obligation.
///
/// **A dishonest implementation fails closed** (`EX-7`). An `execute_weakened`
/// that ignores its argument and runs fully confined makes every control arm
/// show the property still holding, which the harness reports as
/// [`RowVerdict::Unproven`] — not admitted. There is no lazy implementation
/// that yields a green verdict.
pub(crate) trait ConformanceBackend: CapsuleBackend {
    /// The supertrait view, spelled by hand.
    ///
    /// `provision` takes `&dyn CapsuleBackend`, and coercing `&dyn
    /// ConformanceBackend` to it is *trait upcasting* — stable from Rust 1.86,
    /// where this workspace's MSRV is 1.85. `clippy::incompatible_msrv` catches
    /// std **APIs** below the floor, not language features, so an upcast here
    /// would compile on the developer's toolchain and fail only on the oldest
    /// one this crate claims to support. Three lines per impl buys that back.
    fn as_capsule_backend(&self) -> &dyn CapsuleBackend;

    /// Run as `execute` does, with exactly one property removed from the
    /// profile.
    fn execute_weakened(
        &self,
        placement: &CapsulePlacement,
        execution: &Execution,
        removal: PropertyRemoval,
    ) -> Result<Observation, BackendError>;

    /// Run as `execute` does, with exactly one authority **granted** to the
    /// capsule.
    ///
    /// A second method rather than an eleventh [`PropertyRemoval`], for the
    /// reason [`AuthorityGrant`] is a second enum: a grant moves the capsule in
    /// the opposite direction from a removal, and a caller counting removals
    /// must not silently count a grant among them. `sec-7`'s ruling — *a control
    /// must negate exactly one protection* — is what both methods serve; which
    /// direction the backend negates it from is the backend's business.
    ///
    /// **Divergence from `design.md:2960`**, whose trait sketch has two methods
    /// and no execution path for `Delta::Granted` at all. Recorded as PHASE-08
    /// `F-13`; the row it serves (14) is in the design's own table.
    fn execute_granted(
        &self,
        placement: &CapsulePlacement,
        execution: &Execution,
        grant: AuthorityGrant,
    ) -> Result<Observation, BackendError>;

    /// Run as `execute` does, and call `observer` exactly once — trusted-side,
    /// after the capsule's top-level process exists and before this returns —
    /// with that process's pid *in the host's pid namespace*.
    ///
    /// Row B5's seam, and it exists because nothing else can supply one.
    /// `CapsuleBackend::execute` is synchronous and yields only an
    /// [`Observation`] after the run is over, so a harness holding it in flight
    /// on a thread still cannot name the process it started; and a pid the
    /// capsule reports about itself is capsule-written state, which invariant 9
    /// and `REQ-448` criterion 3 forbid as evidence.
    ///
    /// A backend that returns without calling `observer` yields
    /// [`Indeterminacy::NoLiveness`], never a held probe — see
    /// [`classify_concurrent`].
    fn execute_observed(
        &self,
        placement: &CapsulePlacement,
        execution: &Execution,
        observer: &dyn Fn(HostPid),
    ) -> Result<Observation, BackendError>;
}

// ---------------------------------------------------------------------------
// The arm vocabulary (`EX-9`, `EX-11`)
// ---------------------------------------------------------------------------

/// What the arm is read off, once liveness is established.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Observed {
    /// Exactly one of two tokens on stdout. Both is
    /// [`Indeterminacy::AmbiguousObservation`] — the payload is wrong — and
    /// neither is [`Indeterminacy::NoObservation`].
    Token {
        held: &'static str,
        failed: &'static str,
    },
    /// A value line, compared for equality — row 6, where the observation is a
    /// value rather than a binary. **A missing value line is
    /// [`Indeterminacy::NoObservation`], never [`ArmResult::Failed`]**, which is
    /// a different thing from a value line carrying the wrong value.
    Exactly(String),
    /// The termination itself is the observation — row 8. The marker still
    /// arrives, because [`Observation`] carries `stdout` whatever the
    /// termination: `sh -c 'echo LIVE; exec sleep 60'` prints before it is
    /// killed.
    ///
    /// **The one payload that cannot print its own marker** is row 8's
    /// `Termination::NotExecutable` case, which by definition never runs
    /// (`EX-12`). Its liveness must come from a **liveness execution preceding
    /// it in the same capsule**, and only if that succeeds is a subsequent
    /// `NotExecutable` meaningful. Without that ordering the sub-row passes on
    /// any host where the shell is missing — for the wrong reason, and looking
    /// exactly like success. That ordering is PHASE-09's to build; the rule is
    /// written here, where the kind is declared, so it cannot be lost.
    Termination(Termination),
}

/// One capsule's payload and what it is read for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Probe {
    pub(crate) argv: Argv,
    pub(crate) observed: Observed,
}

/// Row B5's observer.
///
/// The subject's pid is not known until the subject is running, so the
/// observer's argv is built from it rather than fixed — the only value any
/// payload interpolates, and it is trusted-side-computed. The pid is the one the
/// *trusted side* observed the subject running under, never one the subject
/// reported (`REQ-448` criterion 3).
/// No `PartialEq`: `argv` is a function pointer, and comparing function
/// pointers is `unpredictable_function_pointer_comparisons` — their addresses
/// are not unique across codegen units. Nothing needs to compare two payloads;
/// what tests compare is the [`ArmResult`] a payload produced. The same
/// constraint carries up through [`ArmShape`], [`Delta`] and [`Row`].
#[derive(Debug, Clone)]
pub(crate) struct PidProbe {
    pub(crate) argv: fn(HostPid) -> Argv,
    pub(crate) observed: Observed,
}

/// What runs, in one arm.
///
/// **Identical on a row's probe and control arms** — a control that changed the
/// payload would not be a control (invariant 3). The arm, not the capsule, is
/// the unit of execution, because six rows across tables A and B are two-capsule
/// and one runs its two concurrently.
#[derive(Debug, Clone)]
pub(crate) enum ArmShape {
    /// One capsule.
    Single(Probe),
    /// Two capsules in sequence: `writer` runs to completion, then `reader`
    /// observes. Rows 1 and B1–B4.
    Sequential { writer: Probe, reader: Probe },
    /// Two capsules concurrently. The trusted side observes the subject's pid
    /// and renders it into the observer's argv as a decimal integer. Row B5.
    Concurrent { subject: Probe, observer: PidProbe },
}

/// What one arm showed about the property under test.
///
/// Uniform across all three [`Observed`] kinds, which is why the vocabulary is
/// *held/failed* rather than *reached/denied*: rows 6 and 8 observe a value and
/// a termination, not a reach.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ArmResult {
    Held,
    Failed,
    /// Every indeterminate arm carries the arm's own diagnostics, which is what
    /// makes it triageable rather than a shrug.
    Indeterminate {
        reason: Indeterminacy,
        termination: Termination,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
}

/// Why an arm established nothing. Closed at four variants (`EX-11`).
///
/// **The concurrent readings, recorded rather than left to be guessed.**
/// [`classify_concurrent`] maps `execute_observed` returning without ever
/// calling the observer to [`Indeterminacy::NoLiveness`] — the observer probe
/// never ran, so nothing established that it ran — and the subject exiting
/// before the observer ran to [`Indeterminacy::NoObservation`] — the observer
/// ran and reported, but the window did not exist, so the arm produced no usable
/// observation of the property. The second is a **stretch** of a variant whose
/// primary sense is *live, but neither token*; it is taken rather than widening a
/// closed vocabulary, and it is written down because the two modes must take
/// different reasons or the two concurrent tests assert the same thing about
/// different inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Indeterminacy {
    /// No liveness marker: the payload did not run. Never folded into
    /// [`ArmResult::Failed`] (invariant 5).
    NoLiveness,
    /// Live, but neither token — or, for [`Observed::Exactly`], no value line.
    NoObservation,
    /// Both tokens, which means the payload is wrong.
    AmbiguousObservation,
    /// The backend failed to run the arm at all. A `String` and not the
    /// `BackendError` itself, so the verdict carries a rendering the backend
    /// cannot later change under it.
    BackendError(String),
}

/// Which arm of a row a verdict is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Which {
    Probe,
    Control,
}

// ---------------------------------------------------------------------------
// The fixture's root: real disk, never tmpfs (`EX-2`, `D1`)
// ---------------------------------------------------------------------------

/// `statfs.f_type` for tmpfs, from `linux/magic.h`.
///
/// Spelled here rather than imported: `rustix` exposes `StatFs::f_type` but no
/// filesystem-magic constants, and `libc` is not a dependency of this crate
/// (adding one is `S4`). [`rustix::fs::FsWord`] is the field's own type, so the
/// comparison needs no cast — `as_conversions` is denied.
const TMPFS_MAGIC: FsWord = 0x0102_1994;

/// The candidate roots, in `D1`'s order.
const XDG_DATA_HOME_VARIABLE: &str = "XDG_DATA_HOME";
const HOME_VARIABLE: &str = "HOME";
/// `$HOME`-relative tail of the second candidate.
const USER_DATA_TAIL: &str = ".local/share";
/// The third candidate's marker: scratch inside the enclosing repository's git
/// directory, which is the precedent
/// `mem.pattern.tooling.tempfile-dev-only-use-git-dir-scratch-index` sets.
const GIT_DIRECTORY_LEAF: &str = ".git";
/// The directory every run's root is created beneath, so one uninstrumented
/// `rm -rf` reclaims a machine that crashed mid-suite.
const SCRATCH_DIRECTORY_LEAF: &str = "doctrine-conformance";

/// Why the fixture could not be built.
///
/// A refusal rather than a panic: [`verify`] is a production path
/// (`backend verify`, PHASE-10), and a host that cannot supply a real-disk
/// scratch root, a Git, or a loopback socket is a fact about the host,
/// reported the way `NotAdmitted::Unavailable` reports a missing shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FixtureFault {
    /// Every candidate root was unwritable, absent, or on tmpfs. Carries the
    /// candidates in the order they were tried, so the operator can see which
    /// of `D1`'s three was reached.
    NoRealDiskRoot { rejected: Vec<PathBuf> },
    /// A filesystem operation the fixture performs on its **own** root failed.
    Io { path: PathBuf, detail: String },
    /// A trusted-side Git invocation building the fixture repository failed.
    Git { argv: Vec<String>, detail: String },
    /// The host named no directory the fixture could declare readable, so every
    /// payload would be `NotExecutable` and the whole run indeterminate for a
    /// reason it never states (`EX-3`).
    NoReadableRoots,
}

/// The next root's discriminator within this process.
///
/// Pid **plus** a counter, and no clock: `HostFacts` carries none by design
/// (`sec-5`), two fixtures built in the same millisecond would collide on one,
/// and a monotonic counter is the honest source for *the next one* anyway.
static ROOT_NONCE: AtomicU32 = AtomicU32::new(0);

/// A directory on real disk, created for one run, removed when it drops.
///
/// **`EX-2` is a claim, so it is checked and not commented.** `DEC-156`
/// requires the fixture root to be on real disk: `sec-5`'s capacity probe must
/// read real available space, and on tmpfs every figure is the mount's, both
/// capacity claims still agree with their own `statvfs`, and `REQ-461`'s only
/// executed evidence quietly becomes a measurement of a RAM disk — with nothing
/// red. So the constructor runs `statfs` on the chosen base and refuses
/// [`TMPFS_MAGIC`]. `std::env::temp_dir()` is `/tmp` and `/tmp` is tmpfs on the
/// host this was built against, which is why the obvious route is the wrong one.
///
/// **`Drop` is the whole of cleanup** (invariant 8). Nothing public on this type
/// removes anything, and this phase introduces no capsule-delete capability
/// anywhere: `DEC-133`/`DEC-137` hold that a harvested capsule is live work, and
/// a delete primitive added here for tidiness is what a later slice would reach
/// for.
#[derive(Debug)]
pub(crate) struct TempRoot {
    path: PathBuf,
}

impl TempRoot {
    /// The first writable, non-tmpfs candidate of `D1`'s three, with a fresh
    /// pid-and-nonce-named directory created inside it.
    ///
    /// The environment is read through [`HostFacts::env_var`] — `std::env::var`
    /// is banned by `clippy.toml`'s `disallowed-methods`, and routing through
    /// the trait is what lets a test drive the selection without mutating the
    /// process environment.
    pub(crate) fn new(host: &dyn HostFacts) -> Result<Self, FixtureFault> {
        let candidates = candidate_bases(host);
        for base in &candidates {
            if let Some(root) = prepare_root(base) {
                return Ok(Self { path: root });
            }
        }
        Err(FixtureFault::NoRealDiskRoot {
            rejected: candidates,
        })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempRoot {
    /// Best effort, and deliberately silent: a `Drop` that could fail loudly
    /// would have to panic, and `panic` is denied. The failure mode this guards
    /// is a developer's machine filling with abandoned roots, which `VA-1`
    /// checks once rather than trusting to a red test.
    fn drop(&mut self) {
        drop(std::fs::remove_dir_all(&self.path));
    }
}

/// `D1`'s ordered candidate list, filtered to what this host actually names.
///
/// `$XDG_DATA_HOME`, then `$HOME/.local/share`, then the enclosing repository's
/// git directory. A candidate whose variable is unset is absent from the list
/// rather than present and failing, so [`FixtureFault::NoRealDiskRoot`] reports
/// what was *tried*.
fn candidate_bases(host: &dyn HostFacts) -> Vec<PathBuf> {
    let mut bases: Vec<PathBuf> = Vec::new();
    if let Some(data_home) = host.env_var(XDG_DATA_HOME_VARIABLE) {
        bases.push(PathBuf::from(data_home));
    }
    if let Some(home) = host.env_var(HOME_VARIABLE) {
        bases.push(PathBuf::from(home).join(USER_DATA_TAIL));
    }
    if let Some(git_directory) = enclosing_git_directory() {
        bases.push(git_directory);
    }
    bases
        .into_iter()
        .map(|base| base.join(SCRATCH_DIRECTORY_LEAF))
        .collect()
}

/// The `.git` of the first ancestor of the working directory that has one.
///
/// The working directory is read directly: `HostFacts` has four methods and no
/// cwd (`sec-5`), and widening it for the third fallback of a test-scaffolding
/// root would move a production trait for a scratch path.
fn enclosing_git_directory() -> Option<PathBuf> {
    let start = std::env::current_dir().ok()?;
    start
        .ancestors()
        .map(|ancestor| ancestor.join(GIT_DIRECTORY_LEAF))
        .find(|candidate| candidate.is_dir())
}

/// Create a run root beneath `base`, or `None` if `base` will not serve.
///
/// Three ways to fail and they are deliberately not distinguished: the base
/// cannot be created, the base is on tmpfs, or the run root cannot be created
/// inside it. All three mean *try the next candidate*, and the last is also the
/// writability check — `create_dir_all` on an existing unwritable directory
/// succeeds, so only creating something proves the base is usable.
fn prepare_root(base: &Path) -> Option<PathBuf> {
    std::fs::create_dir_all(base).ok()?;
    if !on_real_disk(base) {
        return None;
    }
    let root = base.join(format!(
        "{}-{}",
        std::process::id(),
        ROOT_NONCE.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&root).ok()?;
    Some(root)
}

/// Whether `path` is on a filesystem that is not tmpfs.
///
/// A failed `statfs` reads as *not* real disk: the check exists to refuse
/// anything it cannot positively establish is backed by a disk, and answering
/// "fine" to an unanswerable probe is the failure `M20` is aimed at.
fn on_real_disk(path: &Path) -> bool {
    matches!(rustix::fs::statfs(path), Ok(stat) if stat.f_type != TMPFS_MAGIC)
}

// ---------------------------------------------------------------------------
// The fixture: a self-contained control plane (`EX-1`, `EX-3`, `EX-4`, `EX-5`)
// ---------------------------------------------------------------------------

/// The layout beneath the run root. Every one of these is created by
/// [`Fixture::new`] and removed by [`TempRoot`]'s `Drop`.
const PROJECT_LEAF: &str = "project";
const CAPSULES_LEAF: &str = "capsules";
const DECOYS_LEAF: &str = "decoys";
const DECOY_CREDENTIAL_LEAF: &str = "credential";
const DECOY_REPOSITORY_LEAF: &str = "repository";
const DECOY_READABLE_INPUT_LEAF: &str = "readable-input";
const DECOY_UNDECLARED_LEAF: &str = "undeclared";
const DECOY_EXECUTABLE_LEAF: &str = "executable";
/// Row 10's readable decoy — the only one of [`InheritableDecoys`]' three that
/// has a name.
const DECOY_DESCRIPTOR_LEAF: &str = "descriptor";

/// The fixture repository's second branch, which is what gives it an object the
/// contracted base cannot reach.
const UNREACHABLE_BRANCH: &str = "unreachable-from-base";
const UNREACHABLE_LEAF: &str = "unreachable.txt";

/// The identity the fixture repository's commits are made under. Pinned rather
/// than guessed, for the reason `provision` pins the capsule's: an unset
/// identity makes Git resolve the hostname, and inside an unshared UTS
/// namespace that is a multi-second DNS stall
/// (`mem.pattern.sandbox.git-ident-unset-dns-stall`).
const FIXTURE_IDENTITY_NAME: &str = "Conformance Fixture";
const FIXTURE_IDENTITY_EMAIL: &str = "conformance@example.invalid";

/// The trusted-side Git the fixture repository is built with. Spelled here
/// because `provision`'s own constant is private to that module, and
/// `provision.rs` is not a file this phase owns (`S1`).
const GIT: &str = "git";

/// Row 5's target: a listener the trusted side owns, on loopback, on a
/// kernel-assigned port. Never the internet (`EX-11`).
const LOOPBACK_ANY_PORT: &str = "127.0.0.1:0";

/// Where the host's mount table is read from for the second-filesystem
/// selection. Parsed, never guessed — hardcoding `/tmp` is what `M18` exists to
/// catch (`T2` step 8).
const MOUNT_TABLE: &str = "/proc/self/mountinfo";
/// `mountinfo`'s mount-point field, zero-indexed.
const MOUNT_POINT_FIELD: usize = 4;

/// The synthesized `[capsule]` table's two required bounds. Small, because
/// every payload the suite runs is a shell one-liner and the wall bound is a
/// containment mechanism rather than a budget (`EX-11`).
const FIXTURE_TIMEOUT_SECONDS: u64 = 120;
const FIXTURE_FILE_SIZE_CAP_MIB: u64 = 64;

/// The suite's self-contained control plane, built once trusted-side before any
/// row runs.
///
/// **Everything it names lives under [`Fixture::root`]** (invariant 7,
/// `VA-2`): the operator's repository, `.doctrine/`, credentials and any export
/// a real transaction could adopt are named by no arm and bound under none.
/// `provision` reads `[capsule]` from `project_root`'s working tree, so a
/// fixture with its own repository is what keeps the suite from testing the
/// operator's configuration instead of the backend's enforcement (`EX-3`).
#[derive(Debug)]
pub(crate) struct Fixture {
    /// Real disk, never tmpfs (`DEC-156`): `sec-5`'s probe must read real
    /// available space, and a resource observation on tmpfs would measure the
    /// mount's size rather than the disk's. `Drop` removes it.
    root: TempRoot,
    /// A self-contained control-plane root: a git repository that supplies the
    /// contracted base, and a `.doctrine/doctrine.toml` carrying the
    /// synthesized `[capsule]` table. `provision` reads this, never the
    /// operator's project.
    project_root: PathBuf,
    base: AcceptedBase,
    /// The host regions this fixture's placements may never reach. Names
    /// `project_root`, its `.doctrine/`, and `capsule_root` — so the fixture's
    /// own canonical repository stands in for the real one, which is never
    /// referenced by any arm.
    scopes: ForbiddenScopes,
    capsule_root: PathBuf,
    /// Row 3's targets. Deliberately **not** members of `scopes`, so the row's
    /// control is a lawful widening.
    decoy_credential: PathBuf,
    decoy_repository: PathBuf,
    /// Row 9's targets: a readable decoy the row writes through, and this
    /// fixture's **own** export — built for this run, adopted by nothing else,
    /// so the control arm's writes cannot reach an export a real transaction
    /// shares.
    decoy_readable_input: PathBuf,
    own_export: PathBuf,
    /// Table C's second capacity row: a path on a filesystem other than the
    /// one `capsule_root` is on, chosen from the host's mounts at build time.
    /// `None` where the host offers no second filesystem, which makes that row
    /// report *skipped* naming the reason rather than passing quietly.
    second_filesystem: Option<PathBuf>,
    /// Row 4's target, and row 2's.
    decoy_undeclared: PathBuf,
    decoy_executable: PathBuf,
    /// Row 10's readable decoy, and the only named member of
    /// [`InheritableDecoys`]. Its own file rather than a share of row 4's, so a
    /// later edit to either row cannot silently change the other's target.
    decoy_descriptor: PathBuf,
    /// Trusted-side, row 5's target.
    listener: TcpListener,
    /// The harness's **own** session, read at build time.
    ///
    /// `EX-12` asks the fixture to record the capsule's sid before the arm runs
    /// and it cannot — the session does not exist until bwrap creates it inside
    /// the child (`F-3`). What *can* be recorded beforehand is this, and it is
    /// what makes a foreign session identifiable afterwards: the sweep refuses
    /// to signal it, so a bug that recorded the harness's own sid kills the
    /// suite's own process tree instead of quietly doing nothing.
    ///
    /// `None` on a host whose `/proc` did not answer. The sweep then signals
    /// nothing, because a sweep that cannot tell its own session from a
    /// capsule's is a sweep that must not fire.
    own_session: Option<SessionId>,
    /// Every session a capsule of this run was observed to lead.
    ///
    /// Appended by the pid seam while a capsule is alive, drained on the way
    /// out. `RefCell` because the fixture is shared immutably across both arms
    /// of a row and the observation is the one thing an arm gives *back* to it.
    observed_sessions: RefCell<Vec<SessionId>>,
}

impl Fixture {
    /// Build the whole control plane, in `T2`'s order.
    pub(crate) fn new(host: &dyn HostFacts) -> Result<Self, FixtureFault> {
        let root = TempRoot::new(host)?;

        let project_root = root.path().join(PROJECT_LEAF);
        let capsule_root = root.path().join(CAPSULES_LEAF);
        let decoys = root.path().join(DECOYS_LEAF);
        for directory in [&project_root, &capsule_root, &decoys] {
            make_directory(directory)?;
        }

        // The decoys, before the repository: the synthesized table declares the
        // readable one, and `provision` probes every declared entry for
        // existence at step 2.
        let decoy_credential = decoys.join(DECOY_CREDENTIAL_LEAF);
        let decoy_repository = decoys.join(DECOY_REPOSITORY_LEAF);
        let decoy_readable_input = decoys.join(DECOY_READABLE_INPUT_LEAF);
        let decoy_undeclared = decoys.join(DECOY_UNDECLARED_LEAF);
        let decoy_executable = decoys.join(DECOY_EXECUTABLE_LEAF);
        let decoy_descriptor = decoys.join(DECOY_DESCRIPTOR_LEAF);
        make_directory(&decoy_readable_input)?;
        make_directory(&decoy_repository)?;
        git(&decoy_repository, &["init", "--quiet"])?;
        write_file(&decoy_credential, DECOY_CREDENTIAL_BODY)?;
        write_file(&decoy_undeclared, DECOY_UNDECLARED_BODY)?;
        write_file(&decoy_executable, DECOY_EXECUTABLE_BODY)?;
        write_file(&decoy_descriptor, DECOY_DESCRIPTOR_BODY)?;
        make_executable(&decoy_executable)?;

        let readable_roots = system_readable_roots(host, root.path());
        if readable_roots.is_empty() {
            return Err(FixtureFault::NoReadableRoots);
        }

        let base = initialise_project(
            &project_root,
            &capsule_config_document(&capsule_root, &readable_roots),
        )?;

        Ok(Self {
            own_export: capsule_root.join(EXPORT_DIRECTORY_LEAF),
            second_filesystem: std::fs::read_to_string(MOUNT_TABLE)
                .ok()
                .and_then(|table| second_filesystem(&capsule_root, root.path(), &table)),
            scopes: ForbiddenScopes::new(
                project_root.clone(),
                project_root.join(CONTROL_PLANE_STATE_LEAF),
                capsule_root.clone(),
                Vec::new(),
            ),
            listener: TcpListener::bind(LOOPBACK_ANY_PORT)
                .map_err(|error| fixture_io(Path::new(LOOPBACK_ANY_PORT), &error))?,
            own_session: own_session(),
            observed_sessions: RefCell::new(Vec::new()),
            root,
            project_root,
            base,
            capsule_root,
            decoy_credential,
            decoy_repository,
            decoy_readable_input,
            decoy_undeclared,
            decoy_executable,
            decoy_descriptor,
        })
    }

    pub(crate) fn root(&self) -> &Path {
        self.root.path()
    }
    pub(crate) fn project_root(&self) -> &Path {
        &self.project_root
    }
    pub(crate) const fn base(&self) -> &AcceptedBase {
        &self.base
    }
    pub(crate) const fn scopes(&self) -> &ForbiddenScopes {
        &self.scopes
    }
    pub(crate) fn capsule_root(&self) -> &Path {
        &self.capsule_root
    }
    pub(crate) fn decoy_credential(&self) -> &Path {
        &self.decoy_credential
    }
    pub(crate) fn decoy_repository(&self) -> &Path {
        &self.decoy_repository
    }
    pub(crate) fn decoy_readable_input(&self) -> &Path {
        &self.decoy_readable_input
    }
    pub(crate) fn own_export(&self) -> &Path {
        &self.own_export
    }
    pub(crate) fn second_filesystem(&self) -> Option<&Path> {
        self.second_filesystem.as_deref()
    }
    pub(crate) fn decoy_undeclared(&self) -> &Path {
        &self.decoy_undeclared
    }
    pub(crate) fn decoy_executable(&self) -> &Path {
        &self.decoy_executable
    }
    pub(crate) fn decoy_descriptor(&self) -> &Path {
        &self.decoy_descriptor
    }

    /// Open row 10's three decoy descriptors, inheritable across `exec`
    /// (`EX-13`).
    ///
    /// **A fresh set per arm, never once per fixture.** The backend's sweep
    /// marks the *parent's* descriptors close-on-exec and that change is
    /// permanent, so row 10's confining probe arm closes any long-lived set and
    /// the control arm that follows it would find nothing left to leak — a row
    /// that passes for no reason (`F-26`).
    pub(crate) fn inheritable_decoys(&self) -> Result<InheritableDecoys, FixtureFault> {
        let readable = rustix::fs::open(
            &self.decoy_descriptor,
            OFlags::RDONLY,
            rustix::fs::Mode::empty(),
        )
        .map_err(|error| fixture_io(&self.decoy_descriptor, &error))?;

        // Never linked into any directory: `O_TMPFILE` names the directory the
        // file's blocks live in — the fixture's own root — and hands back the
        // only handle there will ever be. So the control arm's mutation is
        // unreachable by name and dies with the descriptor, and this phase
        // still introduces no unlink and no delete primitive (invariant 8,
        // `VA-3`).
        let root = self.root.path();
        let writable = rustix::fs::open(
            root,
            OFlags::WRONLY | OFlags::TMPFILE,
            rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
        )
        .map_err(|error| fixture_io(root, &error))?;

        let pair_label = Path::new(DECOY_SOCKET_PAIR_LABEL);
        let (near, peer) = UnixStream::pair().map_err(|error| fixture_io(pair_label, &error))?;
        let socket = OwnedFd::from(near);
        make_inheritable(&socket).map_err(|error| fixture_io(pair_label, &error))?;

        Ok(InheritableDecoys {
            readable,
            writable,
            socket,
            peer,
        })
    }
    pub(crate) const fn listener(&self) -> &TcpListener {
        &self.listener
    }

    pub(crate) const fn own_session(&self) -> Option<SessionId> {
        self.own_session
    }

    /// Record the session of a capsule process seen alive (`EX-12`, `D3`).
    ///
    /// The harness's own session is **refused**, not merely skipped by the
    /// sweep. A defect that fed this the wrong pid — the wrapper's, or the
    /// trusted side's — would otherwise arm the sweep against the suite's own
    /// process tree, and the observable failure would be the test runner dying
    /// rather than a leaked capsule surviving.
    pub(crate) fn note_capsule_session(&self, capsule: HostPid) {
        if let Some(session) = session_of(capsule) {
            self.note_session(session);
        }
    }

    /// The recording rule alone, over a session already read.
    ///
    /// Split from [`Self::note_capsule_session`] so both halves are testable
    /// without spawning: the pid→session read needs a live process, and the
    /// refusal-and-dedup rule needs a session id with no members at all.
    fn note_session(&self, session: SessionId) {
        if Some(session) == self.own_session {
            return;
        }
        let mut sessions = self.observed_sessions.borrow_mut();
        if !sessions.contains(&session) {
            sessions.push(session);
        }
    }

    /// Signal every live member of every session this run created (`EX-12`).
    ///
    /// **By session, never by process group.** `RV-346` `F-27` strengthened row
    /// 7's payload to a descendant that leaves the original *process group*
    /// precisely so a process-group-only backend cannot pass — so a
    /// process-group kill here could not reap the survivor its own control arm
    /// creates. When a row's payload is strengthened, its containment is part of
    /// the payload.
    ///
    /// Drains: sweeping twice is lawful and the second is a no-op, which is what
    /// lets this be called after every row *and* from `Drop` without the second
    /// call signalling a pid the kernel has since recycled.
    pub(crate) fn sweep_observed_sessions(&self) -> Vec<SessionId> {
        let Some(own) = self.own_session else {
            return Vec::new();
        };
        let swept: Vec<SessionId> = self.observed_sessions.borrow_mut().drain(..).collect();
        for session in &swept {
            kill_session(*session, own);
        }
        swept
    }
}

/// Row 10's three decoy descriptors: the **only** descriptors this suite ever
/// leaves inheritable across `exec` (`EX-13`).
///
/// [`PropertyRemoval::DescriptorsClosed`] removes the backend's parent-side
/// sweep. A sweep with nothing to close is a removal that changes nothing, so
/// the row needs descriptors that are already inheritable when the arm spawns —
/// and Rust opens its own files `O_CLOEXEC`, which is the trap PHASE-05 `VT-4`
/// records. Hence `rustix::fs::open` without [`OFlags::CLOEXEC`] for the two
/// files, and [`make_inheritable`] for the socket end `UnixStream::pair` opened
/// closed.
///
/// **No descriptor the trusted side holds for real is ever made inheritable.**
/// The pair's far end is held here, for real, and keeps close-on-exec.
///
/// Closing all three is `Drop`'s, so a set outlives exactly the arm that holds
/// it.
#[derive(Debug)]
pub(crate) struct InheritableDecoys {
    /// A named, readable file under the fixture's own root.
    readable: OwnedFd,
    /// A write-only file that was never linked into any directory. Named by
    /// nothing, so a capsule that inherits it can write but cannot reach what it
    /// wrote, and the blocks are reclaimed when the last handle closes.
    writable: OwnedFd,
    /// One end of a socket pair — a descriptor over no filesystem at all, which
    /// is the third *kind* of thing an inherited descriptor can be.
    socket: OwnedFd,
    /// The far end, held so the pair stays a pair for as long as the decoys
    /// exist. Deliberately untouched by [`make_inheritable`].
    peer: UnixStream,
}

impl InheritableDecoys {
    /// How many descriptors above the standard streams a capsule inherits when
    /// the sweep is skipped — what row 10's payload counts in `/proc/self/fd`.
    pub(crate) const COUNT: usize = 3;

    /// The three, in the order this type documents them.
    pub(crate) fn descriptors(&self) -> [BorrowedFd<'_>; Self::COUNT] {
        [
            self.readable.as_fd(),
            self.writable.as_fd(),
            self.socket.as_fd(),
        ]
    }

    /// The far end of the socket pair, held for real and close-on-exec.
    pub(crate) fn retained_peer(&self) -> BorrowedFd<'_> {
        self.peer.as_fd()
    }
}

/// Clear close-on-exec, so a descriptor the fixture chose survives the capsule's
/// `exec` — the inverse of the backend's parent-side sweep.
///
/// Applied to [`InheritableDecoys`]' three and to nothing else. A caller that
/// widened that set would be widening invariant 12's only exception.
fn make_inheritable<Fd: AsFd>(fd: Fd) -> Result<(), rustix::io::Errno> {
    let flags = fcntl_getfd(&fd)?;
    fcntl_setfd(&fd, flags.difference(FdFlags::CLOEXEC))
}

/// Whether `fd` survives an `exec`.
fn is_inheritable<Fd: AsFd>(fd: Fd) -> bool {
    matches!(fcntl_getfd(fd), Ok(flags) if !flags.contains(FdFlags::CLOEXEC))
}

/// [`FixtureFault::Io`] over anything that displays, because four call sites
/// would otherwise each spell the same closure.
fn fixture_io(path: &Path, error: &dyn std::fmt::Display) -> FixtureFault {
    FixtureFault::Io {
        path: path.to_path_buf(),
        detail: error.to_string(),
    }
}

/// Signal every live process whose session is `session`.
///
/// `own` is passed rather than read here so the refusal is a *parameter* of the
/// operation: there is no route to this function that has not already had to
/// name the session it must not touch.
///
/// A pid that vanishes between the enumeration and the signal is the normal
/// case, not an error — the point of the sweep is that these processes are
/// exiting or should be — so a failed signal is discarded.
fn kill_session(session: SessionId, own: SessionId) {
    if session == own {
        return;
    }
    for facts in process_table() {
        if facts.session != session {
            continue;
        }
        let Ok(pid) = rustix::process::Pid::from_raw(facts.pid.0).ok_or(()) else {
            continue;
        };
        let _signalled = rustix::process::kill_process(pid, rustix::process::Signal::KILL);
    }
}

/// The belt behind `run_row`'s sweep (`EX-12`, `VA-1`).
///
/// `run_row` sweeps after every row, which is where a survivor is *supposed* to
/// die. This catches the run that never reached that line — a panic in an arm,
/// an early return from a claim — and it runs **before** [`TempRoot`]'s `Drop`
/// removes the tree, because field drops follow the type's own `Drop`.
///
/// This kills processes and removes nothing: invariant 8's no-delete rule is
/// about capsules on disk, and cleanup there is still `TempRoot`'s alone.
impl Drop for Fixture {
    fn drop(&mut self) {
        let _swept = self.sweep_observed_sessions();
    }
}

/// `.doctrine`, spelled here because `provision`'s constant is private to that
/// module. `doctrine::DOCTRINE_TOML` supplies the document's own path, so only
/// the directory leaf is duplicated.
const CONTROL_PLANE_STATE_LEAF: &str = ".doctrine";

const DECOY_CREDENTIAL_BODY: &str = "decoy-token: not-a-real-credential\n";
const DECOY_UNDECLARED_BODY: &str = "declared to no placement\n";
const DECOY_EXECUTABLE_BODY: &str = "#!/bin/sh\necho decoy\n";
const DECOY_DESCRIPTOR_BODY: &str = "readable through an inherited descriptor\n";

/// What a socket-pair failure is reported *at*. [`FixtureFault::Io`] carries a
/// path and a socket pair has none; naming it is honest where reusing the run
/// root would blame a directory that is fine.
const DECOY_SOCKET_PAIR_LABEL: &str = "<decoy socket pair>";

/// What the kernel appends to `/proc/<pid>/fd/<n>`'s target once the file has no
/// remaining link — how the write-only decoy's namelessness is read back.
const DELETED_SUFFIX: &str = " (deleted)";
const EXECUTABLE_MODE: u32 = 0o755;

/// The synthesized control-plane document: a `[capsule]` table over the
/// fixture's own paths, and the `[interpretation]` policy `provision` reads
/// back from the contracted base.
///
/// `readable-roots` **must** include a shell, or every payload is
/// `NotExecutable` and the run is indeterminate for a reason it never names
/// (`EX-3`, and `verify`'s own shell precheck says the same thing one level up).
/// No `closure-roots`: declaring them would require a closure resolver, and the
/// resolver is admitted against the policy (`sec-3` step 6) — a second moving
/// part in a fixture whose job is to be boring.
fn capsule_config_document(capsule_root: &Path, readable_roots: &[PathBuf]) -> String {
    let readable = readable_roots
        .iter()
        .map(|root| format!("\"{}\"", root.display()))
        .collect::<Vec<String>>()
        .join(", ");

    format!(
        "[capsule]\n\
         root = \"{root}\"\n\
         readable-roots = [{readable}]\n\
         execution-timeout-seconds = {FIXTURE_TIMEOUT_SECONDS}\n\
         file-size-cap-mib = {FIXTURE_FILE_SIZE_CAP_MIB}\n\
         \n\
         [interpretation]\n\
         schema = 1\n\
         {EMPTY_FORBIDDEN_EXECUTABLES}\n\
         interpreted_paths = []\n\
         \n\
         [[interpretation.verification]]\n\
         argv = [\"true\"]\n",
        root = capsule_root.display(),
    )
}

/// The fixture's own repository, and the base it contracts.
///
/// **One object unreachable from the base, deliberately** (`T2` step 2): the
/// second commit lands on its own branch and the working tree is left detached
/// at the base. Without it the repository's object set *equals* the export's,
/// and `the_clones_object_set_is_exactly_the_exports` passes under a clone that
/// copied everything — which is the defect the claim exists to catch.
fn initialise_project(project_root: &Path, document: &str) -> Result<AcceptedBase, FixtureFault> {
    git(project_root, &["init", "--quiet"])?;
    git(
        project_root,
        &["config", "user.name", FIXTURE_IDENTITY_NAME],
    )?;
    git(
        project_root,
        &["config", "user.email", FIXTURE_IDENTITY_EMAIL],
    )?;

    write_file(&project_root.join(DOCTRINE_TOML), document)?;
    git(project_root, &["add", "."])?;
    git(project_root, &["commit", "--quiet", "-m", "base"])?;
    let base = AcceptedBase::new(git(project_root, &["rev-parse", "HEAD"])?);

    git(
        project_root,
        &["switch", "--quiet", "-c", UNREACHABLE_BRANCH],
    )?;
    write_file(&project_root.join(UNREACHABLE_LEAF), DECOY_UNDECLARED_BODY)?;
    git(project_root, &["add", "."])?;
    git(project_root, &["commit", "--quiet", "-m", "unreachable"])?;
    git(
        project_root,
        &["switch", "--quiet", "--detach", base.as_str()],
    )?;

    Ok(base)
}

/// One trusted-side Git invocation, returning its trimmed stdout.
fn git(directory: &Path, arguments: &[&str]) -> Result<String, FixtureFault> {
    let argv = || {
        std::iter::once(GIT.to_owned())
            .chain(arguments.iter().map(|word| (*word).to_owned()))
            .collect::<Vec<String>>()
    };
    let output = Command::new(GIT)
        .arg("-C")
        .arg(directory)
        .args(arguments)
        .output()
        .map_err(|error| FixtureFault::Git {
            argv: argv(),
            detail: error.to_string(),
        })?;
    if !output.status.success() {
        return Err(FixtureFault::Git {
            argv: argv(),
            detail: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn make_directory(path: &Path) -> Result<(), FixtureFault> {
    std::fs::create_dir_all(path).map_err(|error| fixture_io(path, &error))
}

/// `File::create` plus `write_all`, because `std::fs::write` is banned
/// crate-wide by `clippy.toml`'s `disallowed-methods`.
fn write_file(path: &Path, contents: &str) -> Result<(), FixtureFault> {
    if let Some(parent) = path.parent() {
        make_directory(parent)?;
    }
    let mut file = File::create(path).map_err(|error| fixture_io(path, &error))?;
    file.write_all(contents.as_bytes())
        .map_err(|error| fixture_io(path, &error))
}

fn make_executable(path: &Path) -> Result<(), FixtureFault> {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(EXECUTABLE_MODE))
        .map_err(|error| fixture_io(path, &error))
}

/// The readable roots the fixture declares, **derived from the host** rather
/// than hardcoded.
///
/// One rule: the *top-level* ancestor of the resolved shell and of every
/// resolved host `PATH` entry — `/nix/store/…/bin` yields `/nix`, `/usr/bin`
/// yields `/usr`. The top-level ancestor and not the entry itself, because a
/// dynamically linked executable needs its loader and libraries, which on a
/// store-based host live under sibling directories of the same top level; a
/// measurement on this host showed `git` runs under `--ro-bind /nix/store` and
/// cannot under its own `bin` directory alone.
///
/// **Any candidate containing operator state is dropped** (invariant 7): the
/// fixture root, `$HOME`, and the working directory. That is what keeps `/home`
/// — which is the top-level ancestor of a `PATH` entry on most hosts — from
/// binding the operator's credentials into a capsule.
///
/// Top-level ancestors are also pairwise non-overlapping by construction, which
/// is what keeps `CapsulePlacement::try_new`'s inner-path collision rule
/// satisfied without a second deduplication pass.
fn system_readable_roots(host: &dyn HostFacts, fixture_root: &Path) -> Vec<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(shell) = host.resolve(Path::new(SHELL)) {
        candidates.push(shell);
    }
    if let Some(raw) = host.env_var(CapsuleEnvVar::Path.name()) {
        for entry in std::env::split_paths(&raw) {
            if let Ok(resolved) = host.resolve(&entry) {
                candidates.push(resolved);
            }
        }
    }

    let operator = operator_regions(host, fixture_root);
    let mut roots: Vec<PathBuf> = Vec::new();
    for candidate in candidates {
        let Some(top) = top_level_ancestor(&candidate) else {
            continue;
        };
        if operator.iter().any(|region| region.starts_with(&top)) {
            continue;
        }
        if !roots.contains(&top) {
            roots.push(top);
        }
    }
    roots
}

/// The regions no readable root may contain: the fixture's own state, the
/// operator's home, and wherever the suite was invoked from.
fn operator_regions(host: &dyn HostFacts, fixture_root: &Path) -> Vec<PathBuf> {
    let mut regions = vec![fixture_root.to_path_buf()];
    if let Some(home) = host.env_var(HOME_VARIABLE) {
        regions.push(PathBuf::from(home));
    }
    if let Ok(working_directory) = std::env::current_dir() {
        regions.push(working_directory);
    }
    regions
}

/// `/nix/store/x/bin` → `/nix`. `None` for a relative path or for `/` itself.
fn top_level_ancestor(path: &Path) -> Option<PathBuf> {
    let mut components = path.components();
    if components.next() != Some(Component::RootDir) {
        return None;
    }
    match components.next() {
        Some(Component::Normal(first)) => Some(Path::new(FILESYSTEM_ROOT).join(first)),
        _ => None,
    }
}

/// A path on a filesystem other than the one `capsule_root` is on, or `None`.
///
/// Selected from the host's **own mount table** rather than by naming `/tmp`:
/// hardcoding a mount is `M18`'s mutation, and a fixture that picks a path on
/// the same filesystem makes `the_capacity_probe_reads_the_filesystem_the_capsule_root_is_on`
/// pass with two identical figures.
///
/// The three conditions, and each rules out a different way of picking wrong:
/// a different `st_dev` (a genuinely different filesystem), a non-zero
/// available figure (`/proc`, `/sys` and friends report nothing), and a figure
/// that differs from the capsule root's (so *the two figures differ* is
/// established at selection, not asserted hopefully at read time). Every probe
/// here is a raw `statvfs`, never `HostFacts::available_bytes` — the selection
/// must not depend on the function the row exists to test.
///
/// **tmpfs is not excluded here**, and that is deliberate rather than an
/// oversight of `DEC-156`. That decision bans tmpfs for the *fixture root*,
/// where a resource observation would measure the mount rather than the disk.
/// This row's claim is only that two paths on two filesystems yield two
/// figures, for which a tmpfs mount is a perfectly good second filesystem —
/// and excluding it was measured to flip this jail to the `None` branch, which
/// would skip the conditional capacity row on the very host the suite is
/// developed against.
///
/// The mount table is a **parameter**, not a read: on every host this suite is
/// developed against the answer is `Some`, so the `None` branch — the one that
/// makes Table C's conditional row report *skipped* — would ship untested if
/// the only way to reach it were to find a host with one filesystem (`A2`).
fn second_filesystem(capsule_root: &Path, fixture_root: &Path, table: &str) -> Option<PathBuf> {
    let here = rustix::fs::stat(capsule_root).ok()?;
    let here_available = available_bytes_of(capsule_root)?;

    mount_points(table).into_iter().find(|point| {
        !point.starts_with(fixture_root)
            && rustix::fs::stat(point).is_ok_and(|there| there.st_dev != here.st_dev)
            && available_bytes_of(point)
                .is_some_and(|available| available != 0 && available != here_available)
    })
}

/// `f_bavail × f_frsize`, the same quantity `SystemHost` reports — computed
/// independently here so the selection does not route through the function
/// under test.
fn available_bytes_of(path: &Path) -> Option<u64> {
    independent_capacity(path).ok().map(|(bytes, _unit)| bytes)
}

/// One independent reading of `path`'s available space, and the allocation unit
/// a tolerance over it is expressed in.
///
/// The crate's **only** independent statvfs arithmetic: the mount-table
/// selection and [`agreed_capacity`] both read through here, so a mutation of
/// the probe cannot be agreed with by either.
fn independent_capacity(path: &Path) -> Result<(u64, u64), String> {
    let stat = rustix::fs::statvfs(path).map_err(|errno| format!("{}: {errno}", path.display()))?;
    let bytes = stat
        .f_bavail
        .checked_mul(stat.f_frsize)
        .ok_or_else(|| format!("{}: the independent figure overflows", path.display()))?;
    Ok((bytes, stat.f_frsize))
}

/// The mount points named by `/proc/self/mountinfo`, in the kernel's order.
///
/// Pure over the file's text so it can be asserted against a hand-built table.
/// Field 4 is the mount point and its whitespace is octal-escaped, so a mount
/// under a directory with a space in its name is decoded rather than truncated.
fn mount_points(table: &str) -> Vec<PathBuf> {
    table
        .lines()
        .filter_map(|line| line.split_whitespace().nth(MOUNT_POINT_FIELD))
        .map(|field| PathBuf::from(decode_mount_field(field)))
        .collect()
}

/// `mountinfo`'s octal escapes: space, tab, newline and backslash.
fn decode_mount_field(field: &str) -> String {
    let mut decoded = String::with_capacity(field.len());
    let mut rest = field;
    while let Some(at) = rest.find('\\') {
        let (before, escaped) = rest.split_at(at);
        decoded.push_str(before);
        if let Some(character) = escaped.get(..4).and_then(decode_octal_escape) {
            decoded.push(character);
            rest = escaped.get(4..).unwrap_or_default();
        } else {
            decoded.push('\\');
            rest = escaped.get(1..).unwrap_or_default();
        }
    }
    decoded.push_str(rest);
    decoded
}

fn decode_octal_escape(escape: &str) -> Option<char> {
    let digits = escape.strip_prefix('\\')?;
    let value = u32::from_str_radix(digits, 8).ok()?;
    char::from_u32(value)
}

// ---------------------------------------------------------------------------
// `BubblewrapBackend`'s admission instrumentation (`EX-8`, `EX-9`, `EX-18`)
// ---------------------------------------------------------------------------

/// The trusted side's end of row 12's descriptors (`D4`).
///
/// [`PropertyRemoval::StdioOwned`] carries no payload, so the descriptors reach
/// the profile by this road rather than through the removal — and
/// `Stdio::inherit()` is not that road (`R3`): it hands the capsule the
/// harness's *own* descriptor 1, which loses the observation the control arm
/// exists to make, so every arm goes indeterminate and the row can never be
/// proven.
///
/// Both ends are created here, trusted-side, and neither is a descriptor the
/// harness holds for real. `UnixStream::pair` rather than `std::io::pipe`
/// because the latter is 1.87 and the MSRV is 1.85 (`C7`).
///
/// **Divergence from `D4`, recorded as PHASE-08 `F-14`.** `D4` says descriptor 0
/// is "a decoy file the fixture opened"; [`ConformanceBackend::execute_weakened`]
/// receives a placement and an execution and no fixture, so there is no channel
/// by which a fixture-opened file could arrive, and the two routes that would
/// build one — a descriptor field on `Execution`, a second `CapsuleStdio`
/// variant — are exactly what `S5` forbids. A socket pair carrying a decoy body
/// meets the substance: descriptor 0 is a trusted-side descriptor the capsule
/// inherits and reads real bytes from, against a probe arm whose descriptor 0 is
/// an empty parent-owned endpoint. Nothing is written to disk, so `VA-1`'s write
/// half and `VA-3`'s no-delete rule are both untouched.
#[derive(Debug)]
struct OwnedStdio {
    input: OwnedFd,
    output: OwnedFd,
    errors: OwnedFd,
}

/// What a capsule reads on descriptor 0 under row 12's control arm, and never
/// under its probe arm.
const STDIO_DECOY_INPUT: &str = "TRUSTED-SIDE-DECOY-INPUT\n";

impl OwnedStdio {
    /// The three endpoints, and the trusted side's capture end for descriptors 1
    /// and 2.
    ///
    /// The decoy body is written and its end dropped before the run, so the
    /// capsule reads it and then sees end-of-file — a capsule blocking forever
    /// on descriptor 0 would be a containment failure of this function's own
    /// making.
    fn opened() -> Result<(Self, UnixStream), BackendError> {
        let (mut source, input) = UnixStream::pair().map_err(|error| mechanism_failed(&error))?;
        source
            .write_all(STDIO_DECOY_INPUT.as_bytes())
            .map_err(|error| mechanism_failed(&error))?;
        drop(source);

        let (capture, output) = UnixStream::pair().map_err(|error| mechanism_failed(&error))?;
        let errors = output
            .try_clone()
            .map_err(|error| mechanism_failed(&error))?;

        Ok((
            Self {
                input: OwnedFd::from(input),
                output: OwnedFd::from(output),
                errors: OwnedFd::from(errors),
            },
            capture,
        ))
    }
}

/// The exhaustive mapping from the property-shaped vocabulary onto
/// `bubblewrap.rs`'s flag-shaped one (`EX-8`).
///
/// Exhaustive by construction: widening [`PropertyRemoval`] fails to compile
/// here, which is the whole reason the vocabulary is an enum rather than a list
/// of names. Nine variants, ten removals — [`Bound`] is where the tenth lives.
///
/// **Which of these deltas are measured, and which are reasoned (`EX-18`).**
/// Three are measured. [`PropertyRemoval::Teardown`] by `EVD-013` — the
/// pid-namespace × `--die-with-parent` 2×2 that settles row 7's control, and it
/// measures nothing else. [`PropertyRemoval::MappedIdentity`] and
/// [`AuthorityGrant::AllCapabilities`] by `EVD-014`, which is also the record of
/// the delta they replaced failing. `EVD-013`'s adjacent fact — that bubblewrap
/// has no `--share-pid` — tells us how
/// [`PropertyRemoval::ProcessVisibility`]'s delta must be *expressed*, which is
/// **not** the same as having seen that delta produce its row's control failure.
/// **Every other delta below is reasoned, and this design cites no measurement
/// for any of them.** A caption claiming otherwise is what `EX-18` exists to
/// correct; measuring the rest is a phase obligation, and a delta that turns out
/// not to produce its row's control failure means the row is wrong rather than
/// that the measurement is inconvenient.
fn weakening_for(removal: PropertyRemoval, stdio: Option<OwnedStdio>) -> Weakening {
    match removal {
        PropertyRemoval::WorkingDirectory => Weakening::WorkingDirectory,
        PropertyRemoval::Teardown => Weakening::Teardown,
        PropertyRemoval::ProcessVisibility => Weakening::ProcessVisibility,
        PropertyRemoval::ResourceBound(Bound::FileSize) => Weakening::FileSizeBound,
        PropertyRemoval::ResourceBound(Bound::Wall) => Weakening::WallBound,
        PropertyRemoval::InputsWritable => Weakening::InputsWritable,
        PropertyRemoval::DescriptorsClosed => Weakening::Descriptors,
        PropertyRemoval::EnvCleared => Weakening::EnvironmentCleared,
        PropertyRemoval::StdioOwned => match stdio {
            Some(OwnedStdio {
                input,
                output,
                errors,
            }) => Weakening::StdioOwned {
                input,
                output,
                errors,
            },
            // Unreachable through `execute_weakened`, which opens the endpoints
            // for exactly this removal. Falling back to the *confining*
            // descriptors rather than to some other axis is the fail-closed
            // reading: the control arm then shows the property still holding and
            // the row reports `Unproven`, which is the honest verdict for a
            // control that was never built.
            None => Weakening::Descriptors,
        },
        PropertyRemoval::MappedIdentity => Weakening::MappedIdentity,
    }
}

/// The same mapping for the one control that **grants** (`EX-18`).
///
/// Measured, by `EVD-014`: `--cap-add ALL` under `--unshare-all` returned
/// `CapInh`/`CapPrm`/`CapEff`/`CapBnd` all `000001ffffffffff` against the probe
/// arm's all-zero, exiting clean — capabilities inside the capsule's *own* user
/// namespace, which is exactly the threat invariant 15 names.
const fn weakening_granting(grant: AuthorityGrant) -> Weakening {
    match grant {
        AuthorityGrant::AllCapabilities => Weakening::AllCapabilities,
    }
}

/// The Linux backend seeks admission, so it owes the suite its controls
/// (`DEC-156`).
///
/// Every method here is [`BubblewrapBackend::run`] under a different profile —
/// *the same code the production path runs* — which is what `D2` buys: there is
/// no second implementation of the confinement profile to drift from this one.
impl ConformanceBackend for BubblewrapBackend<'_> {
    fn as_capsule_backend(&self) -> &dyn CapsuleBackend {
        self
    }

    fn execute_weakened(
        &self,
        placement: &CapsulePlacement,
        execution: &Execution,
        removal: PropertyRemoval,
    ) -> Result<Observation, BackendError> {
        let owned = match removal {
            PropertyRemoval::StdioOwned => Some(OwnedStdio::opened()?),
            _ => None,
        };
        let (stdio, capture) = match owned {
            Some((stdio, capture)) => (Some(stdio), Some(capture)),
            None => (None, None),
        };

        let profile = WeakenedProfile::weakened(weakening_for(removal, stdio));
        let mut observation = self.run(placement, execution, &profile)?;

        // The capsule's output went to descriptors this side owns, so `run` saw
        // nothing to capture. Read it here — and only after dropping the profile,
        // which holds the write ends: a socket pair reports end-of-file when its
        // peer is fully closed, and this side is one of the peers.
        drop(profile);
        if let Some(mut capture) = capture {
            let mut captured = Vec::new();
            capture
                .read_to_end(&mut captured)
                .map_err(|error| mechanism_failed(&error))?;
            observation.stdout = captured;
        }
        Ok(observation)
    }

    fn execute_granted(
        &self,
        placement: &CapsulePlacement,
        execution: &Execution,
        grant: AuthorityGrant,
    ) -> Result<Observation, BackendError> {
        self.run(
            placement,
            execution,
            &WeakenedProfile::weakened(weakening_granting(grant)),
        )
    }

    fn execute_observed(
        &self,
        placement: &CapsulePlacement,
        execution: &Execution,
        observer: &dyn Fn(HostPid),
    ) -> Result<Observation, BackendError> {
        // The backend's seam hands over the **immediate child**, which under the
        // wall bound is `timeout(1)` and never the subject. The descent to the
        // capsule's own top-level process happens here, trusted-side, inside the
        // window where the child is spawned and not yet waited on.
        //
        // A descent that finds nothing does **not** call back, and
        // `classify_concurrent` reads that as `Indeterminacy::NoLiveness`. That
        // is the honest outcome: a pid we could not establish is not evidence,
        // and calling back with the wrapper's pid would be worse than silence.
        let relay = |pid: i32| {
            if let Some(capsule) = observed_capsule_process(HostPid(pid)) {
                observer(capsule);
            }
        };
        self.run(
            placement,
            execution,
            &WeakenedProfile::confining().observed_by(&relay),
        )
    }
}

// ---------------------------------------------------------------------------
// The pid seam and the session it names (`T5`, `EX-7`, `EX-12`, `D3`)
// ---------------------------------------------------------------------------

const PROC_DIRECTORY: &str = "/proc";
const PROC_SELF: &str = "self";
const STAT_LEAF: &str = "stat";
/// `/proc/<pid>/stat` fields, counted **after** the parenthesised `comm`. That
/// field may itself contain spaces and parentheses — `sh (deleted)` is a real
/// process name — so the split is on the **last** `)` in the line and never on
/// whitespace from the left.
const STAT_STATE_FIELD: usize = 0;
const STAT_PARENT_FIELD: usize = 1;
const STAT_SESSION_FIELD: usize = 3;
/// The process state of a process that has exited and not yet been waited on.
const ZOMBIE_STATE: &str = "Z";
/// The capsule's top-level process does not exist at the instant its wrapper is
/// spawned, so the descent polls. Half a second at two milliseconds; a capsule
/// that has not reached its own session by then is reported as no liveness
/// rather than guessed at.
const CAPSULE_DISCOVERY_ATTEMPTS: u32 = 250;
const CAPSULE_DISCOVERY_INTERVAL: Duration = Duration::from_millis(2);

/// A session as the host sees it.
///
/// Distinct from [`HostPid`] because `EX-12`'s containment turns on the
/// difference: a **process-group** kill cannot reap the descendant row 7's
/// payload deliberately detaches into its own session, and a bare `i32` makes
/// the two indistinguishable at the point where the mistake is cheap to make.
/// `M13` mutates the session field index into the process-group one; this
/// newtype is why that mutation has something to red against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SessionId(pub(crate) i32);

/// The three numbers the descent needs from one `/proc` entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProcessFacts {
    pub(crate) pid: HostPid,
    pub(crate) parent: HostPid,
    pub(crate) session: SessionId,
}

/// The capsule's top-level process: the **nearest** descendant of the trusted
/// side's immediate child that leads a session of its own.
///
/// Measured on this host (plan-time, and again here): the tree under the
/// confining profile is `timeout` → `bwrap` → `bwrap`'s sandbox process, and
/// only the last of the three has `session == pid` — bubblewrap's
/// `--new-session` calls `setsid` there, inside the pid namespace, and the
/// kernel reports that session to a host-namespace reader as the leader's host
/// pid. So *leads its own session* identifies the subject without knowing how
/// many wrappers stand between, which is what makes the same rule work under
/// `Weakening::WallBound` (no `timeout`) and `Weakening::ProcessVisibility` (no
/// pid namespace).
///
/// **Nearest, not first found.** Row 7's payload deliberately detaches a
/// descendant into a *second* new session, so a later arm has two candidates and
/// the deeper one is the escapee rather than the subject. Ties break on the
/// lower pid, so the answer is a function of the table rather than of `/proc`'s
/// directory order.
///
/// **A leader, not a member.** Once the top-level process exits, its surviving
/// children keep its session id but none of them *is* the leader, and this
/// answers `None` rather than naming a survivor. That is the right answer: the
/// row wants a pid whose liveness it can reason about, and a departed leader's
/// orphan is not that pid.
///
/// The harness's own session needs no exclusion here and gets none: a process
/// that leads the harness's session predates the child, so it can never be the
/// child's descendant. `own_session` exists for `EX-12`'s sweep (`F-3`), not for
/// this filter — a guard that cannot fire is a guard that cannot be tested.
///
/// Pure: the table is a parameter. `A2`'s lesson — the branch this host never
/// takes ships untested unless it can be forced — applies to every shape of
/// process tree, and none of them can be conjured on demand.
fn capsule_session_leader(child: HostPid, table: &[ProcessFacts]) -> Option<HostPid> {
    table
        .iter()
        .filter(|facts| facts.session.0 == facts.pid.0)
        .filter_map(|facts| depth_from(facts.pid, child, table).map(|depth| (depth, facts.pid)))
        .min_by_key(|(depth, pid)| (*depth, pid.0))
        .map(|(_, pid)| pid)
}

/// Parent hops from `pid` up to `ancestor`, or `None` when `ancestor` is not one.
///
/// Bounded by the table's own length, so a `/proc` snapshot torn mid-read into a
/// parent cycle terminates rather than hanging the trusted side.
fn depth_from(pid: HostPid, ancestor: HostPid, table: &[ProcessFacts]) -> Option<u32> {
    let mut current = pid;
    for hop in 0..table.len() {
        if current == ancestor {
            return u32::try_from(hop).ok();
        }
        let facts = table.iter().find(|entry| entry.pid == current)?;
        if facts.parent.0 == 0 {
            return None;
        }
        current = facts.parent;
    }
    None
}

/// Poll `/proc` until the capsule's top-level process exists, or give up.
fn observed_capsule_process(child: HostPid) -> Option<HostPid> {
    for _ in 0..CAPSULE_DISCOVERY_ATTEMPTS {
        if let Some(capsule) = capsule_session_leader(child, &process_table()) {
            return Some(capsule);
        }
        std::thread::sleep(CAPSULE_DISCOVERY_INTERVAL);
    }
    None
}

/// Every process the trusted side can read, as of one sweep.
///
/// Unreadable and vanishing entries are dropped rather than refused: `/proc` is
/// a moving target and a process that exits mid-sweep is not an error.
pub(crate) fn process_table() -> Vec<ProcessFacts> {
    let Ok(entries) = std::fs::read_dir(PROC_DIRECTORY) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let pid: i32 = entry.file_name().to_str()?.parse().ok()?;
            let facts = stat_of(&entry.path().join(STAT_LEAF))?;
            Some(ProcessFacts {
                pid: HostPid(pid),
                parent: facts.parent,
                session: facts.session,
            })
        })
        .collect()
}

/// The session a live process belongs to — `EX-12`'s sweep predicate.
pub(crate) fn session_of(pid: HostPid) -> Option<SessionId> {
    stat_of(&stat_path(pid)).map(|facts| facts.session)
}

/// Whether the capsule named by `pid` is still running its payload — row B5's
/// window (`EX-7`).
///
/// Impure half only. The judgement is [`still_running`], which is where the two
/// ways of getting this wrong are stated and tested.
fn capsule_still_running(pid: HostPid) -> bool {
    stat_of(&stat_path(pid)).is_some_and(|facts| still_running(pid, &facts))
}

/// Is this `/proc` entry still the capsule we named, and still running?
///
/// Two independent things, and dropping either one gives a wrong answer that
/// looks right:
///
/// - **Reaped.** A process that has exited but not been waited on keeps its
///   `/proc` entry, so "the directory exists" is not liveness. Row B5's window
///   is about a payload that is still executing, and a zombie is executing
///   nothing.
/// - **Recycled.** Between naming the pid and asking after it, the kernel may
///   have handed that number to something else. The capsule was identified as a
///   session *leader*, so `session == pid` still holds for it and holds for
///   almost nothing else; a recycled pid fails it.
const fn still_running(pid: HostPid, facts: &StatFacts) -> bool {
    !facts.reaped && facts.session.0 == pid.0
}

fn stat_path(pid: HostPid) -> PathBuf {
    Path::new(PROC_DIRECTORY)
        .join(pid.0.to_string())
        .join(STAT_LEAF)
}

/// The trusted side's **own** session, recorded so a foreign one is
/// identifiable.
///
/// `F-3`: `EX-12` asks the fixture to record the capsule's session before the
/// arm runs, and it cannot — the session does not exist until bubblewrap creates
/// it inside the child. This is the half that *can* be recorded beforehand, and
/// it is the half that makes the other identifiable.
pub(crate) fn own_session() -> Option<SessionId> {
    let path = Path::new(PROC_DIRECTORY).join(PROC_SELF).join(STAT_LEAF);
    stat_of(&path).map(|facts| facts.session)
}

/// What one `/proc/<pid>/stat` line says, past its `comm`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StatFacts {
    /// The process has exited and not yet been waited on.
    reaped: bool,
    parent: HostPid,
    session: SessionId,
}

fn stat_of(path: &Path) -> Option<StatFacts> {
    let text = std::fs::read_to_string(path).ok()?;
    let after_comm = text.rsplit_once(')')?.1;
    let fields: Vec<&str> = after_comm.split_whitespace().collect();
    Some(StatFacts {
        reaped: fields.get(STAT_STATE_FIELD).copied() == Some(ZOMBIE_STATE),
        parent: HostPid(fields.get(STAT_PARENT_FIELD)?.parse().ok()?),
        session: SessionId(fields.get(STAT_SESSION_FIELD)?.parse().ok()?),
    })
}

// ---------------------------------------------------------------------------
// The delta and row vocabulary (`EX-8`, `EX-14`)
// ---------------------------------------------------------------------------

/// How a control arm's placement differs from the probe arm's.
///
/// Exactly one value, so *differs by one thing* is a type rather than a promise
/// (invariant 3).
#[derive(Debug, Clone)]
pub(crate) enum Delta {
    /// The second capsule's placement is rebuilt on the *first* capsule's
    /// `TransactionRoot`. The freshness control.
    ///
    /// It re-points **only the second placement**, never provisioning a second
    /// transaction into the first's root: `sec-3` step 9 creates the transaction
    /// root exclusively and refuses, and
    /// `provision_onto_an_existing_transaction_root_refuses_and_removes_nothing`
    /// asserts the refusal — a control the system refuses to build proves
    /// nothing. Both transactions are still provisioned normally, so the arms
    /// differ by this and nothing else.
    SharedRoot,
    /// The placement is widened by the row's declared readable entries, rebuilt
    /// through `CapsulePlacement::try_new` — that the rebuilt placement is still
    /// lawful is itself evidence. Rows 2, 3, 4.
    Widened(fn(&Fixture) -> Vec<MountedPath>),
    /// The placement's network posture becomes `Permitted`. Row 5.
    NetworkPermitted,
    /// The placement is unchanged; one profile property is removed from the
    /// backend. Rows 6–13 and B5.
    Removed(PropertyRemoval),
    /// The placement is unchanged; the backend *grants* the capsule one
    /// authority it otherwise withholds. Row 14, and the only granting control
    /// in the table.
    ///
    /// A grant rather than a removal because some protections have no off switch
    /// to remove. It is a distinct variant rather than a [`PropertyRemoval`]
    /// member so the asymmetry is visible in the type: [`Delta::Widened`] widens
    /// the *mount set* and a capability grant touches no mount, so reusing it
    /// would make the two indistinguishable in the verdict.
    Granted(AuthorityGrant),
}

/// One row of either admission table.
#[derive(Debug, Clone)]
pub(crate) struct Row {
    pub(crate) id: RowId,
    pub(crate) shape: ArmShape,
    pub(crate) delta: Delta,
}

/// A row of either admission table.
///
/// Table B's rows are freshness axes, not properties, so one enum spans both
/// rather than a [`Property`] key that cannot hold half of what the verdict
/// reports.
#[derive(Debug, Clone, PartialEq, Eq)]
#[expect(
    dead_code,
    reason = "SL-248: `Property` is declared empty at PHASE-07 by EX-14, so `RowId::Property` \
              is uninhabited and cannot be constructed until PHASE-09 lands the first eight \
              variants. At item level rather than on the variant, for the same reason as \
              `Row` above. Self-clears then."
)]
pub(crate) enum RowId {
    /// An enforcement claim of `SPEC-030` § *Platform backend contract*, one per
    /// channel rather than one per clause.
    Property(Property),
    /// One of `REQ-450` criterion 1's five freshness axes.
    Axis(Axis),
}

/// One variant per table A row, **ordered as table A is**.
///
/// Declared empty at this phase: `EX-14` defers membership, and the variants
/// arrive with their rows — eight after PHASE-09, fourteen after PHASE-10.
///
/// ## What the compiler checks here, and what it does not
///
/// [`RowId::Property`] keys the verdict, so a row the suite can construct that
/// this enum cannot name is a **compile error**. That is the one machine-checked
/// projection of table A (`sec-9` `R9`), and it is the whole of it.
///
/// Everything else — that this enum's membership matches the design document's
/// table, that its ordering matches, that no row of the document is missing — is
/// **reader discipline**. No test, comment or doc line in this unit checks the
/// *design document* against anything, and
/// `every_row_id_is_covered_by_exactly_one_table` in particular asserts a
/// property of the **code's own** tables and says nothing whatever about the
/// document's row list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Property {}

/// `REQ-450` criterion 1's five freshness axes. Closed and complete.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Axis {
    Checkout,
    Repository,
    Runtime,
    TemporaryState,
    Process,
}

/// What a row established.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RowVerdict {
    /// The probe held and the control failed. The property is enforced, and the
    /// control licenses the inference that the enforcement is what did it.
    Proven,
    /// The probe arm did not hold: the property is not enforced.
    Violated,
    /// The control arm still held, so removing the property changed nothing and
    /// the probe's result has no established cause.
    Unproven,
    /// Either arm was indeterminate. The row establishes nothing in either
    /// direction, which is not the same as either failing.
    Indeterminate { arm: Which, detail: Indeterminacy },
}

// ---------------------------------------------------------------------------
// The verdict (`EX-15`)
// ---------------------------------------------------------------------------

/// The recorded admission verdict `DEC-156` requires — backend, host and date,
/// so `REQ-459` criterion 3's *independently* has an artefact to point at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AdmissionVerdict {
    pub(crate) backend: BackendId,
    pub(crate) host: HostDescriptor,
    pub(crate) date: String,
    pub(crate) outcome: Admission,
    /// Every row of tables A and B, including the proven ones.
    pub(crate) rows: Vec<(RowId, RowVerdict)>,
    /// Table C. Reported, never admitted on.
    pub(crate) auxiliary: Vec<(Claim, AuxOutcome)>,
}

/// The outcome. There is exactly **one** green path (invariant 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Admission {
    /// Every row in tables A and B is [`RowVerdict::Proven`].
    Admitted,
    NotAdmitted {
        reason: NotAdmitted,
    },
}

/// Why a backend was not admitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NotAdmitted {
    /// `POL-002` facet 3: what is missing and what would satisfy it. Read from
    /// `CapsuleBackend::availability`, or from the suite failing to find a usable
    /// shell, **before any row runs**.
    Unavailable { missing: String, remedy: String },
    /// At least one row was not [`RowVerdict::Proven`].
    Rows,
}

/// A table C claim and the section that owes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Claim {
    pub(crate) section: &'static str,
    pub(crate) name: &'static str,
}

/// What a table C claim showed. Reported, never admitted on — in **either**
/// direction: a failed claim cannot block admission and a skipped one cannot
/// grant it, because [`admission`] is computed from the row list alone and is
/// given no access to these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AuxOutcome {
    Passed,
    Failed(String),
    Skipped(String),
}

/// The host an admission verdict was recorded on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HostDescriptor {
    pub(crate) os: String,
    pub(crate) kernel: String,
    pub(crate) arch: String,
}

/// The **whole** derivation of [`HostDescriptor`], in one function on purpose.
///
/// `verify`'s three parameters are fixed (`EX-2`) and `HostFacts` (`sec-5`)
/// carries no OS, kernel or architecture, so the descriptor has no source at
/// this altitude. This reads `std::env::consts` (compile-time constants) and
/// [`KERNEL_RELEASE`] from disk, falling back to [`UNKNOWN_HOST_FACT`] when the
/// read fails or yields nothing. `std::fs::read_to_string` is not among
/// `clippy.toml`'s disallowed methods, and `conformance` is engine tier where
/// `provision` already reaches disk.
///
/// **Kept behind one name deliberately.** The design-faithful source is
/// `rustix::system::uname()`, which needs the `system` feature on this crate's
/// manifest — a file this phase does not own — and widening `HostFacts` with a
/// `descriptor()` method is the better long-term home. Either swap is a
/// replacement of this one function; if the derivation is ever inlined across
/// call sites, that cheapness is lost.
fn host_descriptor() -> HostDescriptor {
    let kernel = std::fs::read_to_string(KERNEL_RELEASE)
        .ok()
        .map(|release| release.trim().to_owned())
        .filter(|release| !release.is_empty())
        .unwrap_or_else(|| UNKNOWN_HOST_FACT.to_owned());

    HostDescriptor {
        os: std::env::consts::OS.to_owned(),
        kernel,
        arch: std::env::consts::ARCH.to_owned(),
    }
}

// ---------------------------------------------------------------------------
// Classification: liveness first, observation second (`EX-10`, `EX-12`)
// ---------------------------------------------------------------------------

/// The non-empty, trimmed lines of an arm's stdout.
fn stdout_lines(observation: &Observation) -> Vec<String> {
    String::from_utf8_lossy(&observation.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

/// An indeterminate arm, carrying whatever diagnostics the arm produced.
///
/// `None` is the arm that produced no observation at all — the backend that
/// never called the observer. Its termination is recorded as
/// `Termination::NotExecutable`, which is the honest reading: the observer
/// payload was never executed.
fn indeterminate(reason: Indeterminacy, observation: Option<&Observation>) -> ArmResult {
    match observation {
        Some(observation) => ArmResult::Indeterminate {
            reason,
            termination: observation.termination,
            stdout: observation.stdout.clone(),
            stderr: observation.stderr.clone(),
        },
        None => ArmResult::Indeterminate {
            reason,
            termination: Termination::NotExecutable,
            stdout: Vec::new(),
            stderr: Vec::new(),
        },
    }
}

/// Read one arm, in two stages, **and the order is the fix**.
///
/// Stage one is liveness: unless the payload proved it ran, the arm is
/// [`ArmResult::Indeterminate`] regardless of what the observation says. A probe
/// that was *denied* and a probe that never ran look identical on an empty
/// stdout, and if [`ArmResult::Failed`] were merely *not held*, a payload that
/// broke on the control arm only would read as `Failed` — which is exactly what
/// a row needs to report [`RowVerdict::Proven`]. A false green, through the very
/// hole a token rule exists to close.
///
/// Stage two is the kind's own reading. `Failed` is a **positive observation**,
/// never the absence of one (invariant 5).
fn classify(observation: &Observation, observed: &Observed) -> ArmResult {
    let lines = stdout_lines(observation);

    // Stage one. `Observation` carries `stdout` whatever the termination, so
    // this check is uniform across all three kinds — including a killed run.
    if !lines.iter().any(|line| line.as_str() == LIVENESS_MARKER) {
        return indeterminate(Indeterminacy::NoLiveness, Some(observation));
    }

    // Stage two.
    match observed {
        Observed::Token { held, failed } => {
            let saw_held = lines.iter().any(|line| line.as_str() == *held);
            let saw_failed = lines.iter().any(|line| line.as_str() == *failed);
            match (saw_held, saw_failed) {
                (true, true) => {
                    indeterminate(Indeterminacy::AmbiguousObservation, Some(observation))
                }
                (true, false) => ArmResult::Held,
                (false, true) => ArmResult::Failed,
                (false, false) => indeterminate(Indeterminacy::NoObservation, Some(observation)),
            }
        }
        Observed::Exactly(expected) => {
            match lines.iter().find(|line| line.as_str() != LIVENESS_MARKER) {
                None => indeterminate(Indeterminacy::NoObservation, Some(observation)),
                Some(value) if value == expected => ArmResult::Held,
                Some(_) => ArmResult::Failed,
            }
        }
        Observed::Termination(expected) => {
            if observation.termination == *expected {
                ArmResult::Held
            } else {
                ArmResult::Failed
            }
        }
    }
}

/// What the trusted side saw of a concurrent arm (row B5).
///
/// A witness rather than an execution, so the classification is pure and the two
/// failure modes `sec-7` names are testable with nothing running.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConcurrentWitness {
    /// `None` when `execute_observed` returned without ever calling back.
    pub(crate) observed_pid: Option<HostPid>,
    /// Whether the subject was still alive at the moment the observer ran.
    pub(crate) subject_live_when_observer_ran: bool,
    /// What the observer capsule produced, when it ran at all.
    pub(crate) observer: Option<Observation>,
}

/// Read a concurrent arm. Two failure modes must classify
/// [`ArmResult::Indeterminate`] rather than [`ArmResult::Failed`], and they take
/// **different** reasons — see [`Indeterminacy`].
fn classify_concurrent(witness: &ConcurrentWitness, observed: &Observed) -> ArmResult {
    // The backend did not implement the B5 seam. A suite reading this as a
    // passing probe would admit the backend it was least able to check.
    if witness.observed_pid.is_none() {
        return indeterminate(Indeterminacy::NoLiveness, witness.observer.as_ref());
    }

    // The observer ran and reported, but the window did not exist.
    if !witness.subject_live_when_observer_ran {
        return indeterminate(Indeterminacy::NoObservation, witness.observer.as_ref());
    }

    match &witness.observer {
        None => indeterminate(Indeterminacy::NoObservation, None),
        Some(observation) => classify(observation, observed),
    }
}

// ---------------------------------------------------------------------------
// Running an arm (`T6`, `EX-6`, `EX-7`)
// ---------------------------------------------------------------------------

/// Which of the backend's three entry points a capsule reaches.
///
/// **It applies to the capsule the arm is read from**, and to no other: the sole
/// capsule of a [`ArmShape::Single`], the *reader* of a
/// [`ArmShape::Sequential`], the *observer* of a [`ArmShape::Concurrent`]. The
/// capsule that merely sets the scene — the writer, the subject — always runs
/// confined, because weakening it would change what the observed capsule is
/// looking at rather than what it is allowed to see, and the row would no longer
/// differ from its probe by one property (invariant 3).
///
/// That rule is also why [`ConformanceBackend::execute_observed`] needs no
/// removal parameter (`F-19`): row B5's control removes process visibility from
/// the **observer**, which is an ordinary weakened capsule, and the subject is
/// the same confined capsule in both arms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Under {
    /// The confining profile — every probe arm, and every control arm whose
    /// delta is a change to the *placement* rather than to the profile.
    Confining,
    Removing(PropertyRemoval),
    Granting(AuthorityGrant),
}

/// Everything running one arm needs, and nothing about which arm it is.
///
/// The two closures are the phase's seams. `capsule` is called **once per
/// capsule the shape runs** (`EX-6`) — a capsule that left state behind
/// contaminates the next, so a two-capsule shape gets two transactions and never
/// one reused — and `execution` supplies the fixture's bounds, environment and
/// stdio around a payload's argv, so the payload is the only thing a row varies.
pub(crate) struct Arm<'a> {
    pub(crate) backend: &'a dyn ConformanceBackend,
    pub(crate) capsule: &'a dyn Fn() -> Result<CapsulePlacement, String>,
    pub(crate) execution: &'a dyn Fn(&Argv) -> Execution,
    /// Whether the concurrent subject is still running its payload. Injected
    /// rather than called directly so both readings of row B5's window are
    /// reachable from a test — the window that existed and the one that did not
    /// — which is the difference between the two concurrent indeterminacies
    /// being *specified* and being *tested* (`S8`).
    pub(crate) live: &'a dyn Fn(HostPid) -> bool,
    /// Where a capsule process seen alive is reported (`EX-12`).
    ///
    /// A separate sink from `live` because the two answer different questions
    /// at different times: `live` is read once, after the observer has run, and
    /// decides whether row B5's window held; this is called the moment the pid
    /// exists and decides what the sweep must reach on the way out. Folding
    /// them together would tie containment to a row shape — and the arm whose
    /// containment matters most, row 7's, is a `Single`.
    pub(crate) noticed: &'a dyn Fn(HostPid),
    pub(crate) under: Under,
}

impl std::fmt::Debug for Arm<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Arm")
            .field("backend", &self.backend.id())
            .field("under", &self.under)
            .finish_non_exhaustive()
    }
}

impl Arm<'_> {
    /// Provision one capsule and run one payload in it.
    ///
    /// The error side is already an [`ArmResult`]: everything that can go wrong
    /// here is the mechanism failing rather than the property failing, and
    /// `EX-11` gives that exactly one reading.
    fn observation(&self, argv: &Argv, under: Under) -> Result<Observation, ArmResult> {
        let placement = (self.capsule)()
            .map_err(|detail| indeterminate(Indeterminacy::BackendError(detail), None))?;
        let execution = (self.execution)(argv);
        let outcome = match under {
            Under::Confining => self.backend.execute(&placement, &execution),
            Under::Removing(removal) => self
                .backend
                .execute_weakened(&placement, &execution, removal),
            Under::Granting(grant) => self.backend.execute_granted(&placement, &execution, grant),
        };
        outcome
            .map_err(|error| indeterminate(Indeterminacy::BackendError(format!("{error:?}")), None))
    }

    fn read(&self, probe: &Probe, under: Under) -> ArmResult {
        match self.observation(&probe.argv, under) {
            Ok(observation) => classify(&observation, &probe.observed),
            Err(failure) => failure,
        }
    }

    /// Row B5's choreography.
    ///
    /// The observer capsule runs **inside** `execute_observed`'s callback, which
    /// the trait defines as the interval between the subject's top-level process
    /// existing and the trusted side waiting on it. So the window is the
    /// callback rather than something raced for on a second thread, and the arm
    /// needs no threads, no `Send` bound on the trait, and no synchronisation.
    ///
    /// The subject's stdout is unread for the duration of the callback: the
    /// backend spawns, calls back, and only then collects output. A payload that
    /// filled a pipe buffer would block until the observer finished — still
    /// alive, so the window is if anything wider, but a reason to keep the
    /// subject's output to the marker and a token.
    fn observe_concurrently(&self, subject: &Probe, observer: &PidProbe) -> ArmResult {
        let placement = match (self.capsule)() {
            Ok(placement) => placement,
            Err(detail) => return indeterminate(Indeterminacy::BackendError(detail), None),
        };
        let execution = (self.execution)(&subject.argv);

        let seen: Cell<Option<HostPid>> = Cell::new(None);
        let alive = Cell::new(false);
        let reported: RefCell<Option<Result<Observation, ArmResult>>> = RefCell::new(None);

        let run_observer = |pid: HostPid| {
            seen.set(Some(pid));
            (self.noticed)(pid);
            let outcome = self.observation(&(observer.argv)(pid), self.under);
            // Sampled **after** the observer capsule finished, not before: the
            // question row B5 asks is whether the observation was taken while
            // the subject was running, and a subject that exited halfway
            // through leaves an observation of nothing in particular.
            alive.set((self.live)(pid));
            *reported.borrow_mut() = Some(outcome);
        };

        let observed = match self
            .backend
            .execute_observed(&placement, &execution, &run_observer)
        {
            Ok(observation) => observation,
            Err(error) => {
                return indeterminate(Indeterminacy::BackendError(format!("{error:?}")), None);
            }
        };

        // The arm is read off the observer, but a subject that never ran leaves
        // nothing to have been observed — and it would otherwise be reported as
        // a held probe, since an observer that finds no live process is exactly
        // what the probe arm expects to see.
        if let ArmResult::Indeterminate {
            reason: Indeterminacy::NoLiveness,
            ..
        } = classify(&observed, &subject.observed)
        {
            return indeterminate(Indeterminacy::NoLiveness, Some(&observed));
        }

        let reported = reported.into_inner();
        let witness = match reported {
            Some(Err(failure)) => return failure,
            Some(Ok(observation)) => ConcurrentWitness {
                observed_pid: seen.get(),
                subject_live_when_observer_ran: alive.get(),
                observer: Some(observation),
            },
            None => ConcurrentWitness {
                observed_pid: None,
                subject_live_when_observer_ran: false,
                observer: None,
            },
        };
        classify_concurrent(&witness, &observer.observed)
    }
}

/// Run one arm of one row.
fn run_arm(arm: &Arm<'_>, shape: &ArmShape) -> ArmResult {
    match shape {
        ArmShape::Single(probe) => arm.read(probe, arm.under),
        ArmShape::Sequential { writer, reader } => {
            let staged = match arm.observation(&writer.argv, Under::Confining) {
                Ok(observation) => observation,
                Err(failure) => return failure,
            };
            match classify(&staged, &writer.observed) {
                ArmResult::Held => arm.read(reader, arm.under),
                // The writer's job is to make the reader's observation mean
                // something. A writer that did not do it is a broken fixture,
                // not a violated property, and reporting it as
                // `ArmResult::Failed` would put a fixture bug into the verdict
                // as evidence about the backend.
                ArmResult::Failed => indeterminate(Indeterminacy::NoObservation, Some(&staged)),
                // The writer's own capsule could not be read: that is already
                // the right answer, and it carries the right reason.
                unresolved @ ArmResult::Indeterminate { .. } => unresolved,
            }
        }
        ArmShape::Concurrent { subject, observer } => arm.observe_concurrently(subject, observer),
    }
}

// ---------------------------------------------------------------------------
// The row verdict algebra (`EX-13`)
// ---------------------------------------------------------------------------

/// Compose two arms into a row verdict. **The probe is read first**, and that
/// ordering is what distinguishes [`RowVerdict::Violated`] from
/// [`RowVerdict::Unproven`]: a failed probe says the guard is broken whatever
/// the control did, and a control that still held says the row is broken.
///
/// Both are not-admitted and both are reported, because they name **different
/// repairs** — `SL-241`'s rule that a guard never seen to fire is not known to
/// work, turned on the suite itself.
fn row_verdict(probe: ArmResult, control: ArmResult) -> RowVerdict {
    match probe {
        ArmResult::Indeterminate { reason, .. } => RowVerdict::Indeterminate {
            arm: Which::Probe,
            detail: reason,
        },
        ArmResult::Failed => RowVerdict::Violated,
        ArmResult::Held => match control {
            ArmResult::Indeterminate { reason, .. } => RowVerdict::Indeterminate {
                arm: Which::Control,
                detail: reason,
            },
            ArmResult::Held => RowVerdict::Unproven,
            ArmResult::Failed => RowVerdict::Proven,
        },
    }
}

// ---------------------------------------------------------------------------
// Admission (`EX-2`, `EX-3`, `EX-15`)
// ---------------------------------------------------------------------------

/// Table A. Empty until PHASE-09 — see [`tables`].
fn table_a() -> Vec<Row> {
    Vec::new()
}

/// Table B. Empty until PHASE-10 — see [`tables`].
fn table_b() -> Vec<Row> {
    Vec::new()
}

/// Tables A and B, which [`admission`] is computed from.
///
/// **Both are empty at PHASE-07, so [`verify`] returns [`Admission::Admitted`]
/// vacuously.** That is correct and temporary — the rows arrive in PHASE-09 and
/// PHASE-10 — and it is exactly the thing a later reader would misread as a
/// passing suite. It is written down here rather than asserted anywhere: no test
/// in this phase asserts that [`verify`]'s outcome is `Admitted`, because such a
/// test would pass under every implementation of the algebra and prove nothing.
/// The algebra is tested through [`verify_over`] against hand-built row sets
/// instead.
fn tables() -> Vec<Row> {
    let mut rows = table_a();
    rows.extend(table_b());
    rows
}

// ---------------------------------------------------------------------------
// Table C: the four auxiliary claims (`EX-14`, `EX-15`, `EX-16`)
// ---------------------------------------------------------------------------

/// The four claims, named once so the report and the tests cannot drift apart.
const READ_ONCE_CLAIM: Claim = Claim {
    section: "sec-4",
    name: "rewriting the policy inside a capsule does not change the bound policy",
};
const OBJECT_SET_CLAIM: Claim = Claim {
    section: "sec-3",
    name: "the clone's object set is exactly the export's",
};
const CAPACITY_CLAIM: Claim = Claim {
    section: "sec-5",
    name: "the capacity probe reads real space at the path it is given",
};
const CAPACITY_FILESYSTEM_CLAIM: Claim = Claim {
    section: "sec-5",
    name: "the capacity probe reads the filesystem the capsule root is on",
};

/// The clone's working tree beneath [`INNER_CAPSULE`]. Spelled here because
/// `provision`'s own constant is private to that module and `provision.rs` is
/// not a file this phase owns (`S1`) — the same reason [`GIT`] is spelled here.
const CAPSULE_REPOSITORY_LEAF: &str = "repo";

/// The interpretation field the read-once claim rewrites, and what it rewrites
/// it to.
///
/// A **policy** field, not a comment: the replacement must be one that *would*
/// change the canonical hash if the hash were re-read, or the claim passes for
/// the wrong reason. Shared with [`capsule_config_document`], which emits the
/// empty form, so the substitution cannot miss.
const EMPTY_FORBIDDEN_EXECUTABLES: &str = "trusted_side_forbidden_executables = []";
const REWRITTEN_FORBIDDEN_EXECUTABLES: &str =
    "trusted_side_forbidden_executables = [\"rewritten-by-the-capsule\"]";

/// `EX-15`'s query. `--batch-check` is **not** optional: bare
/// `--batch-all-objects` is a fatal error (`'--batch-all-objects' requires a
/// batch mode`), verified by execution.
const OBJECT_NAME_QUERY: [&str; 3] = [
    "cat-file",
    "--batch-all-objects",
    "--batch-check=%(objectname)",
];

/// Why the conditional capacity claim skips. It names the absence — a skip with
/// an empty reason is a silent pass wearing a label.
const NO_SECOND_FILESYSTEM: &str = "this host named no filesystem other than the capsule root's";

/// Table C's four claims (`EX-14`, `EX-15`, `EX-16`).
///
/// **Reported, never admitted on.** [`admission`] takes the row list alone, so
/// widening happened here and not there: a table C claim has no path to the
/// verdict in either direction, which is what makes a skip lawful here where
/// `DEC-156` forbids one in an admission.
fn auxiliary_claims(
    backend: &dyn ConformanceBackend,
    host: &dyn HostFacts,
    fixture: &Fixture,
) -> Vec<(Claim, AuxOutcome)> {
    vec![
        (READ_ONCE_CLAIM, read_once_claim(backend, host, fixture)),
        (OBJECT_SET_CLAIM, object_set_claim(backend, host, fixture)),
        (CAPACITY_CLAIM, capacity_claim(host, fixture.capsule_root())),
        (
            CAPACITY_FILESYSTEM_CLAIM,
            capacity_filesystem_claim(host, fixture.capsule_root(), fixture.second_filesystem()),
        ),
    ]
}

/// Every claim skipped for one reason — the fixture they all need could not be
/// built. Reported rather than omitted, so the report's shape does not change
/// with the host's luck.
fn claims_skipped(reason: &str) -> Vec<(Claim, AuxOutcome)> {
    [
        READ_ONCE_CLAIM,
        OBJECT_SET_CLAIM,
        CAPACITY_CLAIM,
        CAPACITY_FILESYSTEM_CLAIM,
    ]
    .into_iter()
    .map(|claim| (claim, AuxOutcome::Skipped(reason.to_owned())))
    .collect()
}

/// `sec-4`'s read-once claim, made physical.
///
/// **Two positive assertions, and the claim is vacuous without the first**: the
/// in-capsule rewrite actually landed — read back trusted-side from the
/// capsule's host path — and the policy in force is still the one captured at
/// provision. A capsule that could not write is the default outcome of a dozen
/// ways to get the mount wrong, and without the read-back this claim passes
/// against one.
fn read_once_claim(
    backend: &dyn ConformanceBackend,
    host: &dyn HostFacts,
    fixture: &Fixture,
) -> AuxOutcome {
    let transaction = match provision_capsule(fixture, host, backend.as_capsule_backend()) {
        Ok(transaction) => transaction,
        Err(refusal) => return AuxOutcome::Failed(refusal),
    };
    let bound = policy_in_force(&transaction);

    match rewrite_policy_inside(backend, fixture, &transaction) {
        Ok(()) => {}
        Err(why) => return AuxOutcome::Failed(why),
    }

    if policy_in_force(&transaction) == bound {
        AuxOutcome::Passed
    } else {
        AuxOutcome::Failed(
            "the bound policy changed after a capsule rewrote its own copy".to_owned(),
        )
    }
}

/// Overwrite the capsule's copy of the policy document from **inside** it, and
/// establish trusted-side that the write landed.
fn rewrite_policy_inside(
    backend: &dyn ConformanceBackend,
    fixture: &Fixture,
    transaction: &CapsuleTransaction,
) -> Result<(), String> {
    let document = std::fs::read_to_string(fixture.project_root().join(DOCTRINE_TOML))
        .map_err(|error| error.to_string())?
        .replace(EMPTY_FORBIDDEN_EXECUTABLES, REWRITTEN_FORBIDDEN_EXECUTABLES);
    let inner = format!("{INNER_CAPSULE}/{CAPSULE_REPOSITORY_LEAF}/{DOCTRINE_TOML}");
    let argv = shell_argv(&format!("printf '%s' '{document}' > {inner}"))?;
    ran_cleanly(backend.execute(&transaction.placement, &harness_execution(&argv)))?;

    let written = profile_owned_host_path(transaction.root(), INNER_CAPSULE)
        .join(CAPSULE_REPOSITORY_LEAF)
        .join(DOCTRINE_TOML);
    let after = std::fs::read_to_string(&written)
        .map_err(|error| format!("{}: {error}", written.display()))?;
    if after.contains(REWRITTEN_FORBIDDEN_EXECUTABLES) {
        Ok(())
    } else {
        Err(format!(
            "the in-capsule rewrite never landed at {}",
            written.display()
        ))
    }
}

/// The policy in force after a run: the hash captured at provision, and **not**
/// a re-read of the capsule's copy.
///
/// `sec-4`'s whole claim is that this function has no reason to touch the
/// filesystem. `M14` is the mutation that gives it one.
const fn policy_in_force(transaction: &CapsuleTransaction) -> PolicyHash {
    transaction.policy_hash
}

/// `sec-3`'s claim: the clone holds the export's objects and no others.
///
/// The comparison is **trusted-side** (`EX-15`) — the capsule prints its own
/// object names, which is evidence, and the trusted side runs the same query on
/// the export and decides, which is authority.
fn object_set_claim(
    backend: &dyn ConformanceBackend,
    host: &dyn HostFacts,
    fixture: &Fixture,
) -> AuxOutcome {
    let transaction = match provision_capsule(fixture, host, backend.as_capsule_backend()) {
        Ok(transaction) => transaction,
        Err(refusal) => return AuxOutcome::Failed(refusal),
    };
    match compare_object_sets(backend, &transaction) {
        Ok(()) => AuxOutcome::Passed,
        Err(why) => AuxOutcome::Failed(why),
    }
}

fn compare_object_sets(
    backend: &dyn ConformanceBackend,
    transaction: &CapsuleTransaction,
) -> Result<(), String> {
    let quoted = OBJECT_NAME_QUERY.map(|word| format!("'{word}'")).join(" ");
    let argv = shell_argv(&format!(
        "{GIT} -C {INNER_CAPSULE}/{CAPSULE_REPOSITORY_LEAF} {quoted}"
    ))?;
    let observation =
        ran_cleanly(backend.execute(&transaction.placement, &harness_execution(&argv)))?;
    let inside = object_names(&String::from_utf8_lossy(&observation.stdout));

    let export = transaction.placement.source().host();
    let listed = git(export, &OBJECT_NAME_QUERY).map_err(|fault| format!("{fault:?}"))?;
    object_sets_agree(&inside, &object_names(&listed))
}

/// Set equality, **in both directions**, over a non-empty set.
///
/// ⊇ alone passes a clone that dragged extra objects in, and that is the whole
/// claim (`M15`). The emptiness guard is the second way this could pass for
/// nothing: two empty sets are equal, and a capsule whose `git` never ran prints
/// nothing.
fn object_sets_agree(inside: &BTreeSet<String>, export: &BTreeSet<String>) -> Result<(), String> {
    if inside.is_empty() {
        return Err("the capsule named no objects at all".to_owned());
    }
    if inside == export {
        return Ok(());
    }
    let extra: Vec<&String> = inside.difference(export).collect();
    let missing: Vec<&String> = export.difference(inside).collect();
    Err(format!(
        "the clone's object set is not the export's: {extra:?} extra, {missing:?} missing"
    ))
}

fn object_names(listing: &str) -> BTreeSet<String> {
    listing
        .lines()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

/// `sec-5`'s probe, at the path it was given (`EX-16`).
///
/// **Never skips.** `REQ-461`'s only executed closure rests on this row, so a
/// host that cannot answer is a *failure* here rather than an absence.
fn capacity_claim(host: &dyn HostFacts, path: &Path) -> AuxOutcome {
    match agreed_capacity(host, path) {
        Ok(_bytes) => AuxOutcome::Passed,
        Err(why) => AuxOutcome::Failed(why),
    }
}

/// The discriminator (`EX-16`): the probe answers about the filesystem the path
/// is on, not about some fixed one.
///
/// Both figures must agree with their **own** independent `statvfs` and differ
/// from each other. The difference alone is not enough — two wrong figures also
/// differ.
fn capacity_filesystem_claim(
    host: &dyn HostFacts,
    capsule_root: &Path,
    elsewhere: Option<&Path>,
) -> AuxOutcome {
    let Some(elsewhere) = elsewhere else {
        return AuxOutcome::Skipped(format!(
            "{NO_SECOND_FILESYSTEM}, so the probe has nothing to be told apart from ({})",
            capsule_root.display()
        ));
    };
    let here = match agreed_capacity(host, capsule_root) {
        Ok(bytes) => bytes,
        Err(why) => return AuxOutcome::Failed(why),
    };
    let there = match agreed_capacity(host, elsewhere) {
        Ok(bytes) => bytes,
        Err(why) => return AuxOutcome::Failed(why),
    };
    if here == there {
        return AuxOutcome::Failed(format!(
            "the probe reported {here} for both {} and {}",
            capsule_root.display(),
            elsewhere.display()
        ));
    }
    AuxOutcome::Passed
}

/// The probe's figure at `path`, checked against a `statvfs` this function
/// performs itself and returned.
///
/// **Bracketed, not compared against a single reading** (`F-28`). Free space is
/// a live quantity: this suite's own parallel fixtures move it by tens of
/// allocation units between two adjacent syscalls, so a single independent
/// reading taken *after* the probe is not a reading of the same instant. Two
/// readings, one either side, bound what the truth can have been while the probe
/// ran, and the figure must land within one allocation unit of that interval.
/// That is the same one-unit tolerance with the measurement's own noise removed,
/// not a widened one — `M16`'s wrong quantity (`f_bfree × f_bsize`, which counts
/// the reserved blocks) is ~92 GiB out on this host, four orders of magnitude
/// beyond any interval two adjacent readings can span.
///
/// Non-zero is asserted separately because an implementation returning 0 agrees
/// with nothing and would otherwise need a coincidence to be caught.
fn agreed_capacity(host: &dyn HostFacts, path: &Path) -> Result<u64, String> {
    let at = |detail: String| format!("{}: {detail}", path.display());
    let (before, unit) = independent_capacity(path)?;
    let reported = host
        .available_bytes(path)
        .map_err(|unknown| at(format!("{unknown:?}")))?;
    let (after, _) = independent_capacity(path)?;

    if reported == 0 {
        return Err(at("the probe reported no available space".to_owned()));
    }
    let low = before.min(after).saturating_sub(unit);
    let high = before.max(after).saturating_add(unit);
    if reported < low || reported > high {
        return Err(at(format!(
            "the probe reported {reported}, outside the {low}..={high} statvfs bracketed it in"
        )));
    }
    Ok(reported)
}

/// A payload under [`SHELL`], which is what every claim's capsule runs.
fn shell_argv(script: &str) -> Result<Argv, String> {
    Argv::try_new(vec![SHELL.to_owned(), "-c".to_owned(), script.to_owned()])
        .ok_or_else(|| "an empty argv".to_owned())
}

/// An observation of a capsule that was expected to succeed, or why not.
///
/// A backend failure and a nonzero exit are different things and both are
/// disqualifying here: a claim reads what a *working* capsule produced.
fn ran_cleanly(observed: Result<Observation, BackendError>) -> Result<Observation, String> {
    let observation = observed.map_err(|error| format!("{error:?}"))?;
    match observation.termination {
        Termination::Exited { code: 0 } => Ok(observation),
        ref other => Err(format!(
            "the claim's payload did not run cleanly: {other:?}, stderr {}",
            String::from_utf8_lossy(&observation.stderr)
        )),
    }
}

/// The slice and phase every harness transaction records as its purpose.
///
/// A transaction's `PhaseIdentity` is "a durable reference to the phase this
/// serves, and nothing more", and the phase these serve is this one. Spelling a
/// real slice id here rather than a placeholder keeps a transaction found on
/// disk after a crash attributable.
const HARNESS_SLICE: &str = "SL-248";
const HARNESS_PHASE: u32 = 8;
const TRANSACTION_ID_PREFIX: &str = "conformance";

/// The next transaction id within this process.
///
/// Pid plus a counter and no clock, for the reason [`ROOT_NONCE`] gives: two
/// transactions provisioned in the same millisecond would collide on a clock,
/// and `sec-3` step 9's exclusive create is what establishes ownership anyway.
static TRANSACTION_NONCE: AtomicU32 = AtomicU32::new(0);

fn next_transaction_id() -> Result<TransactionId, String> {
    TransactionId::try_new(format!(
        "{TRANSACTION_ID_PREFIX}-{}-{}",
        std::process::id(),
        TRANSACTION_NONCE.fetch_add(1, Ordering::Relaxed)
    ))
    .map_err(|refusal| format!("{refusal:?}"))
}

/// One transaction, provisioned into the fixture's own capsule root.
///
/// **The fixture's project root, never the operator's** (invariant 7): the
/// request names `fixture.project_root()`, which is the synthetic repository
/// `T2` built, so `provision` reads the synthesized `[capsule]` table and
/// nothing on this machine outside the fixture root is named.
fn provision_capsule(
    fixture: &Fixture,
    host: &dyn HostFacts,
    backend: &dyn CapsuleBackend,
) -> Result<CapsuleTransaction, String> {
    let request = ProvisionRequest {
        repository_root: fixture.project_root().to_path_buf(),
        base: fixture.base().clone(),
        phase: PhaseIdentity {
            slice: HARNESS_SLICE.to_owned(),
            phase: HARNESS_PHASE,
        },
        id: next_transaction_id()?,
        refinement: None,
        network: NetworkPosture::Denied,
    };
    provision(&request, host, backend).map_err(|refusal| format!("{refusal:?}"))
}

/// A placement decomposed back into the parts it was built from.
///
/// `accepted_base` is not readable off a [`CapsulePlacement`] — `try_new`
/// checks it against the source export and discards it — so it comes from the
/// fixture, which is the same base every transaction here contracts.
fn parts_of(placement: &CapsulePlacement, base: &AcceptedBase) -> PlacementParts {
    PlacementParts {
        root: placement.root().clone(),
        source: placement.source().clone(),
        writable: placement.writable().to_vec(),
        readable: placement.readable().to_vec(),
        working_directory: placement.working_directory().clone(),
        network: placement.network(),
        accepted_base: base.clone(),
    }
}

/// The control arm's placement: the probe's, differing by exactly one delta.
///
/// **Every rebuild goes back through [`CapsulePlacement::try_new`]**, and that
/// is evidence rather than ceremony: the widened control passes the same
/// validating constructor the probe's placement passed, so a row that proves a
/// property cannot be dismissed as having proved that the control was
/// malformed. A refusal is reported as the mechanism failing (`EX-11`).
///
/// `first_root` is `Some` only for the **second** capsule of a two-capsule arm.
/// [`Delta::SharedRoot`] re-points that one placement onto the first
/// transaction's root; it never provisions a second transaction into the
/// first's root, which `sec-3` step 9 refuses outright.
fn placed_under(
    delta: &Delta,
    fixture: &Fixture,
    placement: CapsulePlacement,
    first_root: Option<&TransactionRoot>,
) -> Result<CapsulePlacement, String> {
    let mut parts = parts_of(&placement, fixture.base());
    match *delta {
        Delta::SharedRoot => match first_root {
            // The first capsule of a `SharedRoot` arm is the one whose root is
            // shared, so it is itself unchanged.
            None => return Ok(placement),
            Some(first) => parts.root = first.clone(),
        },
        Delta::Widened(entries) => parts.readable.extend(entries(fixture)),
        Delta::NetworkPermitted => parts.network = NetworkPosture::Permitted,
        // Placement-identical controls: `T4`'s weakening does the work, and the
        // probe's placement must reach the backend untouched (invariant 4).
        Delta::Removed(_) | Delta::Granted(_) => return Ok(placement),
    }
    CapsulePlacement::try_new(parts, fixture.scopes()).map_err(|refusal| format!("{refusal:?}"))
}

/// The backend-side control a delta implies. Only two of the five are
/// backend-side; the other three are placement rebuilds.
const fn under_for(delta: &Delta) -> Under {
    match *delta {
        Delta::Removed(removal) => Under::Removing(removal),
        Delta::Granted(grant) => Under::Granting(grant),
        Delta::SharedRoot | Delta::Widened(_) | Delta::NetworkPermitted => Under::Confining,
    }
}

/// The bounds and stdio every harness payload runs under.
///
/// The two bounds are the fixture's own declared `[capsule]` values, so a
/// payload that hangs is killed by the same wall bound `provision` would have
/// applied — the suite has no separate timeout policy to drift from it.
fn harness_execution(argv: &Argv) -> Execution {
    Execution::new(
        argv.clone(),
        CapsuleEnv::complete(),
        Duration::from_secs(FIXTURE_TIMEOUT_SECONDS),
        ByteCount::from_bytes(FIXTURE_FILE_SIZE_CAP_MIB * BYTES_PER_MIB),
        CapsuleStdio::EmptyInputCapturedOutput,
    )
}

const BYTES_PER_MIB: u64 = 1024 * 1024;

/// Run one row: the probe arm, then the control arm, then the algebra.
///
/// **The probe arm's placement is exactly what `provision` returned**
/// (invariant 4) — no rebuild, no clone-and-edit, no delta. That is what makes
/// the row a statement about the shipping configuration rather than about a
/// configuration the suite assembled to be provable.
///
/// Both arms are run unconditionally, in that order. [`row_verdict`] needs both
/// readings, and short-circuiting on a failed probe would make
/// [`RowVerdict::Violated`] cheaper to reach than [`RowVerdict::Proven`] — the
/// wrong asymmetry for a suite whose green path must be the expensive one.
fn run_row(
    backend: &dyn ConformanceBackend,
    host: &dyn HostFacts,
    fixture: &Fixture,
    row: &Row,
) -> RowVerdict {
    let live = |pid: HostPid| capsule_still_running(pid);
    let noticed = |pid: HostPid| fixture.note_capsule_session(pid);

    let untouched = || {
        provision_capsule(fixture, host, backend.as_capsule_backend())
            .map(|transaction| transaction.placement)
    };
    let probe = run_arm(
        &Arm {
            backend,
            capsule: &untouched,
            execution: &harness_execution,
            live: &live,
            noticed: &noticed,
            under: Under::Confining,
        },
        &row.shape,
    );

    // Held across the control arm's capsules so `SharedRoot`'s second placement
    // can be re-pointed onto the first's root. It is the *root* that is kept and
    // not the transaction: nothing in this crate removes a transaction root, and
    // the fixture's own `Drop` reclaims the whole capsule root at the end of the
    // run (invariant 8).
    let first_root: RefCell<Option<TransactionRoot>> = RefCell::new(None);
    let deltaed = || {
        let transaction = provision_capsule(fixture, host, backend.as_capsule_backend())?;
        let placement = placed_under(
            &row.delta,
            fixture,
            transaction.placement,
            first_root.borrow().as_ref(),
        )?;
        let mut first = first_root.borrow_mut();
        if first.is_none() {
            *first = Some(placement.root().clone());
        }
        Ok(placement)
    };
    let control = run_arm(
        &Arm {
            backend,
            capsule: &deltaed,
            execution: &harness_execution,
            live: &live,
            noticed: &noticed,
            under: under_for(&row.delta),
        },
        &row.shape,
    );

    // On the way out of **every** row, not only the last: a survivor left by
    // row 7's control arm would otherwise still be running while the next row's
    // arms provision, and `EX-12`'s failure is silent — a leaked process per
    // run, found by a developer whose machine is slowly filling with them.
    let _swept = fixture.sweep_observed_sessions();

    row_verdict(probe, control)
}

/// The outcome, **computed from the row list alone**.
///
/// That auxiliary outcomes cannot reach admission in either direction is
/// structural rather than a promise: this function is not given them.
fn admission(rows: &[(RowId, RowVerdict)]) -> Admission {
    if rows
        .iter()
        .all(|(_, verdict)| matches!(*verdict, RowVerdict::Proven))
    {
        Admission::Admitted
    } else {
        Admission::NotAdmitted {
            reason: NotAdmitted::Rows,
        }
    }
}

/// Row ids appearing in more than one of `tables`.
///
/// A pure helper over a slice of tables so the covering property can be asserted
/// over the code's own tables *and* over a hand-built pair that deliberately
/// shares one — the second is what gives the assertion any force while the
/// code's tables are empty.
fn row_ids_in_more_than_one_table(tables: &[&[Row]]) -> Vec<RowId> {
    let mut shared: Vec<RowId> = Vec::new();
    for table in tables {
        for row in *table {
            let tables_holding_it = tables
                .iter()
                .filter(|other| other.iter().any(|candidate| candidate.id == row.id))
                .count();
            if tables_holding_it > 1 && !shared.contains(&row.id) {
                shared.push(row.id.clone());
            }
        }
    }
    shared
}

/// The whole suite, parameterised by backend. `REQ-459` criterion 3: a second
/// backend is admitted by passing these assertions, never by editing them.
///
/// **Three parameters, and no `CapsuleConfig`** (`EX-3`). Taking the operator's
/// `[capsule]` table would be wrong twice over: the suite would be testing the
/// operator's configuration rather than the backend's enforcement, and a table
/// whose `readable-roots` omitted a shell would make every probe
/// `NotExecutable` and the whole run indeterminate for a reason it never named.
/// The fixture synthesizes its own `CapsuleConfig` over its own root instead, so
/// the only host facts admission depends on are the backend's availability and a
/// working shell.
///
/// `today` is a parameter rather than a read, for the reason `HostFacts` carries
/// no clock: `src/clock.rs` is the single home for wall-clock reads and
/// `main.rs` is the shell that performs it.
pub(crate) fn verify(
    backend: &dyn ConformanceBackend,
    host: &dyn HostFacts,
    today: String,
) -> AdmissionVerdict {
    // Built lazily, and that laziness is invariant 1's ordering rather than an
    // optimisation: `verify_over` reports availability and the shell **before**
    // any row runs, and a fixture built here would put a git-and-disk build
    // ahead of both — so a host with no `bwrap` would be told about its disk.
    // The first row to need it builds it; a host that cannot build one gets
    // every row `Indeterminate` naming the fault, which is `EX-11`'s rule for a
    // mechanism that failed rather than a property that did.
    let fixture: OnceCell<Result<Fixture, FixtureFault>> = OnceCell::new();
    let run = |mechanism: &dyn ConformanceBackend, row: &Row| match fixture
        .get_or_init(|| Fixture::new(host))
    {
        Ok(fixture) => run_row(mechanism, host, fixture, row),
        Err(fault) => RowVerdict::Indeterminate {
            arm: Which::Probe,
            detail: Indeterminacy::BackendError(format!("{fault:?}")),
        },
    };

    let claims = || match fixture.get_or_init(|| Fixture::new(host)) {
        Ok(fixture) => auxiliary_claims(backend, host, fixture),
        Err(fault) => claims_skipped(&format!("{fault:?}")),
    };

    verify_over(backend, host, today, &tables(), &claims, &run)
}

/// [`verify`] over an injected row set and row runner.
///
/// Split out so the algebra can be exercised against hand-built row sets and a
/// counting runner — without it, *runs no row* is unfalsifiable while the real
/// tables are empty. [`verify`]'s own signature is untouched.
///
/// **One green path** (invariant 1), in this order: availability, then the
/// shell, then every row. No skip, no early return and no conditional reaches
/// [`Admission::Admitted`].
fn verify_over(
    backend: &dyn ConformanceBackend,
    host: &dyn HostFacts,
    today: String,
    rows: &[Row],
    auxiliary: &dyn Fn() -> Vec<(Claim, AuxOutcome)>,
    run_row: &dyn Fn(&dyn ConformanceBackend, &Row) -> RowVerdict,
) -> AdmissionVerdict {
    let backend_id = backend.id();
    let host_facts = host_descriptor();

    if let Availability::Unavailable { missing, remedy } = backend.availability() {
        return AdmissionVerdict {
            backend: backend_id,
            host: host_facts,
            date: today,
            outcome: Admission::NotAdmitted {
                reason: NotAdmitted::Unavailable { missing, remedy },
            },
            rows: Vec::new(),
            auxiliary: Vec::new(),
        };
    }

    // A host with no usable shell is *unavailable*, naming what is missing —
    // never a violated row. Every payload runs under `/bin/sh -c`.
    if !host.path_exists(Path::new(SHELL)) {
        return AdmissionVerdict {
            backend: backend_id,
            host: host_facts,
            date: today,
            outcome: Admission::NotAdmitted {
                reason: NotAdmitted::Unavailable {
                    missing: SHELL.to_owned(),
                    remedy: SHELL_REMEDY.to_owned(),
                },
            },
            rows: Vec::new(),
            auxiliary: Vec::new(),
        };
    }

    let auxiliary = auxiliary();

    let verdicts: Vec<(RowId, RowVerdict)> = rows
        .iter()
        .map(|row| (row.id.clone(), run_row(backend, row)))
        .collect();
    let outcome = admission(&verdicts);

    AdmissionVerdict {
        backend: backend_id,
        host: host_facts,
        date: today,
        outcome,
        rows: verdicts,
        auxiliary,
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs::File;
    use std::io::Write as _;
    use std::os::fd::AsRawFd as _;
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use tempfile::TempDir;

    use super::Weakening as ProfileWeakening;
    use super::{
        Admission, AdmissionVerdict, ArmResult, ArmShape, AuthorityGrant, AuxOutcome, Axis, Bound,
        Claim, ConcurrentWitness, ConformanceBackend, DELETED_SUFFIX, Delta, Fixture,
        HOME_VARIABLE, HostPid, Indeterminacy, InheritableDecoys, LIVENESS_MARKER, NotAdmitted,
        Observed, PidProbe, Probe, PropertyRemoval, Row, RowId, RowVerdict, SHELL, TMPFS_MAGIC,
        TempRoot, Which, admission, available_bytes_of, capsule_config_document, classify,
        classify_concurrent, decode_mount_field, git, is_inheritable, mount_points, on_real_disk,
        prepare_root, row_ids_in_more_than_one_table, row_verdict, second_filesystem,
        system_readable_roots, top_level_ancestor, verify, verify_over,
    };
    use super::{
        Arm, BYTES_PER_MIB, FIXTURE_FILE_SIZE_CAP_MIB, FIXTURE_TIMEOUT_SECONDS, Under,
        capsule_still_running, harness_execution, next_transaction_id, run_arm, still_running,
        under_for,
    };
    use super::{
        CAPACITY_CLAIM, CAPACITY_FILESYSTEM_CLAIM, DOCTRINE_TOML, EMPTY_FORBIDDEN_EXECUTABLES,
        NO_SECOND_FILESYSTEM, OBJECT_SET_CLAIM, READ_ONCE_CLAIM, capacity_claim,
        capacity_filesystem_claim, claims_skipped, object_set_claim, object_sets_agree,
        read_once_claim,
    };
    use super::{OwnedStdio, weakening_for, weakening_granting};
    use super::{
        ProcessFacts, STAT_LEAF, SessionId, StatFacts, capsule_session_leader, depth_from,
        own_session, process_table, session_of, stat_of,
    };
    use crate::backend::bubblewrap::{BubblewrapBackend, SpawnOptions, confinement_argv};
    use crate::backend::fixture::{WITNESS_ID, WitnessBackend, exited};
    use crate::backend::{
        AcceptedBase, Availability, BackendError, BackendId, CapsuleBackend, CapsuleEnv,
        CapsulePlacement, CapsuleStdio, Execution, ForbiddenScopes, InnerPath, MountedPath,
        NetworkPosture, Observation, PlacementParts, SourceExport, Termination, TransactionRoot,
    };
    use crate::config::{Argv, ByteCount};
    use crate::host::HostFacts;
    use crate::host::SystemHost;
    use crate::host::fixture::FixtureHost;

    // ── Payload tokens, test-local ─────────────────────────────────────────
    //
    // The real payload constants are PHASE-08's, with their rows. These are the
    // two-token shape every classification test reads.

    const HELD: &str = "HELD";
    const HELD_NOT: &str = "HELD-NOT";
    const A_VALUE: &str = "1048576";
    const ANOTHER_VALUE: &str = "0";

    const TODAY: &str = "2026-08-09";

    // ── Placement geometry (`D6`) ──────────────────────────────────────────
    //
    // Built locally: `backend.rs`'s `lawful_parts` is private to its own test
    // module and `bubblewrap.rs` has its own `placement_with`, so a third local
    // builder is the established shape rather than duplication of a shared one.

    const CANONICAL_REPOSITORY: &str = "/srv/repo";
    const CONTROL_PLANE_STATE: &str = "/srv/repo/.doctrine";
    const CAPSULE_ROOT: &str = "/var/lib/doctrine";
    const CREDENTIALS: &str = "/home/agent/.ssh";
    const TRANSACTION_ROOT: &str = "/var/lib/doctrine/tx/0007";
    const BASE_OID: &str = "1f0e3dad99908345f7439f8ffabdffc4";
    const EXPORT: &str = "/var/lib/doctrine/export/1f0e3dad99908345f7439f8ffabdffc4";
    const READABLE: &str = "/nix/store/bash";
    const WORKING_DIRECTORY: &str = "/capsule";

    fn inner(path: &str) -> InnerPath {
        InnerPath::try_new(PathBuf::from(path)).expect("fixture inner paths are absolute")
    }

    fn placement() -> CapsulePlacement {
        let parts = PlacementParts {
            root: TransactionRoot::new(PathBuf::from(TRANSACTION_ROOT)),
            source: SourceExport::new(
                PathBuf::from(EXPORT),
                AcceptedBase::new(BASE_OID.to_owned()),
            ),
            writable: Vec::new(),
            readable: vec![MountedPath::new(PathBuf::from(READABLE), inner(READABLE))],
            working_directory: inner(WORKING_DIRECTORY),
            network: NetworkPosture::Denied,
            accepted_base: AcceptedBase::new(BASE_OID.to_owned()),
        };
        let scopes = ForbiddenScopes::new(
            PathBuf::from(CANONICAL_REPOSITORY),
            PathBuf::from(CONTROL_PLANE_STATE),
            PathBuf::from(CAPSULE_ROOT),
            vec![PathBuf::from(CREDENTIALS)],
        );
        CapsulePlacement::try_new(parts, &scopes).expect("the local fixture placement is lawful")
    }

    fn argv(words: &[&str]) -> Argv {
        Argv::try_new(words.iter().map(|word| (*word).to_owned()).collect())
            .expect("fixture argv is non-empty")
    }

    fn execution() -> Execution {
        Execution::new(
            argv(&[SHELL, "-c", "echo LIVE; echo HELD"]),
            CapsuleEnv::complete(),
            Duration::from_secs(5),
            ByteCount::from_bytes(1024),
            CapsuleStdio::EmptyInputCapturedOutput,
        )
    }

    // ── Observations ───────────────────────────────────────────────────────

    fn observed_stdout(termination: Termination, lines: &[&str]) -> Observation {
        Observation {
            termination,
            stdout: lines.join("\n").into_bytes(),
            stderr: Vec::new(),
            disk_used: ByteCount::from_bytes(0),
        }
    }

    /// An ordinary clean exit printing `lines`.
    fn ran(lines: &[&str]) -> Observation {
        observed_stdout(Termination::Exited { code: 0 }, lines)
    }

    fn token() -> Observed {
        Observed::Token {
            held: HELD,
            failed: HELD_NOT,
        }
    }

    // ── The stub backends (`T7`, `D3`) ─────────────────────────────────────
    //
    // One double, over `backend::fixture::WitnessBackend` — a second
    // `CapsuleBackend` double would be a second definition of what a backend
    // does, which the project rules forbid.

    /// What a stub's `execute_weakened` does.
    #[derive(Debug)]
    enum Weakening {
        /// The dishonest backend `EX-7` names: it ignores its removal and runs
        /// exactly as `execute` does, so both arms observe the same thing.
        DelegatesToExecute,
        /// An honest control: the removal changes what the capsule observes.
        Answers(Result<Observation, BackendError>),
    }

    /// What a stub's `execute_observed` does with its observer.
    #[derive(Debug)]
    enum ObserverSeam {
        CallsBack(HostPid),
        /// The backend that did not implement the B5 seam.
        NeverCallsBack,
    }

    #[derive(Debug)]
    struct Stub {
        inner: WitnessBackend,
        weakening: Weakening,
        seam: ObserverSeam,
        /// Which entry point each call came through, in order — the evidence
        /// that a weakening reached the capsule it was meant for and no other.
        reached: RefCell<Vec<Under>>,
    }

    impl Stub {
        fn new(inner: WitnessBackend, weakening: Weakening, seam: ObserverSeam) -> Self {
            Self {
                inner,
                weakening,
                seam,
                reached: RefCell::new(Vec::new()),
            }
        }

        fn ignoring_its_removal(stdout: &[&str]) -> Self {
            Self::new(
                WitnessBackend::always(Ok(ran(stdout))),
                Weakening::DelegatesToExecute,
                ObserverSeam::CallsBack(HostPid(4242)),
            )
        }

        fn weakening_honestly(probe: &[&str], weakened: &[&str]) -> Self {
            Self::new(
                WitnessBackend::always(Ok(ran(probe))),
                Weakening::Answers(Ok(ran(weakened))),
                ObserverSeam::CallsBack(HostPid(4242)),
            )
        }

        fn unavailable(missing: &str, remedy: &str) -> Self {
            Self::new(
                WitnessBackend::always(exited(0, "")).reporting(Availability::Unavailable {
                    missing: missing.to_owned(),
                    remedy: remedy.to_owned(),
                }),
                Weakening::DelegatesToExecute,
                ObserverSeam::CallsBack(HostPid(4242)),
            )
        }

        fn never_calling_back() -> Self {
            Self::new(
                WitnessBackend::always(Ok(ran(&[LIVENESS_MARKER, HELD]))),
                Weakening::DelegatesToExecute,
                ObserverSeam::NeverCallsBack,
            )
        }

        /// One scripted outcome per `execute` call, and one fixed answer for
        /// every weakened call.
        fn scripted(script: Vec<Observation>, weakened: Observation) -> Self {
            Self::new(
                WitnessBackend::scripted(
                    script.into_iter().map(Ok).collect(),
                    Ok(ran(&[LIVENESS_MARKER])),
                ),
                Weakening::Answers(Ok(weakened)),
                ObserverSeam::CallsBack(HostPid(4242)),
            )
        }

        fn executions(&self) -> usize {
            self.inner.calls().len()
        }

        fn reached(&self) -> Vec<Under> {
            self.reached.borrow().clone()
        }
    }

    impl CapsuleBackend for Stub {
        fn id(&self) -> BackendId {
            self.inner.id()
        }

        fn availability(&self) -> Availability {
            self.inner.availability()
        }

        fn execute(
            &self,
            placement: &CapsulePlacement,
            execution: &Execution,
        ) -> Result<Observation, BackendError> {
            self.reached.borrow_mut().push(Under::Confining);
            self.inner.execute(placement, execution)
        }
    }

    impl ConformanceBackend for Stub {
        fn as_capsule_backend(&self) -> &dyn CapsuleBackend {
            self
        }

        fn execute_weakened(
            &self,
            placement: &CapsulePlacement,
            execution: &Execution,
            removal: PropertyRemoval,
        ) -> Result<Observation, BackendError> {
            match &self.weakening {
                Weakening::DelegatesToExecute => self.execute(placement, execution),
                Weakening::Answers(answer) => {
                    self.reached.borrow_mut().push(Under::Removing(removal));
                    answer.clone()
                }
            }
        }

        fn execute_granted(
            &self,
            placement: &CapsulePlacement,
            execution: &Execution,
            grant: AuthorityGrant,
        ) -> Result<Observation, BackendError> {
            match &self.weakening {
                Weakening::DelegatesToExecute => self.execute(placement, execution),
                Weakening::Answers(answer) => {
                    self.reached.borrow_mut().push(Under::Granting(grant));
                    answer.clone()
                }
            }
        }

        fn execute_observed(
            &self,
            placement: &CapsulePlacement,
            execution: &Execution,
            observer: &dyn Fn(HostPid),
        ) -> Result<Observation, BackendError> {
            if let ObserverSeam::CallsBack(pid) = self.seam {
                observer(pid);
            }
            self.execute(placement, execution)
        }
    }

    // ── Row helpers ────────────────────────────────────────────────────────

    /// Row B5's observer argv, rendered from the trusted-side pid. The one value
    /// any payload interpolates.
    fn observer_argv(pid: HostPid) -> Argv {
        argv(&[
            SHELL,
            "-c",
            &format!(
                "echo {LIVENESS_MARKER}; if kill -0 {} 2>/dev/null; then echo {HELD_NOT}; else echo {HELD}; fi",
                pid.0
            ),
        ])
    }

    /// A widening that names no entry. `Delta::Widened`'s payload type is
    /// PHASE-08's `Fixture`; nothing here constructs one.
    fn widens_nothing(_fixture: &Fixture) -> Vec<MountedPath> {
        Vec::new()
    }

    fn a_probe() -> Probe {
        Probe {
            argv: argv(&[SHELL, "-c", "echo LIVE; echo HELD"]),
            observed: token(),
        }
    }

    fn row(axis: Axis, shape: ArmShape, delta: Delta) -> Row {
        Row {
            id: RowId::Axis(axis),
            shape,
            delta,
        }
    }

    /// Four hand-built rows spanning every arm shape and every delta variant.
    fn four_rows() -> Vec<Row> {
        vec![
            row(
                Axis::Checkout,
                ArmShape::Single(a_probe()),
                Delta::Removed(PropertyRemoval::WorkingDirectory),
            ),
            row(
                Axis::Repository,
                ArmShape::Sequential {
                    writer: a_probe(),
                    reader: a_probe(),
                },
                Delta::SharedRoot,
            ),
            row(
                Axis::Runtime,
                ArmShape::Concurrent {
                    subject: a_probe(),
                    observer: PidProbe {
                        argv: observer_argv,
                        observed: token(),
                    },
                },
                Delta::Widened(widens_nothing),
            ),
            row(
                Axis::TemporaryState,
                ArmShape::Single(a_probe()),
                Delta::Granted(AuthorityGrant::AllCapabilities),
            ),
        ]
    }

    fn one_more_row() -> Row {
        row(
            Axis::Process,
            ArmShape::Single(a_probe()),
            Delta::NetworkPermitted,
        )
    }

    /// A host that has a usable shell. `FixtureHost::new()` is the host on which
    /// nothing exists, so this is the one that must be built.
    fn host_with_shell() -> FixtureHost {
        FixtureHost::new().with_resolution(SHELL, SHELL)
    }

    /// A runner answering a scripted verdict per row, in order, and counting how
    /// many times it was asked.
    struct ScriptedRunner {
        verdicts: RefCell<Vec<RowVerdict>>,
        calls: Cell<usize>,
    }

    impl ScriptedRunner {
        fn new(verdicts: Vec<RowVerdict>) -> Self {
            Self {
                verdicts: RefCell::new(verdicts),
                calls: Cell::new(0),
            }
        }

        fn run(&self, _backend: &dyn ConformanceBackend, _row: &Row) -> RowVerdict {
            self.calls.set(self.calls.get() + 1);
            let mut remaining = self.verdicts.borrow_mut();
            if remaining.is_empty() {
                RowVerdict::Proven
            } else {
                remaining.remove(0)
            }
        }
    }

    fn verdict_over(
        backend: &dyn ConformanceBackend,
        host: &dyn HostFacts,
        rows: &[Row],
        auxiliary: Vec<(Claim, AuxOutcome)>,
        runner: &ScriptedRunner,
    ) -> AdmissionVerdict {
        verify_over(
            backend,
            host,
            TODAY.to_owned(),
            rows,
            &|| auxiliary.clone(),
            &|backend, row| runner.run(backend, row),
        )
    }

    fn reason(result: &ArmResult) -> Indeterminacy {
        match result {
            ArmResult::Indeterminate { reason, .. } => reason.clone(),
            other => panic!("expected an indeterminate arm, got {other:?}"),
        }
    }

    // ── VT-1: classification ───────────────────────────────────────────────

    /// `EX-10`'s two-stage order, over **all three** `Observed` kinds — the
    /// `Exactly` and `Termination` paths are the ones a single-kind test misses.
    ///
    /// Every fixture here carries a **positive** observation that stage two
    /// would read (the failed token, a wrong value, the expected termination),
    /// so deleting the liveness gate changes the answer. A fixture with nothing
    /// to read would classify `Indeterminate` whether or not the gate exists and
    /// would prove nothing.
    #[test]
    fn no_liveness_marker_is_indeterminate_not_failed() {
        let token_arm = classify(&ran(&[HELD_NOT]), &token());
        assert_eq!(reason(&token_arm), Indeterminacy::NoLiveness);
        assert_ne!(token_arm, ArmResult::Failed);

        let value_arm = classify(
            &ran(&[ANOTHER_VALUE]),
            &Observed::Exactly(A_VALUE.to_owned()),
        );
        assert_eq!(reason(&value_arm), Indeterminacy::NoLiveness);
        assert_ne!(value_arm, ArmResult::Failed);

        let killed = observed_stdout(Termination::TimedOut, &[]);
        let termination_arm = classify(&killed, &Observed::Termination(Termination::TimedOut));
        assert_eq!(reason(&termination_arm), Indeterminacy::NoLiveness);
        assert_ne!(termination_arm, ArmResult::Held);

        // The fixtures must discriminate: with the marker present, each of the
        // three reads its own positive answer.
        assert_eq!(
            classify(&ran(&[LIVENESS_MARKER, HELD_NOT]), &token()),
            ArmResult::Failed
        );
        assert_eq!(
            classify(
                &ran(&[LIVENESS_MARKER, ANOTHER_VALUE]),
                &Observed::Exactly(A_VALUE.to_owned())
            ),
            ArmResult::Failed
        );
        assert_eq!(
            classify(
                &observed_stdout(Termination::TimedOut, &[LIVENESS_MARKER]),
                &Observed::Termination(Termination::TimedOut)
            ),
            ArmResult::Held
        );
    }

    /// Both tokens means the payload is wrong, not that the property held.
    #[test]
    fn both_tokens_present_is_ambiguous_not_held() {
        let arm = classify(&ran(&[LIVENESS_MARKER, HELD, HELD_NOT]), &token());
        assert_eq!(reason(&arm), Indeterminacy::AmbiguousObservation);
        assert_ne!(arm, ArmResult::Held);

        // Discriminating: the same payload with only the held token does hold,
        // so the ambiguity is what moved the answer.
        assert_eq!(
            classify(&ran(&[LIVENESS_MARKER, HELD]), &token()),
            ArmResult::Held
        );
    }

    /// Row 6's shape: nothing to compare is not the same as comparing unequal.
    #[test]
    fn a_missing_value_line_is_indeterminate_rather_than_unequal() {
        let arm = classify(
            &ran(&[LIVENESS_MARKER]),
            &Observed::Exactly(A_VALUE.to_owned()),
        );
        assert_eq!(reason(&arm), Indeterminacy::NoObservation);
        assert_ne!(arm, ArmResult::Failed);
    }

    /// The sibling the test above needs: without it, a rule that classified
    /// *everything* `Indeterminate` would satisfy it.
    #[test]
    fn a_wrong_value_line_is_failed_rather_than_indeterminate() {
        assert_eq!(
            classify(
                &ran(&[LIVENESS_MARKER, ANOTHER_VALUE]),
                &Observed::Exactly(A_VALUE.to_owned())
            ),
            ArmResult::Failed
        );
        assert_eq!(
            classify(
                &ran(&[LIVENESS_MARKER, A_VALUE]),
                &Observed::Exactly(A_VALUE.to_owned())
            ),
            ArmResult::Held
        );
    }

    /// `Observation` carries `stdout` whatever the termination, so a killed run
    /// still established its liveness.
    #[test]
    fn a_termination_observation_reads_its_marker_from_a_killed_run() {
        let killed = observed_stdout(Termination::TimedOut, &[LIVENESS_MARKER]);
        assert_eq!(
            classify(&killed, &Observed::Termination(Termination::TimedOut)),
            ArmResult::Held
        );
        // Discriminating: the same killed run read for a *different*
        // termination fails rather than holding.
        assert_eq!(
            classify(
                &killed,
                &Observed::Termination(Termination::Exited { code: 0 })
            ),
            ArmResult::Failed
        );
    }

    /// The backend that did not implement the B5 seam. A suite reading this as
    /// a passing probe would admit the backend it was least able to check.
    ///
    /// The assertion is on the **reason**, not merely on indeterminacy: the two
    /// concurrent failure modes take different reasons precisely so this test
    /// and its sibling below assert different things about different inputs.
    #[test]
    fn an_observed_execution_that_never_calls_back_is_indeterminate_not_held() {
        let backend = Stub::never_calling_back();
        let seen: RefCell<Option<HostPid>> = RefCell::new(None);
        let observation = backend
            .execute_observed(&placement(), &execution(), &|pid| {
                *seen.borrow_mut() = Some(pid);
            })
            .expect("the stub runs");

        let witness = ConcurrentWitness {
            observed_pid: seen.into_inner(),
            subject_live_when_observer_ran: true,
            observer: Some(observation),
        };
        assert_eq!(witness.observed_pid, None);

        let arm = classify_concurrent(&witness, &token());
        assert_eq!(reason(&arm), Indeterminacy::NoLiveness);
        assert_ne!(arm, ArmResult::Held);

        // Discriminating: a backend that *does* call back, over the same
        // observer output, yields a held arm — so the missing callback is what
        // moved the answer, not the payload.
        let honest = Stub::ignoring_its_removal(&[LIVENESS_MARKER, HELD]);
        let seen: RefCell<Option<HostPid>> = RefCell::new(None);
        let observation = honest
            .execute_observed(&placement(), &execution(), &|pid| {
                *seen.borrow_mut() = Some(pid);
            })
            .expect("the stub runs");
        let called_back = ConcurrentWitness {
            observed_pid: seen.into_inner(),
            subject_live_when_observer_ran: true,
            observer: Some(observation),
        };
        assert!(called_back.observed_pid.is_some());
        assert_eq!(classify_concurrent(&called_back, &token()), ArmResult::Held);
    }

    /// The observer ran and reported, but the window did not exist — so the arm
    /// observed nothing about the property, whatever its payload printed.
    #[test]
    fn a_subject_that_exited_before_the_observer_ran_is_indeterminate() {
        let witness = ConcurrentWitness {
            observed_pid: Some(HostPid(4242)),
            subject_live_when_observer_ran: false,
            observer: Some(ran(&[LIVENESS_MARKER, HELD])),
        };
        let arm = classify_concurrent(&witness, &token());
        assert_eq!(reason(&arm), Indeterminacy::NoObservation);
        assert_ne!(arm, ArmResult::Held);

        // Discriminating: the observer's own output classifies `Held`, so the
        // dead window is the only thing that moved the answer.
        let live = ConcurrentWitness {
            subject_live_when_observer_ran: true,
            ..witness
        };
        assert_eq!(classify_concurrent(&live, &token()), ArmResult::Held);
    }

    /// Row B5's observer argv is rendered from the pid the **trusted side**
    /// observed, never one the subject reported about itself (`REQ-448`
    /// criterion 3). It is the one value any payload interpolates, which is why
    /// `PidProbe` carries a function where every other payload carries an
    /// `Argv`.
    #[test]
    fn the_observer_payload_is_rendered_from_the_trusted_side_pid() {
        let probe = PidProbe {
            argv: observer_argv,
            observed: token(),
        };
        let rendered = (probe.argv)(HostPid(4242));

        assert!(
            rendered.as_slice().iter().any(|word| word.contains("4242")),
            "the observed pid must reach the payload"
        );
        assert_eq!(probe.observed, token());

        // Discriminating: a different pid renders a different payload, so the
        // interpolation is real and not a fixed string that happens to match.
        assert_ne!((probe.argv)(HostPid(9)).as_slice(), rendered.as_slice());
    }

    /// An indeterminate arm carries its own diagnostics — that is what makes it
    /// triageable rather than a shrug.
    #[test]
    fn an_indeterminate_arm_carries_its_termination_and_output() {
        let observation = Observation {
            termination: Termination::Signalled { signal: 9 },
            stdout: b"noise".to_vec(),
            stderr: b"why".to_vec(),
            disk_used: ByteCount::from_bytes(0),
        };
        match classify(&observation, &token()) {
            ArmResult::Indeterminate {
                reason,
                termination,
                stdout,
                stderr,
            } => {
                assert_eq!(reason, Indeterminacy::NoLiveness);
                assert_eq!(termination, Termination::Signalled { signal: 9 });
                assert_eq!(stdout, b"noise".to_vec());
                assert_eq!(stderr, b"why".to_vec());
            }
            other => panic!("expected an indeterminate arm, got {other:?}"),
        }
    }

    /// The false-green path stated directly, and it is a **composition** test:
    /// if `Failed` were merely *not `Held`*, a payload breaking on the control
    /// arm only would read `Proven`.
    #[test]
    fn an_arm_specific_breakage_on_the_control_arm_does_not_yield_proven() {
        let probe = classify(&ran(&[LIVENESS_MARKER, HELD]), &token());
        // The control's payload broke: no marker, so nothing it printed is read.
        let control = classify(&ran(&[]), &token());
        assert_eq!(probe, ArmResult::Held);
        assert_eq!(reason(&control), Indeterminacy::NoLiveness);

        assert_eq!(
            row_verdict(probe, control),
            RowVerdict::Indeterminate {
                arm: Which::Control,
                detail: Indeterminacy::NoLiveness,
            }
        );
    }

    // ── VT-2: the row verdict algebra ──────────────────────────────────────

    #[test]
    fn held_probe_and_failed_control_is_proven() {
        assert_eq!(
            row_verdict(ArmResult::Held, ArmResult::Failed),
            RowVerdict::Proven
        );
    }

    /// The probe is read **first**. Both arms failed here, so reading the
    /// control first would report `Proven` for a property that is not enforced.
    #[test]
    fn failed_probe_is_violated_even_when_the_control_failed() {
        assert_eq!(
            row_verdict(ArmResult::Failed, ArmResult::Failed),
            RowVerdict::Violated
        );
        // Discriminating: with a *held* probe the same control yields `Proven`,
        // so it is the probe arm that decided this.
        assert_eq!(
            row_verdict(ArmResult::Held, ArmResult::Failed),
            RowVerdict::Proven
        );
    }

    /// A control that still held means removing the property changed nothing.
    #[test]
    fn held_control_is_unproven_rather_than_proven() {
        assert_eq!(
            row_verdict(ArmResult::Held, ArmResult::Held),
            RowVerdict::Unproven
        );
    }

    #[test]
    fn an_indeterminate_arm_is_never_proven() {
        let broken = || ArmResult::Indeterminate {
            reason: Indeterminacy::BackendError("the mechanism failed".to_owned()),
            termination: Termination::NotExecutable,
            stdout: Vec::new(),
            stderr: Vec::new(),
        };

        assert_eq!(
            row_verdict(broken(), ArmResult::Failed),
            RowVerdict::Indeterminate {
                arm: Which::Probe,
                detail: Indeterminacy::BackendError("the mechanism failed".to_owned()),
            }
        );
        assert_eq!(
            row_verdict(ArmResult::Held, broken()),
            RowVerdict::Indeterminate {
                arm: Which::Control,
                detail: Indeterminacy::BackendError("the mechanism failed".to_owned()),
            }
        );
    }

    /// `EX-7`: there is no lazy implementation that yields green. A backend
    /// whose `execute_weakened` delegates to `execute` shows the property still
    /// holding on every control arm, so **every** removal reads `Unproven`.
    #[test]
    fn a_backend_ignoring_its_removal_yields_unproven_for_every_row() {
        let dishonest = Stub::ignoring_its_removal(&[LIVENESS_MARKER, HELD]);
        let placement = placement();
        let execution = execution();

        for removal in PropertyRemoval::ALL {
            let probe = classify(
                &dishonest
                    .execute(&placement, &execution)
                    .expect("the stub runs"),
                &token(),
            );
            let control = classify(
                &dishonest
                    .execute_weakened(&placement, &execution, *removal)
                    .expect("the stub runs"),
                &token(),
            );
            assert_eq!(
                row_verdict(probe, control),
                RowVerdict::Unproven,
                "a backend ignoring its removal must not prove a row"
            );
        }
        assert_eq!(PropertyRemoval::ALL.len(), 10);

        // Discriminating: an honest backend, whose removal changes what the
        // capsule observes, proves the same row through the same pipeline.
        let honest =
            Stub::weakening_honestly(&[LIVENESS_MARKER, HELD], &[LIVENESS_MARKER, HELD_NOT]);
        let probe = classify(
            &honest.execute(&placement, &execution).expect("runs"),
            &token(),
        );
        let control = classify(
            &honest
                .execute_weakened(&placement, &execution, PropertyRemoval::MappedIdentity)
                .expect("runs"),
            &token(),
        );
        assert_eq!(row_verdict(probe, control), RowVerdict::Proven);
    }

    /// The same fails-closed property for the one control that **grants**.
    ///
    /// It needs its own case because the grant travels a second method: a
    /// backend that implemented `execute_weakened` honestly and delegated
    /// `execute_granted` to `execute` would pass the test above and still prove
    /// nothing about row 14.
    #[test]
    fn a_backend_ignoring_its_grant_yields_unproven() {
        let placement = placement();
        let execution = execution();

        let dishonest = Stub::ignoring_its_removal(&[LIVENESS_MARKER, HELD]);
        let probe = classify(
            &dishonest
                .execute(&placement, &execution)
                .expect("the stub runs"),
            &token(),
        );
        let control = classify(
            &dishonest
                .execute_granted(&placement, &execution, AuthorityGrant::AllCapabilities)
                .expect("the stub runs"),
            &token(),
        );
        assert_eq!(row_verdict(probe, control), RowVerdict::Unproven);

        // Discriminating: a backend whose grant changes what the capsule
        // observes proves the row through the same pipeline.
        let honest =
            Stub::weakening_honestly(&[LIVENESS_MARKER, HELD], &[LIVENESS_MARKER, HELD_NOT]);
        let probe = classify(
            &honest.execute(&placement, &execution).expect("runs"),
            &token(),
        );
        let control = classify(
            &honest
                .execute_granted(&placement, &execution, AuthorityGrant::AllCapabilities)
                .expect("runs"),
            &token(),
        );
        assert_eq!(row_verdict(probe, control), RowVerdict::Proven);
    }

    // ── VT-3: the admission algebra ────────────────────────────────────────

    /// One row of each non-proven kind, plus a proven one — a set with no
    /// `Proven` row would not tell `all` from `any`.
    #[test]
    fn admitted_requires_every_row_proven() {
        let rows = four_rows();
        let backend = Stub::ignoring_its_removal(&[LIVENESS_MARKER, HELD]);
        let host = host_with_shell();

        let mixed = ScriptedRunner::new(vec![
            RowVerdict::Proven,
            RowVerdict::Violated,
            RowVerdict::Unproven,
            RowVerdict::Indeterminate {
                arm: Which::Probe,
                detail: Indeterminacy::NoLiveness,
            },
        ]);
        let verdict = verdict_over(&backend, &host, &rows, Vec::new(), &mixed);
        assert_eq!(
            verdict.outcome,
            Admission::NotAdmitted {
                reason: NotAdmitted::Rows
            }
        );
        assert_eq!(verdict.rows.len(), 4);

        // Discriminating: the same rows, all proven, do admit — so it is the
        // non-proven rows that moved the outcome, and `any` would not have.
        let all_proven = ScriptedRunner::new(vec![RowVerdict::Proven; 4]);
        let verdict = verdict_over(&backend, &host, &rows, Vec::new(), &all_proven);
        assert_eq!(verdict.outcome, Admission::Admitted);

        // And each non-proven kind blocks on its own.
        for blocking in [
            RowVerdict::Violated,
            RowVerdict::Unproven,
            RowVerdict::Indeterminate {
                arm: Which::Control,
                detail: Indeterminacy::NoObservation,
            },
        ] {
            let one_bad = ScriptedRunner::new(vec![
                RowVerdict::Proven,
                RowVerdict::Proven,
                RowVerdict::Proven,
                blocking,
            ]);
            let verdict = verdict_over(&backend, &host, &rows, Vec::new(), &one_bad);
            assert_ne!(verdict.outcome, Admission::Admitted);
        }
    }

    /// Availability is read **before** any row runs, and the row set is
    /// non-empty so that *runs no row* is falsifiable at all.
    #[test]
    fn an_unavailable_backend_is_not_admitted_and_runs_no_row() {
        let backend = Stub::unavailable("bwrap", "install bubblewrap");
        let host = host_with_shell();
        let rows = four_rows();
        let runner = ScriptedRunner::new(Vec::new());

        let verdict = verdict_over(&backend, &host, &rows, Vec::new(), &runner);
        assert_eq!(
            verdict.outcome,
            Admission::NotAdmitted {
                reason: NotAdmitted::Unavailable {
                    missing: "bwrap".to_owned(),
                    remedy: "install bubblewrap".to_owned(),
                }
            }
        );
        assert_eq!(runner.calls.get(), 0, "no row may run");
        assert_eq!(backend.executions(), 0, "no capsule may be executed");
        assert!(verdict.rows.is_empty());

        // Discriminating: an available backend over the same rows and the same
        // runner runs every one of them, so the counter is not inert.
        let available = Stub::ignoring_its_removal(&[LIVENESS_MARKER, HELD]);
        let runner = ScriptedRunner::new(Vec::new());
        let verdict = verdict_over(&available, &host, &rows, Vec::new(), &runner);
        assert_eq!(runner.calls.get(), rows.len());
        assert_eq!(verdict.outcome, Admission::Admitted);
    }

    /// A host with no usable shell is *unavailable*, naming what is missing —
    /// never a violated row. `FixtureHost::new()` is the shell-less host.
    #[test]
    fn a_host_without_a_usable_shell_is_unavailable_not_violated() {
        let backend = Stub::ignoring_its_removal(&[LIVENESS_MARKER, HELD]);
        let verdict = verify(&backend, &FixtureHost::new(), TODAY.to_owned());

        match verdict.outcome {
            Admission::NotAdmitted {
                reason: NotAdmitted::Unavailable { missing, remedy },
            } => {
                assert!(!missing.is_empty(), "the missing fact must be named");
                assert_eq!(missing, SHELL);
                assert!(!remedy.is_empty(), "POL-002 facet 3 owes a remedy");
            }
            other => panic!("a shell-less host must be Unavailable, got {other:?}"),
        }
        assert!(verdict.rows.is_empty());

        // Discriminating: the same backend on a host that *has* a shell is not
        // refused for the shell.
        let verdict = verify(&backend, &host_with_shell(), TODAY.to_owned());
        assert_ne!(
            verdict.outcome,
            Admission::NotAdmitted {
                reason: NotAdmitted::Unavailable {
                    missing: SHELL.to_owned(),
                    remedy: super::SHELL_REMEDY.to_owned(),
                }
            }
        );
    }

    /// Table C is reported and never admitted on. A failed claim cannot block
    /// admission, and a passed one cannot rescue a non-proven row.
    #[test]
    fn auxiliary_outcomes_do_not_reach_the_admission() {
        let backend = Stub::ignoring_its_removal(&[LIVENESS_MARKER, HELD]);
        let host = host_with_shell();
        let rows = four_rows();
        let claim = Claim {
            section: "sec-2",
            name: "argv carries no caller text",
        };

        let auxiliary = vec![
            (
                claim,
                AuxOutcome::Failed("the claim did not hold".to_owned()),
            ),
            (
                Claim {
                    section: "sec-3",
                    name: "provisioning is idempotent",
                },
                AuxOutcome::Passed,
            ),
        ];
        let all_proven = ScriptedRunner::new(vec![RowVerdict::Proven; 4]);
        let verdict = verdict_over(&backend, &host, &rows, auxiliary.clone(), &all_proven);
        assert_eq!(
            verdict.outcome,
            Admission::Admitted,
            "a failed auxiliary claim must not block admission"
        );
        assert_eq!(verdict.auxiliary.len(), 2);
        assert_eq!(verdict.auxiliary[0].0.section, "sec-2");
        assert_eq!(verdict.auxiliary[0].0.name, "argv carries no caller text");

        // And the other direction: a passing claim cannot rescue a broken row.
        let one_bad = ScriptedRunner::new(vec![
            RowVerdict::Proven,
            RowVerdict::Proven,
            RowVerdict::Proven,
            RowVerdict::Violated,
        ]);
        let verdict = verdict_over(&backend, &host, &rows, auxiliary, &one_bad);
        assert_eq!(
            verdict.outcome,
            Admission::NotAdmitted {
                reason: NotAdmitted::Rows
            }
        );

        // The algebra itself: `admission` is given the rows alone.
        assert_eq!(
            admission(&[(RowId::Axis(Axis::Checkout), RowVerdict::Proven)]),
            Admission::Admitted
        );
    }

    /// The green-skip `DEC-156` forbids, wearing the opposite mask: a *skipped*
    /// auxiliary claim must not silently block admission either.
    #[test]
    fn a_skipped_auxiliary_claim_leaves_the_verdict_admitted() {
        let backend = Stub::ignoring_its_removal(&[LIVENESS_MARKER, HELD]);
        let host = host_with_shell();
        let rows = four_rows();
        let auxiliary = vec![(
            Claim {
                section: "sec-5",
                name: "capacity is advisory",
            },
            AuxOutcome::Skipped("no second filesystem on this host".to_owned()),
        )];
        let all_proven = ScriptedRunner::new(vec![RowVerdict::Proven; 4]);

        let verdict = verdict_over(&backend, &host, &rows, auxiliary, &all_proven);
        assert_eq!(verdict.outcome, Admission::Admitted);
        assert_eq!(verdict.auxiliary.len(), 1);
    }

    /// `DEC-156`: admission is a **recorded verdict** naming backend, host and
    /// date. The outcome is deliberately not asserted — tables A and B are empty
    /// at this phase, so any assertion on it would be vacuous.
    #[test]
    fn the_verdict_names_backend_host_and_date() {
        let backend = Stub::ignoring_its_removal(&[LIVENESS_MARKER, HELD]);
        let verdict = verify(&backend, &host_with_shell(), TODAY.to_owned());

        assert_eq!(verdict.backend, WITNESS_ID);
        assert_eq!(verdict.date, TODAY);
        assert!(!verdict.host.os.is_empty());
        assert!(!verdict.host.kernel.is_empty());
        assert!(!verdict.host.arch.is_empty());
    }

    /// The covering property, asserted over the **code's own** tables and over a
    /// hand-built pair that shares a row id.
    ///
    /// This says nothing about any document: it is a property of the two
    /// functions above it, and the compiler's own check is that `RowId::Property`
    /// keys the verdict.
    #[test]
    fn every_row_id_is_covered_by_exactly_one_table() {
        // Vacuous at PHASE-07 — `table_a` and `table_b` are both empty until
        // PHASE-09 and PHASE-10 populate them. Asserted anyway so it starts
        // biting the moment a row lands.
        let a = super::table_a();
        let b = super::table_b();
        assert!(row_ids_in_more_than_one_table(&[&a, &b]).is_empty());

        // The assertion that has force today: a pair deliberately sharing one
        // row id.
        let mut first = four_rows();
        let shared = one_more_row();
        first.push(shared.clone());
        let second = vec![
            shared,
            row(
                Axis::Repository,
                ArmShape::Single(a_probe()),
                Delta::SharedRoot,
            ),
        ];

        let overlapping = row_ids_in_more_than_one_table(&[&first, &second]);
        assert_eq!(
            overlapping,
            vec![RowId::Axis(Axis::Repository), RowId::Axis(Axis::Process)]
        );

        // Discriminating: two disjoint tables report nothing.
        let disjoint_a = vec![row(
            Axis::Checkout,
            ArmShape::Single(a_probe()),
            Delta::SharedRoot,
        )];
        let disjoint_b = vec![row(
            Axis::Process,
            ArmShape::Single(a_probe()),
            Delta::NetworkPermitted,
        )];
        assert!(row_ids_in_more_than_one_table(&[&disjoint_a, &disjoint_b]).is_empty());
    }

    // ── The fixture root (`EX-2`, `D1`, `T1`) ──────────────────────────────

    /// `EX-2`: the run's root is on real disk, never tmpfs.
    ///
    /// Asserted against an independent `statfs` on the chosen root rather than
    /// against the constructor's own opinion of it, and against
    /// `std::env::temp_dir()` — the route `D1` rejects — where this host makes
    /// that route a tmpfs. `M20` is the sole evidence for this criterion and
    /// nothing else in the suite would notice its damage: on tmpfs both capacity
    /// claims still agree with their own `statvfs`.
    #[test]
    fn the_fixture_root_is_on_a_non_tmpfs_filesystem() {
        let root = TempRoot::new(&SystemHost).expect("this host offers a real-disk scratch root");
        assert!(root.path().is_dir());

        let stat = rustix::fs::statfs(root.path()).expect("the chosen root can be probed");
        assert_ne!(
            stat.f_type,
            TMPFS_MAGIC,
            "the fixture root at {} is on tmpfs",
            root.path().display()
        );

        // The refusal itself, exercised where the host can supply a tmpfs.
        // `std::env::temp_dir()` is `/tmp` and `/tmp` is tmpfs on the host this
        // was built against (`A3`); guarded rather than asserted flat, so a host
        // whose `/tmp` is real disk does not red a working mechanism.
        let temporary = std::env::temp_dir();
        if !on_real_disk(&temporary) {
            assert!(
                prepare_root(&temporary).is_none(),
                "a tmpfs base was accepted as a fixture root"
            );
        }

        // Two roots in one process do not collide — the nonce, not the clock.
        let second = TempRoot::new(&SystemHost).expect("a second root is available");
        assert_ne!(root.path(), second.path());
    }

    /// `Drop` is the whole of cleanup (invariant 8), and it is recursive.
    ///
    /// The root is populated with a nested directory and a file before it drops,
    /// so a non-recursive removal fails here rather than succeeding on an empty
    /// directory. `M21` is its mutation.
    #[test]
    fn the_fixture_root_is_removed_when_the_fixture_is_dropped() {
        let path = {
            let root = TempRoot::new(&SystemHost).expect("this host offers a real-disk root");
            let nested = root.path().join("nested");
            std::fs::create_dir(&nested).expect("the root is writable");
            let mut file = std::fs::File::create(nested.join("payload"))
                .expect("a file can be created beneath the root");
            file.write_all(b"payload").expect("the payload is written");
            assert!(nested.is_dir());
            root.path().to_path_buf()
        };

        assert!(
            !path.exists(),
            "the fixture root at {} survived its Drop",
            path.display()
        );
    }

    // ── The fixture (`T2`, `EX-1`, `EX-3`, `EX-4`, `EX-5`) ─────────────────

    /// `mountinfo` escapes whitespace in the mount-point field, and a mount
    /// under a directory with a space in its name is the discriminating case:
    /// splitting on whitespace without decoding truncates it to a *prefix* that
    /// still `stat`s, so the wrong filesystem is selected silently.
    #[test]
    fn a_mount_point_is_decoded_rather_than_truncated() {
        let table = "\
36 25 0:32 / /run/media/my\\040disk rw,relatime shared:18 - ext4 /dev/sdb1 rw\n\
37 25 0:33 / /tab\\011here rw - tmpfs tmpfs rw\n\
38 25 0:34 / /plain rw - ext4 /dev/sdc1 rw\n";

        assert_eq!(
            mount_points(table),
            vec![
                PathBuf::from("/run/media/my disk"),
                PathBuf::from("/tab\there"),
                PathBuf::from("/plain"),
            ]
        );
    }

    /// A trailing lone backslash, and an escape that is not three octal digits,
    /// are passed through rather than swallowing the rest of the field.
    #[test]
    fn a_malformed_mount_escape_is_passed_through() {
        assert_eq!(decode_mount_field("/a\\zz9b"), "/a\\zz9b");
        assert_eq!(decode_mount_field("/trailing\\"), "/trailing\\");
    }

    /// The *top-level* ancestor, not the entry: a dynamically linked executable
    /// needs its loader and libraries, which on a store-based host live under
    /// sibling directories of the same top level.
    #[test]
    fn a_readable_root_is_the_top_level_ancestor_of_an_entry() {
        assert_eq!(
            top_level_ancestor(Path::new("/nix/store/abc-git/bin")),
            Some(PathBuf::from("/nix"))
        );
        assert_eq!(
            top_level_ancestor(Path::new("/usr")),
            Some(PathBuf::from("/usr"))
        );
        assert_eq!(top_level_ancestor(Path::new("/")), None);
        assert_eq!(top_level_ancestor(Path::new("relative/bin")), None);
    }

    /// Invariant 7 / `VA-2`: no readable root may contain the operator's home,
    /// even when a `PATH` entry lives there.
    ///
    /// The discriminating fixture is a `PATH` whose entries span **both** a
    /// lawful top level and the operator's — a `PATH` of store paths alone
    /// passes under an implementation with no exclusion at all.
    #[test]
    fn no_readable_root_contains_the_operators_home() {
        let host = FixtureHost::default()
            .with_env(HOME_VARIABLE, "/home/operator")
            .with_env("PATH", "/nix/store/abc-git/bin:/home/operator/.local/bin")
            .with_resolution(SHELL, "/nix/store/abc-bash/bin/sh")
            .with_resolution("/nix/store/abc-git/bin", "/nix/store/abc-git/bin")
            .with_resolution("/home/operator/.local/bin", "/home/operator/.local/bin");

        let roots = system_readable_roots(&host, Path::new("/tmp/fixture-root"));

        assert_eq!(roots, vec![PathBuf::from("/nix")]);
    }

    /// A fixture root that is itself a candidate's top level is excluded too —
    /// otherwise the suite binds its own control plane in as a readable input.
    #[test]
    fn no_readable_root_contains_the_fixture_root() {
        let host = FixtureHost::default()
            .with_env("PATH", "/scratch/bin")
            .with_resolution(SHELL, "/scratch/bin/sh")
            .with_resolution("/scratch/bin", "/scratch/bin");

        assert!(system_readable_roots(&host, Path::new("/scratch/run-1")).is_empty());
    }

    /// The synthesized document is the one `provision` parses — asserted by
    /// parsing it with the same function, not by eyeballing the format string.
    /// `EX-3`: its `readable-roots` must carry a shell, or every payload is
    /// `NotExecutable` and the run is indeterminate for a reason it never names.
    #[test]
    fn the_synthesized_table_is_the_one_provision_parses() {
        let capsule_root = PathBuf::from("/scratch/run-1/capsules");
        let document = capsule_config_document(
            &capsule_root,
            &[PathBuf::from("/nix"), PathBuf::from("/bin")],
        );

        let parsed = crate::config::parse_capsule_config(&document)
            .expect("the synthesized table is the shape provision parses");

        assert_eq!(
            parsed.readable_roots(),
            [PathBuf::from("/nix"), PathBuf::from("/bin")]
        );
        assert!(
            document.contains("[interpretation]"),
            "provision reads the policy from the same document: {document}"
        );
    }

    /// `EX-4` and `VA-2` in one assertion: every artefact the fixture builds
    /// lies beneath its own root, so no arm can name the operator's repository,
    /// `.doctrine/`, credentials or export.
    ///
    /// The decoys are the discriminating part — a fixture that pointed
    /// `decoy_credential` at the operator's real `~/.gitconfig` would satisfy
    /// every *other* claim in this suite.
    #[test]
    fn every_artefact_the_fixture_builds_lies_beneath_its_own_root() {
        let fixture = Fixture::new(&SystemHost).expect("this host can host the fixture");
        let root = fixture.root().to_path_buf();

        let beneath: Vec<&Path> = vec![
            fixture.project_root(),
            fixture.capsule_root(),
            fixture.decoy_credential(),
            fixture.decoy_repository(),
            fixture.decoy_readable_input(),
            fixture.own_export(),
            fixture.decoy_undeclared(),
            fixture.decoy_executable(),
            fixture.decoy_descriptor(),
        ];
        for path in beneath {
            assert!(
                path.starts_with(&root),
                "{} escapes the fixture root {}",
                path.display(),
                root.display()
            );
        }

        for scope in fixture.scopes().members() {
            assert!(
                scope.starts_with(&root),
                "the forbidden scope {} names a region outside the fixture",
                scope.display()
            );
        }

        // Row 3's decoys are lawfully widenable precisely because they are not
        // scopes; a fixture that named them would make row 3 unrunnable.
        let scoped: Vec<&Path> = fixture.scopes().members().collect();
        assert!(!scoped.contains(&fixture.decoy_credential()));
        assert!(!scoped.contains(&fixture.decoy_repository()));

        let address = fixture
            .listener()
            .local_addr()
            .expect("the listener is bound");
        assert!(
            address.ip().is_loopback(),
            "row 5's target is not the internet"
        );
        assert_ne!(address.port(), 0);
    }

    /// `EX-13`'s two halves in one assertion: row 10's three decoys survive an
    /// `exec`, and every descriptor the trusted side holds *for real* does not.
    ///
    /// The first half is the one with a trap under it. Rust opens its own files
    /// `O_CLOEXEC` (PHASE-05 `VT-4`), so a fixture that reached for `File::open`
    /// would hand the row three descriptors the sweep never had to close — and
    /// the removal would change nothing while the row still passed.
    #[test]
    fn the_row_ten_decoys_are_the_only_inheritable_descriptors() {
        let fixture = Fixture::new(&SystemHost).expect("this host can host the fixture");
        let decoys = fixture
            .inheritable_decoys()
            .expect("the fixture can open row 10's decoys");

        for descriptor in decoys.descriptors() {
            assert!(
                is_inheritable(descriptor),
                "a row 10 decoy is close-on-exec, so removing the sweep would change nothing"
            );
        }

        // Held for real, and so never inheritable: the pair's far end, and the
        // fixture's own listener.
        assert!(
            !is_inheritable(decoys.retained_peer()),
            "the socket pair's retained end leaks into the capsule"
        );
        assert!(
            !is_inheritable(fixture.listener()),
            "row 5's trusted-side listener leaks into the capsule"
        );
    }

    /// The write-only decoy is reachable by no name, so the control arm's
    /// mutation dies with the descriptor (`EX-13`, `VA-1`'s write half).
    ///
    /// Two assertions, and the test is vacuous without the second: the file is
    /// genuinely writable, and it adds no entry to the fixture root. A decoy
    /// that could not be written proves nothing about a capsule that inherits
    /// it.
    #[test]
    fn the_write_only_decoy_is_reachable_by_no_name() {
        let fixture = Fixture::new(&SystemHost).expect("this host can host the fixture");
        let before = entry_names(fixture.root());

        let decoys = fixture
            .inheritable_decoys()
            .expect("the fixture can open row 10's decoys");
        let [_readable, writable, _socket] = decoys.descriptors();

        let written = rustix::io::write(writable, b"the control arm's mutation\n")
            .expect("the write-only decoy is writable");
        assert_ne!(written, 0, "the write-only decoy accepted no bytes");

        assert_eq!(
            before,
            entry_names(fixture.root()),
            "the write-only decoy was linked into the fixture root"
        );

        // Unnamed, but still the fixture's own root's filesystem — invariant 7
        // holds for a file with no name as much as for one with a name.
        let target = std::fs::read_link(format!("/proc/self/fd/{}", writable.as_raw_fd()))
            .expect("the kernel names the decoy's origin");
        let shown = target.to_string_lossy().into_owned();
        assert!(
            shown.starts_with(&fixture.root().to_string_lossy().into_owned()),
            "the write-only decoy was created outside the fixture root: {shown}"
        );
        assert!(
            shown.ends_with(DELETED_SUFFIX),
            "the write-only decoy still has a link: {shown}"
        );
    }

    /// A decoy set is per-arm state, not fixture state (`F-26`).
    ///
    /// The backend's parent-side sweep marks the *parent's* descriptors
    /// close-on-exec, permanently. Row 10's confining probe arm therefore closes
    /// whatever set was open when it ran, and a set held once per fixture would
    /// leave the control arm nothing to leak — a row passing for no reason.
    #[test]
    fn a_decoy_set_opened_after_a_sweep_is_inheritable_again() {
        let fixture = Fixture::new(&SystemHost).expect("this host can host the fixture");
        let swept = fixture
            .inheritable_decoys()
            .expect("the fixture can open row 10's decoys");

        // What `mark_inherited_descriptors_close_on_exec` does to the parent,
        // applied here to this set alone rather than to the whole process.
        for descriptor in swept.descriptors() {
            rustix::io::fcntl_setfd(descriptor, rustix::io::FdFlags::CLOEXEC)
                .expect("the sweep can close a decoy");
            assert!(!is_inheritable(descriptor));
        }

        let fresh = fixture
            .inheritable_decoys()
            .expect("a second decoy set opens");
        for descriptor in fresh.descriptors() {
            assert!(
                is_inheritable(descriptor),
                "the control arm inherited a set the probe arm's sweep had already closed"
            );
        }
    }

    /// How many descriptors above the standard streams row 10's payload counts.
    #[test]
    fn a_decoy_set_is_three_descriptors_of_three_kinds() {
        let fixture = Fixture::new(&SystemHost).expect("this host can host the fixture");
        let decoys = fixture
            .inheritable_decoys()
            .expect("the fixture can open row 10's decoys");

        assert_eq!(decoys.descriptors().len(), InheritableDecoys::COUNT);
        let numbers: BTreeSet<i32> = decoys
            .descriptors()
            .iter()
            .map(std::os::fd::AsRawFd::as_raw_fd)
            .collect();
        assert_eq!(
            numbers.len(),
            InheritableDecoys::COUNT,
            "two of row 10's decoys are the same descriptor"
        );
        assert!(
            numbers.iter().all(|number| *number > 2),
            "a decoy sits at or below the standard streams, which is row 12's mechanism"
        );
    }

    /// The leaf names directly beneath `path`, sorted — a listing that changes
    /// only when something is *linked* there.
    fn entry_names(path: &Path) -> BTreeSet<std::ffi::OsString> {
        let Ok(entries) = std::fs::read_dir(path) else {
            return BTreeSet::new();
        };
        entries
            .filter_map(|entry| entry.ok().map(|entry| entry.file_name()))
            .collect()
    }

    /// The second-filesystem selection is only informative if the path it names
    /// is genuinely on another device — otherwise Table C's conditional row
    /// compares a figure with itself and passes for free. `M17`, `M18`.
    #[test]
    fn the_second_filesystem_is_on_another_device_or_absent() {
        let fixture = Fixture::new(&SystemHost).expect("this host can host the fixture");

        let Some(other) = fixture.second_filesystem() else {
            // Not an assertion: `None` is a lawful answer about the host, and
            // it is exactly what makes Table C's conditional row *skip*. Said
            // out loud so a vacuous pass is visible in the run's output rather
            // than indistinguishable from a real one.
            eprintln!("this host offers no second filesystem; the check is vacuous here");
            return;
        };
        eprintln!("second filesystem: {}", other.display());
        let here = rustix::fs::stat(fixture.capsule_root()).expect("the capsule root stats");
        let there = rustix::fs::stat(other).expect("the selected mount stats");

        assert_ne!(
            here.st_dev,
            there.st_dev,
            "{} is on the same device as the capsule root",
            other.display()
        );
        assert_ne!(available_bytes_of(other), Some(0));
        assert_ne!(
            available_bytes_of(other),
            available_bytes_of(fixture.capsule_root())
        );
    }

    /// `A2`: on every host this suite is developed against the selection
    /// answers `Some`, so the `None` branch — the one that makes Table C's
    /// conditional capacity row report *skipped* rather than pass quietly —
    /// would ship untested. The mount table is a parameter precisely so it can
    /// be forced here.
    ///
    /// Two tables, and each forces `None` for a different reason: a host with
    /// one filesystem, and a host whose only other mounts are pseudo-
    /// filesystems reporting no space at all.
    #[test]
    fn a_host_with_no_second_filesystem_selects_none() {
        let fixture = Fixture::new(&SystemHost).expect("this host can host the fixture");
        let root = fixture.root();

        assert_eq!(second_filesystem(fixture.capsule_root(), root, ""), None);
        assert_eq!(
            second_filesystem(
                fixture.capsule_root(),
                root,
                "36 25 0:32 / /proc rw - proc proc rw\n\
                 37 25 0:33 / /sys rw - sysfs sysfs rw\n",
            ),
            None,
            "a pseudo-filesystem reporting no space is not a second filesystem"
        );
    }

    /// `T2` step 2: the fixture repository must hold an object the contracted
    /// base cannot reach, or `the_clones_object_set_is_exactly_the_exports` is
    /// vacuous — it would pass under a clone that copied the whole repository.
    #[test]
    fn the_fixture_repository_holds_an_object_unreachable_from_the_base() {
        let fixture = Fixture::new(&SystemHost).expect("this host can host the fixture");

        let reachable = git(
            fixture.project_root(),
            &["rev-list", "--objects", fixture.base().as_str()],
        )
        .expect("the base is a commit");
        let all = git(fixture.project_root(), &["rev-list", "--objects", "--all"])
            .expect("the repository has refs");

        assert!(
            all.len() > reachable.len(),
            "the fixture repository holds nothing the base cannot reach"
        );
        // And the working tree matches the base, so `provision`'s working-tree
        // read of `[capsule]` sees the same document the base commits.
        assert_eq!(
            git(fixture.project_root(), &["rev-parse", "HEAD"]).expect("a detached HEAD"),
            fixture.base().as_str()
        );
    }

    // ── Table C: the four auxiliary claims (`EX-14`…`EX-16`) ───────────────

    /// `EX-14` executed: a capsule overwrites its own copy of the policy
    /// document and the policy in force is unmoved. `M14` is the mutation that
    /// makes [`policy_in_force`] re-read the capsule's copy.
    ///
    /// The vacuity guard is not decoration — the claim's rewrite is a textual
    /// substitution, and a document that never held the empty form would be
    /// "rewritten" to itself and pass against nothing.
    #[test]
    fn rewriting_doctrine_toml_inside_a_capsule_does_not_change_the_bound_policy() {
        let fixture = Fixture::new(&SystemHost).expect("this host can host the fixture");
        let backend = BubblewrapBackend::new(&SystemHost);
        assert_eq!(
            backend.availability(),
            Availability::Available,
            "table C's executed claims need a real mechanism"
        );

        let document = std::fs::read_to_string(fixture.project_root().join(DOCTRINE_TOML))
            .expect("the fixture writes a policy document");
        assert!(
            document.contains(EMPTY_FORBIDDEN_EXECUTABLES),
            "the rewrite has nothing to replace, so the claim would be vacuous"
        );

        assert_eq!(
            read_once_claim(&backend, &SystemHost, &fixture),
            AuxOutcome::Passed
        );
    }

    /// `EX-15` executed: the clone holds the export's objects and no others,
    /// decided trusted-side.
    ///
    /// The three direct drives are what make the comparison's *shape* the thing
    /// under test rather than this host's luck: ⊇ alone (`M15`) passes a clone
    /// that dragged extra objects in, and two empty sets are equal.
    #[test]
    fn the_clones_object_set_is_exactly_the_exports() {
        let fixture = Fixture::new(&SystemHost).expect("this host can host the fixture");
        let backend = BubblewrapBackend::new(&SystemHost);
        assert_eq!(backend.availability(), Availability::Available);

        assert_eq!(
            object_set_claim(&backend, &SystemHost, &fixture),
            AuxOutcome::Passed
        );

        let names = |names: &[&str]| -> BTreeSet<String> {
            names.iter().map(|name| (*name).to_owned()).collect()
        };
        let export = names(&["a", "b"]);
        assert_eq!(object_sets_agree(&export, &export), Ok(()));
        assert!(
            object_sets_agree(&names(&["a", "b", "c"]), &export).is_err(),
            "a superset is not the export's object set"
        );
        assert!(
            object_sets_agree(&names(&["a"]), &export).is_err(),
            "a subset is not the export's object set"
        );
        assert!(
            object_sets_agree(&BTreeSet::new(), &BTreeSet::new()).is_err(),
            "a capsule whose git never ran names nothing, and nothing is not evidence"
        );
    }

    /// `EX-16` executed, unconditional half: the probe reads real space at the
    /// path it is given. `REQ-461`'s executed closure rests on this row, so it
    /// never skips. `M16`.
    #[test]
    fn the_capacity_probe_reads_real_space_at_the_path_it_is_given() {
        let fixture = Fixture::new(&SystemHost).expect("this host can host the fixture");
        let root = fixture.capsule_root();

        assert_eq!(capacity_claim(&SystemHost, root), AuxOutcome::Passed);

        // Re-derived here rather than trusted from the claim: a claim that
        // checked the probe against itself would pass under any implementation.
        // Bracketed for the reason `agreed_capacity` documents — sibling tests
        // in this very suite are writing to this filesystem as it is read.
        let unit = |path: &Path| {
            rustix::fs::statvfs(path)
                .expect("the capsule root stats")
                .f_frsize
        };
        let before = available_bytes_of(root).expect("the capsule root stats");
        let reported = SystemHost
            .available_bytes(root)
            .expect("this host answers about its own scratch root");
        let after = available_bytes_of(root).expect("the capsule root stats");

        assert_ne!(before, 0, "the fixture root reports no space at all");
        assert!(
            reported >= before.min(after) - unit(root)
                && reported <= before.max(after) + unit(root),
            "{reported} is outside the bracket two independent readings put it in \
             ({before}, {after})"
        );
    }

    /// `EX-16` executed, conditional half: the probe answers about the
    /// filesystem the path is on, not about some fixed one. `M17`, `M18` red
    /// this row and leave the unconditional one green — that separation is the
    /// signal.
    #[test]
    fn the_capacity_probe_reads_the_filesystem_the_capsule_root_is_on() {
        let fixture = Fixture::new(&SystemHost).expect("this host can host the fixture");

        let Some(elsewhere) = fixture.second_filesystem() else {
            eprintln!("this host offers no second filesystem; the claim skips here");
            return;
        };

        assert_eq!(
            capacity_filesystem_claim(&SystemHost, fixture.capsule_root(), Some(elsewhere)),
            AuxOutcome::Passed
        );
    }

    /// `A2`: this host always takes the `Some` branch, so the skip ships
    /// untested unless the absence is forced. The reason must **name** the
    /// absence — a skip with an empty reason is a silent pass wearing a label.
    /// `M19`.
    #[test]
    fn a_missing_second_filesystem_reports_skipped_naming_the_reason() {
        let fixture = Fixture::new(&SystemHost).expect("this host can host the fixture");

        let outcome = capacity_filesystem_claim(&SystemHost, fixture.capsule_root(), None);

        let AuxOutcome::Skipped(reason) = outcome else {
            panic!("a missing second filesystem is a skip, not {outcome:?}");
        };
        assert!(
            reason.contains(NO_SECOND_FILESYSTEM),
            "the reason does not name the absence: {reason}"
        );
        assert!(
            reason.contains(&fixture.capsule_root().display().to_string()),
            "the reason does not name what there was nothing to tell apart from: {reason}"
        );
    }

    /// The fixture is what all four claims need, so a fixture that cannot be
    /// built skips all four — reported rather than omitted, so the report's
    /// shape does not change with the host's luck.
    #[test]
    fn a_fixture_that_cannot_be_built_skips_every_claim_naming_the_fault() {
        let fault = "the fixture root could not be made";

        let skipped = claims_skipped(fault);

        assert_eq!(
            skipped
                .iter()
                .map(|(claim, _)| *claim)
                .collect::<Vec<Claim>>(),
            vec![
                READ_ONCE_CLAIM,
                OBJECT_SET_CLAIM,
                CAPACITY_CLAIM,
                CAPACITY_FILESYSTEM_CLAIM
            ],
            "the report carries all four claims however the host behaved"
        );
        assert!(
            skipped.iter().all(
                |(_, outcome)| matches!(outcome, AuxOutcome::Skipped(reason) if reason == fault)
            ),
            "a skip that does not carry the fault is a silent pass: {skipped:?}"
        );
    }

    /// `Bound` carries the two removals `PropertyRemoval::ResourceBound` stands
    /// for — nine variants, ten removals.
    #[test]
    fn the_resource_bound_variant_carries_two_removals() {
        let bounds: Vec<Bound> = PropertyRemoval::ALL
            .iter()
            .filter_map(|removal| match removal {
                PropertyRemoval::ResourceBound(bound) => Some(*bound),
                _ => None,
            })
            .collect();
        assert_eq!(bounds, vec![Bound::FileSize, Bound::Wall]);
    }

    // ── VA-4: one axis, one delta (`T4`) ───────────────────────────────────
    //
    // `M1`…`M10` bite here. Each case is a prediction about *one* axis, stated
    // as the multiset of argv words it adds and removes against the confining
    // baseline plus the spawn options it switches off — so a delta that also
    // moves a second thing fails its own case, and a delta that moves the second
    // thing *instead* fails two.

    const CONFINING_OPTIONS: SpawnOptions = SpawnOptions {
        wall_bounded: true,
        file_size_capped: true,
        descriptors_closed: true,
        parent_owned_stdio: true,
    };

    /// A non-empty `--setenv` list, because `M5` predicts that dropping it reds
    /// the environment case and an empty list makes dropping it a no-op.
    fn capsule_environment() -> Vec<(&'static str, String)> {
        vec![
            ("DOCTRINE_CAPSULE", "1".to_owned()),
            ("HOME", WORKING_DIRECTORY.to_owned()),
        ]
    }

    fn assembled(weakening: Option<&ProfileWeakening>) -> Vec<String> {
        confinement_argv(
            &placement(),
            &capsule_environment(),
            9,
            &argv(&["/bin/true"]),
            weakening,
        )
    }

    /// The multiset difference between two argvs as `(removed, added)`, both
    /// sorted — so a case says *what* moved without restating the whole order,
    /// which `argv_is_assembled_in_the_declared_order` already holds.
    fn word_delta(baseline: &[String], variant: &[String]) -> (Vec<String>, Vec<String>) {
        let mut counts: BTreeMap<&str, isize> = BTreeMap::new();
        for word in baseline {
            *counts.entry(word.as_str()).or_default() -= 1;
        }
        for word in variant {
            *counts.entry(word.as_str()).or_default() += 1;
        }

        let mut removed = Vec::new();
        let mut added = Vec::new();
        for (word, count) in counts {
            let sink = if count < 0 { &mut removed } else { &mut added };
            for _ in 0..count.abs() {
                sink.push(word.to_owned());
            }
        }
        (removed, added)
    }

    fn sorted(words: &[&str]) -> Vec<String> {
        let mut owned: Vec<String> = words.iter().map(|word| (*word).to_owned()).collect();
        owned.sort();
        owned
    }

    fn assert_axis(
        label: &str,
        weakening: &ProfileWeakening,
        removed: &[&str],
        added: &[&str],
        options: SpawnOptions,
    ) {
        assert_eq!(
            word_delta(&assembled(None), &assembled(Some(weakening))),
            (sorted(removed), sorted(added)),
            "{label}: the argv delta is not exactly its own"
        );
        assert_eq!(
            SpawnOptions::under(Some(weakening)),
            options,
            "{label}: the spawn options are not exactly its own"
        );
    }

    #[test]
    fn each_removal_changes_exactly_its_own_flags() {
        let (stdio, _capture) = OwnedStdio::opened().expect("a socket pair and a decoy body");

        assert_axis(
            "working directory",
            &weakening_for(PropertyRemoval::WorkingDirectory, None),
            &["--chdir", WORKING_DIRECTORY],
            &[],
            CONFINING_OPTIONS,
        );
        assert_axis(
            "teardown",
            &weakening_for(PropertyRemoval::Teardown, None),
            &["--die-with-parent"],
            &[],
            CONFINING_OPTIONS,
        );
        assert_axis(
            "process visibility",
            &weakening_for(PropertyRemoval::ProcessVisibility, None),
            &["--unshare-all"],
            &[
                "--unshare-user-try",
                "--unshare-ipc",
                "--unshare-net",
                "--unshare-uts",
                "--unshare-cgroup-try",
            ],
            CONFINING_OPTIONS,
        );
        assert_axis(
            "file-size bound",
            &weakening_for(PropertyRemoval::ResourceBound(Bound::FileSize), None),
            &[],
            &[],
            SpawnOptions {
                file_size_capped: false,
                ..CONFINING_OPTIONS
            },
        );
        assert_axis(
            "wall bound",
            &weakening_for(PropertyRemoval::ResourceBound(Bound::Wall), None),
            &[],
            &[],
            SpawnOptions {
                wall_bounded: false,
                ..CONFINING_OPTIONS
            },
        );
        assert_axis(
            "inputs writable",
            &weakening_for(PropertyRemoval::InputsWritable, None),
            &["--ro-bind", "--ro-bind"],
            &["--bind", "--bind"],
            CONFINING_OPTIONS,
        );
        assert_axis(
            "descriptors",
            &weakening_for(PropertyRemoval::DescriptorsClosed, None),
            &[],
            &[],
            SpawnOptions {
                descriptors_closed: false,
                ..CONFINING_OPTIONS
            },
        );
        assert_axis(
            "environment cleared",
            &weakening_for(PropertyRemoval::EnvCleared, None),
            &["--clearenv"],
            &[],
            CONFINING_OPTIONS,
        );
        assert_axis(
            "stdio owned",
            &weakening_for(PropertyRemoval::StdioOwned, Some(stdio)),
            &[],
            &[],
            SpawnOptions {
                parent_owned_stdio: false,
                ..CONFINING_OPTIONS
            },
        );
        assert_axis(
            "mapped identity",
            &weakening_for(PropertyRemoval::MappedIdentity, None),
            &["--uid", "1000", "--gid", "1000"],
            &[],
            CONFINING_OPTIONS,
        );
        assert_axis(
            "capability grant",
            &weakening_granting(AuthorityGrant::AllCapabilities),
            &[],
            &["--cap-add", "ALL"],
            CONFINING_OPTIONS,
        );
    }

    /// The `--setenv` list survives every axis byte-for-byte, including the one
    /// that drops `--clearenv` — `VA-4` names this, and the multiset above cannot
    /// see a *reordering* of it.
    #[test]
    fn every_axis_leaves_the_setenv_list_byte_identical() {
        let (stdio, _capture) = OwnedStdio::opened().expect("a socket pair and a decoy body");
        let expected = setenv_list(&assembled(None));
        assert_eq!(expected.len(), capsule_environment().len());

        let axes: Vec<ProfileWeakening> = PropertyRemoval::ALL
            .iter()
            .map(|removal| {
                weakening_for(
                    *removal,
                    match removal {
                        PropertyRemoval::StdioOwned => {
                            Some(OwnedStdio::opened().expect("a socket pair").0)
                        }
                        _ => None,
                    },
                )
            })
            .chain(std::iter::once(weakening_granting(
                AuthorityGrant::AllCapabilities,
            )))
            .collect();
        drop(stdio);

        for axis in &axes {
            assert_eq!(
                setenv_list(&assembled(Some(axis))),
                expected,
                "{axis:?} moved the `--setenv` list"
            );
        }
    }

    /// The `(name, value)` pairs following each `--setenv`, in order.
    fn setenv_list(assembled: &[String]) -> Vec<(String, String)> {
        assembled
            .windows(3)
            .filter(|window| window.first().is_some_and(|word| word == "--setenv"))
            .filter_map(|window| match window {
                [_, name, value] => Some((name.clone(), value.clone())),
                _ => None,
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Running an arm (`T6`, `EX-6`, `EX-7`)
    // -----------------------------------------------------------------------
    //
    // Tables A and B are empty this phase, so nothing else runs a `Sequential`
    // or a `Concurrent` shape. These are the whole of their coverage until
    // PHASE-09 and PHASE-10 (`C3`: the `VT` keywords are a floor).

    /// The fixture's bounds and stdio around a payload's argv — the shape the
    /// real harness's closure will have.
    fn executing(argv: &Argv) -> Execution {
        Execution::new(
            argv.clone(),
            CapsuleEnv::complete(),
            Duration::from_secs(5),
            ByteCount::from_bytes(1024),
            CapsuleStdio::EmptyInputCapturedOutput,
        )
    }

    /// Counts what `EX-6` is about: one transaction per capsule, never a reuse.
    fn counting_capsules(count: &Cell<usize>) -> impl Fn() -> Result<CapsulePlacement, String> {
        move || {
            count.set(count.get() + 1);
            Ok(placement())
        }
    }

    fn arm<'a>(
        backend: &'a Stub,
        capsule: &'a dyn Fn() -> Result<CapsulePlacement, String>,
        live: &'a dyn Fn(HostPid) -> bool,
        under: Under,
    ) -> Arm<'a> {
        Arm {
            backend,
            capsule,
            execution: &executing,
            live,
            noticed: &|_pid| (),
            under,
        }
    }

    const ALWAYS_LIVE: &dyn Fn(HostPid) -> bool = &|_pid| true;
    const NEVER_LIVE: &dyn Fn(HostPid) -> bool = &|_pid| false;

    #[test]
    fn a_single_arm_runs_one_capsule_and_reads_it() {
        let backend = Stub::scripted(vec![ran(&[LIVENESS_MARKER, HELD])], ran(&[]));
        let count = Cell::new(0);
        let capsule = counting_capsules(&count);

        assert_eq!(
            run_arm(
                &arm(&backend, &capsule, ALWAYS_LIVE, Under::Confining),
                &ArmShape::Single(a_probe())
            ),
            ArmResult::Held
        );
        assert_eq!(count.get(), 1);
    }

    /// `EX-6`, and the reading: the arm is the *reader's*, not the writer's.
    /// The two payloads observe opposite things here, so a harness that read the
    /// writer would answer `Held` where this answers `Failed`.
    #[test]
    fn a_sequential_arm_provisions_one_transaction_per_capsule() {
        let backend = Stub::scripted(
            vec![
                ran(&[LIVENESS_MARKER, HELD]),
                ran(&[LIVENESS_MARKER, HELD_NOT]),
            ],
            ran(&[]),
        );
        let count = Cell::new(0);
        let capsule = counting_capsules(&count);

        assert_eq!(
            run_arm(
                &arm(&backend, &capsule, ALWAYS_LIVE, Under::Confining),
                &ArmShape::Sequential {
                    writer: a_probe(),
                    reader: a_probe(),
                }
            ),
            ArmResult::Failed
        );
        assert_eq!(count.get(), 2, "one transaction per capsule, never a reuse");
        assert_eq!(backend.executions(), 2);
    }

    /// A writer that did not write leaves the reader with nothing to observe.
    /// Reporting that as `Failed` would put a broken fixture into the verdict as
    /// evidence about the backend — and the reader must not run at all, since
    /// its observation would be of a state nobody established.
    #[test]
    fn a_sequential_arm_whose_writer_did_not_write_establishes_nothing() {
        let backend = Stub::scripted(
            vec![
                ran(&[LIVENESS_MARKER, HELD_NOT]),
                ran(&[LIVENESS_MARKER, HELD]),
            ],
            ran(&[]),
        );
        let count = Cell::new(0);
        let capsule = counting_capsules(&count);

        let result = run_arm(
            &arm(&backend, &capsule, ALWAYS_LIVE, Under::Confining),
            &ArmShape::Sequential {
                writer: a_probe(),
                reader: a_probe(),
            },
        );

        assert_eq!(reason(&result), Indeterminacy::NoObservation);
        assert_ne!(result, ArmResult::Failed);
        assert_eq!(count.get(), 1, "the reader never ran");
        assert_eq!(backend.executions(), 1);
    }

    /// The observer capsule runs inside `execute_observed`'s callback, so the
    /// execute order is observer-then-subject. Two capsules, one arm.
    #[test]
    fn a_concurrent_arm_reads_the_observer_taken_while_the_subject_ran() {
        let backend = Stub::scripted(
            vec![ran(&[LIVENESS_MARKER, HELD]), ran(&[LIVENESS_MARKER, HELD])],
            ran(&[]),
        );
        let count = Cell::new(0);
        let capsule = counting_capsules(&count);

        assert_eq!(
            run_arm(
                &arm(&backend, &capsule, ALWAYS_LIVE, Under::Confining),
                &ArmShape::Concurrent {
                    subject: a_probe(),
                    observer: PidProbe {
                        argv: observer_argv,
                        observed: token(),
                    },
                }
            ),
            ArmResult::Held
        );
        assert_eq!(count.get(), 2);
    }

    /// `S8`, half one: the subject exited before the observer ran. The observer
    /// *did* report — it reported about a process that was no longer there — so
    /// the arm produced no usable observation, and that is
    /// `NoObservation` rather than a held probe.
    #[test]
    fn a_concurrent_arm_that_missed_its_window_reports_no_observation() {
        let backend = Stub::scripted(
            vec![ran(&[LIVENESS_MARKER, HELD]), ran(&[LIVENESS_MARKER, HELD])],
            ran(&[]),
        );
        let count = Cell::new(0);
        let capsule = counting_capsules(&count);

        let result = run_arm(
            &arm(&backend, &capsule, NEVER_LIVE, Under::Confining),
            &ArmShape::Concurrent {
                subject: a_probe(),
                observer: PidProbe {
                    argv: observer_argv,
                    observed: token(),
                },
            },
        );

        assert_eq!(reason(&result), Indeterminacy::NoObservation);
        assert_ne!(result, ArmResult::Held);
    }

    /// `S8`, half two: the backend never called back. Distinct reason, distinct
    /// repair — this one is a backend that did not implement the seam, and a
    /// suite reading it as a passing probe would admit the backend it was least
    /// able to check. The observer capsule must not have run at all.
    #[test]
    fn a_concurrent_arm_against_a_silent_seam_reports_no_liveness() {
        let backend = Stub::never_calling_back();
        let count = Cell::new(0);
        let capsule = counting_capsules(&count);

        let result = run_arm(
            &arm(&backend, &capsule, ALWAYS_LIVE, Under::Confining),
            &ArmShape::Concurrent {
                subject: a_probe(),
                observer: PidProbe {
                    argv: observer_argv,
                    observed: token(),
                },
            },
        );

        assert_eq!(reason(&result), Indeterminacy::NoLiveness);
        assert_ne!(result, ArmResult::Held);
        assert_eq!(count.get(), 1, "the observer capsule never ran");
    }

    /// A subject that never printed its marker never ran, so there was nothing
    /// to observe — and this is the case that would otherwise read as a *held*
    /// probe, because an observer that finds no live process is exactly what the
    /// probe arm expects to see.
    #[test]
    fn a_concurrent_arm_whose_subject_never_ran_reports_no_liveness() {
        let backend = Stub::scripted(vec![ran(&[LIVENESS_MARKER, HELD]), ran(&[])], ran(&[]));
        let count = Cell::new(0);
        let capsule = counting_capsules(&count);

        let result = run_arm(
            &arm(&backend, &capsule, ALWAYS_LIVE, Under::Confining),
            &ArmShape::Concurrent {
                subject: a_probe(),
                observer: PidProbe {
                    argv: observer_argv,
                    observed: token(),
                },
            },
        );

        assert_eq!(reason(&result), Indeterminacy::NoLiveness);
        assert_ne!(result, ArmResult::Held);
    }

    /// Invariant 3, as a property of the harness rather than of a row: the
    /// capsule that sets the scene always runs confined, and only the capsule
    /// the arm is read from is weakened.
    #[test]
    fn a_sequential_control_weakens_the_reader_and_never_the_writer() {
        let removal = PropertyRemoval::EnvCleared;
        let backend = Stub::scripted(
            vec![ran(&[LIVENESS_MARKER, HELD])],
            ran(&[LIVENESS_MARKER, HELD_NOT]),
        );
        let count = Cell::new(0);
        let capsule = counting_capsules(&count);

        assert_eq!(
            run_arm(
                &arm(&backend, &capsule, ALWAYS_LIVE, Under::Removing(removal)),
                &ArmShape::Sequential {
                    writer: a_probe(),
                    reader: a_probe(),
                }
            ),
            ArmResult::Failed
        );
        assert_eq!(
            backend.reached(),
            vec![Under::Confining, Under::Removing(removal)]
        );
    }

    /// `F-19`: row B5's control removes process visibility from the **observer**,
    /// which is an ordinary weakened capsule — the subject is the same confined
    /// capsule in both arms. That is why `execute_observed` needs no removal
    /// parameter.
    #[test]
    fn a_concurrent_control_weakens_the_observer_and_never_the_subject() {
        let removal = PropertyRemoval::ProcessVisibility;
        let backend = Stub::scripted(
            vec![ran(&[LIVENESS_MARKER, HELD])],
            ran(&[LIVENESS_MARKER, HELD_NOT]),
        );
        let count = Cell::new(0);
        let capsule = counting_capsules(&count);

        assert_eq!(
            run_arm(
                &arm(&backend, &capsule, ALWAYS_LIVE, Under::Removing(removal)),
                &ArmShape::Concurrent {
                    subject: a_probe(),
                    observer: PidProbe {
                        argv: observer_argv,
                        observed: token(),
                    },
                }
            ),
            ArmResult::Failed
        );
        assert_eq!(
            backend.reached(),
            vec![Under::Removing(removal), Under::Confining],
            "the observer runs first, inside the subject's callback"
        );
    }

    /// A grant is the other direction, and it must reach the same capsule.
    #[test]
    fn a_granting_control_reaches_the_capsule_the_arm_is_read_from() {
        let grant = AuthorityGrant::AllCapabilities;
        let backend = Stub::scripted(Vec::new(), ran(&[LIVENESS_MARKER, HELD_NOT]));
        let count = Cell::new(0);
        let capsule = counting_capsules(&count);

        assert_eq!(
            run_arm(
                &arm(&backend, &capsule, ALWAYS_LIVE, Under::Granting(grant)),
                &ArmShape::Single(a_probe())
            ),
            ArmResult::Failed
        );
        assert_eq!(backend.reached(), vec![Under::Granting(grant)]);
    }

    /// The window is the observer's *whole* run, not its start. Sampling
    /// liveness before the observer executes would credit an observer that
    /// outlived its subject — it would have looked at a process that was there
    /// when it was launched and gone by the time it looked. Here the subject
    /// dies during the observer's run, and only an after-the-fact sample sees
    /// it.
    #[test]
    fn the_window_closes_when_the_observer_finishes_not_when_it_starts() {
        let backend = Stub::scripted(
            vec![ran(&[LIVENESS_MARKER, HELD]), ran(&[LIVENESS_MARKER, HELD])],
            ran(&[]),
        );
        let count = Cell::new(0);
        let capsule = counting_capsules(&count);
        let observer_ran = |_pid: HostPid| count.get() < 2;

        let result = run_arm(
            &arm(&backend, &capsule, &observer_ran, Under::Confining),
            &ArmShape::Concurrent {
                subject: a_probe(),
                observer: PidProbe {
                    argv: observer_argv,
                    observed: token(),
                },
            },
        );

        assert_eq!(reason(&result), Indeterminacy::NoObservation);
    }

    /// A transaction that could not be provisioned is the mechanism failing, not
    /// the property failing (`EX-11`).
    #[test]
    fn an_arm_whose_capsule_could_not_be_provisioned_names_the_mechanism() {
        let backend = Stub::scripted(Vec::new(), ran(&[]));
        let refusal = || Err("Capacity".to_owned());

        let result = run_arm(
            &arm(&backend, &refusal, ALWAYS_LIVE, Under::Confining),
            &ArmShape::Single(a_probe()),
        );

        assert_eq!(
            reason(&result),
            Indeterminacy::BackendError("Capacity".to_owned())
        );
        assert_eq!(backend.executions(), 0);
    }

    // -----------------------------------------------------------------------
    // The session sweep (`T8`, `EX-12`, `D3`)
    // -----------------------------------------------------------------------
    //
    // That a *real* escaped descendant is reaped is `T10`'s
    // `the_orphan_left_by_a_teardown_or_visibility_control_is_reaped_by_the_harness`,
    // which needs a live capsule to discriminate. What is testable here is the
    // recording rule — which is where the sweep can be armed against the wrong
    // session, and the only place the mistake is still cheap.

    /// The harness's own session must never reach the swept set. Skipping it at
    /// signal time would be enough to be safe; refusing it at *record* time is
    /// what makes the refusal observable, because a sweep that signalled nothing
    /// and a sweep that recorded nothing look identical from outside.
    #[test]
    fn the_harness_never_records_its_own_session_as_a_capsules() {
        let fixture = Fixture::new(&SystemHost).expect("this host can host the fixture");
        let own = fixture.own_session().expect("this host answers /proc");
        let leader = HostPid(own.0);

        fixture.note_capsule_session(leader);

        assert_eq!(
            fixture.sweep_observed_sessions(),
            Vec::new(),
            "the harness's own session leader was recorded as a capsule"
        );
    }

    /// A pid with no `/proc` entry is a process that has already gone, which is
    /// the sweep's success case and not something to record. Recording it would
    /// arm the sweep against a **recycled** pid — the kernel hands the number
    /// out again, and the next holder is not this run's.
    #[test]
    fn a_vanished_capsule_records_no_session() {
        let fixture = Fixture::new(&SystemHost).expect("this host can host the fixture");

        fixture.note_capsule_session(HostPid(-1));

        assert_eq!(fixture.sweep_observed_sessions(), Vec::new());
    }

    /// Two capsules of one arm can lead one session between them, and the sweep
    /// enumerates `/proc` once per session — so a duplicate is a second full
    /// walk for nothing. The drain is the other half: sweeping is called after
    /// every row *and* from `Drop`, and the second call must not signal a pid
    /// the kernel has since recycled.
    ///
    /// Driven with a session id above every live one, so the sweep runs its
    /// whole real path — enumerate, match, signal — and finds no member. A test
    /// that named a session with members would be a test that sends `SIGKILL`
    /// to this machine.
    #[test]
    fn the_swept_set_holds_each_session_once_and_is_drained_by_sweeping() {
        let fixture = Fixture::new(&SystemHost).expect("this host can host the fixture");
        let memberless = SessionId(
            process_table()
                .iter()
                .map(|facts| facts.session.0)
                .max()
                .unwrap_or(0)
                .saturating_add(1),
        );

        fixture.note_session(memberless);
        fixture.note_session(memberless);

        assert_eq!(fixture.sweep_observed_sessions(), vec![memberless]);
        assert_eq!(
            fixture.sweep_observed_sessions(),
            Vec::new(),
            "sweeping drains, so the second sweep cannot signal a recycled pid"
        );
    }

    // -----------------------------------------------------------------------
    // Delta routing and transaction identity (`T7`, `EX-10`)
    // -----------------------------------------------------------------------
    //
    // What each delta *does to a placement* is `T10`'s two mandated `VT-2`
    // tests, which need two real provisioned transactions to discriminate:
    // `a_probe_arm_placement_is_byte_identical_to_what_provision_returned` and
    // `the_shared_root_delta_repoints_only_the_second_placement`. What is
    // testable without one is the routing — which deltas are placement-side and
    // which are backend-side — and the identity every transaction is allocated.

    /// Three deltas rebuild a placement and two reach the backend, and nothing
    /// may do both: a delta that both widened the mount set and weakened the
    /// profile would make a row's two arms differ by two things, which is the
    /// one thing invariant 3 forbids.
    #[test]
    fn only_the_two_backend_side_deltas_reach_the_profile() {
        let removal = PropertyRemoval::EnvCleared;
        let grant = AuthorityGrant::AllCapabilities;

        assert_eq!(
            under_for(&Delta::Removed(removal)),
            Under::Removing(removal)
        );
        assert_eq!(under_for(&Delta::Granted(grant)), Under::Granting(grant));

        for placement_side in [
            Delta::SharedRoot,
            Delta::Widened(|fixture| {
                vec![MountedPath::new(
                    fixture.decoy_credential().to_path_buf(),
                    inner("/capsule/decoy"),
                )]
            }),
            Delta::NetworkPermitted,
        ] {
            assert_eq!(
                under_for(&placement_side),
                Under::Confining,
                "{placement_side:?} is a placement rebuild, so its arm runs confined"
            );
        }
    }

    /// Two capsules in one arm are two transactions, and `sec-3` step 9 creates
    /// each root **exclusively** — so a repeated id would refuse the second
    /// capsule of every two-capsule row, which reads as a mechanism failure on a
    /// perfectly good backend.
    #[test]
    fn every_transaction_is_allocated_its_own_id() {
        let minted: Vec<String> = (0..8)
            .map(|_| {
                next_transaction_id()
                    .expect("the harness mints a lawful id")
                    .as_str()
                    .to_owned()
            })
            .collect();

        let distinct: BTreeSet<&String> = minted.iter().collect();
        assert_eq!(distinct.len(), minted.len(), "{minted:?}");
    }

    /// The suite has no timeout policy of its own. Both bounds are read off the
    /// same two constants the synthesized `[capsule]` table declares, so a
    /// payload that hangs is killed by the bound `provision` would have applied
    /// — a second, drifting number here is how a harness ends up outliving the
    /// capsules it is supposed to bound.
    #[test]
    fn a_payload_runs_under_the_bounds_the_fixture_declares() {
        let execution = harness_execution(&argv(&["true"]));

        assert_eq!(
            execution.timeout(),
            Duration::from_secs(FIXTURE_TIMEOUT_SECONDS)
        );
        assert_eq!(
            execution.file_size_cap(),
            ByteCount::from_bytes(FIXTURE_FILE_SIZE_CAP_MIB * BYTES_PER_MIB)
        );
        assert!(
            capsule_config_document(Path::new("/capsule"), &[PathBuf::from("/bin")]).contains(
                &format!("execution-timeout-seconds = {FIXTURE_TIMEOUT_SECONDS}")
            ),
            "the declared table and the harness read the same constant"
        );
    }

    // -----------------------------------------------------------------------
    // The pid seam (`T5`, `EX-7`, `D3`)
    // -----------------------------------------------------------------------

    /// The harness's session, in every fixture below.
    const HARNESS_SESSION: i32 = 100;

    /// One row of a `/proc` snapshot.
    fn facts(pid: i32, parent: i32, session: i32) -> ProcessFacts {
        ProcessFacts {
            pid: HostPid(pid),
            parent: HostPid(parent),
            session: SessionId(session),
        }
    }

    /// The tree this host actually produces, measured under the confining
    /// profile: `timeout` → `bwrap` → the sandbox process, and only the last
    /// leads a session of its own.
    fn measured_tree() -> Vec<ProcessFacts> {
        vec![
            facts(HARNESS_SESSION, 1, HARNESS_SESSION),
            facts(200, HARNESS_SESSION, HARNESS_SESSION), // timeout -k
            facts(300, 200, HARNESS_SESSION),             // bwrap, outer
            facts(400, 300, 400),                         // the capsule, --new-session
        ]
    }

    #[test]
    fn the_capsule_is_the_session_leader_below_the_immediate_child() {
        assert_eq!(
            capsule_session_leader(HostPid(200), &measured_tree()),
            Some(HostPid(400))
        );
    }

    /// Row 7's payload detaches a **second** session leader below the same
    /// child. Nearest-wins is the whole difference between naming the subject
    /// and naming the escapee; a first-match implementation passes the fixture
    /// above and fails this one.
    #[test]
    fn an_escaping_orphan_is_not_mistaken_for_the_capsule() {
        // The escapee is listed **first**: `/proc`'s directory order is not
        // numeric and not stable, so a first-match implementation must fail here
        // rather than pass by luck.
        let mut table = vec![
            facts(500, 400, 500), // the setsid'd descendant
            facts(600, 500, 500), // and its own child, for good measure
        ];
        table.extend(measured_tree());
        assert_eq!(
            capsule_session_leader(HostPid(200), &table),
            Some(HostPid(400))
        );
    }

    /// Another agent's capsule, running concurrently under a different child, is
    /// a foreign session leader too. Descent from *this* child is what excludes
    /// it — the session predicate alone does not.
    #[test]
    fn a_foreign_session_outside_this_subtree_is_not_the_capsule() {
        // Its pid is **lower** than the capsule's, so the depth-0 tie-break
        // cannot rescue an implementation that forgot to descend.
        let mut table = vec![facts(150, HARNESS_SESSION, 150)];
        table.extend(measured_tree());
        assert_eq!(
            capsule_session_leader(HostPid(300), &table),
            Some(HostPid(400))
        );
    }

    /// The harness's own session leader satisfies `session == pid` too, and needs
    /// no special case: it predates the child, so it lies *upward* in the tree
    /// and the descent never reaches it. This is the fixture that says so.
    #[test]
    fn the_harnesss_own_session_leader_is_never_the_capsule() {
        let table = vec![
            facts(HARNESS_SESSION, 1, HARNESS_SESSION),
            facts(200, HARNESS_SESSION, HARNESS_SESSION),
        ];
        assert_eq!(capsule_session_leader(HostPid(200), &table), None);
    }

    /// The top-level process has exited and its child survives, carrying the
    /// dead leader's session id. A session *member* is not a session *leader*:
    /// naming that survivor would hand the row a pid whose liveness means
    /// nothing about the subject.
    #[test]
    fn a_survivor_of_a_departed_leader_is_not_the_capsule() {
        let table = vec![
            facts(HARNESS_SESSION, 1, HARNESS_SESSION),
            facts(200, HARNESS_SESSION, HARNESS_SESSION),
            facts(300, 200, HARNESS_SESSION),
            facts(550, 300, 400), // reparented; session 400's leader is gone
        ];
        assert_eq!(capsule_session_leader(HostPid(200), &table), None);
    }

    /// No callback rather than a wrong one: `classify_concurrent` reads silence
    /// as `Indeterminacy::NoLiveness`, and a pid we could not establish is not
    /// evidence.
    #[test]
    fn a_capsule_that_never_appears_yields_no_pid() {
        let table = vec![
            facts(HARNESS_SESSION, 1, HARNESS_SESSION),
            facts(200, HARNESS_SESSION, HARNESS_SESSION),
            facts(300, 200, HARNESS_SESSION),
        ];
        assert_eq!(capsule_session_leader(HostPid(200), &table), None);
    }

    /// `/proc` is read entry by entry and can be torn between them, so a parent
    /// chain that closes on itself is reachable. It must terminate.
    #[test]
    fn a_torn_parent_cycle_terminates() {
        let table = vec![facts(10, 11, 10), facts(11, 10, 11)];
        assert_eq!(depth_from(HostPid(10), HostPid(999), &table), None);
        assert_eq!(capsule_session_leader(HostPid(999), &table), None);
    }

    /// `comm` is attacker-adjacent: it is the payload's own `argv[0]` basename,
    /// it is not escaped, and it may hold spaces, parentheses and digits. This
    /// line's `comm` is chosen so that splitting from the **left** on
    /// whitespace, or on the **first** `)`, reads a different field —
    /// and so that ppid, pgrp and sid are three distinct numbers, which is what
    /// gives `M13`'s field-index mutation something to red against.
    #[test]
    fn stat_is_read_past_the_last_paren_of_a_hostile_comm() {
        let dir = TempDir::new().expect("a scratch directory");
        let path = dir.path().join(STAT_LEAF);
        let mut file = File::create(&path).expect("the fixture opens");
        file.write_all(b"4242 (evil ) 9 9 9 9) S 111 222 333 0 -1 4194304 0 0\n")
            .expect("the fixture writes");
        drop(file);
        assert_eq!(
            stat_of(&path),
            Some(StatFacts {
                reaped: false,
                parent: HostPid(111),
                session: SessionId(333),
            }),
            "ppid 111, pgrp 222 and sid 333 are distinct on purpose"
        );
    }

    /// A vanished process is not an error — `/proc` is a moving target.
    #[test]
    fn a_missing_stat_is_absence_rather_than_failure() {
        let dir = TempDir::new().expect("a scratch directory");
        assert_eq!(stat_of(&dir.path().join(STAT_LEAF)), None);
    }

    /// A leader that has exited but not been reaped still has a readable
    /// `/proc` entry with its own sid in it, so "the entry is there" is not
    /// liveness. `T8`'s sweep signs off on the session only once the leader is
    /// gone, and a zombie leader read as alive would hang the sweep out to the
    /// wall bound every time.
    #[test]
    fn a_zombie_leader_is_not_running() {
        let leader = HostPid(4242);
        let facts = |state: &str| StatFacts {
            reaped: state == "Z",
            parent: HostPid(1),
            session: SessionId(leader.0),
        };

        assert!(still_running(leader, &facts("S")));
        assert!(!still_running(leader, &facts("Z")));
    }

    /// The other half of the recycling problem: the kernel reuses pids, so an
    /// entry at the capsule's pid that is *not* its own session leader is a
    /// different process wearing the number.
    #[test]
    fn a_recycled_pid_is_not_the_capsule_still_running() {
        let leader = HostPid(4242);
        assert!(!still_running(
            leader,
            &StatFacts {
                reaped: false,
                parent: HostPid(1),
                session: SessionId(9),
            }
        ));
    }

    /// The live wiring: this process is alive and readable, and it is *not* a
    /// session leader — so a `false` here can only have come from reading this
    /// process's real sid. A reader that took the wrong pid's file, or dropped
    /// the leader predicate, would answer `true`.
    #[test]
    fn the_liveness_reader_reads_the_pid_it_was_given() {
        let own = HostPid(std::process::id().try_into().expect("a positive pid"));
        assert_ne!(
            own_session(),
            Some(SessionId(own.0)),
            "the harness is not a session leader, or this test proves nothing"
        );

        assert!(!capsule_still_running(own));
        assert!(!capsule_still_running(HostPid(-1)), "no such entry");
    }

    /// The live half: the two `/proc` readers agree about this process, and the
    /// sweep sees it. Cheap, and it is the only thing that catches `PROC_SELF`
    /// and the `/proc/<pid>` join disagreeing.
    #[test]
    fn the_harness_can_read_its_own_session_by_both_routes() {
        let own = own_session().expect("this process has a session");
        let pid = HostPid(i32::try_from(std::process::id()).expect("a host pid fits in i32"));
        assert_eq!(session_of(pid), Some(own));
        let table = process_table();
        assert!(
            table.iter().any(|entry| entry.pid == pid),
            "the sweep did not find the process running it"
        );
    }
}
