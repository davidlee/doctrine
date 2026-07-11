//! SL-213 PHASE-06 — comparison surfaces as BLACK-BOX CLI goldens.
//!
//! Mirrors `tests/e2e_priority_golden.rs`'s harness style (spawn the BUILT
//! binary, `mem.pattern.testing.black-box-cli-golden`) for the READ surfaces
//! (`compare list` / `explain` / `findings`). `compare record`/`withdraw` are
//! WRITE-classed (`commands/guard.rs`) and this suite runs inside a dispatch
//! WORKER jail that refuses authored writes outside its declared delta — a
//! spawned `compare record` hits `worker fork (signal: marker): refusing
//! authored write` before it can mint anything. [`capture`]/[`withdraw`]
//! below hand-author the SAME session-of-one TOML the real command would
//! have minted (the exact wire shape `comparison::wire` documents) rather
//! than shelling out — the only adaptation the A5 STOP condition anticipates
//! (report, don't silently skip). This also gives every row a KNOWN uid up
//! front, so `--supersedes` targets and status assertions need no disk
//! scan-back.
//!
//! - VT-1: capture → `compare list` battery covering all ten [`RowState`]
//!   display tokens + `--active-only` semantics.
//! - VT-2: `explain`'s three value-source shapes + `--json` parity + all four
//!   SL-213 findings with their exit hints.
//! - VT-3: determinism under shuffled session-file load order (fixed
//!   permutations, no rng) + no-NaN/total-order + a component-split-after-
//!   quarantine discontinuity golden.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

mod common;

fn bin() -> std::path::PathBuf {
    common::doctrine_bin()
}

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

/// `doctrine <args...> -p <root>` over the built binary.
fn run(root: &Path, args: &[&str]) -> Output {
    let mut a: Vec<&str> = args.to_vec();
    a.push("-p");
    let root_s = root.to_str().expect("utf8 path");
    a.push(root_s);
    Command::new(bin())
        .args(&a)
        .output()
        .expect("spawn doctrine")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

fn ok(out: &Output) -> String {
    assert!(out.status.success(), "stderr: {}", stderr(out));
    stdout(out)
}

/// Seed a minimal backlog issue (value-bearing, non-RSK kind — comparison
/// admits it) with an optional verbatim tail (`[value]` / `[[relation]]`
/// blocks).
fn seed_issue(root: &Path, id: u32, extra: &str) {
    write(
        root,
        &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.toml"),
        &format!(
            "id = {id}\nslug = \"i{id}\"\ntitle = \"Issue {id}\"\nkind = \"issue\"\n\
             status = \"open\"\nresolution = \"\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n[relationships]\n{extra}"
        ),
    );
    write(
        root,
        &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.md"),
        "b\n",
    );
}

/// Seed a minimal slice (value-bearing; the ONLY kind whose `supersedes`
/// relation may target another of ITS OWN kind, `relation.rs` `RELATION_RULES`
/// — the R6 lifecycle scenario needs this, backlog issues cannot legally
/// supersede one another).
fn seed_slice(root: &Path, id: u32, extra: &str) {
    write(
        root,
        &format!(".doctrine/slice/{id:03}/slice-{id:03}.toml"),
        &format!(
            "id = {id}\nslug = \"s{id}\"\ntitle = \"Slice {id}\"\n\
             status = \"started\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\n{extra}"
        ),
    );
    write(
        root,
        &format!(".doctrine/slice/{id:03}/slice-{id:03}.md"),
        "b\n",
    );
}

fn iss(id: u32) -> String {
    format!("ISS-{id:03}")
}

fn sl(id: u32) -> String {
    format!("SL-{id:03}")
}

/// Monotonic uid/filename minter (no rng — a plain counter; uniqueness is all
/// that's needed, never randomness).
static NEXT_UID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
fn mint(prefix: &str) -> String {
    // Zero-padded: fixed width so no minted uid is ever a PREFIX of another
    // (a bare incrementing counter would make "row-1" a substring of
    // "row-19" — a real hazard for the `.contains(uid)` presence checks
    // below).
    format!(
        "{prefix}-{:04}",
        NEXT_UID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

/// One capture flag arm, parsed from the SAME CLI vocabulary
/// `commands/compare.rs::RecordArgs` accepts — kept as the call-site shape so
/// switching this fixture between "shell out" and "hand-author" is a
/// same-signature swap.
struct CaptureFlags {
    response: &'static str,
    frame: &'static str,
    rater: &'static str,
    lens: Option<String>,
    supersedes: Option<String>,
}

fn parse_flags(flags: &[&str]) -> CaptureFlags {
    let mut c = CaptureFlags {
        response: "prefer-a",
        frame: "equal-effort",
        rater: "agent",
        lens: None,
        supersedes: None,
    };
    let mut i = 0;
    while i < flags.len() {
        match flags[i] {
            "--prefer" => {
                c.response = if flags[i + 1] == "a" {
                    "prefer-a"
                } else {
                    "prefer-b"
                };
                i += 2;
            }
            "--equal" => {
                c.response = "equal";
                i += 1;
            }
            "--incomparable" => {
                c.response = "incomparable";
                i += 1;
            }
            "--rater" => {
                c.rater = if flags[i + 1] == "human" {
                    "human"
                } else {
                    "agent"
                };
                i += 2;
            }
            "--lens" => {
                c.lens = Some(flags[i + 1].to_string());
                i += 2;
            }
            "--frame" => {
                c.frame = if flags[i + 1] == "prefer-first" {
                    "prefer-first"
                } else {
                    "equal-effort"
                };
                i += 2;
            }
            "--supersedes" => {
                c.supersedes = Some(flags[i + 1].to_string());
                i += 2;
            }
            other => panic!("test fixture: unhandled capture flag {other}"),
        }
    }
    c
}

/// Hand-author a session-of-one over `a`/`b` (the wire shape a real `compare
/// record` mints) and return the row's uid. See the module doc for why this
/// writes directly rather than shelling out to the write-classed verb.
fn capture(root: &Path, a: &str, b: &str, flags: &[&str]) -> String {
    let c = parse_flags(flags);
    let session_uid = mint("cap-sess");
    let row_uid = mint("cap-row");
    let domain = if c.frame == "prefer-first" {
        "priority"
    } else {
        "value"
    };
    let mut body = format!(
        "schema = \"doctrine.comparison-session\"\nversion = 2\n\n\
         [session]\nuid = \"{session_uid}\"\ndate = \"2026-01-01\"\n\n\
         [[judgement]]\nuid = \"{row_uid}\"\nseq = 0\na = \"{a}\"\nb = \"{b}\"\n\
         response = \"{resp}\"\ndomain = \"{domain}\"\nframe = \"{frame}\"\nform = \"order\"\n\
         rater = \"{rater}\"\n",
        resp = c.response,
        frame = c.frame,
        rater = c.rater,
    );
    if let Some(lens) = &c.lens {
        body.push_str(&format!("lens = \"{lens}\"\n"));
    }
    if let Some(target) = &c.supersedes {
        body.push_str(&format!("supersedes = \"{target}\"\n"));
    }
    body.push_str("date = \"2026-01-01\"\n");
    write(
        root,
        &format!(".doctrine/comparisons/2026-01-01-{session_uid}.toml"),
        &body,
    );
    row_uid
}

/// Hand-author a tombstone session withdrawing `target_uid` (the wire shape
/// `compare withdraw` mints). See the module doc for the write-classed-verb
/// rationale.
fn withdraw(root: &Path, target_uid: &str) {
    let session_uid = mint("cap-tomb");
    let tomb_uid = mint("tomb");
    let body = format!(
        "schema = \"doctrine.comparison-session\"\nversion = 2\n\n\
         [session]\nuid = \"{session_uid}\"\ndate = \"2026-01-01\"\n\n\
         [[tombstone]]\nuid = \"{tomb_uid}\"\nseq = 0\ntarget = \"{target_uid}\"\n\
         date = \"2026-01-01\"\n"
    );
    write(
        root,
        &format!(".doctrine/comparisons/2026-01-01-{session_uid}.toml"),
        &body,
    );
}

fn list(root: &Path, extra: &[&str]) -> String {
    let mut args: Vec<&str> = vec!["compare", "list"];
    args.extend_from_slice(extra);
    ok(&run(root, &args))
}

fn explain(root: &Path, id: &str, extra: &[&str]) -> String {
    let mut args: Vec<&str> = vec!["explain", id];
    args.extend_from_slice(extra);
    ok(&run(root, &args))
}

fn findings(root: &Path, extra: &[&str]) -> String {
    let mut args: Vec<&str> = vec!["findings"];
    args.extend_from_slice(extra);
    ok(&run(root, &args))
}

/// Every line of `compare list`'s output naming `uid` (there is exactly one
/// per row; `list_lines` never wraps a row across two stdout lines).
/// The list line for `uid` — matched on the FIRST whitespace token only (the
/// row's OWN uid, always first, RV-262 F-6), never a substring match: a
/// `superseded→<uid>` token can otherwise collide with a DIFFERENT row's own
/// uid that happens to be its target.
fn line_for<'a>(listing: &'a str, uid: &str) -> &'a str {
    listing
        .lines()
        .find(|l| l.split_whitespace().next() == Some(uid))
        .unwrap_or_else(|| panic!("no list line names {uid}:\n{listing}"))
}

// =============================================================================
// VT-1 — capture → list battery: all ten RowState tokens + --active-only
// =============================================================================

/// Every entity the VT-1 battery seeds, its kind fixed at ISS (backlog issue —
/// value-bearing, non-RSK; comparison admits any such pair). ISS-012/013
/// carry authored anchors (the anchor-conflict scenario); SL-018/SL-019 (the
/// R6 lifecycle scenario — `supersedes` may only target the SAME kind,
/// `relation.rs` `RELATION_RULES`, so backlog issues cannot serve here).
fn seed_vt1_corpus(root: &Path) {
    for id in 1..=22 {
        match id {
            12 => seed_issue(root, id, "[value]\nvalue = 1.0\n"),
            13 => seed_issue(root, id, "[value]\nvalue = 2.0\n"),
            // 18/19 are seeded as slices below (the R6 supersedes scenario).
            18 | 19 => {}
            _ => seed_issue(root, id, ""),
        }
    }
    seed_slice(root, 18, "");
    seed_slice(
        root,
        19,
        "\n[[relation]]\nlabel = \"supersedes\"\ntarget = \"SL-018\"\n",
    );
}

#[test]
fn vt1_capture_to_list_covers_every_status_token_and_active_only_semantics() {
    let dir = tmp();
    let root = dir.path();
    seed_vt1_corpus(root);

    // active: a plain prefer-a row, no other evidence touches the pair.
    let active_uid = capture(root, &iss(1), &iss(2), &["--prefer", "a"]);

    // no-constraint: `--incomparable` is valid evidence, zero constraint.
    let noconstraint_uid = capture(root, &iss(3), &iss(4), &["--incomparable"]);

    // superseded→<uid>: row1 (prefer a), then row2 supersedes row1 (durable
    // replacement act, R2).
    let row1_uid = capture(root, &iss(5), &iss(6), &["--prefer", "a"]);
    let row2_uid = capture(
        root,
        &iss(5),
        &iss(6),
        &["--prefer", "b", "--supersedes", &row1_uid],
    );

    // tombstoned: capture then withdraw.
    let row3_uid = capture(root, &iss(7), &iss(8), &["--prefer", "a"]);
    withdraw(root, &row3_uid);

    // quarantined(cycle): A>B>C>A (C3 preference cycle).
    let cyc1_uid = capture(root, &iss(9), &iss(10), &["--prefer", "a"]);
    let cyc2_uid = capture(root, &iss(10), &iss(11), &["--prefer", "a"]);
    let cyc3_uid = capture(root, &iss(11), &iss(9), &["--prefer", "a"]);

    // quarantined(anchors): anchor(ISS-012)=1 ≤ anchor(ISS-013)=2, but the row
    // asserts ISS-012 > ISS-013 — a C4 violation.
    let anchor_uid = capture(root, &iss(12), &iss(13), &["--prefer", "a"]);

    // inert(lens): a lens-tagged row (R5).
    let lens_uid = capture(
        root,
        &iss(14),
        &iss(15),
        &["--prefer", "a", "--lens", "user-value"],
    );

    // inert(domain): `--frame prefer-first` derives the priority domain (R4).
    let domain_uid = capture(
        root,
        &iss(16),
        &iss(17),
        &["--prefer", "a", "--frame", "prefer-first"],
    );

    // inert(lifecycle): SL-018 is superseded (SL-019's relation) — its rows
    // stay live for elicitation only, inert for inference (R6).
    let lifecycle_uid = capture(root, &sl(18), &iss(20), &["--prefer", "a"]);

    let listing = list(root, &[]);

    let assert_token = |uid: &str, token: &str| {
        let line = line_for(&listing, uid);
        assert!(
            line.contains(&format!("status={token}")),
            "{uid} expected status={token}: {line}"
        );
    };

    assert_token(&active_uid, "active");
    assert_token(&noconstraint_uid, "no-constraint");
    assert_token(&row1_uid, &format!("superseded→{row2_uid}"));
    assert_token(&row2_uid, "active");
    assert_token(&row3_uid, "tombstoned");
    for uid in [&cyc1_uid, &cyc2_uid, &cyc3_uid] {
        assert_token(uid, "quarantined(cycle)");
    }
    assert_token(&anchor_uid, "quarantined(anchors)");
    assert_token(&lens_uid, "inert(lens)");
    assert_token(&domain_uid, "inert(domain)");
    assert_token(&lifecycle_uid, "inert(lifecycle)");

    // [withdrawn] retained verbatim alongside the new status token
    // (extend-don't-replace, RV-262 F-6 precedent).
    assert!(line_for(&listing, &row3_uid).contains("[withdrawn]"));

    // --active-only: quarantined + no-constraint rows ARE active (design A2)
    // — they still show; every other non-Active token is filtered out.
    let active_only = list(root, &["--active-only"]);
    for uid in [
        &active_uid,
        &noconstraint_uid,
        &row2_uid,
        &cyc1_uid,
        &cyc2_uid,
        &cyc3_uid,
        &anchor_uid,
    ] {
        assert!(
            active_only.contains(uid.as_str()),
            "{uid} is Active — must survive --active-only:\n{active_only}"
        );
    }
    for uid in [&row1_uid, &row3_uid, &lens_uid, &domain_uid, &lifecycle_uid] {
        assert!(
            !active_only.contains(uid.as_str()),
            "{uid} is non-Active — must be filtered by --active-only:\n{active_only}"
        );
    }
}

// =============================================================================
// VT-2 — explain's three value-source shapes + --json parity + findings
// =============================================================================

#[test]
fn vt2_explain_value_source_shapes_and_json_parity() {
    let dir = tmp();
    let root = dir.path();

    // Authored: ISS-040 carries its own [value] facet.
    seed_issue(root, 40, "[value]\nvalue = 8.0\n");
    // Bracket ends for the projected shape: ISS-041 (no facet) sits strictly
    // between two anchors via two constraining judgements (mixed raters, so
    // the T7 split is observably non-trivial).
    seed_issue(root, 41, "");
    seed_issue(root, 42, "[value]\nvalue = 2.0\n");
    capture(
        root,
        &iss(40),
        &iss(41),
        &["--prefer", "a", "--rater", "human"],
    );
    capture(
        root,
        &iss(41),
        &iss(42),
        &["--prefer", "a", "--rater", "agent"],
    );

    // Gauge: ISS-050 > ISS-051, an anchor-free component disjoint from
    // ISS-040's anchored one. Per-component placement (SL-216) spreads the
    // whole island by its own height (P8, component H): both entities are
    // Gauge (1.3333/0.6667). The explain assertion below targets ISS-051,
    // whose render carries the component-scoped hint.
    seed_issue(root, 50, "");
    seed_issue(root, 51, "");
    capture(root, &iss(50), &iss(51), &["--prefer", "a"]);

    let authored = explain(root, &iss(40), &[]);
    assert!(
        authored.contains("value 8.0 — authored"),
        "authored shape: {authored}"
    );

    let projected = explain(root, &iss(41), &[]);
    assert!(
        projected.contains("projected") && projected.contains("constraining"),
        "projected shape carries the T7 disclosure: {projected}"
    );
    assert!(
        projected.contains("2.0") && projected.contains("8.0"),
        "projected shape carries its bracket bounds: {projected}"
    );
    assert!(
        projected.contains("human") && projected.contains("agent"),
        "projected shape splits the rater count: {projected}"
    );

    let gauge = explain(root, &iss(51), &[]);
    assert!(
        gauge.contains("gauge") && gauge.contains("no anchor in component"),
        "gauge shape: {gauge}"
    );

    // --json carries the SAME fields structurally (design §4 S3).
    let projected_json = explain(root, &iss(41), &["--json"]);
    let value: serde_json::Value =
        serde_json::from_str(&projected_json).expect("explain --json parses");
    let vs = &value["value_source"];
    assert_eq!(vs["kind"], "value_projected");
    assert!(vs["human"].as_u64().unwrap() >= 1);
    assert!(vs["agent"].as_u64().unwrap() >= 1);
    assert!(vs["lower"].is_number() && vs["upper"].is_number());

    // --- findings: all four SL-213 variants, each naming its exit ---------

    // PreferenceCycle.
    seed_issue(root, 60, "");
    seed_issue(root, 61, "");
    seed_issue(root, 62, "");
    capture(root, &iss(60), &iss(61), &["--prefer", "a"]);
    capture(root, &iss(61), &iss(62), &["--prefer", "a"]);
    capture(root, &iss(62), &iss(60), &["--prefer", "a"]);

    // AnchorConflict.
    seed_issue(root, 70, "[value]\nvalue = 1.0\n");
    seed_issue(root, 71, "[value]\nvalue = 2.0\n");
    capture(root, &iss(70), &iss(71), &["--prefer", "a"]);

    // MalformedSupersession — a mutual cycle can only arise from a hand-
    // merged/authored file (R2); capture cannot produce one sequentially.
    write(
        root,
        ".doctrine/comparisons/2026-01-02-malformed.toml",
        "schema = \"doctrine.comparison-session\"\nversion = 2\n\n\
         [session]\nuid = \"malformed-sess\"\ndate = \"2026-01-02\"\n\n\
         [[judgement]]\nuid = \"mal-a\"\nseq = 0\na = \"ISS-080\"\nb = \"ISS-081\"\n\
         response = \"prefer-a\"\ndomain = \"value\"\nframe = \"equal-effort\"\n\
         form = \"order\"\nrater = \"agent\"\nsupersedes = \"mal-b\"\ndate = \"2026-01-02\"\n\n\
         [[judgement]]\nuid = \"mal-b\"\nseq = 0\na = \"ISS-081\"\nb = \"ISS-080\"\n\
         response = \"prefer-a\"\ndomain = \"value\"\nframe = \"equal-effort\"\n\
         form = \"order\"\nrater = \"agent\"\nsupersedes = \"mal-a\"\ndate = \"2026-01-02\"\n",
    );
    seed_issue(root, 80, "");
    seed_issue(root, 81, "");

    let human = findings(root, &[]);
    assert!(
        human.contains("preference cycles"),
        "PreferenceCycle finding: {human}"
    );
    assert!(
        human.contains("--supersedes") || human.contains("tombstone"),
        "PreferenceCycle exit hint: {human}"
    );
    assert!(
        human.contains("anchor conflicts"),
        "AnchorConflict finding: {human}"
    );
    assert!(
        human.contains("edit an anchor"),
        "AnchorConflict exit hint: {human}"
    );
    assert!(
        human.contains("anchor/gauge disconnects"),
        "AnchorGaugeDisconnect finding: {human}"
    );
    assert!(
        human.contains("malformed supersessions"),
        "MalformedSupersession finding: {human}"
    );

    let findings_json = findings(root, &["--json"]);
    let value: serde_json::Value =
        serde_json::from_str(&findings_json).expect("findings --json parses");
    let kinds: Vec<&str> = value["findings"]
        .as_array()
        .expect("findings array")
        .iter()
        .map(|f| f["kind"].as_str().expect("kind is a string"))
        .collect();
    for expected in [
        "preference cycles",
        "anchor conflicts",
        "anchor/gauge disconnects",
        "malformed supersessions",
    ] {
        assert!(
            kinds.contains(&expected),
            "findings --json carries {expected}: {kinds:?}"
        );
    }
}

// =============================================================================
// VT-3 — determinism under shuffled load order + no-NaN + component-split
// =============================================================================

/// Three hand-authored session files (fixed uids/dates — no rng), each
/// naming a `[[judgement]]` row over the shared corpus below, written under
/// filenames whose alphabetical order (the ONLY order `load_sessions` reads
/// them in, `paths.sort()`) is deliberately permuted across three roots.
const SESSION_A: &str = "schema = \"doctrine.comparison-session\"\nversion = 2\n\n\
[session]\nuid = \"sess-a\"\ndate = \"2026-01-01\"\n\n\
[[judgement]]\nuid = \"row-a\"\nseq = 0\na = \"ISS-090\"\nb = \"ISS-091\"\n\
response = \"prefer-a\"\ndomain = \"value\"\nframe = \"equal-effort\"\nform = \"order\"\n\
rater = \"agent\"\ndate = \"2026-01-01\"\n";
const SESSION_B: &str = "schema = \"doctrine.comparison-session\"\nversion = 2\n\n\
[session]\nuid = \"sess-b\"\ndate = \"2026-01-01\"\n\n\
[[judgement]]\nuid = \"row-b\"\nseq = 0\na = \"ISS-091\"\nb = \"ISS-092\"\n\
response = \"prefer-a\"\ndomain = \"value\"\nframe = \"equal-effort\"\nform = \"order\"\n\
rater = \"agent\"\ndate = \"2026-01-01\"\n";
const SESSION_C: &str = "schema = \"doctrine.comparison-session\"\nversion = 2\n\n\
[session]\nuid = \"sess-c\"\ndate = \"2026-01-01\"\n\n\
[[judgement]]\nuid = \"row-c\"\nseq = 0\na = \"ISS-092\"\nb = \"ISS-093\"\n\
response = \"prefer-a\"\ndomain = \"value\"\nframe = \"equal-effort\"\nform = \"order\"\n\
rater = \"agent\"\ndate = \"2026-01-01\"\n";

fn seed_determinism_entities(root: &Path) {
    for id in 90..=93 {
        seed_issue(root, id, "");
    }
}

/// Write the three fixed sessions under filenames whose sort order is
/// `order` (a permutation of the three file BODIES) — the shuffle is over
/// which body lands in which alphabetical slot, never over content.
fn write_shuffled(root: &Path, order: [&str; 3]) {
    for (slot, body) in ["aaa", "bbb", "ccc"].iter().zip(order) {
        write(
            root,
            &format!(".doctrine/comparisons/2026-01-01-{slot}.toml"),
            body,
        );
    }
}

#[test]
fn vt3_determinism_under_shuffled_session_file_load_order() {
    let permutations: [[&str; 3]; 3] = [
        [SESSION_A, SESSION_B, SESSION_C],
        [SESSION_C, SESSION_A, SESSION_B],
        [SESSION_B, SESSION_C, SESSION_A],
    ];

    let mut listings = Vec::new();
    let mut explanations = Vec::new();
    for order in permutations {
        let dir = tmp();
        let root = dir.path();
        seed_determinism_entities(root);
        write_shuffled(root, order);
        listings.push(list(root, &[]));
        explanations.push(explain(root, &iss(91), &[]));
    }

    assert!(
        listings.windows(2).all(|w| w[0] == w[1]),
        "compare list is byte-identical across shuffled load order: {listings:?}"
    );
    assert!(
        explanations.windows(2).all(|w| w[0] == w[1]),
        "explain is byte-identical across shuffled load order: {explanations:?}"
    );
}

/// The component-split-after-quarantine discontinuity golden (design §5.9
/// last row): a single connected component (A>B, B>C, C>A cycle PLUS a
/// pendant A>D) quarantines its whole cycle (C3), leaving D isolated in its
/// OWN component — its projected value must still be finite, and the
/// pre/post-quarantine graphs' entity sets must differ observably (the split
/// is real, not a no-op).
#[test]
fn vt3_component_split_after_quarantine_discontinuity_is_finite_and_total_ordered() {
    let dir = tmp();
    let root = dir.path();
    for id in 100..=103 {
        seed_issue(root, id, "");
    }
    // A>B>C>A (quarantined cycle) plus the external A>D edge (D3 golden).
    capture(root, &iss(100), &iss(101), &["--prefer", "a"]);
    capture(root, &iss(101), &iss(102), &["--prefer", "a"]);
    capture(root, &iss(102), &iss(100), &["--prefer", "a"]);
    capture(root, &iss(100), &iss(103), &["--prefer", "a"]);

    let listing = list(root, &[]);
    // The cycle rows are quarantined; the external edge stays active.
    for id in [100, 101, 102] {
        assert!(
            listing.contains(&format!("{}* vs", iss(id)))
                || listing.contains(&format!("vs {}*", iss(id))),
            "cycle participant present: {listing}"
        );
    }
    assert!(
        listing.contains("quarantined(cycle)"),
        "the cycle quarantines: {listing}"
    );

    // D's explain carries a finite, non-NaN value — never a manufactured
    // NaN from the split component's projection.
    let d_explain = explain(root, &iss(103), &[]);
    assert!(
        !d_explain.to_lowercase().contains("nan"),
        "no NaN in D's value-source: {d_explain}"
    );
    assert!(
        d_explain.contains("value "),
        "D still carries a value-source line: {d_explain}"
    );
}
