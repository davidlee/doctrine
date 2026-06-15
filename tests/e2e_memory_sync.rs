//! SL-018 PHASE-03 EX-4/EX-5 — end-to-end over the built binary.
//!
//! Drives the real `doctrine` executable through the corpus-sync surface in temp
//! dirs: the populate-from-embed reach (PHASE-05 — the embed now carries the real
//! orientation corpus), the no-root clean no-op (Charge XI), the `memory sync
//! install` hook wiring (a SEPARATE `SessionStart` entry coexisting with `boot
//! install`'s, OQ-E), and the client gitignore denylist via the full installer.

#![allow(
    clippy::expect_used,
    clippy::tests_outside_test_module,
    reason = "integration test: `expect` is the idiomatic fail-fast, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");

/// Run `doctrine <args…>` rooted at `cwd`, returning (success, stdout). Does NOT
/// assert success — the no-root case must exit 0 too, but callers verify intent.
fn run(cwd: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(BIN)
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("spawn doctrine");
    (
        out.status.success(),
        String::from_utf8(out.stdout).expect("utf8 stdout"),
    )
}

/// A doctrine repo is anything `root::find` resolves — a `.git` marker suffices.
fn doctrine_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir(dir.path().join(".git")).expect("mark repo");
    dir
}

#[test]
fn sync_populates_the_shipped_corpus_then_is_idempotent_and_retrievable() {
    // PHASE-05: the embed now carries the real orientation corpus, so an in-repo
    // sync lands every master under shipped/ (gitignored), a re-sync is inert, and
    // a shipped master surfaces through `retrieve` on its scope — the end-to-end
    // reach over the built binary. The `mem.<key>` alias symlinks beside each uid
    // dir are NOT shipped as duplicates (gather_assets admits canonical uids only).
    let repo = doctrine_repo();

    let (ok, stdout) = run(repo.path(), &["memory", "sync", "-y", "-p", &path(&repo)]);
    assert!(ok, "in-repo sync must exit 0: {stdout}");
    assert!(
        stdout.contains(" new, 0 changed") && !stdout.contains("0 new,"),
        "the populated embed must plan writes: {stdout}"
    );
    let shipped = repo.path().join(".doctrine/memory/shipped");
    assert!(shipped.is_dir(), "sync must create shipped/");
    let masters: Vec<_> = std::fs::read_dir(&shipped)
        .expect("read shipped/")
        .filter_map(|e| {
            e.ok()
                .map(|e| e.file_name().into_string().unwrap_or_default())
        })
        .collect();
    assert!(
        masters.len() >= 12,
        "the corpus must ship ≥12 masters (OQ-A skeleton), got {}: {masters:?}",
        masters.len()
    );
    assert!(
        masters.iter().all(|n| n.starts_with("mem_")),
        "only canonical uid dirs ship — no `mem.<key>` alias duplicates: {masters:?}"
    );

    // Re-sync is inert (idempotent) — identical embed vs disk plans no writes.
    let (ok, stdout) = run(repo.path(), &["memory", "sync", "-y", "-p", &path(&repo)]);
    assert!(ok, "re-sync must exit 0: {stdout}");
    assert!(
        stdout.contains("0 new, 0 changed"),
        "a re-sync of the identical corpus must be inert: {stdout}"
    );

    // A shipped master surfaces through retrieve on its command scope.
    let (ok, stdout) = run(
        repo.path(),
        &[
            "memory",
            "retrieve",
            "-p",
            &path(&repo),
            "--command",
            "doctrine",
        ],
    );
    assert!(ok, "retrieve must exit 0: {stdout}");
    assert!(
        stdout.contains("mem.signpost.doctrine.overview")
            && stdout.contains("staleness: reference"),
        "a shipped master must surface via its scope with non-decaying staleness: {stdout}"
    );
}

#[test]
fn sync_outside_a_doctrine_repo_writes_nothing() {
    // `root::find` walks CWD up to `/`, so a true no-root needs an ancestry with
    // zero markers — the default temp base may itself sit under a stray repo. Pick
    // a base whose chain to `/` is marker-free so this exercises the Charge XI
    // branch deterministically rather than an incidental empty-embed no-op.
    let base = marker_free_base();
    let bare = tempfile::Builder::new()
        .tempdir_in(&base)
        .expect("tempdir in marker-free base");
    let (ok, stdout) = run(bare.path(), &["memory", "sync"]);
    assert!(
        ok,
        "no-root sync must exit 0 (the M1 hook is harmless): {stdout}"
    );
    assert!(
        stdout.contains("Not in a doctrine repo"),
        "no-root sync must announce the no-op: {stdout}"
    );
    assert!(
        !bare.path().join(".doctrine").exists(),
        "no-root sync must not write anything"
    );
}

/// The first temp base whose ancestry to `/` carries no root marker, so a tempdir
/// under it resolves to no doctrine root. Panics if every candidate is polluted —
/// a loud, honest failure beats a silently mis-targeted assertion.
fn marker_free_base() -> std::path::PathBuf {
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

#[test]
fn dry_run_prints_the_plan_without_writing() {
    let repo = doctrine_repo();
    let (ok, stdout) = run(
        repo.path(),
        &["memory", "sync", "--dry-run", "-p", &path(&repo)],
    );
    assert!(ok, "{stdout}");
    assert!(
        stdout.contains("[dry-run]"),
        "dry-run must tag its output: {stdout}"
    );
    assert!(!repo.path().join(".doctrine/memory/shipped").exists());
}

#[test]
fn sync_install_wires_a_separate_session_hook_coexisting_with_boot() {
    let repo = doctrine_repo();
    let settings = repo.path().join(".claude/settings.local.json");

    // boot install first (claude harness explicit — a bare repo auto-detects none).
    let (ok, out) = run(
        repo.path(),
        &[
            "boot",
            "install",
            "-p",
            &path(&repo),
            "--agent",
            "claude",
            "-y",
        ],
    );
    assert!(ok, "boot install: {out}");

    // then sync install — a SEPARATE SessionStart entry.
    let (ok, out) = run(
        repo.path(),
        &["memory", "sync", "install", "-p", &path(&repo), "-y"],
    );
    assert!(ok, "sync install: {out}");

    let json = std::fs::read_to_string(&settings).expect("settings written");
    assert!(json.contains(" boot\""), "boot hook present: {json}");
    assert!(
        json.contains(" memory sync\""),
        "sync hook present as a distinct command: {json}"
    );

    // re-running sync install is idempotent — no second sync entry.
    let (ok, _) = run(
        repo.path(),
        &["memory", "sync", "install", "-p", &path(&repo), "-y"],
    );
    assert!(ok);
    let json = std::fs::read_to_string(&settings).expect("settings");
    assert_eq!(
        json.matches("memory sync\"").count(),
        1,
        "sync hook must not duplicate on re-run: {json}"
    );
}

#[test]
fn full_install_gitignores_the_shipped_corpus() {
    let repo = doctrine_repo();
    let (ok, out) = run(repo.path(), &["install", "-p", &path(&repo), "-y"]);
    assert!(ok, "install: {out}");
    let gitignore = std::fs::read_to_string(repo.path().join(".gitignore")).expect("gitignore");
    assert!(
        gitignore.contains(".doctrine/memory/shipped/"),
        "the client denylist must ignore the shipped corpus: {gitignore}"
    );
}

/// The repo path as a `&str` arg (tempdirs are UTF-8 here).
fn path(dir: &tempfile::TempDir) -> String {
    dir.path().to_str().expect("utf8 path").to_owned()
}

// ===========================================================================
// SL-069 PHASE-04 — integration: embed, sync, retrieval surface
// ===========================================================================

/// VT-1 (SL-069): sync from clean state materialises exactly 27 INV-signatured
/// shipped dirs under `.doctrine/memory/shipped/`.
#[test]
fn sync_produces_exactly_27_shipped_dirs() {
    let repo = doctrine_repo();
    let (ok, _) = run(repo.path(), &["memory", "sync", "-y", "-p", &path(&repo)]);
    assert!(ok, "sync must exit 0");
    let shipped = repo.path().join(".doctrine/memory/shipped");
    assert!(shipped.is_dir(), "sync must create shipped/");
    let masters: Vec<_> = std::fs::read_dir(&shipped)
        .expect("read shipped/")
        .filter_map(|e| {
            e.ok()
                .map(|e| e.file_name().into_string().unwrap_or_default())
        })
        .filter(|n| n.starts_with("mem_"))
        .collect();
    assert_eq!(
        masters.len(),
        27,
        "SL-069 corpus must ship exactly 27 INV dirs, got {}: {masters:?}",
        masters.len()
    );
}

/// VT-2 (SL-069): each of the 13 new shipped memories is retrievable via `memory
/// find --path-scope` by its exact key, and carries the ADR-002 shipped signature
/// (`repo=""`, `anchor_kind=none`) in its `memory show` output.
#[test]
fn each_new_shipped_memory_finds_by_key_and_has_shipped_signature() {
    let repo = doctrine_repo();
    let (ok, _) = run(repo.path(), &["memory", "sync", "-y", "-p", &path(&repo)]);
    assert!(ok, "sync must exit 0");

    // The 13 new shipped memories from PHASE-01, each with a scope path/command
    // that `memory find` will match.
    let new_memories: &[(&str, &[&str])] = &[
        (
            "mem.signpost.doctrine.install",
            &["--command", "doctrine install"],
        ),
        (
            "mem.concept.doctrine.boot-snapshot",
            &["--command", "doctrine boot"],
        ),
        (
            "mem.concept.doctrine.reading-entities",
            &["--command", "doctrine slice"],
        ),
        (
            "mem.signpost.doctrine.reference-docs",
            &["--path-scope", ".doctrine/using-doctrine.md"],
        ),
        (
            "mem.signpost.doctrine.relating-entities",
            &["--command", "doctrine link"],
        ),
        (
            "mem.signpost.doctrine.recording-memories",
            &["--command", "doctrine memory record"],
        ),
        (
            "mem.signpost.doctrine.backlog",
            &["--command", "doctrine backlog"],
        ),
        ("mem.signpost.doctrine.adrs", &["--command", "doctrine adr"]),
        (
            "mem.signpost.doctrine.specs",
            &["--command", "doctrine spec"],
        ),
        (
            "mem.signpost.doctrine.requirements",
            &["--command", "doctrine coverage"],
        ),
        (
            "mem.signpost.doctrine.audit",
            &["--command", "doctrine review"],
        ),
        (
            "mem.signpost.doctrine.revisions",
            &["--command", "doctrine revision"],
        ),
        (
            "mem.signpost.doctrine.policies-standards",
            &["--command", "doctrine policy"],
        ),
    ];

    let p = path(&repo);
    let shipped = repo.path().join(".doctrine/memory/shipped");

    // Build a uid→key map from the shipped corpus TOML files.
    let mut uid_by_key: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::new();
    for entry in std::fs::read_dir(&shipped).expect("read shipped/") {
        let entry = entry.expect("entry");
        let name = entry.file_name().into_string().unwrap_or_default();
        if !name.starts_with("mem_") {
            continue;
        }
        let toml_path = shipped.join(&name).join("memory.toml");
        let toml_text = std::fs::read_to_string(&toml_path).expect("read memory.toml");
        // Extract the key line.
        for line in toml_text.lines() {
            if let Some(key) = line.strip_prefix("memory_key = \"") {
                let key = key.strip_suffix('"').unwrap_or(key);
                uid_by_key.insert(key.to_string(), name.clone());
                break;
            }
        }
    }

    for (key, _scope_args) in new_memories {
        let uid = uid_by_key
            .get(*key)
            .unwrap_or_else(|| panic!("key {key} not found in shipped corpus"));

        // Read the shipped TOML to verify ADR-002 signature.
        let toml_path = shipped.join(uid).join("memory.toml");
        let toml_text = std::fs::read_to_string(&toml_path)
            .unwrap_or_else(|e| panic!("read {toml_path:?}: {e}"));
        assert!(
            toml_text.contains("repo = \"\""),
            "{key} must have empty repo in shipped TOML"
        );
        assert!(
            toml_text.contains("anchor_kind = \"none\""),
            "{key} must have anchor_kind=none in shipped TOML"
        );

        // Verify findable via `memory find` by UID query.
        let (ok, stdout) = run(repo.path(), &["memory", "find", "-p", &p, "--query", uid]);
        assert!(ok, "find for {key} (uid {uid}) must exit 0: {stdout}");
        assert!(
            stdout.contains(uid),
            "find for {key} must return uid {uid}:\n{stdout}"
        );
    }
}
