// SPDX-License-Identifier: GPL-3.0-only
//! SL-056 PHASE-06 — `doctrine worktree fork --base <B> --branch <name> --dir
//! <path> [--worker]` end-to-end over the BUILT binary.
//!
//! * VT-1: happy path — human status on stderr, machine-clean (empty) stdout, the
//!   worktree+branch exist at `<B>`, and the status line DECLARES `--worker` (and
//!   does not, solo); the three pre-`add` refusals (dir-exists / branch-exists /
//!   B-not-a-commit) each exit non-zero and leave NO fork.
//! * VT-2: compensating cleanup — a provision failure after `git worktree add`
//!   exits non-zero AND leaves NO leftover worktree/branch (asserted GONE).
//! * VT-4: `fork` Orchestrator refusal drives the real CLI — refused in a worker
//!   PROCESS whatever tree it stands in, naming the verb and the cause.
//!
//! SL-254 PHASE-05 (`DEC-207`): `--worker` no longer stamps
//! `.doctrine/state/dispatch/worker` — nothing does; worker identity is the
//! `DOCTRINE_WORKER` env var of the process. The flag survives as the fork's
//! declaration of intent (it drives the durable `(slice, phase)` binding and the
//! status line), so VT-1's marker-present/marker-absent pair becomes a
//! declaration-present/absent pair on the observable status line.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

mod common;

/// SL-254 PHASE-05: the one refusal cause (`marker::WORKER_ENV_CAUSE`).
const WORKER_CAUSE: &str =
    "`DOCTRINE_WORKER` is set, so this process is a worker: if that is wrong, unset it";

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

fn init_repo(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    std::fs::create_dir_all(dir.join(".doctrine")).unwrap();
    std::fs::write(dir.join("a.txt"), "hello").unwrap();
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "base"]);
}

/// Make a real linked worktree fork of `src` at `<holder>/fork` on branch `branch`.
fn add_fork(src: &Path, holder: &Path, branch: &str) -> PathBuf {
    let base = git(src, &["rev-parse", "HEAD"]);
    let fork = holder.join("fork");
    git(
        src,
        &[
            "worktree",
            "add",
            "-b",
            branch,
            fork.to_str().unwrap(),
            &base,
        ],
    );
    fork
}

/// Run `doctrine <args>` in `cwd`; env governed by `worker` (Some(true) sets
/// DOCTRINE_WORKER=1; None removes it).
fn run(cwd: &Path, worker: Option<bool>, args: &[&str]) -> Output {
    let mut cmd = common::doctrine_cmd(cwd);
    cmd.args(args);
    match worker {
        Some(true) => {
            cmd.env("DOCTRINE_WORKER", "1");
        }
        Some(false) | None => {
            cmd.env_remove("DOCTRINE_WORKER");
        }
    }
    cmd.output().expect("spawn doctrine")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

/// Branch exists in `src`'s repo.
fn branch_exists(src: &Path, branch: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(src)
        .args(["rev-parse", "--verify", "--quiet", branch])
        .output()
        .expect("spawn git")
        .status
        .success()
}

/// The fork dir is a registered worktree of `src`.
fn worktree_registered(src: &Path, dir: &Path) -> bool {
    git(src, &["worktree", "list"]).contains(&dir.to_string_lossy().into_owned())
}

// --- VT-1: happy path (solo + worker) + pre-add refusals ---

#[test]
fn fork_happy_path_solo_and_worker() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let base = git(src.path(), &["rev-parse", "HEAD"]);
    let holder = tempfile::tempdir().unwrap();

    // --- solo: no worker declaration ---
    let solo_dir = holder.path().join("solo");
    let out = run(
        src.path(),
        None,
        &[
            "worktree",
            "fork",
            "--base",
            &base,
            "--branch",
            "wkr-solo",
            "--dir",
            solo_dir.to_str().unwrap(),
        ],
    );
    assert!(
        out.status.success(),
        "solo fork must succeed; stderr: {}",
        stderr(&out)
    );
    // Human status on STDERR; stdout stays empty (SL-156 — the fork builds into its
    // own in-tree `<dir>/target`, no env contract is emitted).
    assert!(
        stderr(&out).contains("forked wkr-solo"),
        "human status on stderr; got: {}",
        stderr(&out)
    );
    assert!(
        stdout(&out).trim().is_empty(),
        "fork stdout is machine-clean (empty); got: {}",
        stdout(&out)
    );
    // The worktree + branch exist AT B.
    assert!(
        worktree_registered(src.path(), &solo_dir),
        "worktree exists"
    );
    assert!(branch_exists(src.path(), "wkr-solo"), "branch exists");
    assert_eq!(
        git(&solo_dir, &["rev-parse", "HEAD"]),
        base,
        "fork branch sits at B"
    );
    // SL-254 PHASE-05: was `!marker_exists(&solo_dir)` — "solo fork has no marker".
    // Nothing stamps a marker any more, so that assertion would be vacuously true
    // whatever `fork` did. The observable that still discriminates solo from
    // `--worker` is the status line, so it is asserted in both directions here and
    // below.
    assert!(
        !stderr(&out).contains("(worker)"),
        "solo fork does NOT declare itself a worker fork; stderr: {}",
        stderr(&out)
    );

    // --- worker: declared on the status line ---
    let wkr_dir = holder.path().join("wkr");
    let out = run(
        src.path(),
        None,
        &[
            "worktree",
            "fork",
            "--base",
            &base,
            "--branch",
            "wkr-worker",
            "--dir",
            wkr_dir.to_str().unwrap(),
            "--worker",
        ],
    );
    assert!(
        out.status.success(),
        "worker fork must succeed; stderr: {}",
        stderr(&out)
    );
    assert!(
        stderr(&out).contains("(worker)"),
        "worker fork DECLARES itself on the status line; stderr: {}",
        stderr(&out)
    );
    assert_eq!(
        git(&wkr_dir, &["rev-parse", "HEAD"]),
        base,
        "worker fork at B"
    );
}

#[test]
fn fork_pre_add_refusals_leave_no_fork() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let base = git(src.path(), &["rev-parse", "HEAD"]);
    let holder = tempfile::tempdir().unwrap();

    // (a) dir already exists.
    let existing = holder.path().join("exists");
    std::fs::create_dir_all(&existing).unwrap();
    let out = run(
        src.path(),
        None,
        &[
            "worktree",
            "fork",
            "--base",
            &base,
            "--branch",
            "r-dir",
            "--dir",
            existing.to_str().unwrap(),
        ],
    );
    assert!(!out.status.success(), "dir-exists must refuse");
    assert!(stderr(&out).contains("already exists"), "names dir-exists");
    assert!(!branch_exists(src.path(), "r-dir"), "no branch created");

    // (b) branch already exists.
    let _ = add_fork(src.path(), holder.path(), "taken");
    let dir_b = holder.path().join("b");
    let out = run(
        src.path(),
        None,
        &[
            "worktree",
            "fork",
            "--base",
            &base,
            "--branch",
            "taken",
            "--dir",
            dir_b.to_str().unwrap(),
        ],
    );
    assert!(!out.status.success(), "branch-exists must refuse");
    assert!(stderr(&out).contains("branch taken"), "names branch-exists");
    assert!(!dir_b.exists(), "no fork dir created");

    // (c) base is not a commit.
    let dir_c = holder.path().join("c");
    let out = run(
        src.path(),
        None,
        &[
            "worktree",
            "fork",
            "--base",
            "deadbeefdeadbeef",
            "--branch",
            "r-base",
            "--dir",
            dir_c.to_str().unwrap(),
        ],
    );
    assert!(!out.status.success(), "B-not-a-commit must refuse");
    assert!(stderr(&out).contains("not a commit"), "names bad base");
    assert!(!branch_exists(src.path(), "r-base"), "no branch created");
    assert!(!dir_c.exists(), "no fork dir created");
}

// --- VT-2: compensating cleanup — provision fails after add, fork rolled back ---

#[test]
fn fork_rolls_back_on_provision_failure() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    // A `.worktreeinclude` naming a withheld tier makes `run_provision` bail
    // (allowlist_violations fail-closed) — a deterministic failure AFTER the
    // `git worktree add` step, forcing the compensating rollback.
    std::fs::write(src.path().join(".worktreeinclude"), ".doctrine/state/**\n").unwrap();
    git(src.path(), &["add", ".worktreeinclude"]);
    git(src.path(), &["commit", "-q", "-m", "bad allowlist"]);

    let holder = tempfile::tempdir().unwrap();
    let dir = holder.path().join("rb");
    let head_before = git(src.path(), &["rev-parse", "HEAD"]);
    let out = run(
        src.path(),
        None,
        &[
            "worktree",
            "fork",
            "--base",
            &head_before,
            "--branch",
            "wkr-rb",
            "--dir",
            dir.to_str().unwrap(),
        ],
    );
    assert!(
        !out.status.success(),
        "provision failure must fail the fork; stdout: {}",
        stdout(&out)
    );
    // GONE — not merely unspawned: the worktree + branch + dir are all reversed.
    assert!(
        !worktree_registered(src.path(), &dir),
        "worktree rolled back (gone); worktree list: {}",
        git(src.path(), &["worktree", "list"])
    );
    assert!(
        !branch_exists(src.path(), "wkr-rb"),
        "branch rolled back (gone)"
    );
    assert!(!dir.exists(), "fork dir reaped");
}

// SL-285 PHASE-02 EX-3 / RV-425 F-6: a base carrying a tracked stamp NEWER than
// the running binary is refused by the fork's regeneration, and the compensation
// reverses the claim, the worktree and the dir alike.
#[test]
fn fork_rolls_back_on_downgrade_refusal() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    common::commit_stamp(src.path(), "999.0.0");
    let base = git(src.path(), &["rev-parse", "HEAD"]);
    let holder = tempfile::tempdir().unwrap();
    let dir = holder.path().join("dg");

    let out = run(
        src.path(),
        None,
        &[
            "worktree",
            "fork",
            "--base",
            &base,
            "--branch",
            "wkr-dg",
            "--dir",
            dir.to_str().unwrap(),
        ],
    );
    assert!(
        !out.status.success(),
        "a recorded version newer than the binary must refuse; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("999.0.0"),
        "the refusal names the recorded version; stderr: {}",
        stderr(&out)
    );
    // GONE — the downgrade refusal is a post-add failure, so it rolls back.
    assert!(
        !worktree_registered(src.path(), &dir),
        "worktree rolled back (gone); worktree list: {}",
        git(src.path(), &["worktree", "list"])
    );
    assert!(
        !branch_exists(src.path(), "wkr-dg"),
        "branch rolled back (gone)"
    );
    assert!(!dir.exists(), "fork dir reaped");
}

// SL-285 PHASE-02: the same downgrade refusal on a BOUND worker fork. The record
// is written between the claim and the worktree, so the compensation must reverse
// the binding too — otherwise a rolled-back fork leaves a record for a fork that
// no longer exists.
#[test]
fn worker_fork_rolls_back_its_binding_on_downgrade_refusal() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    common::commit_stamp(src.path(), "999.0.0");
    let coord = std::fs::canonicalize(src.path()).unwrap();
    let base = git(&coord, &["rev-parse", "HEAD"]);
    let name = "wkr-bound-dg";
    let branch = format!("dispatch/{name}");
    // The binding only takes when the dir is the `<coord>/.worktrees/<name>`
    // layout the resolver strips — the same fixture the bound-fork path assumes.
    let dir = coord.join(".worktrees").join(name);

    let out = run(
        &coord,
        None,
        &[
            "worktree",
            "fork",
            "--base",
            &base,
            "--branch",
            &branch,
            "--dir",
            dir.to_str().unwrap(),
            "--worker",
            "--slice",
            "285",
            "--phase",
            "PHASE-01",
        ],
    );
    assert!(
        !out.status.success(),
        "a recorded version newer than the binary must refuse; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("999.0.0"),
        "the refusal names the recorded version; stderr: {}",
        stderr(&out)
    );
    // GONE — every leg, the bounding record included.
    assert!(
        !worktree_registered(&coord, &dir),
        "worktree rolled back (gone); worktree list: {}",
        git(&coord, &["worktree", "list"])
    );
    assert!(!branch_exists(&coord, &branch), "branch rolled back (gone)");
    assert!(!dir.exists(), "fork dir reaped");
    let record = coord
        .join(".doctrine/state/dispatch/record")
        .join(format!("{name}.toml"));
    assert!(
        !record.exists(),
        "the durable binding is reversed with the fork: {}",
        record.display()
    );
}

// --- VT-4: Orchestrator refusal drives the real CLI ---

#[test]
fn fork_refused_under_worker_mode() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let base = git(src.path(), &["rev-parse", "HEAD"]);
    let holder = tempfile::tempdir().unwrap();
    let fork = add_fork(src.path(), holder.path(), "wkr-guard");

    // SL-254 PHASE-05: the two cases below were "marked linked worktree, env unset"
    // and "DOCTRINE_WORKER on a non-linked tree", asserting two DIFFERENT messages.
    // One signal now answers for both, so they are re-pointed at what survived their
    // merger: the same worker process is refused in a linked fork and on the primary
    // tree alike, both naming the verb and the cause (`DEC-207`).

    // (1) A worker process standing in a linked worktree ⇒ refused, names verb.
    let target = holder.path().join("nope1");
    let out = run(
        &fork,
        Some(true),
        &[
            "worktree",
            "fork",
            "--base",
            &base,
            "--branch",
            "child1",
            "--dir",
            target.to_str().unwrap(),
        ],
    );
    assert!(
        !out.status.success(),
        "fork refused in a worker process inside a linked worktree; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("`fork`"),
        "refusal names the verb; stderr: {}",
        stderr(&out)
    );
    assert!(
        stderr(&out).contains(WORKER_CAUSE),
        "refusal carries the NAMED cause; stderr: {}",
        stderr(&out)
    );
    assert!(!target.exists(), "refused fork creates nothing");

    // (2) The SAME worker process on the primary (non-linked) tree ⇒ same refusal.
    let target = holder.path().join("nope2");
    let out = run(
        src.path(),
        Some(true),
        &[
            "worktree",
            "fork",
            "--base",
            &base,
            "--branch",
            "child2",
            "--dir",
            target.to_str().unwrap(),
        ],
    );
    assert!(
        !out.status.success(),
        "fork refused when DOCTRINE_WORKER set; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("`fork`"),
        "refusal names the verb; stderr: {}",
        stderr(&out)
    );
    assert!(
        stderr(&out).contains(WORKER_CAUSE),
        "topology does not change the cause; stderr: {}",
        stderr(&out)
    );
    assert!(!target.exists(), "refused fork creates nothing");
}

// ---------------------------------------------------------------------------
// SL-285 PHASE-03 — projection fidelity from an INSTALLED source
//
// `fork` composes the copy with `install::regenerate`, so the property is not
// "something was written" but "what was written equals what a fresh install on
// the same inputs writes". Everything here drives the built binary; the fixtures
// come from `common::installed_source` (two harnesses, detection-only) and
// `common::fresh_inputs` (the fork's tracked inputs alone).
// ---------------------------------------------------------------------------

/// The projection domain EX-1 compares. Base backings are INCLUDED — install
/// writes them, so the comparison also proves install is idempotent on the
/// committed tree; the stamp (`.doctrine/doctrine.toml`) is the only exclusion,
/// because install records it and regeneration does not (DEC-401).
const PROJECTION_DOMAIN: &[&str] = &[
    ".doctrine/skills",
    ".doctrine/agents",
    // No workflows: doctrine ships none (no `plugins/doctrine/workflows`, the
    // manifest lists the root but nothing populates it), so both trees are
    // legitimately empty and comparing it would only compare two absences.
    ".doctrine/memory/shipped",
    ".agents/skills",
    ".claude/agents",
    ".claude/skills",
    ".codex",
    "AGENTS.md",
    "CLAUDE.md",
    ".mcp.json",
    ".gitignore",
    ".doctrine/project-orientation.md",
];

/// Second generation compares the REGENERATED tier only: the base backings are
/// the same commit in both trees, so including them would pad a claim about
/// regeneration with bytes nobody regenerated.
const REGENERATED_DOMAIN: &[&str] = &[
    ".doctrine/skills",
    ".agents/skills",
    ".claude/agents",
    ".claude/skills",
    // `.claude/workflows` excluded for the same reason as EX-1's: none shipped.
    ".codex",
];

/// `doctrine install --yes -p <root>`, with NO `--agent`: an explicit agent is a
/// directive that skips detection, and the whole point is that the harness set is
/// derived from the tree's own committed bytes.
fn install(root: &Path) -> Output {
    common::doctrine_cmd(root)
        .args(["install", "--yes", "-p"])
        .arg(root)
        .output()
        .expect("spawn doctrine install")
}

/// A tree's projection digest with every symlink re-expressed by its RESOLVED
/// target, relative to that tree's own root.
///
/// `common::tree_digest` records symlinks UNFOLLOWED as `link:<value>`, so a raw
/// comparison would flag a difference that is only the two trees' roots whenever a
/// link carries an absolute target. Resolving each link and stripping its own root
/// makes `link:.doctrine/skills/code-review` mean the same thing in both trees, and
/// a dangling link fails here by name instead of passing as a byte difference.
fn resolved_digest(root: &Path, domain: &[&str]) -> std::collections::BTreeMap<PathBuf, String> {
    let root = std::fs::canonicalize(root).expect("canonicalize digest root");
    common::digest_domain(&root, domain)
        .into_iter()
        .map(|(rel, digest)| {
            let Some(target) = digest.strip_prefix("link:") else {
                return (rel, digest);
            };
            let link = root.join(&rel);
            let canonical = std::fs::canonicalize(&link);
            assert!(
                canonical.is_ok(),
                "harness link {} -> {target} must resolve (clippy::panic is denied \
                 workspace-wide, so a failure is an assertion, not a panic)",
                link.display()
            );
            let resolved = canonical.expect("checked: the link resolves");
            let stripped = resolved.strip_prefix(&root);
            assert!(
                stripped.is_ok(),
                "harness link {} must resolve inside its own tree, got {}",
                link.display(),
                resolved.display()
            );
            let own = stripped.expect("checked: inside its own tree");
            (rel, format!("link:{}", own.display()))
        })
        .collect()
}

/// Every entry of each harness link dir resolves to a REAL payload inside its own
/// tree. The payload's SHAPE is the member kind's: a `skills` link points at a
/// canon DIRECTORY and must carry a `SKILL.md` (that is what makes the projected
/// skill readable rather than a dangling name), while an `agents`/`workflows`
/// link points at a canon FILE (`.doctrine/agents/<f>`, `.doctrine/workflows/<f>`).
/// Accepting either shape everywhere would let a skill dir projected as a bare
/// file — or an agent def projected as a dir — pass.
fn assert_links_resolve(root: &Path, dirs: &[&str]) {
    let root = std::fs::canonicalize(root).expect("canonicalize link root");
    let mut seen = 0usize;
    for dir in dirs {
        let d = root.join(dir);
        if !d.exists() {
            continue;
        }
        let is_skill_dir = Path::new(dir).file_name() == Some(std::ffi::OsStr::new("skills"));
        for entry in std::fs::read_dir(&d).expect("read link dir") {
            let path = entry.expect("link dir entry").path();
            let canonical = std::fs::canonicalize(&path);
            assert!(
                canonical.is_ok(),
                "{} must resolve — a dangling link projects a name, not a payload",
                path.display()
            );
            let resolved = canonical.expect("checked: the link resolves");
            assert!(
                resolved.starts_with(&root),
                "{} must resolve inside its own tree, got {}",
                path.display(),
                resolved.display()
            );
            if is_skill_dir {
                assert!(
                    resolved.join("SKILL.md").is_file(),
                    "{} must resolve to a skill dir carrying SKILL.md, got {}",
                    path.display(),
                    resolved.display()
                );
            } else {
                assert!(
                    resolved.is_file(),
                    "{} must resolve to a real def file, got {}",
                    path.display(),
                    resolved.display()
                );
            }
            seen += 1;
        }
    }
    assert!(seen > 0, "positive control: {dirs:?} carried link entries");
}

/// SL-285 PHASE-03 EX-1 / VT-1: a fork of an installed source projects what an
/// install on the fork's OWN inputs projects — not what the source tree holds.
#[test]
fn fork_from_installed_source_projects_like_a_fresh_install() {
    let tmp = tempfile::tempdir().unwrap();
    let source = common::installed_source(&tmp.path().join("src"));

    // R3 precondition: a second install must converge to the COMMITTED bytes, so
    // the wiring the fork inherits is what `plan_wire` would write. A dirty tree
    // here means the fork's regeneration refuses (`stale source wiring`) — STOP
    // and report the paths; never relax this comparison.
    let again = install(&source);
    assert!(
        again.status.success(),
        "second install must succeed: {}",
        stderr(&again)
    );
    assert_eq!(
        git(&source, &["status", "--porcelain"]),
        "",
        "STOP (R3): install is not idempotent on the committed tree, so the fork's \
         regeneration would refuse stale tracked wiring"
    );

    let base = git(&source, &["rev-parse", "HEAD"]);
    let holder = tempfile::tempdir().unwrap();
    let fork = holder.path().join("fork");
    let out = run(
        &source,
        None,
        &[
            "worktree",
            "fork",
            "--base",
            &base,
            "--branch",
            "proj",
            "--dir",
            fork.to_str().unwrap(),
        ],
    );
    assert!(
        out.status.success(),
        "fork from an installed source must succeed: {}",
        stderr(&out)
    );
    assert!(
        !stderr(&out).contains("stale source wiring"),
        "STOP (R3): the fork's regeneration refused tracked wiring: {}",
        stderr(&out)
    );

    // The baseline: install on the fork's tracked inputs alone (no derived tier,
    // no copied allowlisted file — regeneration reads neither).
    let (_fresh_holder, fresh) = common::fresh_inputs(&fork);
    let fresh_install = install(&fresh);
    assert!(
        fresh_install.status.success(),
        "install on the fresh inputs must succeed: {}",
        stderr(&fresh_install)
    );

    assert_links_resolve(&fork, &[".claude/skills", ".agents/skills"]);

    let got = resolved_digest(&fork, PROJECTION_DOMAIN);
    let want = resolved_digest(&fresh, PROJECTION_DOMAIN);

    // Positive control, per member and per side: `digest_domain` records a missing
    // member as `absent`, so a `contains_key` on one member proves nothing and two
    // absent-in-both trees would compare equal. EVERY compared member must be a
    // real projection output in the fork AND in the fresh install — no member of
    // this domain is legitimately absent: each is either a shipped install output
    // (skills, agents, workflows, the memory corpus, the codex config, the two
    // harness files, the managed ignore block, the orientation doc) or a tracked
    // base backing install rewrites, and the source here is a real install.
    for &rel in PROJECTION_DOMAIN {
        for (side, digest) in [("fork", &got), ("fresh install", &want)] {
            let value = digest.get(Path::new(rel));
            assert!(
                value.is_some_and(|digest| digest != common::ABSENT),
                "positive control: {rel} is absent in the {side}, so the equality \
                 below would be vacuous about it"
            );
        }
    }

    // One named artefact per family, so "populated" is a claim about what the legs
    // actually projected rather than about a directory entry's existence.
    assert!(
        fork.join(".codex/hooks.json").is_file(),
        "control: the codex hook family is projected into the fork"
    );
    let skills_dir = fork.join(".doctrine/skills");
    let mut with_skill_md = 0usize;
    for entry in std::fs::read_dir(&skills_dir).expect("read the fork's .doctrine/skills") {
        if entry
            .expect("skills entry")
            .path()
            .join("SKILL.md")
            .is_file()
        {
            with_skill_md += 1;
        }
    }
    assert!(
        with_skill_md > 0,
        "control: .doctrine/skills carries at least one skill dir with SKILL.md"
    );
    assert!(
        std::fs::read_dir(fork.join(".doctrine/memory/shipped"))
            .expect("read the fork's .doctrine/memory/shipped")
            .next()
            .is_some(),
        "control: the shipped memory corpus is materialised (non-empty)"
    );

    assert_eq!(
        got, want,
        "the fork's projection must equal a fresh install on the fork's own inputs"
    );
}

/// SL-285 PHASE-03 EX-2 / VT-1: regeneration leaves nothing tracked-dirty, and
/// creates no `.pi/skills` (pi reads `.agents/skills`; a duplicate dir makes it
/// warn). The control is non-vacuous: `assert_links_resolve` proves both link
/// dirs carry resolving entries, not merely that they exist.
#[test]
fn fork_from_installed_source_leaves_empty_porcelain_and_no_pi_skills() {
    let tmp = tempfile::tempdir().unwrap();
    let source = common::installed_source(&tmp.path().join("src"));
    let base = git(&source, &["rev-parse", "HEAD"]);
    let holder = tempfile::tempdir().unwrap();
    let fork = holder.path().join("fork");

    let out = run(
        &source,
        None,
        &[
            "worktree",
            "fork",
            "--base",
            &base,
            "--branch",
            "clean",
            "--dir",
            fork.to_str().unwrap(),
        ],
    );
    assert!(
        out.status.success(),
        "fork from an installed source must succeed: {}",
        stderr(&out)
    );

    // Positive control: regeneration actually projected and linked the derived
    // tier — every entry resolves to a payload inside the fork.
    assert_links_resolve(&fork, &[".claude/skills", ".agents/skills"]);

    assert_eq!(
        git(&fork, &["status", "--porcelain"]),
        "",
        "every generated path must be ignored: a dirty porcelain is a real \
         ignore-contract defect, not a fixture problem"
    );
    assert!(
        !fork.join(".pi/skills").exists(),
        "no .pi/skills is created — pi reads .agents/skills and warns on a duplicate"
    );
}

/// SL-285 PHASE-03 EX-4 / VT-1: a fork of a fork regenerates the same harness set
/// — the derived tier is derived from the parent's own inputs, so generation 2
/// must not shed a link, a def or a baked hook.
#[test]
fn second_generation_fork_matches_the_first() {
    let tmp = tempfile::tempdir().unwrap();
    let source = common::installed_source(&tmp.path().join("src"));
    let base = git(&source, &["rev-parse", "HEAD"]);
    let holder = tempfile::tempdir().unwrap();
    let first = holder.path().join("gen1");

    let out = run(
        &source,
        None,
        &[
            "worktree",
            "fork",
            "--base",
            &base,
            "--branch",
            "gen1",
            "--dir",
            first.to_str().unwrap(),
        ],
    );
    assert!(out.status.success(), "first fork: {}", stderr(&out));

    // Generation 2 forks FROM a linked worktree, not from the primary tree.
    let base2 = git(&first, &["rev-parse", "HEAD"]);
    let second = holder.path().join("gen2");
    let out = run(
        &first,
        None,
        &[
            "worktree",
            "fork",
            "--base",
            &base2,
            "--branch",
            "gen2",
            "--dir",
            second.to_str().unwrap(),
        ],
    );
    assert!(
        out.status.success(),
        "second generation fork: {}",
        stderr(&out)
    );

    // Positive controls: the child really is provisioned, and clean.
    assert!(
        std::fs::read_dir(second.join(".agents/skills"))
            .expect("regenerated .agents/skills")
            .next()
            .is_some(),
        "control: generation 2 carries the codex link dir"
    );
    assert_eq!(
        git(&second, &["status", "--porcelain"]),
        "",
        "generation 2 leaves nothing tracked-dirty"
    );

    assert_links_resolve(&first, &[".claude/skills", ".agents/skills"]);
    assert_links_resolve(&second, &[".claude/skills", ".agents/skills"]);

    // Bytes, not names: same defs, same links (resolved in their own tree), same
    // baked hook command.
    assert_eq!(
        resolved_digest(&first, REGENERATED_DOMAIN),
        resolved_digest(&second, REGENERATED_DOMAIN),
        "generation 2 must regenerate the SAME derived tier as generation 1"
    );
}
