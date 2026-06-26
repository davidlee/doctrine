//! SL-066 PHASE-05 — `revision approve` + `revision apply`, black-box over the built
//! binary (plan VT-1..VT-5).
//!
//! VT-1 — clean status-only apply: every `status` row lands via the requirement setter,
//! N RecDocs in one commit (the worktree carries N REC dirs), REC schema unchanged,
//! REV → `done`.
//! VT-2 — the approval checkpoint: apply refused when `approval != approved`; after
//! `revision approve`, apply proceeds.
//! VT-3 — the all-or-nothing from-guard: a `status` row whose `from` != current
//! `ReqStatus` aborts the WHOLE apply, surfaces the stale set, writes nothing (the
//! requirement toml + the REC tree are untouched).
//! VT-4 — terminal disposition (M1): a REV carrying surfaced-for-manual rows stays
//! `started` post-apply (status landed, manual list printed); a status-only REV reaches
//! `done`.
//! VT-5 — introduce/create/modify/move/prose rows are NOT auto-applied — surfaced-for-
//! manual only (the target is never mutated, no REC minted for them).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

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

    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.path.join(rel)).unwrap()
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

    /// Seed a requirement with an explicit current status (the from-capture source +
    /// the setter target).
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

    /// Count the REC dirs (numeric children under `.doctrine/rec`).
    fn rec_count(&self) -> usize {
        let rec_root = self.path.join(".doctrine/rec");
        if !rec_root.is_dir() {
            return 0;
        }
        std::fs::read_dir(&rec_root)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.path().is_dir() && e.file_name().to_string_lossy().parse::<u32>().is_ok())
            .count()
    }

    fn req_status(&self, id: u32) -> String {
        let toml = self.read(&format!(
            ".doctrine/requirement/{id:03}/requirement-{id:03}.toml"
        ));
        for line in toml.lines() {
            if let Some(rest) = line.trim().strip_prefix("status") {
                return rest
                    .trim_start_matches([' ', '=', '"'])
                    .trim_end_matches('"')
                    .trim()
                    .to_owned();
            }
        }
        panic!("no status in requirement {id}: {toml}");
    }

    fn rev_status(&self) -> String {
        let toml = self.read(".doctrine/revision/001/revision-001.toml");
        for line in toml.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("status") {
                if rest.trim_start().starts_with('=') {
                    return rest
                        .trim_start_matches([' ', '='])
                        .split('#')
                        .next()
                        .unwrap()
                        .trim()
                        .trim_matches('"')
                        .to_owned();
                }
            }
        }
        panic!("no status in revision: {toml}");
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

/// Seed a started REV with two `status` rows (REQ-201 active→retired, REQ-202
/// active→deprecated), approved and ready to apply.
fn seed_two_status_rev(repo: &Repo) {
    repo.seed_req(201, "active");
    repo.seed_req(202, "active");
    ok(&repo.run(&["revision", "new", "retire two reqs"]));
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
        "status",
        "--target",
        "REQ-202",
        "--to-status",
        "deprecated",
    ]));
    ok(&repo.run(&["revision", "status", "REV-001", "started"]));
}

/// VT-1: a clean status-only apply lands every `status` row via the setter, emits N
/// RecDocs in one commit (N REC dirs in the worktree), the REC schema is unchanged, and
/// the REV reaches `done` (dependents would unblock — `done` is terminal).
#[test]
fn clean_status_only_apply_lands_rows_emits_recs_and_reaches_done() {
    let repo = Repo::new();
    seed_two_status_rev(&repo);
    ok(&repo.run(&["revision", "approve", "REV-001"]));

    let out = ok(&repo.run(&["revision", "apply", "REV-001"]));

    // Both status rows landed via the setter.
    assert_eq!(repo.req_status(201), "retired", "REQ-201 landed: {out}");
    assert_eq!(repo.req_status(202), "deprecated", "REQ-202 landed: {out}");

    // N RecDocs — one per status row — in the worktree (the operator commits them all
    // in one commit; the grain is N status acts, each REC self-describing).
    assert_eq!(repo.rec_count(), 2, "one REC per status row: {out}");

    // The REC schema is unchanged: a populated rec carries `[[status_delta]]`.
    let rec_toml = repo.read(".doctrine/rec/001/rec-001.toml");
    assert!(
        rec_toml.contains("[[status_delta]]") && rec_toml.contains("[rec]"),
        "REC schema unchanged (status_delta + [rec]): {rec_toml}"
    );

    // A status-only REV reaches `done` (the dependents-unblock payoff).
    assert_eq!(repo.rev_status(), "done", "status-only REV → done: {out}");
}

/// VT-2: the apply-time checkpoint. Apply is refused while `approval != approved`; once
/// `revision approve` records the approval, apply proceeds.
#[test]
fn apply_refused_until_approved() {
    let repo = Repo::new();
    seed_two_status_rev(&repo);

    // Not approved yet — apply refused, nothing landed.
    let refused = repo.run(&["revision", "apply", "REV-001"]);
    assert!(
        !refused.status.success(),
        "apply refused while approval != approved"
    );
    assert!(
        String::from_utf8_lossy(&refused.stderr).contains("not approved"),
        "refusal names the missing approval: {}",
        String::from_utf8_lossy(&refused.stderr)
    );
    assert_eq!(
        repo.req_status(201),
        "active",
        "nothing landed pre-approval"
    );
    assert_eq!(repo.rec_count(), 0, "no REC minted pre-approval");

    // After approve, apply proceeds.
    ok(&repo.run(&["revision", "approve", "REV-001"]));
    ok(&repo.run(&["revision", "apply", "REV-001"]));
    assert_eq!(repo.req_status(201), "retired", "lands after approval");
}

/// VT-3: the pre-flight all-or-nothing from-guard. A `status` row whose captured `from`
/// no longer matches the target's current `ReqStatus` (the target moved since draft)
/// aborts the WHOLE apply — surfacing the stale set — and writes NOTHING (the other
/// status row's target is untouched, and no REC is minted).
#[test]
fn from_guard_aborts_whole_apply_and_writes_nothing() {
    let repo = Repo::new();
    seed_two_status_rev(&repo);
    ok(&repo.run(&["revision", "approve", "REV-001"]));

    // Simulate an intervening reconcile move: REQ-201 drifted from `active` (its captured
    // `from`) to `deprecated`. Apply must now abort the WHOLE thing.
    repo.seed_req(201, "deprecated");

    let aborted = repo.run(&["revision", "apply", "REV-001"]);
    assert!(
        !aborted.status.success(),
        "apply aborts on a stale from-guard"
    );
    let err = String::from_utf8_lossy(&aborted.stderr);
    assert!(
        err.contains("REQ-201") && err.contains("active") && err.contains("deprecated"),
        "surfaces the stale set (expected from vs actual): {err}"
    );

    // All-or-nothing: NOTHING written — REQ-202 (a clean row) is untouched, no REC minted,
    // and the drifted REQ-201 keeps its current value.
    assert_eq!(
        repo.req_status(202),
        "active",
        "the clean row's target is untouched (all-or-nothing)"
    );
    assert_eq!(
        repo.req_status(201),
        "deprecated",
        "the drifted target unchanged"
    );
    assert_eq!(repo.rec_count(), 0, "no REC minted on an aborted apply");
    assert_eq!(repo.rev_status(), "started", "REV not advanced on abort");
}

/// VT-4 / M1: terminal disposition. A REV carrying surfaced-for-manual rows stays
/// `started` after apply (the status row landed + the manual list printed); a
/// status-only REV reaches `done`. `done` never lies — it means every row landed.
#[test]
fn mixed_rev_stays_started_status_only_reaches_done() {
    let repo = Repo::new();
    repo.seed_adr(6);
    repo.seed_req(201, "active");
    ok(&repo.run(&["revision", "new", "mixed payload"]));
    // One status row (auto) + one prose modify row (surfaced-for-manual).
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
        "revision", "change", "add", "REV-001", "--action", "modify", "--target", "ADR-006",
    ]));
    ok(&repo.run(&["revision", "status", "REV-001", "started"]));
    ok(&repo.run(&["revision", "approve", "REV-001"]));

    let out = ok(&repo.run(&["revision", "apply", "REV-001"]));

    // The status row landed …
    assert_eq!(
        repo.req_status(201),
        "retired",
        "the status row landed: {out}"
    );
    assert_eq!(repo.rec_count(), 1, "one REC for the one status row");
    // … the manual row was surfaced (printed) …
    assert!(
        out.contains("surfaced for manual") && out.contains("modify") && out.contains("ADR-006"),
        "manual rows surfaced/listed at apply: {out}"
    );
    // … and the REV HELD at `started` (manual rows still outstanding — done would lie).
    assert_eq!(
        repo.rev_status(),
        "started",
        "a mixed REV stays started post-apply (M1): {out}"
    );
}

/// VT-5: introduce/create/modify/move/prose rows are NOT auto-applied — surfaced-for-
/// manual only. The target prose entity is never mutated and no REC is minted for them.
#[test]
fn non_status_rows_are_surfaced_not_applied() {
    let repo = Repo::new();
    repo.seed_adr(6);
    ok(&repo.run(&["revision", "new", "prose only"]));
    // A prose modify row — no engine seam in v1.
    ok(&repo.run(&[
        "revision", "change", "add", "REV-001", "--action", "modify", "--target", "ADR-006",
    ]));
    ok(&repo.run(&["revision", "status", "REV-001", "started"]));
    ok(&repo.run(&["revision", "approve", "REV-001"]));

    let adr_before = repo.read(".doctrine/adr/006/adr-006.toml");
    let out = ok(&repo.run(&["revision", "apply", "REV-001"]));

    // Surfaced-for-manual: listed, NOT auto-applied.
    assert!(
        out.contains("surfaced for manual") && out.contains("modify") && out.contains("ADR-006"),
        "prose row surfaced-for-manual: {out}"
    );
    // The target entity is untouched (no auto-apply seam) and no REC minted.
    assert_eq!(
        repo.read(".doctrine/adr/006/adr-006.toml"),
        adr_before,
        "the prose target is never mutated by apply"
    );
    assert_eq!(repo.rec_count(), 0, "no REC for a surfaced-for-manual row");
    // A prose-only REV (no status rows) holds at `started` — nothing auto-landed, the
    // manual remainder is the operator's (done would lie).
    assert_eq!(
        repo.rev_status(),
        "started",
        "prose-only REV stays started: {out}"
    );
}
