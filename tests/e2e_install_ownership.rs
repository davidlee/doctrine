// SPDX-License-Identifier: GPL-3.0-only
//! SL-281 PHASE-04 — ownership-driven install convergence, end-to-end over the
//! built binary (VT-2).
//!
//! Every project is a fresh tempdir; global runs override `HOME` on the child
//! `Command` only. Every absence assertion pairs with a `plant_*` helper that
//! proved the entry existed beforehand and with a report-line assertion for the
//! same path; every "unchanged" assertion compares against a value read before
//! the run.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

mod common;

/// Default arguments: one harness, one skill — small and fast.
const BASE_ARGS: &[&str] = &["--agent", "claude", "--skill", "code-review"];
const CLAUDE_WORKER: &str = ".doctrine/agents/claude/dispatch-worker.md";
const WORKER_LINK: &str = ".claude/agents/dispatch-worker.md";
const CLAUDE_WORKER_LINK_VALUE: &str = "../../.doctrine/agents/claude/dispatch-worker.md";
const RESOLVE_MARKER: &str = "{{ prompt resolve --role worker }}";

struct Project {
    tmp: tempfile::TempDir,
}

impl Project {
    fn new() -> Self {
        Self {
            tmp: tempfile::tempdir().expect("tempdir"),
        }
    }

    fn root(&self) -> &Path {
        self.tmp.path()
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.root().join(rel)
    }

    /// `doctrine install <args> -p <root>`, extra env on the child only;
    /// asserts success and returns stdout.
    fn run(&self, args: &[&str], env: &[(&str, &OsStr)]) -> String {
        let mut cmd = common::doctrine_cmd(self.root());
        cmd.arg("install").args(args).arg("-p").arg(self.root());
        for (k, v) in env {
            cmd.env(k, v);
        }
        let out = cmd.output().expect("spawn doctrine");
        assert!(
            out.status.success(),
            "install {args:?} failed: {}\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
        String::from_utf8(out.stdout).expect("utf8 stdout")
    }

    /// A live install (`--yes`) with the default one-harness arguments.
    fn install(&self) -> String {
        self.install_with(&[], &[])
    }

    fn install_with(&self, extra: &[&str], env: &[(&str, &OsStr)]) -> String {
        let args: Vec<&str> = BASE_ARGS
            .iter()
            .chain(extra)
            .chain(&["--yes"])
            .copied()
            .collect();
        self.run(&args, env)
    }

    /// Plant a file (creating parents); asserts it now exists.
    fn plant_file(&self, rel: &str, body: &str) -> PathBuf {
        let p = self.path(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, body).unwrap();
        assert!(fs::symlink_metadata(&p).is_ok(), "planted {rel} exists");
        p
    }

    /// Plant a symlink (creating parents); asserts it now exists as one.
    fn plant_symlink(&self, rel: &str, value: &str) -> PathBuf {
        let p = self.path(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(value, &p).unwrap();
        assert!(is_symlink(&p), "planted {rel} is a symlink");
        p
    }
}

fn is_symlink(p: &Path) -> bool {
    fs::symlink_metadata(p).is_ok_and(|m| m.file_type().is_symlink())
}

/// Relative path → `file:<sha256>` | `link:<value>` | `dir`, over a whole tree
/// (symlinks are recorded, never followed).
fn tree_digest(dir: &Path) -> BTreeMap<PathBuf, String> {
    fn walk(root: &Path, dir: &Path, acc: &mut BTreeMap<PathBuf, String>) {
        for e in fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            let rel = p.strip_prefix(root).unwrap().to_path_buf();
            let meta = fs::symlink_metadata(&p).unwrap();
            if meta.file_type().is_symlink() {
                let v = fs::read_link(&p).unwrap();
                acc.insert(rel, format!("link:{}", v.display()));
            } else if meta.is_dir() {
                acc.insert(rel, "dir".to_string());
                walk(root, &p, acc);
            } else {
                acc.insert(
                    rel,
                    format!("file:{}", common::sha256(&fs::read(&p).unwrap())),
                );
            }
        }
    }
    let mut acc = BTreeMap::new();
    walk(dir, dir, &mut acc);
    acc
}

/// A report line starting (after indentation) with `verb` and ending with `suffix`.
fn assert_line(out: &str, verb: &str, path_suffix: &str) {
    assert!(
        out.lines()
            .any(|l| l.trim_start().starts_with(verb) && l.trim_end().ends_with(path_suffix)),
        "no `{verb} … {path_suffix}` line in:\n{out}"
    );
}

#[test]
fn reinstall_restores_edited_canon_and_keeps_user_config() {
    if common::under_worker_marker() {
        return;
    }
    let p = Project::new();
    p.install();
    let canon = p.path(CLAUDE_WORKER);
    let original = fs::read(&canon).unwrap();
    assert!(!original.is_empty(), "control: canon materialised");
    fs::write(&canon, [original.as_slice(), b"EDITED"].concat()).unwrap();
    let toml = p.path(common::DOCTRINE_TOML);
    let mut cfg = fs::read_to_string(&toml).unwrap();
    cfg.push_str("# user edit\n");
    fs::write(&toml, &cfg).unwrap();

    p.install();

    assert_eq!(fs::read(&canon).unwrap(), original, "canon restored");
    assert!(
        fs::read_to_string(&toml).unwrap().contains("# user edit"),
        "user config survives reinstall"
    );
}

#[test]
fn stale_layout_link_is_relinked() {
    if common::under_worker_marker() {
        return;
    }
    let p = Project::new();
    let link = p.plant_symlink(
        WORKER_LINK,
        "../../.doctrine/agents/codex/dispatch-worker.md",
    );
    let flat = p.plant_file(".doctrine/agents/dispatch-worker.md", "old flat canon");
    assert!(!link.exists(), "control: the planted link dangles");

    let out = p.install();

    assert_eq!(
        fs::read_link(&link).unwrap(),
        Path::new(CLAUDE_WORKER_LINK_VALUE)
    );
    assert!(link.exists(), "the relinked path resolves");
    assert_line(&out, "relinked", "dispatch-worker.md");
    assert!(fs::symlink_metadata(&flat).is_err(), "flat canon pruned");
    assert_line(&out, "pruned", ".doctrine/agents/dispatch-worker.md");
}

#[test]
fn regular_file_is_kept_and_reported() {
    if common::under_worker_marker() {
        return;
    }
    let p = Project::new();
    let mine = p.plant_file(".claude/agents/capsule-worker.md", "mine");

    let out = p.install();

    assert!(!is_symlink(&mine), "still a regular file");
    assert_eq!(fs::read_to_string(&mine).unwrap(), "mine");
    assert_line(
        &out,
        "kept",
        "capsule-worker.md (real file shadows doctrine def — remove it to receive updates)",
    );
}

#[test]
fn orphan_canon_file_is_pruned() {
    if common::under_worker_marker() {
        return;
    }
    let p = Project::new();
    let orphan = p.plant_file(".doctrine/agents/claude/retired-def.md", "retired");

    let out = p.install();

    assert!(fs::symlink_metadata(&orphan).is_err(), "orphan removed");
    assert_line(&out, "pruned", ".doctrine/agents/claude/retired-def.md");
    assert!(p.path(CLAUDE_WORKER).is_file(), "control: convergence ran");
}

#[test]
fn orphan_skill_and_link_are_pruned() {
    if common::under_worker_marker() {
        return;
    }
    let p = Project::new();
    let canon = p.plant_file(".doctrine/skills/retired-skill/SKILL.md", "retired");
    let link = p.plant_symlink(
        ".claude/skills/retired-skill",
        "../../.doctrine/skills/retired-skill",
    );

    let out = p.install();

    assert!(
        fs::symlink_metadata(canon.parent().unwrap()).is_err(),
        "orphan skill dir removed"
    );
    assert!(fs::symlink_metadata(&link).is_err(), "orphan link removed");
    assert_line(&out, "pruned", ".doctrine/skills/retired-skill");
    assert_line(&out, "pruned", ".claude/skills/retired-skill");
    assert!(
        is_symlink(&p.path(".claude/skills/code-review")),
        "control: the wanted skill is linked"
    );
}

#[test]
fn dry_run_lists_prune_and_changes_nothing() {
    if common::under_worker_marker() {
        return;
    }
    let p = Project::new();
    p.install();
    let orphan = p.plant_file(".doctrine/agents/claude/retired-def.md", "retired");
    let before = tree_digest(p.root());
    assert!(
        before.contains_key(Path::new(".doctrine/agents/claude/retired-def.md")),
        "control: the digest sees the orphan"
    );

    let mut args = BASE_ARGS.to_vec();
    args.push("--dry-run");
    let out = p.run(&args, &[]);

    assert_line(
        &out,
        "would prune",
        ".doctrine/agents/claude/retired-def.md",
    );
    assert!(orphan.is_file(), "orphan untouched");
    assert_eq!(tree_digest(p.root()), before, "dry-run mutated nothing");
}

#[test]
fn symlinked_doctrine_blocks_prune() {
    if common::under_worker_marker() {
        return;
    }
    let real = tempfile::tempdir().expect("tempdir");
    let p = Project::new();
    std::os::unix::fs::symlink(real.path(), p.path(".doctrine")).unwrap();
    let keep = real.path().join("agents/claude/keep-me.md");
    fs::create_dir_all(keep.parent().unwrap()).unwrap();
    fs::write(&keep, "keep me").unwrap();
    assert!(keep.is_file(), "control: planted through the real dir");

    let out = p.install();

    assert_eq!(fs::read_to_string(&keep).unwrap(), "keep me");
    let worker = real.path().join("agents/claude/dispatch-worker.md");
    let body = fs::read_to_string(&worker).expect("member written through the symlink");
    assert!(!body.contains(RESOLVE_MARKER), "member is baked");
    assert!(
        out.lines()
            .any(|l| l.trim_start().starts_with("prune blocked")),
        "no `prune blocked` line in:\n{out}"
    );
}

#[test]
fn global_reports_orphan_and_keeps_it() {
    if common::under_worker_marker() {
        return;
    }
    let p = Project::new();
    let home = tempfile::tempdir().expect("tempdir");
    let orphan = home.path().join(".doctrine/agents/claude/retired-def.md");
    fs::create_dir_all(orphan.parent().unwrap()).unwrap();
    fs::write(&orphan, "retired").unwrap();
    assert!(orphan.is_file(), "control: orphan planted");

    let out = p.install_with(&["--global"], &[("HOME", home.path().as_os_str())]);

    assert_line(
        &out,
        "orphan",
        "retired-def.md (global install never removes; delete it if unwanted)",
    );
    assert_eq!(fs::read_to_string(&orphan).unwrap(), "retired");
    assert!(
        home.path().join(CLAUDE_WORKER).is_file(),
        "control: the global leg ran"
    );
}

#[test]
fn pi_without_npx_still_writes_defs() {
    if common::under_worker_marker() {
        return;
    }
    let p = Project::new();
    let empty_path = tempfile::tempdir().expect("tempdir");
    p.run(
        &["--agent", "pi", "--yes"],
        &[("PATH", empty_path.path().as_os_str())],
    );

    let link = p.path(".pi/agents/dispatch-worker.md");
    assert_eq!(
        fs::read_link(&link).unwrap(),
        Path::new("../../.doctrine/agents/pi/dispatch-worker.md")
    );
    assert!(link.exists(), "the pi def link resolves");
}
