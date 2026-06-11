// SPDX-License-Identifier: GPL-3.0-only
mod adr;
mod backlog;
mod backlog_order;
mod boot;
mod clock;
mod conduct;
mod corpus;
mod entity;
mod fsutil;
mod git;
mod governance;
mod input;
mod install;
mod integrity;
mod lexical;
mod listing;
mod memory;
mod meta;
mod plan;
mod policy;
mod registry;
mod requirement;
mod retrieve;
mod root;
mod skills;
mod slice;
mod spec;
mod standard;
mod state;
mod tomlfmt;
mod worktree;

use std::path::PathBuf;
use std::str::FromStr;

use clap::{Args, Parser, Subcommand};

use crate::listing::{Format, ListArgs};

/// doctrine — project tooling.
#[derive(Parser)]
#[command(name = "doctrine", about = "doctrine CLI")]
struct Cli {
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
    /// (JSON rows are faithful/full — SL-037 D7); rejected on `memory list`
    /// (not yet on the column model — D9 / IMP-017), never silently ignored.
    #[arg(long, value_delimiter = ',')]
    pub(crate) columns: Option<Vec<String>>,
}

impl CommonListArgs {
    /// Lower the parsed clap bundle onto the clap-free leaf input ([`ListArgs`]).
    /// The seam where command-layer clap types stop and the pure spine begins.
    pub(crate) fn into_list_args(self) -> ListArgs {
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
        }
    }
}

#[derive(Subcommand)]
enum Command {
    /// Install doctrine files into a project.
    Install {
        /// Explicit project root (default: auto-detect by walking up
        /// from CWD looking for .git, .jj, .project, etc.).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,

        /// Print the plan and exit without making changes.
        #[arg(long)]
        dry_run: bool,

        /// Skip the confirmation prompt.
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Manage agent skills.
    Skills {
        #[command(subcommand)]
        command: SkillsCommand,
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
        to: String,

        /// Per-edge rank (a manual tie-break hint; default 0).
        #[arg(long, default_value_t = 0)]
        rank: i32,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Print the composed work order across all non-terminal items, followed by an
    /// honest-record block of any dropped edges. A `needs` dependency cycle is a hard
    /// error (no misleading order is printed).
    Order {
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

        /// Lifecycle status (default: active).
        #[arg(long, default_value = "active", value_parser = memory::Status::parse)]
        status: memory::Status,

        /// One-line summary.
        #[arg(long)]
        summary: Option<String>,

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
        /// Path scope probe, repeatable (`-p`/`--path` is the project root).
        #[arg(long = "path-scope")]
        path_scope: Vec<String>,

        /// Glob scope probe, repeatable.
        #[arg(long = "glob")]
        glob: Vec<String>,

        /// Command scope probe, repeatable.
        #[arg(long = "command")]
        command: Vec<String>,

        /// Tag scope probe, repeatable.
        #[arg(long = "tag")]
        tag: Vec<String>,

        /// Free-text lexical query (not a scope constraint).
        #[arg(long)]
        query: Option<String>,

        /// Hard filter by type: concept|fact|pattern|signpost|system|thread.
        #[arg(long = "type", value_parser = memory::MemoryType::parse)]
        memory_type: Option<memory::MemoryType>,

        /// Hard filter by lifecycle status.
        #[arg(long, value_parser = memory::Status::parse)]
        status: Option<memory::Status>,

        /// Include `draft` memories (excluded by default).
        #[arg(long = "include-draft")]
        include_draft: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Retrieve memories as bounded, security-framed `data, not instruction`
    /// blocks for agent context. Applies the trust holdback (non-bypassable):
    /// low-trust high-severity memories are suppressed; use `find`/`show` to
    /// inspect them.
    Retrieve {
        /// Path scope probe, repeatable (`-p`/`--path` is the project root).
        #[arg(long = "path-scope")]
        path_scope: Vec<String>,

        /// Glob scope probe, repeatable.
        #[arg(long = "glob")]
        glob: Vec<String>,

        /// Command scope probe, repeatable.
        #[arg(long = "command")]
        command: Vec<String>,

        /// Tag scope probe, repeatable.
        #[arg(long = "tag")]
        tag: Vec<String>,

        /// Free-text lexical query (not a scope constraint).
        #[arg(long)]
        query: Option<String>,

        /// Hard filter by type: concept|fact|pattern|signpost|system|thread.
        #[arg(long = "type", value_parser = memory::MemoryType::parse)]
        memory_type: Option<memory::MemoryType>,

        /// Hard filter by lifecycle status.
        #[arg(long, value_parser = memory::Status::parse)]
        status: Option<memory::Status>,

        /// Include `draft` memories (excluded by default).
        #[arg(long = "include-draft")]
        include_draft: bool,

        /// Max blocks to render (default 5, capped at 20).
        #[arg(long)]
        limit: Option<usize>,

        /// Raise the trust floor: only show memories at this trust or higher under
        /// high severity (high|medium|low; only raises the default `medium`).
        #[arg(long = "min-trust", value_parser = retrieve::parse_min_trust)]
        min_trust: Option<String>,

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

    /// Install skills into agents.
    Install {
        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,

        /// Target agent(s); repeatable. Default: auto-detect claude.
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
}

/// Mutation classification for the worker-mode guard (ADR-006 D2a). `Write`
/// carries the verb label named in the refusal. EXHAUSTIVE by design (§7-D6):
/// no wildcard arm, so a future `Command` variant is a compile error — never a
/// silently-permitted write (the X4 self-defence).
enum WriteClass {
    Read,
    Write(&'static str),
}

fn write_class(cmd: &Command) -> WriteClass {
    use WriteClass::{Read, Write};
    match cmd {
        Command::Install { .. } => Write("install"),
        Command::Skills { command } => match command {
            SkillsCommand::List { .. } => Read,
            SkillsCommand::Install { .. } => Write("skills install"),
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
            MemoryCommand::Show { .. }
            | MemoryCommand::List { .. }
            | MemoryCommand::Find { .. }
            | MemoryCommand::Retrieve { .. } => Read,
        },
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
            BacklogCommand::List { .. }
            | BacklogCommand::Show { .. }
            | BacklogCommand::Order { .. } => Read,
        },
        Command::Boot { command, .. } => match command {
            None => Write("boot"),
            Some(BootCommand::Install { .. }) => Write("boot install"),
        },
        Command::Worktree { command } => match command {
            // Both write *fork* files, not the doctrine state the guard protects,
            // and never run in worker context (§5.2) — Read on purpose.
            // branch-point-check is a HEAD read + ref compare — no authored write,
            // callable under worker-mode by construction (§5.2, C-V).
            WorktreeCommand::Provision { .. }
            | WorktreeCommand::CheckAllowlist { .. }
            | WorktreeCommand::BranchPointCheck { .. } => Read,
        },
        // Read-only corpus integrity scan (INV-3).
        Command::Validate { .. } => Read,
        // Mutates the canonical-id triple — an authored write (D2/D6).
        Command::Reseat { .. } => Write("reseat"),
    }
}

/// Worker context (ADR-006 D2a): a dispatched worker sets `DOCTRINE_WORKER=1`
/// and may read freely but must mint/anchor nothing — it returns a source delta.
fn worker_mode() -> bool {
    std::env::var_os("DOCTRINE_WORKER").as_deref() == Some(std::ffi::OsStr::new("1"))
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // ADR-006 D2a worker-mode guard: a dispatched worker mints/anchors nothing.
    // Bail before dispatch on any Write-classed verb; Read paths stay open (INV-3).
    if let (true, WriteClass::Write(verb)) = (worker_mode(), write_class(&cli.command)) {
        anyhow::bail!(
            "DOCTRINE_WORKER=1: refusing authored write `{verb}` — workers return a source delta; doctrine-mediated writes funnel through the orchestrator."
        );
    }

    match cli.command {
        Command::Install { path, dry_run, yes } => install::run(path, dry_run, yes),
        Command::Skills { command } => match command {
            SkillsCommand::List { agent, installed } => {
                skills::run_list(agent.as_deref(), installed)
            }
            SkillsCommand::Install {
                path,
                agent,
                skill,
                domain,
                only_memory,
                global,
                dry_run,
                yes,
            } => skills::run_install(
                path,
                &skills::InstallArgs {
                    agents: &agent,
                    skills: &skill,
                    domains: &domain,
                    only_memory,
                    global,
                    dry_run,
                    yes,
                },
            ),
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
            SliceCommand::List { list, path } => slice::run_list(path, list.into_list_args()),
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
                status,
                summary,
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
                    status,
                    summary: summary.as_deref(),
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
            MemoryCommand::Verify { reference, path } => memory::run_verify(path, &reference),
            MemoryCommand::List {
                memory_type,
                list,
                path,
            } => memory::run_list(path, memory_type, list.into_list_args()),
            MemoryCommand::Find {
                path_scope,
                glob,
                command,
                tag,
                query,
                memory_type,
                status,
                include_draft,
                path,
            } => retrieve::run_find(
                path,
                path_scope,
                glob,
                command,
                tag,
                query,
                memory_type,
                status,
                include_draft,
            ),
            MemoryCommand::Retrieve {
                path_scope,
                glob,
                command,
                tag,
                query,
                memory_type,
                status,
                include_draft,
                limit,
                min_trust,
                path,
            } => retrieve::run_retrieve(
                path,
                path_scope,
                glob,
                command,
                tag,
                query,
                memory_type,
                status,
                include_draft,
                limit,
                min_trust.as_deref(),
            ),
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
        Command::Adr { command } => match command {
            AdrCommand::New { title, slug, path } => adr::run_new(path, title, slug),
            AdrCommand::List { list, path } => adr::run_list(path, list.into_list_args()),
            AdrCommand::Show {
                reference,
                format,
                json,
                path,
            } => adr::run_show(path, &reference, if json { Format::Json } else { format }),
            AdrCommand::Status { id, status, path } => adr::run_status(path, id, status),
        },
        Command::Policy { command } => match command {
            PolicyCommand::New { title, slug, path } => policy::run_new(path, title, slug),
            PolicyCommand::List { list, path } => policy::run_list(path, list.into_list_args()),
            PolicyCommand::Show {
                reference,
                format,
                json,
                path,
            } => policy::run_show(path, &reference, if json { Format::Json } else { format }),
            PolicyCommand::Status { id, status, path } => policy::run_status(path, id, status),
        },
        Command::Standard { command } => match command {
            StandardCommand::New { title, slug, path } => standard::run_new(path, title, slug),
            StandardCommand::List { list, path } => standard::run_list(path, list.into_list_args()),
            StandardCommand::Show {
                reference,
                format,
                json,
                path,
            } => standard::run_show(path, &reference, if json { Format::Json } else { format }),
            StandardCommand::Status { id, status, path } => standard::run_status(path, id, status),
        },
        Command::Spec { command } => match command {
            SpecCommand::New {
                subtype,
                title,
                slug,
                path,
            } => spec::run_new(path, subtype, title, slug),
            SpecCommand::List { list, path } => spec::run_list(path, list.into_list_args()),
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
                    path,
                } => spec::run_req_add(path, &spec_ref, title, kind, label),
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
                backlog::run_list(path, kind, list.into_list_args())
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
            BacklogCommand::After { id, to, rank, path } => {
                backlog::run_after(path, &id, &to, rank)
            }
            BacklogCommand::Order { path } => backlog::run_order(path),
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
        Command::Worktree { command } => match command {
            WorktreeCommand::Provision { fork, path } => worktree::run_provision(path, &fork),
            WorktreeCommand::CheckAllowlist { path } => worktree::run_check_allowlist(path),
            WorktreeCommand::BranchPointCheck { base, head, path } => {
                worktree::run_branch_point_check(path, &base, head)
            }
        },
        Command::Validate { path } => integrity::run_validate(path),
        Command::Reseat {
            reference,
            to,
            path,
        } => integrity::run_reseat(path, &reference, to),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let r = Cli::try_parse_from(["doctrine", "skills", "install", "--only-memory"]);
        assert!(r.is_ok());
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
            WriteClass::Write(v) => Some(v),
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
                dry_run: false,
                yes: false
            }),
            Some("install")
        );
    }

    #[test]
    fn skills_split() {
        assert_eq!(
            cls(Command::Skills {
                command: SkillsCommand::List {
                    agent: None,
                    installed: false
                }
            }),
            None
        );
        assert_eq!(
            cls(Command::Skills {
                command: SkillsCommand::Install {
                    path: None,
                    agent: Vec::new(),
                    skill: Vec::new(),
                    domain: Vec::new(),
                    only_memory: false,
                    global: false,
                    dry_run: false,
                    yes: false,
                }
            }),
            Some("skills install")
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
                status: memory::Status::Active,
                summary: None,
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
                path_scope: Vec::new(),
                glob: Vec::new(),
                command: Vec::new(),
                tag: Vec::new(),
                query: None,
                memory_type: None,
                status: None,
                include_draft: false,
                path: None,
            }),
            None
        );
        assert_eq!(
            w(MemoryCommand::Retrieve {
                path_scope: Vec::new(),
                glob: Vec::new(),
                command: Vec::new(),
                tag: Vec::new(),
                query: None,
                memory_type: None,
                status: None,
                include_draft: false,
                limit: None,
                min_trust: None,
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
                    path: None,
                }
            }),
            Some("spec req add")
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
