// SPDX-License-Identifier: GPL-3.0-only
mod entity;
mod fsutil;
mod install;
mod memory;
mod root;
mod skills;
mod slice;
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
                global,
                dry_run,
                yes,
            } => skills::run_install(path, &agent, &skill, &domain, global, dry_run, yes),
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
    }
}
