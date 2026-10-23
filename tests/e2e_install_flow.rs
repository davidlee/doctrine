// SPDX-License-Identifier: GPL-3.0-only
//! SL-282 PHASE-05 VT-1 — the interactive install over the built binary: the
//! overview, the one `Proceed? [y/N/d]` prompt, and the path-limited commit
//! hint (RV-421 F-10: an unrelated staged file never rides the printed
//! `git commit`).
//!
//! Every project is a fresh git repo (`common::init_repo`) unless a row says
//! otherwise. Every "writes nothing" row asserts the run reached the prompt.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Output;

mod common;

use common::{
    INSTALL_MARKER, SAFE_CODEX_HOME, doctrine_cmd, doctrine_toml, git, init_repo, tree_digest,
};

/// The running binary's version: what a complete install records.
const B: &str = env!("CARGO_PKG_VERSION");
/// One harness, one skill — small and fast.
const ARGS: &[&str] = &["install", "--agent", "claude", "--skill", "code-review"];
const PROMPT: &str = "Proceed? [y/N/d]";
/// The prompt line as a terminal shows it once answered `N` (stdin not echoed).
const ABORTED: &str = "Proceed? [y/N/d] (d = show every step) Aborted.";
const HEADER: &str = "Doctrine will set this project up for: claude";
const DETAILS: &str = "Project root:";
const NOTHING: &str = "Nothing to commit.";
const HINT: &str = "Next:";
const HARNESS_QUESTION: &str = "No agent harness found here. Set up for which?";

/// `doctrine <args…> -p <root>` from `root`, with `input` on stdin.
fn install_with_stdin(root: &Path, args: &[&str], input: &str) -> Output {
    let mut cmd = doctrine_cmd(root);
    cmd.args(args).arg("-p").arg(root);
    common::output_with_stdin(cmd, input)
}

/// Stdout of a run that must succeed.
fn stdout_ok(out: Output) -> String {
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(
        out.status.success(),
        "install failed: {stdout}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    stdout
}

/// A `--yes` install of `ARGS` plus `extra`, with extra env, from `cwd`.
fn install_yes(cwd: &Path, extra: &[&str], env: &[(&str, &OsStr)]) -> String {
    let mut cmd = doctrine_cmd(cwd);
    cmd.args(ARGS).arg("--yes").args(extra);
    for (k, v) in env {
        cmd.env(k, v);
    }
    stdout_ok(cmd.output().expect("spawn doctrine"))
}

/// The printed hint: the commands indented under the `Next:` line.
fn hint(out: &str) -> Vec<&str> {
    let mut lines = out.lines().skip_while(|l| *l != HINT);
    assert!(lines.next().is_some(), "no commit hint: {out}");
    lines.map_while(|l| l.strip_prefix("  ")).collect()
}

/// The hint's `git add` and `git commit` pathspecs (no fixture path needs
/// quoting).
fn pathspecs(hint: &[&str]) -> (BTreeSet<String>, BTreeSet<String>) {
    let words = |s: &str| s.split_whitespace().map(str::to_string).collect();
    let line = |head: &str| {
        *hint
            .iter()
            .find(|l| l.starts_with(head))
            .unwrap_or_else(|| panic!("{head}: {hint:?}"))
    };
    let add = line("git add -- ").strip_prefix("git add -- ").unwrap();
    let commit = line("git commit ")
        .rsplit_once(" -- ")
        .map_or("", |(_, p)| p);
    (words(add), words(commit))
}

/// Run the hint's commands in order as one `sh` script in `dir`, asserting
/// success. One command per line, no shell operator (RV-422 F-10).
fn execute(hint: &[&str], dir: &Path) {
    assert!(hint.iter().all(|l| !l.contains("&&")), "{hint:?}");
    let script = hint.join("\n");
    let out = std::process::Command::new("sh")
        .arg("-c")
        .arg(&script)
        .current_dir(dir)
        .output()
        .expect("spawn sh");
    assert!(
        out.status.success(),
        "{script}: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Paths `git status` reports as changed or new and not ignored.
fn git_changes(root: &Path) -> BTreeSet<String> {
    git(root, &["status", "--porcelain", "--untracked-files=all"])
        .lines()
        .map(|l| l[3..].to_string())
        .collect()
}

fn assert_stamped(root: &Path, out: &str) {
    let tracked = fs::read_to_string(doctrine_toml(root)).unwrap();
    assert!(
        tracked.contains(&format!("installed-version = \"{B}\"")),
        "{tracked}\n{out}"
    );
    assert!(root.join(INSTALL_MARKER).is_file(), "{out}");
}

fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    init_repo(dir.path());
    dir
}

#[test]
fn interactive_y_shows_overview_one_prompt_and_hint() {
    if common::under_worker_marker() {
        return;
    }
    let dir = repo();
    let root = dir.path();

    let out = stdout_ok(install_with_stdin(root, ARGS, "y\n"));

    assert!(out.contains(HEADER), "{out}");
    for bullet in [
        "  • .gitignore ",
        "  • .doctrine/ ",
        "  • CLAUDE.md ",
        "  • claude ",
        &format!("record doctrine {B} as this repo's version"),
    ] {
        assert!(out.contains(bullet), "{bullet}: {out}");
    }
    assert_eq!(out.matches("Proceed?").count(), 1, "{out}");
    assert!(out.contains(PROMPT), "{out}");
    assert!(!out.contains("[y/N/a]"), "no per-step prompt: {out}");
    assert!(!out.contains(DETAILS), "details not shown: {out}");
    assert!(!root.join(".doctrine/hymns").exists(), "{out}");
    assert_stamped(root, &out);

    // The real assertion: the hint lists exactly the tracked writes.
    let (add, commit) = pathspecs(&hint(&out));
    assert_eq!(add, commit, "the commit is limited to what it adds");
    assert_eq!(add, git_changes(root), "{out}");
    for path in ["CLAUDE.md", ".gitignore", ".doctrine/doctrine.toml"] {
        assert!(add.contains(path), "{path}: {add:?}");
    }
    assert!(
        hint(&out)
            .iter()
            .any(|l| l.starts_with(&format!("git commit -m 'chore: install doctrine {B}' -- "))),
        "{out}"
    );
}

#[test]
fn details_then_yes_proceeds() {
    if common::under_worker_marker() {
        return;
    }
    let dir = repo();
    let root = dir.path();

    let out = stdout_ok(install_with_stdin(root, ARGS, "d\ny\n"));

    assert_eq!(out.matches(PROMPT).count(), 2, "asked again: {out}");
    let first = out.find(PROMPT).unwrap();
    let second = out.rfind(PROMPT).unwrap();
    for marker in [DETAILS, "converge framework canon"] {
        let at = out
            .find(marker)
            .unwrap_or_else(|| panic!("{marker}: {out}"));
        assert!(
            first < at && at < second,
            "{marker} between the prompts: {out}"
        );
    }
    // RV-422 F-9: a real run's details carry no --dry-run headings.
    assert!(!out.contains("--dry-run"), "{out}");
    assert_stamped(root, &out);
}

#[test]
fn no_and_details_then_no_write_nothing() {
    if common::under_worker_marker() {
        return;
    }
    for (input, prompts) in [("n\n", 1), ("d\nn\n", 2), ("d\n", 2)] {
        let dir = repo();
        let root = dir.path();
        let before = tree_digest(root);

        let out = stdout_ok(install_with_stdin(root, ARGS, input));

        assert!(out.contains(HEADER), "control: overview printed: {out}");
        assert_eq!(out.matches(PROMPT).count(), prompts, "{input:?}: {out}");
        assert_eq!(out.lines().last(), Some(ABORTED), "{input:?}: {out}");
        assert_eq!(tree_digest(root), before, "{input:?} writes nothing");
    }
}

#[test]
fn yes_flag_prints_overview_and_asks_nothing() {
    if common::under_worker_marker() {
        return;
    }
    let dir = repo();
    let root = dir.path();

    let out = install_yes(root, &[], &[]);

    assert!(out.contains(HEADER), "{out}");
    assert!(!out.contains("Proceed?"), "{out}");
    assert!(!out.contains(DETAILS), "{out}");
    assert_stamped(root, &out);
}

#[test]
fn dry_run_prints_overview_and_details_and_writes_nothing() {
    if common::under_worker_marker() {
        return;
    }
    let dir = repo();
    let root = dir.path();
    let before = tree_digest(root);

    let out = stdout_ok(install_with_stdin(
        root,
        &[ARGS, &["--dry-run"]].concat(),
        "",
    ));

    for marker in [
        HEADER,
        DETAILS,
        "Gitignore block (not written under --dry-run):",
    ] {
        assert!(out.contains(marker), "{marker}: {out}");
    }
    assert!(!out.contains("Proceed?"), "{out}");
    assert!(!out.contains(HINT), "{out}");
    assert_eq!(tree_digest(root), before, "dry-run writes nothing");
}

/// RV-421 F-10 (control): executing the printed hint commits doctrine's files
/// and leaves unrelated staged work staged.
#[test]
fn hint_commit_excludes_unrelated_staged_work() {
    if common::under_worker_marker() {
        return;
    }
    let dir = repo();
    let root = dir.path();
    fs::write(root.join("notes.txt"), "mine").unwrap();
    git(root, &["add", "notes.txt"]);

    let out = install_yes(root, &[], &[]);
    let hint = hint(&out);
    let (add, commit) = pathspecs(&hint);
    assert!(!add.contains("notes.txt") && !add.is_empty(), "{hint:?}");
    assert_eq!(add, commit, "{hint:?}");

    execute(&hint, root);
    let committed = git(root, &["show", "--name-only", "--format=", "HEAD"]);
    assert!(committed.contains("CLAUDE.md"), "control: {committed}");
    assert!(!committed.contains("notes.txt"), "{committed}");
    assert_eq!(git(root, &["diff", "--cached", "--name-only"]), "notes.txt");
}

#[test]
fn hint_from_a_nested_directory_is_prefixed_with_cd_root() {
    if common::under_worker_marker() {
        return;
    }
    let dir = repo();
    let root = fs::canonicalize(dir.path()).unwrap();
    let sub = root.join("sub");
    fs::create_dir(&sub).unwrap();

    let out = install_yes(&sub, &[], &[]);

    let hint = hint(&out);
    assert_eq!(hint[0], format!("cd {}", root.display()), "{hint:?}");
    assert!(hint[1].starts_with("git add -- "), "{hint:?}");
    execute(&hint, &sub);
    assert!(
        git_changes(&root).is_empty(),
        "every tracked write committed: {hint:?}"
    );
}

#[test]
fn second_install_says_nothing_to_commit() {
    if common::under_worker_marker() {
        return;
    }
    let dir = repo();
    let root = dir.path();
    let first = install_yes(root, &[], &[]);
    let hint = hint(&first);
    assert!(!pathspecs(&hint).0.is_empty(), "control: {hint:?}");
    execute(&hint, root);

    let out = install_yes(root, &[], &[]);

    assert!(out.lines().any(|l| l == NOTHING), "{out}");
    assert!(!out.contains(HINT), "{out}");
}

#[test]
fn removed_starter_appears_in_the_hint() {
    if common::under_worker_marker() {
        return;
    }
    let dir = repo();
    let root = dir.path();
    let hymns = root.join(".doctrine/hymns/role");
    fs::create_dir_all(&hymns).unwrap();
    let body = fs::read(common::repo_root().join("install/hymns/role/worker.md")).unwrap();
    fs::write(hymns.join("worker.md"), body).unwrap();
    fs::write(hymns.join("worker.toml"), "replaces = \"role/worker\"\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-q", "-m", "starter"]);

    let out = install_yes(root, &[], &[]);

    assert!(
        out.contains("remove 1 unedited starter copy so their slots follow framework updates"),
        "{out}"
    );
    let (add, _) = pathspecs(&hint(&out));
    for path in [
        ".doctrine/hymns/role/worker.md",
        ".doctrine/hymns/role/worker.toml",
    ] {
        assert!(add.contains(path), "{path}: {out}");
    }
    execute(&hint(&out), root);
    let committed = git(root, &["show", "--name-status", "--format=", "HEAD"]);
    assert!(
        committed.contains("D\t.doctrine/hymns/role/worker.md"),
        "{committed}"
    );
}

#[test]
fn codex_links_are_listed_without_delegation_disclosure() {
    if common::under_worker_marker() {
        return;
    }
    let dir = repo();
    let root = dir.path();
    let out = stdout_ok(
        doctrine_cmd(root)
            .args([
                "install",
                "--agent",
                "codex",
                "--skill",
                "code-review",
                "--yes",
            ])
            .output()
            .unwrap(),
    );
    assert!(!hint(&out).is_empty(), "control: hint printed: {out}");
    assert!(!out.contains("written by npx"), "{out}");
    assert!(
        !root.join("skills-lock.json").exists(),
        "doctrine writes no skills lockfile: {out}"
    );
    let block = fs::read_to_string(root.join(".gitignore")).unwrap();
    assert!(
        block
            .lines()
            .any(|line| line == "/.agents/skills/code-review"),
        "{block}"
    );
    assert_eq!(
        fs::read_link(root.join(".agents/skills/code-review")).unwrap(),
        Path::new("../../.doctrine/skills/code-review")
    );
}

#[test]
fn global_and_only_memory_print_no_hint() {
    if common::under_worker_marker() {
        return;
    }
    let home = tempfile::tempdir().unwrap();
    let dir = repo();
    let out = install_yes(
        dir.path(),
        &["--global"],
        &[("HOME", home.path().as_os_str())],
    );
    assert!(out.contains("Done."), "control: the run proceeded: {out}");
    assert!(!out.contains(HINT) && !out.contains(NOTHING), "{out}");

    let dir = repo();
    let mut cmd = doctrine_cmd(dir.path());
    cmd.args(["install", "--agent", "claude", "--only-memory", "--yes"]);
    let out = stdout_ok(cmd.output().unwrap());
    assert!(out.contains("corpus sync:"), "control: legs ran: {out}");
    assert!(!out.contains(HINT) && !out.contains(NOTHING), "{out}");
}

#[test]
fn no_git_no_hint() {
    if common::under_worker_marker() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir(root.join(".doctrine")).unwrap();
    let mut cmd = doctrine_cmd(root);
    cmd.args(ARGS).args(["--yes", "-p"]).arg(root);
    let out = stdout_ok(cmd.output().unwrap());
    assert_stamped(root, &out);
    assert!(!out.contains(HINT) && !out.contains(NOTHING), "{out}");
}

#[test]
fn no_harness_interactive_asks_once() {
    if common::under_worker_marker() {
        return;
    }
    let bare = ["install", "--skill", "code-review"];
    let dir = repo();
    let root = dir.path();
    let out = stdout_ok(install_with_stdin(root, &bare, "claude\ny\n"));
    assert_eq!(out.matches(HARNESS_QUESTION).count(), 1, "{out}");
    assert!(out.contains(HEADER), "{out}");
    assert!(root.join("CLAUDE.md").is_file(), "{out}");

    let dir = repo();
    let root = dir.path();
    let out = stdout_ok(install_with_stdin(root, &bare, "\ny\n"));
    assert_eq!(out.matches(HARNESS_QUESTION).count(), 1, "{out}");
    assert!(
        out.contains("Doctrine found no agent harness here; harness steps will be skipped"),
        "{out}"
    );
    assert!(!root.join("CLAUDE.md").exists(), "{out}");
}

#[test]
fn hint_omits_untracked_paths_the_projects_own_gitignore_ignores() {
    if common::under_worker_marker() {
        return;
    }
    let dir = repo();
    let root = dir.path();
    fs::write(root.join(".gitignore"), "/CLAUDE.md\n").unwrap();

    let out = install_yes(root, &[], &[]);

    let (add, commit) = pathspecs(&hint(&out));
    assert!(!add.contains("CLAUDE.md") && !add.is_empty(), "{out}");
    assert_eq!(add, commit, "{out}");
    execute(&hint(&out), root);
    assert!(
        root.join("CLAUDE.md").is_file(),
        "control: install wrote it"
    );
}

#[test]
fn hint_keeps_a_tracked_path_even_when_gitignore_matches_it() {
    if common::under_worker_marker() {
        return;
    }
    let dir = repo();
    let root = dir.path();
    fs::write(root.join("CLAUDE.md"), "mine\n").unwrap();
    git(root, &["add", "-f", "CLAUDE.md"]);
    fs::write(root.join(".gitignore"), "/CLAUDE.md\n").unwrap();

    let out = install_yes(root, &[], &[]);

    let (add, _) = pathspecs(&hint(&out));
    assert!(add.contains("CLAUDE.md"), "{out}");
    execute(&hint(&out), root);
}

/// SL-286 PHASE-04 (RV-427 F-3): the shared spawn seam pins `CODEX_HOME` to a
/// path that never exists, so no test child can read or write the developer's
/// real codex config. Purely structural — nothing is spawned here; a suite that
/// needs a real `CODEX_HOME` overrides this with its own `.env` call.
#[test]
fn doctrine_cmd_points_codex_home_at_a_nonexistent_path() {
    let cmd = doctrine_cmd(&common::repo_root());
    let envs: Vec<(&OsStr, Option<&OsStr>)> = cmd.get_envs().collect();
    assert!(
        envs.contains(&(OsStr::new("CODEX_HOME"), Some(OsStr::new(SAFE_CODEX_HOME)))),
        "CODEX_HOME not pinned: {envs:?}"
    );
    assert!(
        !Path::new(SAFE_CODEX_HOME).exists(),
        "the safe CODEX_HOME must never be created: {SAFE_CODEX_HOME}"
    );
}
