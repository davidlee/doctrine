//! SL-039 PHASE-03 — `backlog order` / `backlog needs` black-box goldens.
//!
//! Pins the CLI surface of the composed-order view and the `needs` set-verb refuse
//! byte-exact (mem.pattern.testing.black-box-cli-golden): the composed order + the
//! honest-record `overrides:` block (VT-6/VT-7), the `needs` dependency-cycle hard
//! error (stderr + non-zero exit, NO order printed — EX-3/VT-5), the soft `after`
//! cycle eviction (the strictly lower-`rank` edge dropped — VT-6 distinguishes the
//! genuine `(rank,age,src,dst)` key from the retired `(src,dst)` stand-in), and the
//! terminal/absent drop named with status+resolution (VT-7 honest record).
//!
//! Idiom (no new test dep): rides the `e2e_list_columns_golden.rs` pattern — spawn
//! `env!("CARGO_BIN_EXE_doctrine")` over a hand-seeded `tempfile::tempdir()` authored
//! tree with FIXED dates (never `backlog new`, which stamps `clock::today()`), and
//! `assert_eq!` byte-exact stdout/stderr.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
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

/// `doctrine backlog order -p <root>`.
fn order(root: &Path) -> Output {
    Command::new(BIN)
        .args(["backlog", "order", "-p"])
        .arg(root)
        .output()
        .expect("spawn doctrine")
}

/// `doctrine backlog needs <id> <prereqs...> -p <root>`.
fn needs(root: &Path, id: &str, prereqs: &[&str]) -> Output {
    Command::new(BIN)
        .args(["backlog", "needs", id])
        .args(prereqs)
        .arg("-p")
        .arg(root)
        .output()
        .expect("spawn doctrine")
}

/// Seed one backlog item with explicit `[relationships]` axes (FIXED dates).
struct Item<'a> {
    kind: &'a str,
    id: u32,
    slug: &'a str,
    title: &'a str,
    status: &'a str,
    resolution: &'a str,
    needs: &'a [&'a str],
    after: &'a [(&'a str, i32)],
}

fn seed(root: &Path, it: &Item<'_>) {
    let dir = root.join(format!(".doctrine/backlog/{}/{:03}", it.kind, it.id));
    fs::create_dir_all(&dir).unwrap();
    let needs = it
        .needs
        .iter()
        .map(|r| format!("\"{r}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let after = it
        .after
        .iter()
        .map(|(to, rank)| format!("{{ to = \"{to}\", rank = {rank} }}"))
        .collect::<Vec<_>>()
        .join(", ");
    fs::write(
        dir.join(format!("backlog-{:03}.toml", it.id)),
        format!(
            "id = {id}\n\
             slug = \"{slug}\"\n\
             title = \"{title}\"\n\
             kind = \"{kind}\"\n\
             status = \"{status}\"\n\
             resolution = \"{resolution}\"\n\
             created = \"2026-01-01\"\n\
             updated = \"2026-01-01\"\n\
             tags = []\n\
             \n\
             [relationships]\n\
             slices = []\n\
             specs = []\n\
             drift = []\n\
             needs = [{needs}]\n\
             after = [{after}]\n\
             triggers = []\n",
            id = it.id,
            slug = it.slug,
            title = it.title,
            kind = it.kind,
            status = it.status,
            resolution = it.resolution,
        ),
    )
    .unwrap();
    fs::write(
        dir.join(format!("backlog-{:03}.md", it.id)),
        format!("# {}\n", it.title),
    )
    .unwrap();
}

// ===========================================================================
// EX-3 / VT-5: a `needs` dependency cycle is a HARD error — non-zero exit, members
// on stderr, NO order table printed.
// ===========================================================================

#[test]
fn order_on_a_needs_cycle_errors_on_stderr_with_no_table() {
    let dir = tmp();
    let root = dir.path();
    seed(
        root,
        &Item {
            kind: "issue",
            id: 1,
            slug: "a",
            title: "A",
            status: "open",
            resolution: "",
            needs: &["ISS-002"],
            after: &[],
        },
    );
    seed(
        root,
        &Item {
            kind: "issue",
            id: 2,
            slug: "b",
            title: "B",
            status: "open",
            resolution: "",
            needs: &["ISS-001"],
            after: &[],
        },
    );

    let out = order(root);
    assert!(!out.status.success(), "a needs cycle exits non-zero");
    assert_eq!(stdout(&out), "", "no misleading order is printed");
    assert_eq!(
        stderr(&out),
        "Error: `backlog order` cannot compose: a `needs` dependency cycle — ISS-001, ISS-002 (resolve it, then re-run)\n"
    );
}

// ===========================================================================
// VT-5: the `needs` SET verb refuses a closing cycle, naming members, nothing written.
// ===========================================================================

#[test]
fn needs_set_verb_refuses_a_closing_cycle_naming_members() {
    let dir = tmp();
    let root = dir.path();
    // A.needs=[B] already; `needs B A` would close the {A,B} cycle.
    seed(
        root,
        &Item {
            kind: "issue",
            id: 1,
            slug: "a",
            title: "A",
            status: "open",
            resolution: "",
            needs: &["ISS-002"],
            after: &[],
        },
    );
    seed(
        root,
        &Item {
            kind: "issue",
            id: 2,
            slug: "b",
            title: "B",
            status: "open",
            resolution: "",
            needs: &[],
            after: &[],
        },
    );

    let before =
        fs::read_to_string(root.join(".doctrine/backlog/issue/002/backlog-002.toml")).unwrap();

    let out = needs(root, "ISS-002", &["ISS-001"]);
    assert!(!out.status.success(), "the closing edge is refused");
    assert_eq!(stdout(&out), "", "no confirm line on refuse");
    assert_eq!(
        stderr(&out),
        "Error: `backlog needs` would close a dependency cycle: ISS-001, ISS-002 (nothing written)\n"
    );
    assert_eq!(
        before,
        fs::read_to_string(root.join(".doctrine/backlog/issue/002/backlog-002.toml")).unwrap(),
        "nothing written on refuse"
    );
}

// ===========================================================================
// VT-6: a soft `after` cycle — the globally-minimal (rank,age,src,dst) edge (the
// strictly lower-`rank` one) is evicted; the order is STILL printed, the eviction
// recorded in the overrides block.
// ===========================================================================

#[test]
fn order_evicts_the_lower_rank_edge_of_a_soft_cycle() {
    let dir = tmp();
    let root = dir.path();
    // X(ISS-001).after=[{to=Y,rank=1}], Y(ISS-002).after=[{to=X,rank=5}].
    seed(
        root,
        &Item {
            kind: "issue",
            id: 1,
            slug: "x",
            title: "X item",
            status: "open",
            resolution: "",
            needs: &[],
            after: &[("ISS-002", 1)],
        },
    );
    seed(
        root,
        &Item {
            kind: "issue",
            id: 2,
            slug: "y",
            title: "Y item",
            status: "open",
            resolution: "",
            needs: &[],
            after: &[("ISS-001", 5)],
        },
    );

    let out = order(root);
    ok(&out);
    // The rank-1 edge (ISS-002 → ISS-001) is the strictly lower-rank one and is the
    // one evicted; the order is still produced.
    assert_eq!(
        stdout(&out),
        "id       kind   status  title\n\
         ISS-001  issue  open    X item\n\
         ISS-002  issue  open    Y item\n\
         \n\
         overrides:\n  \
         ISS-002 → ISS-001 dropped (soft cycle)\n"
    );
}

// ===========================================================================
// VT-7: a `needs` whose prereq is TERMINAL (closed/wont-do) and one ABSENT — both
// dropped + recorded with status+resolution / absent; the live (non-terminal) node
// still orders; cross-kind dep honoured.
// ===========================================================================

#[test]
fn order_records_terminal_and_absent_drops_with_status_and_resolution() {
    let dir = tmp();
    let root = dir.path();
    // ISS-001 needs CHR-001 (terminal closed/wont-do) AND ISS-099 (absent).
    seed(
        root,
        &Item {
            kind: "issue",
            id: 1,
            slug: "a",
            title: "A",
            status: "open",
            resolution: "",
            needs: &["CHR-001", "ISS-099"],
            after: &[],
        },
    );
    seed(
        root,
        &Item {
            kind: "chore",
            id: 1,
            slug: "gone",
            title: "Gone chore",
            status: "closed",
            resolution: "wont-do",
            needs: &[],
            after: &[],
        },
    );

    let out = order(root);
    ok(&out);
    assert_eq!(
        stdout(&out),
        "id       kind   status  title\n\
         ISS-001  issue  open    A\n\
         \n\
         overrides:\n  \
         CHR-001 → ISS-001 dropped (dangling: CHR-001 closed/wont-do)\n  \
         ISS-099 → ISS-001 dropped (dangling: ISS-099 absent)\n"
    );
}

// ===========================================================================
// Happy path: a clean hard-needs order with no drops prints the table alone (no
// overrides block) — the honest record is silent when nothing was dropped.
// ===========================================================================

#[test]
fn order_prints_the_composed_order_with_no_overrides_block_when_clean() {
    let dir = tmp();
    let root = dir.path();
    // ISS-001 needs ISS-002 ⇒ ISS-002 precedes ISS-001; nothing dropped.
    seed(
        root,
        &Item {
            kind: "issue",
            id: 1,
            slug: "a",
            title: "A",
            status: "open",
            resolution: "",
            needs: &["ISS-002"],
            after: &[],
        },
    );
    seed(
        root,
        &Item {
            kind: "issue",
            id: 2,
            slug: "b",
            title: "B",
            status: "open",
            resolution: "",
            needs: &[],
            after: &[],
        },
    );

    let out = order(root);
    ok(&out);
    assert_eq!(
        stdout(&out),
        "id       kind   status  title\n\
         ISS-002  issue  open    B\n\
         ISS-001  issue  open    A\n"
    );
}
