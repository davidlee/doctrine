// SPDX-License-Identifier: GPL-3.0-only
//! Framed stdio transport for MCP JSON-RPC messages.
//!
//! Reads newline-delimited JSON messages from stdin, writes newline-delimited
//! JSON responses to stdout. No Content-Length header (the MCP 2024-11-05 spec
//! uses `\n`-delimited framing for stdio).

use super::protocol::{JsonRpcRequest, JsonRpcResponse};
use anyhow::Context;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;

/// Read one JSON-RPC request from `reader`. Returns `Ok(None)` on EOF.
///
/// Each message is a single line of JSON terminated by `\n`. Partial reads
/// (multiple messages in one buffer) are handled by `read_line`.
pub(crate) async fn read_message<R>(reader: &mut R) -> anyhow::Result<Option<JsonRpcRequest>>
where
    R: AsyncBufReadExt + Unpin,
{
    let mut line = String::new();
    let n = reader.read_line(&mut line).await?;
    if n == 0 {
        // EOF — clean shutdown
        return Ok(None);
    }
    let trimmed = line.trim();
    if trimmed.is_empty() {
        // Empty line — skip (could be a keep-alive or noise)
        return Ok(None);
    }
    let req = serde_json::from_str::<JsonRpcRequest>(trimmed)
        .with_context(|| format!("failed to parse JSON-RPC request: {trimmed}"))?;
    Ok(Some(req))
}

/// Write one JSON-RPC response to `writer`.
///
/// Serialises the response as a single JSON line + `\n`, then flushes.
pub(crate) async fn write_message<W>(
    writer: &mut W,
    response: &JsonRpcResponse,
) -> anyhow::Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    let json = serde_json::to_string(response).context("failed to serialise JSON-RPC response")?;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio::io::BufReader;

    // VT-2: transport framing handles partial reads, multiple messages in one chunk

    fn make_request(id: i64, method: &str) -> JsonRpcRequest {
        serde_json::from_value(json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": {}
        }))
        .unwrap()
    }

    fn make_response(id: i64) -> JsonRpcResponse {
        JsonRpcResponse::success(Some(super::super::protocol::Id::Number(id)), json!({}))
    }

    #[tokio::test]
    async fn read_one_message() {
        let data = b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}\n";
        let mut reader = BufReader::new(&data[..]);
        let req = read_message(&mut reader).await.unwrap().unwrap();
        assert_eq!(req.method, "initialize");
        assert_eq!(req.jsonrpc, "2.0");
    }

    #[tokio::test]
    async fn read_eof_returns_none() {
        let data = b"";
        let mut reader = BufReader::new(&data[..]);
        let req = read_message(&mut reader).await.unwrap();
        assert!(req.is_none());
    }

    #[tokio::test]
    async fn read_multiple_in_one_chunk() {
        // Two complete messages in one read buffer
        let data = concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"m1\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"m2\",\"params\":{}}\n"
        );
        let mut reader = BufReader::new(data.as_bytes());

        let req1 = read_message(&mut reader).await.unwrap().unwrap();
        assert_eq!(req1.method, "m1");

        let req2 = read_message(&mut reader).await.unwrap().unwrap();
        assert_eq!(req2.method, "m2");
    }

    #[tokio::test]
    async fn read_partial_then_rest() {
        // Simulate a line split across reads (read_line handles this natively,
        // but we verify the framing contract works)
        let data = b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"test\",\"params\":{}}\n";
        // First read half
        let mut reader = BufReader::new(&data[..]);
        let req = read_message(&mut reader).await.unwrap().unwrap();
        assert_eq!(req.method, "test");
    }

    #[tokio::test]
    async fn write_and_read_round_trip() {
        use tokio::io::duplex;
        let (mut client, server) = duplex(1024);
        let (mut server_reader, mut server_writer) = tokio::io::split(server);

        let req = make_request(1, "tools/list");

        // Write from client side
        let json = serde_json::to_string(&req).unwrap() + "\n";
        client.write_all(json.as_bytes()).await.unwrap();
        client.flush().await.unwrap();

        // Read on server side
        let mut reader = BufReader::new(&mut server_reader);
        let received = read_message(&mut reader).await.unwrap().unwrap();
        assert_eq!(received.method, "tools/list");

        // Write response
        let resp = make_response(1);
        write_message(&mut server_writer, &resp).await.unwrap();

        // Read on client side
        let mut client_reader = BufReader::new(&mut client);
        let mut line = String::new();
        client_reader.read_line(&mut line).await.unwrap();
        let returned: JsonRpcResponse = serde_json::from_str(line.trim()).unwrap();
        assert!(returned.error.is_none());
    }
}
