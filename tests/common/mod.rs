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

pub(crate) use test_support::{LinkedTrees, doctrine_bin, repo_root, under_worker_marker};

/// Entity-tree roots, from the same bytes the binary compiles — `src/kinds/dirs.rs`
/// imports nothing precisely so a fixture can plant `.doctrine/…` without typing
/// a second copy of the path (STD-001; SL-233 RV-321 F-4).
#[path = "../../src/kinds/dirs.rs"]
mod kind_dirs;

pub(crate) use kind_dirs::{REVIEW_DIR, SLICE_DIR};

/// Canonical config path — mirrors `src/dtoml::DOCTRINE_TOML` (which
/// integration tests can't import from a binary-only crate).
pub(crate) const DOCTRINE_TOML: &str = ".doctrine/doctrine.toml";

/// The degraded user-MCP line's text: the codex leg runs but degrades, because
/// the child's `CODEX_HOME` is pinned to [`SAFE_CODEX_HOME`]. Mirrors
/// `src/codex_mcp.rs`'s `text::PREFIX` + `text::SKIPPED` — the header half a test
/// asserts on; the cause clause differs per degradation.
pub(crate) const CODEX_MCP_SKIPPED: &str = "codex: user MCP registration skipped";

/// The local install marker, relative to the project root — mirrors
/// `install::INSTALL_MARKER_REL` (SL-282).
pub(crate) const INSTALL_MARKER: &str = ".doctrine/state/install.toml";

/// The project's `doctrine.toml`, which carries the tracked install stamp.
pub(crate) fn doctrine_toml(root: &std::path::Path) -> std::path::PathBuf {
    root.join(DOCTRINE_TOML)
}

/// Plant the local marker install writes, recording `version` for claude.
pub(crate) fn write_marker(root: &std::path::Path, version: &str) {
    let path = root.join(INSTALL_MARKER);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        path,
        format!("version = \"{version}\"\nharnesses = [\"claude\"]\n"),
    )
    .unwrap();
}

/// Plant the tracked stamp: a `doctrine.toml` carrying only
/// `[install] installed-version`.
pub(crate) fn write_stamp(root: &std::path::Path, version: &str) {
    let path = doctrine_toml(root);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        path,
        format!("[install]\ninstalled-version = \"{version}\"\n"),
    )
    .unwrap();
}

/// Plant AND commit the tracked stamp — the F-6 lever: a worktree added from this
/// commit inherits it, so regeneration refuses the downgrade (SL-285 PHASE-02).
///
/// `-f`: the sync-side fixtures ignore `.doctrine/` wholesale (the provision
/// fixture does), while the production stamp is tracked — forcing the add is the
/// faithful shape and keeps the lever independent of a fixture's ignore rules.
pub(crate) fn commit_stamp(root: &std::path::Path, version: &str) {
    write_stamp(root, version);
    git(root, &["add", "-f", DOCTRINE_TOML]);
    git(root, &["commit", "-q", "-m", "stamp"]);
}

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

/// An absolute path that is *never created* — [`doctrine_cmd`] pins `CODEX_HOME`
/// here, so no test child can read or write the developer's real codex config
/// (RV-427 F-3, SL-286). Codex suites that need a real `CODEX_HOME` override it
/// with `.env` (fixture-wins: a later `env` call overrides this one).
pub(crate) const SAFE_CODEX_HOME: &str = "/nonexistent/doctrine-tests/codex-home";

/// Put the binary under test on a fake `PATH` directory as `doctrine` — a
/// symlink, so the running binary and the `PATH` entry canonicalise to the same
/// file, which the codex user MCP bake rule requires (SL-286 DEC-404). Lives
/// here because this module is the one place that names the binary's path.
#[cfg(unix)]
pub(crate) fn link_doctrine_into(bin_dir: &std::path::Path) {
    std::os::unix::fs::symlink(doctrine_bin(), bin_dir.join("doctrine"))
        .expect("link doctrine onto the fake PATH");
}

/// The built binary, spawned with `cwd` **bound** and the worker-mode env leg
/// stripped — the single spawn seam for every integration test (IMP-352).
///
/// Two inputs a fixture must declare rather than inherit from the harness:
///
/// * **cwd.** Bound so the child operates on the scratch root the fixture built,
///   not the *ambient* tree it happens to be spawned from. (Since SL-254 PHASE-05
///   the worker verdict no longer consults a root at all, but cwd still decides
///   which tree the child's other verbs read.)
/// * **`DOCTRINE_WORKER`.** Since `DEC-207` this is the WHOLE of worker identity,
///   and it is inherited from the parent process — so a test run from inside a real
///   worker would otherwise see every authored-write fixture refused.
///
/// This is a *declared* stand-down, not a bypass: the child of a test binary is a
/// fixture process operating on a scratch root, not the worker agent the env leg
/// identifies.
///
/// A test whose subject IS the ambient signal (the worker-guard goldens) sets it
/// back: `doctrine_cmd(dir).env("DOCTRINE_WORKER", "1")` — a later `env` call
/// overrides this `env_remove`.
///
/// Pass the root the test actually operates on: its scratch tempdir, or
/// [`repo_root`] for the few goldens that read the real tree.
pub(crate) fn doctrine_cmd(cwd: &std::path::Path) -> std::process::Command {
    let mut cmd = std::process::Command::new(doctrine_bin());
    cmd.current_dir(cwd)
        .env_remove("DOCTRINE_WORKER")
        .env("CODEX_HOME", SAFE_CODEX_HOME);
    cmd
}

/// The one environment in which the codex user-MCP install leg **proposes**
/// (SL-286):
///
/// * the running executable canonicalises to the `doctrine` entry on `PATH`
///   ([`link_doctrine_into`]), which the bake rule compares;
/// * a fake `codex` sits on `PATH` and appends each invocation's argv
///   (space-joined, one line per call) to its private log — so a test can assert
///   what WOULD run without running it;
/// * `HOME` is an absolute scratch path and `CODEX_HOME` is **unset**, so the leg
///   reads and writes a fixture user config rather than degrading under
///   [`SAFE_CODEX_HOME`].
///
/// Any leg missing one of those degrades instead of proposing, so a test that
/// wants the propose path must take every input from here.
#[cfg(unix)]
pub(crate) struct ProposingEnv {
    pub(crate) home: std::path::PathBuf,
    pub(crate) bin: std::path::PathBuf,
    argv_log: std::path::PathBuf,
}

#[cfg(unix)]
impl ProposingEnv {
    /// Lay out `<root>/home` and `<root>/bin` under an existing scratch root.
    pub(crate) fn new(root: &std::path::Path) -> Self {
        use std::os::unix::fs::PermissionsExt as _;

        let home = root.join("home");
        let bin = root.join("bin");
        std::fs::create_dir_all(&home).expect("create fixture HOME");
        std::fs::create_dir_all(&bin).expect("create fixture PATH dir");
        link_doctrine_into(&bin);

        let argv_log = root.join("codex-argv");
        let codex = bin.join("codex");
        // One line per call, each argv element followed by `\x1f`: the unit
        // separator cannot occur in a path or a flag, so the boundary between
        // two elements carries no join ambiguity (a space-joined log could not
        // tell one argument with a space from two arguments). The log path is
        // single-quoted so no character in it reaches the shell.
        std::fs::write(
            &codex,
            format!(
                "#!/bin/sh\nfor a in \"$@\"; do printf '%s\\037' \"$a\"; done >> '{}'\nprintf '\\n' >> '{}'\n",
                argv_log.display(),
                argv_log.display(),
            ),
        )
        .expect("write fake codex");
        std::fs::set_permissions(&codex, std::fs::Permissions::from_mode(0o755))
            .expect("make fake codex executable");

        Self {
            home,
            bin,
            argv_log,
        }
    }

    /// The spawn seam ([`doctrine_cmd`]) with this environment's `HOME`/`PATH`
    /// bound and `CODEX_HOME` removed — a later `env` call still overrides.
    pub(crate) fn doctrine(&self, cwd: &std::path::Path) -> std::process::Command {
        let mut cmd = doctrine_cmd(cwd);
        cmd.env_remove("CODEX_HOME")
            .env("HOME", &self.home)
            .env("PATH", &self.bin);
        cmd
    }

    /// Every `codex` invocation seen so far, one argv per call — each element
    /// intact, so a value with a space in it stays one value; empty when the fake
    /// was never run (the log does not exist).
    pub(crate) fn calls(&self) -> Vec<Vec<String>> {
        let Ok(log) = std::fs::read_to_string(&self.argv_log) else {
            return Vec::new();
        };
        log.lines()
            // `split_terminator` drops only the trailing separator the writer
            // always emits, so an empty element would still be one.
            .map(|line| line.split_terminator('\x1f').map(str::to_string).collect())
            .collect()
    }

    /// The user-level config the propose leg would edit: `home/.codex/config.toml`.
    pub(crate) fn user_config(&self) -> std::path::PathBuf {
        self.home.join(".codex/config.toml")
    }
}

/// Run `cmd` with `input` piped to its stdin (then closed); output captured.
pub(crate) fn output_with_stdin(
    mut cmd: std::process::Command,
    input: &str,
) -> std::process::Output {
    use std::io::Write as _;
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = cmd.spawn().expect("spawn command");
    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(input.as_bytes())
        .expect("write stdin");
    child.wait_with_output().expect("wait for command")
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
// worker-fork fixtures (ISS-028 / SL-236 §9; re-cut at SL-254 PHASE-05)
//
// Worker mode is now the `DOCTRINE_WORKER` env leg ALONE (`DEC-207`) — a property of
// the process, not of the tree. So the anti-cheat these helpers used to enforce (a
// marker file in a bare tempdir is never refused, because the marker leg required
// `is_linked_worktree && marker_present`) no longer has a subject: there is no marker,
// and topology does not participate in the verdict.
//
// What survives is the fixture's OTHER job — standing up a genuine linked worktree, so
// a test exercising fork-shaped behaviour is exercising a real fork. The refusal itself
// is now provoked by setting `DOCTRINE_WORKER=1` on the child process under test, which
// is both simpler and closer to how a real worker is established (the confinement argv
// sets it). ISS-028 closes here: a fork is no longer refused merely for standing in a
// stamped tree, which is what broke tests that shell the doctrine CLI.
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

/// Assert `root` is a GENUINE linked worktree, not a bare tempdir standing in for one.
pub(crate) fn assert_linked_fork(root: &std::path::Path) {
    assert!(
        is_linked_worktree(root),
        "fixture at {} must be a GENUINE linked worktree, not a bare tempdir",
        root.display()
    );
}

/// Fork `src` into a real linked worktree at `dest` on a new `branch`, and
/// self-validate. `src` must already be an initialised repo. To make the fork's
/// PROCESS a worker, set `DOCTRINE_WORKER=1` on the command under test — the tree
/// itself no longer carries identity (`DEC-207`).
pub(crate) fn linked_fork(src: &std::path::Path, dest: &std::path::Path, branch: &str) {
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
    assert_linked_fork(dest);
}

/// The ids a `design adopt` report's `unchanged` line lists, as a set — the
/// parser having read every one of them back (`SL-261` PHASE-05 `A2`).
///
/// Shared by the suites whose parser-readout probe forces a whitespace head and
/// reads the decomposition out of `adopt --dry-run`, on the same reasoning as
/// [`sha256`] above: one copy, not one per suite.
pub(crate) fn unchanged_ids(report: &str) -> std::collections::BTreeSet<&str> {
    report
        .lines()
        .find_map(|line| line.strip_prefix("unchanged "))
        .map(|ids| ids.split_whitespace().collect())
        .unwrap_or_default()
}

/// Relative path -> `file:<sha256>` | `link:<value>` | `dir`, over a whole tree
/// (symlinks are recorded, never followed). Equal digests mean nothing changed.
pub(crate) fn tree_digest(
    dir: &std::path::Path,
) -> std::collections::BTreeMap<std::path::PathBuf, String> {
    use std::fs;
    fn walk(
        root: &std::path::Path,
        dir: &std::path::Path,
        acc: &mut std::collections::BTreeMap<std::path::PathBuf, String>,
    ) {
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
                acc.insert(rel, format!("file:{}", sha256(&fs::read(&p).unwrap())));
            }
        }
    }
    let mut acc = std::collections::BTreeMap::new();
    walk(dir, dir, &mut acc);
    acc
}

/// [`digest_domain`]'s marker for a domain member that is missing from a tree.
///
/// A caller comparing two digests must assert the value is NOT this: two trees
/// that never ran a projection produce equal maps of `absent` values.
pub(crate) const ABSENT: &str = "absent";

/// [`tree_digest`] restricted to `domain`, with a missing member recorded as
/// [`ABSENT`] rather than dropped.
///
/// `tree_digest` takes a DIRECTORY root (it `read_dir`-unwraps it), so it cannot
/// digest one plain-file path on its own (`AGENTS.md`, `.mcp.json`); a domain
/// member may be either a dir or a file, and this projection covers both.
///
/// Recording a missing member rather than omitting it keeps the absence
/// VISIBLE; it does not by itself make a comparison non-vacuous — two trees that
/// never ran a projection still produce equal maps — so a caller must assert
/// every compared value is not [`ABSENT`].
///
/// `.git` and everything beneath it is excluded: VCS metadata is not a
/// projection output, and its shape differs per checkout (a linked worktree's
/// `.git` is a file naming the parent's gitdir, a primary tree's is a dir).
pub(crate) fn digest_domain(
    root: &std::path::Path,
    domain: &[&str],
) -> std::collections::BTreeMap<std::path::PathBuf, String> {
    let whole = tree_digest(root);
    let mut acc = std::collections::BTreeMap::new();
    for rel in domain {
        let prefix = std::path::Path::new(*rel);
        if std::fs::symlink_metadata(root.join(*rel)).is_err() {
            acc.insert(prefix.to_path_buf(), ABSENT.to_string());
            continue;
        }
        acc.extend(
            whole
                .iter()
                // `Path::starts_with` is component-wise, so this drops `.git`
                // itself and everything under it, never `.github`.
                .filter(|(path, _)| !path.starts_with(".git") && path.starts_with(prefix))
                .map(|(path, digest)| (path.clone(), digest.clone())),
        );
    }
    acc
}

// ---------------------------------------------------------------------------
// installed-source fixtures (SL-285 PHASE-03: projection fidelity)
//
// A fork regenerates the derived tier from the SOURCE's tracked inputs, so a
// fixture standing one up must be a real install: a tracked `installed-version`,
// both harnesses derivable from COMMITTED bytes, and every install output
// committed — the fork is a checkout of HEAD, so an uncommitted install would
// leave the fork un-installed and every assertion vacuously true.
// ---------------------------------------------------------------------------

/// The tracked files an install writes or rewrites, relative to the project root.
///
/// Staged EXPLICITLY, never `git add -A`: a sweep would commit whatever a leg
/// happened to write, `.codex/` above all — the shipped managed `.gitignore` block
/// carries no entry for it (SL-285 PHASE-03).
const INSTALL_TRACKED: &[&str] = &[
    ".gitignore",
    ".claude/settings.json",
    "AGENTS.md",
    "CLAUDE.md",
    ".mcp.json",
    DOCTRINE_TOML,
    ".doctrine/project-orientation.md",
];

/// The projection domain: every member `install` projects into a tree, and which a
/// fork's `regenerate` must therefore reproduce (SL-285 PHASE-03 EX-1).
///
/// Derived members come first; the base backings — files install REWRITES from
/// scratch rather than deriving — trail them. The split exists so a caller
/// asserting about REGENERATION alone can take [`regenerated_domain`] and not pad
/// the claim with bytes nobody regenerated. The stamp (`.doctrine/doctrine.toml`)
/// is the one exclusion: install records it and regeneration does not (DEC-401).
pub(crate) const PROJECTION_DOMAIN: &[&str] = &[
    ".doctrine/skills",
    ".doctrine/agents",
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

/// [`PROJECTION_DOMAIN`]'s base backings: identical in two trees at the same
/// commit, so comparing them says nothing about regeneration.
const BASE_BACKINGS: &[&str] = &[
    "AGENTS.md",
    "CLAUDE.md",
    ".mcp.json",
    ".gitignore",
    ".doctrine/project-orientation.md",
];

/// [`PROJECTION_DOMAIN`] minus [`BASE_BACKINGS`] — the regenerated tier alone.
///
/// The workflow roots are deliberately NOT members: doctrine ships no workflows
/// (no `plugins/doctrine/workflows`; the manifest lists the root but nothing
/// populates it), so both sides are legitimately empty and comparing them would
/// only ever compare two absences.
pub(crate) fn regenerated_domain() -> Vec<&'static str> {
    PROJECTION_DOMAIN
        .iter()
        .copied()
        .filter(|rel| !BASE_BACKINGS.contains(rel))
        .collect()
}

/// `doctrine install --yes -p <root>` with NO `--agent`: an explicit agent is a
/// directive that skips detection, and the fixtures' point is that the harness set
/// is derived from the tree's own committed bytes.
///
/// Non-asserting, so the caller decides what a failed install means; this is the
/// one install spawn, and [`install_into`] is the committing wrapper.
pub(crate) fn install(root: &std::path::Path) -> std::process::Output {
    doctrine_cmd(root)
        .args(["install", "--yes", "-p"])
        .arg(root)
        .output()
        .expect("spawn doctrine install")
}

/// Write `text` at `root/rel`, creating parents. Private: the plants below are
/// this module's only writers, so the path is always a literal.
fn plant(root: &std::path::Path, rel: &str, text: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().expect("planted path has a parent"))
        .expect("create planted dir");
    std::fs::write(&path, text).expect("plant file");
}

/// Install `root` for real, then commit what the install wrote.
///
/// Two harnesses, **detection only** (SL-285 PHASE-03): `.claude/settings.json`
/// of `{}` makes `detect_harnesses` select claude, and a regular `AGENTS.md` —
/// never a link onto `CLAUDE.md` — selects codex. No `--agent` is passed: an
/// explicit agent is a directive that skips detection, and the point of the
/// fixture is that `regenerate` re-derives the same set from these same committed
/// bytes.
///
/// `.gitignore` gains `/.codex` (appended, never clobbered) before the install: the
/// managed block the install splices in has no such entry, and the codex config is
/// regenerated, so it must never be tracked.
pub(crate) fn install_into(root: &std::path::Path) {
    plant(root, ".claude/settings.json", "{}\n");
    plant(root, "AGENTS.md", "# Agents\n");
    let ignore = root.join(".gitignore");
    let prior = std::fs::read_to_string(&ignore).unwrap_or_default();
    std::fs::write(&ignore, format!("{prior}/.codex\n")).expect("append /.codex ignore");

    let out = install(root);
    assert!(
        out.status.success(),
        "install in {} failed: {}",
        root.display(),
        String::from_utf8_lossy(&out.stderr)
    );

    let mut add = vec!["add", "--"];
    add.extend(
        INSTALL_TRACKED
            .iter()
            .copied()
            .filter(|rel| root.join(*rel).exists()),
    );
    git(root, &add);
    git(root, &["commit", "-q", "-m", "installed"]);
}

/// A repo carrying a committed install — the SOURCE a fork is provisioned from.
///
/// Returns the canonical root, as `root::find` and `regenerate` will see it.
/// Proves the fork inherits a tracked `installed-version`, so regeneration runs
/// its legs instead of taking the uninstalled-source skip.
pub(crate) fn installed_source(dir: &std::path::Path) -> std::path::PathBuf {
    init_repo(dir);
    install_into(dir);
    std::fs::canonicalize(dir).expect("canonicalize installed source")
}

/// Add a linked sibling worktree of `source` at `fork`, on branch `feat`.
///
/// Sibling is meant literally: `worktree provision` verifies `<fork>` sits beside
/// the source (VT-6) and refuses anything else, so the caller owns that layout.
/// One shape, because every provision fixture needs a worktree git itself made —
/// a directory standing in for one is not a fork.
pub(crate) fn add_sibling_worktree(source: &std::path::Path, fork: &std::path::Path) {
    git(
        source,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "feat",
            fork.to_str().expect("utf8 fork path"),
        ],
    );
}

/// A detached worktree of `src`'s HEAD: the fork's **tracked** inputs and nothing
/// else — no derived tier, no copied allowlisted file.
///
/// This is the projection-fidelity baseline (PHASE-03 EX-1): an `install` here
/// starts from the same inputs `regenerate` saw inside the fork, so the two
/// projections must agree. A real `.git` is required — install reads tracked
/// paths — and a detached worktree is how git produces that tree exactly. The
/// caller removes the worktree; the returned `TempDir` owns the directory.
pub(crate) fn fresh_inputs(src: &std::path::Path) -> (tempfile::TempDir, std::path::PathBuf) {
    let holder = tempfile::tempdir().expect("tempdir for fresh inputs");
    let dest = holder.path().join("fresh");
    let head = git(src, &["rev-parse", "HEAD"]);
    git(
        src,
        &[
            "worktree",
            "add",
            "-q",
            "--detach",
            dest.to_str().expect("utf8 fresh path"),
            &head,
        ],
    );
    (holder, dest)
}
