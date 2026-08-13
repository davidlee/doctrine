// SPDX-License-Identifier: GPL-3.0-only
//! SL-152 PHASE-02 — `doctrine worktree create-fork` end-to-end over the BUILT
//! binary (design §5.1/§5.2). The claude `WorktreeCreate` hook verb: reads the thin
//! `{cwd, name}` payload on STDIN, resolves the coord-tree root from the PAYLOAD cwd
//! (`git -C <cwd> --show-toplevel`, NOT the process cwd — G2/I5), and prints the
//! created absolute path ALONE on stdout.
//!
//! **SL-254 PHASE-06 — retargeted onto the Passthrough arm.** The Fork arm and its
//! positional arming-dir discriminator are deleted with the claude dispatch arm (D1):
//! `worktree fork --worker` is the sole worker-fork writer, so every harness-created
//! worktree reaching this verb is benign. The Fork-arm cases die with their subject;
//! the base-pinning claim they carried survives in `e2e_dispatch_h1_integration.rs`,
//! retargeted onto that writer. Everything below is the Passthrough contract, which
//! is unchanged and still load-bearing — the hook entry stays precisely so a
//! harness-created tree is still provisioned rather than left bare.
//!
//! * VT-3 — provision source = coord tree (I2): a gitignored sentinel present in the
//!   coord tree (absent from any commit) lands in the created tree, which is DETACHED
//!   at the coord tip and claims no branch. (SL-254 PHASE-05: the arms used to be
//!   told apart by the worker marker too. The marker went with `DEC-207`; the
//!   detached-HEAD/no-branch property is asserted directly, and is now the proof that
//!   this verb cannot mint a worker fork at all.)
//! * VT-4 — fail-closed: malformed/empty/cwdless payload ⇒ named refusal (no panic);
//!   a cwd outside any repo ⇒ `no-root`.
//! * VT-5 — name collision: a live `.worktrees/<name>` ⇒ refusal.
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

mod common;

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

fn payload(cwd: &Path, name: &str) -> String {
    format!("{{\"cwd\": \"{}\", \"name\": \"{}\"}}", cwd.display(), name)
}

/// Run `doctrine <args>` with `payload` on STDIN. Process cwd = `cwd` (mirrors the
/// hook firing with the orchestrator's cwd). CARGO_TARGET_DIR/DOCTRINE_WORKER cleared
/// so provisioning into the fork is deterministic and this Orchestrator-classed verb
/// runs as the orchestrator process it models, not a worker.
fn run(cwd: &Path, payload: &str, args: &[&str]) -> Output {
    let mut child = common::doctrine_cmd(cwd)
        .args(args)
        .env_remove("CARGO_TARGET_DIR")
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

/// The branch a worktree's HEAD points at, or `None` for a detached HEAD.
///
/// SL-254 PHASE-05: replaced the `worker_marker` helper. With the marker gone, this
/// was what distinguished the Fork arm (on `dispatch/<name>`) from the benign
/// Passthrough arm (detached). PHASE-06 deleted the Fork arm, so it now pins the
/// stronger claim: this verb NEVER claims a branch.
fn head_branch(dir: &Path) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["symbolic-ref", "-q", "--short", "HEAD"])
        .output()
        .expect("spawn git");
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
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

// --- VT-3 + VT-7: benign detached tree, provisioned, no branch, path-only ---

#[test]
fn passthrough_creates_detached_provisions_and_claims_no_dispatch_branch() {
    let root = tempfile::tempdir().unwrap();
    init_repo(root.path());
    let root_canon = std::fs::canonicalize(root.path()).unwrap();
    seed_sentinel(root.path());

    // ADVANCE HEAD before the spawn: a detached passthrough tracks the coord tip, and
    // pinning that explicitly is what keeps this from silently reading as "base B".
    std::fs::write(root.path().join("drift.txt"), "post-B").unwrap();
    git(root.path(), &["add", "drift.txt"]);
    git(root.path(), &["commit", "-q", "-m", "advance HEAD"]);

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
    // SL-254 PHASE-05: this was the `!worker_marker(&dir).exists()` negative — the
    // proof that the benign arm is not mistaken for a worker. The marker retired
    // (`DEC-207`), so the surviving negative is that the arm claims NO branch at
    // all: it cannot be confused with a `dispatch/<name>` fork. PHASE-06 deleted the
    // Fork arm outright, which makes this the whole verb's property, not one arm's.
    assert_eq!(
        head_branch(&dir),
        None,
        "the created worktree is in detached HEAD state — it claims no dispatch branch"
    );
    assert!(
        !git(root.path(), &["branch", "--list", "dispatch/*"]).contains("dispatch/"),
        "create-fork minted no `dispatch/<name>` branch — it cannot produce a worker fork"
    );
    // Provisioned via the SAME copier (I2).
    assert_eq!(
        std::fs::read_to_string(dir.join("sentinel.txt")).unwrap(),
        "from coord tree",
        "passthrough provisioned from the coord tree"
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

// --- VT-5: name collision on a live `.worktrees/<name>` dir ---

#[test]
fn name_collision_refuses() {
    let root = tempfile::tempdir().unwrap();
    init_repo(root.path());
    let root_canon = std::fs::canonicalize(root.path()).unwrap();

    // First spawn lands `.worktrees/agent-dup`.
    assert!(
        run(&root_canon, &payload(&root_canon, "agent-dup"), CREATE)
            .status
            .success(),
        "first spawn succeeds"
    );
    // SL-254 PHASE-06: the Fork-arm half of this case (its distinct `fork-refused`
    // token, raised by `fork_core`'s branch claim) died with the arm — `fork_core`
    // itself and that refusal are still pinned, in `create.rs`'s unit suite, through
    // the worker-fork path that survives.
    //
    // Passthrough collision on the live dir ⇒ `name-collision`.
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
