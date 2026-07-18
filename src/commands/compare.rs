// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine compare` — the pairwise comparison capture verb (SL-210
//! PHASE-02; capture surface reworked for schema v2 in SL-213 PHASE-01). The
//! impure command shell over the pure wire model in `crate::comparison`: it
//! resolves refs to kinds, checks admissibility, mints the impure edge (uuid
//! v7 session/row uids + today's date), and writes a merge-clean
//! session-of-one file under `.doctrine/comparisons/`.
//!
//! v2 capture (SL-213 design S1): the response is a mutually-exclusive group
//! `--prefer | --equal | --incomparable` (exactly one); `--supersedes <uid>`
//! is validated against the loaded corpus (unknown uid = hard error — the
//! only moment a human is present); the frame derives the domain silently
//! (`prefer-first` ⇒ `priority`), via the wire tier's single table.
//!
//! Layering (ADR-001): a `command`-tier shell. Impurity (clock/uuid/disk) lives
//! here; the pure model never sees a path or a clock.
//!
//! Clap shape (EX-4): the design's PRE-AUTHORISED fallback — capture is the
//! `record` subcommand (`compare record <A> <B> …`), beside `list` / `withdraw`.
//! The primary shape (bare capture args coexisting with subcommands via
//! `args_conflicts_with_subcommands` + `subcommand_negates_reqs`) was tried
//! first and genuinely fought clap: `subcommand_negates_reqs` does NOT relax
//! REQUIRED positionals (`<A>`/`<B>`) that share an argument slot with a
//! subcommand name, so `compare list` demanded the capture positionals. The
//! subcommand group is cosmetic, not structural. `list` / `withdraw` are stubbed
//! here and land in PHASE-03 — their arg shapes are fixed now.

use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::{Args, Subcommand, ValueEnum};

use crate::comparison::{
    self, COMPARISONS_DIR, ComparisonSession, FRAME_EQUAL_EFFORT, FRAME_MORE_WORK,
    FRAME_PREFER_FIRST, Judgement, RaterKind, ResolutionStatus, Response, RowForm, RowSummary,
    SessionHeader, Tombstone, load_sessions,
};
use crate::priority::config::PriorityConfig;
use crate::priority::elicit::{
    CandidateKind, DecisionContext, ElicitInputs, ElicitQueue, EntryPayload, FrontierItem,
    ItemCosting, Participant, QueueEntry, QueueState, SizingInputs, assemble,
};
use crate::priority::graph::{self, PriorityGraph};
use crate::priority::view::ReasonKind;

/// `doctrine compare <SUBCOMMAND>` — the comparison-ledger verb group.
#[derive(Args)]
pub(crate) struct CompareArgs {
    #[command(subcommand)]
    pub(crate) action: CompareAction,
}

/// The comparison subcommands. `record` captures; `list` / `withdraw` are
/// PHASE-03 stubs (their arg shapes are fixed now so the read-side phase only
/// fills the run paths).
#[derive(Subcommand)]
pub(crate) enum CompareAction {
    /// Capture a pairwise value comparison as a session-of-one file.
    Record(RecordArgs),
    /// List captured comparison judgements (SL-210 PHASE-03).
    List(ListArgs),
    /// Withdraw a judgement row by uid (SL-210 PHASE-03).
    Withdraw(WithdrawArgs),
    /// Surface the ranked elicitation queue — the next comparisons/anchor
    /// reviews worth asking (SL-217 PHASE-03; read-only, D18).
    Elicit(ElicitArgs),
}

/// `compare record <A> <B> <response> [flags]` — the full capture surface.
/// The response is a mutually-exclusive group (SL-213 S1): exactly one of
/// `--prefer` / `--equal` / `--incomparable`. `domain` derives from the frame
/// (never typed); `form` is fixed at `order` (no flag mints it; `--magnitude`
/// awaits RFC-019 OQ-6).
#[derive(Args)]
#[command(group = clap::ArgGroup::new("response").required(true).multiple(false))]
pub(crate) struct RecordArgs {
    /// First entity — full canonical ref (e.g. `SL-204`).
    pub(crate) a: String,
    /// Second entity — full canonical ref (e.g. `IMP-118`).
    pub(crate) b: String,
    /// Preferred side: the literal `a` / `b`, or the full ref of one side.
    #[arg(long, group = "response")]
    pub(crate) prefer: Option<String>,
    /// The two sides carry exactly equal value (compiles to a class merge).
    #[arg(long, group = "response")]
    pub(crate) equal: bool,
    /// Considered "these don't compare" — recorded as asked, zero constraint.
    #[arg(long, group = "response")]
    pub(crate) incomparable: bool,
    /// Supersede a prior judgement row by uid (explicit revision — the target
    /// must exist in the ledger).
    #[arg(long)]
    pub(crate) supersedes: Option<String>,
    /// Comparison frame (closed vocab; default `equal-effort`). `prefer-first`
    /// asks "under a binding capacity cutoff, which do you keep?" and records
    /// a priority-domain row — not value-bearing. `more-work` asks "which is
    /// more work? — winner is the costlier" and records an estimate-domain row.
    #[arg(long, value_enum, default_value_t = FrameArg::EqualEffort)]
    pub(crate) frame: FrameArg,
    /// Who rendered the judgement (default `agent`).
    #[arg(long, value_enum, default_value_t = RaterArg::Agent)]
    pub(crate) rater: RaterArg,
    /// Optional rater identity (free text).
    #[arg(long)]
    pub(crate) by: Option<String>,
    /// Optional value lens (the IDE-035 seam).
    #[arg(long)]
    pub(crate) lens: Option<String>,
    /// Optional free-text note.
    #[arg(long)]
    pub(crate) note: Option<String>,
    /// Optional audience tag — the session-header field (OQ-1/T4).
    #[arg(long)]
    pub(crate) audience: Option<String>,
    /// Explicit project root (default: auto-detect).
    #[arg(short = 'p', long)]
    pub(crate) path: Option<PathBuf>,
}

/// `compare list [<ID>] [--active-only]` — arg shape for PHASE-03, status
/// column + `--active-only` filter SL-213 PHASE-06 (design §4 S2).
#[derive(Args)]
pub(crate) struct ListArgs {
    /// Optional participation filter — a canonical ref; rows where `a` or `b`
    /// equals it.
    pub(crate) id: Option<String>,
    /// Show only `ResolutionStatus::Active` rows — quarantined and
    /// no-constraint rows ARE active (they still show; they are visibly
    /// non-constraining, design D13/S2).
    #[arg(long)]
    pub(crate) active_only: bool,
    /// Explicit project root (default: auto-detect).
    #[arg(short = 'p', long)]
    pub(crate) path: Option<PathBuf>,
}

/// `compare withdraw <ROW-UID> [--note]` — arg shape for PHASE-03.
#[derive(Args)]
pub(crate) struct WithdrawArgs {
    /// The judgement row uid to withdraw (full uid — never a prefix).
    pub(crate) uid: String,
    /// Optional note recording the reason for withdrawal.
    #[arg(long)]
    pub(crate) note: Option<String>,
    /// Explicit project root (default: auto-detect).
    #[arg(short = 'p', long)]
    pub(crate) path: Option<PathBuf>,
}

/// `compare elicit [--depth K] [--limit N] [--kind …] [--json]` — surface the
/// ranked elicitation queue (design §3). Read-only end to end (D18): every entry
/// carries the exact answer command; no TTY interaction, no writes.
#[derive(Args)]
pub(crate) struct ElicitArgs {
    /// Frontier band depth to probe (top-K). Overrides `[priority.elicit] depth`
    /// (default `ELICIT_DEPTH`); clamped to ≥ 1.
    #[arg(long)]
    pub(crate) depth: Option<usize>,
    /// Cap the number of entries DISPLAYED (default `ELICIT_LIMIT`). The full
    /// pool is still ranked — the cap is display-only.
    #[arg(long)]
    pub(crate) limit: Option<usize>,
    /// Show only entries of this kind (filter applied POST-ranking).
    #[arg(long, value_enum)]
    pub(crate) kind: Option<KindArg>,
    /// Emit the schema-v1 JSON envelope instead of the human render.
    #[arg(long)]
    pub(crate) json: bool,
    /// Explicit project root (default: auto-detect).
    #[arg(short = 'p', long)]
    pub(crate) path: Option<PathBuf>,
}

/// Clap adapter for the candidate-kind filter — keeps clap out of the pure
/// `priority::elicit` tier. Ties back to [`CandidateKind`] (no magic strings).
#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum KindArg {
    Comparison,
    AnchorReview,
    SizingProbe,
    ClaimReprobe,
}

impl KindArg {
    fn matches(self, kind: CandidateKind) -> bool {
        matches!(
            (self, kind),
            (KindArg::Comparison, CandidateKind::Comparison)
                | (KindArg::AnchorReview, CandidateKind::AnchorReview)
                | (KindArg::SizingProbe, CandidateKind::SizingProbe)
                | (KindArg::ClaimReprobe, CandidateKind::ClaimReprobe)
        )
    }
}

/// Clap adapter for the closed frame vocab — keeps clap out of the pure
/// `comparison` engine tier (ADR-001). The kebab-cased variant names are the
/// CLI tokens; [`Self::as_frame`] ties them back to the engine constants (no
/// magic strings, STD-001).
#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum FrameArg {
    EqualEffort,
    PreferFirst,
    MoreWork,
}

impl FrameArg {
    fn as_frame(self) -> &'static str {
        match self {
            Self::EqualEffort => FRAME_EQUAL_EFFORT,
            Self::PreferFirst => FRAME_PREFER_FIRST,
            Self::MoreWork => FRAME_MORE_WORK,
        }
    }
}

/// Clap adapter for the rater kind — the shell boundary that keeps clap out of
/// the pure model's [`RaterKind`].
#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum RaterArg {
    Human,
    Agent,
}

impl RaterArg {
    fn to_kind(self) -> RaterKind {
        match self {
            Self::Human => RaterKind::Human,
            Self::Agent => RaterKind::Agent,
        }
    }
}

/// Dispatch `doctrine compare` — capture (`record`) or a PHASE-03 stub.
pub(crate) fn run_compare(args: CompareArgs) -> anyhow::Result<()> {
    match args.action {
        CompareAction::Record(record) => run_capture(&record),
        CompareAction::List(list) => run_list(&list),
        CompareAction::Withdraw(withdraw) => run_withdraw(&withdraw),
        CompareAction::Elicit(elicit) => run_elicit(&elicit),
    }
}

/// Resolve one participant ref to `(canonical id, kind prefix)`. Enforces the
/// full canonical form (design D4 — bare ids are cross-kind-ambiguous) and that
/// the entity exists (dangling evidence refused), riding the facet resolver so
/// there is no parallel corpus scan.
fn resolve_participant(root: &Path, raw: &str) -> anyhow::Result<(String, &'static str)> {
    // D4: full `SL-123` form only — rejects a bare `123`.
    let (kref, _id) = crate::integrity::parse_canonical_ref(raw)?;
    // Existence — a well-formed but absent ref is refused here.
    let (_path, canonical) = crate::commands::facet::resolve_entity_path_and_canonical(root, raw)?;
    Ok((canonical, kref.kind.prefix))
}

/// Resolve the response group to a wire [`Response`]. `--prefer` accepts the
/// literal `a` / `b`, or the full ref of either side; anything else is
/// refused. Clap enforces exactly-one-of, so the fallthrough is `--prefer`.
fn resolve_response(
    args: &RecordArgs,
    canonical_a: &str,
    canonical_b: &str,
) -> anyhow::Result<Response> {
    if args.equal {
        return Ok(Response::Equal);
    }
    if args.incomparable {
        return Ok(Response::Incomparable);
    }
    let prefer = args
        .prefer
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("one of --prefer / --equal / --incomparable is required"))?;
    match prefer {
        "a" => Ok(Response::PreferA),
        "b" => Ok(Response::PreferB),
        other if other == canonical_a => Ok(Response::PreferA),
        other if other == canonical_b => Ok(Response::PreferB),
        other => anyhow::bail!(
            "--prefer `{other}` must be `a`, `b`, or one of the two refs ({canonical_a} / {canonical_b})"
        ),
    }
}

/// Validate `--supersedes` against the loaded corpus (SL-213 S1): the target
/// must be an existing judgement row's uid — capture is the only moment a
/// human is present, so an unknown uid is a hard error, not a load-time
/// warning. Resolution happens here in the shell; the pure tiers never trust
/// an unresolved ref.
fn validate_supersedes(root: &Path, target: &str) -> anyhow::Result<()> {
    let sessions = load_sessions(root)?;
    let known = sessions
        .iter()
        .any(|s| s.judgements.iter().any(|j| j.uid == target));
    if !known {
        anyhow::bail!(
            "--supersedes `{target}` names no judgement row — supersession targets an existing row uid (see `compare list`)"
        );
    }
    Ok(())
}

/// The capture flow: resolve → admissibility → build → validate → mint (impure
/// edge) → write a fresh session-of-one file (clobber-refusing).
fn run_capture(args: &RecordArgs) -> anyhow::Result<()> {
    use std::io::Write;

    let root = crate::root::find(args.path.clone(), &crate::root::default_markers())?;

    // Resolve both refs (existence + kind). Order: refs before admissibility so
    // a dangling ref is the first refusal.
    let (canonical_a, kind_a) = resolve_participant(&root, &args.a)?;
    let (canonical_b, kind_b) = resolve_participant(&root, &args.b)?;

    // The frame implies the domain (S1) — the wire table is the single source.
    // Derived before admissibility, which routes by it.
    let frame = args.frame.as_frame();
    let domain = comparison::domain_for_frame(frame)
        .ok_or_else(|| anyhow::anyhow!("frame `{frame}` maps to no domain"))?;

    // Pair admissibility over the resolved kinds, routed by the frame-DERIVED
    // domain (SL-219 D3): the estimate domain admits work + record kinds (RSK
    // included); value frames keep the value admit set (record pairs / RSK
    // participants refused, with the human-readable reason) — the priority
    // domain reuses it (design D2).
    if domain == comparison::DOMAIN_ESTIMATE {
        comparison::admissible_estimate_pair(kind_a, kind_b)
    } else {
        comparison::admissible_value_pair(kind_a, kind_b)
    }
    .map_err(|reason| anyhow::anyhow!(reason))?;

    let response = resolve_response(args, &canonical_a, &canonical_b)?;

    // Supersession target resolved against the corpus before any write.
    if let Some(target) = args.supersedes.as_deref() {
        validate_supersedes(&root, target)?;
    }

    // Impure edge: v7 uids (hyphenated, per the schema) + today's date.
    let session_uid = uuid::Uuid::now_v7().to_string();
    let row_uid = uuid::Uuid::now_v7().to_string();
    let date = crate::clock::today();

    let judgement = Judgement {
        uid: row_uid,
        seq: 0,
        a: canonical_a,
        b: Some(canonical_b),
        response: Some(response),
        domain: domain.to_string(),
        frame: frame.to_string(),
        form: RowForm::Order,
        magnitude: None,
        est_lower: None,
        est_upper: None,
        supersedes: args.supersedes.clone(),
        lens: args.lens.clone(),
        rater: args.rater.to_kind(),
        by: args.by.clone(),
        note: args.note.clone(),
        date: Some(date.clone()),
        observed_at: None,
        basis: None,
        admission: None,
    };
    comparison::validate_judgement(&judgement)?;

    let session = comparison::session_of_one(
        SessionHeader {
            uid: session_uid.clone(),
            date: date.clone(),
            audience: args.audience.clone(),
        },
        judgement,
    );
    let text = comparison::to_toml(&session)?;

    // Write a fresh file — clobber-refusing, dir created lazily. The v7 session
    // uid keys the filename, so distinct invocations never collide (append-only,
    // design D2).
    let path = write_session_file(&root, &date, &session_uid, &text)?;

    writeln!(
        std::io::stdout(),
        "compare: captured {} — session {}",
        path.display(),
        session_uid
    )?;
    Ok(())
}

/// The value-anchor payload the `value set|pin` verbs mint (SL-220 §4). Pure
/// data; [`record_anchor`] performs the impure edge (uid/date) + write. Rides
/// the `compare record` mint path VERBATIM — the shell stamps `frame =
/// value-anchor`, `domain = value`, `form = anchor`.
pub(crate) struct AnchorSpec {
    /// The single subject (a resolved canonical id).
    pub(crate) subject: String,
    pub(crate) magnitude: Option<f64>,
    pub(crate) rater: RaterKind,
    /// `Some(Pin)` for the gated `value pin` path; `None` for `value set`.
    pub(crate) admission: Option<crate::comparison::AdmissionKind>,
    pub(crate) by: Option<String>,
    pub(crate) basis: Option<String>,
    pub(crate) lens: Option<String>,
    pub(crate) note: Option<String>,
    pub(crate) supersedes: Option<String>,
    /// Estimate-domain: lower bound (cost-anchor rows only).
    pub(crate) est_lower: Option<f64>,
    /// Estimate-domain: upper bound (cost-anchor rows only).
    pub(crate) est_upper: Option<f64>,
}

/// Mint + write ONE value-anchor session-of-one via the `compare record` path
/// (SL-220 §4): stamp the anchor discriminators, validate the wire row, resolve
/// `--supersedes` scope (anchor-form target sharing subject/domain/lens)
/// against the corpus, then append a fresh clobber-refusing file. EVERY
/// invocation mints (D10 — no idempotency guard); the row uid/date are the
/// single impure edge, exactly as `run_capture`.
pub(crate) fn record_anchor(root: &Path, spec: &AnchorSpec) -> anyhow::Result<PathBuf> {
    let session_uid = uuid::Uuid::now_v7().to_string();
    let row_uid = uuid::Uuid::now_v7().to_string();
    let date = crate::clock::today();

    // Estimate-domain vs value-domain anchor detection:
    let is_estimate = spec.est_lower.is_some() || spec.est_upper.is_some();
    let domain = if is_estimate {
        comparison::DOMAIN_ESTIMATE.to_string()
    } else {
        comparison::DOMAIN_VALUE.to_string()
    };
    let frame = if is_estimate {
        comparison::FRAME_COST_ANCHOR.to_string()
    } else {
        comparison::FRAME_VALUE_ANCHOR.to_string()
    };
    let judgement = Judgement {
        uid: row_uid,
        seq: 0,
        a: spec.subject.clone(),
        b: None,
        response: None,
        domain,
        frame,
        form: RowForm::Anchor,
        magnitude: spec.magnitude,
        est_lower: spec.est_lower,
        est_upper: spec.est_upper,
        supersedes: spec.supersedes.clone(),
        lens: spec.lens.clone(),
        rater: spec.rater.clone(),
        by: spec.by.clone(),
        note: spec.note.clone(),
        date: Some(date.clone()),
        observed_at: None,
        basis: spec.basis.clone(),
        admission: spec.admission,
    };
    comparison::validate_judgement(&judgement)?;

    // Supersession scope (SL-220 §4): resolve the target uid against the corpus
    // and check its (subject, domain, lens) match — refused AT CAPTURE.
    if let Some(target_uid) = spec.supersedes.as_deref() {
        let sessions = load_sessions(root)?;
        let target = sessions
            .iter()
            .flat_map(|s| &s.judgements)
            .find(|j| j.uid == target_uid)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "--supersedes `{target_uid}` names no judgement row — supersession targets an existing row uid (see `compare list`)"
                )
            })?;
        comparison::validate_anchor_supersedes(&judgement, target)
            .map_err(|reason| anyhow::anyhow!(reason))?;
    }

    let session = comparison::session_of_one(
        SessionHeader {
            uid: session_uid.clone(),
            date: date.clone(),
            audience: None,
        },
        judgement,
    );
    let text = comparison::to_toml(&session)?;
    write_session_file(root, &date, &session_uid, &text)
}

/// Append ONE session carrying tombstones for `targets` (SL-220 §4): the
/// `value clear` / `value pin --retire` withdrawal path, reusing the create-new
/// session seam. `targets` is caller-guaranteed non-empty (the no-op is
/// short-circuited upstream). Stamps the CURRENT wire discriminators.
pub(crate) fn record_tombstones(
    root: &Path,
    targets: &[String],
    note: Option<&str>,
) -> anyhow::Result<PathBuf> {
    let session_uid = uuid::Uuid::now_v7().to_string();
    let date = crate::clock::today();
    let tombstones: Vec<Tombstone> = targets
        .iter()
        .enumerate()
        .map(|(i, target)| Tombstone {
            uid: uuid::Uuid::now_v7().to_string(),
            seq: u32::try_from(i).unwrap_or(u32::MAX),
            target: target.clone(),
            date: date.clone(),
            note: note.map(str::to_string),
        })
        .collect();
    let session = ComparisonSession {
        schema: comparison::COMPARISON_SCHEMA.to_string(),
        version: comparison::COMPARISON_VERSION,
        session: SessionHeader {
            uid: session_uid.clone(),
            date: date.clone(),
            audience: None,
        },
        judgements: Vec::new(),
        tombstones,
    };
    let text = comparison::to_toml(&session)?;
    write_session_file(root, &date, &session_uid, &text)
}

/// Write a fresh session file `<date>-<uid>.toml` under
/// `.doctrine/comparisons/` — clobber-refusing (`create-new`), directory made
/// lazily. The single disk seam shared by capture, withdrawal, and the SL-220
/// value-anchor verbs (design D2 — append-only, exactly one new file per act;
/// never rewrites an existing path).
pub(crate) fn write_session_file(
    root: &Path,
    date: &str,
    session_uid: &str,
    text: &str,
) -> anyhow::Result<PathBuf> {
    use std::io::Write;
    let dir = root.join(".doctrine").join(COMPARISONS_DIR);
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("create comparisons dir {}", dir.display()))?;
    let path = dir.join(format!("{date}-{session_uid}.toml"));
    let mut file = crate::fsutil::create_new_file(&path)
        .with_context(|| format!("write comparison session {}", path.display()))?;
    file.write_all(text.as_bytes())
        .with_context(|| format!("write comparison session {}", path.display()))?;
    Ok(path)
}

/// Render one joined row for `list` (SL-213 PHASE-06, design §4 S2): the FULL
/// row uid (never a prefix — uuid-v7 prefixes share a timestamp bucket and
/// collide, and this line feeds `withdraw`, RV-262 F-6), the joined
/// [`RowState`] display token, the pair with the preferred side marked `*`,
/// frame, rater (+identity when present), date, note when present, and a
/// `[withdrawn]` marker when the row is `ResolutionStatus::Tombstoned`
/// (retained verbatim alongside the new `status=tombstoned` token —
/// pre-existing golden text, extend-don't-replace).
fn render_row(row: &RowSummary) -> String {
    // Minimal v2 adaptation (SL-213 PHASE-01 EX-4): the `*` marks a preferred
    // side; `=` / `~` render equal / incomparable. An anchor row (SL-220 §1:
    // single subject, no pairwise payload) renders its subject alone — the
    // full mixed-fixture render treatment is PHASE-06.
    let pair = match (row.response, row.b.as_deref()) {
        (Some(Response::PreferA), Some(b)) => format!("{}* vs {}", row.a, b),
        (Some(Response::PreferB), Some(b)) => format!("{} vs {}*", row.a, b),
        (Some(Response::Equal), Some(b)) => format!("{} = {}", row.a, b),
        (Some(Response::Incomparable), Some(b)) => format!("{} ~ {}", row.a, b),
        _ => row.a.clone(),
    };
    let by = row
        .by
        .as_deref()
        .map(|b| format!(":{b}"))
        .unwrap_or_default();
    let note = row
        .note
        .as_deref()
        .map(|n| format!("  note: {n}"))
        .unwrap_or_default();
    let withdrawn = matches!(row.state.resolution, ResolutionStatus::Tombstoned);
    let tomb = if withdrawn { "  [withdrawn]" } else { "" };
    format!(
        "{uid}  status={status}  {pair}  frame={frame}  rater={rater}{by}  {date}{note}{tomb}",
        uid = row.uid,
        // The claim-aware join (SL-220 §2): active anchor rows wear their
        // ClaimResolution token; everything else the RowState join.
        status = row.display_token(),
        frame = row.frame,
        rater = row.rater_token,
        date = row.date,
    )
}

/// Filter + render the pipeline's joined rows for `list` (design §4 S2, D13:
/// "the pipeline computes it anyway"). `filter` keeps rows where either side
/// matches the participation ref; `active_only` keeps `ResolutionStatus::
/// Active` rows only — quarantined and no-constraint rows ARE active (design
/// A2), so they still show under `--active-only`. Ordering comes for free
/// from `resolve`'s own total-key sort (`(date, session_uid, seq)`) that
/// `RowSummary`s are already in.
fn list_lines(rows: &[RowSummary], filter: Option<&str>, active_only: bool) -> Vec<String> {
    rows.iter()
        .filter(|r| filter.is_none_or(|id| r.a == id || r.b.as_deref() == Some(id)))
        .filter(|r| !active_only || matches!(r.state.resolution, ResolutionStatus::Active))
        .map(render_row)
        .collect()
}

/// `compare list [<ID>] [--active-only]` — read the ledger back through the
/// SAME resolve → compile pipeline `explain`/`findings` consume (SL-213
/// PHASE-06, design D13): every row joins its [`ResolutionStatus`] with its
/// (Active-only) `CompilationStatus` into ONE display token. A missing
/// comparisons directory is an empty listing, not an error.
fn run_list(args: &ListArgs) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(args.path.clone(), &crate::root::default_markers())?;
    let pipeline = crate::priority::graph::load_comparison_pipeline_for_root(&root)?;
    let lines = list_lines(&pipeline.rows, args.id.as_deref(), args.active_only);

    let mut out = std::io::stdout();
    if lines.is_empty() {
        writeln!(out, "compare: no judgements recorded")?;
        return Ok(());
    }
    for line in lines {
        writeln!(out, "{line}")?;
    }
    Ok(())
}

/// `compare withdraw <ROW-UID> [--note]` — append-only withdrawal. Scans the
/// ledger to confirm the uid names a live judgement row (unknown / tombstone-row
/// / already-withdrawn each refused), then writes a NEW session-of-one file
/// carrying only the tombstone — never editing an existing file (design D2, the
/// same create-new seam as capture).
fn run_withdraw(args: &WithdrawArgs) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(args.path.clone(), &crate::root::default_markers())?;
    let sessions = load_sessions(&root)?;

    // The uid must name a live judgement row.
    let Some(target_session) = sessions
        .iter()
        .find(|s| s.judgements.iter().any(|j| j.uid == args.uid))
    else {
        // Distinguish a tombstone-row uid from a wholly-unknown uid.
        let is_tombstone = sessions
            .iter()
            .any(|s| s.tombstones.iter().any(|t| t.uid == args.uid));
        if is_tombstone {
            anyhow::bail!(
                "{} is a tombstone, not a judgement — withdraw targets judgement rows",
                args.uid
            );
        }
        anyhow::bail!("unknown row uid `{}` — no judgement carries it", args.uid);
    };

    // Idempotency: refuse a second withdrawal of the same row.
    let already = sessions
        .iter()
        .any(|s| s.tombstones.iter().any(|t| t.target == args.uid));
    if already {
        anyhow::bail!("{} is already withdrawn", args.uid);
    }

    // Impure edge: fresh session/tombstone uids + today.
    let session_uid = uuid::Uuid::now_v7().to_string();
    let tombstone_uid = uuid::Uuid::now_v7().to_string();
    let date = crate::clock::today();

    // Ride the target file's wire discriminators, so the tombstone matches the
    // ledger version it withdraws from (no hand-set version magic).
    let session = ComparisonSession {
        schema: target_session.schema.clone(),
        version: target_session.version,
        session: SessionHeader {
            uid: session_uid.clone(),
            date: date.clone(),
            audience: None,
        },
        judgements: Vec::new(),
        tombstones: vec![Tombstone {
            uid: tombstone_uid,
            seq: 0,
            target: args.uid.clone(),
            date: date.clone(),
            note: args.note.clone(),
        }],
    };
    let text = comparison::to_toml(&session)?;
    let path = write_session_file(&root, &date, &session_uid, &text)?;

    writeln!(
        std::io::stdout(),
        "compare: withdrew {} — tombstone {}",
        args.uid,
        path.display()
    )?;
    Ok(())
}

/// `compare elicit` — assemble the ranked queue over the SAME resolve→compile
/// pipeline the read surfaces consume, then render (design §3). Read-only (D18):
/// no session or authored write on any path. The `PriorityGraph` supplies the
/// frontier ranking + per-item costing (D6); the comparison `Pipeline` supplies
/// the active evidence + anchors + projection the assembler recompiles over.
fn run_elicit(args: &ElicitArgs) -> anyhow::Result<()> {
    let root = crate::root::find(args.path.clone(), &crate::root::default_markers())?;
    let cfg = crate::priority::config::load(&root);
    let graph = graph::build(&root)?;
    let pipeline = graph::load_comparison_pipeline_for_root(&root)?;
    let depth = args.depth.unwrap_or(cfg.elicit.depth).max(1);

    let inputs = build_elicit_inputs(&graph, &pipeline, &cfg, depth);
    let queue = assemble(&inputs, DecisionContext::Sequencing { depth });

    let ctx = RenderCtx::new(&graph, &pipeline, &cfg);
    if args.json {
        render_elicit_json(&queue, depth, args, &ctx)
    } else {
        render_elicit_human(&queue, args, &ctx)
    }
}

/// Compose the pure [`ElicitInputs`] from the built graph + loaded pipeline
/// (design §2 shell seam). The frontier is the top-`depth` band by final score
/// (id-lex tiebreak — determinism); costing is the D6 `(multiplier, est_cost,
/// bare_estimate)` per entity; active evidence + anchors + projection come
/// straight off the pipeline. Borrows the pipeline's owned pairwise view
/// (`active_pairwise`), so the returned inputs live no longer than `pipeline`.
fn build_elicit_inputs<'a>(
    graph: &PriorityGraph,
    pipeline: &'a comparison::Pipeline,
    cfg: &PriorityConfig,
    depth: usize,
) -> ElicitInputs<'a> {
    // Frontier: final score DESC, canonical-id ASC — the deterministic top-K band.
    let mut ranked: Vec<(&_, f64)> = graph.score.iter().map(|(k, &s)| (k, s)).collect();
    ranked.sort_by(|(ka, sa), (kb, sb)| sb.total_cmp(sa).then_with(|| ka.cmp(kb)));
    let frontier: Vec<FrontierItem> = ranked
        .iter()
        .take(depth)
        .filter_map(|(key, _)| {
            graph.attrs.get(key).map(|attr| FrontierItem {
                id: key.canonical(),
                kind: attr.kind.prefix.to_string(),
            })
        })
        .collect();

    // Costing: D6 multiplier + β-skewed est_cost + bare flag, for every scored entity.
    // The SAME loop harvests the sizing-probe target pool (SL-219 §4): an item
    // with an authored estimate and an estimate-admissible kind contributes its
    // AUTHORED `est_cost` (the ladder's authored branch — never projected/gauge,
    // so the anchored-membership postcondition holds by construction).
    let mut costing: std::collections::BTreeMap<String, ItemCosting> =
        std::collections::BTreeMap::new();
    let mut estimated_costs: std::collections::BTreeMap<String, f64> =
        std::collections::BTreeMap::new();
    for (key, attr) in &graph.attrs {
        if let Some((multiplier, est_cost, bare_estimate)) = graph.item_costing(key, cfg) {
            let canonical = key.canonical();
            costing.insert(
                canonical.clone(),
                ItemCosting {
                    multiplier,
                    est_cost,
                    bare_estimate,
                },
            );
            // SL-222 PHASE-06: the target pool also includes anchored-tier
            // estimate claim costs (Pin/Human from estimate_claims). These
            // replace the old facet-anchor targets where facets are absent.
            if !bare_estimate
                && comparison::admissible_estimate_pair(attr.kind.prefix, attr.kind.prefix).is_ok()
            {
                estimated_costs.insert(canonical, est_cost);
            }
        }
    }
    // Phase-06: also include anchored-tier claim costs (Pin/Human) — items
    // with claim rows but no authored estimate facet (the claim is the anchor
    // source, not the facet).
    for (item, claim) in &pipeline.estimate_claims.anchored {
        if comparison::admissible_estimate_pair(
            item.as_str().split_once('-').map_or("", |(p, _)| p),
            item.as_str().split_once('-').map_or("", |(p, _)| p),
        )
        .is_ok()
            && !estimated_costs.contains_key(item.as_str())
        {
            estimated_costs.insert(item.clone(), claim.operative);
        }
    }
    // Phase-06: priors (Agent/Migrated) only enter as probe-eligible
    // when knob is SET; when UNSET they retire from probes (the shell's
    // `demote_agent_evidence` knob gates this downstream).
    if cfg.compare.demote_agent_evidence {
        for (item, claim) in &pipeline.estimate_claims.priors {
            if pipeline
                .estimate_claims
                .anchored
                .contains_key(item.as_str())
            {
                continue;
            }
            let prefix = item.as_str().split_once('-').map_or("", |(p, _)| p);
            if comparison::admissible_estimate_pair(prefix, prefix).is_ok()
                && !estimated_costs.contains_key(item.as_str())
            {
                estimated_costs.insert(item.clone(), claim.operative);
            }
        }
    }

    // Est-evidenced set (SL-219 §4 pool predicate): items touched by ANY
    // resolved-active est-domain row, any compilation status. An `incomparable`
    // row (→ NoConstraint) still lands here and retires the probe.
    let mut est_evidenced: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for row in &pipeline.rows {
        if row.domain == comparison::DOMAIN_ESTIMATE
            && matches!(row.state.resolution, comparison::ResolutionStatus::Active)
        {
            est_evidenced.insert(row.a.clone());
            if let Some(b) = &row.b {
                est_evidenced.insert(b.clone());
            }
        }
    }

    // SL-220 §3 rung-5 set: items carrying an unmigrated `[value]` facet —
    // valued for queue purposes like a prior. Deleted at PHASE-09 (the raw
    // facet is no longer parsed); always empty.
    let facet_valued: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    ElicitInputs {
        // The pairwise view (SL-220 §2): `assemble` recompiles its baseline
        // from pairwise evidence only — anchor rows terminate at the claims
        // pass and never reach a compile consumer (RV-278 F-6).
        active: pipeline.active_pairwise().iter().collect(),
        anchors: pipeline.value.anchors.clone(),
        frontier,
        costing,
        projection: pipeline.value.projection.clone(),
        rank_decay: cfg.elicit.rank_decay,
        confirm_boost: cfg.elicit.confirm_boost,
        demote_agent_evidence: cfg.compare.demote_agent_evidence,
        sizing: SizingInputs {
            estimated_costs,
            est_evidenced,
            // SL-222 E10: only anchored-tier estimate claims are valid
            // probe targets. The filtered `estimated_costs` (with no
            // facet-only costs) feeds the median, and this set gates which
            // items are eligible.
            estimate_anchored_targets: pipeline.estimate_claims.anchored.keys().cloned().collect(),
        },
        // SL-220 §3/D14: the claim facts — probe retirement + the
        // knob-independent claim-reprobe source.
        claims: pipeline.value_claims.clone(),
        facet_valued,
    }
}

/// The display slice after the POST-ranking `--kind` filter + `--limit` display
/// cap (the full pool is ranked; only the view is capped, design §3).
fn displayed<'q>(queue: &'q ElicitQueue, args: &ElicitArgs) -> Vec<&'q QueueEntry> {
    let limit = args.limit.unwrap_or(crate::priority::config::ELICIT_LIMIT);
    queue
        .entries
        .iter()
        .filter(|e| args.kind.is_none_or(|k| k.matches(e.kind)))
        .take(limit)
        .collect()
}

/// The bare-estimate mask glyph (D17) — the ⚠ marker on a projected-but-bare
/// participant (STD-001: one definition for the human render).
const MASK_MARK: &str = "⚠";

/// The D15 state token + footer detail (design §3 / D15 — the precise stall /
/// stable / m=0 wording). Takes the whole queue so `Stable` can scope its claim
/// by the `excluded_value_insensitive` (`m = 0`, D6) count.
fn state_footer(queue: &ElicitQueue) -> (&'static str, String) {
    let sizing = sizing_suffix(queue);
    match &queue.state {
        QueueState::Candidates => (
            "candidates",
            format!("candidates outstanding — comparisons / anchor-reviews worth asking{sizing}"),
        ),
        QueueState::Stalled { depth } => (
            "stalled",
            format!(
                "stalled at depth {depth}: greedy one-step yield exhausted — NOT a stability \
                 claim (a bridge question may remain){sizing}"
            ),
        ),
        QueueState::Stable { depth } => {
            // D15: the claim is INTERNAL order among the CURRENT top-K members —
            // never prefix-membership (D5). Scoped further when m=0 exclusions
            // exist (D6): those pairs are value-insensitive, outside the claim.
            let base = format!(
                "stable: value_dim order among the current top-{depth} frontier members is \
                 stable over the joint set (internal order, current members — not membership)"
            );
            let scope = match queue.excluded_value_insensitive {
                0 => String::new(),
                n => format!("; {n} pair(s) value-insensitive (zero weight), outside the claim"),
            };
            ("stable", format!("{base}{scope}{sizing}"))
        }
    }
}

/// The SL-219 §4 sizing-debt disclosure clause appended to any state line —
/// residual un-sized top-K items (disclosed, never gating) and the fallback
/// tier-2 "no calibration target" note. Empty when there is no sizing debt, so
/// a corpus with no est activity renders byte-identically to pre-SL-219.
fn sizing_suffix(queue: &ElicitQueue) -> String {
    let mut parts: Vec<String> = Vec::new();
    if queue.sizing_residual > 0 {
        parts.push(format!(
            "{} top-K item(s) carry unresolved sizing — costs may move on new evidence",
            queue.sizing_residual
        ));
    }
    if queue.sizing_no_calibration_target {
        parts.push(
            "no estimated item to calibrate against — estimate any item to seed sizing".to_string(),
        );
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("; {}", parts.join("; "))
    }
}

/// Human render (design §3): per entry the rank/kind spine, ask line,
/// participants with fetched context (title/status/S3 value shape/estimate-or-
/// bare ⚠), reasons prose, and the exact answer command; then the D15 footer.
fn render_elicit_human(
    queue: &ElicitQueue,
    args: &ElicitArgs,
    ctx: &RenderCtx<'_>,
) -> anyhow::Result<()> {
    use std::io::Write;
    let mut out = std::io::stdout();
    let shown = displayed(queue, args);
    if shown.is_empty() {
        writeln!(out, "elicit: no candidates")?;
    }
    for (i, entry) in shown.iter().enumerate() {
        write!(out, "{}", entry_human(i + 1, entry, ctx))?;
    }
    let (_, detail) = state_footer(queue);
    writeln!(out, "state: {detail}")?;
    // SL-218 D2: knob-on disclosure — shared fragment, appended only when on
    // so knob-off output stays byte-identical (INV-1).
    if ctx.cfg.compare.demote_agent_evidence {
        writeln!(
            out,
            "{}",
            crate::priority::render::AGENT_DEMOTION_DISCLOSURE
        )?;
    }
    Ok(())
}

/// One entry's human block (design §3): spine line, ask line, participants with
/// fetched context, reasons prose, and the exact answer command. Returns the
/// framed multi-line string (trailing newline) — the render loop concatenates.
fn entry_human(rank: usize, entry: &QueueEntry, ctx: &RenderCtx<'_>) -> String {
    let kind = match entry.kind {
        CandidateKind::Comparison => "comparison",
        CandidateKind::AnchorReview => "anchor-review",
        CandidateKind::SizingProbe => "sizing-probe",
        CandidateKind::ClaimReprobe => "claim-reprobe",
    };
    let mut parts = vec![format!(
        "{rank}. [{kind}] score {:.3}  yield {:+}  impact {:.3}\n",
        entry.score, entry.guaranteed_yield, entry.guaranteed_impact
    )];
    match &entry.payload {
        EntryPayload::Comparison { a, b, .. } => {
            parts.push(format!("   ask: {} vs {}\n", a.id, b.id));
            parts.push(participant_human(ctx, a));
            parts.push(participant_human(ctx, b));
        }
        EntryPayload::AnchorReview { subject, .. } => {
            let anchor = subject
                .anchor
                .map_or_else(|| "—".to_string(), |v| format!("{v}"));
            parts.push(format!(
                "   review: anchor on {} (value {anchor})\n",
                subject.id
            ));
        }
        EntryPayload::SizingProbe { subject, target } => {
            parts.push(format!(
                "   size: {} — which is more work vs {} (est {:.2}, {})?\n",
                subject.id, target.id, target.estimate, target.tier_label
            ));
            parts.push(participant_human(ctx, subject));
        }
        // SL-220 D14 — the minimal spine line; tier-token render is PHASE-06.
        EntryPayload::ClaimReprobe {
            subject,
            mean,
            low,
            high,
            distinct,
            rows,
            ..
        } => {
            parts.push(format!(
                "   reprobe: contested claim on {subject} — mean {mean}, {distinct} distinct \
                 magnitudes ({low} ‥ {high}) over {rows} rows\n"
            ));
        }
    }
    let reasons: Vec<&str> = entry.reasons.iter().map(|r| r.text.as_str()).collect();
    if !reasons.is_empty() {
        parts.push(format!("   reasons: {}\n", reasons.join("; ")));
    }
    parts.push(format!("   answer: {}\n", answer_command(entry)));
    parts.concat()
}

/// One participant's human context lines (design §3): id + title + status, then
/// the S3 value shape and the estimate cell; a trailing ⚠-mask line when the
/// engine flagged the participant (bare estimate masks the projection, D17).
fn participant_human(ctx: &RenderCtx<'_>, p: &Participant) -> String {
    let (title, status) = ctx.context(&p.id).unwrap_or(("", None));
    let status = status.unwrap_or("—");
    let value = ctx
        .value_fragment(&p.id)
        .unwrap_or_else(|| "value —".to_string());
    let est = ctx.estimate_display(&p.id);
    let mut parts = vec![format!(
        "   • {} \"{title}\" ({status})\n     {value}  ·  {est}\n",
        p.id
    )];
    if !p.annotations.is_empty() {
        parts.push(format!("     {MASK_MARK} {}\n", p.annotations.join("; ")));
    }
    parts.concat()
}

/// The exact answer command a curator runs to record the judgement (design §3 —
/// no TTY, the command IS the capture leg). Comparison → the `compare record`
/// call over the pair; anchor-review → the revise/uphold resolving actions
/// (mirrors the JSON `exits`).
fn answer_command(entry: &QueueEntry) -> String {
    match &entry.payload {
        EntryPayload::Comparison { a, b, .. } => format!(
            "doctrine compare record {} {} --prefer a   (| --prefer b | --equal | --incomparable)",
            a.id, b.id
        ),
        EntryPayload::AnchorReview { subject, .. } => format!(
            "doctrine value set {} <v>   (revise)   |   supersede/withdraw the cited rows (uphold)",
            subject.id
        ),
        EntryPayload::SizingProbe { subject, target } => format!(
            "doctrine compare record {} {} --prefer a   (| --prefer b | --equal | --incomparable) \
             --frame more-work",
            subject.id, target.id
        ),
        // The resolving action for a contested claim is a superseding row
        // (design §4 supersession scope; correction is supersession).
        EntryPayload::ClaimReprobe { subject, .. } => format!(
            "doctrine value set {subject} <v> --rater human --supersedes <row-uid>   \
             (or `value pin` the settled magnitude)"
        ),
    }
}

/// The join context the elicit render needs beyond the pure [`ElicitQueue`]:
/// per-participant value source + estimate come from a shell join
/// (`Participant.id → PriorityGraph / Pipeline`), since `assemble` keeps
/// participants LEAN (design D16). Built once per render from the same graph +
/// pipeline `run_elicit` already loaded (no second scan).
struct RenderCtx<'a> {
    graph: &'a PriorityGraph,
    pipeline: &'a comparison::Pipeline,
    cfg: &'a PriorityConfig,
    keys: std::collections::BTreeMap<String, crate::relation_graph::EntityKey>,
}

impl<'a> RenderCtx<'a> {
    fn new(
        graph: &'a PriorityGraph,
        pipeline: &'a comparison::Pipeline,
        cfg: &'a PriorityConfig,
    ) -> Self {
        let keys = graph.attrs.keys().map(|k| (k.canonical(), *k)).collect();
        Self {
            graph,
            pipeline,
            cfg,
            keys,
        }
    }

    /// The structural value block for a participant (design §3 / D16):
    /// `{provenance, point}` from the SINGLE value-source precedence
    /// (`surface::value_source_reason` — the SL-220 §3 evidence ladder) with
    /// the D11 token map (`value_source_token`) naming the provenance, plus
    /// the STRUCTURAL bounds (`surface::class_bounds_structural`,
    /// open/closed/unbounded retained — the human `explain` path flattens,
    /// this surface must not, web review). `None` when the entity has no
    /// citable value source (an un-constrained bare item — the numeric
    /// default floor is not a source, surface.rs S3).
    fn value_block(&self, canonical: &str) -> Option<serde_json::Value> {
        let key = *self.keys.get(canonical)?;
        let reason = crate::priority::surface::value_source_reason(self.graph, key, self.pipeline)?;
        let provenance = reason.value_source_token()?;
        let point = match &reason {
            ReasonKind::ValuePin { value, .. }
            | ReasonKind::ValueClaim { value, .. }
            | ReasonKind::ValueProjected { value, .. }
            | ReasonKind::ValueGauge { value, .. } => *value,
            ReasonKind::ValueUnmigratedFacet => 0.0,
            _ => return None,
        };
        let bounds = crate::priority::surface::class_bounds_structural(
            &self.pipeline.value.constraint_set,
            canonical,
        );
        Some(value_block_json(provenance, point, bounds))
    }

    /// The participant's scalar cost estimate (`est_cost`, D7) — `None` when the
    /// estimate is BARE (no authored facet); the bare case is disclosed instead
    /// by the `projection masked by bare estimate` annotation on the participant
    /// (D17), never by a synthesized number here.
    fn estimate(&self, canonical: &str) -> Option<f64> {
        let key = *self.keys.get(canonical)?;
        let (_, est_cost, bare) = self.graph.item_costing(&key, self.cfg)?;
        (!bare).then_some(est_cost)
    }

    /// Fetched display context for the human render (design §3): the entity's
    /// title + status. `None` for an id absent from the graph (defensive — a
    /// participant is always a scored entity).
    fn context(&self, canonical: &str) -> Option<(&str, Option<&str>)> {
        let key = self.keys.get(canonical)?;
        let attr = self.graph.attrs.get(key)?;
        Some((attr.title.as_str(), attr.status.as_deref()))
    }

    /// The human S3 value-source fragment — reuses `render::value_source_fragment`
    /// verbatim (the SINGLE value-shape template, shared with `explain`). `None`
    /// when the entity has no citable value source.
    fn value_fragment(&self, canonical: &str) -> Option<String> {
        let key = *self.keys.get(canonical)?;
        let reason = crate::priority::surface::value_source_reason(self.graph, key, self.pipeline)?;
        crate::priority::render::value_source_fragment(&reason)
    }

    /// The human estimate cell: the scalar `est_cost`, or a dash when bare (the
    /// ⚠ mask itself rides the participant annotations, D17 — not re-derived here).
    fn estimate_display(&self, canonical: &str) -> String {
        match self
            .keys
            .get(canonical)
            .and_then(|k| self.graph.item_costing(k, self.cfg))
        {
            Some((_, est, false)) => format!("est {est:.2}"),
            _ => "est —".to_string(),
        }
    }

    /// One lean participant (design D16): id + value/estimate block + the
    /// engine's annotations (the bare-estimate mask rides these).
    fn participant_json(&self, p: &Participant) -> serde_json::Value {
        serde_json::json!({
            "id": p.id,
            "value": self.value_block(&p.id),
            "estimate": self.estimate(&p.id),
            "annotations": p.annotations,
        })
    }
}

/// JSON schema-v1 envelope (design §3 / D16). Byte-stable (`BTree` key order
/// from `serde_json::Value`); NO trailing newline (the golden contract, mirrors
/// `render.rs::finish`).
fn render_elicit_json(
    queue: &ElicitQueue,
    depth: usize,
    args: &ElicitArgs,
    ctx: &RenderCtx<'_>,
) -> anyhow::Result<()> {
    use std::io::Write;
    let text = serde_json::to_string_pretty(&elicit_envelope(queue, depth, args, ctx))?;
    write!(std::io::stdout(), "{text}")?;
    Ok(())
}

/// The schema-v1 envelope value (pure — the byte-stability + golden anchor).
fn elicit_envelope(
    queue: &ElicitQueue,
    depth: usize,
    args: &ElicitArgs,
    ctx: &RenderCtx<'_>,
) -> serde_json::Value {
    let (state_token, state_detail) = state_footer(queue);
    let entries: Vec<serde_json::Value> = displayed(queue, args)
        .iter()
        .enumerate()
        .map(|(i, e)| entry_json(i + 1, e, ctx))
        .collect();
    let mut envelope = serde_json::json!({
        "schema": "doctrine.elicit-queue",
        "version": 1,
        "context": { "kind": "sequencing", "depth": depth },
        "state": state_token,
        "state_detail": state_detail,
        "excluded_value_insensitive": queue.excluded_value_insensitive,
        "entries": entries,
    });
    // SL-218 D2: ADDITIVE key, knob-on only (knob-off bytes identical, INV-1).
    if ctx.cfg.compare.demote_agent_evidence
        && let Some(obj) = envelope.as_object_mut()
    {
        obj.insert(
            "agent_evidence_demoted".to_string(),
            serde_json::Value::Bool(true),
        );
    }
    envelope
}

/// One entry's JSON (spine + kind payload + kind-specific ask). Reason/ask codes
/// ride the engine's structured shapes verbatim (D16 findings-parity).
fn entry_json(rank: usize, entry: &QueueEntry, ctx: &RenderCtx<'_>) -> serde_json::Value {
    let kind = match entry.kind {
        CandidateKind::Comparison => "comparison",
        CandidateKind::AnchorReview => "anchor-review",
        CandidateKind::SizingProbe => "sizing-probe",
        CandidateKind::ClaimReprobe => "claim-reprobe",
    };
    let reasons: Vec<serde_json::Value> = entry
        .reasons
        .iter()
        .map(|r| serde_json::json!({ "code": r.code, "text": r.text }))
        .collect();
    let payload = match &entry.payload {
        EntryPayload::Comparison { a, b, ask } => serde_json::json!({
            "participants": [ctx.participant_json(a), ctx.participant_json(b)],
            "ask": comparison_ask_json(ask),
        }),
        EntryPayload::AnchorReview { subject, ask } => serde_json::json!({
            "subject": {
                "id": subject.id,
                "anchor": subject.anchor,
                "conflict_pairs": subject.conflict_pairs,
                "quarantined_rows": subject.quarantined_rows,
            },
            "ask": anchor_ask_json(ask, subject),
        }),
        EntryPayload::SizingProbe { subject, target } => sizing_probe_json(subject, target),
        // SL-220 D14 — minimal payload; tier-token render detail is PHASE-06.
        EntryPayload::ClaimReprobe {
            subject,
            tier,
            mean,
            low,
            high,
            distinct,
            rows,
        } => serde_json::json!({
            "subject": { "id": subject },
            "tier": format!("{tier:?}").to_lowercase(),
            "mean": mean,
            "interval": { "low": low, "high": high, "distinct": distinct },
            "rows": rows,
        }),
    };
    let mut obj = serde_json::json!({
        "rank": rank,
        "kind": kind,
        "guaranteed_yield": entry.guaranteed_yield,
        "guaranteed_impact": entry.guaranteed_impact,
        "score": entry.score,
        "yield_basis": match entry.yield_basis {
            crate::priority::elicit::YieldBasis::OrderBearingAnswers => "order-bearing-answers",
            crate::priority::elicit::YieldBasis::CanonicalResolvingActions => {
                "canonical-resolving-actions"
            }
            crate::priority::elicit::YieldBasis::Calibration => "calibration",
        },
        "reasons": reasons,
    });
    if let (Some(map), serde_json::Value::Object(pmap)) = (obj.as_object_mut(), payload) {
        for (k, v) in pmap {
            map.insert(k, v);
        }
    }
    obj
}

/// The comparison ask block (design §3): the canonical equal-effort/value frame
/// (D1 — comparison entries carry no other frame), the answer tokens, and each
/// token's disclosed yield. No `yield_note`/`exits` (those are anchor-only).
fn comparison_ask_json(ask: &crate::priority::elicit::AskSpec) -> serde_json::Value {
    serde_json::json!({
        "frame": FRAME_EQUAL_EFFORT,
        "domain": comparison::DOMAIN_VALUE,
        "answers": ask.answers,
        "yield_by_answer": ask.yield_by_answer,
    })
}

/// The sizing-probe payload (SL-219 §4): a lean `subject { id, annotations }`,
/// the `target { id, estimate }` (the median authored anchor sized against), and
/// the invariant `ask`. NO `yield_by_answer` — a probe makes no yield claim; no
/// hypothetical machinery ran. Additive kind, schema stays v1 (consumers switch
/// on `kind`).
fn sizing_probe_json(
    subject: &Participant,
    target: &crate::priority::elicit::SizingTarget,
) -> serde_json::Value {
    serde_json::json!({
        "subject": { "id": subject.id, "annotations": subject.annotations },
        "target": { "id": target.id, "estimate": target.estimate },
        "ask": sizing_ask_json(),
    })
}

/// The sizing-probe ask block (SL-219 §4): the `more-work` frame, `estimate`
/// domain, and the invariant answer tokens keyed off the ONE elicit definition
/// (STD-001). No per-answer yield.
fn sizing_ask_json() -> serde_json::Value {
    serde_json::json!({
        "frame": comparison::FRAME_MORE_WORK,
        "domain": comparison::DOMAIN_ESTIMATE,
        "answers": crate::priority::elicit::SIZING_PROBE_ANSWERS,
    })
}

/// The anchor-review ask block (design §3, RV-269 F-3): answers + per-answer
/// yield + the conditional-yield `yield_note` + `exits` (suggested resolving
/// actions per answer token; uphold is not one executable command).
fn anchor_ask_json(
    ask: &crate::priority::elicit::AskSpec,
    subject: &crate::priority::elicit::AnchorSubject,
) -> serde_json::Value {
    serde_json::json!({
        "answers": ask.answers,
        "yield_by_answer": ask.yield_by_answer,
        "yield_note": ask.yield_note,
        "exits": anchor_exits_json(subject),
    })
}

/// The per-answer suggested resolving actions for an anchor-review entry
/// (design §3): `revise-anchor` re-authors the suspect value; `uphold-anchor`
/// retires the cited closure (supersede or withdraw each quarantined row) —
/// arrays of suggested commands, not a single executable, keyed by answer token
/// (STD-001: tokens shared with `elicit`).
fn anchor_exits_json(subject: &crate::priority::elicit::AnchorSubject) -> serde_json::Value {
    let revise = vec![format!("doctrine value set {} <v>", subject.id)];
    let mut uphold: Vec<String> = Vec::new();
    if let Some(first) = subject.quarantined_rows.first() {
        uphold.push(format!(
            "doctrine compare record <a> <b> <response> --supersedes {first}"
        ));
    }
    for uid in &subject.quarantined_rows {
        uphold.push(format!("doctrine compare withdraw {uid}"));
    }
    serde_json::json!({
        crate::priority::elicit::ANSWER_REVISE_ANCHOR: revise,
        crate::priority::elicit::ANSWER_UPHOLD_ANCHOR: uphold,
    })
}

/// A structural C6 bound as `{kind, value?}` (design §3): mirrors the `Bound`
/// enum so the open/closed distinction survives (`[null, 2.8]` would lose it —
/// web review).
fn bound_json(b: comparison::Bound) -> serde_json::Value {
    match b {
        comparison::Bound::Unbounded => serde_json::json!({ "kind": "unbounded" }),
        comparison::Bound::Open(v) => serde_json::json!({ "kind": "open", "value": v }),
        comparison::Bound::Closed(v) => serde_json::json!({ "kind": "closed", "value": v }),
    }
}

/// The participant value block (design §3): `{provenance, point}` always, plus
/// structural `bounds` when the entity sits in a compiled class.
fn value_block_json(
    provenance: &str,
    point: f64,
    bounds: Option<comparison::ValueBounds>,
) -> serde_json::Value {
    let mut v = serde_json::json!({ "provenance": provenance, "point": point });
    if let (Some(b), Some(map)) = (bounds, v.as_object_mut()) {
        map.insert(
            "bounds".to_string(),
            serde_json::json!({ "lower": bound_json(b.lower), "upper": bound_json(b.upper) }),
        );
    }
    v
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    /// Create a tempdir `root::find` resolves as a project root (facet.rs house
    /// pattern).
    fn mk_project_root() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(".project"), "").unwrap();
        std::fs::create_dir_all(tmp.path().join(".doctrine")).unwrap();
        std::fs::write(tmp.path().join(crate::dtoml::DOCTRINE_TOML), "").unwrap();
        let root = tmp.path().to_path_buf();
        (tmp, root)
    }

    /// Seed a minimal resolvable entity of `prefix` with `status`, returning its
    /// canonical id. Rides `entity::id_path` + `integrity::kind_by_prefix`, so it
    /// works for any numbered kind regardless of its tree/stem layout.
    fn seed_entity(root: &Path, prefix: &str, id: u32, status: &str) -> String {
        let padded = format!("{id:03}");
        let kref = crate::integrity::kind_by_prefix(prefix).expect("valid prefix");
        let toml_path = crate::entity::id_path(root, kref.kind, id, crate::entity::Ext::Toml);
        std::fs::create_dir_all(toml_path.parent().unwrap()).unwrap();
        std::fs::write(
            &toml_path,
            format!(
                "id = {id}\nslug = \"t{padded}\"\ntitle = \"Test {prefix}-{padded}\"\nstatus = \"{status}\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n"
            ),
        )
        .unwrap();
        crate::listing::canonical_id(prefix, id)
    }

    /// The `.toml` session files currently under `.doctrine/comparisons/`.
    fn session_files(root: &Path) -> Vec<PathBuf> {
        let dir = root.join(".doctrine").join(COMPARISONS_DIR);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            return Vec::new();
        };
        let mut files: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "toml"))
            .collect();
        files.sort();
        files
    }

    /// A bare `record` capture over two refs, defaults elsewhere.
    fn capture(root: &Path, a: &str, b: &str, prefer: &str) -> RecordArgs {
        RecordArgs {
            a: a.to_string(),
            b: b.to_string(),
            prefer: Some(prefer.to_string()),
            equal: false,
            incomparable: false,
            supersedes: None,
            frame: FrameArg::EqualEffort,
            rater: RaterArg::Agent,
            by: None,
            lens: None,
            note: None,
            audience: None,
            path: Some(root.to_path_buf()),
        }
    }

    #[test]
    fn compare_capture_writes_session_of_one() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 204, "accepted");
        seed_entity(&root, "IMP", 118, "accepted");

        run_capture(&capture(&root, "SL-204", "IMP-118", "a")).unwrap();

        let files = session_files(&root);
        assert_eq!(files.len(), 1, "exactly one session file");
        let name = files[0].file_name().unwrap().to_string_lossy();
        assert!(
            name.ends_with(".toml") && name.contains('-'),
            "filename is <date>-<uid>.toml: {name}"
        );

        let session = comparison::parse(&std::fs::read_to_string(&files[0]).unwrap()).unwrap();
        assert_eq!(session.judgements.len(), 1);
        assert!(session.tombstones.is_empty());
        let j = &session.judgements[0];
        assert_eq!(j.seq, 0);
        assert_eq!(j.a, "SL-204");
        assert_eq!(j.b.as_deref(), Some("IMP-118"));
        assert_eq!(j.response, Some(Response::PreferA));
        assert_eq!(j.domain, comparison::DOMAIN_VALUE);
        assert_eq!(j.frame, FRAME_EQUAL_EFFORT);
        assert_eq!(j.form, RowForm::Order);
        assert_eq!(j.rater, RaterKind::Agent);
        assert_eq!(j.magnitude, None);
        assert_eq!(j.supersedes, None);
    }

    #[test]
    fn second_capture_never_touches_first_file() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 204, "accepted");
        seed_entity(&root, "IMP", 118, "accepted");

        run_capture(&capture(&root, "SL-204", "IMP-118", "a")).unwrap();
        let first = session_files(&root);
        assert_eq!(first.len(), 1);
        let first_path = first[0].clone();
        let first_bytes = std::fs::read(&first_path).unwrap();

        run_capture(&capture(&root, "SL-204", "IMP-118", "b")).unwrap();
        let second = session_files(&root);
        assert_eq!(second.len(), 2, "a fresh file per invocation");
        assert!(first_path.exists(), "the first file survives");
        assert_eq!(
            std::fs::read(&first_path).unwrap(),
            first_bytes,
            "the first file is byte-identical after the second capture"
        );
    }

    #[test]
    fn compare_refuses_missing_ref() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 204, "accepted");
        // SL-999 is well-formed but absent.
        let err = run_capture(&capture(&root, "SL-204", "SL-999", "a")).unwrap_err();
        assert!(!err.to_string().is_empty());
        assert!(session_files(&root).is_empty(), "no file on a dangling ref");
    }

    #[test]
    fn compare_refuses_record_pair() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "QUE", 1, "open");
        seed_entity(&root, "QUE", 2, "open");
        let err = run_capture(&capture(&root, "QUE-001", "QUE-002", "a")).unwrap_err();
        assert!(
            err.to_string().contains("not value-bearing"),
            "admissibility reason surfaced: {err}"
        );
        assert!(session_files(&root).is_empty(), "no file on a record pair");
    }

    #[test]
    fn compare_refuses_rsk_participant() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 204, "accepted");
        seed_entity(&root, "RSK", 1, "open");
        let err = run_capture(&capture(&root, "SL-204", "RSK-001", "a")).unwrap_err();
        assert!(
            err.to_string().contains("excluded from value comparison"),
            "RSK admissibility reason surfaced: {err}"
        );
        assert!(
            session_files(&root).is_empty(),
            "no file on an RSK participant"
        );
    }

    #[test]
    fn compare_admits_terminal_status_participant() {
        let (_tmp, root) = mk_project_root();
        // A closed slice — existence only, no status gate (design).
        seed_entity(&root, "SL", 204, "closed");
        seed_entity(&root, "IMP", 118, "accepted");
        run_capture(&capture(&root, "SL-204", "IMP-118", "a")).unwrap();
        assert_eq!(
            session_files(&root).len(),
            1,
            "terminal-status participant admitted"
        );
    }

    #[test]
    fn flag_surface_lands_frame_rater_by() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 204, "accepted");
        seed_entity(&root, "IMP", 118, "accepted");

        let args = RecordArgs {
            frame: FrameArg::PreferFirst,
            rater: RaterArg::Human,
            by: Some("david".to_string()),
            lens: Some("user-value".to_string()),
            note: Some("auth unblocks the pilot".to_string()),
            ..capture(&root, "SL-204", "IMP-118", "a")
        };
        run_capture(&args).unwrap();

        let files = session_files(&root);
        let session = comparison::parse(&std::fs::read_to_string(&files[0]).unwrap()).unwrap();
        let j = &session.judgements[0];
        assert_eq!(j.frame, FRAME_PREFER_FIRST);
        assert_eq!(j.rater, RaterKind::Human);
        assert_eq!(j.by.as_deref(), Some("david"));
        assert_eq!(j.lens.as_deref(), Some("user-value"));
        assert_eq!(j.note.as_deref(), Some("auth unblocks the pilot"));
    }

    #[test]
    fn prefer_literal_shorthand() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 204, "accepted");
        seed_entity(&root, "IMP", 118, "accepted");

        run_capture(&capture(&root, "SL-204", "IMP-118", "a")).unwrap();
        run_capture(&capture(&root, "SL-204", "IMP-118", "b")).unwrap();

        let mut responses: Vec<Response> = session_files(&root)
            .iter()
            .map(|p| {
                comparison::parse(&std::fs::read_to_string(p).unwrap())
                    .unwrap()
                    .judgements[0]
                    .response
                    .unwrap()
            })
            .collect();
        responses.sort_by_key(|r| format!("{r:?}"));
        assert_eq!(responses, vec![Response::PreferA, Response::PreferB]);
    }

    #[test]
    fn prefer_full_ref_resolves() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 204, "accepted");
        seed_entity(&root, "IMP", 118, "accepted");
        run_capture(&capture(&root, "SL-204", "IMP-118", "IMP-118")).unwrap();
        let files = session_files(&root);
        let session = comparison::parse(&std::fs::read_to_string(&files[0]).unwrap()).unwrap();
        assert_eq!(session.judgements[0].response, Some(Response::PreferB));
    }

    /// S1: `--equal` and `--incomparable` land their wire responses; the
    /// default frame keeps the value domain.
    #[test]
    fn equal_and_incomparable_capture() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 204, "accepted");
        seed_entity(&root, "IMP", 118, "accepted");

        let equal = RecordArgs {
            prefer: None,
            equal: true,
            ..capture(&root, "SL-204", "IMP-118", "unused")
        };
        run_capture(&equal).unwrap();

        let incomparable = RecordArgs {
            prefer: None,
            incomparable: true,
            ..capture(&root, "SL-204", "IMP-118", "unused")
        };
        run_capture(&incomparable).unwrap();

        let mut responses: Vec<Response> = session_files(&root)
            .iter()
            .map(|p| {
                let s = comparison::parse(&std::fs::read_to_string(p).unwrap()).unwrap();
                assert_eq!(s.judgements[0].domain, comparison::DOMAIN_VALUE);
                s.judgements[0].response.unwrap()
            })
            .collect();
        responses.sort_by_key(|r| format!("{r:?}"));
        assert_eq!(responses, vec![Response::Equal, Response::Incomparable]);
    }

    /// S1: `--frame prefer-first` silently derives `domain = priority` —
    /// users never type a domain (VT-3).
    #[test]
    fn prefer_first_frame_derives_priority_domain() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 204, "accepted");
        seed_entity(&root, "IMP", 118, "accepted");

        let args = RecordArgs {
            frame: FrameArg::PreferFirst,
            ..capture(&root, "SL-204", "IMP-118", "a")
        };
        run_capture(&args).unwrap();

        let files = session_files(&root);
        let session = comparison::parse(&std::fs::read_to_string(&files[0]).unwrap()).unwrap();
        let j = &session.judgements[0];
        assert_eq!(j.domain, comparison::DOMAIN_PRIORITY);
        assert_eq!(j.frame, FRAME_PREFER_FIRST);
    }

    /// SL-219 VT-2: `--frame more-work` derives `domain = estimate` (e2e-lite
    /// through `run_capture`) — the captured session-of-one carries the
    /// estimate domain and the more-work frame.
    #[test]
    fn more_work_frame_captures_estimate_domain_row() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 204, "accepted");
        seed_entity(&root, "IMP", 118, "accepted");

        let args = RecordArgs {
            frame: FrameArg::MoreWork,
            ..capture(&root, "SL-204", "IMP-118", "a")
        };
        run_capture(&args).unwrap();

        let files = session_files(&root);
        let session = comparison::parse(&std::fs::read_to_string(&files[0]).unwrap()).unwrap();
        let j = &session.judgements[0];
        assert_eq!(j.domain, comparison::DOMAIN_ESTIMATE);
        assert_eq!(j.frame, comparison::FRAME_MORE_WORK);
    }

    /// SL-219 VT-2: capture routes admissibility by the frame-DERIVED domain —
    /// a record-kind pair goes through `admissible_estimate_pair` under
    /// `more-work` (admitted), and `admissible_value_pair` under the default
    /// value frame (refused).
    #[test]
    fn record_kind_pair_admitted_in_estimate_domain_refused_in_value() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "QUE", 1, "open");
        seed_entity(&root, "QUE", 2, "open");

        // Value domain (default frame): refused.
        let err = run_capture(&capture(&root, "QUE-001", "QUE-002", "a")).unwrap_err();
        assert!(
            err.to_string().contains("not value-bearing"),
            "value-domain refusal surfaced: {err}"
        );
        assert!(session_files(&root).is_empty(), "no file on a refusal");

        // Estimate domain (more-work frame): admitted.
        let args = RecordArgs {
            frame: FrameArg::MoreWork,
            ..capture(&root, "QUE-001", "QUE-002", "a")
        };
        run_capture(&args).unwrap();
        assert_eq!(
            session_files(&root).len(),
            1,
            "record pair admitted in the estimate domain"
        );
    }

    /// SL-219 D3: RSK participates in estimate comparison (settle-cost is
    /// comparable even where value is not) — the same participant the value
    /// domain refuses.
    #[test]
    fn rsk_participant_admitted_in_estimate_domain() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 204, "accepted");
        seed_entity(&root, "RSK", 1, "open");

        let args = RecordArgs {
            frame: FrameArg::MoreWork,
            ..capture(&root, "SL-204", "RSK-001", "a")
        };
        run_capture(&args).unwrap();
        assert_eq!(
            session_files(&root).len(),
            1,
            "RSK admitted in the estimate domain"
        );
    }

    /// S1: `--supersedes` with an unknown uid is a hard error before any
    /// write; a known judgement uid lands in the row (VT-3).
    #[test]
    fn supersedes_validated_against_corpus() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 204, "accepted");
        seed_entity(&root, "IMP", 118, "accepted");

        // Unknown uid — refused, nothing written.
        let unknown = RecordArgs {
            supersedes: Some("not-a-real-uid".to_string()),
            ..capture(&root, "SL-204", "IMP-118", "a")
        };
        let err = run_capture(&unknown).unwrap_err();
        assert!(
            err.to_string().contains("names no judgement row"),
            "unknown supersedes target refused: {err}"
        );
        assert!(session_files(&root).is_empty(), "no file on a bad target");

        // Known uid — the edge lands in the new row.
        run_capture(&capture(&root, "SL-204", "IMP-118", "a")).unwrap();
        let target = only_judgement_uid(&root);
        let known = RecordArgs {
            supersedes: Some(target.clone()),
            ..capture(&root, "SL-204", "IMP-118", "b")
        };
        run_capture(&known).unwrap();

        let superseding: Vec<Option<String>> = session_files(&root)
            .iter()
            .map(|p| {
                comparison::parse(&std::fs::read_to_string(p).unwrap())
                    .unwrap()
                    .judgements[0]
                    .supersedes
                    .clone()
            })
            .collect();
        assert!(
            superseding.contains(&Some(target)),
            "the superseding row carries the target uid"
        );
    }

    #[test]
    fn prefer_bogus_value_refused() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 204, "accepted");
        seed_entity(&root, "IMP", 118, "accepted");
        let err = run_capture(&capture(&root, "SL-204", "IMP-118", "SL-999")).unwrap_err();
        assert!(err.to_string().contains("--prefer"), "got: {err}");
        assert!(session_files(&root).is_empty(), "no file on a bad --prefer");
    }

    #[test]
    fn compare_refuses_bare_ref() {
        // D4: bare ids are refused before any resolution.
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 204, "accepted");
        seed_entity(&root, "IMP", 118, "accepted");
        let err = run_capture(&capture(&root, "204", "IMP-118", "a")).unwrap_err();
        assert!(!err.to_string().is_empty());
        assert!(session_files(&root).is_empty(), "no file on a bare ref");
    }

    #[test]
    fn audience_lands_in_session_header() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 204, "accepted");
        seed_entity(&root, "IMP", 118, "accepted");

        // Present → lands in [session].
        let with_aud = RecordArgs {
            audience: Some("stakeholder".to_string()),
            ..capture(&root, "SL-204", "IMP-118", "a")
        };
        run_capture(&with_aud).unwrap();
        let files = session_files(&root);
        let text = std::fs::read_to_string(&files[0]).unwrap();
        let session = comparison::parse(&text).unwrap();
        assert_eq!(session.session.audience.as_deref(), Some("stakeholder"));

        // Absent → absent field (no `audience` key).
        std::fs::remove_file(&files[0]).unwrap();
        run_capture(&capture(&root, "SL-204", "IMP-118", "a")).unwrap();
        let files = session_files(&root);
        let text = std::fs::read_to_string(&files[0]).unwrap();
        assert!(
            !text.contains("audience"),
            "no audience key when flag absent:\n{text}"
        );
        let session = comparison::parse(&text).unwrap();
        assert!(session.session.audience.is_none());
    }

    /// EX-4: the fallback clap shape parses `record` capture and the `list` /
    /// `withdraw` subcommands. A local Parser wraps the `compare` subcommand
    /// group so the shape is exercised in isolation from the top-level `Command`
    /// enum.
    #[test]
    fn clap_shape_accepts_record_and_subcommands() {
        use clap::Parser;
        #[derive(Parser)]
        struct Wrap {
            #[command(subcommand)]
            action: CompareAction,
        }

        // record capture — positionals + one response arm.
        let w = Wrap::try_parse_from(["x", "record", "SL-204", "IMP-118", "--prefer", "a"])
            .expect("record form parses");
        let CompareAction::Record(r) = w.action else {
            panic!("expected record");
        };
        assert_eq!(r.a, "SL-204");
        assert_eq!(r.prefer.as_deref(), Some("a"));

        // The other response arms parse alone.
        for arm in ["--equal", "--incomparable"] {
            Wrap::try_parse_from(["x", "record", "SL-204", "IMP-118", arm])
                .unwrap_or_else(|e| panic!("{arm} parses alone: {e}"));
        }

        // The response group is mutually exclusive and required (VT-3).
        assert!(
            Wrap::try_parse_from([
                "x", "record", "SL-204", "IMP-118", "--prefer", "a", "--equal"
            ])
            .is_err(),
            "two response arms refused"
        );
        assert!(
            Wrap::try_parse_from([
                "x",
                "record",
                "SL-204",
                "IMP-118",
                "--equal",
                "--incomparable"
            ])
            .is_err(),
            "two flag arms refused"
        );
        assert!(
            Wrap::try_parse_from(["x", "record", "SL-204", "IMP-118"]).is_err(),
            "a response arm is required"
        );

        // `list` subcommand — no positionals demanded.
        let w = Wrap::try_parse_from(["x", "list"]).expect("list subcommand parses");
        assert!(matches!(w.action, CompareAction::List(_)));

        // `withdraw <uid>` subcommand.
        let w = Wrap::try_parse_from(["x", "withdraw", "some-uid"])
            .expect("withdraw subcommand parses");
        assert!(matches!(w.action, CompareAction::Withdraw(_)));

        // `record` without its required positionals is refused.
        assert!(
            Wrap::try_parse_from(["x", "record"]).is_err(),
            "record demands its positionals"
        );
    }

    // ----- PHASE-03: list / withdraw --------------------------------------

    /// A judgement row with the given key fields; optionals absent, response
    /// pinned to prefer-a.
    fn mk_judgement(uid: &str, seq: u32, a: &str, b: &str) -> Judgement {
        Judgement {
            uid: uid.to_string(),
            seq,
            a: a.to_string(),
            b: Some(b.to_string()),
            response: Some(Response::PreferA),
            domain: comparison::DOMAIN_VALUE.to_string(),
            frame: FRAME_EQUAL_EFFORT.to_string(),
            form: RowForm::Order,
            magnitude: None,
            est_lower: None,
            est_upper: None,
            supersedes: None,
            lens: None,
            rater: RaterKind::Agent,
            by: None,
            note: None,
            date: Some("2026-07-10".to_string()),
            observed_at: None,
            basis: None,
            admission: None,
        }
    }

    /// An in-memory session carrying the given judgements (no tombstones).
    fn mk_session(date: &str, uid: &str, judgements: Vec<Judgement>) -> ComparisonSession {
        ComparisonSession {
            schema: comparison::COMPARISON_SCHEMA.to_string(),
            version: comparison::COMPARISON_VERSION,
            session: SessionHeader {
                uid: uid.to_string(),
                date: date.to_string(),
                audience: None,
            },
            judgements,
            tombstones: Vec::new(),
        }
    }

    /// Build render-ready [`RowSummary`] rows over in-memory sessions (no
    /// disk) — the SAME pipeline `run_list` composes, minus the entity scan
    /// (empty `StatusMap`/`AnchorMap`; `list`'s own unit tests exercise
    /// ordering/filtering, not R6/anchors).
    fn rows_of(sessions: &[ComparisonSession]) -> Vec<RowSummary> {
        comparison::pipeline_from_sessions(
            sessions,
            &comparison::StatusMap::new(),
            &comparison::AnchorMap::new(),
            &comparison::VALUE_PROJECTION_PARAMS,
            &comparison::VALUE_PROJECTION_PARAMS,
            &std::collections::BTreeMap::new(),
            1.0,
            0.65,
        )
        .unwrap()
        .rows
    }

    fn withdraw_args(root: &Path, uid: &str, note: Option<&str>) -> WithdrawArgs {
        WithdrawArgs {
            uid: uid.to_string(),
            note: note.map(str::to_string),
            path: Some(root.to_path_buf()),
        }
    }

    /// The single judgement uid currently in the ledger (tests capture exactly one).
    fn only_judgement_uid(root: &Path) -> String {
        load_sessions(root)
            .unwrap()
            .iter()
            .flat_map(|s| s.judgements.iter())
            .map(|j| j.uid.clone())
            .next()
            .expect("a judgement row exists")
    }

    /// The single tombstone uid currently in the ledger.
    fn only_tombstone_uid(root: &Path) -> String {
        load_sessions(root)
            .unwrap()
            .iter()
            .flat_map(|s| s.tombstones.iter())
            .map(|t| t.uid.clone())
            .next()
            .expect("a tombstone row exists")
    }

    /// `mk_judgement` with an explicit row date — the total-key order (design
    /// §2 R3: `(date, session_uid, seq)`) sorts on the JUDGEMENT row's own
    /// `date` field (`resolve.rs`'s tested behaviour), which a real capture
    /// always stamps identically to its session's date (one shared
    /// `clock::today()` call, `run_capture`) — this fixture keeps that
    /// invariant honest rather than decoupling the two dates.
    fn mk_judgement_dated(uid: &str, seq: u32, a: &str, b: &str, date: &str) -> Judgement {
        let mut j = mk_judgement(uid, seq, a, b);
        j.date = Some(date.to_string());
        j
    }

    /// SL-220 §2 token plumbing: an anchor row lists with its CLAIM token
    /// (here `anchored` — human tier), a single-subject pair cell, and no
    /// `CompilationStatus` token; the pairwise row beside it is untouched.
    /// Full mixed-fixture list goldens ride PHASE-06.
    #[test]
    fn compare_list_renders_anchor_rows_with_claim_tokens() {
        let mut anchor = mk_judgement("row-anchor", 0, "SL-204", "unused");
        anchor.b = None;
        anchor.response = None;
        anchor.form = RowForm::Anchor;
        anchor.frame = comparison::FRAME_VALUE_ANCHOR.to_string();
        anchor.magnitude = Some(6.5);
        anchor.rater = RaterKind::Human;
        let session = mk_session(
            "2026-07-10",
            "sess-a",
            vec![anchor, mk_judgement("row-pair", 1, "SL-204", "IMP-118")],
        );
        let lines = list_lines(&rows_of(&[session]), None, false);
        assert_eq!(lines.len(), 2);
        assert!(
            lines[0].contains("status=anchored") && lines[0].contains("SL-204"),
            "anchor row wears its claim token: {}",
            lines[0]
        );
        assert!(
            !lines[0].contains("vs"),
            "single subject, no pair cell: {}",
            lines[0]
        );
        assert!(
            lines[1].contains("status=active") && lines[1].contains("SL-204* vs IMP-118"),
            "the pairwise row is untouched: {}",
            lines[1]
        );
    }

    /// SL-220 §2: a superseded anchor row keeps its RESOLUTION token (claim
    /// tokens are Active-only), `--active-only` drops it, and an agent-tier
    /// head renders `prior` — the D4 routing made visible.
    #[test]
    fn compare_list_supersession_and_prior_tokens_on_anchor_rows() {
        let mk_anchor = |uid: &str, seq: u32, magnitude: f64| {
            let mut j = mk_judgement(uid, seq, "SL-204", "unused");
            j.b = None;
            j.response = None;
            j.form = RowForm::Anchor;
            j.frame = comparison::FRAME_VALUE_ANCHOR.to_string();
            j.magnitude = Some(magnitude);
            j // rater stays Agent (the fixture default): a `prior` claim
        };
        let session = || {
            let mut head = mk_anchor("row-new", 1, 2.0);
            head.supersedes = Some("row-old".to_string());
            mk_session(
                "2026-07-10",
                "sess-b",
                vec![mk_anchor("row-old", 0, 1.0), head],
            )
        };

        let all = list_lines(&rows_of(&[session()]), None, false);
        assert_eq!(all.len(), 2);
        assert!(
            all[0].contains("row-old") && all[0].contains("status=superseded→row-new"),
            "superseded anchor keeps its resolution token: {}",
            all[0]
        );
        assert!(
            all[1].contains("row-new") && all[1].contains("status=prior"),
            "agent-tier head routes to `prior` (D4): {}",
            all[1]
        );
        let active_only = list_lines(&rows_of(&[session()]), None, true);
        assert_eq!(active_only.len(), 1, "only the head survives --active-only");
    }

    /// SL-220 PHASE-06 §2/§6 (RV-278 F-8): two SAME-tier anchor claims that
    /// disagree on magnitude resolve to a `conflicted` token — the winning
    /// tier's internal contest ("contested" for an anchored tier) surfaced in
    /// `compare list`, never a `CompilationStatus` token.
    #[test]
    fn compare_list_conflicting_anchor_claims_render_conflicted() {
        let mk_anchor = |uid: &str, seq: u32, magnitude: f64| {
            let mut j = mk_judgement(uid, seq, "SL-204", "unused");
            j.b = None;
            j.response = None;
            j.form = RowForm::Anchor;
            j.frame = comparison::FRAME_VALUE_ANCHOR.to_string();
            j.magnitude = Some(magnitude);
            j.rater = RaterKind::Human; // anchored tier — the contest is "contested"
            j
        };
        // Two CONCURRENT human anchors on SL-204 with distinct magnitudes —
        // separate sessions so R3 (same-session revision) never reduces them.
        let s1 = mk_session("2026-07-10", "sess-c", vec![mk_anchor("row-lo", 0, 4.0)]);
        let s2 = mk_session("2026-07-11", "sess-d", vec![mk_anchor("row-hi", 0, 8.0)]);
        let lines = list_lines(&rows_of(&[s1, s2]), None, false);
        assert_eq!(lines.len(), 2);
        assert!(
            lines.iter().all(|l| l.contains("status=conflicted")),
            "both same-tier disagreeing anchors wear `conflicted`: {lines:?}"
        );
    }

    #[test]
    fn compare_list_orders_by_total_key() {
        // Cross-file ordering by (date, session_uid, seq) — independent of the
        // order sessions are supplied in. Rows fed late-then-early must render
        // early-then-late.
        let late = mk_session(
            "2026-07-10",
            "sess-b",
            vec![mk_judgement_dated(
                "row-late",
                0,
                "SL-204",
                "IMP-118",
                "2026-07-10",
            )],
        );
        let early = mk_session(
            "2026-07-08",
            "sess-a",
            vec![mk_judgement_dated(
                "row-early",
                0,
                "SL-204",
                "CHR-042",
                "2026-07-08",
            )],
        );
        // Same-date tie broken by session uid then seq.
        let same_date = mk_session(
            "2026-07-08",
            "sess-c",
            vec![
                mk_judgement_dated("row-seq1", 1, "IDE-001", "IMP-118", "2026-07-08"),
                mk_judgement_dated("row-seq0", 0, "IDE-001", "SL-204", "2026-07-08"),
            ],
        );

        let lines = list_lines(&rows_of(&[late, same_date, early]), None, false);
        let order: Vec<&str> = lines
            .iter()
            .map(|l| l.split_whitespace().next().unwrap())
            .collect();
        // (2026-07-08, sess-a) < (2026-07-08, sess-c, seq0) < (…, seq1) < (2026-07-10, sess-b)
        assert_eq!(order, ["row-early", "row-seq0", "row-seq1", "row-late"]);
    }

    #[test]
    fn compare_list_filters_by_participant() {
        let session = mk_session(
            "2026-07-10",
            "sess-1",
            vec![
                mk_judgement("keep", 0, "SL-204", "IMP-118"),
                mk_judgement("drop", 1, "SL-999", "CHR-042"),
            ],
        );
        let lines = list_lines(&rows_of(&[session]), Some("IMP-118"), false);
        assert_eq!(lines.len(), 1, "only the participating row survives");
        assert!(
            lines[0].contains("keep"),
            "kept the IMP-118 row: {}",
            lines[0]
        );
    }

    #[test]
    fn list_renders_full_row_uid() {
        // The listing feeds `withdraw`, so it must print the COMPLETE uid, never
        // a colliding prefix (RV-262 F-6).
        let uid = "0197f3a2-6c2f-7d4e-8f5a-2b3c4d5e6f7a";
        let session = mk_session(
            "2026-07-10",
            "sess-1",
            vec![mk_judgement(uid, 0, "SL-204", "IMP-118")],
        );
        let lines = list_lines(&rows_of(&[session]), None, false);
        assert_eq!(lines.len(), 1);
        assert!(
            lines[0].contains(uid),
            "full uid rendered, not a prefix: {}",
            lines[0]
        );
    }

    /// SL-219 VT-3 (design §2 row-status routing): the `list` status column
    /// routes per-row to the row's OWNING domain system — an est-domain
    /// quarantined row renders quarantined while value rows are unaffected
    /// (no value bias: the est evidence never touches the value compile).
    #[test]
    fn list_status_routes_per_row_to_owning_domain_system() {
        // Est-domain rows (seeded via DOMAIN_ESTIMATE) forming a C3
        // preference cycle — quarantined in the ESTIMATE system only.
        let est = |uid: &str, seq: u32, a: &str, b: &str| {
            let mut j = mk_judgement(uid, seq, a, b);
            j.domain = comparison::DOMAIN_ESTIMATE.to_string();
            j.frame = comparison::FRAME_MORE_WORK.to_string();
            j
        };
        let session = mk_session(
            "2026-07-10",
            "sess-1",
            vec![
                mk_judgement("value-row", 0, "SL-204", "IMP-118"),
                est("est-a", 1, "SL-204", "IMP-118"),
                est("est-b", 2, "IMP-118", "CHR-042"),
                est("est-c", 3, "CHR-042", "SL-204"),
            ],
        );
        let lines = list_lines(&rows_of(&[session]), None, false);
        assert_eq!(lines.len(), 4);
        let line_of = |uid: &str| {
            lines
                .iter()
                .find(|l| l.contains(uid))
                .unwrap_or_else(|| panic!("{uid} listed: {lines:?}"))
        };
        // The value row is untouched by the est-domain cycle (no value bias).
        assert!(
            line_of("value-row").contains("status=active"),
            "value row unaffected: {}",
            line_of("value-row")
        );
        for uid in ["est-a", "est-b", "est-c"] {
            assert!(
                line_of(uid).contains("status=quarantined(cycle)"),
                "{uid} quarantined by its owning (estimate) system: {}",
                line_of(uid)
            );
        }
    }

    #[test]
    fn withdraw_appends_tombstone() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 204, "accepted");
        seed_entity(&root, "IMP", 118, "accepted");
        run_capture(&capture(&root, "SL-204", "IMP-118", "a")).unwrap();

        let before = session_files(&root);
        assert_eq!(before.len(), 1);
        let orig_path = before[0].clone();
        let orig_bytes = std::fs::read(&orig_path).unwrap();
        let uid = only_judgement_uid(&root);

        run_withdraw(&withdraw_args(&root, &uid, Some("wrong way round"))).unwrap();

        // A NEW file appended; the original is byte-identical (append-only).
        let after = session_files(&root);
        assert_eq!(after.len(), 2, "a fresh tombstone file was appended");
        assert!(orig_path.exists());
        assert_eq!(
            std::fs::read(&orig_path).unwrap(),
            orig_bytes,
            "the judgement file is untouched by withdraw"
        );

        // The tombstone is a session-of-one targeting the row, seq 0, note kept.
        let sessions = load_sessions(&root).unwrap();
        let tomb = sessions
            .iter()
            .flat_map(|s| s.tombstones.iter())
            .find(|t| t.target == uid)
            .expect("tombstone targets the withdrawn row");
        assert_eq!(tomb.seq, 0);
        assert_eq!(tomb.note.as_deref(), Some("wrong way round"));

        // list now marks that row withdrawn (display-only interpretation).
        let lines = list_lines(&rows_of(&sessions), None, false);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains(&uid), "full uid present: {}", lines[0]);
        assert!(
            lines[0].contains("[withdrawn]"),
            "row rendered withdrawn: {}",
            lines[0]
        );
    }

    #[test]
    fn refuses_double_withdraw() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 204, "accepted");
        seed_entity(&root, "IMP", 118, "accepted");
        run_capture(&capture(&root, "SL-204", "IMP-118", "a")).unwrap();
        let uid = only_judgement_uid(&root);

        run_withdraw(&withdraw_args(&root, &uid, None)).unwrap();
        let err = run_withdraw(&withdraw_args(&root, &uid, None)).unwrap_err();
        assert!(
            err.to_string().contains("already withdrawn"),
            "second withdraw refused: {err}"
        );
        // No third file — the refused withdraw writes nothing.
        assert_eq!(session_files(&root).len(), 2, "refusal writes no file");
    }

    #[test]
    fn withdraw_refuses_unknown_and_tombstone_row() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 204, "accepted");
        seed_entity(&root, "IMP", 118, "accepted");
        run_capture(&capture(&root, "SL-204", "IMP-118", "a")).unwrap();
        let uid = only_judgement_uid(&root);

        // Unknown uid — no judgement carries it.
        let err = run_withdraw(&withdraw_args(&root, "not-a-real-uid", None)).unwrap_err();
        assert!(
            err.to_string().contains("unknown row uid"),
            "unknown uid refused: {err}"
        );

        // A tombstone-row uid is not a judgement — refused.
        run_withdraw(&withdraw_args(&root, &uid, None)).unwrap();
        let tomb_uid = only_tombstone_uid(&root);
        let err = run_withdraw(&withdraw_args(&root, &tomb_uid, None)).unwrap_err();
        assert!(
            err.to_string().contains("tombstone"),
            "tombstone-row uid refused: {err}"
        );
    }

    #[test]
    fn list_empty_ledger_is_not_an_error() {
        let (_tmp, root) = mk_project_root();
        // No comparisons dir at all — an empty listing, not a failure.
        let sessions = load_sessions(&root).unwrap();
        assert!(sessions.is_empty());
        assert!(list_lines(&rows_of(&sessions), None, false).is_empty());
        run_list(&ListArgs {
            id: None,
            active_only: false,
            path: Some(root),
        })
        .unwrap();
    }

    // ── SL-217 PHASE-03: the elicit shell ────────────────────────────────────

    /// Seed a value-bearing slice (`.toml` + `.md`) the priority scan discovers —
    /// no value facet ⇒ `DEFAULT_VALUE`, so it earns a score and enters the
    /// frontier. Returns its canonical id.
    fn seed_scored_slice(root: &Path, id: u32) -> String {
        let p = format!("{id:03}");
        let dir = root.join(".doctrine").join("slice").join(&p);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("slice-{p}.toml")),
            format!(
                "id = {id}\nslug = \"s{p}\"\ntitle = \"S\"\nstatus = \"proposed\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n"
            ),
        )
        .unwrap();
        std::fs::write(dir.join(format!("slice-{p}.md")), "scope\n").unwrap();
        crate::listing::canonical_id("SL", id)
    }

    /// `ElicitArgs` with everything defaulted (no filters, human render).
    fn elicit_args(root: &Path) -> ElicitArgs {
        ElicitArgs {
            depth: None,
            limit: None,
            kind: None,
            json: false,
            path: Some(root.to_path_buf()),
        }
    }

    /// T2: the shell composes deterministic, D6-costed [`ElicitInputs`] from the
    /// built graph + loaded pipeline. Pins the frontier ordering (id-lex tiebreak),
    /// the multiplier decomposition (`coeff.value × kind_weight × tag_term`), and
    /// the bare-estimate flag for an estimate-free value-bearing item.
    #[test]
    fn build_elicit_inputs_is_deterministic_and_d6_costed() {
        let (_tmp, root) = mk_project_root();
        let id1 = seed_scored_slice(&root, 1);
        let id2 = seed_scored_slice(&root, 2);
        let cfg = crate::priority::config::load(&root);
        let graph = graph::build(&root).unwrap();
        let pipeline = graph::load_comparison_pipeline_for_root(&root).unwrap();

        let a = build_elicit_inputs(&graph, &pipeline, &cfg, 8);
        let b = build_elicit_inputs(&graph, &pipeline, &cfg, 8);
        // Determinism: byte-identical inputs across repeated builds.
        assert_eq!(format!("{a:?}"), format!("{b:?}"));

        // Frontier holds both seeded items; equal score ⇒ id-lex order (SL-001 first).
        let ids: Vec<&String> = a.frontier.iter().map(|f| &f.id).collect();
        assert!(ids.contains(&&id1) && ids.contains(&&id2));
        assert_eq!(a.frontier.first().map(|f| &f.id), Some(&id1));

        // D6 costing: no tags ⇒ tag_term = 1, no estimate ⇒ bare + empty-corpus
        // cost anchor; multiplier collapses to coeff.value × kind_weight(SL).
        let c = a.costing.get(&id1).expect("id1 is costed");
        let expected_m = cfg.coefficients.value * cfg.kind_weight("SL");
        assert!(
            (c.multiplier - expected_m).abs() < 1e-9,
            "m={}",
            c.multiplier
        );
        assert!(c.bare_estimate, "no estimate facet ⇒ bare");
        assert!(c.est_cost > 0.0);
    }

    /// T2: `--depth` clamps the frontier band; a depth of 1 keeps only the top item.
    #[test]
    fn build_elicit_inputs_respects_depth_band() {
        let (_tmp, root) = mk_project_root();
        let id1 = seed_scored_slice(&root, 1);
        let _id2 = seed_scored_slice(&root, 2);
        let cfg = crate::priority::config::load(&root);
        let graph = graph::build(&root).unwrap();
        let pipeline = graph::load_comparison_pipeline_for_root(&root).unwrap();

        let inputs = build_elicit_inputs(&graph, &pipeline, &cfg, 1);
        assert_eq!(inputs.frontier.len(), 1);
        assert_eq!(inputs.frontier.first().map(|f| &f.id), Some(&id1));
    }

    /// T1/T2: the arm runs read-only end-to-end over an empty ledger and renders
    /// a state line without error (EX-1 read-only: no session file is written).
    #[test]
    fn elicit_over_empty_ledger_is_read_only_and_ok() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "IMP", 1, "open");
        run_elicit(&elicit_args(&root)).unwrap();
        assert!(
            session_files(&root).is_empty(),
            "elicit is read-only — no session file minted"
        );
    }

    // ── T3: JSON schema-v1 shapers (design §3 / D16) ────────────────────────

    /// A structural C6 bound serializes to `{kind, value?}` — open/closed carry
    /// the value, unbounded omits it (the web-review distinction `[null, 2.8]`
    /// would erase).
    #[test]
    fn bound_json_carries_open_closed_unbounded() {
        assert_eq!(
            bound_json(comparison::Bound::Open(2.5)),
            serde_json::json!({ "kind": "open", "value": 2.5 })
        );
        assert_eq!(
            bound_json(comparison::Bound::Closed(5.0)),
            serde_json::json!({ "kind": "closed", "value": 5.0 })
        );
        assert_eq!(
            bound_json(comparison::Bound::Unbounded),
            serde_json::json!({ "kind": "unbounded" })
        );
    }

    /// The participant value block carries `{provenance, point}` always, and
    /// STRUCTURAL `bounds` when the entity sits in a compiled class; a class-less
    /// value (authored floor, no interval) omits `bounds` entirely.
    #[test]
    fn value_block_json_shapes_projected_and_classless() {
        let projected = value_block_json(
            "projected",
            2.6,
            Some(comparison::ValueBounds {
                lower: comparison::Bound::Open(2.5),
                upper: comparison::Bound::Open(2.8),
            }),
        );
        assert_eq!(
            projected,
            serde_json::json!({
                "provenance": "projected",
                "point": 2.6,
                "bounds": {
                    "lower": { "kind": "open", "value": 2.5 },
                    "upper": { "kind": "open", "value": 2.8 },
                },
            })
        );

        let classless = value_block_json("authored", 4.0, None);
        assert_eq!(
            classless,
            serde_json::json!({ "provenance": "authored", "point": 4.0 })
        );
        assert!(
            classless.get("bounds").is_none(),
            "no compiled class ⇒ no bounds key"
        );
    }

    /// Anchor-review `exits` is keyed by the two answer tokens: revise re-authors
    /// the suspect value; uphold suggests superseding then withdrawing each cited
    /// row (arrays of suggested actions, not one executable — design §3).
    #[test]
    fn anchor_exits_json_keys_by_answer_token() {
        let subject = crate::priority::elicit::AnchorSubject {
            id: "IMP-274".to_string(),
            anchor: Some(5.0),
            conflict_pairs: vec![("IMP-198".to_string(), "IMP-274".to_string())],
            quarantined_rows: vec!["uid-1".to_string(), "uid-2".to_string()],
        };
        assert_eq!(
            anchor_exits_json(&subject),
            serde_json::json!({
                "revise-anchor": ["doctrine value set IMP-274 <v>"],
                "uphold-anchor": [
                    "doctrine compare record <a> <b> <response> --supersedes uid-1",
                    "doctrine compare withdraw uid-1",
                    "doctrine compare withdraw uid-2",
                ],
            })
        );
    }

    /// A suspect anchor with no cited rows yields a revise action and an empty
    /// uphold list (nothing to retire) — no panic on the empty closure.
    #[test]
    fn anchor_exits_json_empty_closure_has_no_uphold_actions() {
        let subject = crate::priority::elicit::AnchorSubject {
            id: "IMP-9".to_string(),
            anchor: None,
            conflict_pairs: vec![],
            quarantined_rows: vec![],
        };
        assert_eq!(
            anchor_exits_json(&subject),
            serde_json::json!({
                "revise-anchor": ["doctrine value set IMP-9 <v>"],
                "uphold-anchor": [],
            })
        );
    }

    /// The comparison ask block carries the canonical equal-effort/value frame
    /// and NO `yield_note`/`exits` (those are anchor-only — schema fidelity).
    #[test]
    fn comparison_ask_json_carries_frame_and_no_anchor_fields() {
        let ask = crate::priority::elicit::AskSpec {
            answers: vec!["prefer-a", "prefer-b", "equal", "incomparable"],
            yield_by_answer: [("prefer-a".to_string(), 3)].into_iter().collect(),
            yield_note: None,
        };
        let v = comparison_ask_json(&ask);
        assert_eq!(
            v.get("frame").and_then(|f| f.as_str()),
            Some("equal-effort")
        );
        assert_eq!(v.get("domain").and_then(|d| d.as_str()), Some("value"));
        assert!(
            v.get("yield_note").is_none(),
            "comparison ask omits yield_note"
        );
        assert!(v.get("exits").is_none(), "comparison ask omits exits");
    }

    // ── T4: human render footer + answer command (design §3 / D15) ──────────

    fn empty_queue(state: QueueState, excluded: usize) -> ElicitQueue {
        ElicitQueue {
            state,
            entries: vec![],
            excluded_value_insensitive: excluded,
            sizing_residual: 0,
            sizing_no_calibration_target: false,
        }
    }

    /// The D15 footer: stall names the depth and disclaims stability; stable
    /// claims value_dim order among the CURRENT top-K members (not membership),
    /// scoped by the m=0 exclusion count when present (D6).
    #[test]
    fn state_footer_names_depth_disclaims_and_scopes_m0() {
        let (cand_tok, cand) = state_footer(&empty_queue(QueueState::Candidates, 0));
        assert_eq!(cand_tok, "candidates");
        assert!(cand.contains("candidates outstanding"));

        let (stall_tok, stall) = state_footer(&empty_queue(QueueState::Stalled { depth: 8 }, 0));
        assert_eq!(stall_tok, "stalled");
        assert!(
            stall.contains("depth 8") && stall.contains("NOT a stability claim"),
            "stall names depth + disclaims: {stall}"
        );

        let (stable_tok, stable) = state_footer(&empty_queue(QueueState::Stable { depth: 5 }, 0));
        assert_eq!(stable_tok, "stable");
        assert!(
            stable.contains("top-5 frontier members") && stable.contains("not membership"),
            "stable is member-scoped: {stable}"
        );
        assert!(
            !stable.contains("value-insensitive"),
            "no m=0 clause when none excluded"
        );

        let (_, scoped) = state_footer(&empty_queue(QueueState::Stable { depth: 5 }, 3));
        assert!(
            scoped.contains("3 pair(s) value-insensitive (zero weight), outside the claim"),
            "m=0 exclusions scope + disclose: {scoped}"
        );
    }

    fn bare_participant(id: &str) -> Participant {
        Participant {
            id: id.to_string(),
            annotations: vec![],
        }
    }

    fn empty_ask() -> crate::priority::elicit::AskSpec {
        crate::priority::elicit::AskSpec {
            answers: vec![],
            yield_by_answer: std::collections::BTreeMap::new(),
            yield_note: None,
        }
    }

    fn spine(payload: EntryPayload, kind: CandidateKind) -> QueueEntry {
        QueueEntry {
            kind,
            guaranteed_yield: 1,
            guaranteed_impact: 0.5,
            score: 0.5,
            yield_basis: crate::priority::elicit::YieldBasis::OrderBearingAnswers,
            reasons: vec![],
            payload,
        }
    }

    /// The exact answer command: comparison → the `compare record` call over the
    /// pair; anchor-review → the revise/uphold resolving actions.
    #[test]
    fn answer_command_per_kind() {
        let cmp = spine(
            EntryPayload::Comparison {
                a: bare_participant("IMP-1"),
                b: bare_participant("IMP-2"),
                ask: empty_ask(),
            },
            CandidateKind::Comparison,
        );
        assert_eq!(
            answer_command(&cmp),
            "doctrine compare record IMP-1 IMP-2 --prefer a   \
             (| --prefer b | --equal | --incomparable)"
        );

        let anchor = spine(
            EntryPayload::AnchorReview {
                subject: crate::priority::elicit::AnchorSubject {
                    id: "IMP-3".to_string(),
                    anchor: Some(5.0),
                    conflict_pairs: vec![],
                    quarantined_rows: vec![],
                },
                ask: empty_ask(),
            },
            CandidateKind::AnchorReview,
        );
        assert!(
            answer_command(&anchor).starts_with("doctrine value set IMP-3 <v>"),
            "anchor revise action: {}",
            answer_command(&anchor)
        );
    }

    // ── SL-219 PHASE-05: sizing-probe render + state disclosure ──────────────

    fn sizing_target(id: &str, estimate: f64) -> crate::priority::elicit::SizingTarget {
        crate::priority::elicit::SizingTarget {
            id: id.to_string(),
            estimate,
            tier_label: "human claim".to_string(),
        }
    }

    /// The sizing-probe ask block is the invariant `more-work` / `estimate` frame
    /// with the four answer tokens — and NO `yield_by_answer` (a probe makes no
    /// yield claim).
    #[test]
    fn sizing_ask_json_is_more_work_estimate_frame() {
        let ask = sizing_ask_json();
        assert_eq!(
            ask,
            serde_json::json!({
                "frame": "more-work",
                "domain": "estimate",
                "answers": ["prefer-a", "prefer-b", "equal", "incomparable"],
            })
        );
        assert!(
            ask.get("yield_by_answer").is_none(),
            "a probe discloses no yield"
        );
    }

    /// The sizing-probe payload: lean `subject { id, annotations }`, `target
    /// { id, estimate }`, and the invariant ask.
    #[test]
    fn sizing_probe_json_shapes_subject_target_ask() {
        let subject = Participant {
            id: "SL-001".to_string(),
            annotations: vec!["projection masked by bare estimate".to_string()],
        };
        assert_eq!(
            sizing_probe_json(&subject, &sizing_target("ISS-007", 4.6)),
            serde_json::json!({
                "subject": {
                    "id": "SL-001",
                    "annotations": ["projection masked by bare estimate"],
                },
                "target": { "id": "ISS-007", "estimate": 4.6 },
                "ask": {
                    "frame": "more-work",
                    "domain": "estimate",
                    "answers": ["prefer-a", "prefer-b", "equal", "incomparable"],
                },
            })
        );
    }

    /// The answer command a sizing probe carries (the exact §4 capture leg).
    #[test]
    fn answer_command_sizing_probe_is_more_work_record() {
        let probe = spine(
            EntryPayload::SizingProbe {
                subject: bare_participant("SL-001"),
                target: sizing_target("ISS-007", 4.6),
            },
            CandidateKind::SizingProbe,
        );
        assert_eq!(
            answer_command(&probe),
            "doctrine compare record SL-001 ISS-007 --prefer a   \
             (| --prefer b | --equal | --incomparable) --frame more-work"
        );
    }

    /// Stable disclosure (§4): residual un-sized top-K debt is appended to the
    /// state line, never gating; the tier-2 flag adds the seed-sizing hint.
    #[test]
    fn state_footer_appends_sizing_debt_disclosure() {
        let mut q = empty_queue(QueueState::Stable { depth: 5 }, 0);
        q.sizing_residual = 2;
        let (_, stable) = state_footer(&q);
        assert!(
            stable.contains(
                "2 top-K item(s) carry unresolved sizing — costs may move on new evidence"
            ),
            "residual sizing debt disclosed on Stable: {stable}"
        );

        let mut q = empty_queue(QueueState::Candidates, 0);
        q.sizing_no_calibration_target = true;
        let (_, cand) = state_footer(&q);
        assert!(
            cand.contains(
                "no estimated item to calibrate against — estimate any item to seed sizing"
            ),
            "tier-2 no-target hint disclosed: {cand}"
        );
    }

    /// Seed a backlog issue carrying an authored `[estimate]` + `[value]` — an
    /// admissible authored-estimate item (a sizing-probe TARGET candidate).
    fn seed_estimated_issue(root: &Path, id: u32, lower: f64, upper: f64) -> String {
        let p = format!("{id:03}");
        let dir = root
            .join(".doctrine")
            .join("backlog")
            .join("issue")
            .join(&p);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("backlog-{p}.toml")),
            format!(
                "id = {id}\nslug = \"i{p}\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"open\"\n\
                 resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 [estimate]\nlower = {lower}\nupper = {upper}\n[value]\nvalue = 5.0\n"
            ),
        )
        .unwrap();
        std::fs::write(dir.join(format!("backlog-{p}.md")), "b\n").unwrap();
        crate::listing::canonical_id("ISS", id)
    }

    /// VT-3 end-to-end (schema-v1 JSON): an un-sized subject + an authored
    /// estimate target ⇒ a `sizing-probe` entry with the zero-claim spine
    /// (guaranteed_yield 0, yield_basis calibration, score 0), the lean
    /// subject/target payload, the invariant ask — and byte-deterministic output.
    #[test]
    fn elicit_envelope_carries_a_sizing_probe_entry_deterministically() {
        let (_tmp, root) = mk_project_root();
        let target = seed_estimated_issue(&root, 7, 2.0, 6.0); // ISS-007, anchor 4.6
        let subject = seed_scored_slice(&root, 1); // SL-001, un-sized
        // ISS-007 needs an anchored-tier estimate claim (E10: facet-only
        // items are no longer valid probe targets).
        let est_session_text = comparison::to_toml(&ComparisonSession {
            schema: comparison::COMPARISON_SCHEMA.to_string(),
            version: comparison::COMPARISON_VERSION,
            session: SessionHeader {
                uid: "iss007-anchor".to_string(),
                date: "2026-07-17".to_string(),
                audience: None,
            },
            judgements: vec![Judgement {
                uid: "iss007-anchor".to_string(),
                seq: 0,
                a: "ISS-007".to_string(),
                b: None,
                response: None,
                domain: comparison::DOMAIN_ESTIMATE.to_string(),
                frame: comparison::FRAME_COST_ANCHOR.to_string(),
                form: RowForm::Anchor,
                magnitude: None,
                supersedes: None,
                lens: None,
                rater: RaterKind::Human,
                by: Some("david".to_string()),
                note: None,
                date: Some("2026-07-17".to_string()),
                observed_at: None,
                basis: None,
                est_lower: Some(2.0),
                est_upper: Some(6.0),
                admission: None,
            }],
            tombstones: Vec::new(),
        })
        .unwrap();
        let comps_dir = root.join(".doctrine/comparisons");
        std::fs::create_dir_all(&comps_dir).unwrap();
        std::fs::write(
            comps_dir.join("2026-07-17-iss007-anchor.toml"),
            &est_session_text,
        )
        .unwrap();

        let cfg = crate::priority::config::load(&root);
        let build = || {
            let graph = graph::build(&root).unwrap();
            let pipeline = graph::load_comparison_pipeline_for_root(&root).unwrap();
            let inputs = build_elicit_inputs(&graph, &pipeline, &cfg, 8);
            let queue = assemble(&inputs, DecisionContext::Sequencing { depth: 8 });
            let ctx = RenderCtx::new(&graph, &pipeline, &cfg);
            serde_json::to_string(&elicit_envelope(&queue, 8, &elicit_args(&root), &ctx)).unwrap()
        };
        let first = build();
        assert_eq!(first, build(), "byte-deterministic output");

        let env: serde_json::Value = serde_json::from_str(&first).unwrap();
        assert_eq!(env["version"], 1, "schema stays v1");
        let probe = env["entries"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["kind"] == "sizing-probe")
            .expect("a sizing-probe entry");
        assert_eq!(probe["guaranteed_yield"], 0);
        assert_eq!(probe["score"], 0.0);
        assert_eq!(probe["yield_basis"], "calibration");
        assert_eq!(probe["subject"]["id"], subject);
        assert_eq!(probe["target"]["id"], target);
        assert!((probe["target"]["estimate"].as_f64().unwrap() - 4.6).abs() < 1e-9);
        assert_eq!(probe["ask"]["frame"], "more-work");
        assert_eq!(probe["ask"]["domain"], "estimate");
        assert!(
            probe["ask"].get("yield_by_answer").is_none(),
            "no yield claim on a probe"
        );
    }

    // ── PHASE-07: estimate mixed-fixture list golden ────────────────────────
    // PRIOR / CONFLICTED / ANCHORED token routing for estimate-domain
    // anchor rows — never a CompilationStatus token.

    /// An estimate-domain anchor judgement with the given tier rater.
    fn est_anchor(
        uid: &str,
        seq: u32,
        a: &str,
        rater: RaterKind,
        magnitude: f64,
        date: &str,
        supersedes: Option<&str>,
    ) -> Judgement {
        Judgement {
            uid: uid.to_string(),
            seq,
            a: a.to_string(),
            b: None,
            response: None,
            domain: comparison::DOMAIN_ESTIMATE.to_string(),
            frame: comparison::FRAME_COST_ANCHOR.to_string(),
            form: RowForm::Anchor,
            magnitude: None,
            est_lower: Some(magnitude * 0.8),
            est_upper: Some(magnitude * 1.2),
            supersedes: supersedes.map(str::to_string),
            lens: None,
            rater,
            by: Some("david".to_string()),
            note: None,
            date: Some(date.to_string()),
            observed_at: None,
            basis: None,
            admission: None,
        }
    }

    #[test]
    fn compare_list_estimate_mixed_fixture_token_routing() {
        // A MIXED estimate fixture:
        // - anchored (human) anchor row → status=anchored
        // - prior (agent) anchor row → status=prior
        // - conflicted (two human anchors disagreeing) → status=conflicted
        // - active (pairwise est row) → status=active (not anchor, so
        //   normal row-summary token, not anchor claim token)
        // NO CompilationStatus token on any row.
        let anchored = est_anchor(
            "row-anchored",
            0,
            "SL-100",
            RaterKind::Human,
            6.0,
            "2026-07-10",
            None,
        );
        let prior = est_anchor(
            "row-prior",
            1,
            "SL-101",
            RaterKind::Agent,
            3.0,
            "2026-07-10",
            None,
        );
        // Two human anchors on the same entity create a conflict
        let conflicted_a = est_anchor(
            "row-conf-a",
            0,
            "SL-102",
            RaterKind::Human,
            4.0,
            "2026-07-10",
            None,
        );
        let conflicted_b = est_anchor(
            "row-conf-b",
            1,
            "SL-102",
            RaterKind::Human,
            8.0,
            "2026-07-11",
            None,
        );
        // A pairwise est row (not an anchor)
        let mut pairwise = mk_judgement("row-pair", 2, "SL-100", "SL-101");
        pairwise.domain = comparison::DOMAIN_ESTIMATE.to_string();
        pairwise.frame = comparison::FRAME_MORE_WORK.to_string();
        pairwise.date = Some("2026-07-10".to_string());

        let sessions = vec![
            mk_session("2026-07-10", "sess-1", vec![anchored, prior, pairwise]),
            mk_session("2026-07-10", "sess-2", vec![conflicted_a]),
            mk_session("2026-07-11", "sess-3", vec![conflicted_b]),
        ];
        let lines = list_lines(&rows_of(&sessions), None, false);
        // All rows present
        let line_of = |uid: &str| {
            lines
                .iter()
                .find(|l| l.contains(uid))
                .unwrap_or_else(|| panic!("{uid} listed: {lines:?}"))
        };

        // Anchored: status=anchored token, NO CompilationStatus
        let l = line_of("row-anchored");
        assert!(
            l.contains("status=anchored"),
            "anchored row wears anchored token: {l}"
        );
        assert!(
            !l.contains("CompilationStatus"),
            "anchored row never carries CompilationStatus: {l}"
        );
        assert!(!l.contains("prior"), "anchored row is not prior: {l}");

        // Prior: status=prior token
        let l = line_of("row-prior");
        assert!(
            l.contains("status=prior"),
            "prior row wears prior token: {l}"
        );
        assert!(
            !l.contains("CompilationStatus"),
            "prior row never carries CompilationStatus: {l}"
        );
        assert!(!l.contains("anchored"), "prior row is not anchored: {l}");

        // Both conflicted rows: status=conflicted token
        for uid in ["row-conf-a", "row-conf-b"] {
            let l = line_of(uid);
            assert!(
                l.contains("status=conflicted"),
                "{uid} wears conflicted token: {l}"
            );
            assert!(
                !l.contains("CompilationStatus"),
                "{uid} never carries CompilationStatus: {l}"
            );
        }

        // Pairwise: normal status (active), never an anchor claim token
        let l = line_of("row-pair");
        assert!(
            l.contains("status=active"),
            "pairwise row wears active token: {l}"
        );
        assert!(
            !l.contains("CompilationStatus"),
            "pairwise row never carries CompilationStatus: {l}"
        );
        assert!(
            !l.contains("anchored") && !l.contains("prior") && !l.contains("conflicted"),
            "pairwise row has no anchor claim token: {l}"
        );
    }
}
