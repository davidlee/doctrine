// SPDX-License-Identifier: GPL-3.0-only
//! The `review` clap subcommand surface (SL-268 PHASE-02 T5).

use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use clap::Subcommand;

use super::{Disposition, Facet, Format, Route, Severity};

#[derive(Debug, Subcommand)]
pub(crate) enum ReviewCommand {
    /// Open a new review ledger targeting an entity via the `reviews` edge.
    /// The `--target` ref is validated up front — a dangling ref is refused
    /// before any id is allocated. Findings are added later with `review raise`.
    New {
        /// What this review reviews (the facet): scope | design | plan |
        /// phase-plan | implementation | code-review | reconciliation.
        #[arg(long, value_parser = Facet::parse)]
        facet: Facet,

        /// The subject canonical ref the review targets, e.g. `SL-024`.
        /// `SL-NNN@PHASE-NN` is accepted as `--target SL-NNN --phase
        /// PHASE-NN`; the `@` here is a phase scope, not an `@path` file read.
        #[arg(long)]
        target: String,

        /// Optional phase scope for a phase-scoped facet, e.g. `PHASE-03`.
        /// Conflicts with an `@PHASE-NN` already on `--target`.
        #[arg(long)]
        phase: Option<String>,

        /// Review title (default: derived from facet + target). `-` reads
        /// stdin, `@path` reads a file.
        #[arg(long)]
        title: Option<String>,

        /// Raiser role label (cooperative; default `raiser`).
        #[arg(long)]
        raiser: Option<String>,

        /// Responder role label (cooperative; default `responder`).
        #[arg(long)]
        responder: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// List reviews by id: id, derived status (+ await), facet, target, title.
    List {
        #[command(flatten)]
        list: crate::CommonListArgs,

        /// Restrict to reviews whose `reviews` edge targets this ref — the
        /// subject canonical ref, e.g. `SL-024` (RFC-032 D5). A bare ref
        /// admits any phase: `SL-024` also matches a `SL-024@PHASE-03` edge.
        /// `SL-NNN@PHASE-NN` is accepted as a target+phase pair and narrows
        /// to that phase; the `@` here is a phase scope, not an `@path` file
        /// read.
        #[arg(long)]
        target: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Show one review: derived status, the `reviews` edge, and the brief.
    Show {
        /// Review reference — `RV-007` or the bare id `7`.
        reference: String,

        /// Output format.
        #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
        format: Format,

        /// Shorthand for `--format json`.
        #[arg(long)]
        json: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Raise a finding on a review (the raiser's verb) — appends an `open`
    /// finding with a fixed, raiser-owned severity/title/detail. Clears the
    /// pass's concluded marker.
    Raise {
        /// Review reference — `RV-007` or the bare id `7`.
        reference: String,

        /// Severity: blocker | major | minor | nit (only `blocker` gates close).
        #[arg(long, value_parser = Severity::parse)]
        severity: Severity,

        /// The finding's title (fixed at raise). `-` reads stdin, `@path`
        /// reads a file.
        #[arg(long)]
        title: String,

        /// The finding's detail (fixed at raise). `-` reads stdin, `@path`
        /// reads a file.
        #[arg(long)]
        detail: String,

        /// Cooperative role assertion: `raiser` | `responder`, or this ledger's declared
        /// labels (default: raiser).
        #[arg(long = "as")]
        role: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Dispose a finding (the responder's verb) — answer an open/contested
    /// finding, setting the responder-owned disposition + response.
    Dispose {
        /// Review reference — `RV-007` or the bare id `7`.
        reference: String,

        /// The finding id, e.g. `F-2`.
        #[arg(long)]
        finding: String,

        /// The disposition: aligned | fix-now | design-wrong | follow-up |
        /// tolerated. A `route:` prefix is refused — pass the route with
        /// `--route`.
        #[arg(long, value_parser = Disposition::parse)]
        disposition: Disposition,

        /// Where the answer routes: review | demonstrate | probe | control |
        /// owner-fix (optional; omitted keeps the finding's current route).
        #[arg(long, value_parser = Route::parse)]
        route: Option<Route>,

        /// The response detail (free-text). `-` reads stdin, `@path` reads a
        /// file.
        #[arg(long)]
        response: String,

        /// Cooperative role assertion: `raiser` | `responder`, or this ledger's declared
        /// labels (default: responder).
        #[arg(long = "as")]
        role: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Amend an already-answered finding (the responder's verb) — update the
    /// response and, optionally, the disposition/route (answered → answered).
    Amend {
        /// Review reference — `RV-007` or the bare id `7`.
        reference: String,

        /// The finding id, e.g. `F-2`.
        #[arg(long)]
        finding: String,

        /// The updated response detail (free-text, required). `-` reads
        /// stdin, `@path` reads a file.
        #[arg(long)]
        response: String,

        /// Why the finding is being amended — recorded on the amend turn
        /// (required, non-empty). `-` reads stdin, `@path` reads a file.
        #[arg(long)]
        note: String,

        /// The replacement disposition (optional; omitted keeps the current
        /// value): aligned | fix-now | design-wrong | follow-up | tolerated.
        #[arg(long, value_parser = Disposition::parse)]
        disposition: Option<Disposition>,

        /// The replacement route (optional; omitted keeps the current value):
        /// review | demonstrate | probe | control | owner-fix.
        #[arg(long, value_parser = Route::parse)]
        route: Option<Route>,

        /// Cooperative role assertion: `raiser` | `responder`, or this ledger's declared
        /// labels (default: responder).
        #[arg(long = "as")]
        role: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Verify an answered finding (the raiser's verb) — accept it (terminal).
    Verify {
        /// Review reference — `RV-007` or the bare id `7`.
        reference: String,

        /// The finding id, e.g. `F-2`.
        #[arg(long)]
        finding: String,

        /// Why the finding is accepted — recorded in the ledger as this turn's
        /// reasoning (optional). `-` reads stdin, `@path` reads a file.
        #[arg(long)]
        note: Option<String>,

        /// Cooperative role assertion: `raiser` | `responder`, or this ledger's declared
        /// labels (default: raiser).
        #[arg(long = "as")]
        role: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Contest an answered finding (the raiser's verb) — hand it back to the
    /// responder (answered → contested).
    Contest {
        /// Review reference — `RV-007` or the bare id `7`.
        reference: String,

        /// The finding id, e.g. `F-2`.
        #[arg(long)]
        finding: String,

        /// What the contest argues — recorded in the ledger as this turn's
        /// reasoning (required, non-empty). `-` reads stdin, `@path` reads a
        /// file.
        #[arg(long)]
        note: String,

        /// Cooperative role assertion: `raiser` | `responder`, or this ledger's declared
        /// labels (default: raiser).
        #[arg(long = "as")]
        role: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Reopen a verified finding (the raiser's verb) — hand it back to the
    /// responder (verified → contested). Clears the pass's concluded marker.
    Reopen {
        /// Review reference — `RV-007` or the bare id `7`.
        reference: String,

        /// The finding id, e.g. `F-2`.
        #[arg(long)]
        finding: String,

        /// Why the finding is reopened — recorded on the reopen turn
        /// (required, non-empty). `-` reads stdin, `@path` reads a file.
        #[arg(long)]
        note: String,

        /// Cooperative role assertion: `raiser` | `responder`, or this ledger's declared
        /// labels (default: raiser).
        #[arg(long = "as")]
        role: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Withdraw a finding (the raiser's verb) — retract an open/answered finding
    /// (terminal).
    Withdraw {
        /// Review reference — `RV-007` or the bare id `7`.
        reference: String,

        /// The finding id, e.g. `F-2`.
        #[arg(long)]
        finding: String,

        /// Why the finding is retracted — recorded in the ledger as this turn's
        /// reasoning (optional). `-` reads stdin, `@path` reads a file.
        #[arg(long)]
        note: Option<String>,

        /// Cooperative role assertion: `raiser` | `responder`, or this ledger's declared
        /// labels (default: raiser).
        #[arg(long = "as")]
        role: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Declare the pass finished (the raiser's verb) — sets the concluded marker
    /// a design run's `Conducted` disposition is admissible over. A later raise
    /// or reopen clears it; conclude again after them. Open findings are fine:
    /// disposing them is the responder's work afterwards.
    Conclude {
        /// Review reference — `RV-007` or the bare id `7`.
        reference: String,

        /// What this pass examined — recorded as the conclude turn's note.
        /// `-` reads stdin, `@path` reads a file.
        #[arg(long)]
        basis: String,

        /// Cooperative role assertion: `raiser` | `responder`, or this ledger's declared
        /// labels (default: raiser).
        #[arg(long = "as")]
        role: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Report a review's derived state and rebuild its baton (cache == recompute).
    Status {
        /// Review reference — `RV-007` or the bare id `7`.
        reference: String,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Prime the review context cache.
    /// Populates the warm-cache from the target slice's selectors — the path-set
    /// the staleness signal hashes (SL-147 PHASE-05).
    Prime {
        /// Review reference — `RV-007` or the bare id `7`.
        reference: String,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Remove a stale per-review lock left by a hard kill (escape hatch).
    Unlock {
        /// Review reference — `RV-007` or the bare id `7`.
        reference: String,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Print the file paths of each review entity directory.
    Paths {
        /// Review reference(s) — `RV-007` or the bare id `7`.
        refs: Vec<String>,

        #[arg(short = 't', long)]
        toml: bool,
        #[arg(short = 'm', long)]
        md: bool,
        #[arg(short = 'e', long)]
        entity: bool,
        #[arg(short = 's', long)]
        single: bool,

        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

impl ReviewCommand {
    /// Resolve every prose field's raw value through `input::resolve_prose`
    /// (SL-268 PHASE-07 D10, design sec-4): `-` reads `stdin` in full, `@path`
    /// reads a file, anything else passes through unchanged. `dispatch` calls
    /// this once, on its first line, before any verb sees its arguments
    /// (D-T1-1) — the `run_*` functions keep taking resolved `String`/`&str`,
    /// so no `run_*` signature changes.
    ///
    /// `refuse_second_dash` runs first, over each variant's prose fields
    /// (D-T1-2): only `Raise` (`--title`/`--detail`) and `Amend`
    /// (`--response`/`--note`) can carry two, but it is called uniformly.
    ///
    /// The empty-required refusal is split by which check exists today
    /// (D-T1-3): `--note` on contest/amend/reopen and `--basis` on conclude
    /// are already refused when blank, by `run_*`'s `ReviewError::NoteRequired`
    /// — that wording is golden-pinned, so this method does NOT add a
    /// CLI-side check for them; resolving `-`/`@path` to blank reaches the
    /// same `run_*` guard. `--title`/`--detail` on raise and `--response` on
    /// dispose/amend have no check today, so `require_nonempty` is added here,
    /// after resolution — this also refuses a literal empty value (Q1: yes).
    /// Optional prose (`new --title`, `verify --note`, `withdraw --note`) is
    /// resolved but never refused.
    ///
    /// `fs_read` must be callable more than once here (`Raise` and `Amend` each
    /// have two prose fields) — it is `impl Fn`, and a reference to it is
    /// handed down to `input::resolve_prose`'s `impl FnOnce`.
    pub(super) fn resolve_prose(
        self,
        stdin: &mut impl Read,
        fs_read: impl Fn(&Path) -> std::io::Result<String>,
    ) -> anyhow::Result<Self> {
        use crate::input::{refuse_second_dash, require_nonempty, resolve_prose as resolve};

        Ok(match self {
            ReviewCommand::New {
                facet,
                target,
                phase,
                title,
                raiser,
                responder,
                path,
            } => {
                let title = title
                    .map(|t| resolve(&t, "--title", stdin, &fs_read))
                    .transpose()?;
                ReviewCommand::New {
                    facet,
                    target,
                    phase,
                    title,
                    raiser,
                    responder,
                    path,
                }
            }
            ReviewCommand::Raise {
                reference,
                severity,
                title,
                detail,
                role,
                path,
            } => {
                refuse_second_dash(&[
                    ("--title", Some(title.as_str())),
                    ("--detail", Some(detail.as_str())),
                ])?;
                let title = resolve(&title, "--title", stdin, &fs_read)?;
                let detail = resolve(&detail, "--detail", stdin, &fs_read)?;
                require_nonempty("--title", &title)?;
                require_nonempty("--detail", &detail)?;
                ReviewCommand::Raise {
                    reference,
                    severity,
                    title,
                    detail,
                    role,
                    path,
                }
            }
            ReviewCommand::Dispose {
                reference,
                finding,
                disposition,
                route,
                response,
                role,
                path,
            } => {
                let response = resolve(&response, "--response", stdin, &fs_read)?;
                require_nonempty("--response", &response)?;
                ReviewCommand::Dispose {
                    reference,
                    finding,
                    disposition,
                    route,
                    response,
                    role,
                    path,
                }
            }
            ReviewCommand::Amend {
                reference,
                finding,
                response,
                note,
                disposition,
                route,
                role,
                path,
            } => {
                refuse_second_dash(&[
                    ("--response", Some(response.as_str())),
                    ("--note", Some(note.as_str())),
                ])?;
                let response = resolve(&response, "--response", stdin, &fs_read)?;
                let note = resolve(&note, "--note", stdin, &fs_read)?;
                require_nonempty("--response", &response)?;
                ReviewCommand::Amend {
                    reference,
                    finding,
                    response,
                    note,
                    disposition,
                    route,
                    role,
                    path,
                }
            }
            ReviewCommand::Verify {
                reference,
                finding,
                note,
                role,
                path,
            } => {
                let note = note
                    .map(|n| resolve(&n, "--note", stdin, &fs_read))
                    .transpose()?;
                ReviewCommand::Verify {
                    reference,
                    finding,
                    note,
                    role,
                    path,
                }
            }
            ReviewCommand::Contest {
                reference,
                finding,
                note,
                role,
                path,
            } => {
                let note = resolve(&note, "--note", stdin, &fs_read)?;
                ReviewCommand::Contest {
                    reference,
                    finding,
                    note,
                    role,
                    path,
                }
            }
            ReviewCommand::Reopen {
                reference,
                finding,
                note,
                role,
                path,
            } => {
                let note = resolve(&note, "--note", stdin, &fs_read)?;
                ReviewCommand::Reopen {
                    reference,
                    finding,
                    note,
                    role,
                    path,
                }
            }
            ReviewCommand::Withdraw {
                reference,
                finding,
                note,
                role,
                path,
            } => {
                let note = note
                    .map(|n| resolve(&n, "--note", stdin, &fs_read))
                    .transpose()?;
                ReviewCommand::Withdraw {
                    reference,
                    finding,
                    note,
                    role,
                    path,
                }
            }
            ReviewCommand::Conclude {
                reference,
                basis,
                role,
                path,
            } => {
                let basis = resolve(&basis, "--basis", stdin, &fs_read)?;
                ReviewCommand::Conclude {
                    reference,
                    basis,
                    role,
                    path,
                }
            }
            other => other,
        })
    }
}
