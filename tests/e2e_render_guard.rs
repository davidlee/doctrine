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
/// The binary is `common::doctrine_bin()`, resolved at runtime from the test exe
/// and spawned through `common::doctrine_cmd`, which binds the cwd and strips the
/// `DOCTRINE_WORKER` env leg.
fn graph_outside_a_project(args: &[&str]) -> Output {
    assert!(
        common::doctrine_bin().is_file(),
        "the built binary doctrine_cmd spawns is missing"
    );
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
