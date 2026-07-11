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

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::{Args, Subcommand, ValueEnum};

use crate::comparison::{
    self, COMPARISONS_DIR, ComparisonSession, FRAME_EQUAL_EFFORT, FRAME_PREFER_FIRST, Judgement,
    RaterKind, Response, RowForm, SessionHeader, Tombstone,
};

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
    /// a priority-domain row — not value-bearing.
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

/// `compare list [<ID>]` — arg shape for PHASE-03.
#[derive(Args)]
pub(crate) struct ListArgs {
    /// Optional participation filter — a canonical ref; rows where `a` or `b`
    /// equals it.
    pub(crate) id: Option<String>,
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

/// Clap adapter for the closed frame vocab — keeps clap out of the pure
/// `comparison` engine tier (ADR-001). The kebab-cased variant names are the
/// CLI tokens; [`Self::as_frame`] ties them back to the engine constants (no
/// magic strings, STD-001).
#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum FrameArg {
    EqualEffort,
    PreferFirst,
}

impl FrameArg {
    fn as_frame(self) -> &'static str {
        match self {
            Self::EqualEffort => FRAME_EQUAL_EFFORT,
            Self::PreferFirst => FRAME_PREFER_FIRST,
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

    // Pair admissibility over the resolved kinds (record pairs / RSK
    // participants refused, with the human-readable reason). The priority
    // domain reuses the value-domain admit set (design D2).
    comparison::admissible_value_pair(kind_a, kind_b).map_err(|reason| anyhow::anyhow!(reason))?;

    let response = resolve_response(args, &canonical_a, &canonical_b)?;

    // The frame implies the domain (S1) — the wire table is the single source.
    let frame = args.frame.as_frame();
    let domain = comparison::domain_for_frame(frame)
        .ok_or_else(|| anyhow::anyhow!("frame `{frame}` maps to no domain"))?;

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
        b: canonical_b,
        response,
        domain: domain.to_string(),
        frame: frame.to_string(),
        form: RowForm::Order,
        magnitude: None,
        supersedes: args.supersedes.clone(),
        lens: args.lens.clone(),
        rater: args.rater.to_kind(),
        by: args.by.clone(),
        note: args.note.clone(),
        date: date.clone(),
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

/// Write a fresh session file `<date>-<uid>.toml` under
/// `.doctrine/comparisons/` — clobber-refusing (`create-new`), directory made
/// lazily. The single disk seam shared by capture and withdrawal (design D2 —
/// append-only, exactly one new file per act; never rewrites an existing path).
fn write_session_file(
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

/// Scan `.doctrine/comparisons/*.toml` and parse every session. A missing
/// directory yields an empty listing (not an error — the ledger is created
/// lazily on first capture). Files are read in filename order for determinism;
/// row ordering is imposed later by the total key, so file order is immaterial.
fn load_sessions(root: &Path) -> anyhow::Result<Vec<ComparisonSession>> {
    let dir = root.join(".doctrine").join(COMPARISONS_DIR);
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => {
            return Err(
                anyhow::Error::new(e).context(format!("read comparisons dir {}", dir.display()))
            );
        }
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .collect();
    paths.sort();

    let mut sessions = Vec::new();
    for path in paths {
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("read comparison session {}", path.display()))?;
        let session = comparison::parse(&text)
            .with_context(|| format!("parse comparison session {}", path.display()))?;
        sessions.push(session);
    }
    Ok(sessions)
}

/// The wire token for a rater kind — mirrors the serde `rename_all = "lowercase"`
/// on [`RaterKind`] (the enum is the single source; no parallel vocab).
fn rater_token(rater: &RaterKind) -> &'static str {
    match rater {
        RaterKind::Human => "human",
        RaterKind::Agent => "agent",
    }
}

/// Render one judgement row for `list`: the FULL row uid (never a prefix —
/// uuid-v7 prefixes share a timestamp bucket and collide, and this line feeds
/// `withdraw`, RV-262 F-6), the pair with the preferred side marked `*`, frame,
/// rater (+identity when present), date, note when present, and a `[withdrawn]`
/// marker when a tombstone targets the row (display-only — resolution is Phase B).
fn render_row(j: &Judgement, withdrawn: bool) -> String {
    // Minimal v2 adaptation (SL-213 PHASE-01 EX-4): the `*` marks a preferred
    // side; `=` / `~` render equal / incomparable. The derived RowState
    // column is PHASE-06.
    let pair = match j.response {
        Response::PreferA => format!("{}* vs {}", j.a, j.b),
        Response::PreferB => format!("{} vs {}*", j.a, j.b),
        Response::Equal => format!("{} = {}", j.a, j.b),
        Response::Incomparable => format!("{} ~ {}", j.a, j.b),
    };
    let by = j.by.as_deref().map(|b| format!(":{b}")).unwrap_or_default();
    let note = j
        .note
        .as_deref()
        .map(|n| format!("  note: {n}"))
        .unwrap_or_default();
    let tomb = if withdrawn { "  [withdrawn]" } else { "" };
    format!(
        "{uid}  {pair}  frame={frame}  rater={rater}{by}  {date}{note}{tomb}",
        uid = j.uid,
        frame = j.frame,
        rater = rater_token(&j.rater),
        date = j.date,
    )
}

/// Order and render the judgement rows across all sessions. Pure — no clock,
/// disk, or IO — so the total-order and filter behaviour is unit-testable
/// without capturing stdout. Rows sort by the total key `(date, session_uid,
/// seq)` (RV-260 F-4); the optional `filter` keeps rows where either side is
/// that id. Tombstone targets are uid-referencing, so marking is
/// file-order-independent.
fn list_lines(sessions: &[ComparisonSession], filter: Option<&str>) -> Vec<String> {
    let withdrawn: BTreeSet<&str> = sessions
        .iter()
        .flat_map(|s| s.tombstones.iter().map(|t| t.target.as_str()))
        .collect();

    let mut rows: Vec<(&str, &str, &Judgement)> = sessions
        .iter()
        .flat_map(|s| {
            let uid = s.session.uid.as_str();
            let date = s.session.date.as_str();
            s.judgements.iter().map(move |j| (date, uid, j))
        })
        .filter(|(_, _, j)| match filter {
            Some(id) => j.a == id || j.b == id,
            None => true,
        })
        .collect();
    rows.sort_by(|(da, ua, ja), (db, ub, jb)| {
        da.cmp(db)
            .then_with(|| ua.cmp(ub))
            .then_with(|| ja.seq.cmp(&jb.seq))
    });

    rows.into_iter()
        .map(|(_, _, j)| render_row(j, withdrawn.contains(j.uid.as_str())))
        .collect()
}

/// `compare list [<ID>]` — read the ledger back. Scans every session, flattens
/// judgement rows, orders by the total key, applies the optional participation
/// filter, and prints one line per row. A missing directory is an empty
/// listing, not an error.
fn run_list(args: &ListArgs) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(args.path.clone(), &crate::root::default_markers())?;
    let sessions = load_sessions(&root)?;
    let lines = list_lines(&sessions, args.id.as_deref());

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
        assert_eq!(j.b, "IMP-118");
        assert_eq!(j.response, Response::PreferA);
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
        assert_eq!(session.judgements[0].response, Response::PreferB);
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
                s.judgements[0].response
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
            b: b.to_string(),
            response: Response::PreferA,
            domain: comparison::DOMAIN_VALUE.to_string(),
            frame: FRAME_EQUAL_EFFORT.to_string(),
            form: RowForm::Order,
            magnitude: None,
            supersedes: None,
            lens: None,
            rater: RaterKind::Agent,
            by: None,
            note: None,
            date: "2026-07-10".to_string(),
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

    #[test]
    fn compare_list_orders_by_total_key() {
        // Cross-file ordering by (date, session_uid, seq) — independent of the
        // order sessions are supplied in. Rows fed late-then-early must render
        // early-then-late.
        let late = mk_session(
            "2026-07-10",
            "sess-b",
            vec![mk_judgement("row-late", 0, "SL-204", "IMP-118")],
        );
        let early = mk_session(
            "2026-07-08",
            "sess-a",
            vec![mk_judgement("row-early", 0, "SL-204", "CHR-042")],
        );
        // Same-date tie broken by session uid then seq.
        let same_date = mk_session(
            "2026-07-08",
            "sess-c",
            vec![
                mk_judgement("row-seq1", 1, "IDE-001", "IMP-118"),
                mk_judgement("row-seq0", 0, "IDE-001", "SL-204"),
            ],
        );

        let lines = list_lines(&[late, same_date, early], None);
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
        let lines = list_lines(&[session], Some("IMP-118"));
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
        let lines = list_lines(&[session], None);
        assert_eq!(lines.len(), 1);
        assert!(
            lines[0].contains(uid),
            "full uid rendered, not a prefix: {}",
            lines[0]
        );
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
        let lines = list_lines(&sessions, None);
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
        assert!(list_lines(&sessions, None).is_empty());
        run_list(&ListArgs {
            id: None,
            path: Some(root),
        })
        .unwrap();
    }
}
