// SPDX-License-Identifier: GPL-3.0-only
//! Map-server error model (SL-072 PHASE-01, SL-076 PHASE-02).

use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};

use crate::concept_map::ConceptMapMutationError;

/// The single error type for the map-server HTTP surface.
///
/// Each variant carries enough context for a self-describing JSON response;
/// the `IntoResponse` impl maps each variant to the correct HTTP status code
/// and serialises diagnostic fields so callers never parse prose strings.
#[derive(Debug, thiserror::Error)]
pub(crate) enum MapServerError {
    #[error("bad entity id: {0}")]
    BadEntityId(String),

    #[error("entity not found: {0}")]
    EntityNotFound(String),

    #[error("asset not found: {0}")]
    AssetNotFound(String),

    #[error("entity markdown not implemented for kind: {0}")]
    MarkdownNotImplemented(&'static str),

    #[error("request body too large")]
    BodyTooLarge,

    #[error("{tool} is unavailable")]
    ToolUnavailable { tool: &'static str },

    #[error("{command} failed with status {status:?}")]
    CommandFailed {
        command: &'static str,
        status: Option<i32>,
        stderr: String,
    },

    #[error("{command} timed out")]
    Timeout { command: &'static str },

    #[error("bad concept-map id: {0}")]
    BadConceptMapId(String),

    #[error("concept map not found: CM-{0:03}")]
    ConceptMapNotFound(u32),

    #[error("duplicate edge at line {line}")]
    DuplicateEdge { line: usize },

    #[error("edge not found")]
    EdgeNotFound,

    #[error("rename would collide with existing node '{0}' at line {1}")]
    NodeCollision(String, usize),

    #[error("empty field: {0}")]
    EmptyField(String),

    #[error("concept map was modified since last read")]
    StaleConceptMap,

    #[error("concept-map parse error: {0}")]
    ConceptMapParseError(String),

    #[error("concept-map I/O error: {0}")]
    ConceptMapIoError(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Maximum stderr bytes included in the JSON response for [`CommandFailed`].
const STDERR_CAP: usize = 8 * 1024; // 8 KiB

/// Truncate a string to at most `STDERR_CAP` bytes, on a UTF-8 boundary.
fn truncate_stderr(stderr: &str) -> &str {
    if stderr.len() <= STDERR_CAP {
        return stderr;
    }
    match stderr.char_indices().nth(STDERR_CAP) {
        Some((idx, _)) => &stderr[..idx],
        None => stderr,
    }
}

impl IntoResponse for MapServerError {
    fn into_response(self) -> axum::response::Response {
        let body: serde_json::Value = match &self {
            Self::BadEntityId(id) => {
                serde_json::json!({
                    "error": "bad_entity_id",
                    "message": format!("bad entity id: {id}"),
                })
            }
            Self::EntityNotFound(id) => {
                serde_json::json!({
                    "error": "entity_not_found",
                    "message": format!("entity not found: {id}"),
                })
            }
            Self::AssetNotFound(id) => {
                serde_json::json!({
                    "error": "asset_not_found",
                    "message": format!("asset not found: {id}"),
                })
            }
            Self::MarkdownNotImplemented(kind) => {
                serde_json::json!({
                    "error": "markdown_not_implemented",
                    "message": format!("entity markdown not implemented for kind: {kind}"),
                })
            }
            Self::BodyTooLarge => {
                serde_json::json!({
                    "error": "body_too_large",
                    "message": "request body too large",
                })
            }
            Self::ToolUnavailable { tool } => {
                serde_json::json!({
                    "error": "tool_unavailable",
                    "message": format!("{tool} is unavailable"),
                    "tool": tool,
                })
            }
            Self::CommandFailed {
                command,
                status,
                stderr,
            } => {
                serde_json::json!({
                    "error": "command_failed",
                    "message": format!("{command} failed with status {status:?}"),
                    "command": command,
                    "status": status,
                    "stderr": truncate_stderr(stderr),
                })
            }
            Self::Timeout { command } => {
                serde_json::json!({
                    "error": "timeout",
                    "message": format!("{command} timed out"),
                    "command": command,
                })
            }
            Self::BadConceptMapId(id) => {
                serde_json::json!({
                    "error": "bad_concept_map_id",
                    "message": format!("bad concept-map id: {id}"),
                })
            }
            Self::ConceptMapNotFound(id) => {
                serde_json::json!({
                    "error": "not_found",
                    "message": format!("concept map CM-{id:03} not found"),
                })
            }
            Self::DuplicateEdge { line } => {
                serde_json::json!({
                    "error": "duplicate_edge",
                    "message": format!("edge already exists at line {line}"),
                    "line": line,
                })
            }
            Self::EdgeNotFound => {
                serde_json::json!({
                    "error": "edge_not_found",
                    "message": "edge not found",
                })
            }
            Self::NodeCollision(existing_label, line) => {
                serde_json::json!({
                    "error": "node_collision",
                    "message": format!("rename would collide with existing node '{existing_label}' at line {line}"),
                    "existing_label": existing_label,
                    "line": line,
                })
            }
            Self::EmptyField(field) => {
                serde_json::json!({
                    "error": "empty_field",
                    "message": format!("{field} must be non-empty"),
                })
            }
            Self::StaleConceptMap => {
                serde_json::json!({
                    "error": "stale_concept_map",
                    "message": "concept map was modified since last read; refresh and retry",
                })
            }
            Self::ConceptMapParseError(msg) => {
                serde_json::json!({
                    "error": "concept_map_parse_error",
                    "message": msg,
                })
            }
            Self::ConceptMapIoError(msg) => {
                serde_json::json!({
                    "error": "concept_map_io_error",
                    "message": msg,
                })
            }
            Self::Other(err) => {
                serde_json::json!({
                    "error": "other",
                    "message": format!("{err:#}"),
                })
            }
        };

        let status = status_code(&self);
        (status, Json(body)).into_response()
    }
}

/// Map each variant to its HTTP status code.
fn status_code(err: &MapServerError) -> StatusCode {
    match err {
        MapServerError::BadEntityId(_)
        | MapServerError::BadConceptMapId(_)
        | MapServerError::EmptyField(_) => StatusCode::BAD_REQUEST,
        MapServerError::EntityNotFound(_)
        | MapServerError::AssetNotFound(_)
        | MapServerError::ConceptMapNotFound(_)
        | MapServerError::EdgeNotFound => StatusCode::NOT_FOUND,
        MapServerError::BodyTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        MapServerError::CommandFailed { .. } => StatusCode::UNPROCESSABLE_ENTITY,
        MapServerError::DuplicateEdge { .. }
        | MapServerError::NodeCollision(..)
        | MapServerError::StaleConceptMap => StatusCode::CONFLICT,
        MapServerError::MarkdownNotImplemented(_) => StatusCode::NOT_IMPLEMENTED,
        MapServerError::ToolUnavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
        MapServerError::Timeout { .. } => StatusCode::GATEWAY_TIMEOUT,
        MapServerError::ConceptMapParseError(_)
        | MapServerError::ConceptMapIoError(_)
        | MapServerError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

impl From<ConceptMapMutationError> for MapServerError {
    fn from(e: ConceptMapMutationError) -> Self {
        match e {
            ConceptMapMutationError::EmptyField(field) => Self::EmptyField(field),
            ConceptMapMutationError::DuplicateEdge { line } => Self::DuplicateEdge { line },
            ConceptMapMutationError::EdgeNotFound => Self::EdgeNotFound,
            ConceptMapMutationError::NodeCollision {
                existing_label,
                line,
            } => Self::NodeCollision(existing_label, line),
            ConceptMapMutationError::MissingDsl => {
                Self::ConceptMapParseError("TOML is missing a `dsl` key".into())
            }
            ConceptMapMutationError::InvalidToml(msg) => Self::ConceptMapParseError(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct ErrorResponse {
        error: String,
        message: String,
        #[serde(default)]
        command: Option<String>,
        #[serde(default)]
        status: Option<i32>,
        #[serde(default)]
        stderr: Option<String>,
        #[serde(default)]
        tool: Option<String>,
    }

    async fn into_error_body(response: axum::response::Response) -> (StatusCode, ErrorResponse) {
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let body: ErrorResponse = serde_json::from_slice(&bytes).unwrap();
        (status, body)
    }

    #[tokio::test]
    async fn bad_entity_id_400() {
        let err = MapServerError::BadEntityId("garbage".into());
        let (status, body) = into_error_body(err.into_response()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body.error, "bad_entity_id");
        assert!(body.message.contains("garbage"));
    }

    #[tokio::test]
    async fn entity_not_found_404() {
        let err = MapServerError::EntityNotFound("SL-999".into());
        let (status, body) = into_error_body(err.into_response()).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body.error, "entity_not_found");
        assert!(body.message.contains("SL-999"));
    }

    #[tokio::test]
    async fn asset_not_found_404() {
        let err = MapServerError::AssetNotFound("icon.png".into());
        let (status, body) = into_error_body(err.into_response()).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body.error, "asset_not_found");
        assert!(body.message.contains("icon.png"));
    }

    #[tokio::test]
    async fn markdown_not_implemented_501() {
        let err = MapServerError::MarkdownNotImplemented("slice");
        let (status, body) = into_error_body(err.into_response()).await;
        assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
        assert_eq!(body.error, "markdown_not_implemented");
        assert!(body.message.contains("slice"));
    }

    #[tokio::test]
    async fn body_too_large_413() {
        let err = MapServerError::BodyTooLarge;
        let (status, body) = into_error_body(err.into_response()).await;
        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(body.error, "body_too_large");
    }

    #[tokio::test]
    async fn tool_unavailable_503() {
        let err = MapServerError::ToolUnavailable { tool: "graphviz" };
        let (status, body) = into_error_body(err.into_response()).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body.error, "tool_unavailable");
        assert_eq!(body.tool.as_deref(), Some("graphviz"));
    }

    #[tokio::test]
    async fn command_failed_422() {
        let err = MapServerError::CommandFailed {
            command: "dot",
            status: Some(1),
            stderr: "syntax error".into(),
        };
        let (status, body) = into_error_body(err.into_response()).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body.error, "command_failed");
        assert_eq!(body.command.as_deref(), Some("dot"));
        assert_eq!(body.status, Some(1));
        assert_eq!(body.stderr.as_deref(), Some("syntax error"));
    }

    #[tokio::test]
    async fn timeout_504() {
        let err = MapServerError::Timeout { command: "dot" };
        let (status, body) = into_error_body(err.into_response()).await;
        assert_eq!(status, StatusCode::GATEWAY_TIMEOUT);
        assert_eq!(body.error, "timeout");
        assert_eq!(body.command.as_deref(), Some("dot"));
    }

    #[tokio::test]
    async fn other_500() {
        let err = MapServerError::Other(anyhow::anyhow!("bang"));
        let (status, body) = into_error_body(err.into_response()).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body.error, "other");
        assert!(body.message.contains("bang"));
    }

    #[tokio::test]
    async fn stderr_truncation_at_8kib_boundary() {
        let long_stderr = "x".repeat(9 * 1024); // 9 KiB
        let err = MapServerError::CommandFailed {
            command: "dot",
            status: Some(1),
            stderr: long_stderr,
        };
        let (_status, body) = into_error_body(err.into_response()).await;
        let stderr_out = body.stderr.as_deref().unwrap();
        // Must be exactly 8 KiB (8192 bytes) — the cap.
        assert_eq!(stderr_out.len(), STDERR_CAP);
    }

    #[tokio::test]
    async fn stderr_below_cap_untouched() {
        let short = "short error".to_string();
        let len = short.len();
        let err = MapServerError::CommandFailed {
            command: "dot",
            status: None,
            stderr: short,
        };
        let (_status, body) = into_error_body(err.into_response()).await;
        assert_eq!(body.stderr.as_deref().unwrap().len(), len);
    }
}
