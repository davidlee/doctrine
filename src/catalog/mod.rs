// SPDX-License-Identifier: GPL-3.0-only
//! Entity corpus catalog — the single source of truth for scanning and
//! hydrating the authored entity corpus (SL-071). Engine-tier (ADR-001):
//! depends on leaf modules + kind modules, never on command modules.
//!
//! - `scan` — the KINDS-driven corpus walk (re-homed from `relation_graph`)
//! - `hydrate` — richer catalog types (PHASE-03)
//! - `graph` — presentation-neutral graph projection (PHASE-04)
//! - `diagnostic` — structured diagnostics (PHASE-03)

use std::path::PathBuf;

use clap::Subcommand;

#[derive(Subcommand)]

pub(crate) enum CatalogCommand {
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

pub(crate) fn dispatch(cmd: CatalogCommand, _color: bool) -> anyhow::Result<()> {
    match cmd {
        CatalogCommand::Scan { root } => run_catalog_scan(root),
        CatalogCommand::Graph { root } => run_catalog_graph(root),
    }
}

// ---------------------------------------------------------------------------
// `doctrine catalog scan --json` / `doctrine catalog graph --json` (SL-071 PHASE-06)
// ---------------------------------------------------------------------------

/// Thin JSON dump of the hydrated `Catalog` — entities, edges, and diagnostics.
/// Developer scaffolding; not gating for acceptance (D12).
pub(crate) fn run_catalog_scan(root_arg: Option<PathBuf>) -> anyhow::Result<()> {
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
pub(crate) fn run_catalog_graph(root_arg: Option<PathBuf>) -> anyhow::Result<()> {
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

pub(crate) mod diagnostic;
pub(crate) mod graph;
pub(crate) mod hydrate;
pub(crate) mod scan;

#[cfg(test)]
pub(crate) mod test_helpers;
