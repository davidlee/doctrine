// SPDX-License-Identifier: GPL-3.0-only
//! Shared integration-test helpers.
//!
//! Single source: `src/test_support.rs`, `#[path]`-included here so integration tests
//! (separate compilation units from the lib unit tests) reuse the same runtime
//! `repo_root()` resolver. See CHR-014.
//!
//! SL-162: added `doctrine_bin()` runtime binary-path resolver.

#![allow(dead_code, unused_imports)] // shared helpers: not every includer uses every fn (SL-162 D5)

#[path = "../../src/test_support.rs"]
mod test_support;

pub(crate) use test_support::{WORKER_MARKER_REL, doctrine_bin, repo_root, under_worker_marker};

/// Entity-tree roots, from the same bytes the binary compiles — `src/kinds/dirs.rs`
/// imports nothing precisely so a fixture can plant `.doctrine/…` without typing
/// a second copy of the path (STD-001; SL-233 RV-321 F-4).
#[path = "../../src/kinds/dirs.rs"]
mod kind_dirs;

pub(crate) use kind_dirs::SLICE_DIR;

/// Canonical config path — mirrors `src/dtoml::DOCTRINE_TOML` (which
/// integration tests can't import from a binary-only crate).
pub(crate) const DOCTRINE_TOML: &str = ".doctrine/doctrine.toml";

/// The digest convention the shell uses (`crate::git::sha256`), mirrored so a
/// test can declare what a content binding must match. Byte-identical to it,
/// and to `contentset`'s own copy — the shape a leaf owns directly rather than
/// depending on the impure `git` seam.
///
/// One copy, here: it was written out three times across the design e2e crates
/// (SL-233 PHASE-16), which is three chances for one of them to drift into a
/// different convention while still passing.
pub(crate) fn sha256(bytes: &[u8]) -> String {
    use sha2::Digest as _;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// The built binary, spawned with `cwd` **bound** and the worker-mode env leg
/// stripped — the single spawn seam for every integration test (IMP-352).
///
/// Two inputs a fixture must declare rather than inherit from the harness:
///
/// * **cwd.** `worker_guard` resolves its root from CWD alone
///   (`crate::root::find(None, …)` in `src/commands/guard.rs` — RV-319 F-1 pins
///   that it ignores `-p`). So an unbound cwd silently roots the child in the
///   *ambient* tree instead of the scratch root the fixture passes to `-p`.
///   Inside a dispatch worker fork that ambient tree is marked, and the guard
///   correctly refuses an authored write the fixture never aimed there.
/// * **`DOCTRINE_WORKER`.** The env leg is root-independent, so binding cwd does
///   not reach it — the two legs are orthogonal and both must be declared.
///
/// This is a *declared* stand-down, not a bypass: the child of a test binary is a
/// fixture process operating on a scratch root, not the worker agent the marker
/// identifies. It grants nothing a worker does not already have — the marker leg
/// is CWD-keyed, so leaving the fork's directory already lifts it on any HEAD.
///
/// A test whose subject IS the ambient signal (the worker-guard goldens) sets it
/// back: `doctrine_cmd(dir).env("DOCTRINE_WORKER", "1")` — a later `env` call
/// overrides this `env_remove`.
///
/// Pass the root the test actually operates on: its scratch tempdir, or
/// [`repo_root`] for the few goldens that read the real tree.
pub(crate) fn doctrine_cmd(cwd: &std::path::Path) -> std::process::Command {
    let mut cmd = std::process::Command::new(doctrine_bin());
    cmd.current_dir(cwd).env_remove("DOCTRINE_WORKER");
    cmd
}

/// A temp base whose ancestry to `/` carries no project marker
/// (`.git`/`.jj`/`.project`/`Cargo.toml`), for tests exercising the **no-root**
/// path: `root::find` (src/root.rs) walks CWD up to `/`, so a stray marker above
/// the system tempdir (e.g. a leftover `/tmp/.git`) would resolve an incidental
/// root and mis-fire the assertion. Panics loudly if no clean base exists — a
/// missed assertion is worse than a failed test. (Tests needing a root instead
/// plant one: `create_dir(dir/".git")`.)
pub(crate) fn marker_free_base() -> std::path::PathBuf {
    let markers = [".git", ".jj", ".project", "Cargo.toml"];
    let candidates = [
        std::path::PathBuf::from("/dev/shm"),
        std::path::PathBuf::from("/var/tmp"),
        std::env::temp_dir(),
    ];
    for base in candidates {
        if base.is_dir()
            && base
                .ancestors()
                .all(|a| markers.iter().all(|m| !a.join(m).exists()))
        {
            return base;
        }
    }
    panic!("no marker-free temp base available to exercise the no-root path");
}

// ---------------------------------------------------------------------------
// worker-fork fixtures (ISS-028 / SL-236 §9)
//
// The worker-mode marker leg is `is_linked_worktree(root) && marker_present(root)`
// (src/worktree/marker.rs `resolve_mode`). A marker file dropped in a bare tempdir
// is therefore NEVER refused — a fixture built that way passes identically before
// and after a guard change and proves nothing. These helpers build a GENUINE linked
// worktree and self-validate both legs, so a test cannot silently degrade into that
// vacuous shape.
// ---------------------------------------------------------------------------

/// `git -C <dir> <args>`, asserting success; returns trimmed stdout.
pub(crate) fn git(dir: &std::path::Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("spawn git");
    assert!(
        out.status.success(),
        "git {args:?} in {} failed: {}",
        dir.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A doctrine-rooted git repo with one commit — `.git` + `.doctrine` make it a
/// project root `root::find` resolves.
pub(crate) fn init_repo(dir: &std::path::Path) {
    std::fs::create_dir_all(dir).expect("create repo dir");
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    std::fs::create_dir_all(dir.join(".doctrine")).expect("create .doctrine");
    std::fs::write(dir.join("a.txt"), "hello").expect("seed file");
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "base"]);
}

/// True iff `root` is a *linked* worktree — `--git-dir` differs from
/// `--git-common-dir`, mirroring `src/worktree/shared.rs::is_linked_worktree`.
pub(crate) fn is_linked_worktree(root: &std::path::Path) -> bool {
    git(root, &["rev-parse", "--git-dir"]) != git(root, &["rev-parse", "--git-common-dir"])
}

/// Assert BOTH marker-leg conditions hold at `root` (anti-cheat: a fixture that
/// only plants the marker file is never refused, so it would prove nothing).
pub(crate) fn assert_marked_linked_fork(root: &std::path::Path) {
    assert!(
        is_linked_worktree(root),
        "fixture at {} must be a GENUINE linked worktree, not a bare tempdir — \
         `resolve_mode` requires is_linked && marker_present",
        root.display()
    );
    assert!(
        root.join(WORKER_MARKER_REL).exists(),
        "fixture at {} must carry the worker marker at {WORKER_MARKER_REL}",
        root.display()
    );
}

/// Fork `src` into a real linked worktree at `dest` on a new `branch`, stamp the
/// worker marker, and self-validate. `src` must already be an initialised repo.
pub(crate) fn marked_linked_fork(src: &std::path::Path, dest: &std::path::Path, branch: &str) {
    let base = git(src, &["rev-parse", "HEAD"]);
    git(
        src,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            branch,
            dest.to_str().expect("utf8 fork path"),
            &base,
        ],
    );
    let marker = dest.join(WORKER_MARKER_REL);
    std::fs::create_dir_all(marker.parent().expect("marker parent")).expect("create marker dir");
    std::fs::write(&marker, b"").expect("stamp worker marker");
    assert_marked_linked_fork(dest);
}
