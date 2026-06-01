mod install;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Heresiarch — project tooling.
#[derive(Parser)]
#[command(name = "heresy", about = "Heresiarch CLI")]
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
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Install { path, dry_run, yes } => install::run(path, dry_run, yes),
    }
}
