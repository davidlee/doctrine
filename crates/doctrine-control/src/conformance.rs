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

use std::fs::File;
use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::os::fd::OwnedFd;
use std::os::unix::fs::PermissionsExt as _;
use std::os::unix::net::UnixStream;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

use doctrine::DOCTRINE_TOML;
use rustix::fs::FsWord;

use crate::backend::bubblewrap::{BubblewrapBackend, WeakenedProfile, Weakening, mechanism_failed};
use crate::backend::{
    AcceptedBase, Availability, BackendError, BackendId, CapsuleBackend, CapsuleEnvVar,
    CapsulePlacement, EXPORT_DIRECTORY_LEAF, Execution, FILESYSTEM_ROOT, ForbiddenScopes,
    MountedPath, Observation, Termination,
};
use crate::config::Argv;
use crate::host::HostFacts;

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
#[expect(
    dead_code,
    reason = "SL-248: the payloads are executed by PHASE-08's harness. At PHASE-07 the shapes \
              are constructed and matched on but nothing runs them, and no derive rescues \
              this — rustc ignores Clone and Debug for dead-code analysis, and PartialEq is \
              unavailable here because PidProbe holds a function pointer. Narrowed to this \
              enum rather than left to the module-level blanket; it self-clears when the \
              harness reads a payload."
)]
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
    /// Trusted-side, row 5's target.
    listener: TcpListener,
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
        make_directory(&decoy_readable_input)?;
        make_directory(&decoy_repository)?;
        git(&decoy_repository, &["init", "--quiet"])?;
        write_file(&decoy_credential, DECOY_CREDENTIAL_BODY)?;
        write_file(&decoy_undeclared, DECOY_UNDECLARED_BODY)?;
        write_file(&decoy_executable, DECOY_EXECUTABLE_BODY)?;
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
            listener: TcpListener::bind(LOOPBACK_ANY_PORT).map_err(|error| FixtureFault::Io {
                path: PathBuf::from(LOOPBACK_ANY_PORT),
                detail: error.to_string(),
            })?,
            root,
            project_root,
            base,
            capsule_root,
            decoy_credential,
            decoy_repository,
            decoy_readable_input,
            decoy_undeclared,
            decoy_executable,
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
    pub(crate) const fn listener(&self) -> &TcpListener {
        &self.listener
    }
}

/// `.doctrine`, spelled here because `provision`'s constant is private to that
/// module. `doctrine::DOCTRINE_TOML` supplies the document's own path, so only
/// the directory leaf is duplicated.
const CONTROL_PLANE_STATE_LEAF: &str = ".doctrine";

const DECOY_CREDENTIAL_BODY: &str = "decoy-token: not-a-real-credential\n";
const DECOY_UNDECLARED_BODY: &str = "declared to no placement\n";
const DECOY_EXECUTABLE_BODY: &str = "#!/bin/sh\necho decoy\n";
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
         trusted_side_forbidden_executables = []\n\
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
    std::fs::create_dir_all(path).map_err(|error| FixtureFault::Io {
        path: path.to_path_buf(),
        detail: error.to_string(),
    })
}

/// `File::create` plus `write_all`, because `std::fs::write` is banned
/// crate-wide by `clippy.toml`'s `disallowed-methods`.
fn write_file(path: &Path, contents: &str) -> Result<(), FixtureFault> {
    if let Some(parent) = path.parent() {
        make_directory(parent)?;
    }
    let io = |error: std::io::Error| FixtureFault::Io {
        path: path.to_path_buf(),
        detail: error.to_string(),
    };
    let mut file = File::create(path).map_err(io)?;
    file.write_all(contents.as_bytes()).map_err(io)
}

fn make_executable(path: &Path) -> Result<(), FixtureFault> {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(EXECUTABLE_MODE)).map_err(
        |error| FixtureFault::Io {
            path: path.to_path_buf(),
            detail: error.to_string(),
        },
    )
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
    let stat = rustix::fs::statvfs(path).ok()?;
    stat.f_bavail.checked_mul(stat.f_frsize)
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
        // `T5` owes this the descent from the immediate child — which under the
        // wall bound is `timeout(1)` — to the capsule's own top-level process,
        // and the session id `EX-12`'s containment needs (`D3`). The seam is
        // here; the descent is not.
        let relay = |pid: i32| observer(HostPid(pid));
        self.run(
            placement,
            execution,
            &WeakenedProfile::confining().observed_by(&relay),
        )
    }
}

// ---------------------------------------------------------------------------
// The delta and row vocabulary (`EX-8`, `EX-14`)
// ---------------------------------------------------------------------------

/// How a control arm's placement differs from the probe arm's.
///
/// Exactly one value, so *differs by one thing* is a type rather than a promise
/// (invariant 3).
#[derive(Debug, Clone)]
#[expect(
    dead_code,
    reason = "SL-248: a delta's payload is applied by PHASE-08's harness — the removal reaches \
              `execute_weakened`, the widening is called with the fixture. At PHASE-07 the \
              variants are constructed and matched on but their payloads are not applied. \
              Self-clears when the harness applies one."
)]
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
#[expect(
    dead_code,
    reason = "SL-248: `run_row` reads `shape` and `delta` at PHASE-08; at PHASE-07 rows are \
              built and counted but never executed. Written at item level, not on the two \
              fields: under cfg(not(test)) the whole struct is dead, rustc reports that at \
              the struct and never descends to the fields, so field-level expectations go \
              unfulfilled in one of the two compilation units and `unfulfilled_lint_\
              expectations` is denied. Self-clears with the harness."
)]
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

/// Table C's claims. Empty until PHASE-08 lands them.
fn auxiliary_claims() -> Vec<(Claim, AuxOutcome)> {
    Vec::new()
}

/// Run one row. PHASE-08's named seam; unreachable at this phase because
/// [`tables`] is empty.
fn run_row(_backend: &dyn ConformanceBackend, _row: &Row) -> RowVerdict {
    RowVerdict::Indeterminate {
        arm: Which::Probe,
        detail: Indeterminacy::BackendError(
            "the executing harness lands in SL-248 PHASE-08".to_owned(),
        ),
    }
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
    verify_over(
        backend,
        host,
        today,
        &tables(),
        auxiliary_claims(),
        &run_row,
    )
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
    auxiliary: Vec<(Claim, AuxOutcome)>,
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
            auxiliary,
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
            auxiliary,
        };
    }

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
    use std::collections::BTreeMap;
    use std::io::Write as _;
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use super::Weakening as ProfileWeakening;
    use super::{
        Admission, AdmissionVerdict, ArmResult, ArmShape, AuthorityGrant, AuxOutcome, Axis, Bound,
        Claim, ConcurrentWitness, ConformanceBackend, Delta, Fixture, HOME_VARIABLE, HostPid,
        Indeterminacy, LIVENESS_MARKER, NotAdmitted, Observed, PidProbe, Probe, PropertyRemoval,
        Row, RowId, RowVerdict, SHELL, TMPFS_MAGIC, TempRoot, Which, admission, available_bytes_of,
        capsule_config_document, classify, classify_concurrent, decode_mount_field, git,
        mount_points, on_real_disk, prepare_root, row_ids_in_more_than_one_table, row_verdict,
        second_filesystem, system_readable_roots, top_level_ancestor, verify, verify_over,
    };
    use super::{OwnedStdio, weakening_for, weakening_granting};
    use crate::backend::bubblewrap::{SpawnOptions, confinement_argv};
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
    }

    impl Stub {
        fn ignoring_its_removal(stdout: &[&str]) -> Self {
            Self {
                inner: WitnessBackend::always(Ok(ran(stdout))),
                weakening: Weakening::DelegatesToExecute,
                seam: ObserverSeam::CallsBack(HostPid(4242)),
            }
        }

        fn weakening_honestly(probe: &[&str], weakened: &[&str]) -> Self {
            Self {
                inner: WitnessBackend::always(Ok(ran(probe))),
                weakening: Weakening::Answers(Ok(ran(weakened))),
                seam: ObserverSeam::CallsBack(HostPid(4242)),
            }
        }

        fn unavailable(missing: &str, remedy: &str) -> Self {
            Self {
                inner: WitnessBackend::always(exited(0, "")).reporting(Availability::Unavailable {
                    missing: missing.to_owned(),
                    remedy: remedy.to_owned(),
                }),
                weakening: Weakening::DelegatesToExecute,
                seam: ObserverSeam::CallsBack(HostPid(4242)),
            }
        }

        fn never_calling_back() -> Self {
            Self {
                inner: WitnessBackend::always(Ok(ran(&[LIVENESS_MARKER, HELD]))),
                weakening: Weakening::DelegatesToExecute,
                seam: ObserverSeam::NeverCallsBack,
            }
        }

        fn executions(&self) -> usize {
            self.inner.calls().len()
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
            self.inner.execute(placement, execution)
        }
    }

    impl ConformanceBackend for Stub {
        fn execute_weakened(
            &self,
            placement: &CapsulePlacement,
            execution: &Execution,
            _removal: PropertyRemoval,
        ) -> Result<Observation, BackendError> {
            match &self.weakening {
                Weakening::DelegatesToExecute => self.execute(placement, execution),
                Weakening::Answers(answer) => answer.clone(),
            }
        }

        fn execute_granted(
            &self,
            placement: &CapsulePlacement,
            execution: &Execution,
            _grant: AuthorityGrant,
        ) -> Result<Observation, BackendError> {
            match &self.weakening {
                Weakening::DelegatesToExecute => self.execute(placement, execution),
                Weakening::Answers(answer) => answer.clone(),
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
            auxiliary,
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
}
