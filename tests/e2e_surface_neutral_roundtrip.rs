//! SL-263 PHASE-04 VT-2 (RV-380 F-2): the real neutral-wire round trip.
//!
//! The in-crate behavioural tests drive the generated `surface.ts` against a
//! shell stub, which proves the transport but cannot observe the child's argv.
//! This test bakes the BUILT doctrine binary as `BIN_PATH`, records a real
//! memory, drives the emitted handler under node with a fixture pi
//! `tool_result`, and asserts the memory block comes back — a genuine round trip
//! through `memory surface --input neutral --format plain` and the
//! `NeutralInput`/`NeutralProbe` decoder, so a one-sided rename of the envelope's
//! key or class vocabulary fails here instead of no-opping pi surfacing in
//! silence.

#![allow(
    clippy::expect_used,
    clippy::tests_outside_test_module,
    reason = "integration test: `expect` is the idiomatic fail-fast, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::process::Command;

mod common;

/// The node driver: load the generated handler, fake a pi host, invoke the
/// `tool_result` handler once, print `RESULT:<json>`.
const DRIVER: &str = r#"
import { pathToFileURL } from "node:url";
const mod = await import(pathToFileURL(process.argv[2]).href);
const handlers = {};
await mod.default({ on: (name, fn) => { handlers[name] = fn; } });
const event = {
  toolName: process.argv[3],
  input: JSON.parse(process.argv[4]),
  content: [{ type: "text", text: "ORIGINAL" }],
};
const ctx = {
  cwd: process.argv[5],
  signal: undefined,
  sessionManager: { getSessionId: () => "sess-roundtrip" },
};
const result = await handlers["tool_result"](event, ctx);
process.stdout.write("RESULT:" + JSON.stringify(result ?? null));
"#;

/// Run `doctrine <args…>` rooted at `cwd`, asserting success; return stdout.
fn run_ok(cwd: &Path, args: &[&str]) -> String {
    let out = common::doctrine_cmd(cwd)
        .args(args)
        .output()
        .expect("spawn doctrine");
    assert!(
        out.status.success(),
        "doctrine {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

#[test]
fn surface_extension_round_trips_through_the_neutral_decoder() {
    if common::under_worker_marker() {
        return;
    } // SL-225 #2: skip in a worker fork

    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();

    // A discoverable doctrine root: git repo + one commit (record captures a git
    // anchor).
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .expect("spawn git");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "t@example.com"]);
    git(&["config", "user.name", "Test"]);
    std::fs::create_dir_all(root.join(".doctrine")).expect("mkdir .doctrine");
    std::fs::write(root.join(".doctrine/.keep"), "").expect("keep");
    git(&["add", ".doctrine"]);
    git(&["commit", "-q", "-m", "base"]);

    // A memory anchored on the path the fixture `tool_result` will probe.
    run_ok(
        root,
        &[
            "memory",
            "record",
            "Round-trip footgun",
            "--type",
            "fact",
            "--path-scope",
            "src/x.rs",
            "--severity",
            "none",
            "--trust",
            "medium",
            "-p",
            root.to_str().expect("utf8 root"),
        ],
    );

    // Generate the extension with the REAL built binary baked as BIN_PATH.
    let install = run_ok(
        root,
        &[
            "boot",
            "install",
            "--agent",
            "codex",
            "-y",
            "-p",
            root.to_str().expect("utf8 root"),
        ],
    );
    assert!(
        install.contains("generated extension .pi/extensions/doctrine/surface.ts"),
        "install must report the surface extension: {install}"
    );
    let surface = root.join(".pi/extensions/doctrine/surface.ts");
    assert!(surface.exists(), "surface.ts generated");
    let generated = std::fs::read_to_string(&surface).expect("read surface.ts");
    assert!(
        generated.contains(&format!(
            "const BIN_PATH = \"{}\";",
            common::doctrine_bin().display()
        )),
        "the built binary is baked as BIN_PATH"
    );

    // Node type-strips `.ts`, but needs the nearest package.json to say "module".
    std::fs::write(root.join("package.json"), r#"{"type":"module"}"#).expect("package.json");
    let driver = root.join("driver.mjs");
    std::fs::write(&driver, DRIVER).expect("driver");

    let output = Command::new("node")
        .arg(&driver)
        .arg(&surface)
        .arg("read")
        .arg(r#"{"path":"src/x.rs"}"#)
        .arg(root)
        .output()
        .expect("run node");
    assert!(
        output.status.success(),
        "node driver failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    let rest = stdout
        .split_once("RESULT:")
        .map(|(_, rest)| rest)
        .unwrap_or_else(|| panic!("driver printed no RESULT: {stdout}"));
    let result: serde_json::Value = serde_json::from_str(rest.trim()).expect("result json");
    let appended = result["content"][1]["text"]
        .as_str()
        .unwrap_or_else(|| panic!("no appended block: {result}"));
    assert!(
        appended.contains("Doctrine memories for this file:"),
        "the path block came back from the real decoder: {appended}"
    );
    assert!(
        appended.contains("Round-trip footgun"),
        "the recorded memory is in the block: {appended}"
    );
}
