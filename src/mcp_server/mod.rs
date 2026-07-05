// SPDX-License-Identifier: GPL-3.0-only
//! Hand-rolled MCP (Model Context Protocol) stdio server.
//!
//! Exposes the doctrine review verbs as MCP tools over stdin/stdout JSON-RPC 2.0.
//! Zero new crate dependencies — uses only `serde`, `serde_json`, and `tokio`
//! already in the dependency tree (design D4).

pub(crate) mod dispatch;
pub(crate) mod protocol;
pub(crate) mod tools;
pub(crate) mod transport;
pub(crate) mod worker_commit;

use std::path::{Path, PathBuf};
use tokio::io::{self, BufReader, BufWriter};

/// Producer of the corpus `--model` keys, injected into the MCP tool layer so it
/// need not reach up into `commands::prompt` (SL-203 D-B). Matches
/// `commands::prompt::model_keys` exactly; a plain `fn` pointer is `Copy` and
/// cannot be null in safe Rust. Dependency inversion severs the back edge
/// `mcp_server → commands` that formed the `{commands, mcp_server}` layering SCC.
pub(crate) type ModelKeysFn = fn(&Path, Option<&str>) -> anyhow::Result<Vec<String>>;

/// Configuration for the MCP server.
pub(crate) struct McpConfig {
    /// Explicit project root (default: auto-detect from cwd).
    pub(crate) path: Option<PathBuf>,
    /// Corpus model-key producer, supplied by `commands::serve` (the forward
    /// edge already depends on `mcp_server`) and threaded to the onboard arm.
    pub(crate) model_keys: ModelKeysFn,
}

/// Run the MCP stdio server.
///
/// Resolves the project root at startup (design D5), then enters a read →
/// dispatch → write loop. Exits cleanly on stdin EOF.
pub(crate) async fn serve(config: McpConfig) -> anyhow::Result<()> {
    let root = crate::root::find(config.path, &crate::root::default_markers())?;
    // Bound once before the loop (Copy fn-ptr; reads cleaner than field access
    // per dispatch iteration — SL-203 §10).
    let model_keys = config.model_keys;

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

        let response = tools::dispatch(&request, &root, model_keys);
        transport::write_message(&mut writer, &response).await?;
    }

    Ok(())
}
