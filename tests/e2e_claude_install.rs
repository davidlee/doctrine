// SPDX-License-Identifier: GPL-3.0-only
//! SL-088 PHASE-04 — `doctrine install --agent claude` end-to-end over the built binary.
//!
//! Drives the consolidated `doctrine install` handler against a temp project and
//! proves the Claude-surface install (design §9):
//!   * VT-1: `install --agent claude --skill code-review` wires skills + agent def.
//!   * VT-2: the dispatch-worker agent def resolves at `.claude/agents/`.
//!   * SL-152: Claude hooks now ship as a skills-directory plugin — `doctrine install`
//!     copies `.claude-plugin/plugin.json` + `hooks/` directly into `.claude/skills/doctrine/`,
//!     so Claude auto-discovers them with no marketplace install step. No SessionStart,
//!     WorktreeCreate, or retired SubagentStart hook is settings-wired.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::fs;
use std::path::Path;

use serde_json::Value;

mod common;
mod design_fixture;

/// Run `doctrine install --agent claude --skill code-review` rooted at `dir`,
/// asserting success; return stdout.
fn install(dir: &Path) -> String {
    let out = common::doctrine_cmd(dir)
        .args([
            "install",
            "--agent",
            "claude",
            "--skill",
            "code-review",
            "--yes",
            "-p",
        ])
        .arg(dir)
        .output()
        .expect("spawn doctrine");
    assert!(
        out.status.success(),
        "install failed: {}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

/// The `hooks.<event>` array of a settings file (empty if the file or event is
/// absent — SL-152 PHASE-06: with no hooks settings-wired the file may not exist).
fn event_entries(settings: &Path, event: &str) -> Vec<Value> {
    let Ok(json) = fs::read_to_string(settings) else {
        return Vec::new();
    };
    let value: Value = serde_json::from_str(&json).expect("valid settings JSON");
    value
        .get("hooks")
        .and_then(|h| h.get(event))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

/// SL-233 PHASE-07 — the five new `install/` assets, by publication address.
/// The sealed invariant hymn takes the established `hymns/*` prefix; the four
/// process fragments take `design-prompts/` because they are deliberately NOT
/// hymns (design.md §7), and a publication address is a logical contract free to
/// diverge from its backing key.
/// PHASE-08 added the three remaining obligation runbooks, so every forward edge
/// of the stage machine now publishes both asset kinds — the `*.md` framing that
/// rides the stage and the `*.toml` acts that end it.
const PUBLISHED_DESIGN_ASSETS: &[&str] = &[
    "hymns/stage/design.md",
    "design-prompts/inquiry.md",
    "design-prompts/drafting.md",
    "design-prompts/reviewing.md",
    "design-prompts/delegation.md",
    "design-prompts/exploring.toml",
    "design-prompts/inquiring.toml",
    "design-prompts/drafting.toml",
    "design-prompts/reviewing.toml",
];

/// A stable phrase from the sealed invariant hymn's body. Asserted rather than a
/// byte-golden so the hymn's prose can be edited without breaking the seal tests.
const SEALED_HYMN_MARKER: &str = "Design stage invariants";

/// An `expose`d hymn slot, used as the POSITIVE CONTROL for the seal assertions:
/// install projects exposed slots as editable starters, so a sealed slot's
/// absence is attributable to the seal rather than to install projecting no
/// hymns at all.
const EXPOSED_HYMN_PROJECTION: &str = ".doctrine/hymns/role/worker.md";

/// `doctrine publication validate` stdout — one `ok <address> -> <backing>` row
/// per declared entry.
fn publication_validate() -> String {
    let out = common::doctrine_cmd(&common::repo_root())
        .args(["publication", "validate"])
        .output()
        .expect("spawn doctrine");
    assert!(
        out.status.success(),
        "publication validate failed: {}",
        String::from_utf8_lossy(&out.stderr),
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

/// The `stage` band alone, resolved for `stage` in the project rooted at `dir`.
///
/// Band-scoped so the assertion is about the stage slot and nothing else, and
/// read at `resolve` — never `explain`, which is a pre-suppression diagnostic
/// that shows both twins ranked and would prove nothing about the seal.
fn resolved_stage_band(dir: &Path, stage: &str) -> String {
    let out = common::doctrine_cmd(dir)
        .args([
            "prompt",
            "resolve",
            "--role",
            "orchestrator",
            "--stage",
            stage,
            "--band",
            "stage",
            "-p",
        ])
        .arg(dir)
        .output()
        .expect("spawn doctrine");
    assert!(
        out.status.success(),
        "prompt resolve failed: {}",
        String::from_utf8_lossy(&out.stderr),
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

/// Assert the post-install state holds for a project at `dir`: the agent def
/// resolves, and NO Claude hooks are settings-wired (they ship via the plugin —
/// SL-152 PHASE-06).
fn assert_installed(dir: &Path) {
    // VT-2: the agent def is a link resolving to materialised content.
    let agent_link = dir.join(".claude/agents/dispatch-worker.md");
    assert!(
        fs::symlink_metadata(&agent_link)
            .unwrap()
            .file_type()
            .is_symlink(),
        "agent path is a symlink"
    );
    let body = fs::read_to_string(&agent_link).expect("agent def resolves");
    assert!(
        body.contains("dispatch worker"),
        "agent link resolves to the dispatch-worker def: {body:.80}"
    );
    assert!(
        dir.join(".doctrine/agents/dispatch-worker.md").is_file(),
        "canonical agent def materialised"
    );

    // SL-152 PHASE-06: no hooks are settings-wired — they ship via the doctrine
    // plugin. The boot (SessionStart) and create-fork (WorktreeCreate) hooks, plus
    // the retired SubagentStart stamp, are all absent (settings file may carry only
    // baseRef, or not exist at all). `event_entries` treats absent-file as empty.
    let settings = dir.join(".claude/settings.local.json");
    assert!(
        event_entries(&settings, "WorktreeCreate").is_empty(),
        "no WorktreeCreate hook settings-wired (ships via plugin)"
    );
    assert!(
        event_entries(&settings, "SessionStart").is_empty(),
        "no SessionStart boot hook settings-wired (ships via plugin)"
    );
    assert!(
        event_entries(&settings, "SubagentStart").is_empty(),
        "no SubagentStart stamp hook after retirement"
    );
}

#[test]
fn install_wires_skills_agent_and_hooks_directly() {
    if common::under_worker_marker() {
        return;
    } // SL-225 #2: skip in a worker fork
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();

    let out = install(dir);
    // IMP-223: skills + hooks now via claude plugin commands. The outcome
    // depends on whether `claude` is on PATH; assert only invariants that
    // hold regardless.
    assert!(
        out.contains("register marketplace + install plugin + agent def for claude"),
        "forward summary mentions plugin path: {out}"
    );
    // Either the reminder (claude absent/skipped) or the commands ran.
    // We don't assert on specific plugin output — environment-dependent.
    // Agent def still installed manually.
    assert!(
        out.contains("linked    dispatch-worker.md"),
        "agents leg: {out}"
    );
    // Old-style hooks/skills copypasta gone.
    assert!(
        !out.contains("hooks (skills-dir plugin):"),
        "no old-style hooks header: {out}"
    );
    assert!(
        !out.contains("linked    code-review"),
        "no old-style skills symlink: {out}"
    );
    assert_installed(dir);
}

#[test]
fn install_agent_pi_dry_run_prints_delegation_plan() {
    if common::under_worker_marker() {
        return;
    } // SL-225 #2: skip in a worker fork
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();

    let out = common::doctrine_cmd(dir)
        .args(["install", "--agent", "pi", "--dry-run", "-p"])
        .arg(dir)
        .output()
        .expect("spawn doctrine");
    assert!(
        out.status.success(),
        "install --agent pi --dry-run failed: {}",
        String::from_utf8_lossy(&out.stderr),
    );
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(
        stdout.contains("pi"),
        "pi agent mentioned in plan: {stdout}"
    );
    assert!(
        stdout.contains("delegates to npx"),
        "npx delegation shown: {stdout}"
    );
    assert!(
        stdout.contains("not executed"),
        "dry-run indicator present: {stdout}"
    );
    // Dry-run must NOT create any files beyond what the temp dir started with.
    assert!(
        !dir.join(".doctrine").exists(),
        "dry-run created no .doctrine dir"
    );
    assert!(!dir.join(".pi").exists(), "dry-run created no .pi dir");
}

/// SL-233 PHASE-07 EX-1 / EX-3 / EX-4 — the sealed invariant hymn and the four
/// closed process fragments ship through the embed and publication surfaces; the
/// sealed slot is not projected as user-editable content; and a user twin at the
/// sealed slot is dropped at resolution.
#[test]
fn sealed_design_hymn_and_four_fragments_ship_installed() {
    if common::under_worker_marker() {
        return;
    } // SL-225 #2: skip in a worker fork

    // EX-3: all five assets carry explicit publication library addresses.
    // Reported as the missing SET, not by dumping 74 declared rows per failure.
    let published = publication_validate();
    let missing: Vec<&&str> = PUBLISHED_DESIGN_ASSETS
        .iter()
        .filter(|address| !published.contains(**address))
        .collect();
    assert!(
        missing.is_empty(),
        "publication manifest is missing library addresses for {missing:?}"
    );

    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    install(dir);

    // EX-4 (projection half): a SEALED slot is not projected as editable content.
    assert!(
        !dir.join(".doctrine/hymns/stage/design.md").exists(),
        "sealed stage/design is not projected as user-editable content"
    );
    // POSITIVE CONTROL for the assertion above: install DOES project exposed
    // slots, so the absence is the seal's doing and not an empty projection.
    assert!(
        dir.join(EXPOSED_HYMN_PROJECTION).is_file(),
        "exposed hymn slots are still projected ({EXPOSED_HYMN_PROJECTION}) — \
         without this the sealed-slot assertion above is vacuous"
    );

    // EX-2: the fragment store is closed and code-owned — no user override, so
    // nothing is projected for a user to edit.
    assert!(
        !dir.join(".doctrine/design-prompts").exists(),
        "the process-fragment store is code-owned, never projected"
    );

    // EX-1 (embed half): the sealed hymn resolves in the stage band.
    let framework = resolved_stage_band(dir, "design");
    assert!(
        framework.contains(SEALED_HYMN_MARKER),
        "sealed stage/design hymn resolves from the embed: {framework}"
    );

    // EX-4 (seal half): a user-provenance twin at the sealed slot is dropped
    // BEFORE matching — framework content wins by active exclusion.
    let twin_dir = dir.join(".doctrine/hymns/stage");
    fs::create_dir_all(&twin_dir).expect("create the twin's dir");
    fs::write(
        twin_dir.join("design.md"),
        "# Usurper\n\nUSURPING-TWIN-SENTINEL\n",
    )
    .expect("write the user twin");
    let after_twin = resolved_stage_band(dir, "design");
    assert!(
        !after_twin.contains("USURPING-TWIN-SENTINEL"),
        "the sealed slot drops its user twin: {after_twin}"
    );
    assert!(
        after_twin.contains(SEALED_HYMN_MARKER),
        "the framework body is still emitted after the twin is dropped: {after_twin}"
    );
}

// ── blast radius (EX-8 test 2, EX-5, VA-2) ────────────────────────────────

/// The tree the blast-radius enumeration walks: everything that ships or runs.
///
/// `.doctrine/**` is excluded deliberately. The slice's own design, plan, notes
/// and review prose *describe* this fork at length; prose about a mechanism is
/// not a consumer of it, and including it would make the bound churn on every
/// authored edit while proving nothing about blast radius. `target/**` is build
/// output. Everything else a release could carry is in scope.
const CONSUMER_DOMAIN: &[&str] = &[
    "src",
    "tests",
    "install",
    "publication",
    "plugins",
    "web",
    "scripts",
    "templates",
    "examples",
    "crates",
    "memory",
    "docs",
];

/// Every repo-relative path under [`CONSUMER_DOMAIN`] whose text contains
/// `needle`. Unreadable (non-UTF8) files cannot mention it and are skipped.
fn files_mentioning(needle: &str) -> std::collections::BTreeSet<String> {
    files_mentioning_under(CONSUMER_DOMAIN, needle)
}

/// [`files_mentioning`] over an explicit set of repo-relative roots, so a sweep
/// can be narrowed to the guidance tree without a second walker (STD-001).
fn files_mentioning_under(dirs: &[&str], needle: &str) -> std::collections::BTreeSet<String> {
    let root = common::repo_root();
    let mut found = std::collections::BTreeSet::new();
    let mut pending: Vec<std::path::PathBuf> = dirs.iter().map(|dir| root.join(dir)).collect();

    while let Some(path) = pending.pop() {
        let meta = match fs::symlink_metadata(&path) {
            Ok(meta) => meta,
            Err(_) => continue,
        };
        if meta.is_symlink() {
            continue; // never follow: a slug symlink would double-count (and can loop)
        }
        if meta.is_dir() {
            let Ok(entries) = fs::read_dir(&path) else {
                continue;
            };
            pending.extend(
                entries
                    .filter_map(|entry| entry.ok())
                    .map(|entry| entry.path()),
            );
            continue;
        }
        if fs::read_to_string(&path).is_ok_and(|text| text.contains(needle)) {
            let rel = path.strip_prefix(&root).unwrap_or(&path);
            found.insert(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    found
}

/// SL-233 PHASE-07 EX-5 / EX-8 / VA-2 — the fragment store and its wire form are
/// bounded by enumeration, not by intention. Set equality against a named
/// allowlist, repo-wide: a new consumer anywhere in the shipped tree fails this
/// test and has to be justified by editing the allowlist deliberately.
#[test]
fn design_prompts_have_no_consumer_outside_the_design_run() {
    // Each entry is a deliberate consumer, and the reason it is one. A fourth
    // file appearing here is the cascade fork spreading — the thing EX-5 bounds.
    let store_allowlist: std::collections::BTreeSet<String> = [
        // The catalogue that OWNS the store name: one `STORE` const the four
        // asset keys derive from (STD-001).
        "src/design_run/prompt.rs",
        // EX-3: the five assets' publication library addresses, plus SL-244
        // PHASE-06's nine narrative contracts under `conditions/`.
        "publication/manifest.toml",
        // This file — the assertions below, and the published-address table.
        "tests/e2e_claude_install.rs",
        // SL-244 PHASE-06 `VT-1`: the corpus set-equality gate lives here, and
        // the criterion's keyword mandate names the prefix in raw bytes. The
        // filter's *value* is derived from `prompt::contract_store()`, so this
        // is a prose mention and not a second source (STD-001).
        "src/commands/design.rs",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();

    // EX-5's second half: `name@digest` belongs to the design run's emission and
    // its grammar, and nowhere else. Not `install`, not the hymn cascade.
    let wire_allowlist: std::collections::BTreeSet<String> = [
        // The grammar: emitted form and `parse_receipt`, one separator const.
        "src/design_run/prompt.rs",
        // The single emission site — `design resume`'s `fragment_section`.
        "src/commands/design.rs",
        // This file — the assertions below.
        "tests/e2e_claude_install.rs",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();

    assert_eq!(
        files_mentioning("design-prompts"),
        store_allowlist,
        "the fragment store has exactly the consumers the allowlist names"
    );
    assert_eq!(
        files_mentioning("name@digest"),
        wire_allowlist,
        "the receipt wire form appears in exactly the files the allowlist names"
    );
}

// ── fragment receipts (EX-8 test 3, VA-4) ─────────────────────────────────

/// The fragment a freshly started run obliges — `design start` lands in
/// `exploring`, which `Fragment::for_stage` maps to inquiry.
const FRAGMENT: &str = "inquiry";

/// A phrase from `install/design-prompts/inquiry.md`, asserted by phrase rather
/// than byte-golden so the guidance prose stays editable (the
/// `SEALED_HYMN_MARKER` convention above).
const FRAGMENT_BODY_MARKER: &str = "Shape the question space before drafting";

/// A syntactically valid digest that is not the asset's — the stale receipt.
const STALE_DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// The digest carried by the `fragment <name>@<digest>` header, which is the
/// only place a caller can learn it — no test re-types a digest (STD-001).
fn emitted_fragment_digest(out: &str, name: &str) -> String {
    out.lines()
        .find_map(|line| line.strip_prefix(&format!("fragment {name}@")))
        .unwrap_or_else(|| panic!("resume names the {name} fragment and its digest:\n{out}"))
        .trim()
        .to_owned()
}

/// VA-4's third assertion: the receipt machinery never costs the caller the
/// TurnEnvelope. Checked in every case, because an envelope that disappears when
/// a body is omitted breaks recovery exactly when recovery is needed.
fn assert_envelope_present(out: &str, case: &str) {
    for field in ["active_path", "open_questions", "next_obligation"] {
        assert!(
            out.lines().any(|line| line.starts_with(field)),
            "the TurnEnvelope is still projected ({case}) — `{field}` missing from:\n{out}"
        );
    }
}

/// SL-233 PHASE-07 EX-8 / VA-4 — a fragment the caller already holds is bound by
/// digest, not re-sent; a receipt whose digest has gone stale is re-sent; and the
/// TurnEnvelope rides in every case.
#[test]
fn known_fragment_receipt_omits_body_stale_receipt_reemits() {
    let fixture = design_fixture::DesignRun::start();

    // Cold: no receipt declared, so the body rides and names its own digest.
    let cold = fixture.resume(&[]);
    let digest = emitted_fragment_digest(&cold, FRAGMENT);
    assert!(
        cold.contains(FRAGMENT_BODY_MARKER),
        "with no receipt the fragment body is emitted whole:\n{cold}"
    );
    assert_envelope_present(&cold, "no receipt");

    // A CURRENT receipt: the header stays — identity is never elided — and the
    // body is omitted.
    let current = format!("{FRAGMENT}@{digest}");
    let held = fixture.resume(&["--known-fragment", &current]);
    // LINE-ANCHORED, and it must stay that way: `fragment {current}` also occurs
    // as a substring of the `known_fragment {current} …` line that
    // `fragment_lines` emits unconditionally, so a bare `contains` here measures
    // the wrong producer and cannot fail whatever `fragment_section` does.
    let identified = format!("fragment {current}");
    assert!(
        held.lines().any(|line| line.starts_with(&identified)),
        "the fragment is still identified when its body is withheld:\n{held}"
    );
    assert!(
        !held.contains(FRAGMENT_BODY_MARKER),
        "a CURRENT receipt omits the body rather than re-sending it:\n{held}"
    );
    assert_envelope_present(&held, "current receipt");

    // A STALE receipt: the caller's bytes no longer match, so they are re-sent.
    // Without this leg, omission-only passes and silently breaks recovery.
    let stale = fixture.resume(&["--known-fragment", &format!("{FRAGMENT}@{STALE_DIGEST}")]);
    assert!(
        stale.contains(FRAGMENT_BODY_MARKER),
        "a STALE receipt re-emits the body:\n{stale}"
    );
    assert_envelope_present(&stale, "stale receipt");
}

// ── core-process guidance (SL-233 PHASE-08 EX-2 / EX-4 / EX-9) ─────────────

/// The paragraph opener the core process is authored under, in the shipped asset
/// (`install/routing-process.md`) and in every projection generated from it.
const CORE_PROCESS_MARKER: &str = "**Core process:**";

/// The retired design-stage verb, in its INVOCATION form. Backticked on purpose:
/// EX-4 governs what the guidance tells an agent to *run*, so the noun phrase
/// "per-slice design" is not a hit and needs no false-positive allowlist.
const RETIRED_DESIGN_VERB: &str = "`slice design`";

/// A backticked invocation the guidance sweep must be able to find. Without it
/// the absence assertion could pass on a broken walker. Chosen to share the
/// retired verb's SHAPE — a bare, unprefixed `` `<command> <verb>` `` span —
/// so the control exercises the same match the assertion relies on.
const LIVE_INVOCATION_CONTROL: &str = "`backlog list`";

/// The tree EX-4 governs: shipped reference assets and shipped skill bodies.
const GUIDANCE_DOMAIN: &[&str] = &["install", "plugins"];

/// The `**Core process:**` paragraph of `text` — marker to the blank line that
/// ends it. Panics rather than yielding `None`: a projection carrying no core
/// process is a broken fixture, and a test that shrugs at that proves nothing.
fn core_process_paragraph(text: &str, source: &str) -> String {
    let from = text
        .find(CORE_PROCESS_MARKER)
        .unwrap_or_else(|| panic!("{source} carries a {CORE_PROCESS_MARKER} paragraph:\n{text}"));
    let rest = &text[from..];
    let end = rest.find("\n\n").unwrap_or(rest.len());
    rest[..end].to_owned()
}

/// Install into `dir`, regenerate the boot snapshot from the embed, return its
/// text. This is the GENERATED surface: `doctrine boot` inlines
/// `routing-process.md` out of the binary's own embed, so what it writes is what
/// a project's agents actually read — not what the repo's working copy says.
fn generated_boot_snapshot(dir: &Path) -> String {
    install(dir);
    let out = common::doctrine_cmd(dir)
        .args(["boot", "-p"])
        .arg(dir)
        .output()
        .expect("spawn doctrine boot");
    assert!(
        out.status.success(),
        "boot failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    fs::read_to_string(dir.join(".doctrine/state/boot.md")).expect("boot wrote the snapshot")
}

/// The doctrine invocations a core-process paragraph names, as argv tails.
///
/// The parser owns no vocabulary of commands — it reads what the text declares.
/// The paragraph's convention is that every command it teaches is written as a
/// full `` `doctrine …` `` span, so the extraction rule is that convention and
/// nothing else. Placeholders (`<id>`) are dropped: they stand for an argument
/// the reader supplies, not for a subcommand the binary must know.
fn invocations_named_by(paragraph: &str) -> Vec<Vec<String>> {
    paragraph
        .split('`')
        .skip(1)
        .step_by(2)
        .filter_map(|span| span.strip_prefix("doctrine "))
        .map(|rest| {
            rest.split_whitespace()
                .filter(|word| !(word.starts_with('<') && word.ends_with('>')))
                .map(str::to_owned)
                .collect()
        })
        .collect()
}

/// SL-233 PHASE-08 EX-4 / EX-9 — `slice design` is no longer advertised as the
/// canonical design-stage verb, in shipped guidance or in what `doctrine boot`
/// generates from it. Deprecation documentation may still name the shim, but
/// only by being added to the allowlist deliberately.
#[test]
fn no_shipped_guidance_advertises_slice_design_as_canonical() {
    // Every deliberate naming of the retired verb, and why it is one. Empty
    // today: no shipped guidance documents the shim.
    let deprecation_allowlist = std::collections::BTreeSet::<String>::new();

    assert_eq!(
        files_mentioning_under(GUIDANCE_DOMAIN, RETIRED_DESIGN_VERB),
        deprecation_allowlist,
        "shipped guidance invokes `slice design` only where the allowlist says \
         it is documenting the deprecation"
    );

    // POSITIVE CONTROL: the sweep does find backticked invocations that are
    // present, so the emptiness above is the verb's absence, not the walker's.
    assert!(
        !files_mentioning_under(GUIDANCE_DOMAIN, LIVE_INVOCATION_CONTROL).is_empty(),
        "the guidance sweep finds {LIVE_INVOCATION_CONTROL}, which IS shipped — \
         without this control the assertion above is vacuous"
    );

    let tmp = tempfile::tempdir().expect("tempdir");
    let snapshot = generated_boot_snapshot(tmp.path());
    let core = core_process_paragraph(&snapshot, "the generated boot snapshot");
    assert!(
        !core.contains(RETIRED_DESIGN_VERB),
        "the generated core process does not teach `slice design`:\n{core}"
    );
}

/// SL-233 PHASE-08 EX-2 / EX-9 — every command the generated core process names
/// is accepted by the binary that generated it. Parsed out of the text at
/// runtime: a test enumerating the commands in its own source is a hardcoded
/// list wearing a parser's name, and drifts the moment the sentence changes.
#[test]
fn every_command_named_by_core_process_is_accepted_by_the_binary() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let snapshot = generated_boot_snapshot(tmp.path());
    let core = core_process_paragraph(&snapshot, "the generated boot snapshot");

    let invocations = invocations_named_by(&core);
    // ANTI-VACUITY. A parser that extracts nothing satisfies the loop below
    // without executing anything — the same defect as the hardcoded list, one
    // step further on. The floor is the canonical family the sentence teaches.
    assert!(
        invocations.len() >= 4,
        "the core process teaches the canonical command family, but only {} \
         invocation(s) parsed out of:\n{core}",
        invocations.len()
    );

    for argv in &invocations {
        let out = common::doctrine_cmd(tmp.path())
            .args(argv)
            .arg("--help")
            .output()
            .expect("spawn doctrine");
        assert!(
            out.status.success(),
            "the core process names `doctrine {}`, which the built binary \
             refuses:\n{}",
            argv.join(" "),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}
