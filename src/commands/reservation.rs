// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine reservation list` — the held-claims survey (SL-148 PHASE-04, REQ-022).
//!
//! Thin command-tier shell over the engine survey [`crate::reserve::survey`]: it
//! resolves the project root and the coordination remote (`--remote` override, else
//! `git::resolve_remote`), calls the engine fetch → `for_each_ref` → parse path, and
//! renders the `{canonical, holder, acquired}` table via the `listing` leaf. All git
//! IO + the ref parsing live in the engine; this module owns only argument shape,
//! remote selection, and rendering (the pure/imperative split).

use std::io::Write;
use std::path::PathBuf;

use anyhow::Context;

use crate::listing::render_table;
use crate::reserve::{self, HeldClaim};

/// The `acquired` column caveat (F-12 / EX-3 / VA-1): the time is the holder's own
/// declared commit date, set client-side — NOT a server-attested clock. Printed under
/// the table so the operator never reads it as authoritative ordering.
const ACQUIRED_CAVEAT: &str = "note: `acquired` is best-effort client-declared metadata (the holder's commit \
     date), not a server-attested clock.";

#[derive(Debug, clap::Subcommand)]
pub(crate) enum ReservationCommand {
    /// Survey the held remote reservations (`refs/doctrine/reservation/*`) as a
    /// {canonical, holder, acquired} table. `acquired` is best-effort,
    /// client-declared metadata (the holder's commit date) — not a server clock.
    List(ListArgs),
}

#[derive(Debug, clap::Args)]
pub(crate) struct ListArgs {
    /// Narrow to one canonical id-space prefix segment, e.g. `SL` (case-insensitive).
    #[arg(short = 'k', long)]
    kind: Option<String>,

    /// Coordination remote to survey (default: the resolved reservation remote —
    /// preferred → origin → sole).
    #[arg(short = 'r', long)]
    remote: Option<String>,

    /// Explicit project root (default: auto-detect).
    #[arg(short = 'p', long)]
    path: Option<PathBuf>,
}

/// Dispatch the reservation survey verb.
pub(crate) fn dispatch(cmd: ReservationCommand) -> anyhow::Result<()> {
    match cmd {
        ReservationCommand::List(args) => run_list(args),
    }
}

fn run_list(args: ListArgs) -> anyhow::Result<()> {
    let root = crate::root::find(args.path, &crate::root::default_markers())?;
    let remote = match args.remote {
        Some(r) => r,
        None => crate::git::resolve_remote(&root)?.context(
            "no reservation remote configured (pass --remote, or configure a git remote)",
        )?,
    };
    // `--kind` matches the upper-cased id-space prefix the survey emits (F-V7).
    let kind = args.kind.map(|k| k.trim().to_ascii_uppercase());
    let held = reserve::survey(&root, &remote, kind.as_deref())?;
    print_survey(&held);
    Ok(())
}

/// Render the survey: a `{canonical, holder, acquired}` table, then the F-12 caveat.
/// Best-effort writes — a broken pipe is harmless for a CLI display.
fn print_survey(held: &[HeldClaim]) {
    let mut out = std::io::stdout();
    if held.is_empty() {
        _ = writeln!(out, "no held reservations");
        return;
    }
    let mut grid: Vec<Vec<String>> = vec![row("canonical", "holder", "acquired")];
    // Stable order: by canonical id.
    let mut held: Vec<&HeldClaim> = held.iter().collect();
    held.sort_by(|a, b| a.canonical.cmp(&b.canonical));
    for h in held {
        grid.push(vec![
            h.canonical.clone(),
            h.holder.clone(),
            h.acquired.clone(),
        ]);
    }
    _ = write!(
        out,
        "{}",
        render_table(&grid, crate::tty::stdout_terminal_width())
    );
    _ = writeln!(out, "{ACQUIRED_CAVEAT}");
}

/// A three-cell header/data row.
fn row(a: &str, b: &str, c: &str) -> Vec<String> {
    vec![a.to_owned(), b.to_owned(), c.to_owned()]
}

#[cfg(test)]
mod tests {
    use super::*;

    // VA-1: the F-12 caveat text states `acquired` is client-declared best-effort,
    // not a server clock — pinned so the survey output always carries it.
    #[test]
    fn acquired_caveat_states_client_declared_best_effort() {
        assert!(ACQUIRED_CAVEAT.contains("best-effort"));
        assert!(ACQUIRED_CAVEAT.contains("client-declared"));
        assert!(ACQUIRED_CAVEAT.contains("not a server-attested clock"));
    }
}
