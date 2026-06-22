// SPDX-License-Identifier: GPL-3.0-only
//! MCP tool definitions (JSON Schema) and handler dispatch.
//!
//! 14 tools: 10 review + 4 memory (`memory_find`, `memory_retrieve`, `memory_show`,
//! `memory_list`). Each review tool calls the matching `review::run_*` function,
//! maps errors through `ReviewError` variant identity (design D8, §5), and
//! returns JSON text. Memory tools are defined but wired in PHASE-04.

use super::protocol::{
    Id, JsonRpcRequest, JsonRpcResponse, McpTool, McpToolResult, ToolsListResult,
};
use crate::memory;
use crate::retrieve;
use crate::review::{self, NewArgs, PrimeArgs, ReviewOutput};
use anyhow::Context;
use serde_json::{Value, json};
use std::path::Path;
use std::str::FromStr;

// ── Tool definitions (function, not const — json!() is non-const) ─────────

/// Return all 10 tool definitions with JSON Schema parameter descriptions.
fn tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "review_new".to_owned(),
            description: "Open a new adversarial review ledger targeting an entity via the `reviews` edge.\n\nReturns: { id: int, canonical: \"RV-NNN\", dir: string }".to_owned(),
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
            description: "List reviews by id with derived status, facet, target, and title.\n\nReturns: { rows: [{ id: \"RV-NNN\", status: \"active\"|\"done\", awaiting: \"raiser\"|\"responder\"|\"none\", facet: string, target: string, title: string }], total?: int } — `total` absent (not null) when uncapped; present (pre-truncation count) when rows were dropped by `limit`.".to_owned(),
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
                        "description": "Cap rows to the most recent N (default: 50; 0 = all). When capped, the response carries a `total` count."
                    }
                },
                "required": []
            }),
        },
        McpTool {
            name: "review_show".to_owned(),
            description: "Show one review: derived status, the reviews edge, and the brief.\n\nReturns: { id: int, canonical: \"RV-NNN\", title: string, status: \"active\"|\"done\", awaiting: \"raiser\"|\"responder\"|\"none\", facet: string, target: string, finding_count: int, findings: [{ id: \"F-N\", status: \"open\"|\"answered\"|\"contested\"|\"verified\"|\"withdrawn\", severity: \"blocker\"|\"major\"|\"minor\"|\"nit\", title: string, detail: string, disposition?: string|null, response?: string|null }], body: string } — `view=summary` blanks `body` → `\"\"`, each finding's `detail` → `\"\"` and `response` → `null`.".to_owned(),
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
            description: "Raise a finding on a review (the raiser's verb) — appends an open finding with fixed severity/title/detail.\n\nReturns: { finding_id: \"F-N\", review_id: int }".to_owned(),
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
            description: "Dispose a finding (the responder's verb) — answer an open/contested finding, setting disposition + response.\n\nReturns: { finding_id: \"F-N\", review_id: int }".to_owned(),
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
            description: "Verify an answered finding (the raiser's verb) — accept it (terminal).\n\nReturns: { finding_id: \"F-N\", review_id: int }".to_owned(),
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
            description: "Contest an answered finding (the raiser's verb) — hand it back to the responder.\n\nReturns: { finding_id: \"F-N\", review_id: int }".to_owned(),
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
            description: "Withdraw a finding (the raiser's verb) — retract an open/answered finding (terminal).\n\nReturns: { finding_id: \"F-N\", review_id: int }".to_owned(),
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
            description: "Report a review's derived state and rebuild its baton (cache == recompute).\n\nReturns: { canonical: \"RV-NNN\", status: \"active\"|\"done\", awaiting: \"raiser\"|\"responder\"|\"none\", findings_count: int, rounds: int, cache_primed: bool, stale_paths: [string] } — `rounds` counts baton handoffs; `cache_primed` is the prime-cache freshness signal, never a gate; `stale_paths` lists paths whose git-sha diverged since prime.".to_owned(),
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
            description: "Populate the reviewer-context warm-cache from a curated domain_map, or (--seed) emit git-changed candidate paths. Normal prime persists the domain_map and returns `{ canonical: \"RV-NNN\", tracked_paths: [string], areas_count: int, tracked_count: int, invariants_count: int, risks_count: int }`; `--seed` emits git-changed candidates (write-nothing) with the same shape but counts zero.".to_owned(),
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
        McpTool {
            name: "memory_find".to_owned(),
            description: "Discovery tool — metadata only, no bodies. Use first to probe context. Holdback-exempt: rows may include memories suppressed by `memory_retrieve`. Do not treat high-risk rows as consumable knowledge; use `memory_show` for inspection then `memory_retrieve` for safe recall. Requires at least one selector or defaults to 20-row cap.\n\nReturns: { kind: 'memory_find', rows: [{ uid, key?, type, status, staleness, trust, severity, spec, title, held_back_on_retrieve }], total: int, offset: int, limit: int, next_offset: int|null }".to_owned(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Free-text search query" },
                    "path_scope": { "type": "array", "items": { "type": "string" }, "description": "Limit results to memories scoped to these paths" },
                    "glob": { "type": "array", "items": { "type": "string" }, "description": "Limit results to memories scoped to these glob patterns" },
                    "command": { "type": "array", "items": { "type": "string" }, "description": "Limit results to memories scoped to these commands" },
                    "tag": { "type": "array", "items": { "type": "string" }, "description": "Limit results to memories with these tags" },
                    "type": { "type": "string", "enum": ["concept", "fact", "pattern", "signpost", "system", "thread"], "description": "Filter by memory type" },
                    "status": { "type": "string", "enum": ["active", "draft", "superseded", "retracted", "archived", "quarantined"], "description": "Filter by memory status" },
                    "lifespan": { "type": "string", "enum": ["semantic", "episodic", "procedural", "working", "identity"], "description": "Filter by lifespan threshold" },
                    "include_draft": { "type": "boolean", "description": "Include draft memories in results (default: false)" },
                    "offset": { "type": "integer", "description": "Pagination offset (default: 0)" },
                    "limit": { "type": "integer", "description": "Max rows to return (no-selector default: 20; 0 rejected)" }
                },
                "required": []
            }),
        },
        McpTool {
            name: "memory_retrieve".to_owned(),
            description: "Agent-context recall with trust holdback. Returns security-framed data blocks (nonce + staleness + attribution). Low-trust ∧ high-severity memories are suppressed. Use after `memory_find` identified relevant candidates. Supply `reference` for single-memory recall through holdback.\n\nReturns: framed text blocks (mem_… header + body), one per recalled memory.".to_owned(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "reference": { "type": "string", "description": "Recall a single memory by uid or key (mutually exclusive with query/scope probes)" },
                    "query": { "type": "string", "description": "Free-text search query (mutually exclusive with reference)" },
                    "path_scope": { "type": "array", "items": { "type": "string" }, "description": "Limit results to memories scoped to these paths" },
                    "glob": { "type": "array", "items": { "type": "string" }, "description": "Limit results to memories scoped to these glob patterns" },
                    "command": { "type": "array", "items": { "type": "string" }, "description": "Limit results to memories scoped to these commands" },
                    "tag": { "type": "array", "items": { "type": "string" }, "description": "Limit results to memories with these tags" },
                    "type": { "type": "string", "enum": ["concept", "fact", "pattern", "signpost", "system", "thread"], "description": "Filter by memory type" },
                    "status": { "type": "string", "enum": ["active", "draft", "superseded", "retracted", "archived", "quarantined"], "description": "Filter by memory status" },
                    "lifespan": { "type": "string", "enum": ["semantic", "episodic", "procedural", "working", "identity"], "description": "Filter by lifespan threshold" },
                    "include_draft": { "type": "boolean", "description": "Include draft memories in results (default: false)" },
                    "offset": { "type": "integer", "description": "Pagination offset (default: 0)" },
                    "limit": { "type": "integer", "description": "Max results (default: 5, capped at 20; 0 rejected)" },
                    "min_trust": { "type": "string", "enum": ["high", "medium", "low"], "description": "Trust floor (default: medium)" }
                },
                "required": []
            }),
        },
        McpTool {
            name: "memory_show".to_owned(),
            description: "Full memory inspection — header, body, relations, wikilinks, backlinks. Use only after selecting an exact uid via `memory_find`. For token efficiency, use `view: summary` to skip body, or `include_body: false`. Held-back memories (field `held_back_on_retrieve: true`) are shown with a metadata warning; do not treat as consumable knowledge.\n\nReturns: { memory: { uid, key?, title, type, status, trust, severity, body?, consumable, held_back_on_retrieve, backlinks: [{ uid, title, type, method }], backlinks_total: int } }".to_owned(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "reference": { "type": "string", "description": "Memory reference by uid or key" },
                    "view": { "type": "string", "enum": ["summary", "full"], "description": "summary skips body (default: summary)" },
                    "include_body": { "type": "boolean", "description": "Include body text in result (default: true)" },
                    "backlinks_limit": { "type": "integer", "description": "Max backlinks to return (default: 20, 0 = unlimited)" }
                },
                "required": ["reference"]
            }),
        },
        McpTool {
            name: "memory_list".to_owned(),
            description: "Browse/index only — all memories, newest first, capped at 50 by default. Prefer scoped `memory_find` for targeted discovery.\n\nReturns: { kind: 'memory', rows: [{ uid, type, status, trust, key?, title }], total: int, offset: int, limit: int, next_offset: int|null }".to_owned(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "type": { "type": "string", "enum": ["concept", "fact", "pattern", "signpost", "system", "thread"], "description": "Filter by memory type" },
                    "substr": { "type": "string", "description": "Case-insensitive substring filter over key + title" },
                    "status": { "type": "array", "items": { "type": "string" }, "description": "Filter by status values" },
                    "tag": { "type": "array", "items": { "type": "string" }, "description": "Tag filter (OR within the axis)" },
                    "limit": { "type": "integer", "description": "Max rows (default: 50; 0 = all)" },
                    "offset": { "type": "integer", "description": "Pagination offset (default: 0)" }
                },
                "required": []
            }),
        },
        McpTool {
            name: "memory_validate".to_owned(),
            description: "Run advisory validation checks on memories — dangling relations, stale verification, draft expiry. Returns a findings list; non-empty means warnings exist.".to_owned(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "reference": { "type": "string", "description": "Optional memory reference by uid or key; omit to validate all memories" },
                    "path": { "type": "string", "description": "Explicit project root (default: auto-detect)" }
                },
                "required": []
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
            let tool_result = McpToolResult::text(out);
            let result_val = serde_json::to_value(&tool_result)
                .unwrap_or_else(|e| json!({"error": e.to_string()}));
            JsonRpcResponse::success(id, result_val)
        }
        Err(e) => map_review_error(id, &e),
    }
}

/// Inner function that can use `?` for clean error propagation.
fn call_tool(_id: Option<Id>, params: Option<&Value>, root: &Path) -> anyhow::Result<String> {
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
            let out = review::run_new(Some(root.to_path_buf()), &args)?;
            Ok(serde_json::to_string(&out)?)
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
            let cap = effective_cap(fields.opt_usize_field("limit"));
            let out = review::run_list(Some(root.to_path_buf()), args)
                .map(|out| project_list_cap(out, cap))?;
            Ok(serde_json::to_string(&out)?)
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
            let out = review::run_show(Some(root.to_path_buf()), &reference, fmt).map(|out| {
                if summary {
                    project_show_summary(out)
                } else {
                    out
                }
            })?;
            Ok(serde_json::to_string(&out)?)
        }
        "review_raise" => {
            let args: review::RaiseArgs = serde_json::from_value(arguments.clone())
                .map_err(|e| anyhow::anyhow!("invalid arguments: {e:#}"))?;
            let role_str = arguments.get("as").and_then(|v| v.as_str());
            let role =
                review::parse_role(role_str, review::Role::Raiser).context("invalid role")?;
            let out = review::run_raise(Some(root.to_path_buf()), &args, role)?;
            Ok(serde_json::to_string(&out)?)
        }
        "review_dispose" => {
            let args: review::DisposeArgs = serde_json::from_value(arguments.clone())
                .map_err(|e| anyhow::anyhow!("invalid arguments: {e:#}"))?;
            let role_str = arguments.get("as").and_then(|v| v.as_str());
            let role =
                review::parse_role(role_str, review::Role::Responder).context("invalid role")?;
            let out = review::run_dispose(Some(root.to_path_buf()), &args, role)?;
            Ok(serde_json::to_string(&out)?)
        }
        "review_verify" => {
            let fields = ExtractFields::from_value(arguments, &["reference", "finding"]);
            let role_str = fields.opt_str_field("as");
            let role = review::parse_role(role_str.as_deref(), review::Role::Raiser)
                .context("invalid role")?;
            let out = review::run_verify(
                Some(root.to_path_buf()),
                &fields.str_field("reference"),
                &fields.str_field("finding"),
                fields.opt_str_field("note").as_deref(),
                role,
            )?;
            Ok(serde_json::to_string(&out)?)
        }
        "review_contest" => {
            let fields = ExtractFields::from_value(arguments, &["reference", "finding"]);
            let role_str = fields.opt_str_field("as");
            let role = review::parse_role(role_str.as_deref(), review::Role::Raiser)
                .context("invalid role")?;
            let out = review::run_contest(
                Some(root.to_path_buf()),
                &fields.str_field("reference"),
                &fields.str_field("finding"),
                fields.opt_str_field("note").as_deref(),
                role,
            )?;
            Ok(serde_json::to_string(&out)?)
        }
        "review_withdraw" => {
            let fields = ExtractFields::from_value(arguments, &["reference", "finding"]);
            let role_str = fields.opt_str_field("as");
            let role = review::parse_role(role_str.as_deref(), review::Role::Raiser)
                .context("invalid role")?;
            let out = review::run_withdraw(
                Some(root.to_path_buf()),
                &fields.str_field("reference"),
                &fields.str_field("finding"),
                role,
            )?;
            Ok(serde_json::to_string(&out)?)
        }
        "review_status" => {
            let fields = ExtractFields::from_value(arguments, &["reference"]);
            let out = review::run_status(Some(root.to_path_buf()), &fields.str_field("reference"))?;
            Ok(serde_json::to_string(&out)?)
        }
        "review_prime" => {
            let args: PrimeArgs = serde_json::from_value(arguments)
                .map_err(|e| anyhow::anyhow!("invalid arguments: {e:#}"))?;
            let out = review::run_prime(Some(root.to_path_buf()), &args)?;
            Ok(serde_json::to_string(&out)?)
        }
        "memory_find" => {
            let fields = ExtractFields::from_value(arguments, &[]);
            let limit = fields.opt_usize_field("limit");
            let has_selectors = fields.opt_str_field("query").is_some()
                || !fields.vec_str_field("path_scope").is_empty()
                || !fields.vec_str_field("glob").is_empty()
                || !fields.vec_str_field("command").is_empty()
                || !fields.vec_str_field("tag").is_empty()
                || fields.opt_str_field("type").is_some()
                || fields.opt_str_field("status").is_some()
                || fields.opt_str_field("lifespan").is_some();
            // No selectors + no explicit limit → default cap of 20 (design §3)
            let effective_limit = if !has_selectors && limit.is_none() {
                Some(20usize)
            } else {
                limit
            };
            let result = retrieve::find_for_mcp(
                Some(root.to_path_buf()),
                fields.vec_str_field("path_scope"),
                fields.vec_str_field("glob"),
                fields.vec_str_field("command"),
                fields.vec_str_field("tag"),
                parse_lifespan(fields.opt_str_field("lifespan"))?,
                fields.opt_str_field("query"),
                parse_memory_type(fields.opt_str_field("type"))?,
                parse_status(fields.opt_str_field("status"))?,
                fields.opt_bool_field("include_draft").unwrap_or(false),
                fields.opt_usize_field("offset").unwrap_or(0),
                effective_limit,
            )?;
            let offset = fields.opt_usize_field("offset").unwrap_or(0);
            let cap = effective_limit.unwrap_or(result.total);
            let next_offset = if offset + cap < result.total {
                Some(offset + cap)
            } else {
                None
            };
            Ok(serde_json::to_string_pretty(&json!({
                "kind": "memory_find",
                "rows": result.rows,
                "total": result.total,
                "offset": offset,
                "limit": cap,
                "next_offset": next_offset,
            }))?)
        }
        "memory_retrieve" => {
            let fields = ExtractFields::from_value(arguments, &[]);
            let reference = fields.opt_str_field("reference");
            let include_draft = fields.opt_bool_field("include_draft").unwrap_or(false);

            // Validate min_trust before use — parse_min_trust errors on bad input
            let min_trust_str = fields.opt_str_field("min_trust");
            let min_trust = min_trust_str
                .as_deref()
                .map(|s| {
                    retrieve::parse_min_trust(s)
                        .map_err(|e| anyhow::anyhow!("invalid arguments: {e}"))
                })
                .transpose()?;

            if let Some(ref_str) = reference {
                // Validate mutual exclusivity: reference alone, no probes
                let has_probes = fields.opt_str_field("query").is_some()
                    || !fields.vec_str_field("path_scope").is_empty()
                    || !fields.vec_str_field("glob").is_empty()
                    || !fields.vec_str_field("command").is_empty()
                    || !fields.vec_str_field("tag").is_empty()
                    || fields.opt_str_field("type").is_some()
                    || fields.opt_str_field("status").is_some()
                    || fields.opt_str_field("lifespan").is_some();
                if has_probes {
                    anyhow::bail!(
                        "invalid arguments: reference is mutually exclusive with query/path_scope/glob/command/tag/type/status/lifespan"
                    );
                }
                // Single-memory path: resolve → check_retrievable → staleness → render
                let mut buf = Vec::new();
                retrieve::retrieve_reference(
                    &mut buf,
                    root,
                    &ref_str,
                    include_draft,
                    min_trust.as_deref(),
                )?;
                Ok(String::from_utf8(buf)?)
            } else {
                // Scope-based path: search → rank → holdback → framed blocks
                let mut buf = Vec::new();
                retrieve::run_retrieve(
                    &mut buf,
                    Some(root.to_path_buf()),
                    fields.vec_str_field("path_scope"),
                    fields.vec_str_field("glob"),
                    fields.vec_str_field("command"),
                    fields.vec_str_field("tag"),
                    parse_lifespan(fields.opt_str_field("lifespan"))?,
                    fields.opt_str_field("query"),
                    parse_memory_type(fields.opt_str_field("type"))?,
                    parse_status(fields.opt_str_field("status"))?,
                    include_draft,
                    fields
                        .opt_usize_field("limit")
                        .unwrap_or(retrieve::RETRIEVE_LIMIT_DEFAULT),
                    min_trust.as_deref(),
                    fields.opt_usize_field("offset").unwrap_or(0),
                    crate::listing::Format::Table,
                    None, // expand (deferred per scope)
                )?;
                Ok(String::from_utf8(buf)?)
            }
        }
        "memory_show" => {
            let fields = ExtractFields::from_value(arguments, &["reference"]);
            let reference = fields.str_field("reference");
            if reference.is_empty() {
                anyhow::bail!("invalid arguments: reference is required");
            }
            let view = fields
                .opt_str_field("view")
                .unwrap_or_else(|| "summary".to_owned());
            let include_body = fields.opt_bool_field("include_body").unwrap_or(true);
            let backlinks_limit = fields.opt_usize_field("backlinks_limit");

            // Get base show JSON via run_show
            let mut buf = Vec::new();
            memory::run_show(
                &mut buf,
                Some(root.to_path_buf()),
                &reference,
                crate::listing::Format::Json,
            )?;
            let json_str = String::from_utf8(buf)?;
            let mut value: serde_json::Value = serde_json::from_str(&json_str)?;

            // Extract uid from the run_show JSON output
            let uid = value
                .get("memory")
                .and_then(|m| m.get("uid"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("invalid memory show response: missing uid"))?
                .to_owned();

            // One collect_all + freeze for both check_retrievable and backlinks (design §4)
            let all = memory::collect_all(root)?;
            let mref = memory::MemoryRef::parse(&uid)?;
            let memory = memory::resolve_memory_from_all(&all, &mref)
                .map_err(|e| anyhow::anyhow!("memory not found: {reference}: {e}"))?;
            let snap = retrieve::freeze(root);

            // check_retrievable → consumable + held_back_on_retrieve + notes
            let (consumable, notes) =
                retrieve::check_retrievable(memory, &snap.part, false, None, &snap.today);
            let held_back_on_retrieve =
                !consumable || retrieve::held_back(memory, retrieve::holdback_floor(None));

            // Backlinks enrichment (design §4)
            let backlinks = memory::backlink_rows_for(root, &all, &uid);
            let backlinks_total = backlinks.len();
            let backlinks_clipped: Vec<serde_json::Value> = backlinks
                .iter()
                .take(backlinks_limit.unwrap_or(20))
                .map(|b| {
                    json!({
                        "uid": b.uid,
                        "title": b.title,
                        "type": b.memory_type,
                        "method": b.method,
                    })
                })
                .collect();

            // Inject enriched fields into the memory object
            if let Some(obj) = value.get_mut("memory").and_then(|v| v.as_object_mut()) {
                obj.insert("consumable".to_owned(), json!(consumable));
                obj.insert(
                    "held_back_on_retrieve".to_owned(),
                    json!(held_back_on_retrieve),
                );
                obj.insert("backlinks".to_owned(), json!(backlinks_clipped));
                obj.insert("backlinks_total".to_owned(), json!(backlinks_total));
            }

            // When not consumable, surface the reason as notes
            if let Some(notes_text) = notes.filter(|_| !consumable)
                && let Some(obj) = value.as_object_mut()
            {
                obj.insert("notes".to_owned(), json!(notes_text));
            }

            // Handle view / include_body
            let view_full = view == "full";
            if !(view_full && include_body)
                && let Some(obj) = value.as_object_mut()
            {
                obj.remove("body");
            }

            Ok(serde_json::to_string_pretty(&value)?)
        }
        "memory_list" => {
            let fields = ExtractFields::from_value(arguments, &[]);
            // Resolve limit before passing: default 50, 0 = all (unbounded)
            let limit_raw = fields.opt_usize_field("limit");
            let limit = match limit_raw {
                Some(0) => usize::MAX,
                None => 50,
                Some(n) => n,
            };
            let result = memory::list_for_mcp(
                root,
                parse_memory_type(fields.opt_str_field("type"))?,
                fields.opt_str_field("substr").as_deref(),
                &fields.vec_str_field("status"),
                &fields.vec_str_field("tag"),
                fields.opt_usize_field("offset").unwrap_or(0),
                limit,
            )?;
            let offset = fields.opt_usize_field("offset").unwrap_or(0);
            let next_offset = if offset + limit < result.total {
                Some(offset + limit)
            } else {
                None
            };
            Ok(serde_json::to_string_pretty(&json!({
                "kind": "memory",
                "rows": result.rows,
                "total": result.total,
                "offset": offset,
                "limit": if limit == usize::MAX { result.total } else { limit },
                "next_offset": next_offset,
            }))?)
        }
        "memory_validate" => {
            let fields = ExtractFields::from_value(arguments, &[]);
            let reference = fields.opt_str_field("reference");
            let path = fields.opt_str_field("path");
            let path_buf = path.map(std::path::PathBuf::from);
            let mut buf = Vec::new();
            let result = memory::run_validate(path_buf, reference.as_deref(), &mut buf);
            let output = String::from_utf8(buf)?;
            match result {
                Ok(()) => Ok(serde_json::to_string_pretty(&json!({
                    "warnings": 0,
                    "output": output
                }))?),
                Err(e) if e.to_string().contains("validation warnings found") => {
                    Ok(serde_json::to_string_pretty(&json!({
                        "warnings": output.lines().count(),
                        "output": output
                    }))?)
                }
                Err(e) => Err(e),
            }
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

    /// Extract an optional boolean (missing or non-boolean ⇒ `None`).
    /// Used for the `include_draft` flag.
    fn opt_bool_field(&self, name: &str) -> Option<bool> {
        self.inner.get(name).and_then(serde_json::Value::as_bool)
    }
}

// ── Argument parse helpers for memory tools ─────────────────────────────

/// Parse a `MemoryType` from an optional string, wrapping errors with the
/// load-bearing "invalid arguments: " prefix so the MCP error mapper (§2,
/// branch 2) routes them to `-32602` (Invalid params) rather than `-32603`.
fn parse_memory_type(s: Option<String>) -> anyhow::Result<Option<crate::memory::MemoryType>> {
    s.map(|v| {
        crate::memory::MemoryType::parse(&v).map_err(|e| anyhow::anyhow!("invalid arguments: {e}"))
    })
    .transpose()
}

/// Parse a memory `Status` from an optional string, wrapping errors with the
/// load-bearing "invalid arguments: " prefix.
fn parse_status(s: Option<String>) -> anyhow::Result<Option<crate::memory::Status>> {
    s.map(|v| {
        crate::memory::Status::parse(&v).map_err(|e| anyhow::anyhow!("invalid arguments: {e}"))
    })
    .transpose()
}

/// Parse a `Lifespan` from an optional string via `FromStr`, wrapping errors
/// with the load-bearing "invalid arguments: " prefix.
fn parse_lifespan(s: Option<String>) -> anyhow::Result<Option<crate::memory::Lifespan>> {
    s.map(|v| {
        crate::memory::Lifespan::from_str(&v).map_err(|e| anyhow::anyhow!("invalid arguments: {e}"))
    })
    .transpose()
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

/// The lean default row cap for `review_list` when the caller names none (IMP-114).
const DEFAULT_REVIEW_LIST_LIMIT: usize = 50;

/// Resolve the effective row cap from the `limit` argument: absent ⇒ the lean
/// default; explicit `0` ⇒ unbounded (the "all" escape hatch — zero rows is never
/// a useful request, so the sentinel is free); explicit `n` ⇒ `n` (IMP-114).
fn effective_cap(limit: Option<usize>) -> Option<usize> {
    match limit {
        None => Some(DEFAULT_REVIEW_LIST_LIMIT),
        Some(0) => None,
        Some(n) => Some(n),
    }
}

/// Cap a `Listed` output to the most recent `cap` rows (the tail — highest ids),
/// stamping `total` with the pre-truncation count so the omission is never silent
/// (IMP-114). A `None` cap, or a list already within the cap, passes through with
/// `total` left `None`. Non-`Listed` outputs pass through.
fn project_list_cap(out: ReviewOutput, cap: Option<usize>) -> ReviewOutput {
    match (out, cap) {
        (
            ReviewOutput::Listed {
                mut rows,
                formatted,
                ..
            },
            Some(n),
        ) if rows.len() > n => {
            let total = rows.len();
            rows = rows.split_off(total - n);
            ReviewOutput::Listed {
                rows,
                total: Some(total),
                formatted,
            }
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
    fn tool_list_has_14_tools() {
        let list = tool_list();
        assert_eq!(list.tools.len(), 15);
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
        assert!(names.contains(&"memory_find"));
        assert!(names.contains(&"memory_retrieve"));
        assert!(names.contains(&"memory_show"));
        assert!(names.contains(&"memory_list"));
        assert!(names.contains(&"memory_validate"));
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
            total: None,
            formatted: "RENDERED TABLE".to_owned(),
        };
        let v = serde_json::to_value(&listed).unwrap();
        assert!(v["Listed"].get("rows").is_some());
        assert!(
            v["Listed"].get("formatted").is_none(),
            "Listed leaked formatted: {v}"
        );
        // total is absent when the list is complete (IMP-114).
        assert!(
            v["Listed"].get("total").is_none(),
            "uncapped total leaked: {v}"
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

    // IMP-114: effective_cap resolves the lean default / explicit / 0=all escape.

    #[test]
    fn effective_cap_resolves_default_explicit_and_all() {
        assert_eq!(effective_cap(None), Some(DEFAULT_REVIEW_LIST_LIMIT));
        assert_eq!(effective_cap(Some(3)), Some(3));
        assert_eq!(effective_cap(Some(0)), None, "0 ⇒ unbounded escape hatch");
    }

    // IMP-114: a cap keeps the most recent N (tail) and stamps total; an
    // uncapped or within-cap list passes through with total absent.

    #[test]
    fn project_list_cap_keeps_tail_and_stamps_total() {
        let make = || ReviewOutput::Listed {
            rows: vec![row("RV-1"), row("RV-2"), row("RV-3")],
            total: None,
            formatted: String::new(),
        };

        // Capped below len: keep the newest 2 (tail), total = 3.
        let ReviewOutput::Listed { rows, total, .. } = project_list_cap(make(), Some(2)) else {
            panic!("expected Listed");
        };
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id, "RV-2", "keeps the tail (most recent)");
        assert_eq!(rows[1].id, "RV-3");
        assert_eq!(total, Some(3), "pre-truncation count surfaced");

        // Cap at or above len: not truncated, total stays None.
        let ReviewOutput::Listed { rows, total, .. } = project_list_cap(make(), Some(5)) else {
            panic!("expected Listed");
        };
        assert_eq!(rows.len(), 3);
        assert_eq!(total, None, "within-cap ⇒ no total");

        // Unbounded (None): everything, total None.
        let ReviewOutput::Listed { rows, total, .. } = project_list_cap(make(), None) else {
            panic!("expected Listed");
        };
        assert_eq!(rows.len(), 3);
        assert_eq!(total, None);
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
        assert_eq!(tools.len(), 14);
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

    // ── Memory MCP handler tests (PHASE-04) ──────────────────────────────

    const MEM_A: &str = "mem_0000000000000000000000000000000a";
    const MEM_B: &str = "mem_0000000000000000000000000000000b";

    /// Seed a single memory record into the temp root.
    fn seed_memory(
        root: &Path,
        uid: &str,
        key: Option<&str>,
        kind: &str,
        status: &str,
        trust: &str,
        title: &str,
        body: &str,
    ) {
        let dir = root.join(format!(".doctrine/memory/items/{uid}"));
        std::fs::create_dir_all(&dir).unwrap();
        let key_line = key.map_or(String::new(), |k| format!("memory_key = \"{k}\"\n"));
        std::fs::write(
            dir.join("memory.toml"),
            format!(
                "memory_uid = \"{uid}\"\n\
                 {key_line}\
                 schema_version = 1\n\
                 memory_type = \"{kind}\"\n\
                 status = \"{status}\"\n\
                 title = \"{title}\"\n\
                 summary = \"\"\n\
                 created = \"2026-01-02\"\n\
                 updated = \"2026-01-02\"\n\
                 \n\
                 [scope]\n\
                 workspace = \"default\"\n\
                 \n\
                 [git]\n\
                 anchor_kind = \"none\"\n\
                 \n\
                 [trust]\n\
                 trust_level = \"{trust}\"\n",
            ),
        )
        .unwrap();
        std::fs::write(dir.join("memory.md"), body).unwrap();
        if let Some(k) = key {
            std::os::unix::fs::symlink(uid, root.join(format!(".doctrine/memory/items/{k}"))).ok();
        }
    }

    /// Seed a minimal memory corpus: two active memories.
    fn seed_memory_corpus(root: &Path) {
        seed_memory(
            root,
            MEM_A,
            Some("mem.pattern.cli.skinny"),
            "pattern",
            "active",
            "high",
            "Skinny CLI",
            "# Skinny CLI\n\nBody A content.",
        );
        seed_memory(
            root,
            MEM_B,
            None,
            "fact",
            "active",
            "medium",
            "A bare fact",
            "# A bare fact\n\nBody B content with [[mem.pattern.cli.skinny]] link.",
        );
        // Add a shipped dir so root::find finds the repo root
        let shipped = root.join(".doctrine/memory/shipped");
        std::fs::create_dir_all(&shipped).unwrap();
    }

    /// Helper: dispatch a memory tool call and return the result JSON.
    fn memory_dispatch(root: &Path, name: &str, args: Value) -> Value {
        let req = tools_call_req(name, args);
        let resp = dispatch(&req, root);
        resp.result.expect("expected success")
    }

    // VT-3: memory_retrieve with min_trust: "banana" returns -32602

    #[test]
    fn memory_retrieve_bad_min_trust_returns_32602() {
        let (_dir, root) = temp_root();
        let req = tools_call_req(
            "memory_retrieve",
            json!({
                "min_trust": "banana"
            }),
        );
        let resp = dispatch(&req, &root);
        let err = resp.error.unwrap();
        assert_eq!(err.code, -32602);
        assert!(err.message.contains("Invalid params"));
    }

    // VT-4: memory_retrieve with reference + query probe returns -32602

    #[test]
    fn memory_retrieve_reference_with_probe_mutual_exclusivity() {
        let (_dir, root) = temp_root();
        let req = tools_call_req(
            "memory_retrieve",
            json!({
                "reference": "mem_xxx",
                "query": "test"
            }),
        );
        let resp = dispatch(&req, &root);
        let err = resp.error.unwrap();
        assert_eq!(err.code, -32602);
        let data = err.data.unwrap();
        assert!(
            data["parse_error"]
                .as_str()
                .unwrap_or("")
                .contains("mutually exclusive")
        );
    }

    // VT-5: memory_show with invalid uid returns error

    #[test]
    fn memory_show_invalid_uid_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        // Must have .git for root::find
        std::fs::create_dir_all(root.join(".git")).unwrap();
        let req = tools_call_req(
            "memory_show",
            json!({
                "reference": "nonexistent"
            }),
        );
        let resp = dispatch(&req, &root);
        assert!(resp.error.is_some(), "expected error for invalid uid");
    }

    // VT-5: memory_show with view: summary excludes body
    // VT-6: memory_show with backlinks_limit: 5 returns ≤5 backlinks
    // VT-7: memory_show with include_body: false excludes body

    #[test]
    fn memory_show_view_summary_excludes_body() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        seed_memory_corpus(root);
        let result = memory_dispatch(
            root,
            "memory_show",
            json!({
                "reference": MEM_A,
                "view": "summary"
            }),
        );
        // Parse the text content
        let text = result["content"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        // body should be absent (summary view)
        assert!(
            parsed.get("body").is_none(),
            "summary view should exclude body"
        );
        // memory metadata should be present
        assert_eq!(parsed["memory"]["uid"], MEM_A);
        assert_eq!(parsed["memory"]["consumable"], true);
    }

    #[test]
    fn memory_show_include_body_false_excludes_body() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        seed_memory_corpus(root);
        let result = memory_dispatch(
            root,
            "memory_show",
            json!({
                "reference": MEM_A,
                "view": "full",
                "include_body": false
            }),
        );
        let text = result["content"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert!(
            parsed.get("body").is_none(),
            "include_body: false should exclude body"
        );
    }

    #[test]
    fn memory_show_view_full_includes_body() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        seed_memory_corpus(root);
        let result = memory_dispatch(
            root,
            "memory_show",
            json!({
                "reference": MEM_A,
                "view": "full"
            }),
        );
        let text = result["content"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert!(
            parsed.get("body").is_some(),
            "full view should include body"
        );
    }

    #[test]
    fn memory_show_includes_backlinks() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        seed_memory_corpus(root);
        // MEM_B has [[mem.pattern.cli.skinny]] wiki link to MEM_A's key
        let result = memory_dispatch(
            root,
            "memory_show",
            json!({
                "reference": MEM_A
            }),
        );
        let text = result["content"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert!(
            parsed["memory"]["backlinks_total"].as_u64().unwrap_or(0) > 0,
            "MEM_A should have backlinks from MEM_B"
        );
        let backlinks = parsed["memory"]["backlinks"].as_array().unwrap();
        assert!(!backlinks.is_empty(), "backlinks array should not be empty");
    }

    #[test]
    fn memory_show_backlinks_limit_caps() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        seed_memory_corpus(root);
        let result = memory_dispatch(
            root,
            "memory_show",
            json!({
                "reference": MEM_A,
                "backlinks_limit": 1
            }),
        );
        let text = result["content"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        let backlinks = parsed["memory"]["backlinks"].as_array().unwrap();
        assert!(backlinks.len() <= 1, "backlinks should be capped at 1");
    }

    // VT-1: memory_find with no args returns capped 20 rows with pagination metadata
    // VT-2: memory_find rows include key and held_back_on_retrieve fields

    #[test]
    fn memory_find_no_args_returns_paginated_results() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        seed_memory_corpus(root);
        let result = memory_dispatch(root, "memory_find", json!({}));
        let text = result["content"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["kind"], "memory_find");
        // With 2 seeds and no selectors → capped at 20
        let rows = parsed["rows"].as_array().unwrap();
        assert!(!rows.is_empty(), "should return rows");
        assert!(rows.len() <= 20, "no-selector default cap should be 20");
        // Pagination metadata
        assert!(parsed["total"].as_u64().is_some());
        assert!(parsed["offset"].as_u64().is_some());
        assert!(parsed["limit"].as_u64().is_some());
        // Each row has key and held_back_on_retrieve fields
        for row in rows {
            assert!(row.get("key").is_some(), "row missing key field");
            assert!(
                row.get("held_back_on_retrieve").is_some(),
                "row missing held_back_on_retrieve"
            );
        }
    }

    #[test]
    fn memory_find_with_selectors_returns_scoped_results() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        seed_memory_corpus(root);
        let result = memory_dispatch(
            root,
            "memory_find",
            json!({
                "query": "Skinny"
            }),
        );
        let text = result["content"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["kind"], "memory_find");
        let rows = parsed["rows"].as_array().unwrap();
        assert!(rows.len() >= 1, "should find at least 1 memory");
        // The Skinny CLI memory should be in results
        let has_skinny = rows.iter().any(|r| r["uid"] == MEM_A);
        assert!(has_skinny, "should include Skinny CLI memory");
    }

    // VT-8: memory_list defaults to 50 rows; limit: 0 returns all

    #[test]
    fn memory_list_default_limit_50() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        seed_memory_corpus(root);
        let result = memory_dispatch(root, "memory_list", json!({}));
        let text = result["content"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["kind"], "memory");
        assert_eq!(parsed["limit"], 50, "default limit should be 50");
        let rows = parsed["rows"].as_array().unwrap();
        assert_eq!(parsed["total"], 2, "should have 2 total memories");
        assert_eq!(rows.len(), 2, "should show all 2 (under 50 cap)");
    }

    #[test]
    fn memory_list_limit_zero_returns_all() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        seed_memory_corpus(root);
        let result = memory_dispatch(
            root,
            "memory_list",
            json!({
                "limit": 0
            }),
        );
        let text = result["content"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["kind"], "memory");
        // limit in response should equal total when limit=0 was requested
        assert_eq!(parsed["limit"], parsed["total"]);
        let rows = parsed["rows"].as_array().unwrap();
        assert_eq!(rows.len() as u64, parsed["total"].as_u64().unwrap());
    }

    // Confirm the MCP response text parses as JSON object (not quoted string)
    // — the double-encoding guard.

    #[test]
    fn memory_find_text_parses_as_json_object() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        seed_memory_corpus(root);
        let result = memory_dispatch(root, "memory_find", json!({}));
        let text = result["content"][0]["text"].as_str().unwrap();
        // Should parse as a JSON object, not a quoted string
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert!(
            parsed.is_object(),
            "memory_find result must be a JSON object"
        );
    }
}
