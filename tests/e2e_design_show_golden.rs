// SPDX-License-Identifier: GPL-3.0-only
//! SL-246 PHASE-05 — `doctrine design show` as BLACK-BOX CLI goldens.
//!
//! `DEC-261` reclaims the bare verb for the **design document**: `design show
//! SL-NNN` renders `design.md` verbatim followed by its inbound knowledge block,
//! and the run's turn envelope keeps its three existing renderings under
//! `--format prompt|json|status`. Nothing is renamed; only the default moves.
//!
//! What this crate pins, over the BUILT binary
//! (`mem.pattern.testing.black-box-cli-golden`):
//!
//! - `VT-1` — the composed read, the moved default, `X2`'s clean error, and
//!   `I1`'s byte-identity on the three envelope renderings.
//! - `VT-2` — the flag partition of §5.2 / `D7`: four refusals stated over the
//!   *rendering* (never over how the rendering was spelled) plus `--path`, which
//!   the partition does not reach.
//! - `VT-3` — `OQ-1`/`QUE-223`: a document behind its run is disclosed on both
//!   arms — a `note:` line on the text arm, a `stale_run` object on the JSON one.
//!
//! **Scope boundary.** `tests/e2e_inspect_golden.rs` is the exhaustive owner of
//! the knowledge block's *contents* — every level, every floor marker, the
//! seven-kind source parity, the cross-kind ordering. This crate proves only that
//! a SECOND caller composes it, so its corpus is deliberately two records.
//!
//! Determinism: the document is hand-written with fixed bytes and the knowledge
//! corpus is hand-seeded with fixed dates (never minted through `knowledge new`,
//! which stamps `clock::today()`). The one volatile span in the whole crate is the
//! design run's uid, which `DesignRun` exposes and every golden placeholders.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::{Path, PathBuf};
use std::process::Output;

mod common;
mod design_fixture;

use design_fixture::{SLICE, SLICE_NUMBER};

// ── the fixture document ──────────────────────────────────────────────────

/// The hand-written `design.md` every document case reads.
///
/// **It carries an UNHELD section marker on purpose.** `<!-- doctrine:section
/// sec-9 -->` is exactly what `design_run::document::parse` refuses when the run
/// does not hold `sec-9` — so a document arm that parsed before printing would
/// hand the reader a refusal instead of their document. `EX-2`/`A2` says it must
/// not, and this marker is that requirement made observable.
const DOCUMENT: &str =
    "# Design — fixture\n\n<!-- doctrine:section sec-9 -->\nA hand-written body line.\n";

/// A markerless document, for the fixtures that must survive `design start
/// --from-design` (which DOES parse, and would refuse [`DOCUMENT`]'s unheld
/// marker at import).
const ADOPTABLE: &str = "# Design — adoptable fixture\n\nAn adoptable body line.\n";

/// The three envelope renderings as the BRANCH-POINT binary emitted them, against
/// `DesignRun::start()`'s cold run, with the run uid replaced by [`UID`].
///
/// Captured by running the pre-`DEC-261` binary before the T1 edit — this is `I1`'s
/// other half, and it cannot be re-derived after the fact.
const UID: &str = "<UID>";

const PRIOR_PROMPT: &str = r#"run <UID> revision 1 stage exploring
watermark absent materialised false
change_log_floor 1 receipt_floor 1
posture breadth (agent-proposed) cursor unset
totals nodes=0 open=0 resolved=0 deferred=0 pruned=0 blocked=0 open_outside_frontier=0 sections=0 sections_outstanding_review=0 changes_since_baseline=0
frontier
blockers
sections
records
changes: UNAVAILABLE — the change log covers revisions from 1 onward, and revision 0 is below that floor; see `design show --full`
truncated false
declare {"run_uid":"<uid>","known_revision":<n>,"submission_id":"<unique>","declare":[{"subject":"inq-2","question":"...","parent":"inq-1","needs":["inq-1"]}],"traversal":{"pin":"inq-2","cursor":"inq-2","posture":"depth","authority":"user-pinned"}}  (omit a key to persist it, send null to clear a scalar, [] to clear a collection)
contract doctrine design contract --format prompt
"#;

/// The ONE span of [`PRIOR_PROMPT`] `EX-7` deliberately moves, and its replacement.
///
/// `src/design_run/render/mod.rs`'s `delta_unavailable_line` pointed at the bare
/// verb, which after `DEC-261` is a document read — so the migration re-points it.
/// The byte-identity assertion applies the substitution and ALSO asserts it
/// applied, so an `EX-7` regression cannot make the golden vacuously true
/// (`mem.pattern.testing.mutation-beat-asserts-application`).
const PRIOR_REPAIR: &str = "see `design show --full`";
const MIGRATED_REPAIR: &str = "see `design show --format prompt --full`";

const PRIOR_STATUS: &str = r#"design run <UID> for slice 233
  stage        exploring (revision 1)
  traversal    breadth posture, agent-proposed; cursor unset
  inquiry      0 nodes — 0 open, 0 resolved, 0 deferred, 0 pruned, 0 blocked
  sections     0 (0 with outstanding review)
  changes      0 since the declared baseline
"#;

const PRIOR_JSON: &str = r#"{
  "schema": "doctrine.design-turn",
  "version": 1,
  "detail": "normal",
  "run": {
    "uid": "<UID>",
    "slice": 233,
    "revision": 1,
    "stage": "exploring",
    "review_policy": "human-only",
    "watermark": null,
    "materialised": false,
    "change_log_floor": 1,
    "receipt_floor": 1,
    "posture": "breadth",
    "posture_authority": "agent-proposed",
    "cursor": null,
    "cursor_stale": false
  },
  "totals": {
    "nodes": 0,
    "open": 0,
    "resolved": 0,
    "deferred": 0,
    "pruned": 0,
    "blocked": 0,
    "open_outside_frontier": 0,
    "sections": 0,
    "sections_outstanding_review": 0,
    "changes_since_baseline": 0
  },
  "next_obligation": null,
  "pinned": null,
  "active_path": [],
  "frontier": [],
  "blockers": [],
  "sections": [],
  "acts": [],
  "durable_records": [],
  "changes": {
    "delta": "unavailable",
    "floor": 1,
    "known_revision": 0
  },
  "review_pass": null,
  "pass_stale": false,
  "outstanding": {
    "blocker": 0,
    "major": 0,
    "minor": 0,
    "nit": 0
  },
  "declaration_example": "{\"run_uid\":\"<uid>\",\"known_revision\":<n>,\"submission_id\":\"<unique>\",\"declare\":[{\"subject\":\"inq-2\",\"question\":\"...\",\"parent\":\"inq-1\",\"needs\":[\"inq-1\"]}],\"traversal\":{\"pin\":\"inq-2\",\"cursor\":\"inq-2\",\"posture\":\"depth\",\"authority\":\"user-pinned\"}}  (omit a key to persist it, send null to clear a scalar, [] to clear a collection)",
  "contract_pointer": "doctrine design contract --format prompt",
  "omitted": {
    "active_path": 0,
    "frontier": 0,
    "blockers": 0,
    "changes": 0,
    "durable_records": 0,
    "sections": 0
  },
  "truncated": false
}
"#;

// ── helpers ───────────────────────────────────────────────────────────────

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

/// `doctrine design show SL-233 <extra…> -p <root>` over the BUILT binary.
///
/// `-p` is always explicit: `root::find` walks CWD to `/` and would otherwise
/// resolve an incidental project root (`mem.pattern.testing.no-root-find-walk`).
fn show(root: &Path, extra: &[&str]) -> Output {
    common::doctrine_cmd(root)
        .arg("design")
        .arg("show")
        .arg(SLICE)
        .args(extra)
        .arg("-p")
        .arg(root)
        .output()
        .expect("spawn doctrine")
}

fn ok(root: &Path, extra: &[&str]) -> String {
    let out = show(root, extra);
    assert!(
        out.status.success(),
        "design show {extra:?} failed: {}",
        stderr(&out)
    );
    stdout(&out)
}

/// Assert a refusal that CONVICTS: non-zero exit, and stderr naming the flag —
/// never merely "stderr is non-empty"
/// (`mem.pattern.review.absence-probe-must-convict`).
fn refused(root: &Path, extra: &[&str], flag: &str) {
    let out = show(root, extra);
    assert!(
        !out.status.success(),
        "design show {extra:?} must be refused; stdout was: {}",
        stdout(&out)
    );
    let err = stderr(&out);
    assert!(
        err.contains(flag),
        "the refusal for {extra:?} must name {flag}; stderr was: {err}"
    );
    assert!(
        stdout(&out).is_empty(),
        "a refused rendering emits nothing on stdout: {}",
        stdout(&out)
    );
}

/// The minimal two-record knowledge corpus: one filled `DEC` and one `CPT`, each
/// authoring ONE inbound `shapes` edge at the fixture slice.
///
/// Deliberately two records, not seven kinds — `tests/e2e_inspect_golden.rs`
/// (`seed_knowledge_corpus`) is the exhaustive owner of level / marker / source-kind
/// parity. Fixed `created`/`updated` dates, hand-seeded: the minting verbs stamp
/// `clock::today()` and would make every golden date-dependent.
fn seed_knowledge(root: &Path) {
    write(
        root,
        ".doctrine/knowledge/decision/801/record-801.toml",
        "schema = \"doctrine.knowledge\"\nversion = 1\n\n\
         id = 801\nslug = \"a-fixture-decision\"\ntitle = \"A fixture decision\"\n\
         record_kind = \"decision\"\nstatus = \"accepted\"\n\
         created = \"2026-01-01\"\nupdated = \"2026-01-01\"\ntags = []\n\n\
         [facet]\ncontext      = \"Because things\"\nchoice       = \"Do the thing\"\n\
         alternatives = [\"Skip it\"]\nrationale    = \"It reduces risk\"\n\
         consequences = [\"Less risk\"]\ndecided_by   = \"epoch\"\n\
         decided_on   = \"2026-01-05\"\n\n\
         [[relation]]\nlabel = \"shapes\"\ntarget = \"SL-233\"\n",
    );
    write(
        root,
        ".doctrine/knowledge/decision/801/record-801.md",
        "Decision body line.\n",
    );
    write(
        root,
        ".doctrine/knowledge/concept/802/record-802.toml",
        "schema = \"doctrine.knowledge\"\nversion = 1\n\n\
         id = 802\nslug = \"a-fixture-concept\"\ntitle = \"A fixture concept\"\n\
         record_kind = \"concept\"\nstatus = \"active\"\n\
         created = \"2026-01-01\"\nupdated = \"2026-01-01\"\ntags = []\n\n\
         [facet]\ndefinition = \"A thing that is a thing\"\n\n\
         [[relation]]\nlabel = \"shapes\"\ntarget = \"SL-233\"\n",
    );
    write(
        root,
        ".doctrine/knowledge/concept/802/record-802.md",
        "Concept body line.\n",
    );
}

/// A root holding a slice, its authored `design.md`, and the two-record knowledge
/// corpus — and **NO design run at all**.
///
/// This is the majority real-corpus shape, not an edge case: runtime state is
/// per-worktree and gitignored, so a landed slice (`SL-244`) has 200 KB of
/// authored design and no snapshot. `A-c`: the document arm must render here.
fn document_only(body: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tmp();
    let root = dir.path().to_path_buf();
    design_fixture::seed_slice_record(&root, SLICE_NUMBER);
    write(
        &root,
        &format!(".doctrine/slice/{SLICE_NUMBER}/design.md"),
        body,
    );
    seed_knowledge(&root);
    (dir, root)
}

/// The same tree, with a run STARTED from the document — so the watermark
/// certifies exactly these bytes and the run is ALIGNED (`materialised = true`).
fn adopted() -> (tempfile::TempDir, PathBuf) {
    let (dir, root) = document_only(ADOPTABLE);
    let out = common::doctrine_cmd(&root)
        .args(["design", "start", SLICE, "--from-design", "-p"])
        .arg(&root)
        .output()
        .expect("spawn doctrine");
    assert!(
        out.status.success(),
        "design start --from-design failed: {}",
        stderr(&out)
    );
    (dir, root)
}

// === VT-1 — the composed read, the moved default, X2, and I1 ================

/// `EX-1`/`EX-2`: the document comes first, VERBATIM, then the knowledge block
/// under `inspect`'s own `"\nknowledge:\n"` frame.
///
/// The verbatim claim is convicted by the unheld `<!-- doctrine:section sec-9 -->`
/// marker (see [`DOCUMENT`]): it survives into stdout unchanged, which a parsing
/// reader could not manage. The inbound order is DERIVED from the run rather than
/// hand-written — cross-kind order is `EntityKey`'s `Ord` (prefix lexical, then
/// id), so `CPT` precedes `DEC` (`mem.fact.doctrine.entity-key-inbound-order`).
#[test]
fn design_show_renders_the_document_then_the_block() {
    let (_dir, root) = document_only(DOCUMENT);

    let out = ok(&root, &["--knowledge", "facets"]);

    assert!(
        out.starts_with(DOCUMENT),
        "the document must lead, byte-for-byte: {out}"
    );
    assert!(
        out.contains("<!-- doctrine:section sec-9 -->"),
        "A2: an unheld marker survives verbatim — the reader gets the document, \
         not the parser's refusal: {out}"
    );
    let frame = out
        .find("\nknowledge:\n")
        .unwrap_or_else(|| panic!("no knowledge: frame in: {out}"));
    assert!(
        frame >= DOCUMENT.len() - 1,
        "the block follows the document, never precedes it: {out}"
    );

    let block = out.split_at(frame).1;
    let cpt = block
        .find("CPT-802")
        .unwrap_or_else(|| panic!("CPT-802 absent from the block: {block}"));
    let dec = block
        .find("DEC-801")
        .unwrap_or_else(|| panic!("DEC-801 absent from the block: {block}"));
    assert!(cpt < dec, "EntityKey order: CPT precedes DEC in: {block}");
    assert!(
        block.contains("Do the thing"),
        "a non-Skip level renders the facet, not just the identity: {block}"
    );

    // A-d / I1: at the DEFAULT level the corpus is not read at all, so the frame
    // is absent — gated on the LEVEL (a property), never on the block's output
    // being empty (a proxy).
    let skipped = ok(&root, &[]);
    assert_eq!(
        skipped, DOCUMENT,
        "at --knowledge skip the document stands alone"
    );
    assert!(!skipped.contains("knowledge:"));
}

/// `EX-1`, proved as **one input read two ways**
/// (`mem.pattern.testing.additive-widening-two-readings`): on the SAME fixture the
/// bare invocation equals `--format document` and DIFFERS from `--format prompt`.
///
/// Equality is what proves the default moved; inequality is what proves it moved
/// *away*. A case asserting only "the output contains the document" would pass with
/// the default still on `prompt`.
#[test]
fn design_show_defaults_to_format_document() {
    let (_dir, root) = adopted();

    let bare = ok(&root, &[]);
    let spelled = ok(&root, &["--format", "document"]);
    let envelope = ok(&root, &["--format", "prompt"]);

    assert_eq!(bare, spelled, "the bare default IS --format document");
    assert_ne!(
        bare, envelope,
        "the default moved AWAY from the envelope rendering"
    );
    assert_eq!(bare, ADOPTABLE, "and what it renders is the document");
}

/// `EX-4`/`X2`: a slice with a run but no `design.md` gets a clean error naming
/// the repair — never an empty document.
///
/// The stderr shape is anyhow's bare-`bail!` form (`Error: <msg>\n`, no
/// `Caused by:`), and the absence of an empty-document success is asserted
/// explicitly: that IS `X2`'s whole point.
#[test]
fn design_show_on_a_slice_without_a_design_errors_cleanly() {
    let dir = tmp();
    let root = dir.path();
    std::fs::create_dir_all(root.join(".doctrine/slice").join(SLICE_NUMBER)).unwrap();

    let out = show(root, &[]);
    assert!(
        !out.status.success(),
        "a missing design.md is an error, not an empty document: {:?}",
        stdout(&out)
    );
    assert!(
        stdout(&out).is_empty(),
        "X2: nothing that could be mistaken for a document: {}",
        stdout(&out)
    );
    let err = stderr(&out);
    assert_eq!(
        err,
        format!("Error: {SLICE}: no design document (doctrine design start {SLICE})\n"),
        "a bare bail!: one line, no `Caused by:`"
    );
}

/// `EX-1`'s conservation half (`I1`): adding `document` and moving the default
/// leaves `prompt`, `json` and `status` byte-for-byte what the branch-point binary
/// emitted.
///
/// The expected bytes were captured from the PRE-change binary and cannot be
/// re-derived here, so they are constants. The run uid is the only volatile span
/// and is placeholdered. One deliberate difference is expressed rather than
/// hidden: `EX-7` re-points `delta_unavailable_line`'s repair at `--format
/// prompt`, and the substitution is asserted to have APPLIED so this golden cannot
/// go vacuously true.
///
/// Honest scope: the COMPLETE byte-identity proof is the migrated
/// `tests/e2e_design_state.rs` / `tests/e2e_design_projection.rs` suites staying
/// green under an explicit `--format prompt`. This case pins it inside one file.
#[test]
fn design_show_envelope_renderings_are_byte_identical_to_the_prior_default() {
    let fixture = design_fixture::DesignRun::start();
    let root = fixture.root.as_path();

    assert!(
        PRIOR_PROMPT.contains(PRIOR_REPAIR),
        "the branch-point capture must carry the span EX-7 moves, or the \
         substitution below proves nothing"
    );
    let expect_prompt = PRIOR_PROMPT.replace(PRIOR_REPAIR, MIGRATED_REPAIR);
    assert_ne!(
        expect_prompt, PRIOR_PROMPT,
        "the EX-7 substitution must have applied"
    );

    for (format, expected) in [
        ("prompt", expect_prompt.as_str()),
        ("json", PRIOR_JSON),
        ("status", PRIOR_STATUS),
    ] {
        let got = ok(root, &["--format", format]).replace(&fixture.uid, UID);
        assert_eq!(got, expected, "--format {format} moved");
    }

    // The other side of the same fact: the bare verb no longer renders the
    // envelope at all. This run has no design.md, so it reaches X2.
    let bare = show(root, &[]);
    assert!(
        !bare.status.success(),
        "the bare verb is the document read now: {}",
        stdout(&bare)
    );
}

// === VT-2 — the flag partition (§5.2 / D7) =================================

/// `EX-3`: `--json` belongs to the document rendering, and the refusal is stated
/// over the RENDERING rather than over how the rendering was spelled.
///
/// Three legs, and the middle one is what matters: `--json --format document` must
/// be LEGAL and byte-identical to bare `--json`. A case that only checked the
/// refusal would also pass under the rejected "refuse `--json` with an explicit
/// `--format`" rule.
#[test]
fn json_is_legal_at_the_document_rendering_and_refused_at_every_envelope_one() {
    let (_dir, root) = adopted();

    let defaulted = ok(&root, &["--json"]);
    let spelled = ok(&root, &["--json", "--format", "document"]);
    assert_eq!(
        defaulted, spelled,
        "defaulted and written-out `document` are ONE rendering"
    );

    let v: serde_json::Value = serde_json::from_str(&defaulted).expect("valid JSON");
    assert_eq!(
        v.get("kind").and_then(serde_json::Value::as_str),
        Some("design")
    );
    assert_eq!(
        v.get("document").and_then(serde_json::Value::as_str),
        Some(ADOPTABLE)
    );
    assert!(
        v.get("knowledge").is_none(),
        "at the default level the key is ABSENT, never null or []: {v}"
    );

    for format in ["prompt", "json", "status"] {
        refused(&root, &["--json", "--format", format], "--json");
    }
}

/// `EX-3`: `--knowledge` is the document's other flag, refused at every envelope
/// rendering — and the refusal fires on a **non-`Skip` LEVEL**, not on the flag
/// being present, because `skip` is the default and a defaulted value cannot be an
/// error.
///
/// The `--knowledge skip --format prompt` leg is the discriminator: without it the
/// case cannot tell "refused the flag" from "refused the level".
#[test]
fn knowledge_with_an_envelope_rendering_is_refused() {
    let (_dir, root) = adopted();

    for format in ["prompt", "json", "status"] {
        refused(
            &root,
            &["--knowledge", "facets", "--format", format],
            "--knowledge",
        );
        refused(
            &root,
            &["--knowledge", "full", "--format", format],
            "--knowledge",
        );
    }

    // The default level is not an error at ANY rendering, explicit or not.
    for format in ["prompt", "json", "status"] {
        let explicit = ok(&root, &["--knowledge", "skip", "--format", format]);
        let implicit = ok(&root, &["--format", format]);
        assert_eq!(
            explicit, implicit,
            "--knowledge skip is byte-identical to no flag at --format {format}"
        );
    }

    // And a non-Skip level IS legal on the document side of the partition.
    let block = ok(&root, &["--knowledge", "facets"]);
    assert!(block.contains("\nknowledge:\n"), "{block}");
}

/// `EX-3`: `--full` widens the turn-envelope projection, so it is refused at
/// `document` (defaulted or spelled) and legal at all three envelope renderings.
///
/// The legal half is the anti-vacuity control: a refusal case alone cannot
/// distinguish "refused in the right place" from "refused everywhere".
#[test]
fn full_is_refused_on_the_document_and_legal_on_every_envelope_rendering() {
    let (_dir, root) = adopted();

    refused(&root, &["--full"], "--full");
    refused(&root, &["--full", "--format", "document"], "--full");

    for format in ["prompt", "json", "status"] {
        let _ = ok(&root, &["--full", "--format", format]);
    }
}

/// `EX-3`: `--known-revision` selects what the envelope is diffed against, so it
/// takes `--full`'s rule unchanged. Same two halves, same reason.
#[test]
fn known_revision_is_refused_on_the_document_the_way_full_is() {
    let (_dir, root) = adopted();

    refused(&root, &["--known-revision", "1"], "--known-revision");
    refused(
        &root,
        &["--known-revision", "1", "--format", "document"],
        "--known-revision",
    );

    for format in ["prompt", "json", "status"] {
        let _ = ok(&root, &["--known-revision", "1", "--format", format]);
    }
}

/// `EX-3`'s qualifier: `--path` is outside the partition **by kind, not by
/// omission** — it selects no content and no projection, and sits on every `Args`
/// struct in this codebase.
///
/// This is the case that stops a later reading of "each flag belongs to one
/// rendering" from refusing a flag every other verb accepts (`F-27`: three design
/// rounds miscounted `ShowArgs` here).
#[test]
fn path_is_legal_at_every_rendering() {
    let (_dir, root) = adopted();

    for format in ["document", "prompt", "json", "status"] {
        // `show` already appends `-p <root>`; assert the long spelling too, so the
        // qualifier is proved for both forms the struct accepts.
        let short = ok(&root, &["--format", format]);
        let long = common::doctrine_cmd(&root)
            .args(["design", "show", SLICE, "--format", format, "--path"])
            .arg(&root)
            .output()
            .expect("spawn doctrine");
        assert!(
            long.status.success(),
            "--path at --format {format}: {}",
            stderr(&long)
        );
        assert_eq!(stdout(&long), short, "-p and --path are one flag");
    }
}

// === VT-3 — OQ-1 / QUE-223 on both arms ====================================

/// `EX-5`: a `design.md` that is not the document its run last materialised is
/// disclosed on BOTH arms — and `D1` binds them to one MEANING, not one string.
///
/// The divergence is driven the supported way: adopt the document (so the
/// watermark certifies it), then hand-edit it. The negative control — the aligned
/// tree emitting NEITHER disclosure — is what makes the positive legs mean
/// something.
#[test]
fn a_document_behind_its_run_is_disclosed_on_both_arms() {
    let (_dir, root) = adopted();

    // Negative control FIRST, on the aligned tree.
    let aligned_text = ok(&root, &[]);
    assert!(
        !aligned_text.contains("note:"),
        "an aligned document is not behind anything: {aligned_text}"
    );
    let aligned_json: serde_json::Value =
        serde_json::from_str(&ok(&root, &["--json"])).expect("valid JSON");
    assert!(
        aligned_json.get("stale_run").is_none(),
        "no key when aligned: {aligned_json}"
    );

    // Now diverge: the bytes on disk stop being the bytes the watermark certifies.
    let edited = "# Design — adoptable fixture\n\nA hand-edited body line.\n";
    write(
        &root,
        &format!(".doctrine/slice/{SLICE_NUMBER}/design.md"),
        edited,
    );

    let text = ok(&root, &[]);
    let note = text
        .lines()
        .next()
        .expect("the note is the first line, ABOVE the document");
    assert!(
        note.starts_with("note: design.md is behind its run (revision 1)"),
        "the text arm names the revision gap: {text}"
    );
    assert!(
        note.contains(&format!("doctrine design materialise {SLICE}")),
        "and the repair: {text}"
    );
    assert!(
        text.ends_with(edited),
        "the document still follows, verbatim: {text}"
    );

    let v: serde_json::Value = serde_json::from_str(&ok(&root, &["--json"])).expect("valid JSON");
    let stale = v
        .get("stale_run")
        .unwrap_or_else(|| panic!("no stale_run on the document object: {v}"));
    assert!(
        stale.is_object(),
        "D1: the JSON arm carries the STRUCTURE, never the rendered sentence: {stale}"
    );
    assert_eq!(
        stale
            .get("run_revision")
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );
    // `document_revision` is `null` BY CONSTRUCTION, not as a placeholder: in both
    // branches of the staleness predicate the document on disk has no revision —
    // either Doctrine never wrote it, or what is there is not Doctrine's render.
    assert!(
        stale
            .get("document_revision")
            .is_some_and(serde_json::Value::is_null),
        "document_revision is present and null: {stale}"
    );
    assert!(
        !v.to_string().contains("is behind its run"),
        "the JSON arm does not carry the text arm's sentence: {v}"
    );
}
