// SPDX-License-Identifier: GPL-3.0-only
mod adr;
mod boot;
mod clock;
mod corpus;
mod entity;
mod fsutil;
mod git;
mod input;
mod install;
mod lexical;
mod memory;
mod meta;
mod plan;
mod registry;
mod requirement;
mod retrieve;
mod root;
mod skills;
mod slice;
mod spec;
mod state;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// doctrine — project tooling.
#[derive(Parser)]
#[command(name = "doctrine", about = "doctrine CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
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

    /// Create and list product / technical specifications.
    Spec {
        #[command(subcommand)]
        command: SpecCommand,
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

    /// List ADRs by id: id, status, slug, title.
    List {
        /// Filter to a single status.
        #[arg(long)]
        status: Option<String>,

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
        /// Filter to a single status.
        #[arg(long)]
        status: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Reassemble a spec into its readable whole and print it to stdout.
    Show {
        /// Canonical spec ref: `PRD-NNN` (product) or `SPEC-NNN` (tech).
        spec_ref: String,

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
enum MemoryCommand {
    /// Mint a uid and scaffold a new memory under `.doctrine/memory/items`.
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

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Resolve a memory by uid or key and print its header + body-as-data.
    Show {
        /// Memory reference: a `mem_<hex>` uid or a `mem.<…>` key.
        reference: String,

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

    /// List recorded memories, newest first; AND-filter by type/status/tag.
    List {
        /// Filter by type: concept|fact|pattern|signpost|system|thread.
        #[arg(long = "type", value_parser = memory::MemoryType::parse)]
        memory_type: Option<memory::MemoryType>,

        /// Filter by lifecycle status.
        #[arg(long, value_parser = memory::Status::parse)]
        status: Option<memory::Status>,

        /// Filter to memories carrying this tag.
        #[arg(long = "tag")]
        tag: Option<String>,

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

    /// List slices by id: id, status, slug, title.
    List {
        /// Filter to a single status.
        #[arg(long)]
        status: Option<String>,

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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

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
            SliceCommand::List { status, path } => slice::run_list(path, status.as_deref()),
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
                },
            ),
            MemoryCommand::Show { reference, path } => memory::run_show(path, &reference),
            MemoryCommand::Verify { reference, path } => memory::run_verify(path, &reference),
            MemoryCommand::List {
                memory_type,
                status,
                tag,
                path,
            } => memory::run_list(path, memory_type, status, tag.as_deref()),
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
            AdrCommand::List { status, path } => adr::run_list(path, status.as_deref()),
            AdrCommand::Status { id, status, path } => adr::run_status(path, id, status),
        },
        Command::Spec { command } => match command {
            SpecCommand::New {
                subtype,
                title,
                slug,
                path,
            } => spec::run_new(path, subtype, title, slug),
            SpecCommand::List { status, path } => spec::run_list(path, status.as_deref()),
            SpecCommand::Show { spec_ref, path } => spec::run_show(path, &spec_ref),
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
