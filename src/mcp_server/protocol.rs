// SPDX-License-Identifier: GPL-3.0-only
//! JSON-RPC 2.0 types for the MCP stdio server.
//!
//! Hand-rolled per design D4 — zero new crate dependencies.
//! The MCP spec (2024-11-05) uses `tools/call` (not `tools/execute`).
//! Response shape: `{ result: { content: [{ type: "text", text: "..." }] } }`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── JSON-RPC 2.0 core types ──────────────────────────────────────────────

/// A JSON-RPC request (or notification when `id` is `None`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JsonRpcRequest {
    pub(crate) jsonrpc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<Id>,
    pub(crate) method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) params: Option<Value>,
}

/// A JSON-RPC response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JsonRpcResponse {
    pub(crate) jsonrpc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<Id>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<JsonRpcError>,
}

/// A JSON-RPC error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JsonRpcError {
    pub(crate) code: i32,
    pub(crate) message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) data: Option<Value>,
}

/// JSON-RPC id: number or string. JSON `null` deserialises as `None`
/// via the `Option<Id>` wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum Id {
    Number(i64),
    String(String),
}

// ── MCP protocol types ───────────────────────────────────────────────────

/// The content of a successful `tools/call` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct McpToolResult {
    pub(crate) content: Vec<McpContent>,
}

/// A single content item within a tool result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct McpContent {
    #[serde(rename = "type")]
    pub(crate) content_type: String,
    pub(crate) text: String,
}

/// The `initialize` response result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct InitializeResult {
    pub(crate) capabilities: Capabilities,
    #[serde(rename = "protocolVersion")]
    pub(crate) protocol_version: String,
    #[serde(rename = "serverInfo")]
    pub(crate) server_info: ServerInfo,
}

/// Server capabilities — tools only (v1 scope, design §5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Capabilities {
    pub(crate) tools: ToolsCap,
}

/// The tools capability — empty struct serialises to `{}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ToolsCap {}

/// Server identity for the `initialize` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ServerInfo {
    pub(crate) name: String,
    pub(crate) version: String,
}

/// A single tool definition returned by `tools/list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct McpTool {
    pub(crate) name: String,
    pub(crate) description: String,
    #[serde(rename = "inputSchema")]
    pub(crate) input_schema: Value,
}

/// The `tools/list` response result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ToolsListResult {
    pub(crate) tools: Vec<McpTool>,
}

// ── Constructors (used by tools.rs) ──────────────────────────────────────

impl JsonRpcResponse {
    /// Build a successful response.
    pub(crate) fn success(id: Option<Id>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_owned(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Build an error response.
    pub(crate) fn error(id: Option<Id>, code: i32, message: String, data: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_owned(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data,
            }),
        }
    }
}

impl McpToolResult {
    /// Wrap a JSON string into the MCP content envelope.
    pub(crate) fn text(text: String) -> Self {
        Self {
            content: vec![McpContent {
                content_type: "text".to_owned(),
                text,
            }],
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // VT-1: Request/Response serialise/deserialise round-trip

    #[test]
    fn request_round_trip() {
        let json = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "review_status", "arguments": { "reference": "1" } }
        });
        let req: JsonRpcRequest = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(req.jsonrpc, "2.0");
        assert!(matches!(req.id, Some(Id::Number(1))));
        assert_eq!(req.method, "tools/call");
        assert!(req.params.is_some());

        let back = serde_json::to_value(&req).unwrap();
        assert_eq!(back["jsonrpc"], "2.0");
        assert_eq!(back["id"], 1);
        assert_eq!(back["method"], "tools/call");
    }

    #[test]
    fn request_string_id() {
        let json = json!({
            "jsonrpc": "2.0",
            "id": "req-1",
            "method": "initialize",
            "params": { "protocolVersion": "2024-11-05" }
        });
        let req: JsonRpcRequest = serde_json::from_value(json).unwrap();
        assert!(matches!(req.id, Some(Id::String(ref s)) if s == "req-1"));
    }

    #[test]
    fn notification_no_id() {
        let json = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        let req: JsonRpcRequest = serde_json::from_value(json).unwrap();
        assert!(req.id.is_none());
        assert_eq!(req.method, "notifications/initialized");
    }

    #[test]
    fn response_success_round_trip() {
        let resp = JsonRpcResponse::success(
            Some(Id::Number(1)),
            json!({ "content": [{ "type": "text", "text": "ok" }] }),
        );
        let val = serde_json::to_value(&resp).unwrap();
        assert_eq!(val["jsonrpc"], "2.0");
        assert_eq!(val["id"], 1);
        assert_eq!(val["result"]["content"][0]["type"], "text");
        assert!(val.get("error").is_none());
    }

    #[test]
    fn response_error_round_trip() {
        let resp = JsonRpcResponse::error(
            Some(Id::Number(1)),
            -32601,
            "Method not found".to_owned(),
            Some(json!({ "method": "bad" })),
        );
        let val = serde_json::to_value(&resp).unwrap();
        assert_eq!(val["error"]["code"], -32601);
        assert_eq!(val["error"]["message"], "Method not found");
        assert_eq!(val["error"]["data"]["method"], "bad");
    }

    #[test]
    fn initialize_result_round_trip() {
        let init = InitializeResult {
            capabilities: Capabilities { tools: ToolsCap {} },
            protocol_version: "2024-11-05".to_owned(),
            server_info: ServerInfo {
                name: "doctrine-mcp".to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
            },
        };
        let val = serde_json::to_value(&init).unwrap();
        assert_eq!(val["capabilities"]["tools"], json!({}));
        assert_eq!(val["protocolVersion"], "2024-11-05");

        let back: InitializeResult = serde_json::from_value(val).unwrap();
        assert_eq!(back.server_info.name, "doctrine-mcp");
    }

    #[test]
    fn tools_list_result_round_trip() {
        let tools = ToolsListResult {
            tools: vec![McpTool {
                name: "review_status".to_owned(),
                description: "Get review status".to_owned(),
                input_schema: json!({
                    "type": "object",
                    "properties": { "reference": { "type": "string" } },
                    "required": ["reference"]
                }),
            }],
        };
        let val = serde_json::to_value(&tools).unwrap();
        assert_eq!(val["tools"][0]["name"], "review_status");
    }

    #[test]
    fn mcp_tool_result_envelope() {
        let result = McpToolResult::text("{\"ok\":true}".to_owned());
        let val = serde_json::to_value(&result).unwrap();
        assert_eq!(val["content"][0]["type"], "text");
        assert_eq!(val["content"][0]["text"], "{\"ok\":true}");
    }

    #[test]
    fn id_null_is_none() {
        let json = serde_json::json!({
            "jsonrpc": "2.0",
            "id": null,
            "method": "test"
        });
        let req: JsonRpcRequest = serde_json::from_value(json).unwrap();
        assert!(req.id.is_none(), "null id should be None");
    }
}
