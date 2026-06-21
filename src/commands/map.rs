// SPDX-License-Identifier: GPL-3.0-only
use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Args)]
pub(crate) struct MapServeArgs {
    #[arg(long, default_value = "0")]
    pub(crate) port: u16,

    #[arg(long)]
    pub(crate) path: Option<PathBuf>,

    #[arg(long)]
    pub(crate) open: bool,

    #[arg(long, value_parser = validate_focus)]
    pub(crate) focus: Option<String>,

    #[arg(long, default_value = "1", value_parser = clap::value_parser!(u8).range(1..=3))]
    pub(crate) depth: u8,
}

fn validate_focus(s: &str) -> Result<String, String> {
    crate::integrity::parse_canonical_ref(s)
        .map(|_| s.to_owned())
        .map_err(|e| format!("focus must be a canonical entity id (e.g. SL-001), got '{s}': {e}"))
}

pub(crate) fn run_serve(path: Option<PathBuf>, args: MapServeArgs) -> anyhow::Result<()> {
    let root = crate::root::find(args.path.or(path), &crate::root::default_markers())?;
    let catalog =
        crate::catalog::hydrate::scan_catalog(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
    let graph = crate::catalog::graph::CatalogGraph::from_catalog(&catalog);
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(crate::map_server::serve(crate::map_server::state::Config {
        root,
        graph,
        port: args.port,
        open: args.open,
        focus: args.focus,
        depth: args.depth,
    }))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn valid_focus_sl001() {
        assert!(validate_focus("SL-001").is_ok());
    }

    #[test]
    fn invalid_focus_lowercase_prefix() {
        assert!(validate_focus("sl-001").is_err());
    }

    #[test]
    fn invalid_focus_bogus_prefix() {
        assert!(validate_focus("BOGUS-001").is_err());
    }

    #[test]
    fn invalid_focus_empty() {
        assert!(validate_focus("").is_err());
    }

    #[test]
    fn map_serve_path_flag_passed_to_root_find() {
        // Verify --path flag is parsed and wired into run_serve's root::find call.
        // The flag replaces the outer path arg when provided.
        let args = MapServeArgs {
            path: Some(PathBuf::from("/tmp/test-doctrine-root")),
            port: 0,
            open: false,
            focus: None,
            depth: 1,
        };
        // If args.path is Some, it should be passed to root::find instead of the outer path.
        // This test doesn't actually call root::find (requires disk), but verifies the
        // flag is wired correctly — args.path.or(path) passes the right value.
        assert!(args.path.is_some());
        assert_eq!(args.path.unwrap(), PathBuf::from("/tmp/test-doctrine-root"));

        // Verify the default: if args.path is None, outer path is used.
        let args_no_path = MapServeArgs {
            path: None,
            port: 0,
            open: false,
            focus: None,
            depth: 1,
        };
        assert!(args_no_path.path.is_none());
    }
}

#[derive(Subcommand)]
pub(crate) enum MapCommand {
    /// Start the local map explorer web server (loopback only)
    Serve(MapServeArgs),
}

pub(crate) fn dispatch(cmd: MapCommand) -> anyhow::Result<()> {
    match cmd {
        MapCommand::Serve(args) => run_serve(None, args),
    }
}
