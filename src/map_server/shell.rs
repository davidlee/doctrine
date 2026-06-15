// SPDX-License-Identifier: GPL-3.0-only
//! Graphviz process bridge (SL-072 PHASE-04).
//!
//! Real [`DotRenderer`] implementation that shells out to `dot -Tsvg`,
//! plus a [`FakeDotRenderer`] for test injection so route tests don't
//! need `dot` installed.
#![allow(
    dead_code,
    reason = "FakeDotRenderer + DOT_BODY_LIMIT consumed in PHASE-05+"
)]

use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;

use crate::map_server::error::MapServerError;
use crate::map_server::state::DotRenderer;

/// Maximum DOT source buffer accepted (1 MiB).
pub(crate) const DOT_BODY_LIMIT: usize = 1_048_576;

/// Timeout for the `dot` process (stdin write + wait).
const DOT_TIMEOUT: Duration = Duration::from_secs(10);

#[async_trait]
impl DotRenderer for crate::map_server::state::RealDotRenderer {
    #[expect(
        clippy::expect_used,
        reason = "stdin configured as Stdio::piped() so take() always returns Some"
    )]
    async fn render_svg(&self, dot: &[u8]) -> Result<Vec<u8>, MapServerError> {
        let mut child = tokio::process::Command::new("dot")
            .arg("-Tsvg")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => MapServerError::ToolUnavailable { tool: "dot" },
                _ => MapServerError::Other(e.into()),
            })?;

        // Write stdin (owned); drop to signal EOF.
        let mut stdin = child.stdin.take().expect("stdin piped");
        let dot_owned = dot.to_vec();
        tokio::time::timeout(DOT_TIMEOUT, stdin.write_all(&dot_owned))
            .await
            .map_err(|_elapsed| MapServerError::Timeout { command: "dot" })?
            .map_err(|e| MapServerError::Other(e.into()))?;
        drop(stdin);

        // Wait for child to finish with timeout.
        let output = tokio::time::timeout(DOT_TIMEOUT, child.wait_with_output())
            .await
            .map_err(|_elapsed| MapServerError::Timeout { command: "dot" })?
            .map_err(|e| MapServerError::Other(e.into()))?;

        if !output.status.success() {
            return Err(MapServerError::CommandFailed {
                command: "dot",
                status: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }

        Ok(output.stdout)
    }
}

/// A test-only [`DotRenderer`] that returns configured responses.
///
/// Replaces the real process spawn so route tests don't need `dot` installed.
pub(crate) struct FakeDotRenderer {
    pub(crate) mode: FakeDotMode,
}

/// Configures the response for [`FakeDotRenderer`].
pub(crate) enum FakeDotMode {
    /// Return the given SVG bytes.
    Success(Vec<u8>),
    /// Simulate `dot` not installed.
    ToolUnavailable,
    /// Simulate `dot` exiting non-zero.
    CommandFailed { stderr: String },
    /// Simulate timeout.
    Timeout,
}

#[async_trait]
impl DotRenderer for FakeDotRenderer {
    async fn render_svg(&self, _dot: &[u8]) -> Result<Vec<u8>, MapServerError> {
        match &self.mode {
            FakeDotMode::Success(svg) => Ok(svg.clone()),
            FakeDotMode::ToolUnavailable => Err(MapServerError::ToolUnavailable { tool: "dot" }),
            FakeDotMode::CommandFailed { stderr } => Err(MapServerError::CommandFailed {
                command: "dot",
                status: Some(1),
                stderr: stderr.clone(),
            }),
            FakeDotMode::Timeout => Err(MapServerError::Timeout { command: "dot" }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fake_success_returns_svg_bytes() {
        let renderer = FakeDotRenderer {
            mode: FakeDotMode::Success(b"<svg></svg>".to_vec()),
        };
        let result = renderer.render_svg(b"digraph { a -> b }").await.unwrap();
        assert_eq!(result, b"<svg></svg>");
    }

    #[tokio::test]
    async fn fake_tool_unavailable_returns_503_error() {
        let renderer = FakeDotRenderer {
            mode: FakeDotMode::ToolUnavailable,
        };
        let err = renderer
            .render_svg(b"digraph { a -> b }")
            .await
            .unwrap_err();
        match err {
            MapServerError::ToolUnavailable { tool } => assert_eq!(tool, "dot"),
            other => panic!("expected ToolUnavailable, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn fake_command_failed_returns_422_error_with_stderr() {
        let renderer = FakeDotRenderer {
            mode: FakeDotMode::CommandFailed {
                stderr: "syntax error".to_owned(),
            },
        };
        let err = renderer.render_svg(b"garbage").await.unwrap_err();
        match err {
            MapServerError::CommandFailed {
                command,
                status,
                stderr,
            } => {
                assert_eq!(command, "dot");
                assert_eq!(status, Some(1));
                assert_eq!(stderr, "syntax error");
            }
            other => panic!("expected CommandFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn fake_timeout_returns_504_error() {
        let renderer = FakeDotRenderer {
            mode: FakeDotMode::Timeout,
        };
        let err = renderer
            .render_svg(b"digraph { a -> b }")
            .await
            .unwrap_err();
        match err {
            MapServerError::Timeout { command } => assert_eq!(command, "dot"),
            other => panic!("expected Timeout, got {other:?}"),
        }
    }

    // Conditional real tests — skipped if dot not on PATH.

    #[tokio::test]
    async fn real_dot_valid_input_returns_svg() {
        if which::which("dot").is_err() {
            return; // skip
        }
        let renderer = crate::map_server::state::RealDotRenderer;
        let result = renderer.render_svg(b"digraph { a -> b }").await.unwrap();
        let svg = String::from_utf8_lossy(&result);
        assert!(svg.contains("<svg"), "expected SVG output, got: {svg}");
    }

    #[tokio::test]
    async fn real_dot_garbage_input_returns_command_failed() {
        if which::which("dot").is_err() {
            return; // skip
        }
        let renderer = crate::map_server::state::RealDotRenderer;
        let err = renderer.render_svg(b"not valid dot").await.unwrap_err();
        match err {
            MapServerError::CommandFailed { stderr, .. } => {
                assert!(!stderr.is_empty(), "expected error output from dot");
            }
            other => panic!("expected CommandFailed, got {other:?}"),
        }
    }
}
