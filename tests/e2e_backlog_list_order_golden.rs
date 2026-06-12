//! SL-051 PHASE-02 — the FOLDED list ordering as BLACK-BOX CLI goldens.
//!
//! PHASE-01 retired the standalone `backlog order` verb and folded the composed
//! `needs`/`after` work order into `backlog list` as a DEFAULT-ON comparator
//! (`--by sequence`), with `--by id` as the classic `(kind.ordinal, id)` opt-out.
//! This module is the rename/replacement of the removed `e2e_backlog_order_golden.rs`:
//! it pins the new surface byte-exact at the BUILT binary
//! (`mem.pattern.testing.black-box-cli-golden`), asserting EVERY surface — stdout
//! AND stderr AND exit code — not just the JSON envelope
//! (`mem.pattern.testing.conformance-asserts-surface-not-just-envelope`).
//!
//! Determinism: every fixture is hand-seeded authored TOML with FIXED dates (never
//! `backlog new`, which would stamp `clock::today()`). Ordering is EDGE-driven —
//! the goldens carry explicit `needs`/`after` edges so the composed sequence is
//! unambiguous, not merely the unconstrained tie-break `(exposure, created, ItemId)`
//! (whose `ItemId` Ord is by PREFIX STRING — `IMP` < `ISS` — so a tie-broken golden
//! would interleave kinds by letter; here the edges decide).
//!
//! Idiom: rides the `e2e_list_columns_golden.rs:96` seeder shape (the full backlog
//! authored stem with a `[relationships]` table) extended with the `needs`/`after`
//! item→item axes, and the `e2e_priority_golden.rs:89` `run`/`stdout`/`stderr` spawn
//! helpers over `env!("CARGO_BIN_EXE_doctrine")`.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

/// Seed one backlog item: `.doctrine/backlog/<kind>/NNN/backlog-NNN.{toml,md}`.
/// `rels` is spliced into the `[relationships]` table verbatim — the caller supplies
/// the `needs = [...]` / `after = [{ to = "X" }]` item→item edges that drive the
/// composed sequence (empty string = an edge-free node). FIXED dates keep the
/// unconstrained tie-break deterministic across runs.
fn seed(root: &Path, kind: &str, id: u32, slug: &str, title: &str, status: &str, rels: &str) {
    let dir = root.join(format!(".doctrine/backlog/{kind}/{id:03}"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("backlog-{id:03}.toml")),
        format!(
            "schema = \"doctrine.backlog\"\n\
             version = 1\n\
             id = {id}\n\
             slug = \"{slug}\"\n\
             title = \"{title}\"\n\
             kind = \"{kind}\"\n\
             status = \"{status}\"\n\
             resolution = \"\"\n\
             created = \"2026-01-02\"\n\
             updated = \"2026-01-02\"\n\
             tags = []\n\
             \n\
             [relationships]\n\
             slices = []\n\
             specs = []\n\
             drift = []\n\
             {rels}"
        ),
    )
    .unwrap();
    fs::write(
        dir.join(format!("backlog-{id:03}.md")),
        format!("# {title}\n"),
    )
    .unwrap();
}

/// `doctrine backlog list <extra...> -p <root>` over the built binary.
fn list(root: &Path, extra: &[&str]) -> Output {
    Command::new(BIN)
        .arg("backlog")
        .arg("list")
        .args(extra)
        .arg("-p")
        .arg(root)
        .output()
        .expect("spawn doctrine")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}
fn ok(out: &Output) {
    assert!(out.status.success(), "stderr: {}", stderr(out));
}

/// The id column of a rendered table body (drops the header + any `overrides:`
/// footer) — used for the membership-set invariant (VT-3).
fn id_set(body: &str) -> BTreeSet<String> {
    let table = body.split("\noverrides:").next().unwrap_or(body);
    table
        .lines()
        .skip(1) // header row
        .filter_map(|l| l.split_whitespace().next())
        .filter(|tok| !tok.is_empty())
        .map(str::to_string)
        .collect()
}

// ===========================================================================
// The edge-driven chain corpus (shared by VT-1/2/3/8): a hard `needs` chain
// ISS-003 ← ISS-002 ← ISS-001 (so 003 precedes 002 precedes 001) plus IMP-005
// soft-`after` ISS-001 (so it tails the chain). EDGE-driven ⇒ the composed order
// is `ISS-003, ISS-002, ISS-001, IMP-005`, which is NEITHER id-order nor the
// prefix-string tie-break — the difference IS the test.
// ===========================================================================
fn seed_chain(root: &Path) {
    seed(
        root,
        "issue",
        1,
        "a",
        "Aye",
        "open",
        "needs = [\"ISS-002\"]\n",
    );
    seed(
        root,
        "issue",
        2,
        "b",
        "Bee",
        "open",
        "needs = [\"ISS-003\"]\n",
    );
    seed(root, "issue", 3, "c", "Cee", "open", "");
    seed(
        root,
        "improvement",
        5,
        "e",
        "Eee",
        "open",
        "after = [{ to = \"ISS-001\" }]\n",
    );
}

// === VT-1 — default-on: the composed `needs`/`after` sequence =============

/// `backlog list` (no flag) = `--by sequence` (default): the edge-driven work
/// order, NOT id-order. Clean survey ⇒ no `overrides:` footer; empty stderr; exit 0.
#[test]
fn vt1_default_emits_the_composed_sequence() {
    let dir = tmp();
    seed_chain(dir.path());
    let out = list(dir.path(), &[]);
    ok(&out);
    assert_eq!(
        stdout(&out),
        "id       kind         status  title\n\
         ISS-003  issue        open    Cee\n\
         ISS-002  issue        open    Bee\n\
         ISS-001  issue        open    Aye\n\
         IMP-005  improvement  open    Eee\n"
    );
    assert_eq!(stderr(&out), "", "clean survey: no advisory on stderr");
}

// === VT-2 — opt-out: `--by id` restores the classic sort ==================

/// `backlog list --by id` = the classic `(kind.ordinal, id)` grouping: issue
/// (ordinal 0) before improvement (ordinal 1), ascending id within a kind. This is
/// DIFFERENT from the default sequence order above — the edges are ignored.
#[test]
fn vt2_by_id_restores_the_classic_kind_then_id_sort() {
    let dir = tmp();
    seed_chain(dir.path());
    let out = list(dir.path(), &["--by", "id"]);
    ok(&out);
    assert_eq!(
        stdout(&out),
        "id       kind         status  title\n\
         ISS-001  issue        open    Aye\n\
         ISS-002  issue        open    Bee\n\
         ISS-003  issue        open    Cee\n\
         IMP-005  improvement  open    Eee\n"
    );
}

// === VT-3 — membership invariant (A-2): set-equal, order-different ========

/// The row SET of the default `list` == the row set of `list --by id` over the same
/// fixture — only ORDER differs. The ordering comparator never filters (A-2): it
/// reorders the identical membership. Asserts BOTH halves: set-equality AND that the
/// rendered orders genuinely differ (otherwise the invariant is vacuous here).
#[test]
fn vt3_membership_is_set_equal_only_order_differs() {
    let dir = tmp();
    seed_chain(dir.path());
    let seq = list(dir.path(), &[]);
    let by_id = list(dir.path(), &["--by", "id"]);
    ok(&seq);
    ok(&by_id);
    assert_eq!(
        id_set(&stdout(&seq)),
        id_set(&stdout(&by_id)),
        "same membership under both orderings"
    );
    assert_ne!(
        stdout(&seq),
        stdout(&by_id),
        "the chain fixture must render in a DIFFERENT order under each"
    );
}

// === VT-4 — filtered compose: survivors keep their GLOBAL relative order ===

/// `backlog list --status open --kind improvement` orders the retained subset by
/// GLOBAL position. The global sequence is `ISS-001, IMP-007, IMP-003` (IMP-007
/// needs ISS-001; IMP-003 needs IMP-007); filtering to the two improvements keeps
/// their global relative order — `IMP-007` then `IMP-003`. That is the REVERSE of
/// `--by id` (`IMP-003` then `IMP-007`), so the contrast proves global-position
/// retention, not an incidental id sort.
#[test]
fn vt4_filtered_subset_keeps_global_position() {
    let dir = tmp();
    seed(dir.path(), "issue", 1, "i1", "Issue one", "open", "");
    seed(
        dir.path(),
        "improvement",
        7,
        "m7",
        "Imp seven",
        "open",
        "needs = [\"ISS-001\"]\n",
    );
    seed(
        dir.path(),
        "improvement",
        3,
        "m3",
        "Imp three",
        "open",
        "needs = [\"IMP-007\"]\n",
    );

    let out = list(dir.path(), &["--status", "open", "--kind", "improvement"]);
    ok(&out);
    assert_eq!(
        stdout(&out),
        "id       kind         status  title\n\
         IMP-007  improvement  open    Imp seven\n\
         IMP-003  improvement  open    Imp three\n",
        "survivors keep global sequence order (IMP-007 before IMP-003)"
    );

    // Contrast: `--by id` reverses them (ascending id, edges ignored).
    let by_id = list(
        dir.path(),
        &["--by", "id", "--status", "open", "--kind", "improvement"],
    );
    ok(&by_id);
    let by_id_body = stdout(&by_id);
    let ids: Vec<&str> = by_id_body
        .lines()
        .skip(1)
        .filter_map(|l| l.split_whitespace().next())
        .collect();
    assert_eq!(ids, vec!["IMP-003", "IMP-007"], "id sort is the reverse");
}

// === VT-5 — footer conditionality: absent clean, present (stdout) on a drop =

/// A clean survey (no dropped edge) emits NO `overrides:` footer.
#[test]
fn vt5_footer_absent_on_a_clean_survey() {
    let dir = tmp();
    seed(
        dir.path(),
        "issue",
        1,
        "a",
        "Aye",
        "open",
        "needs = [\"ISS-002\"]\n",
    );
    seed(dir.path(), "issue", 2, "b", "Bee", "open", "");
    let out = list(dir.path(), &[]);
    ok(&out);
    assert_eq!(
        stdout(&out),
        "id       kind   status  title\n\
         ISS-002  issue  open    Bee\n\
         ISS-001  issue  open    Aye\n"
    );
    assert!(
        !stdout(&out).contains("overrides:"),
        "clean survey: no footer"
    );
}

/// A soft `after` edge that CONTRADICTS a hard `needs` is dropped; the `overrides:`
/// honest-record footer prints on STDOUT (table mode), below the rows. Here ISS-001
/// `needs ISS-002` (hard: 002 first) while ISS-002 `after ISS-001` (soft: 001 first)
/// — the hard edge wins, the soft edge is dropped with the `contradicts a need`
/// reason. The `→` is the literal U+2192 arrow. Empty stderr (table mode).
#[test]
fn vt5_footer_present_on_stdout_when_a_soft_edge_is_dropped() {
    let dir = tmp();
    seed(
        dir.path(),
        "issue",
        1,
        "a",
        "Aye",
        "open",
        "needs = [\"ISS-002\"]\n",
    );
    seed(
        dir.path(),
        "issue",
        2,
        "b",
        "Bee",
        "open",
        "after = [{ to = \"ISS-001\" }]\n",
    );
    let out = list(dir.path(), &[]);
    ok(&out);
    assert_eq!(
        stdout(&out),
        "id       kind   status  title\n\
         ISS-002  issue  open    Bee\n\
         ISS-001  issue  open    Aye\n\
         \n\
         overrides:\n\
         \x20\x20ISS-001 \u{2192} ISS-002 dropped (contradicts a need)\n"
    );
    assert_eq!(stderr(&out), "", "table mode: footer on stdout, not stderr");
}

// === VT-6 — cycle degrade: id-sort + stderr warning + exit 0 ==============

/// A `needs` dependency cycle ⇒ the id-sorted table on STDOUT (never empty) + the
/// cycle advisory on STDERR + exit 0 (never non-zero). This SUPERSEDES the deleted
/// file's line-167 hard-error golden. The warning carries NO trailing newline (it is
/// the tail of the diagnostic stream).
#[test]
fn vt6_needs_cycle_degrades_to_id_sort_with_a_warning_and_exit_zero() {
    let dir = tmp();
    seed(
        dir.path(),
        "issue",
        1,
        "a",
        "Aye",
        "open",
        "needs = [\"ISS-002\"]\n",
    );
    seed(
        dir.path(),
        "issue",
        2,
        "b",
        "Bee",
        "open",
        "needs = [\"ISS-001\"]\n",
    );
    let out = list(dir.path(), &[]);
    ok(&out); // exit 0 — the degrade never fails the command
    assert_eq!(
        stdout(&out),
        "id       kind   status  title\n\
         ISS-001  issue  open    Aye\n\
         ISS-002  issue  open    Bee\n",
        "degrade to id-sort on stdout, never empty"
    );
    assert_eq!(
        stderr(&out),
        "backlog list: `needs` dependency cycle — ISS-001, ISS-002 \
         — ordering by id (resolve, then re-run)",
        "the advisory on stderr (no trailing newline)"
    );
}

// === VT-9 — off-sequence tail: a terminal row sorts AFTER the live chain ===

/// The Sequence comparator tails any retained row with NO composed position via
/// `unwrap_or(usize::MAX)` (`src/backlog.rs` `list_rows`). A row gets no `pos` when
/// it is TERMINAL — `project` filters terminal items out of the ordering graph — so
/// the only way to reveal such a row in Sequence mode is `--all`/`--status`. No other
/// test drives that branch.
///
/// Fixture: a live chain ISS-006 `needs` ISS-005 (composes to ISS-005, ISS-006) plus a
/// CLOSED, edge-free ISS-001 (terminal ⇒ no `pos`, hidden by default, revealed by
/// `--all`). The discriminator is the LOWEST id sitting LAST: the composed order is
/// `ISS-005, ISS-006, ISS-001` — ISS-001 tails despite id 1, which only holds while
/// the sentinel is `usize::MAX`. Invert it (e.g. `0`) and ISS-001 sorts to the front.
#[test]
fn vt9_terminal_row_tails_the_live_chain_under_all() {
    let dir = tmp();
    seed(
        dir.path(),
        "issue",
        6,
        "f",
        "Eff",
        "open",
        "needs = [\"ISS-005\"]\n",
    );
    seed(dir.path(), "issue", 5, "e", "Eee", "open", "");
    // Closed ⇒ terminal: excluded from the ordering graph, so no composed `pos`.
    seed(dir.path(), "issue", 1, "a", "Aye", "closed", "");
    let out = list(dir.path(), &["--all"]);
    ok(&out);
    assert_eq!(
        stdout(&out),
        "id       kind   status  title\n\
         ISS-005  issue  open    Eee\n\
         ISS-006  issue  open    Eff\n\
         ISS-001  issue  closed  Aye\n",
        "the terminal ISS-001 tails the live chain despite the lowest id"
    );
    assert_eq!(stderr(&out), "", "clean survey: no advisory on stderr");
}

// === VT-7 — verb retired: `backlog order` is an unknown subcommand ========

/// `backlog order` is now an unknown-subcommand clap error: non-zero exit, the clap
/// `unrecognized subcommand` surface on stderr, nothing on stdout. (The compute moved
/// into `list --by sequence`; the verb is gone.)
#[test]
fn vt7_backlog_order_is_an_unknown_subcommand() {
    let dir = tmp();
    let out = Command::new(BIN)
        .arg("backlog")
        .arg("order")
        .arg("-p")
        .arg(dir.path())
        .output()
        .expect("spawn doctrine");
    assert!(!out.status.success(), "retired verb must exit non-zero");
    assert_eq!(stdout(&out), "", "no rows for a retired verb");
    let err = stderr(&out);
    assert!(
        err.contains("unrecognized subcommand 'order'"),
        "clap unknown-subcommand surface: {err}"
    );
    assert!(
        !err.contains("panic"),
        "clean clap error, not a panic: {err}"
    );
}

// === VT-8 — JSON sequence: array order == composed sequence, shape unchanged =

/// `backlog list --json` over the chain corpus: the array order EQUALS the composed
/// sequence (`ISS-003, ISS-002, ISS-001, IMP-005`), and the `{kind, rows}` envelope
/// SHAPE is unchanged from PHASE-00 — every row carries `id/kind/resolution/slug/
/// status/title`, keys alpha-sorted, NO trailing newline, and NO `overrides` key
/// (the diagnostic stays OUT of the envelope). Clean survey ⇒ empty stderr.
#[test]
fn vt8_json_array_order_is_the_composed_sequence_envelope_unchanged() {
    let dir = tmp();
    seed_chain(dir.path());
    let out = list(dir.path(), &["--json"]);
    ok(&out);
    assert_eq!(
        stdout(&out),
        "{\n  \"kind\": \"backlog\",\n  \"rows\": [\n    \
         {\n      \"id\": \"ISS-003\",\n      \"kind\": \"issue\",\n      \
         \"resolution\": null,\n      \"slug\": \"c\",\n      \"status\": \"open\",\n      \
         \"title\": \"Cee\"\n    },\n    \
         {\n      \"id\": \"ISS-002\",\n      \"kind\": \"issue\",\n      \
         \"resolution\": null,\n      \"slug\": \"b\",\n      \"status\": \"open\",\n      \
         \"title\": \"Bee\"\n    },\n    \
         {\n      \"id\": \"ISS-001\",\n      \"kind\": \"issue\",\n      \
         \"resolution\": null,\n      \"slug\": \"a\",\n      \"status\": \"open\",\n      \
         \"title\": \"Aye\"\n    },\n    \
         {\n      \"id\": \"IMP-005\",\n      \"kind\": \"improvement\",\n      \
         \"resolution\": null,\n      \"slug\": \"e\",\n      \"status\": \"open\",\n      \
         \"title\": \"Eee\"\n    }\n  ]\n}"
    );
    assert!(
        !stdout(&out).contains("overrides"),
        "the honest-record stays out of the JSON envelope"
    );
    assert_eq!(stderr(&out), "", "clean survey: empty stderr in JSON mode");
}

/// JSON mode with a DROPPED edge: the envelope on stdout stays clean (no `overrides`
/// key, rows in composed sequence), and the advisory footer routes to STDERR — the
/// diagnostic-routing surface a JSON-envelope-only golden would miss.
#[test]
fn vt8_json_diagnostic_routes_to_stderr_envelope_stays_clean() {
    let dir = tmp();
    seed(
        dir.path(),
        "issue",
        1,
        "a",
        "Aye",
        "open",
        "needs = [\"ISS-002\"]\n",
    );
    seed(
        dir.path(),
        "issue",
        2,
        "b",
        "Bee",
        "open",
        "after = [{ to = \"ISS-001\" }]\n",
    );
    let out = list(dir.path(), &["--json"]);
    ok(&out);
    let body = stdout(&out);
    let v: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
    assert_eq!(v["kind"], "backlog");
    assert!(v.get("overrides").is_none(), "no overrides key in envelope");
    let ids: Vec<&str> = v["rows"]
        .as_array()
        .expect("rows array")
        .iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec!["ISS-002", "ISS-001"], "composed sequence in JSON");
    assert_eq!(
        stderr(&out),
        "\noverrides:\n  ISS-001 \u{2192} ISS-002 dropped (contradicts a need)\n",
        "the honest-record advisory routes to stderr under --json"
    );
}
