//! SL-025 PHASE-05 VT-2 — `memory new` is a command alias of `memory record`.
//!
//! Both surface names dispatch the IDENTICAL handler (`memory::run_record`), so a
//! memory minted via `memory new` is structurally identical to one minted via
//! `memory record`. The alias is a clap `visible_alias`, unreachable from a unit
//! test — proven here against the built binary (the backlog A-7 alias precedent).

#![allow(
    clippy::expect_used,
    clippy::tests_outside_test_module,
    reason = "integration test: `expect` is the idiomatic fail-fast, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::process::Command;

mod common;

fn bin() -> std::path::PathBuf {
    common::doctrine_bin()
}

/// Run `doctrine <args…>` rooted at `dir`, asserting success; return stdout.
fn doctrine(dir: &Path, args: &[&str]) -> String {
    let out = Command::new(bin())
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

/// Read the single `memory.toml` for `uid`, with the volatile axes stripped, so two
/// records made the same way compare equal regardless of uid / timestamp / anchor.
fn normalised_toml(dir: &Path, uid: &str) -> String {
    let path = dir
        .join(".doctrine/memory/items")
        .join(uid)
        .join("memory.toml");
    let raw = std::fs::read_to_string(&path).expect("read memory.toml");
    raw.lines()
        .filter(|l| {
            let k = l.split('=').next().unwrap_or("").trim();
            !matches!(
                k,
                "memory_uid" | "created" | "updated" | "commit" | "checkout_state_id"
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
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
fn memory_new_is_an_alias_of_memory_record_and_creates_an_identical_entity() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();

    // identical args, the only difference is the verb name.
    let common = &[
        "--type",
        "pattern",
        "--summary",
        "one liner",
        "--tag",
        "cli",
    ];

    let via_record = doctrine(
        dir,
        &[&["memory", "record", "Alias proof"], common.as_slice()].concat(),
    );
    let via_new = doctrine(
        dir,
        &[&["memory", "new", "Alias proof"], common.as_slice()].concat(),
    );

    let uid_record = parse_uid(&via_record);
    let uid_new = parse_uid(&via_new);
    assert_ne!(uid_record, uid_new, "each call mints a fresh uid");

    // both resolve via `show` (so both are valid, parseable entities).
    let shown_record = doctrine(dir, &["memory", "show", &uid_record]);
    let shown_new = doctrine(dir, &["memory", "show", &uid_new]);
    assert!(shown_record.contains("Alias proof"));
    assert!(shown_new.contains("Alias proof"));

    // and the persisted entities are byte-identical once the volatile axes (uid,
    // timestamps) are stripped — same handler, same scaffold.
    assert_eq!(
        normalised_toml(dir, &uid_record),
        normalised_toml(dir, &uid_new),
        "memory new and memory record produce the identical entity shape"
    );
}
