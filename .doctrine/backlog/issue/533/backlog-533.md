# ISS-533: MCP server replies to JSON-RPC notifications

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`mcp_server::serve`'s loop (`src/mcp_server/mod.rs`) writes a response for every
message `tools::dispatch` returns, including notifications (no `id`, e.g.
`notifications/initialized`), which get `{"id": null, "result": {}}`. JSON-RPC
2.0 says a server must not reply to a notification. Strict clients may log or
reject the stray reply.

Fix sketch: skip the write when the request has no `id`, or make `dispatch`
return `Option<JsonRpcResponse>`.

Found by the pi review of SL-286 PHASE-01 (pre-existing; not introduced there).
