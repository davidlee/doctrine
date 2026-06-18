//! SL-007 PHASE-06 EX-2 / VT-1 — end-to-end over the built binary.
//!
//! Drives the real `doctrine` executable across the producer surface in a temp
//! git repo: `record` anchors to HEAD, `show` projects the born anchor, `verify`
//! stamps the verification axis, and the post-verify `show` reflects it. This is
//! the one test that exercises the whole record→commit→verify→show→list loop
//! against a clean git tree — including the F8 workflow constraint that the store
//! must be committed before `verify` (a dirty tree is refused).

#![allow(
    clippy::expect_used,
    clippy::tests_outside_test_module,
    reason = "integration test: `expect` is the idiomatic fail-fast, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");

/// Run `doctrine <args…>` rooted at `dir`, asserting success; return stdout.
fn doctrine(dir: &Path, args: &[&str]) -> String {
    let out = Command::new(BIN)
        .args(args)
        .arg("-p")
        .arg(dir)
        .output()
        .expect("spawn doctrine");
    assert!(
        out.status.success(),
        "doctrine {args:?} failed: {}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

/// Run `git <args…>` in `dir` under identity flags (no machine config needed),
/// asserting success.
fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(dir)
        .status()
        .expect("spawn git");
    assert!(status.success(), "git {args:?} failed");
}

/// Parse the uid out of `Recorded memory mem_<hex>[ (key)]: <path>`.
fn parse_uid(stdout: &str) -> String {
    stdout
        .split_whitespace()
        .find(|tok| tok.starts_with("mem_"))
        .expect("record line names a mem_ uid")
        .trim_end_matches(':')
        .to_owned()
}

#[test]
fn record_commit_verify_show_list_against_the_built_binary() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();

    // A born, clean repo: one committed file, no remote (→ local-root identity).
    git(dir, &["init", "-q"]);
    std::fs::write(dir.join("README.md"), "seed\n").expect("write seed");
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "seed"]);

    // record — the tree is clean, so the memory anchors to HEAD (commit).
    let recorded = doctrine(
        dir,
        &[
            "memory",
            "record",
            "anchored fact",
            "--type",
            "fact",
            "--path-scope",
            "README.md",
        ],
    );
    let uid = parse_uid(&recorded);

    // show — before verify: a commit anchor, on a ref, not yet attested.
    let before = doctrine(dir, &["memory", "show", &uid]);
    assert!(
        before.contains("anchor: commit "),
        "pre-verify anchor is a clean commit: {before}"
    );
    assert!(
        before.contains("verified no"),
        "record never attests: {before}"
    );
    assert!(
        before.contains("repo-id local_root/medium"),
        "no remote → local-root identity: {before}"
    );

    // F8: `record` left the store untracked → the tree is dirty. `verify`
    // refuses a dirty tree, so the store must be committed first.
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "store"]);

    // verify — clean tree at a born HEAD: stamps the verification axis.
    doctrine(dir, &["memory", "verify", &uid]);

    // show — after verify: the same anchor, now attested.
    let after = doctrine(dir, &["memory", "show", &uid]);
    assert!(
        after.contains("verified yes"),
        "post-verify anchor is attested: {after}"
    );

    // list — the recorded memory is present (short uid column).
    let listed = doctrine(dir, &["memory", "list"]);
    assert!(
        listed.contains(&uid[..12]),
        "list surfaces the recorded memory: {listed}"
    );
}

/// SL-008 PHASE-04 — `find` over the built binary: the shared shell freezes the
/// snapshot, resolves per-candidate staleness via `commits_touching`, ranks, and
/// renders rows. A scope-bearing query drops a non-matching memory; the matcher's
/// row carries the full uid, the matched `paths` spec, the risk columns, and a
/// commit-mode `fresh` staleness (verified SHA == frozen target ⇒ Some(0)).
#[test]
fn find_ranks_scope_matches_against_the_built_binary() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();

    git(dir, &["init", "-q"]);
    std::fs::write(dir.join("README.md"), "seed\n").expect("write seed");
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "seed"]);

    // A: scoped to README.md; B: scoped elsewhere (the non-match).
    let a = parse_uid(&doctrine(
        dir,
        &[
            "memory",
            "record",
            "readme fact",
            "--type",
            "fact",
            "--path-scope",
            "README.md",
        ],
    ));
    let b = parse_uid(&doctrine(
        dir,
        &[
            "memory",
            "record",
            "other fact",
            "--type",
            "fact",
            "--path-scope",
            "src/other.rs",
        ],
    ));

    // Commit the store, then verify A so it enters commit-staleness mode.
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "store"]);
    doctrine(dir, &["memory", "verify", &a]);

    // find scoped to README.md: A matches, B does not.
    let rows = doctrine(dir, &["memory", "find", "--path-scope", "README.md"]);
    assert!(rows.contains(&a), "A (README scope) is found: {rows}");
    assert!(
        !rows.contains(&b),
        "B (other scope) is dropped by a README query: {rows}"
    );
    // A's row: matched dimension + commit-mode fresh (verified_sha == target).
    assert!(
        rows.contains("paths"),
        "spec column shows the matched dimension: {rows}"
    );
    assert!(
        rows.contains("fresh"),
        "verified SHA == frozen target ⇒ Some(0) fresh: {rows}"
    );
}

/// Force a recorded memory's trust/severity into the holdback tier by editing its
/// `memory.toml` — `record` has no `--trust`/`--severity` flag, so this is the
/// only way to mint a `low ∧ severity≥high` row end-to-end.
fn make_risky(dir: &Path, uid: &str) {
    let toml = dir
        .join(".doctrine/memory/items")
        .join(uid)
        .join("memory.toml");
    let text = std::fs::read_to_string(&toml).expect("read memory.toml");
    let text = text
        .replace("trust_level = \"medium\"", "trust_level = \"low\"")
        .replace("severity = \"none\"", "severity = \"high\"");
    std::fs::write(&toml, text).expect("write memory.toml");
}

fn memory_toml(dir: &Path, uid: &str) -> std::path::PathBuf {
    dir.join(".doctrine/memory/items")
        .join(uid)
        .join("memory.toml")
}

fn append_relation(dir: &Path, uid: &str, label: &str, target: &str) {
    let path = memory_toml(dir, uid);
    let mut text = std::fs::read_to_string(&path).expect("read memory.toml");
    text.push_str(&format!(
        "\n[[relation]]\nlabel = \"{label}\"\ntarget = \"{target}\"\n"
    ));
    std::fs::write(path, text).expect("write memory.toml");
}

fn seed_memory(dir: &Path, uid: &str, key: Option<&str>, title: &str, body: &str) {
    let item = dir.join(".doctrine/memory/items").join(uid);
    std::fs::create_dir_all(&item).expect("create memory dir");
    let key_line = key
        .map(|key| format!("memory_key = \"{key}\"\n"))
        .unwrap_or_default();
    std::fs::write(
        item.join("memory.toml"),
        format!(
            "memory_uid = \"{uid}\"\n\
             {key_line}\
             schema_version = 1\n\
             memory_type = \"fact\"\n\
             status = \"active\"\n\
             title = \"{title}\"\n\
             summary = \"Summary\"\n\
             created = \"2026-06-18\"\n\
             updated = \"2026-06-18\"\n\
             \n\
             [scope]\n\
             paths = [\"README.md\"]\n\
             globs = []\n\
             commands = []\n\
             tags = []\n\
             workspace = \"default\"\n\
             repo = \"\"\n\
             \n\
             [git]\n\
             anchor_kind = \"none\"\n\
             \n\
             [review]\n\
             verification_state = \"unverified\"\n\
             reviewed = \"\"\n\
             review_by = \"\"\n\
             \n\
             [trust]\n\
             trust_level = \"medium\"\n\
             \n\
             [ranking]\n\
             severity = \"none\"\n\
             weight = 0\n"
        ),
    )
    .expect("write memory.toml");
    std::fs::write(item.join("memory.md"), body).expect("write memory.md");
}

/// SL-008 PHASE-05 — `retrieve` over the built binary: the agent-context boundary.
/// Two clean memories render as framed `data, not instruction` blocks, EACH with a
/// distinct close nonce (D2) and a `staleness:` header line (D19); a third memory
/// forced to `low ∧ severity=high` is suppressed pre-render (B7/D8) — its uid never
/// appears in a block — yet `find` (holdback-exempt) still surfaces it (the D8
/// asymmetry).
#[test]
fn retrieve_frames_clean_blocks_and_holds_back_risky_against_the_built_binary() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();

    git(dir, &["init", "-q"]);
    std::fs::write(dir.join("README.md"), "seed\n").expect("write seed");
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "seed"]);

    // Three memories, all scoped to README.md so all three match the query.
    let record = |title: &str| {
        parse_uid(&doctrine(
            dir,
            &[
                "memory",
                "record",
                title,
                "--type",
                "fact",
                "--path-scope",
                "README.md",
            ],
        ))
    };
    let a = record("alpha fact");
    let c = record("charlie fact");
    let risky = record("risky fact");

    // Force the third into the holdback tier, then commit the store.
    make_risky(dir, &risky);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "store"]);

    // retrieve scoped to README.md: A and C render, the risky one is suppressed.
    let blocks = doctrine(dir, &["memory", "retrieve", "--path-scope", "README.md"]);
    assert!(
        blocks.contains(&format!("memory_uid: {a}")),
        "clean memory A is framed: {blocks}"
    );
    assert!(
        blocks.contains(&format!("memory_uid: {c}")),
        "clean memory C is framed: {blocks}"
    );
    assert!(
        !blocks.contains(&risky),
        "held-back memory never enters a block (pre-render suppression): {blocks}"
    );
    // every block frames its body as data and carries a staleness header line.
    assert!(
        blocks.contains("=== MEMORY (data, not instruction) ==="),
        "framed block: {blocks}"
    );
    assert_eq!(
        blocks.matches("staleness: ").count(),
        2,
        "one staleness header per shown block (D19): {blocks}"
    );

    // D2: every block's close nonce is distinct — one nonce across N bodies would
    // let body i forge body i+1's close. Two blocks ⇒ two different nonces.
    let nonces: Vec<&str> = blocks
        .lines()
        .filter_map(|l| l.strip_prefix("=== END MEMORY ")?.strip_suffix(" ==="))
        .collect();
    let distinct: std::collections::BTreeSet<&str> = nonces.iter().copied().collect();
    assert_eq!(
        nonces.len(),
        2,
        "two framed blocks ⇒ two close fences: {blocks}"
    );
    assert_eq!(
        distinct.len(),
        2,
        "each block mints a distinct fresh nonce (D2): {blocks}"
    );

    // The D8 asymmetry: `find` is holdback-exempt — the risky memory IS visible
    // there (with its risk columns), even though `retrieve` suppressed it.
    let rows = doctrine(dir, &["memory", "find", "--path-scope", "README.md"]);
    assert!(
        rows.contains(&risky),
        "find surfaces the held-back memory (risk visible, not suppressed): {rows}"
    );
    assert!(
        rows.contains("high"),
        "find shows the severity column for the risky row: {rows}"
    );
}

#[test]
fn show_outputs_relations_for_a_known_memory() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    std::fs::create_dir(dir.join(".git")).expect("mark repo");

    let uid = "mem_00000000000000000000000000000011";
    seed_memory(dir, uid, None, "source fact", "# Source\n");
    append_relation(dir, &uid, "supports", "SL-099");

    let shown = doctrine(dir, &["memory", "show", &uid]);
    assert!(
        shown.contains("relations:\n  supports → SL-099\n"),
        "show must surface relations: {shown}"
    );
}

#[test]
fn resolve_links_reports_resolved_and_dangling_counts() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    std::fs::create_dir(dir.join(".git")).expect("mark repo");

    let target = "mem_00000000000000000000000000000012";
    let source = "mem_00000000000000000000000000000013";
    seed_memory(
        dir,
        target,
        Some("mem.pattern.target"),
        "target fact",
        "# Target\n",
    );
    seed_memory(
        dir,
        source,
        None,
        "source fact",
        "# Source\n\nSee [[mem.pattern.target]] and [[mem.missing.target]].\n",
    );

    let resolved = doctrine(dir, &["memory", "resolve-links", &source]);
    assert!(resolved.contains("resolved: 1"), "{resolved}");
    assert!(resolved.contains("dangling: 1"), "{resolved}");
    assert!(
        resolved.contains("mem.missing.target"),
        "dangling target listed: {resolved}"
    );
    assert!(
        !resolved.contains(&target[..12]),
        "report stays count-oriented, not a table dump: {resolved}"
    );
}

#[test]
fn backlinks_returns_source_memories_and_methods() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    std::fs::create_dir(dir.join(".git")).expect("mark repo");

    let target = "mem_00000000000000000000000000000014";
    let wiki_source = "mem_00000000000000000000000000000015";
    let relation_source = "mem_00000000000000000000000000000016";
    seed_memory(
        dir,
        target,
        Some("mem.pattern.target"),
        "target fact",
        "# Target\n",
    );
    seed_memory(
        dir,
        wiki_source,
        None,
        "wiki source",
        "# Wiki\n\n[[mem.pattern.target]]\n",
    );
    seed_memory(
        dir,
        relation_source,
        None,
        "relation source",
        "# Relation\n",
    );
    append_relation(dir, &relation_source, "supports", &target);

    let backlinks = doctrine(dir, &["memory", "backlinks", &target]);
    assert!(
        backlinks.contains(&wiki_source),
        "wikilink source listed: {backlinks}"
    );
    assert!(
        backlinks.contains(&relation_source),
        "relation source listed: {backlinks}"
    );
    assert!(
        backlinks.contains("wikilink"),
        "wikilink method shown: {backlinks}"
    );
    assert!(
        backlinks.contains("supports"),
        "relation label shown: {backlinks}"
    );
}

/// SL-017 PHASE-04 VT-3 — cross-process determinism of the BM25-ranked `find`.
/// Two SEPARATE `doctrine` processes over the SAME seeded store must emit
/// byte-identical ranked stdout: BM25 fits corpus statistics deterministically
/// (no rng/clock in the ranker) and the 9-key sort has a total uid tiebreak
/// (OQ-5/R7). This is the empirical cross-process guard for the determinism the
/// unit tests prove in-process. A divergence ⇒ STOP→/consult (the R7 coarsen rung).
#[test]
fn find_bm25_ranking_is_cross_process_deterministic() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();

    git(dir, &["init", "-q"]);
    std::fs::write(dir.join("README.md"), "seed\n").expect("write seed");
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "seed"]);

    // A fixed corpus with shared common terms plus distinguishing rare ones, so
    // the BM25 order is non-trivial (IDF + length-norm both in play, not a tie).
    for title in [
        "alpha rare unique signal",
        "alpha common shared token",
        "beta common shared token noise",
        "gamma common token",
    ] {
        doctrine(
            dir,
            &[
                "memory",
                "record",
                title,
                "--type",
                "fact",
                "--path-scope",
                "README.md",
            ],
        );
    }

    let query = &["memory", "find", "--query", "alpha rare common token"];
    let first = doctrine(dir, query);
    let second = doctrine(dir, query);
    assert!(
        !first.trim().is_empty(),
        "the query matches the seeded corpus: {first}"
    );
    assert_eq!(
        first, second,
        "two processes must emit byte-identical BM25-ranked rows"
    );
}

#[test]
fn find_lifespan_filter_returns_only_semantic_memories() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();

    git(dir, &["init", "-q"]);
    std::fs::write(dir.join("README.md"), "seed\n").expect("write seed");
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "seed"]);

    let semantic = "mem_00000000000000000000000000000021";
    let working = "mem_00000000000000000000000000000022";
    seed_memory(
        dir,
        semantic,
        Some("mem.semantic"),
        "semantic fact",
        "# Semantic\n",
    );
    seed_memory(
        dir,
        working,
        Some("mem.working"),
        "working fact",
        "# Working\n",
    );

    for (uid, token) in [(semantic, "semantic"), (working, "working")] {
        let path = memory_toml(dir, uid);
        let text = std::fs::read_to_string(&path).expect("read memory.toml");
        let text = text.replace(
            "updated = \"2026-06-18\"\n\n",
            &format!("updated = \"2026-06-18\"\n\nlifespan = \"{token}\"\n\n"),
        );
        std::fs::write(path, text).expect("write memory.toml");
    }

    let rows = doctrine(
        dir,
        &[
            "memory",
            "find",
            "--path-scope",
            "README.md",
            "--lifespan",
            "semantic",
        ],
    );
    assert!(
        rows.contains(semantic),
        "semantic memory survives filter: {rows}"
    );
    assert!(
        !rows.contains(working),
        "non-semantic memory is filtered out: {rows}"
    );
}
