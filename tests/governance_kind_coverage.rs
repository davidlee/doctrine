// SPDX-License-Identifier: GPL-3.0-only
//! SL-249 PHASE-07 — the governance kind-coverage canary (`VT-1`, `VT-2`).
//!
//! `SPEC-019` and `PRD-010` said the knowledge corpus had **four** record kinds.
//! It has had seven since SL-159 (`EVD`, `HYP`) and SL-197 (`CPT`); neither slice
//! landed the governance axis it scoped. `REV-050` amended both entities. This
//! file is what stops that gap reopening — a prose claim about a set has no
//! compiler, so it gets a test instead (`DEC-176`).
//!
//! It is a **project-local** test in `tests/`, deliberately not a `validate` rule:
//! the four tier files it reads are this repo's own governance corpus, and a rule
//! in `src/` would make every installed project carry doctrine's housekeeping
//! (`POL-002`, `EX-8`).
//!
//! ## The two assertions
//!
//! * **Coverage** — every kind in `kinds::RECORD` appears in **paired form** —
//!   `assumption (ASM)` — in all four authored tiers: `spec-019.toml`,
//!   `spec-019.md`, `spec-010.toml`, `spec-010.md`. Seven kinds × four tiers = 28.
//!   Paired, strictly adjacent, because the pre-amendment `spec-019.toml` listed
//!   the long names and the prefixes as two *separate* lists — a co-presence check
//!   scores that 4/7 when its real score is 0/7, and would ship a canary blind to
//!   the exact defect it exists to catch (`RV-349` `F-5`).
//! * **The identity** — per tier, the number of `four`s equals the number the
//!   allowlist accounts for, *and* each allowlist entry's phrase occurs exactly as
//!   often as it claims. Two halves, reported separately: a duplicated exempt
//!   phrase fails the second while the first can still balance (`F-11`).
//!
//! Presence alone was tried and rejected across six review rounds: it goes green
//! the moment someone writes the word, and says nothing about the 31 stale sites
//! left behind. The identity is an arithmetic statement a reader can evaluate
//! rather than a totality claim a reader must trust (`R4`).
//!
//! ## Why the whitespace collapse is not a nicety
//!
//! `spec-019.md` wraps its one legitimate `four` mid-phrase: a line ends
//! `…touches four` and the next begins `coupled sites — …`. On raw bytes the
//! exempt phrase matches **zero** times, so the identity reads `32 == 0` on the
//! day it is authored and the only green path is rewording an accurate sentence
//! to satisfy a test (`F-9`, `F-14`). Collapse first, always. A fixture asserts
//! both directions so a future "simplification" that drops it fails loudly.
//!
//! ## Why the kind list is parsed, not imported
//!
//! `kinds::RECORD` is `pub(crate)` and `src/lib.rs` exports nothing from `mod
//! kinds` — the export list is the whole crate boundary and
//! `tests/architecture_layering.rs` asserts it. Widening it to satisfy a test
//! would breach that (`D-1`). The canary parses `src/kinds/mod.rs` instead, which
//! is the `tests/arg_path_convention.rs` precedent — and is required regardless,
//! since the *pairing* of long name to prefix lives in the const identifiers
//! (`ASSUMPTION_KIND` ↔ `ASM`) and is not recoverable from `RECORD` alone.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

mod common;

use regex::Regex;

// ---------------------------------------------------------------------------
// The checker — one pure function over borrowed text, so the live arm and every
// fixture arm are literally the same code.
// ---------------------------------------------------------------------------

/// One accepted occurrence-set of `four` in one tier. `why` is not decoration:
/// `EX-5` requires the argument for an exemption to live in the test, and it is
/// emitted in the failure message so whoever trips it reads the reasoning rather
/// than deleting the entry.
struct Exempt {
    tier: &'static str,
    phrase: &'static str,
    expect: usize,
    why: &'static str,
}

/// One authored tier under test — a name for failure messages and its bytes.
struct Tier<'a> {
    name: &'a str,
    text: &'a str,
}

/// Runs of whitespace to a single space. Applied before **every** phrase match:
/// a governance claim that survives a line wrap must survive it here too.
fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Occurrences of the word `four`, case-folded (`### Four kinds, one engine` was
/// the most prominent stale site in the corpus).
fn count_four(text: &str) -> usize {
    Regex::new(r"(?i)\bfour\b")
        .expect("static regex")
        .find_iter(text)
        .count()
}

/// Non-overlapping, case-folded occurrences of a phrase.
fn count_phrase(text: &str, phrase: &str) -> usize {
    text.to_lowercase().matches(&phrase.to_lowercase()).count()
}

/// Long name then prefix, strictly adjacent — `assumption (ASM)`. The long name
/// is case-folded (it starts sentences); the prefix is exact-case (it is an
/// identifier). Backticks are optional because the corpus writes ``concept
/// (`CPT`)`` and the identity must not turn on markup.
fn paired_form(text: &str, long: &str, prefix: &str) -> bool {
    Regex::new(&format!(
        r"(?i:\b{}\b)\s*\(\s*`?{}`?\s*\)",
        regex::escape(long),
        regex::escape(prefix)
    ))
    .expect("kind regex")
    .is_match(text)
}

/// Returns the failure list — empty means pass. It *returns* rather than asserts
/// so a fixture can pin **which** failures and **how many**, which is the only
/// thing that makes the pre-amendment control a control.
fn check(tiers: &[Tier<'_>], kinds: &[(String, String)], allow: &[Exempt]) -> Vec<String> {
    let mut out = Vec::new();
    for tier in tiers {
        // Collapse once, up front — every match below is against collapsed text.
        let text = collapse(tier.text);

        for (long, prefix) in kinds {
            if !paired_form(&text, long, prefix) {
                out.push(format!(
                    "{}: missing paired form — {long} ({prefix})",
                    tier.name
                ));
            }
        }

        // The identity, per tier (`D-2`): aggregating across tiers would let a
        // `four` migrate from one entity into the other and pass silently.
        let entries: Vec<&Exempt> = allow.iter().filter(|e| e.tier == tier.name).collect();
        let allowed: usize = entries.iter().map(|e| e.expect).sum();
        let actual = count_four(&text);
        if actual != allowed {
            out.push(format!(
                "{}: 'four' appears {actual} time(s) but the allowlist accounts for {allowed}",
                tier.name
            ));
        }
        // The second half, reported separately: the total can balance while an
        // individual entry is wrong (`F-11`).
        for e in entries {
            let seen = count_phrase(&text, e.phrase);
            if seen != e.expect {
                out.push(format!(
                    "{}: exempt phrase \"{}\" appears {seen} time(s), allowlist expects {} — {}",
                    tier.name, e.phrase, e.expect, e.why
                ));
            }
        }
    }

    // An entry naming a tier nobody read is an exemption with no subject — it
    // silently widens the allowance for whatever tier it is later matched to.
    let names: Vec<&str> = tiers.iter().map(|t| t.name).collect();
    for e in allow {
        if !names.contains(&e.tier) {
            out.push(format!(
                "allowlist entry names a tier not under test: {}",
                e.tier
            ));
        }
    }

    out
}

// ---------------------------------------------------------------------------
// The allowlist — one entry, and the argument for it is the entry.
// ---------------------------------------------------------------------------

const ALLOW: &[Exempt] = &[Exempt {
    tier: "spec-019.md",
    phrase: "four coupled sites",
    expect: 1,
    why: "Counts INTEGRATION SITES, not record kinds: integrity::KINDS, \
          RELATION_RULES, the outbound_for dispatch, and the exact-coverage \
          invariant test. Independent of the kind count; stays correct at seven. \
          A blanket ban would force the REV to reword an accurate sentence to \
          make a test green (RV-349 F-9).",
}];

/// The record kinds **as of SL-249**, frozen. The fixture arms are a historical
/// snapshot of a corpus that no longer exists, so their expected failure counts
/// are historical facts and must not drift when an eighth kind lands — the live
/// arm is what tracks `kinds::RECORD`. `frozen_kinds_are_still_real_kinds` (T7)
/// keeps the two from diverging in the direction that matters: a rename or a
/// removal here is caught, an addition is not, because an addition does not
/// change what the pre-amendment corpus said.
fn frozen_record_kinds() -> Vec<(String, String)> {
    [
        ("assumption", "ASM"),
        ("decision", "DEC"),
        ("question", "QUE"),
        ("constraint", "CON"),
        ("evidence", "EVD"),
        ("hypothesis", "HYP"),
        ("concept", "CPT"),
    ]
    .into_iter()
    .map(|(l, p)| (l.to_string(), p.to_string()))
    .collect()
}

// ---------------------------------------------------------------------------
// Fixture 1 — the compensating control: the real pre-amendment bytes.
// ---------------------------------------------------------------------------

/// The four frozen tiers, recovered from `59f77ce10` (see the fixture README).
fn pre_amendment() -> Vec<(String, String)> {
    let dir = common::repo_root().join("tests/fixtures/governance_kind_coverage/pre_amendment");
    [
        "spec-019.toml",
        "spec-019.md",
        "spec-010.toml",
        "spec-010.md",
    ]
    .into_iter()
    .map(|name| {
        // The two prose tiers are ~60 KB combined, so they are line-accurate
        // excerpts rather than copies; `…_excerpts_are_complete` proves the
        // excerpt is not short.
        let file = if name.ends_with(".md") {
            format!("{name}.excerpt")
        } else {
            name.to_string()
        };
        let path = dir.join(&file);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()));
        (name.to_string(), text)
    })
    .collect()
}

/// A short excerpt would make the control a lie: it would under-report the very
/// staleness the control exists to demonstrate. So the excerpt's completeness is
/// asserted, not trusted.
#[test]
fn pre_amendment_excerpts_are_complete() {
    let tiers = pre_amendment();
    let by = |n: &str| -> String {
        tiers
            .iter()
            .find(|(name, _)| name == n)
            .map(|(_, t)| t.clone())
            .expect("tier present")
    };

    assert_eq!(
        count_four(&collapse(&by("spec-019.md"))),
        25,
        "spec-019.md excerpt must carry all 25 pre-amendment `four` occurrences"
    );
    assert_eq!(
        count_four(&collapse(&by("spec-010.md"))),
        4,
        "spec-010.md excerpt must carry all 4 pre-amendment `four` occurrences"
    );
    assert_eq!(count_four(&collapse(&by("spec-019.toml"))), 3);
    assert_eq!(count_four(&collapse(&by("spec-010.toml"))), 0);

    // `\bfour\b` is collapse-invariant — 32 either way. `D9` says that observation
    // is itself part of the point: the collapse is needed for the *phrase*, and
    // costs the word count nothing.
    let total_collapsed: usize = tiers.iter().map(|(_, t)| count_four(&collapse(t))).sum();
    let total_raw: usize = tiers.iter().map(|(_, t)| count_four(t)).sum();
    assert_eq!((total_collapsed, total_raw), (32, 32));

    // The wrap, preserved verbatim: present once collapsed, absent raw.
    let prose = by("spec-019.md");
    assert_eq!(count_phrase(&collapse(&prose), "four coupled sites"), 1);
    assert_eq!(count_phrase(&prose, "four coupled sites"), 0);
}

/// **The test that would have gone red, frozen in the tree.** The same `check`
/// over the real pre-amendment bytes, asserting the *exact* failure set the live
/// arm would have reported the day before `REV-050`.
#[test]
fn pre_amendment_corpus_fails_with_the_exact_expected_findings() {
    let owned = pre_amendment();
    let tiers: Vec<Tier<'_>> = owned
        .iter()
        .map(|(n, t)| Tier { name: n, text: t })
        .collect();

    let failures = check(&tiers, &frozen_record_kinds(), ALLOW);

    let missing_paired: Vec<&String> = failures
        .iter()
        .filter(|f| f.contains("missing paired form"))
        .collect();
    // 28 assertions, 8 satisfied: the four original kinds in the two *prose*
    // tiers only. Neither TOML tier carried a single paired form.
    assert_eq!(
        missing_paired.len(),
        20,
        "expected 28 - 8 = 20 missing paired forms, got: {missing_paired:#?}"
    );
    for (tier, n) in [
        ("spec-019.toml", 7),
        ("spec-010.toml", 7),
        ("spec-019.md", 3),
        ("spec-010.md", 3),
    ] {
        assert_eq!(
            missing_paired
                .iter()
                .filter(|f| f.starts_with(&format!("{tier}:")))
                .count(),
            n,
            "{tier} missing-paired-form count"
        );
    }
    // The three absent kinds are absent from *every* tier, TOML and prose alike.
    for prefix in ["EVD", "HYP", "CPT"] {
        assert_eq!(
            missing_paired.iter().filter(|f| f.contains(prefix)).count(),
            4,
            "{prefix} should be missing from all four tiers"
        );
    }

    // Three identity failures: 3-vs-0, 25-vs-1, 4-vs-0. `spec-010.toml` (0-vs-0)
    // passes — a tier with no `four` at all is already correct on that half, and
    // it is the paired-form half that catches it.
    let identity: Vec<&String> = failures
        .iter()
        .filter(|f| f.contains("but the allowlist accounts for"))
        .collect();
    assert_eq!(identity.len(), 3, "identity failures: {identity:#?}");
    for want in [
        "spec-019.toml: 'four' appears 3 time(s) but the allowlist accounts for 0",
        "spec-019.md: 'four' appears 25 time(s) but the allowlist accounts for 1",
        "spec-010.md: 'four' appears 4 time(s) but the allowlist accounts for 0",
    ] {
        assert!(
            identity.iter().any(|f| f.as_str() == want),
            "expected identity failure {want:?} in {identity:#?}"
        );
    }

    // The per-entry half *passes* here — the exempt phrase occurs exactly once,
    // as it claims. Only the total is wrong. That is the split `F-11` bought.
    assert!(
        !failures.iter().any(|f| f.contains("exempt phrase")),
        "the exempt phrase itself was correct pre-amendment: {failures:#?}"
    );

    assert_eq!(failures.len(), 23, "total failures: {failures:#?}");
}

// ---------------------------------------------------------------------------
// Fixtures 2–5 — synthetic, and dependent on nothing outside this file. Each
// passes an empty kind list: the paired-form half is fixture 1's job, and these
// exist to prove the *identity* halves see what they claim to see.
// ---------------------------------------------------------------------------

fn identity_only(name: &'static str, text: &'static str, allow: &[Exempt]) -> Vec<String> {
    check(&[Tier { name, text }], &[], allow)
}

const PHRASE_A: &str = "four coupled sites";
const PHRASE_B: &str = "four wire keys";

fn entry(tier: &'static str, phrase: &'static str, expect: usize) -> Exempt {
    Exempt {
        tier,
        phrase,
        expect,
        why: "fixture",
    }
}

/// Fixture 2 (`F-11`) — a duplicated exempt phrase. Two arms, and the second is
/// the one that pays for the per-entry half: the totals **balance** and the
/// duplicate is still caught.
#[test]
fn duplicated_exempt_phrase_is_caught_even_when_the_total_balances() {
    // Arm 1 — one entry, phrase twice. Both halves fire.
    let f = identity_only(
        "t",
        "four coupled sites and four coupled sites",
        &[entry("t", PHRASE_A, 1)],
    );
    assert_eq!(f.len(), 2, "{f:#?}");
    assert!(
        f.iter()
            .any(|m| m.contains("appears 2 time(s) but the allowlist accounts for 1"))
    );
    assert!(
        f.iter()
            .any(|m| m.contains("exempt phrase") && m.contains("appears 2"))
    );

    // Arm 2 — two entries, one expected each, so the allowlist accounts for 2.
    // The tier has phrase A twice and phrase B never: the **total balances**
    // (2 == 2) and an aggregate-only check would pass it silently.
    let f = identity_only(
        "t",
        "four coupled sites, and again four coupled sites",
        &[entry("t", PHRASE_A, 1), entry("t", PHRASE_B, 1)],
    );
    assert!(
        !f.iter()
            .any(|m| m.contains("but the allowlist accounts for")),
        "the total was supposed to balance: {f:#?}"
    );
    assert_eq!(f.len(), 2, "both per-entry counts are wrong: {f:#?}");
    assert!(
        f.iter()
            .any(|m| m.contains(PHRASE_A) && m.contains("appears 2"))
    );
    assert!(
        f.iter()
            .any(|m| m.contains(PHRASE_B) && m.contains("appears 0"))
    );
}

/// Fixture 3 — a new, non-exempt `four` slips in beside a legitimate one. The
/// per-entry half is happy; the total is not. This is the everyday regression:
/// someone writes "the four record kinds" again.
#[test]
fn a_new_non_exempt_four_fails_the_total() {
    let f = identity_only(
        "t",
        "four coupled sites, and the four record kinds",
        &[entry("t", PHRASE_A, 1)],
    );
    assert_eq!(f.len(), 1, "{f:#?}");
    assert_eq!(
        f[0],
        "t: 'four' appears 2 time(s) but the allowlist accounts for 1"
    );
}

/// Fixture 4 — the exempt sentence is deleted or reworded. A stale allowlist
/// entry must expire **loudly**: an exemption that outlives its subject is how an
/// allowlist rots into a blanket permission.
#[test]
fn a_stale_allowlist_entry_expires_loudly() {
    let f = identity_only("t", "nothing to see here", &[entry("t", PHRASE_A, 1)]);
    assert_eq!(f.len(), 2, "{f:#?}");
    assert!(
        f.iter()
            .any(|m| m.contains("appears 0 time(s) but the allowlist accounts for 1"))
    );
    assert!(
        f.iter()
            .any(|m| m.contains("exempt phrase") && m.contains("appears 0"))
    );
}

/// Fixture 5 (`F-14`) — the collapse, proven load-bearing rather than merely
/// present. The same bytes pass **with** it and fail **without** it, so a future
/// simplification that drops it cannot go green.
#[test]
fn the_collapse_is_load_bearing() {
    const WRAPPED: &str = "Admitting the record kinds touches four\n  coupled sites \
                           — integrity::KINDS, RELATION_RULES, the outbound_for \
                           dispatch, and the exact-coverage invariant test.\n";

    // Without the collapse the phrase is invisible…
    assert_eq!(count_phrase(WRAPPED, PHRASE_A), 0);
    // …and with it, it is exactly where the author put it.
    assert_eq!(count_phrase(&collapse(WRAPPED), PHRASE_A), 1);
    // The word itself is collapse-invariant; only the phrase turns on it.
    assert_eq!(count_four(WRAPPED), count_four(&collapse(WRAPPED)));

    // End to end: `check` collapses, so the wrapped sentence is accepted.
    let f = identity_only("t", WRAPPED, &[entry("t", PHRASE_A, 1)]);
    assert!(
        f.is_empty(),
        "wrapped exempt phrase must be accepted: {f:#?}"
    );

    // And the counterfactual, spelled out: a checker that skipped the collapse
    // would see 1 `four` against 0 matched phrases — the `32 == 0` failure that
    // makes rewording an accurate sentence the only green path (`F-9`).
    assert_ne!(
        count_four(WRAPPED),
        count_phrase(WRAPPED, PHRASE_A),
        "without the collapse the identity cannot be satisfied at all"
    );
}

// ---------------------------------------------------------------------------
// The live arm (`VT-1`) — the four authored tiers, against `kinds::RECORD`.
// ---------------------------------------------------------------------------

/// The four authored tier files, **path-listed**. Deliberately not a directory
/// walk: `.doctrine/spec/tech/019/` also holds `handover.md`, `interactions.toml`
/// and `members.toml`, none of which belong to either entity — and `handover.md`
/// is gitignored scratch that still carries a stale four-kind claim by design.
/// A glob here would read files the REV never amended and could not amend
/// (`D9` scope).
const LIVE_TIERS: &[(&str, &str)] = &[
    ("spec-019.toml", ".doctrine/spec/tech/019/spec-019.toml"),
    ("spec-019.md", ".doctrine/spec/tech/019/spec-019.md"),
    ("spec-010.toml", ".doctrine/spec/product/010/spec-010.toml"),
    ("spec-010.md", ".doctrine/spec/product/010/spec-010.md"),
];

fn live_tiers() -> Vec<(String, String)> {
    let root = common::repo_root();
    LIVE_TIERS
        .iter()
        .map(|(name, rel)| {
            let path = root.join(rel);
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read authored tier {}: {e}", path.display()));
            ((*name).to_string(), text)
        })
        .collect()
}

/// Every `pub(crate) const <NAME>: &str = "<value>";` in the file, so a prefix is
/// read as its **value** rather than as its identifier. They coincide today
/// (`ASM` = `"ASM"`); a canary that assumed so would silently check the wrong
/// token the day they stop.
fn str_consts(ast: &syn::File) -> std::collections::BTreeMap<String, String> {
    ast.items
        .iter()
        .filter_map(|item| {
            let syn::Item::Const(c) = item else {
                return None;
            };
            let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = &*c.expr
            else {
                return None;
            };
            Some((c.ident.to_string(), s.value()))
        })
        .collect()
}

/// The record kinds, read out of `src/kinds/mod.rs` — the totality argument.
///
/// Two independent parses, and they must agree:
///   * the `stem: "record"` `Kind` consts, which are the only place the long name
///     is paired with the prefix (`ASSUMPTION_KIND` ↔ `prefix: ASM`);
///   * the `RECORD` slice, which is the declared membership list.
///
/// A new record kind joins all 28 assertions with **no edit to this file**. That
/// is what makes this a coverage assertion rather than a spot check — and it is
/// why the disagreement below panics instead of silently preferring one parse.
fn record_kinds_from_source() -> Vec<(String, String)> {
    let path = common::repo_root().join("src/kinds/mod.rs");
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let ast = syn::parse_file(&text).expect("parse src/kinds/mod.rs");
    let consts = str_consts(&ast);
    let resolve = |ident: &str| -> String {
        consts
            .get(ident)
            .unwrap_or_else(|| panic!("kind prefix `{ident}` is not a &str const in kinds/mod.rs"))
            .clone()
    };

    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut declared: Option<Vec<String>> = None;

    for item in &ast.items {
        let syn::Item::Const(c) = item else {
            continue;
        };

        // `pub(crate) const RECORD: &[&str] = &[ASM, DEC, …];`
        if c.ident == "RECORD" {
            let syn::Expr::Reference(r) = &*c.expr else {
                panic!("RECORD is not a slice reference");
            };
            let syn::Expr::Array(arr) = &*r.expr else {
                panic!("RECORD is not an array literal");
            };
            declared = Some(
                arr.elems
                    .iter()
                    .map(|e| {
                        let syn::Expr::Path(p) = e else {
                            panic!("RECORD element is not a path");
                        };
                        resolve(&p.path.require_ident().expect("RECORD ident").to_string())
                    })
                    .collect(),
            );
            continue;
        }

        // `pub(crate) const ASSUMPTION_KIND: Kind = Kind { …, prefix: ASM, stem: "record" };`
        let Some(stem_name) = c.ident.to_string().strip_suffix("_KIND").map(str::to_owned) else {
            continue;
        };
        let syn::Expr::Struct(lit) = &*c.expr else {
            continue;
        };
        let mut prefix: Option<String> = None;
        let mut stem: Option<String> = None;
        for field in &lit.fields {
            let syn::Member::Named(id) = &field.member else {
                continue;
            };
            match id.to_string().as_str() {
                "prefix" => {
                    if let syn::Expr::Path(p) = &field.expr
                        && let Some(id) = p.path.get_ident()
                    {
                        prefix = Some(resolve(&id.to_string()));
                    }
                }
                "stem" => {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &field.expr
                    {
                        stem = Some(s.value());
                    }
                }
                _ => {}
            }
        }
        if stem.as_deref() == Some("record")
            && let Some(prefix) = prefix
        {
            pairs.push((stem_name.to_lowercase().replace('_', " "), prefix));
        }
    }

    let declared = declared.expect("no `RECORD` const in src/kinds/mod.rs");
    let from_consts: std::collections::BTreeSet<&String> = pairs.iter().map(|(_, p)| p).collect();
    let from_record: std::collections::BTreeSet<&String> = declared.iter().collect();
    assert_eq!(
        from_consts, from_record,
        "the two parses of the record-kind set disagree: `stem: \"record\"` consts \
         vs the `RECORD` slice. One of them was edited without the other, and the \
         canary cannot tell which is right."
    );
    assert!(
        !pairs.is_empty(),
        "no record kinds parsed out of kinds/mod.rs"
    );
    pairs
}

/// The fixture arms freeze a kind list because they describe a corpus that no
/// longer exists. This is the one direction that still has to track reality: a
/// kind **renamed or removed** invalidates the control's arithmetic, so it fails
/// here. A kind *added* deliberately does not — it changes nothing about what
/// the pre-amendment corpus said.
#[test]
fn frozen_kinds_are_still_real_kinds() {
    let live = record_kinds_from_source();
    for pair in frozen_record_kinds() {
        assert!(
            live.contains(&pair),
            "frozen fixture kind {pair:?} is no longer a record kind — the \
             pre_amendment control's expected counts need re-deriving, not \
             deleting: {live:?}"
        );
    }
}

/// `VT-1`. Every kind in `kinds::RECORD`, in paired form, in all four authored
/// tiers — and every `four` in those tiers accounted for.
///
/// **This arm cannot stage a red.** It was written after `REV-050` made the
/// corpus correct, so it passed on its first run and proves nothing on its own.
/// Its controls are real tests in the tree, not a claim: the `pre_amendment`
/// fixture above (the same `check`, over the real stale bytes, asserting the
/// exact failure set) and `the_live_arm_notices_a_wrong_allowlist` below (this
/// arm's own path, perturbed).
#[test]
fn every_record_kind_is_enumerated_in_both_governance_entities() {
    let owned = live_tiers();
    let tiers: Vec<Tier<'_>> = owned
        .iter()
        .map(|(n, t)| Tier { name: n, text: t })
        .collect();
    let kinds = record_kinds_from_source();

    let failures = check(&tiers, &kinds, ALLOW);
    assert!(
        failures.is_empty(),
        "SPEC-019 / PRD-010 no longer enumerate the record kinds the corpus has \
         ({} kinds × {} tiers). Amend the entities — or, if a `four` is genuinely \
         about something other than the kind count, add an allowlist entry with \
         the argument for it:\n{failures:#?}",
        kinds.len(),
        tiers.len()
    );
}

/// The live arm's own positive control: perturb the allowlist against the **live**
/// tiers and watch the identity fire. Without this, "no failures" over a corpus
/// that happens to be correct is indistinguishable from a checker that looks at
/// nothing.
#[test]
fn the_live_arm_notices_a_wrong_allowlist() {
    let owned = live_tiers();
    let tiers: Vec<Tier<'_>> = owned
        .iter()
        .map(|(n, t)| Tier { name: n, text: t })
        .collect();

    let perturbed = &[Exempt {
        expect: ALLOW[0].expect + 1,
        ..ALLOW[0]
    }];
    let failures = check(&tiers, &record_kinds_from_source(), perturbed);

    assert!(
        failures
            .iter()
            .any(|f| f == "spec-019.md: 'four' appears 1 time(s) but the allowlist accounts for 2"),
        "the identity is vacuous over the live tiers: {failures:#?}"
    );
    assert!(
        failures
            .iter()
            .any(|f| f.contains("exempt phrase") && f.contains("allowlist expects 2")),
        "the per-entry half is vacuous over the live tiers: {failures:#?}"
    );
    // Exactly those two: all 28 paired forms are present, and the other three
    // tiers carry no `four` and no exemption. The perturbation is the only defect.
    assert_eq!(failures.len(), 2, "{failures:#?}");
}
