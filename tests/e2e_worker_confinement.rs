// SPDX-License-Identifier: GPL-3.0-only
//! SL-254 PHASE-02, VT-2 / VT-3 — the confinement floor is real, not a fiction the
//! docs assert. DEC-212 (the never-vacuous-pass rule this slice exists to enforce)
//! says a verification test that can silently degrade into "did nothing, still
//! green" is worse than no test at all: it launders a fail-open regression as a
//! passing suite. So this file does not assert against a description of bwrap's
//! behaviour — it extracts the SHIPPED `scripts/spawn-confined.sh` Linux
//! `PREFIX=( bwrap … )` array tokens AT TEST TIME (same technique as
//! `src/worktree/jail.rs`'s `spawn_core_tokens` / `bwrap_core_argv_matches_spawn_core_flags`,
//! read alongside this file rather than duplicated by hand) and actually execs
//! `bwrap` with them. If a future edit narrows or drops the `--ro-bind / /` grant,
//! this test fails on the NEXT run — it does not need anyone to remember to update
//! a hand-copied flag list.
//!
//! What must be true, empirically, not by inference from the script's prose:
//! - VT-2: the worker CAN write inside its own worktree ($D, `--bind $D $D`).
//! - VT-3: the worker CANNOT write outside it — refused by the KERNEL's read-only
//!   bind (`--ro-bind / /`), not by an agent-level or hook-level check. There is no
//!   hook and no agent in this test process at all; a refusal here can only be the
//!   mount namespace.
//! - `DOCTRINE_WORKER=1` rides the confined environment (`--setenv DOCTRINE_WORKER 1`),
//!   the signal `doctrine`'s worker-mode gates key off.
//! - On a host with no `bwrap` on PATH, the live proof SKIPS with a named reason and
//!   returns early — it must NEVER fall through and report a vacuous pass (the
//!   exact fail-open shape DEC-212 exists to delete).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]
#![allow(
    clippy::print_stderr,
    reason = "the standard host-capability skip idiom (see e2e_worker_gate_skip.rs, e2e_worktree_jail_prefix.rs) reports its reason on stderr before returning early"
)]

use std::process::Command;

mod common;

/// Is `bwrap` resolvable on THIS host's PATH? Mirrors `spawn-confined.sh`'s own
/// `command -v bwrap` probe (D7) — the same question, asked the same way.
fn bwrap_on_path() -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join("bwrap").is_file())
}

/// Read the shipped spawn script, at test time — never a copy pasted into this
/// file. That is the whole point (see module doc): a script edit is caught here
/// automatically, rather than silently outrunning a hand-maintained twin.
fn spawn_script_text() -> String {
    let path = common::repo_root().join("scripts/spawn-confined.sh");
    std::fs::read_to_string(&path).expect("read scripts/spawn-confined.sh")
}

/// Extract the Linux `PREFIX=( bwrap … )` array's tokens, in order, with quotes
/// stripped — everything BETWEEN the `bwrap` launcher token and the array-closing
/// `)`, i.e. the full flag set actually shipped (including the harness config bind
/// and the `--setenv DOCTRINE_WORKER 1` leg — this test needs both, unlike
/// `jail.rs`'s `spawn_core_tokens`, which deliberately filters them out to compare
/// only the OS-independent core against `bwrap_core_argv`).
///
/// Parsing approach mirrors `src/worktree/jail.rs`'s `spawn_core_tokens` exactly:
/// drop comment lines (a `#` capability-probe comment block mentions "bwrap" too),
/// splice `\`-continuations, split on whitespace, then anchor on the `PREFIX=(`
/// array-literal token rather than the bare `bwrap` word — SL-254's `command -v
/// bwrap` capability probe (D7) put a second, EARLIER `bwrap` token in the script,
/// so anchoring on the word itself would silently start comparing the wrong span.
fn extract_prefix_array_tokens() -> Vec<String> {
    let raw = spawn_script_text();
    let code = raw
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
        .replace("\\\n", " ");
    let toks: Vec<String> = code.split_whitespace().map(str::to_string).collect();
    let open = toks
        .iter()
        .position(|t| t == "PREFIX=(")
        .expect("inline PREFIX array literal token");
    assert_eq!(
        toks.get(open + 1).map(String::as_str),
        Some("bwrap"),
        "the inline PREFIX array must open with the bwrap launcher token"
    );
    toks.get(open + 2..)
        .expect("tokens after the bwrap launcher token")
        .iter()
        .take_while(|t| t.as_str() != ")")
        .map(|t| t.trim_matches('"').to_string())
        .collect()
}

/// Substitute the two shell variables the script leaves in the extracted tokens —
/// `$CFG_DIR` (the harness config dir bind) and `$D` (the worker's worktree bind) —
/// with real temp-dir paths this test controls. Exact-token match only: the script
/// never interpolates these mid-token (each rides as its own argv element), so a
/// whole-token compare is faithful to how bwrap itself would see them.
fn substitute(tokens: &[String], cfg_dir: &str, d: &str) -> Vec<String> {
    tokens
        .iter()
        .map(|t| match t.as_str() {
            "$CFG_DIR" => cfg_dir.to_string(),
            "$D" => d.to_string(),
            other => other.to_string(),
        })
        .collect()
}

/// VT-2 + VT-3, live: run the SHIPPED bwrap tokens for real and prove, by kernel
/// refusal alone, that a confined worker is boxed to its own worktree.
#[test]
fn bwrap_confinement_boxes_writes_to_worktree_only() {
    // DEC-212: never a vacuous pass. No bwrap on PATH ⇒ named skip, early return —
    // the live assertions below never run, and none of them can be misread as
    // having exercised anything.
    if !bwrap_on_path() {
        eprintln!("skipping bwrap_confinement_boxes_writes_to_worktree_only: `bwrap` not on PATH");
        return;
    }

    let tokens = extract_prefix_array_tokens();
    assert!(
        tokens.contains(&"--ro-bind".to_string()),
        "shipped array must carry the whole-fs read-only grant: {tokens:?}"
    );
    assert!(
        tokens.iter().any(|t| t == "DOCTRINE_WORKER"),
        "shipped array must set DOCTRINE_WORKER: {tokens:?}"
    );

    // $D — the worker's own worktree, granted rw via `--bind $D $D`.
    let worktree = tempfile::tempdir().expect("worktree tempdir");
    // $CFG_DIR — the harness config dir, granted rw the same way; content is
    // irrelevant here, only that the bind resolves and the jail accepts it.
    let cfg_dir = tempfile::tempdir().expect("cfg_dir tempdir");
    // A SEPARATE temp dir the confined process never gets an explicit bind for —
    // under `--ro-bind / /` this must be read-only inside the jail, same as
    // everywhere else outside $D and $CFG_DIR.
    let outside = tempfile::tempdir().expect("outside tempdir");

    let argv = substitute(
        &tokens,
        cfg_dir.path().to_str().expect("cfg_dir utf8"),
        worktree.path().to_str().expect("worktree utf8"),
    );

    // The probe: print DOCTRINE_WORKER, then attempt three writes and report each
    // as a machine-checkable token rather than free text — `if`'s own exit status
    // decides success/failure, so a shell redirection-ordering quirk in stderr
    // capture (observed manually: dash reports a failed `>` target to the REAL
    // stderr before a trailing `2>` redirection takes effect) can't muddy the
    // result. `outside` is a bind-less temp dir; `repo_root` is this project's own
    // working tree — both must refuse, both by the SAME kernel mechanism as the
    // rest of "/" under `--ro-bind / /`.
    let repo_root = common::repo_root();
    let root_probe_path = repo_root.join("SL254-VT3-confinement-probe-do-not-commit");
    let probe = format!(
        "echo worker_env=${{DOCTRINE_WORKER:-MISSING}}; \
         if echo probe >'{inside}/inside.txt' 2>/dev/null; then echo inside=ok; else echo inside=refused; fi; \
         if echo probe >'{outside}/outside.txt' 2>/dev/null; then echo outside=ok; else echo outside=refused; fi; \
         if echo probe >'{root}' 2>/dev/null; then echo root=ok; else echo root=refused; fi",
        inside = worktree.path().display(),
        outside = outside.path().display(),
        root = root_probe_path.display(),
    );

    let output = Command::new("bwrap")
        .args(&argv)
        .arg("--")
        .arg("/bin/sh")
        .arg("-c")
        .arg(&probe)
        .output()
        .expect("spawn bwrap");

    // Best-effort cleanup: a passing test proves this file was never created, but
    // don't leave a stray artefact behind if a future regression ever does. Not
    // `let _ =` (must-use-ignoring is denied): only act, and only report, when
    // there is actually something to clean up.
    if root_probe_path.exists() {
        std::fs::remove_file(&root_probe_path).expect("cleanup stray root probe file");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "confined probe exited non-zero; stdout={stdout}\nstderr={stderr}"
    );

    assert!(
        stdout.contains("worker_env=1"),
        "DOCTRINE_WORKER=1 must be visible inside the confined process; stdout={stdout}"
    );
    assert!(
        stdout.contains("inside=ok"),
        "a write inside $D must succeed (--bind $D $D); stdout={stdout}"
    );
    assert!(
        stdout.contains("outside=refused"),
        "a write to an unbound temp dir must be refused by --ro-bind / /, not merely discouraged; stdout={stdout}"
    );
    assert!(
        stdout.contains("root=refused"),
        "a write into this project's own tree must be refused by the same kernel bind; stdout={stdout}"
    );
    assert!(
        !root_probe_path.exists(),
        "the refused root write must have left no file behind — a kernel refusal, not a hook that ran too late"
    );
}
