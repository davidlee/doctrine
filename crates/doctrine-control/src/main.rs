// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine-control` — the capsule control binary (SL-248, `DEC-153`).
//!
//! PHASE-06 lands the first verb, `provision` (`EX-17`); PHASE-10 `T10` lands
//! the second, `backend verify`. There is deliberately **no** `transaction
//! show`: nothing operates a transaction yet, and PHASE-06's tests inspect the
//! returned value directly rather than a rendering of it.
//!
//! **Bin-only, permanently** (`sec-6`). A `tests/` file cannot link a bin-only
//! package (`E0433`), and adding a lib target to rescue one would force the
//! conformance suite's weakening vocabulary public (`E0603`). Every test in this
//! crate is a `#[cfg(test)]` module inside the unit it tests.
//!
//! Layering (`ADR-001`): `main` is `command` — it may reach anything, and
//! nothing reaches it.

mod backend;
mod capacity;
mod config;
mod conformance;
mod host;
mod provision;
mod transaction;

use std::fmt::Write as _;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use backend::bubblewrap::BubblewrapBackend;
use backend::{AcceptedBase, NetworkPosture};
use conformance::{Admission, AdmissionVerdict, NotAdmitted, RowVerdict, verify};
use host::SystemHost;
use provision::{ProvisionRefusal, ProvisionRequest, host_capsule_config, provision};
use transaction::{PhaseIdentity, TransactionId, TransactionIdRefusal};

const VERB_PROVISION: &str = "provision";
/// `backend` is a noun with verbs under it, not a verb — `DEC-160`'s second
/// entry point is spelled `backend verify`, and the noun is what makes room for
/// a second mechanism's verbs without renaming this one.
const NOUN_BACKEND: &str = "backend";
const VERB_VERIFY: &str = "verify";

const FLAG_REPOSITORY: &str = "--repository";
const FLAG_BASE: &str = "--base";
const FLAG_SLICE: &str = "--slice";
const FLAG_PHASE: &str = "--phase";
const FLAG_REFINEMENT: &str = "--refinement";
const FLAG_NETWORK: &str = "--network";

/// Both entry points in one constant (`STD-001`): the two verbs are one surface,
/// and a second constant is how a usage line starts omitting the newer one.
const USAGE: &str = "usage:\n  \
doctrine-control provision --repository <path> --base <oid> --slice <SL-NNN> \
--phase <N> [--refinement <path>] [--network]\n  \
doctrine-control backend verify";

/// The byte this process exits with when a verb succeeded.
const EXIT_ADMITTED: u8 = 0;
/// The byte this process exits with when a verb refused — including
/// `backend verify` on a backend that was not admitted (`EX-13`).
const EXIT_REFUSED: u8 = 1;

fn main() -> ExitCode {
    // `ExitCode` rather than `std::process::exit`: the latter skips every
    // destructor, and this binary's whole subject matter is directories a
    // failing path must clean up.
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let outcome = run(&arguments);
    let status = exit_status(&outcome);
    // Reported on both arms and identically — the message *is* the structured
    // output `EX-13` requires, and it must not thin out on the failing side.
    report(&outcome.unwrap_or_else(|refusal| refusal));
    ExitCode::from(status)
}

/// The exit byte, as a number rather than an [`ExitCode`].
///
/// `ExitCode` is opaque — no `PartialEq`, no accessor — so a test written
/// against it can only assert the `Result`'s discriminant and *claim* the
/// mapping to a status. `EX-13` requires the exit code itself to be evidence, so
/// this returns the byte `main` exits with and a test compares the number.
fn exit_status(outcome: &Result<String, String>) -> u8 {
    if outcome.is_ok() {
        EXIT_ADMITTED
    } else {
        EXIT_REFUSED
    }
}

/// One locked handle, written through [`std::io::Write`].
///
/// `clippy::print_stdout` / `print_stderr` are `deny` and target the macros; the
/// ban's reason string asks for exactly this shape. A failed write is not
/// reported — there is nowhere left to report it to.
fn report(message: &str) {
    let mut stderr = std::io::stderr().lock();
    let _written = writeln!(stderr, "{message}");
}

fn run(arguments: &[String]) -> Result<String, String> {
    let rest = arguments.get(1..).unwrap_or_default();
    match arguments.first().map(String::as_str) {
        Some(VERB_PROVISION) => run_provision(rest),
        Some(NOUN_BACKEND) => run_backend(rest),
        Some(other) => Err(format!("unknown verb {other:?}\n{USAGE}")),
        None => Err(USAGE.to_owned()),
    }
}

/// The verbs under the `backend` noun. One today.
fn run_backend(arguments: &[String]) -> Result<String, String> {
    let rest = arguments.get(1..).unwrap_or_default();
    match arguments.first().map(String::as_str) {
        Some(VERB_VERIFY) => run_backend_verify(rest),
        Some(other) => Err(format!("unknown backend verb {other:?}\n{USAGE}")),
        None => Err(format!("{NOUN_BACKEND} takes a verb\n{USAGE}")),
    }
}

/// `backend verify` — the on-host admission verdict (`EX-13`, `DEC-160`).
///
/// **No options, and that is `EX-3` rather than an unfinished parser.** The
/// suite synthesizes its own `CapsuleConfig` over its own fixture root, so the
/// only host facts admission depends on are the backend's availability and a
/// working shell. Taking `--repository` here would offer the operator's
/// `[capsule]` table as an input to a verdict about the *backend*, which is the
/// confusion `verify`'s three-parameter signature exists to prevent.
fn run_backend_verify(arguments: &[String]) -> Result<String, String> {
    if let Some(unexpected) = arguments.first() {
        return Err(format!("unknown option {unexpected:?}\n{USAGE}"));
    }
    let host = SystemHost;
    // No `with_kill_grace`: that bound comes from the operator's `[capsule]`
    // table, which this verb deliberately does not read (see above). The
    // fixture's own bounds are what the suite's payloads run under.
    let backend = BubblewrapBackend::new(&host);
    // The wall-clock read lives **here**, in the shell. `verify` takes `today`
    // as a parameter and owns no clock, which is what makes a verdict
    // reproducible from its own record — the pure/imperative split, and the
    // same reason `provision` is handed a minted `TransactionId`.
    admit(&verify(&backend, &host, doctrine::today()))
}

/// The verdict, rendered once and routed to the exit status by its outcome
/// **alone**.
///
/// One rendering on both arms: a not-admitted verdict that reported less than an
/// admitted one would make the failing case the harder one to diagnose, which is
/// backwards. `Err` is the whole of `EX-13`'s *exiting nonzero*, via
/// [`exit_status`].
fn admit(verdict: &AdmissionVerdict) -> Result<String, String> {
    let rendered = render_verdict(verdict);
    match verdict.outcome {
        Admission::Admitted => Ok(rendered),
        Admission::NotAdmitted { .. } => Err(rendered),
    }
}

/// The verdict as structured lines — one fact per line, `key=value` where the
/// value is a fact and `{:?}` where it is a variant.
///
/// **`{:?}` rather than a match arm per variant, deliberately** (`EX-15`,
/// `sec-9` `R9`): a hand-written name table over [`Property`] would be a second
/// unchecked enumeration of table A, and the phase adds none. `Debug` is derived
/// from the enum itself, so a variant added without a name here is impossible
/// rather than merely unlikely.
///
/// Every row is rendered, including the proven ones, because
/// [`AdmissionVerdict::rows`] is the whole run and a reader diagnosing a refusal
/// needs to see what *did* hold. The unproven ones are additionally named on the
/// outcome line so the refusal states its own cause without a scan.
/// Lines assembled and joined rather than written into one buffer, because
/// `clippy::use_debug` is `deny` and fires on `{:?}` inside the `write!` family
/// while permitting it inside `format!` — the lint is about debugging remnants
/// reaching an output handle, and `render_refusal` below is the same shape.
fn render_verdict(verdict: &AdmissionVerdict) -> String {
    let mut lines = vec![
        format!(
            "backend={} os={} kernel={} arch={} date={}",
            verdict.backend.as_str(),
            verdict.host.os,
            verdict.host.kernel,
            verdict.host.arch,
            verdict.date,
        ),
        format!("outcome={}", render_outcome(verdict)),
    ];
    lines.extend(
        verdict
            .rows
            .iter()
            .map(|(id, row)| format!("row {id:?}={row:?}")),
    );
    lines.extend(
        verdict
            .auxiliary
            .iter()
            .map(|(claim, outcome)| format!("claim {}/{}={outcome:?}", claim.section, claim.name)),
    );
    lines.extend(verdict.observations.iter().map(|(unrowed, reading)| {
        format!(
            "observation {}/{}={reading:?}",
            unrowed.section, unrowed.name
        )
    }));
    lines.join("\n")
}

/// The outcome line, which for a row refusal **names the rows that were not
/// proven**.
///
/// A refusal reading only `not-admitted reason=rows` would be true and useless:
/// the operator's next question is always *which row*, and answering it here is
/// what makes the nonzero exit actionable rather than merely correct.
fn render_outcome(verdict: &AdmissionVerdict) -> String {
    match verdict.outcome {
        Admission::Admitted => "admitted".to_owned(),
        Admission::NotAdmitted {
            reason:
                NotAdmitted::Unavailable {
                    ref missing,
                    ref remedy,
                },
        } => {
            // `POL-002` facet 3: what was absent, and what would satisfy it.
            format!("not-admitted reason=unavailable missing={missing} remedy={remedy}")
        }
        Admission::NotAdmitted {
            reason: NotAdmitted::Rows,
        } => {
            let unproven: Vec<String> = verdict
                .rows
                .iter()
                .filter(|(_, row)| !matches!(*row, RowVerdict::Proven))
                .map(|(id, row)| format!("{id:?}={row:?}"))
                .collect();
            format!("not-admitted reason=rows unproven=[{}]", unproven.join(" "))
        }
    }
}

/// The parsed command line, before any of it is validated.
#[derive(Debug, Default)]
struct Options {
    repository: Option<PathBuf>,
    base: Option<String>,
    slice: Option<String>,
    phase: Option<String>,
    refinement: Option<PathBuf>,
    network: bool,
}

/// `--flag value` pairs and one bare switch.
///
/// Hand-rolled rather than `clap`: `C8` forbids a new compiled dependency in
/// this phase, and this verb's whole surface is six options. A parser generator
/// arrives when there is a second verb to justify it.
fn parse_options(arguments: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut rest = arguments.iter();
    while let Some(argument) = rest.next() {
        let mut value = || {
            rest.next()
                .cloned()
                .ok_or_else(|| format!("{argument} takes a value\n{USAGE}"))
        };
        match argument.as_str() {
            FLAG_REPOSITORY => options.repository = Some(PathBuf::from(value()?)),
            FLAG_BASE => options.base = Some(value()?),
            FLAG_SLICE => options.slice = Some(value()?),
            FLAG_PHASE => options.phase = Some(value()?),
            FLAG_REFINEMENT => options.refinement = Some(PathBuf::from(value()?)),
            FLAG_NETWORK => options.network = true,
            other => return Err(format!("unknown option {other:?}\n{USAGE}")),
        }
    }
    Ok(options)
}

fn run_provision(arguments: &[String]) -> Result<String, String> {
    let options = parse_options(arguments)?;
    let required = |name: &str| format!("{name} is required\n{USAGE}");

    let repository_root = options
        .repository
        .ok_or_else(|| required(FLAG_REPOSITORY))?;
    let base = AcceptedBase::new(options.base.ok_or_else(|| required(FLAG_BASE))?);
    let slice = options.slice.ok_or_else(|| required(FLAG_SLICE))?;
    let phase: u32 = options
        .phase
        .ok_or_else(|| required(FLAG_PHASE))?
        .parse()
        .map_err(|_ignored| format!("{FLAG_PHASE} takes a number\n{USAGE}"))?;

    let request = ProvisionRequest {
        repository_root,
        base,
        phase: PhaseIdentity { slice, phase },
        // `EX-7`: minted **in the shell**, so no clock and no entropy source
        // reaches `provision` itself and a test can hand the same id twice.
        id: mint_transaction_id().map_err(|refusal| format!("{refusal:?}"))?,
        refinement: options.refinement,
        network: if options.network {
            NetworkPosture::Permitted
        } else {
            NetworkPosture::Denied
        },
    };

    let host = SystemHost;
    // `D3`: the configured kill grace is threaded here, at the verb, and not
    // moved onto `Execution` — which would change a type `EX-10` enumerates for
    // no observable difference. `notes.md` item 35 stays owed.
    let config = host_capsule_config(&request.repository_root, &host)
        .map_err(|refusal| render_refusal(&refusal))?;
    let backend = BubblewrapBackend::new(&host).with_kill_grace(config.bounds().kill_grace());

    let transaction =
        provision(&request, &host, &backend).map_err(|refusal| render_refusal(&refusal))?;
    // The backend is named because an observation that cannot be attributed to a
    // mechanism cannot be attributed to an admission verdict either.
    Ok(format!(
        "provisioned id={} backend={} root={}",
        transaction.id.as_str(),
        transaction.backend.as_str(),
        transaction.root().path().display()
    ))
}

/// A collision-resistant identity from the shell's two cheapest distinguishing
/// facts: the wall clock in nanoseconds and this process's id.
///
/// Neither alone is enough — two processes can read the same nanosecond, and one
/// pid is reused — and the pair is not a guarantee either, which is exactly why
/// step 9's exclusive create is what establishes ownership (`EX-13`) rather than
/// the id's entropy. No `uuid`: `C8` forbids a new compiled dependency, and
/// nothing here needs a global identifier.
fn mint_transaction_id() -> Result<TransactionId, TransactionIdRefusal> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    TransactionId::try_new(format!("tx-{nanos:x}-{:x}", std::process::id()))
}

/// The refusal, with the paths and the `[capsule]` keys an operator must edit.
///
/// This is `ProvisionRefusal::paths` / `keys` — and, transitively,
/// `ConfigRefusal::keys`, `PlacementRefusal::paths` and
/// `ProfileRefusal::paths`/`keys` — reaching their first real consumer. Those
/// accessors exist so a refusal can name what to fix without an operator
/// parsing a sentence, and this is where that becomes true.
fn render_refusal(refusal: &ProvisionRefusal) -> String {
    let mut rendered = format!("provision refused: {refusal:?}");
    for path in refusal.paths() {
        let _written = write!(rendered, "\n  path: {}", path.display());
    }
    for key in refusal.keys() {
        let _written = write!(rendered, "\n  key:  capsule.{key}");
    }
    rendered
}

#[cfg(test)]
mod tests {
    use super::{
        Admission, AdmissionVerdict, EXIT_ADMITTED, EXIT_REFUSED, NOUN_BACKEND, NotAdmitted,
        RowVerdict, USAGE, VERB_VERIFY, admit, exit_status, run,
    };
    use crate::backend::BackendId;
    use crate::conformance::{
        AuxOutcome, Axis, Claim, HostDescriptor, Indeterminacy, Property, Reading, RowId, Unrowed,
        Which,
    };

    /// `run` over borrowed literals, since every caller here is one.
    fn run_words(words: &[&str]) -> Result<String, String> {
        let arguments: Vec<String> = words.iter().map(|word| (*word).to_owned()).collect();
        run(&arguments)
    }

    const A_DATE: &str = "2026-08-09";
    /// The header every expected rendering below opens with, so the assertions
    /// state the *outcome* they are about and not the fixture's own constants
    /// four times over.
    const A_HEADER: &str = "backend=bubblewrap os=linux kernel=6.1.0 arch=x86_64 date=2026-08-09";

    /// A verdict with a fixed header, over the given outcome and rows.
    ///
    /// The outcome is supplied rather than computed: `conformance::admission` is
    /// private to that module and is where the rows-to-outcome rule is tested.
    /// What is under test here is the rendering and the exit status of a verdict
    /// *as given*, which is the whole of what the shell contributes.
    fn a_verdict(outcome: Admission, rows: Vec<(RowId, RowVerdict)>) -> AdmissionVerdict {
        AdmissionVerdict {
            backend: BackendId::new("bubblewrap"),
            host: HostDescriptor {
                os: "linux".to_owned(),
                kernel: "6.1.0".to_owned(),
                arch: "x86_64".to_owned(),
            },
            date: A_DATE.to_owned(),
            outcome,
            rows,
            auxiliary: Vec::new(),
            observations: Vec::new(),
        }
    }

    fn not_admitted_on_rows() -> Admission {
        Admission::NotAdmitted {
            reason: NotAdmitted::Rows,
        }
    }

    /// The refusal `EX-13` describes, asserted on **both** halves at once: the
    /// exit byte and the whole structured output.
    ///
    /// Asserting only "exits nonzero" would pass against a binary that failed to
    /// build a verdict at all, panicked, or refused for an unrelated reason.
    /// Asserting only that the failing row's name appears would pass against a
    /// renderer that names every row on the outcome line (`F-33`: a claim
    /// asserting only that its own line appeared was a live false green here).
    /// So: the byte, and the output compared **whole**.
    ///
    /// The three non-`Proven` verdicts are all present, because the filter under
    /// test is *not `Proven`* and a filter written as `== Violated` would pass a
    /// test carrying only violations.
    #[test]
    fn an_unproven_row_is_named_in_the_structured_output_and_exits_nonzero() {
        let outcome = admit(&a_verdict(
            not_admitted_on_rows(),
            vec![
                (
                    RowId::Property(Property::FreshMutableState),
                    RowVerdict::Proven,
                ),
                (
                    RowId::Property(Property::ProcessTreeTeardown),
                    RowVerdict::Unproven,
                ),
                (
                    RowId::Property(Property::ClosedEnvironment),
                    RowVerdict::Violated,
                ),
                (
                    RowId::Axis(Axis::Checkout),
                    RowVerdict::Indeterminate {
                        arm: Which::Probe,
                        detail: Indeterminacy::NoLiveness,
                    },
                ),
                (RowId::Axis(Axis::Process), RowVerdict::Proven),
            ],
        ));

        assert_eq!(
            exit_status(&outcome),
            EXIT_REFUSED,
            "a backend that is not admitted must exit nonzero"
        );
        assert_eq!(
            outcome,
            Err([
                A_HEADER,
                "outcome=not-admitted reason=rows unproven=[\
                 Property(ProcessTreeTeardown)=Unproven \
                 Property(ClosedEnvironment)=Violated \
                 Axis(Checkout)=Indeterminate { arm: Probe, detail: NoLiveness }]",
                "row Property(FreshMutableState)=Proven",
                "row Property(ProcessTreeTeardown)=Unproven",
                "row Property(ClosedEnvironment)=Violated",
                "row Axis(Checkout)=Indeterminate { arm: Probe, detail: NoLiveness }",
                "row Axis(Process)=Proven",
            ]
            .join("\n"))
        );
    }

    /// The naming above is of the row that *was* unproven, not of a fixed one.
    ///
    /// This is the step that proves the previous assertion can fail. Move the
    /// single failure to a different row of the same table and the outcome line
    /// must move with it — a renderer naming a hard-coded row, the first row, or
    /// every row satisfies exactly one of these two and not both.
    #[test]
    fn the_outcome_line_names_whichever_row_failed_rather_than_a_fixed_one() {
        let rendering_when_row_fails = |failing: Property| {
            let rows = [Property::ProcessTreeTeardown, Property::ClosedEnvironment]
                .into_iter()
                .map(|property| {
                    let verdict = if property == failing {
                        RowVerdict::Unproven
                    } else {
                        RowVerdict::Proven
                    };
                    (RowId::Property(property), verdict)
                })
                .collect();
            admit(&a_verdict(not_admitted_on_rows(), rows))
        };

        assert_eq!(
            rendering_when_row_fails(Property::ProcessTreeTeardown),
            Err([
                A_HEADER,
                "outcome=not-admitted reason=rows \
                 unproven=[Property(ProcessTreeTeardown)=Unproven]",
                "row Property(ProcessTreeTeardown)=Unproven",
                "row Property(ClosedEnvironment)=Proven",
            ]
            .join("\n"))
        );
        assert_eq!(
            rendering_when_row_fails(Property::ClosedEnvironment),
            Err([
                A_HEADER,
                "outcome=not-admitted reason=rows \
                 unproven=[Property(ClosedEnvironment)=Unproven]",
                "row Property(ProcessTreeTeardown)=Proven",
                "row Property(ClosedEnvironment)=Unproven",
            ]
            .join("\n"))
        );
    }

    /// `POL-002` facet 3's descriptive-absence path: absent bubblewrap, the
    /// verdict names what was missing and what would satisfy it, and still
    /// exits nonzero.
    ///
    /// The alternative this convicts is a suite that skips green on a host
    /// without the backend — a skip that reads as success is the same lie in a
    /// different costume. `rows` is empty here because nothing ran, and that is
    /// visible in the output rather than inferred: an empty row list renders as
    /// no `row` lines at all.
    #[test]
    fn an_unavailable_backend_names_what_is_missing_and_what_would_satisfy_it() {
        let outcome = admit(&a_verdict(
            Admission::NotAdmitted {
                reason: NotAdmitted::Unavailable {
                    missing: "bwrap".to_owned(),
                    remedy: "install bubblewrap".to_owned(),
                },
            },
            Vec::new(),
        ));

        assert_eq!(exit_status(&outcome), EXIT_REFUSED);
        assert_eq!(
            outcome,
            Err([
                A_HEADER,
                "outcome=not-admitted reason=unavailable missing=bwrap \
                 remedy=install bubblewrap",
            ]
            .join("\n"))
        );
    }

    /// The one green path, and the auxiliary material that rides it.
    ///
    /// Table C claims and the two unrowed credential observations are
    /// **reported** and admitted on in neither direction (`EX-12`, `sec-9`
    /// `R8`), so they appear here on the admitted rendering: a failing claim
    /// beside `outcome=admitted` is the shape a reader has to be able to see.
    #[test]
    fn an_admitted_verdict_reports_every_row_and_exits_zero() {
        let mut verdict = a_verdict(
            Admission::Admitted,
            vec![(
                RowId::Property(Property::FreshMutableState),
                RowVerdict::Proven,
            )],
        );
        verdict.auxiliary = vec![(
            Claim {
                section: "sec-7",
                name: "capsule-env",
            },
            AuxOutcome::Failed("read nothing".to_owned()),
        )];
        verdict.observations = vec![(
            Unrowed {
                section: "sec-9",
                name: "no-new-privs",
            },
            Reading::Unread("no fixture".to_owned()),
        )];

        let outcome = admit(&verdict);

        assert_eq!(exit_status(&outcome), EXIT_ADMITTED);
        assert_eq!(
            outcome,
            Ok([
                A_HEADER,
                "outcome=admitted",
                "row Property(FreshMutableState)=Proven",
                "claim sec-7/capsule-env=Failed(\"read nothing\")",
                "observation sec-9/no-new-privs=Unread(\"no fixture\")",
            ]
            .join("\n"))
        );
    }

    /// The dispatch, asserted **without running the suite**.
    ///
    /// Every case here refuses before `backend verify` reaches a backend, which
    /// is what keeps a dispatch assertion off the ~50-capsule critical path the
    /// conformance suite already pays for once. What it establishes is exactly
    /// the reachability: `backend` is no longer an unknown verb, `verify` is
    /// reached under it, and `verify` alone still is not a verb.
    ///
    /// Compared **whole**, per `F-33`: an assertion that its own token appears
    /// somewhere passes against a message that also says something else.
    #[test]
    fn the_backend_verify_verb_is_dispatched_rather_than_reported_unknown() {
        assert_eq!(
            run_words(&[NOUN_BACKEND]),
            Err(format!("{NOUN_BACKEND} takes a verb\n{USAGE}")),
            "`backend` alone must ask for its verb, not report itself unknown"
        );
        assert_eq!(
            run_words(&[NOUN_BACKEND, VERB_VERIFY, "--nope"]),
            Err(format!("unknown option {:?}\n{USAGE}", "--nope")),
            "an option refusal here proves `verify` was reached under `backend`"
        );
        assert_eq!(
            run_words(&[NOUN_BACKEND, "inspect"]),
            Err(format!("unknown backend verb {:?}\n{USAGE}", "inspect")),
            "a second noun verb is unknown, and says which noun it was unknown to"
        );
        // `verify` is a verb of `backend`, not of the binary — so the bare form
        // stays unknown rather than quietly aliasing the expensive one.
        assert_eq!(
            run_words(&[VERB_VERIFY]),
            Err(format!("unknown verb {:?}\n{USAGE}", VERB_VERIFY)),
        );
    }
}
