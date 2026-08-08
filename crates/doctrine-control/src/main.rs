// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine-control` — the capsule control binary (SL-248, `DEC-153`).
//!
//! PHASE-06 lands the first verb, `provision` (`EX-17`). `backend verify`
//! follows in PHASE-10. There is deliberately **no** `transaction show`: nothing
//! operates a transaction yet, and this phase's tests inspect the returned value
//! directly rather than a rendering of it.
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
mod host;
mod provision;
mod transaction;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use backend::bubblewrap::BubblewrapBackend;
use backend::{AcceptedBase, NetworkPosture};
use host::SystemHost;
use provision::{ProvisionRefusal, ProvisionRequest, host_capsule_config, provision};
use transaction::{PhaseIdentity, TransactionId, TransactionIdRefusal};

const VERB_PROVISION: &str = "provision";

const FLAG_REPOSITORY: &str = "--repository";
const FLAG_BASE: &str = "--base";
const FLAG_SLICE: &str = "--slice";
const FLAG_PHASE: &str = "--phase";
const FLAG_REFINEMENT: &str = "--refinement";
const FLAG_NETWORK: &str = "--network";

const USAGE: &str = "usage: doctrine-control provision \
--repository <path> --base <oid> --slice <SL-NNN> --phase <N> \
[--refinement <path>] [--network]";

fn main() -> ExitCode {
    // `ExitCode` rather than `std::process::exit`: the latter skips every
    // destructor, and this binary's whole subject matter is directories a
    // failing path must clean up.
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match run(&arguments) {
        Ok(message) => {
            report(&message);
            ExitCode::SUCCESS
        }
        Err(message) => {
            report(&message);
            ExitCode::FAILURE
        }
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
    match arguments.first().map(String::as_str) {
        Some(VERB_PROVISION) => run_provision(&arguments[1..]),
        Some(other) => Err(format!("unknown verb {other:?}\n{USAGE}")),
        None => Err(USAGE.to_owned()),
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

    let repository_root = options.repository.ok_or_else(|| required(FLAG_REPOSITORY))?;
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
    let config = host_capsule_config(&request.repository_root, &host).map_err(render_refusal)?;
    let backend = BubblewrapBackend::new(&host).with_kill_grace(config.bounds().kill_grace());

    let transaction = provision(&request, &host, &backend).map_err(render_refusal)?;
    Ok(format!(
        "provisioned id={} root={}",
        transaction.id.as_str(),
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
fn render_refusal(refusal: ProvisionRefusal) -> String {
    let mut rendered = format!("provision refused: {refusal:?}");
    for path in refusal.paths() {
        rendered.push_str(&format!("\n  path: {}", path.display()));
    }
    for key in refusal.keys() {
        rendered.push_str(&format!("\n  key:  capsule.{key}"));
    }
    rendered
}
