// SPDX-License-Identifier: GPL-3.0-only
//! MCP tool definitions (JSON Schema) and handler dispatch.
//!
//! Each tool calls the matching `review::run_*` function, maps errors through
//! `ReviewError` variant identity (design D8, §5), and returns the
//! `ReviewOutput` as JSON text.

use super::protocol::{
    Id, JsonRpcRequest, JsonRpcResponse, McpTool, McpToolResult, ToolsListResult,
};
use crate::review::{self, NewArgs, PrimeArgs, ReviewOutput};
use anyhow::Context;
use serde_json::{Value, json};
use std::path::Path;

// ── Tool definitions (function, not const — json!() is non-const) ─────────

/// Return all 10 tool definitions with JSON Schema parameter descriptions.
fn tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "review_new".to_owned(),
            description: "Open a new adversarial review ledger targeting an entity via the `reviews` edge.".to_owned(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "facet": {
                        "type": "string",
                        "description": "What this review reviews: scope | design | plan | phase-plan | implementation | code-review | reconciliation",
                        "enum": ["scope", "design", "plan", "phase-plan", "implementation", "code-review", "reconciliation"]
                    },
                    "target": {
                        "type": "string",
                        "description": "The subject canonical ref the review targets, e.g. SL-024"
                    },
                    "phase": {
                        "type": "string",
                        "description": "Optional phase scope, e.g. PHASE-03"
                    },
                    "title": {
                        "type": "string",
                        "description": "Review title (default: derived from facet + target)"
                    },
                    "raiser": {
                        "type": "string",
                        "description": "Raiser role label (default: raiser)"
                    },
                    "responder": {
                        "type": "string",
                        "description": "Responder role label (default: responder)"
                    }
                },
                "required": ["facet", "target"]
            }),
        },
        McpTool {
            name: "review_list".to_owned(),
            description: "List reviews by id with derived status, facet, target, and title.".to_owned(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "substr": { "type": "string", "description": "Case-insensitive substring filter over slug + title" },
                    "regexp": { "type": "string", "description": "Regex over canonical-id + slug + title" },
                    "status": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Filter by status: active | done"
                    },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Tag filter (OR within the axis)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Opt-in cap: return at most this many rows (default: all)"
                    }
                },
                "required": []
            }),
        },
        McpTool {
            name: "review_show".to_owned(),
            description: "Show one review: derived status, the reviews edge, and the brief.".to_owned(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "reference": { "type": "string", "description": "Review reference: RV-007 or the bare id 7" },
                    "format": { "type": "string", "enum": ["table", "json"], "description": "Output format (default: table)" },
                    "view": { "type": "string", "enum": ["full", "summary"], "description": "summary drops the brief body + per-finding prose, keeping the finding skeleton (default: full)" }
                },
                "required": ["reference"]
            }),
        },
        McpTool {
            name: "review_raise".to_owned(),
            description: "Raise a finding on a review (the raiser's verb) — appends an open finding with fixed severity/title/detail.".to_owned(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "reference": { "type": "string", "description": "Review reference: RV-007 or the bare id 7" },
                    "severity": { "type": "string", "enum": ["blocker", "major", "minor", "nit"], "description": "Severity (only blocker gates close)" },
                    "title": { "type": "string", "description": "The finding's title (fixed at raise)" },
                    "detail": { "type": "string", "description": "The finding's detail (fixed at raise)" },
                    "as": { "type": "string", "description": "Cooperative role assertion (default: raiser)" }
                },
                "required": ["reference", "severity", "title", "detail"]
            }),
        },
        McpTool {
            name: "review_dispose".to_owned(),
            description: "Dispose a finding (the responder's verb) — answer an open/contested finding, setting disposition + response.".to_owned(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "reference": { "type": "string", "description": "Review reference: RV-007 or the bare id 7" },
                    "finding": { "type": "string", "description": "The finding id, e.g. F-2" },
                    "disposition": { "type": "string", "description": "The disposition: fixed | design-wrong | tolerated" },
                    "response": { "type": "string", "description": "The response detail (free-text)" },
                    "as": { "type": "string", "description": "Cooperative role assertion (default: responder)" }
                },
                "required": ["reference", "finding", "disposition", "response"]
            }),
        },
        McpTool {
            name: "review_verify".to_owned(),
            description: "Verify an answered finding (the raiser's verb) — accept it (terminal).".to_owned(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "reference": { "type": "string", "description": "Review reference: RV-007 or the bare id 7" },
                    "finding": { "type": "string", "description": "The finding id, e.g. F-2" },
                    "note": { "type": "string", "description": "Ephemeral handoff chatter for the baton log" },
                    "as": { "type": "string", "description": "Cooperative role assertion (default: raiser)" }
                },
                "required": ["reference", "finding"]
            }),
        },
        McpTool {
            name: "review_contest".to_owned(),
            description: "Contest an answered finding (the raiser's verb) — hand it back to the responder.".to_owned(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "reference": { "type": "string", "description": "Review reference: RV-007 or the bare id 7" },
                    "finding": { "type": "string", "description": "The finding id, e.g. F-2" },
                    "note": { "type": "string", "description": "Ephemeral handoff chatter for the baton log" },
                    "as": { "type": "string", "description": "Cooperative role assertion (default: raiser)" }
                },
                "required": ["reference", "finding"]
            }),
        },
        McpTool {
            name: "review_withdraw".to_owned(),
            description: "Withdraw a finding (the raiser's verb) — retract an open/answered finding (terminal).".to_owned(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "reference": { "type": "string", "description": "Review reference: RV-007 or the bare id 7" },
                    "finding": { "type": "string", "description": "The finding id, e.g. F-2" },
                    "as": { "type": "string", "description": "Cooperative role assertion (default: raiser)" }
                },
                "required": ["reference", "finding"]
            }),
        },
        McpTool {
            name: "review_status".to_owned(),
            description: "Report a review's derived state and rebuild its baton (cache == recompute).".to_owned(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "reference": { "type": "string", "description": "Review reference: RV-007 or the bare id 7" }
                },
                "required": ["reference"]
            }),
        },
        McpTool {
            name: "review_prime".to_owned(),
            description: "Populate the reviewer-context warm-cache from a curated domain_map, or (--seed) emit git-changed candidate paths.".to_owned(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "reference": { "type": "string", "description": "Review reference: RV-007 or the bare id 7" },
                    "seed": { "type": "boolean", "description": "Emit git-changed candidate paths (a starting point, not authority) and exit instead of priming" },
                    "from": { "type": "string", "description": "Read the curated domain_map from a file (default: stdin)" }
                },
                "required": ["reference"]
            }),
        },
    ]
}

// ── Public API ───────────────────────────────────────────────────────────

/// Return the full tool list for `tools/list`.
pub(crate) fn tool_list() -> ToolsListResult {
    ToolsListResult { tools: tools() }
}

/// Dispatch a JSON-RPC request to the matching handler.
///
/// Returns a proper JSON-RPC error response on unknown methods or validation
/// failures (never an `anyhow::Error` for recoverable dispatch problems).
pub(crate) fn dispatch(request: &JsonRpcRequest, root: &Path) -> JsonRpcResponse {
    let id = request.id.clone();
    match request.method.as_str() {
        "initialize" => handle_initialize(id),
        "tools/list" => handle_tools_list(id),
        "tools/call" => handle_tools_call(id, request.params.as_ref(), root),
        "notifications/initialized" => JsonRpcResponse::success(id, json!({})),
        _ => JsonRpcResponse::error(
            id,
            -32601,
            format!("Method not found: {}", request.method),
            Some(json!({ "method": request.method })),
        ),
    }
}

// ── Method handlers ──────────────────────────────────────────────────────

fn handle_initialize(id: Option<Id>) -> JsonRpcResponse {
    let result = serde_json::to_value(super::protocol::InitializeResult {
        capabilities: super::protocol::Capabilities {
            tools: super::protocol::ToolsCap {},
        },
        protocol_version: "2024-11-05".to_owned(),
        server_info: super::protocol::ServerInfo {
            name: "doctrine-mcp".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
        },
    });
    match result {
        Ok(val) => JsonRpcResponse::success(id, val),
        Err(e) => JsonRpcResponse::error(
            id,
            -32603,
            "Internal error".to_owned(),
            Some(json!({ "message": e.to_string() })),
        ),
    }
}

fn handle_tools_list(id: Option<Id>) -> JsonRpcResponse {
    let result =
        serde_json::to_value(tool_list()).unwrap_or_else(|e| json!({ "error": e.to_string() }));
    JsonRpcResponse::success(id, result)
}

fn handle_tools_call(id: Option<Id>, params: Option<&Value>, root: &Path) -> JsonRpcResponse {
    match call_tool(id.clone(), params, root) {
        Ok(out) => {
            let json_text = serde_json::to_string(&out)
                .unwrap_or_else(|e| json!({"serialization_error": e.to_string()}).to_string());
            let tool_result = McpToolResult::text(json_text);
            let result_val = serde_json::to_value(&tool_result)
                .unwrap_or_else(|e| json!({"error": e.to_string()}));
            JsonRpcResponse::success(id, result_val)
        }
        Err(e) => map_review_error(id, &e),
    }
}

/// Inner function that can use `?` for clean error propagation.
fn call_tool(_id: Option<Id>, params: Option<&Value>, root: &Path) -> anyhow::Result<ReviewOutput> {
    let params = params.context("params is required for tools/call")?;

    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .context("missing 'name' field in tools/call params")?;

    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);

    match name {
        "review_new" => {
            let args: NewArgs = serde_json::from_value(arguments)
                .map_err(|e| anyhow::anyhow!("invalid arguments: {e:#}"))?;
            review::run_new(Some(root.to_path_buf()), &args)
        }
        "review_list" => {
            // Hand-extract the optional filter axes (matching the other read verbs)
            // rather than serde-deserializing the clap-mirror `ListArgs`, whose
            // non-`Option` fields are serde-required and reject every MCP call (ISS-033).
            let fields = ExtractFields::from_value(arguments, &[]);
            let args = crate::listing::ListArgs {
                substr: fields.opt_str_field("substr"),
                regexp: fields.opt_str_field("regexp"),
                status: fields.vec_str_field("status"),
                tags: fields.vec_str_field("tags"),
                ..Default::default()
            };
            let limit = fields.opt_usize_field("limit");
            review::run_list(Some(root.to_path_buf()), args)
                .map(|out| project_list_limit(out, limit))
        }
        "review_show" => {
            let reference = arguments
                .get("reference")
                .and_then(|v| v.as_str())
                .map(str::to_owned)
                .unwrap_or_default();
            let format = arguments
                .get("format")
                .and_then(|v| v.as_str())
                .map(str::to_owned);
            let fmt = match format.as_deref() {
                Some("json") => crate::listing::Format::Json,
                _ => crate::listing::Format::Table,
            };
            let summary = arguments.get("view").and_then(|v| v.as_str()) == Some("summary");
            review::run_show(Some(root.to_path_buf()), &reference, fmt).map(|out| {
                if summary {
                    project_show_summary(out)
                } else {
                    out
                }
            })
        }
        "review_raise" => {
            let args: review::RaiseArgs = serde_json::from_value(arguments.clone())
                .map_err(|e| anyhow::anyhow!("invalid arguments: {e:#}"))?;
            let role_str = arguments.get("as").and_then(|v| v.as_str());
            let role =
                review::parse_role(role_str, review::Role::Raiser).context("invalid role")?;
            review::run_raise(Some(root.to_path_buf()), &args, role)
        }
        "review_dispose" => {
            let args: review::DisposeArgs = serde_json::from_value(arguments.clone())
                .map_err(|e| anyhow::anyhow!("invalid arguments: {e:#}"))?;
            let role_str = arguments.get("as").and_then(|v| v.as_str());
            let role =
                review::parse_role(role_str, review::Role::Responder).context("invalid role")?;
            review::run_dispose(Some(root.to_path_buf()), &args, role)
        }
        "review_verify" => {
            let fields = ExtractFields::from_value(arguments, &["reference", "finding"]);
            let role_str = fields.opt_str_field("as");
            let role = review::parse_role(role_str.as_deref(), review::Role::Raiser)
                .context("invalid role")?;
            review::run_verify(
                Some(root.to_path_buf()),
                &fields.str_field("reference"),
                &fields.str_field("finding"),
                fields.opt_str_field("note").as_deref(),
                role,
            )
        }
        "review_contest" => {
            let fields = ExtractFields::from_value(arguments, &["reference", "finding"]);
            let role_str = fields.opt_str_field("as");
            let role = review::parse_role(role_str.as_deref(), review::Role::Raiser)
                .context("invalid role")?;
            review::run_contest(
                Some(root.to_path_buf()),
                &fields.str_field("reference"),
                &fields.str_field("finding"),
                fields.opt_str_field("note").as_deref(),
                role,
            )
        }
        "review_withdraw" => {
            let fields = ExtractFields::from_value(arguments, &["reference", "finding"]);
            let role_str = fields.opt_str_field("as");
            let role = review::parse_role(role_str.as_deref(), review::Role::Raiser)
                .context("invalid role")?;
            review::run_withdraw(
                Some(root.to_path_buf()),
                &fields.str_field("reference"),
                &fields.str_field("finding"),
                role,
            )
        }
        "review_status" => {
            let fields = ExtractFields::from_value(arguments, &["reference"]);
            review::run_status(Some(root.to_path_buf()), &fields.str_field("reference"))
        }
        "review_prime" => {
            let args: PrimeArgs = serde_json::from_value(arguments)
                .map_err(|e| anyhow::anyhow!("invalid arguments: {e:#}"))?;
            review::run_prime(Some(root.to_path_buf()), &args)
        }
        _ => anyhow::bail!("Tool not found: {name}"),
    }
}

// ── Small helper: extract string fields from a JSON value ────────────────

struct ExtractFields {
    inner: Value,
}

impl ExtractFields {
    fn from_value(inner: Value, _required: &[&str]) -> Self {
        Self { inner }
    }

    fn str_field(&self, name: &str) -> String {
        self.inner
            .get(name)
            .and_then(|v| v.as_str())
            .map(str::to_owned)
            .unwrap_or_default()
    }

    fn opt_str_field(&self, name: &str) -> Option<String> {
        self.inner
            .get(name)
            .and_then(|v| v.as_str())
            .map(str::to_owned)
    }

    /// Extract a string array (missing or non-array ⇒ empty vec; non-string
    /// members dropped). Mirrors the missing-tolerant `*_str_field` posture.
    fn vec_str_field(&self, name: &str) -> Vec<String> {
        self.inner
            .get(name)
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Extract an optional unsigned integer (missing or non-integer ⇒ `None`),
    /// narrowed to `usize`. Used for the `review_list` `limit` cap.
    fn opt_usize_field(&self, name: &str) -> Option<usize> {
        self.inner
            .get(name)
            .and_then(serde_json::Value::as_u64)
            .and_then(|n| usize::try_from(n).ok())
    }
}

/// Trim a `Showed` output to its summary projection (IMP-113 #2): blank the brief
/// `body` and each finding's `detail`/`response` prose, keeping the finding
/// skeleton (id / status / severity / title / disposition). Non-`Showed` outputs
/// pass through. Applied MCP-side; the `run_show` engine is untouched.
fn project_show_summary(out: ReviewOutput) -> ReviewOutput {
    match out {
        ReviewOutput::Showed {
            id,
            canonical,
            title,
            status,
            awaiting,
            facet,
            target,
            findings_count,
            findings,
            body: _,
            formatted,
        } => {
            let findings = findings
                .into_iter()
                .map(|f| review::Finding {
                    detail: String::new(),
                    response: None,
                    ..f
                })
                .collect();
            ReviewOutput::Showed {
                id,
                canonical,
                title,
                status,
                awaiting,
                facet,
                target,
                findings_count,
                findings,
                body: String::new(),
                formatted,
            }
        }
        other => other,
    }
}

/// Truncate a `Listed` output's rows to an opt-in `limit` (IMP-113 #3). A `None`
/// limit (the default) passes everything through — the cap is agent-requested,
/// never a silent engine cap. Non-`Listed` outputs pass through.
fn project_list_limit(out: ReviewOutput, limit: Option<usize>) -> ReviewOutput {
    match (out, limit) {
        (
            ReviewOutput::Listed {
                mut rows,
                formatted,
            },
            Some(n),
        ) => {
            rows.truncate(n);
            ReviewOutput::Listed { rows, formatted }
        }
        (other, _) => other,
    }
}

// ── Error mapping (design §5) ────────────────────────────────────────────

/// Map an `anyhow::Error` from a review verb to a JSON-RPC error response.
///
/// Downcasts to `ReviewError` by variant identity — never by string-parsing
/// (design D8). Unmatched errors fall through as `Internal`.
fn map_review_error(id: Option<Id>, err: &anyhow::Error) -> JsonRpcResponse {
    let msg = err.to_string();

    // Tool not found → -32601 (detected before the ReviewError downcast path)
    if let Some(name) = msg.strip_prefix("Tool not found: ") {
        let tool_name = name.to_owned();
        return JsonRpcResponse::error(id, -32601, msg, Some(json!({ "name": tool_name })));
    }

    if msg.starts_with("invalid arguments:") {
        return JsonRpcResponse::error(
            id,
            -32602,
            "Invalid params".to_owned(),
            Some(json!({ "parse_error": msg })),
        );
    }

    // Downcast to ReviewError by variant identity
    if let Some(re) = err.downcast_ref::<review::ReviewError>() {
        return match re {
            review::ReviewError::NotFound { reference } => JsonRpcResponse::error(
                id,
                -32000,
                "Review not found".to_owned(),
                Some(json!({
                    "code": "NOT_FOUND",
                    "reference": reference
                })),
            ),
            review::ReviewError::RoleMismatch {
                expected,
                actual,
                verb,
            } => JsonRpcResponse::error(
                id,
                -32602,
                format!(
                    "Role mismatch: {} is the {}'s verb, not the {}'s",
                    verb.as_str(),
                    expected.as_str(),
                    actual.as_str()
                ),
                Some(json!({
                    "code": "ROLE_MISMATCH",
                    "expected": expected.as_str(),
                    "actual": actual.as_str(),
                    "verb": verb.as_str()
                })),
            ),
            review::ReviewError::StateMismatch {
                finding,
                current,
                required,
            } => JsonRpcResponse::error(
                id,
                -32602,
                format!(
                    "State mismatch on {finding}: current {} != required {}",
                    current.as_str(),
                    required.as_str()
                ),
                Some(json!({
                    "code": "STATE_MISMATCH",
                    "finding": finding,
                    "current": current.as_str(),
                    "required": required.as_str()
                })),
            ),
            review::ReviewError::DanglingRef { target } => JsonRpcResponse::error(
                id,
                -32000,
                format!("Target not found: {target}"),
                Some(json!({
                    "code": "DANGLING_REF",
                    "target": target
                })),
            ),
            review::ReviewError::LockContention { canonical, details } => JsonRpcResponse::error(
                id,
                -32000,
                format!("Lock contention: {canonical}: {details}"),
                Some(json!({
                    "code": "LOCK_CONTENTION",
                    "canonical": canonical,
                    "details": details
                })),
            ),
            review::ReviewError::Internal { source } => JsonRpcResponse::error(
                id,
                -32603,
                "Internal error".to_owned(),
                Some(json!({
                    "code": "INTERNAL",
                    "message": source.to_string()
                })),
            ),
        };
    }

    // Catch-all: unknown anyhow error → internal
    JsonRpcResponse::error(
        id,
        -32603,
        "Internal error".to_owned(),
        Some(json!({
            "code": "INTERNAL",
            "message": msg
        })),
    )
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::review::ReviewError;

    /// Helper: create a temp root dir with the markers needed by `root::find`.
    fn temp_root() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let _ = std::fs::create_dir_all(root.join(".git"));
        let _ = std::fs::create_dir_all(root.join(".doctrine").join("review"));
        (dir, root)
    }

    /// Helper: create a test JsonRpcRequest for tools/call.
    fn tools_call_req(name: &str, args: Value) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_owned(),
            id: Some(Id::Number(1)),
            method: "tools/call".to_owned(),
            params: Some(json!({
                "name": name,
                "arguments": args
            })),
        }
    }

    // VT-3: tool list response contains exactly 10 tools with correct names

    #[test]
    fn tool_list_has_10_tools() {
        let list = tool_list();
        assert_eq!(list.tools.len(), 10);
    }

    #[test]
    fn tool_list_names() {
        let list = tool_list();
        let names: Vec<&str> = list.tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"review_new"));
        assert!(names.contains(&"review_list"));
        assert!(names.contains(&"review_show"));
        assert!(names.contains(&"review_raise"));
        assert!(names.contains(&"review_dispose"));
        assert!(names.contains(&"review_verify"));
        assert!(names.contains(&"review_contest"));
        assert!(names.contains(&"review_withdraw"));
        assert!(names.contains(&"review_status"));
        assert!(names.contains(&"review_prime"));
    }

    // ISS-033: review_list must accept its advertised (all-optional) arg shapes —
    // empty `{}` and a `status` filter — rather than rejecting every call -32602.

    #[test]
    fn review_list_empty_args_succeeds() {
        let (_dir, root) = temp_root();
        let req = tools_call_req("review_list", json!({}));
        let resp = dispatch(&req, &root);
        assert!(
            resp.error.is_none(),
            "review_list {{}} errored: {:?}",
            resp.error
        );
        assert!(resp.result.is_some());
    }

    #[test]
    fn review_list_status_filter_succeeds() {
        let (_dir, root) = temp_root();
        let req = tools_call_req("review_list", json!({ "status": ["done"] }));
        let resp = dispatch(&req, &root);
        assert!(
            resp.error.is_none(),
            "review_list status filter errored: {:?}",
            resp.error
        );
        assert!(resp.result.is_some());
    }

    // IMP-113 #1: the human render cache must not ship on the MCP wire — `Listed`
    // and `Status` carry a `formatted` field the structured payload already covers.

    #[test]
    fn listed_and_status_omit_formatted_in_json() {
        let listed = ReviewOutput::Listed {
            rows: vec![],
            formatted: "RENDERED TABLE".to_owned(),
        };
        let v = serde_json::to_value(&listed).unwrap();
        assert!(v["Listed"].get("rows").is_some());
        assert!(
            v["Listed"].get("formatted").is_none(),
            "Listed leaked formatted: {v}"
        );

        let status = ReviewOutput::Status {
            canonical: "RV-1".to_owned(),
            status: "done".to_owned(),
            awaiting: "none".to_owned(),
            findings_count: 0,
            rounds: 0,
            cache_primed: true,
            stale_paths: vec![],
            formatted: "RENDERED STATUS".to_owned(),
        };
        let v = serde_json::to_value(&status).unwrap();
        assert!(
            v["Status"].get("formatted").is_none(),
            "Status leaked formatted: {v}"
        );
    }

    // IMP-113 #2: summary view drops the brief body + per-finding prose, keeps skeleton.

    #[test]
    fn project_show_summary_blanks_prose_keeps_skeleton() {
        let out = ReviewOutput::Showed {
            id: 1,
            canonical: "RV-1".to_owned(),
            title: "T".to_owned(),
            status: "done".to_owned(),
            awaiting: "none".to_owned(),
            facet: "reconciliation".to_owned(),
            target: "SL-1".to_owned(),
            findings_count: 1,
            findings: vec![sample_finding()],
            body: "BIG BRIEF BODY".to_owned(),
            formatted: String::new(),
        };
        let ReviewOutput::Showed { body, findings, .. } = project_show_summary(out) else {
            panic!("expected Showed");
        };
        assert!(body.is_empty(), "body should be blanked");
        assert_eq!(findings.len(), 1);
        assert!(
            findings[0].detail.is_empty(),
            "detail prose should be dropped"
        );
        assert!(
            findings[0].response.is_none(),
            "response prose should be dropped"
        );
        // skeleton retained
        assert_eq!(findings[0].title, "t");
        assert_eq!(findings[0].disposition.as_deref(), Some("tolerated"));
    }

    // IMP-113 #3: limit caps rows only when set (opt-in, never a silent cap).

    #[test]
    fn project_list_limit_caps_only_when_set() {
        let make = || ReviewOutput::Listed {
            rows: vec![row("RV-1"), row("RV-2"), row("RV-3")],
            formatted: String::new(),
        };
        let ReviewOutput::Listed { rows, .. } = project_list_limit(make(), Some(2)) else {
            panic!("expected Listed");
        };
        assert_eq!(rows.len(), 2);

        let ReviewOutput::Listed { rows, .. } = project_list_limit(make(), None) else {
            panic!("expected Listed");
        };
        assert_eq!(rows.len(), 3, "no limit passes everything through");
    }

    fn sample_finding() -> crate::review::Finding {
        crate::review::Finding {
            id: "F-1".to_owned(),
            status: crate::review::FindingStatus::Verified,
            severity: crate::review::Severity::Minor,
            title: "t".to_owned(),
            detail: "long detail prose".to_owned(),
            disposition: Some("tolerated".to_owned()),
            response: Some("long response prose".to_owned()),
        }
    }

    fn row(id: &str) -> crate::review::ListRow {
        crate::review::ListRow {
            id: id.to_owned(),
            status: "done".to_owned(),
            awaiting: "none".to_owned(),
            facet: "f".to_owned(),
            target: "t".to_owned(),
            title: "x".to_owned(),
        }
    }

    // VT-7: unknown tool name returns -32601

    #[test]
    fn unknown_tool_returns_32601() {
        let (_dir, root) = temp_root();
        let req = tools_call_req("nonexistent", json!({}));
        let resp = dispatch(&req, &root);
        let err = resp.error.unwrap();
        assert_eq!(err.code, -32601);
        assert!(err.message.contains("Tool not found"));
    }

    #[test]
    fn unknown_method_returns_32601() {
        let (_dir, root) = temp_root();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_owned(),
            id: Some(Id::Number(1)),
            method: "bad/method".to_owned(),
            params: None,
        };
        let resp = dispatch(&req, &root);
        let err = resp.error.unwrap();
        assert_eq!(err.code, -32601);
        assert!(err.message.contains("Method not found"));
    }

    // VT-5: ReviewError::RoleMismatch maps to -32602 with structured data payload

    #[test]
    fn role_mismatch_error_mapping() {
        let err = ReviewError::RoleMismatch {
            expected: crate::review::Role::Raiser,
            actual: crate::review::Role::Responder,
            verb: crate::review::Verb::Dispose,
        };
        let e = anyhow::anyhow!(err);
        let resp = map_review_error(Some(Id::Number(1)), &e);
        let err = resp.error.unwrap();
        assert_eq!(err.code, -32602);
        let data = err.data.unwrap();
        assert_eq!(data["code"], "ROLE_MISMATCH");
        assert_eq!(data["expected"], "raiser");
        assert_eq!(data["actual"], "responder");
        assert_eq!(data["verb"], "dispose");
    }

    // VT-6: ReviewError::NotFound maps to -32000 with NOT_FOUND code

    #[test]
    fn not_found_error_mapping() {
        let err = ReviewError::NotFound {
            reference: "RV-999".to_owned(),
        };
        let e = anyhow::anyhow!(err);
        let resp = map_review_error(Some(Id::Number(1)), &e);
        let err = resp.error.unwrap();
        assert_eq!(err.code, -32000);
        let data = err.data.unwrap();
        assert_eq!(data["code"], "NOT_FOUND");
        assert_eq!(data["reference"], "RV-999");
    }

    #[test]
    fn state_mismatch_error_mapping() {
        let err = ReviewError::StateMismatch {
            finding: "F-3".to_owned(),
            current: crate::review::FindingStatus::Verified,
            required: crate::review::FindingStatus::Open,
        };
        let e = anyhow::anyhow!(err);
        let resp = map_review_error(Some(Id::Number(1)), &e);
        let err = resp.error.unwrap();
        assert_eq!(err.code, -32602);
        let data = err.data.unwrap();
        assert_eq!(data["code"], "STATE_MISMATCH");
    }

    #[test]
    fn lock_contention_error_mapping() {
        let err = ReviewError::LockContention {
            canonical: "RV-001".to_owned(),
            details: "held by pid 12345".to_owned(),
        };
        let e = anyhow::anyhow!(err);
        let resp = map_review_error(Some(Id::Number(1)), &e);
        let err = resp.error.unwrap();
        assert_eq!(err.code, -32000);
        let data = err.data.unwrap();
        assert_eq!(data["code"], "LOCK_CONTENTION");
    }

    #[test]
    fn internal_error_mapping() {
        let err = ReviewError::Internal {
            source: anyhow::anyhow!("disk full"),
        };
        let e = anyhow::anyhow!(err);
        let resp = map_review_error(Some(Id::Number(1)), &e);
        let err = resp.error.unwrap();
        assert_eq!(err.code, -32603);
        let data = err.data.unwrap();
        assert_eq!(data["code"], "INTERNAL");
    }

    #[test]
    fn initialize_response() {
        let resp = handle_initialize(Some(Id::Number(1)));
        let result = resp.result.unwrap();
        assert_eq!(result["protocolVersion"], "2024-11-05");
        assert_eq!(result["capabilities"]["tools"], json!({}));
        assert_eq!(result["serverInfo"]["name"], "doctrine-mcp");
    }

    #[test]
    fn notification_initialized_returns_empty() {
        let (_dir, root) = temp_root();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_owned(),
            id: None,
            method: "notifications/initialized".to_owned(),
            params: None,
        };
        let resp = dispatch(&req, &root);
        assert!(resp.id.is_none());
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap(), json!({}));
    }

    #[test]
    fn tools_list_response_structure() {
        let (_dir, root) = temp_root();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_owned(),
            id: Some(Id::Number(1)),
            method: "tools/list".to_owned(),
            params: None,
        };
        let resp = dispatch(&req, &root);
        let result = resp.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 10);
    }

    #[test]
    fn review_raise_invalid_args_returns_32602() {
        let (_dir, root) = temp_root();
        let req = tools_call_req(
            "review_raise",
            json!({
                "reference": "1"
            }),
        );
        let resp = dispatch(&req, &root);
        let err = resp.error.unwrap();
        assert_eq!(err.code, -32602);
    }
}
