// SPDX-License-Identifier: GPL-3.0-only
//! SL-109 PHASE-04 — integration tests for the MCP stdio server.
//!
//! Spawns `doctrine serve --mcp -p <root>` as a subprocess, drives the MCP
//! protocol handshake and tool round-trips over stdin/stdout JSON-RPC 2.0,
//! and verifies authored state on disk.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");

// ── Helpers ──────────────────────────────────────────────────────────────

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

/// Seed a slice into the temp root (needed as a review target).
fn seed_slice(root: &Path, id: u32, title: &str, slug: &str) {
    let name = format!("{id:03}");
    let dir = root.join(format!(".doctrine/slice/{name}"));
    fs::create_dir_all(&dir).unwrap();
    let toml = format!(
        "id = {id}\n\
         slug = \"{slug}\"\n\
         title = \"{title}\"\n\
         status = \"proposed\"\n\
         created = \"2026-06-14\"\n\
         updated = \"2026-06-14\"\n\
         \n\
         [relationships]\n\
         needs = []\n\
         after = []\n"
    );
    fs::write(dir.join(format!("slice-{name}.toml")), &toml).unwrap();
    fs::write(
        dir.join(format!("slice-{name}.md")),
        format!("# {title}\n\n## Context\n\n## Scope & Objectives\n\n## Non-Goals\n\n## Summary\n\n## Follow-Ups\n"),
    )
    .unwrap();
}

/// Spawn the MCP server subprocess with piped stdin/stdout.
fn spawn_server(root: &Path) -> Child {
    Command::new(BIN)
        .arg("serve")
        .arg("--mcp")
        .arg("--path")
        .arg(root)
        .env_remove("DOCTRINE_WORKER")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn doctrine serve --mcp")
}

/// Write a JSON-RPC request line to the server's stdin.
fn send_request(stdin: &mut impl Write, req: &Value) {
    let line = serde_json::to_string(req).expect("serialise request");
    writeln!(stdin, "{line}").expect("write request");
    stdin.flush().expect("flush stdin");
}

/// Read one JSON-RPC response line from the server's stdout.
fn read_response(stdout: &mut BufReader<impl std::io::Read>) -> Value {
    let mut line = String::new();
    stdout.read_line(&mut line).expect("read response line");
    let trimmed = line.trim();
    assert!(!trimmed.is_empty(), "empty response line");
    serde_json::from_str(trimmed).expect("parse JSON-RPC response")
}

/// Send a request and read its response (convenience wrapper).
fn call(
    stdin: &mut impl Write,
    stdout: &mut BufReader<impl std::io::Read>,
    method: &str,
    params: Option<&Value>,
) -> Value {
    let req = make_request(1, method, params);
    send_request(stdin, &req);
    read_response(stdout)
}

/// Build a JSON-RPC 2.0 request.
fn make_request(id: i64, method: &str, params: Option<&Value>) -> Value {
    let mut req = serde_json::Map::new();
    req.insert("jsonrpc".to_owned(), "2.0".into());
    req.insert("id".to_owned(), id.into());
    req.insert("method".to_owned(), method.into());
    if let Some(p) = params {
        req.insert("params".to_owned(), p.clone());
    }
    Value::Object(req)
}

/// Make a `tools/call` params object.
fn tools_call_params(name: &str, arguments: Value) -> Value {
    let mut params = serde_json::Map::new();
    params.insert("name".to_owned(), name.into());
    params.insert("arguments".to_owned(), arguments);
    Value::Object(params)
}

/// Extract the JSON text content from a `tools/call` MCP result envelope.
fn tool_result_text(resp: &Value) -> &str {
    resp["result"]["content"][0]["text"]
        .as_str()
        .expect("text content")
}

/// Kill the server and drain stderr (ignoring output).
fn kill(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}

// ── VT-1: MCP handshake (initialize) ─────────────────────────────────────

#[test]
fn vt1_initialize_handshake() {
    let dir = tmp();
    let root = dir.path();

    // Create markers so root::find succeeds
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    // Send initialize
    let params = serde_json::json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": { "name": "test", "version": "1.0" }
    });
    let resp = call(&mut stdin, &mut reader, "initialize", Some(&params));

    assert!(
        resp.get("error").is_none(),
        "initialize should not error: {resp:?}"
    );
    let result = resp.get("result").expect("result present");
    assert_eq!(result["capabilities"]["tools"], serde_json::json!({}));
    assert_eq!(result["protocolVersion"], "2024-11-05");
    assert_eq!(result["serverInfo"]["name"], "doctrine-mcp");

    kill(child);
}

// ── VT-2: tools/list returns 14 tools ────────────────────────────────────

#[test]
fn vt2_tools_list() {
    let dir = tmp();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    let resp = call(&mut stdin, &mut reader, "tools/list", None);

    assert!(
        resp.get("error").is_none(),
        "tools/list should not error: {resp:?}"
    );
    let tools = resp["result"]["tools"].as_array().expect("tools array");
    assert_eq!(tools.len(), 14, "expected 14 tools, got {tools:?}");

    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    for expected in &[
        "review_new",
        "review_list",
        "review_show",
        "review_raise",
        "review_dispose",
        "review_verify",
        "review_contest",
        "review_withdraw",
        "review_status",
        "review_prime",
        "memory_find",
        "memory_retrieve",
        "memory_show",
        "memory_list",
    ] {
        assert!(
            names.contains(expected),
            "missing tool: {expected}\ngot: {names:?}"
        );
    }

    kill(child);
}

// ── VT-3: review_new creates review dir ──────────────────────────────────

#[test]
fn vt3_review_new() {
    let dir = tmp();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();
    seed_slice(root, 1, "Test Slice", "test-slice");

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    // Handshake
    let _ = call(
        &mut stdin,
        &mut reader,
        "initialize",
        Some(&serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0" }
        })),
    );

    // Create review
    let params = tools_call_params(
        "review_new",
        serde_json::json!({ "facet": "design", "target": "SL-001" }),
    );
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));

    assert!(
        resp.get("error").is_none(),
        "review_new should not error: {resp:?}"
    );
    let text = tool_result_text(&resp);
    let out: Value = serde_json::from_str(text).expect("parse ReviewOutput JSON");

    // Check Created variant (externally-tagged enum → {"Created": {...}})
    let created = &out["Created"];
    assert_eq!(created["id"], 1, "first review id should be 1");
    let canonical = created["canonical"].as_str().expect("canonical");
    assert!(
        canonical.starts_with("RV-"),
        "expected RV-NNN, got {canonical}"
    );

    // Verify on-disk state
    let review_dir = root.join(format!(".doctrine/review/001"));
    assert!(
        review_dir.is_dir(),
        "review dir should exist at {review_dir:?}"
    );
    let toml_path = review_dir.join("review-001.toml");
    assert!(toml_path.exists(), "review TOML should exist");
    let toml_content = fs::read_to_string(&toml_path).unwrap();
    assert!(
        toml_content.contains("facet"),
        "TOML should contain facet:\n{toml_content}"
    );
    assert!(
        toml_content.contains("SL-001"),
        "TOML should reference SL-001:\n{toml_content}"
    );

    kill(child);
}

// ── VT-4: full raise → dispose → verify cycle ────────────────────────────

#[test]
fn vt4_full_cycle() {
    let dir = tmp();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();
    seed_slice(root, 1, "Test Slice", "test-slice");

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    // Handshake
    let _ = call(
        &mut stdin,
        &mut reader,
        "initialize",
        Some(&serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0" }
        })),
    );

    // 1. review_new
    let params = tools_call_params(
        "review_new",
        serde_json::json!({ "facet": "design", "target": "SL-001" }),
    );
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    assert!(resp.get("error").is_none(), "review_new: {resp:?}");
    let out: Value = serde_json::from_str(tool_result_text(&resp)).unwrap();
    let created = &out["Created"];
    let review_id = created["id"].as_u64().expect("review id") as u32;

    // 2. review_raise (as raiser)
    let params = tools_call_params(
        "review_raise",
        serde_json::json!({
            "reference": review_id.to_string(),
            "severity": "major",
            "title": "Test Finding",
            "detail": "This is a test finding detail.",
            "as": "raiser"
        }),
    );
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    assert!(resp.get("error").is_none(), "review_raise: {resp:?}");
    let out: Value = serde_json::from_str(tool_result_text(&resp)).unwrap();
    let raised = &out["Raised"];
    let finding_id = raised["finding_id"].as_str().expect("finding_id");
    assert_eq!(raised["review_id"], review_id);

    // 3. review_dispose (as responder)
    let params = tools_call_params(
        "review_dispose",
        serde_json::json!({
            "reference": review_id.to_string(),
            "finding": finding_id,
            "disposition": "fixed",
            "response": "Fixed the issue.",
            "as": "responder"
        }),
    );
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    assert!(resp.get("error").is_none(), "review_dispose: {resp:?}");
    let out: Value = serde_json::from_str(tool_result_text(&resp)).unwrap();
    assert_eq!(out["Disposed"]["finding_id"].as_str().unwrap(), finding_id);

    // 4. review_verify (as raiser)
    let params = tools_call_params(
        "review_verify",
        serde_json::json!({
            "reference": review_id.to_string(),
            "finding": finding_id,
            "note": "looks good",
            "as": "raiser"
        }),
    );
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    assert!(resp.get("error").is_none(), "review_verify: {resp:?}");
    let out: Value = serde_json::from_str(tool_result_text(&resp)).unwrap();
    assert_eq!(out["Verified"]["finding_id"].as_str().unwrap(), finding_id);

    // Verify on-disk state: finding is verified
    let toml_content =
        fs::read_to_string(root.join(".doctrine/review/001/review-001.toml")).unwrap();
    assert!(
        toml_content.contains("status = \"verified\""),
        "finding should be verified in TOML:\n{toml_content}"
    );
    assert!(
        toml_content.contains("disposition = \"fixed\""),
        "disposition should be in TOML:\n{toml_content}"
    );
    assert!(
        toml_content.contains("response = \"Fixed the issue.\""),
        "response should be in TOML:\n{toml_content}"
    );

    kill(child);
}

// ── VT-5: review_show JSON returns valid data ────────────────────────────

#[test]
fn vt5_review_show_json() {
    let dir = tmp();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();
    seed_slice(root, 1, "Test Slice", "test-slice");

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    // Handshake
    let _ = call(
        &mut stdin,
        &mut reader,
        "initialize",
        Some(&serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0" }
        })),
    );

    // Create review + raise a finding so there's data to show
    let params = tools_call_params(
        "review_new",
        serde_json::json!({ "facet": "design", "target": "SL-001", "title": "Show Test" }),
    );
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    assert!(resp.get("error").is_none(), "review_new: {resp:?}");

    // review_show with format=json
    let params = tools_call_params(
        "review_show",
        serde_json::json!({ "reference": "1", "format": "json" }),
    );
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    assert!(resp.get("error").is_none(), "review_show: {resp:?}");

    let text = tool_result_text(&resp);
    let out: Value = serde_json::from_str(text).expect("parse ReviewOutput JSON");
    let showed = &out["Showed"];

    assert!(showed.get("id").is_some(), "should have id field");
    assert!(
        showed.get("canonical").is_some(),
        "should have canonical field"
    );
    assert!(showed.get("title").is_some(), "should have title field");
    assert!(showed.get("status").is_some(), "should have status field");

    kill(child);
}

// ── VT-6: invalid tool → -32601 ──────────────────────────────────────────

#[test]
fn vt6_invalid_tool() {
    let dir = tmp();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    let params = tools_call_params("nonexistent_tool", serde_json::json!({}));
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));

    let err = resp.get("error").expect("should have error");
    assert_eq!(err["code"], -32601, "expected -32601, got {resp:?}");
    assert!(err["message"].as_str().unwrap().contains("Tool not found"));

    kill(child);
}

// ── VT-7: bad args → -32602 ──────────────────────────────────────────────

#[test]
fn vt7_bad_args() {
    let dir = tmp();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    // Missing required fields (severity, title, detail)
    let params = tools_call_params("review_raise", serde_json::json!({ "reference": "1" }));
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));

    let err = resp.get("error").expect("should have error");
    assert_eq!(err["code"], -32602, "expected -32602, got {resp:?}");
    assert!(
        err["data"]["parse_error"].is_string(),
        "should have parse_error data: {resp:?}"
    );

    kill(child);
}

// ── VT-8: raise as responder → ROLE_MISMATCH ─────────────────────────────

#[test]
fn vt8_role_mismatch() {
    let dir = tmp();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();
    seed_slice(root, 1, "Test Slice", "test-slice");

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    // Handshake
    let _ = call(
        &mut stdin,
        &mut reader,
        "initialize",
        Some(&serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0" }
        })),
    );

    // Create review
    let params = tools_call_params(
        "review_new",
        serde_json::json!({ "facet": "design", "target": "SL-001" }),
    );
    let _ = call(&mut stdin, &mut reader, "tools/call", Some(&params));

    // Try to raise as responder
    let params = tools_call_params(
        "review_raise",
        serde_json::json!({
            "reference": "1",
            "severity": "minor",
            "title": "Bad role",
            "detail": "detail",
            "as": "responder"
        }),
    );
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));

    let err = resp.get("error").expect("should have error");
    assert_eq!(err["code"], -32602, "expected -32602, got {resp:?}");
    assert_eq!(
        err["data"]["code"], "ROLE_MISMATCH",
        "expected ROLE_MISMATCH, got {resp:?}"
    );

    kill(child);
}

// ── VT-9: verify already-verified → STATE_MISMATCH ───────────────────────

#[test]
fn vt9_state_mismatch() {
    let dir = tmp();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();
    seed_slice(root, 1, "Test Slice", "test-slice");

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    // Handshake
    let _ = call(
        &mut stdin,
        &mut reader,
        "initialize",
        Some(&serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0" }
        })),
    );

    // Create review + full cycle to get a verified finding
    let params = tools_call_params(
        "review_new",
        serde_json::json!({ "facet": "design", "target": "SL-001" }),
    );
    let _ = call(&mut stdin, &mut reader, "tools/call", Some(&params));

    let params = tools_call_params(
        "review_raise",
        serde_json::json!({
            "reference": "1",
            "severity": "minor",
            "title": "Cycle test",
            "detail": "detail"
        }),
    );
    let _ = call(&mut stdin, &mut reader, "tools/call", Some(&params));

    let params = tools_call_params(
        "review_dispose",
        serde_json::json!({
            "reference": "1",
            "finding": "F-1",
            "disposition": "fixed",
            "response": "done",
            "as": "responder"
        }),
    );
    let _ = call(&mut stdin, &mut reader, "tools/call", Some(&params));

    let params = tools_call_params(
        "review_verify",
        serde_json::json!({
            "reference": "1",
            "finding": "F-1",
            "as": "raiser"
        }),
    );
    let _ = call(&mut stdin, &mut reader, "tools/call", Some(&params));

    // Try to verify again
    let params = tools_call_params(
        "review_verify",
        serde_json::json!({
            "reference": "1",
            "finding": "F-1",
            "as": "raiser"
        }),
    );
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));

    let err = resp.get("error").expect("should have error");
    assert_eq!(err["code"], -32602, "expected -32602, got {resp:?}");
    assert_eq!(
        err["data"]["code"], "STATE_MISMATCH",
        "expected STATE_MISMATCH, got {resp:?}"
    );

    kill(child);
}

// ── Memory MCP E2E tests (SL-131 PHASE-05) ─────────────────────────────

const MEM_C: &str = "mem_0000000000000000000000000000000c";
const MEM_D: &str = "mem_0000000000000000000000000000000d";
const MEM_E: &str = "mem_0000000000000000000000000000000e";

/// Seed a single memory record (adapted from e2e_list_columns_golden.rs).
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
    fs::create_dir_all(&dir).unwrap();
    let key_line = key.map_or(String::new(), |k| format!("memory_key = \"{k}\"\n"));
    let severity = if trust == "low" { "high" } else { "medium" };
    fs::write(
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
             [review]\n\
             verification_state = \"verified\"\n\
             reviewed = \"2026-01-02\"\n\
             \n\
             [git]\n\
             anchor_kind = \"none\"\n\
             \n\
             [ranking]\n\
             severity = \"{severity}\"\n\
             \n\
             [trust]\n\
             trust_level = \"{trust}\"\n"
        ),
    )
    .unwrap();
    fs::write(dir.join("memory.md"), body).unwrap();
    if let Some(k) = key {
        std::os::unix::fs::symlink(uid, root.join(format!(".doctrine/memory/items/{k}"))).ok();
    }
}

/// Seed a memory corpus with varied trust/type for MCP E2E testing.
fn seed_memory_corpus(root: &Path) {
    // High-trust pattern (visible to all trust levels)
    seed_memory(
        root,
        MEM_C,
        Some("mem.pattern.e2e-safe"),
        "pattern",
        "active",
        "high",
        "E2E Safe Pattern",
        "# E2E Safe Pattern\n\nAlways visible.",
    );
    // Medium-trust fact
    seed_memory(
        root,
        MEM_D,
        None,
        "fact",
        "active",
        "medium",
        "E2E Fact",
        "# E2E Fact\n\nA fact with [[mem.pattern.e2e-safe]] link.",
    );
    // Low-trust, high-severity — should be held back
    seed_memory(
        root,
        MEM_E,
        None,
        "fact",
        "active",
        "low",
        "E2E Low-Trust High-Severity",
        "# Low-Trust\n\nThis should be suppressed by trust holdback.",
    );
    let shipped = root.join(".doctrine/memory/shipped");
    fs::create_dir_all(&shipped).unwrap();
}

// EX-4: memory_find + memory_list round-trip against seeded corpus

#[test]
fn memory_find_and_list_roundtrip() {
    let dir = tmp();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();
    seed_memory_corpus(root);

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    // memory_find: scoped query for "safe"
    let params = tools_call_params("memory_find", serde_json::json!({ "query": "safe" }));
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    assert!(resp.get("error").is_none(), "memory_find: {resp:?}");

    let text = tool_result_text(&resp);
    let out: Value = serde_json::from_str(text).expect("parse memory_find JSON");
    assert_eq!(out["kind"], "memory_find");
    assert!(
        out["total"].as_u64().unwrap() >= 1,
        "should find at least 1 memory"
    );
    let rows = out["rows"].as_array().unwrap();
    for row in rows {
        assert!(row.get("uid").is_some());
        assert!(row.get("key").is_some());
        assert!(row.get("type").is_some());
        assert!(row.get("held_back_on_retrieve").is_some());
    }

    // memory_list with type filter
    let params = tools_call_params("memory_list", serde_json::json!({ "type": "fact" }));
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    assert!(resp.get("error").is_none(), "memory_list: {resp:?}");

    let text = tool_result_text(&resp);
    let out: Value = serde_json::from_str(text).expect("parse memory_list JSON");
    assert_eq!(out["kind"], "memory");
    assert_eq!(out["total"], 2, "should have 2 facts");
    let rows = out["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 2);

    kill(child);
}

// EX-5: memory_retrieve with min_trust: "high" suppresses low-trust memory

#[test]
fn memory_retrieve_min_trust_suppression() {
    let dir = tmp();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();
    seed_memory_corpus(root);

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    // memory_retrieve with min_trust: "high" should suppress MEM_E (low trust, high severity)
    let params = tools_call_params(
        "memory_retrieve",
        serde_json::json!({
            "query": "Low-Trust",
            "min_trust": "high"
        }),
    );
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    assert!(
        resp.get("error").is_none(),
        "memory_retrieve high: {resp:?}"
    );

    let text = tool_result_text(&resp);
    assert!(
        !text.contains("Low-Trust"),
        "low-trust high-severity memory should be suppressed by min_trust:high"
    );

    // Low-trust high-severity is ALSO suppressed by default (medium floor is non-bypassable)
    let params = tools_call_params(
        "memory_retrieve",
        serde_json::json!({ "query": "Low-Trust" }),
    );
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    assert!(
        resp.get("error").is_none(),
        "memory_retrieve default: {resp:?}"
    );

    let text = tool_result_text(&resp);
    assert!(
        !text.contains("Low-Trust"),
        "low-trust high-severity memory should be suppressed by default min_trust"
    );

    kill(child);
}

// EX-6: memory_show returns consumable/notes/backlinks for known uid

#[test]
fn memory_show_consumable_notes_backlinks() {
    let dir = tmp();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();
    seed_memory_corpus(root);

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    let params = tools_call_params(
        "memory_show",
        serde_json::json!({
            "reference": MEM_C,
            "view": "summary"
        }),
    );
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    assert!(resp.get("error").is_none(), "memory_show: {resp:?}");

    let text = tool_result_text(&resp);
    let out: Value = serde_json::from_str(text).expect("parse memory_show JSON");

    let memory = &out["memory"];
    assert!(memory.get("consumable").is_some(), "missing consumable");
    assert!(
        memory.get("held_back_on_retrieve").is_some(),
        "missing held_back"
    );
    assert!(memory.get("backlinks").is_some(), "missing backlinks");
    assert!(
        memory.get("backlinks_total").is_some(),
        "missing backlinks_total"
    );
    assert!(
        memory["consumable"].as_bool().unwrap(),
        "high-trust active should be consumable"
    );
    assert!(
        memory["backlinks_total"].as_u64().unwrap() >= 1,
        "MEM_C should have backlinks from MEM_D"
    );

    assert!(
        out.get("body").is_none(),
        "summary view should exclude body"
    );

    kill(child);
}

// EX-7: memory_retrieve with reference to held-back memory returns error

#[test]
fn memory_retrieve_reference_to_held_back_memory_returns_error() {
    let dir = tmp();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();
    seed_memory_corpus(root);

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    let params = tools_call_params(
        "memory_retrieve",
        serde_json::json!({
            "reference": MEM_E
        }),
    );
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));

    let err = resp
        .get("error")
        .expect("should have error for held-back memory");
    assert_eq!(err["code"], -32603, "held-back should be internal error");
    assert!(
        err["data"]["message"]
            .as_str()
            .unwrap_or("")
            .contains("held back"),
        "expected held-back message, got {resp:?}"
    );

    kill(child);
}
