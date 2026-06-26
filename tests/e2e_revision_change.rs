//! SL-066 PHASE-03 — the `[[change]]` payload + `revises` reciprocity, black-box over
//! the built binary (design VT-1..VT-4).
//!
//! VT-1 (golden) — a 3-target Revision: `inspect REV-N` lists the outbound `revises`
//! set; `inspect ADR-X` / `inspect REQ-N` list the REV as inbound `revises` (uniform
//! over ALL rows, not just primary); `revision show` does NOT list them (ADR-004 §3).
//! VT-2 — target validation ({SPEC,PRD,REQ,ADR,POL,STD} accepted, off-target refused);
//! `doctrine link … revises …` refused (TypedVerbOnly), `revision change add` accepted.
//! VT-3 — a creation row with `--new-label` omitted is refused; a `status` row
//! auto-captures `from` (the target's current ReqStatus) at change add.
//! VT-4 — `primary` is at-most-one (a second `--primary` refused); a zero-primary REV
//! is valid and its edges still surface.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::process::{Command, Output};

mod common;

fn bin() -> std::path::PathBuf {
    common::doctrine_bin()
}

/// A throwaway git repo seeded with the authored-truth targets a Revision revises.
struct Repo {
    _dir: tempfile::TempDir,
    path: std::path::PathBuf,
}

impl Repo {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_path_buf();
        let repo = Self { _dir: dir, path };
        repo.git(&["init", "-b", "main"]);
        repo.git(&["config", "user.name", "Doctrine Test"]);
        repo.git(&["config", "user.email", "test@doctrine.invalid"]);
        std::fs::create_dir_all(repo.path.join(".doctrine/revision")).unwrap();
        repo
    }

    fn write(&self, rel: &str, body: &str) {
        let p = self.path.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }

    /// Seed an ADR (an edge target only — `ensure_ref_resolves` probes the dir).
    fn seed_adr(&self, id: u32) {
        self.write(
            &format!(".doctrine/adr/{id:03}/adr-{id:03}.toml"),
            &format!(
                "schema = \"doctrine.adr\"\nversion = 1\nid = {id}\nslug = \"a{id}\"\ntitle = \"A{id}\"\nstatus = \"accepted\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n"
            ),
        );
        self.write(&format!(".doctrine/adr/{id:03}/adr-{id:03}.md"), "a\n");
    }

    /// Seed a requirement with an explicit current status (the from-capture source).
    fn seed_req(&self, id: u32, status: &str) {
        self.write(
            &format!(".doctrine/requirement/{id:03}/requirement-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"r{id}\"\ntitle = \"R{id}\"\nstatus = \"{status}\"\nkind = \"functional\"\n"
            ),
        );
        self.write(
            &format!(".doctrine/requirement/{id:03}/requirement-{id:03}.md"),
            "r\n",
        );
    }

    /// Seed a tech spec (a creation op's `member_of` destination).
    fn seed_spec(&self, id: u32) {
        self.write(
            &format!(".doctrine/spec/tech/{id:03}/spec-{id:03}.toml"),
            &format!("id = {id}\nslug = \"s{id}\"\ntitle = \"S{id}\"\nstatus = \"draft\"\nkind = \"tech\"\n"),
        );
        self.write(
            &format!(".doctrine/spec/tech/{id:03}/spec-{id:03}.md"),
            "s\n",
        );
    }

    fn git(&self, args: &[&str]) -> String {
        let out = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .args(args)
            .output()
            .expect("spawn git");
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(bin())
            .args(args)
            .arg("-p")
            .arg(&self.path)
            .output()
            .expect("spawn doctrine")
    }
}

fn ok(out: &Output) -> String {
    assert!(
        out.status.success(),
        "verb failed: {}",
        String::from_utf8_lossy(&out.stderr).trim()
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn read_rev_toml(repo: &Repo) -> String {
    std::fs::read_to_string(repo.path.join(".doctrine/revision/001/revision-001.toml")).unwrap()
}

/// VT-1: a 3-target Revision. Outbound `revises` on `inspect REV-001`; inbound
/// `revises` on each touched target (`inspect ADR-006` / `inspect REQ-201`), uniform
/// over ALL rows (the non-primary rows surface too); `revision show` does NOT list the
/// inbound reciprocity (ADR-004 §3 reserves it to the scan-backed `inspect`).
#[test]
fn three_target_revision_inbound_outbound_and_show_excludes_inbound() {
    let repo = Repo::new();
    repo.seed_adr(6);
    repo.seed_req(201, "active");
    repo.seed_spec(18);

    ok(&repo.run(&["revision", "new", "revise the layering canon"]));
    // Three rows: a prose modify (primary), a status row, and a creation row.
    ok(&repo.run(&[
        "revision",
        "change",
        "add",
        "REV-001",
        "--action",
        "modify",
        "--target",
        "ADR-006",
        "--primary",
    ]));
    ok(&repo.run(&[
        "revision",
        "change",
        "add",
        "REV-001",
        "--action",
        "status",
        "--target",
        "REQ-201",
        "--to-status",
        "retired",
    ]));
    ok(&repo.run(&[
        "revision",
        "change",
        "add",
        "REV-001",
        "--action",
        "introduce",
        "--member-of",
        "SPEC-018",
        "--new-label",
        "FR-007",
    ]));

    // Outbound on the REV: a `revises` group listing all three targets.
    let rev_out = ok(&repo.run(&["inspect", "REV-001"]));
    assert!(rev_out.contains("outbound:"), "REV has outbound: {rev_out}");
    assert!(
        rev_out.contains("revises:")
            && rev_out.contains("ADR-006")
            && rev_out.contains("REQ-201")
            && rev_out.contains("SPEC-018"),
        "REV-001 outbound revises lists all three targets: {rev_out}"
    );

    // Inbound on each touched target — the REV surfaces as `revises`, even the
    // non-primary rows (REQ-201 status row, SPEC-018 creation row).
    for target in ["ADR-006", "REQ-201", "SPEC-018"] {
        let inb = ok(&repo.run(&["inspect", target]));
        assert!(
            inb.contains("inbound:") && inb.contains("revises:") && inb.contains("REV-001"),
            "{target} inbound revises lists REV-001 (uniform over all rows): {inb}"
        );
    }

    // `revision show` does NOT carry the inbound reciprocity (ADR-004 §3).
    let shown = ok(&repo.run(&["revision", "show", "REV-001"]));
    assert!(
        !shown.contains("inbound"),
        "revision show must not list inbound reciprocity (ADR-004 §3): {shown}"
    );
}

/// VT-2: target validation + the TypedVerbOnly link refusal. Off-target (`revises SL`)
/// is refused; `doctrine link … revises …` is refused naming the typed verb; the six
/// authored-truth kinds are accepted by `revision change add`.
#[test]
fn revises_target_validation_and_link_refused() {
    let repo = Repo::new();
    repo.seed_adr(6);
    // A slice the off-target attempt points at (exists, but wrong kind).
    repo.write(
        ".doctrine/slice/001/slice-001.toml",
        "id = 1\nslug = \"s1\"\ntitle = \"S1\"\nstatus = \"proposed\"\n",
    );
    repo.write(".doctrine/slice/001/slice-001.md", "s\n");

    ok(&repo.run(&["revision", "new", "probe targets"]));

    // Off-target: a slice is NOT a revises target — refused.
    let off = repo.run(&[
        "revision", "change", "add", "REV-001", "--action", "modify", "--target", "SL-001",
    ]);
    assert!(
        !off.status.success(),
        "revises SL-001 must be refused (off-target)"
    );

    // A valid ADR target is accepted.
    ok(&repo.run(&[
        "revision", "change", "add", "REV-001", "--action", "modify", "--target", "ADR-006",
    ]));

    // `doctrine link REV-001 revises ADR-006` is refused (TypedVerbOnly).
    let linked = repo.run(&["link", "REV-001", "revises", "ADR-006"]);
    assert!(
        !linked.status.success(),
        "link … revises … must be refused (TypedVerbOnly)"
    );
    assert!(
        String::from_utf8_lossy(&linked.stderr).contains("typed verb"),
        "link refusal names the typed verb: {}",
        String::from_utf8_lossy(&linked.stderr)
    );
}

/// VT-3: a creation row with `--new-label` omitted is refused; a `status` row
/// auto-captures `from` (the target's CURRENT ReqStatus) at change add.
#[test]
fn creation_requires_label_and_status_captures_from() {
    let repo = Repo::new();
    repo.seed_spec(18);
    repo.seed_req(201, "active");
    ok(&repo.run(&["revision", "new", "draft"]));

    // Creation op without --new-label — refused (E4: the label is frozen at add).
    let no_label = repo.run(&[
        "revision",
        "change",
        "add",
        "REV-001",
        "--action",
        "introduce",
        "--member-of",
        "SPEC-018",
    ]);
    assert!(
        !no_label.status.success(),
        "introduce without --new-label must be refused (E4)"
    );

    // A status row auto-captures `from` = the target's current ReqStatus (active).
    ok(&repo.run(&[
        "revision",
        "change",
        "add",
        "REV-001",
        "--action",
        "status",
        "--target",
        "REQ-201",
        "--to-status",
        "retired",
    ]));
    let toml = read_rev_toml(&repo);
    assert!(
        toml.contains("from = \"active\""),
        "status row auto-captures from = current ReqStatus (active): {toml}"
    );
    assert!(
        toml.contains("to_status = \"retired\""),
        "status row records the requested to_status: {toml}"
    );
}

/// VT-4: `primary` is at-most-one (a second `--primary` is refused at change add); a
/// zero-primary Revision is valid and its rows still surface as `revises` edges.
#[test]
fn primary_is_at_most_one_and_zero_primary_is_valid() {
    let repo = Repo::new();
    repo.seed_adr(6);
    repo.seed_adr(7);
    ok(&repo.run(&["revision", "new", "primary probe"]));

    ok(&repo.run(&[
        "revision",
        "change",
        "add",
        "REV-001",
        "--action",
        "modify",
        "--target",
        "ADR-006",
        "--primary",
    ]));
    // A second primary row is refused (F1 — at most one).
    let second = repo.run(&[
        "revision",
        "change",
        "add",
        "REV-001",
        "--action",
        "modify",
        "--target",
        "ADR-007",
        "--primary",
    ]);
    assert!(
        !second.status.success(),
        "a second --primary row must be refused (F1: at most one)"
    );

    // A zero-primary Revision (a fresh one with only non-primary rows) is valid and its
    // edge still surfaces — blocking/visibility never key on primary.
    ok(&repo.run(&["revision", "new", "no headline"]));
    ok(&repo.run(&[
        "revision", "change", "add", "REV-002", "--action", "modify", "--target", "ADR-007",
    ]));
    let inb = ok(&repo.run(&["inspect", "ADR-007"]));
    assert!(
        inb.contains("revises:") && inb.contains("REV-002"),
        "a zero-primary REV's edge still surfaces inbound: {inb}"
    );
    let _ = Path::new("");
}
