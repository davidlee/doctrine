// SPDX-License-Identifier: GPL-3.0-only
mod adr;
mod backlog;
mod backlog_order;
mod boot;
mod catalog;
mod clock;
mod commands;
mod concept_map;
mod conduct;
mod contentset;
mod corpus;
mod coverage;
mod coverage_scan;
mod coverage_store;
mod coverage_verify;
mod coverage_view;
mod dep_seq;
mod dispatch;
mod dispatch_config;
mod dtoml;
mod entity;
mod estimate;
mod facet_write;
mod fsutil;
mod git;
mod governance;
mod input;
mod install;
mod integrity;
mod kinds;
mod knowledge;
mod lazyspec;
mod ledger;
mod lexical;
mod lifecycle;
pub(crate) mod links;
mod listing;
mod map_server;
mod mcp_server;
mod memory;
mod meta;
mod plan;
mod policy;
mod priority;
mod projection;
mod rec;
mod reconcile;
mod registry;
mod relation;
mod relation_graph;
mod requirement;
mod retrieve;
mod review;
mod revision;
mod rfc;
mod root;
mod skills;
mod slice;
mod spec;
mod standard;
mod state;
mod status;
mod supersede;
mod tag;
mod tomlfmt;
mod tty;
mod value;
mod verify;
mod worktree;

use std::io;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{Args, Parser, Subcommand};

use crate::commands::map::MapServeArgs;
// unused: Ext, id_path, rel_path
use crate::listing::{Format, ListArgs};

fn parse_expand_depth(s: &str) -> Result<usize, String> {
    let depth = s
        .parse::<usize>()
        .map_err(|_err| "expand depth must be a number")?;
    if depth == 0 {
        return Err("expand depth must be >= 1".to_string());
    }
    Ok(depth)
}

/// doctrine — project tooling.
#[derive(Parser)]
#[command(name = "doctrine", about = "doctrine CLI")]
struct Cli {
    /// Control colour output
    #[arg(long, default_value = "auto", global = true)]
    color: clap::ColorChoice,

    #[command(subcommand)]
    command: Command,
}

/// The shared, invariant list-surface flags (SL-025 §5.2) — one composable
/// `#[derive(Args)]` bundle flattened into every kind's `list` variant. It is the
/// mandatory spine of the read surface: a kind cannot quietly grow bespoke list
/// flags. Lives command-side (not in the `listing` leaf) so `clap` stays out of
/// the leaf (ADR-001 / A-3); `--format` wires `Format::from_str` via `value_parser`
/// rather than `ValueEnum`, which would drag clap into the leaf.
#[derive(Args, Debug)]
pub(crate) struct CommonListArgs {
    /// Substring filter on slug+title (case-insensitive).
    #[arg(long, short = 'f')]
    pub(crate) filter: Option<String>,

    /// Regex over canonical-id + slug + title.
    #[arg(long, short = 'r')]
    pub(crate) regexp: Option<String>,

    /// Make the regex case-insensitive.
    #[arg(long, short = 'i')]
    pub(crate) case_insensitive: bool,

    /// Status filter, multi-value (`-s draft,active`); any value reveals the
    /// hide-set.
    #[arg(long, short = 's', value_delimiter = ',')]
    pub(crate) status: Vec<String>,

    /// Tag filter, repeatable (OR logic).
    #[arg(long, short = 't')]
    pub(crate) tag: Vec<String>,

    /// Show every state, including the kind's terminal hide-set.
    #[arg(long, short = 'a')]
    pub(crate) all: bool,

    /// Output format.
    #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
    pub(crate) format: Format,

    /// Shorthand for `--format json`.
    #[arg(long)]
    pub(crate) json: bool,

    /// Select/order visible table columns, e.g. `--columns id,status,slug`.
    /// Unknown names error with the available set. No effect with `--json`
    /// (JSON rows are faithful/full — SL-037 D7).
    #[arg(long, value_delimiter = ',')]
    pub(crate) columns: Option<Vec<String>>,
}

impl CommonListArgs {
    /// Lower the parsed clap bundle onto the clap-free leaf input ([`ListArgs`]).
    /// The seam where command-layer clap types stop and the pure spine begins.
    pub(crate) fn into_list_args(self, color: bool) -> ListArgs {
        ListArgs {
            substr: self.filter,
            regexp: self.regexp,
            case_insensitive: self.case_insensitive,
            status: self.status,
            tags: self.tag,
            all: self.all,
            format: self.format,
            json: self.json,
            columns: self.columns,
            // Resolve terminal capability ONCE at the clap→leaf seam (SL-053 SL-079 D3):
            // colour is now injected by the caller via --color flag resolution;
            // term_width is still resolved here (no flag override).
            render: crate::listing::RenderOpts {
                color,
                term_width: crate::tty::stdout_terminal_width(),
            },
        }
    }
}

/// Shared scope/filter/format fields for `MemoryCommand::Find` and
/// `MemoryCommand::Retrieve`. Both variants flatten this struct via
/// `#[command(flatten)]` — each shared field is defined once (DRY).
#[derive(Args, Debug)]
pub(crate) struct FindRetrieveArgs {
    /// Path scope probe, repeatable (`-p`/`--path` is the project root).
    #[arg(long = "path-scope")]
    pub(crate) path_scope: Vec<String>,

    /// Glob scope probe, repeatable.
    #[arg(long = "glob")]
    pub(crate) glob: Vec<String>,

    /// Command scope probe, repeatable.
    #[arg(long = "command")]
    pub(crate) command: Vec<String>,

    /// Tag scope probe, repeatable.
    #[arg(long = "tag")]
    pub(crate) tag: Vec<String>,

    /// Free-text lexical query (not a scope constraint).
    #[arg(long = "query")]
    pub(crate) flag_query: Option<String>,

    /// Hard filter by type: concept|fact|pattern|signpost|system|thread.
    #[arg(long = "type", value_parser = memory::MemoryType::parse)]
    pub(crate) memory_type: Option<memory::MemoryType>,

    /// Hard filter by lifecycle status.
    #[arg(long, value_parser = memory::Status::parse)]
    pub(crate) status: Option<memory::Status>,

    /// Hard filter by lifespan.
    #[arg(long, value_parser = memory::Lifespan::from_str)]
    pub(crate) lifespan: Option<memory::Lifespan>,

    /// Output format.
    #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
    pub(crate) format: Format,

    /// Shorthand for `--format json`.
    #[arg(long)]
    pub(crate) json: bool,

    /// Include `draft` memories (excluded by default).
    #[arg(long = "include-draft")]
    pub(crate) include_draft: bool,

    /// Skip first N results (default 0).
    #[arg(long, default_value_t = 0)]
    pub(crate) offset: usize,

    /// Page number (1-based; sugar over --offset). Mutually exclusive with --offset.
    #[arg(long, conflicts_with = "offset")]
    pub(crate) page: Option<usize>,

    /// Max results to show.
    #[arg(long)]
    pub(crate) limit: Option<usize>,

    /// Explicit project root (default: auto-detect).
    #[arg(short = 'p', long)]
    pub(crate) path: Option<PathBuf>,

    /// Expand graph by traversing relations N levels deep (retrieve only).
    #[arg(long, value_parser = parse_expand_depth)]
    pub(crate) expand: Option<usize>,
}

use crate::commands::facet::{EstimateClearArgs, EstimateSetArgs, ValueClearArgs, ValueSetArgs};

#[derive(clap::Subcommand)]
enum EstimateAction {
    /// Set estimate bounds
    Set(EstimateSetArgs),
    /// Clear the estimate facet
    Clear(EstimateClearArgs),
}

#[derive(clap::Subcommand)]
enum ValueAction {
    /// Set value magnitude
    Set(ValueSetArgs),
    /// Clear the value facet
    Clear(ValueClearArgs),
}

#[derive(Subcommand)]
enum Command {
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

    /// Debug catalog inspection — thin JSON dump of the hydrated entity corpus
    /// (`scan`) and its graph projection (`graph`). Developer-facing; not gating
    /// for acceptance (SL-071 D12).
    Catalog {
        #[command(subcommand)]
        command: CatalogCommand,
    },

    /// List available skills and their install status. Hidden deprecated alias
    /// — the consolidated `install` surface is the primary path.
    #[command(hide = true)]
    Skills {
        #[command(subcommand)]
        command: SkillsCommand,
    },

    /// Start the local map explorer web server.
    Map {
        #[command(subcommand)]
        command: MapCommand,
    },

    /// Create, list, and show concept maps — DSL-driven relationship diagrams.
    ConceptMap {
        #[command(subcommand)]
        command: ConceptMapCommand,
    },

    /// Create and list slices — the unit of intentional change.
    Slice {
        #[command(subcommand)]
        command: SliceCommand,
    },

    /// Record, show, and list memories.
    Memory {
        #[command(subcommand)]
        command: MemoryCommand,
    },

    /// Create, show, and list adversarial-review ledgers (the RV kind, ADR-007).
    Review {
        #[command(subcommand)]
        command: ReviewCommand,
    },

    /// Create, show, and list reconciliation records (the REC kind, SPEC-002).
    Rec {
        #[command(subcommand)]
        command: RecCommand,
    },

    /// Create, show, and transition revisions (the REV change-axis kind, ADR-013).
    Revision {
        #[command(subcommand)]
        command: RevisionCommand,
    },

    /// Reconcile ONE requirement against observed coverage — the sole author of
    /// reconciled requirement status (SL-044). Applies exactly one move and emits
    /// one atomic REC. `--to` is required for accept/revise, omitted for redesign.
    Reconcile {
        /// The requirement to reconcile, canonical `REQ-NNN`.
        req: String,

        /// The owning slice this act is recorded against, canonical `SL-NNN`.
        #[arg(long)]
        slice: String,

        /// The reconciliation move: accept | revise | redesign.
        #[arg(long = "move", value_parser = rec::RecMove::parse)]
        r#move: rec::RecMove,

        /// The explicit target status (required for accept/revise; omit for
        /// redesign). The WRITTEN status — never derived from coverage (NF-001).
        #[arg(long, value_enum)]
        to: Option<requirement::ReqStatus>,

        /// Optional operator note (surfaced; not stored in the REC).
        #[arg(long)]
        note: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Requirement coverage: the read-only drift view (`show`) plus the
    /// observed-tier write path (`record`/`verify`/`forget`, SL-057).
    Coverage {
        #[command(subcommand)]
        command: CoverageCommand,
    },

    /// Read-only cross-kind relation view of one entity (`<ID>` = SL-NNN, ADR-NNN,
    /// …): its authored outbound relations, the derived inbound relations, and any
    /// unresolved / free-text dangling targets — grouped, direct-only (one hop).
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

    /// Read-only cross-kind importance survey: every ELIGIBLE entity in importance
    /// order (actionability, then consequence desc, then canonical-id), each blocked
    /// row carrying a BLOCKED badge and its direct blocker. Terminal and
    /// promoted-backlog items are excluded unless `--all`. Advisory — never writes.
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

    /// Read-only advisory worklist: the ACTIONABLE entities (eligible AND unblocked),
    /// in composed dependency/sequence order. Blocked items are absent (the divergence
    /// from `survey`). Mutates nothing.
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

    /// Read-only blocker view of one entity (`<ID>` = SL-NNN, ISS-NNN, …): its direct
    /// blocked-by prerequisites + the items it is blocking. `--transitive` walks both
    /// chains. Display depth never reorders.
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

    /// Read-only structured explanation of one entity's priority (`<ID>`): its
    /// eligibility reason, the transitive blocker chain, the order-key contributors,
    /// any evicted soft-sequence edges, and its consequence — always to root.
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
        command: AdrCommand,
    },

    /// Create and list governance policies (standing rules).
    Policy {
        #[command(subcommand)]
        command: PolicyCommand,
    },

    /// Create and list governance standards (standing conventions of practice).
    Standard {
        #[command(subcommand)]
        command: StandardCommand,
    },

    /// Create and list RFC discussion artifacts — governance-neutral deliberation.
    Rfc {
        #[command(subcommand)]
        command: RfcCommand,
    },

    /// Create and list product / technical specifications.
    Spec {
        #[command(subcommand)]
        command: SpecCommand,
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
        command: BacklogCommand,
    },

    /// Capture and survey durable knowledge records (assumption / decision /
    /// question / constraint).
    Knowledge {
        #[command(subcommand)]
        command: KnowledgeCommand,
    },

    /// Start the MCP stdio server (`serve --mcp`).
    Serve {
        #[command(flatten)]
        args: commands::serve::ServeArgs,
    },

    /// Regenerate the cache-friendly governance snapshot, or `boot install` to wire it.
    Boot {
        /// Wire the `@`-import + per-harness session refresh (omit to regenerate).
        #[command(subcommand)]
        command: Option<BootCommand>,

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
        command: WorktreeCommand,
    },

    /// Dispatch coordination-branch projection (SL-064 / ADR-012): the
    /// integration-sync seam that materialises reviewable refs from
    /// `dispatch/<slice>`. Orchestrator-classed — refused under worker-mode.
    Dispatch {
        #[command(subcommand)]
        command: DispatchCommand,
    },

    /// Scan every numbered entity kind for id-integrity violations (ADR-006 D3
    /// detect-half): dir basename == toml id, no intra-kind duplicate id, and
    /// alias target equality. Exits non-zero on any violation.
    Validate {
        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Renumber an entity's canonical id (ADR-006 D3 repair). Takes a canonical
    /// ref (`SL-031`), moves it to the next free trunk-aware id or `--to <NNN>`,
    /// and reports inbound prose citations as danglers (never rewrites them).
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

    /// Author a tier-1 `[[relation]]` edge: `link SL-048 governed_by ADR-010`
    /// (SL-048 §5.4). The label must be `link`-writable for the source kind, and the
    /// target must resolve to an entity of a legal kind (forward-edge validation,
    /// §5.5). Idempotent — re-linking an existing edge is a no-op.
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

    /// Remove a tier-1 `[[relation]]` edge authored by `link` (SL-048 §5.4). Symmetric
    /// on the same write seam; idempotent — unlinking an absent edge is a no-op.
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

    /// Append a hard prerequisite to a source entity's `needs` axis: `needs SL-060
    /// SL-047` (SL-060 §5.4). Generic cross-kind: SRC and TGT resolve via the same
    /// canonical-ref seam as `link`. SRC must be a dep/seq-authoring kind (slice or a
    /// backlog kind); TGT must resolve AND be work-like (slice or backlog) — a
    /// free-text or non-work-like target is refused at author time. Idempotent.
    Needs {
        /// The source entity's canonical ref, e.g. `SL-060`.
        source: String,
        /// The prerequisite target's canonical ref, e.g. `SL-047`.
        target: String,
        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Append a soft-sequence edge to a source entity's `after` axis: `after SL-060
    /// SL-047 [--rank N]` (SL-060 §5.4). Generic cross-kind with the same author-time
    /// target gate as `needs`. Records `{ to, rank }` (rank default 0). Idempotent.
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

    /// Read-only project orientation dashboard: active work, blocked items, boot
    /// staleness, recent commits. 10–20 lines human output; structured JSON.
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

    /// Record that NEW supersedes OLD: `supersede ADR-012 ADR-004` (SL-062 §5.4).
    /// ADR-first — one parse-once / hold-both / write-once transaction writes
    /// `NEW.supersedes += OLD`, `OLD.superseded_by += NEW` (the single sanctioned
    /// reverse carve-out, ADR-004 §5), and flips `OLD.status → superseded`. Refuses
    /// a self-edge, cross-kind refs, a non-ADR kind, and an OLD already superseded by
    /// a different ADR. Idempotent — a re-run with all three surfaces present is a no-op.
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
}

#[derive(Subcommand)]

enum CatalogCommand {
    /// Thin JSON dump of the hydrated entity corpus `Catalog` — entities,
    /// edges, and diagnostics. Output is always JSON (no format choice).
    Scan {
        /// Explicit corpus root (default: auto-detect from CWD).
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Thin JSON dump of the `CatalogGraph` — nodes and edges.
    /// Output is always JSON (no format choice).
    Graph {
        /// Explicit corpus root (default: auto-detect from CWD).
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum WorktreeCommand {
    /// Copy allowlisted gitignored files from the source tree into `<fork>` —
    /// the sole copy path; the coordination/runtime tier is always excluded.
    Provision {
        /// The target sibling worktree to populate.
        fork: PathBuf,

        /// Explicit source project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Static smell test: nonzero exit if any `.worktreeinclude` pattern names a
    /// withheld tier or uses unsupported syntax (`!`/anchoring).
    CheckAllowlist {
        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// HEAD-stationarity assert at the batch-commit boundary (SL-031, D5
    /// concurrency extension): exit 0 if coordination HEAD still equals the
    /// orchestrator's pre-spawn base, 1 otherwise (→ re-dispatch). Not a
    /// merge-base compute (C-V).
    BranchPointCheck {
        /// The orchestrator's pre-spawn captured base commit `B`.
        #[arg(long)]
        base: String,

        /// HEAD to compare against (default: `git rev-parse HEAD`).
        #[arg(long)]
        head: Option<String>,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Create an orchestrator-owned worktree fork off `<base>` on a NEW branch,
    /// provision it (the sole copier), optionally stamp the worker marker, and emit
    /// the per-worktree env contract on stdout (SL-056 PHASE-06). Orchestrator-classed
    /// — refused under worker-mode. Atomic via compensating rollback.
    Fork {
        /// The base commit `B` the fork is created from (the orchestrator's
        /// captured coordination HEAD).
        #[arg(long)]
        base: String,

        /// The NEW branch to create at `<base>` for the fork.
        #[arg(long)]
        branch: String,

        /// The fork worktree directory (must not already exist; unique per branch).
        #[arg(long)]
        dir: PathBuf,

        /// Stamp the worker-mode marker so the fork resolves to worker mode.
        #[arg(long)]
        worker: bool,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Create OR resume the dispatch coordination worktree for a slice on branch
    /// `dispatch/<slice>` off the resolved trunk (SL-064 §2). MARKERLESS — the
    /// coordination tree IS the orchestrator, so no worker marker is stamped;
    /// provisions via the sole copier and regenerates the runtime phase sheets
    /// from committed `plan.toml`. A live worktree already on `dispatch/<slice>`
    /// is refused (`coordination-live`); a branch with no live worktree resumes
    /// (reattach, never a second branch). Orchestrator-classed — refused under
    /// worker-mode.
    Coordinate {
        /// The slice id (bare number, e.g. `64`) whose `dispatch/<slice>`
        /// coordination worktree to create or resume.
        #[arg(long)]
        slice: u32,

        /// The coordination worktree directory (must not already exist).
        #[arg(long)]
        dir: PathBuf,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Import a worker fork's single-commit delta into the coordination index,
    /// NON-committing (SL-056 PHASE-07, ADR-006 D7: import ≠ commit). Stationary-
    /// head case only — fails closed with a distinct token on any precond/belt
    /// violation (`head-moved`/`tree-unclean`/`multi-commit`/`doctrine-touch`/
    /// `claude-touch`); never auto-merges. Orchestrator-classed — refused under
    /// worker-mode.
    Import {
        /// The orchestrator's pre-spawn captured base commit `B`.
        #[arg(long)]
        base: String,

        /// The fork branch carrying the single non-merge commit `S` (`S^ == B`).
        #[arg(long)]
        fork: String,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Land a solo multi-commit isolated-worktree TDD branch onto the coordination
    /// branch with ancestry PRESERVED via `git merge --no-ff` (NEVER `--squash` —
    /// the verb cannot express a squash; SL-056 PHASE-08, design §6). Solo
    /// `/execute`'s analog of `import`. Fails closed with a distinct token on any
    /// precond/merge violation (`tree-unclean`/`no-such-fork`/`worktree-gone`/
    /// `dispatch-fork`/`merge-conflict`/`wedged-merge`/`inconsistent-merge-state`).
    /// Orchestrator-classed — refused under worker-mode.
    Land {
        /// The solo fork branch to merge onto the coordination branch.
        #[arg(long)]
        fork: String,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Reap a spent worktree fork in ONE idempotent act (SL-056 PHASE-09, design
    /// §8) — deletes ONLY when the fork has provably landed via the two-leg
    /// (ancestry ∪ patch-id) durable-git oracle (§8.1). Fails closed with a distinct
    /// token (`not-landed`/`squash-uncertifiable`); `--superseded-head <SHA>` reaps
    /// iff the SHA equals the branch's current head (a movement-guard, not a landing
    /// proof); `--force` bypasses the oracle; `--dry-run` prints the verdict and
    /// destroys nothing. A crash-interrupted gc completes (or names the leftover) on
    /// rerun (§8.2). Orchestrator-classed — refused under worker-mode.
    Gc {
        /// The fork branch to reap.
        #[arg(long)]
        fork: String,

        /// Reap iff this SHA equals the branch's current head (the moved-HEAD
        /// re-dispatch case: a spent-yet-never-landed fork). A movement-guard, not a
        /// landing proof.
        #[arg(long)]
        superseded_head: Option<String>,

        /// Bypass the landed oracle and reap knowingly.
        #[arg(long)]
        force: bool,

        /// Compute and print the per-fork verdict, destroying nothing.
        #[arg(long)]
        dry_run: bool,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Print the resolved worker-mode and cause (SL-056 §3). `--assert` derives a
    /// non-zero `stale-marker` exit from the SAME state the human line reads.
    /// Read-classed — open to workers.
    Status {
        /// Gate exit: non-zero with a `stale-marker` token if a stray marker sits
        /// in this linked worktree (clean direct-writer entry ⇒ exit 0).
        #[arg(long)]
        assert: bool,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Post-spawn base==B check for the claude `/dispatch` arm (SL-064 §8): prove
    /// the spawned worker worktree's HEAD descends from the base `B` it was meant
    /// to fork off. Diagnostic only — fail-loud, NEVER removes the fork. Read-classed
    /// (callable under worker-mode). Distinct token per refusal
    /// (`no-worker-head`/`not-isolated`/`unstamped`/`wrong-base`/`branch-mismatch`).
    VerifyWorker {
        /// The base commit `B` the worker was meant to fork off (the
        /// orchestrator's coordination HEAD at spawn).
        #[arg(long)]
        base: String,

        /// The worker worktree to verify — the git `-C` root for every probe.
        #[arg(long)]
        dir: PathBuf,

        /// The worker fork branch S — binds HEAD(--dir) == tip(S) (dir↔branch coherence).
        #[arg(long)]
        branch: Option<String>,
    },

    /// Manage the worker-mode disk marker (SL-056 §3). `--clear` removes it at the
    /// cwd tree root with a loud receipt — the self-brick cure; never refused by
    /// the marker conjunct itself.
    Marker {
        /// Remove the marker at the cwd tree root.
        #[arg(long)]
        clear: bool,

        /// Confirm a clear inside a linked worktree (the accident-fence).
        #[arg(long)]
        operator: bool,

        /// Provision + stamp the worker marker into the `SubagentStart` payload's
        /// worktree (SL-056 PHASE-10). Reads `{cwd, agent_type}` JSON on stdin;
        /// the claude harness spawn path's mark step.
        #[arg(long)]
        stamp_subagent: bool,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum DispatchCommand {
    /// Materialise reviewable refs from `dispatch/<slice>` (SL-064 / ADR-012
    /// §4). The stage selector is required and single-choice; PHASE-04 ships
    /// `--prepare-review` (stage-1: create `review/<slice>` + `phase/<slice>-NN`
    /// under a CAS journal, never writing trunk). Orchestrator-classed — refused
    /// under worker-mode.
    Sync {
        /// The slice id (bare number, e.g. `64`) whose `dispatch/<slice>`
        /// coordination branch to project.
        #[arg(long)]
        slice: u32,

        /// Stage-1: create the reviewable `review/<slice>` and `phase/<slice>-NN`
        /// refs from the dispatch tip; never writes trunk.
        #[arg(long, group = "stage", required = true)]
        prepare_review: bool,

        /// Stage-2: replay the prepared journal idempotently and project the
        /// audited code units (opt-in `--trunk`/`--edge`); runs from parent/root
        /// after the coordination worktree is removed. Never auto-resolves.
        #[arg(long, group = "stage", required = true)]
        integrate: bool,

        /// Read-only (SL-121 §3(b)): print the committed journal's trunk-row
        /// `planned_new_oid` — the row whose target is `--trunk` — to stdout and
        /// exit; the close step-3a verify read surface. Tree-reads `dispatch/<slice>`,
        /// writes nothing.
        #[arg(long, group = "stage", required = true)]
        show_journal_trunk_oid: bool,

        /// Project the cumulative code units onto this trunk ref, fast-forward-only +
        /// expected-tip CAS (e.g. `refs/heads/main`) under `--integrate`; names the
        /// row to read under `--show-journal-trunk-oid`. Absent under `--integrate` ⇒
        /// trunk is left untouched.
        #[arg(long, conflicts_with = "prepare_review")]
        trunk: Option<String>,

        /// Stage-2 only: advance this standing aggregate ref to the `review/<slice>`
        /// bundle (e.g. `refs/heads/edge`). Absent ⇒ no aggregate written.
        #[arg(long, requires = "integrate")]
        edge: Option<String>,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Funnel-time recording: append a per-phase code boundary to
    /// `.doctrine/dispatch/<slice>/boundaries.toml` (design §4.3). The
    /// orchestrator runs this between the funnel's code commit and its knowledge
    /// commit; stage-1 `sync --prepare-review` tree-reads the committed file to
    /// cut the claude-arm `phase/<slice>-NN` deliverables. Orchestrator-classed —
    /// refused under worker-mode.
    RecordBoundary {
        /// The slice id (bare number, e.g. `64`) whose ledger to append.
        #[arg(long)]
        slice: u32,

        /// The `PHASE-NN` id this code boundary belongs to.
        #[arg(long)]
        phase: String,

        /// Commit-ish for HEAD before the phase's code landed (resolved to a
        /// full oid; the empty-phase test compares it to `--code-end`).
        #[arg(long)]
        code_start: String,

        /// Commit-ish for the phase's cumulative code tip, *before* the knowledge
        /// record commit (resolved to a full oid — the tree the cut snapshots).
        #[arg(long)]
        code_end: String,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Advance dispatch/<slice>'s base by merging current trunk into it in the
    /// live coordination worktree (design SL-127 §3.2). Merge-only; re-run
    /// `sync --prepare-review` after. Orchestrator-classed — refused under worker-mode.
    RefreshBase {
        #[arg(long)]
        slice: u32,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Create OR resume the coordination worktree and emit the dispatch env
    /// contract on stdout (design §2). Orchestrator-classed — refused under
    /// worker-mode.
    Setup {
        /// The slice id (bare number, e.g. `85`).
        #[arg(long)]
        slice: u32,

        /// The coordination worktree directory (must not already exist).
        #[arg(long)]
        dir: PathBuf,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Candidate lifecycle (SL-068 / design §5.3). `create` publishes a
    /// reviewable/landable candidate at `candidate/<slice>/<label>` by computing
    /// the no-ff 3-way merge of a verified source ref onto a base, under zero-oid
    /// CAS. Orchestrator-classed — refused under worker-mode.
    Candidate {
        #[command(subcommand)]
        command: CandidateCommand,
    },

    /// Read the plan and runtime phase sheets; print ordered phase rollup
    /// and identify the next actionable phase(s). Read-only — callable from
    /// anywhere.
    PlanNext {
        /// The slice id (bare number).
        #[arg(long)]
        slice: u32,

        /// Emit JSON instead of human-readable table.
        #[arg(long)]
        json: bool,

        /// Explicit project root.
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Read-only full dispatch rollup: coordination state, phase table,
    /// trunk drift, sync state, candidate summary, next-step guidance.
    Status {
        /// The slice id (bare number, e.g. `85`).
        #[arg(long)]
        slice: u32,

        /// Emit JSON instead of human-readable table.
        #[arg(long)]
        json: bool,

        /// Explicit project root.
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Print the resolved `[dispatch] deliver_to` trunk delivery ref to stdout
    /// (SL-128 / IMP-124). Read-only — callable from anywhere.
    DeliverTo {
        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum CandidateCommand {
    /// Create a candidate (the happy path: provenance gate → no-ff 3-way merge →
    /// zero-oid CAS branch → recorded row). A content conflict aborts cleanly,
    /// writing no row/ref/worktree.
    Create {
        /// The slice id (bare number, e.g. `68`).
        #[arg(long)]
        slice: u32,

        /// The human label (e.g. `review-001`); the ref is
        /// `candidate/<slice>/<label>` and the id `cand-<slice>-<label>`.
        #[arg(long, visible_alias = "target")]
        label: String,

        /// Flavour: `audit` | `experiment`.
        #[arg(long, default_value = "audit")]
        kind: String,

        /// Role: `review_surface` | `close_target` | `scratch`.
        #[arg(long)]
        role: String,

        /// Payload: `impl_bundle` | `code`.
        #[arg(long)]
        payload: String,

        /// The base ref the merge is computed against (e.g. `refs/heads/main`).
        #[arg(long)]
        base: String,

        /// The source ref merged in. Defaults to `review/<slice>` for a
        /// `review_surface`; required otherwise (e.g. a `phase/<slice>-NN`).
        #[arg(long)]
        source: Option<String>,

        /// An optional prior candidate id this fresh row supersedes (EX-2).
        #[arg(long)]
        supersedes: Option<String>,

        /// Also materialise a linked worktree at the candidate branch (opt-in
        /// here; mandatory-for-review is PHASE-03).
        #[arg(long)]
        worktree: bool,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Status (SL-068 PHASE-04): a read-only self-describing surface — lists the
    /// evidence refs and the candidate interaction branches in separate groups,
    /// reports each candidate's base/source/tip/status/admission, surfaces ref
    /// drift, and prints the safe next command(s). Read-classed — never mutates a
    /// ref or the ledger, so it works under worker-mode.
    Status {
        /// The slice id (bare number, e.g. `68`).
        #[arg(long)]
        slice: u32,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Admit (SL-068 PHASE-05): pin a recorded candidate's committed tip as the
    /// immutable OID a downstream verb (close/review) targets, after validating
    /// provenance (the recorded merge is the Doctrine candidate merge and an
    /// ancestor of the admitted tip) and re-reading the ref. Writes ONLY
    /// `candidates.toml` — never an evidence/candidate ref. Orchestrator-classed.
    Admit {
        /// The slice id (bare number, e.g. `68`).
        #[arg(long)]
        slice: u32,

        /// Role: `review_surface` | `close_target` (scratch is not admissible).
        #[arg(long)]
        role: String,

        /// The candidate ref to admit (e.g. `refs/heads/candidate/064/close-001`).
        #[arg(long)]
        candidate: String,

        /// The governing review (e.g. `RV-007`).
        #[arg(long)]
        review: Option<String>,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum BootCommand {
    /// Wire the `@`-import into CLAUDE.md/AGENTS.md and refresh each harness's
    /// session hook.
    Install {
        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,

        /// Target harness(es): claude, codex. Repeatable. Default: auto-detect.
        #[arg(long = "agent")]
        agent: Vec<String>,

        /// Compute and report the plan without writing anything.
        #[arg(long)]
        dry_run: bool,

        /// Skip the confirmation prompt.
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum AdrCommand {
    /// Allocate the next id and scaffold a new ADR.
    New {
        /// ADR title (prompted for if omitted).
        title: Option<String>,

        /// Explicit slug (default: derived from the title).
        #[arg(long)]
        slug: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// List ADRs by id: ADR-id, status, slug, title.
    List {
        #[command(flatten)]
        list: CommonListArgs,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Show one ADR: its metadata, relationships, and prose body.
    Show {
        /// ADR reference — `ADR-007` or the bare id `7`.
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

    /// Set an ADR's status (edit-preserving; a no-op if unchanged).
    Status {
        /// ADR id (numeric).
        id: u32,

        /// New status (required): proposed|accepted|rejected|superseded|deprecated.
        #[arg(long)]
        status: adr::AdrStatus,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum PolicyCommand {
    /// Allocate the next id and scaffold a new policy.
    New {
        /// Policy title (prompted for if omitted).
        title: Option<String>,

        /// Explicit slug (default: derived from the title).
        #[arg(long)]
        slug: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// List policies by id: POL-id, status, slug, title.
    List {
        #[command(flatten)]
        list: CommonListArgs,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Show one policy: its metadata, relationships, and prose body.
    Show {
        /// Policy reference — `POL-007` or the bare id `7`.
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

    /// Set a policy's status (edit-preserving; a no-op if unchanged).
    Status {
        /// Policy id (numeric).
        id: u32,

        /// New status (required): draft|required|deprecated|retired.
        #[arg(long)]
        status: policy::PolicyStatus,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum StandardCommand {
    /// Allocate the next id and scaffold a new standard.
    New {
        /// Standard title (prompted for if omitted).
        title: Option<String>,

        /// Explicit slug (default: derived from the title).
        #[arg(long)]
        slug: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// List standards by id: STD-id, status, slug, title.
    List {
        #[command(flatten)]
        list: CommonListArgs,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Show one standard: its metadata, relationships, and prose body.
    Show {
        /// Standard reference — `STD-007` or the bare id `7`.
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

    /// Set a standard's status (edit-preserving; a no-op if unchanged).
    Status {
        /// Standard id (numeric).
        id: u32,

        /// New status (required): draft|default|required|deprecated|retired.
        #[arg(long)]
        status: standard::StandardStatus,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum RfcCommand {
    /// Allocate the next id and scaffold a new RFC.
    New {
        /// RFC title (prompted for if omitted).
        title: Option<String>,

        /// Explicit slug (default: derived from the title).
        #[arg(long)]
        slug: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// List RFCs by id: RFC-id, status, slug, title.
    List {
        #[command(flatten)]
        list: CommonListArgs,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Show one RFC: its metadata, relationships, and prose body.
    Show {
        /// RFC reference — `RFC-007` or the bare id `7`.
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

    /// Set an RFC's status (edit-preserving; a no-op if unchanged).
    Status {
        /// RFC id (numeric).
        id: u32,

        /// New status (required): open|resolved|withdrawn.
        #[arg(long)]
        status: rfc::RfcStatus,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum SpecCommand {
    /// Allocate the next id in the subtype's namespace and scaffold a new spec.
    New {
        /// Spec subtype: product | tech.
        subtype: spec::SpecSubtype,

        /// Spec title (prompted for if omitted).
        title: Option<String>,

        /// Explicit slug (default: derived from the title).
        #[arg(long)]
        slug: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// List specs per subtype: id, status, slug, #members.
    List {
        #[command(flatten)]
        list: CommonListArgs,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Reassemble a spec into its readable whole and print it to stdout.
    Show {
        /// Canonical spec ref: `PRD-NNN` (product) or `SPEC-NNN` (tech).
        spec_ref: String,

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

    /// Check FK integrity across the corpus (or one spec): dangling member /
    /// interaction FKs, duplicate labels, and (corpus-wide) orphan requirements.
    Validate {
        /// Canonical spec ref to scope the check to (`PRD-NNN` / `SPEC-NNN`);
        /// omitted → the whole corpus (the only mode that checks for orphans).
        spec_ref: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Operate on a spec's requirements (membership).
    Req {
        #[command(subcommand)]
        command: SpecReqCommand,
    },
}

#[derive(Subcommand)]
enum SpecReqCommand {
    /// Reserve a requirement and append it to a spec as a labelled member.
    Add {
        /// Canonical spec ref: `PRD-NNN` (product) or `SPEC-NNN` (tech).
        spec_ref: String,

        /// Requirement title (prompted for if omitted).
        title: Option<String>,

        /// Requirement kind: functional | quality.
        #[arg(long)]
        kind: requirement::ReqKind,

        /// Explicit membership label (default: next free FR-/NF- for the kind).
        #[arg(long)]
        label: Option<String>,

        /// Explicit slug (default: derived from the title, bounded to a safe length).
        #[arg(long)]
        slug: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Transition a requirement's authored status (free any→any, edit-preserving).
    Status {
        /// Canonical requirement ref: `REQ-NNN` (by id only — no slug derivation).
        req_ref: String,

        /// Target status: pending | in-progress | active | deprecated | retired |
        /// superseded.
        #[arg(long)]
        to: requirement::ReqStatus,

        /// Operator note (accepted for v1; not yet stored on the requirement).
        #[arg(long)]
        note: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// List a spec's requirement members — authored roster (id, label, kind,
    /// status).
    List {
        /// Canonical spec ref: PRD-NNN | SPEC-NNN.
        spec_ref: String,

        #[command(flatten)]
        list: CommonListArgs,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum BacklogCommand {
    /// Allocate the next id in the kind's namespace and scaffold a new item.
    New {
        /// Item kind: issue | improvement | chore | risk | idea.
        kind: backlog::ItemKind,

        /// Item title (prompted for if omitted).
        title: Option<String>,

        /// Explicit slug (default: derived from the title).
        #[arg(long)]
        slug: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Survey items across all kinds; filters AND together. Hides terminal
    /// (resolved/closed) by default — `--all` or an explicit `--status` reveals.
    List {
        /// Only this kind.
        #[arg(long)]
        kind: Option<backlog::ItemKind>,

        /// Row order: `sequence` (the composed `needs`/`after` work order, default) or
        /// `id` (the classic kind-then-id grouping).
        #[arg(long = "by", value_enum, default_value_t = backlog::OrderBy::Sequence)]
        by: backlog::OrderBy,

        #[command(flatten)]
        list: CommonListArgs,

        /// Title substring filter (DEPRECATED alias of `--filter`; `--filter` wins).
        substr: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Reassemble one item by id (`ISS-007`) — kind auto-detected from the prefix.
    Show {
        /// Canonical item ref (e.g. ISS-007); the prefix selects the kind.
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

    /// Transition one item's status (and resolution) in place — kind auto-detected
    /// from the prefix. Coupling holds: a terminal status requires a resolution, a
    /// non-terminal forbids one (re-opening auto-clears it).
    Edit {
        /// Canonical item ref (e.g. ISS-007); the prefix selects the kind.
        id: String,

        /// The target status (open | triaged | started | resolved | closed).
        #[arg(long)]
        status: backlog::Status,

        /// The resolution (required by a terminal status, forbidden otherwise).
        #[arg(long)]
        resolution: Option<backlog::Resolution>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Append hard prerequisites to an item's `needs` axis — kind auto-detected from
    /// the prefix. Validates every ref exists, then refuses a closing dependency
    /// cycle (naming the members; nothing written).
    Needs {
        /// The dependent item ref (e.g. ISS-007); the prefix selects the kind.
        id: String,

        /// One or more prerequisite refs the item must wait on.
        #[arg(required = true)]
        prereqs: Vec<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Append one soft-sequence edge to an item's `after` axis — kind auto-detected
    /// from the prefix. Validates the target exists; never rejects a cycle (a soft
    /// preference, surfaced and evicted at `order` time).
    After {
        /// The item ref that should run after the target (e.g. ISS-007).
        id: String,

        /// The predecessor ref this item should follow.
        /// Required unless --prune is set.
        #[arg(required_unless_present = "prune")]
        to: Option<String>,

        /// Per-edge rank (a manual tie-break hint; default 0).
        #[arg(long, default_value_t = 0)]
        rank: i32,

        /// Remove matching after edges instead of appending.
        #[arg(long, conflicts_with = "prune")]
        remove: bool,

        /// Drop every dangling after edge from the source item.
        #[arg(long, conflicts_with = "remove")]
        prune: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Add and/or remove tags on an item — kind auto-detected from the prefix. Tags
    /// are lowercased and validated `[a-z0-9_:-]` (colon namespacing, e.g.
    /// `area:backlog`); the stored set is sorted. At least one add or remove required.
    Tag {
        /// Canonical item ref (e.g. ISS-007); the prefix selects the kind.
        id: String,

        /// Tags to add (positional, repeatable).
        tags: Vec<String>,

        /// Tags to remove, repeatable (`-d security -d area:backlog`).
        #[arg(long = "remove", short = 'd')]
        remove: Vec<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum KnowledgeCommand {
    /// Allocate the next id in the kind's namespace and scaffold a new record
    /// (seeded default status, empty `[facet]`, empty `[evidence]`).
    New {
        /// Record kind: assumption | decision | question | constraint.
        kind: knowledge::RecordKind,

        /// Record title (prompted for if omitted).
        title: Option<String>,

        /// Explicit slug (default: derived from the title).
        #[arg(long)]
        slug: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Survey records across all four kinds; filters AND together. Hides settled
    /// states by default — `--all` or an explicit `--status` reveals.
    List {
        #[command(flatten)]
        list: CommonListArgs,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Reassemble one record by id (`ASM-007`) — kind auto-detected from the prefix.
    Show {
        /// Canonical record ref (e.g. ASM-007); the prefix selects the kind.
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

    /// Transition one record's status in place — kind auto-detected from the prefix.
    /// The `<state>` must be in the kind's vocabulary (a foreign-kind state is refused).
    Status {
        /// Canonical record ref (e.g. ASM-007); the prefix selects the kind.
        id: String,

        /// The target status (in the kind's vocabulary).
        state: String,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

/// The `coverage` subcommand group (SL-057 D4): the read-only `show` view plus
/// the observed-tier write path (`record`/`verify`/`forget`). `show` carries the
/// former bare-`coverage <ref>` surface verbatim (behaviour preserved).
#[derive(Subcommand)]
enum CoverageCommand {
    /// Read-only requirement coverage / drift view. `<reference>` is REQ-NNN (one
    /// row) or PRD-/SPEC-NNN (a member fan). Derived observed coverage + the drift
    /// verdict against authored status — never writes, never derives status.
    Show {
        /// Canonical ref: REQ-NNN | PRD-NNN | SPEC-NNN.
        reference: String,

        /// Select/order visible table columns (e.g. `--columns id,status,verdict`).
        #[arg(long, value_delimiter = ',')]
        columns: Option<Vec<String>>,

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

    /// Record one observed coverage cell into a slice's `coverage.toml`. With any
    /// check field (`--alias`/`--command`/`--matcher-*`/`--extra-args`/`--regex`)
    /// the cell is a `VT` recipe (leans Planned until verified); with none it is a
    /// `VA`/`VH` attestation stamped with today (or `--attested-date`).
    Record {
        /// Slice the cell is recorded under — `SL-NNN` or the bare number.
        #[arg(long)]
        slice: String,

        /// The requirement this evidence covers — `REQ-NNN`.
        #[arg(long)]
        requirement: String,

        /// The contributing change — `SL-NNN` (often the same as `--slice`).
        #[arg(long)]
        change: String,

        /// Verification mode: `VT` | `VA` | `VH`.
        #[arg(long)]
        mode: String,

        /// Observed status for a `VA`/`VH` attestation (default: verified). Ignored
        /// for a `VT` record (the verifier derives it; it leans Planned at record).
        #[arg(long, value_parser = coverage_store::parse_status)]
        status: Option<requirement::CoverageStatus>,

        /// VT-check alias into `[verification.aliases]` (XOR `--command`).
        #[arg(long)]
        alias: Option<String>,

        /// VT-check literal command argv, repeatable (XOR `--alias`).
        #[arg(long = "command")]
        command: Vec<String>,

        /// Extra args appended to the resolved base argv, repeatable.
        #[arg(long = "extra-args")]
        extra_args: Vec<String>,

        /// Matcher source: `stdout` | `stderr` | `file:<glob>`.
        #[arg(long = "matcher-source")]
        matcher_source: Option<String>,

        /// Matcher pattern (substring, or regex with `--regex`).
        #[arg(long = "matcher-pattern")]
        matcher_pattern: Option<String>,

        /// Treat `--matcher-pattern` as a `regex_lite` pattern.
        #[arg(long)]
        regex: bool,

        /// Attestation date override (`YYYY-MM-DD`) for `VA`/`VH` (default: today).
        #[arg(long = "attested-date")]
        attested_date: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Re-derive `VT` coverage status by re-running each entry's check. A single
    /// `<slice>` re-derives that slice; `--all` re-derives every slice (the
    /// global-dedup set — a shared check runs once across the invocation).
    Verify {
        /// The slice to verify — `SL-NNN` or the bare number (omit with `--all`).
        slice: Option<String>,

        /// Verify every slice in the corpus.
        #[arg(long)]
        all: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Erase one coverage cell (the 4-tuple key) from a slice's store. Prints the
    /// withdrawn cell — a deletion that flips a composite green is never silent.
    Forget {
        /// Slice the cell lives under — `SL-NNN` or the bare number.
        #[arg(long)]
        slice: String,

        /// The requirement the cell covers — `REQ-NNN`.
        #[arg(long)]
        requirement: String,

        /// The contributing change — `SL-NNN`.
        #[arg(long)]
        change: String,

        /// Verification mode: `VT` | `VA` | `VH`.
        #[arg(long)]
        mode: String,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum MemoryCommand {
    /// Mint a uid and scaffold a new memory under `.doctrine/memory/items`.
    /// `memory new` is the uniform canonical alias (SL-025 §5.4 / D8); both names
    /// dispatch the identical handler — skills may migrate `record → new` at leisure.
    #[command(visible_alias = "new")]
    Record {
        /// Memory title.
        title: String,

        /// Memory type (required): concept|fact|pattern|signpost|system|thread.
        #[arg(long = "type", value_parser = memory::MemoryType::parse)]
        memory_type: memory::MemoryType,

        /// Key alias `mem.<type>.<domain>.<subject>` (shorthand normalized).
        #[arg(long)]
        key: Option<String>,

        /// Lifespan classification.
        #[arg(long, value_parser = memory::Lifespan::from_str)]
        lifespan: Option<memory::Lifespan>,

        /// Lifecycle status (default: active).
        #[arg(long, default_value = "active", value_parser = memory::Status::parse)]
        status: memory::Status,

        /// One-line summary.
        #[arg(long)]
        summary: Option<String>,

        /// Review-by date carried in `[review].review_by`.
        #[arg(long)]
        review_by: Option<String>,

        /// Provenance source, repeatable, in `KIND:REF` form.
        #[arg(long = "provenance-source", value_parser = memory::Provenance::parse_flag)]
        provenance_source: Vec<memory::Provenance>,

        /// Trust level carried in `[trust].trust_level`.
        #[arg(long = "trust")]
        trust: Option<String>,

        /// Severity carried in `[ranking].severity`.
        #[arg(long = "severity")]
        severity: Option<String>,

        /// Tag, repeatable — written to `scope.tags`.
        #[arg(long = "tag")]
        tag: Vec<String>,

        /// Path scope, repeatable — written to `scope.paths`.
        #[arg(long = "path-scope")]
        path_scope: Vec<String>,

        /// Glob scope, repeatable — written to `scope.globs`.
        #[arg(long = "glob")]
        glob: Vec<String>,

        /// Command scope, repeatable — written to `scope.commands`.
        #[arg(long = "command")]
        command: Vec<String>,

        /// Repo identity override (`--repo`), e.g. `github.com/org/repo` — kind
        /// `explicit`, confidence `high`; userinfo is stripped.
        #[arg(long = "repo")]
        repo: Option<String>,

        /// Mint a GLOBAL orientation master: suppress the git born frame
        /// (`repo=""`, anchor `none`) and write into the repo-root `memory/` tree
        /// instead of `items/` (SL-018 — the corpus authoring path).
        #[arg(long = "global")]
        global: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Resolve a memory by uid or key and print its header + body-as-data.
    Show {
        /// Memory reference: a `mem_<hex>` uid or a `mem.<…>` key.
        reference: String,

        /// Output format. `--json` is shorthand; see `--format`.
        #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
        format: Format,

        /// Shorthand for `--format json`.
        #[arg(long)]
        json: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Attest a memory against the current working tree: stamp its verification
    /// axis (refuses a dirty tree — no false attestation).
    Verify {
        /// Memory reference: a `mem_<hex>` uid or a `mem.<…>` key.
        reference: String,

        /// Allow verification on dirty tree (stamps `checkout_state_id`).
        #[arg(long)]
        allow_dirty: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Run advisory validation checks on memories (dangling relations, stale verification, draft expiry).
    Validate {
        /// Optional memory reference: a `mem_<hex>` uid or a `mem.<…>` key.
        reference: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// List recorded memories, newest first; AND-filter on the shared spine.
    List {
        /// Filter by type: concept|fact|pattern|signpost|system|thread. The one
        /// kind-specific axis (beside the shared flags — backlog `--kind` precedent).
        #[arg(long = "type", value_parser = memory::MemoryType::parse)]
        memory_type: Option<memory::MemoryType>,

        /// Shared list flags: -f/-r/-i/-s/-t/-a/--format/--json (SL-025).
        #[command(flatten)]
        list: CommonListArgs,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Find memories by scope/query, ranked; rows carry trust + severity so the
    /// holdback-exempt find surface keeps risk visible.
    Find {
        /// Positional query (zero or one; maps to --query). Mutually exclusive with --query.
        query: Option<String>,

        #[command(flatten)]
        args: FindRetrieveArgs,
    },

    /// Retrieve memories as bounded, security-framed `data, not instruction`
    /// blocks for agent context. Applies the trust holdback (non-bypassable):
    /// low-trust high-severity memories are suppressed; use `find`/`show` to
    /// inspect them.
    Retrieve {
        #[command(flatten)]
        args: FindRetrieveArgs,

        /// Raise the trust floor: only show memories at this trust or higher under
        /// high severity (high|medium|low; only raises the default `medium`).
        #[arg(long = "min-trust", value_parser = retrieve::parse_min_trust)]
        min_trust: Option<String>,
    },

    /// Resolve memory wikilinks for one memory or the whole corpus.
    ResolveLinks {
        /// Optional memory reference: a `mem_<hex>` uid or a `mem.<…>` key.
        reference: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Show reverse links into one target from wikilinks and authored relations.
    Backlinks {
        /// Target reference: a `mem_<hex>` uid, a `mem.<…>` key, or another target token.
        reference: String,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Add and/or remove tags on a memory — tags are lowercased and validated
    /// `[a-z0-9_:-]` (colon namespacing, e.g. `area:memory`); the stored set is
    /// sorted. At least one add or remove required.
    Tag {
        /// Memory reference: a `mem_<hex>` uid or a `mem.<…>` key.
        reference: String,

        /// Tags to add (positional, repeatable).
        tags: Vec<String>,

        /// Tags to remove, repeatable (`-d security -d area:memory`).
        #[arg(long = "remove", short = 'd')]
        remove: Vec<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Transition one memory's status. `<state>` must be one of the 6 lifecycle
    /// states (active/draft/superseded/retracted/archived/quarantined).
    /// `--by <OTHER>` is required for superseded (records the successor relation)
    /// and forbidden otherwise.
    Status {
        /// Memory reference: a `mem_<hex>` uid or a `mem.<…>` key.
        reference: String,

        /// The target status: active|draft|superseded|retracted|archived|quarantined.
        state: String,

        /// Successor reference (required for superseded, forbidden otherwise).
        #[arg(long)]
        by: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Edit a memory's fields in a single read→mutate→write transaction.
    /// At least one flag required. `--status` delegates to the status-transition
    /// core (superseded refused — use `memory status superseded --by`).
    /// `--key` late-binds only on an unkeyed memory (immutable once recorded).
    /// Scope arrays replace. `updated` stamped once on any change.
    Edit {
        /// Memory reference: a `mem_<hex>` uid or a `mem.<…>` key.
        reference: String,

        /// Replace the title (non-empty after trim).
        #[arg(long)]
        title: Option<String>,

        /// Replace the summary (free text).
        #[arg(long)]
        summary: Option<String>,

        /// Transition status (active|draft|retracted|archived|quarantined).
        /// Superseded is refused — use `memory status superseded --by <OTHER>`.
        #[arg(long)]
        status: Option<String>,

        /// Replace the lifespan (semantic|episodic|procedural|working|identity).
        /// An empty value leaves the existing lifespan unchanged.
        #[arg(long)]
        lifespan: Option<String>,

        /// Set or replace the review-by date (`YYYY-MM-DD`); empty string clears.
        #[arg(long)]
        review_by: Option<String>,

        /// Set the trust level (low|medium|high).
        #[arg(long)]
        trust: Option<String>,

        /// Set the severity (critical|high|medium|low|none).
        #[arg(long)]
        severity: Option<String>,

        /// Late-bind the memory key (shorthand normalized via `mem.` prefix).
        /// Refused if the memory already has a key set.
        #[arg(long)]
        key: Option<String>,

        /// Replace the scope.paths array (repeatable).
        #[arg(long = "path-scope")]
        path_scope: Vec<String>,

        /// Replace the scope.globs array (repeatable).
        #[arg(long = "glob")]
        glob: Vec<String>,

        /// Replace the scope.commands array (repeatable).
        #[arg(long = "command")]
        command: Vec<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Materialize the embedded global-memory corpus into the gitignored
    /// `.doctrine/memory/shipped/`, or `memory sync install` to wire the
    /// session hook. Outside a doctrine repo this is a clean no-op.
    Sync {
        /// Wire the `SessionStart` refresh hook (omit to run the sync).
        #[command(subcommand)]
        command: Option<SyncCommand>,

        /// Compute and print the plan without writing anything.
        #[arg(long)]
        dry_run: bool,

        /// Skip the confirmation prompt.
        #[arg(short = 'y', long)]
        yes: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum SyncCommand {
    /// Wire a separate `SessionStart` hook running `doctrine memory sync` (mirrors
    /// `boot install`; the hook degrades to a clean no-op in non-doctrine repos).
    Install {
        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,

        /// Compute and report the plan without writing anything.
        #[arg(long)]
        dry_run: bool,

        /// Skip the confirmation prompt.
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum SliceCommand {
    /// Allocate the next id and scaffold a new slice.
    New {
        /// Slice title (prompted for if omitted).
        title: Option<String>,

        /// Explicit slug (default: derived from the title).
        #[arg(long)]
        slug: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Scaffold a design-doc sibling into an existing slice.
    Design {
        /// Slice id to attach the design doc to.
        id: u32,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Scaffold an implementation plan (plan.toml + plan.md) into a slice.
    Plan {
        /// Slice id to attach the plan to.
        id: u32,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Materialise phase tracking from a slice's plan into the state tree.
    Phases {
        /// Slice id whose plan declares the phases.
        id: u32,

        /// Remove orphan tracking whose plan phase is gone (destructive).
        #[arg(long)]
        prune: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Scaffold a durable notes.md scratchpad into a slice (on-demand).
    Notes {
        /// Slice id to attach the notes file to.
        id: u32,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Record a phase status transition into its runtime tracking.
    Phase {
        /// Slice id owning the phase.
        id: u32,

        /// Canonical phase id, e.g. PHASE-01.
        phase_id: String,

        /// New status.
        #[arg(long)]
        status: state::PhaseStatus,

        /// Optional note appended to the progress log.
        #[arg(long)]
        note: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Classify and write a slice lifecycle transition; prints the move's
    /// classification (advance / back-edge / skip / abandon). Refuses the closure
    /// seam (→ reconcile only from audit, → done only from reconcile) and leaving
    /// a terminal status (done / abandoned).
    Status {
        /// Slice id to transition.
        id: u32,

        /// Target lifecycle state.
        state: slice::SliceStatus,

        /// Optional note — surfaced in the transition output, not stored.
        #[arg(long)]
        note: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// List slices by id: id, status, phases, slug, title.
    List {
        #[command(flatten)]
        list: CommonListArgs,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Show one slice: its metadata and scope body (not design/plan/notes).
    Show {
        /// Slice reference — `SL-025` or the bare id `25`.
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
}

#[derive(Subcommand)]
enum ConceptMapCommand {
    /// Create a new concept map.
    New {
        /// Concept-map title (prompted for if omitted).
        title: Option<String>,

        /// Explicit slug (default: derived from the title).
        #[arg(long)]
        slug: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// List concept maps.
    List {
        #[command(flatten)]
        list: CommonListArgs,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Show a concept map's metadata and DSL.
    Show {
        /// Concept-map reference — `CM-001` or the bare id `1`.
        reference: String,

        /// Output format.
        #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
        format: Format,

        /// Show edges table from parsed DSL.
        #[arg(long)]
        edges: bool,

        /// Show nodes table from parsed DSL.
        #[arg(long)]
        nodes: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Parse the DSL and run heuristic checks.
    Check {
        /// Concept-map reference — `CM-001` or the bare id `1`.
        id: String,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Add an edge to a concept map's DSL.
    Add {
        id: String,
        source: String,
        rel: String,
        target: String,
        #[arg(long)]
        force: bool,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Remove an edge from a concept map's DSL.
    Remove {
        id: String,
        source: String,
        rel: String,
        target: String,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Rename a node label across all DSL edges.
    RenameNode {
        id: String,
        old: String,
        new: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        case_sensitive: bool,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Export a concept map to DOT, Mermaid, or JSON.
    Export {
        id: String,
        #[arg(long, value_enum)]
        format: concept_map::ExportFormat,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum ReviewCommand {
    /// Open a new review ledger targeting an entity via the `reviews` edge.
    /// The `--target` ref is validated up front — a dangling ref is refused
    /// before any id is allocated. Findings are added later with `review raise`.
    New {
        /// What this review reviews (the facet): scope | design | plan |
        /// phase-plan | implementation | code-review | reconciliation.
        #[arg(long, value_parser = review::Facet::parse)]
        facet: review::Facet,

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
        list: CommonListArgs,

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
        #[arg(long, value_parser = review::Severity::parse)]
        severity: review::Severity,

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
}

#[derive(Subcommand)]
enum RecCommand {
    /// Open a new reconciliation record — the immutable ledger of one
    /// reconciliation act (the REC kind, SPEC-002). A fresh REC is a skeleton
    /// (empty deltas/evidence); the reconcile writer (Slice B) populates it.
    New {
        /// The reconciliation move: accept | revise | redesign.
        #[arg(long = "move", value_parser = rec::RecMove::parse)]
        r#move: rec::RecMove,

        /// Optional owning slice, e.g. `SL-042` (a freestanding REC omits it).
        #[arg(long)]
        owning_slice: Option<String>,

        /// Optional decision ref this act records against, e.g. `DEC-005-C`.
        #[arg(long = "decision")]
        decision_ref: Option<String>,

        /// REC title (default: derived from the move).
        #[arg(long)]
        title: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// List reconciliation records by id: id, move, owning slice, title.
    List {
        #[command(flatten)]
        list: CommonListArgs,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Show one reconciliation record: move, edges, deltas/evidence, rationale.
    Show {
        /// REC reference — `REC-007` or the bare id `7`.
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
}

#[derive(Subcommand)]
enum RevisionCommand {
    /// Open a new revision — a pending revise-intent against authored truth (the
    /// REV change-axis kind, ADR-013). A fresh REV is a skeleton (`proposed`,
    /// `approval=none`, no change rows); `revision change add` (PHASE-03) populates
    /// the typed `[[change]]` payload.
    New {
        /// REV title.
        title: Option<String>,

        /// Explicit slug (default: derived from the title).
        #[arg(long)]
        slug: Option<String>,

        /// Provenance RFC reference (e.g. `RFC-007`). Authors ONE `originates_from`
        /// relation row — a typed provenance edge, NOT a `[[change]]` payload.
        #[arg(long)]
        originates_from: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Show one revision: status, approval, change rows, rationale.
    Show {
        /// REV reference — `REV-007` or the bare id `7`.
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

    /// Transition a revision's lifecycle: `revision status <REV-N> <state>`
    /// (proposed→started→done; abandoned from any non-terminal). Approval-blind.
    Status {
        /// REV reference — `REV-007` or the bare id `7`.
        reference: String,

        /// Target lifecycle state.
        state: revision::RevStatus,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Author the typed `[[change]]` payload — the `revises` rows (the touched-entity
    /// set). The ONLY writer of a `revises` edge; `doctrine link … revises …` is
    /// refused (`TypedVerbOnly`).
    Change {
        #[command(subcommand)]
        command: RevisionChangeCommand,
    },

    /// Record an explicit approval (`approval = approved`) on the orthogonal approval
    /// axis — the enabling act for the apply checkpoint. `revision apply` refuses unless
    /// approved (invoker-blind: a solo dev self-approves; ADR-009).
    Approve {
        /// REV reference — `REV-007` or the bare id `7`.
        reference: String,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Apply an approved revision: auto-land its `status` rows (each via the requirement
    /// status setter + one REC), surface introduce/create/modify/move/prose rows for
    /// manual handling. Refused unless `approval = approved`. A pre-flight all-or-nothing
    /// from-guard aborts the whole apply if any target moved since the change was drafted.
    Apply {
        /// REV reference — `REV-007` or the bare id `7`.
        reference: String,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum RevisionChangeCommand {
    /// Append one `[[change]]` row to a revision (design §4.4). Shape-routed by
    /// `--action`: a creation op (`introduce`/`create`) takes `--new-label` (required,
    /// frozen) + `--member-of` (a live SPEC); an existing-target op
    /// (`modify`/`retire`/`move`/`status`) takes `--target` (a live peer FK), with
    /// `--to-status` and an auto-captured `from` for a `status` row.
    Add {
        /// REV reference — `REV-007` or the bare id `7`.
        reference: String,

        /// The change action.
        #[arg(long)]
        action: revision::ChangeAction,

        /// Existing-target ops: the live peer FK (`REQ-201`, `ADR-006`).
        #[arg(long)]
        target: Option<String>,

        /// `status` rows: the requested target status.
        #[arg(long = "to-status")]
        to_status: Option<String>,

        /// Creation ops: the frozen membership label (required for `introduce`/`create`).
        #[arg(long = "new-label")]
        new_label: Option<String>,

        /// Creation ops: the destination spec (a live `SPEC-NNN`).
        #[arg(long = "member-of")]
        member_of: Option<String>,

        /// `introduce`: the new requirement's statement line (optional).
        #[arg(long = "new-statement")]
        new_statement: Option<String>,

        /// Mark this row the revision's headline subject (display-only; at most one).
        #[arg(long)]
        primary: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum MapCommand {
    /// Start the local map explorer web server (loopback only)
    Serve(MapServeArgs),
}

#[derive(Subcommand)]
enum ExportCommand {
    /// Emit the corpus as a single lazyspec Brief (JSON) on stdout (SL-026).
    Lazyspec {
        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum SkillsCommand {
    /// List available skills and their install status.
    List {
        /// Agent to report status for (default: claude).
        #[arg(short = 'a', long)]
        agent: Option<String>,

        /// Only show skills already installed.
        #[arg(long)]
        installed: bool,
    },
}

// ---------------------------------------------------------------------------
// `doctrine catalog scan --json` / `doctrine catalog graph --json` (SL-071 PHASE-06)
// ---------------------------------------------------------------------------

/// Thin JSON dump of the hydrated `Catalog` — entities, edges, and diagnostics.
/// Developer scaffolding; not gating for acceptance (D12).
fn run_catalog_scan(root_arg: Option<PathBuf>) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(root_arg, &crate::root::default_markers())?;
    if !root.join(".doctrine").is_dir() {
        anyhow::bail!("no .doctrine directory found at '{}'", root.display());
    }
    let catalog = crate::catalog::hydrate::scan_catalog(&root)?;
    let json = serde_json::to_string_pretty(&catalog)
        .map_err(|e| anyhow::anyhow!("failed to serialize catalog: {e}"))?;
    write!(std::io::stdout(), "{json}")?;
    Ok(())
}

/// Thin JSON dump of the `CatalogGraph` — nodes and edges.
/// Developer scaffolding; not gating for acceptance (D12).
fn run_catalog_graph(root_arg: Option<PathBuf>) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(root_arg, &crate::root::default_markers())?;
    if !root.join(".doctrine").is_dir() {
        anyhow::bail!("no .doctrine directory found at '{}'", root.display());
    }
    let catalog = crate::catalog::hydrate::scan_catalog(&root)?;
    let graph = crate::catalog::graph::CatalogGraph::from_catalog(&catalog);
    let json = serde_json::to_string_pretty(&graph)
        .map_err(|e| anyhow::anyhow!("failed to serialize graph: {e}"))?;
    write!(std::io::stdout(), "{json}")?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let color = crate::tty::resolve_color(cli.color);

    // ADR-006 D2a / SL-056 §3 worker-mode guard: a dispatched worker mints/anchors
    // nothing. Bail before dispatch on any Write-classed verb; Read / MarkerClear
    // paths stay open (INV-3 / the self-brick carve-out).
    crate::commands::guard::worker_guard(&cli.command)?;

    match cli.command {
        Command::Install {
            path,
            agent,
            skill,
            domain,
            only_memory,
            global,
            dry_run,
            yes,
        } => install::run(
            path,
            &install::InstallArgs {
                agents: &agent,
                skills: &skill,
                domains: &domain,
                only_memory,
                global,
                dry_run,
                yes,
            },
        ),
        Command::Skills { command } => match command {
            SkillsCommand::List { agent, installed } => {
                skills::run_list(agent.as_deref(), installed)
            }
        },
        Command::ConceptMap { command } => match command {
            ConceptMapCommand::New { title, slug, path } => concept_map::run_new(path, title, slug),
            ConceptMapCommand::List { list, path } => {
                concept_map::run_list(path, list.into_list_args(color))
            }
            ConceptMapCommand::Show {
                reference,
                format,
                edges,
                nodes,
                path,
            } => concept_map::run_show(path, &reference, format, edges, nodes),
            ConceptMapCommand::Check { id, path } => concept_map::run_check(path, &id),
            ConceptMapCommand::Add {
                id,
                source,
                rel,
                target,
                force,
                path,
            } => concept_map::run_add(path, &id, &source, &rel, &target, force),
            ConceptMapCommand::Remove {
                id,
                source,
                rel,
                target,
                path,
            } => concept_map::run_remove(path, &id, &source, &rel, &target),
            ConceptMapCommand::RenameNode {
                id,
                old,
                new,
                dry_run,
                case_sensitive,
                path,
            } => concept_map::run_rename_node(path, &id, &old, &new, dry_run, case_sensitive),
            ConceptMapCommand::Export { id, format, path } => {
                concept_map::run_export(path, &id, &format)
            }
        },
        Command::Slice { command } => match command {
            SliceCommand::New { title, slug, path } => slice::run_new(path, title, slug),
            SliceCommand::Design { id, path } => slice::run_design(path, id),
            SliceCommand::Plan { id, path } => slice::run_plan(path, id),
            SliceCommand::Phases { id, prune, path } => slice::run_phases(path, id, prune),
            SliceCommand::Notes { id, path } => slice::run_notes(path, id),
            SliceCommand::Phase {
                id,
                phase_id,
                status,
                note,
                path,
            } => slice::run_phase(path, id, &phase_id, status, note.as_deref()),
            SliceCommand::Status {
                id,
                state,
                note,
                path,
            } => slice::run_status(path, id, state, note.as_deref()),
            SliceCommand::List { list, path } => slice::run_list(path, list.into_list_args(color)),
            SliceCommand::Show {
                reference,
                format,
                json,
                path,
            } => slice::run_show(path, &reference, if json { Format::Json } else { format }),
        },
        Command::Memory { command } => match command {
            MemoryCommand::Record {
                title,
                memory_type,
                key,
                lifespan,
                status,
                summary,
                review_by,
                provenance_source,
                trust,
                severity,
                tag,
                path_scope,
                glob,
                command,
                repo,
                global,
                path,
            } => memory::run_record(
                path,
                &memory::RecordArgs {
                    title: &title,
                    memory_type,
                    key: key.as_deref(),
                    lifespan,
                    status,
                    summary: summary.as_deref(),
                    review_by: review_by.as_deref(),
                    sources: &provenance_source,
                    trust_level: trust.as_deref(),
                    severity: severity.as_deref(),
                    tags: &tag,
                    paths: &path_scope,
                    globs: &glob,
                    commands: &command,
                    repo: repo.as_deref(),
                    global,
                },
            ),
            MemoryCommand::Show {
                reference,
                format,
                json,
                path,
            } => memory::run_show(
                &mut io::stdout(),
                path,
                &reference,
                if json { Format::Json } else { format },
            ),
            MemoryCommand::Verify {
                reference,
                allow_dirty,
                path,
            } => memory::run_verify(path, &reference, allow_dirty),
            MemoryCommand::Validate { reference, path } => {
                match memory::run_validate(path, reference.as_deref()) {
                    Ok(()) => Ok(()),
                    Err(e) if e.to_string().contains("validation warnings found") => {
                        // Exit with code 1 for validation warnings - this is the expected CLI behavior
                        #[expect(
                            clippy::disallowed_methods,
                            reason = "CLI tool needs to exit with code 1 for validation warnings"
                        )]
                        {
                            std::process::exit(1);
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            MemoryCommand::List {
                memory_type,
                list,
                path,
            } => memory::run_list(
                &mut io::stdout(),
                path,
                memory_type,
                list.into_list_args(color),
            ),
            MemoryCommand::Find { query, args } => {
                // Merge positional query + --query; mutually exclusive.
                let free_query = match (query, args.flag_query) {
                    (Some(_), Some(_)) => {
                        anyhow::bail!("cannot specify both a positional query and --query")
                    }
                    (q, None) | (None, q) => q,
                };
                // Resolve offset: page sugar or explicit.
                let page_size = args.limit.unwrap_or(retrieve::RETRIEVE_LIMIT_DEFAULT);
                let offset = match args.page {
                    Some(0) => anyhow::bail!("--page must be >= 1"),
                    Some(p) => (p - 1) * page_size,
                    None => args.offset,
                };
                let resolved_format = if args.json { Format::Json } else { args.format };
                retrieve::run_find(
                    &mut io::stdout(),
                    args.path,
                    args.path_scope,
                    args.glob,
                    args.command,
                    args.tag,
                    args.lifespan,
                    free_query,
                    args.memory_type,
                    args.status,
                    args.include_draft,
                    resolved_format,
                    offset,
                    args.limit,
                )
            }
            MemoryCommand::Retrieve { args, min_trust } => {
                // Resolve offset: page sugar or explicit.
                let page_size = args.limit.unwrap_or(retrieve::RETRIEVE_LIMIT_DEFAULT);
                let offset = match args.page {
                    Some(0) => anyhow::bail!("--page must be >= 1"),
                    Some(p) => (p - 1) * page_size,
                    None => args.offset,
                };
                let resolved_format = if args.json { Format::Json } else { args.format };
                // limit is passed as Option<usize>; run_retrieve resolves through
                // default/max internally.
                let retrieve_limit = args.limit.unwrap_or(retrieve::RETRIEVE_LIMIT_DEFAULT);
                retrieve::run_retrieve(
                    &mut io::stdout(),
                    args.path,
                    args.path_scope,
                    args.glob,
                    args.command,
                    args.tag,
                    args.lifespan,
                    args.flag_query,
                    args.memory_type,
                    args.status,
                    args.include_draft,
                    retrieve_limit,
                    min_trust.as_deref(),
                    offset,
                    resolved_format,
                    args.expand,
                )
            }
            MemoryCommand::ResolveLinks { reference, path } => {
                memory::run_resolve_links(path, reference.as_deref())
            }
            MemoryCommand::Backlinks { reference, path } => memory::run_backlinks(path, &reference),
            MemoryCommand::Tag {
                reference,
                tags,
                remove,
                path,
            } => memory::run_tag(path, &reference, &tags, &remove),
            MemoryCommand::Status {
                reference,
                state,
                by,
                path,
            } => memory::run_status(path, &reference, &state, by.as_deref(), color),
            MemoryCommand::Edit {
                reference,
                title,
                summary,
                status,
                lifespan,
                review_by,
                trust,
                severity,
                key,
                path_scope,
                glob,
                command,
                path,
            } => {
                let fields = memory::EditFields {
                    title,
                    summary,
                    status,
                    lifespan,
                    review_by,
                    trust,
                    severity,
                    key,
                    path_scope: if path_scope.is_empty() {
                        None
                    } else {
                        Some(path_scope)
                    },
                    glob: if glob.is_empty() { None } else { Some(glob) },
                    command: if command.is_empty() {
                        None
                    } else {
                        Some(command)
                    },
                };
                memory::run_edit(path, &reference, &fields)
            }
            MemoryCommand::Sync {
                command,
                dry_run: sync_dry_run,
                yes: sync_yes,
                path: sync_path,
            } => match command {
                None => corpus::run_sync(sync_path, sync_dry_run, sync_yes),
                Some(SyncCommand::Install { path, dry_run, yes }) => {
                    corpus::run_sync_install(path, dry_run, yes)
                }
            },
        },
        Command::Review { command } => match command {
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
                let out = review::run_new(
                    path,
                    &review::NewArgs {
                        facet,
                        target,
                        phase,
                        title,
                        raiser,
                        responder,
                    },
                )?;
                let rendered = review::print_review(&out);
                write!(std::io::stdout(), "{rendered}")?;
                Ok(())
            }
            ReviewCommand::List { list, path } => {
                use std::io::Write;
                let out = review::run_list(path, list.into_list_args(color))?;
                let rendered = review::print_review(&out);
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
                let out =
                    review::run_show(path, &reference, if json { Format::Json } else { format })?;
                let rendered = review::print_review(&out);
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
                let role = review::parse_role(role.as_deref(), review::Role::Raiser)?;
                let out = review::run_raise(
                    path,
                    &review::RaiseArgs {
                        reference,
                        severity,
                        title,
                        detail,
                    },
                    role,
                )?;
                let rendered = review::print_review(&out);
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
                let role = review::parse_role(role.as_deref(), review::Role::Responder)?;
                let out = review::run_dispose(
                    path,
                    &review::DisposeArgs {
                        reference,
                        finding,
                        disposition,
                        response,
                    },
                    role,
                )?;
                let rendered = review::print_review(&out);
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
                let role = review::parse_role(role.as_deref(), review::Role::Raiser)?;
                let out = review::run_verify(path, &reference, &finding, note.as_deref(), role)?;
                let rendered = review::print_review(&out);
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
                let role = review::parse_role(role.as_deref(), review::Role::Raiser)?;
                let out = review::run_contest(path, &reference, &finding, note.as_deref(), role)?;
                let rendered = review::print_review(&out);
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
                let role = review::parse_role(role.as_deref(), review::Role::Raiser)?;
                let out = review::run_withdraw(path, &reference, &finding, role)?;
                let rendered = review::print_review(&out);
                write!(std::io::stdout(), "{rendered}")?;
                Ok(())
            }
            ReviewCommand::Status { reference, path } => {
                use std::io::Write;
                let out = review::run_status(path, &reference)?;
                let rendered = review::print_review(&out);
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
                let out = review::run_prime(
                    path,
                    &review::PrimeArgs {
                        reference,
                        seed,
                        from,
                    },
                )?;
                let rendered = review::print_review(&out);
                write!(std::io::stdout(), "{rendered}")?;
                Ok(())
            }
            ReviewCommand::Unlock { reference, path } => {
                use std::io::Write;
                let out = review::run_unlock(path, &reference)?;
                let rendered = review::print_review(&out);
                write!(std::io::stdout(), "{rendered}")?;
                Ok(())
            }
        },
        Command::Rec { command } => match command {
            RecCommand::New {
                r#move,
                owning_slice,
                decision_ref,
                title,
                path,
            } => rec::run_new(
                path,
                &rec::NewArgs {
                    r#move,
                    owning_slice,
                    decision_ref,
                    title,
                },
            ),
            RecCommand::List { list, path } => rec::run_list(path, list.into_list_args(color)),
            RecCommand::Show {
                reference,
                format,
                json,
                path,
            } => rec::run_show(path, &reference, if json { Format::Json } else { format }),
        },
        Command::Revision { command } => match command {
            RevisionCommand::New {
                title,
                slug,
                path,
                originates_from,
            } => revision::run_new(path, title, slug, originates_from.as_deref()),
            RevisionCommand::Show {
                reference,
                format,
                json,
                path,
            } => revision::run_show(path, &reference, if json { Format::Json } else { format }),
            RevisionCommand::Status {
                reference,
                state,
                path,
            } => revision::run_status(path, &reference, state, color),
            RevisionCommand::Change { command } => match command {
                RevisionChangeCommand::Add {
                    reference,
                    action,
                    target,
                    to_status,
                    new_label,
                    member_of,
                    new_statement,
                    primary,
                    path,
                } => revision::run_change_add(
                    path,
                    &reference,
                    &revision::ChangeAddArgs {
                        action,
                        target,
                        to_status,
                        new_label,
                        member_of,
                        new_statement,
                        primary,
                    },
                ),
            },
            RevisionCommand::Approve { reference, path } => revision::run_approve(path, &reference),
            RevisionCommand::Apply { reference, path } => revision::run_apply(path, &reference),
        },
        Command::Reconcile {
            req,
            slice,
            r#move,
            to,
            note,
            path,
        } => reconcile::run(
            path,
            &reconcile::ReconcileArgs {
                req,
                slice,
                r#move,
                to,
                note,
            },
        ),
        Command::Coverage { command } => match command {
            CoverageCommand::Show {
                reference,
                columns,
                format,
                json,
                path,
            } => coverage_view::run(path, &reference, columns.as_deref(), format, json, color),
            CoverageCommand::Record {
                slice,
                requirement,
                change,
                mode,
                status,
                alias,
                command,
                extra_args,
                matcher_source,
                matcher_pattern,
                regex,
                attested_date,
                path,
            } => coverage_store::run_record(
                path,
                &coverage_store::CoverageRecordArgs {
                    slice: &slice,
                    requirement: &requirement,
                    change: &change,
                    mode: &mode,
                    status,
                    alias: alias.as_deref(),
                    command: &command,
                    extra_args: &extra_args,
                    matcher_source: matcher_source.as_deref(),
                    matcher_pattern: matcher_pattern.as_deref(),
                    regex,
                    attested_date: attested_date.as_deref(),
                },
            ),
            CoverageCommand::Verify { slice, all, path } => {
                coverage_verify::run_cli(path, slice.as_deref(), all)
            }
            CoverageCommand::Forget {
                slice,
                requirement,
                change,
                mode,
                path,
            } => coverage_store::run_forget(path, &slice, &requirement, &change, &mode),
        },
        Command::Inspect {
            id,
            format,
            json,
            path,
        } => commands::inspect::run_inspect(path, &id, format, json),
        Command::Survey {
            all,
            format,
            json,
            path,
        } => priority::run_survey(
            path,
            all,
            format,
            json,
            crate::listing::RenderOpts {
                color,
                term_width: crate::tty::stdout_terminal_width(),
            },
        ),
        Command::Next { format, json, path } => priority::run_next(
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
        } => priority::run_blockers(
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
        } => priority::run_explain(
            path,
            &id,
            format,
            json,
            crate::listing::RenderOpts {
                color,
                term_width: crate::tty::stdout_terminal_width(),
            },
        ),
        Command::Adr { command } => match command {
            AdrCommand::New { title, slug, path } => adr::run_new(path, title, slug),
            AdrCommand::List { list, path } => adr::run_list(path, list.into_list_args(color)),
            AdrCommand::Show {
                reference,
                format,
                json,
                path,
            } => adr::run_show(path, &reference, if json { Format::Json } else { format }),
            AdrCommand::Status { id, status, path } => adr::run_status(path, id, status, color),
        },
        Command::Policy { command } => match command {
            PolicyCommand::New { title, slug, path } => policy::run_new(path, title, slug),
            PolicyCommand::List { list, path } => {
                policy::run_list(path, list.into_list_args(color))
            }
            PolicyCommand::Show {
                reference,
                format,
                json,
                path,
            } => policy::run_show(path, &reference, if json { Format::Json } else { format }),
            PolicyCommand::Status { id, status, path } => {
                policy::run_status(path, id, status, color)
            }
        },
        Command::Standard { command } => match command {
            StandardCommand::New { title, slug, path } => standard::run_new(path, title, slug),
            StandardCommand::List { list, path } => {
                standard::run_list(path, list.into_list_args(color))
            }
            StandardCommand::Show {
                reference,
                format,
                json,
                path,
            } => standard::run_show(path, &reference, if json { Format::Json } else { format }),
            StandardCommand::Status { id, status, path } => {
                standard::run_status(path, id, status, color)
            }
        },
        Command::Rfc { command } => match command {
            RfcCommand::New { title, slug, path } => rfc::run_new(path, title, slug),
            RfcCommand::List { list, path } => rfc::run_list(path, list.into_list_args(color)),
            RfcCommand::Show {
                reference,
                format,
                json,
                path,
            } => rfc::run_show(path, &reference, if json { Format::Json } else { format }),
            RfcCommand::Status { id, status, path } => rfc::run_status(path, id, status, color),
        },
        Command::Spec { command } => match command {
            SpecCommand::New {
                subtype,
                title,
                slug,
                path,
            } => spec::run_new(path, subtype, title, slug),
            SpecCommand::List { list, path } => spec::run_list(path, list.into_list_args(color)),
            SpecCommand::Show {
                spec_ref,
                format,
                json,
                path,
            } => spec::run_show(path, &spec_ref, if json { Format::Json } else { format }),
            SpecCommand::Validate { spec_ref, path } => {
                spec::run_validate(path, spec_ref.as_deref())
            }
            SpecCommand::Req { command } => match command {
                SpecReqCommand::Add {
                    spec_ref,
                    title,
                    kind,
                    label,
                    slug,
                    path,
                } => spec::run_req_add(path, &spec_ref, title, kind, label, slug),
                SpecReqCommand::Status {
                    req_ref,
                    to,
                    note,
                    path,
                } => spec::run_req_status(path, &req_ref, to, note),
                SpecReqCommand::List {
                    spec_ref,
                    list,
                    path,
                } => spec::run_req_list(path, &spec_ref, list.into_list_args(color)),
            },
        },
        Command::Export { command } => match command {
            ExportCommand::Lazyspec { path } => {
                // Impure shell only: resolve root + read the clock/version at the
                // boundary, then hand pure data to `run_export_lazyspec`.
                use std::io::Write;
                let root = crate::root::find(path, &crate::root::default_markers())?;
                let now = crate::clock::now_timestamp()?;
                let version = env!("CARGO_PKG_VERSION");
                let json = lazyspec::run_export_lazyspec(&root, &now, version)?;
                writeln!(std::io::stdout(), "{json}")?;
                Ok(())
            }
        },
        Command::Backlog { command } => match command {
            BacklogCommand::New {
                kind,
                title,
                slug,
                path,
            } => backlog::run_new(path, kind, title, slug),
            BacklogCommand::List {
                kind,
                by,
                mut list,
                substr,
                path,
            } => {
                // A-7: the positional `[SUBSTR]` is a DEPRECATED alias of `--filter`;
                // `--filter` WINS when both are given (the positional folds in only
                // when `--filter` is absent). Documented precedence, not an error.
                if list.filter.is_none() {
                    list.filter = substr;
                }
                backlog::run_list(path, kind, by, list.into_list_args(color))
            }
            BacklogCommand::Show {
                id,
                format,
                json,
                path,
            } => backlog::run_show(path, &id, if json { Format::Json } else { format }),
            BacklogCommand::Edit {
                id,
                status,
                resolution,
                path,
            } => backlog::run_edit(path, &id, status, resolution),
            BacklogCommand::Needs { id, prereqs, path } => backlog::run_needs(path, &id, &prereqs),
            BacklogCommand::After {
                id,
                to,
                rank,
                remove,
                prune,
                path,
            } => backlog::run_after(path, &id, to.as_deref(), rank, remove, prune),
            BacklogCommand::Tag {
                id,
                tags,
                remove,
                path,
            } => backlog::run_tag(path, &id, &tags, &remove),
        },
        Command::Knowledge { command } => match command {
            KnowledgeCommand::New {
                kind,
                title,
                slug,
                path,
            } => knowledge::run_new(path, kind, title, slug),
            KnowledgeCommand::List { list, path } => {
                knowledge::run_list(path, list.into_list_args(color))
            }
            KnowledgeCommand::Show {
                id,
                format,
                json,
                path,
            } => knowledge::run_show(path, &id, if json { Format::Json } else { format }),
            KnowledgeCommand::Status { id, state, path } => {
                knowledge::run_status(path, &id, &state, color)
            }
        },
        Command::Serve { args } => commands::serve::run_serve(args),
        Command::Boot {
            command,
            check,
            path: boot_path,
        } => match command {
            None if check => boot::run_check(boot_path),
            None => boot::run(boot_path),
            Some(BootCommand::Install {
                path,
                agent,
                dry_run,
                yes,
            }) => boot::run_install(path, &agent, dry_run, yes),
        },
        Command::Catalog { command } => match command {
            CatalogCommand::Scan { root } => run_catalog_scan(root),
            CatalogCommand::Graph { root } => run_catalog_graph(root),
        },
        Command::Worktree { command } => match command {
            WorktreeCommand::Provision { fork, path } => worktree::run_provision(path, &fork),
            WorktreeCommand::CheckAllowlist { path } => worktree::run_check_allowlist(path),
            WorktreeCommand::BranchPointCheck { base, head, path } => {
                worktree::run_branch_point_check(path, &base, head)
            }
            WorktreeCommand::Fork {
                base,
                branch,
                dir,
                worker,
                path,
            } => worktree::run_fork(path, &base, &branch, &dir, worker),
            WorktreeCommand::Coordinate { slice, dir, path } => {
                worktree::run_coordinate(path, slice, &dir)
            }
            WorktreeCommand::Import { base, fork, path } => {
                worktree::run_import(path, &base, &fork)
            }
            WorktreeCommand::Land { fork, path } => worktree::run_land(path, &fork),
            WorktreeCommand::Gc {
                fork,
                superseded_head,
                force,
                dry_run,
                path,
            } => worktree::run_gc(path, &fork, superseded_head.as_deref(), force, dry_run),
            WorktreeCommand::Status { assert, path } => worktree::run_status(path, assert),
            WorktreeCommand::VerifyWorker { base, dir, branch } => {
                worktree::run_verify_worker(&base, &dir, branch.as_deref())
            }
            WorktreeCommand::Marker {
                clear,
                operator,
                stamp_subagent,
                path,
            } => {
                if stamp_subagent {
                    worktree::run_stamp_subagent(path)
                } else if clear {
                    worktree::run_marker_clear(path, operator)
                } else {
                    anyhow::bail!("`worktree marker` requires `--clear` or `--stamp-subagent`")
                }
            }
        },
        Command::Dispatch { command } => match command {
            DispatchCommand::Sync {
                slice,
                integrate,
                show_journal_trunk_oid,
                trunk,
                edge,
                path,
                ..
            } => {
                // The `stage` group is `required = true` single-choice: exactly one
                // of `--prepare-review` / `--integrate` / `--show-journal-trunk-oid`
                // is set, so the booleans select the stage in order (no unreachable
                // arm).
                if show_journal_trunk_oid {
                    // SL-128 D3: absent `--trunk` defaults from `[dispatch] deliver_to`;
                    // explicit `--trunk` still wins. `--integrate` is unchanged.
                    dispatch::run_show_journal_trunk_oid(path, slice, trunk.as_deref())
                } else if integrate {
                    dispatch::run_integrate(path, slice, trunk.as_deref(), edge.as_deref())
                } else {
                    dispatch::run_prepare_review(path, slice)
                }
            }
            DispatchCommand::RecordBoundary {
                slice,
                phase,
                code_start,
                code_end,
                path,
            } => dispatch::run_record_boundary(path, slice, &phase, &code_start, &code_end),
            DispatchCommand::RefreshBase { slice, path } => dispatch::run_refresh_base(path, slice),
            DispatchCommand::Setup { slice, dir, path } => {
                // Read the harness signal here in the shell (ISS-031 placement
                // guard); a `CLAUDE`-prefixed env var marks the Claude arm, whose
                // outside-root coordination dir silently produces a wrong base.
                let claude_harness =
                    std::env::vars_os().any(|(k, _v)| k.to_string_lossy().starts_with("CLAUDE"));
                dispatch::run_setup(path, slice, &dir, claude_harness)
            }
            DispatchCommand::Candidate { command } => match command {
                CandidateCommand::Create {
                    slice,
                    label,
                    kind,
                    role,
                    payload,
                    base,
                    source,
                    supersedes,
                    worktree,
                    path,
                } => {
                    let req = dispatch::CreateRequest {
                        slice,
                        label,
                        kind: dispatch::parse_kind(&kind)?,
                        role: dispatch::parse_role(&role)?,
                        payload: dispatch::parse_payload(&payload)?,
                        base,
                        source,
                        supersedes,
                        worktree,
                        created_at: clock::today(),
                    };
                    dispatch::run_candidate_create(path, &req)
                }
                CandidateCommand::Status { slice, path } => {
                    dispatch::run_candidate_status(path, slice)
                }
                CandidateCommand::Admit {
                    slice,
                    role,
                    candidate,
                    review,
                    path,
                } => {
                    let req = dispatch::AdmitRequest {
                        slice,
                        role: dispatch::parse_role(&role)?,
                        candidate,
                        review,
                        admitted_at: clock::today(),
                    };
                    dispatch::run_candidate_admit(path, &req)
                }
            },
            DispatchCommand::PlanNext { slice, json, path } => {
                dispatch::run_plan_next(path, slice, json)
            }
            DispatchCommand::Status { slice, json, path } => {
                dispatch::run_status(path, slice, json)
            }
            DispatchCommand::DeliverTo { path } => dispatch::run_deliver_to(path),
        },
        Command::Validate { path } => commands::validate::run_validate(path),
        Command::Reseat {
            reference,
            to,
            path,
        } => integrity::run_reseat(path, &reference, to),
        Command::Link {
            source,
            label,
            target,
            path,
        } => commands::relation::run_link(path, &source, &label, &target),
        Command::Unlink {
            source,
            label,
            target,
            path,
        } => commands::relation::run_unlink(path, &source, &label, &target),
        Command::Needs {
            source,
            target,
            path,
        } => commands::dep_seq::run_needs_edge(path, &source, &target),
        Command::After {
            source,
            target,
            rank,
            remove,
            prune,
            path,
        } => {
            if prune {
                commands::dep_seq::run_after_prune(path, &source)
            } else if remove {
                // target is guaranteed Some by clap (required_unless_present="prune")
                commands::dep_seq::run_after_remove(
                    path,
                    &source,
                    target.as_deref().unwrap_or(""),
                    rank,
                )
            } else {
                commands::dep_seq::run_after_edge(
                    path,
                    &source,
                    target.as_deref().unwrap_or(""),
                    rank,
                )
            }
        }
        Command::Status { format, json, path } => status::run(path, format, json),
        Command::Estimate { action } => match action {
            EstimateAction::Set(args) => commands::facet::run_estimate_set(&args),
            EstimateAction::Clear(args) => commands::facet::run_estimate_clear(&args),
        },
        Command::Value { action } => match action {
            ValueAction::Set(args) => commands::facet::run_value_set(&args),
            ValueAction::Clear(args) => commands::facet::run_value_clear(&args),
        },
        Command::Supersede { new, old, path } => {
            commands::supersede::run_supersede(path, &new, &old)
        }
        Command::Map { command } => match command {
            MapCommand::Serve(args) => commands::map::run_serve(None, args),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    #[test]
    fn only_memory_conflicts_with_skill() {
        let r = Cli::try_parse_from([
            "doctrine",
            "skills",
            "install",
            "--only-memory",
            "--skill",
            "code-review",
        ]);
        assert!(r.is_err());
    }

    #[test]
    fn only_memory_conflicts_with_domain() {
        let r = Cli::try_parse_from([
            "doctrine",
            "skills",
            "install",
            "--only-memory",
            "--domain",
            "doctrine",
        ]);
        assert!(r.is_err());
    }

    #[test]
    fn only_memory_alone_parses() {
        // The consolidated install surface (SL-088); --only-memory moved from
        // the removed `skills install` to `install`.
        let r = Cli::try_parse_from(["doctrine", "install", "--only-memory"]);
        assert!(r.is_ok());
    }

    #[test]
    fn skills_install_is_gone() {
        let r = Cli::try_parse_from(["doctrine", "skills", "install"]);
        assert!(r.is_err());
    }
}

// ---------------------------------------------------------------------------
// The estimate / value handler tests moved to commands/facet.rs
// This placeholder keeps the line count stable until fmt.

#[cfg(test)]
mod write_class_tests {
    use super::*;
    use crate::commands::guard::{WriteClass, write_class};

    // Read => None, Write(label) => Some(label). The compiler's totality (no
    // wildcard in `write_class`) proves every variant is *handled*; this table
    // pins the Read/Write split + verb labels (VT-1).
    fn cls(cmd: Command) -> Option<&'static str> {
        match write_class(&cmd) {
            WriteClass::Read => None,
            // All refused classes carry a verb label; the guard refuses each.
            WriteClass::Write(v) | WriteClass::Orchestrator(v) | WriteClass::Hookmint(v) => Some(v),
            // The bespoke MarkerClear class is neither Read nor a guarded Write;
            // the dedicated `worktree_marker_is_bespoke_class` test pins it.
            WriteClass::MarkerClear => None,
        }
    }

    // The 8-field shared list flags — every `list` verb is a Read; a helper
    // tames the construction noise across the kinds.
    fn clist() -> CommonListArgs {
        CommonListArgs {
            filter: None,
            regexp: None,
            case_insensitive: false,
            status: Vec::new(),
            tag: Vec::new(),
            all: false,
            format: Format::Table,
            json: false,
            columns: None,
        }
    }

    #[test]
    fn install_is_write() {
        assert_eq!(
            cls(Command::Install {
                path: None,
                agent: Vec::new(),
                skill: Vec::new(),
                domain: Vec::new(),
                only_memory: false,
                global: false,
                dry_run: false,
                yes: false
            }),
            Some("install")
        );
    }

    #[test]
    fn skills_list_is_read() {
        assert_eq!(
            cls(Command::Skills {
                command: SkillsCommand::List {
                    agent: None,
                    installed: false
                }
            }),
            None
        );
    }

    #[test]
    fn slice_split() {
        let w = |c| cls(Command::Slice { command: c });
        assert_eq!(
            w(SliceCommand::New {
                title: None,
                slug: None,
                path: None
            }),
            Some("slice new")
        );
        assert_eq!(
            w(SliceCommand::Design { id: 0, path: None }),
            Some("slice design")
        );
        assert_eq!(
            w(SliceCommand::Plan { id: 0, path: None }),
            Some("slice plan")
        );
        assert_eq!(
            w(SliceCommand::Phases {
                id: 0,
                prune: false,
                path: None
            }),
            Some("slice phases")
        );
        assert_eq!(
            w(SliceCommand::Notes { id: 0, path: None }),
            Some("slice notes")
        );
        assert_eq!(
            w(SliceCommand::Phase {
                id: 0,
                phase_id: String::new(),
                status: state::PhaseStatus::Planned,
                note: None,
                path: None,
            }),
            Some("slice phase")
        );
        assert_eq!(
            w(SliceCommand::Status {
                id: 0,
                state: slice::SliceStatus::Proposed,
                note: None,
                path: None,
            }),
            Some("slice status")
        );
        assert_eq!(
            w(SliceCommand::List {
                list: clist(),
                path: None
            }),
            None
        );
        assert_eq!(
            w(SliceCommand::Show {
                reference: String::new(),
                format: Format::Table,
                json: false,
                path: None,
            }),
            None
        );
    }

    #[test]
    fn memory_split() {
        let w = |c| cls(Command::Memory { command: c });
        assert_eq!(
            w(MemoryCommand::Record {
                title: String::new(),
                memory_type: memory::MemoryType::Concept,
                key: None,
                lifespan: None,
                status: memory::Status::Active,
                summary: None,
                review_by: None,
                provenance_source: Vec::new(),
                trust: None,
                severity: None,
                tag: Vec::new(),
                path_scope: Vec::new(),
                glob: Vec::new(),
                command: Vec::new(),
                repo: None,
                global: false,
                path: None,
            }),
            Some("memory record")
        );
        assert_eq!(
            w(MemoryCommand::Verify {
                reference: String::new(),
                allow_dirty: false,
                path: None
            }),
            Some("memory verify")
        );
        assert_eq!(
            w(MemoryCommand::Show {
                reference: String::new(),
                format: Format::Table,
                json: false,
                path: None,
            }),
            None
        );
        assert_eq!(
            w(MemoryCommand::List {
                memory_type: None,
                list: clist(),
                path: None,
            }),
            None
        );
        assert_eq!(
            w(MemoryCommand::Find {
                query: None,
                args: FindRetrieveArgs {
                    path_scope: Vec::new(),
                    glob: Vec::new(),
                    command: Vec::new(),
                    tag: Vec::new(),
                    flag_query: None,
                    memory_type: None,
                    status: None,
                    lifespan: None,
                    include_draft: false,
                    format: Format::Table,
                    json: false,
                    offset: 0,
                    page: None,
                    limit: None,
                    path: None,
                    expand: None,
                },
            }),
            None
        );
        assert_eq!(
            w(MemoryCommand::Retrieve {
                args: FindRetrieveArgs {
                    path_scope: Vec::new(),
                    glob: Vec::new(),
                    command: Vec::new(),
                    tag: Vec::new(),
                    flag_query: None,
                    memory_type: None,
                    status: None,
                    lifespan: None,
                    include_draft: false,
                    format: Format::Table,
                    json: false,
                    offset: 0,
                    page: None,
                    limit: None,
                    path: None,
                    expand: None,
                },
                min_trust: None,
            }),
            None
        );
        assert_eq!(
            w(MemoryCommand::ResolveLinks {
                reference: None,
                path: None,
            }),
            None
        );
        assert_eq!(
            w(MemoryCommand::Backlinks {
                reference: String::new(),
                path: None,
            }),
            None
        );
        // Nested Option — bare `memory sync` AND `memory sync install` are both Write.
        assert_eq!(
            w(MemoryCommand::Sync {
                command: None,
                dry_run: false,
                yes: false,
                path: None,
            }),
            Some("memory sync")
        );
        assert_eq!(
            w(MemoryCommand::Sync {
                command: Some(SyncCommand::Install {
                    path: None,
                    dry_run: false,
                    yes: false,
                }),
                dry_run: false,
                yes: false,
                path: None,
            }),
            Some("memory sync install")
        );
        assert_eq!(
            w(MemoryCommand::Status {
                reference: String::new(),
                state: String::new(),
                by: None,
                path: None,
            }),
            Some("memory status")
        );
        assert_eq!(
            w(MemoryCommand::Edit {
                reference: String::new(),
                title: None,
                summary: None,
                status: None,
                lifespan: None,
                review_by: None,
                trust: None,
                severity: None,
                key: None,
                path_scope: vec![],
                glob: vec![],
                command: vec![],
                path: None,
            }),
            Some("memory edit")
        );
    }

    #[test]
    fn memory_record_new_flags_parse_and_reach_the_variant() {
        let cli = Cli::try_parse_from([
            "doctrine",
            "memory",
            "record",
            "T",
            "--type",
            "fact",
            "--lifespan",
            "semantic",
            "--review-by",
            "2026-08-01",
            "--provenance-source",
            "code:src/main.rs:42",
            "--trust",
            "low",
            "--severity",
            "critical",
        ])
        .unwrap();
        let Command::Memory {
            command:
                MemoryCommand::Record {
                    lifespan,
                    review_by,
                    provenance_source,
                    trust,
                    severity,
                    ..
                },
        } = cli.command
        else {
            panic!("expected memory record");
        };
        assert_eq!(lifespan, Some(memory::Lifespan::Semantic));
        assert_eq!(review_by.as_deref(), Some("2026-08-01"));
        assert_eq!(provenance_source.len(), 1);
        assert_eq!(provenance_source[0].kind, "code");
        assert_eq!(provenance_source[0].ref_, "src/main.rs:42");
        assert_eq!(trust.as_deref(), Some("low"));
        assert_eq!(severity.as_deref(), Some("critical"));
    }

    #[test]
    fn memory_record_invalid_lifespan_is_rejected() {
        let cli = Cli::try_parse_from([
            "doctrine",
            "memory",
            "record",
            "T",
            "--type",
            "fact",
            "--lifespan",
            "bogus",
        ]);
        assert!(cli.is_err());
    }

    #[test]
    fn memory_find_retrieve_lifespan_flag_parses_on_the_shared_args() {
        let find =
            Cli::try_parse_from(["doctrine", "memory", "find", "--lifespan", "semantic"]).unwrap();
        let Command::Memory {
            command: MemoryCommand::Find { args, .. },
        } = find.command
        else {
            panic!("expected memory find");
        };
        assert_eq!(args.lifespan, Some(memory::Lifespan::Semantic));

        let retrieve =
            Cli::try_parse_from(["doctrine", "memory", "retrieve", "--lifespan", "working"])
                .unwrap();
        let Command::Memory {
            command: MemoryCommand::Retrieve { args, .. },
        } = retrieve.command
        else {
            panic!("expected memory retrieve");
        };
        assert_eq!(args.lifespan, Some(memory::Lifespan::Working));
    }

    #[test]
    fn memory_find_invalid_lifespan_is_rejected() {
        let cli = Cli::try_parse_from(["doctrine", "memory", "find", "--lifespan", "garbage"]);
        assert!(cli.is_err());
    }

    #[test]
    fn adr_split() {
        let w = |c| cls(Command::Adr { command: c });
        assert_eq!(
            w(AdrCommand::New {
                title: None,
                slug: None,
                path: None
            }),
            Some("adr new")
        );
        assert_eq!(
            w(AdrCommand::Status {
                id: 0,
                status: adr::AdrStatus::Proposed,
                path: None,
            }),
            Some("adr status")
        );
        assert_eq!(
            w(AdrCommand::List {
                list: clist(),
                path: None
            }),
            None
        );
        assert_eq!(
            w(AdrCommand::Show {
                reference: String::new(),
                format: Format::Table,
                json: false,
                path: None,
            }),
            None
        );
    }

    #[test]
    fn policy_split() {
        let w = |c| cls(Command::Policy { command: c });
        assert_eq!(
            w(PolicyCommand::New {
                title: None,
                slug: None,
                path: None
            }),
            Some("policy new")
        );
        assert_eq!(
            w(PolicyCommand::Status {
                id: 0,
                status: policy::PolicyStatus::Draft,
                path: None,
            }),
            Some("policy status")
        );
        assert_eq!(
            w(PolicyCommand::List {
                list: clist(),
                path: None
            }),
            None
        );
        assert_eq!(
            w(PolicyCommand::Show {
                reference: String::new(),
                format: Format::Table,
                json: false,
                path: None,
            }),
            None
        );
    }

    #[test]
    fn standard_split() {
        let w = |c| cls(Command::Standard { command: c });
        assert_eq!(
            w(StandardCommand::New {
                title: None,
                slug: None,
                path: None
            }),
            Some("standard new")
        );
        assert_eq!(
            w(StandardCommand::Status {
                id: 0,
                status: standard::StandardStatus::Draft,
                path: None,
            }),
            Some("standard status")
        );
        assert_eq!(
            w(StandardCommand::List {
                list: clist(),
                path: None
            }),
            None
        );
        assert_eq!(
            w(StandardCommand::Show {
                reference: String::new(),
                format: Format::Table,
                json: false,
                path: None,
            }),
            None
        );
    }

    #[test]
    fn spec_split() {
        let w = |c| cls(Command::Spec { command: c });
        assert_eq!(
            w(SpecCommand::New {
                subtype: spec::SpecSubtype::Product,
                title: None,
                slug: None,
                path: None,
            }),
            Some("spec new")
        );
        // Three levels deep: Spec -> Req -> Add.
        assert_eq!(
            w(SpecCommand::Req {
                command: SpecReqCommand::Add {
                    spec_ref: String::new(),
                    title: None,
                    kind: requirement::ReqKind::Functional,
                    label: None,
                    slug: None,
                    path: None,
                }
            }),
            Some("spec req add")
        );
        // sibling: Spec -> Req -> Status is also a Write.
        assert_eq!(
            w(SpecCommand::Req {
                command: SpecReqCommand::Status {
                    req_ref: String::new(),
                    to: requirement::ReqStatus::Active,
                    note: None,
                    path: None,
                }
            }),
            Some("spec req status")
        );
        assert_eq!(
            w(SpecCommand::List {
                list: clist(),
                path: None
            }),
            None
        );
        assert_eq!(
            w(SpecCommand::Show {
                spec_ref: String::new(),
                format: Format::Table,
                json: false,
                path: None,
            }),
            None
        );
        assert_eq!(
            w(SpecCommand::Validate {
                spec_ref: None,
                path: None
            }),
            None
        );
    }

    #[test]
    fn backlog_split() {
        let w = |c| cls(Command::Backlog { command: c });
        assert_eq!(
            w(BacklogCommand::New {
                kind: backlog::ItemKind::Issue,
                title: None,
                slug: None,
                path: None,
            }),
            Some("backlog new")
        );
        assert_eq!(
            w(BacklogCommand::Edit {
                id: String::new(),
                status: backlog::Status::Open,
                resolution: None,
                path: None,
            }),
            Some("backlog edit")
        );
        assert_eq!(
            w(BacklogCommand::List {
                kind: None,
                by: backlog::OrderBy::Sequence,
                list: clist(),
                substr: None,
                path: None,
            }),
            None
        );
        assert_eq!(
            w(BacklogCommand::Show {
                id: String::new(),
                format: Format::Table,
                json: false,
                path: None,
            }),
            None
        );
    }

    #[test]
    fn knowledge_split() {
        let w = |c| cls(Command::Knowledge { command: c });
        assert_eq!(
            w(KnowledgeCommand::New {
                kind: knowledge::RecordKind::Assumption,
                title: None,
                slug: None,
                path: None,
            }),
            Some("knowledge new")
        );
        assert_eq!(
            w(KnowledgeCommand::Status {
                id: String::new(),
                state: String::new(),
                path: None,
            }),
            Some("knowledge status")
        );
        assert_eq!(
            w(KnowledgeCommand::List {
                list: clist(),
                path: None,
            }),
            None
        );
        assert_eq!(
            w(KnowledgeCommand::Show {
                id: String::new(),
                format: Format::Table,
                json: false,
                path: None,
            }),
            None
        );
    }

    #[test]
    fn boot_split() {
        // Bare regenerate (None) AND `boot install` are both Write. `--check` is
        // a read-only sentry but the superset (§5.2) sweeps the whole verb to
        // Write — workers never run it, and over-refusing a read is the safe side.
        assert_eq!(
            cls(Command::Boot {
                command: None,
                check: false,
                path: None
            }),
            Some("boot")
        );
        assert_eq!(
            cls(Command::Boot {
                command: None,
                check: true,
                path: None
            }),
            Some("boot")
        );
        assert_eq!(
            cls(Command::Boot {
                command: Some(BootCommand::Install {
                    path: None,
                    agent: Vec::new(),
                    dry_run: false,
                    yes: false,
                }),
                check: false,
                path: None,
            }),
            Some("boot install")
        );
    }

    #[test]
    fn worktree_is_read() {
        // Deliberate (§5.2): these write *fork* files, not the doctrine state the
        // guard protects, and never run in worker context.
        assert_eq!(
            cls(Command::Worktree {
                command: WorktreeCommand::Provision {
                    fork: PathBuf::from("x"),
                    path: None,
                }
            }),
            None
        );
        assert_eq!(
            cls(Command::Worktree {
                command: WorktreeCommand::CheckAllowlist { path: None }
            }),
            None
        );
        // SL-056 §3: `worktree status` reads the resolved mode — Read (open to
        // workers), so it survives the guard.
        assert_eq!(
            cls(Command::Worktree {
                command: WorktreeCommand::Status {
                    assert: false,
                    path: None,
                }
            }),
            None
        );
    }

    // SL-056 §3/§5: `worktree marker --clear` is the bespoke MarkerClear class —
    // NOT a guarded Write (locking the marker's remover behind the marker is a
    // self-brick). The guard must not refuse it; its own fences live in the handler.
    #[test]
    fn worktree_marker_is_bespoke_class() {
        let c = Command::Worktree {
            command: WorktreeCommand::Marker {
                clear: true,
                operator: false,
                stamp_subagent: false,
                path: None,
            },
        };
        assert!(
            matches!(write_class(&c), WriteClass::MarkerClear),
            "marker --clear must be the bespoke MarkerClear class"
        );
        // And therefore not seen as a guarded Write by `cls`.
        assert_eq!(cls(c), None);
    }

    // SL-056 PHASE-10: `worktree marker --stamp-subagent` is the Hookmint class —
    // refused under worker-mode via the SAME branch as Orchestrator/Write (NO
    // verb-identity carve-out), carries the "marker --stamp-subagent" verb label.
    #[test]
    fn worktree_marker_stamp_subagent_is_hookmint() {
        let c = Command::Worktree {
            command: WorktreeCommand::Marker {
                clear: false,
                operator: false,
                stamp_subagent: true,
                path: None,
            },
        };
        assert!(
            matches!(
                write_class(&c),
                WriteClass::Hookmint("marker --stamp-subagent")
            ),
            "marker --stamp-subagent must be the Hookmint class"
        );
        // A guarded Write to `cls` — the worker-mode guard refuses it.
        assert_eq!(cls(c), Some("marker --stamp-subagent"));
    }

    // SL-056 PHASE-06: `worktree fork` is the FIRST Orchestrator-classed verb —
    // refused under worker-mode, carries the "fork" verb label.
    #[test]
    fn worktree_fork_is_orchestrator() {
        let c = Command::Worktree {
            command: WorktreeCommand::Fork {
                base: "B".to_string(),
                branch: "wkr".to_string(),
                dir: PathBuf::from("x"),
                worker: false,
                path: None,
            },
        };
        assert!(
            matches!(write_class(&c), WriteClass::Orchestrator("fork")),
            "fork must be Orchestrator(\"fork\")"
        );
        // The guard treats it like a Write: cls surfaces the verb label.
        assert_eq!(cls(c), Some("fork"));
    }

    // SL-064 PHASE-04: `dispatch sync --prepare-review` is Orchestrator-classed —
    // refused under worker-mode, carries the "dispatch-sync" verb label (EX-1).
    #[test]
    fn dispatch_sync_is_orchestrator() {
        let c = Command::Dispatch {
            command: DispatchCommand::Sync {
                slice: 64,
                prepare_review: true,
                integrate: false,
                show_journal_trunk_oid: false,
                trunk: None,
                edge: None,
                path: None,
            },
        };
        assert!(
            matches!(write_class(&c), WriteClass::Orchestrator("dispatch-sync")),
            "dispatch sync must be Orchestrator(\"dispatch-sync\")"
        );
        assert_eq!(cls(c), Some("dispatch-sync"));
    }

    // SL-064 PHASE-05: `dispatch sync --integrate` is the same Orchestrator verb
    // class (EX-6) — the trunk-writing stage inherits the worker-mode refusal.
    #[test]
    fn dispatch_sync_integrate_is_orchestrator() {
        let c = Command::Dispatch {
            command: DispatchCommand::Sync {
                slice: 64,
                prepare_review: false,
                integrate: true,
                show_journal_trunk_oid: false,
                trunk: None,
                edge: None,
                path: None,
            },
        };
        assert!(
            matches!(write_class(&c), WriteClass::Orchestrator("dispatch-sync")),
            "dispatch sync --integrate must be Orchestrator(\"dispatch-sync\")"
        );
        assert_eq!(cls(c), Some("dispatch-sync"));
    }

    #[test]
    fn inspect_is_read() {
        // SL-046: the cross-kind relation view reads only — never mints/derives.
        assert_eq!(
            cls(Command::Inspect {
                id: "SL-046".to_string(),
                format: Format::Table,
                json: false,
                path: None,
            }),
            None
        );
    }

    #[test]
    fn validate_is_read_reseat_is_write() {
        // Corpus integrity: the scan reads (INV-3); reseat mutates the canonical
        // triple, so it is a worker-refused authored write (D2/D6).
        assert_eq!(cls(Command::Validate { path: None }), None);
        assert_eq!(
            cls(Command::Reseat {
                reference: "SL-001".to_string(),
                to: None,
                path: None,
            }),
            Some("reseat")
        );
    }

    // SL-118 PHASE-03: Estimate/Value write-class tests.

    fn estimate_cmd() -> Command {
        Command::Estimate {
            action: EstimateAction::Set(EstimateSetArgs {
                id: "SL-001".into(),
                lower: Some(1.0),
                upper: Some(3.0),
                exact: None,
                path: None,
            }),
        }
    }

    #[test]
    fn estimate_is_write() {
        assert_eq!(cls(estimate_cmd()), Some("estimate"));
    }

    #[test]
    fn value_is_write() {
        let c = Command::Value {
            action: ValueAction::Set(ValueSetArgs {
                id: "SL-001".into(),
                magnitude: 42.0,
                path: None,
            }),
        };
        assert_eq!(cls(c), Some("value"));
    }

    // ── PHASE-01: Behaviour-preservation verification net (SL-115) ──────────────

    #[test]
    fn help_snapshot_top_level() {
        let help = <Cli as clap::CommandFactory>::command()
            .render_help()
            .to_string();
        assert!(help.contains("doctrine CLI"), "top-level about text");
        assert!(help.contains("Usage: doctrine"), "usage line");
        assert!(help.contains("Commands:"), "commands section");
        // Representative subcommands that would be visibly absent if
        // the top-level command tree is accidentally restructured.
        assert!(help.contains("  install"), "install command present");
        assert!(help.contains("  slice"), "slice command present");
        assert!(help.contains("  memory"), "memory command present");
        assert!(help.contains("  adr"), "adr command present");
        assert!(help.contains("  spec"), "spec command present");
        assert!(help.contains("  dispatch"), "dispatch command present");
        assert!(help.contains("  help"), "help command always present");
        assert!(help.contains("Options:"), "global options");
        assert!(help.contains("--color"), "color flag in help");
    }

    #[test]
    fn help_snapshot_slice_subcommand() {
        let help = <Cli as clap::CommandFactory>::command()
            .find_subcommand_mut("slice")
            .unwrap()
            .render_help()
            .to_string();
        assert!(help.contains("Create and list slices"));
        assert!(help.contains("new"));
        assert!(help.contains("design"));
        assert!(help.contains("plan"));
        assert!(help.contains("list"));
        assert!(help.contains("show"));
    }

    #[test]
    fn help_snapshot_memory_subcommand() {
        let help = <Cli as clap::CommandFactory>::command()
            .find_subcommand_mut("memory")
            .unwrap()
            .render_help()
            .to_string();
        assert!(help.contains("Record, show, and list memories"));
        assert!(help.contains("record"));
        assert!(help.contains("find"));
        assert!(help.contains("retrieve"));
        assert!(help.contains("list"));
    }

    #[test]
    fn help_snapshot_adr_subcommand() {
        let help = <Cli as clap::CommandFactory>::command()
            .find_subcommand_mut("adr")
            .unwrap()
            .render_help()
            .to_string();
        assert!(help.contains("Create and list architecture decision records"));
        assert!(help.contains("new"));
        assert!(help.contains("list"));
        assert!(help.contains("show"));
        assert!(help.contains("status"));
    }

    #[test]
    fn help_snapshot_spec_subcommand() {
        let help = <Cli as clap::CommandFactory>::command()
            .find_subcommand_mut("spec")
            .unwrap()
            .render_help()
            .to_string();
        assert!(help.contains("Create and list product / technical specifications"));
        assert!(help.contains("new"));
        assert!(help.contains("list"));
        assert!(help.contains("show"));
        assert!(help.contains("validate"));
        assert!(help.contains("req"));
    }

    // ── Parse-regression tests ──────────────────────────────────────────────────

    // (a) CommonListArgs value_delimiter
    #[test]
    fn parse_list_status_value_delimiter_equivalence() {
        let a =
            Cli::try_parse_from(["doctrine", "slice", "list", "--status", "draft,active"]).unwrap();
        let b = Cli::try_parse_from([
            "doctrine", "slice", "list", "--status", "draft", "--status", "active",
        ])
        .unwrap();
        let Command::Slice {
            command: SliceCommand::List { list: la, .. },
        } = a.command
        else {
            panic!("expected SliceCommand::List");
        };
        let Command::Slice {
            command: SliceCommand::List { list: lb, .. },
        } = b.command
        else {
            panic!("expected SliceCommand::List");
        };
        assert_eq!(la.status, lb.status);
        assert_eq!(la.status, ["draft", "active"]);
    }

    // (b) FindRetrieveArgs conflicts_with="offset"
    #[test]
    fn parse_find_retrieve_offset_conflicts_with_page() {
        let r = Cli::try_parse_from(["doctrine", "memory", "find", "--offset", "5", "--page", "2"]);
        assert!(r.is_err(), "offset + page should conflict");
    }

    // (c) FindRetrieveArgs value_parser on MemoryType, Status, Lifespan
    #[test]
    fn parse_find_memory_type_valid() {
        let r = Cli::try_parse_from(["doctrine", "memory", "find", "--type", "concept"]);
        assert!(r.is_ok());
    }

    #[test]
    fn parse_find_memory_type_invalid() {
        let r = Cli::try_parse_from(["doctrine", "memory", "find", "--type", "banana"]);
        assert!(r.is_err());
    }

    #[test]
    fn parse_find_status_valid() {
        let r = Cli::try_parse_from(["doctrine", "memory", "find", "--status", "active"]);
        assert!(r.is_ok());
    }

    #[test]
    fn parse_find_status_invalid() {
        let r = Cli::try_parse_from(["doctrine", "memory", "find", "--status", "foobar"]);
        assert!(r.is_err());
    }

    #[test]
    fn parse_find_lifespan_valid() {
        let r = Cli::try_parse_from(["doctrine", "memory", "find", "--lifespan", "semantic"]);
        assert!(r.is_ok());
    }

    #[test]
    fn parse_find_lifespan_invalid() {
        let r = Cli::try_parse_from(["doctrine", "memory", "find", "--lifespan", "quantum"]);
        assert!(r.is_err());
    }

    // (d) Retrieve value_parser=retrieve::parse_min_trust
    #[test]
    fn parse_retrieve_min_trust_valid() {
        let r = Cli::try_parse_from(["doctrine", "memory", "retrieve", "--min-trust", "high"]);
        assert!(r.is_ok());
    }

    #[test]
    fn parse_retrieve_min_trust_invalid() {
        let r = Cli::try_parse_from(["doctrine", "memory", "retrieve", "--min-trust", "banana"]);
        assert!(r.is_err());
    }

    // (e) DispatchCommand::Sync stage selection
    #[test]
    fn parse_dispatch_sync_prepare_review_parses() {
        let r = Cli::try_parse_from([
            "doctrine",
            "dispatch",
            "sync",
            "--slice",
            "99",
            "--prepare-review",
        ]);
        assert!(r.is_ok(), "sync with --prepare-review");
    }

    #[test]
    fn parse_dispatch_sync_integrate_parses_without_trunk() {
        let r = Cli::try_parse_from([
            "doctrine",
            "dispatch",
            "sync",
            "--slice",
            "99",
            "--integrate",
        ]);
        assert!(r.is_ok(), "sync with --integrate alone (trunk is optional)");
    }

    #[test]
    fn parse_dispatch_sync_missing_stage_errors() {
        let r = Cli::try_parse_from(["doctrine", "dispatch", "sync", "--slice", "99"]);
        assert!(r.is_err(), "sync without a stage selector should error");
    }

    // ── Non-SPINE_KINDS CommonListArgs consumers ────────────────────────────────

    #[test]
    fn parse_concept_map_list_common_list_args() {
        let r = Cli::try_parse_from([
            "doctrine",
            "concept-map",
            "list",
            "--filter",
            "test",
            "--tag",
            "a,b",
            "--all",
            "--json",
        ]);
        assert!(r.is_ok(), "concept-map list with CommonListArgs");
        let parsed = r.unwrap();
        let Command::ConceptMap {
            command: ConceptMapCommand::List { .. },
        } = parsed.command
        else {
            panic!("expected ConceptMapCommand::List");
        };
    }

    #[test]
    fn parse_review_list_common_list_args() {
        let r = Cli::try_parse_from([
            "doctrine",
            "review",
            "list",
            "--status",
            "open",
            "--format",
            "json",
            "--columns",
            "id,title",
        ]);
        assert!(r.is_ok(), "review list with CommonListArgs");
        let parsed = r.unwrap();
        let Command::Review {
            command: ReviewCommand::List { .. },
        } = parsed.command
        else {
            panic!("expected ReviewCommand::List");
        };
    }

    #[test]
    fn parse_rec_list_common_list_args() {
        let r = Cli::try_parse_from([
            "doctrine",
            "rec",
            "list",
            "--all",
            "--regexp",
            "test.*",
            "--case-insensitive",
        ]);
        assert!(r.is_ok(), "rec list with CommonListArgs");
        let parsed = r.unwrap();
        let Command::Rec {
            command: RecCommand::List { .. },
        } = parsed.command
        else {
            panic!("expected RecCommand::List");
        };
    }
}
