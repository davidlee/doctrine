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

/// One row in the top-level help table.
struct HelpEntry {
    name: String,
    about: String,
}

/// Render the top-level command list as a comfy-table, replacing clap's built-in
/// help output. Called from `main()` when `--help` is requested at the top level.
pub(crate) fn render_top_level_help(color: bool, term_width: Option<u16>) -> String {
    use crate::listing::{self, Column, ColumnPaint, RenderOpts};

    let cmd = <crate::Cli as CommandFactory>::command();
    let entries: Vec<HelpEntry> = cmd
        .get_subcommands()
        .filter(|sub| !sub.is_hide_set())
        .map(|sub| {
            let name = sub.get_name().to_string();
            let about = sub.get_about().map_or(String::new(), ToString::to_string);
            HelpEntry { name, about }
        })
        .collect();

    if entries.is_empty() {
        return String::new();
    }

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
    listing::render_columns(&entries, cols, opts)
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
            let about = sub.get_about().map_or(String::new(), |a| first_sentence(&a.to_string()));
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
                    command: if i == 0 { parent.clone() } else { String::new() },
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
        } => crate::boot::dispatch(command, check, emit, path, color),
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
