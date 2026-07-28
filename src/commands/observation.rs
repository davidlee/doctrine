// SPDX-License-Identifier: GPL-3.0-only
//! CLI adapter for `doctrine observation` — thin argument adaptation and
//! rendering over the shared observation service (SL-231 PHASE-03, design §3.1, §4).
//!
//! The service owns all validation, storage, replay, correction, and query policy.
//! This module adapts CLI arguments, supplies allowlisted enrichment, and renders
//! output. It does NOT re-implement any of those policies.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use clap::Subcommand;

use crate::listing::Format;
use crate::observation::resolve::{ControlOutcome, Resolution};
use crate::observation::store::Receipt;
use crate::observation::wire::{
    self, Diagnostic, Envelope, EscapeContext, Facets, ObservationKind, Origin, PRODUCT_SURFACE,
    Payload, escape_hostile, merge_explicit_facets,
};

// ── Named constants (STD-001) ─────────────────────────────────────────────

/// The CLI interface value for automatic enrichment.
const INTERFACE_CLI: &str = "cli";
/// The command identifier for `observation record`.
const COMMAND_OBSERVATION_RECORD: &str = "observation record";

// ── CLI subcommand types ──────────────────────────────────────────────────

/// Arguments for `observation record friction`.
#[derive(clap::Args, Debug)]
pub(crate) struct ObservationRecordArgs {
    #[command(subcommand)]
    pub(crate) kind: RecordKind,
}

/// The observation kind to record (friction only for public CLI).
#[derive(Subcommand, Debug)]
pub(crate) enum RecordKind {
    /// Record a friction observation.
    Friction(FrictionRecordArgs),
}

/// Arguments specific to a friction record.
#[derive(clap::Args, Debug)]
pub(crate) struct FrictionRecordArgs {
    /// The friction summary (required, non-empty).
    pub(crate) summary: String,

    /// Optional detail.
    #[arg(long)]
    pub(crate) detail: Option<String>,

    /// Caller-supplied UUID (`UUIDv7` recommended). Default: generated.
    #[arg(long)]
    pub(crate) uid: Option<String>,

    /// Disable automatic enrichment.
    #[arg(long)]
    pub(crate) no_enrich: bool,

    /// Explicit project root (default: auto-detect).
    #[arg(short = 'p', long)]
    pub(crate) path: Option<PathBuf>,
}

/// Arguments for `observation show`.
#[derive(clap::Args, Debug)]
pub(crate) struct ObservationShowArgs {
    /// The UUID of the observation to show.
    uid: String,

    /// Show the raw record (no resolution applied).
    #[arg(long)]
    raw: bool,

    /// Output format.
    #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
    format: Format,

    /// Shorthand for `--format json`.
    #[arg(long)]
    json: bool,

    /// Explicit project root (default: auto-detect).
    #[arg(short = 'p', long)]
    path: Option<PathBuf>,
}

/// Arguments for `observation list`.
#[derive(clap::Args, Debug)]
pub(crate) struct ObservationListArgs {
    /// Include history (all records including inactive and controls).
    #[arg(long)]
    history: bool,

    /// Filter by kind.
    #[arg(long, value_enum)]
    kind: Option<KwKind>,

    /// Filter by time from (inclusive, ISO 8601).
    #[arg(long)]
    time_from: Option<String>,

    /// Filter by time to (exclusive, ISO 8601).
    #[arg(long)]
    time_to: Option<String>,

    /// Max results per page.
    #[arg(long, default_value_t = 20)]
    limit: usize,

    /// Output format.
    #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
    format: Format,

    /// Shorthand for `--format json`.
    #[arg(long)]
    json: bool,

    /// Explicit project root (default: auto-detect).
    #[arg(short = 'p', long)]
    path: Option<PathBuf>,
}

/// Arguments for `observation search`.
#[derive(clap::Args, Debug)]
pub(crate) struct ObservationSearchArgs {
    /// The search query text.
    query: String,

    /// Include history (all records including inactive and controls).
    #[arg(long)]
    history: bool,

    /// Filter by kind.
    #[arg(long, value_enum)]
    kind: Option<KwKind>,

    /// Filter by time from (inclusive, ISO 8601).
    #[arg(long)]
    time_from: Option<String>,

    /// Filter by time to (exclusive, ISO 8601).
    #[arg(long)]
    time_to: Option<String>,

    /// Max results per page.
    #[arg(long, default_value_t = 20)]
    limit: usize,

    /// Output format.
    #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
    format: Format,

    /// Shorthand for `--format json`.
    #[arg(long)]
    json: bool,

    /// Explicit project root (default: auto-detect).
    #[arg(short = 'p', long)]
    path: Option<PathBuf>,
}

/// Arguments for `observation supersede`.
#[derive(clap::Args, Debug)]
pub(crate) struct ObservationSupersedeArgs {
    /// The UUID of the observation being superseded.
    old_uid: String,

    /// The UUID of the replacement observation.
    replacement_uid: String,

    /// Optional reason for the supersession.
    #[arg(long)]
    reason: Option<String>,

    /// Explicit project root (default: auto-detect).
    #[arg(short = 'p', long)]
    path: Option<PathBuf>,
}

/// Arguments for `observation retract`.
#[derive(clap::Args, Debug)]
pub(crate) struct ObservationRetractArgs {
    /// The UUID of the observation to retract.
    target_uid: String,

    /// Optional reason for the retraction.
    #[arg(long)]
    reason: Option<String>,

    /// Explicit project root (default: auto-detect).
    #[arg(short = 'p', long)]
    path: Option<PathBuf>,
}

/// Keyword-kind filter (clap-compatible, maps to `ObservationKind`).
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub(crate) enum KwKind {
    Friction,
    Measurement,
    Supersession,
    Retraction,
}

impl KwKind {
    fn to_kind(self) -> ObservationKind {
        match self {
            KwKind::Friction => ObservationKind::Friction,
            KwKind::Measurement => ObservationKind::Measurement,
            KwKind::Supersession => ObservationKind::Supersession,
            KwKind::Retraction => ObservationKind::Retraction,
        }
    }
}

// ── Observation command ───────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub(crate) enum ObservationCommand {
    /// Record a friction observation.
    Record(ObservationRecordArgs),
    /// Show an observation by UUID.
    Show(ObservationShowArgs),
    /// List observations with optional filters.
    List(ObservationListArgs),
    /// Full-text search over observations.
    Search(ObservationSearchArgs),
    /// Supersede one observation with another.
    Supersede(ObservationSupersedeArgs),
    /// Retract an observation.
    Retract(ObservationRetractArgs),
}

// ── Rendering helpers ─────────────────────────────────────────────────────

/// Escape an envelope metadata value for a slot that occupies ONE line.
///
/// Header, resolution, and correction-chain lines are single-line by
/// construction, so a surviving newline forges a line — and these render
/// *above* the untrusted-input framing, where a reader takes them for
/// trustworthy envelope metadata. `EscapeContext::Block` is for genuinely
/// multi-line payload content (summary, detail), never for metadata.
fn escape_metadata(value: &str) -> String {
    escape_hostile(value, EscapeContext::Inline)
}

/// Render a heading line for observation content: frames the content as
/// untrusted data in agent-facing rendering (EX-5).
fn render_untrusted_heading(heading: &str) -> String {
    format!("── {heading} (untrusted input) ──")
}

/// Render load and resolve diagnostics to stderr, one per line.
///
/// Every diagnostic is a single-line slot: `path: reason`, with both fields
/// escaped via `escape_metadata`. Renders nothing when both channels are empty.
fn render_diagnostics(load_diags: &[Diagnostic], outcomes: &BTreeMap<String, ControlOutcome>) {
    let mut stderr = std::io::stderr();
    for d in load_diags {
        writeln!(
            stderr,
            "{}: {}",
            escape_metadata(&d.path),
            escape_metadata(&d.reason)
        )
        .ok();
    }
    for (control_uid, outcome) in outcomes {
        if let ControlOutcome::Inert { reason } = outcome {
            writeln!(
                stderr,
                "{}: {}",
                escape_metadata(control_uid),
                escape_metadata(reason)
            )
            .ok();
        }
    }
}

/// Render a single observation as a table row.
fn observation_row(envelope: &Envelope, idx: Option<usize>) -> Vec<String> {
    let summary = match &envelope.payload {
        Payload::Friction { summary, .. } => summary.clone(),
        Payload::Measurement { source, .. } => format!("measurement:{source}"),
        Payload::Supersession {
            old_uid,
            replacement_uid,
            ..
        } => format!("{old_uid} → {replacement_uid}"),
        Payload::Retraction { target_uid, .. } => format!("retract {target_uid}"),
    };
    let kind_str = format!("{:?}", envelope.kind()).to_lowercase();
    let escaped_summary = escape_hostile(&summary, EscapeContext::Inline);
    let escaped_uid = escape_hostile(&envelope.uid, EscapeContext::Inline);
    let escaped_recorded_at = escape_hostile(&envelope.recorded_at, EscapeContext::Inline);
    let mut row = vec![escaped_uid, escaped_recorded_at, kind_str, escaped_summary];
    if let Some(n) = idx {
        row.insert(0, n.to_string());
    }
    row
}

/// Render a list of observations as a table.
fn render_observations_table(
    envelopes: &[&Envelope],
    numbered: bool,
    term_width: Option<u16>,
) -> String {
    let headers = if numbered {
        vec![
            "#".to_string(),
            "uid".to_string(),
            "recorded_at".to_string(),
            "kind".to_string(),
            "summary".to_string(),
        ]
    } else {
        vec![
            "uid".to_string(),
            "recorded_at".to_string(),
            "kind".to_string(),
            "summary".to_string(),
        ]
    };
    let mut rows: Vec<Vec<String>> = vec![headers];
    for (i, env) in envelopes.iter().enumerate() {
        let idx = if numbered { Some(i + 1) } else { None };
        rows.push(observation_row(env, idx));
    }
    crate::listing::render_table(&rows, term_width)
}

/// Render a single observation as detailed output.
fn render_observation_detail(envelope: &Envelope, resolution: Option<&Resolution>) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();

    // Headers with untrusted-data framing
    if let Payload::Friction { summary, detail } = &envelope.payload {
        writeln!(out, "uid: {}", escape_metadata(&envelope.uid)).ok();
        out.push_str("kind: friction\n");
        writeln!(
            out,
            "recorded_at: {}",
            escape_metadata(&envelope.recorded_at)
        )
        .ok();
        writeln!(out, "{}", render_untrusted_heading("summary")).ok();
        writeln!(out, "  {}", escape_hostile(summary, EscapeContext::Block)).ok();
        if let Some(d) = detail {
            writeln!(out, "{}", render_untrusted_heading("detail")).ok();
            for line in d.lines() {
                writeln!(out, "  {}", escape_hostile(line, EscapeContext::Block)).ok();
            }
        }
    } else {
        writeln!(out, "uid: {}", escape_metadata(&envelope.uid)).ok();
        writeln!(
            out,
            "kind: {}",
            format!("{:?}", envelope.kind()).to_lowercase()
        )
        .ok();
        writeln!(
            out,
            "recorded_at: {}",
            escape_metadata(&envelope.recorded_at)
        )
        .ok();
    }

    // Facets
    if let Some(facets) = envelope.facets.as_ref() {
        writeln!(out, "{}", render_untrusted_heading("facets")).ok();
        let facet_json = serde_json::to_string_pretty(facets)
            .unwrap_or_else(|_| "<unrepresentable>".to_string());
        for line in facet_json.lines() {
            writeln!(out, "  {}", escape_hostile(line, EscapeContext::Block)).ok();
        }
    }

    // Resolution info
    if let Some(res) = resolution {
        if let Some(state) = res.state(&envelope.uid) {
            match state {
                crate::observation::resolve::ResolvedState::Active(_) => {}
                crate::observation::resolve::ResolvedState::Superseded {
                    by, control_uid, ..
                } => {
                    writeln!(
                        out,
                        "\n── superseded by: {} (control: {}) ──",
                        escape_metadata(by),
                        escape_metadata(control_uid)
                    )
                    .ok();
                }
                crate::observation::resolve::ResolvedState::Retracted { control_uid, .. } => {
                    writeln!(
                        out,
                        "\n── retracted (control: {}) ──",
                        escape_metadata(control_uid)
                    )
                    .ok();
                }
            }
        }

        // Correction chain
        let chain = res.correction_chain(&envelope.uid);
        if chain.len() > 1 {
            out.push_str("\n── correction chain ──\n");
            for (uid, env) in &chain {
                let kind_str = format!("{:?}", env.kind()).to_lowercase();
                writeln!(
                    out,
                    "  {} ({kind_str}) {}",
                    escape_metadata(uid),
                    escape_metadata(&env.recorded_at)
                )
                .ok();
            }
        }
    }

    out
}

/// Render an observation as JSON.
fn render_observation_json(envelope: &Envelope, resolution: Option<&Resolution>) -> String {
    let mut val = serde_json::to_value(envelope).unwrap_or(serde_json::Value::Null);
    if let Some(res) = resolution
        && let Some(state) = res.state(&envelope.uid)
    {
        let status = match state {
            crate::observation::resolve::ResolvedState::Active(_) => "active",
            crate::observation::resolve::ResolvedState::Superseded { by, .. } => {
                if let Some(obj) = val.as_object_mut() {
                    obj.insert(
                        "superseded_by".to_string(),
                        serde_json::Value::String(by.clone()),
                    );
                }
                "superseded"
            }
            crate::observation::resolve::ResolvedState::Retracted { .. } => "retracted",
        };
        if let Some(obj) = val.as_object_mut() {
            obj.insert(
                "resolved_status".to_string(),
                serde_json::Value::String(status.to_string()),
            );
        }
    }
    serde_json::to_string_pretty(&val).unwrap_or_else(|_| "{}".to_string())
}

// ── Root resolution ───────────────────────────────────────────────────────

/// Resolve the project root from the optional path argument.
fn resolve_root(path: Option<PathBuf>) -> Result<PathBuf> {
    crate::root::find(path, &crate::root::default_markers())
}

// ── Enrichment (allowlist only — design §3.1) ─────────────────────────────

/// Build automatic enrichment facets for the CLI adapter.
/// Only writes to the named allowlist fields.
fn enrich_cli(no_enrich: bool, root: &std::path::Path) -> Facets {
    if no_enrich {
        return Facets::default();
    }

    let mut facets = Facets::default();

    // Determine repository_context from worker mode
    let repo_ctx = if crate::worktree::env_worker_set() || crate::worktree::marker_present(root) {
        Some("worker".to_string())
    } else {
        Some("primary".to_string())
    };

    facets.execution = Some(wire::ExecutionFacet {
        schema_version: 1,
        interface: Some(INTERFACE_CLI.to_string()),
        interface_origin: Some(Origin::Automatic),
        product_surface: Some(PRODUCT_SURFACE.to_string()),
        product_surface_origin: Some(Origin::Automatic),
        command: Some(COMMAND_OBSERVATION_RECORD.to_string()),
        command_origin: Some(Origin::Automatic),
        repository_context: repo_ctx,
        repository_context_origin: Some(Origin::Automatic),
        ..Default::default()
    });

    facets
}

// ── Dispatch ──────────────────────────────────────────────────────────────

/// Route an `ObservationCommand` to the shared service and render output.
pub(crate) fn dispatch(cmd: ObservationCommand, color: bool) -> Result<()> {
    match cmd {
        ObservationCommand::Record(args) => run_record(args),
        ObservationCommand::Show(args) => run_show(args),
        ObservationCommand::List(args) => run_list(args, color),
        ObservationCommand::Search(args) => run_search(args, color),
        ObservationCommand::Supersede(args) => run_supersede(args),
        ObservationCommand::Retract(args) => run_retract(args),
    }
}

// ── Record ────────────────────────────────────────────────────────────────

fn run_record(args: ObservationRecordArgs) -> Result<()> {
    let RecordKind::Friction(friction_args) = args.kind;

    let root = resolve_root(friction_args.path)?;
    let service =
        crate::observation::Service::new(root.clone(), crate::observation::SourceRegistry::empty());

    let uid = friction_args
        .uid
        .unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
    let recorded_at = crate::clock::now_timestamp()?;

    let auto_facets = enrich_cli(friction_args.no_enrich, &root);
    let facets = merge_explicit_facets(auto_facets, None);

    let envelope = Envelope {
        schema: wire::SCHEMA.to_string(),
        schema_version: wire::SCHEMA_VERSION,
        uid,
        recorded_at,
        facets: Some(facets),
        payload: Payload::Friction {
            summary: friction_args.summary,
            detail: friction_args.detail,
        },
    };

    let receipt: Receipt = service.record_friction(&envelope)?.into();
    let json_out = serde_json::to_string_pretty(&receipt)?;
    writeln!(std::io::stdout(), "{json_out}")?;
    Ok(())
}

// ── Show ──────────────────────────────────────────────────────────────────

/// Reject a uid that is not a valid UUID, before it reaches path computation.
///
/// `store::shard_dir` is structurally panic-free, but a uid that never names a
/// record has no business reaching the disk seam. The rejected uid is echoed
/// back, which makes the diagnostic a terminal boundary like any other render:
/// it is escaped, and so is the reason that embeds it.
fn reject_invalid_uid(label: &str, uid: &str) -> Result<()> {
    if let Some(d) = wire::validate_uid(uid) {
        anyhow::bail!(
            "invalid {label} '{}': {}",
            escape_hostile(uid, EscapeContext::Inline),
            escape_hostile(&d.reason, EscapeContext::Inline)
        );
    }
    Ok(())
}

fn run_show(args: ObservationShowArgs) -> Result<()> {
    reject_invalid_uid("uid", &args.uid)?;
    let root = resolve_root(args.path)?;
    let service =
        crate::observation::Service::new(root, crate::observation::SourceRegistry::empty());

    let envelope = service.load_one(&args.uid)?;

    let resolution = if args.raw {
        None
    } else {
        let (all_envs, diags) = service.load_all();
        let (res, outcomes) = crate::observation::resolve::resolve(all_envs);
        render_diagnostics(&diags, &outcomes);
        Some(res)
    };

    let effective_format = if args.json { Format::Json } else { args.format };

    match effective_format {
        Format::Json => {
            let json_out = render_observation_json(&envelope, resolution.as_ref());
            writeln!(std::io::stdout(), "{json_out}")?;
        }
        Format::Table => {
            let out = render_observation_detail(&envelope, resolution.as_ref());
            write!(std::io::stdout(), "{out}")?;
        }
    }

    Ok(())
}

// ── Query + render (shared by list and search) ────────────────────────────

/// Load the corpus, resolve it, apply the projection and filter, render
/// results to stdout, and render load + resolve diagnostics to stderr.
#[expect(
    clippy::too_many_arguments,
    reason = "query+render surface: projection, filter, search, and output params are all independently variable"
)]
fn query_and_render(
    root: PathBuf,
    projection: crate::observation::query::Projection,
    filter: &crate::observation::query::Filter,
    search: Option<&str>,
    limit: usize,
    empty_msg: &str,
    json: bool,
    format: Format,
) -> Result<()> {
    let service =
        crate::observation::Service::new(root, crate::observation::SourceRegistry::empty());

    let (all_envs, diags) = service.load_all();
    let (resolution, outcomes) = crate::observation::resolve::resolve(all_envs);
    let matched = crate::observation::query::query(&resolution, projection, filter, search);
    let results: Vec<Envelope> = matched.into_iter().take(limit).cloned().collect();

    render_diagnostics(&diags, &outcomes);

    let term_width = crate::tty::stdout_terminal_width();
    let effective_format = if json { Format::Json } else { format };

    match effective_format {
        Format::Json => {
            let envs: Vec<&Envelope> = results.iter().collect();
            let json_out = crate::listing::json_envelope("observations", &envs)?;
            writeln!(std::io::stdout(), "{json_out}")?;
        }
        Format::Table => {
            let envs: Vec<&Envelope> = results.iter().collect();
            if envs.is_empty() {
                writeln!(std::io::stdout(), "{empty_msg}")?;
            } else {
                let table = render_observations_table(&envs, true, term_width);
                write!(std::io::stdout(), "{table}")?;
            }
        }
    }

    Ok(())
}

// ── List ──────────────────────────────────────────────────────────────────

fn run_list(args: ObservationListArgs, _color: bool) -> Result<()> {
    let root = resolve_root(args.path)?;
    let projection = if args.history {
        crate::observation::query::Projection::History
    } else {
        crate::observation::query::Projection::Active
    };
    let filter = crate::observation::query::Filter {
        kind: args.kind.map(KwKind::to_kind),
        time_from: args.time_from.clone(),
        time_to: args.time_to.clone(),
    };
    query_and_render(
        root,
        projection,
        &filter,
        None,
        args.limit,
        "(no observations)",
        args.json,
        args.format,
    )
}

// ── Search ────────────────────────────────────────────────────────────────

fn run_search(args: ObservationSearchArgs, _color: bool) -> Result<()> {
    let root = resolve_root(args.path)?;
    let projection = if args.history {
        crate::observation::query::Projection::History
    } else {
        crate::observation::query::Projection::Active
    };
    let filter = crate::observation::query::Filter {
        kind: args.kind.map(KwKind::to_kind),
        time_from: args.time_from.clone(),
        time_to: args.time_to.clone(),
    };
    query_and_render(
        root,
        projection,
        &filter,
        Some(&args.query),
        args.limit,
        "(no results)",
        args.json,
        args.format,
    )
}

// ── Supersede ─────────────────────────────────────────────────────────────

fn run_supersede(args: ObservationSupersedeArgs) -> Result<()> {
    reject_invalid_uid("old_uid", &args.old_uid)?;
    reject_invalid_uid("replacement_uid", &args.replacement_uid)?;
    let root = resolve_root(args.path)?;
    let service =
        crate::observation::Service::new(root, crate::observation::SourceRegistry::empty());

    let uid = uuid::Uuid::now_v7().to_string();
    let recorded_at = crate::clock::now_timestamp()?;

    let receipt = service.record_supersession(
        &uid,
        &recorded_at,
        &args.old_uid,
        &args.replacement_uid,
        args.reason.as_deref(),
    )?;

    let receipt: Receipt = receipt.into();
    let json_out = serde_json::to_string_pretty(&receipt)?;
    writeln!(std::io::stdout(), "{json_out}")?;
    Ok(())
}

// ── Retract ───────────────────────────────────────────────────────────────

fn run_retract(args: ObservationRetractArgs) -> Result<()> {
    reject_invalid_uid("target_uid", &args.target_uid)?;
    let root = resolve_root(args.path)?;
    let service =
        crate::observation::Service::new(root, crate::observation::SourceRegistry::empty());

    let uid = uuid::Uuid::now_v7().to_string();
    let recorded_at = crate::clock::now_timestamp()?;

    let receipt =
        service.record_retraction(&uid, &recorded_at, &args.target_uid, args.reason.as_deref())?;

    let receipt: Receipt = receipt.into();
    let json_out = serde_json::to_string_pretty(&receipt)?;
    writeln!(std::io::stdout(), "{json_out}")?;
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test code is exempt from panic-family lints"
)]
mod tests {
    use super::*;

    #[test]
    fn escape_hostile_strips_ansi_escapes() {
        let input = "normal \x1b[31mred\x1b[0m text";
        let out = escape_hostile(input, EscapeContext::Block);
        assert!(!out.contains('\x1b'), "ANSI escapes must be stripped");
        assert!(out.contains("\\x1b"), "ANSI escapes must be replaced");
        assert!(out.contains("normal "), "normal text preserved");
        assert!(out.contains(" text"), "trailing text preserved");
    }

    #[test]
    fn escape_hostile_strips_control_chars() {
        let input = "hello\x00world\x01";
        let out = escape_hostile(input, EscapeContext::Block);
        assert!(!out.contains('\x00'), "NUL must be stripped");
        assert!(out.contains("\\x00"), "NUL must be replaced");
        assert!(out.contains("\\x01"), "SOH must be replaced");
    }

    #[test]
    fn escape_hostile_block_preserves_newlines_and_tabs() {
        let input = "line1\n\tline2";
        let out = escape_hostile(input, EscapeContext::Block);
        assert!(out.contains('\n'), "newlines preserved in block context");
        assert!(out.contains('\t'), "tabs preserved in block context");
    }

    #[test]
    fn escape_hostile_inline_escapes_newlines_and_tabs() {
        let input = "line1\n\tline2";
        let out = escape_hostile(input, EscapeContext::Inline);
        assert!(
            !out.contains('\n'),
            "newlines must be escaped in inline context"
        );
        assert!(
            !out.contains('\t'),
            "tabs must be escaped in inline context"
        );
        assert!(
            out.contains("\\n"),
            "newlines must become literal backslash-n"
        );
        assert!(out.contains("\\t"), "tabs must become literal backslash-t");
    }

    #[test]
    fn escape_hostile_block_preserves_non_ascii() {
        let input = "émoji — dash 🎉 CJK: 中文 text";
        let out = escape_hostile(input, EscapeContext::Block);
        assert_eq!(
            out, input,
            "non-ASCII text must survive round-trip intact in block context"
        );
    }

    #[test]
    fn escape_hostile_inline_preserves_non_ascii() {
        let input = "émoji — dash 🎉 CJK: 中文 text";
        let out = escape_hostile(input, EscapeContext::Inline);
        assert_eq!(
            out, input,
            "non-ASCII text must survive round-trip intact in inline context (no newlines/tabs)"
        );
    }

    #[test]
    fn escape_hostile_escapes_c1_controls() {
        // C1 controls U+0080..=U+009F must be escaped
        let input = "before\u{80}after\u{9f}end";
        let out = escape_hostile(input, EscapeContext::Block);
        assert!(!out.contains('\u{80}'), "C1 control U+0080 must be escaped");
        assert!(out.contains("\\x80"), "C1 control U+0080 must be \\x80");
        assert!(out.contains("\\x9f"), "C1 control U+009F must be \\x9f");
        assert!(out.contains("before"), "surrounding text preserved");
        assert!(out.contains("after"), "surrounding text preserved");
        assert!(out.contains("end"), "surrounding text preserved");
    }

    #[test]
    fn escape_hostile_escapes_del() {
        // D3 (RV-317): DEL (U+007F) must be escaped.
        let input = "before\x7fafter";
        let out = escape_hostile(input, EscapeContext::Block);
        assert!(!out.contains('\x7f'), "DEL must be escaped");
        assert!(out.contains("\\x7f"), "DEL must be represented as \\x7f");
        assert!(out.contains("before"), "surrounding text preserved");
        assert!(out.contains("after"), "surrounding text preserved");
    }

    #[test]
    fn escape_hostile_escapes_broad_c0_controls() {
        // D4.4 (RV-317): C0 controls beyond ESC/BEL/CR must be escaped.
        // Test a spread from U+0001 to U+001F, plus DEL at U+007F.
        // Skip \n (0x0a), \t (0x09), and the already-tested ESC (0x1b).
        let c0_samples: Vec<char> = (1u8..=0x1f)
            .filter(|&b| b != b'\t' && b != b'\n' && b != 0x1b)
            .map(char::from)
            .chain(std::iter::once('\x7f'))
            .collect();
        for &c in &c0_samples {
            let input = format!("a{c}z");
            let out = escape_hostile(&input, EscapeContext::Block);
            assert!(
                !out.contains(c),
                "control char U+{:04X} must be escaped",
                u32::from(c)
            );
            assert!(
                out.contains(&format!("\\x{:02x}", u32::from(c))),
                "control char U+{:04X} must become hex escape",
                u32::from(c)
            );
        }
    }

    #[test]
    fn escape_hostile_handles_crlf_adjacent_pair() {
        // D4.5 (RV-317): CRLF as an adjacent pair.
        let input = "line1\r\nline2";
        // Block: \r is escaped, \n passes through.
        let out_block = escape_hostile(input, EscapeContext::Block);
        assert!(!out_block.contains('\r'), "CR must be escaped in Block");
        assert!(out_block.contains("\\x0d"), "CR must become \\x0d in Block");
        assert!(out_block.contains('\n'), "LF preserved in Block");
        assert!(
            out_block.contains("\\x0d\n"),
            "CRLF pair should become \\x0d + LF in Block"
        );

        // Inline: both are escaped.
        let out_inline = escape_hostile(input, EscapeContext::Inline);
        assert!(!out_inline.contains('\r'), "CR must be escaped in Inline");
        assert!(!out_inline.contains('\n'), "LF must be escaped in Inline");
        assert!(
            out_inline.contains("\\x0d\\n"),
            "CRLF pair should become \\x0d\\n in Inline"
        );
    }

    #[test]
    fn escape_hostile_preserves_combining_marks() {
        // D4.3 (RV-317): decomposed forms with combining marks.
        // 'e' + combining acute accent (U+0301) = 2 chars, 3 bytes, 1 grapheme.
        let input = "cafe\u{0301} résumé";
        let out = escape_hostile(input, EscapeContext::Block);
        assert_eq!(out, input, "combining marks must survive round-trip intact");
        let out_inline = escape_hostile(input, EscapeContext::Inline);
        assert_eq!(
            out_inline, input,
            "combining marks must survive round-trip intact in Inline"
        );
    }

    #[test]
    fn receipt_serializes_correctly() {
        let r = Receipt {
            uid: "019f1234-5678-7abc-8def-0123456789ab".to_string(),
            kind: "friction".to_string(),
            recorded_at: "2026-01-01T00:00:00Z".to_string(),
            rel_path: ".doctrine/observations/records/ab/019f1234-5678-7abc-8def-0123456789ab.toml"
                .to_string(),
            outcome: "created".to_string(),
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("019f1234"), "uid in receipt");
        assert!(json.contains("created"), "outcome in receipt");
    }
}
