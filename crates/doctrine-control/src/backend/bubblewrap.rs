// SPDX-License-Identifier: GPL-3.0-only
//! `backend::bubblewrap` — the Linux confinement profile (SL-248 `sec-2`
//! § *The bubblewrap backend*, `EX-1`…`EX-19`).
//!
//! One implementation of [`crate::backend::CapsuleBackend`], self-contained per
//! `DEC-155`. Naming is plural-ready throughout: this is *a backend*, never
//! *the confinement profile* — a second profile is expected to land beside it
//! and must not inherit this one's vocabulary.
//!
//! **The flag tokens are named once, here** (`EX-4`, `STD-001`). The root
//! package's `src/worktree/` carries its own bubblewrap vocabulary under a
//! byte-parity contract with `scripts/pi-spawn-confined.sh`; `DEC-155` is the
//! decision *not* to reuse it, so nothing here imports from it (invariant 10)
//! and nothing there is widened.
//!
//! **The inner layout is imported, not restated** (PHASE-04 `D9`): the four
//! `INNER_*` constants and the six reserved destinations live in
//! [`crate::backend`], which is their single home.
//!
//! **Three things are enforced outside the namespace**, because a bound a
//! capsule can reach is not a bound: the wall clock (`timeout -k`), the
//! per-file size cap (`RLIMIT_FSIZE` on the child), and the descriptor sweep
//! (`/proc/self/fd`, marked in the parent before the fork). None of the three
//! has a bubblewrap flag behind it — an already-open descriptor is not a
//! namespace, a mount, or an environment entry (`EX-15`, `EX-16`, `EX-18`).
//!
//! **Resolution precedes validation precedes binding** (`EX-14`, invariant 3).
//! bubblewrap dereferences the source path of a `--ro-bind`, so a lexically
//! contained declared root pointing elsewhere would bind elsewhere; `RV-346`
//! `F-1` demonstrated it by execution. Every declared entry and every
//! resolver-returned path is fully resolved before anything looks at it, the
//! resolved path is what [`crate::backend::CapsulePlacement::try_new`]
//! validates, and the resolved path is what is bound.
//!
//! **Two `expect(unsafe_code)` sites, and two is the budget** (`F-1/R`, this
//! slice's ruling on `S1`). The workspace denies `unsafe` by default; the
//! child's `RLIMIT_FSIZE` and the descriptor sweep have no safe route, and
//! `the_unsafe_budget_is_exactly_two_sites` holds the count at two so the
//! ceiling is mechanically visible rather than merely intended.
//!
//! Layering (`ADR-001`): this file is a **submodule** of `backend`, which the
//! gate maps to the `backend` unit, so it adds no `layering.toml` row
//! (`EX-19`, `VA-4`). It does reach [`crate::host`] — `EX-13`'s host `PATH`
//! read and `EX-9`/`EX-12`'s existence probes are what `EN-4` calls this
//! phase's hermeticity — so the `backend` unit acquires the out-edge
//! `backend → host`. Both units are `leaf` and `host` is out=0, so nothing
//! cycles and no tier inverts (`D1`; the divergence from `sec-6`'s unit table
//! is `F-2`).

use std::collections::BTreeSet;
use std::fs::File;
use std::io;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, OwnedFd, RawFd};
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::time::Duration;

use rustix::fs::Dir;
use rustix::io::{Errno, FdFlags, fcntl_getfd, fcntl_setfd};
use rustix::process::{Resource, Rlimit, setrlimit};

use crate::backend::{
    Availability, BackendError, BackendId, CapsuleBackend, CapsuleEnv, CapsuleEnvVar,
    CapsulePlacement, CapsuleStdio, Execution, INNER_AGENT, INNER_CAPSULE, INNER_DEV, INNER_PROC,
    INNER_SOURCE, INNER_TMP, NetworkPosture, Observation, Termination, TransactionRoot,
};
use crate::config::{
    Argv, ByteCount, CapsuleConfig, KEY_CLOSURE_RESOLVER, KEY_CLOSURE_ROOTS, KEY_READABLE_ROOTS,
};
use crate::host::HostFacts;

// ---------------------------------------------------------------------------
// Identity, the executables, and the capsule's declared credentials
// ---------------------------------------------------------------------------

/// This backend's stable identity, recorded in an admission verdict.
const BACKEND_ID: BackendId = BackendId::new("bubblewrap");

/// The confinement mechanism itself.
const BWRAP_EXECUTABLE: &str = "bwrap";

/// The wall bound, applied from **outside** the namespace (`EX-15`).
const TIMEOUT_EXECUTABLE: &str = "timeout";

/// `timeout`'s "then send `SIGKILL` after this long" flag.
const KILL_GRACE_FLAG: &str = "-k";

/// The capsule's declared identity, fixed by the profile and **not**
/// configurable (`D10`, `EX-5`): an operator-chosen uid would be a second way
/// to weaken a capsule, and `sec-5` deliberately carries no credential knob.
///
/// `--unshare-all` implies `--unshare-user`, which is what makes the two flags
/// legal for an unprivileged trusted side. Measured nested inside this
/// project's own jail (`F-5`): `uid=1000 gid=1000`. The supplementary group
/// that survives is `sec-7` rows 13/14's, not this phase's — do not read these
/// two flags as having discharged invariant 15.
const CAPSULE_UID: u32 = 1000;
/// The capsule's declared group; see [`CAPSULE_UID`].
const CAPSULE_GID: u32 = 1000;

/// The grace `timeout -k` waits before escalating to `SIGKILL`, when the
/// caller supplies none.
///
/// `config::ResourceBounds` already carries `execution-kill-grace-seconds`, but
/// [`Execution`] — fixed by PHASE-04 and not this phase's to widen — does not,
/// so the configured figure has no route into `execute`. [`BubblewrapBackend::with_kill_grace`]
/// is that route for PHASE-06; this is the floor when it is not used. Recorded
/// as a finding rather than papered over.
const DEFAULT_KILL_GRACE: Duration = Duration::from_secs(5);

// ---------------------------------------------------------------------------
// The flag tokens — named once, here, and nowhere else (EX-4, VA-2)
// ---------------------------------------------------------------------------

/// Prepended ahead of every confinement flag (`D11`). The only lawful source
/// for [`Termination::NotExecutable`]: bubblewrap exits 1 when `execvp` fails,
/// which is indistinguishable from a capsule that exits 1, and its stderr note
/// is capsule-forgeable (invariant 8). The JSON status carries `exit-code`
/// **only when the child actually ran**, and its absence is a parent-side
/// observation.
const FLAG_JSON_STATUS_FD: &str = "--json-status-fd";
const FLAG_UNSHARE_ALL: &str = "--unshare-all";
const FLAG_UID: &str = "--uid";
const FLAG_GID: &str = "--gid";
const FLAG_PROC: &str = "--proc";
const FLAG_DEV: &str = "--dev";
const FLAG_TMPFS: &str = "--tmpfs";
const FLAG_RO_BIND: &str = "--ro-bind";
const FLAG_BIND: &str = "--bind";
const FLAG_CHDIR: &str = "--chdir";
const FLAG_DIE_WITH_PARENT: &str = "--die-with-parent";
const FLAG_NEW_SESSION: &str = "--new-session";
const FLAG_CLEARENV: &str = "--clearenv";
const FLAG_SETENV: &str = "--setenv";
/// Emitted **last** and **only** for [`NetworkPosture::Permitted`]. The
/// inversion `DEC-155` names: the worktree arm's default is the permissive
/// floor, and a capsule must be default-denied.
const FLAG_SHARE_NET: &str = "--share-net";

/// What `Weakening::ProcessVisibility` puts in place of `--unshare-all`.
///
/// bubblewrap has **no `--share-pid`** (`EX-9`), so removing pid isolation is
/// the one axis expressed by naming the namespaces that remain rather than by
/// subtracting a flag. The `-try` forms because `--unshare-all` itself uses
/// them, and a bare `--unshare-user` fails on a host without unprivileged user
/// namespaces. Measured accepted and effective on bwrap 0.11.2 (`S7`).
const FLAG_UNSHARE_NET: &str = "--unshare-net";
const NON_PID_UNSHARE_SET: [&str; 5] = [
    "--unshare-user-try",
    "--unshare-ipc",
    FLAG_UNSHARE_NET,
    "--unshare-uts",
    "--unshare-cgroup-try",
];

/// What `Weakening::AllCapabilities` appends, and the only flag it appends.
const FLAG_CAP_ADD: &str = "--cap-add";
const CAPABILITY_ALL: &str = "ALL";

// ---------------------------------------------------------------------------
// The environment, and the one variable read from the host
// ---------------------------------------------------------------------------

/// `PATH`'s *host* role: the variable this backend reads to derive the capsule's
/// own (`EX-13`). Its *inside* role — the name it is set under — is
/// [`CapsuleEnvVar::name`], and this is defined from it so the two roles cannot
/// drift apart (`STD-001`).
const HOST_PATH_VARIABLE: &str = CapsuleEnvVar::Path.name();

/// What a `PATH` list is joined with. Not `std::env::join_paths`, whose
/// `OsString` result would need a lossless-to-`String` step this argv cannot
/// take anyway.
const PATH_SEPARATOR: &str = ":";

// ---------------------------------------------------------------------------
// The descriptor sweep's bounds, and the status channel
// ---------------------------------------------------------------------------

/// The sweep's **floor**, and it is load-bearing (`RV-346` `F-30`).
///
/// *Every descriptor is closed* and *every descriptor above 2 is closed* differ
/// by exactly the three a shell hands a process by default, and a sweep is
/// described by its bound as much as by its action. Those three are **replaced**
/// by the parent instead (invariant 14) — marking them close-on-exec would give
/// the capsule no way to report at all.
const SWEEP_FLOOR: RawFd = 3;

/// Where the trusted side's own open descriptors are enumerated, in the parent
/// and **before** the fork. Enumerate-and-mark rather than close-after-fork:
/// a post-fork closure runs in the window where allocation is unsafe, and
/// reading a directory there is precisely the allocation to avoid.
const PROC_SELF_FD: &str = "/proc/self/fd";

/// Where bubblewrap's JSON status is collected, beneath the transaction root
/// and outside every bound path, so the capsule can neither read nor forge it.
const STATUS_FILE_LEAF: &str = "bwrap-status.json";

/// The key whose **presence** means the child was reached (`D11`). Substring,
/// not a parse: this is a one-bit question and a JSON dependency to answer it
/// would be a new crate.
const STATUS_EXIT_CODE_KEY: &str = "\"exit-code\"";

// ---------------------------------------------------------------------------
// The measured termination table (D12)
// ---------------------------------------------------------------------------

/// `timeout(1)`'s own exit code when it fired. Measured, coreutils 9.11.
const TIMEOUT_EXIT_CODE: i32 = 124;

/// What a shell-style wait status adds to a signal number.
const SIGNALLED_EXIT_BASE: i32 = 128;

/// `SIGXFSZ` — what `RLIMIT_FSIZE` raises on the offending write, so
/// `128 + 25 = 153` is the file-size-cap termination. Measured.
const SIGXFSZ: i32 = 25;

// ---------------------------------------------------------------------------
// The weakening seam (SL-248 PHASE-08 `T3`, `EX-8`, `EX-9`, `D2`)
// ---------------------------------------------------------------------------

/// One axis of the confining profile, switched off.
///
/// **This is backend-tier vocabulary and names no `conformance` type.**
/// `.doctrine/adr/001/layering.toml` classifies `backend` as a *leaf* and
/// `conformance` as an *engine* with an out-edge to it; a `bubblewrap.rs` that
/// imported `PropertyRemoval` would put a leaf→engine edge in a graph that
/// already has the reverse, which is ADR-001's cycle (`D2`). So the mapping
/// runs the other way: `conformance.rs` matches its own removals onto these.
///
/// An **enum, not a bag of booleans**, and that is the point: a weakened run
/// differs from the confining profile along *exactly one* axis, and a type that
/// cannot express two at once makes that structural instead of promised. There
/// is no `Weakening::None` — absence is `Option`'s job, and the confining
/// profile is the one with nothing selected.
///
/// The eleven axes are ten property removals plus one authority grant. Adding a
/// twelfth is a governed decision, not a convenience.
#[derive(Debug)]
pub(crate) enum Weakening {
    /// Omit `--chdir`, so the capsule starts wherever bubblewrap leaves it.
    WorkingDirectory,
    /// Omit `--die-with-parent`. Measured (`EVD-013`, and again at plan time)
    /// to be what actually reaps an escaping detached grandchild — the pid
    /// namespace is not.
    Teardown,
    /// Replace `--unshare-all` with [`NON_PID_UNSHARE_SET`], leaving the pid
    /// namespace shared.
    ProcessVisibility,
    /// Skip [`apply_file_size_cap`], so `RLIMIT_FSIZE` is never set.
    FileSizeBound,
    /// Exec `bwrap` directly, with no `timeout -k` wrapper outside it.
    WallBound,
    /// `--ro-bind` becomes `--bind` for `/source` **and** every declared
    /// readable entry. Same paths, same inner destinations, same count —
    /// attachment alone changes.
    InputsWritable,
    /// Skip [`mark_inherited_descriptors_close_on_exec`]. The status
    /// descriptor is still cleared of `CLOEXEC`; the order those two run in is
    /// load-bearing and does not move.
    Descriptors,
    /// Omit `--clearenv`. The `--setenv` list is left byte-identical.
    EnvironmentCleared,
    /// Replace the three standard stream endpoints with descriptors the caller
    /// owns (`D4`). Nothing above descriptor 2 moves.
    ///
    /// The descriptors arrive here rather than on [`Execution`] because the
    /// weakening is a property of the weakened *run*, not of the execution
    /// request — which is what keeps a descriptor channel out of the
    /// production vocabulary (`S5`).
    StdioOwned {
        input: OwnedFd,
        output: OwnedFd,
        errors: OwnedFd,
    },
    /// Omit `--uid`/`--gid`, touching no namespace.
    MappedIdentity,
    /// Append `--cap-add ALL`, every other flag unchanged. The one *grant*
    /// among ten removals. Measured effective on bwrap 0.11.2: `CapEff` moves
    /// from `0000000000000000` to `000001ffffffffff` (`S7`).
    AllCapabilities,
}

/// How one run of the profile differs from the confining one.
///
/// [`WeakenedProfile::confining`] is the profile [`CapsuleBackend::execute`]
/// runs, and it selects nothing — so *the production path and the conformance
/// suite's probe arm are the same code*, which is the whole reason the seam is
/// here rather than a second implementation of the profile in `conformance.rs`
/// (`D2`).
///
/// The observer is orthogonal to the weakening: a probe arm observes an
/// otherwise fully confining run.
pub(crate) struct WeakenedProfile<'o> {
    weakening: Option<Weakening>,
    observer: Option<&'o dyn Fn(i32)>,
}

impl std::fmt::Debug for WeakenedProfile<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeakenedProfile")
            .field("weakening", &self.weakening)
            .field("observer", &self.observer.map(|_| "…"))
            .finish()
    }
}

impl<'o> WeakenedProfile<'o> {
    /// The full confining profile: nothing removed, nothing granted.
    pub(crate) const fn confining() -> Self {
        Self {
            weakening: None,
            observer: None,
        }
    }

    pub(crate) const fn weakened(weakening: Weakening) -> Self {
        Self {
            weakening: Some(weakening),
            observer: None,
        }
    }

    /// Call `observer` exactly once, trusted-side, with the capsule's host-side
    /// pid while it is still alive.
    #[must_use]
    pub(crate) fn observed_by(mut self, observer: &'o dyn Fn(i32)) -> Self {
        self.observer = Some(observer);
        self
    }

    const fn weakening(&self) -> Option<&Weakening> {
        self.weakening.as_ref()
    }
}

/// What a profile changes about the **spawn** rather than about the argv.
///
/// Four of the eleven axes leave every bubblewrap word untouched and move a
/// property of the child process instead, so an argv diff alone reads them as
/// "changed nothing" — which is exactly the reading `VA-4` must not be given.
/// They are named here as data, read once by [`BubblewrapBackend::run`] and
/// compared once by `each_removal_changes_exactly_its_own_flags`, so the run and
/// the assertion cannot drift apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpawnOptions {
    /// `timeout -k` wraps the confinement argv.
    pub(crate) wall_bounded: bool,
    /// `RLIMIT_FSIZE` is applied to the child.
    pub(crate) file_size_capped: bool,
    /// Inherited descriptors above 2 are marked close-on-exec.
    pub(crate) descriptors_closed: bool,
    /// Descriptors 0, 1 and 2 come from the parent-owned endpoints
    /// [`Execution::stdio`] names, rather than from descriptors the caller owns.
    pub(crate) parent_owned_stdio: bool,
}

impl SpawnOptions {
    /// The confining profile — `None` — selects every option; each axis switches
    /// exactly one of them off and leaves the other three alone.
    pub(crate) fn under(weakening: Option<&Weakening>) -> Self {
        Self {
            wall_bounded: !matches!(weakening, Some(Weakening::WallBound)),
            file_size_capped: !matches!(weakening, Some(Weakening::FileSizeBound)),
            descriptors_closed: !matches!(weakening, Some(Weakening::Descriptors)),
            parent_owned_stdio: !matches!(weakening, Some(Weakening::StdioOwned { .. })),
        }
    }
}

// ---------------------------------------------------------------------------
// The backend
// ---------------------------------------------------------------------------

/// The Linux confinement profile.
///
/// Borrows [`HostFacts`] at construction (`D2`): [`CapsuleBackend::execute`] is
/// fixed by PHASE-04 and carries no host, but `EX-13`'s `PATH` derivation needs
/// one at execute time. Still dyn-compatible, which PHASE-06 `EX-6` requires.
pub(crate) struct BubblewrapBackend<'h> {
    host: &'h dyn HostFacts,
    kill_grace: Duration,
}

impl std::fmt::Debug for BubblewrapBackend<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BubblewrapBackend")
            .field("kill_grace", &self.kill_grace)
            .finish_non_exhaustive()
    }
}

impl<'h> BubblewrapBackend<'h> {
    /// The only constructor (`D2`).
    pub(crate) const fn new(host: &'h dyn HostFacts) -> Self {
        Self {
            host,
            kill_grace: DEFAULT_KILL_GRACE,
        }
    }

    /// The configured `execution-kill-grace-seconds`, when the caller has one.
    ///
    /// A builder rather than a constructor parameter, so `D2`'s
    /// [`BubblewrapBackend::new`] shape is unchanged and PHASE-06 still has a
    /// route for the configured figure.
    pub(crate) fn with_kill_grace(mut self, kill_grace: Duration) -> Self {
        self.kill_grace = kill_grace;
        self
    }

    /// Whether `name` is reachable through the host's `PATH`.
    ///
    /// Through [`HostFacts`] rather than `which`, which is what makes
    /// [`BubblewrapBackend::availability`] assertable against a table instead
    /// of a host (`EN-4`).
    fn executable_on_path(&self, name: &str) -> bool {
        let Some(raw) = self.host.env_var(HOST_PATH_VARIABLE) else {
            return false;
        };
        std::env::split_paths(&raw).any(|dir| self.host.path_exists(&dir.join(name)))
    }
}

impl CapsuleBackend for BubblewrapBackend<'_> {
    fn id(&self) -> BackendId {
        BACKEND_ID
    }

    /// What is missing **and** what would satisfy it (`EX-1`, `POL-002`
    /// facet 3). Never a suite that skips green: an absent mechanism is
    /// reported, not stepped over.
    fn availability(&self) -> Availability {
        for (executable, remedy) in [
            (
                BWRAP_EXECUTABLE,
                "install bubblewrap (package `bubblewrap`) so that `bwrap` is on PATH",
            ),
            (
                TIMEOUT_EXECUTABLE,
                "install coreutils so that `timeout` is on PATH",
            ),
        ] {
            if !self.executable_on_path(executable) {
                return Availability::Unavailable {
                    missing: executable.to_owned(),
                    remedy: remedy.to_owned(),
                };
            }
        }
        Availability::Available
    }

    /// Run `execution` inside a capsule shaped by `placement`.
    ///
    /// Rust owns the orchestration and generates no shell (`DEC-160`): quoting
    /// on a security boundary is the hazard that trade buys out of.
    ///
    /// The order below is load-bearing. The descriptor sweep runs **before**
    /// the status channel is cleared of `CLOEXEC`, because the sweep would
    /// otherwise mark the very descriptor bubblewrap is told to write to; and
    /// the status file is read and removed **before** `disk_used` is measured,
    /// because it is the trusted side's bookkeeping and not the capsule's
    /// residue.
    fn execute(
        &self,
        placement: &CapsulePlacement,
        execution: &Execution,
    ) -> Result<Observation, BackendError> {
        self.run(placement, execution, &WeakenedProfile::confining())
    }
}

impl BubblewrapBackend<'_> {
    /// The one confining profile, parameterised (`D2`, `T3`).
    ///
    /// [`CapsuleBackend::execute`] is this with nothing selected. There is no
    /// second assembly of the profile anywhere in the tree — a conformance
    /// suite that re-implemented it would be testing its own copy, on a
    /// security boundary, and the copy is the thing guaranteed to drift.
    ///
    /// The order below is load-bearing and unchanged by any weakening. The
    /// descriptor sweep runs **before** the status channel is cleared of
    /// `CLOEXEC`, because the sweep would otherwise mark the very descriptor
    /// bubblewrap is told to write to; and the status file is read and removed
    /// **before** `disk_used` is measured, because it is the trusted side's
    /// bookkeeping and not the capsule's residue.
    pub(crate) fn run(
        &self,
        placement: &CapsulePlacement,
        execution: &Execution,
        profile: &WeakenedProfile<'_>,
    ) -> Result<Observation, BackendError> {
        if let Availability::Unavailable { .. } = self.availability() {
            return Err(BackendError::Unavailable { id: BACKEND_ID });
        }

        let bound = identity_bound_paths(placement);
        let inner_path = render_path_list(&derived_inner_path(self.host, &bound));
        let environment = capsule_environment(execution.env(), &inner_path);

        let status_path = placement.root().path().join(STATUS_FILE_LEAF);
        let status_file = File::create(&status_path).map_err(|error| mechanism_failed(&error))?;

        let confinement = confinement_argv(
            placement,
            &environment,
            status_file.as_raw_fd(),
            execution.argv(),
            profile.weakening(),
        );
        let options = SpawnOptions::under(profile.weakening());
        let argv = if options.wall_bounded {
            wall_bounded_argv(execution.timeout(), self.kill_grace, &confinement)
        } else {
            confinement
        };
        let (program, arguments) =
            argv.split_first()
                .ok_or_else(|| BackendError::MechanismFailed {
                    detail: "the assembled argument vector is empty".to_owned(),
                })?;

        let mut command = Command::new(program);
        command.args(arguments);
        let [input, output, errors] = match profile.weakening() {
            Some(Weakening::StdioOwned {
                input,
                output,
                errors,
            }) => [
                owned_stdio(input)?,
                owned_stdio(output)?,
                owned_stdio(errors)?,
            ],
            _ => standard_stream_endpoints(execution.stdio()).map(endpoint_stdio),
        };
        command.stdin(input).stdout(output).stderr(errors);
        if options.file_size_capped {
            apply_file_size_cap(&mut command, execution.file_size_cap());
        }

        #[cfg(test)]
        let _window = serialised_descriptor_window();
        if options.descriptors_closed {
            mark_inherited_descriptors_close_on_exec().map_err(|error| mechanism_failed(&error))?;
        }
        clear_close_on_exec(&status_file).map_err(|error| mechanism_failed(&error))?;

        let observed = spawn_and_wait(command, profile.observer)?;
        drop(status_file);

        let status_text = std::fs::read_to_string(&status_path).unwrap_or_default();
        let child_ran = status_text.contains(STATUS_EXIT_CODE_KEY);
        std::fs::remove_file(&status_path).map_err(|error| mechanism_failed(&error))?;

        Ok(Observation {
            termination: classify_termination(exit_report(observed.status), child_ran),
            stdout: observed.stdout,
            stderr: observed.stderr,
            disk_used: bytes_beneath(placement.root().path())
                .map_err(|error| mechanism_failed(&error))?,
        })
    }
}

/// Keeps two `run` calls out of each other's descriptor window (`F-31`).
///
/// [`mark_inherited_descriptors_close_on_exec`] and [`clear_close_on_exec`]
/// both mutate **process-wide** descriptor flags, and what reads those flags is
/// `fork`. Two runs in flight at once therefore corrupt each other's handover:
/// measured on this host, a capsule was spawned with `--json-status-fd 4` and no
/// descriptor 4 — another transaction's status file was sitting at 6 — and
/// blocked at bubblewrap's user-namespace handshake for ever, holding the
/// harness's capture pipe, so the arm that spawned it never returned.
///
/// **Test-only, and not the fix.** The hazard is the mechanism's, not the
/// suite's: it is live for any caller that runs two capsules at once, which is
/// what `ArmShape::Concurrent` (row B5) is. Narrowing the window to the fork
/// itself, in production, is what closes it. This keeps `cargo test`'s parallel
/// runner — today's only multi-threaded caller — off a defect it did not
/// introduce, so the phase's evidence is about the phase.
#[cfg(test)]
fn serialised_descriptor_window() -> std::sync::MutexGuard<'static, ()> {
    static WINDOW: std::sync::Mutex<()> = std::sync::Mutex::new(());
    WINDOW
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// A caller-owned descriptor as a child endpoint, duplicated rather than
/// consumed: the same profile may be run more than once, and the caller keeps
/// the other end.
fn owned_stdio(descriptor: &OwnedFd) -> Result<Stdio, BackendError> {
    descriptor
        .try_clone()
        .map(Stdio::from)
        .map_err(|error| mechanism_failed(&error))
}

/// Run to completion, giving `observer` the child's host-side pid while it is
/// alive.
///
/// Without an observer this is exactly `Command::output()`. With one it is
/// `spawn` + `wait_with_output`, which is what `output()` does internally — the
/// callback goes in the window between them, and `wait_with_output` is what
/// keeps the piped stdout drained rather than deadlocked against a capsule
/// filling the pipe.
///
/// **The pid handed over is the immediate child's**, which under the wall bound
/// is `timeout(1)`, not the capsule's top-level process. `T5` replaces this
/// with the capsule's own — `REQ-448` criterion 3 wants the trusted parent's
/// observation of the *subject*. The seam is here; the descent is not.
fn spawn_and_wait(
    mut command: Command,
    observer: Option<&dyn Fn(i32)>,
) -> Result<std::process::Output, BackendError> {
    let Some(observer) = observer else {
        return command.output().map_err(|error| mechanism_failed(&error));
    };

    let child = command.spawn().map_err(|error| mechanism_failed(&error))?;
    observer(host_pid(&child));
    child
        .wait_with_output()
        .map_err(|error| mechanism_failed(&error))
}

/// `Child::id` is a `u32` and a pid is an `i32`; `as` is denied, and a pid
/// large enough to fail this conversion is a kernel that has changed shape.
fn host_pid(child: &std::process::Child) -> i32 {
    i32::try_from(child.id()).unwrap_or(-1)
}

/// The one shape a host-side I/O failure takes in this backend — never a
/// capsule's own nonzero exit (`EX-14`). `pub(crate)` because `conformance.rs`'s
/// `impl ConformanceBackend` opens descriptors on this backend's behalf and owes
/// its failures the same shape.
pub(crate) fn mechanism_failed(error: &io::Error) -> BackendError {
    BackendError::MechanismFailed {
        detail: error.to_string(),
    }
}

// ---------------------------------------------------------------------------
// T5: the declared readable set, and closure expansion
// ---------------------------------------------------------------------------

/// The largest resolver output this backend will look at.
///
/// Checked **first** (`D5` step 1), because an unbounded read is the hazard the
/// bound exists for.
const RESOLVER_OUTPUT_LIMIT: usize = 1 << 20;

/// Why a readable set could not be built, each variant carrying the path, the
/// resolver, or the figure it is about (`D6`).
///
/// Structured, never formatted, and for the reason `notes.md` item 17 records:
/// a fieldless variant keeps the offending entry inside the variant's
/// identifier, where no test can assert it and no operator can grep it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProfileRefusal {
    /// A `readable-roots` entry that does not exist.
    AbsentReadableRoot { path: PathBuf },
    /// A `closure-roots` entry that is not realised. **Provisioning realises
    /// nothing** (`EX-12`): the operator builds the toolchain out of band, and
    /// a resolver is never asked to evaluate anything.
    UnrealisedClosureRoot { path: PathBuf },
    /// An entry that exists but cannot be fully resolved (`EX-14`).
    UnresolvableEntry { path: PathBuf },
    /// The resolver's own nonzero exit, or no status at all.
    ResolverFailed { resolver: Argv, status: Option<i32> },
    /// A resolver line that is empty or not absolute — output is data, never
    /// trust (`EX-11`).
    ResolverPathNotAbsolute { line: String },
    /// A resolver path that does not exist on this host.
    ResolverPathAbsent { path: PathBuf },
    /// Resolver output above [`RESOLVER_OUTPUT_LIMIT`].
    ResolverOutputTooLarge { bytes: usize, limit: usize },
    /// Every declared source produced nothing. A capsule with no readable
    /// input can execute nothing, and silently producing one would report a
    /// confinement success that is really a configuration failure.
    EmptyReadableSet,
}

impl ProfileRefusal {
    /// The paths this refusal is about, structured rather than formatted.
    pub(crate) fn paths(&self) -> &[PathBuf] {
        match self {
            Self::AbsentReadableRoot { path }
            | Self::UnrealisedClosureRoot { path }
            | Self::UnresolvableEntry { path }
            | Self::ResolverPathAbsent { path } => std::slice::from_ref(path),
            Self::ResolverFailed { .. }
            | Self::ResolverPathNotAbsolute { .. }
            | Self::ResolverOutputTooLarge { .. }
            | Self::EmptyReadableSet => &[],
        }
    }

    /// The `[capsule]` keys an operator must edit, taken from `config`'s own
    /// constants and never a fresh literal (`STD-001`).
    ///
    /// [`ProfileRefusal::UnresolvableEntry`] names both list keys: the rule is
    /// the same one either list's entry failed, and narrowing it would need a
    /// second variant that says nothing new.
    pub(crate) fn keys(&self) -> &[&'static str] {
        match self {
            Self::AbsentReadableRoot { .. } => &[KEY_READABLE_ROOTS],
            Self::UnrealisedClosureRoot { .. } => &[KEY_CLOSURE_ROOTS],
            Self::UnresolvableEntry { .. } | Self::EmptyReadableSet => {
                &[KEY_READABLE_ROOTS, KEY_CLOSURE_ROOTS]
            }
            Self::ResolverFailed { .. }
            | Self::ResolverPathNotAbsolute { .. }
            | Self::ResolverPathAbsent { .. }
            | Self::ResolverOutputTooLarge { .. } => &[KEY_CLOSURE_RESOLVER],
        }
    }
}

/// What a closure resolver produced: **spawn and collect only, no policy**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QueryOutput {
    pub(crate) exit_status: Option<i32>,
    pub(crate) stdout: Vec<u8>,
}

/// Running the operator's declared closure resolver.
///
/// The seam is split pure/impure and **every policy decision is on the pure
/// side** (`D4`). A seam returning `Result<Vec<PathBuf>, ProfileRefusal>`
/// directly would move the refusals into the test fixture, and
/// `resolver_nonzero_exit_refuses_naming_the_resolver` would then pass whether
/// the production code mapped a nonzero exit or not.
///
/// The **caller** checks `closure-resolver[0]`'s normalized basename against
/// the transaction's `trusted_side_forbidden_executables` (PHASE-06 `EX-9`,
/// `VT-5`). It is not done here, and a reader who does not see it named here
/// would reasonably conclude it does not exist.
pub(crate) trait ClosureQuery {
    /// Run `resolver` against one already-realised path.
    fn query(&self, resolver: &Argv, realised: &Path) -> io::Result<QueryOutput>;
}

/// The production [`ClosureQuery`]: one spawn, no policy.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpawnedClosureQuery;

impl ClosureQuery for SpawnedClosureQuery {
    fn query(&self, resolver: &Argv, realised: &Path) -> io::Result<QueryOutput> {
        let words = resolver.as_slice();
        let (program, arguments) = words
            .split_first()
            .ok_or_else(|| io::Error::other("the resolver argument vector is empty"))?;
        let observed = Command::new(program)
            .args(arguments)
            .arg(realised)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        Ok(QueryOutput {
            exit_status: observed.status.code(),
            stdout: observed.stdout,
        })
    }
}

/// The **pure** half of the resolver seam (`D4`), and its check order is
/// load-bearing (`D5`): size, then status, then per-line shape.
///
/// **Status before parsing**, because a resolver that failed *and* printed
/// plausible paths must refuse *as a failed resolver*. With the checks the
/// other way round the nonzero-exit rule is unreachable and its mandated test
/// is vacuous.
fn closure_members(output: &QueryOutput, resolver: &Argv) -> Result<Vec<PathBuf>, ProfileRefusal> {
    if output.stdout.len() > RESOLVER_OUTPUT_LIMIT {
        return Err(ProfileRefusal::ResolverOutputTooLarge {
            bytes: output.stdout.len(),
            limit: RESOLVER_OUTPUT_LIMIT,
        });
    }
    if output.exit_status != Some(0) {
        return Err(ProfileRefusal::ResolverFailed {
            resolver: resolver.clone(),
            status: output.exit_status,
        });
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut members = Vec::new();
    for line in text.lines() {
        let candidate = Path::new(line.trim());
        if !candidate.is_absolute() {
            return Err(ProfileRefusal::ResolverPathNotAbsolute {
                line: line.to_owned(),
            });
        }
        members.push(candidate.to_path_buf());
    }
    Ok(members)
}

/// `EX-14`, once: an entry that exists but does not fully resolve refuses.
fn resolve_or_refuse(host: &dyn HostFacts, path: &Path) -> Result<PathBuf, ProfileRefusal> {
    host.resolve(path)
        // The refusal is structured on purpose (`notes.md` item 17): the
        // operator needs the path and the key, and an `io::Error` string in a
        // field no test can assert is not an improvement on either.
        .map_err(|_source| ProfileRefusal::UnresolvableEntry {
            path: path.to_path_buf(),
        })
}

/// One `readable-roots` entry: probed, then resolved (`EX-9`, `EX-14`).
///
/// The existence probe is separate from the `closure-roots` one on purpose:
/// the two refusals name different keys, and sharing one guard would make an
/// absent declared root and an unrealised toolchain indistinguishable to the
/// operator reading the refusal.
fn resolved_readable_root(
    host: &dyn HostFacts,
    declared: &Path,
) -> Result<PathBuf, ProfileRefusal> {
    if !host.path_exists(declared) {
        return Err(ProfileRefusal::AbsentReadableRoot {
            path: declared.to_path_buf(),
        });
    }
    resolve_or_refuse(host, declared)
}

/// One `closure-roots` entry: probed, then resolved (`EX-12`, `EX-14`).
fn resolved_closure_root(host: &dyn HostFacts, declared: &Path) -> Result<PathBuf, ProfileRefusal> {
    if !host.path_exists(declared) {
        return Err(ProfileRefusal::UnrealisedClosureRoot {
            path: declared.to_path_buf(),
        });
    }
    resolve_or_refuse(host, declared)
}

/// Expand one realised closure root into its members, each bound
/// **individually** (`EX-10`) and each resolved before anything looks at it.
///
/// `D5` step 4 — existence and resolution of each returned path — lives here
/// rather than in [`closure_members`], because it needs the host and
/// [`closure_members`] is the pure half.
fn expand_closure_root(
    host: &dyn HostFacts,
    query: &dyn ClosureQuery,
    resolver: &Argv,
    realised: &Path,
) -> Result<Vec<PathBuf>, ProfileRefusal> {
    let output =
        query
            .query(resolver, realised)
            .map_err(|_source| ProfileRefusal::ResolverFailed {
                resolver: resolver.clone(),
                status: None,
            })?;

    let mut resolved = Vec::new();
    for member in closure_members(&output, resolver)? {
        if !host.path_exists(&member) {
            return Err(ProfileRefusal::ResolverPathAbsent { path: member });
        }
        resolved.push(resolve_or_refuse(host, &member)?);
    }
    Ok(resolved)
}

/// The readable set, in declared order: `readable-roots` as declared, then each
/// `closure-roots` entry's members.
///
/// There is **no host-shaped default and no fallback**. Only the two declared
/// lists ever become readable inputs (`EX-8`), which is why no arrangement of
/// configuration — including the branches taken when a list or the resolver is
/// absent — can make a host-wide artefact store readable whole.
fn readable_paths(
    readable_roots: &[PathBuf],
    closure_roots: &[PathBuf],
    resolver: Option<&Argv>,
    host: &dyn HostFacts,
    query: &dyn ClosureQuery,
) -> Result<Vec<PathBuf>, ProfileRefusal> {
    let mut bound = Vec::new();

    for declared in readable_roots {
        bound.push(resolved_readable_root(host, declared)?);
    }

    // A non-empty `closure-roots` with no resolver is already refused by
    // `config::ConfigRefusal::ClosureRootsWithoutResolver`, so the `None` arm
    // reaches here only with an empty list — and if it ever did not, the
    // empty-set rule below is the fail-closed floor.
    if let Some(resolver) = resolver {
        for declared in closure_roots {
            let realised = resolved_closure_root(host, declared)?;
            bound.extend(expand_closure_root(host, query, resolver, &realised)?);
        }
    }

    if bound.is_empty() {
        return Err(ProfileRefusal::EmptyReadableSet);
    }
    Ok(bound)
}

/// The seam PHASE-06's provisioning calls (`D3`).
///
/// The backend itself chooses no path (invariant 1): resolution happens here,
/// *before* [`CapsulePlacement::try_new`] validates, and what is validated is
/// what is bound.
pub(crate) fn readable_set(
    config: &CapsuleConfig,
    host: &dyn HostFacts,
    query: &dyn ClosureQuery,
) -> Result<Vec<PathBuf>, ProfileRefusal> {
    readable_paths(
        config.readable_roots(),
        config.closure_roots(),
        config.closure_resolver(),
        host,
        query,
    )
}

// ---------------------------------------------------------------------------
// T6: the derived inner PATH
// ---------------------------------------------------------------------------

/// **Directional** containment: `entry` is equal to or a descendant of `bound`.
///
/// Deliberately *not* [`crate::backend`]'s `overlaps`, which is bidirectional
/// (PHASE-04 `EX-6`) and would admit `/usr/bin` on the strength of a bound
/// *file* `/usr/bin/git`. Two rules, two functions; using one for the other
/// fails in opposite directions. Component-wise, never textual.
fn is_within(entry: &Path, bound: &Path) -> bool {
    entry.starts_with(bound)
}

/// The host `PATH` entries beneath any bound path, in host order, deduplicated.
///
/// An entry that does not resolve is **dropped, never refused** (`D8`): the
/// host `PATH` is the operator's environment, not this project's
/// configuration, and refusing a capsule because the trusted side holds a stale
/// entry is the over-denial direction one level down from `EX-3`.
///
/// A `Vec` for the result and a [`BTreeSet`] as the membership guard (`D9`):
/// a set alone reorders, and host `PATH` order is what
/// `inner_path_draws_from_every_bound_path_in_host_path_order` asserts.
fn derived_inner_path(host: &dyn HostFacts, bound: &[PathBuf]) -> Vec<PathBuf> {
    let Some(raw) = host.env_var(HOST_PATH_VARIABLE) else {
        return Vec::new();
    };

    let mut seen: BTreeSet<PathBuf> = BTreeSet::new();
    let mut derived = Vec::new();
    for entry in std::env::split_paths(&raw) {
        let Ok(resolved) = host.resolve(&entry) else {
            continue;
        };
        if !bound.iter().any(|root| is_within(&resolved, root)) {
            continue;
        }
        if seen.insert(resolved.clone()) {
            derived.push(resolved);
        }
    }
    derived
}

/// The bound paths a host `PATH` entry can be measured against.
///
/// Only the **identity-mapped** readable entries: a declared readable input
/// keeps its resolved host path as its inner path, so a host `PATH` entry
/// beneath one names the same place inside. The source export and the
/// transaction's own writable areas are bound at fixed inner destinations
/// (`/source`, `/capsule`, `/agent`), so a host `PATH` entry beneath them
/// would name a path that does not exist inside the capsule.
fn identity_bound_paths(placement: &CapsulePlacement) -> Vec<PathBuf> {
    placement
        .readable()
        .iter()
        .filter(|entry| entry.inner().as_path() == entry.host())
        .map(|entry| entry.host().to_path_buf())
        .collect()
}

fn render_path_list(entries: &[PathBuf]) -> String {
    entries
        .iter()
        .map(|entry| entry.to_string_lossy().into_owned())
        .collect::<Vec<String>>()
        .join(PATH_SEPARATOR)
}

/// The capsule's environment as `(name, value)` pairs, **sorted by name**
/// (`EX-4`). Every value is either a constant of the trusted side or the
/// derived `PATH`; no caller-supplied text reaches here (`EX-11`).
fn capsule_environment(env: &CapsuleEnv, inner_path: &str) -> Vec<(&'static str, String)> {
    let mut pairs: Vec<(&'static str, String)> = env
        .vars()
        .map(|var| {
            let value = var
                .fixed_value()
                .map_or_else(|| inner_path.to_owned(), str::to_owned);
            (var.name(), value)
        })
        .collect();
    pairs.sort_by_key(|(name, _)| *name);
    pairs
}

// ---------------------------------------------------------------------------
// T7: argv assembly, in EX-4's order
// ---------------------------------------------------------------------------

/// A profile-owned mount's host side, derived from the inner constant so the
/// layout has one spelling: `/capsule` under the transaction root is
/// `<root>/capsule`.
pub(crate) fn profile_owned_host_path(root: &TransactionRoot, inner: &str) -> PathBuf {
    root.path().join(inner.trim_start_matches('/'))
}

fn push_bind(argv: &mut Vec<String>, flag: &str, host: &Path, inner: &Path) {
    argv.push(flag.to_owned());
    argv.push(host.to_string_lossy().into_owned());
    argv.push(inner.to_string_lossy().into_owned());
}

/// The `bwrap` invocation, in `EX-4`'s order.
///
/// `--tmpfs /tmp` precedes the `--ro-bind` block and that is **not**
/// bookkeeping: measured (`F-4`), the reverse order leaves `/tmp` empty inside
/// the capsule, so a declared readable input resolving beneath `/tmp`
/// disappears silently with exit 0.
///
/// The three profile-owned mounts (`/source`, `/capsule`, `/agent`) are emitted
/// ahead of the declared entries of their block. They cannot arrive through the
/// declared vectors — `RESERVED_INNER_DESTINATIONS` refuses an entry naming
/// them — so the profile derives them from the placement's typed fields.
///
/// `weakening` switches off exactly one axis and nothing else — that *nothing
/// else* is the property `each_removal_changes_exactly_its_own_flags` asserts
/// against this function's output, and the reason each branch below is a
/// conditional over the confining assembly rather than a second assembly.
pub(crate) fn confinement_argv(
    placement: &CapsulePlacement,
    environment: &[(&'static str, String)],
    status_fd: RawFd,
    argv: &Argv,
    weakening: Option<&Weakening>,
) -> Vec<String> {
    let network_permitted = placement.network() == NetworkPosture::Permitted;
    let enumerated = matches!(weakening, Some(Weakening::ProcessVisibility));

    let mut assembled = vec![
        BWRAP_EXECUTABLE.to_owned(),
        FLAG_JSON_STATUS_FD.to_owned(),
        status_fd.to_string(),
    ];

    if enumerated {
        // `EX-9`/`D5`: no `--share-pid` exists, so this axis names the
        // namespaces that remain. **And the network posture moves with it** —
        // `--share-net` is bubblewrap's only re-share flag and pairs with
        // `--unshare-all`, so under the enumerated set a permitted network is
        // expressed by omitting `--unshare-net` instead. Emitting both is an
        // argv error, which is a control the mechanism refuses to build.
        assembled.extend(
            NON_PID_UNSHARE_SET
                .iter()
                .filter(|flag| !(network_permitted && **flag == FLAG_UNSHARE_NET))
                .map(|flag| (*flag).to_owned()),
        );
    } else {
        assembled.push(FLAG_UNSHARE_ALL.to_owned());
    }

    if !matches!(weakening, Some(Weakening::MappedIdentity)) {
        assembled.extend([
            FLAG_UID.to_owned(),
            CAPSULE_UID.to_string(),
            FLAG_GID.to_owned(),
            CAPSULE_GID.to_string(),
        ]);
    }
    assembled.extend(
        [
            FLAG_PROC, INNER_PROC, FLAG_DEV, INNER_DEV, FLAG_TMPFS, INNER_TMP,
        ]
        .map(str::to_owned),
    );

    // Attachment alone: same paths, same inner destinations, same count.
    let input_flag = if matches!(weakening, Some(Weakening::InputsWritable)) {
        FLAG_BIND
    } else {
        FLAG_RO_BIND
    };
    push_bind(
        &mut assembled,
        input_flag,
        placement.source().host(),
        Path::new(INNER_SOURCE),
    );
    for entry in placement.readable() {
        push_bind(
            &mut assembled,
            input_flag,
            entry.host(),
            entry.inner().as_path(),
        );
    }

    for inner in [INNER_CAPSULE, INNER_AGENT] {
        push_bind(
            &mut assembled,
            FLAG_BIND,
            &profile_owned_host_path(placement.root(), inner),
            Path::new(inner),
        );
    }
    for entry in placement.writable() {
        push_bind(
            &mut assembled,
            FLAG_BIND,
            entry.host(),
            entry.inner().as_path(),
        );
    }

    if !matches!(weakening, Some(Weakening::WorkingDirectory)) {
        assembled.push(FLAG_CHDIR.to_owned());
        assembled.push(
            placement
                .working_directory()
                .as_path()
                .to_string_lossy()
                .into_owned(),
        );
    }
    if !matches!(weakening, Some(Weakening::Teardown)) {
        assembled.push(FLAG_DIE_WITH_PARENT.to_owned());
    }
    assembled.push(FLAG_NEW_SESSION.to_owned());
    if !matches!(weakening, Some(Weakening::EnvironmentCleared)) {
        assembled.push(FLAG_CLEARENV.to_owned());
    }
    // The `--setenv` list stays byte-identical under every axis, including the
    // one that drops `--clearenv` — `VA-4` names this.
    for (name, value) in environment {
        assembled.push(FLAG_SETENV.to_owned());
        assembled.push((*name).to_owned());
        assembled.push(value.clone());
    }
    if network_permitted && !enumerated {
        assembled.push(FLAG_SHARE_NET.to_owned());
    }
    if matches!(weakening, Some(Weakening::AllCapabilities)) {
        assembled.push(FLAG_CAP_ADD.to_owned());
        assembled.push(CAPABILITY_ALL.to_owned());
    }

    assembled.extend(argv.as_slice().iter().cloned());
    assembled
}

/// `timeout -k <grace> <secs> bwrap …` — the wall bound wraps the exec from
/// **outside**, where the capsule cannot reach it (`EX-15`).
fn wall_bounded_argv(
    timeout: Duration,
    kill_grace: Duration,
    confinement: &[String],
) -> Vec<String> {
    let mut argv = vec![
        TIMEOUT_EXECUTABLE.to_owned(),
        KILL_GRACE_FLAG.to_owned(),
        kill_grace.as_secs().to_string(),
        timeout.as_secs().to_string(),
    ];
    argv.extend_from_slice(confinement);
    argv
}

// ---------------------------------------------------------------------------
// T8: the streams, the termination table, and the disk figure
// ---------------------------------------------------------------------------

/// What the parent puts at descriptors 0, 1 and 2 — **replaced**, never marked
/// close-on-exec (invariant 14, `RV-346` `F-30`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StreamEndpoint {
    /// Reads return EOF immediately.
    EmptySource,
    /// A one-way endpoint the parent created and reads.
    CapturedOutput,
}

const fn standard_stream_endpoints(stdio: CapsuleStdio) -> [StreamEndpoint; 3] {
    match stdio {
        CapsuleStdio::EmptyInputCapturedOutput => [
            StreamEndpoint::EmptySource,
            StreamEndpoint::CapturedOutput,
            StreamEndpoint::CapturedOutput,
        ],
    }
}

fn endpoint_stdio(endpoint: StreamEndpoint) -> Stdio {
    match endpoint {
        StreamEndpoint::EmptySource => Stdio::null(),
        StreamEndpoint::CapturedOutput => Stdio::piped(),
    }
}

/// What the parent waited on, as the two facts a wait status carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExitReport {
    code: Option<i32>,
    signal: Option<i32>,
}

fn exit_report(status: ExitStatus) -> ExitReport {
    ExitReport {
        code: status.code(),
        signal: status.signal(),
    }
}

/// The measured mapping (`D12`), from execution rather than documentation.
///
/// **Known ambiguity, recorded and not solved:** `timeout(1)` collapses "the
/// capsule called `exit(143)`" and "the capsule was killed by `SIGTERM`" into
/// the same status. PHASE-09 row 8 must choose unambiguous payloads; this
/// function cannot tell them apart and does not pretend to.
fn classify_termination(report: ExitReport, child_ran: bool) -> Termination {
    if let Some(signal) = report.signal {
        return Termination::Signalled { signal };
    }
    let Some(code) = report.code else {
        return Termination::NotExecutable;
    };
    if code == TIMEOUT_EXIT_CODE {
        return Termination::TimedOut;
    }
    if let Some(signal) = code.checked_sub(SIGNALLED_EXIT_BASE).filter(|n| *n > 0) {
        if signal == SIGXFSZ {
            return Termination::FileSizeExceeded;
        }
        return Termination::Signalled { signal };
    }
    if child_ran {
        Termination::Exited { code }
    } else {
        Termination::NotExecutable
    }
}

/// Bytes resident beneath the transaction root, computed **trusted-side after
/// the run** (`EX-15`): a per-file cap does not catch a capsule that writes
/// many small files.
///
/// Symlinks are counted as entries, never followed — a capsule-created link out
/// of the tree must not add someone else's bytes to this figure.
fn bytes_beneath(root: &Path) -> io::Result<ByteCount> {
    let mut total: u64 = 0;
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                pending.push(entry.path());
            } else {
                total = total.saturating_add(metadata.len());
            }
        }
    }
    Ok(ByteCount::from_bytes(total))
}

// ---------------------------------------------------------------------------
// T9: the parent-side descriptor sweep
// ---------------------------------------------------------------------------

/// The sweep's rule, pure: everything at or above [`SWEEP_FLOOR`], minus the
/// handle the enumeration itself holds (`D14`).
///
/// `read_dir` on `/proc/self/fd` holds an open descriptor that appears in its
/// own listing. Marking it is harmless; counting it as inherited makes the
/// returned count wrong and the caller's test brittle.
fn inherited_descriptors(listed: &[RawFd], own: RawFd) -> Vec<RawFd> {
    listed
        .iter()
        .copied()
        .filter(|fd| *fd >= SWEEP_FLOOR && *fd != own)
        .collect()
}

/// Mark every inherited descriptor close-on-exec, in the parent and before the
/// fork (`EX-16`, `EX-18`, invariant 12). Returns the count marked.
///
/// A standalone function so `VT-4` can call it (`D13`): a sweep buried inside
/// [`CapsuleBackend::execute`] would only be testable by running a capsule, and
/// `VT-4` is explicitly the pure, parent-side half — the executed half is
/// `sec-7` row 10 (PHASE-09).
///
/// **Two facts bound what this is for.** Rust opens its own files `O_CLOEXEC`,
/// so this backend's own handles are already closed by construction; what the
/// sweep exists for is whatever the trusted-side process inherited from *its*
/// parent, or an FFI path opened without the flag. And the residual is a
/// descriptor opened between the sweep and the spawn: provisioning is
/// single-threaded through `sec-3`'s step list, so that window has no writer —
/// but that is a **known limit** resting on `sec-7` row 10 having seen the
/// sweep fire, not on this paragraph. The provisioning-time resolution race is
/// a known limit in the same register: a declared path could be re-pointed
/// after resolution, which closing would need the bind performed against an
/// open descriptor rather than a path.
///
/// A descriptor that vanishes between the listing and the marking is skipped
/// rather than failing the sweep: another thread closing a handle is not a
/// confinement failure.
fn mark_inherited_descriptors_close_on_exec() -> io::Result<usize> {
    let handle = File::open(PROC_SELF_FD)?;
    let own = handle.as_raw_fd();
    let mut directory = Dir::new(handle)?;

    let mut listed: Vec<RawFd> = Vec::new();
    while let Some(entry) = directory.read() {
        let entry = entry?;
        if let Ok(name) = entry.file_name().to_str()
            && let Ok(fd) = name.parse::<RawFd>()
        {
            listed.push(fd);
        }
    }

    let mut marked: usize = 0;
    for fd in inherited_descriptors(&listed, own) {
        // The one route from a raw descriptor number to something `rustix` will
        // accept. `/proc/self/fd` yields integers; `BorrowedFd::borrow_raw` is
        // the only constructor, and `close_range` — the direct form that would
        // need no descriptor at all — is absent from `rustix` 1.1.4 and would
        // need `libc`, which `Cargo.toml`'s zero-new-crates posture rules out.
        #[expect(
            unsafe_code,
            reason = "EX-16's descriptor sweep: /proc/self/fd yields raw integers and \
                      BorrowedFd::borrow_raw is the only route to an AsFd. Sound here — \
                      the descriptor was listed by this process an instant ago and the \
                      borrow does not outlive the loop body. One of exactly two sites \
                      (F-1/R); the_unsafe_budget_is_exactly_two_sites holds the count"
        )]
        let borrowed = unsafe { BorrowedFd::borrow_raw(fd) };
        let Ok(flags) = fcntl_getfd(borrowed) else {
            continue;
        };
        match fcntl_setfd(borrowed, flags | FdFlags::CLOEXEC) {
            Ok(()) => marked = marked.saturating_add(1),
            // The descriptor was closed between the read and the mark. Skipped,
            // and **only** for this errno: any other failure is a sweep that did
            // not do what it says, and silently continuing past one would be the
            // vacuous-guard class one level down.
            Err(Errno::BADF) => {}
            Err(other) => return Err(other.into()),
        }
    }
    Ok(marked)
}

/// Clear close-on-exec, so one descriptor the parent chose survives the exec.
///
/// Called **after** the sweep, on the status channel only: the sweep marks
/// everything above 2, and bubblewrap's `--json-status-fd` needs exactly one of
/// those to cross. Measured (`D11`): the status descriptor does not reach the
/// capsule — `/proc/self/fd` inside shows only 0, 1 and 2 — so this does not
/// weaken invariant 12.
fn clear_close_on_exec<Fd: AsFd>(fd: Fd) -> io::Result<()> {
    let flags = fcntl_getfd(&fd)?;
    fcntl_setfd(&fd, flags.difference(FdFlags::CLOEXEC))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// T10: the per-file size cap
// ---------------------------------------------------------------------------

/// Set `RLIMIT_FSIZE` on the **child**, before the `bwrap` exec (`EX-15`).
///
/// Outside the namespace and unreachable from inside it: a capsule cannot raise
/// a limit it has no privilege over, and the soft and hard limits are set
/// together so it cannot raise the soft one to the hard one either. The
/// external-wrapper escape that would parallel `timeout -k` is unavailable —
/// neither `prlimit` nor `setpriv` is on `PATH` in this project's jail.
fn apply_file_size_cap(command: &mut Command, cap: ByteCount) {
    let bytes = cap.as_u64();
    // `pre_exec` is unsafe because its closure runs between `fork` and `exec`,
    // where only async-signal-safe work is permitted. `setrlimit` is a single
    // syscall with no allocation, which is exactly what that contract allows.
    #[expect(
        unsafe_code,
        reason = "EX-15's per-file cap: CommandExt::pre_exec is the only route to set \
                  RLIMIT_FSIZE on the child rather than the parent. Sound here — the \
                  closure performs one allocation-free syscall, which satisfies \
                  pre_exec's async-signal-safety contract. One of exactly two sites \
                  (F-1/R); the_unsafe_budget_is_exactly_two_sites holds the count"
    )]
    unsafe {
        command.pre_exec(move || {
            setrlimit(
                Resource::Fsize,
                Rlimit {
                    current: Some(bytes),
                    maximum: Some(bytes),
                },
            )?;
            Ok(())
        });
    }
}

// ---------------------------------------------------------------------------
// Tests. Bin-only crate: every test is a `#[cfg(test)] mod` inside the unit it
// tests — a `tests/` file cannot link this package (PHASE-01 `EX-5`).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::io::Write as _;
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use super::*;
    use crate::backend::{
        AcceptedBase, ForbiddenScopes, InnerPath, MountedPath, PlacementParts, SourceExport,
    };
    use crate::config::{parse_capsule_config, root_capsule_config};
    use crate::host::fixture::FixtureHost;

    // ── Fixture geometry ───────────────────────────────────────────────────

    const CANONICAL_REPOSITORY: &str = "/srv/repo";
    const CONTROL_PLANE_STATE: &str = "/srv/repo/.doctrine";
    const CAPSULE_ROOT: &str = "/var/lib/doctrine";
    const CREDENTIALS: &str = "/home/agent/.ssh";
    const BASE_OID: &str = "1f0e3dad99908345f7439f8ffabdffc4";
    const TRANSACTION_ROOT: &str = "/var/lib/doctrine/tx/0001";
    const LAWFUL_EXPORT: &str = "/var/lib/doctrine/export/1f0e3dad99908345f7439f8ffabdffc4";
    const LAWFUL_HOST: &str = "/nix/store/bash";
    const STORE: &str = "/nix/store";

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

    /// A placement whose every entry is identity-mapped, which is the ordinary
    /// shape: a declared readable input keeps its resolved host path inside.
    fn placement_with(
        readable: Vec<MountedPath>,
        writable: Vec<MountedPath>,
        network: NetworkPosture,
    ) -> CapsulePlacement {
        CapsulePlacement::try_new(
            PlacementParts {
                root: TransactionRoot::new(PathBuf::from(TRANSACTION_ROOT)),
                source: SourceExport::new(
                    PathBuf::from(LAWFUL_EXPORT),
                    AcceptedBase::new(BASE_OID.to_owned()),
                ),
                writable,
                readable,
                working_directory: inner(INNER_CAPSULE),
                network,
                accepted_base: AcceptedBase::new(BASE_OID.to_owned()),
            },
            &scopes(),
        )
        .expect("the fixture placement is lawful")
    }

    fn lawful_placement() -> CapsulePlacement {
        placement_with(
            vec![mount(LAWFUL_HOST, LAWFUL_HOST)],
            Vec::new(),
            NetworkPosture::Denied,
        )
    }

    fn argv(words: &[&str]) -> Argv {
        Argv::try_new(words.iter().map(|word| (*word).to_owned()).collect())
            .expect("fixture argument vectors are non-empty")
    }

    fn assembled(placement: &CapsulePlacement) -> Vec<String> {
        confinement_argv(placement, &[], 9, &argv(&["/bin/true"]), None)
    }

    fn assembled_under(placement: &CapsulePlacement, weakening: &Weakening) -> Vec<String> {
        confinement_argv(placement, &[], 9, &argv(&["/bin/true"]), Some(weakening))
    }

    fn position(argv: &[String], token: &str) -> usize {
        argv.iter()
            .position(|word| word == token)
            .unwrap_or_else(|| panic!("`{token}` must appear in the assembled argv"))
    }

    /// A [`ClosureQuery`] that answers from a table and records that it was
    /// asked — "provisioning never realises it" is a claim about a call that
    /// never happened, so the call has to be observable.
    #[derive(Debug, Default)]
    struct FixtureQuery {
        answer: Option<QueryOutput>,
        asked: std::cell::RefCell<Vec<PathBuf>>,
    }

    impl FixtureQuery {
        fn answering(exit_status: Option<i32>, stdout: &str) -> Self {
            Self {
                answer: Some(QueryOutput {
                    exit_status,
                    stdout: stdout.as_bytes().to_vec(),
                }),
                asked: std::cell::RefCell::new(Vec::new()),
            }
        }

        fn never_asked(&self) -> bool {
            self.asked.borrow().is_empty()
        }
    }

    impl ClosureQuery for FixtureQuery {
        fn query(&self, _resolver: &Argv, realised: &Path) -> io::Result<QueryOutput> {
            self.asked.borrow_mut().push(realised.to_path_buf());
            self.answer
                .clone()
                .ok_or_else(|| io::Error::other("this fixture answers nothing"))
        }
    }

    fn resolver() -> Argv {
        argv(&["nix-store", "--query", "--requisites"])
    }

    fn refusal_of(
        readable_roots: &[&str],
        closure_roots: &[&str],
        host: &FixtureHost,
        query: &FixtureQuery,
    ) -> ProfileRefusal {
        set_of(readable_roots, closure_roots, host, query)
            .expect_err("this readable set must refuse")
    }

    fn set_of(
        readable_roots: &[&str],
        closure_roots: &[&str],
        host: &FixtureHost,
        query: &FixtureQuery,
    ) -> Result<Vec<PathBuf>, ProfileRefusal> {
        let readable: Vec<PathBuf> = readable_roots.iter().map(PathBuf::from).collect();
        let closure: Vec<PathBuf> = closure_roots.iter().map(PathBuf::from).collect();
        readable_paths(&readable, &closure, Some(&resolver()), host, query)
    }

    // ── T4: identity, availability, the declared credentials ───────────────

    #[test]
    fn availability_names_what_is_missing_and_the_remedy() {
        let bare = FixtureHost::new();
        let backend = BubblewrapBackend::new(&bare);
        let Availability::Unavailable { missing, remedy } = backend.availability() else {
            panic!("a host with no PATH cannot run this backend");
        };
        assert_eq!(missing, BWRAP_EXECUTABLE);
        assert!(
            remedy.contains("bubblewrap"),
            "the remedy must say what would satisfy it, got {remedy:?}"
        );

        // bubblewrap present, `timeout` still absent: the second executable is
        // named on its own, not folded into the first.
        let half = FixtureHost::new()
            .with_env("PATH", "/usr/bin")
            .with_resolution("/usr/bin/bwrap", "/usr/bin/bwrap");
        let Availability::Unavailable { missing, remedy } =
            BubblewrapBackend::new(&half).availability()
        else {
            panic!("a host without `timeout` cannot run this backend");
        };
        assert_eq!(missing, TIMEOUT_EXECUTABLE);
        assert!(remedy.contains("coreutils"), "got {remedy:?}");

        let whole = half.with_resolution("/usr/bin/timeout", "/usr/bin/timeout");
        assert_eq!(
            BubblewrapBackend::new(&whole).availability(),
            Availability::Available
        );
    }

    #[test]
    fn the_capsule_identity_is_declared_and_not_configurable() {
        let host = FixtureHost::new();
        let backend = BubblewrapBackend::new(&host);
        assert_eq!(backend.id(), BACKEND_ID);

        // Two placements that differ in every configurable way carry the same
        // credentials: there is no route by which an operator supplies one.
        let first = assembled(&lawful_placement());
        let second = assembled(&placement_with(
            vec![mount("/usr/bin", "/usr/bin"), mount("/bin/sh", "/bin/sh")],
            vec![mount(
                "/var/lib/doctrine/tx/0001/extra",
                "/var/lib/doctrine/tx/0001/extra",
            )],
            NetworkPosture::Permitted,
        ));
        for tokens in [&first, &second] {
            let uid = position(tokens, FLAG_UID);
            let gid = position(tokens, FLAG_GID);
            assert_eq!(tokens.get(uid + 1).map(String::as_str), Some("1000"));
            assert_eq!(tokens.get(gid + 1).map(String::as_str), Some("1000"));
        }
        assert_eq!(CAPSULE_UID, 1000);
        assert_eq!(CAPSULE_GID, 1000);
    }

    // ── T5: the declared readable set and closure expansion ────────────────

    #[test]
    fn absent_readable_root_refuses_naming_the_entry_and_the_config_key() {
        let host = FixtureHost::new();
        let query = FixtureQuery::default();
        let refusal = refusal_of(&["/opt/toolchain"], &[], &host, &query);

        assert_eq!(
            refusal,
            ProfileRefusal::AbsentReadableRoot {
                path: PathBuf::from("/opt/toolchain"),
            }
        );
        assert_eq!(refusal.paths(), [PathBuf::from("/opt/toolchain")]);
        assert_eq!(refusal.keys(), [KEY_READABLE_ROOTS]);
    }

    #[test]
    fn unrealised_closure_root_refuses_and_provisioning_never_realises_it() {
        let host = FixtureHost::new();
        let query = FixtureQuery::answering(Some(0), "/nix/store/a\n");
        let refusal = refusal_of(&[], &[".doctrine/capsule-toolchain"], &host, &query);

        assert_eq!(
            refusal,
            ProfileRefusal::UnrealisedClosureRoot {
                path: PathBuf::from(".doctrine/capsule-toolchain"),
            }
        );
        assert_eq!(refusal.keys(), [KEY_CLOSURE_ROOTS]);
        assert!(
            query.never_asked(),
            "an unrealised entry must refuse before the resolver is even asked — \
             provisioning realises nothing (EX-12)"
        );
        assert!(
            !host.path_exists(Path::new(".doctrine/capsule-toolchain")),
            "and nothing may have been created by the attempt"
        );
    }

    #[test]
    fn resolver_returning_a_relative_or_absent_path_refuses() {
        let host = FixtureHost::new().with_resolution("/toolchain", "/nix/store/toolchain");

        let relative = FixtureQuery::answering(Some(0), "nix/store/a\n");
        assert_eq!(
            refusal_of(&[], &["/toolchain"], &host, &relative),
            ProfileRefusal::ResolverPathNotAbsolute {
                line: "nix/store/a".to_owned(),
            }
        );

        let empty_line = FixtureQuery::answering(Some(0), "/nix/store/a\n\n");
        assert_eq!(
            refusal_of(&[], &["/toolchain"], &host, &empty_line),
            ProfileRefusal::ResolverPathNotAbsolute {
                line: String::new(),
            }
        );

        let absent = FixtureQuery::answering(Some(0), "/nix/store/never-realised\n");
        let refusal = refusal_of(&[], &["/toolchain"], &host, &absent);
        assert_eq!(
            refusal,
            ProfileRefusal::ResolverPathAbsent {
                path: PathBuf::from("/nix/store/never-realised"),
            }
        );
        assert_eq!(refusal.keys(), [KEY_CLOSURE_RESOLVER]);
    }

    #[test]
    fn resolver_nonzero_exit_refuses_naming_the_resolver() {
        let host = FixtureHost::new().with_resolution("/toolchain", "/nix/store/toolchain");
        // A resolver that failed *and* printed plausible paths. `D5` puts the
        // status check before the parse for exactly this fixture: with the
        // checks the other way round the refusal would name the path, and the
        // fact that the resolver itself failed would never be reported.
        let query = FixtureQuery::answering(Some(1), "/nix/store/plausible\n");
        let refusal = refusal_of(&[], &["/toolchain"], &host, &query);

        assert_eq!(
            refusal,
            ProfileRefusal::ResolverFailed {
                resolver: resolver(),
                status: Some(1),
            }
        );
        assert_eq!(refusal.keys(), [KEY_CLOSURE_RESOLVER]);
    }

    #[test]
    fn closure_expansion_binds_each_returned_path_individually_never_their_common_parent() {
        let host = FixtureHost::new()
            .with_resolution("/toolchain", "/nix/store/toolchain")
            .with_resolution("/nix/store/a", "/nix/store/a")
            .with_resolution("/nix/store/b", "/nix/store/b");
        let query = FixtureQuery::answering(Some(0), "/nix/store/a\n/nix/store/b\n");

        let bound = set_of(&[], &["/toolchain"], &host, &query).expect("this closure expands");
        assert_eq!(
            bound,
            vec![PathBuf::from("/nix/store/a"), PathBuf::from("/nix/store/b")]
        );
        assert!(
            !bound.contains(&PathBuf::from(STORE)),
            "the common parent is the whole store, which is what EX-8 forbids"
        );
    }

    #[test]
    fn a_single_file_readable_root_is_bound_as_declared() {
        // The shebang class: the kernel resolves an interpreter before `PATH`
        // exists, and binding `/bin` instead would silently widen the capsule.
        let host = FixtureHost::new().with_resolution("/bin/sh", "/bin/sh");
        let query = FixtureQuery::default();

        assert_eq!(
            set_of(&["/bin/sh"], &[], &host, &query).expect("a file entry is lawful"),
            vec![PathBuf::from("/bin/sh")]
        );
    }

    #[test]
    fn the_readable_set_is_never_empty() {
        let host = FixtureHost::new().with_resolution("/toolchain", "/nix/store/toolchain");
        let query = FixtureQuery::answering(Some(0), "");

        let refusal = refusal_of(&[], &["/toolchain"], &host, &query);
        assert_eq!(refusal, ProfileRefusal::EmptyReadableSet);
        assert_eq!(
            refusal.keys(),
            [KEY_READABLE_ROOTS, KEY_CLOSURE_ROOTS],
            "an empty set is about both lists, since either could have filled it"
        );
    }

    #[test]
    fn the_bound_paths_are_the_resolved_paths() {
        // `RV-346` `F-1`: bubblewrap dereferences a `--ro-bind` source, so a
        // declared root pointing elsewhere would bind elsewhere.
        let host = FixtureHost::new().with_resolution("/opt/tools", "/srv/real-tools");
        let query = FixtureQuery::default();

        assert_eq!(
            set_of(&["/opt/tools"], &[], &host, &query).expect("a resolvable entry is lawful"),
            vec![PathBuf::from("/srv/real-tools")]
        );
    }

    #[test]
    fn no_configuration_makes_a_host_wide_store_readable_whole() {
        // `EX-8` / `VA-3`, asserted over **every** branch — including the ones
        // taken when a list or the resolver is absent, which is where a
        // host-shaped default would hide if one existed.
        let host = FixtureHost::new()
            .with_resolution("/toolchain", "/nix/store/toolchain")
            .with_resolution("/nix/store/a", "/nix/store/a")
            .with_resolution(STORE, STORE);
        let expanding = FixtureQuery::answering(Some(0), "/nix/store/a\n");
        let empty = FixtureQuery::default();

        let branches = [
            // Both lists declared.
            set_of(&[STORE], &["/toolchain"], &host, &expanding),
            // Readable roots only — the resolver is never consulted.
            set_of(&[STORE], &[], &host, &empty),
            // Closure roots only.
            set_of(&[], &["/toolchain"], &host, &expanding),
            // No resolver at all: the absent-configuration branch.
            readable_paths(
                &[PathBuf::from(STORE)],
                &[],
                None,
                &host,
                &empty as &dyn ClosureQuery,
            ),
        ];

        for branch in branches {
            let bound = branch.expect("each of these branches is lawful");
            let stored: Vec<&PathBuf> = bound
                .iter()
                .filter(|path| path.as_path() == Path::new(STORE))
                .collect();
            assert_eq!(
                stored.len(),
                usize::from(bound.contains(&PathBuf::from(STORE))),
                "the store appears only when an operator declared it by name"
            );
        }

        // And nothing derives it: with the store undeclared, no branch invents
        // it from a member beneath it.
        let derived = set_of(&[], &["/toolchain"], &host, &expanding).expect("lawful");
        assert!(!derived.contains(&PathBuf::from(STORE)));
    }

    #[test]
    fn resolver_output_above_the_bound_refuses() {
        let oversized = QueryOutput {
            exit_status: Some(0),
            stdout: vec![b'/'; RESOLVER_OUTPUT_LIMIT + 1],
        };
        assert_eq!(
            closure_members(&oversized, &resolver()).expect_err("an unbounded read is the hazard"),
            ProfileRefusal::ResolverOutputTooLarge {
                bytes: RESOLVER_OUTPUT_LIMIT + 1,
                limit: RESOLVER_OUTPUT_LIMIT,
            }
        );
    }

    #[test]
    fn the_spawned_query_collects_exit_status_and_stdout_and_decides_nothing() {
        let output = SpawnedClosureQuery
            .query(&argv(&["echo"]), Path::new("/nix/store/realised"))
            .expect("`echo` is on PATH in this jail");

        assert_eq!(output.exit_status, Some(0));
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "/nix/store/realised",
            "the realised path is handed to the resolver one at a time, as its last word"
        );
    }

    #[test]
    fn the_configured_readable_set_rides_the_same_rules() {
        // The seam PHASE-06 calls, over a real parsed `[capsule]` table, so the
        // free functions above are not the only thing under test.
        let parsed = parse_capsule_config(
            r#"
            [capsule]
            root = "/var/lib/doctrine"
            readable-roots = ["/bin/sh"]
            execution-timeout-seconds = 600
            file-size-cap-mib = 64
            "#,
        )
        .expect("this table is well formed");
        let host = FixtureHost::new().with_resolution("/bin/sh", "/bin/sh");
        let config = root_capsule_config(parsed, &host).expect("the root is configured");

        assert_eq!(
            readable_set(&config, &host, &FixtureQuery::default()).expect("lawful"),
            vec![PathBuf::from("/bin/sh")]
        );
    }

    // ── T6: the derived inner PATH ─────────────────────────────────────────

    #[test]
    fn inner_path_is_derived_only_from_bound_paths() {
        let host = FixtureHost::new()
            .with_env("PATH", "/home/u/.local/bin:/usr/bin")
            .with_resolution("/home/u/.local/bin", "/home/u/.local/bin")
            .with_resolution("/usr/bin", "/usr/bin");

        assert_eq!(
            derived_inner_path(&host, &[PathBuf::from("/usr/bin")]),
            vec![PathBuf::from("/usr/bin")],
            "`/home/u/.local/bin` is beneath no bound path and is dropped"
        );
    }

    #[test]
    fn inner_path_draws_from_every_bound_path_in_host_path_order() {
        // The fixture is chosen so host order and sorted order **disagree**:
        // `/usr/bin` precedes `/opt/toolchain/bin` on the host and follows it
        // lexically. A fixture where the two coincide asserts nothing, which
        // `M10` demonstrated by redding nothing against the first one written.
        let host = FixtureHost::new()
            .with_env("PATH", "/usr/bin:/opt/toolchain/bin:/usr/bin")
            .with_resolution("/opt/toolchain/bin", "/opt/toolchain/bin")
            .with_resolution("/usr/bin", "/usr/bin");

        let derived = derived_inner_path(
            &host,
            &[PathBuf::from("/usr/bin"), PathBuf::from("/opt/toolchain")],
        );
        assert_eq!(
            derived,
            vec![
                PathBuf::from("/usr/bin"),
                PathBuf::from("/opt/toolchain/bin"),
            ],
            "host `PATH` order, deduplicated — not the order of the bound paths, \
             and not sorted"
        );
        let mut sorted = derived.clone();
        sorted.sort();
        assert_ne!(
            derived, sorted,
            "the fixture must discriminate: if host order and sorted order agree, \
             the assertion above holds under either rule"
        );
    }

    #[test]
    fn a_file_readable_root_contributes_no_inner_path_entry() {
        // The case a naive containment test gets wrong: `backend::overlaps` is
        // bidirectional and would admit `/bin` on the strength of the bound
        // *file* `/bin/sh`, handing the capsule every binary in `/bin`.
        let host = FixtureHost::new()
            .with_env("PATH", "/bin")
            .with_resolution("/bin", "/bin");

        assert_eq!(
            derived_inner_path(&host, &[PathBuf::from("/bin/sh")]),
            Vec::<PathBuf>::new()
        );
        assert!(is_within(Path::new("/bin/sh"), Path::new("/bin")));
        assert!(!is_within(Path::new("/bin"), Path::new("/bin/sh")));
    }

    #[test]
    fn a_stale_host_path_entry_is_dropped_not_refused() {
        // `D8`: the host `PATH` is the operator's environment, not this
        // project's configuration. Only *declared* entries refuse.
        let host = FixtureHost::new()
            .with_env("PATH", "/gone:/usr/bin")
            .with_resolution("/usr/bin", "/usr/bin");

        assert_eq!(
            derived_inner_path(&host, &[PathBuf::from("/usr/bin")]),
            vec![PathBuf::from("/usr/bin")]
        );
    }

    #[test]
    fn only_identity_mapped_readable_entries_contribute_to_the_inner_path() {
        let placement = placement_with(
            vec![mount("/usr/bin", "/usr/bin"), mount("/opt/kit", "/kit")],
            Vec::new(),
            NetworkPosture::Denied,
        );
        assert_eq!(
            identity_bound_paths(&placement),
            vec![PathBuf::from("/usr/bin")],
            "a host `PATH` entry beneath `/opt/kit` would name a path that does \
             not exist inside the capsule"
        );
    }

    // ── T7: argv assembly ──────────────────────────────────────────────────

    #[test]
    fn bubblewrap_argv_denies_network_unless_posture_is_permitted() {
        let denied = assembled(&lawful_placement());
        assert!(
            !denied.iter().any(|word| word == FLAG_SHARE_NET),
            "a capsule is default-denied; the worktree arm's permissive floor is \
             the inversion DEC-155 names"
        );

        let permitted = assembled(&placement_with(
            vec![mount(LAWFUL_HOST, LAWFUL_HOST)],
            Vec::new(),
            NetworkPosture::Permitted,
        ));
        assert!(permitted.iter().any(|word| word == FLAG_SHARE_NET));
    }

    #[test]
    fn bubblewrap_argv_places_share_net_only_after_unshare_all() {
        let permitted = assembled(&placement_with(
            vec![mount(LAWFUL_HOST, LAWFUL_HOST)],
            Vec::new(),
            NetworkPosture::Permitted,
        ));
        assert!(
            position(&permitted, FLAG_SHARE_NET) > position(&permitted, FLAG_UNSHARE_ALL),
            "`--share-net` before `--unshare-all` is undone by it, so the flag's \
             position decides whether it takes effect at all"
        );
    }

    /// `D5`'s trap, and the reason `ProcessVisibility` cannot be composed from
    /// the network axis independently.
    ///
    /// `--share-net` is bubblewrap's only re-share flag and it pairs with
    /// `--unshare-all`. Once the enumerated set replaces `--unshare-all`, a
    /// permitted network has to be expressed by **omitting `--unshare-net`**.
    /// Emitting both is an argv error — a control the mechanism refuses to
    /// build, which proves nothing (`S7`).
    ///
    /// The discriminating fixture is the *pair*: the permitted placement alone
    /// passes under an implementation that drops `--unshare-net`
    /// unconditionally, and the denied one alone passes under an
    /// implementation that never drops it.
    #[test]
    fn the_enumerated_unshare_set_expresses_a_permitted_network_by_omission() {
        let permitted = assembled_under(
            &placement_with(
                vec![mount(LAWFUL_HOST, LAWFUL_HOST)],
                Vec::new(),
                NetworkPosture::Permitted,
            ),
            &Weakening::ProcessVisibility,
        );
        assert!(
            !permitted.iter().any(|word| word == FLAG_SHARE_NET),
            "`--share-net` has no `--unshare-all` to re-share against here: {permitted:?}"
        );
        assert!(
            !permitted.iter().any(|word| word == FLAG_UNSHARE_NET),
            "a permitted network under the enumerated set is the omission of \
             `--unshare-net`: {permitted:?}"
        );

        let denied = assembled_under(
            &placement_with(
                vec![mount(LAWFUL_HOST, LAWFUL_HOST)],
                Vec::new(),
                NetworkPosture::Denied,
            ),
            &Weakening::ProcessVisibility,
        );
        assert!(
            denied.iter().any(|word| word == FLAG_UNSHARE_NET),
            "removing pid isolation must not also permit the network: {denied:?}"
        );
        assert!(
            !denied.iter().any(|word| word == FLAG_UNSHARE_ALL),
            "the enumerated set replaces `--unshare-all` rather than joining it: {denied:?}"
        );
    }

    #[test]
    fn argv_is_assembled_in_the_declared_order() {
        let placement = placement_with(
            vec![mount("/usr/bin", "/usr/bin")],
            Vec::new(),
            NetworkPosture::Denied,
        );
        let environment = capsule_environment(&CapsuleEnv::complete(), "/usr/bin");
        let tokens = confinement_argv(&placement, &environment, 9, &argv(&["/bin/true"]), None);

        assert_eq!(
            tokens,
            vec![
                "bwrap",
                "--json-status-fd",
                "9",
                "--unshare-all",
                "--uid",
                "1000",
                "--gid",
                "1000",
                "--proc",
                "/proc",
                "--dev",
                "/dev",
                "--tmpfs",
                "/tmp",
                "--ro-bind",
                LAWFUL_EXPORT,
                "/source",
                "--ro-bind",
                "/usr/bin",
                "/usr/bin",
                "--bind",
                "/var/lib/doctrine/tx/0001/capsule",
                "/capsule",
                "--bind",
                "/var/lib/doctrine/tx/0001/agent",
                "/agent",
                "--chdir",
                "/capsule",
                "--die-with-parent",
                "--new-session",
                "--clearenv",
                "--setenv",
                "GIT_AUTHOR_EMAIL",
                "capsule@doctrine.invalid",
                "--setenv",
                "GIT_AUTHOR_NAME",
                "Doctrine Capsule",
                "--setenv",
                "GIT_COMMITTER_EMAIL",
                "capsule@doctrine.invalid",
                "--setenv",
                "GIT_COMMITTER_NAME",
                "Doctrine Capsule",
                "--setenv",
                "HOME",
                "/agent",
                "--setenv",
                "PATH",
                "/usr/bin",
                "--setenv",
                "TERM",
                "dumb",
                "/bin/true",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<String>>()
        );
    }

    #[test]
    fn readable_and_writable_entries_are_bound_in_declared_order() {
        let tokens = assembled(&placement_with(
            vec![mount("/usr/bin", "/usr/bin"), mount("/bin/sh", "/bin/sh")],
            vec![
                mount(
                    "/var/lib/doctrine/tx/0001/first",
                    "/var/lib/doctrine/tx/0001/first",
                ),
                mount(
                    "/var/lib/doctrine/tx/0001/second",
                    "/var/lib/doctrine/tx/0001/second",
                ),
            ],
            NetworkPosture::Denied,
        ));

        assert!(position(&tokens, "/usr/bin") < position(&tokens, "/bin/sh"));
        assert!(
            position(&tokens, "/var/lib/doctrine/tx/0001/first")
                < position(&tokens, "/var/lib/doctrine/tx/0001/second")
        );
        assert!(
            position(&tokens, "/bin/sh") < position(&tokens, "/var/lib/doctrine/tx/0001/first"),
            "every read-only bind precedes every writable one"
        );
    }

    #[test]
    fn a_readable_root_resolving_under_tmp_is_bound_after_the_tmpfs() {
        // `EX-3`'s lawful case, and `F-4` is the executed control: with the
        // order reversed a real `bwrap` leaves `/tmp` empty inside the capsule
        // — the declared input vanishes, silently, exit 0.
        let tokens = assembled(&placement_with(
            vec![mount("/tmp/undertmp", "/tmp/undertmp")],
            Vec::new(),
            NetworkPosture::Denied,
        ));

        assert!(position(&tokens, INNER_TMP) < position(&tokens, "/tmp/undertmp"));
    }

    #[test]
    fn the_capsule_environment_is_set_in_sorted_order() {
        let environment = capsule_environment(&CapsuleEnv::complete(), "/usr/bin");
        let names: Vec<&str> = environment.iter().map(|(name, _)| *name).collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();

        assert_eq!(names, sorted);
        assert_eq!(names.len(), CapsuleEnvVar::ALL.len());
        assert!(environment.contains(&(HOST_PATH_VARIABLE, "/usr/bin".to_owned())));
    }

    // ── T8: streams, termination, disk ─────────────────────────────────────

    #[test]
    fn termination_maps_each_measured_wait_status_to_its_variant() {
        let cases = [
            ((Some(124), None, true), Termination::TimedOut),
            ((Some(153), None, true), Termination::FileSizeExceeded),
            (
                (Some(137), None, true),
                Termination::Signalled { signal: 9 },
            ),
            ((Some(0), None, true), Termination::Exited { code: 0 }),
            ((Some(1), None, true), Termination::Exited { code: 1 }),
            // bwrap exits 1 when `execvp` fails, which is indistinguishable
            // from a capsule that exits 1 — the JSON status is the only lawful
            // discriminator (`D11`), and it is a parent-side observation.
            ((Some(1), None, false), Termination::NotExecutable),
            ((None, Some(9), true), Termination::Signalled { signal: 9 }),
        ];

        for ((code, signal, child_ran), expected) in cases {
            assert_eq!(
                classify_termination(ExitReport { code, signal }, child_ran),
                expected,
                "code {code:?}, signal {signal:?}, child_ran {child_ran}"
            );
        }
    }

    #[test]
    fn the_standard_streams_are_replaced_not_marked_close_on_exec() {
        assert_eq!(
            standard_stream_endpoints(CapsuleStdio::EmptyInputCapturedOutput),
            [
                StreamEndpoint::EmptySource,
                StreamEndpoint::CapturedOutput,
                StreamEndpoint::CapturedOutput,
            ]
        );
        assert_eq!(
            SWEEP_FLOOR, 3,
            "marking 0/1/2 close-on-exec would leave a capsule unable to report; \
             two mechanisms, two invariants (RV-346 F-30)"
        );
    }

    #[test]
    fn the_wall_bound_wraps_the_exec_from_outside() {
        let confinement = assembled(&lawful_placement());
        let bounded = wall_bounded_argv(
            Duration::from_secs(600),
            Duration::from_secs(7),
            &confinement,
        );

        assert_eq!(
            bounded
                .iter()
                .take(4)
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["timeout", "-k", "7", "600"],
        );
        assert_eq!(bounded.get(4).map(String::as_str), Some(BWRAP_EXECUTABLE));
        assert!(
            bounded.ends_with(&confinement),
            "the confinement argv is wrapped, never rewritten"
        );

        let host = FixtureHost::new();
        assert_eq!(
            BubblewrapBackend::new(&host)
                .with_kill_grace(Duration::from_secs(7))
                .kill_grace,
            Duration::from_secs(7),
            "the configured grace has a route in, since `Execution` carries none"
        );
    }

    #[test]
    fn disk_used_counts_every_file_beneath_the_transaction_root() {
        let root =
            std::env::temp_dir().join(format!("doctrine-capsule-disk-{}", std::process::id()));
        let nested = root.join("capsule").join("out");
        std::fs::create_dir_all(&nested).expect("the fixture tree is creatable");
        for (leaf, bytes) in [("a", 10_usize), ("b", 32)] {
            let mut file = std::fs::File::create(nested.join(leaf)).expect("creatable");
            file.write_all(&vec![b'x'; bytes]).expect("writable");
        }

        let measured = bytes_beneath(&root).expect("the tree is walkable");
        std::fs::remove_dir_all(&root).expect("the fixture tree is removable");

        assert_eq!(
            measured,
            ByteCount::from_bytes(42),
            "a per-file cap does not catch many small files, so the whole-tree \
             figure is computed trusted-side after the run"
        );
    }

    // ── T9: the descriptor sweep ───────────────────────────────────────────

    #[test]
    fn the_sweep_starts_above_the_standard_streams() {
        // `RV-346` `F-30` as a rule rather than a fix: a sweep is described by
        // its bound as much as by its action, and an unstated bound reads as
        // *everything*.
        assert_eq!(
            inherited_descriptors(&[0, 1, 2, 3, 5], 9),
            vec![3, 5],
            "0, 1 and 2 are the three a shell hands a process by default, and \
             they are replaced rather than swept"
        );
    }

    #[test]
    fn the_sweep_skips_its_own_directory_handle() {
        assert_eq!(
            inherited_descriptors(&[3, 4, 5], 4),
            vec![3, 5],
            "`/proc/self/fd`'s own handle appears in its own listing (D14)"
        );
    }

    /// Note for the next reader: this marks `CLOEXEC` on **every** descriptor
    /// above 2 in the test binary, including the harness's. That is benign —
    /// `CLOEXEC` takes effect only at `exec`, and nothing in this crate's suite
    /// passes a descriptor to a child expecting it to survive. It stops being
    /// benign the moment one does.
    #[test]
    fn every_descriptor_above_two_is_marked_close_on_exec_before_the_exec() {
        // A `std::fs::File` is opened `O_CLOEXEC` and would pass against a
        // backend that swept nothing. `dup(2)` does not set the flag, so this
        // descriptor discriminates (`mem.pattern.tests.guard-needs-a-discriminating-difference`).
        let owned = File::open("/proc/self/fd").expect("this process can read its own descriptors");
        let inherited = rustix::io::dup(&owned).expect("dup is available");

        let before = fcntl_getfd(&inherited).expect("the descriptor is live");
        assert!(
            !before.contains(FdFlags::CLOEXEC),
            "precondition: a dup'd descriptor must start without the flag, or this \
             test would pass against a sweep that did nothing"
        );

        let marked = mark_inherited_descriptors_close_on_exec().expect("the sweep runs");
        assert!(marked > 0, "there is at least this test's own descriptor");
        assert!(
            fcntl_getfd(&inherited)
                .expect("still live")
                .contains(FdFlags::CLOEXEC)
        );
    }

    // ── T10: the per-file size cap, and the unsafe budget ──────────────────

    #[test]
    fn the_file_size_cap_is_applied_to_the_child_before_the_exec() {
        // Executed, not reasoned: the cap is set in `pre_exec`, so the only
        // honest evidence is a child that hits it. `SIGXFSZ` is what
        // `RLIMIT_FSIZE` raises on the offending write.
        let target =
            std::env::temp_dir().join(format!("doctrine-capsule-fsize-{}", std::process::id()));
        let mut command = Command::new("dd");
        command
            .arg("if=/dev/zero")
            .arg(format!("of={}", target.display()))
            .arg("bs=4096")
            .arg("count=16")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        apply_file_size_cap(&mut command, ByteCount::from_bytes(1024));

        let status = command.status().expect("`dd` is on PATH in this jail");
        let _ = std::fs::remove_file(&target);

        assert_eq!(
            status.signal(),
            Some(SIGXFSZ),
            "the child must be killed by SIGXFSZ, which is the cap taking effect \
             inside the child rather than on the parent"
        );
        assert_eq!(
            classify_termination(
                ExitReport {
                    code: Some(SIGNALLED_EXIT_BASE + SIGXFSZ),
                    signal: None,
                },
                true,
            ),
            Termination::FileSizeExceeded,
            "and `timeout` reports it as wait status 153"
        );
    }

    /// The `VA`-shaped half of `F-1/R` step 4, mechanised.
    ///
    /// `deny` in place of `forbid` is only acceptable while the ceiling is
    /// visible: two is the budget, not a starting point.
    ///
    /// Both needles are composed at run time, so this test's own source does
    /// not count itself; and the source is whitespace-normalised first, so an
    /// attribute `rustfmt` has broken across lines is counted the same as one
    /// on a single line. A count that depended on formatting would under-count
    /// a third site, which is the one direction this check must not fail in.
    #[test]
    fn the_unsafe_budget_is_exactly_two_sites() {
        let source = include_str!("bubblewrap.rs");
        let normalised = source
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
            .replace("( ", "(");
        let expected = format!("#[{}(", "expect") + "unsafe_code";
        let forbidden = format!("#[{}(", "allow") + "unsafe_code";
        let block = format!("{} {{", "unsafe");

        assert_eq!(
            normalised.matches(expected.as_str()).count(),
            2,
            "exactly two sites: EX-15's pre_exec and EX-16's BorrowedFd::borrow_raw. \
             A third is a finding and a stop, not a judgement call (F-1/R)"
        );
        assert_eq!(
            normalised.matches(block.as_str()).count(),
            2,
            "and each exception guards exactly one unsafe block — an exception \
             covering a widened block would keep the count at two while widening \
             the budget"
        );
        assert!(
            !normalised.contains(forbidden.as_str()),
            "`allow_attributes` is denied; `expect` only, so an unused exception \
             is itself a lint"
        );
    }
}
