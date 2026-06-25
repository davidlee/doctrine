// SPDX-License-Identifier: GPL-3.0-only
//! SL-152 PHASE-02 — `doctrine worktree create-fork` end-to-end over the BUILT
//! binary (design §5.1/§5.2). The claude `WorktreeCreate` hook verb: reads the thin
//! `{cwd, name}` payload on STDIN, resolves the coord-tree root from the PAYLOAD cwd
//! (`git -C <cwd> --show-toplevel`, NOT the process cwd — G2/I5), discriminates
//! POSITIONALLY (cwd IS the arming dir `<root>/.doctrine/state/dispatch/spawn` ⇒ Fork
//! off the arming `base`; anywhere else ⇒ benign Passthrough), and prints the created
//! absolute path ALONE on stdout.
//!
//! * VT-1 — base==B drift-immune: a Fork pins the arming `base` even when HEAD moved.
//! * VT-2/VT-3 — provision source = coord tree (I2): a gitignored sentinel present in
//!   the coord tree (absent from any commit) lands in the created fork; Passthrough is
//!   detached, provisioned, and NOT worker-marked; Fork IS worker-marked.
//! * VT-4 — fail-closed: malformed/empty/cwdless payload ⇒ named refusal (no panic);
//!   a cwd outside any repo ⇒ `no-root`.
//! * VT-5 — name collision: a live `dispatch/<name>`/`.worktrees/<name>` ⇒ refusal.
//! * VT-7 — stdout discipline (G1/D11): stdout is EXACTLY the path, no `KEY=value`.
//! * VT-8 — pass-through compensation (G3): a forced provision failure leaves NO tree.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");
const CREATE: &[&str] = &["worktree", "create-fork"];

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("spawn git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A real repo standing in for the coordination tree (create-fork resolves `root`
/// from the payload cwd's `--show-toplevel`, so a plain repo is a faithful root).
fn init_repo(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    std::fs::write(dir.join("a.txt"), "hello").unwrap();
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "base"]);
}

/// Seed a gitignored, allowlisted sentinel present ONLY in the coord tree's working
/// dir (never committed) — its arrival in a created fork proves the provision SOURCE
/// is the coord tree, not the fresh checkout (I2).
fn seed_sentinel(root: &Path) {
    std::fs::write(root.join(".gitignore"), "sentinel.txt\n").unwrap();
    std::fs::write(root.join(".worktreeinclude"), "sentinel.txt\n").unwrap();
    std::fs::write(root.join("sentinel.txt"), "from coord tree").unwrap();
}

/// Arm a dispatch-worker spawn: write `<root>/.doctrine/state/dispatch/spawn/base`
/// and return the canonicalised arming dir (the payload cwd for a Fork).
fn arm(root: &Path, base: &str) -> PathBuf {
    let dir = root.join(".doctrine/state/dispatch/spawn");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("base"), base).unwrap();
    std::fs::canonicalize(&dir).unwrap()
}

fn payload(cwd: &Path, name: &str) -> String {
    format!("{{\"cwd\": \"{}\", \"name\": \"{}\"}}", cwd.display(), name)
}

/// Run `doctrine <args>` with `payload` on STDIN. Process cwd = `cwd` (mirrors the
/// hook firing with the orchestrator's cwd). CARGO_TARGET_DIR/DOCTRINE_WORKER cleared
/// so provisioning into the fork is deterministic and the worker guard sees a clean
/// (markerless) parent.
fn run(cwd: &Path, payload: &str, args: &[&str]) -> Output {
    let mut child = Command::new(BIN)
        .args(args)
        .current_dir(cwd)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("DOCTRINE_WORKER")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn doctrine");
    child
        .stdin
        .take()
        .expect("stdin piped")
        .write_all(payload.as_bytes())
        .expect("write payload");
    child.wait_with_output().expect("wait doctrine")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

fn assert_refusal(out: &Output, token: &str) {
    assert!(
        !out.status.success(),
        "must refuse ({token}); stdout: {}, stderr: {}",
        stdout(out),
        stderr(out)
    );
    assert!(
        stderr(out).contains(token),
        "refusal names `{token}`; stderr: {}",
        stderr(out)
    );
}

fn worker_marker(dir: &Path) -> PathBuf {
    dir.join(".doctrine/state/dispatch/worker")
}

/// Assert stdout is EXACTLY the created path — one line, no `KEY=value` env contract
/// (the D11/G1 discipline that distinguishes create-fork from `fork`). Returns the dir.
fn assert_stdout_is_path_only(out: &Output) -> PathBuf {
    let s = stdout(out);
    assert_eq!(
        s.lines().count(),
        1,
        "stdout is exactly one line (the path); got: {s:?}"
    );
    assert!(
        !s.contains('='),
        "no KEY=value env contract on stdout (D11); got: {s:?}"
    );
    PathBuf::from(s.trim())
}

// --- VT-1 + VT-2 + VT-7(Fork): pin base B, provision from coord tree, path-only ---

#[test]
fn fork_pins_base_provisions_from_coord_tree_marks_worker_and_prints_path_only() {
    let root = tempfile::tempdir().unwrap();
    init_repo(root.path());
    let root_canon = std::fs::canonicalize(root.path()).unwrap();
    seed_sentinel(root.path());

    // Capture B, then ADVANCE HEAD past it with a tracked commit.
    let b = git(root.path(), &["rev-parse", "HEAD"]);
    std::fs::write(root.path().join("drift.txt"), "post-B").unwrap();
    git(root.path(), &["add", "drift.txt"]);
    git(root.path(), &["commit", "-q", "-m", "advance HEAD past B"]);
    assert_ne!(b, git(root.path(), &["rev-parse", "HEAD"]), "HEAD advanced");

    let spawn = arm(root.path(), &b);
    let out = run(&spawn, &payload(&spawn, "agent-deadbeef"), CREATE);
    assert!(
        out.status.success(),
        "fork create must succeed; stderr: {}",
        stderr(&out)
    );

    let dir = assert_stdout_is_path_only(&out);
    assert_eq!(
        dir,
        root_canon.join(".worktrees/agent-deadbeef"),
        "created at <root>/.worktrees/<name>"
    );
    // VT-1: the fork HEAD is EXACTLY B though HEAD advanced (base explicit, drift-immune).
    assert_eq!(
        git(&dir, &["rev-parse", "HEAD"]),
        b,
        "fork pinned to base B, not the moved HEAD"
    );
    // VT-2: the gitignored sentinel was provisioned FROM the coord tree.
    assert_eq!(
        std::fs::read_to_string(dir.join("sentinel.txt")).unwrap(),
        "from coord tree",
        "provision source is the coord tree (I2)"
    );
    // Fork arm is worker-marked.
    assert!(
        worker_marker(&dir).exists(),
        "Fork worktree is worker-marked"
    );
}

// --- VT-3 + VT-7(Passthrough): benign detached tree, provisioned, NOT marked ---

#[test]
fn passthrough_creates_detached_provisions_and_is_not_worker_marked() {
    let root = tempfile::tempdir().unwrap();
    init_repo(root.path());
    let root_canon = std::fs::canonicalize(root.path()).unwrap();
    seed_sentinel(root.path());

    // Benign: payload cwd = the coord ROOT (NOT the arming dir) ⇒ Passthrough.
    let out = run(&root_canon, &payload(&root_canon, "bold-oak-a3f2"), CREATE);
    assert!(
        out.status.success(),
        "passthrough must succeed; stderr: {}",
        stderr(&out)
    );

    let dir = assert_stdout_is_path_only(&out);
    assert_eq!(dir, root_canon.join(".worktrees/bold-oak-a3f2"));
    // Detached HEAD at the coord tree tip.
    assert_eq!(
        git(&dir, &["rev-parse", "HEAD"]),
        git(root.path(), &["rev-parse", "HEAD"]),
        "detached at the coord tree HEAD"
    );
    let symref = Command::new("git")
        .arg("-C")
        .arg(&dir)
        .args(["symbolic-ref", "-q", "HEAD"])
        .output()
        .unwrap();
    assert!(
        !symref.status.success(),
        "passthrough worktree is in detached HEAD state (no branch)"
    );
    // Provisioned via the SAME copier, but NOT worker-marked (I2).
    assert_eq!(
        std::fs::read_to_string(dir.join("sentinel.txt")).unwrap(),
        "from coord tree",
        "passthrough provisioned from the coord tree"
    );
    assert!(
        !worker_marker(&dir).exists(),
        "passthrough worktree is NOT worker-marked"
    );
}

// --- VT-4: fail-closed — malformed / empty / cwdless / rootless ⇒ named refusal ---

#[test]
fn malformed_empty_and_rootless_payloads_refuse_without_panic() {
    let root = tempfile::tempdir().unwrap();
    init_repo(root.path());
    let root_canon = std::fs::canonicalize(root.path()).unwrap();

    // Absent cwd ⇒ missing-cwd.
    assert_refusal(
        &run(&root_canon, "{\"name\": \"agent-abc123\"}", CREATE),
        "missing-cwd",
    );
    // Malformed JSON folds to an empty payload ⇒ missing-cwd.
    assert_refusal(&run(&root_canon, "not json at all", CREATE), "missing-cwd");
    // Empty stdin ⇒ missing-cwd.
    assert_refusal(&run(&root_canon, "", CREATE), "missing-cwd");
    // Bad name ⇒ bad-name, with the specific sanitiser reason surfaced.
    let out = run(&root_canon, &payload(&root_canon, "a/b"), CREATE);
    assert_refusal(&out, "bad-name");
    assert!(
        stderr(&out).contains("slash"),
        "names the specific reason; stderr: {}",
        stderr(&out)
    );
    // cwd resolves but is OUTSIDE any git repo ⇒ no-root.
    let outside = tempfile::tempdir().unwrap();
    assert_refusal(
        &run(
            &root_canon,
            &payload(outside.path(), "agent-abc123"),
            CREATE,
        ),
        "no-root",
    );
}

// --- VT-5: name collision on BOTH arms (distinct token per arm) ---

#[test]
fn name_collision_refuses_on_both_arms() {
    let root = tempfile::tempdir().unwrap();
    init_repo(root.path());
    let root_canon = std::fs::canonicalize(root.path()).unwrap();
    let b = git(root.path(), &["rev-parse", "HEAD"]);
    let spawn = arm(root.path(), &b);

    // First Fork lands `dispatch/agent-dup` + `.worktrees/agent-dup`.
    assert!(
        run(&spawn, &payload(&spawn, "agent-dup"), CREATE)
            .status
            .success(),
        "first fork succeeds"
    );
    // Fork arm collision ⇒ fork_core's `fork-refused` (shared-machinery token).
    assert_refusal(
        &run(&spawn, &payload(&spawn, "agent-dup"), CREATE),
        "fork-refused",
    );
    // Passthrough arm collision on the live `.worktrees/agent-dup` dir ⇒ name-collision.
    assert_refusal(
        &run(&root_canon, &payload(&root_canon, "agent-dup"), CREATE),
        "name-collision",
    );
}

// --- VT-8: pass-through compensation — a forced provision failure leaves no tree ---

#[test]
fn passthrough_compensates_on_provision_failure() {
    let root = tempfile::tempdir().unwrap();
    init_repo(root.path());
    let root_canon = std::fs::canonicalize(root.path()).unwrap();
    // A `.worktreeinclude` naming a WITHHELD tier ⇒ run_provision fails closed.
    std::fs::write(root.path().join(".worktreeinclude"), ".doctrine/state/*\n").unwrap();

    let out = run(&root_canon, &payload(&root_canon, "agent-doomed"), CREATE);
    assert!(
        !out.status.success(),
        "a provision failure must exit non-zero; stdout: {}",
        stdout(&out)
    );
    // G3: NO worktree dir / registration survives the compensation.
    let dir = root_canon.join(".worktrees/agent-doomed");
    assert!(!dir.exists(), "compensation removed the half-created dir");
    let live = git(root.path(), &["worktree", "list", "--porcelain"]);
    assert!(
        !live.contains("agent-doomed"),
        "no surviving worktree registration; list: {live}"
    );
}
