// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine review` — the RV adversarial-review ledger kind (SL-040, ADR-007).
//! One generic `facet`-parameterized ledger reviews any subject via the outbound
//! `reviews` edge, coordinated (in later phases) by a turn-based baton.
//!
//! This file follows the `worktree.rs` one-file shape: a **pure core** (this
//! phase — the closed vocabulary enums, the `derived_status` summary, the `can`
//! transition predicate, and the `toml_string`-escaped render fns) and a future
//! **impure shell** (the `entity::Kind` row, verb handlers, baton/lock/cache
//! coordination — PHASE-02+). PHASE-01 ships the pure core only: no engine row,
//! no `main.rs` command surface, no I/O.
//!
//! Every closed vocabulary enum carries an `as_str` render mirror plus a
//! `&[&str]` const array, kept in lockstep by a per-enum drift canary test
//! (the `adr.rs`/`backlog.rs` pattern). The arrays are stood up for the
//! PHASE-02 engine/CLI vocabulary; the suppression is `cfg_attr(not(test), …)`
//! so the canary round-trips (real uses) do not leave it unfulfilled in the
//! test build (mem.pattern.lint.dead-code-expect-vs-cfg-test /
//! mem.pattern.lint.dead-code-self-clearing-leaf).
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "pure-core vocabulary + predicates stood up in SL-040 PHASE-01; first non-test consumers (engine row, verb handlers) land in PHASE-02/03 — self-clearing"
    )
)]

use std::str::FromStr;

use clap::Subcommand;

use crate::tomlfmt::toml_string;

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

        /// The disposition (free-text; e.g. fixed / design-wrong / tolerated).
        #[arg(long)]
        disposition: String,

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

    /// Verify an answered finding (the raiser's verb) — accept it (terminal).
    Verify {
        /// Review reference — `RV-007` or the bare id `7`.
        reference: String,

        /// The finding id, e.g. `F-2`.
        #[arg(long)]
        finding: String,

        /// Ephemeral handoff chatter for the baton log — NOT durable rationale
        /// (durable justification belongs in a finding).
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

        /// Ephemeral handoff chatter for the baton log — NOT durable rationale
        /// (durable justification belongs in a finding).
        #[arg(long)]
        note: Option<String>,

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

    /// Populate the reviewer-context warm-cache from a curated `domain_map`, or
    /// (`--seed`) emit git-changed candidate paths to curate from (ADR-007 D-C10).
    Prime {
        /// Review reference — `RV-007` or the bare id `7`.
        reference: String,

        /// Emit git-changed candidate paths (a starting point, not authority) and
        /// exit, instead of priming. Writes nothing.
        #[arg(long)]
        seed: bool,

        /// Read the curated `domain_map` from a file (default: stdin).
        #[arg(long)]
        from: Option<PathBuf>,

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

pub(crate) fn dispatch(cmd: ReviewCommand, color: bool) -> anyhow::Result<()> {
    match cmd {
        ReviewCommand::New {
            facet,
            target,
            phase,
            title,
            raiser,
            responder,
            path,
        } => {
            use std::io::Write;
            let out = run_new(
                path,
                &NewArgs {
                    facet,
                    target,
                    phase,
                    title,
                    raiser,
                    responder,
                },
            )?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::List { list, path } => {
            use std::io::Write;
            let out = run_list(path, list.into_list_args(color))?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Show {
            reference,
            format,
            json,
            path,
        } => {
            use std::io::Write;
            let out = run_show(path, &reference, if json { Format::Json } else { format })?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Raise {
            reference,
            severity,
            title,
            detail,
            role,
            path,
        } => {
            use std::io::Write;
            let role = parse_role(role.as_deref(), Role::Raiser)?;
            let out = run_raise(
                path,
                &RaiseArgs {
                    reference,
                    severity,
                    title,
                    detail,
                },
                role,
            )?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Dispose {
            reference,
            finding,
            disposition,
            response,
            role,
            path,
        } => {
            use std::io::Write;
            let role = parse_role(role.as_deref(), Role::Responder)?;
            let out = run_dispose(
                path,
                &DisposeArgs {
                    reference,
                    finding,
                    disposition,
                    response,
                },
                role,
            )?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Verify {
            reference,
            finding,
            note,
            role,
            path,
        } => {
            use std::io::Write;
            let role = parse_role(role.as_deref(), Role::Raiser)?;
            let out = run_verify(path, &reference, &finding, note.as_deref(), role)?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Contest {
            reference,
            finding,
            note,
            role,
            path,
        } => {
            use std::io::Write;
            let role = parse_role(role.as_deref(), Role::Raiser)?;
            let out = run_contest(path, &reference, &finding, note.as_deref(), role)?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Withdraw {
            reference,
            finding,
            role,
            path,
        } => {
            use std::io::Write;
            let role = parse_role(role.as_deref(), Role::Raiser)?;
            let out = run_withdraw(path, &reference, &finding, role)?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Status { reference, path } => {
            use std::io::Write;
            let out = run_status(path, &reference)?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Prime {
            reference,
            seed,
            from,
            path,
        } => {
            use std::io::Write;
            let out = run_prime(
                path,
                &PrimeArgs {
                    reference,
                    seed,
                    from,
                },
            )?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Unlock { reference, path } => {
            use std::io::Write;
            let out = run_unlock(path, &reference)?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Paths {
            refs,
            toml,
            md,
            entity,
            single,
            path,
        } => {
            use std::io::Write;
            let root = crate::root::find(path, &crate::root::default_markers())?;
            let review_root = root.join(REVIEW_DIR);
            let sel = crate::paths::PathSelection {
                toml,
                md,
                entity,
                single,
            };
            let mut all_lines: Vec<String> = Vec::new();
            for r in &refs {
                let id = parse_ref(r)?;
                let name = format!("{id:03}");
                let entity_dir = review_root.join(&name);
                let toml_name = format!("review-{name}.toml");
                let md_name = format!("review-{name}.md");
                let set = crate::paths::scan_entity_dir(
                    &entity_dir,
                    &entity_dir.join(&toml_name),
                    Some(&entity_dir.join(&md_name)),
                    &root,
                )?;
                let lines = crate::paths::select_paths(&set, &sel)?;
                all_lines.extend(lines);
            }
            write!(std::io::stdout(), "{}", all_lines.join("\n"))?;
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Closed vocabulary enums (each: `as_str` render mirror + a `&[&str]` known-set,
// lockstep-guarded by a drift canary test).
// ---------------------------------------------------------------------------

/// What a review reviews — the facet (design §5, D-C11). The closed 7-set with
/// **no `drift`** (D-C11 dropped it → the future Drift Ledger kind, IMP-022).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", try_from = "String", into = "String")]
pub(crate) enum Facet {
    Scope,
    Design,
    Plan,
    PhasePlan,
    Implementation,
    CodeReview,
    Reconciliation,
}

impl Facet {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Scope => "scope",
            Self::Design => "design",
            Self::Plan => "plan",
            Self::PhasePlan => "phase-plan",
            Self::Implementation => "implementation",
            Self::CodeReview => "code-review",
            Self::Reconciliation => "reconciliation",
        }
    }
}

/// The `Facet` known-set — closed, no `drift` (D-C11). Lockstep-guarded against
/// the enum by `facet_known_set_matches_variants`.
const FACETS: &[&str] = &[
    "scope",
    "design",
    "plan",
    "phase-plan",
    "implementation",
    "code-review",
    "reconciliation",
];

/// A finding's lifecycle status (design §5). Single-owner edges only — never
/// free-edited; `verified`/`withdrawn` are terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum FindingStatus {
    Open,
    Answered,
    Contested,
    Verified,
    Withdrawn,
}

impl FindingStatus {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Answered => "answered",
            Self::Contested => "contested",
            Self::Verified => "verified",
            Self::Withdrawn => "withdrawn",
        }
    }

    /// Whether this status is terminal (`verified`/`withdrawn`) — a finding that
    /// no verb moves on. A review is `Done` iff every finding is terminal
    /// (design §8, D-C9a).
    pub(crate) const fn is_terminal(self) -> bool {
        matches!(self, Self::Verified | Self::Withdrawn)
    }
}

/// The `FindingStatus` known-set. Lockstep-guarded by
/// `finding_status_known_set_matches_variants`.
const FINDING_STATUSES: &[&str] = &["open", "answered", "contested", "verified", "withdrawn"];

/// A finding's severity (design §5, raiser-owned, fixed at raise). Only
/// `blocker` gates `/close` (D-C9b).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", try_from = "String", into = "String")]
pub(crate) enum Severity {
    Blocker,
    Major,
    Minor,
    Nit,
}

impl Severity {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Blocker => "blocker",
            Self::Major => "major",
            Self::Minor => "minor",
            Self::Nit => "nit",
        }
    }

    /// Parse a `--severity` token against the closed 4-set (the `Facet::parse`
    /// pattern — keeps the pure-core enum clap-free). `blocker` is the only
    /// severity that gates `/close` (D-C9b).
    pub(crate) fn parse(s: &str) -> Result<Self, String> {
        match s {
            "blocker" => Ok(Self::Blocker),
            "major" => Ok(Self::Major),
            "minor" => Ok(Self::Minor),
            "nit" => Ok(Self::Nit),
            other => Err(format!(
                "unknown severity `{other}` (known: {})",
                SEVERITIES.join(", ")
            )),
        }
    }
}

impl TryFrom<String> for Severity {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::parse(&s)
    }
}

impl From<Severity> for String {
    fn from(s: Severity) -> Self {
        s.as_str().to_owned()
    }
}

/// The `Severity` known-set. Lockstep-guarded by
/// `severity_known_set_matches_variants`.
const SEVERITIES: &[&str] = &["blocker", "major", "minor", "nit"];

/// The party asserting a verb (`--as`, design §5). Cooperative role assertion,
/// not security (ADR-007 Negative).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Role {
    Raiser,
    Responder,
}

impl Role {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Raiser => "raiser",
            Self::Responder => "responder",
        }
    }
}

/// The `Role` known-set. Lockstep-guarded by `role_known_set_matches_variants`.
const ROLES: &[&str] = &["raiser", "responder"];

/// A review's derived status (design §8, D-C8) — **never stored**, computed from
/// the findings at read time. Total over the finding-status enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReviewStatus {
    Active,
    Done,
}

impl ReviewStatus {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Done => "done",
        }
    }
}

/// The `ReviewStatus` known-set. Lockstep-guarded by
/// `review_status_known_set_matches_variants`.
pub(crate) const REVIEW_STATUSES: &[&str] = &["active", "done"];

/// Whose turn the review summarizes to (design §8, D-C2 — the baton caches this).
/// A *priority summary* for display/handoff routing, **not** an independent gate
/// (the gate is per-finding `can()`, D7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Await {
    Raiser,
    Responder,
    None,
}

impl Await {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Raiser => "raiser",
            Self::Responder => "responder",
            Self::None => "none",
        }
    }
}

/// The five write verbs that move a finding's status (design §5). `status` and
/// the read/coordination verbs are not transition verbs and are not modelled
/// here — `can` answers "may this verb fire on a finding in `from` for `role`?".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Verb {
    Raise,
    Dispose,
    Verify,
    Contest,
    Withdraw,
}

impl Verb {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Raise => "raise",
            Self::Dispose => "dispose",
            Self::Verify => "verify",
            Self::Contest => "contest",
            Self::Withdraw => "withdraw",
        }
    }

    /// The role a verb statically requires, knowable without a finding (design
    /// §6 responsibility split): raise/verify/contest/withdraw are the raiser's;
    /// dispose is the responder's. The `with_turn` wrapper checks this; the
    /// per-finding `can` check (state-dependent) is the closure's job.
    pub(crate) const fn required_role(self) -> Role {
        match self {
            Self::Raise | Self::Verify | Self::Contest | Self::Withdraw => Role::Raiser,
            Self::Dispose => Role::Responder,
        }
    }
}

// ---------------------------------------------------------------------------
// Transition predicate (design §5, D-C4/D-C5)
// ---------------------------------------------------------------------------

/// Whether `verb` may fire on a finding currently in `from` (or, for `raise`,
/// not yet existing — `None`) when asserted by `role`. Pure and total: the
/// single-owner edge table (design §5), every other combination refused.
///
/// | verb     | from               | role      | → |
/// |----------|--------------------|-----------|---|
/// | raise    | (none)             | raiser    | open |
/// | dispose  | open \| contested  | responder | answered |
/// | verify   | answered           | raiser    | verified (terminal) |
/// | contest  | answered           | raiser    | contested |
/// | withdraw | open \| answered   | raiser    | withdrawn (terminal) |
pub(crate) const fn can(verb: Verb, from: Option<FindingStatus>, role: Role) -> bool {
    // Static role check first — the half `with_turn` also owns; refuse a
    // role/verb mismatch regardless of state.
    if !role_eq(role, verb.required_role()) {
        return false;
    }
    matches!(
        (verb, from),
        (Verb::Raise, None)
            | (
                Verb::Dispose,
                Some(FindingStatus::Open | FindingStatus::Contested)
            )
            | (Verb::Verify | Verb::Contest, Some(FindingStatus::Answered))
            | (
                Verb::Withdraw,
                Some(FindingStatus::Open | FindingStatus::Answered)
            )
    )
}

/// Const-context `Role` equality (the derived `PartialEq` is not `const`).
const fn role_eq(a: Role, b: Role) -> bool {
    matches!(
        (a, b),
        (Role::Raiser, Role::Raiser) | (Role::Responder, Role::Responder)
    )
}

// ---------------------------------------------------------------------------
// Structured return types (SL-109, design D1/D8)
// ---------------------------------------------------------------------------

/// The structured output of a review verb — one variant per verb, carrying
/// exactly the data its consumers need. `#[derive(Serialize)]` for MCP
/// transport; the CLI path formats via `print_review()` in `main.rs`.
#[derive(Debug, Serialize)]
pub(crate) enum ReviewOutput {
    Created {
        id: u32,
        canonical: String,
        dir: PathBuf,
    },
    Raised {
        finding_id: String,
        review_id: u32,
    },
    Disposed {
        finding_id: String,
        review_id: u32,
    },
    Verified {
        finding_id: String,
        review_id: u32,
    },
    Contested {
        finding_id: String,
        review_id: u32,
    },
    Withdrawn {
        finding_id: String,
        review_id: u32,
    },
    Showed {
        id: u32,
        canonical: String,
        title: String,
        status: String,
        awaiting: String,
        facet: String,
        target: String,
        #[serde(rename = "finding_count")]
        findings_count: usize,
        findings: Vec<Finding>,
        body: String,
        #[serde(skip)]
        formatted: String,
    },
    Listed {
        rows: Vec<ListRow>,
        /// Pre-truncation row count, set MCP-side only when an output cap dropped
        /// rows (IMP-114). `None` (absent on the wire) ⇒ the rows are complete —
        /// keeps uncapped lists and the CLI path byte-unchanged.
        #[serde(skip_serializing_if = "Option::is_none")]
        total: Option<usize>,
        #[serde(skip)]
        formatted: String,
    },
    Primed {
        canonical: String,
        tracked_paths: Vec<String>,
        areas_count: usize,
        tracked_count: usize,
        invariants_count: usize,
        risks_count: usize,
        #[serde(skip)]
        is_seed: bool,
    },
    Status {
        canonical: String,
        status: String,
        awaiting: String,
        findings_count: usize,
        rounds: usize,
        cache_primed: bool,
        stale_paths: Vec<String>,
        #[serde(skip)]
        formatted: String,
    },
    Unlocked {
        canonical: String,
        #[serde(skip)]
        formatted: String,
    },
}

/// Format a [`ReviewOutput`] for CLI human consumption — the single formatting
/// pass, one match arm per variant, following the output contract (§4 design.md).
/// Returns the formatted string; the caller writes it to stdout.
pub(crate) fn print_review(out: &ReviewOutput) -> String {
    match out {
        ReviewOutput::Created {
            id,
            canonical: _,
            dir,
        } => {
            format!("Created review {:03}: {}\n", id, dir.display())
        }
        ReviewOutput::Raised {
            finding_id,
            review_id,
        } => {
            format!("Raised {} on {}\n", finding_id, canonical_id(*review_id))
        }
        ReviewOutput::Disposed {
            finding_id,
            review_id,
        } => {
            format!(
                "Disposed {} on {} (answered)\n",
                finding_id,
                canonical_id(*review_id)
            )
        }
        ReviewOutput::Verified {
            finding_id,
            review_id,
        } => {
            format!(
                "Verified {} on {} (verified)\n",
                finding_id,
                canonical_id(*review_id)
            )
        }
        ReviewOutput::Contested {
            finding_id,
            review_id,
        } => {
            format!(
                "Contested {} on {} (contested)\n",
                finding_id,
                canonical_id(*review_id)
            )
        }
        ReviewOutput::Withdrawn {
            finding_id,
            review_id,
        } => {
            format!(
                "Withdrew {} on {} (withdrawn)\n",
                finding_id,
                canonical_id(*review_id)
            )
        }
        ReviewOutput::Showed { formatted, .. }
        | ReviewOutput::Listed { formatted, .. }
        | ReviewOutput::Status { formatted, .. } => formatted.clone(),
        ReviewOutput::Primed {
            canonical,
            tracked_paths,
            areas_count,
            tracked_count,
            invariants_count,
            risks_count,
            is_seed,
        } => {
            if *is_seed {
                let mut s = format!(
                    "# {canonical} prime --seed: {} git-changed candidate(s) — curate into a domain_map (not authority)\n",
                    tracked_paths.len()
                );
                for path in tracked_paths {
                    s.push_str(path);
                    s.push('\n');
                }
                s
            } else {
                format!(
                    "{canonical} primed — {areas_count} area(s), {tracked_count} tracked path(s), {invariants_count} invariant(s), {risks_count} risk(s)\n"
                )
            }
        }
        ReviewOutput::Unlocked {
            canonical,
            formatted,
        } => {
            if formatted.is_empty() {
                format!("{canonical} is not locked\n")
            } else {
                formatted.clone()
            }
        }
    }
}

/// Structured error from the review engine — each variant carries typed fields
/// so the MCP transport layer can map to JSON-RPC error codes by variant
/// identity, never by string-parsing (design D8; RV-092 F-1).
#[derive(Debug)]
#[expect(dead_code, reason = "variants constructed in PHASE-02 verb handlers")]
pub(crate) enum ReviewError {
    NotFound {
        reference: String,
    },
    RoleMismatch {
        expected: Role,
        actual: Role,
        verb: Verb,
    },
    StateMismatch {
        finding: String,
        current: FindingStatus,
        required: FindingStatus,
    },
    DanglingRef {
        target: String,
    },
    LockContention {
        canonical: String,
        details: String,
    },
    Internal {
        source: anyhow::Error,
    },
}

impl fmt::Display for ReviewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { reference } => {
                write!(f, "review not found: {reference}")
            }
            Self::RoleMismatch {
                expected,
                actual,
                verb,
            } => {
                write!(
                    f,
                    "`{}` is the {}'s verb; --as {} cannot assert it",
                    verb.as_str(),
                    expected.as_str(),
                    actual.as_str()
                )
            }
            Self::StateMismatch {
                finding,
                current,
                required,
            } => {
                write!(
                    f,
                    "out of turn on {finding}: current status {} != required {}",
                    current.as_str(),
                    required.as_str()
                )
            }
            Self::DanglingRef { target } => {
                write!(f, "target not found: {target}")
            }
            Self::LockContention { canonical, details } => {
                write!(f, "{canonical}: {details}")
            }
            Self::Internal { source } => {
                write!(f, "{source}")
            }
        }
    }
}

impl std::error::Error for ReviewError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Internal { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Derived status (design §8, D-C8 / D7)
// ---------------------------------------------------------------------------

/// The status carrier `derived_status` reads — the finding's current status is
/// all the summary needs. The full authored `Finding` (with severity/title/…)
/// lands in PHASE-02/03; this keeps the pure core dependency-free.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FindingState {
    pub(crate) status: FindingStatus,
}

/// The review's derived status + summarized turn (design §8, D-C8). Total over
/// the finding-status enum; never stored (computed at `show`/`list`/`status`).
///
/// - empty ⇒ `(Done, None)` — no findings, nothing to reconcile.
/// - any `open`/`contested` ⇒ `(Active, Responder)` — work awaits the responder.
/// - else any `answered` ⇒ `(Active, Raiser)` — work awaits the raiser.
/// - all `∈ {verified, withdrawn}` ⇒ `(Done, None)`.
///
/// `await` is a *priority summary* (open/contested wins display), never an
/// exclusive gate — the turn gate is per-finding `can` (D7).
pub(crate) fn derived_status(findings: &[FindingState]) -> (ReviewStatus, Await) {
    if findings.is_empty() {
        return (ReviewStatus::Done, Await::None);
    }
    if findings
        .iter()
        .any(|f| matches!(f.status, FindingStatus::Open | FindingStatus::Contested))
    {
        return (ReviewStatus::Active, Await::Responder);
    }
    if findings.iter().any(|f| f.status == FindingStatus::Answered) {
        return (ReviewStatus::Active, Await::Raiser);
    }
    (ReviewStatus::Done, Await::None)
}

// ---------------------------------------------------------------------------
// Render (pure — user free-text spliced via `toml_string`, design §4/§5)
// ---------------------------------------------------------------------------

/// A finding as raised, for rendering. Raiser-owned fixed fields plus the
/// responder-owned mutable pair; `status` is transition-graph-owned. The
/// authored on-disk shape of a `[[finding]]` (design §5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Finding {
    pub(crate) id: String,
    pub(crate) status: FindingStatus,
    pub(crate) severity: Severity,
    pub(crate) title: String,
    pub(crate) detail: String,
    pub(crate) disposition: Option<String>,
    pub(crate) response: Option<String>,
}

/// Render a single `[[finding]]` TOML block (design §5). Every user free-text
/// field (`title`, `detail`, `disposition`, `response`) is emitted through
/// `toml_string` so a `"`, `\`, newline, or `]` can neither break the document
/// nor inject a key (mem.pattern.render.toml-splice-escape-user-values). The
/// id/status/severity fields are closed vocabularies, rendered bare.
pub(crate) fn render_finding(finding: &Finding) -> String {
    let mut out = String::new();
    out.push_str("[[finding]]\n");
    push_line(&mut out, "id", &toml_string(&finding.id));
    push_line(&mut out, "status", &toml_string(finding.status.as_str()));
    push_line(
        &mut out,
        "severity",
        &toml_string(finding.severity.as_str()),
    );
    push_line(&mut out, "title", &toml_string(&finding.title));
    push_line(&mut out, "detail", &toml_string(&finding.detail));
    if let Some(disposition) = &finding.disposition {
        push_line(&mut out, "disposition", &toml_string(disposition));
    }
    if let Some(response) = &finding.response {
        push_line(&mut out, "response", &toml_string(response));
    }
    out
}

/// Emit a single `key = value` line (no `push`/`format!` of literals — repo
/// clippy bans the noisy forms; this is the sanctioned string-assembly shape).
fn push_line(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = ");
    out.push_str(value);
    out.push('\n');
}

// ===========================================================================
// Impure shell (SL-040 PHASE-02) — the `entity::Kind` row, the authored readers,
// and the `new`/`show`/`list` command surface. Verbs / baton / lock / cache are
// PHASE-03+. Review mirrors `slice` (raw `entity::Kind`, derived status,
// `state_dir`), NOT `adr`'s `GovKind` spine (design D1).
// ===========================================================================

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io::{self, Read as _, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::contentset::{self, ContentSet};
use crate::entity::{self, Kind, LocalFs, Materialised};
use crate::listing::{self, Column, Format, ListArgs};

/// Relative dir of the review tree inside the project root. A distinct top-level
/// authored tree (design §4), parallel to `.doctrine/slice`.
pub(crate) const REVIEW_DIR: &str = ".doctrine/review";

/// The review kind: `review-NNN.toml` + `review-NNN.md` + `NNN-slug` symlink,
/// riding the kind-blind engine (design D1, the slice shape). The scaffold is
/// inert — review renders its fileset eagerly (facet/target exceed `ScaffoldCtx`,
/// the `materialise_named` rationale) via [`materialise_fresh_prebuilt`], so the
/// `Kind.scaffold` fn is never called for review; it exists only to satisfy the
/// `Kind` descriptor `integrity::KINDS` references.
pub(crate) const REVIEW_KIND: Kind = Kind {
    dir: REVIEW_DIR,
    prefix: crate::kinds::RV,
    stem: "review",
    scaffold: review_scaffold_unused,
};

/// Inert scaffold — see [`REVIEW_KIND`]. Review never rides `Kind.scaffold`
/// (its fileset is built eagerly in [`run_new`]); this is the descriptor stub.
fn review_scaffold_unused(_ctx: &entity::ScaffoldCtx<'_>) -> anyhow::Result<entity::Fileset> {
    anyhow::bail!("review materialises eagerly, not via Kind.scaffold")
}

impl Facet {
    /// Parse a `--facet` token against the closed 7-set (the `memory.rs`
    /// `MemoryType::parse` pattern — keeps the pure-core enum free of clap). The
    /// error names every valid facet, mirroring `listing::validate_statuses`.
    pub(crate) fn parse(s: &str) -> Result<Self, String> {
        match s {
            "scope" => Ok(Self::Scope),
            "design" => Ok(Self::Design),
            "plan" => Ok(Self::Plan),
            "phase-plan" => Ok(Self::PhasePlan),
            "implementation" => Ok(Self::Implementation),
            "code-review" => Ok(Self::CodeReview),
            "reconciliation" => Ok(Self::Reconciliation),
            other => Err(format!(
                "unknown facet `{other}` (known: {})",
                FACETS.join(", ")
            )),
        }
    }
}

impl TryFrom<String> for Facet {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::parse(&s)
    }
}

impl From<Facet> for String {
    fn from(f: Facet) -> Self {
        f.as_str().to_owned()
    }
}

// ---------------------------------------------------------------------------
// Render (eager — the authored ledger toml + the `## Brief` md companion)
// ---------------------------------------------------------------------------

/// The parsed `[target]` edge (design §5/§7): the subject canonical ref and an
/// optional phase scope. Validated at `new`; the edge `RV-NNN ──reviews──▶ ref`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct Target {
    #[serde(rename = "ref")]
    reference: String,
    #[serde(default)]
    phase: Option<String>,
}

/// The `[review]` metadata table (design §5): the facet and the two role labels.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct ReviewMeta {
    facet: String,
    raiser: String,
    responder: String,
}

/// Render `review-NNN.toml` from the embedded template (design §4). Every
/// closed-vocab field (`facet`) and user-supplied string (`slug`/`title`/
/// `target.ref`/`phase`/role labels) is spliced through `toml_string` so a
/// hostile value cannot break the document or inject a key
/// (mem.pattern.render.toml-splice-escape-user-values). The optional `[target].
/// phase` line is present iff a phase was given.
fn render_review_toml(
    id: u32,
    slug: &str,
    title: &str,
    review: &ReviewMeta,
    target: &Target,
) -> anyhow::Result<String> {
    let phase_line = match &target.phase {
        Some(p) => {
            let mut line = String::from("phase = ");
            line.push_str(&toml_string(p));
            line
        }
        None => String::new(),
    };
    Ok(crate::install::asset_text("templates/review.toml")?
        .replace("{{id}}", &id.to_string())
        .replace("{{slug}}", &toml_string(slug))
        .replace("{{title}}", &toml_string(title))
        .replace("{{facet}}", &toml_string(&review.facet))
        .replace("{{raiser}}", &toml_string(&review.raiser))
        .replace("{{responder}}", &toml_string(&review.responder))
        .replace("{{target_ref}}", &toml_string(&target.reference))
        .replace("{{target_phase}}", &phase_line))
}

/// Render `review-NNN.md` — the `## Brief` companion (design §5/D-C6). Plain
/// markdown token substitution (no toml-splice escaping: markdown body, not a
/// structured value).
fn render_review_md(canonical: &str, facet: &str, target_ref: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/review.md")?
        .replace("{{ref}}", canonical)
        .replace("{{facet}}", facet)
        .replace("{{target}}", target_ref))
}

// ---------------------------------------------------------------------------
// CLI: `review new`
// ---------------------------------------------------------------------------

/// The bundled `review new` arguments — one struct to dodge the clippy arg-ceiling
/// (mem.pattern.lint.cli-handler-args-struct).
#[derive(Deserialize)]
pub(crate) struct NewArgs {
    pub(crate) facet: Facet,
    pub(crate) target: String,
    pub(crate) phase: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) raiser: Option<String>,
    pub(crate) responder: Option<String>,
}

/// `doctrine review new --facet F --target REF [--phase P]` — allocate a fresh RV
/// and write its authored ledger (empty findings) plus the `## Brief` md. The
/// `[target].ref` is validated up front (design §7): a dangling / unknown-prefix
/// ref is refused BEFORE any id is claimed, so a bad edge never mints an entity.
/// The empty-ledger RV is the real `Active`/await=`Raiser` state (D-C8).
pub(crate) fn run_new(path: Option<PathBuf>, args: &NewArgs) -> anyhow::Result<ReviewOutput> {
    let root = crate::root::find(path, &crate::root::default_markers())?;

    // Forward-edge validation (design §7): refuse a dangling / unknown target
    // BEFORE claiming an id. Reuses the corpus id table (integrity::KINDS).
    crate::integrity::ensure_ref_resolves(&root, &args.target)?;

    let title = args
        .title
        .clone()
        .unwrap_or_else(|| format!("{} review of {}", args.facet.as_str(), args.target));
    let slug = crate::input::resolve_slug(&title, None)?;
    let review = ReviewMeta {
        facet: args.facet.as_str().to_owned(),
        raiser: args.raiser.clone().unwrap_or_else(|| "raiser".to_owned()),
        responder: args
            .responder
            .clone()
            .unwrap_or_else(|| "responder".to_owned()),
    };
    let target = Target {
        reference: args.target.clone(),
        phase: args.phase.clone(),
    };

    let trunk_ids = crate::git::trunk_entity_ids(&root, REVIEW_DIR)?;
    let out: Materialised = entity::materialise_fresh_prebuilt(
        &LocalFs,
        &root,
        REVIEW_DIR,
        REVIEW_KIND.prefix,
        &trunk_ids,
        |id, canonical| {
            let name = format!("{id:03}");
            Ok(vec![
                entity::Artifact::File {
                    rel_path: PathBuf::from(format!("{name}/review-{name}.toml")),
                    body: render_review_toml(id, &slug, &title, &review, &target)?,
                },
                entity::Artifact::File {
                    rel_path: PathBuf::from(format!("{name}/review-{name}.md")),
                    body: render_review_md(canonical, &review.facet, &target.reference)?,
                },
                entity::Artifact::Symlink {
                    rel_path: PathBuf::from(format!("{name}-{slug}")),
                    target: name,
                },
            ])
        },
    )?;

    let id = out
        .eid
        .numeric_id()
        .context("review kind must yield a numeric id")?;
    Ok(ReviewOutput::Created {
        id,
        canonical: canonical_id(id),
        dir: out.dir,
    })
}

// ---------------------------------------------------------------------------
// show / list — review computes its OWN derived status (never the shared reader)
// ---------------------------------------------------------------------------

/// One authored `[[finding]]` row, read as data for `show`/`list` derived status.
/// A faithful mirror of the on-disk shape (the raiser/responder/status fields);
/// the closed-vocab strings are validated only where a transition needs them
/// (PHASE-03) — `show`/`list` read them verbatim.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct FindingRow {
    id: String,
    status: String,
    severity: String,
    title: String,
    detail: String,
    #[serde(default)]
    disposition: Option<String>,
    #[serde(default)]
    response: Option<String>,
}

/// The full `review-NNN.toml` read as data (design §5) — id/slug/title (NO stored
/// status, D-C8), the `[review]` and `[target]` tables, and the append-only
/// findings. Review's own readers parse this; the shared strict `Meta` is never
/// asked for a status review does not store (D2).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct ReviewDoc {
    id: u32,
    slug: String,
    title: String,
    review: ReviewMeta,
    target: Target,
    #[serde(default)]
    finding: Vec<FindingRow>,
}

impl ReviewDoc {
    /// Map the authored finding-status strings to the pure [`FindingStatus`] for
    /// the derived-status summary. An out-of-vocab status (a hand-edit) is treated
    /// as `Open` — non-terminal, keeping the review `Active` rather than silently
    /// closing it (the conservative read; the write path validates in PHASE-03).
    fn finding_states(&self) -> Vec<FindingState> {
        self.finding
            .iter()
            .map(|f| FindingState {
                status: parse_finding_status(&f.status),
            })
            .collect()
    }

    /// The review's derived `(ReviewStatus, Await)` (design §8) — computed at read
    /// time, never stored.
    fn derived(&self) -> (ReviewStatus, Await) {
        derived_status(&self.finding_states())
    }
}

/// Parse an authored finding-status string into [`FindingStatus`], defaulting an
/// unknown value to `Open` (conservative — see [`ReviewDoc::finding_states`]).
fn parse_finding_status(s: &str) -> FindingStatus {
    match s {
        "answered" => FindingStatus::Answered,
        "contested" => FindingStatus::Contested,
        "verified" => FindingStatus::Verified,
        "withdrawn" => FindingStatus::Withdrawn,
        _ => FindingStatus::Open,
    }
}

/// Read one review's `review-NNN.toml` as data.
fn read_review(review_root: &Path, id: u32) -> anyhow::Result<ReviewDoc> {
    let name = format!("{id:03}");
    let path = review_root.join(&name).join(format!("review-{name}.toml"));
    let text = fs::read_to_string(&path)
        .with_context(|| format!("review {name} not found at {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("Failed to parse {}", path.display()))
}

/// A review's authored outbound relation (SL-046 §5.2/§5.3): the single
/// `[target].ref` subject edge `RV-N ──reviews──▶ target` →
/// [`RelationLabel::Reviews`]. Reads via the existing `read_review` reader (no new
/// TOML parse). Always exactly one edge (the target ref is required at `new`).
pub(crate) fn relation_edges(
    root: &Path,
    id: u32,
) -> anyhow::Result<Vec<crate::relation::RelationEdge>> {
    use crate::relation::{RelationEdge, RelationLabel};
    let doc = read_review(&root.join(REVIEW_DIR), id)?;
    Ok(vec![RelationEdge::new(
        RelationLabel::Reviews,
        doc.target.reference,
    )])
}

/// A review's DERIVED status string (`"active"`/`"done"`) for the cross-kind
/// priority scan (SL-047 §5.2). An RV authors no `status` field (D-C8); its status
/// is `derived_status` over the AUTHORED finding ledger — authored-tier, not a
/// runtime read. Reads via the existing `read_review` reader (no new TOML parse),
/// then runs the same pure `derived` the `show`/`list`/`status` surfaces use.
pub(crate) fn derived_status_string(root: &Path, id: u32) -> anyhow::Result<String> {
    let doc = read_review(&root.join(REVIEW_DIR), id)?;
    let (status, _await) = doc.derived();
    Ok(status.as_str().to_string())
}

/// Read every `review-NNN.toml` under the review tree as data (for `list`).
fn read_reviews(review_root: &Path) -> anyhow::Result<Vec<ReviewDoc>> {
    let mut docs = Vec::new();
    for id in entity::scan_ids(review_root)? {
        docs.push(read_review(review_root, id)?);
    }
    Ok(docs)
}

/// The `RV-NNN` canonical id for a numeric review id, via the single id-form
/// authority.
fn canonical_id(id: u32) -> String {
    listing::canonical_id(REVIEW_KIND.prefix, id)
}

/// The `reviews`-edge line shown in `show` and as the `target` column: the
/// outbound edge to the subject, with an optional `@phase` scope (ADR-004).
fn edge_label(doc: &ReviewDoc) -> String {
    match &doc.target.phase {
        Some(p) => {
            let mut s = doc.target.reference.clone();
            s.push('@');
            s.push_str(p);
            s
        }
        None => doc.target.reference.clone(),
    }
}

/// Parse a review reference — `RV-007`, `rv-7`, or the bare id `7` — to its id.
fn parse_ref(reference: &str) -> anyhow::Result<u32> {
    let digits = reference
        .strip_prefix("RV-")
        .or_else(|| reference.strip_prefix("rv-"))
        .unwrap_or(reference);
    digits.parse::<u32>().with_context(|| {
        format!("not a review reference: `{reference}` (expected `RV-007` or `7`)")
    })
}

/// One unresolved blocker holding a target's closure open (design §7, D8/D-C9b):
/// the canonical RV id (`RV-007`) and the offending finding id (`F-2`). Surfaced
/// by the close-gate to name *why* a closure-seam transition is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockerRef {
    pub(crate) rv: String,
    pub(crate) finding: String,
}

/// Pure check (design §7): the unresolved blocker findings *this* RV holds against
/// its target. A finding gates iff `severity == Blocker && status ∉ {verified,
/// withdrawn}` — but ONLY on an **Active** review (`derived_status == Active`,
/// D-C8): a `Done` ledger (every finding terminal, D-C9a) holds nothing, even if a
/// stray non-terminal status were hand-edited in (the derived gate already
/// excludes that by keeping it Active). No I/O — operates on already-read data so
/// the scan shell stays thin (the `integrity::scan_kind` shape).
fn doc_unresolved_blockers(doc: &ReviewDoc) -> Vec<BlockerRef> {
    if doc.derived().0 != ReviewStatus::Active {
        return Vec::new();
    }
    doc.finding
        .iter()
        .filter(|f| Severity::parse(&f.severity) == Ok(Severity::Blocker))
        .filter(|f| !parse_finding_status(&f.status).is_terminal())
        .map(|f| BlockerRef {
            rv: canonical_id(doc.id),
            finding: f.id.clone(),
        })
        .collect()
}

/// The reverse close-gate scan (design §7, D8/D-C9b) — a **standalone scoped scan**
/// over `.doctrine/review/*`, NOT the spec `Registry` (wrong cohesion) and NOT a
/// general reverse index (scope non-goal). Returns every unresolved blocker
/// (`severity == Blocker && status ∉ {verified, withdrawn}`) on an Active RV whose
/// `[target].ref` matches `subject_ref`. The thin shell: read every ledger, then
/// the pure [`doc_unresolved_blockers`] filters each. O(#RV)/close (R2 — fine at
/// scale, index later). Used by the slice-close command shell one-way
/// (`slice`-shell → `review`-query); `review` must not import `slice` (ADR-001).
pub(crate) fn unresolved_blockers_for(
    root: &Path,
    subject_ref: &str,
) -> anyhow::Result<Vec<BlockerRef>> {
    let review_root = root.join(REVIEW_DIR);
    if !review_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut blockers = Vec::new();
    for doc in read_reviews(&review_root)? {
        if doc.target.reference == subject_ref {
            blockers.extend(doc_unresolved_blockers(&doc));
        }
    }
    Ok(blockers)
}

/// `doctrine review show <RV-NNN>` — read the RV as data and render the readable
/// whole (`Table`) or the faithful toml-as-data + brief (`Json`). The status is
/// DERIVED here (review never asks the shared reader for a stored status, D-C8);
/// the `reviews` edge is rendered as `RV-NNN ──reviews──▶ <target>`.
pub(crate) fn run_show(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
) -> anyhow::Result<ReviewOutput> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let review_root = root.join(REVIEW_DIR);
    let id = parse_ref(reference)?;
    let doc = read_review(&review_root, id)?;
    let body = read_brief(&review_root, id)?;
    let (status, awaiting) = doc.derived();
    let formatted = match format {
        Format::Table => format_show(&doc, &body),
        Format::Json => show_json(&doc, &body)?,
    };
    let canonical = canonical_id(id);
    let title = doc.title.clone();
    let facet = doc.review.facet.clone();
    let target = edge_label(&doc);
    let findings_count = doc.finding.len();
    let findings: Vec<Finding> = doc
        .finding
        .iter()
        .map(|fr| Finding {
            id: fr.id.clone(),
            status: parse_finding_status(&fr.status),
            severity: Severity::parse(&fr.severity).unwrap_or(Severity::Major),
            title: fr.title.clone(),
            detail: fr.detail.clone(),
            disposition: fr.disposition.clone(),
            response: fr.response.clone(),
        })
        .collect();
    Ok(ReviewOutput::Showed {
        id,
        canonical,
        title,
        status: status.as_str().to_owned(),
        awaiting: awaiting.as_str().to_owned(),
        facet,
        target,
        findings_count,
        findings,
        body,
        formatted,
    })
}

/// Read the `review-NNN.md` brief body (the prose companion).
fn read_brief(review_root: &Path, id: u32) -> anyhow::Result<String> {
    let name = format!("{id:03}");
    let path = review_root.join(&name).join(format!("review-{name}.md"));
    fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))
}

/// Render the `Table` show: identity header, the derived status + await, the
/// `reviews` edge, then the brief body. House style — `Vec<String>` joined by
/// `concat` (avoids the `push_str(&format!)` lint).
fn format_show(doc: &ReviewDoc, body: &str) -> String {
    let (status, awaited) = doc.derived();
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("{} — {}\n", canonical_id(doc.id), doc.title));
    parts.push(format!(
        "{} · {} · await={}\n",
        doc.review.facet,
        status.as_str(),
        awaited.as_str()
    ));
    parts.push(format!(
        "{} ──reviews──▶ {}\n",
        canonical_id(doc.id),
        edge_label(doc)
    ));
    parts.push(format!(
        "findings: {} (raiser {} · responder {})\n",
        doc.finding.len(),
        doc.review.raiser,
        doc.review.responder
    ));
    parts.push(format!("\n{body}"));
    parts.concat()
}

/// The faithful JSON `show` row — the toml-as-data plus the derived status (the
/// one computed field surfaced; never stored) and the brief body.
#[derive(Debug, Serialize)]
struct ShowJson<'a> {
    #[serde(flatten)]
    doc: &'a ReviewDoc,
    status: &'a str,
    awaiting: &'a str,
}

/// Render the `Json` show under the shared `{kind, …}` envelope.
fn show_json(doc: &ReviewDoc, body: &str) -> anyhow::Result<String> {
    let (status, awaited) = doc.derived();
    let row = ShowJson {
        doc,
        status: status.as_str(),
        awaiting: awaited.as_str(),
    };
    let value = serde_json::json!({ "kind": "review", "review": row, "body": body });
    serde_json::to_string_pretty(&value).context("failed to serialize review show JSON")
}

/// The `review list` row tuple: the doc plus its derived status (computed once).
type ReviewRow = (ReviewDoc, ReviewStatus, Await);

const REVIEW_COLUMNS: [Column<ReviewRow>; 5] = [
    Column {
        name: "id",
        header: "id",
        cell: |(d, _, _)| canonical_id(d.id),
        paint: listing::ColumnPaint::Fixed(owo_colors::DynColors::Ansi(
            owo_colors::AnsiColors::Cyan,
        )),
    },
    Column {
        name: "status",
        header: "status",
        cell: |(_, s, a)| {
            let mut cell = s.as_str().to_owned();
            cell.push_str(" (await ");
            cell.push_str(a.as_str());
            cell.push(')');
            cell
        },
        // ByValue reads the row's RAW derived status, NOT the emitted composite
        // `active (await …)` cell (F-4) — matching the cell text would drop colour.
        paint: listing::ColumnPaint::ByValue(|(_, s, _)| listing::status_hue(s.as_str())),
    },
    Column {
        name: "facet",
        header: "facet",
        cell: |(d, _, _)| d.review.facet.clone(),
        paint: listing::ColumnPaint::None,
    },
    Column {
        name: "target",
        header: "target",
        cell: |(d, _, _)| edge_label(d),
        paint: listing::ColumnPaint::None,
    },
    Column {
        name: "title",
        header: "title",
        cell: |(d, _, _)| d.title.clone(),
        paint: listing::ColumnPaint::Alternate([listing::TITLE_EVEN, listing::TITLE_ODD]),
    },
];

/// The default visible column set for `review list`.
const REVIEW_DEFAULT: &[&str] = &["id", "status", "facet", "target", "title"];

/// A review's filterable projection (design §5 list axes). The derived status is
/// the filter status (review stores none); `canonical` is the regex domain.
fn key(d: &ReviewDoc) -> listing::FilterFields {
    let (status, _) = d.derived();
    listing::FilterFields {
        canonical: canonical_id(d.id),
        slug: d.slug.clone(),
        title: d.title.clone(),
        status: status.as_str().to_owned(),
        tags: Vec::new(),
    }
}

/// `review list` rows as a string — the compute half of [`run_list`]. No hide-set
/// (an RV is either Active or Done; both are listed), sorted by id, each row
/// carrying its derived status.
fn list_rows(root: &Path, mut args: ListArgs) -> anyhow::Result<(String, Vec<ListRow>)> {
    listing::validate_statuses(&args.status, REVIEW_STATUSES)?;
    let render = args.render;
    let columns = args.columns.take();
    let (filter, format) = listing::build(args)?;
    let review_root = root.join(REVIEW_DIR);
    let mut docs = listing::retain(read_reviews(&review_root)?, &filter, |_| false, key);
    docs.sort_by_key(|d| d.id);
    let rows: Vec<ReviewRow> = docs
        .into_iter()
        .map(|d| {
            let (status, awaited) = d.derived();
            (d, status, awaited)
        })
        .collect();
    let formatted = match format {
        Format::Table => {
            let sel = listing::select_columns(&REVIEW_COLUMNS, REVIEW_DEFAULT, columns.as_deref())?;
            listing::render_columns(&rows, &sel, render)
        }
        Format::Json => listing::json_envelope("review", &json_rows(&rows))?,
    };
    Ok((formatted, json_rows(&rows)))
}

/// Faithful JSON rows for `list` — the prefixed id, derived status/await, facet,
/// target edge, and title.
#[derive(Debug, Serialize)]
pub(crate) struct ListRow {
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) awaiting: String,
    pub(crate) facet: String,
    pub(crate) target: String,
    pub(crate) title: String,
}

fn json_rows(rows: &[ReviewRow]) -> Vec<ListRow> {
    rows.iter()
        .map(|(d, status, awaited)| ListRow {
            id: canonical_id(d.id),
            status: status.as_str().to_owned(),
            awaiting: awaited.as_str().to_owned(),
            facet: d.review.facet.clone(),
            target: edge_label(d),
            title: d.title.clone(),
        })
        .collect()
}

/// `doctrine review list` — list reviews by id with derived status, facet, and
/// the `reviews`-edge target.
pub(crate) fn run_list(path: Option<PathBuf>, args: ListArgs) -> anyhow::Result<ReviewOutput> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (formatted, rows) = list_rows(&root, args)?;
    Ok(ReviewOutput::Listed {
        rows,
        total: None,
        formatted,
    })
}

// ===========================================================================
// PHASE-03 — the verb family + runtime coordination (the turn guard).
//
// The full finding lifecycle (raise/dispose/verify/contest/withdraw) rides ONE
// higher-order seam, `with_turn` (design §6, D6) — the single home of
// D-C3 (authored-first/baton-last ordering), D-C4 (the static verb→role gate),
// and D-C4a (the create_new lock + the sha256 CAS, fired in TWO distinct windows:
// entry — a hand-edit landing BEFORE this invocation; pre-write — a hand-edit
// landing DURING it). The lock serializes concurrent invocations; the CAS catches
// out-of-band human edits the lock cannot see (no invocation ⇒ no lock).
//
// Locus = the parent tree's gitignored runtime state, `.doctrine/state/review/NNN/`
// (D4/D-C7). A review verb whose resolved root is a *fork* bails — fork-invoked
// review is IMP-024, not yet supported (the pilot invariant, enforced at root
// resolution).
// ===========================================================================

/// The runtime baton (design §6, D-C2) — gitignored, regenerable, never authored.
/// `await`/`authored_hash` are cache-derivable from the authored ledger (the
/// recompute floor); `rounds`/`contests`/`handoff` are non-derivable observability
/// bookkeeping (lost on baton loss — acceptable, D-C2).
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
struct Baton {
    /// The summarized turn (D-C8) — a display/routing convenience, never a gate.
    #[serde(default)]
    awaiting: String,
    /// The CAS key: sha256 of the authored ledger bytes this baton was last
    /// reconciled against (D-C4a). A divergence ⇒ an out-of-band edit landed.
    #[serde(default)]
    authored_hash: String,
    /// A coarse turn counter — bumped each turn (observability only).
    #[serde(default)]
    rounds: u32,
    /// How many `contest` turns this review has seen (observability only).
    #[serde(default)]
    contests: u32,
    /// Ephemeral handoff chatter (design D10) — the `--note` on contest/verify
    /// lands here, NOT durable rationale. Lost on baton loss by design.
    #[serde(default)]
    handoff: Vec<String>,
}

/// The runtime subtree for one review's baton + lock (design §6). Parent-tree
/// locus, gitignored (`.gitignore` already covers `.doctrine/state/`).
fn state_dir(root: &Path, id: u32) -> PathBuf {
    root.join(".doctrine/state/review").join(format!("{id:03}"))
}

fn baton_path(root: &Path, id: u32) -> PathBuf {
    state_dir(root, id).join("baton.toml")
}

fn lock_path(root: &Path, id: u32) -> PathBuf {
    state_dir(root, id).join("lock")
}

/// Read the baton if present (`None` = cold — treat as a fresh recompute, D-C4a).
fn read_baton(root: &Path, id: u32) -> anyhow::Result<Option<Baton>> {
    let path = baton_path(root, id);
    match fs::read_to_string(&path) {
        Ok(text) => Ok(Some(toml::from_str(&text).with_context(|| {
            format!("Failed to parse baton {}", path.display())
        })?)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("Failed to read baton {}", path.display())),
    }
}

/// Write the baton atomically (temp+rename), creating the state subtree first.
fn write_baton(root: &Path, id: u32, baton: &Baton) -> anyhow::Result<()> {
    let dir = state_dir(root, id);
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let body = toml::to_string(baton).context("serialize baton")?;
    crate::fsutil::write_atomic(&baton_path(root, id), body.as_bytes())
}

/// Compute the `(await, authored_hash)` the baton should carry for a ledger whose
/// findings are `findings` and whose bytes hash to `hash` — the D-C2 recompute
/// floor reused by entry-CAS heal, the per-turn refresh, and `status`.
fn reconcile_baton_fields(findings: &[FindingState], hash: &str) -> (String, String) {
    let (_, awaited) = derived_status(findings);
    (awaited.as_str().to_owned(), hash.to_owned())
}

/// A RAII lock: `create_new` the lockfile on construction (an `AlreadyExists`
/// race is the caller's "RV-NNN busy" bail), remove it on `drop` — covering the
/// normal AND panic paths (NOT a hard-kill `-9`, which leaves a stale lock for
/// `review unlock`). The lock serializes concurrent *invocations* only; it is
/// held within one invocation and the turn persists via the baton (design §6).
struct LockGuard {
    path: PathBuf,
}

impl LockGuard {
    /// Acquire the per-review lock, writing a `pid timestamp` diagnostic body
    /// (`review unlock` surfaces it on a stale lock). `AlreadyExists` ⇒ a
    /// concurrent invocation holds it ⇒ a clean "busy; re-run" bail, no clobber.
    fn acquire(root: &Path, id: u32) -> anyhow::Result<Self> {
        let dir = state_dir(root, id);
        fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
        let path = lock_path(root, id);
        match crate::fsutil::create_new_file(&path) {
            Ok(mut file) => {
                let stamp = crate::clock::now_timestamp().unwrap_or_default();
                let body = format!("pid = {}\nacquired = \"{stamp}\"\n", std::process::id());
                // Best-effort diagnostics body; a write failure does not invalidate
                // the lock (the file's existence is the mutex, not its contents).
                file.write_all(body.as_bytes())
                    .with_context(|| format!("write lock body {}", path.display()))?;
                Ok(Self { path })
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                anyhow::bail!(
                    "{} busy (another `review` invocation holds the lock); re-run \
                     (a stale lock from a hard kill clears with `review unlock`)",
                    canonical_id(id)
                )
            }
            Err(e) => Err(e).with_context(|| format!("acquire lock {}", path.display())),
        }
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        // Best-effort: a failed removal leaves a stale lock for `review unlock`.
        // Drop cannot propagate an error; the must-use Result is deliberately
        // discarded into a binding (the sanctioned form under the repo lint).
        let _ignored = fs::remove_file(&self.path);
    }
}

/// Map an authored `FindingRow`'s status string to the pure [`FindingState`] for
/// `derived_status`/baton reconciliation (the conservative read of §8).
fn finding_states_of(doc: &ReviewDoc) -> Vec<FindingState> {
    doc.finding
        .iter()
        .map(|f| FindingState {
            status: parse_finding_status(&f.status),
        })
        .collect()
}

/// Read the authored ledger bytes + the parsed doc for a review id — the step-2
/// snapshot the two CAS windows compare against.
fn read_authored(root: &Path, id: u32) -> anyhow::Result<(String, ReviewDoc)> {
    let name = format!("{id:03}");
    let path = root
        .join(REVIEW_DIR)
        .join(&name)
        .join(format!("review-{name}.toml"));
    let text = fs::read_to_string(&path)
        .with_context(|| format!("review {name} not found at {}", path.display()))?;
    let doc: ReviewDoc =
        toml::from_str(&text).with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok((text, doc))
}

/// The authored ledger path for a review id.
fn authored_path(root: &Path, id: u32) -> PathBuf {
    let name = format!("{id:03}");
    root.join(REVIEW_DIR)
        .join(&name)
        .join(format!("review-{name}.toml"))
}

/// Resolve the project root for a review verb and ENFORCE the pilot invariant
/// (design D4/D-C1): a verb whose resolved root is a *fork* (linked worktree)
/// bails — fork-invoked review is IMP-024, not yet supported. The baton/lock live
/// in the parent tree's gitignored state, which a fork's `WITHHELD` tier keeps it
/// from seeing (`worktree.rs:71`). The guard lives here, in the shell, at root
/// resolution — every verb routes through it.
fn resolve_review_root(path: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    if crate::worktree::is_linked_worktree(&root).unwrap_or(false) {
        anyhow::bail!(
            "review verbs are not supported on a worktree fork (IMP-024): the turn \
             baton lives in the parent tree's gitignored state, which a fork cannot \
             co-write. Run `review` from the parent tree."
        );
    }
    Ok(root)
}

/// A test seam for the pre-write CAS window (design §6 step 5). The default is a
/// no-op; a concurrency test injects a hand-edit here to fire mid-invocation,
/// between the step-2 read and the step-5 write — deterministically, without
/// threads. `with_turn` is the production entry point (no hook).
type MidTurnHook<'a> = &'a dyn Fn();

/// The single turn-taking seam (design §6, D6). Runs the numbered protocol:
///
/// 1. acquire the `create_new` lock (RAII) — `AlreadyExists` ⇒ "busy; re-run".
/// 2. read + snapshot the authored ledger bytes.
/// 3. ENTRY CAS: `sha256(authored) ≠ baton.authored_hash` ⇒ heal the baton (the
///    D-C2 recompute), bail "ledger changed underneath — re-run" (missing baton
///    ⇒ cold, proceed). Catches an edit landing BEFORE this invocation.
/// 4. STATIC role check: `role == verb.required_role()` — mismatch ⇒ bail (D-C4).
/// 5. AUTHORED FIRST: run the closure `f` (per-finding `can()` + the edit), then
///    PRE-WRITE CAS (re-read bytes ≠ the step-2 snapshot ⇒ bail, do NOT write —
///    catches an edit landing DURING this invocation), else `write_atomic`.
/// 6. recompute `await` + the new hash from the written ledger.
/// 7. BATON LAST: `write_atomic` the baton.
/// 8. release the lock (`LockGuard` drop).
fn with_turn<F, T>(root: &Path, id: u32, verb: Verb, role: Role, f: F) -> anyhow::Result<T>
where
    F: FnOnce(&mut toml_edit::DocumentMut, &[FindingRow]) -> anyhow::Result<T>,
{
    with_turn_hooked(root, id, verb, role, &|| {}, f)
}

/// `with_turn` with an injectable mid-turn hook (the pre-write CAS test seam).
fn with_turn_hooked<F, T>(
    root: &Path,
    id: u32,
    verb: Verb,
    role: Role,
    mid_turn: MidTurnHook<'_>,
    f: F,
) -> anyhow::Result<T>
where
    F: FnOnce(&mut toml_edit::DocumentMut, &[FindingRow]) -> anyhow::Result<T>,
{
    // 1. acquire lock (RAII — released on every exit path below, incl. panic).
    let _lock = LockGuard::acquire(root, id)?;

    // 2. read + snapshot the authored bytes.
    let (snapshot, doc) = read_authored(root, id)?;
    let snapshot_hash = crate::git::sha256(snapshot.as_bytes());

    // 3. ENTRY CAS — an edit landed BEFORE this invocation (baton stale).
    //    (a missing baton ⇒ cold — proceed; the per-turn write seeds it.)
    if let Some(baton) = read_baton(root, id)?.filter(|b| b.authored_hash != snapshot_hash) {
        // Heal: recompute await from the authored truth (D-C2), refresh the
        // baton's CAS key, preserve the observability counters, then bail.
        let (awaiting, hash) = reconcile_baton_fields(&finding_states_of(&doc), &snapshot_hash);
        let healed = Baton {
            awaiting,
            authored_hash: hash,
            ..baton
        };
        write_baton(root, id, &healed)?;
        anyhow::bail!(
            "{} ledger changed underneath the baton — re-run (the baton has been \
             refreshed from the authored ledger)",
            canonical_id(id)
        );
    }

    // 4. STATIC role check (D-C4) — the half the wrapper owns.
    if role != verb.required_role() {
        return Err(ReviewError::RoleMismatch {
            expected: verb.required_role(),
            actual: role,
            verb,
        }
        .into());
    }

    // 5. AUTHORED FIRST — the closure runs the per-finding can() + applies the
    //    edit-preserving edit, then the PRE-WRITE CAS re-reads the bytes.
    let mut document = snapshot
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", authored_path(root, id).display()))?;
    let result = f(&mut document, &doc.finding)?;

    // Test seam: a hand-edit injected here lands AFTER the step-2 read and BEFORE
    // the step-5 write — the exact window the pre-write CAS must catch.
    mid_turn();

    // PRE-WRITE CAS — the authored bytes must still match the step-2 snapshot.
    let current = fs::read_to_string(authored_path(root, id))
        .with_context(|| format!("re-read {}", authored_path(root, id).display()))?;
    if crate::git::sha256(current.as_bytes()) != snapshot_hash {
        anyhow::bail!(
            "{} ledger changed underneath this turn — re-run (a hand-edit landed \
             mid-turn; nothing was written, no clobber)",
            canonical_id(id)
        );
    }
    let new_body = document.to_string();
    crate::fsutil::write_atomic(&authored_path(root, id), new_body.as_bytes())?;

    // 6. recompute await + the new hash from the just-written ledger.
    let new_hash = crate::git::sha256(new_body.as_bytes());
    let new_doc: ReviewDoc = toml::from_str(&new_body)
        .with_context(|| format!("re-parse {}", authored_path(root, id).display()))?;
    let (awaiting, hash) = reconcile_baton_fields(&finding_states_of(&new_doc), &new_hash);

    // 7. BATON LAST — preserve the observability counters across the turn.
    let prior = read_baton(root, id)?.unwrap_or_default();
    let contests = prior.contests + u32::from(verb == Verb::Contest);
    let baton = Baton {
        awaiting,
        authored_hash: hash,
        rounds: prior.rounds + 1,
        contests,
        // Handoff chatter is appended by the verb shell AFTER this turn write
        // (D10) — carry the prior log forward untouched here.
        handoff: prior.handoff,
    };
    write_baton(root, id, &baton)?;

    // 8. release lock — `_lock` drops at scope end.
    Ok(result)
}

// ---------------------------------------------------------------------------
// Finding-scoped edit-preserving toml_edit (the governance.rs:290 pattern,
// extended to a `[[finding]]` array element). Comments / unknown keys survive.
// ---------------------------------------------------------------------------

/// Locate the `[[finding]]` table whose `id == finding_id`, returning a mutable
/// handle. The lookup is by the authored `id` field, never by array position (an
/// append-only ledger never renumbers, but order is not identity).
fn finding_table_mut<'a>(
    doc: &'a mut toml_edit::DocumentMut,
    finding_id: &str,
) -> anyhow::Result<&'a mut toml_edit::Table> {
    let array = doc
        .get_mut("finding")
        .and_then(toml_edit::Item::as_array_of_tables_mut)
        .ok_or_else(|| anyhow::anyhow!("ledger has no findings"))?;
    array
        .iter_mut()
        .find(|t| t.get("id").and_then(toml_edit::Item::as_str) == Some(finding_id))
        .ok_or_else(|| anyhow::anyhow!("no finding `{finding_id}` in the ledger"))
}

/// Apply a single-owner status transition (design §5): set the finding's
/// `status`, plus any responder-owned `disposition`/`response`. Edit-preserving —
/// the table is mutated in place, so comments / unknown keys / sibling findings
/// survive (the `governance.rs:290` contract at finding scope). User free-text
/// rides `toml_edit::value`, which quotes/escapes it (the structured-write twin of
/// the render path's `toml_string`).
fn apply_transition(
    table: &mut toml_edit::Table,
    new_status: FindingStatus,
    disposition: Option<&str>,
    response: Option<&str>,
) {
    table.insert("status", toml_edit::value(new_status.as_str()));
    if let Some(d) = disposition {
        table.insert("disposition", toml_edit::value(d));
    }
    if let Some(r) = response {
        table.insert("response", toml_edit::value(r));
    }
}

/// Append a fresh `[[finding]]` with id `F-<max+1>` (design §5, append-only —
/// never renumber, never reuse). Raiser-owned fields are fixed here at raise; the
/// status is seeded `open`; the responder pair is absent until a `dispose`.
fn append_finding(
    doc: &mut toml_edit::DocumentMut,
    existing: &[FindingRow],
    severity: Severity,
    title: &str,
    detail: &str,
) -> String {
    let next = next_finding_id(existing);
    let mut row = toml_edit::Table::new();
    row.insert("id", toml_edit::value(&next));
    row.insert("status", toml_edit::value(FindingStatus::Open.as_str()));
    row.insert("severity", toml_edit::value(severity.as_str()));
    row.insert("title", toml_edit::value(title));
    row.insert("detail", toml_edit::value(detail));
    if let Some(array) = doc
        .entry("finding")
        .or_insert_with(|| toml_edit::Item::ArrayOfTables(toml_edit::ArrayOfTables::new()))
        .as_array_of_tables_mut()
    {
        array.push(row);
    }
    next
}

/// The next append-only finding id: `F-<max+1>` over the existing `F-<n>` ids
/// (design §5). Robust to a gap / a non-conforming id (skipped in the max scan).
fn next_finding_id(existing: &[FindingRow]) -> String {
    let max = existing
        .iter()
        .filter_map(|f| f.id.strip_prefix("F-"))
        .filter_map(|n| n.parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    format!("F-{}", max + 1)
}

/// The current authored status of a finding (for the per-finding `can()` gate).
fn finding_status_of(existing: &[FindingRow], finding_id: &str) -> anyhow::Result<FindingStatus> {
    let row = existing
        .iter()
        .find(|f| f.id == finding_id)
        .ok_or_else(|| anyhow::anyhow!("no finding `{finding_id}` in the ledger"))?;
    Ok(parse_finding_status(&row.status))
}

// ---------------------------------------------------------------------------
// The write verbs — each rides `with_turn`; the closure owns the per-finding gate
// ---------------------------------------------------------------------------

/// Bundled `review raise` args (the clippy arg-ceiling — `cli-handler-args-struct`).
#[derive(Deserialize)]
pub(crate) struct RaiseArgs {
    pub(crate) reference: String,
    pub(crate) severity: Severity,
    pub(crate) title: String,
    pub(crate) detail: String,
}

/// `doctrine review raise <RV-NNN> --severity --title --detail [--as raiser]` —
/// append a fresh `open` finding (design §5). Append-only; `raise` is the raiser's
/// and is NOT await-blocked (it may fire even while `await=Responder`, D7/§8).
pub(crate) fn run_raise(
    path: Option<PathBuf>,
    args: &RaiseArgs,
    role: Role,
) -> anyhow::Result<ReviewOutput> {
    let root = resolve_review_root(path)?;
    let id = parse_ref(&args.reference)?;
    let new_id = with_turn(&root, id, Verb::Raise, role, |doc, existing| {
        // Per-finding gate: `raise` targets a fresh (None) finding (design §5).
        if !can(Verb::Raise, None, role) {
            return Err(ReviewError::RoleMismatch {
                expected: Verb::Raise.required_role(),
                actual: role,
                verb: Verb::Raise,
            }
            .into());
        }
        Ok(append_finding(
            doc,
            existing,
            args.severity,
            &args.title,
            &args.detail,
        ))
    })?;
    Ok(ReviewOutput::Raised {
        finding_id: new_id,
        review_id: id,
    })
}

/// Bundled `review dispose` args.
#[derive(Deserialize)]
pub(crate) struct DisposeArgs {
    pub(crate) reference: String,
    pub(crate) finding: String,
    pub(crate) disposition: String,
    pub(crate) response: String,
}

/// `doctrine review dispose <RV-NNN> --finding F-n --disposition --response
/// [--as responder]` — the responder answers a finding (open|contested →
/// answered, design §5). Sets the responder-owned `disposition`/`response`.
pub(crate) fn run_dispose(
    path: Option<PathBuf>,
    args: &DisposeArgs,
    role: Role,
) -> anyhow::Result<ReviewOutput> {
    let root = resolve_review_root(path)?;
    let id = parse_ref(&args.reference)?;
    with_turn(&root, id, Verb::Dispose, role, |doc, existing| {
        let from = finding_status_of(existing, &args.finding)?;
        gate(Verb::Dispose, from, role, &args.finding)?;
        let table = finding_table_mut(doc, &args.finding)?;
        apply_transition(
            table,
            FindingStatus::Answered,
            Some(&args.disposition),
            Some(&args.response),
        );
        Ok(())
    })?;
    Ok(ReviewOutput::Disposed {
        finding_id: args.finding.clone(),
        review_id: id,
    })
}

/// `doctrine review verify <RV-NNN> --finding F-n [--as raiser] [--note …]` — the
/// raiser accepts an answered finding (answered → verified, terminal, design §5).
/// `--note` is ephemeral handoff chatter → the baton log (D10), NOT rationale.
pub(crate) fn run_verify(
    path: Option<PathBuf>,
    reference: &str,
    finding: &str,
    note: Option<&str>,
    role: Role,
) -> anyhow::Result<ReviewOutput> {
    let root = resolve_review_root(path)?;
    let id = parse_ref(reference)?;
    run_raiser_transition(
        &root,
        id,
        Verb::Verify,
        FindingStatus::Verified,
        finding,
        note,
        role,
    )?;
    Ok(ReviewOutput::Verified {
        finding_id: finding.to_owned(),
        review_id: id,
    })
}

/// `doctrine review contest <RV-NNN> --finding F-n [--as raiser] [--note …]` — the
/// raiser rejects an answered finding (answered → contested, design §5), handing
/// it back to the responder. `--note` is ephemeral handoff chatter (D10).
pub(crate) fn run_contest(
    path: Option<PathBuf>,
    reference: &str,
    finding: &str,
    note: Option<&str>,
    role: Role,
) -> anyhow::Result<ReviewOutput> {
    let root = resolve_review_root(path)?;
    let id = parse_ref(reference)?;
    run_raiser_transition(
        &root,
        id,
        Verb::Contest,
        FindingStatus::Contested,
        finding,
        note,
        role,
    )?;
    Ok(ReviewOutput::Contested {
        finding_id: finding.to_owned(),
        review_id: id,
    })
}

/// `doctrine review withdraw <RV-NNN> --finding F-n [--as raiser]` — the raiser
/// retracts a finding (open|answered → withdrawn, terminal, design §5).
pub(crate) fn run_withdraw(
    path: Option<PathBuf>,
    reference: &str,
    finding: &str,
    role: Role,
) -> anyhow::Result<ReviewOutput> {
    let root = resolve_review_root(path)?;
    let id = parse_ref(reference)?;
    run_raiser_transition(
        &root,
        id,
        Verb::Withdraw,
        FindingStatus::Withdrawn,
        finding,
        None,
        role,
    )?;
    Ok(ReviewOutput::Withdrawn {
        finding_id: finding.to_owned(),
        review_id: id,
    })
}

/// The shared shell for the three raiser status-only transitions
/// (verify/contest/withdraw): gate per-finding, apply the status, and route an
/// optional `--note` to the baton's ephemeral handoff log (D10). Disposition /
/// response are responder-owned, so these never touch them.
fn run_raiser_transition(
    root: &Path,
    id: u32,
    verb: Verb,
    to: FindingStatus,
    finding: &str,
    note: Option<&str>,
    role: Role,
) -> anyhow::Result<()> {
    with_turn(root, id, verb, role, |doc, existing| {
        let from = finding_status_of(existing, finding)?;
        gate(verb, from, role, finding)?;
        let table = finding_table_mut(doc, finding)?;
        apply_transition(table, to, None, None);
        Ok(())
    })?;
    // Handoff chatter (D10) — appended to the baton AFTER the turn's baton write,
    // so it survives as the latest baton state (ephemeral, lost on baton loss).
    if let (Some(n), Some(mut baton)) = (note, read_baton(root, id)?) {
        baton.handoff.push(format!("{}: {n}", verb.as_str()));
        write_baton(root, id, &baton)?;
    }
    Ok(())
}

/// The canonical required status for each verb — the state a finding must be
/// in for the verb to act on it. Compound cases pick the first valid status.
fn required_for(verb: Verb) -> FindingStatus {
    match verb {
        Verb::Dispose | Verb::Withdraw | Verb::Raise => FindingStatus::Open,
        Verb::Verify | Verb::Contest => FindingStatus::Answered,
    }
}

/// The per-finding gate (design §6 — the closure's half): refuse an out-of-turn
/// write with a message naming the verb, the finding, and its current state.
fn gate(verb: Verb, from: FindingStatus, role: Role, finding: &str) -> anyhow::Result<()> {
    if !can(verb, Some(from), role) {
        // Role mismatch already caught by `with_turn` step 4; here it is always
        // a state mismatch.
        return Err(ReviewError::StateMismatch {
            finding: finding.to_owned(),
            current: from,
            required: required_for(verb),
        }
        .into());
    }
    Ok(())
}

/// The past-tense label for a verb's success line.
#[expect(
    dead_code,
    reason = "used by print_review in main.rs via pub(crate) export"
)]
pub(crate) fn verb_past(verb: Verb) -> &'static str {
    match verb {
        Verb::Raise => "Raised",
        Verb::Dispose => "Disposed",
        Verb::Verify => "Verified",
        Verb::Contest => "Contested",
        Verb::Withdraw => "Withdrew",
    }
}

// ===========================================================================
// PHASE-05 — the reviewer-context warm-cache (`cache.toml`) + `prime` (design §9,
// D9, D-C10). The cache is the reviewer's *learned* model — runtime, regenerable,
// never authored, DECOUPLED from any LLM token cache (T-b: doctrine makes no
// attempt to observe token-cache warmth). It lives beside the baton/lock in the
// parent tree's gitignored state.
//
// Shape (§9): a curated, load-bearing `domain_map` (`[[area]]` name/purpose/paths,
// T-a — NOT a mechanical read-log), `[[invariant]]`/`[[risk]]` annotations, and a
// `[hashes]` table = the `ContentSet` over `⋃ area.paths` — the staleness baseline.
// Staleness is the pure `stored.diff(compute(parent_root, ⋃ paths))` (T-b naming:
// `current` vs `stale`); it is an optimization SIGNAL surfaced by `status`, never
// a gate. Single parent root; absence⇒stale (R1, in `contentset`).
// ===========================================================================

/// One `[[area]]` of the curated `domain_map` (§9, T-a): a named region of the
/// subject, its purpose, and the load-bearing paths the reviewer must hold in
/// context. The `⋃` of every area's `paths` is what `[hashes]` covers.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
struct CacheArea {
    name: String,
    #[serde(default)]
    purpose: String,
    #[serde(default)]
    paths: Vec<String>,
}

/// A free-text `[[invariant]]` or `[[risk]]` annotation (§9) — a single `text`
/// field. Curated context for the reviewer; carried verbatim, never hashed.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
struct CacheNote {
    text: String,
}

/// The warm-cache document — `cache.toml` (§9). `area`/`invariant`/`risk` are the
/// curated reviewer context; `hashes` is the `ContentSet` baseline over `⋃ area.
/// paths`, the comparison key. `serde` round-trips the lot; `hashes` is rebuilt
/// from `area.paths` on every `prime` so it cannot drift from the `domain_map`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
struct Cache {
    #[serde(default, rename = "area")]
    areas: Vec<CacheArea>,
    #[serde(default, rename = "invariant")]
    invariants: Vec<CacheNote>,
    #[serde(default, rename = "risk")]
    risks: Vec<CacheNote>,
    #[serde(default)]
    hashes: BTreeMap<String, String>,
}

impl Cache {
    /// The de-duplicated, sorted union of every area's paths — the set the
    /// `[hashes]` baseline covers and the set `status` recomputes against.
    fn tracked_paths(&self) -> Vec<String> {
        let mut set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for area in &self.areas {
            for path in &area.paths {
                set.insert(path.clone());
            }
        }
        set.into_iter().collect()
    }

    /// The stored `[hashes]` reconstituted as a `ContentSet` staleness baseline.
    fn baseline(&self) -> ContentSet {
        ContentSet::from_hashes(self.hashes.clone())
    }
}

/// The `cache.toml` path for a review id — beside `baton.toml`/`lock` in the
/// parent tree's gitignored state subtree (§6/§9).
fn cache_path(root: &Path, id: u32) -> PathBuf {
    state_dir(root, id).join("cache.toml")
}

/// Read the warm-cache if present (`None` = unprimed — no staleness signal to
/// report yet, design §9). A parse failure is a hard error (the file is ours).
fn read_cache(root: &Path, id: u32) -> anyhow::Result<Option<Cache>> {
    let path = cache_path(root, id);
    match fs::read_to_string(&path) {
        Ok(text) => Ok(Some(toml::from_str(&text).with_context(|| {
            format!("Failed to parse warm-cache {}", path.display())
        })?)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("Failed to read {}", path.display())),
    }
}

/// Write the warm-cache atomically (temp+rename), creating the state subtree
/// first. The caller holds the per-review lock (§9 — prime serialises its write
/// against a concurrent prime/status).
fn write_cache(root: &Path, id: u32, cache: &Cache) -> anyhow::Result<()> {
    let dir = state_dir(root, id);
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let body = toml::to_string(cache).context("serialize warm-cache")?;
    crate::fsutil::write_atomic(&cache_path(root, id), body.as_bytes())
}

/// The staleness verdict for a primed cache (T-b naming): `current` when the
/// stored `[hashes]` baseline still matches the live `⋃ paths`, else `stale` with
/// the drifted paths listed (changed + removed[absence⇒stale, R1] + added). Pure
/// `diff` over the impure `compute` — the staleness DIFF is pure, `compute` (disk
/// + sha2) is the shell.
fn cache_staleness(root: &Path, cache: &Cache) -> anyhow::Result<CacheVerdict> {
    let live = contentset::compute(root, &cache.tracked_paths())
        .context("hash the warm-cache's tracked paths")?;
    let drift = cache.baseline().diff(&live);
    let mut drifted: Vec<String> = Vec::new();
    drifted.extend(drift.changed);
    drifted.extend(drift.removed);
    drifted.extend(drift.added);
    if drifted.is_empty() {
        Ok(CacheVerdict::Current)
    } else {
        drifted.sort();
        drifted.dedup();
        Ok(CacheVerdict::Stale(drifted))
    }
}

/// The warm-cache staleness verdict (§9, T-b). `Stale` carries the drifted paths.
enum CacheVerdict {
    Current,
    Stale(Vec<String>),
}

// ---------------------------------------------------------------------------
// `review prime` (Read class for authored conduct — no authored mutation — but it
// acquires the per-review lock to serialize the cache write, design §9).
// ---------------------------------------------------------------------------

/// Bundled `review prime` args (the clippy arg-ceiling — `cli-handler-args-struct`).
#[derive(Deserialize)]
pub(crate) struct PrimeArgs {
    pub(crate) reference: String,
    /// `--seed`: emit git-changed candidate paths (a starting point, NOT
    /// authority — it writes nothing) and exit, instead of priming.
    pub(crate) seed: bool,
    /// `--from <file>`: read the curated `domain_map` from a file rather than stdin.
    pub(crate) from: Option<PathBuf>,
}

/// `doctrine review prime <RV-NNN>` — populate the warm-cache from a curated
/// `domain_map` (design §9, T-a). Two modes:
///
/// - `--seed`: emit git-changed candidate paths for the reviewer to curate FROM
///   (a starting point, not authority — writes nothing, takes no lock).
/// - otherwise: read the `domain_map` (TOML: `[[area]]`/`[[invariant]]`/`[[risk]]`)
///   from `--from <file>` or stdin, validate it, hash `⋃ area.paths`
///   (`contentset::compute`), and write `cache.toml`. Read-class for authored
///   conduct (it mutates no authored ledger) but it ACQUIRES THE PER-REVIEW LOCK
///   to serialize the cache write against a concurrent prime/status (§9). It runs
///   neither the baton nor the CAS — only the lock around the cache write.
pub(crate) fn run_prime(path: Option<PathBuf>, args: &PrimeArgs) -> anyhow::Result<ReviewOutput> {
    let root = resolve_review_root(path)?;
    let id = parse_ref(&args.reference)?;
    // The review must exist (the cache is a review's learned model) — fail early
    // with the same "not found" message the verbs give before touching state.
    let _ = read_authored(&root, id)?;

    if args.seed {
        return emit_seed_candidates(&root, id);
    }

    let supplied = if let Some(file) = &args.from {
        fs::read_to_string(file).with_context(|| format!("read domain_map {}", file.display()))?
    } else {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .context("read domain_map from stdin")?;
        buf
    };
    let mut cache: Cache = toml::from_str(&supplied)
        .context("parse the supplied domain_map (expected [[area]]/[[invariant]]/[[risk]] TOML)")?;
    validate_domain_map(&cache)?;

    // Serialize the cache write against a concurrent prime/status (§9). The lock —
    // and ONLY the lock — is reused from PHASE-03; no baton, no CAS.
    let _lock = LockGuard::acquire(&root, id)?;
    // Rebuild `[hashes]` from the curated `⋃ paths` so the baseline cannot drift
    // from the `domain_map` (any value the supplier put under `[hashes]` is ignored).
    let baseline = contentset::compute(&root, &cache.tracked_paths())
        .context("hash the curated domain_map paths")?;
    cache.hashes = baseline.hashes().clone();
    write_cache(&root, id, &cache)?;

    Ok(ReviewOutput::Primed {
        canonical: canonical_id(id),
        tracked_paths: cache.tracked_paths(),
        areas_count: cache.areas.len(),
        tracked_count: cache.tracked_paths().len(),
        invariants_count: cache.invariants.len(),
        risks_count: cache.risks.len(),
        is_seed: false,
    })
}

/// Validate a supplied `domain_map` (§9): at least one area, every area named, every
/// area carrying at least one path, and every path relative (the `ContentSet` is a
/// root-relative `(relpath, hash)` map — an absolute path escapes the parent root).
fn validate_domain_map(cache: &Cache) -> anyhow::Result<()> {
    if cache.areas.is_empty() {
        anyhow::bail!(
            "domain_map has no [[area]] — a primed cache needs at least one curated area"
        );
    }
    for area in &cache.areas {
        if area.name.trim().is_empty() {
            anyhow::bail!("an [[area]] is missing a `name`");
        }
        if area.paths.is_empty() {
            anyhow::bail!(
                "area `{}` has no `paths` — every area must track at least one path",
                area.name
            );
        }
        for path in &area.paths {
            if Path::new(path).is_absolute() || path.contains("..") {
                anyhow::bail!(
                    "area `{}` path `{path}` is not root-relative (no absolute paths or `..`)",
                    area.name
                );
            }
        }
    }
    Ok(())
}

/// `review prime --seed` — emit git-changed candidate paths under the parent root
/// (working-tree + staged changes vs HEAD, plus untracked) as a STARTING POINT for
/// the reviewer to curate from (§9, T-a). It is not authority and writes nothing.
/// Reuses the `git.rs` impure seam (`git_text`); the reviewer pares this down to
/// the load-bearing set.
fn emit_seed_candidates(root: &Path, id: u32) -> anyhow::Result<ReviewOutput> {
    let porcelain = crate::git::git_text(root, &["status", "--porcelain", "--untracked-files=all"])
        .context("git status for prime --seed candidates")?;
    let mut paths: Vec<String> = Vec::new();
    for line in porcelain.lines() {
        // Porcelain v1: 2 status columns then the path. Skip the 2 status bytes
        // and trim the separating space(s) — robust to the ` D `/`D  ` spacing
        // variants. A rename shows `old -> new`; the new path is the candidate.
        let rest = line.get(2..).unwrap_or("").trim();
        let candidate = rest.rsplit(" -> ").next().unwrap_or(rest);
        if !candidate.is_empty() {
            paths.push(candidate.to_owned());
        }
    }
    paths.sort();
    paths.dedup();
    let tracked_count = paths.len();
    Ok(ReviewOutput::Primed {
        canonical: canonical_id(id),
        tracked_paths: paths,
        areas_count: 0,
        tracked_count,
        invariants_count: 0,
        risks_count: 0,
        is_seed: true,
    })
}

// ---------------------------------------------------------------------------
// status (Read class) + unlock (escape hatch)
// ---------------------------------------------------------------------------

/// `doctrine review status <RV-NNN>` — report the derived state and REBUILD the
/// baton (the cache == a fresh recompute, design §8/§Verification). Read-class for
/// authored conduct (no authored mutation), but it acquires the lock to serialize
/// the baton write against a concurrent verb. When a warm-cache is primed, it also
/// reports the cache staleness signal (`current`/`stale`, §9 — a signal, not a gate).
pub(crate) fn run_status(path: Option<PathBuf>, reference: &str) -> anyhow::Result<ReviewOutput> {
    let root = resolve_review_root(path)?;
    let id = parse_ref(reference)?;
    let _lock = LockGuard::acquire(&root, id)?;
    let (text, doc) = read_authored(&root, id)?;
    let hash = crate::git::sha256(text.as_bytes());
    let states = finding_states_of(&doc);
    let (status, awaited) = derived_status(&states);
    let (awaiting, authored_hash) = reconcile_baton_fields(&states, &hash);
    let prior = read_baton(&root, id)?.unwrap_or_default();
    let rebuilt = Baton {
        awaiting,
        authored_hash,
        ..prior
    };
    write_baton(&root, id, &rebuilt)?;

    let mut formatted = format!(
        "{} — {} · await={} · findings {} · rounds {}\n",
        canonical_id(id),
        status.as_str(),
        awaited.as_str(),
        doc.finding.len(),
        rebuilt.rounds
    );

    let mut cache_primed = false;
    let mut stale_paths: Vec<String> = Vec::new();
    if let Some(cache) = read_cache(&root, id)? {
        cache_primed = true;
        match cache_staleness(&root, &cache)? {
            CacheVerdict::Current => {
                formatted.push_str("cache: current\n");
            }
            CacheVerdict::Stale(paths) => {
                let joined = paths.join(", ");
                stale_paths = paths;
                formatted.push_str("cache: stale (");
                formatted.push_str(&joined);
                formatted.push_str(")\n");
            }
        }
    }

    Ok(ReviewOutput::Status {
        canonical: canonical_id(id),
        status: status.as_str().to_owned(),
        awaiting: awaited.as_str().to_owned(),
        findings_count: doc.finding.len(),
        rounds: usize::try_from(rebuilt.rounds).unwrap_or(0),
        cache_primed,
        stale_paths,
        formatted,
    })
}

/// `doctrine review unlock <RV-NNN>` — the escape hatch for a stale lock left by a
/// hard kill (`-9`, which RAII cannot cover, design §6/R-b). Removes the lockfile;
/// its `pid`/`acquired` body aids the operator's "is this really stale?" judgement
/// (printed before removal).
pub(crate) fn run_unlock(path: Option<PathBuf>, reference: &str) -> anyhow::Result<ReviewOutput> {
    let root = resolve_review_root(path)?;
    let id = parse_ref(reference)?;
    let canonical = canonical_id(id);
    let lock = lock_path(&root, id);
    match fs::read_to_string(&lock) {
        Ok(body) => {
            let mut formatted = format!("Removing stale lock for {canonical}:\n");
            for line in body.lines() {
                formatted.push_str("  ");
                formatted.push_str(line);
                formatted.push('\n');
            }
            fs::remove_file(&lock).with_context(|| format!("remove lock {}", lock.display()))?;
            Ok(ReviewOutput::Unlocked {
                canonical,
                formatted,
            })
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(ReviewOutput::Unlocked {
            canonical,
            formatted: String::new(),
        }),
        Err(e) => Err(e).with_context(|| format!("read lock {}", lock.display())),
    }
}

/// Parse a `--as` role token (the cooperative role assertion, design §5 — NOT a
/// security boundary, ADR-007 Negative). Defaults to the verb's required role when
/// omitted, so a single-party drive need not toggle `--as` on every call.
pub(crate) fn parse_role(token: Option<&str>, default: Role) -> anyhow::Result<Role> {
    match token {
        None => Ok(default),
        Some("raiser") => Ok(Role::Raiser),
        Some("responder") => Ok(Role::Responder),
        Some(other) => anyhow::bail!("unknown --as role `{other}` (known: raiser, responder)"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- derived_status: total + named cases (VT-1 / VT-2) -------------------

    fn states(statuses: &[FindingStatus]) -> Vec<FindingState> {
        statuses
            .iter()
            .map(|&status| FindingState { status })
            .collect()
    }

    #[test]
    fn derived_status_empty_is_done_none() {
        assert_eq!(derived_status(&[]), (ReviewStatus::Done, Await::None));
    }

    #[test]
    fn derived_status_any_open_or_contested_is_active_responder() {
        assert_eq!(
            derived_status(&states(&[FindingStatus::Open])),
            (ReviewStatus::Active, Await::Responder)
        );
        assert_eq!(
            derived_status(&states(&[FindingStatus::Contested])),
            (ReviewStatus::Active, Await::Responder)
        );
        // open + answered ⇒ open wins ⇒ Responder.
        assert_eq!(
            derived_status(&states(&[FindingStatus::Answered, FindingStatus::Open])),
            (ReviewStatus::Active, Await::Responder)
        );
    }

    #[test]
    fn derived_status_answered_and_none_open_is_active_raiser() {
        assert_eq!(
            derived_status(&states(&[FindingStatus::Answered])),
            (ReviewStatus::Active, Await::Raiser)
        );
        // answered + a terminal one, none open ⇒ Raiser.
        assert_eq!(
            derived_status(&states(&[FindingStatus::Answered, FindingStatus::Verified])),
            (ReviewStatus::Active, Await::Raiser)
        );
    }

    #[test]
    fn derived_status_all_terminal_is_done_none() {
        assert_eq!(
            derived_status(&states(&[
                FindingStatus::Verified,
                FindingStatus::Withdrawn
            ])),
            (ReviewStatus::Done, Await::None)
        );
        assert_eq!(
            derived_status(&states(&[FindingStatus::Verified])),
            (ReviewStatus::Done, Await::None)
        );
        assert_eq!(
            derived_status(&states(&[FindingStatus::Withdrawn])),
            (ReviewStatus::Done, Await::None)
        );
    }

    /// VT-1: total over the enum — every combination of up to two statuses
    /// yields a `(ReviewStatus, Await)` without panic or gap.
    #[test]
    fn derived_status_total_over_enum() {
        let all = [
            FindingStatus::Open,
            FindingStatus::Answered,
            FindingStatus::Contested,
            FindingStatus::Verified,
            FindingStatus::Withdrawn,
        ];
        // Singletons and every ordered pair.
        for &a in &all {
            let _single = derived_status(&states(&[a]));
            for &b in &all {
                let (status, awaited) = derived_status(&states(&[a, b]));
                // The invariant the carrier must always hold: Done ⇔ None.
                assert_eq!(
                    status == ReviewStatus::Done,
                    awaited == Await::None,
                    "Done iff await=None for [{}, {}]",
                    a.as_str(),
                    b.as_str()
                );
            }
        }
    }

    // -- can(): single-owner edges (VT-3) -----------------------------------

    #[test]
    fn can_valid_single_owner_edges_pass() {
        use FindingStatus::{Answered, Contested, Open};
        assert!(can(Verb::Raise, None, Role::Raiser));
        assert!(can(Verb::Dispose, Some(Open), Role::Responder));
        assert!(can(Verb::Dispose, Some(Contested), Role::Responder));
        assert!(can(Verb::Verify, Some(Answered), Role::Raiser));
        assert!(can(Verb::Contest, Some(Answered), Role::Raiser));
        assert!(can(Verb::Withdraw, Some(Open), Role::Raiser));
        assert!(can(Verb::Withdraw, Some(Answered), Role::Raiser));
    }

    #[test]
    fn can_wrong_role_refused() {
        use FindingStatus::{Answered, Open};
        // dispose is the responder's; the raiser may not.
        assert!(!can(Verb::Dispose, Some(Open), Role::Raiser));
        // verify is the raiser's; the responder may not.
        assert!(!can(Verb::Verify, Some(Answered), Role::Responder));
        // raise is the raiser's.
        assert!(!can(Verb::Raise, None, Role::Responder));
    }

    #[test]
    fn can_wrong_from_state_refused() {
        use FindingStatus::{Answered, Open, Verified, Withdrawn};
        // dispose only from open|contested.
        assert!(!can(Verb::Dispose, Some(Answered), Role::Responder));
        // verify/contest only from answered.
        assert!(!can(Verb::Verify, Some(Open), Role::Raiser));
        assert!(!can(Verb::Contest, Some(Open), Role::Raiser));
        // withdraw only from open|answered, not contested/terminal.
        assert!(!can(
            Verb::Withdraw,
            Some(FindingStatus::Contested),
            Role::Raiser
        ));
        // nothing fires on a terminal finding.
        assert!(!can(Verb::Verify, Some(Verified), Role::Raiser));
        assert!(!can(Verb::Dispose, Some(Withdrawn), Role::Responder));
        // raise requires a fresh finding (None), never an existing one.
        assert!(!can(Verb::Raise, Some(Open), Role::Raiser));
    }

    // -- enum ↔ array drift canaries (VT-4) ---------------------------------

    #[test]
    fn facet_known_set_matches_variants() {
        let from_variants: Vec<&str> = [
            Facet::Scope,
            Facet::Design,
            Facet::Plan,
            Facet::PhasePlan,
            Facet::Implementation,
            Facet::CodeReview,
            Facet::Reconciliation,
        ]
        .iter()
        .map(|f| f.as_str())
        .collect();
        assert_eq!(from_variants, FACETS.to_vec());
        // D-C11: `drift` is NOT a facet.
        assert!(!FACETS.contains(&"drift"));
        assert_eq!(FACETS.len(), 7);
    }

    #[test]
    fn finding_status_known_set_matches_variants() {
        let from_variants: Vec<&str> = [
            FindingStatus::Open,
            FindingStatus::Answered,
            FindingStatus::Contested,
            FindingStatus::Verified,
            FindingStatus::Withdrawn,
        ]
        .iter()
        .map(|s| s.as_str())
        .collect();
        assert_eq!(from_variants, FINDING_STATUSES.to_vec());
    }

    #[test]
    fn severity_known_set_matches_variants() {
        let from_variants: Vec<&str> = [
            Severity::Blocker,
            Severity::Major,
            Severity::Minor,
            Severity::Nit,
        ]
        .iter()
        .map(|s| s.as_str())
        .collect();
        assert_eq!(from_variants, SEVERITIES.to_vec());
    }

    #[test]
    fn role_known_set_matches_variants() {
        let from_variants: Vec<&str> = [Role::Raiser, Role::Responder]
            .iter()
            .map(|r| r.as_str())
            .collect();
        assert_eq!(from_variants, ROLES.to_vec());
    }

    #[test]
    fn review_status_known_set_matches_variants() {
        let from_variants: Vec<&str> = [ReviewStatus::Active, ReviewStatus::Done]
            .iter()
            .map(|s| s.as_str())
            .collect();
        assert_eq!(from_variants, REVIEW_STATUSES.to_vec());
    }

    #[test]
    fn await_str_forms() {
        assert_eq!(Await::Raiser.as_str(), "raiser");
        assert_eq!(Await::Responder.as_str(), "responder");
        assert_eq!(Await::None.as_str(), "none");
    }

    #[test]
    fn verb_str_and_required_role() {
        assert_eq!(Verb::Raise.as_str(), "raise");
        assert_eq!(Verb::Dispose.as_str(), "dispose");
        assert_eq!(Verb::Verify.as_str(), "verify");
        assert_eq!(Verb::Contest.as_str(), "contest");
        assert_eq!(Verb::Withdraw.as_str(), "withdraw");
        // Static verb→role (design §6 responsibility split).
        assert_eq!(Verb::Raise.required_role(), Role::Raiser);
        assert_eq!(Verb::Verify.required_role(), Role::Raiser);
        assert_eq!(Verb::Contest.required_role(), Role::Raiser);
        assert_eq!(Verb::Withdraw.required_role(), Role::Raiser);
        assert_eq!(Verb::Dispose.required_role(), Role::Responder);
    }

    // -- is_terminal mirror -------------------------------------------------

    #[test]
    fn finding_status_terminal_set() {
        assert!(FindingStatus::Verified.is_terminal());
        assert!(FindingStatus::Withdrawn.is_terminal());
        assert!(!FindingStatus::Open.is_terminal());
        assert!(!FindingStatus::Answered.is_terminal());
        assert!(!FindingStatus::Contested.is_terminal());
    }

    // -- render escaping (toml_string splice) -------------------------------

    #[test]
    fn render_finding_escapes_hostile_free_text() {
        let finding = Finding {
            id: "F-1".to_owned(),
            status: FindingStatus::Open,
            severity: Severity::Major,
            // A hostile title: a quote, a backslash, a newline, and a `]`.
            title: "a\"b\\c\nd]e".to_owned(),
            detail: "plain".to_owned(),
            disposition: None,
            response: None,
        };
        let rendered = render_finding(&finding);
        // The rendered block must parse back as valid TOML with the value intact
        // — proof the splice did not break the document or inject a key.
        let parsed: toml::Value = toml::from_str(&rendered).unwrap();
        let finding_tbl = parsed["finding"].as_array().unwrap()[0].as_table().unwrap();
        assert_eq!(finding_tbl["title"].as_str().unwrap(), "a\"b\\c\nd]e");
        assert_eq!(finding_tbl["id"].as_str().unwrap(), "F-1");
        assert_eq!(finding_tbl["status"].as_str().unwrap(), "open");
        assert_eq!(finding_tbl["severity"].as_str().unwrap(), "major");
    }

    #[test]
    fn render_finding_emits_responder_fields_when_present() {
        let finding = Finding {
            id: "F-2".to_owned(),
            status: FindingStatus::Answered,
            severity: Severity::Nit,
            title: "t".to_owned(),
            detail: "d".to_owned(),
            disposition: Some("fixed".to_owned()),
            response: Some("done in r\"123".to_owned()),
        };
        let rendered = render_finding(&finding);
        let parsed: toml::Value = toml::from_str(&rendered).unwrap();
        let tbl = parsed["finding"].as_array().unwrap()[0].as_table().unwrap();
        assert_eq!(tbl["disposition"].as_str().unwrap(), "fixed");
        assert_eq!(tbl["response"].as_str().unwrap(), "done in r\"123");
    }

    // -- impure shell (PHASE-02): new / show / list --------------------------

    use std::path::Path;

    /// Plant a minimal slice dir so `SL-001` is a resolvable target ref. Just the
    /// numeric dir + sister toml — enough for `ensure_ref_resolves` (a dir probe).
    fn plant_slice_target(root: &Path, id: u32) {
        let name = format!("{id:03}");
        let dir = root.join(".doctrine/slice").join(&name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("slice-{name}.toml")),
            format!("id = {id}\nslug = \"t\"\ntitle = \"T\"\nstatus = \"proposed\"\n"),
        )
        .unwrap();
    }

    fn meta(facet: &str) -> ReviewMeta {
        ReviewMeta {
            facet: facet.to_owned(),
            raiser: "rev".to_owned(),
            responder: "auth".to_owned(),
        }
    }

    /// PHASE-02: the rendered ledger toml round-trips into `ReviewDoc`, carries NO
    /// stored status (D-C8), and an optional `[target].phase` is present/absent
    /// exactly as supplied.
    #[test]
    fn render_review_toml_round_trips_without_a_stored_status() {
        let target = Target {
            reference: "SL-024".to_owned(),
            phase: Some("PHASE-03".to_owned()),
        };
        let body = render_review_toml(
            7,
            "design-review",
            "Design review",
            &meta("design"),
            &target,
        )
        .unwrap();
        // No stored status — the storage rule forbids derived data (D-C8).
        let value: toml::Value = toml::from_str(&body).unwrap();
        assert!(
            value.get("status").is_none(),
            "ledger stores no status: {body}"
        );
        let doc: ReviewDoc = toml::from_str(&body).unwrap();
        assert_eq!(doc.id, 7);
        assert_eq!(doc.review.facet, "design");
        assert_eq!(doc.target.reference, "SL-024");
        assert_eq!(doc.target.phase.as_deref(), Some("PHASE-03"));
        assert!(doc.finding.is_empty(), "fresh ledger has no findings");
    }

    /// Render-splice escaping: a hostile title round-trips intact through
    /// `toml_string` (mem.pattern.render.toml-splice-escape-user-values).
    #[test]
    fn render_review_toml_escapes_a_hostile_title() {
        let target = Target {
            reference: "SL-001".to_owned(),
            phase: None,
        };
        let hostile = "a\"b\\c\nd]e";
        let body = render_review_toml(1, "s", hostile, &meta("scope"), &target).unwrap();
        let doc: ReviewDoc = toml::from_str(&body).unwrap();
        assert_eq!(doc.title, hostile);
        // phase absent ⇒ no phase key.
        assert!(doc.target.phase.is_none());
    }

    /// VT-4 (show): a fresh empty-ledger RV renders derived status `active` with
    /// done status, await `none`, and the `reviews` edge to the target.
    #[test]
    fn show_renders_empty_ledger_done_and_the_edge() {
        let doc = ReviewDoc {
            id: 3,
            slug: "s".to_owned(),
            title: "Design review of SL-024".to_owned(),
            review: meta("design"),
            target: Target {
                reference: "SL-024".to_owned(),
                phase: None,
            },
            finding: Vec::new(),
        };
        let out = format_show(&doc, "## Brief\n");
        assert!(out.contains("RV-003 — Design review of SL-024"), "{out}");
        // empty ⇒ Done, await=None.
        assert!(out.contains("done · await=none"), "{out}");
        assert!(out.contains("RV-003 ──reviews──▶ SL-024"), "edge: {out}");
        assert!(out.contains("findings: 0"), "{out}");
    }

    /// VT-4 (list): the empty-ledger RV lists with derived status `done`
    /// (await none), facet, and the target edge — no stored status read.
    #[test]
    fn list_renders_empty_ledger_done_and_the_edge() {
        let doc = ReviewDoc {
            id: 5,
            slug: "s".to_owned(),
            title: "Plan review".to_owned(),
            review: meta("plan"),
            target: Target {
                reference: "SL-009".to_owned(),
                phase: Some("PHASE-02".to_owned()),
            },
            finding: Vec::new(),
        };
        let (status, awaited) = doc.derived();
        let rows = vec![(doc, status, awaited)];
        let sel = listing::select_columns(&REVIEW_COLUMNS, REVIEW_DEFAULT, None).unwrap();
        let out = listing::render_columns(&rows, &sel, listing::RenderOpts::default());
        let lines: Vec<&str> = out.lines().collect();
        assert!(lines[0].starts_with("id"), "header: {:?}", lines[0]);
        assert!(lines[1].starts_with("RV-005"), "{:?}", lines[1]);
        assert!(lines[1].contains("done (await none)"), "{:?}", lines[1]);
        assert!(lines[1].contains("plan"), "{:?}", lines[1]);
        // phase-scoped edge `SL-009@PHASE-02`.
        assert!(lines[1].contains("SL-009@PHASE-02"), "{:?}", lines[1]);
    }

    /// Derived status reflects a non-terminal finding: an `open` finding keeps the
    /// review Active awaiting the Responder (D-C8) — read straight from authored
    /// findings, never a stored status.
    #[test]
    fn derived_status_reads_findings_not_a_stored_status() {
        let mut doc = ReviewDoc {
            id: 1,
            slug: "s".to_owned(),
            title: "t".to_owned(),
            review: meta("design"),
            target: Target {
                reference: "SL-001".to_owned(),
                phase: None,
            },
            finding: vec![FindingRow {
                id: "F-1".to_owned(),
                status: "open".to_owned(),
                severity: "major".to_owned(),
                title: "t".to_owned(),
                detail: "d".to_owned(),
                disposition: None,
                response: None,
            }],
        };
        assert_eq!(doc.derived(), (ReviewStatus::Active, Await::Responder));
        // all-terminal ⇒ Done.
        doc.finding[0].status = "verified".to_owned();
        assert_eq!(doc.derived(), (ReviewStatus::Done, Await::None));
    }

    // -- run_new end-to-end (VT-2: dangling ref refused at creation) ----------

    fn new_args(facet: Facet, target: &str) -> NewArgs {
        NewArgs {
            facet,
            target: target.to_owned(),
            phase: None,
            title: None,
            raiser: None,
            responder: None,
        }
    }

    /// `review new` mints an RV with an empty ledger + seeded `## Brief`, and the
    /// ledger round-trips through the real readers (Active/Raiser, the edge).
    #[test]
    fn run_new_creates_an_empty_ledger_rv_against_a_real_target() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        plant_slice_target(root, 24);
        run_new(Some(root.to_path_buf()), &new_args(Facet::Design, "SL-024")).unwrap();

        let review_root = root.join(REVIEW_DIR);
        let doc = read_review(&review_root, 1).unwrap();
        assert_eq!(doc.id, 1);
        assert_eq!(doc.target.reference, "SL-024");
        assert!(doc.finding.is_empty());
        assert_eq!(doc.derived(), (ReviewStatus::Done, Await::None));
        let brief = read_brief(&review_root, 1).unwrap();
        assert!(brief.contains("## Brief"), "brief seeded: {brief}");
        // The `NNN-slug` alias symlink landed.
        assert!(
            std::fs::symlink_metadata(review_root.join("001-design-review-of-sl-024"))
                .map(|m| m.file_type().is_symlink())
                .unwrap_or(false),
            "alias symlink planted"
        );
    }

    /// VT-2: a dangling `[target].ref` (well-formed but no entity) is refused at
    /// creation — and no RV directory is minted (§7).
    #[test]
    fn run_new_refuses_a_dangling_target_and_mints_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // SL-099 has no entity dir.
        let err =
            run_new(Some(root.to_path_buf()), &new_args(Facet::Design, "SL-099")).unwrap_err();
        assert!(
            err.to_string().contains("does not resolve"),
            "dangling ref refused: {err}"
        );
        assert!(
            entity::scan_ids(&root.join(REVIEW_DIR)).unwrap().is_empty(),
            "no RV minted on a refused target"
        );
    }

    /// VT-2: an unknown-prefix target is refused at creation (§7).
    #[test]
    fn run_new_refuses_an_unknown_prefix_target() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let err = run_new(Some(root.to_path_buf()), &new_args(Facet::Scope, "ZZ-001")).unwrap_err();
        assert!(
            err.to_string().contains("unknown kind prefix"),
            "unknown prefix refused: {err}"
        );
    }

    /// `Facet::parse` accepts the closed 7-set and rejects `drift` (D-C11) /
    /// garbage with a helpful message.
    #[test]
    fn facet_parse_accepts_the_seven_and_rejects_drift() {
        assert_eq!(Facet::parse("phase-plan").unwrap(), Facet::PhasePlan);
        assert_eq!(Facet::parse("code-review").unwrap(), Facet::CodeReview);
        assert!(
            Facet::parse("drift").is_err(),
            "drift is not a facet (D-C11)"
        );
        let err = Facet::parse("bogus").unwrap_err();
        assert!(err.contains("unknown facet"), "{err}");
    }

    // =====================================================================
    // PHASE-03 — verb family + the turn guard (VT-1..10)
    // =====================================================================

    /// Stand up a fresh RV (id 1) targeting a planted SL-001, in a tempdir whose
    /// root is not a git tree (the fork guard's `is_linked_worktree` returns Err
    /// ⇒ treated not-a-fork ⇒ proceeds). Returns the root.
    fn fixture_rv() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        plant_slice_target(root, 1);
        run_new(Some(root.to_path_buf()), &new_args(Facet::Design, "SL-001")).unwrap();
        tmp
    }

    fn raise_args(reference: &str, sev: Severity, title: &str) -> RaiseArgs {
        RaiseArgs {
            reference: reference.to_owned(),
            severity: sev,
            title: title.to_owned(),
            detail: "d".to_owned(),
        }
    }

    fn dispose_args(reference: &str, finding: &str) -> DisposeArgs {
        DisposeArgs {
            reference: reference.to_owned(),
            finding: finding.to_owned(),
            disposition: "fixed".to_owned(),
            response: "done".to_owned(),
        }
    }

    fn read_doc(root: &Path, id: u32) -> ReviewDoc {
        read_review(&root.join(REVIEW_DIR), id).unwrap()
    }

    /// A full raise→dispose→verify lifecycle drives the finding through its
    /// states; the ledger reflects each transition and the baton tracks `await`.
    #[test]
    fn lifecycle_raise_dispose_verify() {
        let tmp = fixture_rv();
        let root = tmp.path();
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Major, "t"),
            Role::Raiser,
        )
        .unwrap();
        // After a raise: one open finding, await=Responder.
        let doc = read_doc(root, 1);
        assert_eq!(doc.finding.len(), 1);
        assert_eq!(doc.finding[0].id, "F-1");
        assert_eq!(doc.finding[0].status, "open");
        assert_eq!(read_baton(root, 1).unwrap().unwrap().awaiting, "responder");

        run_dispose(
            Some(root.to_path_buf()),
            &dispose_args("RV-001", "F-1"),
            Role::Responder,
        )
        .unwrap();
        let doc = read_doc(root, 1);
        assert_eq!(doc.finding[0].status, "answered");
        assert_eq!(doc.finding[0].disposition.as_deref(), Some("fixed"));
        assert_eq!(doc.finding[0].response.as_deref(), Some("done"));
        assert_eq!(read_baton(root, 1).unwrap().unwrap().awaiting, "raiser");

        run_verify(
            Some(root.to_path_buf()),
            "RV-001",
            "F-1",
            None,
            Role::Raiser,
        )
        .unwrap();
        let doc = read_doc(root, 1);
        assert_eq!(doc.finding[0].status, "verified");
        // All-terminal ⇒ Done / await=none.
        assert_eq!(read_baton(root, 1).unwrap().unwrap().awaiting, "none");
    }

    /// VT-1: field ownership disjoint — raiser fields (id/title/detail/severity)
    /// are fixed at raise; a dispose mutates ONLY the responder pair + status.
    #[test]
    fn vt1_raiser_fields_immutable_responder_fields_mutable() {
        let tmp = fixture_rv();
        let root = tmp.path();
        run_raise(
            Some(root.to_path_buf()),
            &RaiseArgs {
                reference: "RV-001".to_owned(),
                severity: Severity::Blocker,
                title: "orig-title".to_owned(),
                detail: "orig-detail".to_owned(),
            },
            Role::Raiser,
        )
        .unwrap();
        run_dispose(
            Some(root.to_path_buf()),
            &dispose_args("RV-001", "F-1"),
            Role::Responder,
        )
        .unwrap();
        let f = &read_doc(root, 1).finding[0];
        // Raiser-owned: unchanged by the responder's turn.
        assert_eq!(f.id, "F-1");
        assert_eq!(f.title, "orig-title");
        assert_eq!(f.detail, "orig-detail");
        assert_eq!(f.severity, "blocker");
        // Responder-owned: set by dispose.
        assert_eq!(f.disposition.as_deref(), Some("fixed"));
        assert_eq!(f.response.as_deref(), Some("done"));
        // Status moved on a single-owner edge.
        assert_eq!(f.status, "answered");
    }

    /// VT-2: finding ids are append-only `F-<max+1>` — never reused, even with a
    /// gap. Three raises land F-1, F-2, F-3.
    #[test]
    fn vt2_finding_ids_are_append_only() {
        let tmp = fixture_rv();
        let root = tmp.path();
        for n in ["a", "b", "c"] {
            run_raise(
                Some(root.to_path_buf()),
                &raise_args("RV-001", Severity::Minor, n),
                Role::Raiser,
            )
            .unwrap();
        }
        let ids: Vec<String> = read_doc(root, 1)
            .finding
            .iter()
            .map(|f| f.id.clone())
            .collect();
        assert_eq!(ids, ["F-1", "F-2", "F-3"]);
        // The pure id allocator: max+1 over existing, robust to a gap.
        let rows = vec![FindingRow {
            id: "F-7".to_owned(),
            status: "open".to_owned(),
            severity: "nit".to_owned(),
            title: "t".to_owned(),
            detail: "d".to_owned(),
            disposition: None,
            response: None,
        }];
        assert_eq!(next_finding_id(&rows), "F-8");
        assert_eq!(next_finding_id(&[]), "F-1");
    }

    /// VT-3: transitions are edit-preserving — a hand-added comment and an
    /// unknown key survive a dispose (the governance.rs:290 contract at finding
    /// scope).
    #[test]
    fn vt3_transitions_are_edit_preserving() {
        let tmp = fixture_rv();
        let root = tmp.path();
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Major, "t"),
            Role::Raiser,
        )
        .unwrap();
        // Hand-add a comment + an unknown top-level key.
        let path = authored_path(root, 1);
        let mut text = fs::read_to_string(&path).unwrap();
        text.push_str("\n# a hand comment\nunknown_key = \"keepme\"\n");
        fs::write(&path, &text).unwrap();
        // The hand-edit changed the bytes — refresh the baton so the entry CAS
        // does not (correctly) abort the next turn on the edit it now reflects.
        run_status(Some(root.to_path_buf()), "RV-001").unwrap();

        run_dispose(
            Some(root.to_path_buf()),
            &dispose_args("RV-001", "F-1"),
            Role::Responder,
        )
        .unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert!(
            after.contains("# a hand comment"),
            "comment survived: {after}"
        );
        assert!(
            after.contains("unknown_key = \"keepme\""),
            "unknown key survived: {after}"
        );
        assert_eq!(read_doc(root, 1).finding[0].status, "answered");
    }

    /// VT-4: render escaping — a hostile title/detail raised then read back
    /// round-trips intact (toml_edit::value quotes/escapes the structured write,
    /// the splice twin of the render path).
    #[test]
    fn vt4_hostile_free_text_round_trips() {
        let tmp = fixture_rv();
        let root = tmp.path();
        let hostile = "a\"b\\c\nd]e";
        run_raise(
            Some(root.to_path_buf()),
            &RaiseArgs {
                reference: "RV-001".to_owned(),
                severity: Severity::Major,
                title: hostile.to_owned(),
                detail: hostile.to_owned(),
            },
            Role::Raiser,
        )
        .unwrap();
        // The ledger is still valid TOML and the value is intact.
        let f = &read_doc(root, 1).finding[0];
        assert_eq!(f.title, hostile);
        assert_eq!(f.detail, hostile);
    }

    /// VT-5(a) + VT-8: two ordered invocations — a lock held by a concurrent
    /// invocation makes the second BAIL (busy), no clobber; after the first
    /// completes the loser re-runs from the refreshed baton and lands a correct
    /// turn. Also asserts `raise` is allowed while await=Responder.
    #[test]
    fn vt5a_lock_serializes_loser_bails_then_re_runs() {
        let tmp = fixture_rv();
        let root = tmp.path();
        // Manually hold the lock (simulating a concurrent invocation in flight).
        let held = LockGuard::acquire(root, 1).unwrap();
        // A second invocation loses the create_new race → clean "busy" bail.
        let err = run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Major, "t"),
            Role::Raiser,
        )
        .unwrap_err();
        assert!(err.to_string().contains("busy"), "loser bailed busy: {err}");
        // Nothing was written — the ledger is untouched.
        assert!(
            read_doc(root, 1).finding.is_empty(),
            "no clobber on a lost lock"
        );
        // The first invocation completes (lock released).
        drop(held);
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Major, "first"),
            Role::Raiser,
        )
        .unwrap();
        // The loser re-runs — and `raise` is allowed even while await=Responder
        // (one open finding ⇒ Responder), landing F-2 (VT-8 raise-not-blocked).
        assert_eq!(read_baton(root, 1).unwrap().unwrap().awaiting, "responder");
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Major, "second"),
            Role::Raiser,
        )
        .unwrap();
        let ids: Vec<String> = read_doc(root, 1)
            .finding
            .iter()
            .map(|f| f.id.clone())
            .collect();
        assert_eq!(ids, ["F-1", "F-2"]);
    }

    /// VT-5(b) + VT-6 + VT-7: a crash between the authored write (step 5) and the
    /// baton write (step 7) leaves the authored ledger ahead of the baton hash;
    /// the NEXT invocation's entry CAS detects it, heals the baton, and bails
    /// "re-run". Simulated by mutating the authored ledger directly (a real
    /// authored-write that the baton never caught up to) then driving a verb.
    #[test]
    fn vt5b_entry_cas_self_heals_a_crash_between_writes() {
        let tmp = fixture_rv();
        let root = tmp.path();
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Major, "t"),
            Role::Raiser,
        )
        .unwrap();
        let baton_before = read_baton(root, 1).unwrap().unwrap();

        // Simulate a crash AFTER the authored write but BEFORE the baton write:
        // the authored ledger gains an edit the baton's hash does not reflect.
        let path = authored_path(root, 1);
        let mut text = fs::read_to_string(&path).unwrap();
        text = text.replace("status = \"open\"", "status = \"answered\"");
        fs::write(&path, &text).unwrap();

        // The next invocation's ENTRY CAS catches the divergence, refreshes the
        // baton from the authored truth, and bails.
        let err = run_dispose(
            Some(root.to_path_buf()),
            &dispose_args("RV-001", "F-1"),
            Role::Responder,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("changed underneath"),
            "entry CAS bailed: {err}"
        );
        // The baton self-healed: its hash now matches the authored bytes, and the
        // await recomputed from the answered finding (Raiser).
        let healed = read_baton(root, 1).unwrap().unwrap();
        assert_ne!(
            healed.authored_hash, baton_before.authored_hash,
            "hash refreshed"
        );
        assert_eq!(
            healed.authored_hash,
            crate::git::sha256(fs::read_to_string(&path).unwrap().as_bytes())
        );
        assert_eq!(
            healed.awaiting, "raiser",
            "await recomputed from authored truth"
        );
        // The ledger was NOT clobbered — the aborted dispose wrote nothing.
        assert_eq!(read_doc(root, 1).finding[0].status, "answered");
    }

    /// VT-5(c) + VT-6: a hand-edit landing AFTER the step-2 read but BEFORE the
    /// step-5 write (the pre-write CAS window) aborts the turn with NO write — the
    /// stale in-memory DocumentMut cannot overwrite the newer authored truth. The
    /// mid-turn hook fires the edit deterministically (no threads).
    #[test]
    fn vt5c_pre_write_cas_aborts_a_mid_turn_edit() {
        let tmp = fixture_rv();
        let root = tmp.path();
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Major, "t"),
            Role::Raiser,
        )
        .unwrap();
        let path = authored_path(root, 1);

        // Drive a dispose under the hooked seam: the hook lands a hand-edit
        // (a second finding) between the in-memory mutation and the write.
        let hook = || {
            let mut text = fs::read_to_string(&path).unwrap();
            text.push_str(
                "\n[[finding]]\nid = \"F-2\"\nstatus = \"open\"\nseverity = \"nit\"\n\
                 title = \"injected\"\ndetail = \"by hand\"\n",
            );
            fs::write(&path, &text).unwrap();
        };
        let err = with_turn_hooked(
            root,
            1,
            Verb::Dispose,
            Role::Responder,
            &hook,
            |doc, existing| {
                let from = finding_status_of(existing, "F-1")?;
                gate(Verb::Dispose, from, Role::Responder, "F-1")?;
                let table = finding_table_mut(doc, "F-1")?;
                apply_transition(table, FindingStatus::Answered, Some("fixed"), Some("done"));
                Ok(())
            },
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("changed underneath this turn"),
            "pre-write CAS aborted: {err}"
        );
        // NO clobber: F-1 is still `open` (the dispose never wrote), and the
        // injected F-2 survives (the abort wrote nothing over it).
        let doc = read_doc(root, 1);
        assert_eq!(doc.finding.len(), 2, "injected finding survived");
        assert_eq!(
            doc.finding[0].status, "open",
            "F-1 not clobbered to answered"
        );
        assert_eq!(doc.finding[1].id, "F-2");
    }

    /// VT-5(d): the same finding contested then verified — once a verify makes the
    /// finding terminal, a contest can no longer fire on it (the per-finding gate
    /// is the lost-update guard at finding granularity). Drives the two verbs in
    /// order and asserts the FINAL ledger reflects only the winner.
    #[test]
    fn vt5d_same_finding_contest_racing_verify() {
        let tmp = fixture_rv();
        let root = tmp.path();
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Major, "t"),
            Role::Raiser,
        )
        .unwrap();
        run_dispose(
            Some(root.to_path_buf()),
            &dispose_args("RV-001", "F-1"),
            Role::Responder,
        )
        .unwrap();
        // Verify wins first → terminal.
        run_verify(
            Some(root.to_path_buf()),
            "RV-001",
            "F-1",
            None,
            Role::Raiser,
        )
        .unwrap();
        assert_eq!(read_doc(root, 1).finding[0].status, "verified");
        // The racing contest now finds the finding terminal → per-finding gate
        // refuses; the ledger is untouched (no double-apply).
        let err = run_contest(
            Some(root.to_path_buf()),
            "RV-001",
            "F-1",
            None,
            Role::Raiser,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("out of turn"),
            "contest gated: {err}"
        );
        assert_eq!(
            read_doc(root, 1).finding[0].status,
            "verified",
            "winner stands"
        );
    }

    /// VT-8: an out-of-turn write is refused by the static role check AND the
    /// per-finding gate.
    #[test]
    fn vt8_out_of_turn_refused() {
        let tmp = fixture_rv();
        let root = tmp.path();
        // Static role: dispose asserted --as raiser ⇒ refused.
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Major, "t"),
            Role::Raiser,
        )
        .unwrap();
        let err = run_dispose(
            Some(root.to_path_buf()),
            &dispose_args("RV-001", "F-1"),
            Role::Raiser, // wrong role
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("responder's verb"),
            "static role: {err}"
        );
        // Per-finding state: verify an open (not answered) finding ⇒ refused.
        let err = run_verify(
            Some(root.to_path_buf()),
            "RV-001",
            "F-1",
            None,
            Role::Raiser,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("out of turn"),
            "per-finding gate: {err}"
        );
        // Nothing moved.
        assert_eq!(read_doc(root, 1).finding[0].status, "open");
    }

    /// VT-9: `status` rebuilds the baton — the cached await equals a fresh
    /// recompute, even after the baton was deleted (cold) or stale.
    #[test]
    fn vt9_status_rebuilds_the_baton() {
        let tmp = fixture_rv();
        let root = tmp.path();
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Major, "t"),
            Role::Raiser,
        )
        .unwrap();
        // Delete the baton (cold) — status must rebuild it == recompute.
        fs::remove_file(baton_path(root, 1)).unwrap();
        run_status(Some(root.to_path_buf()), "RV-001").unwrap();
        let baton = read_baton(root, 1).unwrap().unwrap();
        let doc = read_doc(root, 1);
        let (_, awaited) = derived_status(&finding_states_of(&doc));
        assert_eq!(baton.awaiting, awaited.as_str(), "cache == recompute");
        assert_eq!(
            baton.authored_hash,
            crate::git::sha256(
                fs::read_to_string(authored_path(root, 1))
                    .unwrap()
                    .as_bytes()
            )
        );
    }

    /// VT-10: a review verb on a fork-resolved root bails (IMP-024 guard), and the
    /// baton/lock sit in the gitignored parent state tree. Builds a real linked
    /// worktree to exercise `is_linked_worktree`.
    #[test]
    fn vt10_fork_root_refused_and_baton_in_parent_state() {
        use std::process::Command;
        let tmp = tempfile::tempdir().unwrap();
        let main = tmp.path().join("main");
        std::fs::create_dir_all(&main).unwrap();
        let git = |dir: &Path, args: &[&str]| {
            let ok = Command::new("git")
                .arg("-C")
                .arg(dir)
                .args(args)
                .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00 +0000")
                .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00 +0000")
                .output()
                .unwrap();
            assert!(
                ok.status.success(),
                "git {args:?}: {}",
                String::from_utf8_lossy(&ok.stderr)
            );
        };
        git(&main, &["init", "-b", "main"]);
        git(&main, &["config", "user.name", "T"]);
        git(&main, &["config", "user.email", "t@t.invalid"]);
        plant_slice_target(&main, 1);
        run_new(Some(main.clone()), &new_args(Facet::Design, "SL-001")).unwrap();
        std::fs::write(main.join("seed"), "x").unwrap();
        git(&main, &["add", "."]);
        git(&main, &["commit", "-m", "seed"]);
        // Add a linked worktree (the fork).
        let fork = tmp.path().join("fork");
        git(&main, &["worktree", "add", fork.to_str().unwrap()]);

        // A verb resolved at the fork root bails (IMP-024).
        let err = run_raise(
            Some(fork.clone()),
            &raise_args("RV-001", Severity::Major, "t"),
            Role::Raiser,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("worktree fork"),
            "fork guard: {err}"
        );

        // A verb on the parent tree works, and the baton lands under the parent's
        // gitignored .doctrine/state/review/ (never the fork).
        run_raise(
            Some(main.clone()),
            &raise_args("RV-001", Severity::Major, "t"),
            Role::Raiser,
        )
        .unwrap();
        assert!(
            main.join(".doctrine/state/review/001/baton.toml").is_file(),
            "baton in parent state"
        );
        assert!(
            !fork.join(".doctrine/state/review/001/baton.toml").exists(),
            "no baton in the fork"
        );
    }

    /// The `--as` role assertion parses cooperatively and defaults to the verb's
    /// required role.
    #[test]
    fn parse_role_defaults_and_validates() {
        assert_eq!(parse_role(None, Role::Responder).unwrap(), Role::Responder);
        assert_eq!(
            parse_role(Some("raiser"), Role::Responder).unwrap(),
            Role::Raiser
        );
        assert!(parse_role(Some("bogus"), Role::Raiser).is_err());
    }

    /// A `--note` on verify/contest is ephemeral handoff chatter → the baton log
    /// (D10), NOT a ledger field (durable rationale promotes to a finding).
    #[test]
    fn note_is_handoff_chatter_in_the_baton_not_the_ledger() {
        let tmp = fixture_rv();
        let root = tmp.path();
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Major, "t"),
            Role::Raiser,
        )
        .unwrap();
        run_dispose(
            Some(root.to_path_buf()),
            &dispose_args("RV-001", "F-1"),
            Role::Responder,
        )
        .unwrap();
        run_contest(
            Some(root.to_path_buf()),
            "RV-001",
            "F-1",
            Some("please address the edge case"),
            Role::Raiser,
        )
        .unwrap();
        let baton = read_baton(root, 1).unwrap().unwrap();
        assert!(
            baton
                .handoff
                .iter()
                .any(|h| h.contains("please address the edge case")),
            "note in baton handoff log: {:?}",
            baton.handoff
        );
        assert_eq!(baton.contests, 1, "contest counter bumped");
        // The note is NOT in the authored ledger.
        let text = fs::read_to_string(authored_path(root, 1)).unwrap();
        assert!(
            !text.contains("please address the edge case"),
            "note not durable: {text}"
        );
    }

    // ---- PHASE-04: reverse close-gate scan (design §7, D8/D-C9b) ----

    /// VT-3 / VT-1: an Active RV with a raised (open) **blocker** finding is
    /// reported by the scan, keyed `RV-NNN`/`F-n`, and matched by `[target].ref`.
    #[test]
    fn vt3_scan_reports_an_unresolved_blocker_on_an_active_rv() {
        let tmp = fixture_rv(); // RV-001 → SL-001
        let root = tmp.path();
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Blocker, "must fix"),
            Role::Raiser,
        )
        .unwrap();
        // Active (one open finding) + blocker ⇒ one BlockerRef RV-001/F-1.
        assert_eq!(read_doc(root, 1).derived().0, ReviewStatus::Active);
        let blockers = unresolved_blockers_for(root, "SL-001").unwrap();
        assert_eq!(
            blockers,
            vec![BlockerRef {
                rv: "RV-001".to_owned(),
                finding: "F-1".to_owned(),
            }]
        );
    }

    /// VT-3: a non-matching `[target].ref` is ignored — the scan is subject-scoped.
    #[test]
    fn vt3_scan_ignores_a_non_matching_target() {
        let tmp = fixture_rv();
        let root = tmp.path();
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Blocker, "must fix"),
            Role::Raiser,
        )
        .unwrap();
        // RV-001 targets SL-001; a query for an unrelated subject finds nothing.
        assert!(unresolved_blockers_for(root, "SL-999").unwrap().is_empty());
    }

    /// VT-3: a non-blocker finding (major/minor/nit) never gates — only `blocker`.
    #[test]
    fn vt3_scan_ignores_a_non_blocker_finding() {
        let tmp = fixture_rv();
        let root = tmp.path();
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Major, "nice to have"),
            Role::Raiser,
        )
        .unwrap();
        assert!(unresolved_blockers_for(root, "SL-001").unwrap().is_empty());
    }

    /// VT-1 / VT-3: a **verified** blocker is terminal ⇒ the RV is Done ⇒ the scan
    /// reports nothing (the finding is resolved AND the review is no longer Active).
    #[test]
    fn vt1_verified_blocker_is_terminal_and_not_reported() {
        let tmp = fixture_rv();
        let root = tmp.path();
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Blocker, "must fix"),
            Role::Raiser,
        )
        .unwrap();
        run_dispose(
            Some(root.to_path_buf()),
            &dispose_args("RV-001", "F-1"),
            Role::Responder,
        )
        .unwrap();
        run_verify(
            Some(root.to_path_buf()),
            "RV-001",
            "F-1",
            None,
            Role::Raiser,
        )
        .unwrap();
        // All findings terminal ⇒ Done (D-C9a) ⇒ no unresolved blocker.
        assert_eq!(read_doc(root, 1).derived().0, ReviewStatus::Done);
        assert!(unresolved_blockers_for(root, "SL-001").unwrap().is_empty());
    }

    /// VT-1 / VT-3: a **withdrawn** blocker is terminal ⇒ Done ⇒ not reported.
    #[test]
    fn vt1_withdrawn_blocker_is_terminal_and_not_reported() {
        let tmp = fixture_rv();
        let root = tmp.path();
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Blocker, "must fix"),
            Role::Raiser,
        )
        .unwrap();
        run_withdraw(Some(root.to_path_buf()), "RV-001", "F-1", Role::Raiser).unwrap();
        assert_eq!(read_doc(root, 1).derived().0, ReviewStatus::Done);
        assert!(unresolved_blockers_for(root, "SL-001").unwrap().is_empty());
    }

    /// VT-1: a **non-terminal** blocker keeps the review Active and gating — a
    /// blocker disposed-but-not-yet-verified (answered) still gates (D-C9a: review
    /// is done only when EVERY finding is terminal ∈ {verified, withdrawn}).
    #[test]
    fn vt1_answered_blocker_keeps_the_review_active_and_gating() {
        let tmp = fixture_rv();
        let root = tmp.path();
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Blocker, "must fix"),
            Role::Raiser,
        )
        .unwrap();
        run_dispose(
            Some(root.to_path_buf()),
            &dispose_args("RV-001", "F-1"),
            Role::Responder,
        )
        .unwrap();
        // answered ∉ {verified, withdrawn} ⇒ Active ⇒ still an unresolved blocker.
        assert_eq!(read_doc(root, 1).derived().0, ReviewStatus::Active);
        let blockers = unresolved_blockers_for(root, "SL-001").unwrap();
        assert_eq!(blockers.len(), 1);
        assert_eq!(blockers[0].finding, "F-1");
    }

    /// VT-3: with no `.doctrine/review/` tree at all, the scan is a clean empty —
    /// the gate degrades gracefully on a slice with no reviews.
    #[test]
    fn vt3_scan_with_no_review_tree_is_empty() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(
            unresolved_blockers_for(tmp.path(), "SL-001")
                .unwrap()
                .is_empty()
        );
    }

    /// `unlock` removes a stale lock; on an unlocked review it is a clean no-op.
    #[test]
    fn unlock_clears_a_stale_lock() {
        let tmp = fixture_rv();
        let root = tmp.path();
        // Plant a stale lock (a hard-kill residue RAII never cleared).
        let lock = lock_path(root, 1);
        fs::create_dir_all(lock.parent().unwrap()).unwrap();
        fs::write(&lock, "pid = 99999\nacquired = \"stale\"\n").unwrap();
        run_unlock(Some(root.to_path_buf()), "RV-001").unwrap();
        assert!(!lock.exists(), "stale lock removed");
        // Idempotent on an unlocked review.
        run_unlock(Some(root.to_path_buf()), "RV-001").unwrap();
    }

    // -- PHASE-05: warm-cache + prime (D-C10, §9) ----------------------------

    /// Write `body` as a tracked source file under `root` (the warm-cache hashes
    /// real bytes on disk).
    fn plant_tracked(root: &Path, rel: &str, body: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, body).unwrap();
    }

    /// A two-area curated `domain_map` over two tracked paths — the `prime` input.
    fn sample_domain_map() -> &'static str {
        r#"
[[area]]
name = "turn protocol"
purpose = "baton/lock/CAS serialize turns"
paths = ["src/review.rs", "src/state.rs"]

[[invariant]]
text = "await derived, never stored (D-C8)"

[[risk]]
text = "stale baton after out-of-band edit"
"#
    }

    /// VT-1: `prime --from` persists the curated `domain_map` AND the `[hashes]`
    /// ContentSet over `⋃ area.paths`; `cache.toml` round-trips and the verdict is
    /// `current` straight after (§9, D-C10).
    #[test]
    fn vt1_prime_persists_domain_map_and_hashes_then_current() {
        let tmp = fixture_rv();
        let root = tmp.path();
        plant_tracked(root, "src/review.rs", "fn review() {}\n");
        plant_tracked(root, "src/state.rs", "fn state() {}\n");

        let map = root.join("map.toml");
        fs::write(&map, sample_domain_map()).unwrap();
        run_prime(
            Some(root.to_path_buf()),
            &PrimeArgs {
                reference: "RV-001".to_owned(),
                seed: false,
                from: Some(map),
            },
        )
        .unwrap();

        // cache.toml landed beside the baton/lock in gitignored state.
        let cache = read_cache(root, 1).unwrap().expect("cache primed");
        assert_eq!(cache.areas.len(), 1);
        assert_eq!(cache.areas[0].name, "turn protocol");
        assert_eq!(cache.invariants.len(), 1);
        assert_eq!(cache.risks.len(), 1);
        // [hashes] = the ContentSet over the union of area.paths.
        assert_eq!(
            cache.hashes.keys().cloned().collect::<Vec<_>>(),
            vec!["src/review.rs".to_owned(), "src/state.rs".to_owned()]
        );
        let expected = contentset::compute(
            root,
            &["src/review.rs".to_owned(), "src/state.rs".to_owned()],
        )
        .unwrap();
        assert_eq!(&cache.hashes, expected.hashes());

        // Read `current` straight after (no drift).
        assert!(matches!(
            cache_staleness(root, &cache).unwrap(),
            CacheVerdict::Current
        ));
    }

    /// VT-2: staleness reports `current` vs `stale`; on a tracked path's content
    /// drift it lists the changed path; an absent tracked path ⇒ stale naming it
    /// (T-b / R1).
    #[test]
    fn vt2_status_reports_current_then_stale_on_drift_and_absence() {
        let tmp = fixture_rv();
        let root = tmp.path();
        plant_tracked(root, "src/review.rs", "original\n");
        plant_tracked(root, "src/state.rs", "state\n");
        let map = root.join("map.toml");
        fs::write(&map, sample_domain_map()).unwrap();
        run_prime(
            Some(root.to_path_buf()),
            &PrimeArgs {
                reference: "RV-001".to_owned(),
                seed: false,
                from: Some(map),
            },
        )
        .unwrap();
        let cache = read_cache(root, 1).unwrap().unwrap();

        // current — nothing changed.
        assert!(matches!(
            cache_staleness(root, &cache).unwrap(),
            CacheVerdict::Current
        ));

        // Mutate a tracked file's bytes ⇒ stale, listing exactly that path.
        fs::write(root.join("src/review.rs"), "MUTATED\n").unwrap();
        match cache_staleness(root, &cache).unwrap() {
            CacheVerdict::Stale(paths) => {
                assert_eq!(paths, vec!["src/review.rs".to_owned()]);
            }
            CacheVerdict::Current => panic!("expected stale after a content drift"),
        }

        // Restore, then REMOVE a tracked file ⇒ absence⇒stale naming it (R1).
        fs::write(root.join("src/review.rs"), "original\n").unwrap();
        fs::remove_file(root.join("src/state.rs")).unwrap();
        match cache_staleness(root, &cache).unwrap() {
            CacheVerdict::Stale(paths) => {
                assert_eq!(paths, vec!["src/state.rs".to_owned()]);
            }
            CacheVerdict::Current => panic!("absent tracked path must be stale (R1)"),
        }
    }

    /// `prime` rebuilds `[hashes]` from the curated `⋃ paths` — any value the
    /// supplier put under `[hashes]` is ignored (the baseline cannot drift from the
    /// `domain_map`).
    #[test]
    fn prime_ignores_supplied_hashes_and_recomputes() {
        let tmp = fixture_rv();
        let root = tmp.path();
        plant_tracked(root, "a.txt", "real content\n");
        let map = root.join("map.toml");
        fs::write(
            &map,
            "[[area]]\nname = \"a\"\npaths = [\"a.txt\"]\n[hashes]\n\"a.txt\" = \"deadbeef\"\n",
        )
        .unwrap();
        run_prime(
            Some(root.to_path_buf()),
            &PrimeArgs {
                reference: "RV-001".to_owned(),
                seed: false,
                from: Some(map),
            },
        )
        .unwrap();
        let cache = read_cache(root, 1).unwrap().unwrap();
        assert_ne!(
            cache.hashes.get("a.txt").map(String::as_str),
            Some("deadbeef")
        );
        assert!(matches!(
            cache_staleness(root, &cache).unwrap(),
            CacheVerdict::Current
        ));
    }

    /// A `domain_map` with no areas (or an area with no paths / no name) is refused —
    /// the cache needs a curated, load-bearing set (§9, T-a).
    #[test]
    fn prime_refuses_an_empty_or_malformed_domain_map() {
        let tmp = fixture_rv();
        let root = tmp.path();
        let map = root.join("map.toml");

        // No areas at all.
        fs::write(&map, "[[invariant]]\ntext = \"x\"\n").unwrap();
        assert!(
            run_prime(
                Some(root.to_path_buf()),
                &PrimeArgs {
                    reference: "RV-001".to_owned(),
                    seed: false,
                    from: Some(map.clone()),
                },
            )
            .is_err()
        );

        // An area with no paths.
        fs::write(&map, "[[area]]\nname = \"a\"\npaths = []\n").unwrap();
        assert!(
            run_prime(
                Some(root.to_path_buf()),
                &PrimeArgs {
                    reference: "RV-001".to_owned(),
                    seed: false,
                    from: Some(map.clone()),
                },
            )
            .is_err()
        );

        // An absolute / escaping path.
        fs::write(
            &map,
            "[[area]]\nname = \"a\"\npaths = [\"../etc/passwd\"]\n",
        )
        .unwrap();
        assert!(
            run_prime(
                Some(root.to_path_buf()),
                &PrimeArgs {
                    reference: "RV-001".to_owned(),
                    seed: false,
                    from: Some(map),
                },
            )
            .is_err()
        );

        // Nothing was written on any refusal.
        assert!(read_cache(root, 1).unwrap().is_none());
    }

    /// `prime` acquires the per-review lock around the cache write — a held lock
    /// makes it bail "busy" (the §9 serialization, reusing the PHASE-03 LockGuard).
    #[test]
    fn prime_serializes_via_the_per_review_lock() {
        let tmp = fixture_rv();
        let root = tmp.path();
        plant_tracked(root, "a.txt", "x\n");
        let map = root.join("map.toml");
        fs::write(&map, "[[area]]\nname = \"a\"\npaths = [\"a.txt\"]\n").unwrap();

        // Hold the lock, then prime must bail busy (no clobber).
        let held = LockGuard::acquire(root, 1).unwrap();
        let err = run_prime(
            Some(root.to_path_buf()),
            &PrimeArgs {
                reference: "RV-001".to_owned(),
                seed: false,
                from: Some(map.clone()),
            },
        )
        .unwrap_err();
        assert!(format!("{err}").contains("busy"), "lock contention: {err}");
        assert!(
            read_cache(root, 1).unwrap().is_none(),
            "no cache written under contention"
        );
        drop(held);

        // Lock free ⇒ prime succeeds.
        run_prime(
            Some(root.to_path_buf()),
            &PrimeArgs {
                reference: "RV-001".to_owned(),
                seed: false,
                from: Some(map),
            },
        )
        .unwrap();
        assert!(read_cache(root, 1).unwrap().is_some());
    }

    /// `status` on an unprimed review reports no cache line (the signal only fires
    /// once a `domain_map` is curated, §9).
    #[test]
    fn status_is_silent_about_an_unprimed_cache() {
        let tmp = fixture_rv();
        let root = tmp.path();
        // No prime — read_cache is None, so status reports the ledger only.
        assert!(read_cache(root, 1).unwrap().is_none());
        run_status(Some(root.to_path_buf()), "RV-001").unwrap();
    }

    // ── PHASE-01: ReviewOutput + ReviewError types ──

    #[test]
    fn review_output_created_serialises_to_json() {
        let out = ReviewOutput::Created {
            id: 42,
            canonical: "RV-042".into(),
            dir: PathBuf::from(".doctrine/review/042"),
        };
        let json = serde_json::to_string(&out).unwrap();
        assert!(json.contains(r#""id":42"#), "json: {json}");
        assert!(json.contains(r#""canonical":"RV-042""#), "json: {json}");
        assert!(
            json.contains(r#""dir":".doctrine/review/042""#),
            "json: {json}"
        );
    }

    #[test]
    fn review_error_downcasts_from_anyhow() {
        let err = ReviewError::RoleMismatch {
            expected: Role::Raiser,
            actual: Role::Responder,
            verb: Verb::Raise,
        };
        let anyhow_err: anyhow::Error = err.into();
        let downcast = anyhow_err
            .downcast_ref::<ReviewError>()
            .expect("ReviewError should downcast from anyhow");
        match downcast {
            ReviewError::RoleMismatch {
                expected,
                actual,
                verb,
            } => {
                assert_eq!(*expected, Role::Raiser);
                assert_eq!(*actual, Role::Responder);
                assert_eq!(*verb, Verb::Raise);
            }
            _ => panic!("wrong variant: {downcast:?}"),
        }
    }

    #[test]
    fn with_turn_accepts_non_unit_closure_return() {
        // Verify the generic parameter T != () compiles — this test is
        // compile-time; the behaviour is verified by the verb handler tests.
        // We just assert that a String-returning closure type-checks.
        let _: fn(&mut toml_edit::DocumentMut, &[FindingRow]) -> anyhow::Result<String> =
            |_, _| anyhow::Ok("F-1".into());
    }

    // ------------------------------------------------------------------
    // PHASE-02 — golden tests: capture current stdout and assert
    // print_review() reproduces it identically (VT-1..VT-10)
    // ------------------------------------------------------------------

    /// Golden: `print_review(&Unlocked)` with no lock produces "RV-001 is not locked".
    #[test]
    fn golden_print_unlocked_not_locked() {
        let out = ReviewOutput::Unlocked {
            canonical: "RV-001".into(),
            formatted: String::new(),
        };
        let rendered = print_review(&out);
        assert_eq!(rendered, "RV-001 is not locked\n");
    }

    /// Golden: `run_unlock` on a review that is not locked returns Unlocked
    /// with empty formatted, and print_review renders correctly.
    #[test]
    fn golden_run_unlock_not_locked() {
        let tmp = fixture_rv();
        let root = tmp.path();
        let out = run_unlock(Some(root.to_path_buf()), "RV-001").unwrap();
        match &out {
            ReviewOutput::Unlocked {
                canonical,
                formatted,
            } => {
                assert_eq!(canonical, "RV-001");
                assert!(formatted.is_empty());
            }
            _ => panic!("expected Unlocked, got {out:?}"),
        }
        let rendered = print_review(&out);
        assert_eq!(rendered, "RV-001 is not locked\n");
    }

    /// Golden: `print_review(&Created)` produces "Created review 001: <dir>".
    #[test]
    fn golden_print_created() {
        let out = ReviewOutput::Created {
            id: 1,
            canonical: "RV-001".into(),
            dir: PathBuf::from(".doctrine/review/001"),
        };
        let rendered = print_review(&out);
        assert_eq!(rendered, "Created review 001: .doctrine/review/001\n");
    }

    /// Golden: `run_new` creates a review and returns Created with correct fields.
    #[test]
    fn golden_run_new() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        plant_slice_target(root, 42);
        let out = run_new(Some(root.to_path_buf()), &new_args(Facet::Design, "SL-042")).unwrap();
        match &out {
            ReviewOutput::Created { id, canonical, dir } => {
                assert_eq!(*id, 1);
                assert_eq!(canonical, "RV-001");
                assert!(dir.to_string_lossy().contains("001"));
            }
            _ => panic!("expected Created, got {out:?}"),
        }
        let rendered = print_review(&out);
        assert!(rendered.starts_with("Created review 001: "));
    }

    /// Golden: `print_review(&Raised)` produces "Raised F-1 on RV-001".
    #[test]
    fn golden_print_raised() {
        let out = ReviewOutput::Raised {
            finding_id: "F-1".into(),
            review_id: 1,
        };
        let rendered = print_review(&out);
        assert_eq!(rendered, "Raised F-1 on RV-001\n");
    }

    /// Golden: `run_raise` on a fresh RV returns Raised with the finding_id.
    #[test]
    fn golden_run_raise() {
        let tmp = fixture_rv();
        let root = tmp.path();
        let out = run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Major, "test finding"),
            Role::Raiser,
        )
        .unwrap();
        match &out {
            ReviewOutput::Raised {
                finding_id,
                review_id,
            } => {
                assert_eq!(finding_id, "F-1");
                assert_eq!(*review_id, 1);
            }
            _ => panic!("expected Raised, got {out:?}"),
        }
        let rendered = print_review(&out);
        assert_eq!(rendered, "Raised F-1 on RV-001\n");
    }

    /// Golden: `print_review(&Disposed)` produces "Disposed F-1 on RV-001 (answered)".
    #[test]
    fn golden_print_disposed() {
        let out = ReviewOutput::Disposed {
            finding_id: "F-1".into(),
            review_id: 1,
        };
        let rendered = print_review(&out);
        assert_eq!(rendered, "Disposed F-1 on RV-001 (answered)\n");
    }

    /// Golden: `run_dispose` on a raised finding returns Disposed.
    #[test]
    fn golden_run_dispose() {
        let tmp = fixture_rv();
        let root = tmp.path();
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Major, "test"),
            Role::Raiser,
        )
        .unwrap();
        let out = run_dispose(
            Some(root.to_path_buf()),
            &dispose_args("RV-001", "F-1"),
            Role::Responder,
        )
        .unwrap();
        match &out {
            ReviewOutput::Disposed {
                finding_id,
                review_id,
            } => {
                assert_eq!(finding_id, "F-1");
                assert_eq!(*review_id, 1);
            }
            _ => panic!("expected Disposed, got {out:?}"),
        }
        let rendered = print_review(&out);
        assert_eq!(rendered, "Disposed F-1 on RV-001 (answered)\n");
    }

    /// Golden: `print_review(&Verified)`.
    #[test]
    fn golden_print_verified() {
        let out = ReviewOutput::Verified {
            finding_id: "F-1".into(),
            review_id: 1,
        };
        let rendered = print_review(&out);
        assert_eq!(rendered, "Verified F-1 on RV-001 (verified)\n");
    }

    /// Golden: `print_review(&Contested)`.
    #[test]
    fn golden_print_contested() {
        let out = ReviewOutput::Contested {
            finding_id: "F-1".into(),
            review_id: 1,
        };
        let rendered = print_review(&out);
        assert_eq!(rendered, "Contested F-1 on RV-001 (contested)\n");
    }

    /// Golden: `print_review(&Withdrawn)`.
    #[test]
    fn golden_print_withdrawn() {
        let out = ReviewOutput::Withdrawn {
            finding_id: "F-1".into(),
            review_id: 1,
        };
        let rendered = print_review(&out);
        assert_eq!(rendered, "Withdrew F-1 on RV-001 (withdrawn)\n");
    }

    /// Golden: `run_verify` end-to-end.
    #[test]
    fn golden_run_verify() {
        let tmp = fixture_rv();
        let root = tmp.path();
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Major, "test"),
            Role::Raiser,
        )
        .unwrap();
        run_dispose(
            Some(root.to_path_buf()),
            &dispose_args("RV-001", "F-1"),
            Role::Responder,
        )
        .unwrap();
        let out = run_verify(
            Some(root.to_path_buf()),
            "RV-001",
            "F-1",
            None,
            Role::Raiser,
        )
        .unwrap();
        let rendered = print_review(&out);
        assert_eq!(rendered, "Verified F-1 on RV-001 (verified)\n");
    }

    /// Golden: `run_show` returns Showed with formatted output.
    #[test]
    fn golden_run_show() {
        let tmp = fixture_rv();
        let root = tmp.path();
        let out = run_show(Some(root.to_path_buf()), "RV-001", Format::Table).unwrap();
        let rendered = print_review(&out);
        // The show output is a multi-line table. Check key lines are present.
        assert!(rendered.contains("RV-001 — "), "show: {rendered}");
        assert!(rendered.contains("design · "), "show: {rendered}");
        assert!(rendered.contains("──reviews──▶"), "show: {rendered}");
    }

    /// Golden: `run_list` returns Listed with formatted table output.
    #[test]
    fn golden_run_list() {
        let tmp = fixture_rv();
        let root = tmp.path();
        let out = run_list(
            Some(root.to_path_buf()),
            ListArgs {
                substr: None,
                regexp: None,
                case_insensitive: false,
                status: Vec::new(),
                tags: Vec::new(),
                all: false,
                format: Format::Table,
                json: false,
                columns: None,
                render: listing::RenderOpts {
                    color: false,
                    term_width: None,
                },
            },
        )
        .unwrap();
        match &out {
            ReviewOutput::Listed {
                rows, formatted, ..
            } => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].id, "RV-001");
                // The formatted table contains the review data.
                assert!(formatted.contains("RV-001"), "list: {formatted}");
            }
            _ => panic!("expected Listed, got {out:?}"),
        }
        let rendered = print_review(&out);
        assert!(rendered.contains("RV-001"), "list: {rendered}");
    }

    /// Golden: `run_status` returns Status with correct fields.
    #[test]
    fn golden_run_status() {
        let tmp = fixture_rv();
        let root = tmp.path();
        let out = run_status(Some(root.to_path_buf()), "RV-001").unwrap();
        let formatted = match &out {
            ReviewOutput::Status {
                canonical,
                status,
                cache_primed,
                formatted,
                ..
            } => {
                assert_eq!(canonical, "RV-001");
                assert_eq!(status, "done");
                assert!(!cache_primed);
                assert!(formatted.contains("RV-001 — "), "status: {formatted}");
                assert!(formatted.contains("done · "), "status: {formatted}");
                formatted.clone()
            }
            _ => panic!("expected Status, got {out:?}"),
        };
        let rendered = print_review(&out);
        assert_eq!(rendered, formatted);
    }

    /// Golden: `run_show` with JSON format returns Showed with JSON-formatted string.
    #[test]
    fn golden_run_show_json() {
        let tmp = fixture_rv();
        let root = tmp.path();
        let out = run_show(Some(root.to_path_buf()), "RV-001", Format::Json).unwrap();
        let rendered = print_review(&out);
        assert!(rendered.contains("\"kind\""), "show json: {rendered}");
        assert!(rendered.contains("\"review\""), "show json: {rendered}");
    }
}
