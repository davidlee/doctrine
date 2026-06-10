//! SL-037 PHASE-04 — IMP-014 cross-verb golden harness (the column-model net).
//!
//! IMP-014 asks for ONE cross-verb black-box net over the shared `src/listing.rs`
//! render surface, so a future `listing.rs` format change must be acknowledged at
//! the shared surface — not slip through N independently-regenerated per-verb
//! goldens. Since IMP-014 was authored the slice grew per-verb golden files and a
//! cross-verb conformance matrix that already pin PARTS of EX-1/EX-2. Re-rendering
//! everything here would DUPLICATE those assertions (parallel implementation,
//! CLAUDE.md). So this module is a GAP-FILL net: it OWNS the genuinely-untested
//! surfaces and CITES the rest by path:line — duplicating nothing.
//!
//! OWNED here (no existing test pins these):
//!   - backlog / slice / spec / governance-`policy` DEFAULT tables (only adr +
//!     standard had default-table goldens before this);
//!   - the `--columns` projection — selection + ORDER + slug-REVEAL — across every
//!     migrated verb (`--columns` was invoked by NO test before this, only named
//!     in comments at e2e_adr_cli_golden.rs:294/296);
//!   - the empty-list header-suppressed `""` path per verb (`listing.rs:296`);
//!   - the spec per-subtype multi-block layout AND the omitted-empty-block case (R3)
//!     at the CLI surface (PHASE-03 pinned it only in-crate);
//!   - the governance `policy` table breadth (adr/standard were pinned; policy was
//!     the unpinned third governance kind — all three share one `governance.rs`
//!     `GOV_COLUMNS`/`GOV_DEFAULT` render path);
//!   - the uniform unknown-column error at the CLI surface;
//!   - the D7 `--columns`-under-`--json` no-op (belt-and-braces).
//!
//! CITED, NOT re-asserted (already green — see the listed path:line):
//!   - memory `--columns` rejection — `tests/e2e_list_conformance.rs:126`
//!     (`columns_flag_is_rejected_on_memory_list_never_silently_ignored`, D9/R4).
//!     EX-2's memory clause is satisfied THERE; duplicating it would be DRY-hostile.
//!   - adr/standard slug-free DEFAULT table + `--json` envelope —
//!     `tests/e2e_adr_cli_golden.rs:286-330` and `tests/e2e_standard_cli_golden.rs:288-332`.
//!     Both ride the IDENTICAL `governance.rs` render path that `policy` exercises
//!     here, so `policy` justifies the breadth representative.
//!   - the cross-verb `{kind, rows}` JSON envelope SHAPE on every kind —
//!     `tests/e2e_list_conformance.rs:149`.
//!
//! Idiom (no new test dep, A1): rides the existing black-box golden pattern from
//! `e2e_adr_cli_golden.rs:28-100` — spawn `env!("CARGO_BIN_EXE_doctrine")` over a
//! hand-seeded `tempfile::tempdir()` authored tree with FIXED dates (never
//! `doctrine <kind> new`, which would stamp `clock::today()`), and `assert_eq!`
//! byte-exact stdout. `CARGO_BIN_EXE_doctrine` resolves to the freshly-built test
//! bin (mem `stale-cargo-bin-exe`).
//!
//! The corpus is a SHARED logical row-set per IMP-014: ids 2/5/7/9, each seeded OUT
//! of id order and spanning visible + hidden statuses, so every golden also pins the
//! per-kind hide-set + ascending sort + prefixed id.

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

/// `doctrine <kind> list <extra...> -p <root>` over the built binary.
fn list(root: &Path, kind: &str, extra: &[&str]) -> Output {
    Command::new(BIN)
        .arg(kind)
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

// ===========================================================================
// Hand-seeded fixture corpora — FIXED dates (A2), each verb's authored tree.
// ===========================================================================

/// backlog: `.doctrine/backlog/<kind>/NNN/backlog-NNN.{toml,md}`. Five kinds ride
/// one stem; the prefixed id is `<KIND-PREFIX>-NNN` (ISS/IMP/CHR/…) and rows sort
/// by `(kind.ordinal, id)` — issue before improvement here, NOT id order.
fn seed_backlog(root: &Path, kind: &str, id: u32, slug: &str, title: &str, status: &str) {
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
             drift = []\n"
        ),
    )
    .unwrap();
    fs::write(
        dir.join(format!("backlog-{id:03}.md")),
        format!("# {title}\n"),
    )
    .unwrap();
}

/// Shared backlog corpus: out of id order, spanning visible (open/triaged) + hidden
/// (closed, a terminal). `closed` must be ABSENT from the default list.
fn seed_backlog_corpus(root: &Path) {
    seed_backlog(
        root,
        "improvement",
        9,
        "shared-cols",
        "Shared columns",
        "open",
    );
    seed_backlog(root, "issue", 2, "flaky-test", "Flaky test", "triaged");
    seed_backlog(root, "chore", 5, "old-chore", "Old chore", "closed");
}

/// slice: `.doctrine/slice/NNN/slice-NNN.{toml,md}`. The `phases` cell reads the
/// GITIGNORED runtime state tree — absent here, so every row's phases cell is `—`
/// (untracked) and JSON `phases` is `null`. Hidden set: `done`/`abandoned`.
fn seed_slice(root: &Path, id: u32, slug: &str, title: &str, status: &str) {
    let dir = root.join(format!(".doctrine/slice/{id:03}"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("slice-{id:03}.toml")),
        format!(
            "id = {id}\n\
             slug = \"{slug}\"\n\
             title = \"{title}\"\n\
             status = \"{status}\"\n\
             created = \"2026-01-02\"\n\
             updated = \"2026-01-02\"\n\
             \n\
             [relationships]\n"
        ),
    )
    .unwrap();
    fs::write(
        dir.join(format!("slice-{id:03}.md")),
        format!("# {title}\n"),
    )
    .unwrap();
}

/// Shared slice corpus: out of id order, spanning visible (proposed/started) +
/// hidden (`done`). No state tree → `phases` cell `—` for every row.
fn seed_slice_corpus(root: &Path) {
    seed_slice(root, 25, "listing-spine", "Listing spine", "started");
    seed_slice(root, 9, "status-rollup", "Status rollup", "proposed");
    seed_slice(root, 4, "old-slice", "Old slice", "done");
}

/// spec: `.doctrine/spec/<product|tech>/NNN/spec-NNN.{toml,md}` + a `members.toml`
/// whose `[[member]]` rows are COUNTED for the `#members` cell (the FK is not
/// dereferenced by `list`, so no REQ tree is needed). Prefixed id is subtype-keyed
/// (`PRD-` for product, `SPEC-` for tech). Hidden set: `superseded`.
fn seed_spec(
    root: &Path,
    subtype: &str,
    id: u32,
    slug: &str,
    title: &str,
    status: &str,
    members: u32,
) {
    let dir = root.join(format!(".doctrine/spec/{subtype}/{id:03}"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("spec-{id:03}.toml")),
        format!(
            "schema = \"doctrine.spec.{subtype}\"\n\
             version = 1\n\
             id = {id}\n\
             slug = \"{slug}\"\n\
             title = \"{title}\"\n\
             status = \"{status}\"\n\
             kind = \"{subtype}\"\n\
             tags = []\n"
        ),
    )
    .unwrap();
    fs::write(dir.join(format!("spec-{id:03}.md")), format!("# {title}\n")).unwrap();
    if members > 0 {
        let mut body = String::new();
        for n in 1..=members {
            body.push_str(&format!(
                "[[member]]\nrequirement = \"REQ-{n:03}\"\nlabel = \"FR-{n}\"\norder = {n}\n"
            ));
        }
        fs::write(dir.join("members.toml"), body).unwrap();
    }
}

/// Shared spec corpus: BOTH subtypes, out of id order, spanning visible + hidden
/// (`superseded`). Drives the two-block layout (product then tech) + the hide-set.
fn seed_spec_corpus(root: &Path) {
    seed_spec(root, "product", 7, "billing", "Billing", "active", 2);
    seed_spec(root, "product", 3, "onboarding", "Onboarding", "draft", 0);
    seed_spec(root, "product", 5, "old-prd", "Old PRD", "superseded", 1);
    seed_spec(root, "tech", 2, "auth-spine", "Auth spine", "active", 3);
    seed_spec(root, "tech", 8, "gone-tech", "Gone tech", "superseded", 0);
}

/// policy: `.doctrine/policy/NNN/policy-NNN.{toml,md}` — one of the three governance
/// kinds (adr/policy/standard) over the shared `governance.rs` `GOV_COLUMNS`/
/// `GOV_DEFAULT` render path. Vocab `draft/required/deprecated/retired`; hidden set
/// `deprecated`/`retired`. Prefixed id `POL-NNN`.
fn seed_policy(root: &Path, id: u32, slug: &str, title: &str, status: &str) {
    let dir = root.join(format!(".doctrine/policy/{id:03}"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("policy-{id:03}.toml")),
        format!(
            "id = {id}\n\
             slug = \"{slug}\"\n\
             title = \"{title}\"\n\
             status = \"{status}\"\n\
             created = \"2026-01-02\"\n\
             updated = \"2026-01-02\"\n\
             \n\
             [relationships]\n\
             supersedes = []\n\
             superseded_by = []\n\
             related = []\n\
             tags = []\n"
        ),
    )
    .unwrap();
    fs::write(
        dir.join(format!("policy-{id:03}.md")),
        format!("# {title}\n"),
    )
    .unwrap();
}

/// Shared policy corpus: out of id order, spanning visible (draft/required) + hidden
/// (`deprecated`/`retired`). Both hidden tokens must be ABSENT from the default list.
fn seed_policy_corpus(root: &Path) {
    seed_policy(root, 4, "commit-style", "Commit style", "required");
    seed_policy(root, 2, "branch-naming", "Branch naming", "draft");
    seed_policy(root, 7, "old-policy", "Old policy", "retired");
    seed_policy(root, 9, "dep-policy", "Dep policy", "deprecated");
}

// ===========================================================================
// T2 — DEFAULT table goldens (VT-1, EX-1 default surface).
// adr/standard defaults are CITED (e2e_adr_cli_golden.rs:286 /
// e2e_standard_cli_golden.rs:288), NOT re-pinned here.
// ===========================================================================

#[test]
fn backlog_list_default_table_is_byte_exact() {
    let dir = tmp();
    seed_backlog_corpus(dir.path());
    let out = list(dir.path(), "backlog", &[]);
    ok(&out);
    // Default = [id, kind, status, title] (slug hidden, SL-037 D4). Sorted by
    // (kind.ordinal, id): issue before improvement; `closed` (terminal) ABSENT.
    assert_eq!(
        stdout(&out),
        "id       kind         status   title\n\
         ISS-002  issue        triaged  Flaky test\n\
         IMP-009  improvement  open     Shared columns\n"
    );
}

#[test]
fn slice_list_default_table_is_byte_exact() {
    let dir = tmp();
    seed_slice_corpus(dir.path());
    let out = list(dir.path(), "slice", &[]);
    ok(&out);
    // Default = [id, status, phases, title]; phases cell `—` (untracked, no state
    // tree); ascending by id; `done` (hidden) ABSENT.
    assert_eq!(
        stdout(&out),
        "id      status    phases  title\n\
         SL-009  proposed  —       Status rollup\n\
         SL-025  started   —       Listing spine\n"
    );
}

#[test]
fn spec_list_default_table_is_byte_exact_multi_block() {
    let dir = tmp();
    seed_spec_corpus(dir.path());
    let out = list(dir.path(), "spec", &[]);
    ok(&out);
    // Default = [id, status, title, #members]; one labelled block per subtype
    // (product then tech), concatenated with NO blank separator; `superseded`
    // (hidden) ABSENT from each block; members count rendered per row.
    assert_eq!(
        stdout(&out),
        "product\n\
         id       status  title       #members\n\
         PRD-003  draft   Onboarding  0\n\
         PRD-007  active  Billing     2\n\
         tech\n\
         id        status  title       #members\n\
         SPEC-002  active  Auth spine  3\n"
    );
}

#[test]
fn policy_list_default_table_is_byte_exact() {
    let dir = tmp();
    seed_policy_corpus(dir.path());
    let out = list(dir.path(), "policy", &[]);
    ok(&out);
    // Governance default = [id, status, title] (slug hidden); ascending by id;
    // `deprecated`/`retired` (hidden) ABSENT. adr/standard ride the IDENTICAL
    // GOV_COLUMNS/GOV_DEFAULT path — pinned at e2e_adr_cli_golden.rs:286 /
    // e2e_standard_cli_golden.rs:288; policy is the breadth representative (T7).
    assert_eq!(
        stdout(&out),
        "id       status    title\n\
         POL-002  draft     Branch naming\n\
         POL-004  required  Commit style\n"
    );
}

// ===========================================================================
// T3 — `--columns` projection goldens (VT-1, EX-1 --columns surface — the core
// gap: NO existing test invokes `--columns`). Each asserts SELECTION + ORDER +
// slug-REVEAL; policy reorders columns out of default order.
// ===========================================================================

#[test]
fn backlog_list_columns_selects_orders_and_reveals_slug() {
    let dir = tmp();
    seed_backlog_corpus(dir.path());
    // `id,slug,status`: drops kind+title, REVEALS slug (hidden by default), and
    // orders slug before status.
    let out = list(dir.path(), "backlog", &["--columns", "id,slug,status"]);
    ok(&out);
    assert_eq!(
        stdout(&out),
        "id       slug         status\n\
         ISS-002  flaky-test   triaged\n\
         IMP-009  shared-cols  open\n"
    );
}

#[test]
fn slice_list_columns_selects_orders_and_reveals_slug() {
    let dir = tmp();
    seed_slice_corpus(dir.path());
    // `id,slug,phases`: reveals slug, drops status+title, phases cell still `—`.
    let out = list(dir.path(), "slice", &["--columns", "id,slug,phases"]);
    ok(&out);
    assert_eq!(
        stdout(&out),
        "id      slug           phases\n\
         SL-009  status-rollup  —\n\
         SL-025  listing-spine  —\n"
    );
}

#[test]
fn spec_list_columns_reveals_slug_per_block() {
    let dir = tmp();
    seed_spec_corpus(dir.path());
    // `id,slug,members`: reveals slug, drops status+title; the selection is resolved
    // ONCE and applied per subtype block (R3) — the labelled-block layout survives.
    let out = list(dir.path(), "spec", &["--columns", "id,slug,members"]);
    ok(&out);
    assert_eq!(
        stdout(&out),
        "product\n\
         id       slug        #members\n\
         PRD-003  onboarding  0\n\
         PRD-007  billing     2\n\
         tech\n\
         id        slug        #members\n\
         SPEC-002  auth-spine  3\n"
    );
}

#[test]
fn policy_list_columns_reorders_and_reveals_slug() {
    let dir = tmp();
    seed_policy_corpus(dir.path());
    // `id,status,slug,title`: a REORDER (slug injected between status and title) +
    // slug reveal — the full available set, out of default order.
    let out = list(dir.path(), "policy", &["--columns", "id,status,slug,title"]);
    ok(&out);
    assert_eq!(
        stdout(&out),
        "id       status    slug           title\n\
         POL-002  draft     branch-naming  Branch naming\n\
         POL-004  required  commit-style   Commit style\n"
    );
}

#[test]
fn unknown_column_errors_with_the_uniform_available_set() {
    let dir = tmp();
    seed_policy_corpus(dir.path());
    // The one uniform unknown-column error (design D3 / select_columns) at the CLI
    // surface — available tokens listed in GOV_COLUMNS declaration order.
    let out = list(dir.path(), "policy", &["--columns", "id,bogus"]);
    assert!(!out.status.success());
    assert_eq!(
        stderr(&out),
        "Error: unknown column `bogus` (available: id, status, slug, title)\n"
    );
}

// ===========================================================================
// T4 — `--json` goldens (VT-1, EX-1 json surface) — proving D2: the JSON envelope
// is UNTOUCHED by the column churn. adr/standard `--json` are CITED
// (e2e_adr_cli_golden.rs:304 / e2e_standard_cli_golden.rs:306). NB: `write!`, not
// `writeln!` — NO trailing newline. members stays an INT.
// ===========================================================================

#[test]
fn backlog_list_json_is_byte_exact() {
    let dir = tmp();
    seed_backlog_corpus(dir.path());
    let out = list(dir.path(), "backlog", &["--json"]);
    ok(&out);
    assert_eq!(
        stdout(&out),
        "{\n  \"kind\": \"backlog\",\n  \"rows\": [\n    {\n      \"id\": \"ISS-002\",\n      \"kind\": \"issue\",\n      \"resolution\": null,\n      \"slug\": \"flaky-test\",\n      \"status\": \"triaged\",\n      \"title\": \"Flaky test\"\n    },\n    {\n      \"id\": \"IMP-009\",\n      \"kind\": \"improvement\",\n      \"resolution\": null,\n      \"slug\": \"shared-cols\",\n      \"status\": \"open\",\n      \"title\": \"Shared columns\"\n    }\n  ]\n}"
    );
}

#[test]
fn slice_list_json_is_byte_exact() {
    let dir = tmp();
    seed_slice_corpus(dir.path());
    let out = list(dir.path(), "slice", &["--json"]);
    ok(&out);
    // phases is STRUCTURED (null here — untracked), never the rendered `—` cell.
    assert_eq!(
        stdout(&out),
        "{\n  \"kind\": \"slice\",\n  \"rows\": [\n    {\n      \"id\": \"SL-009\",\n      \"phases\": null,\n      \"slug\": \"status-rollup\",\n      \"status\": \"proposed\",\n      \"title\": \"Status rollup\"\n    },\n    {\n      \"id\": \"SL-025\",\n      \"phases\": null,\n      \"slug\": \"listing-spine\",\n      \"status\": \"started\",\n      \"title\": \"Listing spine\"\n    }\n  ]\n}"
    );
}

#[test]
fn spec_list_json_is_byte_exact() {
    let dir = tmp();
    seed_spec_corpus(dir.path());
    let out = list(dir.path(), "spec", &["--json"]);
    ok(&out);
    // ONE envelope spanning both subtypes; each row carries `subtype`; `members`
    // stays an INT (not the rendered cell, D2).
    assert_eq!(
        stdout(&out),
        "{\n  \"kind\": \"spec\",\n  \"rows\": [\n    {\n      \"id\": \"PRD-003\",\n      \"members\": 0,\n      \"slug\": \"onboarding\",\n      \"status\": \"draft\",\n      \"subtype\": \"product\"\n    },\n    {\n      \"id\": \"PRD-007\",\n      \"members\": 2,\n      \"slug\": \"billing\",\n      \"status\": \"active\",\n      \"subtype\": \"product\"\n    },\n    {\n      \"id\": \"SPEC-002\",\n      \"members\": 3,\n      \"slug\": \"auth-spine\",\n      \"status\": \"active\",\n      \"subtype\": \"tech\"\n    }\n  ]\n}"
    );
}

#[test]
fn policy_list_json_is_byte_exact() {
    let dir = tmp();
    seed_policy_corpus(dir.path());
    let out = list(dir.path(), "policy", &["--json"]);
    ok(&out);
    assert_eq!(
        stdout(&out),
        "{\n  \"kind\": \"policy\",\n  \"rows\": [\n    {\n      \"id\": \"POL-002\",\n      \"slug\": \"branch-naming\",\n      \"status\": \"draft\",\n      \"title\": \"Branch naming\"\n    },\n    {\n      \"id\": \"POL-004\",\n      \"slug\": \"commit-style\",\n      \"status\": \"required\",\n      \"title\": \"Commit style\"\n    }\n  ]\n}"
    );
}

#[test]
fn columns_under_json_is_a_no_op_byte_identical_to_plain_json() {
    // D7 / A5: `--columns` is taken BEFORE the JSON build, so a `--columns X --json`
    // invocation must be byte-identical to plain `--json` (the projection is
    // table-only; JSON stays faithful/full). A *subset* request (`id` only) is the
    // load-bearing case: were the JSON path to wrongly honour `--columns`, the
    // projected envelope would drop status/slug/title and diverge — a full-set
    // request would only catch reordering, not field-filtering (D2).
    let dir = tmp();
    seed_policy_corpus(dir.path());
    let plain = list(dir.path(), "policy", &["--json"]);
    let projected = list(dir.path(), "policy", &["--columns", "id", "--json"]);
    ok(&plain);
    ok(&projected);
    assert_eq!(stdout(&plain), stdout(&projected));
}

// ===========================================================================
// T5 — empty-list `""` path per verb (VT-2, EX-2). `render_columns` suppresses the
// header on empty rows (listing.rs:296) → stdout is LITERALLY empty.
// ===========================================================================

#[test]
fn empty_list_suppresses_the_header_on_every_migrated_verb() {
    let dir = tmp();
    for kind in ["backlog", "slice", "spec", "policy"] {
        let out = list(dir.path(), kind, &[]);
        ok(&out);
        assert_eq!(
            stdout(&out),
            "",
            "{kind} list on an empty tree must emit \"\""
        );
    }
}

// ===========================================================================
// T6 — spec multi-block + omitted-empty-block (VT-2, EX-2 / R3). The full
// two-subtype layout is pinned by `spec_list_default_table_is_byte_exact_multi_block`
// (T2); here the OMITTED-empty-block case: a tech-only corpus → the product block
// (label line included) is suppressed entirely, not rendered as an empty header.
// ===========================================================================

#[test]
fn spec_omits_an_empty_subtype_block_entirely() {
    let dir = tmp();
    // Tech-only corpus: no product specs at all → the `product` label + grid are
    // both omitted; only the `tech` block renders.
    seed_spec(
        dir.path(),
        "tech",
        2,
        "auth-spine",
        "Auth spine",
        "active",
        3,
    );
    let out = list(dir.path(), "spec", &[]);
    ok(&out);
    assert_eq!(
        stdout(&out),
        "tech\n\
         id        status  title       #members\n\
         SPEC-002  active  Auth spine  3\n"
    );
}

// ===========================================================================
// T8 — memory `--columns` rejection (VT-2, EX-2). CITED, NOT duplicated: this is
// already green at `tests/e2e_list_conformance.rs:126`
// (`columns_flag_is_rejected_on_memory_list_never_silently_ignored`, D9/R4). A
// second copy here would be parallel implementation (CLAUDE.md / A4). This module
// treats that test as the harness's memory-rejection assertion by reference.
// ===========================================================================
