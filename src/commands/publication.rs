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

/// Admit the embedded publication manifest and print a per-entry pass line that
/// reads EVERY parsed field (dead-field discipline). Admission is fail-closed, so
/// a malformed / unlicensed / duplicate manifest surfaces as a non-zero exit via
/// the propagated [`crate::publication::AdmissionError`]. Rides the `run_validate`
/// output shape (`writeln!` to stdout — sidesteps the `print_stdout` deny — plus a
/// non-zero error return).
pub(crate) fn run_publication_validate() -> anyhow::Result<()> {
    use std::io::Write;
    let manifest = crate::publication::PublicationManifest::load()?;
    let entries = manifest.entries();
    writeln!(
        std::io::stdout(),
        "publication validate: {} entr{} declared",
        entries.len(),
        if entries.len() == 1 { "y" } else { "ies" }
    )?;
    for entry in entries {
        writeln!(std::io::stdout(), "  ok  {}", entry.report_line())?;
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
