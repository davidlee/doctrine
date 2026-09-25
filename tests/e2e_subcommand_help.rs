// SPDX-License-Identifier: GPL-3.0-only
//! Black-box e2e for the SL-208 PHASE-02 subcommand-help intercept. These invoke
//! the built binary — exercising the real parse-error → intercept → exit-status
//! path in `main.rs`, not just the pure renderer. Six tests, one per plan VT. The
//! binary is resolved at runtime via `common::doctrine_bin()` (a `current_exe`
//! sibling lookup), NOT a compile-time baked path — CHR-014 / SL-162 ban `env!`
//! path-baking, which breaks under the shared jail target-dir.

mod common;

fn run(args: &[&str]) -> (bool, i32, String) {
    let out = common::doctrine_cmd(&common::repo_root())
        .args(args)
        .output()
        .expect("spawn doctrine");
    let mut merged = String::from_utf8_lossy(&out.stdout).into_owned();
    merged.push_str(&String::from_utf8_lossy(&out.stderr));
    (
        out.status.success(),
        out.status.code().unwrap_or(-1),
        merged,
    )
}

// VT-1: `worktree --help` → exit 0; cozy-table `│` separators AND a subcommand
// name (`provision`) present.
#[test]
fn cli_subcommand_help_worktree() {
    let (ok, _code, out) = run(&["worktree", "--help"]);
    assert!(ok, "worktree --help must exit 0");
    assert!(out.contains('│'), "cozy-table separator present");
    assert!(out.contains("provision"), "subcommand name present");
}

// VT-2: `--color never worktree --help` → exit 0; NO ANSI escape byte anywhere.
// The `--color` value `never` must not be mistaken for a subcommand name (the
// clap-tree walk, not a `!starts_with('-')` filter).
#[test]
fn cli_subcommand_help_plain_mode() {
    let (ok, _code, out) = run(&["--color", "never", "worktree", "--help"]);
    assert!(ok, "--color never worktree --help must exit 0");
    assert!(!out.contains('\u{1b}'), "plain mode emits no ANSI escapes");
    assert!(out.contains("provision"), "still renders the worktree help");
}

// VT-3: `worktree` with no verb → nonzero exit. The missing-subcommand error
// contract is preserved (surfaced as MissingSubcommand / the
// DisplayHelpOnMissingArgumentOrSubcommand error branch) — a verb-less parent is
// an error, never silently rendered as help.
#[test]
fn cli_missing_subcommand_errors() {
    let (ok, code, _out) = run(&["worktree"]);
    assert!(!ok, "worktree with no verb must be an error (nonzero exit)");
    assert!(
        code != 0,
        "nonzero exit code for the missing-subcommand error"
    );
}

// VT-4: `worktree help provision` renders provision's help → exit 0. The `help`
// token is stripped and the walk resolves worktree → provision.
#[test]
fn cli_help_help_path() {
    let (ok, _code, out) = run(&["worktree", "help", "provision"]);
    assert!(ok, "worktree help provision must exit 0");
    assert!(out.contains("provision"), "renders provision's help");
    // The `--help` spelling of the same path resolves identically.
    let (ok2, _c2, out2) = run(&["worktree", "provision", "--help"]);
    assert!(ok2, "worktree provision --help must exit 0");
    assert!(out2.contains("provision"), "renders provision's help");
}

// VT-5: top-level `--help` → exit 0; the family headings (`change`,
// `governance`) are unchanged (top-level rendering is byte-stable, EX-5).
#[test]
fn cli_top_level_help_unchanged() {
    let (ok, _code, out) = run(&["--help"]);
    assert!(ok, "--help must exit 0");
    assert!(out.contains("change"), "top-level family heading `change`");
    assert!(
        out.contains("governance"),
        "top-level family heading `governance`"
    );
}

// VT-6: `memory sync --help` (depth-2) → exit 0; cozy-table `│` present.
#[test]
fn cli_subcommand_help_depth2() {
    let (ok, _code, out) = run(&["memory", "sync", "--help"]);
    assert!(ok, "memory sync --help must exit 0");
    assert!(out.contains('│'), "cozy-table separator present at depth 2");
}

// SL-233 PHASE-04 EX-5, RE-PREMISED by SL-246 `EX-8` (DEC-261) — not dropped.
//
// The original claim was that `design show`'s DEFAULT is a strict subset surface
// which `--full` widens. `DEC-261` voids that premise at the root: the default is
// no longer an envelope projection at all, it is the design document, and a
// document has no projection to widen (`--full` is now REFUSED there). What
// survives, and is what the help text was really guarding, is that `--full` is the
// widening **of the envelope rendering** — so the help must teach
// `--format prompt --full` rather than a bare `--full`.
//
// Two legs, because either alone is evadable. (a) UNCHANGED by `DEC-261`: `design`
// exposes no `inspect`-style verb, so nothing design-specific can invert the
// established metadata-only meaning of `inspect` by claiming the name. (b) `show`'s
// own help names `--full` as THE widening AND ties it to the envelope rendering, so
// a reader learns the pair rather than a flag that would now be refused on its own.
#[test]
fn design_show_help_names_full_as_the_envelope_widening() {
    let (ok, _code, family) = run(&["design", "--help"]);
    assert!(ok, "design --help must exit 0");
    assert!(family.contains("show"), "the family lists `show`: {family}");
    // SL-266 PHASE-03 `VA-1`: the tree verb and the format value both surface.
    assert!(family.contains("tree"), "the family lists `tree`: {family}");
    assert!(
        !family.contains("inspect"),
        "design exposes no `inspect`-style verb, so `show` cannot be the wider \
         of a pair: {family}"
    );

    let (ok, _code, show) = run(&["design", "show", "--help"]);
    assert!(ok, "design show --help must exit 0");
    assert!(
        show.contains("status, tree"),
        "`--format` lists the `tree` value: {show}"
    );
    assert!(
        show.contains("--full"),
        "`show`'s help names the widening: {show}"
    );
    assert!(
        show.to_lowercase().contains("widen"),
        "and says which direction it goes: {show}"
    );
    // The re-premise itself: `--full` is the ENVELOPE's widening, and the help says
    // whose. Without this the assertions above would still pass on help text that
    // implied a bare `--full` widens the default — the exact reading `DEC-261` made
    // false and `run_show` now refuses.
    assert!(
        show.contains("turn envelope") && show.contains("--format prompt|json|status"),
        "`--full`'s help ties the widening to the envelope renderings, not to the \
         default: {show}"
    );
    assert!(
        show.contains("prompt"),
        "so a reader learns `--format prompt --full`: {show}"
    );
}
