// SPDX-License-Identifier: GPL-3.0-only
//! Top-level CLI dispatch — the `Command` enum, its sub-enums, and the thin
//! dispatch match that routes each verb. Moved here from `main.rs` in SL-115
//! PHASE-04 so `main.rs` is reduced to the binary entrypoint stub (~250 LOC).

use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use clap::CommandFactory;
use clap::Subcommand;

use crate::commands::config::ConfigCommand;
use crate::commands::facet::{
    EstimateClearArgs, EstimateSetArgs, RiskClearArgs, RiskSetArgs, ValueClearArgs, ValueSetArgs,
};
use crate::listing::Format;
use crate::search::SearchArgs;

// ── shared action enums (Estimate / Value) ──────────────────────────────────

#[derive(clap::Subcommand)]
pub(crate) enum EstimateAction {
    /// Set estimate bounds
    Set(EstimateSetArgs),
    /// Clear the estimate facet
    Clear(EstimateClearArgs),
}

#[derive(clap::Subcommand)]
pub(crate) enum ValueAction {
    /// Set value bounds
    Set(ValueSetArgs),
    /// Clear the value facet
    Clear(ValueClearArgs),
}

/// `doctrine risk set` / `doctrine risk clear`
#[derive(clap::Subcommand)]
pub(crate) enum RiskAction {
    /// Set risk likelihood/impact/origin/controls
    Set(RiskSetArgs),
    /// Clear the risk facet
    Clear(RiskClearArgs),
}

// ── top-level Command enum ──────────────────────────────────────────────────

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Install doctrine files into a project.
    Install {
        /// Explicit project root (default: auto-detect by walking up
        /// from CWD looking for .git, .jj, .project, etc.).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,

        /// Target agent(s); repeatable. Default: auto-detect.
        #[arg(short = 'a', long)]
        agent: Vec<String>,

        /// Skill id(s) to install; repeatable. Default: all.
        #[arg(short = 's', long)]
        skill: Vec<String>,

        /// Domain(s) to install; repeatable. Default: all.
        #[arg(short = 'd', long)]
        domain: Vec<String>,

        /// Install only the memory skills (record-memory + retrieve-memory).
        /// Mutually exclusive with --skill / --domain.
        #[arg(long, conflicts_with_all = ["skill", "domain"])]
        only_memory: bool,

        /// Install to the user directory instead of the project.
        #[arg(short = 'g', long)]
        global: bool,

        /// Print the plan and exit without making changes.
        #[arg(long)]
        dry_run: bool,

        /// Skip the confirmation prompt.
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Debug catalog inspection.
    ///
    /// Thin JSON dump of the hydrated entity corpus (`scan`) and its graph
    /// projection (`graph`). Developer-facing; not gating for acceptance (SL-071 D12).
    Catalog {
        #[command(subcommand)]
        command: crate::catalog::CatalogCommand,
    },

    /// List available skills and their install status.
    ///
    /// Hidden deprecated alias — the consolidated `install` surface is the primary path.
    #[command(hide = true)]
    Skills {
        #[command(subcommand)]
        command: crate::skills::SkillsCommand,
    },

    /// Start the local map explorer web server.
    Map {
        #[command(subcommand)]
        command: crate::commands::map::MapCommand,
    },

    /// Create, list, and show concept maps — DSL-driven relationship diagrams.
    ConceptMap {
        #[command(subcommand)]
        command: crate::concept_map::ConceptMapCommand,
    },

    /// Create and list slices — the unit of intentional change.
    Slice {
        #[command(subcommand)]
        command: crate::slice::SliceCommand,
    },

    /// Record, show, and list memories.
    Memory {
        #[command(subcommand)]
        command: crate::memory::MemoryCommand,
    },

    /// Create, show, and list adversarial-review ledgers (the RV kind, ADR-007).
    Review {
        #[command(subcommand)]
        command: crate::review::ReviewCommand,
    },

    /// Create, show, and list reconciliation records (the REC kind, SPEC-002).
    Rec {
        #[command(subcommand)]
        command: crate::rec::RecCommand,
    },

    /// Full-text search over the entity corpus.
    Search(SearchArgs),

    /// Create, show, and transition revisions (the REV change-axis kind, ADR-013).
    Revision {
        #[command(subcommand)]
        command: crate::revision::RevisionCommand,
    },

    /// Reconcile ONE requirement against observed coverage.
    ///
    /// The sole author of reconciled requirement status (SL-044). Applies exactly
    /// one move and emits one atomic REC. `--to` is required for accept/revise,
    /// omitted for redesign.
    Reconcile {
        /// The requirement to reconcile, canonical `REQ-NNN`.
        req: String,

        /// The owning slice this act is recorded against, canonical `SL-NNN`.
        #[arg(long)]
        slice: String,

        /// The reconciliation move: accept | revise | redesign.
        #[arg(long = "move", value_parser = crate::rec::RecMove::parse)]
        r#move: crate::rec::RecMove,

        /// The explicit target status (required for accept/revise; omit for
        /// redesign). The WRITTEN status — never derived from coverage (NF-001).
        #[arg(long, value_enum)]
        to: Option<crate::requirement::ReqStatus>,

        /// Optional operator note (surfaced; not stored in the REC).
        #[arg(long)]
        note: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Requirement coverage.
    ///
    /// The read-only drift view (`show`) plus the observed-tier write path
    /// (`record`/`verify`/`forget`, SL-057).
    Coverage {
        #[command(subcommand)]
        command: crate::commands::coverage::CoverageCommand,
    },

    /// Read-only cross-kind relation view.
    ///
    /// Shows one entity's authored outbound relations, derived inbound relations,
    /// and any unresolved / free-text dangling targets — grouped, direct-only
    /// (one hop).
    Inspect {
        /// Canonical ref of the entity to inspect (e.g. `SL-046`, `ADR-004`).
        id: String,

        /// Output format (table | json).
        #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
        format: Format,

        /// Shorthand for `--format json`.
        #[arg(long)]
        json: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Read-only cross-kind importance survey.
    ///
    /// Every ELIGIBLE entity in importance order (actionability, then consequence
    /// desc, then canonical-id), each blocked row carrying a BLOCKED badge and its
    /// direct blocker. Terminal and promoted-backlog items are excluded unless
    /// `--all`. Advisory — never writes.
    Survey {
        /// Include terminal + promoted-backlog items (the complete view).
        #[arg(long)]
        all: bool,

        /// Output format (table | json).
        #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
        format: Format,

        /// Shorthand for `--format json`.
        #[arg(long)]
        json: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Read-only advisory worklist.
    ///
    /// The ACTIONABLE entities (eligible AND unblocked), in composed
    /// dependency/sequence order. Blocked items are absent (the divergence from
    /// `survey`). Mutates nothing.
    Next {
        /// Output format (table | json).
        #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
        format: Format,

        /// Shorthand for `--format json`.
        #[arg(long)]
        json: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Read-only blocker view.
    ///
    /// Shows one entity's direct blocked-by prerequisites + the items it is
    /// blocking. `--transitive` walks both chains. Display depth never reorders.
    Blockers {
        /// Canonical ref of the entity (e.g. `ISS-007`, `SL-046`).
        id: String,

        /// Walk the full transitive blocked-by / blocking chains.
        #[arg(long)]
        transitive: bool,

        /// Output format (table | json).
        #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
        format: Format,

        /// Shorthand for `--format json`.
        #[arg(long)]
        json: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Read-only structured priority explanation.
    ///
    /// Explains one entity's priority: its eligibility reason, the transitive
    /// blocker chain, the order-key contributors, any evicted soft-sequence edges,
    /// and its consequence — always to root.
    Explain {
        /// Canonical ref of the entity (e.g. `ISS-007`, `SL-046`).
        id: String,

        /// Output format (table | json).
        #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
        format: Format,

        /// Shorthand for `--format json`.
        #[arg(long)]
        json: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Create and list architecture decision records.
    Adr {
        #[command(subcommand)]
        command: crate::adr::AdrCommand,
    },

    /// Create and list governance policies (standing rules).
    Policy {
        #[command(subcommand)]
        command: crate::policy::PolicyCommand,
    },

    /// Create and list governance standards (standing conventions of practice).
    Standard {
        #[command(subcommand)]
        command: crate::standard::StandardCommand,
    },

    /// Create and list RFC discussion artifacts — governance-neutral deliberation.
    Rfc {
        #[command(subcommand)]
        command: crate::rfc::RfcCommand,
    },

    /// Create and list product / technical specifications.
    Spec {
        #[command(subcommand)]
        command: crate::spec::SpecCommand,
    },

    /// Export the doctrine corpus to an external interchange format.
    Export {
        #[command(subcommand)]
        command: ExportCommand,
    },

    /// Capture and survey backlog work-intake items (issue / improvement /
    /// chore / risk / idea).
    Backlog {
        #[command(subcommand)]
        command: crate::backlog::BacklogCommand,
    },

    /// Capture and survey durable knowledge records (assumption / decision /
    /// question / constraint).
    Knowledge {
        #[command(subcommand)]
        command: crate::knowledge::KnowledgeCommand,
    },

    /// Add or remove tags on entity kinds that surface tags (SL-136).
    Tag {
        #[command(subcommand)]
        command: crate::commands::tag::TagCommand,
    },

    /// Survey held remote id reservations (`refs/doctrine/reservation/*`, SL-148).
    Reservation {
        #[command(subcommand)]
        command: crate::commands::reservation::ReservationCommand,
    },

    /// Start the MCP stdio server (`serve --mcp`).
    Serve {
        #[command(flatten)]
        args: crate::commands::serve::ServeArgs,
    },

    /// Regenerate the governance snapshot.
    ///
    /// Regenerate the cache-friendly governance snapshot, or `boot install` to wire it.
    Boot {
        /// Wire the `@`-import + per-harness session refresh (omit to regenerate).
        #[command(subcommand)]
        command: Option<crate::boot::BootCommand>,

        /// Emit the snapshot to stdout after regenerating (mutually exclusive with --check).
        #[arg(long, conflicts_with = "check")]
        emit: bool,

        /// Report disk staleness + unpopulated sections without writing (the
        /// disk sentry). Ignored when the `install` subcommand is given.
        #[arg(long)]
        check: bool,

        /// Explicit project root (default: auto-detect). Used by the bare
        /// regenerate; `boot install` carries its own `-p`.
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Provision a worktree fork (allowlisted copy, coordination tier excluded).
    Worktree {
        #[command(subcommand)]
        command: crate::worktree::WorktreeCommand,
    },

    /// Dispatch coordination-branch projection.
    ///
    /// The integration-sync seam (SL-064 / ADR-012) that materialises reviewable
    /// refs from `dispatch/<slice>`. Orchestrator-classed — refused under worker-mode.
    Dispatch {
        #[command(subcommand)]
        command: crate::dispatch::DispatchCommand,
    },

    /// Scan entity ids for integrity violations.
    ///
    /// Scans every numbered entity kind for id-integrity violations (ADR-006 D3
    /// detect-half): dir basename == toml id, no intra-kind duplicate id, and
    /// alias target equality. Exits non-zero on any violation.
    Validate {
        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Renumber an entity's canonical id.
    ///
    /// ADR-006 D3 repair. Takes a canonical ref (`SL-031`), moves it to the next
    /// free trunk-aware id or `--to <NNN>`, and reports inbound prose citations as
    /// danglers (never rewrites them).
    Reseat {
        /// Canonical ref to renumber, e.g. `SL-031` (never a bare id).
        reference: String,

        /// Explicit target id (default: the next free trunk-aware id).
        #[arg(long)]
        to: Option<u32>,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Read-only relation projection views (SL-137).
    ///
    /// `doctrine relation list` — filter-and-project relation edges.
    /// `doctrine relation census` — group edges by label with resolution tallies.
    Relation {
        #[command(subcommand)]
        command: crate::commands::relation::RelationCommand,
    },

    /// Author a tier-1 relation edge.
    ///
    /// `link SL-048 governed_by ADR-010` (SL-048 §5.4). The label must be
    /// `link`-writable for the source kind, and the target must resolve to an entity
    /// of a legal kind (forward-edge validation, §5.5). Idempotent — re-linking an
    /// existing edge is a no-op.
    Link {
        /// The source entity's canonical ref (e.g. `SL-048`) or memory ref (`mem_<uid>`, `mem.<key>`).
        source: String,
        /// The relation label, e.g. `governed_by`, `consumes`, `related`.
        label: String,
        /// The target — a canonical ref (`ADR-010`) for validated labels, free text
        /// for `drift`.
        target: String,
        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Inspect and modify doctrine.toml [priority] coefficients.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },

    /// Remove a tier-1 relation edge.
    ///
    /// Removes an edge authored by `link` (SL-048 §5.4). Symmetric on the same write
    /// seam; idempotent — unlinking an absent edge is a no-op.
    Unlink {
        /// The source entity's canonical ref (e.g. `SL-048`) or memory ref (`mem_<uid>`, `mem.<key>`).
        source: String,
        /// The relation label to remove, e.g. `governed_by`.
        label: String,
        /// The target ref the edge points at.
        target: String,
        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Append a hard prerequisite.
    ///
    /// `needs SL-060 SL-047` (SL-060 §5.4). Generic cross-kind: SRC and TGT resolve
    /// via the same canonical-ref seam as `link`. SRC must be a dep/seq-authoring
    /// kind (slice or a backlog kind); TGT must resolve AND be work-like (slice or
    /// backlog) — a free-text or non-work-like target is refused at author time.
    /// Idempotent.
    Needs {
        /// The source entity's canonical ref, e.g. `SL-060`.
        source: String,
        /// The prerequisite target's canonical ref, e.g. `SL-047`.
        target: String,
        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Append a soft-sequence edge.
    ///
    /// `after SL-060 SL-047 [--rank N]` (SL-060 §5.4). Generic cross-kind with the
    /// same author-time target gate as `needs`. Records `{ to, rank }` (rank
    /// default 0). Idempotent.
    After {
        /// The source entity's canonical ref, e.g. `SL-060`.
        source: String,
        /// The predecessor target's canonical ref, e.g. `SL-047`.
        /// Required unless --prune is set (PHASE-03 pre-wire).
        #[arg(required_unless_present = "prune")]
        target: Option<String>,
        /// Per-edge manual tie-break rank. On append: sets the new edge's rank
        /// (default 0). On --remove: upper bound — only edges with rank ≤ N are
        /// removed. Ignored with --prune.
        #[arg(long, default_value_t = 0)]
        rank: i32,
        /// Remove matching after edges instead of appending.
        #[arg(long, conflicts_with = "prune")]
        remove: bool,
        /// Drop every dangling after edge from the source entity.
        #[arg(long, conflicts_with = "remove")]
        prune: bool,
        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Read-only project orientation dashboard.
    ///
    /// Active work, blocked items, boot staleness, recent commits. 10–20 lines
    /// human output; structured JSON.
    Status {
        /// Output format (table | json).
        #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
        format: Format,

        /// Shorthand for `--format json`.
        #[arg(long)]
        json: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Record that NEW supersedes OLD.
    ///
    /// `supersede ADR-012 ADR-004` (SL-062 §5.4). ADR-first — one parse-once /
    /// hold-both / write-once transaction writes `NEW.supersedes += OLD`,
    /// `OLD.superseded_by += NEW` (the single sanctioned reverse carve-out, ADR-004
    /// §5), and flips `OLD.status → superseded`. Refuses a self-edge, cross-kind
    /// refs, a non-ADR kind, and an OLD already superseded by a different ADR.
    /// Idempotent — a re-run with all three surfaces present is a no-op.
    Supersede {
        /// The superseding entity's canonical ref, e.g. `ADR-012`.
        new: String,
        /// The superseded entity's canonical ref, e.g. `ADR-004`.
        old: String,
        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Set or clear the [estimate] facet
    Estimate {
        #[command(subcommand)]
        action: EstimateAction,
    },
    /// Set or clear the [value] facet
    Value {
        #[command(subcommand)]
        action: ValueAction,
    },
    /// Set or clear the [facet] on a risk item
    Risk {
        #[command(subcommand)]
        action: RiskAction,
    },
}

// ── help rendering ───────────────────────────────────────────────────────────

/// A navigational grouping of top-level commands (SL-150). Header-only: `members`
/// are command names matched against the live clap tree. `suppress_verbs` keeps the
/// family's commands header-only in the boot map (infra is operational / skill-driven,
/// not boot-time authoring routing — D7); the flag rides the struct so the suppression
/// is compile-linked to the family, not a separate stringly-typed key list (F-4).
struct Family {
    key: &'static str,
    members: &'static [&'static str],
    suppress_verbs: bool,
}

/// The 8-family taxonomy (SL-150 §5.2). The ONLY hand-maintained classification of
/// the top-level command surface; the drift-guard test asserts it partitions the
/// visible clap subcommands exactly (INV-1/INV-2). Families render in this declared
/// order; members within a family render in member-array order (INV-4).
static FAMILIES: &[Family] = &[
    Family {
        key: "change",
        suppress_verbs: false,
        members: &[
            "slice",
            "revision",
            "rfc",
            "rec",
            "review",
            "reconcile",
            "coverage",
        ],
    },
    Family {
        key: "governance",
        suppress_verbs: false,
        members: &["adr", "policy", "standard", "spec"],
    },
    Family {
        key: "knowledge",
        suppress_verbs: false,
        members: &["memory", "knowledge", "backlog"],
    },
    Family {
        key: "relations",
        suppress_verbs: false,
        members: &["link", "unlink", "needs", "after", "supersede"],
    },
    Family {
        key: "facets",
        suppress_verbs: false,
        members: &["estimate", "value", "risk", "tag"],
    },
    Family {
        key: "reports",
        suppress_verbs: false,
        members: &["status", "next", "blockers", "survey", "explain"],
    },
    Family {
        key: "explore",
        suppress_verbs: false,
        members: &["search", "inspect", "relation", "concept-map", "map"],
    },
    Family {
        key: "infra",
        suppress_verbs: true,
        members: &[
            "install",
            "boot",
            "serve",
            "config",
            "validate",
            "reseat",
            "export",
            "reservation",
            "worktree",
            "dispatch",
            "catalog",
        ],
    },
];

/// Verbs every entity kind shares; subtracted to leave the distinctive set (SL-150).
/// `status` is deliberately NOT in the spine — not universal, lifecycle-bearing, so it
/// surfaces as distinctive where present.
const SPINE: &[&str] = &["new", "list", "show", "paths"];

/// One row in the top-level help table.
struct HelpEntry {
    name: String,
    about: String,
}

/// Render the top-level command list as a comfy-table, replacing clap's built-in
/// help output. Called from `main()` when `--help` is requested at the top level.
///
/// SL-150: commands are grouped by [`FAMILIES`] (declared order; members in member-array
/// order — INV-4) and rendered from ONE underlying table (shared column widths) via
/// [`crate::listing::render_grouped`], which injects a full-width family-heading band at
/// each group boundary. No column header row — families are the structure (A2). A member
/// that does not resolve to a visible command is skipped (the drift test guards against
/// that ever happening in practice).
pub(crate) fn render_top_level_help(color: bool, term_width: Option<u16>) -> String {
    use crate::listing::{self, Column, ColumnPaint, RenderOpts};

    let cmd = <crate::Cli as CommandFactory>::command();
    let about_of = |name: &str| -> Option<String> {
        cmd.get_subcommands()
            .find(|sub| !sub.is_hide_set() && sub.get_name() == name)
            .map(|sub| sub.get_about().map_or(String::new(), ToString::to_string))
    };

    let groups: Vec<(&str, Vec<HelpEntry>)> = FAMILIES
        .iter()
        .map(|fam| {
            let entries: Vec<HelpEntry> = fam
                .members
                .iter()
                .filter_map(|name| {
                    about_of(name).map(|about| HelpEntry {
                        name: (*name).to_string(),
                        about,
                    })
                })
                .collect();
            (fam.key, entries)
        })
        .collect();

    let cols: &[&Column<HelpEntry>] = &[
        &Column {
            name: "command",
            header: "command",
            cell: |e| e.name.clone(),
            paint: ColumnPaint::None,
        },
        &Column {
            name: "description",
            header: "description",
            cell: |e| e.about.clone(),
            paint: ColumnPaint::Alternate([listing::TITLE_EVEN, listing::TITLE_ODD]),
        },
    ];

    let opts = RenderOpts { color, term_width };
    listing::render_grouped(&groups, cols, opts)
}

/// Render the dense boot-map projection (SL-150 §5.4) — a plain-text, PUSH-tier
/// command surface for the boot snapshot. PURE: a function of the compiled clap
/// tree + the [`FAMILIES`]/[`SPINE`] taxonomy only — no clock/rng/disk/tty, so
/// two runs are byte-identical (INV-3).
///
/// Layout:
/// - a spine legend line once at the top (the [`SPINE`] verbs);
/// - per family (FAMILIES order), a header line `{key}  {member member …}` —
///   all members bare, member-array order (INV-4);
/// - a sub-line for a command IFF its distinctive set (subcommand verbs − SPINE,
///   in clap derive order — INV-4) is non-empty AND its family's `suppress_verbs`
///   is false. Leaves (no subcommands) and infra families get a header only (D7).
///
/// Names (family keys and sub-line command names) share one left-padded field
/// width so the surface scans as a column.
pub(crate) fn render_boot_map() -> String {
    use std::fmt::Write as _;

    let cmd = <crate::Cli as CommandFactory>::command();

    // Distinctive verbs for one command: its visible subcommand names, minus the
    // SPINE, preserving clap derive order. Empty for leaves / spine-only commands.
    let distinctive = |name: &str| -> Vec<String> {
        cmd.get_subcommands()
            .find(|sub| !sub.is_hide_set() && sub.get_name() == name)
            .map(|sub| {
                sub.get_subcommands()
                    .filter(|g| !g.is_hide_set() && g.get_name() != "help")
                    .map(|g| g.get_name().to_string())
                    .filter(|verb| !SPINE.contains(&verb.as_str()))
                    .collect()
            })
            .unwrap_or_default()
    };

    // Shared name-field width: the longest family key, plus a two-space gutter.
    let pad = FAMILIES.iter().map(|f| f.key.len()).max().unwrap_or(0) + 2;

    let mut out = String::new();
    out.push_str("SPINE: ");
    out.push_str(&SPINE.join(" "));
    out.push_str(" (+status where lifecycle) \u{2014} entity kinds\n\n");

    for fam in FAMILIES {
        _ = writeln!(out, "{:<pad$}{}", fam.key, fam.members.join(" "));
        if fam.suppress_verbs {
            continue;
        }
        for member in fam.members {
            let verbs = distinctive(member);
            if verbs.is_empty() {
                continue;
            }
            // sub-line: two-space indent + the same padded name field + verbs.
            _ = writeln!(out, "  {:<pad$}{}", member, verbs.join(" "));
        }
    }
    out
}

/// One row in the `--commands` subcommand-grouped help table.
struct VerbEntry {
    command: String,
    verb: String,
    description: String,
}

/// Truncate a description to its first sentence for the summary table.
/// Splits on `. ` where the next character is uppercase or a backtick —
/// avoids false splits on abbreviations ("e.g.", "i.e.", "§5.3").
/// If no such break exists, returns the full text unchanged.
fn first_sentence(about: &str) -> String {
    let mut pos = 0;
    while let Some(candidate) = about[pos..].find(". ") {
        let abs = pos + candidate;
        // Skip known abbreviations: "e.g. " and "i.e. "
        if about[..abs].ends_with("e.g") || about[..abs].ends_with("i.e") {
            pos = abs + 1; // advance past this period, keep looking
            continue;
        }
        // Check the character after ". " — must start a new sentence
        if let Some(next_char) = about[abs + 2..].chars().next()
            && (next_char.is_ascii_uppercase() || next_char == '`')
        {
            // Return up to and including the period (drop the space)
            return about[..=abs].to_string();
        }
        pos = abs + 1; // advance past this period, keep looking
    }
    about.to_string()
}

/// Render the `--help --commands` table: three-column (`command | verb | description`)
/// with each top-level command's subcommands grouped beneath it. The command name
/// appears only on the first subcommand row; continuation rows leave it blank.
/// Leaf commands (no subcommands) get a single row with an em-dash in the verb column.
/// Descriptions are truncated to the first sentence for scanability — full text
/// is available via `doctrine <command> <verb> --help`.
pub(crate) fn render_commands_table(color: bool, term_width: Option<u16>) -> String {
    use crate::listing::{self, Column, ColumnPaint, RenderOpts};

    let cmd = <crate::Cli as CommandFactory>::command();
    let mut entries: Vec<VerbEntry> = Vec::new();

    for sub in cmd
        .get_subcommands()
        .filter(|s| !s.is_hide_set() && s.get_name() != "help")
    {
        let parent = sub.get_name().to_string();
        let grandchildren: Vec<_> = sub
            .get_subcommands()
            .filter(|g| !g.is_hide_set() && g.get_name() != "help")
            .collect();

        if grandchildren.is_empty() {
            // Leaf command — single row, em-dash placeholder in verb column.
            let about = sub
                .get_about()
                .map_or(String::new(), |a| first_sentence(&a.to_string()));
            entries.push(VerbEntry {
                command: parent,
                verb: "\u{2014}".to_string(),
                description: about,
            });
        } else {
            for (i, gc) in grandchildren.into_iter().enumerate() {
                let verb = gc.get_name().to_string();
                let desc = gc
                    .get_about()
                    .map_or(String::new(), |a| first_sentence(&a.to_string()));
                entries.push(VerbEntry {
                    command: if i == 0 {
                        parent.clone()
                    } else {
                        String::new()
                    },
                    verb,
                    description: desc,
                });
            }
        }
    }

    if entries.is_empty() {
        return String::new();
    }

    let cols: &[&Column<VerbEntry>] = &[
        &Column {
            name: "command",
            header: "command",
            cell: |e| e.command.clone(),
            paint: ColumnPaint::None,
        },
        &Column {
            name: "verb",
            header: "verb",
            cell: |e| e.verb.clone(),
            paint: ColumnPaint::None,
        },
        &Column {
            name: "description",
            header: "description",
            cell: |e| e.description.clone(),
            paint: ColumnPaint::Alternate([listing::TITLE_EVEN, listing::TITLE_ODD]),
        },
    ];

    let mut out = listing::render_columns(&entries, cols, RenderOpts { color, term_width });
    out.push_str("\nFor arguments & options: doctrine <command> <verb> --help\n");
    out
}

// ── ExportCommand ───────────────────────────────────────────────────────────

#[derive(Subcommand)]
pub(crate) enum ExportCommand {
    /// Emit the corpus as a single lazyspec Brief (JSON) on stdout (SL-026).
    Lazyspec {
        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

// ── dispatch ────────────────────────────────────────────────────────────────

pub(crate) fn dispatch(cmd: Command, color: bool) -> Result<()> {
    match cmd {
        Command::Install {
            path,
            agent,
            skill,
            domain,
            only_memory,
            global,
            dry_run,
            yes,
        } => crate::install::run(
            path,
            &crate::install::InstallArgs {
                agents: &agent,
                skills: &skill,
                domains: &domain,
                only_memory,
                global,
                dry_run,
                yes,
            },
        ),
        Command::Skills { command } => crate::skills::dispatch(command, color),
        Command::ConceptMap { command } => crate::concept_map::dispatch(command, color),
        Command::Slice { command } => crate::slice::dispatch(command, color),
        Command::Memory {
            command:
                crate::memory::MemoryCommand::Sync {
                    command,
                    dry_run: sync_dry_run,
                    yes: sync_yes,
                    path: sync_path,
                },
        } => match command {
            None => crate::corpus::run_sync(sync_path, sync_dry_run, sync_yes),
            Some(crate::memory::SyncCommand::Install { path, dry_run, yes }) => {
                crate::corpus::run_sync_install(path, dry_run, yes)
            }
        },
        Command::Memory { command } => crate::memory::dispatch(command, color),
        Command::Review { command } => crate::review::dispatch(command, color),
        Command::Rec { command } => crate::rec::dispatch(command, color),
        Command::Search(args) => crate::search::run(
            args,
            crate::listing::RenderOpts {
                color,
                term_width: crate::tty::stdout_terminal_width(),
            },
        ),
        Command::Revision { command } => crate::revision::dispatch(command, color),
        Command::Reconcile {
            req,
            slice,
            r#move,
            to,
            note,
            path,
        } => crate::reconcile::run(
            path,
            &crate::reconcile::ReconcileArgs {
                req,
                slice,
                r#move,
                to,
                note,
            },
        ),
        Command::Coverage { command } => crate::commands::coverage::dispatch(command, color),
        Command::Inspect {
            id,
            format,
            json,
            path,
        } => crate::commands::inspect::run_inspect(path, &id, format, json),
        Command::Survey {
            all,
            format,
            json,
            path,
        } => crate::priority::run_survey(
            path,
            all,
            format,
            json,
            crate::listing::RenderOpts {
                color,
                term_width: crate::tty::stdout_terminal_width(),
            },
        ),
        Command::Next { format, json, path } => crate::priority::run_next(
            path,
            format,
            json,
            crate::listing::RenderOpts {
                color,
                term_width: crate::tty::stdout_terminal_width(),
            },
        ),
        Command::Blockers {
            id,
            transitive,
            format,
            json,
            path,
        } => crate::priority::run_blockers(
            path,
            &id,
            transitive,
            format,
            json,
            crate::listing::RenderOpts {
                color,
                term_width: crate::tty::stdout_terminal_width(),
            },
        ),
        Command::Explain {
            id,
            format,
            json,
            path,
        } => crate::priority::run_explain(
            path,
            &id,
            format,
            json,
            crate::listing::RenderOpts {
                color,
                term_width: crate::tty::stdout_terminal_width(),
            },
        ),
        Command::Adr { command } => crate::adr::dispatch(command, color),
        Command::Policy { command } => crate::policy::dispatch(command, color),
        Command::Standard { command } => crate::standard::dispatch(command, color),
        Command::Rfc { command } => crate::rfc::dispatch(command, color),
        Command::Spec { command } => crate::spec::dispatch(command, color),
        Command::Export { command } => match command {
            ExportCommand::Lazyspec { path } => {
                let root = crate::root::find(path, &crate::root::default_markers())?;
                let now = crate::clock::now_timestamp()?;
                let version = env!("CARGO_PKG_VERSION");
                let json = crate::lazyspec::run_export_lazyspec(&root, &now, version)?;
                writeln!(std::io::stdout(), "{json}")?;
                Ok(())
            }
        },
        Command::Backlog { command } => crate::backlog::dispatch(command, color),
        Command::Knowledge { command } => crate::knowledge::dispatch(command, color),
        Command::Tag { command } => crate::commands::tag::dispatch(command),
        Command::Reservation { command } => crate::commands::reservation::dispatch(command),
        Command::Serve { args } => crate::commands::serve::run_serve(args),
        Command::Boot {
            command,
            check,
            emit,
            path,
        } => crate::boot::dispatch(command, check, emit, path, color, render_boot_map),
        Command::Catalog { command } => crate::catalog::dispatch(command, color),
        Command::Worktree { command } => crate::worktree::dispatch(command),
        Command::Dispatch { command } => crate::dispatch::dispatch(command, color),
        Command::Validate { path } => crate::commands::validate::run_validate(path),
        Command::Reseat {
            reference,
            to,
            path,
        } => crate::integrity::run_reseat(path, &reference, to),
        Command::Relation { command } => match command {
            crate::commands::relation::RelationCommand::List {
                include_memory,
                label,
                target,
                source_kind,
                unresolved,
                format,
                json,
                path,
            } => crate::commands::relation::run_relation_list(
                path,
                include_memory,
                label,
                target,
                source_kind,
                unresolved,
                format,
                json,
            ),
            crate::commands::relation::RelationCommand::Census {
                include_memory,
                format,
                json,
                path,
            } => crate::commands::relation::run_relation_census(path, include_memory, format, json),
        },
        Command::Link {
            source,
            label,
            target,
            path,
        } => crate::commands::relation::run_link(path, &source, &label, &target),
        Command::Config { command } => {
            let root = crate::root::find(None, &crate::root::default_markers())?;
            match command {
                ConfigCommand::Show(ref args) => {
                    crate::commands::config::run_config_show(&root, args)
                }
                ConfigCommand::Set(ref args) => {
                    crate::commands::config::run_config_set(&root, args)
                }
                ConfigCommand::Get(ref args) => {
                    crate::commands::config::run_config_get(&root, args)
                }
                ConfigCommand::Unset(ref args) => {
                    crate::commands::config::run_config_unset(&root, args)
                }
            }
        }
        Command::Unlink {
            source,
            label,
            target,
            path,
        } => crate::commands::relation::run_unlink(path, &source, &label, &target),
        Command::Needs {
            source,
            target,
            path,
        } => crate::commands::dep_seq::run_needs_edge(path, &source, &target),
        Command::After {
            source,
            target,
            rank,
            remove,
            prune,
            path,
        } => {
            if prune {
                crate::commands::dep_seq::run_after_prune(path, &source)
            } else if remove {
                crate::commands::dep_seq::run_after_remove(
                    path,
                    &source,
                    target.as_deref().unwrap_or(""),
                    rank,
                )
            } else {
                crate::commands::dep_seq::run_after_edge(
                    path,
                    &source,
                    target.as_deref().unwrap_or(""),
                    rank,
                )
            }
        }
        Command::Status { format, json, path } => crate::status::run(path, format, json),
        Command::Estimate { action } => match action {
            EstimateAction::Set(args) => crate::commands::facet::run_estimate_set(&args),
            EstimateAction::Clear(args) => crate::commands::facet::run_estimate_clear(&args),
        },
        Command::Value { action } => match action {
            ValueAction::Set(args) => crate::commands::facet::run_value_set(&args),
            ValueAction::Clear(args) => crate::commands::facet::run_value_clear(&args),
        },
        Command::Risk { action } => match action {
            RiskAction::Set(args) => crate::commands::facet::run_risk_set(&args),
            RiskAction::Clear(args) => crate::commands::facet::run_risk_clear(&args),
        },
        Command::Supersede { new, old, path } => {
            crate::commands::supersede::run_supersede(path, &new, &old)
        }
        Command::Map { command } => crate::commands::map::dispatch(command),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test code: fail-fast on internal invariant violations"
)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    /// Visible top-level command names: `!is_hide_set` and ≠ `help` — the
    /// classification denominator (mirrors the render filters, INV-1 EDGE).
    fn visible_commands() -> Vec<String> {
        let cmd = <crate::Cli as CommandFactory>::command();
        cmd.get_subcommands()
            .filter(|s| !s.is_hide_set() && s.get_name() != "help")
            .map(|s| s.get_name().to_string())
            .collect()
    }

    /// VT-1 / EX-2 — the FAMILIES ⟷ clap-tree drift guard (design §9). Three
    /// assertions; set equality alone is insufficient (a command in two families
    /// dedups in the union and would pass), so this builds a name→family collision
    /// map. INV-1 (total partition, no orphan), INV-2 (no phantom), no duplicate.
    #[test]
    fn families_partition_the_visible_command_tree() {
        let visible: std::collections::BTreeSet<String> = visible_commands().into_iter().collect();

        // (a) no duplicate member: a second insert for a name is a collision.
        let mut owner: BTreeMap<&str, &str> = BTreeMap::new();
        for fam in FAMILIES {
            for &member in fam.members {
                if let Some(prev) = owner.insert(member, fam.key) {
                    panic!(
                        "command `{member}` is in two families (`{prev}` and `{}`)",
                        fam.key
                    );
                }
            }
        }

        // (b) no phantom: every member resolves to a real visible command.
        for (&member, &family) in &owner {
            assert!(
                visible.contains(member),
                "FAMILIES member `{member}` (family `{family}`) is not a visible command"
            );
        }

        // (c) no orphan: every visible command is in some family (INV-1).
        for name in &visible {
            assert!(
                owner.contains_key(name.as_str()),
                "visible command `{name}` is not classified into any family"
            );
        }

        // The slice's stated census: 44 visible top-level commands (A1).
        assert_eq!(visible.len(), 44, "expected 44 visible top-level commands");
    }

    /// R-a — narrow-width WRAP case (design watchout): at a width that forces the
    /// description column to wrap, band injection must still map each continuation
    /// line to the right family (a continuation has a blank first column, so
    /// `is_table_row_start` is false and no spurious band is emitted mid-row). Assert
    /// exactly 8 family headings survive wrapping, and every `  {key}` heading is
    /// immediately preceded by a blank line (never mid-row).
    #[test]
    fn narrow_width_wrap_keeps_eight_bands_and_no_mid_row_heading() {
        let out = render_top_level_help(false, Some(40));
        let lines: Vec<&str> = out.lines().collect();
        let keys: std::collections::BTreeSet<&str> = [
            "change",
            "governance",
            "knowledge",
            "relations",
            "facets",
            "reports",
            "explore",
            "infra",
        ]
        .into_iter()
        .collect();
        let mut headings = 0;
        for (i, line) in lines.iter().enumerate() {
            if let Some(rest) = line.strip_prefix("  ")
                && keys.contains(rest)
            {
                headings += 1;
                assert!(
                    i > 0 && lines[i - 1].is_empty(),
                    "wrapped output put family band `{rest}` mid-row (no blank above)"
                );
            }
        }
        assert_eq!(headings, 8, "all 8 family bands must survive wrapping");
        // Wrapping actually happened: some line exceeds none and a continuation
        // (blank first column, no separator-leading token) exists.
        assert!(
            lines.iter().any(|l| {
                l.starts_with(' ') && !l.trim_start().is_empty() && {
                    let head = l.split('\u{2502}').next().unwrap_or(l);
                    head.chars().all(char::is_whitespace)
                }
            }),
            "the 40-col width must actually wrap at least one description"
        );
    }

    /// VA-1 — colour-ON smoke (design §9): the family-heading band paints its
    /// background SGR escape and pads edge-to-edge to `term_width`. NOT a byte
    /// golden — asserts escape-code presence + full-width pad, not exact bytes.
    #[test]
    fn colour_on_help_paints_full_width_family_bands() {
        let out = render_top_level_help(true, Some(80));
        // A band carries an SGR background escape (`\x1b[…m`).
        assert!(
            out.contains('\u{1b}'),
            "colour-on help must emit ANSI escapes"
        );
        // The first family band header is present and painted.
        assert!(
            out.contains("change"),
            "first family heading `change` must appear"
        );
        // Full-width pad: at least one painted line reaches the 80-col width once
        // ANSI escapes are stripped (the band fills edge-to-edge).
        let widest = out
            .lines()
            .map(|l| crate::listing::strip_ansi(l).chars().count())
            .max()
            .unwrap_or(0);
        assert!(
            widest >= 80,
            "a band line must pad to term_width (80); widest visible was {widest}"
        );
    }
}
