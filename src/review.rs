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

use crate::tomlfmt::toml_string;

// ---------------------------------------------------------------------------
// Closed vocabulary enums (each: `as_str` render mirror + a `&[&str]` known-set,
// lockstep-guarded by a drift canary test).
// ---------------------------------------------------------------------------

/// What a review reviews — the facet (design §5, D-C11). The closed 7-set with
/// **no `drift`** (D-C11 dropped it → the future Drift Ledger kind, IMP-022).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
const REVIEW_STATUSES: &[&str] = &["active", "done"];

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
/// - empty ⇒ `(Active, Raiser)` — the raiser goes first; empty ≠ done (the
///   SL-009-divergence-proof case).
/// - any `open`/`contested` ⇒ `(Active, Responder)` — work awaits the responder.
/// - else any `answered` ⇒ `(Active, Raiser)` — work awaits the raiser.
/// - all `∈ {verified, withdrawn}` ⇒ `(Done, None)`.
///
/// `await` is a *priority summary* (open/contested wins display), never an
/// exclusive gate — the turn gate is per-finding `can` (D7).
pub(crate) fn derived_status(findings: &[FindingState]) -> (ReviewStatus, Await) {
    if findings.is_empty() {
        return (ReviewStatus::Active, Await::Raiser);
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
#[derive(Debug, Clone, PartialEq, Eq)]
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

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

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
    prefix: "RV",
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
pub(crate) fn run_new(path: Option<PathBuf>, args: &NewArgs) -> anyhow::Result<()> {
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
    writeln!(
        io::stdout(),
        "Created review {id:03}: {}",
        out.dir.display()
    )?;
    Ok(())
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

/// `doctrine review show <RV-NNN>` — read the RV as data and render the readable
/// whole (`Table`) or the faithful toml-as-data + brief (`Json`). The status is
/// DERIVED here (review never asks the shared reader for a stored status, D-C8);
/// the `reviews` edge is rendered as `RV-NNN ──reviews──▶ <target>`.
pub(crate) fn run_show(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let review_root = root.join(REVIEW_DIR);
    let id = parse_ref(reference)?;
    let doc = read_review(&review_root, id)?;
    let body = read_brief(&review_root, id)?;
    let out = match format {
        Format::Table => format_show(&doc, &body),
        Format::Json => show_json(&doc, &body)?,
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
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
    },
    Column {
        name: "facet",
        header: "facet",
        cell: |(d, _, _)| d.review.facet.clone(),
    },
    Column {
        name: "target",
        header: "target",
        cell: |(d, _, _)| edge_label(d),
    },
    Column {
        name: "title",
        header: "title",
        cell: |(d, _, _)| d.title.clone(),
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
fn list_rows(root: &Path, mut args: ListArgs) -> anyhow::Result<String> {
    listing::validate_statuses(&args.status, REVIEW_STATUSES)?;
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
    match format {
        Format::Table => {
            let sel = listing::select_columns(&REVIEW_COLUMNS, REVIEW_DEFAULT, columns.as_deref())?;
            Ok(listing::render_columns(&rows, &sel))
        }
        Format::Json => listing::json_envelope("review", &json_rows(&rows)),
    }
}

/// Faithful JSON rows for `list` — the prefixed id, derived status/await, facet,
/// target edge, and title.
#[derive(Debug, Serialize)]
struct ListRow {
    id: String,
    status: String,
    awaiting: String,
    facet: String,
    target: String,
    title: String,
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
pub(crate) fn run_list(path: Option<PathBuf>, args: ListArgs) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let mut out = io::stdout();
    write!(out, "{}", list_rows(&root, args)?)?;
    Ok(())
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
    fn derived_status_empty_is_active_raiser() {
        assert_eq!(derived_status(&[]), (ReviewStatus::Active, Await::Raiser));
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
    /// await `raiser`, and the `reviews` edge to the target.
    #[test]
    fn show_renders_empty_ledger_active_raiser_and_the_edge() {
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
        // empty ⇒ Active, await=Raiser (PHASE-01 derived_status, D-C8).
        assert!(out.contains("active · await=raiser"), "{out}");
        assert!(out.contains("RV-003 ──reviews──▶ SL-024"), "edge: {out}");
        assert!(out.contains("findings: 0"), "{out}");
    }

    /// VT-4 (list): the empty-ledger RV lists with derived status `active`
    /// (await raiser), facet, and the target edge — no stored status read.
    #[test]
    fn list_renders_derived_status_facet_and_edge() {
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
        let out = listing::render_columns(&rows, &sel);
        let lines: Vec<&str> = out.lines().collect();
        assert!(lines[0].starts_with("id"), "header: {:?}", lines[0]);
        assert!(lines[1].starts_with("RV-005"), "{:?}", lines[1]);
        assert!(lines[1].contains("active (await raiser)"), "{:?}", lines[1]);
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
        assert_eq!(doc.derived(), (ReviewStatus::Active, Await::Raiser));
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
}
