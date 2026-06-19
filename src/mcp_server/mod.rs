// SPDX-License-Identifier: GPL-3.0-only
//! Hand-rolled MCP (Model Context Protocol) stdio server.
//!
//! Exposes the doctrine review verbs as MCP tools over stdin/stdout JSON-RPC 2.0.
//! Zero new crate dependencies — uses only `serde`, `serde_json`, and `tokio`
//! already in the dependency tree (design D4).

pub(crate) mod protocol;
pub(crate) mod tools;
pub(crate) mod transport;

use std::path::PathBuf;
use tokio::io::{self, BufReader, BufWriter};

/// Configuration for the MCP server.
pub(crate) struct McpConfig {
    /// Explicit project root (default: auto-detect from cwd).
    pub(crate) path: Option<PathBuf>,
}

/// Run the MCP stdio server.
///
/// Resolves the project root at startup (design D5), then enters a read →
/// dispatch → write loop. Exits cleanly on stdin EOF.
pub(crate) async fn serve(config: McpConfig) -> anyhow::Result<()> {
    let root = crate::root::find(config.path, &crate::root::default_markers())?;

    // Lock stdin/stdout with buffered I/O
    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut reader = BufReader::new(stdin);
    let mut writer = BufWriter::new(stdout);

    loop {
        let request = match transport::read_message(&mut reader).await {
            Ok(Some(req)) => req,
            Ok(None) => {
                // EOF — clean shutdown
                break;
            }
            Err(e) => {
                // Parse error — write error response and continue
                let resp = protocol::JsonRpcResponse::error(
                    None,
                    -32700,
                    "Parse error".to_owned(),
                    Some(serde_json::json!({ "message": e.to_string() })),
                );
                transport::write_message(&mut writer, &resp).await?;
                continue;
            }
        };

        let response = tools::dispatch(&request, &root);
        transport::write_message(&mut writer, &response).await?;
    }

    Ok(())
}
