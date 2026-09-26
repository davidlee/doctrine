// SPDX-License-Identifier: GPL-3.0-only
//! The `review` clap subcommand surface (SL-268 PHASE-02 T5).

use std::path::PathBuf;
use std::str::FromStr;

use clap::Subcommand;

use super::{Disposition, Facet, Format, Route, Severity};

#[derive(Subcommand)]
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
        #[arg(long)]
        target: String,

        /// Optional phase scope for a phase-scoped facet, e.g. `PHASE-03`.
        #[arg(long)]
        phase: Option<String>,

        /// Review title (default: derived from facet + target).
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
        /// subject canonical ref, e.g. `SL-024` (RFC-032 D5). Phase scope is
        /// ignored: `SL-024` also admits a `SL-024@PHASE-03` edge.
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
    /// finding with a fixed, raiser-owned severity/title/detail.
    Raise {
        /// Review reference — `RV-007` or the bare id `7`.
        reference: String,

        /// Severity: blocker | major | minor | nit (only `blocker` gates close).
        #[arg(long, value_parser = Severity::parse)]
        severity: Severity,

        /// The finding's title (fixed at raise).
        #[arg(long)]
        title: String,

        /// The finding's detail (fixed at raise).
        #[arg(long)]
        detail: String,

        /// Cooperative role assertion (default: raiser).
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

        /// The response detail (free-text).
        #[arg(long)]
        response: String,

        /// Cooperative role assertion (default: responder).
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

        /// The updated response detail (free-text, required).
        #[arg(long)]
        response: String,

        /// Why the finding is being amended — recorded on the amend turn
        /// (required, non-empty).
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

        /// Cooperative role assertion (default: responder).
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
        /// reasoning (optional).
        #[arg(long)]
        note: Option<String>,

        /// Cooperative role assertion (default: raiser).
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
        /// reasoning (required, non-empty).
        #[arg(long)]
        note: String,

        /// Cooperative role assertion (default: raiser).
        #[arg(long = "as")]
        role: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Reopen a verified finding (the raiser's verb) — hand it back to the
    /// responder (verified → contested).
    Reopen {
        /// Review reference — `RV-007` or the bare id `7`.
        reference: String,

        /// The finding id, e.g. `F-2`.
        #[arg(long)]
        finding: String,

        /// Why the finding is reopened — recorded on the reopen turn
        /// (required, non-empty).
        #[arg(long)]
        note: String,

        /// Cooperative role assertion (default: raiser).
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
        /// reasoning (optional).
        #[arg(long)]
        note: Option<String>,

        /// Cooperative role assertion (default: raiser).
        #[arg(long = "as")]
        role: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Declare the pass finished (the raiser's verb) — sets the concluded marker
    /// a design run's `Conducted` disposition is admissible over. Idempotent, and
    /// there is no unset. Open findings are fine: disposing them is the
    /// responder's work afterwards.
    Conclude {
        /// Review reference — `RV-007` or the bare id `7`.
        reference: String,

        /// Cooperative role assertion (default: raiser).
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
