// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine estimate` / `doctrine value` — facet set/clear commands (SL-118 PHASE-03).
//! SL-129: uses `entity::id_path`

use std::path::PathBuf;

use anyhow::Context;
use clap::{Args, ValueEnum};

use crate::comparison::{AdmissionKind, RaterKind};

#[cfg(test)]
use crate::catalog::scan::ScanMode;

/// `doctrine estimate set <ID> ...`
#[derive(Args)]
pub(crate) struct EstimateSetArgs {
    /// Canonical entity ref (e.g. SL-118, ADR-001)
    pub(crate) id: String,
    /// Lower bound (>= 0, finite); omit with -x
    pub(crate) lower: Option<f64>,
    /// Upper bound (>= lower, finite); omit with -x
    #[arg(allow_hyphen_values = true)]
    pub(crate) upper: Option<f64>,
    /// Point estimate — sets lower == upper == N
    #[arg(long = "exact", short = 'x', conflicts_with_all = ["lower", "upper"])]
    pub(crate) exact: Option<f64>,
    /// Explicit project root (default: auto-detect)
    #[arg(short = 'p', long)]
    pub(crate) path: Option<PathBuf>,
}

#[derive(Args)]
pub(crate) struct EstimateClearArgs {
    /// Canonical entity ref (e.g. SL-118, ADR-001)
    pub(crate) id: String,
    /// Explicit project root (default: auto-detect)
    #[arg(short = 'p', long)]
    pub(crate) path: Option<PathBuf>,
}

/// Clap adapter for the anchor rater — the shell boundary that keeps clap out
/// of the pure [`RaterKind`]. `migrated` is NOT offered: that provenance is
/// minted only by the facet-import path, never a live `value set`.
#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum AnchorRaterArg {
    Human,
    Agent,
}

impl AnchorRaterArg {
    fn to_kind(self) -> RaterKind {
        match self {
            Self::Human => RaterKind::Human,
            Self::Agent => RaterKind::Agent,
        }
    }
}

/// `doctrine value set <ID> <magnitude> --rater human|agent [flags]` (SL-220
/// §4): mints a session-of-one value anchor via the `compare record` path.
/// `--rater` is MANDATORY (no default — a default fabricates provenance).
#[derive(Args)]
pub(crate) struct ValueSetArgs {
    /// Canonical entity ref (e.g. SL-118)
    pub(crate) id: String,
    /// Magnitude (any finite f64 — may be negative)
    #[arg(allow_hyphen_values = true)]
    pub(crate) magnitude: f64,
    /// Who rendered the claim — MANDATORY (no default: a default fabricates
    /// provenance).
    #[arg(long, value_enum)]
    pub(crate) rater: AnchorRaterArg,
    /// Optional rater identity (free text).
    #[arg(long)]
    pub(crate) by: Option<String>,
    /// Optional evidence citation (e.g. `REQ-059`).
    #[arg(long)]
    pub(crate) basis: Option<String>,
    /// Optional value lens (the IDE-035 seam).
    #[arg(long)]
    pub(crate) lens: Option<String>,
    /// Optional free-text note.
    #[arg(long)]
    pub(crate) note: Option<String>,
    /// Supersede a prior ANCHOR row by uid (same subject/domain/lens) — the
    /// explicit correction path. Without it a new row coexists as concurrent
    /// evidence.
    #[arg(long)]
    pub(crate) supersedes: Option<String>,
    /// Explicit project root (default: auto-detect)
    #[arg(short = 'p', long)]
    pub(crate) path: Option<PathBuf>,
}

/// `doctrine value pin <ID> <magnitude> --by <who> [flags]` — the gated
/// human-in-the-loop pin (SL-220 §4). `value pin <ID> --retire` retires the
/// active pin(s). Both require an interactive operator session (D13) and are
/// refused under worker-mode (guard.rs). `--by` is MANDATORY for a pin.
#[derive(Args)]
pub(crate) struct ValuePinArgs {
    /// Canonical entity ref (e.g. SL-118)
    pub(crate) id: String,
    /// Magnitude (any finite f64) — required unless `--retire`.
    #[arg(allow_hyphen_values = true, required_unless_present = "retire")]
    pub(crate) magnitude: Option<f64>,
    /// Retire the active pin(s) on the subject (gated) — no magnitude.
    #[arg(long, conflicts_with_all = ["magnitude", "by", "basis", "supersedes"])]
    pub(crate) retire: bool,
    /// Operator identity — MANDATORY for a pin (absent on `--retire`).
    #[arg(long, required_unless_present = "retire")]
    pub(crate) by: Option<String>,
    /// Optional evidence citation (e.g. `REQ-059`).
    #[arg(long)]
    pub(crate) basis: Option<String>,
    /// Optional free-text note.
    #[arg(long)]
    pub(crate) note: Option<String>,
    /// Supersede a prior ANCHOR row by uid (same subject/domain/lens).
    #[arg(long)]
    pub(crate) supersedes: Option<String>,
    /// Explicit project root (default: auto-detect)
    #[arg(short = 'p', long)]
    pub(crate) path: Option<PathBuf>,
}

/// `doctrine value clear <ID> [--lens <lens>] [--note <text>]` (SL-220 §4):
/// tombstones ALL active unlensed value-domain anchor rows on the subject
/// (`--lens` targets that lens's rows instead). REFUSED while a pin is active.
#[derive(Args)]
pub(crate) struct ValueClearArgs {
    /// Canonical entity ref (e.g. SL-118)
    pub(crate) id: String,
    /// Optional free-text note recorded on the tombstones.
    #[arg(long)]
    pub(crate) note: Option<String>,
    /// Clear the rows carrying THIS lens (default: the unlensed rows).
    #[arg(long)]
    pub(crate) lens: Option<String>,
    /// Explicit project root (default: auto-detect)
    #[arg(short = 'p', long)]
    pub(crate) path: Option<PathBuf>,
}

/// `doctrine risk set <ID> ...`
#[derive(Args)]
pub(crate) struct RiskSetArgs {
    /// Canonical entity ref (e.g. RSK-001)
    pub(crate) id: String,

    /// Likelihood axis level
    #[arg(long, value_enum)]
    pub(crate) likelihood: Option<crate::risk::RiskLevel>,

    /// Impact axis level
    #[arg(long, value_enum)]
    pub(crate) impact: Option<crate::risk::RiskLevel>,

    /// Risk origin (free-text label)
    #[arg(long)]
    pub(crate) origin: Option<String>,

    /// Controls — each occurrence replaces the entire list (not additive)
    #[arg(
        long,
        long_help = "Controls — each occurrence replaces the entire list (not additive)"
    )]
    pub(crate) controls: Vec<String>,

    /// Explicit project root (default: auto-detect)
    #[arg(short = 'p', long)]
    pub(crate) path: Option<PathBuf>,
}

/// `doctrine risk clear <ID>`
#[derive(Args)]
pub(crate) struct RiskClearArgs {
    /// Canonical entity ref (e.g. RSK-001)
    pub(crate) id: String,

    /// Explicit project root (default: auto-detect)
    #[arg(short = 'p', long)]
    pub(crate) path: Option<PathBuf>,
}

/// Resolve a canonical ref like `SL-118` / `ADR-003` to the entity TOML path.
/// Returns the `PathBuf` and the resolved canonical id string.
pub(crate) fn resolve_entity_path_and_canonical(
    root: &std::path::Path,
    raw: &str,
) -> anyhow::Result<(PathBuf, String)> {
    let (kref, id) = crate::integrity::parse_resolvable_ref(root, raw)?;
    let path = crate::entity::id_path(root, kref.kind, id, crate::entity::Ext::Toml);
    if !path.exists() {
        anyhow::bail!("entity not found: {raw}");
    }
    let canonical = crate::listing::canonical_id(kref.kind.prefix, id);
    Ok((path, canonical))
}

/// Read the `kind` field from a backlog entity TOML.
fn read_kind(path: &std::path::Path) -> anyhow::Result<String> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("entity not found at {}", path.display()))?;
    let doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    match doc.get("kind").and_then(toml_edit::Item::as_str) {
        Some(s) => Ok(s.to_owned()),
        None => anyhow::bail!("no 'kind' field — not a backlog item"),
    }
}

pub(crate) fn run_estimate_set(args: &EstimateSetArgs) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(args.path.clone(), &crate::root::default_markers())?;
    let (path, canonical) = resolve_entity_path_and_canonical(&root, &args.id)?;

    // Determine bounds from -x or positionals.
    let (lower, upper) = match args.exact {
        Some(n) => (n, n),
        None => match (args.lower, args.upper) {
            (Some(l), Some(u)) => (l, u),
            (Some(_), None) => anyhow::bail!(
                "estimate set: LOWER without UPPER — supply both bounds, or -x/--exact for a point estimate"
            ),
            (None, Some(_)) => anyhow::bail!(
                "estimate set: UPPER without LOWER — supply both bounds, or -x/--exact for a point estimate"
            ),
            (None, None) => anyhow::bail!(
                "estimate set: supply both LOWER and UPPER, or -x/--exact for a point estimate"
            ),
        },
    };

    // Build facet & validate (the COMPLETE rule from PHASE-01).
    let facet = crate::estimate::EstimateFacet { lower, upper };
    crate::estimate::validate(&facet)?;

    // Write via the leaf.
    let fields: &[(&str, f64)] = &[("lower", lower), ("upper", upper)];
    let changed = crate::facet_write::apply_set(&path, "estimate", fields)?;

    if changed {
        writeln!(
            std::io::stdout(),
            "estimate set: {canonical} lower={lower} upper={upper}"
        )?;
    } else {
        writeln!(
            std::io::stdout(),
            "estimate unchanged: {canonical} lower={lower} upper={upper}"
        )?;
    }
    Ok(())
}

pub(crate) fn run_estimate_clear(args: &EstimateClearArgs) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(args.path.clone(), &crate::root::default_markers())?;
    let (path, canonical) = resolve_entity_path_and_canonical(&root, &args.id)?;
    let cleared = crate::facet_write::apply_clear(&path, "estimate")?;
    if cleared {
        writeln!(std::io::stdout(), "estimate cleared: {canonical}")?;
    } else {
        writeln!(std::io::stdout(), "no estimate to clear: {canonical}")?;
    }
    Ok(())
}

/// The REV-022 Q1 scoring-inert warning for a value facet written on a
/// non-value-bearing kind — `None` when the target kind carries value. Pure:
/// the decision is over the resolved canonical id's prefix, the shell writes it.
/// Value stays writable regardless (design § REV-022 Q1 warn — warn, still write).
fn scoring_inert_warning(canonical: &str) -> Option<String> {
    let prefix = canonical.split('-').next().unwrap_or_default();
    if crate::kinds::is_value_bearing(prefix) {
        return None;
    }
    Some(format!(
        "warning: value on {canonical} is scoring-inert — {prefix} is not value-bearing; scoring ignores this facet (ADR-015 § Value-source resolution)"
    ))
}

/// D13 pin gate — pure over the injected TTY bit (the shell reads stdin, same
/// pattern as date/uid injection): the gated `value pin` family requires an
/// interactive operator session. Refused when stdin is not a TTY, naming the
/// posture.
fn require_interactive(is_interactive: bool) -> anyhow::Result<()> {
    if !is_interactive {
        anyhow::bail!(
            "value pin requires an interactive operator session — stdin is not a TTY; a pin is a human-in-the-loop admission, run it from an interactive terminal"
        );
    }
    Ok(())
}

/// One resolved-ACTIVE value anchor row on a subject (SL-220 §4): its uid, its
/// lens, and whether it is a pin.
struct ActiveAnchor {
    uid: String,
    lens: Option<String>,
    is_pin: bool,
}

/// The LIVE value-domain anchor rows on `subject` (SL-220 §4): the corpus
/// resolved through the shared `resolve` pass so supersession and tombstones
/// are already reduced. "Live" is `Active` (unlensed) OR `InertLens`
/// (lens-tagged) — the same selection the claims pass makes, since `resolve`
/// marks every lens-tagged row `InertLens` unconditionally. An empty
/// `StatusMap` suffices (anchor liveness needs no entity-status classes).
fn active_value_anchors(
    root: &std::path::Path,
    subject: &str,
) -> anyhow::Result<Vec<ActiveAnchor>> {
    use crate::comparison::ResolutionStatus;
    let sessions = crate::comparison::load_sessions(root)?;
    let resolution = crate::comparison::resolve(&sessions, &crate::comparison::StatusMap::new())?;
    let mut out = Vec::new();
    for (j, status) in &resolution.rows {
        if !matches!(
            status,
            ResolutionStatus::Active | ResolutionStatus::InertLens
        ) || j.domain != crate::comparison::DOMAIN_VALUE
            || !matches!(j.form, crate::comparison::RowForm::Anchor)
            || j.a != subject
        {
            continue;
        }
        out.push(ActiveAnchor {
            uid: j.uid.clone(),
            lens: j.lens.clone(),
            is_pin: j.admission == Some(AdmissionKind::Pin),
        });
    }
    Ok(out)
}

pub(crate) fn run_value_set(args: &ValueSetArgs) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(args.path.clone(), &crate::root::default_markers())?;
    let (_path, canonical) = resolve_entity_path_and_canonical(&root, &args.id)?;

    // REV-022 Q1 / D7: an anchor on a non-value-bearing kind is scoring-inert.
    // Warn (to stderr) but still capture — the claim lands, just inert.
    if let Some(warning) = scoring_inert_warning(&canonical) {
        writeln!(std::io::stderr(), "{warning}")?;
    }

    let spec = crate::commands::compare::AnchorSpec {
        subject: canonical.clone(),
        magnitude: args.magnitude,
        rater: args.rater.to_kind(),
        admission: None,
        by: args.by.clone(),
        basis: args.basis.clone(),
        lens: args.lens.clone(),
        note: args.note.clone(),
        supersedes: args.supersedes.clone(),
    };
    let path = crate::commands::compare::record_anchor(&root, &spec)?;
    writeln!(
        std::io::stdout(),
        "value set: {canonical} magnitude={} — session {}",
        args.magnitude,
        path.display()
    )?;
    Ok(())
}

/// `doctrine value pin` — the gated human-in-the-loop pin (SL-220 §4). The
/// public entry reads the real TTY; the inner takes the bit injected so both
/// branches unit-test (the shell-seam pattern).
pub(crate) fn run_value_pin(args: &ValuePinArgs) -> anyhow::Result<()> {
    use std::io::IsTerminal;
    run_value_pin_inner(args, std::io::stdin().is_terminal())
}

fn run_value_pin_inner(args: &ValuePinArgs, is_interactive: bool) -> anyhow::Result<()> {
    use std::io::Write;
    require_interactive(is_interactive)?;
    let root = crate::root::find(args.path.clone(), &crate::root::default_markers())?;
    let (_path, canonical) = resolve_entity_path_and_canonical(&root, &args.id)?;

    if args.retire {
        let uids: Vec<String> = active_value_anchors(&root, &canonical)?
            .into_iter()
            .filter(|a| a.is_pin)
            .map(|a| a.uid)
            .collect();
        if uids.is_empty() {
            anyhow::bail!("{canonical}: no active pin to retire");
        }
        let path = crate::commands::compare::record_tombstones(&root, &uids, args.note.as_deref())?;
        writeln!(
            std::io::stdout(),
            "value pin retired: {canonical} ({} row(s)) — session {}",
            uids.len(),
            path.display()
        )?;
        return Ok(());
    }

    // Clap's `required_unless_present` guarantees both are present off `--retire`.
    let magnitude = args.magnitude.ok_or_else(|| {
        anyhow::anyhow!("value pin requires a MAGNITUDE (or --retire to retire the active pin)")
    })?;
    let by = args.by.clone().ok_or_else(|| {
        anyhow::anyhow!("value pin requires --by <who> — a pin records its operator")
    })?;

    if let Some(warning) = scoring_inert_warning(&canonical) {
        writeln!(std::io::stderr(), "{warning}")?;
    }

    let spec = crate::commands::compare::AnchorSpec {
        subject: canonical.clone(),
        magnitude,
        rater: RaterKind::Human,
        admission: Some(AdmissionKind::Pin),
        by: Some(by),
        basis: args.basis.clone(),
        lens: None,
        note: args.note.clone(),
        supersedes: args.supersedes.clone(),
    };
    let path = crate::commands::compare::record_anchor(&root, &spec)?;
    writeln!(
        std::io::stdout(),
        "value pinned: {canonical} magnitude={magnitude} — session {}",
        path.display()
    )?;
    Ok(())
}

pub(crate) fn run_value_clear(args: &ValueClearArgs) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(args.path.clone(), &crate::root::default_markers())?;
    let (_path, canonical) = resolve_entity_path_and_canonical(&root, &args.id)?;

    let active = active_value_anchors(&root, &canonical)?;
    // A pin is a durable admission — correction is `value pin --retire`, never
    // clear (design §4).
    if active.iter().any(|a| a.is_pin) {
        anyhow::bail!(
            "{canonical}: a pin is active — value clear is refused; retire the pin first with `doctrine value pin {canonical} --retire`"
        );
    }
    // Unlensed rows by default; `--lens` targets that lens's rows explicitly.
    let uids: Vec<String> = active
        .into_iter()
        .filter(|a| a.lens.as_deref() == args.lens.as_deref())
        .map(|a| a.uid)
        .collect();
    if uids.is_empty() {
        writeln!(std::io::stdout(), "no value anchor to clear: {canonical}")?;
        return Ok(());
    }
    let path = crate::commands::compare::record_tombstones(&root, &uids, args.note.as_deref())?;
    writeln!(
        std::io::stdout(),
        "value cleared: {canonical} ({} row(s)) — session {}",
        uids.len(),
        path.display()
    )?;
    Ok(())
}

pub(crate) fn run_risk_set(args: &RiskSetArgs) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(args.path.clone(), &crate::root::default_markers())?;
    let (path, canonical) = resolve_entity_path_and_canonical(&root, &args.id)?;

    // Kind gate: must be a risk item.
    let kind = read_kind(&path)?;
    if kind != "risk" {
        anyhow::bail!("{canonical}: risk set requires a risk item, got {kind}");
    }

    // At-least-one axis guard.
    if args.likelihood.is_none() && args.impact.is_none() {
        anyhow::bail!("risk set: must supply at least one of --likelihood or --impact");
    }

    // Build FacetField list.
    let mut fields: Vec<crate::facet_write::FacetField> = Vec::new();
    if let Some(ref level) = args.likelihood {
        fields.push(crate::facet_write::FacetField::Str {
            key: "likelihood",
            value: level.as_str().to_owned(),
        });
    }
    if let Some(ref level) = args.impact {
        fields.push(crate::facet_write::FacetField::Str {
            key: "impact",
            value: level.as_str().to_owned(),
        });
    }
    if let Some(ref origin) = args.origin {
        fields.push(crate::facet_write::FacetField::Str {
            key: "origin",
            value: origin.clone(),
        });
    }
    if !args.controls.is_empty() {
        fields.push(crate::facet_write::FacetField::Arr {
            key: "controls",
            values: args.controls.clone(),
        });
    }

    let changed = crate::facet_write::apply_set_mixed(&path, "facet", &fields)?;

    // Build echo parts (Vec<String> + join — house style).
    if changed {
        let mut parts: Vec<String> = Vec::new();
        if let Some(ref level) = args.likelihood {
            parts.push(format!("likelihood={}", level.as_str()));
        }
        if let Some(ref level) = args.impact {
            parts.push(format!("impact={}", level.as_str()));
        }
        if let Some(ref origin) = args.origin {
            parts.push(format!("origin={origin:?}"));
        }
        if !args.controls.is_empty() {
            let list: Vec<String> = args.controls.iter().map(|c| format!("{c:?}")).collect();
            parts.push(format!("controls=[{}]", list.join(", ")));
        }
        let detail = parts.join(" ");
        writeln!(std::io::stdout(), "risk set: {canonical} {detail}")?;
    } else {
        // Unchanged — same detail pattern.
        let mut parts: Vec<String> = Vec::new();
        if let Some(ref level) = args.likelihood {
            parts.push(format!("likelihood={}", level.as_str()));
        }
        if let Some(ref level) = args.impact {
            parts.push(format!("impact={}", level.as_str()));
        }
        if let Some(ref origin) = args.origin {
            parts.push(format!("origin={origin:?}"));
        }
        if !args.controls.is_empty() {
            let list: Vec<String> = args.controls.iter().map(|c| format!("{c:?}")).collect();
            parts.push(format!("controls=[{}]", list.join(", ")));
        }
        let detail = parts.join(" ");
        writeln!(std::io::stdout(), "risk unchanged: {canonical} {detail}")?;
    }
    Ok(())
}

pub(crate) fn run_risk_clear(args: &RiskClearArgs) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(args.path.clone(), &crate::root::default_markers())?;
    let (path, canonical) = resolve_entity_path_and_canonical(&root, &args.id)?;

    // Kind gate.
    let kind = read_kind(&path)?;
    if kind != "risk" {
        anyhow::bail!("{canonical}: risk clear requires a risk item, got {kind}");
    }

    let cleared = crate::facet_write::apply_clear(&path, "facet")?;
    if cleared {
        writeln!(std::io::stdout(), "risk cleared: {canonical}")?;
    } else {
        writeln!(std::io::stdout(), "no risk facet to clear: {canonical}")?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    /// Seed a minimal entity TOML for testing, returning (toml_path, canonical_id).
    fn seed_entity(root: &std::path::Path, prefix: &str, id: u32) -> (std::path::PathBuf, String) {
        let padded = format!("{id:03}");
        let kref = crate::integrity::kind_by_prefix(prefix).expect("valid prefix");
        let toml_path = crate::entity::id_path(&root, kref.kind, id, crate::entity::Ext::Toml);
        let dir = toml_path.parent().unwrap().to_path_buf();
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            &toml_path,
            format!(
                "id = {id}\nslug = \"t{padded}\"\ntitle = \"Test {prefix}-{padded}\"\nstatus = \"accepted\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n"
            ),
        )
        .unwrap();
        let canonical = crate::listing::canonical_id(prefix, id);
        (toml_path, canonical)
    }

    /// Create a tempdir that `root::find` can resolve as a project root.
    fn mk_project_root() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(".project"), "").unwrap();
        std::fs::create_dir_all(tmp.path().join(".doctrine")).unwrap();
        std::fs::write(tmp.path().join(crate::dtoml::DOCTRINE_TOML), "").unwrap();
        let root = tmp.path().to_path_buf();
        (tmp, root)
    }

    // ---- VT-8: invalid matrix rejected ----

    #[test]
    fn vt8_neither_mode_rejected() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let args = EstimateSetArgs {
            id: "SL-118".into(),
            lower: None,
            upper: None,
            exact: None,
            path: Some(root),
        };
        let err = run_estimate_set(&args).unwrap_err().to_string();
        assert!(
            err.contains("supply both LOWER and UPPER"),
            "neither-bound message names both: {err}"
        );
    }

    #[test]
    fn vt8_one_lone_positional_rejected() {
        // IMP-139: LOWER without UPPER gets a SPECIFIC message naming which bound
        // is missing — not the same generic text as the neither case.
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let args = EstimateSetArgs {
            id: "SL-118".into(),
            lower: Some(1.0),
            upper: None,
            exact: None,
            path: Some(root),
        };
        let err = run_estimate_set(&args).unwrap_err().to_string();
        assert!(
            err.contains("LOWER without UPPER"),
            "lone-lower message names the missing bound: {err}"
        );
    }

    #[test]
    fn vt8_lone_upper_rejected() {
        // IMP-139: the symmetric case — UPPER without LOWER.
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let args = EstimateSetArgs {
            id: "SL-118".into(),
            lower: None,
            upper: Some(5.0),
            exact: None,
            path: Some(root),
        };
        let err = run_estimate_set(&args).unwrap_err().to_string();
        assert!(
            err.contains("UPPER without LOWER"),
            "lone-upper message names the missing bound: {err}"
        );
    }

    #[test]
    fn vt8_negative_lower_rejected() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let args = EstimateSetArgs {
            id: "SL-118".into(),
            lower: Some(-1.0),
            upper: Some(5.0),
            exact: None,
            path: Some(root),
        };
        let err = run_estimate_set(&args).unwrap_err().to_string();
        assert!(err.contains("lower must be >= 0"), "got: {err}");
    }

    #[test]
    fn vt8_upper_lt_lower_rejected() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let args = EstimateSetArgs {
            id: "SL-118".into(),
            lower: Some(5.0),
            upper: Some(2.0),
            exact: None,
            path: Some(root),
        };
        let err = run_estimate_set(&args).unwrap_err().to_string();
        assert!(err.contains("upper must be >= lower"), "got: {err}");
    }

    #[test]
    fn vt8_inf_lower_rejected() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let args = EstimateSetArgs {
            id: "SL-118".into(),
            lower: Some(f64::INFINITY),
            upper: Some(5.0),
            exact: None,
            path: Some(root),
        };
        let err = run_estimate_set(&args).unwrap_err().to_string();
        assert!(err.contains("finite"), "got: {err}");
    }

    #[test]
    fn vt8_nan_lower_rejected() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let args = EstimateSetArgs {
            id: "SL-118".into(),
            lower: Some(f64::NAN),
            upper: Some(5.0),
            exact: None,
            path: Some(root),
        };
        let err = run_estimate_set(&args).unwrap_err().to_string();
        assert!(err.contains("finite"), "got: {err}");
    }

    #[test]
    fn vt8_entity_not_found_rejected() {
        let (_tmp, root) = mk_project_root();
        // No entity seeded; resolve fails.
        let err = resolve_entity_path_and_canonical(&root, "SL-999")
            .unwrap_err()
            .to_string();
        assert!(err.contains("does not resolve to an entity"), "got: {err}");
    }

    // ---- VT-9: -x N sets lower == upper == N ----

    #[test]
    fn vt9_exact_sets_point_estimate() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let args = EstimateSetArgs {
            id: "SL-118".into(),
            lower: None,
            upper: None,
            exact: Some(3.0),
            path: Some(root.clone()),
        };
        run_estimate_set(&args).unwrap();
        let (path, _) = resolve_entity_path_and_canonical(&root, "SL-118").unwrap();
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("lower = 3.0"), "missing lower:\n{body}");
        assert!(body.contains("upper = 3.0"), "missing upper:\n{body}");
    }

    // ---- SL-220 §4: value anchor capture verbs -------------------------------

    /// A `value set` args builder with the mandatory `--rater` supplied and
    /// every optional absent.
    fn set_args(id: &str, magnitude: f64, root: &std::path::Path) -> ValueSetArgs {
        ValueSetArgs {
            id: id.into(),
            magnitude,
            rater: AnchorRaterArg::Human,
            by: None,
            basis: None,
            lens: None,
            note: None,
            supersedes: None,
            path: Some(root.to_path_buf()),
        }
    }

    /// The live value-anchor rows on `subject` — read back through the corpus
    /// resolve pass (excludes superseded/tombstoned rows).
    fn live_anchors(root: &std::path::Path, subject: &str) -> Vec<ActiveAnchor> {
        active_value_anchors(root, subject).unwrap()
    }

    /// `value set` mints a session-of-one anchor with the stamped
    /// frame/domain/form and the given magnitude; the row is live.
    #[test]
    fn value_set_mints_anchor_session() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        run_value_set(&set_args("SL-118", 42.0, &root)).unwrap();

        let sessions = crate::comparison::load_sessions(&root).unwrap();
        let rows: Vec<_> = sessions.iter().flat_map(|s| &s.judgements).collect();
        assert_eq!(rows.len(), 1, "one anchor row minted");
        let j = rows[0];
        assert_eq!(j.a, "SL-118");
        assert_eq!(j.domain, crate::comparison::DOMAIN_VALUE);
        assert_eq!(j.frame, crate::comparison::FRAME_VALUE_ANCHOR);
        assert!(matches!(j.form, crate::comparison::RowForm::Anchor));
        assert_eq!(j.magnitude, Some(42.0));
        assert_eq!(j.b, None);
        assert_eq!(j.admission, None);
    }

    /// D10: EVERY invocation mints — two identical `value set`s yield TWO rows
    /// (no no-op/idempotency guard), both live as concurrent evidence.
    #[test]
    fn value_set_mints_every_invocation() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        run_value_set(&set_args("SL-118", 7.0, &root)).unwrap();
        run_value_set(&set_args("SL-118", 7.0, &root)).unwrap();
        assert_eq!(
            live_anchors(&root, "SL-118").len(),
            2,
            "two live rows (D10)"
        );
    }

    /// Negatives are admissible (mirrors `value::validate` — no range policy).
    #[test]
    fn value_set_admits_negative_magnitude() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        run_value_set(&set_args("SL-118", -5.0, &root)).unwrap();
        let sessions = crate::comparison::load_sessions(&root).unwrap();
        let j = sessions[0].judgements.first().unwrap();
        assert_eq!(j.magnitude, Some(-5.0));
    }

    /// Non-finite magnitude is refused at capture, mirroring `value::validate`.
    #[test]
    fn value_set_rejects_non_finite() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        for bad in [f64::INFINITY, f64::NAN, f64::NEG_INFINITY] {
            let err = run_value_set(&set_args("SL-118", bad, &root))
                .unwrap_err()
                .to_string();
            assert!(err.contains("finite"), "{bad}: got {err}");
        }
    }

    /// D7 warn posture: an anchor on a non-value-bearing kind warns but still
    /// captures; the paired admissibility property over ALL_KINDS pins that the
    /// warn fires exactly for the non-value-bearing kinds (mirrors `value set`).
    #[test]
    fn value_anchor_warn_posture_mirrors_all_kinds() {
        for &kind in crate::kinds::ALL_KINDS {
            let canonical = crate::listing::canonical_id(kind, 1);
            let warns = scoring_inert_warning(&canonical).is_some();
            assert_eq!(
                warns,
                !crate::kinds::is_value_bearing(kind),
                "{kind}: warns iff not value-bearing"
            );
        }
        // …and the row STILL lands on a non-value-bearing kind (warn, not block).
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "QUE", 7);
        run_value_set(&set_args("QUE-007", 3.0, &root)).unwrap();
        assert_eq!(
            live_anchors(&root, "QUE-007").len(),
            1,
            "captured despite warn"
        );
    }

    /// `--supersedes` scope is refused at capture for a FOREIGN subject
    /// (through the real verb — the pure four-way refusal battery lives in
    /// `wire`).
    #[test]
    fn value_set_supersedes_foreign_subject_refused() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        seed_entity(&root, "SL", 119);
        run_value_set(&set_args("SL-118", 5.0, &root)).unwrap();
        let target = live_anchors(&root, "SL-118")[0].uid.clone();

        let mut args = set_args("SL-119", 9.0, &root);
        args.supersedes = Some(target);
        let err = run_value_set(&args).unwrap_err().to_string();
        assert!(err.contains("within one subject"), "got: {err}");
    }

    /// An unknown `--supersedes` target is refused (not deferred).
    #[test]
    fn value_set_supersedes_unknown_target_refused() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let mut args = set_args("SL-118", 5.0, &root);
        args.supersedes = Some("no-such-uid".into());
        let err = run_value_set(&args).unwrap_err().to_string();
        assert!(err.contains("names no judgement row"), "got: {err}");
    }

    /// The D13 pin gate is pure over the injected TTY bit — BOTH branches:
    /// non-interactive refuses naming the posture; interactive passes the gate.
    #[test]
    fn require_interactive_gates_both_branches() {
        let err = require_interactive(false).unwrap_err().to_string();
        assert!(err.contains("not a TTY"), "names the posture: {err}");
        assert!(require_interactive(true).is_ok());
    }

    /// `value pin` (interactive branch injected) mints a human anchor stamped
    /// `admission = pin`.
    #[test]
    fn value_pin_mints_pin_row() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let args = ValuePinArgs {
            id: "SL-118".into(),
            magnitude: Some(6.5),
            retire: false,
            by: Some("david".into()),
            basis: None,
            note: None,
            supersedes: None,
            path: Some(root.clone()),
        };
        run_value_pin_inner(&args, true).unwrap();
        let sessions = crate::comparison::load_sessions(&root).unwrap();
        let j = sessions[0].judgements.first().unwrap();
        assert_eq!(j.admission, Some(AdmissionKind::Pin));
        assert_eq!(j.rater, RaterKind::Human);
        assert_eq!(j.magnitude, Some(6.5));
    }

    /// `value pin` refuses the non-interactive branch BEFORE any write.
    #[test]
    fn value_pin_refuses_non_interactive() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let args = ValuePinArgs {
            id: "SL-118".into(),
            magnitude: Some(6.5),
            retire: false,
            by: Some("david".into()),
            basis: None,
            note: None,
            supersedes: None,
            path: Some(root.clone()),
        };
        let err = run_value_pin_inner(&args, false).unwrap_err().to_string();
        assert!(err.contains("not a TTY"), "got: {err}");
        assert!(live_anchors(&root, "SL-118").is_empty(), "no row written");
    }

    /// `value clear` tombstones all active unlensed rows; `--lens` targets a
    /// lens's rows and leaves the unlensed ones live.
    #[test]
    fn value_clear_tombstones_unlensed_rows() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        run_value_set(&set_args("SL-118", 1.0, &root)).unwrap();
        run_value_set(&set_args("SL-118", 2.0, &root)).unwrap();
        let mut lensed = set_args("SL-118", 3.0, &root);
        lensed.lens = Some("user-value".into());
        run_value_set(&lensed).unwrap();
        assert_eq!(live_anchors(&root, "SL-118").len(), 3);

        run_value_clear(&ValueClearArgs {
            id: "SL-118".into(),
            note: None,
            lens: None,
            path: Some(root.clone()),
        })
        .unwrap();
        // The two unlensed rows are gone; the lensed row survives.
        let live = live_anchors(&root, "SL-118");
        assert_eq!(live.len(), 1);
        assert_eq!(live[0].lens.as_deref(), Some("user-value"));
    }

    /// `value clear` is refused while a pin is active, naming the remedy.
    #[test]
    fn value_clear_refused_under_active_pin() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let pin = ValuePinArgs {
            id: "SL-118".into(),
            magnitude: Some(6.5),
            retire: false,
            by: Some("david".into()),
            basis: None,
            note: None,
            supersedes: None,
            path: Some(root.clone()),
        };
        run_value_pin_inner(&pin, true).unwrap();
        let err = run_value_clear(&ValueClearArgs {
            id: "SL-118".into(),
            note: None,
            lens: None,
            path: Some(root.clone()),
        })
        .unwrap_err()
        .to_string();
        assert!(err.contains("--retire"), "names the remedy: {err}");
    }

    /// `value pin --retire` tombstones the active pin; afterwards the ladder
    /// falls through (no live pin remains).
    #[test]
    fn value_pin_retire_tombstones_the_pin() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let pin = ValuePinArgs {
            id: "SL-118".into(),
            magnitude: Some(6.5),
            retire: false,
            by: Some("david".into()),
            basis: None,
            note: None,
            supersedes: None,
            path: Some(root.clone()),
        };
        run_value_pin_inner(&pin, true).unwrap();
        assert!(live_anchors(&root, "SL-118").iter().any(|a| a.is_pin));

        let retire = ValuePinArgs {
            id: "SL-118".into(),
            magnitude: None,
            retire: true,
            by: None,
            basis: None,
            note: None,
            supersedes: None,
            path: Some(root.clone()),
        };
        run_value_pin_inner(&retire, true).unwrap();
        assert!(
            live_anchors(&root, "SL-118").is_empty(),
            "the pin row is tombstoned"
        );
    }

    // ---- VT-11: catalog scan round-trip ----
    // The catalog scan reads facets from the toml; we seed the toml directly
    // and assert the catalog carries the data. Handler tests (VT-8/9/10)
    // already prove the write path.

    #[test]
    fn vt11_catalog_scan_estimate_readback() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".doctrine")).unwrap();
        std::fs::write(root.join(crate::dtoml::DOCTRINE_TOML), "").unwrap();
        // Seed an entity with [estimate] present.
        let padded = "118";
        let dir = root.join(".doctrine/slice").join(padded);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("slice-{padded}.toml")),
            "id = 118\nslug = \"t118\"\ntitle = \"Test\"\nstatus = \"accepted\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n[estimate]\nlower = 2.0\nupper = 8.0\n",
        )
        .unwrap();
        std::fs::write(dir.join(format!("slice-{padded}.md")), "# Test body\n").unwrap();
        // Scan catalog and find the entity.
        let catalog = crate::catalog::hydrate::scan_catalog(root, ScanMode::default()).unwrap();
        let entity = catalog
            .entities
            .iter()
            .find(|e| e.kind_label == "SL" && matches!(&e.key, crate::catalog::hydrate::CatalogKey::Numbered(k) if k.id == 118))
            .expect("SL-118 should be in the catalog");
        let est = entity
            .estimate
            .as_ref()
            .expect("estimate should be present");
        assert_eq!(est.lower, 2.0);
        assert_eq!(est.upper, 8.0);
    }

    #[test]
    fn vt11_catalog_scan_estimate_clear_readback() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".doctrine")).unwrap();
        std::fs::write(root.join(crate::dtoml::DOCTRINE_TOML), "").unwrap();
        // Seed an entity WITHOUT [estimate] — simulating clear.
        let padded = "118";
        let dir = root.join(".doctrine/slice").join(padded);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("slice-{padded}.toml")),
            "id = 118\nslug = \"t118\"\ntitle = \"Test\"\nstatus = \"accepted\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n",
        )
        .unwrap();
        std::fs::write(dir.join(format!("slice-{padded}.md")), "# Test body\n").unwrap();
        // Scan catalog — estimate should be absent.
        let catalog = crate::catalog::hydrate::scan_catalog(root, ScanMode::default()).unwrap();
        let entity = catalog
            .entities
            .iter()
            .find(|e| e.kind_label == "SL" && matches!(&e.key, crate::catalog::hydrate::CatalogKey::Numbered(k) if k.id == 118))
            .expect("SL-118 should be in the catalog");
        assert!(
            entity.estimate.is_none(),
            "estimate should be None after clear, got: {:?}",
            entity.estimate
        );
    }

    // ---- VT-12: slice typed-reader round-trip ----
    // Seed a toml with [estimate] / [value] present and read back via
    // parse_optional — the pure engine path.

    #[test]
    fn vt12_slice_typed_reader_roundtrip() {
        let toml_body = "id = 118\nslug = \"t118\"\ntitle = \"Test\"\nstatus = \"accepted\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n[estimate]\nlower = 3.0\nupper = 7.0\n";
        let val: toml::Table = toml_body.parse().unwrap();
        let parsed =
            crate::estimate::parse_optional(val.get("estimate").and_then(|v| v.as_table()))
                .unwrap()
                .expect("estimate should be present");
        assert_eq!(parsed.lower, 3.0);
        assert_eq!(parsed.upper, 7.0);
    }

    #[test]
    fn vt12_value_typed_reader_roundtrip() {
        let toml_body = "id = 118\nslug = \"t118\"\ntitle = \"Test\"\nstatus = \"accepted\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n[value]\nvalue = 99.0\n";
        let val: toml::Table = toml_body.parse().unwrap();
        let parsed = crate::value::parse_optional(val.get("value").and_then(|v| v.as_table()))
            .unwrap()
            .expect("value should be present");
        assert_eq!(parsed.value, 99.0);
    }

    // ---- VT-1: risk set --likelihood low --impact medium writes both to [facet] ----

    #[test]
    fn risk_set_writes_both_axes() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, canonical) = seed_entity(&root, "RSK", 1);
        // Overwrite with kind = "risk" so read_kind passes.
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind = format!("{body}kind = \"risk\"\n");
        std::fs::write(&toml_path, with_kind).unwrap();

        let args = RiskSetArgs {
            id: canonical,
            likelihood: Some(crate::risk::RiskLevel::Low),
            impact: Some(crate::risk::RiskLevel::Medium),
            origin: None,
            controls: vec![],
            path: Some(root.clone()),
        };
        run_risk_set(&args).unwrap();

        let after = std::fs::read_to_string(&toml_path).unwrap();
        assert!(
            after.contains("likelihood = \"low\""),
            "missing likelihood:\n{after}"
        );
        assert!(
            after.contains("impact = \"medium\""),
            "missing impact:\n{after}"
        );
    }

    // ---- VT-2: risk set --likelihood only — partial write ----

    #[test]
    fn risk_set_likelihood_only() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, canonical) = seed_entity(&root, "RSK", 2);
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind = format!("{body}kind = \"risk\"\n");
        std::fs::write(&toml_path, with_kind).unwrap();

        let args = RiskSetArgs {
            id: canonical,
            likelihood: Some(crate::risk::RiskLevel::High),
            impact: None,
            origin: None,
            controls: vec![],
            path: Some(root.clone()),
        };
        run_risk_set(&args).unwrap();

        let after = std::fs::read_to_string(&toml_path).unwrap();
        assert!(
            after.contains("likelihood = \"high\""),
            "missing likelihood:\n{after}"
        );
        assert!(
            !after.contains("impact"),
            "impact should be absent:\n{after}"
        );
    }

    // ---- VT-3: risk set with neither axis → error ----

    #[test]
    fn risk_set_no_axis_rejected() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, canonical) = seed_entity(&root, "RSK", 3);
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind = format!("{body}kind = \"risk\"\n");
        std::fs::write(&toml_path, with_kind).unwrap();

        let args = RiskSetArgs {
            id: canonical,
            likelihood: None,
            impact: None,
            origin: None,
            controls: vec![],
            path: Some(root),
        };
        let err = run_risk_set(&args).unwrap_err().to_string();
        assert!(
            err.contains("must supply at least one of --likelihood or --impact"),
            "got: {err}"
        );
    }

    // ---- VT-4: risk set on non-risk item → kind-gate error ----

    #[test]
    fn risk_set_on_non_risk_kind_rejected() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, _) = seed_entity(&root, "ISS", 1);
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind = format!("{body}kind = \"issue\"\n");
        std::fs::write(&toml_path, with_kind).unwrap();

        let args = RiskSetArgs {
            id: "ISS-001".into(),
            likelihood: Some(crate::risk::RiskLevel::Low),
            impact: None,
            origin: None,
            controls: vec![],
            path: Some(root),
        };
        let err = run_risk_set(&args).unwrap_err().to_string();
        assert!(err.contains("risk set requires a risk item"), "got: {err}");
    }

    // ---- VT-5: risk set on non-backlog entity → error ----

    #[test]
    fn risk_set_on_non_backlog_rejected() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 1);

        let args = RiskSetArgs {
            id: "SL-001".into(),
            likelihood: Some(crate::risk::RiskLevel::Low),
            impact: None,
            origin: None,
            controls: vec![],
            path: Some(root),
        };
        let err = run_risk_set(&args).unwrap_err().to_string();
        assert!(
            err.contains("no 'kind' field — not a backlog item"),
            "got: {err}"
        );
    }

    // ---- VT-6: risk clear removes [facet] table ----

    #[test]
    fn risk_clear_removes_facet() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, canonical) = seed_entity(&root, "RSK", 6);
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind_and_facet = format!("{body}kind = \"risk\"\n[facet]\nlikelihood = \"low\"\n");
        std::fs::write(&toml_path, with_kind_and_facet).unwrap();

        let args = RiskClearArgs {
            id: canonical,
            path: Some(root.clone()),
        };
        run_risk_clear(&args).unwrap();

        let after = std::fs::read_to_string(&toml_path).unwrap();
        assert!(
            !after.contains("[facet]"),
            "[facet] should be gone:\n{after}"
        );
    }

    // ---- VT-7: risk clear on absent facet → no-op echo ----

    #[test]
    fn risk_clear_absent_noop() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, canonical) = seed_entity(&root, "RSK", 7);
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind = format!("{body}kind = \"risk\"\n");
        std::fs::write(&toml_path, with_kind).unwrap();

        let args = RiskClearArgs {
            id: canonical,
            path: Some(root),
        };
        // No error — just no-op.
        run_risk_clear(&args).unwrap();
    }

    // ---- VT-8: risk set idempotent — same values → no-op echo ----

    #[test]
    fn risk_set_idempotent_noop() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, canonical) = seed_entity(&root, "RSK", 8);
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind_and_facet =
            format!("{body}kind = \"risk\"\n[facet]\nlikelihood = \"low\"\nimpact = \"medium\"\n");
        std::fs::write(&toml_path, with_kind_and_facet).unwrap();

        let args = RiskSetArgs {
            id: canonical,
            likelihood: Some(crate::risk::RiskLevel::Low),
            impact: Some(crate::risk::RiskLevel::Medium),
            origin: None,
            controls: vec![],
            path: Some(root),
        };
        // Should succeed (no error), and the file should be unchanged.
        run_risk_set(&args).unwrap();
    }

    // ---- VT-9: risk set --origin writes origin string ----

    #[test]
    fn risk_set_origin() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, canonical) = seed_entity(&root, "RSK", 9);
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind = format!("{body}kind = \"risk\"\n");
        std::fs::write(&toml_path, with_kind).unwrap();

        let args = RiskSetArgs {
            id: canonical,
            likelihood: Some(crate::risk::RiskLevel::Low),
            impact: None,
            origin: Some("supply-chain".into()),
            controls: vec![],
            path: Some(root.clone()),
        };
        run_risk_set(&args).unwrap();

        let after = std::fs::read_to_string(&toml_path).unwrap();
        assert!(
            after.contains("origin = \"supply-chain\""),
            "missing origin:\n{after}"
        );
    }

    // ---- VT-10: risk set --controls A --controls B writes ["A", "B"] ----

    #[test]
    fn risk_set_controls() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, canonical) = seed_entity(&root, "RSK", 10);
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind = format!("{body}kind = \"risk\"\n");
        std::fs::write(&toml_path, with_kind).unwrap();

        let args = RiskSetArgs {
            id: canonical,
            likelihood: Some(crate::risk::RiskLevel::Low),
            impact: None,
            origin: None,
            controls: vec!["A".into(), "B".into()],
            path: Some(root.clone()),
        };
        run_risk_set(&args).unwrap();

        let after = std::fs::read_to_string(&toml_path).unwrap();
        assert!(
            after.contains("controls = [\"A\", \"B\"]"),
            "missing controls array:\n{after}"
        );
    }

    // ---- VT-11: risk set preserves non-managed facet keys ----

    #[test]
    fn risk_set_preserves_unknown_sibling() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, canonical) = seed_entity(&root, "RSK", 11);
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind_and_facet =
            format!("{body}kind = \"risk\"\n[facet]\nlikelihood = \"low\"\nnotes = \"keep me\"\n");
        std::fs::write(&toml_path, with_kind_and_facet).unwrap();

        let args = RiskSetArgs {
            id: canonical,
            likelihood: Some(crate::risk::RiskLevel::High),
            impact: None,
            origin: None,
            controls: vec![],
            path: Some(root.clone()),
        };
        run_risk_set(&args).unwrap();

        let after = std::fs::read_to_string(&toml_path).unwrap();
        assert!(
            after.contains("likelihood = \"high\""),
            "likelihood not updated:\n{after}"
        );
        assert!(
            after.contains("notes = \"keep me\""),
            "non-managed sibling lost:\n{after}"
        );
    }

    // ---- VT-12 (VT-18 in design): risk set on risk item with absent [facet] table — allocates ----

    #[test]
    fn risk_set_allocates_absent_facet() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, canonical) = seed_entity(&root, "RSK", 12);
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind = format!("{body}kind = \"risk\"\n");
        std::fs::write(&toml_path, with_kind).unwrap();

        let args = RiskSetArgs {
            id: canonical,
            likelihood: Some(crate::risk::RiskLevel::Critical),
            impact: Some(crate::risk::RiskLevel::Critical),
            origin: None,
            controls: vec![],
            path: Some(root.clone()),
        };
        run_risk_set(&args).unwrap();

        let after = std::fs::read_to_string(&toml_path).unwrap();
        assert!(
            after.contains("[facet]"),
            "[facet] should be allocated:\n{after}"
        );
        assert!(
            after.contains("likelihood = \"critical\""),
            "missing likelihood:\n{after}"
        );
        assert!(
            after.contains("impact = \"critical\""),
            "missing impact:\n{after}"
        );
    }
}
