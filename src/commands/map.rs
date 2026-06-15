// SPDX-License-Identifier: GPL-3.0-only
use std::path::PathBuf;

use clap::Args;

#[derive(Args)]
pub(crate) struct MapServeArgs {
    #[arg(long, default_value = "0")]
    pub(crate) port: u16,

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
    let root = crate::root::find(path, &crate::root::default_markers())?;
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
}
