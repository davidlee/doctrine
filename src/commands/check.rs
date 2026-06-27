// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine check {quick|commit|gate}` — the cadence proxy verb (SL-163).
//!
//! Resolves a project-declared check command from the OWNED `[verification]`
//! contract and proxy-executes it: inherit stdio, no timeout, forward the exit
//! code (incl. `128+signo` on signal death). The pure cadence resolution lives in
//! the `verify` leaf ([`crate::verify::resolve_check`]); this module is the impure
//! shell (ADR-001) — root detection, the config read, spawn, and exit forwarding.
//!
//! The OPPOSITE posture to `coverage_verify::run_argv` (pipe + capture + cap): a
//! dev gate streams live and may legitimately run long (design §5.4 / D5).

use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use anyhow::bail;
use clap::Subcommand;

use crate::verify::{CheckKind, CheckPlan};

/// The three check cadences as a clap subcommand. Each carries `-p/--path` (CR-F6),
/// threaded to root detection (aids e2e temp-root). clap-owned (ADR-001 / A2) —
/// bridges to the leaf [`CheckKind`] via [`From`], keeping the leaf clap-free.
#[derive(Debug, Subcommand)]
pub(crate) enum CheckCommand {
    /// Per-edit cadence. Unconfigured ⇒ an owned no-op (exit 0; never fails a hook).
    Quick {
        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
    /// Per-commit cadence. Unconfigured ⇒ `just check`.
    Commit {
        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
    /// End-of-phase cadence. Unconfigured ⇒ `just gate`.
    Gate {
        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

impl CheckCommand {
    /// The `-p/--path` override for this invocation.
    fn path(&self) -> Option<PathBuf> {
        match self {
            CheckCommand::Quick { path }
            | CheckCommand::Commit { path }
            | CheckCommand::Gate { path } => path.clone(),
        }
    }
}

impl From<&CheckCommand> for CheckKind {
    fn from(cmd: &CheckCommand) -> Self {
        match cmd {
            CheckCommand::Quick { .. } => CheckKind::Quick,
            CheckCommand::Commit { .. } => CheckKind::Commit,
            CheckCommand::Gate { .. } => CheckKind::Gate,
        }
    }
}

/// `doctrine check <kind>` — resolve the cadence's plan from the owned config and
/// act: print + exit-0 the owned no-op, error toward the key on an empty override,
/// or proxy-spawn the resolved argv (diverging via [`run_proxy`]).
pub(crate) fn dispatch(cmd: CheckCommand) -> anyhow::Result<()> {
    let root = crate::root::find(cmd.path(), &crate::root::default_markers())?;
    let cfg = crate::coverage_store::load_config(&root)?;
    let kind = CheckKind::from(&cmd);
    match crate::verify::resolve_check(&cfg, kind) {
        CheckPlan::Noop(note) => {
            println!("{note}");
            // Owned no-op: doctrine exits 0 itself — no child spawned (CR-F3).
            #[expect(
                clippy::disallowed_methods,
                reason = "the check verb forwards a terminal exit status; an owned no-op exits 0 (design §5.4)"
            )]
            {
                std::process::exit(0);
            }
        }
        CheckPlan::Empty(k) => bail!(
            "[verification].{} is empty — set a non-empty argv in {}",
            k.key(),
            crate::dtoml::DOCTRINE_TOML
        ),
        CheckPlan::Run(argv) => run_proxy(&root, &argv, kind),
    }
}

/// Proxy-spawn `argv` with `cwd == root`, INHERITING stdio (live stream; not piped)
/// and NO timeout (design §5.4 / D5). Reached only with a non-empty `argv` (INV-2).
/// Diverges via [`std::process::exit`] on a completed child — safe, stdio is
/// inherited so nothing is buffered/owned to flush (R2). A missing program
/// (`ENOENT`) yields an actionable error naming the owned config key (D3).
fn run_proxy(root: &Path, argv: &[String], kind: CheckKind) -> anyhow::Result<()> {
    let Some((program, args)) = argv.split_first() else {
        // INV-2: resolve_check never yields Run([]) — defend rather than panic.
        bail!("internal: empty check argv (resolve_check INV-2 violated)");
    };
    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .status()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!(
                    "`{program}` not found — set [verification].{} in {}",
                    kind.key(),
                    crate::dtoml::DOCTRINE_TOML
                )
            } else {
                anyhow::Error::new(e).context(format!("failed to spawn `{program}`"))
            }
        })?;
    #[expect(
        clippy::disallowed_methods,
        reason = "true exit forwarding (CR-F5): the proxied child's code/signal is the verb's terminal status (design §5.4)"
    )]
    {
        std::process::exit(exit_code(status));
    }
}

/// True exit forwarding (CR-F5): a normal exit yields its code; a signal-killed
/// child re-exits `128 + signo` (shell convention), not a flattened `1`. Unix-only
/// branch; doctrine targets linux/nixos.
fn exit_code(status: ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        status.signal().map_or(1, |s| 128 + s)
    }
    #[cfg(not(unix))]
    {
        1
    }
}
