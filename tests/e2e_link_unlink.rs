// SPDX-License-Identifier: GPL-3.0-only
//! SL-048 PHASE-05 — `link`/`unlink` + forward & corpus validation as BLACK-BOX
//! goldens over the built binary (design §5.4/§5.5).
//!
//! Covers the plan's end-to-end acceptance (EX-1 / VT-5 / VT-6) plus the forward
//! kind-check refusal (VT-2) and the report-only corpus validate (VT-3/VT-4):
//! - `link SL-048 governed_by ADR-010` validates, persists ONE `[[relation]]` row,
//!   is surfaced by `slice show`, and appears in ADR-010's DERIVED inbound as
//!   "governs" via `inspect` (EX-1 / VT-5).
//! - `unlink` round-trips; a second `unlink` is an idempotent no-op (Absent); a
//!   second `link` of the same triple is idempotent (Noop) (VT-6).
//! - `link` refuses an illegal target KIND (`governed_by SL-003`, a slice not
//!   ADR·POL·STD) and a dangling target; a non-`Writable` label names its owning
//!   verb (VT-2 / EX-2).
//! - `validate` reports a `[[relation]]` dangler and is report-only — never rewrites.

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

/// Run the binary against the temp corpus. DOCTRINE_WORKER is explicitly UNSET — the
/// self-arm guard refuses authored writes (`link`/`unlink`) under it, and a stray
/// inherited var would spuriously red the round-trip (mem.pattern.dispatch.
/// worker-verify-unset-doctrine-worker).
fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .args(args)
        .arg("-p")
        .arg(root)
        .env_remove("DOCTRINE_WORKER")
        .output()
        .expect("spawn doctrine")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

/// Seed a minimal slice (no relations) — the migrated shape carries no
/// `[relationships]` table (slice dropped it entirely, OD-3 / PHASE-04).
fn seed_slice(root: &Path, id: u32) {
    let dir = root.join(format!(".doctrine/slice/{id:03}"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("slice-{id:03}.toml")),
        format!(
            "id = {id}\nslug = \"s\"\ntitle = \"fixture\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n"
        ),
    )
    .unwrap();
    fs::write(
        dir.join(format!("slice-{id:03}.md")),
        "# fixture\n\nbody.\n",
    )
    .unwrap();
}

/// Seed a minimal ADR — a valid `governed_by` target kind.
fn seed_adr(root: &Path, id: u32) {
    let dir = root.join(format!(".doctrine/adr/{id:03}"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("adr-{id:03}.toml")),
        format!(
            "id = {id}\nslug = \"a\"\ntitle = \"fixture adr\"\nstatus = \"accepted\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n"
        ),
    )
    .unwrap();
    fs::write(dir.join(format!("adr-{id:03}.md")), "# adr\n\nbody.\n").unwrap();
}

fn slice_toml(root: &Path, id: u32) -> String {
    fs::read_to_string(root.join(format!(".doctrine/slice/{id:03}/slice-{id:03}.toml"))).unwrap()
}

// --- EX-1 / VT-5: link persists + surfaces in show + derived inbound -------

#[test]
fn link_persists_shows_and_appears_in_derived_inbound() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 48);
    seed_adr(root, 10);

    // link SL-048 governed_by ADR-010
    let out = run(root, &["link", "SL-048", "governed_by", "ADR-010"]);
    assert!(out.status.success(), "link must succeed: {}", stderr(&out));
    assert!(
        stdout(&out).contains("linked"),
        "reports linked: {}",
        stdout(&out)
    );

    // (a) persisted as ONE [[relation]] row with both cells.
    let toml = slice_toml(root, 48);
    assert!(
        toml.contains("[[relation]]"),
        "wrote a relation array: {toml}"
    );
    assert!(toml.contains("label = \"governed_by\""), "{toml}");
    assert!(toml.contains("target = \"ADR-010\""), "{toml}");
    assert_eq!(
        toml.matches("[[relation]]").count(),
        1,
        "exactly one row: {toml}"
    );

    // (b) surfaced by `slice show` on the source.
    let show = run(root, &["slice", "show", "48"]);
    assert!(show.status.success(), "slice show: {}", stderr(&show));
    let st = stdout(&show);
    assert!(
        st.contains("governed_by") && st.contains("ADR-010"),
        "slice show surfaces the new edge: {st}"
    );

    // (c) appears in ADR-010's DERIVED inbound as "governs" (the inbound_name, X5).
    let inspect = run(root, &["inspect", "ADR-010"]);
    assert!(inspect.status.success(), "inspect: {}", stderr(&inspect));
    let it = stdout(&inspect);
    assert!(
        it.contains("governs") && it.contains("SL-048"),
        "ADR-010 derived inbound renders 'governs SL-048': {it}"
    );
}

// --- VT-6: unlink round-trip + double-unlink/double-link idempotency ------

#[test]
fn unlink_round_trips_and_both_directions_are_idempotent() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 48);
    seed_adr(root, 10);

    // A second identical link is a Noop (idempotent) — file gains no second row.
    assert!(
        run(root, &["link", "SL-048", "governed_by", "ADR-010"])
            .status
            .success()
    );
    let noop = run(root, &["link", "SL-048", "governed_by", "ADR-010"]);
    assert!(noop.status.success());
    assert!(
        stdout(&noop).contains("already linked"),
        "{}",
        stdout(&noop)
    );
    assert_eq!(
        slice_toml(root, 48).matches("[[relation]]").count(),
        1,
        "a re-link never adds a duplicate row"
    );

    // unlink removes the edge from storage, show, and the derived inbound.
    let un = run(root, &["unlink", "SL-048", "governed_by", "ADR-010"]);
    assert!(un.status.success(), "unlink: {}", stderr(&un));
    assert!(stdout(&un).contains("unlinked"), "{}", stdout(&un));
    assert!(
        !slice_toml(root, 48).contains("governed_by"),
        "the edge is gone from storage"
    );
    let inspect = run(root, &["inspect", "ADR-010"]);
    assert!(
        !stdout(&inspect).contains("SL-048"),
        "ADR-010 no longer shows the inbound: {}",
        stdout(&inspect)
    );

    // A second unlink is an idempotent no-op (Absent).
    let again = run(root, &["unlink", "SL-048", "governed_by", "ADR-010"]);
    assert!(
        again.status.success(),
        "double-unlink succeeds: {}",
        stderr(&again)
    );
    assert!(stdout(&again).contains("not linked"), "{}", stdout(&again));
}

// --- VT-2 / EX-2: forward & policy refusals -------------------------------

#[test]
fn link_refuses_illegal_target_kind_dangler_and_lifecycle_label() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 48);
    seed_slice(root, 3);
    seed_adr(root, 10);

    // (a) legal-KIND refusal: governed_by → ADR·POL·STD, so a slice target is refused
    // even though SL-003 RESOLVES (ensure_ref_resolves passes; the new kind assertion
    // catches it).
    let bad_kind = run(root, &["link", "SL-048", "governed_by", "SL-003"]);
    assert!(!bad_kind.status.success(), "illegal target kind must fail");
    assert!(
        stderr(&bad_kind).contains("ADR") || stderr(&bad_kind).contains("target must"),
        "names the legal target kinds: {}",
        stderr(&bad_kind)
    );
    assert!(
        !slice_toml(root, 48).contains("[[relation]]"),
        "a refused link writes nothing"
    );

    // (b) dangling target: governed_by → ADR-999 (no entity) is refused.
    let dangle = run(root, &["link", "SL-048", "governed_by", "ADR-999"]);
    assert!(!dangle.status.success(), "dangling target must fail");
    assert!(
        stderr(&dangle).contains("does not resolve"),
        "names the dangler: {}",
        stderr(&dangle)
    );

    // (c) LifecycleOnly label (governance supersedes) names its owning verb, never
    // plain link.
    let lifecycle = run(root, &["link", "ADR-010", "supersedes", "ADR-010"]);
    assert!(
        !lifecycle.status.success(),
        "supersedes is not link-writable"
    );
    assert!(
        stderr(&lifecycle).contains("supersede verb"),
        "names the owning verb: {}",
        stderr(&lifecycle)
    );
}

// --- VT-3: validate reports a [[relation]] dangler, report-only -----------

#[test]
fn validate_reports_relation_dangler_without_rewriting() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 48);
    seed_adr(root, 10);
    // Link, then DELETE the target out from under the edge to create a dangler.
    assert!(
        run(root, &["link", "SL-048", "governed_by", "ADR-010"])
            .status
            .success()
    );
    fs::remove_dir_all(root.join(".doctrine/adr/010")).unwrap();

    let before = slice_toml(root, 48);
    let val = run(root, &["validate"]);
    assert!(!val.status.success(), "a dangler forces a non-zero exit");
    let report = stdout(&val);
    assert!(
        report.contains("SL-048") && report.contains("ADR-010") && report.contains("dangling"),
        "validate reports the dangling edge: {report}"
    );
    // Report-only: the source TOML is byte-unchanged.
    assert_eq!(
        before,
        slice_toml(root, 48),
        "validate never rewrites the corpus"
    );
}
