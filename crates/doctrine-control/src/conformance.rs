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

use std::path::Path;

use crate::backend::{
    Availability, BackendError, BackendId, CapsuleBackend, CapsulePlacement, Execution,
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
// The delta and row vocabulary (`EX-8`, `EX-14`)
// ---------------------------------------------------------------------------

/// The suite's self-contained control plane.
///
/// Forward-declared here because [`Delta::Widened`] names it. **PHASE-08 `EX-1`
/// owns it** and fills it with the run's root, its own git repository, the
/// synthesized `[capsule]` table, the scopes and every row's decoy. Nothing at
/// this phase constructs one, and nothing at this phase should give it fields, a
/// constructor or a `Drop` — that would be PHASE-08's work done early, in the
/// file PHASE-08 will land it in.
#[derive(Debug)]
pub(crate) struct Fixture;

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
    use std::path::PathBuf;
    use std::time::Duration;

    use super::{
        Admission, AdmissionVerdict, ArmResult, ArmShape, AuthorityGrant, AuxOutcome, Axis, Bound,
        Claim, ConcurrentWitness, ConformanceBackend, Delta, Fixture, HostPid, Indeterminacy,
        LIVENESS_MARKER, NotAdmitted, Observed, PidProbe, Probe, PropertyRemoval, Row, RowId,
        RowVerdict, SHELL, Which, admission, classify, classify_concurrent,
        row_ids_in_more_than_one_table, row_verdict, verify, verify_over,
    };
    use crate::backend::fixture::{WITNESS_ID, WitnessBackend, exited};
    use crate::backend::{
        AcceptedBase, Availability, BackendError, BackendId, CapsuleBackend, CapsuleEnv,
        CapsulePlacement, CapsuleStdio, Execution, ForbiddenScopes, InnerPath, MountedPath,
        NetworkPosture, Observation, PlacementParts, SourceExport, Termination, TransactionRoot,
    };
    use crate::config::{Argv, ByteCount};
    use crate::host::HostFacts;
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
}
