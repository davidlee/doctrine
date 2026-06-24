// SPDX-License-Identifier: GPL-3.0-only
//! Coverage CLI shell — thin command-tier wrapper over the coverage engine
//! modules (`coverage_store`, `coverage_verify`, `coverage_view`).

use std::str::FromStr;

use anyhow::Result;
use clap::Subcommand;

use crate::coverage_store;
use crate::coverage_verify;
use crate::coverage_view;
use crate::listing::Format;

#[derive(Subcommand)]
pub(crate) enum CoverageCommand {
    /// Show requirement coverage and drift.
    /// `<reference>` is REQ-NNN (one row) or PRD-/SPEC-NNN (a member fan).
    /// Derived observed coverage + the drift verdict against authored status —
    /// never writes, never derives status.
    Show {
        /// Canonical ref: REQ-NNN | PRD-NNN | SPEC-NNN.
        reference: String,

        /// Select/order visible table columns (e.g. `--columns id,status,verdict`).
        #[arg(long, value_delimiter = ',')]
        columns: Option<Vec<String>>,

        /// Output format (table | json).
        #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
        format: Format,

        /// Shorthand for `--format json`.
        #[arg(long)]
        json: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<std::path::PathBuf>,
    },

    /// Record a coverage observation.
    /// With any check field the cell is a `VT` recipe (leans Planned until
    /// verified); with none it is a `VA`/`VH` attestation.
    Record {
        /// Slice the cell is recorded under — `SL-NNN` or the bare number.
        #[arg(long)]
        slice: String,

        /// The requirement this evidence covers — `REQ-NNN`.
        #[arg(long)]
        requirement: String,

        /// The contributing change — `SL-NNN` (often the same as `--slice`).
        #[arg(long)]
        change: String,

        /// Verification mode: `VT` | `VA` | `VH`.
        #[arg(long)]
        mode: String,

        /// Observed status for a `VA`/`VH` attestation (default: verified). Ignored
        /// for a `VT` record (the verifier derives it; it leans Planned at record).
        #[arg(long, value_parser = coverage_store::parse_status)]
        status: Option<crate::requirement::CoverageStatus>,

        /// VT-check alias into `[verification.aliases]` (XOR `--command`).
        #[arg(long)]
        alias: Option<String>,

        /// VT-check literal command argv, repeatable (XOR `--alias`).
        #[arg(long = "command")]
        command: Vec<String>,

        /// Extra args appended to the resolved base argv, repeatable.
        #[arg(long = "extra-args")]
        extra_args: Vec<String>,

        /// Matcher source: `stdout` | `stderr` | `file:<glob>`.
        #[arg(long = "matcher-source")]
        matcher_source: Option<String>,

        /// Matcher pattern (substring, or regex with `--regex`).
        #[arg(long = "matcher-pattern")]
        matcher_pattern: Option<String>,

        /// Treat `--matcher-pattern` as a `regex_lite` pattern.
        #[arg(long)]
        regex: bool,

        /// Attestation date override (`YYYY-MM-DD`) for `VA`/`VH` (default: today).
        #[arg(long = "attested-date")]
        attested_date: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<std::path::PathBuf>,
    },

    /// Re-derive `VT` coverage status by re-running each entry's check. A single
    /// `<slice>` re-derives that slice; `--all` re-derives every slice (the
    /// global-dedup set — a shared check runs once across the invocation).
    Verify {
        /// The slice to verify — `SL-NNN` or the bare number (omit with `--all`).
        slice: Option<String>,

        /// Verify every slice in the corpus.
        #[arg(long)]
        all: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<std::path::PathBuf>,
    },

    /// Erase one coverage cell (the 4-tuple key) from a slice's store. Prints the
    /// withdrawn cell — a deletion that flips a composite green is never silent.
    Forget {
        /// Slice the cell lives under — `SL-NNN` or the bare number.
        #[arg(long)]
        slice: String,

        /// The requirement the cell covers — `REQ-NNN`.
        #[arg(long)]
        requirement: String,

        /// The contributing change — `SL-NNN`.
        #[arg(long)]
        change: String,

        /// Verification mode: `VT` | `VA` | `VH`.
        #[arg(long)]
        mode: String,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<std::path::PathBuf>,
    },
}

pub(crate) fn dispatch(cmd: CoverageCommand, color: bool) -> Result<()> {
    match cmd {
        CoverageCommand::Show {
            reference,
            columns,
            format,
            json,
            path,
        } => coverage_view::run(path, &reference, columns.as_deref(), format, json, color),
        CoverageCommand::Record {
            slice,
            requirement,
            change,
            mode,
            status,
            alias,
            command,
            extra_args,
            matcher_source,
            matcher_pattern,
            regex,
            attested_date,
            path,
        } => coverage_store::run_record(
            path,
            &coverage_store::CoverageRecordArgs {
                slice: &slice,
                requirement: &requirement,
                change: &change,
                mode: &mode,
                status,
                alias: alias.as_deref(),
                command: &command,
                extra_args: &extra_args,
                matcher_source: matcher_source.as_deref(),
                matcher_pattern: matcher_pattern.as_deref(),
                regex,
                attested_date: attested_date.as_deref(),
            },
        ),
        CoverageCommand::Verify { slice, all, path } => {
            coverage_verify::run_cli(path, slice.as_deref(), all)
        }
        CoverageCommand::Forget {
            slice,
            requirement,
            change,
            mode,
            path,
        } => coverage_store::run_forget(path, &slice, &requirement, &change, &mode),
    }
}
