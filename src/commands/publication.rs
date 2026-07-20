// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine publication validate` — admit the shipped publication manifest and
//! report every declared entry (SL-223 D-A).
//!
//! This is the PRODUCTION CONSUMER that makes the `publication` engine's
//! admission API reachable under `deny(unused)` (a `pub(crate)` seam a binary
//! reaches only from `#[cfg(test)]` is a hard compile error — codex RV-287 F-4),
//! the observable licence/resolvability gate, and (PHASE-04, host-gated) the
//! release-artifact probe. It is a *publication*-tier validation verb — it does
//! NOT surface asset bytes to the user; `library list|tree|show` are a later slice.

use clap::Subcommand;

/// `doctrine publication <verb>` — publication-tier verbs. `validate` admits the
/// shipped manifest; resolve/emit-backed reporting extends it in PHASE-03.
#[derive(Debug, Subcommand)]
pub(crate) enum PublicationCommand {
    /// Admit the shipped publication manifest and report each declared entry.
    Validate,
}

/// Admit the embedded publication manifest, then resolve + emit every declared
/// entry through an [`EmbeddedAdapter`]-bound [`Resolver`] into a sink — the
/// production consumer that makes `load`→`resolve`→`emit` and every error type
/// reachable under `deny(unused)` (PHASE-03). Prints a per-entry line that reads
/// every parsed field (dead-field discipline), collects any resolve/emit
/// failures, and `bail!`s non-zero when the manifest is malformed / unlicensed /
/// duplicate (admission) or an entry fails to resolve/emit. Rides the
/// `run_validate` output shape (`writeln!` to stdout — sidesteps `print_stdout`).
pub(crate) fn run_publication_validate() -> anyhow::Result<()> {
    use crate::publication::{EmbeddedAdapter, PublicationManifest, Resolver};
    use std::io::Write;

    let resolver = Resolver::new(PublicationManifest::load()?, EmbeddedAdapter);
    let entries = resolver.manifest().entries();
    let count = entries.len();
    let plural = if count == 1 { "y" } else { "ies" };

    let mut stdout = std::io::stdout();
    writeln!(
        stdout,
        "publication validate: {count} entr{plural} declared"
    )?;

    let mut sink = std::io::sink();
    let mut failures: Vec<String> = Vec::new();
    for entry in entries {
        match resolver.emit(entry.address(), &mut sink) {
            Ok(()) => writeln!(stdout, "  ok    {}", entry.report_line())?,
            Err(err) => {
                writeln!(stdout, "  FAIL  {} — {err}", entry.address().as_str())?;
                failures.push(entry.address().as_str().to_string());
            }
        }
    }

    if !failures.is_empty() {
        anyhow::bail!(
            "publication validate: {} of {count} entr{plural} failed to resolve/emit: {}",
            failures.len(),
            failures.join(", ")
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // VT-5: the production consumer runs green over the shipped embedded manifest
    // (exercises load + the whole admission API on a non-test path).
    #[test]
    fn run_publication_validate_admits_shipped_manifest() {
        run_publication_validate().expect("shipped manifest validates");
    }
}
