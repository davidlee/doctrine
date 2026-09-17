// SPDX-License-Identifier: GPL-3.0-only
//! SL-245 PHASE-05 — the `-X` render guard, proved end to end (VT-3, VT-4).
//!
//! The claim under test is an ORDER, not merely an exit status: `run_graph`
//! calls `terminal_image::prepare` BEFORE `root::find`, so a `-X` that cannot be
//! honoured is refused as such rather than masked by "no project root".
//!
//! That makes the cwd load-bearing. Both tests run from a base whose ancestry to
//! `/` carries NO project marker (`common::marker_free_base`), so the *other*
//! reason to exit non-zero — no root — is live and competing. A plain
//! `tempfile::tempdir()` would sit under the system tempdir, which may itself sit
//! under a marker, resolve an incidental root, and silently retire the contest.
//!
//! And so each test asserts BOTH ways: positively that the render message is
//! there, and negatively that the root error is NOT. Without the negative leg,
//! a `prepare` moved back after `root::find` would still exit non-zero and the
//! test would still pass — which is the whole claim, unconvicted.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::process::Output;

mod common;

/// `root::find`'s own refusal. Its ABSENCE is what proves the guard ran first.
const NO_ROOT_ERROR: &str = "No project root found";

/// `doctrine graph <args…>` from a cwd outside any doctrine project, with stdout
/// on a pipe (so the live `open_render_terminal` sees a non-terminal).
///
/// Spawned through `common::doctrine_cmd`, which resolves the binary, binds the
/// cwd and strips the `DOCTRINE_WORKER` env leg. The path is NOT resolved here:
/// `spawn_cwd_convention.rs` makes `doctrine_cmd` the only way a test reaches the
/// binary, and this file was reaching past it (RV-369 F-5) — undetected, because
/// the call sat inside an `assert!` and that scan walks a `syn` AST, which does
/// not descend into macro tokens.
fn graph_outside_a_project(args: &[&str]) -> Output {
    let base = common::marker_free_base();
    let cwd = tempfile::tempdir_in(base).expect("marker-free tempdir");
    common::doctrine_cmd(cwd.path())
        .arg("graph")
        .args(args)
        .output()
        .expect("spawn doctrine")
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

/// VT-3 — `--format json` with `-X` is refused on the FORMAT, before the root
/// lookup and before the terminal is ever touched.
#[test]
fn a_non_dot_format_is_refused_before_the_root_lookup() {
    let out = graph_outside_a_project(&["--format", "json", "-X"]);
    let message = stderr(&out);

    assert!(!out.status.success(), "-X with --format json must refuse");
    assert!(
        out.stdout.is_empty(),
        "a refusal must put nothing on stdout, got {:?}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(message.contains("--format dot"), "{message}");
    assert!(message.contains("json"), "{message}");
    assert!(message.contains("drop -X"), "{message}");
    assert!(
        !message.contains(NO_ROOT_ERROR),
        "the root lookup ran first — the guard's order is broken: {message}"
    );
}

/// VT-4 — `-X` at the default `dot` format, off a terminal, is refused by the
/// live `open_render_terminal` stdout check. Still before the root lookup.
#[test]
fn rendering_off_a_terminal_is_refused_before_the_root_lookup() {
    let out = graph_outside_a_project(&["-X"]);
    let message = stderr(&out);

    assert!(!out.status.success(), "-X off a terminal must refuse");
    assert!(
        out.stdout.is_empty(),
        "a refusal must put nothing on stdout, got {:?}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        message.contains("needs stdout to be a terminal"),
        "{message}"
    );
    assert!(message.contains("drop -X"), "{message}");
    assert!(
        !message.contains(NO_ROOT_ERROR),
        "the root lookup ran first — the guard's order is broken: {message}"
    );
}

/// The regression bar (EX-4) at the e2e seam: WITHOUT `-X` the very same rootless
/// invocation still fails on the root, with no render message anywhere near it.
/// Without this, both tests above would pass against a build that refused every
/// `graph` invocation.
#[test]
fn without_the_flag_the_root_lookup_is_still_what_refuses() {
    let out = graph_outside_a_project(&[]);
    let message = stderr(&out);

    assert!(!out.status.success(), "no root, no graph");
    assert!(message.contains(NO_ROOT_ERROR), "{message}");
    assert!(
        !message.contains("--render"),
        "the guard must not run without -X: {message}"
    );
}

// ── The endpoint check, against a real controlling terminal ────────────────
//
// RV-369 F-1. Everything above runs with stdout on a PIPE, so it exercises the
// REFUSAL paths only — and every unit test of `tty::endpoint` injects its probe
// values. Between them they left the one thing no pure test can see uncovered:
// what the impure shell actually FEEDS the pure decision. The shipped shell fed
// it two `st_rdev` values that can never be equal, so `-X` refused on every
// terminal in existence, and the whole suite stayed green.
//
// `script` (util-linux) is the cheapest real pty available: it opens one, makes
// it the controlling terminal of a fresh session, and runs the command there.

/// Both messages that mean "the endpoint check refused". Their ABSENCE is the
/// claim; `rendering_off_a_terminal_is_refused_before_the_root_lookup` above is
/// the positive control that they are reachable at all.
const NOT_A_TERMINAL: &str = "needs stdout to be a terminal";
const NOT_THE_CONTROLLING_TERMINAL: &str = "needs stdout to be the terminal you are running in";

/// `doctrine graph <args…>` with stdout on a REAL pty.
///
/// The binary path, the bound cwd and the stripped `DOCTRINE_WORKER` leg all come
/// from `common::doctrine_cmd` — this only wraps its program in a pty, because a
/// `Command` cannot allocate one. `script` merges the child's stderr into the pty,
/// so a refusal arrives on `script`'s stdout.
fn graph_on_a_pty(args: &[&str]) -> Output {
    let base = common::marker_free_base();
    let cwd = tempfile::tempdir_in(base).expect("marker-free tempdir");
    let seam = common::doctrine_cmd(cwd.path());
    let program = seam.get_program().to_str().expect("utf8 binary path");
    assert!(
        !program.contains(char::is_whitespace) && !program.contains('\''),
        "the pty wrapper passes the binary path through a shell command string; \
         a path needing quoting would silently run something else: {program}"
    );
    let command = std::iter::once(program)
        .chain(["graph"])
        .chain(args.iter().copied())
        .collect::<Vec<_>>()
        .join(" ");

    let mut pty = std::process::Command::new("script");
    pty.current_dir(cwd.path())
        .env_remove("DOCTRINE_WORKER")
        .args(["-qec", &command, "/dev/null"]);
    pty.output()
        .expect("spawn script (util-linux) for a real pty")
}

/// F-1 — with stdout on a real controlling terminal, `-X` must get PAST the
/// endpoint check. What it refuses on afterwards is the environment's business
/// (`script`'s pty reports no pixel size, and CI has no kitty); that the endpoint
/// check itself resolved is the claim, and it is the one the shipped code failed.
#[test]
fn a_real_controlling_terminal_passes_the_endpoint_check() {
    let out = graph_on_a_pty(&["-X"]);
    let seen = String::from_utf8_lossy(&out.stdout).to_string();

    assert!(
        !seen.contains(NOT_THE_CONTROLLING_TERMINAL),
        "stdout IS the controlling terminal, so the topology check must resolve. \
         Comparing `st_rdev` of stdout with `st_rdev` of a `/dev/tty` fd is what \
         breaks this: the latter reports the `/dev/tty` devnode itself, never the \
         pts it redirects to, so the two can never be equal: {seen}"
    );
    assert!(
        !seen.contains(NOT_A_TERMINAL),
        "stdout is a pty — the isatty leg must pass too: {seen}"
    );
    assert!(
        !seen.contains(NO_ROOT_ERROR),
        "the guard still runs before the root lookup on the pty path: {seen}"
    );
}
