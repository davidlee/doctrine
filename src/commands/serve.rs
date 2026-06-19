// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine serve` — start the MCP stdio server.

use anyhow::Context;
use clap::Args;
use std::path::PathBuf;

/// Arguments for `doctrine serve`.
#[derive(Args)]
pub(crate) struct ServeArgs {
    /// Start the MCP (Model Context Protocol) stdio server, exposing review
    /// verbs as tools over JSON-RPC 2.0.
    #[arg(long)]
    pub(crate) mcp: bool,

    /// Explicit project root (default: auto-detect).
    #[arg(long)]
    pub(crate) path: Option<PathBuf>,
}

/// Run `doctrine serve`.
///
/// With `--mcp`: starts the MCP stdio server in a tokio runtime.
/// Other serve modes are deferred (design D6).
pub(crate) fn run_serve(args: ServeArgs) -> anyhow::Result<()> {
    if args.mcp {
        let rt = tokio::runtime::Runtime::new().context("failed to create tokio runtime")?;
        rt.block_on(crate::mcp_server::serve(crate::mcp_server::McpConfig {
            path: args.path,
        }))
    } else {
        anyhow::bail!("`serve` requires --mcp (other serve modes not yet implemented)");
    }
}
