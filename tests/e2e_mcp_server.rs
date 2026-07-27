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

mod common;

fn bin() -> std::path::PathBuf {
    common::doctrine_bin()
}

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
    Command::new(bin())
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

// ── VT-2: tools/list returns 22 tools ────────────────────────────────────

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
    assert_eq!(tools.len(), 29, "expected 29 tools, got {tools:?}");

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
        "memory_search",
        "memory_retrieve",
        "memory_show",
        "memory_list",
        "memory_validate",
        "memory_record",
        "memory_edit",
        "doctrine_onboard",
        "worker_commit",
        // SL-199 PHASE-03 dispatch funnel write surface.
        "dispatch_import",
        "dispatch_conclude_phase",
        "dispatch_reap",
        // SL-206 PHASE-03 dispatch funnel read surface.
        "dispatch_phase_receipt",
        "dispatch_next_ready",
        "dispatch_authored_divergence",
        // SL-228 PHASE-01 Move-E read surface (only tree-state gets an MCP tool).
        "dispatch_tree_state",
        // SL-228 PHASE-05 — the funnel's evidence-producing write verb.
        "dispatch_verify",
        // SL-228 PHASE-06 — the single-prescription funnel oracle (distinct from
        // `dispatch_next_ready`, the readiness authority's wrapper).
        "dispatch_next",
        // SL-231 PHASE-04 — the bounded, friction-only capture adapter.
        "observation_record",
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

// EX-4: memory_search + memory_list round-trip against seeded corpus

#[test]
fn memory_search_and_list_roundtrip() {
    let dir = tmp();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();
    seed_memory_corpus(root);

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    // memory_search: scoped query for "safe"
    let params = tools_call_params("memory_search", serde_json::json!({ "query": "safe" }));
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    assert!(resp.get("error").is_none(), "memory_search: {resp:?}");

    let text = tool_result_text(&resp);
    let out: Value = serde_json::from_str(text).expect("parse memory_search JSON");
    assert_eq!(out["kind"], "memory_search");
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

// ── SL-164 PHASE-02: memory_record / memory_edit / doctrine_onboard ───

#[test]
fn e2e_memory_record_and_show_roundtrip() {
    let dir = tmp();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();
    fs::create_dir_all(root.join(".doctrine/memory/shipped")).unwrap();

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    let title = format!("e2e-record-test-{}", std::process::id());

    // Record new memory
    let params = tools_call_params(
        "memory_record",
        serde_json::json!({
            "title": &title,
            "memory_type": "fact",
            "summary": "E2E test record summary"
        }),
    );
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    assert!(resp.get("error").is_none(), "memory_record: {resp:?}");

    let text = tool_result_text(&resp);
    assert!(text.contains("Recorded"), "expected Recorded in: {text}");
    let out: Value = serde_json::from_str(text).expect("parse record JSON");
    let uid = out["Recorded"]["uid"].as_str().expect("uid").to_owned();

    // Now show it
    let params = tools_call_params("memory_show", serde_json::json!({"reference": &uid}));
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    assert!(resp.get("error").is_none(), "memory_show: {resp:?}");

    let text = tool_result_text(&resp);
    let out: Value = serde_json::from_str(text).expect("parse show JSON");
    assert_eq!(out["memory"]["uid"], uid);
    assert_eq!(out["memory"]["title"], title);

    kill(child);
}

#[test]
fn e2e_memory_edit_roundtrip() {
    let dir = tmp();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();
    fs::create_dir_all(root.join(".doctrine/memory/shipped")).unwrap();

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    // Record a test memory
    let params = tools_call_params(
        "memory_record",
        serde_json::json!({
            "title": "Pre-edit Title",
            "memory_type": "fact"
        }),
    );
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    assert!(resp.get("error").is_none(), "memory_record: {resp:?}");
    let text = tool_result_text(&resp);
    let out: Value = serde_json::from_str(text).expect("parse record JSON");
    let uid = out["Recorded"]["uid"].as_str().expect("uid").to_owned();

    // Edit the title
    let new_title = format!("Edited-{}", std::process::id());
    let params = tools_call_params(
        "memory_edit",
        serde_json::json!({
            "reference": &uid,
            "title": &new_title
        }),
    );
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    assert!(resp.get("error").is_none(), "memory_edit: {resp:?}");

    let text = tool_result_text(&resp);
    assert!(text.contains("Edited memory"), "expected Edited: {text}");

    // Verify via memory_show
    let params = tools_call_params("memory_show", serde_json::json!({"reference": &uid}));
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    assert!(resp.get("error").is_none(), "memory_show: {resp:?}");

    let text = tool_result_text(&resp);
    let out: Value = serde_json::from_str(text).expect("parse show JSON");
    assert_eq!(out["memory"]["title"], new_title, "title should be edited");

    kill(child);
}

/// The one refusal wording for `body_mode` with no `body`, asserted as a
/// literal because this integration crate cannot see the `pub(crate)` const it
/// mirrors (`memory::BODY_MODE_REQUIRES_BODY`). That duplication is the point:
/// the rule is implemented once, in `run_edit`, and this is the drift detector
/// on the MCP surface that inherits it (SL-230 PHASE-05 D-P5-3).
const BODY_MODE_REQUIRES_BODY: &str = "body_mode requires body — a mode with no body to apply it \
     to is never an edit (CLI: --body-mode requires --body)";

/// Extract a `-32602` refusal's worded detail (`error.data.parse_error`) —
/// `error.message` is the generic "Invalid params" for every such error.
fn parse_error_detail(resp: &Value) -> &str {
    resp["error"]["data"]["parse_error"]
        .as_str()
        .expect("parse_error detail")
}

/// Record a memory over the wire, returning its uid.
fn record_memory(
    stdin: &mut impl Write,
    reader: &mut BufReader<impl std::io::Read>,
    arguments: Value,
) -> String {
    let params = tools_call_params("memory_record", arguments);
    let resp = call(stdin, reader, "tools/call", Some(&params));
    assert!(resp.get("error").is_none(), "memory_record: {resp:?}");
    let out: Value = serde_json::from_str(tool_result_text(&resp)).expect("parse record JSON");
    out["Recorded"]["uid"].as_str().expect("uid").to_owned()
}

/// Read a memory's prose back over the wire. `view: full` is what keeps `body`
/// in the projection (summary drops it), and `body` sits at the TOP level of
/// the show envelope, alongside `memory`.
fn show_body(
    stdin: &mut impl Write,
    reader: &mut BufReader<impl std::io::Read>,
    uid: &str,
) -> String {
    let params = tools_call_params(
        "memory_show",
        serde_json::json!({"reference": uid, "view": "full"}),
    );
    let resp = call(stdin, reader, "tools/call", Some(&params));
    assert!(resp.get("error").is_none(), "memory_show: {resp:?}");
    let out: Value = serde_json::from_str(tool_result_text(&resp)).expect("parse show JSON");
    out["body"].as_str().expect("body in full view").to_owned()
}

// SL-230 PHASE-05 VT-1: the body fields round-trip over the real JSON-RPC
// transport — `memory_record` seeds the prose, `memory_edit` with
// `body_mode: append` GROWS it (the prior text survives, which is what
// separates append from the default replace).
#[test]
fn e2e_memory_body_record_edit_append_roundtrip() {
    let dir = tmp();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();
    fs::create_dir_all(root.join(".doctrine/memory/shipped")).unwrap();

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    let uid = record_memory(
        &mut stdin,
        &mut reader,
        serde_json::json!({
            "title": "Body Roundtrip",
            "memory_type": "fact",
            "body": "recorded prose\n"
        }),
    );

    let recorded = show_body(&mut stdin, &mut reader, &uid);
    assert!(
        recorded.contains("recorded prose"),
        "record must land the body: {recorded:?}"
    );

    let params = tools_call_params(
        "memory_edit",
        serde_json::json!({
            "reference": &uid,
            "body": "appended prose\n",
            "body_mode": "append"
        }),
    );
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    assert!(resp.get("error").is_none(), "memory_edit: {resp:?}");

    let appended = show_body(&mut stdin, &mut reader, &uid);
    assert!(
        appended.contains("recorded prose"),
        "append must preserve the prior text: {appended:?}"
    );
    assert!(
        appended.contains("appended prose"),
        "append must add the new text: {appended:?}"
    );
    assert!(
        appended.len() > recorded.len(),
        "the body must grow: {recorded:?} -> {appended:?}"
    );

    kill(child);
}

// SL-230 PHASE-05 VT-1 (D-P5-3): `body_mode` with no `body` is refused over the
// wire with the SAME worded message the CLI raises — because the MCP adapter
// delegates to `run_edit`, where the rule is implemented exactly once. The
// reference is a real, existing memory, so the refusal is demonstrably the
// totality guard and not a resolution failure.
#[test]
fn e2e_body_mode_without_body_is_rejected() {
    let dir = tmp();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();
    fs::create_dir_all(root.join(".doctrine/memory/shipped")).unwrap();

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    let uid = record_memory(
        &mut stdin,
        &mut reader,
        serde_json::json!({"title": "Mode Without Body", "memory_type": "fact"}),
    );

    let params = tools_call_params(
        "memory_edit",
        serde_json::json!({"reference": &uid, "body_mode": "append"}),
    );
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    let err = resp.get("error").expect("should have error");
    assert_eq!(err["code"], -32602, "{resp:?}");
    let detail = parse_error_detail(&resp);
    assert!(
        detail.contains(BODY_MODE_REQUIRES_BODY),
        "the MCP refusal must carry the CLI's wording verbatim: {detail}"
    );

    kill(child);
}

// RV-313 F-3: `body_mode` on `memory_record` must be REFUSED over the wire, not
// silently ignored. It is absent from the tool's `input_schema` on purpose, but
// `input_schema` is advisory — a client can send the key regardless, and serde
// drops unknown fields silently. So the adapter accepts the field solely to
// forward it into `run_record`, whose existing refusal is the single authored
// copy of the rule (the D-P5-3 discipline the `edit` totality guard already
// follows). The falsifier this pins is the pre-fix behaviour: a caller asking
// for `append` at birth got a silent `replace`.
#[test]
fn e2e_body_mode_on_record_is_rejected() {
    let dir = tmp();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();
    fs::create_dir_all(root.join(".doctrine/memory/shipped")).unwrap();

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    let params = tools_call_params(
        "memory_record",
        serde_json::json!({
            "title": "Mode At Birth",
            "memory_type": "fact",
            "body": "initial prose\n",
            "body_mode": "append"
        }),
    );
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    let err = resp.get("error").expect("body_mode on record must error");
    assert_eq!(err["code"], -32602, "{resp:?}");
    let detail = parse_error_detail(&resp);
    assert!(
        detail.contains("not valid on `record`"),
        "the MCP refusal must carry `run_record`'s wording: {detail}"
    );

    kill(child);
}

#[test]
fn e2e_onboard_returns_non_empty() {
    let dir = tmp();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();
    fs::create_dir_all(root.join(".doctrine/memory/shipped")).unwrap();

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    let params = tools_call_params("doctrine_onboard", serde_json::json!({}));
    let resp = call(&mut stdin, &mut reader, "tools/call", Some(&params));
    assert!(resp.get("error").is_none(), "doctrine_onboard: {resp:?}");

    let text = tool_result_text(&resp);
    assert!(
        !text.is_empty(),
        "doctrine_onboard should return non-empty markdown"
    );
    assert!(
        text.contains("# Doctrine MCP Onboarding"),
        "should contain onboarding header: {text}"
    );
    assert!(
        text.contains("CLI → MCP Tool Mapping"),
        "should contain mapping table: {text}"
    );
    // Contract change (SL-187 PHASE-03): the two-memory "Onboarding Memories"
    // load is gone (now carried by the cached boot sector); doctrine_onboard
    // instead teaches model-band self-identification.
    assert!(
        !text.contains("Onboarding Memories"),
        "onboarding memories section should be dropped: {text}"
    );
    assert!(
        text.contains("model-keys"),
        "should name the `prompt model-keys` command: {text}"
    );
    assert!(
        text.contains("--band model"),
        "should give the model self-resolve directive: {text}"
    );
    assert!(
        text.contains("anthropic/claude-sonnet-4"),
        "should list at least one emitted model key: {text}"
    );

    kill(child);
}

// ── SL-231 PHASE-04 (VT-1): `observation_record` — the bounded, friction-only
//    MCP capture adapter over the shared observation service (design §3.3).
// ─────────────────────────────────────────────────────────────────────────────

/// The MCP capture tool's name, as the wire carries it.
const OBSERVATION_RECORD: &str = "observation_record";

/// A well-formed `UUIDv7` for a caller-supplied `uid`.
const OBS_UID_A: &str = "019f1234-5678-7abc-8def-0123456789ab";
const OBS_UID_B: &str = "019f1234-5678-7abc-8def-0123456789ac";

/// Seed the minimal marker set `root::find` needs to resolve a doctrine root.
fn seed_observation_root(root: &Path) {
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".doctrine/review")).unwrap();
}

/// Call `observation_record` and return the raw JSON-RPC response.
fn record_observation(
    stdin: &mut impl Write,
    reader: &mut BufReader<impl std::io::Read>,
    arguments: Value,
) -> Value {
    let params = tools_call_params(OBSERVATION_RECORD, arguments);
    call(stdin, reader, "tools/call", Some(&params))
}

/// Parse a successful `observation_record` response into its receipt object.
fn receipt_of(resp: &Value) -> Value {
    assert!(resp.get("error").is_none(), "expected a receipt: {resp:?}");
    serde_json::from_str(tool_result_text(resp)).expect("receipt JSON")
}

/// The sorted key set of a JSON object.
fn key_set(v: &Value) -> Vec<String> {
    let mut keys: Vec<String> = v
        .as_object()
        .expect("object")
        .keys()
        .map(String::clone)
        .collect();
    keys.sort();
    keys
}

/// VT-1: the MCP receipt is the CLI receipt — same key set, same kind, same
/// idempotency semantics — and the record it publishes is a real friction
/// observation carrying the MCP enrichment allowlist.
#[test]
fn observation_record_matches_cli_contract() {
    let dir = tmp();
    let root = dir.path();
    seed_observation_root(root);

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    let resp = record_observation(
        &mut stdin,
        &mut reader,
        serde_json::json!({
            "uid": OBS_UID_A,
            "summary": "the funnel refused with no remedy",
            "detail": "worker could not tell which belt fired",
        }),
    );
    let receipt = receipt_of(&resp);

    assert_eq!(
        key_set(&receipt),
        vec!["kind", "outcome", "recorded_at", "rel_path", "uid"],
        "the receipt is the design §3.1 contract, no more and no less: {receipt}"
    );
    assert_eq!(receipt["uid"], OBS_UID_A, "{receipt}");
    assert_eq!(receipt["kind"], "friction", "{receipt}");
    assert_eq!(receipt["outcome"], "created", "{receipt}");
    assert!(
        receipt["recorded_at"]
            .as_str()
            .is_some_and(|s| !s.is_empty()),
        "the server supplies recorded_at: {receipt}"
    );

    // The receipt's rel_path is relative and names a record that really landed.
    let rel = receipt["rel_path"].as_str().expect("rel_path");
    assert!(
        !Path::new(rel).is_absolute(),
        "rel_path must be repository-relative, got {rel}"
    );
    let stored = fs::read_to_string(root.join(rel)).expect("published record");
    assert!(
        stored.contains("the funnel refused with no remedy"),
        "summary must be stored verbatim: {stored}"
    );
    assert!(
        stored.contains("worker could not tell which belt fired"),
        "detail must be stored: {stored}"
    );
    assert!(
        stored.contains("kind = \"friction\""),
        "the record is a friction observation: {stored}"
    );

    // Enrichment allowlist: the NAMED MCP interface / product / command, all
    // marked automatic. Nothing else is invented by the server.
    assert!(
        stored.contains("interface = \"mcp\""),
        "automatic interface must name the MCP surface: {stored}"
    );
    assert!(
        stored.contains("product_surface = \"doctrine\""),
        "automatic product_surface: {stored}"
    );
    assert!(
        stored.contains("command = \"observation_record\""),
        "automatic command names the tool: {stored}"
    );
    assert!(
        stored.contains("interface_origin = \"automatic\""),
        "automatic values carry automatic origin: {stored}"
    );

    // Idempotency: the SAME uid + the SAME caller intent replays, never collides.
    let replay = record_observation(
        &mut stdin,
        &mut reader,
        serde_json::json!({
            "uid": OBS_UID_A,
            "summary": "the funnel refused with no remedy",
            "detail": "worker could not tell which belt fired",
        }),
    );
    let replay_receipt = receipt_of(&replay);
    assert_eq!(replay_receipt["outcome"], "replayed", "{replay_receipt}");

    // Explicit facet values WIN over the automatic enrichment.
    let explicit = record_observation(
        &mut stdin,
        &mut reader,
        serde_json::json!({
            "uid": OBS_UID_B,
            "summary": "explicit facets win",
            "facets": {
                "execution": { "interface": "pi", "interface_origin": "explicit" },
                "correlation": { "agent_id": "agent-ad06" }
            }
        }),
    );
    let explicit_receipt = receipt_of(&explicit);
    let explicit_rel = explicit_receipt["rel_path"].as_str().expect("rel_path");
    let explicit_stored = fs::read_to_string(root.join(explicit_rel)).expect("published record");
    assert!(
        explicit_stored.contains("interface = \"pi\""),
        "an explicit interface must beat the automatic one: {explicit_stored}"
    );
    assert!(
        !explicit_stored.contains("interface = \"mcp\""),
        "the automatic interface must not survive alongside the explicit one: {explicit_stored}"
    );
    assert!(
        explicit_stored.contains("product_surface = \"doctrine\""),
        "unshadowed automatic fields still enrich: {explicit_stored}"
    );
    assert!(
        explicit_stored.contains("agent_id = \"agent-ad06\""),
        "an ALREADY-SUPPLIED opaque agent id is mapped through: {explicit_stored}"
    );

    kill(child);
}

/// VT-1: the CLI and MCP receipts are the SAME contract. Recorded through both
/// adapters into one root, the two receipts carry identical key sets and the
/// same `kind`/`outcome` vocabulary.
#[test]
fn observation_record_receipt_matches_cli_receipt_shape() {
    let dir = tmp();
    let root = dir.path();
    seed_observation_root(root);

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);
    let mcp_receipt = receipt_of(&record_observation(
        &mut stdin,
        &mut reader,
        serde_json::json!({ "summary": "captured over MCP" }),
    ));
    kill(child);

    // `current_dir` is load-bearing: the worker-fork guard resolves the root from
    // the CWD, so driving the CLI from the seeded fixture keeps this hermetic —
    // it must not inherit whatever tree the test binary happens to run in.
    let out = Command::new(bin())
        .args(["observation", "record", "friction", "captured over the CLI"])
        .arg("--path")
        .arg(root)
        .current_dir(root)
        .env_remove("DOCTRINE_WORKER")
        .output()
        .expect("run observation record");
    assert!(out.status.success(), "CLI record failed: {out:?}");
    let cli_receipt: Value = serde_json::from_slice(&out.stdout).expect("CLI receipt JSON");

    assert_eq!(
        key_set(&mcp_receipt),
        key_set(&cli_receipt),
        "the MCP receipt must be the CLI receipt: {mcp_receipt} vs {cli_receipt}"
    );
    assert_eq!(mcp_receipt["kind"], cli_receipt["kind"]);
    assert_eq!(mcp_receipt["outcome"], cli_receipt["outcome"]);
}

/// VT-1: the SERVER resolves the repository root. The caller cannot name one —
/// the schema carries no path field, and a smuggled `path`/`root` argument is
/// refused outright rather than silently honoured.
#[test]
fn observation_record_uses_registered_primary_root() {
    let dir = tmp();
    let root = dir.path();
    seed_observation_root(root);

    let decoy_dir = tmp();
    let decoy = decoy_dir.path();
    seed_observation_root(decoy);

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    // The input schema exposes NO path field — the absence is the mechanism.
    let list = call(&mut stdin, &mut reader, "tools/list", None);
    let tool = list["result"]["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .find(|t| t["name"] == OBSERVATION_RECORD)
        .expect("observation_record is registered")
        .clone();
    let props = key_set(&tool["inputSchema"]["properties"]);
    assert_eq!(
        props,
        vec!["detail", "enrich", "facets", "summary", "uid"],
        "the schema carries exactly the design §3.3 fields: {props:?}"
    );

    // A smuggled caller path is refused, and writes NOTHING to the decoy.
    for key in ["path", "root"] {
        let resp = record_observation(
            &mut stdin,
            &mut reader,
            serde_json::json!({ "summary": "redirect me", key: decoy.to_string_lossy() }),
        );
        let err = resp.get("error").unwrap_or_else(|| {
            panic!("a caller-supplied `{key}` must be refused: {resp:?}");
        });
        assert_eq!(err["code"], -32602, "{resp:?}");
        assert!(
            parse_error_detail(&resp).contains(key),
            "the refusal names the offending key: {resp:?}"
        );
    }
    assert!(
        !decoy.join(".doctrine/observations").exists(),
        "no record may land outside the server-resolved root"
    );

    // A plain call lands under the server-resolved root.
    let receipt = receipt_of(&record_observation(
        &mut stdin,
        &mut reader,
        serde_json::json!({ "summary": "lands in the registered root" }),
    ));
    let rel = receipt["rel_path"].as_str().expect("rel_path");
    assert!(root.join(rel).is_file(), "record must land under {root:?}");
    assert!(
        !decoy.join(rel).exists(),
        "record must NOT land under the decoy root"
    );

    kill(child);
}

/// VT-1: friction ONLY. Measurement authority and the two correction controls
/// are unrepresentable in the schema and explicitly refused on the wire; no
/// record of any other kind can be created through this surface.
#[test]
fn observation_record_refuses_non_friction_authority() {
    let dir = tmp();
    let root = dir.path();
    seed_observation_root(root);

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    let forbidden = [
        // Measurement authority — a machine-source claim.
        serde_json::json!({ "summary": "s", "kind": "measurement" }),
        serde_json::json!({ "summary": "s", "source": "claude-p" }),
        serde_json::json!({ "summary": "s", "counters": { "tokens": 1 } }),
        serde_json::json!({ "summary": "s", "gauges": { "ratio": 1.0 } }),
        // Supersession control.
        serde_json::json!({ "summary": "s", "old_uid": OBS_UID_A, "replacement_uid": OBS_UID_B }),
        // Retraction control.
        serde_json::json!({ "summary": "s", "target_uid": OBS_UID_A }),
    ];
    for args in forbidden {
        let resp = record_observation(&mut stdin, &mut reader, args.clone());
        let err = resp
            .get("error")
            .unwrap_or_else(|| panic!("must refuse {args}: {resp:?}"));
        assert_eq!(err["code"], -32602, "refusing {args}: {resp:?}");
        assert!(
            parse_error_detail(&resp).contains("friction"),
            "the refusal must say this surface creates friction only: {resp:?}"
        );
    }

    // Nothing was published by any refused call.
    assert!(
        !root.join(".doctrine/observations").exists(),
        "a refused call must publish nothing"
    );

    // A missing summary is refused too — friction without a summary is not a
    // capture, and the service would reject it anyway.
    let resp = record_observation(
        &mut stdin,
        &mut reader,
        serde_json::json!({ "detail": "d" }),
    );
    assert!(resp.get("error").is_some(), "summary is required: {resp:?}");

    kill(child);
}

/// VT-1 / EX-5: hostile MCP input is escaped before it is rendered back to the
/// caller. The refusal echoes the offending uid, so it goes through the same
/// terminal escaper the CLI renderer uses — a raw ESC must never reach a client.
#[test]
fn observation_record_escapes_hostile_input_in_refusals() {
    let dir = tmp();
    let root = dir.path();
    seed_observation_root(root);

    let mut child = spawn_server(root);
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    let resp = record_observation(
        &mut stdin,
        &mut reader,
        serde_json::json!({
            "summary": "hostile",
            "uid": "\u{1b}[31mnot-a-uuid\u{1b}[0m\u{7f}\nSECOND LINE",
        }),
    );
    assert!(
        resp.get("error").is_some(),
        "a bad uid must refuse: {resp:?}"
    );
    let detail = parse_error_detail(&resp);
    assert!(
        !detail.contains('\u{1b}'),
        "no raw ESC may survive into a rendered refusal: {detail:?}"
    );
    assert!(
        detail.contains("\\x1b"),
        "the ANSI escape must be neutralised as a literal: {detail:?}"
    );
    assert!(
        !detail.contains('\u{7f}'),
        "no raw DEL may survive: {detail:?}"
    );
    assert!(
        !detail.contains('\n'),
        "the refusal is a single logical line — no forged line: {detail:?}"
    );

    kill(child);
}
