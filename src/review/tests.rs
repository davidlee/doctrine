// SPDX-License-Identifier: GPL-3.0-only
//! The `review` command-tier unit tests: 97 as of SL-268 PHASE-02 `T4`,
//! unchanged in substance from `src/review.rs` at `a3bd0764c` (only `use` lines
//! and module paths differed there); 105 as of PHASE-03, which added the
//! fail-safe-read and disclosure tests (`T1`–`T6`).

use super::*;
use super::{prime::*, read::*, turn::*, verbs::*};
use crate::review_ledger::{
    BlockerRef, DISPOSITIONS, EFFECT_UNKNOWN_SEVERITY, FACETS, FINDING_STATUSES, OutstandingCounts,
    ROLES, ROUTES, SEVERITIES, Vocab, VocabDefect, VocabField, gates_as_blocker, next_finding_id,
    observe_pass, outstanding_by_severity, read_pass_facts, undisposed_blockers,
    unresolved_blockers_for, vocabulary_defects,
};

// -- derived_status: total + named cases (VT-1 / VT-2) -------------------

fn states(statuses: &[FindingStatus]) -> Vec<FindingState> {
    statuses
        .iter()
        .map(|&status| FindingState {
            status: Vocab::Known(status),
        })
        .collect()
}

#[test]
fn derived_status_empty_is_done_none() {
    assert_eq!(derived_status(&[]), (ReviewStatus::Done, Await::None));
}

#[test]
fn derived_status_any_open_or_contested_is_active_responder() {
    assert_eq!(
        derived_status(&states(&[FindingStatus::Open])),
        (ReviewStatus::Active, Await::Responder)
    );
    assert_eq!(
        derived_status(&states(&[FindingStatus::Contested])),
        (ReviewStatus::Active, Await::Responder)
    );
    // open + answered ⇒ open wins ⇒ Responder.
    assert_eq!(
        derived_status(&states(&[FindingStatus::Answered, FindingStatus::Open])),
        (ReviewStatus::Active, Await::Responder)
    );
}

#[test]
fn derived_status_answered_and_none_open_is_active_raiser() {
    assert_eq!(
        derived_status(&states(&[FindingStatus::Answered])),
        (ReviewStatus::Active, Await::Raiser)
    );
    // answered + a terminal one, none open ⇒ Raiser.
    assert_eq!(
        derived_status(&states(&[FindingStatus::Answered, FindingStatus::Verified])),
        (ReviewStatus::Active, Await::Raiser)
    );
}

#[test]
fn derived_status_all_terminal_is_done_none() {
    assert_eq!(
        derived_status(&states(&[
            FindingStatus::Verified,
            FindingStatus::Withdrawn
        ])),
        (ReviewStatus::Done, Await::None)
    );
    assert_eq!(
        derived_status(&states(&[FindingStatus::Verified])),
        (ReviewStatus::Done, Await::None)
    );
    assert_eq!(
        derived_status(&states(&[FindingStatus::Withdrawn])),
        (ReviewStatus::Done, Await::None)
    );
}

/// VT-1: total over the enum — every combination of up to two statuses
/// yields a `(ReviewStatus, Await)` without panic or gap.
#[test]
fn derived_status_total_over_enum() {
    let all = [
        FindingStatus::Open,
        FindingStatus::Answered,
        FindingStatus::Contested,
        FindingStatus::Verified,
        FindingStatus::Withdrawn,
    ];
    // Singletons and every ordered pair.
    for &a in &all {
        let _single = derived_status(&states(&[a]));
        for &b in &all {
            let (status, awaited) = derived_status(&states(&[a, b]));
            // The invariant the carrier must always hold: Done ⇔ None.
            assert_eq!(
                status == ReviewStatus::Done,
                awaited == Await::None,
                "Done iff await=None for [{}, {}]",
                a.as_str(),
                b.as_str()
            );
        }
    }
}

// -- can(): single-owner edges (VT-3) -----------------------------------

#[test]
fn can_valid_single_owner_edges_pass() {
    use FindingStatus::{Answered, Contested, Open, Verified};
    assert!(can(Act::Raise, None, Role::Raiser));
    assert!(can(Act::Dispose, Some(Open), Role::Responder));
    assert!(can(Act::Dispose, Some(Contested), Role::Responder));
    assert!(can(Act::Amend, Some(Answered), Role::Responder));
    assert!(can(Act::Verify, Some(Answered), Role::Raiser));
    assert!(can(Act::Contest, Some(Answered), Role::Raiser));
    assert!(can(Act::Reopen, Some(Verified), Role::Raiser));
    assert!(can(Act::Withdraw, Some(Open), Role::Raiser));
    assert!(can(Act::Withdraw, Some(Answered), Role::Raiser));
}

#[test]
fn can_wrong_role_refused() {
    use FindingStatus::{Answered, Open, Verified};
    // dispose/amend are the responder's; the raiser may not.
    assert!(!can(Act::Dispose, Some(Open), Role::Raiser));
    assert!(!can(Act::Amend, Some(Answered), Role::Raiser));
    // verify is the raiser's; the responder may not.
    assert!(!can(Act::Verify, Some(Answered), Role::Responder));
    // reopen is the raiser's; the responder may not.
    assert!(!can(Act::Reopen, Some(Verified), Role::Responder));
    // raise is the raiser's.
    assert!(!can(Act::Raise, None, Role::Responder));
}

#[test]
fn can_wrong_from_state_refused() {
    use FindingStatus::{Answered, Open, Verified, Withdrawn};
    // dispose only from open|contested.
    assert!(!can(Act::Dispose, Some(Answered), Role::Responder));
    // amend only from answered.
    assert!(!can(Act::Amend, Some(Open), Role::Responder));
    // verify/contest only from answered.
    assert!(!can(Act::Verify, Some(Open), Role::Raiser));
    assert!(!can(Act::Contest, Some(Open), Role::Raiser));
    // reopen only from verified.
    assert!(!can(Act::Reopen, Some(Answered), Role::Raiser));
    // withdraw only from open|answered, not contested/terminal.
    assert!(!can(
        Act::Withdraw,
        Some(FindingStatus::Contested),
        Role::Raiser
    ));
    // nothing fires on a terminal finding.
    assert!(!can(Act::Verify, Some(Verified), Role::Raiser));
    assert!(!can(Act::Dispose, Some(Withdrawn), Role::Responder));
    // raise requires a fresh finding (None), never an existing one.
    assert!(!can(Act::Raise, Some(Open), Role::Raiser));
}

// -- enum ↔ array drift canaries (VT-4) ---------------------------------

#[test]
fn facet_known_set_matches_variants() {
    let from_variants: Vec<&str> = [
        Facet::Scope,
        Facet::Design,
        Facet::Plan,
        Facet::PhasePlan,
        Facet::Implementation,
        Facet::CodeReview,
        Facet::Reconciliation,
    ]
    .iter()
    .map(|f| f.as_str())
    .collect();
    assert_eq!(from_variants, FACETS.to_vec());
    // D-C11: `drift` is NOT a facet.
    assert!(!FACETS.contains(&"drift"));
    assert_eq!(FACETS.len(), 7);
}

#[test]
fn finding_status_known_set_matches_variants() {
    let from_variants: Vec<&str> = [
        FindingStatus::Open,
        FindingStatus::Answered,
        FindingStatus::Contested,
        FindingStatus::Verified,
        FindingStatus::Withdrawn,
    ]
    .iter()
    .map(|s| s.as_str())
    .collect();
    assert_eq!(from_variants, FINDING_STATUSES.to_vec());
}

#[test]
fn severity_known_set_matches_variants() {
    let from_variants: Vec<&str> = [
        Severity::Blocker,
        Severity::Major,
        Severity::Minor,
        Severity::Nit,
    ]
    .iter()
    .map(|s| s.as_str())
    .collect();
    assert_eq!(from_variants, SEVERITIES.to_vec());
}

#[test]
fn role_known_set_matches_variants() {
    let from_variants: Vec<&str> = [Role::Raiser, Role::Responder]
        .iter()
        .map(|r| r.as_str())
        .collect();
    assert_eq!(from_variants, ROLES.to_vec());
}

#[test]
fn review_status_known_set_matches_variants() {
    let from_variants: Vec<&str> = [ReviewStatus::Active, ReviewStatus::Done]
        .iter()
        .map(|s| s.as_str())
        .collect();
    assert_eq!(from_variants, REVIEW_STATUSES.to_vec());
}

#[test]
fn await_str_forms() {
    assert_eq!(Await::Raiser.as_str(), "raiser");
    assert_eq!(Await::Responder.as_str(), "responder");
    assert_eq!(Await::None.as_str(), "none");
}

#[test]
fn verb_str_and_required_role() {
    assert_eq!(Act::Raise.as_str(), "raise");
    assert_eq!(Act::Dispose.as_str(), "dispose");
    assert_eq!(Act::Amend.as_str(), "amend");
    assert_eq!(Act::Verify.as_str(), "verify");
    assert_eq!(Act::Contest.as_str(), "contest");
    assert_eq!(Act::Reopen.as_str(), "reopen");
    assert_eq!(Act::Withdraw.as_str(), "withdraw");
    // Static verb→role (design §6 responsibility split).
    assert_eq!(Act::Raise.required_role(), Role::Raiser);
    assert_eq!(Act::Verify.required_role(), Role::Raiser);
    assert_eq!(Act::Contest.required_role(), Role::Raiser);
    assert_eq!(Act::Reopen.required_role(), Role::Raiser);
    assert_eq!(Act::Withdraw.required_role(), Role::Raiser);
    assert_eq!(Act::Dispose.required_role(), Role::Responder);
    assert_eq!(Act::Amend.required_role(), Role::Responder);
}

// -- SL-268 PHASE-03: fail-safe closed-vocabulary reads (D15, DEC-319) ----

/// EX-1: `FindingStatus::parse` accepts exactly the five and, on anything else,
/// names the whole known set — no silent `Open` fallback.
#[test]
fn finding_status_parse_accepts_the_five_and_names_the_set() {
    for (token, want) in [
        ("open", FindingStatus::Open),
        ("answered", FindingStatus::Answered),
        ("contested", FindingStatus::Contested),
        ("verified", FindingStatus::Verified),
        ("withdrawn", FindingStatus::Withdrawn),
    ] {
        assert_eq!(FindingStatus::parse(token), Ok(want));
    }
    let err = FindingStatus::parse("zombie").unwrap_err();
    assert_eq!(
        err,
        "unknown finding status `zombie` (known: open, answered, contested, verified, withdrawn)"
    );
}

/// EX-3: an out-of-vocabulary value survives the read verbatim — `as_str` and
/// serde both give back the raw string, and `known()` admits nothing.
#[test]
fn vocab_unknown_round_trips_the_raw_value() {
    let status = Vocab::<FindingStatus>::read("zombie");
    assert_eq!(status, Vocab::Unknown("zombie".to_owned()));
    assert_eq!(status.as_str(), "zombie");
    assert_eq!(status.known(), None);
    assert_eq!(serde_json::to_value(&status).unwrap(), "zombie");

    let known = Vocab::<Severity>::read("blocker");
    assert_eq!(known.known(), Some(Severity::Blocker));
    assert_eq!(known.as_str(), "blocker");
    assert_eq!(serde_json::to_value(&known).unwrap(), "blocker");
}

/// EX-1 / VT-1: an out-of-vocabulary status reads non-terminal — it keeps the
/// review `Active` awaiting the responder, alone or beside a terminal sibling.
#[test]
fn unknown_status_reads_non_terminal() {
    let zombie = FindingState {
        status: Vocab::Unknown("zombie".to_owned()),
    };
    let verified = FindingState {
        status: Vocab::Known(FindingStatus::Verified),
    };
    assert_eq!(
        derived_status(std::slice::from_ref(&zombie)),
        (ReviewStatus::Active, Await::Responder)
    );
    assert_eq!(derived_status(&[zombie, verified]).0, ReviewStatus::Active);

    // The same through the authored read: a hand-edited ledger.
    let tmp = fixture_rv();
    let root = tmp.path();
    for title in ["a", "b"] {
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Minor, title),
            Role::Raiser,
        )
        .unwrap();
    }
    hand_edit_finding(root, 1, "F-1", "status", "zombie");
    hand_edit_finding(root, 1, "F-2", "status", "verified");
    assert_eq!(
        read_doc(root, 1).derived(),
        (ReviewStatus::Active, Await::Responder)
    );
}

/// EX-1 / VT-1: no act applies to an out-of-vocabulary status. Each of the four
/// finding acts refuses with the typed `UnknownStatus`, naming the known set and
/// the remedy, and the ledger bytes are untouched.
#[test]
fn unknown_status_refuses_every_act() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Major, "t"),
        Role::Raiser,
    )
    .unwrap();
    hand_edit_finding(root, 1, "F-1", "status", "zombie");
    // Heal the baton's CAS key so the acts reach the per-finding gate.
    run_status(Some(root.to_path_buf()), "RV-001").unwrap();
    let before = fs::read_to_string(authored_path(root, 1)).unwrap();

    let p = || Some(root.to_path_buf());
    let errs = [
        run_dispose(p(), &dispose_args("RV-001", "F-1"), Role::Responder).unwrap_err(),
        run_verify(p(), "RV-001", "F-1", None, Role::Raiser).unwrap_err(),
        run_contest(p(), "RV-001", "F-1", "n", Role::Raiser).unwrap_err(),
        run_withdraw(p(), "RV-001", "F-1", None, Role::Raiser).unwrap_err(),
    ];
    for err in errs {
        match err.downcast_ref::<ReviewError>() {
            Some(ReviewError::UnknownStatus { finding, raw }) => {
                assert_eq!((finding.as_str(), raw.as_str()), ("F-1", "zombie"));
            }
            other => panic!("expected UnknownStatus, got {other:?} ({err})"),
        }
        let text = err.to_string();
        for known in FINDING_STATUSES {
            assert!(text.contains(known), "{text} names {known}");
        }
        assert!(text.contains("ledger TOML"), "{text}");
    }
    assert_eq!(fs::read_to_string(authored_path(root, 1)).unwrap(), before);
}

/// EX-2 / VT-2: one `gates_as_blocker` classification feeds all three blocker
/// predicates — an out-of-vocabulary severity gates as `blocker`, and an
/// out-of-vocabulary status counts as not-terminal. The predicates keep their
/// state differences (SL-244): an `answered` blocker holds the review open but
/// no longer holds a design run's edge.
#[test]
fn unknown_severity_gates_as_blocker() {
    assert!(gates_as_blocker("crit"));
    assert!(gates_as_blocker("blocker"));
    for known in ["major", "minor", "nit"] {
        assert!(!gates_as_blocker(known), "{known}");
    }

    let tmp = fixture_rv();
    let root = tmp.path();
    for title in ["open crit", "answered crit", "zombie blocker"] {
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Minor, title),
            Role::Raiser,
        )
        .unwrap();
    }
    hand_edit_finding(root, 1, "F-1", "severity", "crit");
    hand_edit_finding(root, 1, "F-2", "severity", "crit");
    hand_edit_finding(root, 1, "F-2", "status", "answered");
    hand_edit_finding(root, 1, "F-3", "severity", "blocker");
    hand_edit_finding(root, 1, "F-3", "status", "zombie");
    let doc = read_doc(root, 1);

    let unresolved: Vec<String> = unresolved_blockers_for(root, "SL-001")
        .unwrap()
        .into_iter()
        .map(|b| b.finding)
        .collect();
    assert_eq!(unresolved, ["F-1", "F-2", "F-3"]);
    assert_eq!(undisposed_blockers(&doc), ["F-1", "F-3"]);
    assert_eq!(outstanding_by_severity(&doc).blocker, 3);
}

/// The `ReviewOutput` of a read verb as its wire JSON (the MCP shape), unwrapped
/// from its externally tagged variant.
fn wire(out: &ReviewOutput, variant: &str) -> serde_json::Value {
    serde_json::to_value(out).unwrap()[variant].clone()
}

/// The CLI rendering of a read verb's output.
fn rendered(out: &ReviewOutput) -> String {
    print_review(out)
}

/// EX-5 / VT-7: only `status` and `severity` are closed vocabularies a read
/// discloses. A legacy free-text `disposition` is carried verbatim and is no
/// defect — while a sibling RV's out-of-vocabulary severity, on the same
/// surface, is one (the positive control: the channel is read).
#[test]
fn legacy_disposition_quiet() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_new(Some(root.to_path_buf()), &new_args(Facet::Design, "SL-001")).unwrap();
    for rv in ["RV-001", "RV-002"] {
        run_raise(
            Some(root.to_path_buf()),
            &raise_args(rv, Severity::Minor, "t"),
            Role::Raiser,
        )
        .unwrap();
    }
    hand_edit_finding(root, 1, "F-1", "disposition", "whatever");
    hand_edit_finding(root, 2, "F-1", "severity", "crit");
    let p = || Some(root.to_path_buf());

    assert_eq!(vocabulary_defects(&read_doc(root, 1)), []);
    let quiet = rendered(&run_show(p(), "RV-001", Format::Table).unwrap());
    assert!(quiet.contains("whatever"), "{quiet}");
    assert!(!quiet.contains("warning:"), "{quiet}");

    assert_eq!(
        vocabulary_defects(&read_doc(root, 2)),
        [VocabDefect {
            finding: "F-1".to_owned(),
            field: VocabField::Severity,
            raw: "crit".to_owned(),
            effect: EFFECT_UNKNOWN_SEVERITY,
        }]
    );
    let loud = rendered(&run_show(p(), "RV-002", Format::Table).unwrap());
    assert!(
        loud.contains(
            "warning: RV-002 F-1 severity `crit` is out of vocabulary; gating as blocker\n"
        ),
        "{loud}"
    );
}

/// EX-3 / VT-3: an out-of-vocabulary value renders verbatim on every surface —
/// the show index, `show --json`, and the typed MCP `findings` — never as the
/// known value a fallback would have guessed.
#[test]
fn unknown_vocab_renders_verbatim_everywhere() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Major, "t"),
        Role::Raiser,
    )
    .unwrap();
    hand_edit_finding(root, 1, "F-1", "severity", "catastrophic");
    hand_edit_finding(root, 1, "F-1", "status", "zombie");
    let p = || Some(root.to_path_buf());

    let table = run_show(p(), "RV-001", Format::Table).unwrap();
    assert!(
        rendered(&table).contains("F-1 │ catastrophic │ zombie │"),
        "{}",
        rendered(&table)
    );
    let finding = &wire(&table, "Showed")["findings"][0];
    assert_eq!(finding["severity"], "catastrophic");
    assert_eq!(finding["status"], "zombie");

    let json: serde_json::Value =
        serde_json::from_str(&rendered(&run_show(p(), "RV-001", Format::Json).unwrap())).unwrap();
    assert_eq!(json["review"]["finding"][0]["severity"], "catastrophic");
    assert_eq!(json["review"]["finding"][0]["status"], "zombie");
}

/// EX-4 / VT-2: an out-of-vocabulary severity is disclosed on every review read
/// surface, each on its caller's channel — the show and status text, the
/// `warnings` field of `Showed`/`Status`/`Listed`, and the close gate's
/// `BlockerRef.reason`.
#[test]
fn unknown_severity_warned_on_every_surface() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Minor, "t"),
        Role::Raiser,
    )
    .unwrap();
    hand_edit_finding(root, 1, "F-1", "severity", "crit");
    let p = || Some(root.to_path_buf());
    let line = "warning: RV-001 F-1 severity `crit` is out of vocabulary; gating as blocker\n";
    let warning = serde_json::json!([{
        "effect": "gating as blocker",
        "field": "severity",
        "finding": "F-1",
        "raw": "crit",
        "rv": "RV-001",
    }]);

    let show = run_show(p(), "RV-001", Format::Table).unwrap();
    assert!(rendered(&show).contains(line), "{}", rendered(&show));
    assert_eq!(wire(&show, "Showed")["warnings"], warning);

    let status = run_status(p(), "RV-001").unwrap();
    assert!(rendered(&status).contains(line), "{}", rendered(&status));
    assert_eq!(wire(&status, "Status")["warnings"], warning);

    let list = run_list(p(), crate::listing::ListArgs::default(), None).unwrap();
    assert_eq!(wire(&list, "Listed")["warnings"], warning);

    let blockers = unresolved_blockers_for(root, "SL-001").unwrap();
    assert_eq!(
        blockers[0].reason.as_deref(),
        Some("severity `crit` is out of vocabulary; gating as blocker")
    );
}

// -- is_terminal mirror -------------------------------------------------

#[test]
fn finding_status_terminal_set() {
    assert!(FindingStatus::Verified.is_terminal());
    assert!(FindingStatus::Withdrawn.is_terminal());
    assert!(!FindingStatus::Open.is_terminal());
    assert!(!FindingStatus::Answered.is_terminal());
    assert!(!FindingStatus::Contested.is_terminal());
}

// -- render escaping (toml_string splice) -------------------------------

#[test]
fn render_finding_escapes_hostile_free_text() {
    let finding = Finding {
        id: "F-1".to_owned(),
        status: Vocab::Known(FindingStatus::Open),
        severity: Vocab::Known(Severity::Major),
        // A hostile title: a quote, a backslash, a newline, and a `]`.
        title: "a\"b\\c\nd]e".to_owned(),
        detail: "plain".to_owned(),
        disposition: None,
        response: None,
    };
    let rendered = render_finding(&finding);
    // The rendered block must parse back as valid TOML with the value intact
    // — proof the splice did not break the document or inject a key.
    let parsed: toml::Value = toml::from_str(&rendered).unwrap();
    let finding_tbl = parsed["finding"].as_array().unwrap()[0].as_table().unwrap();
    assert_eq!(finding_tbl["title"].as_str().unwrap(), "a\"b\\c\nd]e");
    assert_eq!(finding_tbl["id"].as_str().unwrap(), "F-1");
    assert_eq!(finding_tbl["status"].as_str().unwrap(), "open");
    assert_eq!(finding_tbl["severity"].as_str().unwrap(), "major");
}

#[test]
fn render_finding_emits_responder_fields_when_present() {
    let finding = Finding {
        id: "F-2".to_owned(),
        status: Vocab::Known(FindingStatus::Answered),
        severity: Vocab::Known(Severity::Nit),
        title: "t".to_owned(),
        detail: "d".to_owned(),
        disposition: Some("fixed".to_owned()),
        response: Some("done in r\"123".to_owned()),
    };
    let rendered = render_finding(&finding);
    let parsed: toml::Value = toml::from_str(&rendered).unwrap();
    let tbl = parsed["finding"].as_array().unwrap()[0].as_table().unwrap();
    assert_eq!(tbl["disposition"].as_str().unwrap(), "fixed");
    assert_eq!(tbl["response"].as_str().unwrap(), "done in r\"123");
}

// -- impure shell (PHASE-02): new / show / list --------------------------

use std::path::Path;

/// Plant a minimal slice dir so `SL-NNN` is a resolvable target ref. Just the
/// numeric dir + sister toml + sister md — enough for `ensure_ref_resolves` (a
/// dir probe) and for `slice::selector_paths` (which reads the toml + md).
fn plant_slice_target(root: &Path, id: u32) {
    plant_slice_with_selectors(root, id, &[]);
}

/// Plant a slice carrying `[[selector]]` entries (every entry `design-target`,
/// intent-agnostic for prime which unions all intents). The slice md body is
/// planted too — `read_slice` reads both tiers.
fn plant_slice_with_selectors(root: &Path, id: u32, selectors: &[&str]) {
    let name = format!("{id:03}");
    let dir = root.join(".doctrine/slice").join(&name);
    std::fs::create_dir_all(&dir).unwrap();
    let mut toml = format!(
        "id = {id}\nslug = \"t\"\ntitle = \"T\"\nstatus = \"proposed\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n"
    );
    for sel in selectors {
        toml.push_str(&format!(
            "\n[[selector]]\nselector = \"{sel}\"\nintent = \"design-target\"\n"
        ));
    }
    std::fs::write(dir.join(format!("slice-{name}.toml")), toml).unwrap();
    std::fs::write(dir.join(format!("slice-{name}.md")), "## Scope\n").unwrap();
}

fn meta(facet: &str) -> ReviewMeta {
    ReviewMeta {
        facet: facet.to_owned(),
        raiser: "rev".to_owned(),
        responder: "auth".to_owned(),
        concluded: false,
        rounds_base: None,
        contests_base: None,
        turn: vec![],
    }
}

/// PHASE-02: the rendered ledger toml round-trips into `ReviewDoc`, carries NO
/// stored status (D-C8), and an optional `[target].phase` is present/absent
/// exactly as supplied.
#[test]
fn render_review_toml_round_trips_without_a_stored_status() {
    let target = Target {
        reference: "SL-024".to_owned(),
        phase: Some("PHASE-03".to_owned()),
    };
    let body = render_review_toml(
        7,
        "design-review",
        "Design review",
        &meta("design"),
        &target,
    )
    .unwrap();
    // No stored status — the storage rule forbids derived data (D-C8).
    let value: toml::Value = toml::from_str(&body).unwrap();
    assert!(
        value.get("status").is_none(),
        "ledger stores no status: {body}"
    );
    let doc: ReviewDoc = toml::from_str(&body).unwrap();
    assert_eq!(doc.id, 7);
    assert_eq!(doc.review.facet, "design");
    assert_eq!(doc.target.reference, "SL-024");
    assert_eq!(doc.target.phase.as_deref(), Some("PHASE-03"));
    assert!(doc.finding.is_empty(), "fresh ledger has no findings");
}

/// Render-splice escaping: a hostile title round-trips intact through
/// `toml_string` (mem.pattern.render.toml-splice-escape-user-values).
#[test]
fn render_review_toml_escapes_a_hostile_title() {
    let target = Target {
        reference: "SL-001".to_owned(),
        phase: None,
    };
    let hostile = "a\"b\\c\nd]e";
    let body = render_review_toml(1, "s", hostile, &meta("scope"), &target).unwrap();
    let doc: ReviewDoc = toml::from_str(&body).unwrap();
    assert_eq!(doc.title, hostile);
    // phase absent ⇒ no phase key.
    assert!(doc.target.phase.is_none());
}

/// VT-4 (show): a fresh empty-ledger RV renders derived status `active` with
/// done status, await `none`, and the `reviews` edge to the target.
#[test]
fn show_renders_empty_ledger_done_and_the_edge() {
    let doc = ReviewDoc {
        id: 3,
        slug: "s".to_owned(),
        title: "Design review of SL-024".to_owned(),
        review: meta("design"),
        target: Target {
            reference: "SL-024".to_owned(),
            phase: None,
        },
        finding: Vec::new(),
        tags: Vec::new(),
        estimate: None,
        value: None,
    };
    let view = ReviewView::of(&doc);
    let out = format_show(&view, "## Brief\n", "points", None, None, 0.0, 1.0);
    assert!(out.contains("RV-003 — Design review of SL-024"), "{out}");
    // empty ⇒ Done, await=None.
    assert!(out.contains("done · await=none"), "{out}");
    assert!(out.contains("RV-003 ──reviews──▶ SL-024"), "edge: {out}");
    assert!(out.contains("findings: 0"), "{out}");
}

/// VT-4 (list): the empty-ledger RV lists with derived status `done`
/// (await none), facet, and the target edge — no stored status read.
#[test]
fn list_renders_empty_ledger_done_and_the_edge() {
    let doc = ReviewDoc {
        id: 5,
        slug: "s".to_owned(),
        title: "Plan review".to_owned(),
        review: meta("plan"),
        target: Target {
            reference: "SL-009".to_owned(),
            phase: Some("PHASE-02".to_owned()),
        },
        finding: Vec::new(),
        tags: Vec::new(),
        estimate: None,
        value: None,
    };
    let (status, awaited) = doc.derived();
    let rows = vec![(doc, status, awaited)];
    let sel = listing::select_columns(&REVIEW_COLUMNS, REVIEW_DEFAULT, None).unwrap();
    let out = listing::render_columns(&rows, &sel, listing::RenderOpts::default());
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines[0].starts_with("id"), "header: {:?}", lines[0]);
    assert!(lines[1].starts_with("RV-005"), "{:?}", lines[1]);
    assert!(lines[1].contains("done (await none)"), "{:?}", lines[1]);
    assert!(lines[1].contains("plan"), "{:?}", lines[1]);
    // phase-scoped edge `SL-009@PHASE-02`.
    assert!(lines[1].contains("SL-009@PHASE-02"), "{:?}", lines[1]);
}

/// Derived status reflects a non-terminal finding: an `open` finding keeps the
/// review Active awaiting the Responder (D-C8) — read straight from authored
/// findings, never a stored status.
#[test]
fn derived_status_reads_findings_not_a_stored_status() {
    let mut doc = ReviewDoc {
        id: 1,
        slug: "s".to_owned(),
        title: "t".to_owned(),
        review: meta("design"),
        target: Target {
            reference: "SL-001".to_owned(),
            phase: None,
        },
        finding: vec![FindingRow {
            id: "F-1".to_owned(),
            status: "open".to_owned(),
            severity: "major".to_owned(),
            title: "t".to_owned(),
            detail: "d".to_owned(),
            disposition: None,
            route: None,
            response: None,
            turn: vec![],
        }],
        tags: Vec::new(),
        estimate: None,
        value: None,
    };
    assert_eq!(doc.derived(), (ReviewStatus::Active, Await::Responder));
    // all-terminal ⇒ Done.
    doc.finding[0].status = "verified".to_owned();
    assert_eq!(doc.derived(), (ReviewStatus::Done, Await::None));
}

// -- run_new end-to-end (VT-2: dangling ref refused at creation) ----------

fn new_args(facet: Facet, target: &str) -> NewArgs {
    NewArgs {
        facet,
        target: target.to_owned(),
        phase: None,
        title: None,
        raiser: None,
        responder: None,
    }
}

/// `review new` mints an RV with an empty ledger + seeded `## Brief`, and the
/// ledger round-trips through the real readers (Active/Raiser, the edge).
#[test]
fn run_new_creates_an_empty_ledger_rv_against_a_real_target() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    plant_slice_target(root, 24);
    run_new(Some(root.to_path_buf()), &new_args(Facet::Design, "SL-024")).unwrap();

    let review_root = root.join(REVIEW_DIR);
    let doc = read_review(&review_root, 1).unwrap();
    assert_eq!(doc.id, 1);
    assert_eq!(doc.target.reference, "SL-024");
    assert!(doc.finding.is_empty());
    assert_eq!(doc.derived(), (ReviewStatus::Done, Await::None));
    let brief = read_brief(&review_root, 1).unwrap();
    assert!(brief.contains("## Brief"), "brief seeded: {brief}");
    // The `NNN-slug` alias symlink landed.
    assert!(
        std::fs::symlink_metadata(review_root.join("001-design-review-of-sl-024"))
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false),
        "alias symlink planted"
    );
}

/// VT-2: a dangling `[target].ref` (well-formed but no entity) is refused at
/// creation — and no RV directory is minted (§7).
#[test]
fn run_new_refuses_a_dangling_target_and_mints_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    // SL-099 has no entity dir.
    let err = run_new(Some(root.to_path_buf()), &new_args(Facet::Design, "SL-099")).unwrap_err();
    // IMP-107: refusal is a typed `DanglingRef` carrying the target (the MCP
    // transport maps it to `DANGLING_REF`), asserted by variant identity.
    match err.downcast_ref::<ReviewError>() {
        Some(ReviewError::DanglingRef { target }) => assert_eq!(target, "SL-099"),
        other => panic!("expected DanglingRef, got {other:?}"),
    }
    assert!(
        entity::scan_ids(&root.join(REVIEW_DIR)).unwrap().is_empty(),
        "no RV minted on a refused target"
    );
}

/// VT-2: an unknown-prefix target is refused at creation (§7).
#[test]
fn run_new_refuses_an_unknown_prefix_target() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let err = run_new(Some(root.to_path_buf()), &new_args(Facet::Scope, "ZZ-001")).unwrap_err();
    // IMP-107: an unknown-prefix target is also a forward-edge refusal, typed
    // as `DanglingRef` (asserted by variant identity, not string content).
    match err.downcast_ref::<ReviewError>() {
        Some(ReviewError::DanglingRef { target }) => assert_eq!(target, "ZZ-001"),
        other => panic!("expected DanglingRef, got {other:?}"),
    }
}

// -- the mint's two halves (SL-244 PHASE-04 T5, DEC-086 steps 3–4) --------

/// The id-claim midpoint reaches the RV mint: `on_reserved` runs on the
/// claimed id before any authored byte, and the id it names is the one the
/// review lands at. That equality is the whole point — from step 3 onward,
/// recovery names the exact target.
#[test]
fn mint_review_offers_the_claimed_id_before_writing_the_ledger() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    plant_slice_target(root, 24);
    let seen = std::cell::RefCell::new(None);

    let out = mint_review(root, &new_args(Facet::Design, "SL-024"), |id, canonical| {
        assert!(
            !root.join(REVIEW_DIR).join("001/review-001.toml").exists(),
            "the midpoint runs before any authored byte"
        );
        *seen.borrow_mut() = Some((id, canonical.to_owned()));
        Ok(())
    })
    .unwrap();

    assert_eq!(seen.into_inner(), Some((1, "RV-001".to_owned())));
    match out {
        ReviewOutput::Created { id, canonical, .. } => {
            assert_eq!(
                (id, canonical.as_str()),
                (1, "RV-001"),
                "mints what it named"
            );
        }
        other => panic!("expected Created, got {other:?}"),
    }
    assert!(read_review(&root.join(REVIEW_DIR), 1).is_ok());
}

/// The resume half: the reservation was claimed and journalled before the
/// crash, so step 4 writes into *that* id rather than claiming a second.
#[test]
fn materialise_review_at_writes_into_the_journalled_reservation() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    plant_slice_target(root, 24);
    let review_root = root.join(REVIEW_DIR);
    std::fs::create_dir_all(review_root.join("003")).unwrap();

    materialise_review_at(root, "RV-003", &new_args(Facet::Design, "SL-024")).unwrap();

    let doc = read_review(&review_root, 3).unwrap();
    assert_eq!(doc.id, 3);
    assert_eq!(doc.target.reference, "SL-024");
    assert!(read_brief(&review_root, 3).unwrap().contains("## Brief"));
    assert_eq!(
        entity::scan_ids(&review_root).unwrap(),
        vec![3],
        "no second reservation claimed on resume"
    );
}

/// A resume that races a completed write reports rather than overwrites —
/// the reservation is not a licence to clobber the record it already holds.
#[test]
fn materialise_review_at_refuses_to_overwrite_a_completed_write() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    plant_slice_target(root, 24);
    let review_root = root.join(REVIEW_DIR);
    mint_review(
        root,
        &new_args(Facet::Design, "SL-024"),
        entity::no_midpoint(),
    )
    .unwrap();

    let err =
        materialise_review_at(root, "RV-001", &new_args(Facet::Design, "SL-024")).unwrap_err();

    assert!(err.to_string().contains("Refusing to overwrite"), "{err}");
    assert_eq!(read_review(&review_root, 1).unwrap().id, 1, "record intact");
}

/// `Facet::parse` accepts the closed 7-set and rejects `drift` (D-C11) /
/// garbage with a helpful message.
#[test]
fn facet_parse_accepts_the_seven_and_rejects_drift() {
    assert_eq!(Facet::parse("phase-plan").unwrap(), Facet::PhasePlan);
    assert_eq!(Facet::parse("code-review").unwrap(), Facet::CodeReview);
    assert!(
        Facet::parse("drift").is_err(),
        "drift is not a facet (D-C11)"
    );
    let err = Facet::parse("bogus").unwrap_err();
    assert!(err.contains("unknown facet"), "{err}");
}

// =====================================================================
// PHASE-03 — verb family + the turn guard (VT-1..10)
// =====================================================================

/// Stand up a fresh RV (id 1) targeting a planted SL-001, in a tempdir whose
/// root is not a git tree (the fork guard's `is_linked_worktree` returns Err
/// ⇒ treated not-a-fork ⇒ proceeds). Returns the root.
fn fixture_rv() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    plant_slice_target(root, 1);
    run_new(Some(root.to_path_buf()), &new_args(Facet::Design, "SL-001")).unwrap();
    tmp
}

fn raise_args(reference: &str, sev: Severity, title: &str) -> RaiseArgs {
    RaiseArgs {
        reference: reference.to_owned(),
        severity: sev,
        title: title.to_owned(),
        detail: "d".to_owned(),
    }
}

fn dispose_args(reference: &str, finding: &str) -> DisposeArgs {
    DisposeArgs {
        reference: reference.to_owned(),
        finding: finding.to_owned(),
        disposition: Disposition::FixNow,
        route: None,
        response: "done".to_owned(),
    }
}

fn read_doc(root: &Path, id: u32) -> ReviewDoc {
    read_review(&root.join(REVIEW_DIR), id).unwrap()
}

/// Hand-edit one field of one finding in an RV's authored ledger (edit-
/// preserving) — the out-of-band write a closed-vocabulary read must survive.
fn hand_edit_finding(root: &Path, id: u32, finding: &str, field: &str, value: &str) {
    let path = authored_path(root, id);
    let mut doc = fs::read_to_string(&path)
        .unwrap()
        .parse::<toml_edit::DocumentMut>()
        .unwrap();
    finding_table_mut(&mut doc, finding)
        .unwrap()
        .insert(field, toml_edit::value(value));
    fs::write(&path, doc.to_string()).unwrap();
}

/// A full raise→dispose→verify lifecycle drives the finding through its
/// states; the ledger reflects each transition and the baton tracks `await`.
#[test]
fn lifecycle_raise_dispose_verify() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Major, "t"),
        Role::Raiser,
    )
    .unwrap();
    // After a raise: one open finding, await=Responder.
    let doc = read_doc(root, 1);
    assert_eq!(doc.finding.len(), 1);
    assert_eq!(doc.finding[0].id, "F-1");
    assert_eq!(doc.finding[0].status, "open");
    assert_eq!(read_baton(root, 1).unwrap().unwrap().awaiting, "responder");

    run_dispose(
        Some(root.to_path_buf()),
        &dispose_args("RV-001", "F-1"),
        Role::Responder,
    )
    .unwrap();
    let doc = read_doc(root, 1);
    assert_eq!(doc.finding[0].status, "answered");
    assert_eq!(doc.finding[0].disposition.as_deref(), Some("fix-now"));
    assert_eq!(doc.finding[0].response.as_deref(), Some("done"));
    assert_eq!(read_baton(root, 1).unwrap().unwrap().awaiting, "raiser");

    run_verify(
        Some(root.to_path_buf()),
        "RV-001",
        "F-1",
        None,
        Role::Raiser,
    )
    .unwrap();
    let doc = read_doc(root, 1);
    assert_eq!(doc.finding[0].status, "verified");
    // All-terminal ⇒ Done / await=none.
    assert_eq!(read_baton(root, 1).unwrap().unwrap().awaiting, "none");
}

/// VT-1: field ownership disjoint — raiser fields (id/title/detail/severity)
/// are fixed at raise; a dispose mutates ONLY the responder pair + status.
#[test]
fn vt1_raiser_fields_immutable_responder_fields_mutable() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &RaiseArgs {
            reference: "RV-001".to_owned(),
            severity: Severity::Blocker,
            title: "orig-title".to_owned(),
            detail: "orig-detail".to_owned(),
        },
        Role::Raiser,
    )
    .unwrap();
    run_dispose(
        Some(root.to_path_buf()),
        &dispose_args("RV-001", "F-1"),
        Role::Responder,
    )
    .unwrap();
    let f = &read_doc(root, 1).finding[0];
    // Raiser-owned: unchanged by the responder's turn.
    assert_eq!(f.id, "F-1");
    assert_eq!(f.title, "orig-title");
    assert_eq!(f.detail, "orig-detail");
    assert_eq!(f.severity, "blocker");
    // Responder-owned: set by dispose.
    assert_eq!(f.disposition.as_deref(), Some("fix-now"));
    assert_eq!(f.response.as_deref(), Some("done"));
    // Status moved on a single-owner edge.
    assert_eq!(f.status, "answered");
}

/// VT-2: finding ids are append-only `F-<max+1>` — never reused, even with a
/// gap. Three raises land F-1, F-2, F-3.
#[test]
fn vt2_finding_ids_are_append_only() {
    let tmp = fixture_rv();
    let root = tmp.path();
    for n in ["a", "b", "c"] {
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", Severity::Minor, n),
            Role::Raiser,
        )
        .unwrap();
    }
    let ids: Vec<String> = read_doc(root, 1)
        .finding
        .iter()
        .map(|f| f.id.clone())
        .collect();
    assert_eq!(ids, ["F-1", "F-2", "F-3"]);
    // The pure id allocator: max+1 over existing, robust to a gap.
    let rows = vec![FindingRow {
        id: "F-7".to_owned(),
        status: "open".to_owned(),
        severity: "nit".to_owned(),
        title: "t".to_owned(),
        detail: "d".to_owned(),
        disposition: None,
        route: None,
        response: None,
        turn: vec![],
    }];
    assert_eq!(next_finding_id(&rows), "F-8");
    assert_eq!(next_finding_id(&[]), "F-1");
}

/// VT-3: transitions are edit-preserving — a hand-added comment and an
/// unknown key survive a dispose (the governance.rs:290 contract at finding
/// scope).
#[test]
fn vt3_transitions_are_edit_preserving() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Major, "t"),
        Role::Raiser,
    )
    .unwrap();
    // Hand-add a comment + an unknown top-level key.
    let path = authored_path(root, 1);
    let mut text = fs::read_to_string(&path).unwrap();
    text.push_str("\n# a hand comment\nunknown_key = \"keepme\"\n");
    fs::write(&path, &text).unwrap();
    // The hand-edit changed the bytes — refresh the baton so the entry CAS
    // does not (correctly) abort the next turn on the edit it now reflects.
    run_status(Some(root.to_path_buf()), "RV-001").unwrap();

    run_dispose(
        Some(root.to_path_buf()),
        &dispose_args("RV-001", "F-1"),
        Role::Responder,
    )
    .unwrap();
    let after = fs::read_to_string(&path).unwrap();
    assert!(
        after.contains("# a hand comment"),
        "comment survived: {after}"
    );
    assert!(
        after.contains("unknown_key = \"keepme\""),
        "unknown key survived: {after}"
    );
    assert_eq!(read_doc(root, 1).finding[0].status, "answered");
}

/// VT-4: render escaping — a hostile title/detail raised then read back
/// round-trips intact (toml_edit::value quotes/escapes the structured write,
/// the splice twin of the render path).
#[test]
fn vt4_hostile_free_text_round_trips() {
    let tmp = fixture_rv();
    let root = tmp.path();
    let hostile = "a\"b\\c\nd]e";
    run_raise(
        Some(root.to_path_buf()),
        &RaiseArgs {
            reference: "RV-001".to_owned(),
            severity: Severity::Major,
            title: hostile.to_owned(),
            detail: hostile.to_owned(),
        },
        Role::Raiser,
    )
    .unwrap();
    // The ledger is still valid TOML and the value is intact.
    let f = &read_doc(root, 1).finding[0];
    assert_eq!(f.title, hostile);
    assert_eq!(f.detail, hostile);
}

/// VT-5(a) + VT-8: two ordered invocations — a lock held by a concurrent
/// invocation makes the second BAIL (busy), no clobber; after the first
/// completes the loser re-runs from the refreshed baton and lands a correct
/// turn. Also asserts `raise` is allowed while await=Responder.
#[test]
fn vt5a_lock_serializes_loser_bails_then_re_runs() {
    let tmp = fixture_rv();
    let root = tmp.path();
    // Manually hold the lock (simulating a concurrent invocation in flight).
    let held = LockGuard::acquire(root, 1).unwrap();
    // A second invocation loses the create_new race → clean "busy" bail.
    let err = run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Major, "t"),
        Role::Raiser,
    )
    .unwrap_err();
    assert!(err.to_string().contains("busy"), "loser bailed busy: {err}");
    // Nothing was written — the ledger is untouched.
    assert!(
        read_doc(root, 1).finding.is_empty(),
        "no clobber on a lost lock"
    );
    // The first invocation completes (lock released).
    drop(held);
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Major, "first"),
        Role::Raiser,
    )
    .unwrap();
    // The loser re-runs — and `raise` is allowed even while await=Responder
    // (one open finding ⇒ Responder), landing F-2 (VT-8 raise-not-blocked).
    assert_eq!(read_baton(root, 1).unwrap().unwrap().awaiting, "responder");
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Major, "second"),
        Role::Raiser,
    )
    .unwrap();
    let ids: Vec<String> = read_doc(root, 1)
        .finding
        .iter()
        .map(|f| f.id.clone())
        .collect();
    assert_eq!(ids, ["F-1", "F-2"]);
}

/// IMP-107: the `acquire` path surfaces `ReviewError::LockContention` by
/// variant identity (not a generic anyhow bail) so the MCP transport maps it
/// to the structured `LOCK_CONTENTION` code carrying the canonical id, while
/// the `review unlock` guidance still rides `details` for the CLI arm.
#[test]
fn acquire_surfaces_lock_contention_by_variant_identity() {
    let tmp = fixture_rv();
    let root = tmp.path();
    // Hold the lock, then a second acquire loses the create_new race.
    let _held = LockGuard::acquire(root, 1).unwrap();
    // `.err()` (not `unwrap_err`) — `LockGuard` is not `Debug` by design.
    let err = LockGuard::acquire(root, 1)
        .err()
        .expect("second acquire must contend");
    match err.downcast_ref::<ReviewError>() {
        Some(ReviewError::LockContention { canonical, details }) => {
            assert_eq!(canonical, "RV-001");
            assert!(
                details.contains("review unlock"),
                "unlock guidance retained: {details}"
            );
        }
        other => panic!("expected LockContention, got {other:?}"),
    }
}

/// VT-5(b) + VT-6 + VT-7: a crash between the authored write (step 5) and the
/// baton write (step 7) leaves the authored ledger ahead of the baton hash;
/// the NEXT invocation's entry CAS detects it, heals the baton, and bails
/// "re-run". Simulated by mutating the authored ledger directly (a real
/// authored-write that the baton never caught up to) then driving a verb.
#[test]
fn vt5b_entry_cas_self_heals_a_crash_between_writes() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Major, "t"),
        Role::Raiser,
    )
    .unwrap();
    let baton_before = read_baton(root, 1).unwrap().unwrap();

    // Simulate a crash AFTER the authored write but BEFORE the baton write:
    // the authored ledger gains an edit the baton's hash does not reflect.
    let path = authored_path(root, 1);
    let mut text = fs::read_to_string(&path).unwrap();
    text = text.replace("status = \"open\"", "status = \"answered\"");
    fs::write(&path, &text).unwrap();

    // The next invocation's ENTRY CAS catches the divergence, refreshes the
    // baton from the authored truth, and bails.
    let err = run_dispose(
        Some(root.to_path_buf()),
        &dispose_args("RV-001", "F-1"),
        Role::Responder,
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("changed underneath"),
        "entry CAS bailed: {err}"
    );
    // The baton self-healed: its hash now matches the authored bytes, and the
    // await recomputed from the answered finding (Raiser).
    let healed = read_baton(root, 1).unwrap().unwrap();
    assert_ne!(
        healed.authored_hash, baton_before.authored_hash,
        "hash refreshed"
    );
    assert_eq!(
        healed.authored_hash,
        crate::git::sha256(fs::read_to_string(&path).unwrap().as_bytes())
    );
    assert_eq!(
        healed.awaiting, "raiser",
        "await recomputed from authored truth"
    );
    // The ledger was NOT clobbered — the aborted dispose wrote nothing.
    assert_eq!(read_doc(root, 1).finding[0].status, "answered");
}

/// VT-5(c) + VT-6: a hand-edit landing AFTER the step-2 read but BEFORE the
/// step-5 write (the pre-write CAS window) aborts the turn with NO write — the
/// stale in-memory DocumentMut cannot overwrite the newer authored truth. The
/// mid-turn hook fires the edit deterministically (no threads).
#[test]
fn vt5c_pre_write_cas_aborts_a_mid_turn_edit() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Major, "t"),
        Role::Raiser,
    )
    .unwrap();
    let path = authored_path(root, 1);

    // Drive a dispose under the hooked seam: the hook lands a hand-edit
    // (a second finding) between the in-memory mutation and the write.
    let hook = || {
        let mut text = fs::read_to_string(&path).unwrap();
        text.push_str(
            "\n[[finding]]\nid = \"F-2\"\nstatus = \"open\"\nseverity = \"nit\"\n\
             title = \"injected\"\ndetail = \"by hand\"\n",
        );
        fs::write(&path, &text).unwrap();
    };
    let err = with_turn_hooked(
        root,
        1,
        Act::Dispose,
        Role::Responder,
        &hook,
        |doc, existing| {
            let from = finding_status_of(existing, "F-1")?;
            gate(Act::Dispose, from, Role::Responder, "F-1")?;
            let table = finding_table_mut(doc, "F-1")?;
            apply_act(
                table,
                Act::Dispose,
                Role::Responder,
                FindingStatus::Answered,
                TurnFields {
                    note: None,
                    disposition: Some("fixed"),
                    route: None,
                    response: Some("done"),
                },
            )
        },
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("changed underneath this turn"),
        "pre-write CAS aborted: {err}"
    );
    // NO clobber: F-1 is still `open` (the dispose never wrote), and the
    // injected F-2 survives (the abort wrote nothing over it).
    let doc = read_doc(root, 1);
    assert_eq!(doc.finding.len(), 2, "injected finding survived");
    assert_eq!(
        doc.finding[0].status, "open",
        "F-1 not clobbered to answered"
    );
    assert_eq!(doc.finding[1].id, "F-2");
}

/// VT-5(d): the same finding contested then verified — once a verify makes the
/// finding terminal, a contest can no longer fire on it (the per-finding gate
/// is the lost-update guard at finding granularity). Drives the two verbs in
/// order and asserts the FINAL ledger reflects only the winner.
#[test]
fn vt5d_same_finding_contest_racing_verify() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Major, "t"),
        Role::Raiser,
    )
    .unwrap();
    run_dispose(
        Some(root.to_path_buf()),
        &dispose_args("RV-001", "F-1"),
        Role::Responder,
    )
    .unwrap();
    // Verify wins first → terminal.
    run_verify(
        Some(root.to_path_buf()),
        "RV-001",
        "F-1",
        None,
        Role::Raiser,
    )
    .unwrap();
    assert_eq!(read_doc(root, 1).finding[0].status, "verified");
    // The racing contest now finds the finding terminal → per-finding gate
    // refuses; the ledger is untouched (no double-apply).
    let err =
        run_contest(Some(root.to_path_buf()), "RV-001", "F-1", "n", Role::Raiser).unwrap_err();
    assert!(
        err.to_string().contains("out of turn"),
        "contest gated: {err}"
    );
    assert_eq!(
        read_doc(root, 1).finding[0].status,
        "verified",
        "winner stands"
    );
}

/// VT-8: an out-of-turn write is refused by the static role check AND the
/// per-finding gate.
#[test]
fn vt8_out_of_turn_refused() {
    let tmp = fixture_rv();
    let root = tmp.path();
    // Static role: dispose asserted --as raiser ⇒ refused.
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Major, "t"),
        Role::Raiser,
    )
    .unwrap();
    let err = run_dispose(
        Some(root.to_path_buf()),
        &dispose_args("RV-001", "F-1"),
        Role::Raiser, // wrong role
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("responder's verb"),
        "static role: {err}"
    );
    // Per-finding state: verify an open (not answered) finding ⇒ refused.
    let err = run_verify(
        Some(root.to_path_buf()),
        "RV-001",
        "F-1",
        None,
        Role::Raiser,
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("out of turn"),
        "per-finding gate: {err}"
    );
    // Nothing moved.
    assert_eq!(read_doc(root, 1).finding[0].status, "open");
}

/// VT-9: `status` rebuilds the baton — the cached await equals a fresh
/// recompute, even after the baton was deleted (cold) or stale.
#[test]
fn vt9_status_rebuilds_the_baton() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Major, "t"),
        Role::Raiser,
    )
    .unwrap();
    // Delete the baton (cold) — status must rebuild it == recompute.
    fs::remove_file(baton_path(root, 1)).unwrap();
    run_status(Some(root.to_path_buf()), "RV-001").unwrap();
    let baton = read_baton(root, 1).unwrap().unwrap();
    let doc = read_doc(root, 1);
    let (_, awaited) = derived_status(&finding_states_of(&doc));
    assert_eq!(baton.awaiting, awaited.as_str(), "cache == recompute");
    assert_eq!(
        baton.authored_hash,
        crate::git::sha256(
            fs::read_to_string(authored_path(root, 1))
                .unwrap()
                .as_bytes()
        )
    );
}

/// VT-10: a review verb on a fork-resolved root bails (IMP-024 guard), and the
/// baton/lock sit in the gitignored parent state tree. Builds a real linked
/// worktree to exercise `is_linked_worktree`.
#[test]
fn vt10_fork_root_refused_and_baton_in_parent_state() {
    use std::process::Command;
    let tmp = tempfile::tempdir().unwrap();
    let main = tmp.path().join("main");
    std::fs::create_dir_all(&main).unwrap();
    let git = |dir: &Path, args: &[&str]| {
        let ok = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00 +0000")
            .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00 +0000")
            .output()
            .unwrap();
        assert!(
            ok.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&ok.stderr)
        );
    };
    git(&main, &["init", "-b", "main"]);
    git(&main, &["config", "user.name", "T"]);
    git(&main, &["config", "user.email", "t@t.invalid"]);
    plant_slice_target(&main, 1);
    run_new(Some(main.clone()), &new_args(Facet::Design, "SL-001")).unwrap();
    std::fs::write(main.join("seed"), "x").unwrap();
    git(&main, &["add", "."]);
    git(&main, &["commit", "-m", "seed"]);
    // Add a linked worktree (the fork).
    let fork = tmp.path().join("fork");
    git(&main, &["worktree", "add", fork.to_str().unwrap()]);

    // A verb resolved at the fork root bails (IMP-024).
    let err = run_raise(
        Some(fork.clone()),
        &raise_args("RV-001", Severity::Major, "t"),
        Role::Raiser,
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("worktree fork"),
        "fork guard: {err}"
    );

    // A verb on the parent tree works, and the baton lands under the parent's
    // gitignored .doctrine/state/review/ (never the fork).
    run_raise(
        Some(main.clone()),
        &raise_args("RV-001", Severity::Major, "t"),
        Role::Raiser,
    )
    .unwrap();
    assert!(
        main.join(".doctrine/state/review/001/baton.toml").is_file(),
        "baton in parent state"
    );
    assert!(
        !fork.join(".doctrine/state/review/001/baton.toml").exists(),
        "no baton in the fork"
    );
}

/// ISS-275: the IMP-024 guard bails on *forks*, not on every linked worktree.
/// A dispatch COORDINATION tree (`dispatch/<NNN>`, numeric suffix) is the sole
/// writer of its branch, so it may drive a review, and its baton lands in its
/// OWN gitignored state tree — `state_dir` is root-derived, so admitting the
/// tree is the whole fix; nothing about the baton's locus needs to move.
/// Without this, the three SL-233 design gates cannot be driven where their
/// sketches live.
#[test]
fn vt10b_coord_worktree_admitted_and_baton_in_its_own_state() {
    use std::process::Command;
    let tmp = tempfile::tempdir().unwrap();
    let main = tmp.path().join("main");
    std::fs::create_dir_all(&main).unwrap();
    let git = |dir: &Path, args: &[&str]| {
        let ok = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00 +0000")
            .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00 +0000")
            .output()
            .unwrap();
        assert!(
            ok.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&ok.stderr)
        );
    };
    git(&main, &["init", "-b", "main"]);
    git(&main, &["config", "user.name", "T"]);
    git(&main, &["config", "user.email", "t@t.invalid"]);
    plant_slice_target(&main, 1);
    run_new(Some(main.clone()), &new_args(Facet::Design, "SL-001")).unwrap();
    std::fs::write(main.join("seed"), "x").unwrap();
    git(&main, &["add", "."]);
    git(&main, &["commit", "-m", "seed"]);
    // The coordination worktree: a linked worktree on `dispatch/<NNN>`.
    let coord = tmp.path().join("coord");
    git(
        &main,
        &[
            "worktree",
            "add",
            "-b",
            "dispatch/001",
            coord.to_str().unwrap(),
        ],
    );

    // The verb is ADMITTED at the coord root (contrast VT-10's fork).
    run_raise(
        Some(coord.clone()),
        &raise_args("RV-001", Severity::Major, "t"),
        Role::Raiser,
    )
    .unwrap();

    // ...and its baton sits in the COORD tree's own gitignored state, not the
    // parent's — the coord tree is the sole writer of its branch.
    assert!(
        coord
            .join(".doctrine/state/review/001/baton.toml")
            .is_file(),
        "baton in the coord tree's own state"
    );
    assert!(
        !main.join(".doctrine/state/review/001/baton.toml").exists(),
        "no baton in the parent tree"
    );
}

/// The `--as` role assertion parses cooperatively and defaults to the verb's
/// required role.
#[test]
fn parse_role_defaults_and_validates() {
    assert_eq!(parse_role(None, Role::Responder).unwrap(), Role::Responder);
    assert_eq!(
        parse_role(Some("raiser"), Role::Responder).unwrap(),
        Role::Raiser
    );
    assert!(parse_role(Some("bogus"), Role::Raiser).is_err());
}

// ---- SL-268 PHASE-04: the turn journal (design sec-2, D1) ----

/// `(act, role)` of every turn on one finding, in file order.
fn turn_acts(doc: &ReviewDoc, finding: &str) -> Vec<(String, String)> {
    doc.finding
        .iter()
        .find(|f| f.id == finding)
        .unwrap()
        .turn
        .iter()
        .map(|t| (t.act.clone(), t.role.clone()))
        .collect()
}

/// Every turn in the ledger, finding and review level.
fn turn_count(doc: &ReviewDoc) -> usize {
    doc.finding.iter().map(|f| f.turn.len()).sum::<usize>() + doc.review.turn.len()
}

/// A raised-then-disposed F-1 on a fresh RV-001 — the common start for the note
/// tests.
fn fixture_answered_f1() -> tempfile::TempDir {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Major, "t"),
        Role::Raiser,
    )
    .unwrap();
    run_dispose(
        Some(root.to_path_buf()),
        &dispose_args("RV-001", "F-1"),
        Role::Responder,
    )
    .unwrap();
    tmp
}

/// EX-3 (the named flip of the former baton-note test, SL-268 PHASE-04 EX-6):
/// a contest or verify `--note` is the turn's recorded reasoning, in the ledger;
/// the baton carries no note at all.
#[test]
fn note_lands_in_turn() {
    let tmp = fixture_answered_f1();
    let root = tmp.path();
    let p = || Some(root.to_path_buf());
    run_contest(
        p(),
        "RV-001",
        "F-1",
        "please address the edge case",
        Role::Raiser,
    )
    .unwrap();
    run_dispose(p(), &dispose_args("RV-001", "F-1"), Role::Responder).unwrap();
    run_verify(p(), "RV-001", "F-1", Some("ok now"), Role::Raiser).unwrap();

    let doc = read_doc(root, 1);
    let turns = &doc.finding[0].turn;
    let contest = turns.iter().find(|t| t.act == "contest").unwrap();
    assert_eq!(
        contest.note.as_deref(),
        Some("please address the edge case")
    );
    let verify = turns.iter().find(|t| t.act == "verify").unwrap();
    assert_eq!(verify.note.as_deref(), Some("ok now"));
    let baton = fs::read_to_string(baton_path(root, 1)).unwrap();
    assert!(
        !baton.contains("please address") && !baton.contains("ok now"),
        "the baton carries no note: {baton}"
    );
}

/// EX-2 / VT-1: every act appends exactly one turn with its act and role, and
/// the parent finding's status moves in the same write (one read after each
/// call sees both). F-1's later turns, appended after F-2 exists, stay F-1's —
/// the file-order placement a TOML parse binds them by (R1).
#[test]
fn every_act_appends_one_turn() {
    let tmp = fixture_rv();
    let root = tmp.path();
    let p = || Some(root.to_path_buf());
    let mut total = 0;
    let mut step = |finding: &str, act: &str, role: &str, status: &str| {
        let doc = read_doc(root, 1);
        total += 1;
        assert_eq!(turn_count(&doc), total, "one turn per act ({act})");
        let last = turn_acts(&doc, finding).pop().unwrap();
        assert_eq!(last, (act.to_owned(), role.to_owned()));
        let row = doc.finding.iter().find(|f| f.id == finding).unwrap();
        assert_eq!(row.status, status, "status moved with the {act} turn");
    };

    run_raise(
        p(),
        &raise_args("RV-001", Severity::Major, "a"),
        Role::Raiser,
    )
    .unwrap();
    step("F-1", "raise", "raiser", "open");
    run_raise(
        p(),
        &raise_args("RV-001", Severity::Minor, "b"),
        Role::Raiser,
    )
    .unwrap();
    step("F-2", "raise", "raiser", "open");
    run_dispose(p(), &dispose_args("RV-001", "F-1"), Role::Responder).unwrap();
    step("F-1", "dispose", "responder", "answered");
    run_contest(p(), "RV-001", "F-1", "no", Role::Raiser).unwrap();
    step("F-1", "contest", "raiser", "contested");
    run_dispose(p(), &dispose_args("RV-001", "F-1"), Role::Responder).unwrap();
    step("F-1", "dispose", "responder", "answered");
    run_verify(p(), "RV-001", "F-1", None, Role::Raiser).unwrap();
    step("F-1", "verify", "raiser", "verified");
    run_withdraw(p(), "RV-001", "F-2", None, Role::Raiser).unwrap();
    step("F-2", "withdraw", "raiser", "withdrawn");

    run_conclude(p(), "RV-001", Role::Raiser).unwrap();
    let doc = read_doc(root, 1);
    assert_eq!(turn_count(&doc), total + 1, "conclude journals one turn");
    let conclude = doc.review.turn.last().unwrap();
    assert_eq!(
        (
            conclude.act.as_str(),
            conclude.role.as_str(),
            conclude.note.as_deref()
        ),
        ("conclude", "raiser", None)
    );
    assert!(doc.review.concluded, "the latch moved with the turn");
    assert_eq!(
        turn_acts(&doc, "F-1")
            .into_iter()
            .map(|(a, _)| a)
            .collect::<Vec<_>>(),
        ["raise", "dispose", "contest", "dispose", "verify"]
    );
}

/// EX-2: a dispose turn snapshots the answer it gave, so a re-dispose updates
/// the finding's current answer without erasing the earlier one.
#[test]
fn dispose_turn_snapshots_answer() {
    let tmp = fixture_answered_f1();
    let root = tmp.path();
    let p = || Some(root.to_path_buf());
    run_contest(p(), "RV-001", "F-1", "partial", Role::Raiser).unwrap();
    let second = DisposeArgs {
        disposition: Disposition::Tolerated,
        response: "second answer".to_owned(),
        ..dispose_args("RV-001", "F-1")
    };
    run_dispose(p(), &second, Role::Responder).unwrap();

    let doc = read_doc(root, 1);
    let f1 = &doc.finding[0];
    assert_eq!(
        (f1.disposition.as_deref(), f1.response.as_deref()),
        (Some("tolerated"), Some("second answer")),
        "the finding carries the current answer"
    );
    let answers: Vec<_> = f1
        .turn
        .iter()
        .filter(|t| t.act == "dispose")
        .map(|t| {
            (
                t.disposition.as_deref(),
                t.response.as_deref(),
                t.note.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        answers,
        [
            (Some("fix-now"), Some("done"), None),
            (Some("tolerated"), Some("second answer"), None)
        ]
    );
}

/// EX-3 / VT-2: a contest needs a non-empty note. A blank one refuses before the
/// lock, so neither the ledger nor the baton is touched.
#[test]
fn contest_requires_note() {
    let tmp = fixture_answered_f1();
    let root = tmp.path();
    let ledger_before = fs::read_to_string(authored_path(root, 1)).unwrap();
    let baton_before = fs::read_to_string(baton_path(root, 1)).unwrap();
    for blank in ["", "  \n\t"] {
        let err = run_contest(
            Some(root.to_path_buf()),
            "RV-001",
            "F-1",
            blank,
            Role::Raiser,
        )
        .unwrap_err();
        match err.downcast_ref::<ReviewError>() {
            Some(ReviewError::NoteRequired { act }) => assert_eq!(*act, Act::Contest),
            other => panic!("expected NoteRequired, got {other:?} ({err})"),
        }
        assert_eq!(err.to_string(), "`contest` requires a non-empty --note");
    }
    assert_eq!(
        fs::read_to_string(authored_path(root, 1)).unwrap(),
        ledger_before
    );
    assert_eq!(
        fs::read_to_string(baton_path(root, 1)).unwrap(),
        baton_before
    );
    assert!(!lock_path(root, 1).exists(), "no lock left behind");
}

/// EX-3: a withdraw `--note` lands in the withdraw turn; without one the turn
/// carries no `note` key.
#[test]
fn withdraw_note_lands_in_turn() {
    let tmp = fixture_rv();
    let root = tmp.path();
    let p = || Some(root.to_path_buf());
    run_raise(
        p(),
        &raise_args("RV-001", Severity::Major, "a"),
        Role::Raiser,
    )
    .unwrap();
    run_raise(
        p(),
        &raise_args("RV-001", Severity::Major, "b"),
        Role::Raiser,
    )
    .unwrap();
    run_withdraw(p(), "RV-001", "F-1", Some("duplicate of F-2"), Role::Raiser).unwrap();
    run_withdraw(p(), "RV-001", "F-2", None, Role::Raiser).unwrap();

    let doc = read_doc(root, 1);
    let noted = doc.finding[0].turn.last().unwrap();
    assert_eq!(
        (noted.act.as_str(), noted.note.as_deref()),
        ("withdraw", Some("duplicate of F-2"))
    );
    assert_eq!(doc.finding[1].turn.last().unwrap().note, None);
}

// =====================================================================
// SL-268 PHASE-05 — amend, reopen (VT-1..3)
// =====================================================================

/// A raised → disposed → verified F-1 on a fresh RV-001 — the common start for
/// the reopen tests.
fn fixture_verified_f1() -> tempfile::TempDir {
    let tmp = fixture_answered_f1();
    let root = tmp.path();
    run_verify(
        Some(root.to_path_buf()),
        "RV-001",
        "F-1",
        None,
        Role::Raiser,
    )
    .unwrap();
    tmp
}

fn amend_args(reference: &str, finding: &str, response: &str, note: &str) -> AmendArgs {
    AmendArgs {
        reference: reference.to_owned(),
        finding: finding.to_owned(),
        response: response.to_owned(),
        note: note.to_owned(),
        disposition: None,
        route: None,
    }
}

/// Stand a fresh RV-001/F-1 up in the given status, driven through the real
/// verbs where possible; `"zombie"` hand-edits an out-of-vocabulary status and
/// heals the baton's entry CAS (the `unknown_status_refuses_every_act` shape).
fn seeded_status(status: &str) -> tempfile::TempDir {
    let tmp = fixture_rv();
    let root = tmp.path();
    let p = || Some(root.to_path_buf());
    run_raise(
        p(),
        &raise_args("RV-001", Severity::Major, "t"),
        Role::Raiser,
    )
    .unwrap();
    match status {
        "open" => {}
        "answered" => {
            run_dispose(p(), &dispose_args("RV-001", "F-1"), Role::Responder).unwrap();
        }
        "contested" => {
            run_dispose(p(), &dispose_args("RV-001", "F-1"), Role::Responder).unwrap();
            run_contest(p(), "RV-001", "F-1", "n", Role::Raiser).unwrap();
        }
        "verified" => {
            run_dispose(p(), &dispose_args("RV-001", "F-1"), Role::Responder).unwrap();
            run_verify(p(), "RV-001", "F-1", None, Role::Raiser).unwrap();
        }
        "withdrawn" => {
            run_withdraw(p(), "RV-001", "F-1", None, Role::Raiser).unwrap();
        }
        "zombie" => {
            hand_edit_finding(root, 1, "F-1", "status", "zombie");
            run_status(p(), "RV-001").unwrap();
        }
        other => panic!("seeded_status: unhandled status {other}"),
    }
    tmp
}

/// VT-1: amend moves answered → answered. Omitted `--disposition`/`--route`
/// keep the finding's dispose-time values (A2); the amend turn snapshots the
/// effective disposition/route/response it leaves standing.
#[test]
fn amend_answered_to_answered() {
    let tmp = fixture_answered_f1();
    let root = tmp.path();
    let out = run_amend(
        Some(root.to_path_buf()),
        &amend_args("RV-001", "F-1", "R2", "n1"),
        Role::Responder,
    )
    .unwrap();
    match out {
        ReviewOutput::Amended {
            finding_id,
            review_id,
        } => assert_eq!((finding_id.as_str(), review_id), ("F-1", 1)),
        other => panic!("expected Amended, got {other:?}"),
    }
    let doc = read_doc(root, 1);
    let f = &doc.finding[0];
    assert_eq!(f.status, "answered");
    assert_eq!(
        f.disposition.as_deref(),
        Some("fix-now"),
        "kept when omitted"
    );
    assert_eq!(f.response.as_deref(), Some("R2"));
    let amend = f.turn.iter().find(|t| t.act == "amend").unwrap();
    assert_eq!(
        (
            amend.role.as_str(),
            amend.note.as_deref(),
            amend.disposition.as_deref(),
            amend.response.as_deref()
        ),
        ("responder", Some("n1"), Some("fix-now"), Some("R2"))
    );
}

/// VT-1: reopen moves verified → contested, journalling the note.
#[test]
fn reopen_verified_to_contested() {
    let tmp = fixture_verified_f1();
    let root = tmp.path();
    let out = run_reopen(
        Some(root.to_path_buf()),
        "RV-001",
        "F-1",
        "n3",
        Role::Raiser,
    )
    .unwrap();
    match out {
        ReviewOutput::Reopened {
            finding_id,
            review_id,
        } => assert_eq!((finding_id.as_str(), review_id), ("F-1", 1)),
        other => panic!("expected Reopened, got {other:?}"),
    }
    let doc = read_doc(root, 1);
    assert_eq!(doc.finding[0].status, "contested");
    let reopen = doc.finding[0].turn.last().unwrap();
    assert_eq!(
        (
            reopen.act.as_str(),
            reopen.role.as_str(),
            reopen.note.as_deref()
        ),
        ("reopen", "raiser", Some("n3"))
    );
}

/// VT-1: amend refuses outside `answered` — every other status and an
/// out-of-vocabulary one each give `StateMismatch`/`UnknownStatus`, ledger
/// unchanged. Also one `RoleMismatch` case.
#[test]
fn amend_refused_outside_answered() {
    for status in ["open", "contested", "verified", "withdrawn"] {
        let tmp = seeded_status(status);
        let root = tmp.path();
        let before = fs::read_to_string(authored_path(root, 1)).unwrap();
        let err = run_amend(
            Some(root.to_path_buf()),
            &amend_args("RV-001", "F-1", "R", "n"),
            Role::Responder,
        )
        .unwrap_err();
        match err.downcast_ref::<ReviewError>() {
            Some(ReviewError::StateMismatch { finding, act, .. }) => {
                assert_eq!((finding.as_str(), *act), ("F-1", Act::Amend));
            }
            other => panic!("expected StateMismatch for {status}, got {other:?} ({err})"),
        }
        assert_eq!(
            fs::read_to_string(authored_path(root, 1)).unwrap(),
            before,
            "{status}"
        );
    }

    let tmp = seeded_status("zombie");
    let root = tmp.path();
    let before = fs::read_to_string(authored_path(root, 1)).unwrap();
    let err = run_amend(
        Some(root.to_path_buf()),
        &amend_args("RV-001", "F-1", "R", "n"),
        Role::Responder,
    )
    .unwrap_err();
    match err.downcast_ref::<ReviewError>() {
        Some(ReviewError::UnknownStatus { finding, raw }) => {
            assert_eq!((finding.as_str(), raw.as_str()), ("F-1", "zombie"));
        }
        other => panic!("expected UnknownStatus, got {other:?} ({err})"),
    }
    assert_eq!(fs::read_to_string(authored_path(root, 1)).unwrap(), before);

    // One role case: amend --as raiser refuses RoleMismatch before the
    // per-finding gate ever runs.
    let tmp = fixture_answered_f1();
    let root = tmp.path();
    let before = fs::read_to_string(authored_path(root, 1)).unwrap();
    let err = run_amend(
        Some(root.to_path_buf()),
        &amend_args("RV-001", "F-1", "R", "n"),
        Role::Raiser,
    )
    .unwrap_err();
    match err.downcast_ref::<ReviewError>() {
        Some(ReviewError::RoleMismatch { act, .. }) => assert_eq!(*act, Act::Amend),
        other => panic!("expected RoleMismatch, got {other:?} ({err})"),
    }
    assert_eq!(fs::read_to_string(authored_path(root, 1)).unwrap(), before);
}

/// VT-1: reopen refuses outside `verified` — every other status and an
/// out-of-vocabulary one each give `StateMismatch`/`UnknownStatus`, ledger
/// unchanged. Also one `RoleMismatch` case.
#[test]
fn reopen_refused_outside_verified() {
    for status in ["open", "answered", "contested", "withdrawn"] {
        let tmp = seeded_status(status);
        let root = tmp.path();
        let before = fs::read_to_string(authored_path(root, 1)).unwrap();
        let err =
            run_reopen(Some(root.to_path_buf()), "RV-001", "F-1", "n", Role::Raiser).unwrap_err();
        match err.downcast_ref::<ReviewError>() {
            Some(ReviewError::StateMismatch { finding, act, .. }) => {
                assert_eq!((finding.as_str(), *act), ("F-1", Act::Reopen));
            }
            other => panic!("expected StateMismatch for {status}, got {other:?} ({err})"),
        }
        assert_eq!(
            fs::read_to_string(authored_path(root, 1)).unwrap(),
            before,
            "{status}"
        );
    }

    let tmp = seeded_status("zombie");
    let root = tmp.path();
    let before = fs::read_to_string(authored_path(root, 1)).unwrap();
    let err = run_reopen(Some(root.to_path_buf()), "RV-001", "F-1", "n", Role::Raiser).unwrap_err();
    match err.downcast_ref::<ReviewError>() {
        Some(ReviewError::UnknownStatus { finding, raw }) => {
            assert_eq!((finding.as_str(), raw.as_str()), ("F-1", "zombie"));
        }
        other => panic!("expected UnknownStatus, got {other:?} ({err})"),
    }
    assert_eq!(fs::read_to_string(authored_path(root, 1)).unwrap(), before);

    // One role case: reopen --as responder refuses RoleMismatch.
    let tmp = fixture_verified_f1();
    let root = tmp.path();
    let before = fs::read_to_string(authored_path(root, 1)).unwrap();
    let err = run_reopen(
        Some(root.to_path_buf()),
        "RV-001",
        "F-1",
        "n",
        Role::Responder,
    )
    .unwrap_err();
    match err.downcast_ref::<ReviewError>() {
        Some(ReviewError::RoleMismatch { act, .. }) => assert_eq!(*act, Act::Reopen),
        other => panic!("expected RoleMismatch, got {other:?} ({err})"),
    }
    assert_eq!(fs::read_to_string(authored_path(root, 1)).unwrap(), before);
}

/// EX-3 / VT-2: amend needs a non-empty note (A3). A blank one refuses before
/// the lock, so neither the ledger nor the baton is touched (the
/// `contest_requires_note` shape).
#[test]
fn amend_requires_note() {
    let tmp = fixture_answered_f1();
    let root = tmp.path();
    let ledger_before = fs::read_to_string(authored_path(root, 1)).unwrap();
    let baton_before = fs::read_to_string(baton_path(root, 1)).unwrap();
    for blank in ["", "  \n\t"] {
        let err = run_amend(
            Some(root.to_path_buf()),
            &amend_args("RV-001", "F-1", "R", blank),
            Role::Responder,
        )
        .unwrap_err();
        match err.downcast_ref::<ReviewError>() {
            Some(ReviewError::NoteRequired { act }) => assert_eq!(*act, Act::Amend),
            other => panic!("expected NoteRequired, got {other:?} ({err})"),
        }
        assert_eq!(err.to_string(), "`amend` requires a non-empty --note");
    }
    assert_eq!(
        fs::read_to_string(authored_path(root, 1)).unwrap(),
        ledger_before
    );
    assert_eq!(
        fs::read_to_string(baton_path(root, 1)).unwrap(),
        baton_before
    );
    assert!(!lock_path(root, 1).exists(), "no lock left behind");
}

/// EX-3 / VT-2: reopen needs a non-empty note, in the same shape.
#[test]
fn reopen_requires_note() {
    let tmp = fixture_verified_f1();
    let root = tmp.path();
    let ledger_before = fs::read_to_string(authored_path(root, 1)).unwrap();
    let baton_before = fs::read_to_string(baton_path(root, 1)).unwrap();
    for blank in ["", "  \n\t"] {
        let err = run_reopen(
            Some(root.to_path_buf()),
            "RV-001",
            "F-1",
            blank,
            Role::Raiser,
        )
        .unwrap_err();
        match err.downcast_ref::<ReviewError>() {
            Some(ReviewError::NoteRequired { act }) => assert_eq!(*act, Act::Reopen),
            other => panic!("expected NoteRequired, got {other:?} ({err})"),
        }
        assert_eq!(err.to_string(), "`reopen` requires a non-empty --note");
    }
    assert_eq!(
        fs::read_to_string(authored_path(root, 1)).unwrap(),
        ledger_before
    );
    assert_eq!(
        fs::read_to_string(baton_path(root, 1)).unwrap(),
        baton_before
    );
    assert!(!lock_path(root, 1).exists(), "no lock left behind");
}

/// VT-3: `Disposition::parse`/`Route::parse` on an unknown token names every
/// token of their own set.
#[test]
fn disposition_refusal_names_set() {
    let err = Disposition::parse("bogus").unwrap_err();
    for token in DISPOSITIONS {
        assert!(err.contains(token), "{err} names {token}");
    }
    let err = Route::parse("bogus").unwrap_err();
    for token in ROUTES {
        assert!(err.contains(token), "{err} names {token}");
    }
}

/// VT-3: the retired `route:` prose prefix is refused pointing at `--route`,
/// naming the routes — not as a generic unknown-disposition message.
#[test]
fn route_prefix_refused() {
    let err = Disposition::parse("route:probe fix-now").unwrap_err();
    assert!(err.contains("--route"), "{err}");
    for token in ROUTES {
        assert!(err.contains(token), "{err} names {token}");
    }
}

/// EX-4 / VT-3: the first journalled write seeds `rounds_base`/`contests_base`
/// from the legacy baton, in the same write; the baton stops counting; status
/// reports base plus turns.
#[test]
fn first_journalled_write_seeds_base() {
    let tmp = fixture_rv();
    let root = tmp.path();
    let p = || Some(root.to_path_buf());
    let text = fs::read_to_string(authored_path(root, 1)).unwrap();
    let legacy = Baton {
        awaiting: "raiser".to_owned(),
        authored_hash: crate::git::sha256(text.as_bytes()),
        rounds: 4,
        contests: 1,
    };
    write_baton(root, 1, &legacy).unwrap();

    run_raise(
        p(),
        &raise_args("RV-001", Severity::Major, "a"),
        Role::Raiser,
    )
    .unwrap();
    let doc = read_doc(root, 1);
    assert_eq!(
        (doc.review.rounds_base, doc.review.contests_base),
        (Some(4), Some(1))
    );
    run_dispose(p(), &dispose_args("RV-001", "F-1"), Role::Responder).unwrap();
    let doc = read_doc(root, 1);
    assert_eq!(doc.review.rounds_base, Some(4), "seeded once, never moved");
    let baton = read_baton(root, 1).unwrap().unwrap();
    assert_eq!(
        (baton.rounds, baton.contests),
        (4, 1),
        "the baton stops counting"
    );

    match run_status(p(), "RV-001").unwrap() {
        ReviewOutput::Status {
            rounds, formatted, ..
        } => {
            assert_eq!(rounds, 6, "base 4 plus 2 turns");
            assert!(formatted.contains("rounds 6"), "{formatted}");
        }
        other => panic!("expected Status, got {other:?}"),
    }
}

// ---- PHASE-04: reverse close-gate scan (design §7, D8/D-C9b) ----

/// VT-3 / VT-1: an Active RV with a raised (open) **blocker** finding is
/// reported by the scan, keyed `RV-NNN`/`F-n`, and matched by `[target].ref`.
#[test]
fn vt3_scan_reports_an_unresolved_blocker_on_an_active_rv() {
    let tmp = fixture_rv(); // RV-001 → SL-001
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Blocker, "must fix"),
        Role::Raiser,
    )
    .unwrap();
    // Active (one open finding) + blocker ⇒ one BlockerRef RV-001/F-1.
    assert_eq!(read_doc(root, 1).derived().0, ReviewStatus::Active);
    let blockers = unresolved_blockers_for(root, "SL-001").unwrap();
    assert_eq!(
        blockers,
        vec![BlockerRef {
            rv: "RV-001".to_owned(),
            finding: "F-1".to_owned(),
            reason: None,
        }]
    );
}

/// VT-3: a non-matching `[target].ref` is ignored — the scan is subject-scoped.
#[test]
fn vt3_scan_ignores_a_non_matching_target() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Blocker, "must fix"),
        Role::Raiser,
    )
    .unwrap();
    // RV-001 targets SL-001; a query for an unrelated subject finds nothing.
    assert!(unresolved_blockers_for(root, "SL-999").unwrap().is_empty());
}

/// VT-3: a non-blocker finding (major/minor/nit) never gates — only `blocker`.
#[test]
fn vt3_scan_ignores_a_non_blocker_finding() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Major, "nice to have"),
        Role::Raiser,
    )
    .unwrap();
    assert!(unresolved_blockers_for(root, "SL-001").unwrap().is_empty());
}

/// VT-1 / VT-3: a **verified** blocker is terminal ⇒ the RV is Done ⇒ the scan
/// reports nothing (the finding is resolved AND the review is no longer Active).
#[test]
fn vt1_verified_blocker_is_terminal_and_not_reported() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Blocker, "must fix"),
        Role::Raiser,
    )
    .unwrap();
    run_dispose(
        Some(root.to_path_buf()),
        &dispose_args("RV-001", "F-1"),
        Role::Responder,
    )
    .unwrap();
    run_verify(
        Some(root.to_path_buf()),
        "RV-001",
        "F-1",
        None,
        Role::Raiser,
    )
    .unwrap();
    // All findings terminal ⇒ Done (D-C9a) ⇒ no unresolved blocker.
    assert_eq!(read_doc(root, 1).derived().0, ReviewStatus::Done);
    assert!(unresolved_blockers_for(root, "SL-001").unwrap().is_empty());
}

/// VT-1 / VT-3: a **withdrawn** blocker is terminal ⇒ Done ⇒ not reported.
#[test]
fn vt1_withdrawn_blocker_is_terminal_and_not_reported() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Blocker, "must fix"),
        Role::Raiser,
    )
    .unwrap();
    run_withdraw(
        Some(root.to_path_buf()),
        "RV-001",
        "F-1",
        None,
        Role::Raiser,
    )
    .unwrap();
    assert_eq!(read_doc(root, 1).derived().0, ReviewStatus::Done);
    assert!(unresolved_blockers_for(root, "SL-001").unwrap().is_empty());
}

/// VT-1: a **non-terminal** blocker keeps the review Active and gating — a
/// blocker disposed-but-not-yet-verified (answered) still gates (D-C9a: review
/// is done only when EVERY finding is terminal ∈ {verified, withdrawn}).
#[test]
fn vt1_answered_blocker_keeps_the_review_active_and_gating() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Blocker, "must fix"),
        Role::Raiser,
    )
    .unwrap();
    run_dispose(
        Some(root.to_path_buf()),
        &dispose_args("RV-001", "F-1"),
        Role::Responder,
    )
    .unwrap();
    // answered ∉ {verified, withdrawn} ⇒ Active ⇒ still an unresolved blocker.
    assert_eq!(read_doc(root, 1).derived().0, ReviewStatus::Active);
    let blockers = unresolved_blockers_for(root, "SL-001").unwrap();
    assert_eq!(blockers.len(), 1);
    assert_eq!(blockers[0].finding, "F-1");
}

// ---- SL-244 PHASE-04: the design run's own blocker predicate (DEC-138) ----

/// SL-244 `VT-3`: the deliberate mirror of
/// [`vt1_answered_blocker_keeps_the_review_active_and_gating`] directly above.
///
/// D-C9b's close-gate carries an `answered` blocker because a slice is not
/// closed until every finding is terminal. The design run's edge asks a
/// different question — *has this pass been disposed of* — and an answered
/// blocker HAS been. The two predicates differ by exactly this state, on
/// purpose (`EX-6`), which is why the same ledger is asserted through both
/// here rather than each in isolation.
#[test]
fn answered_blocker_is_not_undisposed() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Blocker, "must fix"),
        Role::Raiser,
    )
    .unwrap();
    run_dispose(
        Some(root.to_path_buf()),
        &dispose_args("RV-001", "F-1"),
        Role::Responder,
    )
    .unwrap();

    // The close-gate still gates on it...
    assert_eq!(unresolved_blockers_for(root, "SL-001").unwrap().len(), 1);
    // ...and the design run's edge does not.
    assert!(undisposed_blockers(&read_doc(root, 1)).is_empty());
}

/// SL-244 `VT-3`: `open` and `contested` are both carried, by the ledger's own
/// `F-n` id — they identify rows on the RV, not subjects in the run.
///
/// Both states in one ledger rather than two tests, because the predicate is a
/// set membership and a per-state test cannot see a filter that drops one of
/// them. The non-blocker severity rides along as the negative control.
#[test]
fn open_and_contested_blockers_are_carried_by_finding_id() {
    let tmp = fixture_rv();
    let root = tmp.path();
    for (severity, title) in [
        (Severity::Blocker, "F-1 stays open"),
        (Severity::Blocker, "F-2 is contested"),
        (Severity::Major, "F-3 never gates"),
    ] {
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", severity, title),
            Role::Raiser,
        )
        .unwrap();
    }
    run_dispose(
        Some(root.to_path_buf()),
        &dispose_args("RV-001", "F-2"),
        Role::Responder,
    )
    .unwrap();
    run_contest(Some(root.to_path_buf()), "RV-001", "F-2", "n", Role::Raiser).unwrap();

    assert_eq!(
        undisposed_blockers(&read_doc(root, 1)),
        vec!["F-1".to_owned(), "F-2".to_owned()]
    );
}

/// SL-244 `EX-6b`: the predicate reads `severity` and `status` per finding and
/// nothing else — in particular it does NOT inherit
/// [`doc_unresolved_blockers`]'s review-level `derived_status` guard.
///
/// The guard is currently **unobservable** for this predicate's own answers:
/// any finding in `{open, contested}` forces `derived_status` to `Active`, so
/// the early return can never fire while there is something to report. That is
/// worth asserting rather than assuming, because it is the reason `EX-6b`
/// argues from coupling (DEC-138 on ADR-007 D7's ground — the review-level
/// status is a display summary and never a gate) rather than from a wrong
/// answer, and a future status rule that broke the implication would make the
/// inherited guard start dropping live blockers silently.
#[test]
fn the_predicate_does_not_read_review_level_status() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Blocker, "must fix"),
        Role::Raiser,
    )
    .unwrap();
    let doc = read_doc(root, 1);
    assert_eq!(
        doc.derived().0,
        ReviewStatus::Active,
        "an open blocker forces Active — the implication this test pins"
    );
    assert_eq!(undisposed_blockers(&doc), vec!["F-1".to_owned()]);
}

/// SL-244 `VT-4`: absence is refusal, not satisfaction.
///
/// The three cases are asserted together because they must give the *same*
/// answer for different reasons, and the failure being guarded is that one of
/// them quietly returns an empty observation instead of none. An empty
/// observation reads as *no blockers*, which clears the very edge an
/// unreadable review has to hold — so `None` and `Some(no blockers)` are
/// opposite verdicts wearing similar shapes, and only a test that names both
/// halves catches a resolver that conflates them.
#[test]
fn unreadable_review_reads_as_unmet() {
    let tmp = fixture_rv();
    let root = tmp.path();

    // Not a ref at all, and a well-formed ref naming no ledger: both unreadable.
    assert!(observe_pass(root, "not-a-ref").is_none());
    assert!(observe_pass(root, "RV-999").is_none());

    // A readable ledger with nothing outstanding is `Some` with an EMPTY
    // blocker set — the positive control that distinguishes "cannot see it"
    // from "saw it, nothing there".
    let clean = observe_pass(root, "RV-001").expect("a readable ledger is observable");
    assert!(clean.undisposed_blockers.is_empty());
    assert!(
        !clean.concluded,
        "a ledger nobody concluded reads unconcluded — absence is the answer, \
         not a missing feature (the marker landed with IMP-392's carve)"
    );

    // And a readable ledger holding a live blocker carries it by F-n id.
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Blocker, "must fix"),
        Role::Raiser,
    )
    .unwrap();
    assert_eq!(
        observe_pass(root, "RV-001").unwrap().undisposed_blockers,
        vec!["F-1".to_owned()]
    );
}

// ---- IMP-392: the concluded-pass marker (SL-244 `sec-4`) ----

/// The marker is a **latch**: absent means not-concluded, `conclude` sets it,
/// and concluding again is a no-op success rather than a refusal.
///
/// Both halves are asserted here because the failure mode is asymmetric — a
/// verb that refuses the second call looks correct in the happy path and
/// breaks exactly the caller who cannot know whether a pass was already
/// closed, which is the caller the latch exists for.
#[test]
fn conclude_latches_the_marker_and_is_idempotent() {
    let tmp = fixture_rv();
    let root = tmp.path();

    assert!(
        !observe_pass(root, "RV-001").unwrap().concluded,
        "absence is not-concluded — a fresh ledger carries no marker"
    );

    let first = run_conclude(Some(root.to_path_buf()), "RV-001", Role::Raiser).unwrap();
    assert!(
        matches!(first, ReviewOutput::Concluded { already: false, .. }),
        "the first conclude sets the latch: {first:?}"
    );
    assert!(observe_pass(root, "RV-001").unwrap().concluded);

    let second = run_conclude(Some(root.to_path_buf()), "RV-001", Role::Raiser).unwrap();
    assert!(
        matches!(second, ReviewOutput::Concluded { already: true, .. }),
        "concluding a concluded pass is a no-op, not a refusal: {second:?}"
    );
    assert!(
        observe_pass(root, "RV-001").unwrap().concluded,
        "and the latch stays set — there is no unset"
    );
}

/// Concluding is the **raiser's** act — *I have finished reading* — on the
/// same authority axis that separates `raise`/`verify`/`withdraw` from the
/// responder's `dispose` (SL-244 `sec-4`).
#[test]
fn conclude_is_the_raisers_verb() {
    let tmp = fixture_rv();
    let root = tmp.path();

    let err = run_conclude(Some(root.to_path_buf()), "RV-001", Role::Responder)
        .expect_err("the responder cannot conclude the raiser's pass");
    assert!(
        err.to_string().contains("raiser"),
        "the refusal names the required role: {err}"
    );
    assert!(!observe_pass(root, "RV-001").unwrap().concluded);
}

/// **Open findings are allowed, and this is the normal case.** A pass that
/// found things concludes with them outstanding; disposing them is the
/// responder's work afterwards. Requiring a clean ledger would make the
/// marker a second, stricter spelling of the gate it feeds.
#[test]
fn conclude_leaves_open_findings_alone() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Blocker, "must fix"),
        Role::Raiser,
    )
    .unwrap();

    run_conclude(Some(root.to_path_buf()), "RV-001", Role::Raiser).unwrap();

    let facts = observe_pass(root, "RV-001").unwrap();
    assert!(facts.concluded, "a live blocker does not block concluding");
    assert_eq!(
        facts.undisposed_blockers,
        vec!["F-1".to_owned()],
        "and the finding is untouched by the pass-level act"
    );
}

// ---- SL-244 PHASE-07: the severity summary (EX-2) ----

/// SL-244 `VT-4`: the summary is deliberately **wider** than the edge predicate.
///
/// Rehoused here from `VT-2` (`DEC-140`/`DEC-141` class) because the comparison
/// cannot be made where the summary renders: `envelope.rs` is leaf-tier and
/// cannot import `review` to ask the gate's predicate the same question.
///
/// One ledger asked twice, not two ledgers: an `answered` blocker has been
/// disposed of, so it does not hold the run's `reviewing → locked` edge — and it
/// is still outstanding, so the lamp must show it. A summary that reused the
/// gate's filter would go dark on exactly the findings a reader still owes work
/// on, which is the failure this pins.
///
/// Both halves are taken off one [`read_pass_facts`] rather than called
/// separately, so the test also pins that the divergence survives the shared
/// reader — two filters over one parse, which is what `D3` bought.
#[test]
fn severity_summary_is_wider_than_the_gate() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Blocker, "must fix"),
        Role::Raiser,
    )
    .unwrap();
    run_dispose(
        Some(root.to_path_buf()),
        &dispose_args("RV-001", "F-1"),
        Role::Responder,
    )
    .unwrap();

    let facts = read_pass_facts(root, "RV-001").unwrap();
    assert!(
        facts.undisposed_blockers.is_empty(),
        "an answered blocker does not hold the run's edge"
    );
    assert_eq!(
        facts.outstanding,
        OutstandingCounts {
            blocker: 1,
            ..OutstandingCounts::default()
        },
        "...and the lamp still shows it"
    );
}

/// SL-244 `EX-2`: outstanding is `status ∉ {verified, withdrawn}`, counted
/// across all four severities.
///
/// One ledger carrying every severity and both terminal states, because the
/// failures being guarded are a dropped severity arm and a terminal state that
/// leaks into a count — neither is visible to a test that exercises one
/// severity at a time. The two terminal rows are the negative control: they sit
/// on severities that are also counted elsewhere in the same ledger, so a
/// filter that stopped excluding them changes a number rather than adding one.
#[test]
fn outstanding_counts_span_the_severities_and_drop_the_terminal() {
    let tmp = fixture_rv();
    let root = tmp.path();
    for (severity, title) in [
        (Severity::Blocker, "F-1 open"),
        (Severity::Major, "F-2 answered"),
        (Severity::Minor, "F-3 contested"),
        (Severity::Nit, "F-4 open"),
        (Severity::Blocker, "F-5 verified"),
        (Severity::Major, "F-6 withdrawn"),
    ] {
        run_raise(
            Some(root.to_path_buf()),
            &raise_args("RV-001", severity, title),
            Role::Raiser,
        )
        .unwrap();
    }
    for finding in ["F-2", "F-3", "F-5"] {
        run_dispose(
            Some(root.to_path_buf()),
            &dispose_args("RV-001", finding),
            Role::Responder,
        )
        .unwrap();
    }
    run_contest(Some(root.to_path_buf()), "RV-001", "F-3", "n", Role::Raiser).unwrap();
    run_verify(
        Some(root.to_path_buf()),
        "RV-001",
        "F-5",
        None,
        Role::Raiser,
    )
    .unwrap();
    run_withdraw(
        Some(root.to_path_buf()),
        "RV-001",
        "F-6",
        None,
        Role::Raiser,
    )
    .unwrap();

    assert_eq!(
        outstanding_by_severity(&read_doc(root, 1)),
        OutstandingCounts {
            blocker: 1,
            major: 1,
            minor: 1,
            nit: 1,
        }
    );
}

/// SL-244 `D5` (the owner's 2026-08-05 ruling): one read, two postures.
///
/// The gate must read an unreadable pass as *refusal* — `None`, never an error a
/// caller can dismiss (`PHASE-04` `EX-5`, pinned by
/// [`unreadable_review_reads_as_unmet`] above). The projection path must **fail
/// loud** — there is no third quiet render state, so a lamp that cannot read its
/// own ledger says so, naming the reference, rather than rendering the silence
/// that means *nothing outstanding*.
///
/// Both postures are asserted over the same reference in one test, because the
/// regression is not either posture alone — it is the two drifting apart onto
/// two parses that disagree about what unreadable means.
#[test]
fn an_unreadable_pass_fails_loud_only_for_the_projection() {
    let tmp = fixture_rv();
    let root = tmp.path();

    for unreadable in ["not-a-ref", "RV-999"] {
        assert!(
            observe_pass(root, unreadable).is_none(),
            "the gate reads {unreadable} as refusal, not as an error"
        );
        let err = read_pass_facts(root, unreadable)
            .expect_err("the projection path refuses to render silence")
            .to_string();
        assert!(
            err.contains(unreadable),
            "the failure names the reference it could not read, got: {err}"
        );
    }

    // The positive control: a readable ledger is Ok through the same reader, so
    // the loud half is not loud about everything.
    assert!(read_pass_facts(root, "RV-001").is_ok());
}

/// VT-3: with no `.doctrine/review/` tree at all, the scan is a clean empty —
/// the gate degrades gracefully on a slice with no reviews.
#[test]
fn vt3_scan_with_no_review_tree_is_empty() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(
        unresolved_blockers_for(tmp.path(), "SL-001")
            .unwrap()
            .is_empty()
    );
}

/// `unlock` removes a stale lock; on an unlocked review it is a clean no-op.
#[test]
fn unlock_clears_a_stale_lock() {
    let tmp = fixture_rv();
    let root = tmp.path();
    // Plant a stale lock (a hard-kill residue RAII never cleared).
    let lock = lock_path(root, 1);
    fs::create_dir_all(lock.parent().unwrap()).unwrap();
    fs::write(&lock, "pid = 99999\nacquired = \"stale\"\n").unwrap();
    run_unlock(Some(root.to_path_buf()), "RV-001").unwrap();
    assert!(!lock.exists(), "stale lock removed");
    // Idempotent on an unlocked review.
    run_unlock(Some(root.to_path_buf()), "RV-001").unwrap();
}

// -- PHASE-05: warm-cache + prime (D-C10, §9) ----------------------------

/// Write `body` as a tracked source file under `root` (the warm-cache hashes
/// real bytes on disk).
fn plant_tracked(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, body).unwrap();
}

/// Stand up a git-backed RV (id 1) → SL-001, with the slice carrying the given
/// selectors and the listed files committed (so `git ls-files` sees them for
/// glob expansion). Returns the tempdir; root = `tmp.path()`.
fn git_fixture_rv_with_selectors(selectors: &[&str], files: &[(&str, &str)]) -> tempfile::TempDir {
    use std::process::Command;
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let git = |args: &[&str]| {
        let ok = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00 +0000")
            .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00 +0000")
            .output()
            .unwrap();
        assert!(
            ok.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&ok.stderr)
        );
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.name", "T"]);
    git(&["config", "user.email", "t@t.invalid"]);
    plant_slice_with_selectors(root, 1, selectors);
    run_new(Some(root.to_path_buf()), &new_args(Facet::Design, "SL-001")).unwrap();
    for (rel, body) in files {
        plant_tracked(root, rel, body);
    }
    git(&["add", "."]);
    git(&["commit", "-m", "seed"]);
    tmp
}

/// VT-1: a GLOB selector resolves to a multi-file set before hashing; the cache
/// `paths` is the resolved union (NOT the glob string), `[hashes]` covers it,
/// and the verdict is `current` straight after (SL-147 PHASE-05).
#[test]
fn vt1_glob_selector_resolves_to_a_fileset_then_current() {
    let tmp = git_fixture_rv_with_selectors(
        &["src/*.rs"],
        &[
            ("src/review.rs", "fn review() {}\n"),
            ("src/state.rs", "fn state() {}\n"),
            ("docs/readme.md", "# not matched\n"),
        ],
    );
    let root = tmp.path();
    run_prime(
        Some(root.to_path_buf()),
        &PrimeArgs {
            reference: "RV-001".to_owned(),
        },
    )
    .unwrap();

    let cache = read_cache(root, 1).unwrap().expect("cache primed");
    // The glob expanded to the two .rs files (the .md is excluded).
    assert_eq!(
        cache.tracked_paths(),
        vec!["src/review.rs".to_owned(), "src/state.rs".to_owned()]
    );
    let expected = contentset::compute(
        root,
        &["src/review.rs".to_owned(), "src/state.rs".to_owned()],
    )
    .unwrap();
    assert_eq!(&cache.hashes, expected.hashes());
    assert!(matches!(
        cache_staleness(root, &cache).unwrap(),
        CacheVerdict::Current
    ));
}

/// ISS-259: entity roots contain committed slug symlinks whose targets are
/// directories. A glob must track the regular files beneath the root without
/// passing the symlink itself to the content hasher.
#[test]
#[cfg(unix)]
fn prime_glob_ignores_tracked_directory_symlink() {
    use std::process::Command;

    let tmp = git_fixture_rv_with_selectors(
        &[".doctrine/spec/product/**"],
        &[(".doctrine/spec/product/001/spec-001.md", "# Product spec\n")],
    );
    let root = tmp.path();
    std::os::unix::fs::symlink("001", root.join(".doctrine/spec/product/001-product-spec"))
        .unwrap();
    for args in [&["add", "."][..], &["commit", "-m", "add slug symlink"][..]] {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    run_prime(
        Some(root.to_path_buf()),
        &PrimeArgs {
            reference: "RV-001".to_owned(),
        },
    )
    .unwrap();

    let cache = read_cache(root, 1).unwrap().expect("cache primed");
    assert_eq!(
        cache.tracked_paths(),
        vec![".doctrine/spec/product/001/spec-001.md".to_owned()]
    );
}

#[test]
fn staged_files_parser_preserves_paths_and_filters_non_files() {
    let listing = concat!(
        "100644 aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa 0\tpath with spaces.rs\0",
        "100755 bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb 0\tscript.sh\0",
        "120000 cccccccccccccccccccccccccccccccccccccccc 0\tdirectory-link\0",
        "160000 dddddddddddddddddddddddddddddddddddddddd 0\tsubmodule\0",
    );
    assert_eq!(
        parse_ls_files_stage_entries(listing).unwrap(),
        vec!["path with spaces.rs".to_owned(), "script.sh".to_owned()]
    );
}

#[test]
fn staged_files_parser_rejects_malformed_record() {
    let err = parse_ls_files_stage_entries("100644 missing-fields-and-tab\0").unwrap_err();
    assert!(
        err.to_string()
            .contains("malformed git ls-files --stage record"),
        "named parser failure: {err:#}"
    );
}

/// VT-1 (literal): a literal selector passes through degenerate (no tree
/// resolution); after prime the verdict is `current`, content drift names it,
/// and removing it ⇒ stale naming it (R1 absence⇒stale preserved).
#[test]
fn vt1_literal_selector_passes_through_and_absence_is_stale() {
    let tmp = git_fixture_rv_with_selectors(
        &["src/review.rs", "src/state.rs"],
        &[("src/review.rs", "original\n"), ("src/state.rs", "state\n")],
    );
    let root = tmp.path();
    run_prime(
        Some(root.to_path_buf()),
        &PrimeArgs {
            reference: "RV-001".to_owned(),
        },
    )
    .unwrap();
    let cache = read_cache(root, 1).unwrap().unwrap();
    // Literals pass through unresolved — the path-set IS the two selectors.
    assert_eq!(
        cache.tracked_paths(),
        vec!["src/review.rs".to_owned(), "src/state.rs".to_owned()]
    );
    assert!(matches!(
        cache_staleness(root, &cache).unwrap(),
        CacheVerdict::Current
    ));

    // Content drift ⇒ stale naming exactly that path.
    fs::write(root.join("src/review.rs"), "MUTATED\n").unwrap();
    match cache_staleness(root, &cache).unwrap() {
        CacheVerdict::Stale(paths) => assert_eq!(paths, vec!["src/review.rs".to_owned()]),
        CacheVerdict::Current => panic!("expected stale after a content drift"),
    }

    // Restore, then REMOVE a tracked literal ⇒ absence⇒stale naming it (R1).
    fs::write(root.join("src/review.rs"), "original\n").unwrap();
    fs::remove_file(root.join("src/state.rs")).unwrap();
    match cache_staleness(root, &cache).unwrap() {
        CacheVerdict::Stale(paths) => assert_eq!(paths, vec!["src/state.rs".to_owned()]),
        CacheVerdict::Current => panic!("absent tracked path must be stale (R1)"),
    }
}

/// VT-2 (a): a non-slice RV target (a phase or backlog ref) cannot source
/// selectors — `run_prime` bails with a NAMED message and writes nothing.
#[test]
fn vt2_prime_bails_named_on_a_non_slice_target() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    // Plant a backlog target dir so `review new` resolves the ref, then mint an
    // RV against it (a non-slice target).
    let dir = root.join(".doctrine/backlog/issue/007");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("backlog-007.toml"), "id = 7\n").unwrap();
    run_new(
        Some(root.to_path_buf()),
        &new_args(Facet::Design, "ISS-007"),
    )
    .unwrap();

    let err = run_prime(
        Some(root.to_path_buf()),
        &PrimeArgs {
            reference: "RV-001".to_owned(),
        },
    )
    .unwrap_err();
    let msg = format!("{err:#}");
    assert!(
        msg.contains("not a slice reference") || msg.contains("needs a slice target"),
        "named non-slice failure: {msg}"
    );
    assert!(read_cache(root, 1).unwrap().is_none(), "nothing written");
}

/// VT-2 (b): a slice with ZERO selectors gives prime no path-set — `run_prime`
/// bails with a NAMED message and writes nothing.
#[test]
fn vt2_prime_bails_named_on_a_slice_with_no_selectors() {
    let tmp = fixture_rv(); // SL-001 has no selectors.
    let root = tmp.path();
    let err = run_prime(
        Some(root.to_path_buf()),
        &PrimeArgs {
            reference: "RV-001".to_owned(),
        },
    )
    .unwrap_err();
    let msg = format!("{err:#}");
    assert!(
        msg.contains("no selectors"),
        "named zero-selector failure: {msg}"
    );
    assert!(read_cache(root, 1).unwrap().is_none(), "nothing written");
}

/// `prime` rebuilds `[hashes]` from the resolved fileset — the baseline is the
/// live content, never a stale value.
#[test]
fn prime_recomputes_hashes_from_the_resolved_fileset() {
    let tmp = git_fixture_rv_with_selectors(&["a.txt"], &[("a.txt", "real content\n")]);
    let root = tmp.path();
    run_prime(
        Some(root.to_path_buf()),
        &PrimeArgs {
            reference: "RV-001".to_owned(),
        },
    )
    .unwrap();
    let cache = read_cache(root, 1).unwrap().unwrap();
    let expected = contentset::compute(root, &["a.txt".to_owned()]).unwrap();
    assert_eq!(&cache.hashes, expected.hashes());
    assert!(matches!(
        cache_staleness(root, &cache).unwrap(),
        CacheVerdict::Current
    ));
}

/// `prime` acquires the per-review lock around the cache write — a held lock
/// makes it bail "busy" (the §9 serialization, reusing the PHASE-03 LockGuard).
#[test]
fn prime_serializes_via_the_per_review_lock() {
    let tmp = git_fixture_rv_with_selectors(&["a.txt"], &[("a.txt", "x\n")]);
    let root = tmp.path();

    // Hold the lock, then prime must bail busy (no clobber).
    let held = LockGuard::acquire(root, 1).unwrap();
    let err = run_prime(
        Some(root.to_path_buf()),
        &PrimeArgs {
            reference: "RV-001".to_owned(),
        },
    )
    .unwrap_err();
    assert!(format!("{err}").contains("busy"), "lock contention: {err}");
    assert!(
        read_cache(root, 1).unwrap().is_none(),
        "no cache written under contention"
    );
    drop(held);

    // Lock free ⇒ prime succeeds.
    run_prime(
        Some(root.to_path_buf()),
        &PrimeArgs {
            reference: "RV-001".to_owned(),
        },
    )
    .unwrap();
    assert!(read_cache(root, 1).unwrap().is_some());
}

/// `status` on an unprimed review reports no cache line (the signal only fires
/// once the cache is primed, §9).
#[test]
fn status_is_silent_about_an_unprimed_cache() {
    let tmp = fixture_rv();
    let root = tmp.path();
    // No prime — read_cache is None, so status reports the ledger only.
    assert!(read_cache(root, 1).unwrap().is_none());
    run_status(Some(root.to_path_buf()), "RV-001").unwrap();
}

// ── PHASE-01: ReviewOutput + ReviewError types ──

#[test]
fn review_output_created_serialises_to_json() {
    let out = ReviewOutput::Created {
        id: 42,
        canonical: "RV-042".into(),
        dir: PathBuf::from(".doctrine/review/042"),
    };
    let json = serde_json::to_string(&out).unwrap();
    assert!(json.contains(r#""id":42"#), "json: {json}");
    assert!(json.contains(r#""canonical":"RV-042""#), "json: {json}");
    assert!(
        json.contains(r#""dir":".doctrine/review/042""#),
        "json: {json}"
    );
}

#[test]
fn review_error_downcasts_from_anyhow() {
    let err = ReviewError::RoleMismatch {
        expected: Role::Raiser,
        actual: Role::Responder,
        act: Act::Raise,
    };
    let anyhow_err: anyhow::Error = err.into();
    let downcast = anyhow_err
        .downcast_ref::<ReviewError>()
        .expect("ReviewError should downcast from anyhow");
    match downcast {
        ReviewError::RoleMismatch {
            expected,
            actual,
            act,
        } => {
            assert_eq!(*expected, Role::Raiser);
            assert_eq!(*actual, Role::Responder);
            assert_eq!(*act, Act::Raise);
        }
        _ => panic!("wrong variant: {downcast:?}"),
    }
}

#[test]
fn with_turn_accepts_non_unit_closure_return() {
    // Verify the generic parameter T != () compiles — this test is
    // compile-time; the behaviour is verified by the verb handler tests.
    // We just assert that a String-returning closure type-checks.
    let _: fn(&mut toml_edit::DocumentMut, &[FindingRow]) -> anyhow::Result<String> =
        |_, _| anyhow::Ok("F-1".into());
}

// ------------------------------------------------------------------
// PHASE-02 — golden tests: capture current stdout and assert
// print_review() reproduces it identically (VT-1..VT-10)
// ------------------------------------------------------------------

/// Golden: `print_review(&Unlocked)` with no lock produces "RV-001 is not locked".
#[test]
fn golden_print_unlocked_not_locked() {
    let out = ReviewOutput::Unlocked {
        canonical: "RV-001".into(),
        formatted: String::new(),
    };
    let rendered = print_review(&out);
    assert_eq!(rendered, "RV-001 is not locked\n");
}

/// Golden: `run_unlock` on a review that is not locked returns Unlocked
/// with empty formatted, and print_review renders correctly.
#[test]
fn golden_run_unlock_not_locked() {
    let tmp = fixture_rv();
    let root = tmp.path();
    let out = run_unlock(Some(root.to_path_buf()), "RV-001").unwrap();
    match &out {
        ReviewOutput::Unlocked {
            canonical,
            formatted,
        } => {
            assert_eq!(canonical, "RV-001");
            assert!(formatted.is_empty());
        }
        _ => panic!("expected Unlocked, got {out:?}"),
    }
    let rendered = print_review(&out);
    assert_eq!(rendered, "RV-001 is not locked\n");
}

/// Golden: `print_review(&Created)` produces "Created review 001: <dir>".
#[test]
fn golden_print_created() {
    let out = ReviewOutput::Created {
        id: 1,
        canonical: "RV-001".into(),
        dir: PathBuf::from(".doctrine/review/001"),
    };
    let rendered = print_review(&out);
    assert_eq!(rendered, "Created review 001: .doctrine/review/001\n");
}

/// Golden: `run_new` creates a review and returns Created with correct fields.
#[test]
fn golden_run_new() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    plant_slice_target(root, 42);
    let out = run_new(Some(root.to_path_buf()), &new_args(Facet::Design, "SL-042")).unwrap();
    match &out {
        ReviewOutput::Created { id, canonical, dir } => {
            assert_eq!(*id, 1);
            assert_eq!(canonical, "RV-001");
            assert!(dir.to_string_lossy().contains("001"));
        }
        _ => panic!("expected Created, got {out:?}"),
    }
    let rendered = print_review(&out);
    assert!(rendered.starts_with("Created review 001: "));
}

/// Golden: `print_review(&Raised)` produces "Raised F-1 on RV-001".
#[test]
fn golden_print_raised() {
    let out = ReviewOutput::Raised {
        finding_id: "F-1".into(),
        review_id: 1,
    };
    let rendered = print_review(&out);
    assert_eq!(rendered, "Raised F-1 on RV-001\n");
}

/// Golden: `run_raise` on a fresh RV returns Raised with the finding_id.
#[test]
fn golden_run_raise() {
    let tmp = fixture_rv();
    let root = tmp.path();
    let out = run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Major, "test finding"),
        Role::Raiser,
    )
    .unwrap();
    match &out {
        ReviewOutput::Raised {
            finding_id,
            review_id,
        } => {
            assert_eq!(finding_id, "F-1");
            assert_eq!(*review_id, 1);
        }
        _ => panic!("expected Raised, got {out:?}"),
    }
    let rendered = print_review(&out);
    assert_eq!(rendered, "Raised F-1 on RV-001\n");
}

/// Golden: `print_review(&Disposed)` produces "Disposed F-1 on RV-001 (answered)".
#[test]
fn golden_print_disposed() {
    let out = ReviewOutput::Disposed {
        finding_id: "F-1".into(),
        review_id: 1,
    };
    let rendered = print_review(&out);
    assert_eq!(rendered, "Disposed F-1 on RV-001 (answered)\n");
}

/// Golden: `run_dispose` on a raised finding returns Disposed.
#[test]
fn golden_run_dispose() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Major, "test"),
        Role::Raiser,
    )
    .unwrap();
    let out = run_dispose(
        Some(root.to_path_buf()),
        &dispose_args("RV-001", "F-1"),
        Role::Responder,
    )
    .unwrap();
    match &out {
        ReviewOutput::Disposed {
            finding_id,
            review_id,
        } => {
            assert_eq!(finding_id, "F-1");
            assert_eq!(*review_id, 1);
        }
        _ => panic!("expected Disposed, got {out:?}"),
    }
    let rendered = print_review(&out);
    assert_eq!(rendered, "Disposed F-1 on RV-001 (answered)\n");
}

/// Golden: `print_review(&Verified)`.
#[test]
fn golden_print_verified() {
    let out = ReviewOutput::Verified {
        finding_id: "F-1".into(),
        review_id: 1,
    };
    let rendered = print_review(&out);
    assert_eq!(rendered, "Verified F-1 on RV-001 (verified)\n");
}

/// Golden: `print_review(&Contested)`.
#[test]
fn golden_print_contested() {
    let out = ReviewOutput::Contested {
        finding_id: "F-1".into(),
        review_id: 1,
    };
    let rendered = print_review(&out);
    assert_eq!(rendered, "Contested F-1 on RV-001 (contested)\n");
}

/// Golden: `print_review(&Withdrawn)`.
#[test]
fn golden_print_withdrawn() {
    let out = ReviewOutput::Withdrawn {
        finding_id: "F-1".into(),
        review_id: 1,
    };
    let rendered = print_review(&out);
    assert_eq!(rendered, "Withdrew F-1 on RV-001 (withdrawn)\n");
}

/// Golden: `run_verify` end-to-end.
#[test]
fn golden_run_verify() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Major, "test"),
        Role::Raiser,
    )
    .unwrap();
    run_dispose(
        Some(root.to_path_buf()),
        &dispose_args("RV-001", "F-1"),
        Role::Responder,
    )
    .unwrap();
    let out = run_verify(
        Some(root.to_path_buf()),
        "RV-001",
        "F-1",
        None,
        Role::Raiser,
    )
    .unwrap();
    let rendered = print_review(&out);
    assert_eq!(rendered, "Verified F-1 on RV-001 (verified)\n");
}

/// Golden: `run_show` returns Showed with formatted output.
#[test]
fn golden_run_show() {
    let tmp = fixture_rv();
    let root = tmp.path();
    let out = run_show(Some(root.to_path_buf()), "RV-001", Format::Table).unwrap();
    let rendered = print_review(&out);
    // The show output is a multi-line table. Check key lines are present.
    assert!(rendered.contains("RV-001 — "), "show: {rendered}");
    assert!(rendered.contains("design · "), "show: {rendered}");
    assert!(rendered.contains("──reviews──▶"), "show: {rendered}");
}

/// Golden: `run_list` returns Listed with formatted table output.
#[test]
fn golden_run_list() {
    let tmp = fixture_rv();
    let root = tmp.path();
    let out = run_list(Some(root.to_path_buf()), list_args(), None).unwrap();
    match &out {
        ReviewOutput::Listed {
            rows, formatted, ..
        } => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].id, "RV-001");
            // The formatted table contains the review data.
            assert!(formatted.contains("RV-001"), "list: {formatted}");
        }
        _ => panic!("expected Listed, got {out:?}"),
    }
    let rendered = print_review(&out);
    assert!(rendered.contains("RV-001"), "list: {rendered}");
}

/// Golden: `run_status` returns Status with correct fields.
#[test]
fn golden_run_status() {
    let tmp = fixture_rv();
    let root = tmp.path();
    let out = run_status(Some(root.to_path_buf()), "RV-001").unwrap();
    let formatted = match &out {
        ReviewOutput::Status {
            canonical,
            status,
            cache_primed,
            formatted,
            ..
        } => {
            assert_eq!(canonical, "RV-001");
            assert_eq!(status, "done");
            assert!(!cache_primed);
            assert!(formatted.contains("RV-001 — "), "status: {formatted}");
            assert!(formatted.contains("done · "), "status: {formatted}");
            formatted.clone()
        }
        _ => panic!("expected Status, got {out:?}"),
    };
    let rendered = print_review(&out);
    assert_eq!(rendered, formatted);
}

/// Golden: `run_show` with JSON format returns Showed with JSON-formatted string.
#[test]
fn golden_run_show_json() {
    let tmp = fixture_rv();
    let root = tmp.path();
    let out = run_show(Some(root.to_path_buf()), "RV-001", Format::Json).unwrap();
    let rendered = print_review(&out);
    assert!(rendered.contains("\"kind\""), "show json: {rendered}");
    assert!(rendered.contains("\"review\""), "show json: {rendered}");
}

// -- IMP-490 (RFC-032 0c): the finding index + `list --target` -------------

/// A shared `ListArgs` for the review list tests (every axis default).
fn list_args() -> ListArgs {
    ListArgs {
        substr: None,
        regexp: None,
        case_insensitive: false,
        status: Vec::new(),
        tags: Vec::new(),
        all: false,
        format: Format::Table,
        json: false,
        columns: None,
        render: listing::RenderOpts {
            color: false,
            term_width: None,
        },
    }
}

/// IMP-490: `show` renders the finding index by default — one row per finding
/// with id/severity/status/disposition/title, not just a count. Before the
/// index, the tier was reachable only via `--json`.
#[test]
fn show_renders_the_finding_index() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Blocker, "Always render the index"),
        Role::Raiser,
    )
    .unwrap();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Nit, "A second finding"),
        Role::Raiser,
    )
    .unwrap();
    let out = run_show(Some(root.to_path_buf()), "RV-001", Format::Table).unwrap();
    let rendered = print_review(&out);
    // The index carries a column header...
    assert!(rendered.contains("severity"), "index header: {rendered}");
    assert!(rendered.contains("disposition"), "index header: {rendered}");
    // ...one line per finding with its tier columns...
    assert!(rendered.contains("F-1"), "index row: {rendered}");
    assert!(rendered.contains("blocker"), "index row: {rendered}");
    assert!(
        rendered.contains("Always render the index"),
        "index row: {rendered}"
    );
    assert!(rendered.contains("F-2"), "index row: {rendered}");
    assert!(rendered.contains("nit"), "index row: {rendered}");
    // ...and the existing count line survives alongside it.
    assert!(rendered.contains("findings: 2"), "count line: {rendered}");
}

/// IMP-490: the index carries the responder's disposition once set.
#[test]
fn show_index_carries_the_disposition() {
    let tmp = fixture_rv();
    let root = tmp.path();
    run_raise(
        Some(root.to_path_buf()),
        &raise_args("RV-001", Severity::Major, "needs a disposition"),
        Role::Raiser,
    )
    .unwrap();
    run_dispose(
        Some(root.to_path_buf()),
        &dispose_args("RV-001", "F-1"),
        Role::Responder,
    )
    .unwrap();
    let out = run_show(Some(root.to_path_buf()), "RV-001", Format::Table).unwrap();
    let rendered = print_review(&out);
    assert!(rendered.contains("fix-now"), "disposition: {rendered}");
    assert!(rendered.contains("answered"), "status: {rendered}");
}

/// IMP-490: an empty ledger renders no index table (the count line stands
/// alone), so a clean pass stays a clean pass.
#[test]
fn finding_index_is_absent_for_an_empty_ledger() {
    assert_eq!(render_finding_index(&[]), "");
}

/// IMP-490: `list --target` admits only reviews whose `reviews` edge targets
/// the given ref (phase scope ignored); no target lists every review.
#[test]
fn list_target_filters_to_the_subject_edge() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    plant_slice_target(root, 1);
    plant_slice_target(root, 2);
    run_new(Some(root.to_path_buf()), &new_args(Facet::Design, "SL-001")).unwrap();
    run_new(Some(root.to_path_buf()), &new_args(Facet::Plan, "SL-002")).unwrap();

    let listed =
        |target: Option<&str>| match run_list(Some(root.to_path_buf()), list_args(), target)
            .unwrap()
        {
            ReviewOutput::Listed { rows, .. } => rows,
            other => panic!("expected Listed, got {other:?}"),
        };

    assert_eq!(listed(None).len(), 2, "no target lists both reviews");
    let only_first = listed(Some("SL-001"));
    assert_eq!(only_first.len(), 1, "SL-001 admits one: {only_first:?}");
    assert_eq!(only_first[0].target, "SL-001");
    assert!(
        listed(Some("SL-404")).is_empty(),
        "an unmatched target lists none"
    );
}
