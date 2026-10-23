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

/// The resolved project state for the MCP server.
///
/// Rootless means no root was found, or the discovered root carries no
/// `.doctrine/` directory. Tool handlers keep their `&Path` signature; the
/// rootless policy is applied in [`tools::dispatch`], whose method arms branch
/// on this state.
pub(crate) enum ProjectRoot {
    /// A doctrine project: `<root>/.doctrine/` exists.
    Doctrine(PathBuf),
    /// No doctrine project. `bare_root` records a root discovered without a
    /// `.doctrine/` dir, so the notice can name it.
    Rootless { bare_root: Option<PathBuf> },
}

impl ProjectRoot {
    /// Resolve the project state: an explicit path wins over a walk up from the
    /// cwd for the default markers ([`crate::root::find_opt`], which reads the
    /// cwd only when no path was given, so `--path` survives a deleted cwd).
    /// Never fails for lack of a root — a root without `.doctrine/` (or no root
    /// at all) is [`ProjectRoot::Rootless`]; only an unreadable cwd errors.
    pub(crate) fn resolve(explicit: Option<PathBuf>) -> anyhow::Result<Self> {
        let bare_root = crate::root::find_opt(explicit, &crate::root::default_markers())?;
        let is_doctrine = bare_root
            .as_deref()
            .is_some_and(crate::root::is_doctrine_project);
        Ok(classify(bare_root, is_doctrine))
    }
}

/// Pure classification: rooted iff a root was found and it has `.doctrine/`.
fn classify(bare_root: Option<PathBuf>, is_doctrine: bool) -> ProjectRoot {
    match bare_root {
        Some(root) if is_doctrine => ProjectRoot::Doctrine(root),
        bare_root => ProjectRoot::Rootless { bare_root },
    }
}

/// Run the MCP stdio server.
///
/// Resolves the project state at startup (design D5) without requiring a
/// project, then enters a read → dispatch → write loop. Exits cleanly on stdin
/// EOF.
pub(crate) async fn serve(config: McpConfig) -> anyhow::Result<()> {
    let project = ProjectRoot::resolve(config.path)?;
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

        let response = tools::dispatch(&request, &project, model_keys);
        transport::write_message(&mut writer, &response).await?;
    }

    Ok(())
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_root_with_doctrine_dir_is_doctrine() {
        let root = PathBuf::from("/project");
        assert!(matches!(
            classify(Some(root.clone()), true),
            ProjectRoot::Doctrine(p) if p == root
        ));
    }

    #[test]
    fn classify_root_without_doctrine_dir_is_rootless() {
        let root = PathBuf::from("/project");
        assert!(matches!(
            classify(Some(root.clone()), false),
            ProjectRoot::Rootless { bare_root: Some(p) } if p == root
        ));
    }

    #[test]
    fn classify_missing_root_is_rootless() {
        assert!(matches!(
            classify(None, false),
            ProjectRoot::Rootless { bare_root: None }
        ));
    }

    /// An explicit path carrying `.doctrine/` is a doctrine project (F6).
    #[test]
    fn resolve_explicit_doctrine_project_is_doctrine() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        std::fs::create_dir(root.join(crate::root::DOCTRINE_DIR)).unwrap();

        assert!(matches!(
            ProjectRoot::resolve(Some(root.clone())).unwrap(),
            ProjectRoot::Doctrine(p) if p == root
        ));
    }

    /// An explicit path without `.doctrine/` is rootless, and names that path
    /// so the notice can point at what fell short (F6).
    #[test]
    fn resolve_explicit_non_doctrine_dir_is_rootless_naming_it() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();

        assert!(matches!(
            ProjectRoot::resolve(Some(root.clone())).unwrap(),
            ProjectRoot::Rootless { bare_root: Some(p) } if p == root
        ));
    }
}
