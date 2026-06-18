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
mod dtoml;
mod entity;
mod estimate;
mod fsutil;
mod git;
mod governance;
mod input;
mod install;
mod integrity;
mod knowledge;
mod ledger;
mod lexical;
mod lifecycle;
pub(crate) mod links;
mod listing;
mod map_server;
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
mod root;
mod skills;
mod slice;
mod spec;
mod standard;
mod state;
mod status;
mod supersede;
mod tomlfmt;
mod tty;
mod verify;
mod worktree;

use std::path::PathBuf;
use std::str::FromStr;

use clap::{Args, Parser, Subcommand};

use crate::commands::map::MapServeArgs;
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

    /// Create and list product / technical specifications.
    Spec {
        #[command(subcommand)]
        command: SpecCommand,
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
    /// (`no-worker-head`/`unstamped`/`wrong-base`).
    VerifyWorker {
        /// The base commit `B` the worker was meant to fork off (the
        /// orchestrator's coordination HEAD at spawn).
        #[arg(long)]
        base: String,

        /// The worker worktree to verify — the git `-C` root for every probe.
        #[arg(long)]
        dir: PathBuf,
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

        /// Stage-2 only: project the cumulative code units onto this trunk ref,
        /// fast-forward-only + expected-tip CAS (e.g. `refs/heads/main`). Absent ⇒
        /// trunk is left untouched.
        #[arg(long, requires = "integrate")]
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

/// Mutation classification for the worker-mode guard (ADR-006 D2a). `Write`
/// carries the verb label named in the refusal. EXHAUSTIVE by design (§7-D6):
/// no wildcard arm, so a future `Command` variant is a compile error — never a
/// silently-permitted write (the X4 self-defence).
enum WriteClass {
    Read,
    Write(&'static str),
    /// Orchestrator-only privileged verbs (SL-056 PHASE-06): `fork` is the FIRST
    /// member; later phases add `import`/`land`/`gc`. Carries the verb label like
    /// `Write`. REFUSED under worker-mode — these are the orchestrator's funnel
    /// operations, never a worker's.
    Orchestrator(&'static str),
    /// `worktree marker --clear` (SL-056 §3, §5): a bespoke class that the
    /// worker-mode guard does NOT refuse (locking the marker's only remover behind
    /// the marker is a self-brick we reject). Its own bespoke refusals live in
    /// `run_marker_clear`.
    MarkerClear,
    /// `worktree marker --stamp-subagent` (SL-056 PHASE-10): the claude harness
    /// spawn path's provision+mark step. REFUSED under worker-mode via the SAME
    /// branch as `Orchestrator`/`Write` — NO verb-identity carve-out. The legit
    /// first stamp passes automatically: the target worktree bears no marker yet,
    /// so `worker_mode == false` (marker-absent ⇒ allow). Carries the verb label.
    Hookmint(&'static str),
}

fn write_class(cmd: &Command) -> WriteClass {
    use WriteClass::{Hookmint, MarkerClear, Orchestrator, Read, Write};
    match cmd {
        Command::Install { .. } => Write("install"),
        Command::Skills { command } => match command {
            SkillsCommand::List { .. } => Read,
        },
        Command::Map { .. } => Write("map"),
        Command::ConceptMap { command } => match command {
            ConceptMapCommand::New { .. } => Write("concept-map new"),
            ConceptMapCommand::Add { .. } => Write("concept-map add"),
            ConceptMapCommand::Remove { .. } => Write("concept-map remove"),
            ConceptMapCommand::RenameNode { .. } => Write("concept-map rename-node"),
            ConceptMapCommand::List { .. }
            | ConceptMapCommand::Show { .. }
            | ConceptMapCommand::Check { .. }
            | ConceptMapCommand::Export { .. } => Read,
        },
        Command::Slice { command } => match command {
            SliceCommand::New { .. } => Write("slice new"),
            SliceCommand::Design { .. } => Write("slice design"),
            SliceCommand::Plan { .. } => Write("slice plan"),
            SliceCommand::Phases { .. } => Write("slice phases"),
            SliceCommand::Notes { .. } => Write("slice notes"),
            SliceCommand::Phase { .. } => Write("slice phase"),
            SliceCommand::Status { .. } => Write("slice status"),
            SliceCommand::List { .. } | SliceCommand::Show { .. } => Read,
        },
        Command::Memory { command } => match command {
            MemoryCommand::Record { .. } => Write("memory record"),
            MemoryCommand::Verify { .. } => Write("memory verify"),
            MemoryCommand::Sync { command, .. } => match command {
                None => Write("memory sync"),
                Some(SyncCommand::Install { .. }) => Write("memory sync install"),
            },
            MemoryCommand::Validate { .. }
            | MemoryCommand::Show { .. }
            | MemoryCommand::List { .. }
            | MemoryCommand::Find { .. }
            | MemoryCommand::Retrieve { .. }
            | MemoryCommand::ResolveLinks { .. }
            | MemoryCommand::Backlinks { .. } => Read,
        },
        Command::Review { command } => match command {
            ReviewCommand::New { .. } => Write("review new"),
            ReviewCommand::Raise { .. } => Write("review raise"),
            ReviewCommand::Dispose { .. } => Write("review dispose"),
            ReviewCommand::Verify { .. } => Write("review verify"),
            ReviewCommand::Contest { .. } => Write("review contest"),
            ReviewCommand::Withdraw { .. } => Write("review withdraw"),
            ReviewCommand::Unlock { .. } => Write("review unlock"),
            ReviewCommand::List { .. }
            | ReviewCommand::Show { .. }
            | ReviewCommand::Status { .. }
            | ReviewCommand::Prime { .. } => Read,
        },
        Command::Rec { command } => match command {
            RecCommand::New { .. } => Write("rec new"),
            RecCommand::List { .. } | RecCommand::Show { .. } => Read,
        },
        Command::Revision { command } => match command {
            RevisionCommand::New { .. } => Write("revision new"),
            RevisionCommand::Status { .. } => Write("revision status"),
            RevisionCommand::Show { .. } => Read,
            RevisionCommand::Change { command } => match command {
                RevisionChangeCommand::Add { .. } => Write("revision change add"),
            },
            RevisionCommand::Approve { .. } => Write("revision approve"),
            RevisionCommand::Apply { .. } => Write("revision apply"),
        },
        // Writes authored requirement status + an authored REC — an authored write.
        Command::Reconcile { .. } => Write("reconcile"),
        Command::Adr { command } => match command {
            AdrCommand::New { .. } => Write("adr new"),
            AdrCommand::Status { .. } => Write("adr status"),
            AdrCommand::List { .. } | AdrCommand::Show { .. } => Read,
        },
        Command::Policy { command } => match command {
            PolicyCommand::New { .. } => Write("policy new"),
            PolicyCommand::Status { .. } => Write("policy status"),
            PolicyCommand::List { .. } | PolicyCommand::Show { .. } => Read,
        },
        Command::Standard { command } => match command {
            StandardCommand::New { .. } => Write("standard new"),
            StandardCommand::Status { .. } => Write("standard status"),
            StandardCommand::List { .. } | StandardCommand::Show { .. } => Read,
        },
        Command::Spec { command } => match command {
            SpecCommand::New { .. } => Write("spec new"),
            SpecCommand::Req { command } => match command {
                SpecReqCommand::Add { .. } => Write("spec req add"),
                SpecReqCommand::Status { .. } => Write("spec req status"),
                // Read-only authored roster (design §5.3).
                SpecReqCommand::List { .. } => Read,
            },
            SpecCommand::List { .. } | SpecCommand::Show { .. } | SpecCommand::Validate { .. } => {
                Read
            }
        },
        Command::Backlog { command } => match command {
            BacklogCommand::New { .. } => Write("backlog new"),
            BacklogCommand::Edit { .. } => Write("backlog edit"),
            BacklogCommand::Needs { .. } => Write("backlog needs"),
            BacklogCommand::After { .. } => Write("backlog after"),
            BacklogCommand::Tag { .. } => Write("backlog tag"),
            BacklogCommand::List { .. } | BacklogCommand::Show { .. } => Read,
        },
        Command::Knowledge { command } => match command {
            KnowledgeCommand::New { .. } => Write("knowledge new"),
            KnowledgeCommand::Status { .. } => Write("knowledge status"),
            KnowledgeCommand::List { .. } | KnowledgeCommand::Show { .. } => Read,
        },
        Command::Boot { command, .. } => match command {
            None => Write("boot"),
            Some(BootCommand::Install { .. }) => Write("boot install"),
        },
        Command::Worktree { command } => match command {
            // Provision/check-allowlist write *fork* files, not the doctrine state
            // the guard protects, and never run in worker context (§5.2) — Read.
            // branch-point-check is a HEAD read + ref compare — no authored write,
            // callable under worker-mode by construction (§5.2, C-V).
            // status reads the resolved mode (SL-056 §3) — open to workers.
            // verify-worker is a HEAD read + marker probe + is-ancestor compare on
            // the worker dir — no authored write, diagnostic only; harmless under
            // worker-mode (design §8.4/§8.6 lists no impersonation test for it).
            WorktreeCommand::Provision { .. }
            | WorktreeCommand::CheckAllowlist { .. }
            | WorktreeCommand::BranchPointCheck { .. }
            | WorktreeCommand::VerifyWorker { .. }
            | WorktreeCommand::Status { .. } => Read,
            // fork creates an orchestrator-owned worktree (SL-056 PHASE-06) — the
            // first Orchestrator-classed verb; refused under worker-mode.
            WorktreeCommand::Fork { .. } => Orchestrator("fork"),
            // coordinate creates/resumes the orchestrator's OWN coordination
            // worktree (SL-064 §2) — markerless, but still an orchestrator funnel
            // operation; refused under worker-mode via the SAME guard as fork (EX-4).
            WorktreeCommand::Coordinate { .. } => Orchestrator("coordinate"),
            // import lands a worker delta into the coordination index (SL-056
            // PHASE-07) — Orchestrator-classed; refused under worker-mode.
            WorktreeCommand::Import { .. } => Orchestrator("import"),
            // land lands a solo fork onto the coordination branch via --no-ff merge
            // (SL-056 PHASE-08) — Orchestrator-classed; refused under worker-mode.
            WorktreeCommand::Land { .. } => Orchestrator("land"),
            // gc reaps a spent worktree fork once provably landed (SL-056 PHASE-09)
            // — Orchestrator-classed; refused under worker-mode.
            WorktreeCommand::Gc { .. } => Orchestrator("gc"),
            // marker --stamp-subagent is the claude spawn path's provision+mark
            // step (SL-056 PHASE-10) — Hookmint, refused under worker-mode (the
            // legit first stamp lands on a marker-absent worktree ⇒ allowed). All
            // other marker forms (--clear, bare) are the bespoke self-brick cure —
            // NOT refused by the worker-mode guard; their fences live in the handler.
            WorktreeCommand::Marker {
                stamp_subagent: true,
                ..
            } => Hookmint("marker --stamp-subagent"),
            WorktreeCommand::Marker { .. } => MarkerClear,
        },
        // dispatch sync projects coordination refs (SL-064 PHASE-04 / ADR-012
        // §4) — Orchestrator-classed across the whole verb class; refused under
        // worker-mode via the SAME guard as coordinate/fork (EX-1).
        Command::Dispatch { command } => match command {
            DispatchCommand::Sync { .. } => Orchestrator("dispatch-sync"),
            DispatchCommand::RecordBoundary { .. } => Orchestrator("dispatch-record-boundary"),
            DispatchCommand::Setup { .. } => Orchestrator("dispatch-setup"),
            // candidate create publishes coordination refs + ledger rows (SL-068
            // §5.3) — Orchestrator-classed like sync/record-boundary; refused
            // under worker-mode.
            DispatchCommand::Candidate { command } => match command {
                CandidateCommand::Create { .. } => Orchestrator("dispatch-candidate-create"),
                // candidate status is a read-only self-describing surface (SL-068
                // PHASE-04) — Read-classed so it works under worker-mode; it
                // mutates no ref and no ledger row.
                CandidateCommand::Status { .. } => Read,
                // candidate admit pins an immutable OID into candidates.toml
                // (SL-068 PHASE-05) — Orchestrator-classed like create; refused
                // under worker-mode.
                CandidateCommand::Admit { .. } => Orchestrator("dispatch-candidate-admit"),
            },
            // plan-next / status — read plan + phase sheets; never mutates a
            // ref or ledger row — Read-classed so it works under worker-mode.
            DispatchCommand::PlanNext { .. } | DispatchCommand::Status { .. } => Read,
        },
        // The coverage group splits per inner verb (SL-057 D2a): `show` is the
        // read-only drift view; `record`/`forget` mutate the observed store, and
        // `verify` re-derives + saves per slice — all authored writes.
        Command::Coverage { command } => match command {
            CoverageCommand::Show { .. } => Read,
            CoverageCommand::Record { .. } => Write("coverage record"),
            CoverageCommand::Verify { .. } => Write("coverage verify"),
            CoverageCommand::Forget { .. } => Write("coverage forget"),
        },
        // Read-only: the corpus integrity scan (INV-3), and the cross-kind relation
        // view (SL-046 — reads only, never mints/derives status).
        // Read-only priority surfaces (SL-047 — derive per query, never write /
        // mint / derive status; ADR-004 stores no reverse field).
        Command::Catalog { .. }
        | Command::Validate { .. }
        | Command::Inspect { .. }
        | Command::Survey { .. }
        | Command::Next { .. }
        | Command::Blockers { .. }
        | Command::Explain { .. }
        | Command::Status { .. } => Read,
        // Mutates the canonical-id triple — an authored write (D2/D6).
        Command::Reseat { .. } => Write("reseat"),
        // Author / remove a tier-1 `[[relation]]` edge — authored writes (SL-048 §5.4).
        Command::Link { .. } => Write("link"),
        Command::Unlink { .. } => Write("unlink"),
        // Author a dep/seq edge into `[relationships]` — authored writes (SL-060 §5.4).
        Command::Needs { .. } => Write("needs"),
        Command::After { .. } => Write("after"),
        // Record a supersession — writes NEW.supersedes, OLD.superseded_by, OLD.status
        // in one transaction (SL-062 §5.4).
        Command::Supersede { .. } => Write("supersede"),
    }
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

/// `doctrine inspect <ID> [--json]` — the COMMAND-LAYER composition (SL-047 §5.4 /
/// SL-046 D1). Renders the cross-kind relation view (via `relation_graph`) AND the
/// priority actionability block (via `priority`), then concatenates / injects: this
/// is the one layer allowed to depend on BOTH (ADR-001 — `relation_graph` sits below
/// `priority` and must never call up into it). The relation portion stays
/// byte-identical; the actionability block is purely additive (EX-2).
///
/// - **human**: the relation render with the actionability block appended below.
/// - **`--json`**: the inspect envelope with an additive `"actionability"` key —
///   the relation surfaces (`outbound`/`inbound`/`danglers`) unchanged.
fn run_inspect(path: Option<PathBuf>, id: &str, format: Format, json: bool) -> anyhow::Result<()> {
    use std::io::{self, Write};
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let resolved = if json { Format::Json } else { format };

    if let Ok(
        crate::memory::MemoryRef::Uid(_)
        | crate::memory::MemoryRef::UidPrefix(_)
        | crate::memory::MemoryRef::Key(_),
    ) = crate::memory::MemoryRef::parse(id)
    {
        let uid = crate::memory::resolve_inspect_uid(&root, id)?;
        let out = crate::memory::memory_inspect_view(&root, &uid, resolved)?;
        write!(std::io::stdout(), "{out}")?;
        return Ok(());
    }

    // SL-050 F2: ONE corpus scan shared by both consumers (was two — relation_graph and
    // priority each walked the corpus). Both `_from` entry points consume this slice;
    // the scan order is the same both saw (KINDS table / id ascending), preserving
    // REQ-077 determinism and the byte-identical relation/priority surfaces (VT-4).
    let mut diagnostics = Vec::new();
    let scanned = relation_graph::scan_entities(&root, &mut diagnostics)?;
    // Surface scan degradation diagnostics to stderr before normal output (D3).
    for diag in &diagnostics {
        writeln!(io::stderr(), "{}: {}", diag.file.display(), diag.message)?;
    }

    let out = match resolved {
        Format::Table => {
            // Relation render FIRST (the cheap oracle): its F6 existence gate (inside
            // render_from → inspect_from on the relation projection) errors a ghost id
            // BEFORE the heavier priority block is built.
            let relation = relation_graph::render_from(&scanned, &root, id, Format::Table)?;
            // Only reached for a minted id (the render gate passed).
            let block = priority::surface::actionability_block_from(&scanned, &root, id)?;
            let block = priority::render::actionability_block_human(&block);
            format!("{relation}{block}")
        }
        Format::Json => {
            // Relation view + gate FIRST, then the priority block (gate inside
            // inspect_from on the relation projection).
            let view = relation_graph::inspect_from(&scanned, &root, id)?;
            let block = priority::surface::actionability_block_from(&scanned, &root, id)?;
            let mut value = relation_graph::inspect_value(&view);
            if let Some(obj) = value.as_object_mut() {
                obj.insert(
                    "actionability".to_string(),
                    priority::render::actionability_block_value(&block),
                );
            }
            serde_json::to_string_pretty(&value)
                .map_err(|e| anyhow::anyhow!("failed to serialize inspect JSON: {e}"))?
        }
    };
    write!(std::io::stdout(), "{out}")?;
    Ok(())
}

/// Worker-mode guard (ADR-006 D2a / SL-056 §3): refuse a Write-classed verb when
/// the cwd tree resolves to worker mode (marker in a linked worktree OR the
/// `DOCTRINE_WORKER` env optimisation). Read / `MarkerClear` pass through. The
/// marker leg is evaluated LAZILY — only a Write verb resolves the root, so a Read
/// verb in a non-doctrine cwd never gains a new failure path (design §3).
fn worker_guard(cmd: &Command) -> anyhow::Result<()> {
    // Write and Orchestrator are both refused under worker-mode with the SAME
    // branches; Read and the bespoke MarkerClear pass through (SL-056 PHASE-06).
    let verb = match write_class(cmd) {
        WriteClass::Write(verb) | WriteClass::Orchestrator(verb) | WriteClass::Hookmint(verb) => {
            verb
        }
        WriteClass::Read | WriteClass::MarkerClear => return Ok(()),
    };
    // No doctrine/project root above the cwd: the marker leg cannot apply. Fall
    // back to the env leg alone (a leaked env on a rootless cwd), never a new error.
    let Ok(root) = root::find(None, &root::default_markers()) else {
        if worktree::env_worker_set() {
            anyhow::bail!("{}: refusing authored write `{verb}`", worktree::DUAL_CAUSE);
        }
        return Ok(());
    };
    let mode = worktree::resolve_mode(&root);
    if !mode.refused {
        return Ok(());
    }
    // The env leg on a NON-linked tree carries the NAMED dual-cause message (never
    // a bare "worker refused"); the marker / linked-fork legs name the verb plainly.
    if mode.is_env_on_nonlinked() {
        anyhow::bail!("{}: refusing authored write `{verb}`", worktree::DUAL_CAUSE);
    }
    anyhow::bail!(
        "worker fork (signal: {}): refusing authored write `{verb}` — workers return a source delta; doctrine-mediated writes funnel through the orchestrator.",
        mode.cause_token()
    );
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let color = crate::tty::resolve_color(cli.color);

    // ADR-006 D2a / SL-056 §3 worker-mode guard: a dispatched worker mints/anchors
    // nothing. Bail before dispatch on any Write-classed verb; Read / MarkerClear
    // paths stay open (INV-3 / the self-brick carve-out).
    worker_guard(&cli.command)?;

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
            } => memory::run_show(path, &reference, if json { Format::Json } else { format }),
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
            } => memory::run_list(path, memory_type, list.into_list_args(color)),
            MemoryCommand::Find { query, args } => {
                // Merge positional query + --query; mutually exclusive.
                let free_query = match (query, args.flag_query) {
                    (Some(_), Some(_)) => {
                        anyhow::bail!("cannot specify both a positional query and --query")
                    }
                    (q, None) | (None, q) => q,
                };
                // Validate --limit.
                if args.limit == Some(0) {
                    anyhow::bail!("--limit must be >= 1");
                }
                // Resolve offset: page sugar or explicit.
                let page_size = args.limit.unwrap_or(retrieve::RETRIEVE_LIMIT_DEFAULT);
                let offset = match args.page {
                    Some(0) => anyhow::bail!("--page must be >= 1"),
                    Some(p) => (p - 1) * page_size,
                    None => args.offset,
                };
                let resolved_format = if args.json { Format::Json } else { args.format };
                retrieve::run_find(
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
                // Validate --limit.
                if args.limit == Some(0) {
                    anyhow::bail!("--limit must be >= 1");
                }
                let retrieve_limit = args
                    .limit
                    .unwrap_or(retrieve::RETRIEVE_LIMIT_DEFAULT)
                    .min(retrieve::RETRIEVE_LIMIT_MAX);
                // Resolve offset: page sugar or explicit.
                let page_size = args.limit.unwrap_or(retrieve::RETRIEVE_LIMIT_DEFAULT);
                let offset = match args.page {
                    Some(0) => anyhow::bail!("--page must be >= 1"),
                    Some(p) => (p - 1) * page_size,
                    None => args.offset,
                };
                let resolved_format = if args.json { Format::Json } else { args.format };
                retrieve::run_retrieve(
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
            } => review::run_new(
                path,
                &review::NewArgs {
                    facet,
                    target,
                    phase,
                    title,
                    raiser,
                    responder,
                },
            ),
            ReviewCommand::List { list, path } => {
                review::run_list(path, list.into_list_args(color))
            }
            ReviewCommand::Show {
                reference,
                format,
                json,
                path,
            } => review::run_show(path, &reference, if json { Format::Json } else { format }),
            ReviewCommand::Raise {
                reference,
                severity,
                title,
                detail,
                role,
                path,
            } => {
                let role = review::parse_role(role.as_deref(), review::Role::Raiser)?;
                review::run_raise(
                    path,
                    &review::RaiseArgs {
                        reference,
                        severity,
                        title,
                        detail,
                    },
                    role,
                )
            }
            ReviewCommand::Dispose {
                reference,
                finding,
                disposition,
                response,
                role,
                path,
            } => {
                let role = review::parse_role(role.as_deref(), review::Role::Responder)?;
                review::run_dispose(
                    path,
                    &review::DisposeArgs {
                        reference,
                        finding,
                        disposition,
                        response,
                    },
                    role,
                )
            }
            ReviewCommand::Verify {
                reference,
                finding,
                note,
                role,
                path,
            } => {
                let role = review::parse_role(role.as_deref(), review::Role::Raiser)?;
                review::run_verify(path, &reference, &finding, note.as_deref(), role)
            }
            ReviewCommand::Contest {
                reference,
                finding,
                note,
                role,
                path,
            } => {
                let role = review::parse_role(role.as_deref(), review::Role::Raiser)?;
                review::run_contest(path, &reference, &finding, note.as_deref(), role)
            }
            ReviewCommand::Withdraw {
                reference,
                finding,
                role,
                path,
            } => {
                let role = review::parse_role(role.as_deref(), review::Role::Raiser)?;
                review::run_withdraw(path, &reference, &finding, role)
            }
            ReviewCommand::Status { reference, path } => review::run_status(path, &reference),
            ReviewCommand::Prime {
                reference,
                seed,
                from,
                path,
            } => review::run_prime(
                path,
                &review::PrimeArgs {
                    reference,
                    seed,
                    from,
                },
            ),
            ReviewCommand::Unlock { reference, path } => review::run_unlock(path, &reference),
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
            RevisionCommand::New { title, slug, path } => revision::run_new(path, title, slug),
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
        } => run_inspect(path, &id, format, json),
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
            WorktreeCommand::VerifyWorker { base, dir } => worktree::run_verify_worker(&base, &dir),
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
                trunk,
                edge,
                path,
                ..
            } => {
                // The `stage` group is `required = true` single-choice: exactly one
                // of `--prepare-review` / `--integrate` is set, so `integrate`
                // alone selects the stage (no unreachable arm needed).
                if integrate {
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
            DispatchCommand::Setup { slice, dir, path } => dispatch::run_setup(path, slice, &dir),
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
        },
        Command::Validate { path } => run_validate(path),
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
        } => run_link(path, &source, &label, &target),
        Command::Unlink {
            source,
            label,
            target,
            path,
        } => run_unlink(path, &source, &label, &target),
        Command::Needs {
            source,
            target,
            path,
        } => run_needs_edge(path, &source, &target),
        Command::After {
            source,
            target,
            rank,
            remove,
            prune,
            path,
        } => {
            if prune {
                run_after_prune(path, &source)
            } else if remove {
                // target is guaranteed Some by clap (required_unless_present="prune")
                run_after_remove(path, &source, target.as_deref().unwrap_or(""), rank)
            } else {
                run_after_edge(path, &source, target.as_deref().unwrap_or(""), rank)
            }
        }
        Command::Status { format, json, path } => status::run(path, format, json),
        Command::Supersede { new, old, path } => run_supersede(path, &new, &old),
        Command::Map { command } => match command {
            MapCommand::Serve(args) => commands::map::run_serve(None, args),
        },
    }
}

/// Resolve a `link`/`unlink` source+label to (the source entity's toml path, the
/// validated label). Shared by both verbs (design §5.4): parse the source ref →
/// `(KindRef, id)`; `relation::validate_link` (the `(source, label)` legality +
/// `link`-writability gate); compute the entity's `<stem>-NNN.toml` path. Target
/// validation is link-only (a dangling target must still be `unlink`-able), so it lives
/// in `run_link`, not here.
fn resolve_link_path(
    root: &std::path::Path,
    source: &str,
    label: &str,
) -> anyhow::Result<(PathBuf, &'static relation::RelationRule)> {
    let (kref, id) = integrity::parse_canonical_ref(source)?;
    let rule = relation::validate_link(kref.kind, label)?;
    let name = format!("{id:03}");
    let toml_path = root
        .join(kref.kind.dir)
        .join(&name)
        .join(format!("{}-{name}.toml", kref.stem));
    Ok((toml_path, rule))
}

/// `doctrine link <SOURCE-ID> <LABEL> <TARGET>` (SL-048 §5.4) — author a tier-1
/// `[[relation]]` edge. Validates the source/label ([`resolve_link_path`]) then the
/// forward target (§5.5 — `Unvalidated` `drift` is free text; every other label's
/// target must BOTH resolve (`ensure_ref_resolves` — never write a dangler) AND pass
/// the legal-KIND assertion), then appends edit-preservingly. Idempotent (a re-link
/// reports `already linked`, file untouched).
fn run_link(path: Option<PathBuf>, source: &str, label: &str, target: &str) -> anyhow::Result<()> {
    use anyhow::Context;
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;

    // Memory branch — detect mem_<uid> / mem.<key> / mem_<prefix> sources
    // and route to memory.toml relations (SL-090 §PHASE-03).
    if let Ok(mref) = memory::MemoryRef::parse(source) {
        let toml_path = memory::resolve_memory_toml_path(&root, &mref)?;
        // Best-effort target validation: if target looks like a canonical ref,
        // validate it resolves. Free-text and mem_* targets pass through.
        if integrity::parse_canonical_ref(target).is_ok() {
            integrity::ensure_ref_resolves(&root, target).with_context(|| {
                format!("target `{target}` does not resolve to an existing entity")
            })?;
        }
        let outcome = memory::append_memory_relation(&toml_path, label, target)?;
        match outcome {
            relation::AppendOutcome::Wrote => {
                writeln!(std::io::stdout(), "linked: {source} {label} {target}")?;
            }
            relation::AppendOutcome::Noop => {
                writeln!(
                    std::io::stdout(),
                    "already linked: {source} {label} {target}"
                )?;
            }
        }
        return Ok(());
    }

    let (toml_path, rule) = resolve_link_path(&root, source, label)?;
    // Forward-edge validation (§5.5): free-text labels skip both gates; validated
    // labels must resolve AND be of a legal target kind.
    if !matches!(rule.target, relation::TargetSpec::Unvalidated) {
        integrity::ensure_ref_resolves(&root, target)?;
        let (tkref, _tid) = integrity::parse_canonical_ref(target)?;
        let (skref, _sid) = integrity::parse_canonical_ref(source)?;
        relation::check_target_kind(rule, skref.kind, tkref.kind.prefix)?;
    }
    let outcome = relation::append_edge(&toml_path, rule.label, target)?;
    match outcome {
        relation::AppendOutcome::Wrote => {
            writeln!(std::io::stdout(), "linked: {source} {label} {target}")?;
        }
        relation::AppendOutcome::Noop => {
            writeln!(
                std::io::stdout(),
                "already linked: {source} {label} {target}"
            )?;
        }
    }
    Ok(())
}

/// `doctrine unlink <SOURCE-ID> <LABEL> <TARGET>` (SL-048 §5.4) — remove a tier-1
/// `[[relation]]` edge. Same validation pipeline (the source/label must still be legal
/// to name the right file); idempotent (an absent edge reports `not linked`).
fn run_unlink(
    path: Option<PathBuf>,
    source: &str,
    label: &str,
    target: &str,
) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;

    // Memory branch — detect mem_<uid> / mem.<key> / mem_<prefix> sources
    // and route to memory.toml relations (SL-090 §PHASE-03).
    if let Ok(mref) = memory::MemoryRef::parse(source) {
        let toml_path = memory::resolve_memory_toml_path(&root, &mref)?;
        // No target validation for unlink (matching existing behaviour for numbered entities).
        let outcome = memory::remove_memory_relation(&toml_path, label, target)?;
        match outcome {
            relation::RemoveOutcome::Removed => {
                writeln!(std::io::stdout(), "unlinked: {source} {label} {target}")?;
            }
            relation::RemoveOutcome::Absent => {
                writeln!(std::io::stdout(), "not linked: {source} {label} {target}")?;
            }
        }
        return Ok(());
    }

    let (toml_path, rule) = resolve_link_path(&root, source, label)?;
    let outcome = relation::remove_edge(&toml_path, rule.label, target)?;
    match outcome {
        relation::RemoveOutcome::Removed => {
            writeln!(std::io::stdout(), "unlinked: {source} {label} {target}")?;
        }
        relation::RemoveOutcome::Absent => {
            writeln!(std::io::stdout(), "not linked: {source} {label} {target}")?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Generic dep/seq verbs: `doctrine needs` / `doctrine after` (SL-060 §5.4)
// ---------------------------------------------------------------------------

/// The work-like membership predicate (SL-060 §5.4, SL-066 §PHASE-04) — the ONE
/// widen-later guard. Work-like = { slice } ∪ { the 5 backlog kinds } ∪ { revision }.
/// Both the dep/seq-authoring SRC set and the admissible TGT set are this same
/// membership (a slice / backlog item / revision may author dep/seq, and may only
/// depend/sequence on another piece of work). REV is admitted as BOTH source and
/// target: a slice or backlog item may `needs`/`after` a REV-NNN, and a REV may
/// itself `needs`/`after` a work item (the IDE-010 payoff). Governance docs
/// (spec/ADR/POL/STD) stay EXCLUDED — depending on governance routes THROUGH a
/// Revision, never the evergreen doc (the SL-060 invariant). A future phase that
/// allows cross-tier dep/seq deletes just this predicate (and its refusal tests).
fn is_work_like(kind: &'static entity::Kind) -> bool {
    matches!(
        kind.prefix,
        "SL" | "ISS" | "IMP" | "CHR" | "RSK" | "IDE" | "REV"
    )
}

/// Resolve a dep/seq source to its TOML path. Validates: canonical-ref parse,
/// work-like kind (slice or backlog). Returns the resolved path.
fn resolve_dep_seq_src_path(root: &std::path::Path, source: &str) -> anyhow::Result<PathBuf> {
    let (skref, sid) = integrity::parse_canonical_ref(source)?;
    anyhow::ensure!(
        is_work_like(skref.kind),
        "`{source}` is a {} entity, which cannot author needs/after — only a slice or a backlog item (issue/improvement/chore/risk/idea) carries dep/seq",
        skref.kind.prefix
    );
    let name = format!("{sid:03}");
    Ok(root
        .join(skref.kind.dir)
        .join(&name)
        .join(format!("{}-{name}.toml", skref.stem)))
}

/// Resolve a generic dep/seq `(SRC, TGT)` pair against the author-time gate (§5.4),
/// returning SRC's `slice-NNN.toml`-shaped path ready for the leaf write. Rides the
/// SAME cross-kind canonical-ref seam as `link` (`integrity::parse_canonical_ref` +
/// the `KindRef` `(dir, stem)` path map) — no new resolver. The three refusals, each
/// a clear, specific message:
///   1. SRC must resolve AND be a dep/seq-authoring (work-like) kind.
///   2. TGT must resolve on disk (free-text / dangling refused) AND be work-like.
///   3. self-edge (SRC == TGT) refused.
fn resolve_dep_seq_src(
    root: &std::path::Path,
    source: &str,
    target: &str,
) -> anyhow::Result<PathBuf> {
    let toml_path = resolve_dep_seq_src_path(root, source)?;
    let (skref, sid) = integrity::parse_canonical_ref(source)?;
    // TGT must resolve on disk — a free-text or dangling target is refused here
    // (never write an edge to a non-entity). `parse_canonical_ref` first so a
    // free-text target surfaces the canonical-ref shape error, then a dir probe.
    let (tkref, tid) = integrity::parse_canonical_ref(target)?;
    integrity::ensure_ref_resolves(root, target)?;
    anyhow::ensure!(
        is_work_like(tkref.kind),
        "`{target}` is a {} entity — needs/after may only target work (a slice or a backlog item); cross-tier dep/seq is not yet allowed",
        tkref.kind.prefix
    );
    anyhow::ensure!(
        !(skref.kind.prefix == tkref.kind.prefix && sid == tid),
        "a {source} edge to itself is not a dependency — self-edges are refused"
    );
    Ok(toml_path)
}

/// `doctrine needs <SRC> <TGT>` (SL-060 §5.4) — append TGT to SRC's `needs` axis.
/// Generic cross-kind: the author-time work-like gate ([`resolve_dep_seq_src`]) then
/// the shared leaf `dep_seq::append`. NO author-time cycle check (deferred to read
/// time by design — the cross-kind cycle oracle is a later phase).
fn run_needs_edge(path: Option<PathBuf>, source: &str, target: &str) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let toml_path = resolve_dep_seq_src(&root, source, target)?;
    dep_seq::append(&toml_path, &dep_seq::RelEdit::Needs(&[target.to_string()]))?;
    writeln!(std::io::stdout(), "{source} needs {target}")?;
    Ok(())
}

/// `doctrine after <SRC> <TGT> [--rank N]` (SL-060 §5.4) — append `{ to, rank }` to
/// SRC's `after` axis through the same gate + leaf. Rank default 0.
fn run_after_edge(
    path: Option<PathBuf>,
    source: &str,
    target: &str,
    rank: i32,
) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let toml_path = resolve_dep_seq_src(&root, source, target)?;
    dep_seq::append(&toml_path, &dep_seq::RelEdit::After { to: target, rank })?;
    let suffix = if rank == 0 {
        String::new()
    } else {
        format!(" (rank {rank})")
    };
    writeln!(std::io::stdout(), "{source} after {target}{suffix}")?;
    Ok(())
}

/// `doctrine after <SRC> <TGT> --remove [--rank N]`
fn run_after_remove(
    path: Option<PathBuf>,
    source: &str,
    target: &str,
    rank: i32,
) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let toml_path = resolve_dep_seq_src(&root, source, target)?;
    let ceiling = if rank == 0 { None } else { Some(rank) };
    let removed = dep_seq::remove(&toml_path, target, ceiling)?;
    if removed == 0 {
        anyhow::bail!("{source} has no after edge to {target}");
    }
    writeln!(
        std::io::stdout(),
        "{source} after {target} removed ({} edge{})",
        removed,
        if removed == 1 { "" } else { "s" }
    )?;
    Ok(())
}

/// `doctrine after <SRC> --prune` (SL-105 PHASE-03) — probe every `after` target
/// of SRC for dangling edges (absent or terminal target) and remove them. Reads
/// the `DepSeq` ONCE before any modifications (collecting dangling targets), then
/// removes in a second pass using the shared `dep_seq::remove` leaf.
fn run_after_prune(path: Option<PathBuf>, source: &str) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let toml_path = resolve_dep_seq_src_path(&root, source)?;

    // 1. Read DepSeq
    let ds = dep_seq::read(&toml_path)?;

    // 2. Probe each after-edge target: absent (dir missing) or terminal (resolved/closed) → dangling
    let mut dropped: Vec<(String, i32, String)> = Vec::new();
    let mut to_drop: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for edge in &ds.after {
        let is_dangling = match integrity::parse_canonical_ref(&edge.to) {
            Ok((kref, tid)) => {
                let target_path = root
                    .join(kref.kind.dir)
                    .join(format!("{tid:03}"))
                    .join(format!("{}-{tid:03}.toml", kref.stem));
                if target_path.exists() {
                    let body = std::fs::read_to_string(&target_path).unwrap_or_default();
                    let val: toml::Value = match toml::from_str(&body) {
                        Ok(v) => v,
                        Err(_) => toml::Value::Table(toml::Table::new()),
                    };
                    let status = val.get("status").and_then(|s| s.as_str()).unwrap_or("");
                    status == "resolved" || status == "closed"
                } else {
                    true
                }
            }
            Err(_) => true,
        };

        if is_dangling {
            let reason = match integrity::parse_canonical_ref(&edge.to) {
                Ok((kref2, tid2)) => {
                    let target_path = root
                        .join(kref2.kind.dir)
                        .join(format!("{tid2:03}"))
                        .join(format!("{}-{tid2:03}.toml", kref2.stem));
                    if target_path.exists() {
                        let body = std::fs::read_to_string(&target_path).unwrap_or_default();
                        let val: toml::Value = match toml::from_str(&body) {
                            Ok(v) => v,
                            Err(_) => toml::Value::Table(toml::Table::new()),
                        };
                        let status = val.get("status").and_then(|s| s.as_str()).unwrap_or("");
                        let resolution = val.get("resolution").and_then(|s| s.as_str()).unwrap_or("");
                        if resolution.is_empty() {
                            status.to_string()
                        } else {
                            format!("{status}/{resolution}")
                        }
                    } else {
                        "absent".to_string()
                    }
                }
                Err(_) => "absent (unparseable ref)".to_string(),
            };
            dropped.push((edge.to.clone(), edge.rank, reason));
            to_drop.insert(edge.to.clone());
        }
    }

    if dropped.is_empty() {
        writeln!(std::io::stdout(), "{source}: nothing to prune")?;
        return Ok(());
    }

    // 3. Remove all edges per unique dangling target (one pass each) via shared leaf
    for target in &to_drop {
        // `None` ceiling → remove every edge matching the target wildcard
        let _ = dep_seq::remove(&toml_path, target, None)?;
    }

    // 4. Report dropped edges
    for (target, rank, reason) in &dropped {
        writeln!(
            std::io::stdout(),
            "{source} after {target} (rank {rank}) dropped (dangling: {reason})"
        )?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Supersession verb: `doctrine supersede` (SL-062 §5.4)
// ---------------------------------------------------------------------------

/// Resolve a supersession ref to its `<stem>-NNN.toml` path plus the canonical ref
/// string. Mirrors `resolve_link_path` (the same `KindRef` `(dir, stem)` path map),
/// but returns the normalised canonical id (`ADR-004`) — the exact string form the
/// `supersedes`/`superseded_by` arrays store (matching `validate`'s derived side,
/// which keys on `listing::canonical_id`).
fn resolve_supersede_path(
    root: &std::path::Path,
    kref: &integrity::KindRef,
    id: u32,
) -> (PathBuf, String) {
    let name = format!("{id:03}");
    let toml_path = root
        .join(kref.kind.dir)
        .join(&name)
        .join(format!("{}-{name}.toml", kref.stem));
    (toml_path, listing::canonical_id(kref.kind.prefix, id))
}

/// `doctrine supersede <NEW> <OLD>` (SL-062 §5.4) — the transactional, ADR-first
/// supersession verb. One parse-once / hold-both / write-once transaction composing
/// Cross-kind supersession for records + governance docs (`ADR` stays same-kind;
/// the four record kinds `ASM`/`DEC`/`QUE`/`CON` ride the §6 matrix). Composes
/// the PHASE-02 pure cores (`dep_seq::apply_string_append` + `dep_seq::apply_status`)
/// over docs parsed once and held in scope: `NEW.supersedes += OLD`,
/// `OLD.superseded_by += NEW` (the single sanctioned reverse carve-out, ADR-004 §5),
/// and flips `OLD.status → superseded`.
///
/// Pre-flight (NO write): refuse a self-edge, cross-kind refs, a non-ADR (no
/// `supersede_policy`) NEW; then parse BOTH docs and verify every touched key/array
/// is scaffold-present (F-1, non-destructive). The not-already-superseded guard (F-D)
/// allows ONLY the idempotent re-run (BOTH files already reciprocal); a different
/// supersessor or hand-drifted carve-out is refused. Writes NEW then OLD — the order
/// that makes a torn state (`NEW.supersedes∋OLD` without the reciprocal) detectable
/// by `doctrine validate`.
fn run_supersede(path: Option<PathBuf>, new: &str, old: &str) -> anyhow::Result<()> {
    use anyhow::Context;
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;

    // Pre-flight resolution + capability gate (NO write).
    let (new_kref, new_id) = integrity::parse_canonical_ref(new)?;
    let (old_kref, old_id) = integrity::parse_canonical_ref(old)?;
    anyhow::ensure!(
        !(new_kref.kind.prefix == old_kref.kind.prefix && new_id == old_id),
        "`{new}` cannot supersede itself — a self-supersession is not a decision change"
    );
    // Cross-kind gating: ADR → same-kind only; records → matrix; mixed → refuse.
    // The old same-kind guard is retained for non-ADR, non-record pairs (e.g. SL→SL).
    let new_is_adr = new_kref.kind.prefix == "ADR";
    let old_is_adr = old_kref.kind.prefix == "ADR";
    let new_is_record = crate::knowledge::RecordKind::from_prefix(new_kref.kind.prefix).is_some();
    let old_is_record = crate::knowledge::RecordKind::from_prefix(old_kref.kind.prefix).is_some();
    let same_family = if new_is_adr && old_is_adr {
        true // ADR family
    } else if new_is_record && old_is_record {
        // Both records: validate matrix. from_prefix already proved Some by the
        // is_some() gate, but each arm needs a non-panicking fallback for clippy.
        let Some(new_record_kind) = crate::knowledge::RecordKind::from_prefix(new_kref.kind.prefix)
        else {
            anyhow::bail!("NEW kind not a valid record kind")
        };
        let Some(old_record_kind) = crate::knowledge::RecordKind::from_prefix(old_kref.kind.prefix)
        else {
            anyhow::bail!("OLD kind not a valid record kind")
        };
        anyhow::ensure!(
            crate::supersede::validate_matrix(new_record_kind, old_record_kind),
            "cross-kind supersession refused: the §6 matrix disallows {} → {}",
            new_kref.kind.prefix,
            old_kref.kind.prefix
        );
        true // record family: matrix passed
    } else if new_kref.kind.prefix == old_kref.kind.prefix {
        true // same kind (e.g. SL→SL); fall through to supersede_policy "not yet supported"
    } else {
        false // cross-family or cross-kind
    };
    anyhow::ensure!(
        same_family,
        "cross-family supersession refused: `{new}` is a {} but `{old}` is a {}",
        new_kref.kind.prefix,
        old_kref.kind.prefix
    );
    let policy = crate::supersede::supersede_policy(new_kref.kind).with_context(|| {
        format!(
            "supersession not yet supported for {} (follow-up F2)",
            new_kref.kind.prefix
        )
    })?;

    // For cross-kind record supersession, OLD status should be based on OLD kind policy
    let old_policy = if !new_is_adr && !old_is_adr && new_kref.kind.prefix != old_kref.kind.prefix {
        crate::supersede::supersede_policy(old_kref.kind).with_context(|| {
            format!(
                "supersession not yet supported for OLD {} (follow-up F2)",
                old_kref.kind.prefix
            )
        })?
    } else {
        policy
    };

    let (new_path, new_ref) = resolve_supersede_path(&root, new_kref, new_id);
    let (old_path, old_ref) = resolve_supersede_path(&root, old_kref, old_id);

    // Parse BOTH docs ONCE and HOLD them in scope (parse-once / hold-both).
    let new_text = std::fs::read_to_string(&new_path)
        .with_context(|| format!("supersede: {new} not found at {}", new_path.display()))?;
    let mut new_doc = new_text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", new_path.display()))?;
    let old_text = std::fs::read_to_string(&old_path)
        .with_context(|| format!("supersede: {old} not found at {}", old_path.display()))?;
    let mut old_doc = old_text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", old_path.display()))?;

    // F-1 pre-flight on OLD's typed carve-out (always typed, both paths).
    let old_carveout = rel_array(&old_doc, policy.carveout_field);
    anyhow::ensure!(
        old_carveout.is_some(),
        "malformed `{old}` at {}: missing seeded `[relationships].{}` array — restore the seeded `[relationships]` arrays before superseding; the file is left untouched",
        old_path.display(),
        policy.carveout_field
    );
    anyhow::ensure!(
        old_doc
            .get("status")
            .and_then(toml_edit::Item::as_str)
            .is_some(),
        "malformed `{old}` at {}: missing seeded top-level `status` — restore the seeded keys before superseding; the file is left untouched",
        old_path.display()
    );
    anyhow::ensure!(
        old_doc
            .get("updated")
            .and_then(toml_edit::Item::as_str)
            .is_some(),
        "malformed `{old}` at {}: missing seeded top-level `updated` — restore the seeded keys before superseding; the file is left untouched",
        old_path.display()
    );

    let old_status = old_doc
        .get("status")
        .and_then(toml_edit::Item::as_str)
        .unwrap_or_default()
        .to_string();

    // Dispatch write path on storage discriminant (SL-095 D7).
    match policy.storage {
        crate::supersede::StorageTarget::RelationRow => {
            use crate::relation::{self, RelationLabel};

            // F-1 pre-flight: read [[relation]] rows for Supersedes on NEW.
            let relation_doc = relation::RelationDoc::parse(&new_text)
                .with_context(|| {
                    format!(
                        "malformed `{new}` at {}: missing seeded `[[relation]]` table — restore the seeded template; the file is left untouched",
                        new_path.display()
                    )
                })?;
            let (edges, _illegal) = relation::read_block(new_kref.kind, &relation_doc);
            let existing_supersedes: Vec<_> = edges
                .iter()
                .filter(|e| e.label == RelationLabel::Supersedes)
                .collect();

            // F-D not-already-superseded guard ([[relation]] path).
            if old_status == policy.superseded_status {
                let carveout = old_carveout.unwrap_or_default();
                let new_lists_old = existing_supersedes.iter().any(|e| e.target == old_ref);
                let single_self = carveout.len() == 1 && carveout.first() == Some(&new_ref);
                if single_self && new_lists_old {
                    writeln!(
                        std::io::stdout(),
                        "already recorded: {new} supersedes {old}"
                    )?;
                    return Ok(());
                }
                if let Some(other) = carveout.iter().find(|x| **x != new_ref) {
                    anyhow::bail!("{old} already superseded by {other}; reopening is deferred");
                }
                anyhow::bail!(
                    "{old} status is superseded but its superseded_by carve-out is empty/inconsistent — run `doctrine validate`"
                );
            }
            // F-1: NEW must not already supersede a different entity.
            if let Some(edge) = existing_supersedes.first() {
                anyhow::bail!("{new} already supersedes {}", edge.target);
            }

            // Write NEW's outbound edge via [[relation]].
            let outcome = relation::append_edge(&new_path, RelationLabel::Supersedes, &old_ref)?;
            if matches!(outcome, relation::AppendOutcome::Noop) {
                writeln!(
                    std::io::stdout(),
                    "already recorded: {new} supersedes {old}"
                )?;
                // Still write OLD's carved-out + status (typed, below).
            } else {
                writeln!(std::io::stdout(), "{new} supersedes {old}")?;
            }

            // OLD: typed carved-out + status flip (unchanged).
            let today = clock::today();
            let status_hint = format!(
                "malformed `{old}`: missing seeded top-level `status`/`updated` — restore the seeded keys; the file is left untouched"
            );
            dep_seq::apply_string_append(&mut old_doc, policy.carveout_field, &new_ref)?;
            dep_seq::apply_status(
                &mut old_doc,
                &[("status", policy.superseded_status), ("updated", &today)],
                &status_hint,
            )?;
            std::fs::write(&old_path, old_doc.to_string())
                .with_context(|| format!("Failed to write {}", old_path.display()))?;
        }
        crate::supersede::StorageTarget::TypedArray { field } => {
            // F-1 pre-flight: typed outbound array must be scaffold-present.
            let new_sup = rel_array(&new_doc, field);
            anyhow::ensure!(
                new_sup.is_some(),
                "malformed `{new}` at {}: missing seeded `[relationships].{}` array — restore the seeded `[relationships]` arrays before superseding; the file is left untouched",
                new_path.display(),
                field
            );

            // F-D not-already-superseded guard (typed path, existing).
            if old_status == old_policy.superseded_status {
                let carveout = old_carveout.unwrap_or_default();
                let new_lists_old = new_sup.unwrap_or_default().contains(&old_ref);
                let single_self = carveout.len() == 1 && carveout.first() == Some(&new_ref);
                if single_self && new_lists_old {
                    writeln!(
                        std::io::stdout(),
                        "already recorded: {new} supersedes {old}"
                    )?;
                    return Ok(());
                }
                if let Some(other) = carveout.iter().find(|x| **x != new_ref) {
                    anyhow::bail!("{old} already superseded by {other}; reopening is deferred");
                }
                anyhow::bail!(
                    "{old} status is superseded but its superseded_by carve-out is empty/inconsistent — run `doctrine validate`"
                );
            }

            // Mutate the held docs (no IO) and write.
            let today = clock::today();
            let status_hint = format!(
                "malformed `{old}`: missing seeded top-level `status`/`updated` — restore the seeded keys; the file is left untouched"
            );
            dep_seq::apply_string_append(&mut new_doc, field, &old_ref)?;
            dep_seq::apply_string_append(&mut old_doc, policy.carveout_field, &new_ref)?;

            // Conditional status flip: skip if OLD record is already terminal (SL-097 D2).
            let old_record_kind = crate::knowledge::RecordKind::from_prefix(old_kref.kind.prefix)
                .context("OLD kind not a valid record kind")?;
            if old_record_kind.is_terminal(&old_status) {
                // Already terminal: skip status flip, update timestamp only.
                old_doc
                    .as_table_mut()
                    .insert("updated", toml_edit::value(today.as_str()));
            } else {
                dep_seq::apply_status(
                    &mut old_doc,
                    &[
                        ("status", old_policy.superseded_status),
                        ("updated", &today),
                    ],
                    &status_hint,
                )?;
            }

            // Write each file ONCE, NEW then OLD.
            std::fs::write(&new_path, new_doc.to_string())
                .with_context(|| format!("Failed to write {}", new_path.display()))?;
            std::fs::write(&old_path, old_doc.to_string())
                .with_context(|| format!("Failed to write {}", old_path.display()))?;

            writeln!(std::io::stdout(), "{new} supersedes {old}")?;
        }
    }
    Ok(())
}

/// Read a `[relationships].<field>` array's string elements off a held doc (pre-flight
/// presence probe + membership reads). `None` iff the seeded array is absent (F-1).
fn rel_array(doc: &toml_edit::DocumentMut, field: &str) -> Option<Vec<String>> {
    doc.get("relationships")
        .and_then(toml_edit::Item::as_table)
        .and_then(|t| t.get(field))
        .and_then(toml_edit::Item::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
}

/// `doctrine validate` — the corpus integrity scan. The COMMAND-LAYER composition
/// (mirrors `run_inspect`, ADR-001): the one layer allowed to depend on BOTH the
/// `integrity` id-scan AND the `relation_graph` relation-edge walk (which depends back
/// on `integrity` — composing them here keeps that edge acyclic). Resolves the root
/// ONCE, concatenates the id-integrity findings (D3 detect-half) with the SL-048
/// relation findings (danglers, `IllegalRows`, supersession drift — §5.5), prints them,
/// and exits non-zero on any. All report-only; nothing is rewritten (the reseat
/// precedent).
fn run_validate(path: Option<PathBuf>) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;

    let mut findings = integrity::id_integrity_findings(&root)?;
    findings.extend(relation_graph::validate_relations(&root)?);

    writeln!(
        std::io::stdout(),
        "validate: scanned {}",
        integrity::scanned_kinds()
    )?;
    if findings.is_empty() {
        writeln!(std::io::stdout(), "validate: corpus clean")?;
        return Ok(());
    }
    for f in &findings {
        writeln!(std::io::stdout(), "  {f}")?;
    }
    anyhow::bail!("validate: {} finding(s)", findings.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    // SL-060 / SL-066 §PHASE-04: the work-like membership predicate is the ONE
    // widen-later guard — exactly { slice } ∪ { the 5 backlog kinds } ∪ { revision },
    // every other admitted kind refused. REV joins as both dep/seq source and target
    // (the IDE-010 payoff); governance docs stay off the allowlist (SL-060 invariant).
    #[test]
    fn is_work_like_is_exactly_slice_plus_backlog_plus_revision() {
        // The work-like set: slice + the five backlog kinds + revision.
        assert!(is_work_like(&slice::SLICE_KIND));
        for k in integrity::KINDS
            .iter()
            .filter(|k| matches!(k.kind.prefix, "ISS" | "IMP" | "CHR" | "RSK" | "IDE" | "REV"))
        {
            assert!(is_work_like(k.kind), "{} is work-like", k.kind.prefix);
        }
        // Every OTHER admitted kind in the corpus table is refused (gov / spec / req /
        // review / reconciliation / knowledge) — the closed allowlist.
        for k in integrity::KINDS.iter().filter(|k| {
            !matches!(
                k.kind.prefix,
                "SL" | "ISS" | "IMP" | "CHR" | "RSK" | "IDE" | "REV"
            )
        }) {
            assert!(
                !is_work_like(k.kind),
                "{} must NOT be work-like (off the allowlist)",
                k.kind.prefix
            );
        }
    }

    // VT-4: `--only-memory` is declared `conflicts_with_all = ["skill", "domain"]`,
    // so clap rejects it at parse time alongside an explicit selector. `try_parse_from`
    // returns the error rather than exiting the process.
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

    /// CHR-008 (SL-078): `run_supersede` writes NEW then OLD — a crash between
    /// writes leaves a torn state (NEW.supersedes ∋ OLD without the reciprocal).
    /// Re-running the same command naturally completes recovery through the
    /// existing flow: F-1 passes, F-D skips (OLD.status ≠ superseded),
    /// push_str_if_absent on NEW is a no-op, push_str_if_absent on OLD writes
    /// the missing entry, and the status flip completes the transaction.
    #[test]
    fn supersede_recovery_from_torn_new_only_state() {
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // Seed ADR-001 (NEW): supersedes = ["ADR-002"], superseded_by = [].
        catalog::test_helpers::write(
            root,
            ".doctrine/adr/001/adr-001.toml",
            "id = 1\nslug = \"a1\"\ntitle = \"A1\"\nstatus = \"accepted\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = [\"ADR-002\"]\nsuperseded_by = []\n",
        );
        catalog::test_helpers::write(root, ".doctrine/adr/001/adr-001.md", "body\n");

        // Seed ADR-002 (OLD) in the torn state: superseded_by = [], status = accepted.
        catalog::test_helpers::write(
            root,
            ".doctrine/adr/002/adr-002.toml",
            "id = 2\nslug = \"a2\"\ntitle = \"A2\"\nstatus = \"accepted\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n",
        );
        catalog::test_helpers::write(root, ".doctrine/adr/002/adr-002.md", "body\n");

        // Act: re-run supersede from the torn state.
        run_supersede(Some(root.to_path_buf()), "ADR-001", "ADR-002")
            .expect("recovery supersede should succeed");

        // Assert: OLD status flipped to superseded.
        let old_toml =
            std::fs::read_to_string(root.join(".doctrine/adr/002/adr-002.toml")).unwrap();
        assert!(
            old_toml.contains("status = \"superseded\""),
            "OLD.status should be superseded, got: {old_toml}"
        );

        // Assert: OLD.superseded_by contains ADR-001.
        assert!(
            old_toml.contains("superseded_by = [\"ADR-001\"]"),
            "OLD.superseded_by should contain ADR-001, got: {old_toml}"
        );

        // Assert: NEW has a [[relation]] label="supersedes" row targeting ADR-002
        // (SL-095 PHASE-03: RelationRow path writes [[relation]] rows, not typed arrays).
        let new_toml =
            std::fs::read_to_string(root.join(".doctrine/adr/001/adr-001.toml")).unwrap();
        assert!(
            new_toml.contains("[[relation]]")
                && new_toml.contains("label = \"supersedes\"")
                && new_toml.contains("target = \"ADR-002\""),
            "NEW should have [[relation]] supersedes → ADR-002: {new_toml}"
        );
    }

    // --- SL-097 PHASE-03: record cross-kind supersession tests ----------------

    #[test]
    fn supersede_same_kind_record_allowed() {
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // Seed ASM-001 (NEW): open status
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/001/record-001.toml",
            "id = 1\nslug = \"a1\"\ntitle = \"A1\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"yes_no\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/001/record-001.md",
            "body\n",
        );

        // Seed ASM-002 (OLD): open status
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/002/record-002.toml",
            "id = 2\nslug = \"a2\"\ntitle = \"A2\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"yes_no\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/002/record-002.md",
            "body\n",
        );

        // Act: supersede same-kind records
        run_supersede(Some(root.to_path_buf()), "ASM-001", "ASM-002")
            .expect("same-kind record supersession should succeed");

        // Assert: OLD status flipped to obsolete
        let old_toml = std::fs::read_to_string(
            root.join(".doctrine/knowledge/assumption/002/record-002.toml"),
        )
        .unwrap();
        assert!(
            old_toml.contains("status = \"obsolete\""),
            "OLD.status should be obsolete, got: {old_toml}"
        );
    }

    #[test]
    fn supersede_cross_kind_allowed_matrix() {
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // Seed DEC-001 (NEW): decision
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/001/record-001.toml",
            "id = 1\nslug = \"d1\"\ntitle = \"D1\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"action_item\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/001/record-001.md",
            "body\n",
        );

        // Seed ASM-002 (OLD): assumption
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/002/record-002.toml",
            "id = 2\nslug = \"a2\"\ntitle = \"A2\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"yes_no\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/002/record-002.md",
            "body\n",
        );

        // Act: supersede cross-kind (DEC → ASM is allowed per §6 matrix)
        run_supersede(Some(root.to_path_buf()), "DEC-001", "ASM-002")
            .expect("cross-kind supersession DEC → ASM should succeed");

        // Assert: OLD status flipped to obsolete
        let old_toml = std::fs::read_to_string(
            root.join(".doctrine/knowledge/assumption/002/record-002.toml"),
        )
        .unwrap();
        assert!(
            old_toml.contains("status = \"obsolete\""),
            "OLD.status should be obsolete, got: {old_toml}"
        );
    }

    #[test]
    fn supersede_cross_kind_refused_matrix() {
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // Seed ASM-001 (NEW): assumption
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/001/record-001.toml",
            "id = 1\nslug = \"a1\"\ntitle = \"A1\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"yes_no\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/001/record-001.md",
            "body\n",
        );

        // Seed DEC-002 (OLD): decision
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/002/record-002.toml",
            "id = 2\nslug = \"d2\"\ntitle = \"D2\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"action_item\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/002/record-002.md",
            "body\n",
        );

        // Act: supersede cross-kind (ASM → DEC is disallowed per §6 matrix)
        let result = run_supersede(Some(root.to_path_buf()), "ASM-001", "DEC-002");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("§6 matrix disallows ASM → DEC")
        );
    }

    #[test]
    fn supersede_question_reopening_refused() {
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // Seed QUE-001 (NEW): question
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/question/001/record-001.toml",
            "id = 1\nslug = \"q1\"\ntitle = \"Q1\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"yes_no\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/question/001/record-001.md",
            "body\n",
        );

        // Seed QUE-002 (OLD): question with terminal status
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/question/002/record-002.toml",
            "id = 2\nslug = \"q2\"\ntitle = \"Q2\"\nstatus = \"answered\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"yes_no\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/question/002/record-002.md",
            "body\n",
        );

        // Act: supersede terminal record - status should NOT flip
        run_supersede(Some(root.to_path_buf()), "QUE-001", "QUE-002")
            .expect("supersession should proceed but not flip terminal status");

        // Assert: OLD status stays answered (terminal status not flipped)
        let old_toml =
            std::fs::read_to_string(root.join(".doctrine/knowledge/question/002/record-002.toml"))
                .unwrap();
        assert!(
            old_toml.contains("status = \"answered\""),
            "OLD.status should remain answered (terminal), got: {old_toml}"
        );
        // But timestamp should be updated
        assert!(
            old_toml.contains("updated = \"2026-06-18\""),
            "OLD.updated should be refreshed, got: {old_toml}"
        );
    }

    #[test]
    fn supersede_cross_family_refused() {
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // Seed ADR-001 (NEW): ADR
        catalog::test_helpers::write(
            root,
            ".doctrine/adr/001/adr-001.toml",
            "id = 1\nslug = \"a1\"\ntitle = \"A1\"\nstatus = \"accepted\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n",
        );
        catalog::test_helpers::write(root, ".doctrine/adr/001/adr-001.md", "body\n");

        // Seed ASM-002 (OLD): assumption
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/002/record-002.toml",
            "id = 2\nslug = \"a2\"\ntitle = \"A2\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"yes_no\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/002/record-002.md",
            "body\n",
        );

        // Act: cross-family supersession (ADR → ASM) should fail
        let result = run_supersede(Some(root.to_path_buf()), "ADR-001", "ASM-002");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cross-family supersession refused")
        );
    }

    #[test]
    fn supersede_self_supersession_refused() {
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // Seed ASM-001
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/001/record-001.toml",
            "id = 1\nslug = \"a1\"\ntitle = \"A1\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"yes_no\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/001/record-001.md",
            "body\n",
        );

        // Act: self-supersession should fail
        let result = run_supersede(Some(root.to_path_buf()), "ASM-001", "ASM-001");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cannot supersede itself")
        );
    }

    #[test]
    fn supersede_already_terminal_no_flip() {
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // Seed DEC-001 (NEW): decision
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/001/record-001.toml",
            "id = 1\nslug = \"d1\"\ntitle = \"D1\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"action_item\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/001/record-001.md",
            "body\n",
        );

        // Seed DEC-002 (OLD): decision with terminal status
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/002/record-002.toml",
            "id = 2\nslug = \"d2\"\ntitle = \"D2\"\nstatus = \"accepted\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"action_item\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/002/record-002.md",
            "body\n",
        );

        // Act: supersede terminal record
        run_supersede(Some(root.to_path_buf()), "DEC-001", "DEC-002")
            .expect("supersession should succeed but not flip terminal status");

        // Assert: OLD status stays accepted (terminal status preserved)
        let old_toml =
            std::fs::read_to_string(root.join(".doctrine/knowledge/decision/002/record-002.toml"))
                .unwrap();
        assert!(
            old_toml.contains("status = \"accepted\""),
            "OLD.status should remain accepted (terminal), got: {old_toml}"
        );
        // Timestamp should still be updated
        assert!(
            old_toml.contains("updated = \"2026-06-18\""),
            "OLD.updated should be refreshed even for terminal, got: {old_toml}"
        );
    }

    #[test]
    fn supersede_idempotent_cross_kind() {
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // Seed CON-001 (NEW): already linked to QUE-002
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/constraint/001/record-001.toml",
            "id = 1\nslug = \"c1\"\ntitle = \"C1\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = [\"QUE-002\"]\nsuperseded_by = []\n[facet]\nkind = \"implementation\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/constraint/001/record-001.md",
            "body\n",
        );

        // Seed QUE-002 (OLD): already superseded by CON-001
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/question/002/record-002.toml",
            "id = 2\nslug = \"q2\"\ntitle = \"Q2\"\nstatus = \"obsolete\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = [\"CON-001\"]\n[facet]\nkind = \"yes_no\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/question/002/record-002.md",
            "body\n",
        );

        // Act: re-run the same supersession (idempotent)
        run_supersede(Some(root.to_path_buf()), "CON-001", "QUE-002")
            .expect("idempotent cross-kind supersession should succeed");

        // Assert: no changes, relationships preserved
        let new_toml = std::fs::read_to_string(
            root.join(".doctrine/knowledge/constraint/001/record-001.toml"),
        )
        .unwrap();
        let old_toml =
            std::fs::read_to_string(root.join(".doctrine/knowledge/question/002/record-002.toml"))
                .unwrap();
        assert!(new_toml.contains("supersedes = [\"QUE-002\"]"));
        assert!(old_toml.contains("superseded_by = [\"CON-001\"]"));
        assert!(old_toml.contains("status = \"obsolete\""));
    }

    #[test]
    fn supersede_decision_to_question_reopening_refused() {
        // VT-4: DEC→QUE (NEW=question, OLD=decision) — decision cannot be
        // superseded by question per the §6 matrix.
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/question/001/record-001.toml",
            "id = 1\nslug = \"q1\"\ntitle = \"Q1\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"yes_no\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/question/001/record-001.md",
            "body\n",
        );

        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/002/record-002.toml",
            "id = 2\nslug = \"d2\"\ntitle = \"D2\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"action_item\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/002/record-002.md",
            "body\n",
        );

        let result = run_supersede(Some(root.to_path_buf()), "QUE-001", "DEC-002");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("§6 matrix disallows QUE → DEC")
        );
    }

    #[test]
    fn supersede_torn_recovery() {
        // VT-10: NEW has the supersedes edge but OLD's superseded_by is missing
        // — re-running the supersede verb should recover (detected as drift into
        // empty/inconsistent carve-out, or recover on re-run if status is not yet
        // superseded).
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // First: create a valid supersession DEC-001 → DEC-002
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/001/record-001.toml",
            "id = 1\nslug = \"d1\"\ntitle = \"D1\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"action_item\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/001/record-001.md",
            "body\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/002/record-002.toml",
            "id = 2\nslug = \"d2\"\ntitle = \"D2\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"action_item\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/002/record-002.md",
            "body\n",
        );

        run_supersede(Some(root.to_path_buf()), "DEC-001", "DEC-002")
            .expect("initial supersession should succeed");

        // Now simulate torn state: remove superseded_by from OLD
        std::fs::write(
            root.join(".doctrine/knowledge/decision/002/record-002.toml"),
            "id = 2\nslug = \"d2\"\ntitle = \"D2\"\nstatus = \"superseded\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"action_item\"\n",
        )
        .unwrap();

        // Re-run: should detect drift (status=superseded but carve-out empty)
        let result = run_supersede(Some(root.to_path_buf()), "DEC-001", "DEC-002");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("superseded_by carve-out is empty")
        );
    }

    /// VT-9: `doctrine link` on a record Supersedes edge must be refused — the
    /// rule row declares LifecycleOnly, so the link verb cannot create it.
    #[test]
    fn link_supersedes_on_record_is_lifecycle_only() {
        use crate::relation::{LinkPolicy, RelationLabel, lookup};
        // We can't call run_link directly (it writes IO), but we can check the
        // rule: the RECORD→RECORD Supersedes row must be LifecycleOnly.
        let rule = lookup(&crate::knowledge::DECISION_KIND, RelationLabel::Supersedes)
            .expect("DECISION_KIND should have a Supersedes rule row");
        assert!(
            matches!(rule.link, LinkPolicy::LifecycleOnly),
            "record Supersedes must be LifecycleOnly"
        );
        // Verify an ADR Supersedes lookup also returns LifecycleOnly.
        let adr_rule = lookup(&crate::adr::ADR_KIND.kind, RelationLabel::Supersedes)
            .expect("ADR_KIND should have a Supersedes rule row");
        assert!(
            matches!(adr_rule.link, LinkPolicy::LifecycleOnly),
            "ADR Supersedes must be LifecycleOnly"
        );
    }

    // --- SL-090 §PHASE-03: memory link/unlink tests ---------------------------

    const MEM_TEST_UID: &str = "mem_018f3a1b2c3d4e5f60718293a4b5c6d7";

    fn seed_sl_toml(root: &std::path::Path, id: u32) {
        let padded = format!("{id:03}");
        let dir = root.join(".doctrine/slice").join(&padded);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("slice-{padded}.toml")),
            format!(
                "id = {id}\nslug = \"t{padded}\"\ntitle = \"Test SL-{padded}\"\n\
                 created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 status = \"accepted\"\n"
            ),
        )
        .unwrap();
        std::fs::write(dir.join(format!("slice-{padded}.md")), "body\n").unwrap();
    }

    fn seed_adr_toml(root: &std::path::Path, id: u32) {
        let padded = format!("{id:03}");
        let dir = root.join(".doctrine/adr").join(&padded);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("adr-{padded}.toml")),
            format!(
                "id = {id}\nslug = \"a{padded}\"\ntitle = \"ADR {padded}\"\n\
                 created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 status = \"accepted\"\n"
            ),
        )
        .unwrap();
        std::fs::write(dir.join(format!("adr-{padded}.md")), "body\n").unwrap();
    }

    fn seed_memory_toml(root: &std::path::Path, uid: &str, content: &str) {
        let mem_dir = root.join(".doctrine/memory/items").join(uid);
        std::fs::create_dir_all(&mem_dir).unwrap();
        std::fs::write(mem_dir.join("memory.toml"), content).unwrap();
    }

    // VT-1 — link mem_<uid> to a canonical ref appends a [[relation]] row.
    #[test]
    fn link_memory_uid_appends_relation_row() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_sl_toml(root, 1);
        seed_memory_toml(root, MEM_TEST_UID, "");

        run_link(Some(root.to_path_buf()), MEM_TEST_UID, "related", "SL-001").unwrap();

        let content = std::fs::read_to_string(
            root.join(format!(".doctrine/memory/items/{MEM_TEST_UID}/memory.toml")),
        )
        .unwrap();
        assert!(content.contains("[[relation]]"), "relation row written");
        assert!(content.contains("label = \"related\""), "label present");
        assert!(content.contains("target = \"SL-001\""), "target present");
    }

    // VT-2 — Re-link is no-op (already linked).
    #[test]
    fn link_memory_uid_repeat_is_noop() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_sl_toml(root, 1);
        let seed = "[[relation]]\nlabel = \"related\"\ntarget = \"SL-001\"\n";
        seed_memory_toml(root, MEM_TEST_UID, seed);

        let before = std::fs::read_to_string(
            root.join(format!(".doctrine/memory/items/{MEM_TEST_UID}/memory.toml")),
        )
        .unwrap();
        run_link(Some(root.to_path_buf()), MEM_TEST_UID, "related", "SL-001").unwrap();
        let after = std::fs::read_to_string(
            root.join(format!(".doctrine/memory/items/{MEM_TEST_UID}/memory.toml")),
        )
        .unwrap();
        assert_eq!(before, after, "file unchanged on re-link");
    }

    // VT-3 — unlink + re-unlink.
    #[test]
    fn unlink_memory_uid_then_repeat() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let seed = "[[relation]]\nlabel = \"related\"\ntarget = \"SL-001\"\n";
        seed_memory_toml(root, MEM_TEST_UID, seed);

        run_unlink(Some(root.to_path_buf()), MEM_TEST_UID, "related", "SL-001").unwrap();
        let after_first = std::fs::read_to_string(
            root.join(format!(".doctrine/memory/items/{MEM_TEST_UID}/memory.toml")),
        )
        .unwrap();
        assert!(
            !after_first.contains("[[relation]]"),
            "relation row removed after unlink"
        );

        // Re-unlink — idempotent, no error.
        run_unlink(Some(root.to_path_buf()), MEM_TEST_UID, "related", "SL-001").unwrap();
    }

    // VT-4 — link with nonexistent canonical ref target fails.
    #[test]
    fn link_memory_uid_bad_target_errors() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_memory_toml(root, MEM_TEST_UID, "");

        let err =
            run_link(Some(root.to_path_buf()), MEM_TEST_UID, "related", "SL-999").unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("does not resolve"),
            "error should mention 'does not resolve', got: {msg}"
        );
    }

    // VT-5 — link with free-text target (no validation).
    #[test]
    fn link_memory_uid_free_text_target_ok() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_memory_toml(root, MEM_TEST_UID, "");

        run_link(
            Some(root.to_path_buf()),
            MEM_TEST_UID,
            "drift",
            "some free text",
        )
        .unwrap();
        let content = std::fs::read_to_string(
            root.join(format!(".doctrine/memory/items/{MEM_TEST_UID}/memory.toml")),
        )
        .unwrap();
        assert!(
            content.contains("target = \"some free text\""),
            "free text stored"
        );
    }

    // VT-6 — behaviour-preservation: numbered-entity link still works.
    #[test]
    fn link_numbered_entity_still_works() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_sl_toml(root, 48);
        seed_adr_toml(root, 10);

        run_link(Some(root.to_path_buf()), "SL-048", "governed_by", "ADR-010").unwrap();

        let toml_path = root.join(".doctrine/slice/048/slice-048.toml");
        let content = std::fs::read_to_string(&toml_path).unwrap();
        assert!(content.contains("[[relation]]"));
        assert!(content.contains("governed_by"));
        assert!(content.contains("ADR-010"));
    }

    // VT-7 — link with mem.<key> source.
    #[test]
    fn link_memory_key_appends_relation_row() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_sl_toml(root, 1);
        seed_memory_toml(root, "mem.fact.cli.skinny", "");

        run_link(
            Some(root.to_path_buf()),
            "mem.fact.cli.skinny",
            "related",
            "SL-001",
        )
        .unwrap();

        let content = std::fs::read_to_string(
            root.join(".doctrine/memory/items/mem.fact.cli.skinny/memory.toml"),
        )
        .unwrap();
        assert!(content.contains("[[relation]]"), "relation row written");
        assert!(content.contains("target = \"SL-001\""), "target present");
    }
}

#[cfg(test)]
mod write_class_tests {
    use super::*;

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
}
