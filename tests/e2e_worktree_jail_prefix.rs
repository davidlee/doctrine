// SPDX-License-Identifier: GPL-3.0-only
//! SL-185 PHASE-02 — `doctrine worktree jail-prefix` end-to-end over the built
//! binary (the subprocess/pi spawn arm's confinement-prefix emitter).
//!
//! The Linux (bwrap) arm is exercised in full here (design §4, D2 cfg-rot
//! mitigation): the whole command shell — argparse, inline-policy build, XR-1
//! canonicalize-before-validate, fail-closed, and the `--out` NUL-delimited argv
//! contract — is real-tested. The macOS (Seatbelt) arm is cfg-gated and covered by
//! VH + `cargo check --target`, not here.
//!
//! - VT-1: `jail-prefix --dir <wt> --out <f>` writes a NUL-delimited bwrap prefix
//!   ending in `--`; split-on-`\0` round-trips to the wrap tokens.
//! - VT-2: a dangerous inline `--extra-rw` (`.git` / a worktree ancestor) is
//!   REJECTED (XR-1 canonicalize-before-validate) ⇒ nonzero, no `--out`.
//! - VT-3: fail-closed — bwrap absent ⇒ nonzero + NO `--out`; present ⇒ success.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

mod common;

fn bin() -> PathBuf {
    common::doctrine_bin()
}

/// The `bwrap` token as bytes — the wrap prefix's first argv element.
const BWRAP: &[u8] = b"bwrap";
/// The bwrap command separator — the wrap prefix's LAST element (`FLAG_ARG_SEP`).
const ARG_SEP: &[u8] = b"--";

/// Is `bwrap` resolvable on THIS host's PATH? (Mirrors the command's own probe.)
/// Guards the present-path assertions so the suite is host-capability-tolerant.
fn bwrap_on_path() -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join("bwrap").is_file())
}

/// Run `doctrine worktree jail-prefix …`; `path_env` overrides the child's `PATH`
/// (None ⇒ inherit) so the bwrap-absence case is simulated portably.
///
/// The child's cwd is set to `out`'s parent (a clean tempdir with no doctrine root
/// above it). `jail-prefix` is Orchestrator-classed — refused under worker-mode —
/// so it must run from a non-worker context, which is exactly its real usage (the
/// orchestrator's spawn script computes the prefix; no worker does). Running the
/// suite from this dispatch worktree (which bears the worker marker) would
/// otherwise be refused before the logic under test even runs.
fn jail_prefix(
    dir: &Path,
    out: &Path,
    extra_rw: &[&Path],
    network: bool,
    path_env: Option<&Path>,
) -> Output {
    let mut args: Vec<String> = vec![
        "worktree".into(),
        "jail-prefix".into(),
        "--dir".into(),
        dir.to_str().unwrap().into(),
        "--out".into(),
        out.to_str().unwrap().into(),
    ];
    if network {
        args.push("--network".into());
    }
    for g in extra_rw {
        args.push("--extra-rw".into());
        args.push(g.to_str().unwrap().into());
    }
    let mut cmd = Command::new(bin());
    cmd.args(&args);
    // Non-worker cwd (see fn doc): out's parent is a clean tempdir with no doctrine
    // root above it, so the orchestrator-class guard resolves non-worker ⇒ allowed.
    if let Some(parent) = out.parent() {
        cmd.current_dir(parent);
    }
    if let Some(p) = path_env {
        cmd.env("PATH", p);
    }
    cmd.output().expect("spawn doctrine")
}

/// Split a NUL-delimited `--out` file into its argv tokens (bytes, non-UTF-8 safe).
fn read_tokens(out: &Path) -> Vec<Vec<u8>> {
    let bytes = std::fs::read(out).expect("read --out");
    bytes.split(|b| *b == 0).map(<[u8]>::to_vec).collect()
}

// ── VT-1: NUL-delimited bwrap prefix ending in `--`, round-trips to wrap tokens ──

#[test]
fn linux_emits_nul_delimited_bwrap_prefix_ending_in_arg_sep() {
    if !bwrap_on_path() {
        return; // present-path assertion; the absent case is VT-3.
    }
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    std::fs::create_dir_all(&wt).unwrap();
    let wt_real = std::fs::canonicalize(&wt).unwrap();
    let out = tmp.path().join("jail.argv");

    let res = jail_prefix(&wt, &out, &[], false, None);
    assert!(
        res.status.success(),
        "bwrap present ⇒ jail-prefix succeeds; stderr: {}",
        String::from_utf8_lossy(&res.stderr)
    );
    assert!(out.exists(), "--out written on success");

    let tokens = read_tokens(&out);
    assert!(tokens.len() > 3, "a real bwrap prefix has many tokens: {tokens:?}");
    // No interior empty token — the NUL join carries no stray delimiter (EX-4).
    assert!(
        tokens.iter().all(|t| !t.is_empty()),
        "split-on-\\0 round-trips to non-empty tokens: {tokens:?}"
    );
    assert_eq!(tokens.first().unwrap().as_slice(), BWRAP, "prefix starts with bwrap");
    assert_eq!(
        tokens.last().unwrap().as_slice(),
        ARG_SEP,
        "prefix terminates in the `--` separator (EX-1)"
    );
    // The worktree is bound + chdir'd — its CANONICAL path rides the argv (EX-1).
    let wt_bytes = wt_real.as_os_str().as_bytes().to_vec();
    assert!(
        tokens.contains(&wt_bytes),
        "canonical worktree path is present in the wrap tokens: {}",
        wt_real.display()
    );
    assert!(
        tokens.iter().any(|t| t.as_slice() == b"--chdir"),
        "wrap chdirs into the worktree"
    );
    // network defaulted deny ⇒ the leaf appended --unshare-net.
    assert!(
        tokens.iter().any(|t| t.as_slice() == b"--unshare-net"),
        "default policy denies network ⇒ --unshare-net present"
    );
}

// ── VT-2: dangerous inline --extra-rw is rejected (XR-1), no --out ───────────────

#[test]
fn dangerous_extra_rw_dotgit_is_rejected_no_out() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    let dotgit = wt.join(".git");
    std::fs::create_dir_all(&dotgit).unwrap();
    let out = tmp.path().join("jail.argv");

    // A grant touching `.git` under the worktree — canonicalizes fine, then
    // validate_policy rejects it (TouchesGit). Fail-closed, no --out.
    let res = jail_prefix(&wt, &out, &[&dotgit], false, None);
    assert!(
        !res.status.success(),
        "a .git extra-rw grant must be rejected; stdout: {}",
        String::from_utf8_lossy(&res.stdout)
    );
    assert!(!out.exists(), "no --out file on a rejected (fail-closed) run");
}

#[test]
fn dangerous_extra_rw_ancestor_is_rejected_no_out() {
    let tmp = tempfile::tempdir().unwrap();
    let root = std::fs::canonicalize(tmp.path()).unwrap();
    let wt = root.join("wt");
    std::fs::create_dir_all(&wt).unwrap();
    let out = root.join("jail.argv");

    // `--extra-rw <wt>/..` realpaths to the worktree's PARENT — an ancestor of the
    // confined worktree. XR-1: canonicalize BEFORE the lexical ancestor test, which
    // then rejects it (a `..` grant would otherwise pass the lexical check).
    let escaping = wt.join("..");
    let res = jail_prefix(&wt, &out, &[&escaping], false, None);
    assert!(
        !res.status.success(),
        "an ancestor-escaping `..` grant must be rejected; stdout: {}",
        String::from_utf8_lossy(&res.stdout)
    );
    assert!(!out.exists(), "no --out file on a rejected (fail-closed) run");
}

#[test]
fn nonexistent_extra_rw_fails_closed_no_out() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    std::fs::create_dir_all(&wt).unwrap();
    let out = tmp.path().join("jail.argv");

    // XR-1 existence obligation: a grant that does not resolve fails closed rather
    // than binding a phantom path.
    let ghost = tmp.path().join("does-not-exist");
    let res = jail_prefix(&wt, &out, &[&ghost], false, None);
    assert!(!res.status.success(), "a non-existent grant fails closed");
    assert!(!out.exists(), "no --out file on a fail-closed run");
}

// ── VT-3: fail-closed on bwrap absence; success + --out on presence ──────────────

#[test]
fn bwrap_absent_fails_closed_no_out() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    std::fs::create_dir_all(&wt).unwrap();
    let out = tmp.path().join("jail.argv");
    // A PATH with no `bwrap` on it ⇒ the Linux arm's presence probe fails ⇒
    // fail-closed. Portable simulation of an absent capability (EX-3).
    let empty_bin = tmp.path().join("empty-bin");
    std::fs::create_dir_all(&empty_bin).unwrap();

    let res = jail_prefix(&wt, &out, &[], false, Some(&empty_bin));
    assert!(
        !res.status.success(),
        "bwrap absent ⇒ nonzero exit; stdout: {}",
        String::from_utf8_lossy(&res.stdout)
    );
    assert!(res.stdout.is_empty(), "fail-closed emits nothing on stdout");
    assert!(!out.exists(), "bwrap absent ⇒ NO --out file (EX-3)");
}

#[test]
fn bwrap_present_writes_out() {
    if !bwrap_on_path() {
        return; // host has no bwrap; the absent case above is authoritative.
    }
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    std::fs::create_dir_all(&wt).unwrap();
    let out = tmp.path().join("jail.argv");

    let res = jail_prefix(&wt, &out, &[], false, None);
    assert!(
        res.status.success(),
        "bwrap present ⇒ success; stderr: {}",
        String::from_utf8_lossy(&res.stderr)
    );
    assert!(out.exists(), "--out present ⇒ the command succeeded");
}
