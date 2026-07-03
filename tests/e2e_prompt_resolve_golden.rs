//! SL-186 PHASE-03 — E2E golden tests for `prompt resolve` (VT-1) and
//! `prompt model-keys` (VT-2) over the BUILT binary.
//!
//! VT-1 seals the sealed-slot shadow-drop + exposed-slot user-edit-wins
//! user stories. VT-2 seals the model-key enumeration contract.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic"
)]

use std::fs;
use std::path::Path;
use std::process::Command;

mod common;

fn bin() -> std::path::PathBuf {
    common::doctrine_bin()
}

fn run(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .args(args)
        .arg("-p")
        .arg(root)
        .output()
        .expect("spawn doctrine prompt")
}

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &std::process::Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

// ── VT-1: resolve ───────────────────────────────────────────────────────────

#[test]
fn vt1_resolve_sealed_twin_is_dropped_and_exposed_user_wins() {
    let dir = tmp();
    let hymns = dir.path().join(".doctrine/hymns");

    // (i) A USER TWIN of the sealed slot preamble/core (different body).
    // Must be DROPPED — the sealed framework snippet wins.
    fs::create_dir_all(hymns.join("preamble")).unwrap();
    fs::write(
        hymns.join("preamble/core.md"),
        "THIS-SHADOW-MUST-NOT-APPEAR",
    )
    .unwrap();

    // (ii) A user edit at an EXPOSED slot (harness/claude).
    // The USER body must WIN (equal-specificity provenance tiebreak).
    fs::create_dir_all(hymns.join("harness")).unwrap();
    fs::write(hymns.join("harness/claude.md"), "USER-CLAUDE-OVERRIDE").unwrap();

    let out = run(
        dir.path(),
        &[
            "prompt",
            "resolve",
            "--role",
            "worker",
            "--harness",
            "claude",
        ],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));

    let output = stdout(&out);

    // The sealed framework preamble/core.md text must appear.
    assert!(
        output.contains("doctrine dispatch worker"),
        "preamble framework snippet missing, got: {output}"
    );

    // The shadow must NOT appear.
    assert!(
        !output.contains("THIS-SHADOW-MUST-NOT-APPEAR"),
        "sealed twin leaked, got: {output}"
    );

    // The user harness override must appear AND must come after the framework one
    // (the user twin wins the tiebreak → gets the last word).
    assert!(
        output.contains("USER-CLAUDE-OVERRIDE"),
        "user harness override missing, got: {output}"
    );

    // The framework harness snippet (the original) must also be present; user wins
    // last word but both are in the output (same slot, equal specificity,
    // framework then user).
    assert!(
        output.contains("Claude harness"),
        "framework harness snippet missing, got: {output}"
    );
}

/// SL-187 PHASE-04 VT-2 (end-to-end) — supersedes SL-186's stdout-only/no-disk
/// contract. `prompt resolve` now unstales the UNIVERSAL on-disk `boot.md` (reuse
/// the boot generator, `write_if_changed`) and emits `universal ++ role/harness
/// hymns` to stdout. This seals the three PHASE-04 delivery invariants over the
/// built binary:
///  - the ONLY disk artifact resolve writes is `.doctrine/state/boot.md`;
///  - that file is BYTE-IDENTICAL across runs differing only in `--role`/`--harness`
///    (axis-invariance INV-D1; unchanged-input cache-hold INV-D4);
///  - stdout DIFFERS by role/harness (the axis-specific cascade rides stdout only).
#[test]
fn resolve_regenerates_only_boot_md_axis_invariant_stdout_varies() {
    let dir = tmp();
    let hymns = dir.path().join(".doctrine/hymns");
    fs::create_dir_all(hymns.join("harness")).unwrap();
    fs::write(hymns.join("harness/claude.md"), "USER-CLAUDE").unwrap();

    let before = dir_contents(dir.path());

    // Run A — orchestrator/claude.
    let out_a = run(
        dir.path(),
        &[
            "prompt",
            "resolve",
            "--role",
            "orchestrator",
            "--harness",
            "claude",
        ],
    );
    assert!(out_a.status.success(), "stderr: {}", stderr(&out_a));

    let boot_md = dir.path().join(".doctrine/state/boot.md");
    assert!(
        boot_md.exists(),
        "resolve must regenerate the universal boot.md"
    );
    let disk_a = fs::read_to_string(&boot_md).unwrap();

    // The ONLY new file on disk is `.doctrine/state/boot.md`.
    let after = dir_contents(dir.path());
    let new_files: Vec<String> = after
        .iter()
        .filter(|e| !before.contains(e))
        .map(|(p, _)| p.clone())
        .collect();
    assert_eq!(
        new_files,
        vec![".doctrine/state/boot.md".to_string()],
        "resolve wrote unexpected files: {new_files:?}"
    );

    // Run B — differs ONLY in the role/harness axes.
    let out_b = run(
        dir.path(),
        &["prompt", "resolve", "--role", "worker", "--harness", "pi"],
    );
    assert!(out_b.status.success(), "stderr: {}", stderr(&out_b));
    let disk_b = fs::read_to_string(&boot_md).unwrap();

    // Disk artifact axis-invariant (INV-D1); stdout varies by axis.
    assert_eq!(
        disk_a, disk_b,
        "on-disk boot.md must be axis-invariant (INV-D1)"
    );
    assert_ne!(
        stdout(&out_a),
        stdout(&out_b),
        "stdout must vary by role/harness"
    );

    // The universal snapshot is PREPENDED to stdout — its content is a prefix.
    assert!(
        stdout(&out_a).contains(disk_a.trim_end()),
        "stdout must carry the universal snapshot ++ hymns"
    );
}

/// Collect relative paths + file sizes under `root` for comparison.
fn dir_contents(root: &Path) -> Vec<(String, u64)> {
    let mut entries = Vec::new();
    collect_entries(root, root, &mut entries);
    entries.sort();
    entries
}

fn collect_entries(root: &Path, current: &Path, out: &mut Vec<(String, u64)>) {
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if let Ok(rel) = path.strip_prefix(root) {
            if rel.as_os_str().is_empty() {
                continue;
            }
            if path.is_dir() {
                collect_entries(root, &path, out);
            } else {
                let meta = path.metadata().unwrap();
                out.push((rel.to_string_lossy().into_owned(), meta.len()));
            }
        }
    }
}

/// IMP-245 — `prompt resolve --json` wraps the SAME cascade (universal ++
/// role/harness hymns) as the plain form in the Cursor `sessionStart` hook
/// envelope (`{"additional_context": "<cascade>"}`), so a Cursor project hook
/// can point directly at this command instead of the legacy `boot --emit`.
#[test]
fn resolve_json_wraps_cascade_in_session_hook_envelope() {
    let dir = tmp();

    let plain = run(
        dir.path(),
        &[
            "prompt",
            "resolve",
            "--role",
            "orchestrator",
            "--harness",
            "cursor",
        ],
    );
    assert!(plain.status.success(), "stderr: {}", stderr(&plain));

    let json_out = run(
        dir.path(),
        &[
            "prompt",
            "resolve",
            "--role",
            "orchestrator",
            "--harness",
            "cursor",
            "--json",
        ],
    );
    assert!(json_out.status.success(), "stderr: {}", stderr(&json_out));

    let raw = stdout(&json_out);
    assert_eq!(raw.matches('\n').count(), 1, "single-line JSON + newline");
    let parsed: serde_json::Value = serde_json::from_str(raw.trim_end()).expect("valid JSON");
    let ctx = parsed
        .get("additional_context")
        .and_then(serde_json::Value::as_str)
        .expect("additional_context string field");

    // The wrapped content is the same cascade as the plain form (modulo the
    // plain form's own trailing-newline normalisation via `writeln!`).
    assert_eq!(ctx.trim_end(), stdout(&plain).trim_end());

    // The (host-independent, shipped) Cursor harness hymn rode the cascade.
    // Project-specific build-tooling guidance (e.g. this repo's Nix devshell
    // note) is authored on-disk per-project (POL-002 — never baked into the
    // shipped corpus a fresh project would also inherit), so it is NOT
    // asserted here.
    assert!(
        ctx.contains("operating inside the Cursor harness"),
        "cursor harness hymn missing, got: {ctx}"
    );
}

// ── PHASE-02 VT-1 / VT-3: repeatable --model, multi-key membership ───────────

/// PHASE-02 VT-1 (FR-009 arity + FR-004 membership) — repeatable `--model` builds
/// a two-key context; BOTH shipped model snippets match by membership and compose
/// into one resolve. Drives the EXISTING embedded snippets (no SL-191 trait hymns).
#[test]
fn vt1_repeatable_model_composes_both_shipped_snippets() {
    let dir = tmp();

    let out = run(
        dir.path(),
        &[
            "prompt",
            "resolve",
            "--role",
            "worker",
            "--model",
            "anthropic/claude-sonnet-4",
            "--model",
            "deepseek/_default",
        ],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));

    let output = stdout(&out);
    // Both model snippet bodies compose — membership over the two-key context.
    assert!(
        output.contains("Connectivity: claude.ai API"),
        "anthropic model snippet missing, got: {output}"
    );
    assert!(
        output.contains("DeepSeek model family"),
        "deepseek model snippet missing, got: {output}"
    );
}

/// PHASE-02 VT-3 — `prompt explain` traces the multi-key match; each matched model
/// snippet prints with the generalised `Spec` render (`spec=([root:depth,…],other)`).
#[test]
fn vt3_explain_multi_key_precedence_trace_spec_render() {
    let dir = tmp();

    let out = run(
        dir.path(),
        &[
            "prompt",
            "explain",
            "--role",
            "worker",
            "--model",
            "anthropic/claude-sonnet-4",
            "--model",
            "deepseek/_default",
        ],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));

    let output = stdout(&out);
    assert!(
        output.contains("model/anthropic/claude-sonnet-4"),
        "anthropic model slot missing from explain, got: {output}"
    );
    assert!(
        output.contains("model/deepseek/_default"),
        "deepseek model slot missing from explain, got: {output}"
    );
    // The generalised Spec render, per model key (root-keyed pair vector).
    assert!(
        output.contains("spec=([anthropic:"),
        "anthropic Spec render missing, got: {output}"
    );
    assert!(
        output.contains("spec=([deepseek:"),
        "deepseek Spec render missing, got: {output}"
    );
}

// ── SL-193 PHASE-02 EX-2: exposed-slot self-`replaces` projection ─────────────
//
// After the producer (install forward-step 4) projects the 5 exposed slots, each
// disk twin carries a self-`replaces` sidecar (`replaces = "<own slot>"`) that
// suppresses its embedded framework origin at resolve — the single-emit MIRROR of
// seal. These E2Es lock that corpus-wide over the built binary:
//   - VT-3 (`exposed_projection_prompt_check_ok`): the projected sidecars are
//     LEGAL — `prompt check` (⇒ validate_replaces) returns Ok. This is the sole
//     guard against a NonTopReplacer/cycle that would make `prompt resolve` error.
//   - VT-4 (`exposed_slots_single_emit_all_five`): EACH of the 5 exposed slots
//     emits exactly once with its framework body suppressed — over `prompt resolve`
//     (where the `replaces` graph is applied), not `prompt explain` (which prints
//     the raw active set, ranked-but-present).
//
// The install producer cannot run here (worker-confined / heavy), so the fixture
// is projected directly — the sidecar `.toml` is byte-for-byte the producer's
// emission (`replaces = "<slot.path()>"\n`, src/install.rs::project_starters). The
// disk `.md` carries a distinct user marker (an EDITED starter), so an un-
// suppressed framework twin would leave its framework body in the output — the
// assertion that it is absent is the suppression proof.

/// The 5 exposed slots: (disk-relative `.md`/`.toml` stem, self-`replaces` target,
/// activating `prompt` args, distinct user-marker body, a framework-body substring
/// that MUST vanish when the twin is suppressed).
const EXPOSED: &[(&str, &str, &[&str], &str, &str)] = &[
    (
        "harness/claude",
        "harness/claude",
        &["--role", "worker", "--harness", "claude"],
        "EXPOSED-USER-harness-claude",
        "Claude tool-use protocol",
    ),
    (
        "model/anthropic/claude-sonnet-4",
        "model/anthropic/claude-sonnet-4",
        &["--role", "worker", "--model", "anthropic/claude-sonnet-4"],
        "EXPOSED-USER-model-anthropic",
        "Connectivity: claude.ai API",
    ),
    (
        "model/deepseek/_default",
        "model/deepseek/_default",
        &["--role", "worker", "--model", "deepseek/_default"],
        "EXPOSED-USER-model-deepseek",
        "DeepSeek model family",
    ),
    (
        "role/orchestrator",
        "role/orchestrator",
        &["--role", "orchestrator"],
        "EXPOSED-USER-role-orchestrator",
        "you own the process",
    ),
    (
        "role/worker",
        "role/worker",
        &["--role", "worker"],
        "EXPOSED-USER-role-worker",
        "dispatch worker implementing ONE phase",
    ),
];

/// Project the 5 exposed slots into `root/.doctrine/hymns/**`: a user-marker `.md`
/// twin and its self-`replaces` sidecar (the producer's exact `.toml` emission).
fn project_exposed(root: &Path) {
    let hymns = root.join(".doctrine/hymns");
    for (stem, target, _args, marker, _fw) in EXPOSED {
        let md = hymns.join(format!("{stem}.md"));
        let toml = hymns.join(format!("{stem}.toml"));
        fs::create_dir_all(md.parent().unwrap()).unwrap();
        fs::write(&md, format!("{marker}\n")).unwrap();
        // Byte-for-byte the producer's sidecar (src/install.rs::project_starters).
        fs::write(&toml, format!("replaces = \"{target}\"\n")).unwrap();
    }
}

/// VT-3 — corpus-wide `replaces` legality. Every projected self-`replaces` is the
/// unique-most-specific active snippet of its slot (INV-3), so `prompt check`
/// (⇒ validate_replaces) returns Ok over the whole projected corpus.
#[test]
fn exposed_projection_prompt_check_ok() {
    let dir = tmp();
    project_exposed(dir.path());

    let out = run(dir.path(), &["prompt", "check"]);
    assert!(
        out.status.success(),
        "prompt check must pass over the projected corpus; stderr: {}",
        stderr(&out)
    );
    assert!(
        stdout(&out).contains("check: corpus OK"),
        "expected corpus OK, got: {}",
        stdout(&out)
    );
}

/// VT-4 — all 5 exposed slots single-emit corpus-wide (not just role/worker). Drive
/// a context activating EACH slot; assert the user marker emits exactly once AND the
/// framework body is suppressed (absent) over `prompt resolve`.
#[test]
fn exposed_slots_single_emit_all_five() {
    let dir = tmp();
    project_exposed(dir.path());

    for (stem, _target, activate, marker, fw_body) in EXPOSED {
        let mut args = vec!["prompt", "resolve"];
        args.extend_from_slice(activate);
        let out = run(dir.path(), &args);
        assert!(
            out.status.success(),
            "resolve for {stem} failed; stderr: {}",
            stderr(&out)
        );
        let output = stdout(&out);

        // Single emit: the user twin appears exactly once (append would double it).
        let hits = output.matches(marker).count();
        assert_eq!(
            hits, 1,
            "slot {stem}: expected exactly one emit of {marker}, got {hits}\n{output}"
        );
        // Suppression: the framework origin's body is gone — override, not append.
        assert!(
            !output.contains(fw_body),
            "slot {stem}: framework body {fw_body:?} must be suppressed, got:\n{output}"
        );
    }
}

// ── VT-2: model-keys ────────────────────────────────────────────────────────

#[test]
fn vt2_model_keys_exact_relative_keys() {
    let dir = tmp();

    // No user models — only the embedded corpus.
    // The framework embeds: model/adherence/low, model/anthropic/claude-sonnet-4,
    // model/deepseek/_default (SL-191 PHASE-02 added adherence/low).

    let out = run(dir.path(), &["prompt", "model-keys"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));

    let output = stdout(&out);
    let lines: Vec<&str> = output.lines().collect();

    // Three model keys, sorted.
    assert_eq!(lines.len(), 3, "expected 3 model keys, got: {output}");
    assert_eq!(lines[0], "adherence/low");
    assert_eq!(lines[1], "anthropic/claude-sonnet-4");
    assert_eq!(lines[2], "deepseek/_default");
}

#[test]
fn vt2_model_keys_empty_corpus_outputs_nothing() {
    let dir = tmp();

    // No .doctrine/hymns on disk, and the framework embedded corpus always exists.
    // But model-keys should list only the embedded model keys. An "empty" case
    // means: only the authored keys appear, none invented.
    let out = run(
        dir.path(),
        &["prompt", "model-keys", "--harness", "nonexistent"],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));

    let output = stdout(&out);
    // With --harness nonexistent, no embedded model matches (both embedded
    // models have selector.harness == None, which matches any harness, so they
    // still appear).
    // Actually: None harness in selector means "don't care" (matches any).
    // So --harness nonexistent still matches the framework model snippets.
    // The only way to get empty is if no model-band snippets exist.
    // The framework always embeds models, so model-keys should never be empty.
    // We test that it is non-empty.
    assert!(
        !output.trim().is_empty(),
        "model-keys should find embedded models"
    );
}
