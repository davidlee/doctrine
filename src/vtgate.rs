// SPDX-License-Identifier: GPL-3.0-only
//! `vtgate` — the pure VT existence/shape gate core (ADR-001 leaf, SL-170 S3 /
//! PHASE-03). Given a parsed [`Plan`] (the PHASE-01 lifted VT model) and an
//! injected `read_file` reader, it judges every **VT-mode** verification
//! criterion against its structured mandate (`test_file` / `keywords` /
//! `patterns`) and returns one of four verdicts per row:
//!
//! - `Pass` — the mandated file exists and every keyword / pattern is present;
//! - `Fail` — the file is missing, or a mandated keyword / pattern is absent
//!   from the source (the gate halts on `Fail` only, INV-4);
//! - `Uncheckable` — no structured mandate to check (`test_file` is `None`, A1);
//! - `Waived` — a human-authorized escape valve (`waived = true`), reason shown.
//!
//! **Threat model is worker OMISSION**, not an adversary (design §5.2): a weak
//! worker skipping mandated work — the SL-169 ship-as-incomplete failure mode.
//! Plain substring over the RAW file is the proportionate floor; `patterns`
//! (line-anchored regex) is the optional stronger-shape escalation. Semantic
//! correctness of the assertion is a non-goal. An adversarial worker planting
//! bait (a keyword hidden in a comment / dead string) is out of scope — the
//! dispatch trust model fails upstream of this gate (ADR-012).
//!
//! **No comment / string stripping** (POL-002): comment and string-literal
//! syntax is host-LANGUAGE convention (`//` in Rust, `#` in Python, …). A gate
//! that strips them would load-bear correctness on the host's language — barred.
//! So matching is over raw bytes; a keyword present only in a comment satisfies
//! (an accepted, documented weakness — it still catches genuine omission, where
//! the keyword is absent entirely). An author wanting a code-shape assertion
//! uses `patterns`, which is itself language-agnostic (a regex the author owns).
//!
//! Pure: std + `regex` only. The fs read (the mandated files are project-root
//! relative), the `plan.toml` load, and the process exit code all live in the
//! impure shell (`crate::slice::run_verify_vt`); this module receives a
//! `read_file: &impl Fn(&str) -> Option<String>` and emits verdicts / a rendered
//! `String`. The gate reads only authored mandate + landed source — never the
//! disposable phase sheet (INV-3).

use regex::Regex;

use crate::plan::{Plan, VerificationCriterion};

/// The verdict for one VT criterion. `Fail` halts the gate; `Uncheckable` and
/// `Waived` are visible, distinct, and NON-halting (INV-4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum VtVerdict {
    /// The mandated file exists and every keyword / pattern is present.
    Pass,
    /// The file is missing, or a mandated keyword / pattern is absent. Halts.
    Fail { reason: String },
    /// No structured mandate (`test_file` is `None`) — nothing to grep (A1).
    Uncheckable,
    /// Human-authorized escape valve (`waived = true`); `reason` is surfaced.
    Waived { reason: String },
}

/// One judged VT row: its immutable id plus the verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VtLine {
    pub id: String,
    pub verdict: VtVerdict,
}

/// The VT verdicts for one phase. `lines` carries ONLY VT-mode rows — VA/VH
/// criteria are parsed but never gated (design §5.5 edge), so a phase with only
/// VA/VH criteria yields an empty `lines`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PhaseVtReport {
    pub phase_id: String,
    pub lines: Vec<VtLine>,
}

/// Fallback shown when a waiver / fail carries no recorded reason.
const NO_REASON: &str = "(no reason recorded)";

/// Judge ONE verification criterion against its structured mandate. ORDER is
/// load-bearing (EX-2):
///
/// 1. `waived` short-circuits FIRST — return `Waived` before any fs read;
/// 2. no `test_file` ⇒ `Uncheckable` (A1 — nothing to grep);
/// 3. the file does not read ⇒ `Fail` (missing);
/// 4. a `keyword` absent from the raw source, or a `pattern` that matches no
///    source line ⇒ `Fail`;
/// 5. otherwise `Pass` (a `test_file` with no keywords/patterns is an
///    existence-only mandate — `Pass` once the file reads).
pub(crate) fn check_vt(
    vt: &VerificationCriterion,
    read_file: &impl Fn(&str) -> Option<String>,
) -> VtVerdict {
    // (1) waiver short-circuits before touching the filesystem.
    if vt.waived {
        let reason = vt
            .waived_reason
            .clone()
            .unwrap_or_else(|| NO_REASON.to_string());
        return VtVerdict::Waived { reason };
    }
    // (2) no structured mandate ⇒ nothing to check.
    let Some(path) = vt.test_file.as_deref() else {
        return VtVerdict::Uncheckable;
    };
    // (3) the mandated file must exist / read.
    let Some(source) = read_file(path) else {
        return VtVerdict::Fail {
            reason: format!("mandated test_file `{path}` not found"),
        };
    };
    // (4) match keywords / patterns over the RAW source — no comment / string
    // stripping (POL-002: that is host-language convention; see module doc).
    for kw in &vt.keywords {
        if !source.contains(kw.as_str()) {
            return VtVerdict::Fail {
                reason: format!("keyword `{kw}` absent from `{path}`"),
            };
        }
    }
    for pat in &vt.patterns {
        match Regex::new(pat) {
            Ok(re) => {
                if !source.lines().any(|line| re.is_match(line)) {
                    return VtVerdict::Fail {
                        reason: format!("pattern `{pat}` matched no line in `{path}`"),
                    };
                }
            }
            Err(_) => {
                return VtVerdict::Fail {
                    reason: format!("pattern `{pat}` is not a valid regex"),
                };
            }
        }
    }
    // (5) file present and every mandate satisfied.
    VtVerdict::Pass
}

/// Judge every phase's VT criteria. Emits a line ONLY for VT-mode rows (id
/// prefix `VT-`); VA/VH rows are parsed but never gated (design §5.5). A phase
/// with no VT rows yields an empty `lines`; an empty plan yields no reports.
pub(crate) fn check_phases(
    plan: &Plan,
    read_file: &impl Fn(&str) -> Option<String>,
) -> Vec<PhaseVtReport> {
    plan.phases
        .iter()
        .map(|ph| PhaseVtReport {
            phase_id: ph.id.clone(),
            lines: ph
                .verification
                .iter()
                .filter(|vt| is_vt_mode(&vt.id))
                .map(|vt| VtLine {
                    id: vt.id.clone(),
                    verdict: check_vt(vt, read_file),
                })
                .collect(),
        })
        .collect()
}

/// Does any judged row halt the gate? `true` iff any `Fail` is present (INV-4).
/// `Uncheckable` / `Waived` are non-halting.
pub(crate) fn has_failure(reports: &[PhaseVtReport]) -> bool {
    reports
        .iter()
        .flat_map(|r| r.lines.iter())
        .any(|l| matches!(l.verdict, VtVerdict::Fail { .. }))
}

/// VT mode is encoded in the id prefix (`VT-1`) — there is no mode field. Only
/// `VT` rows are gated; `VA` (agent) / `VH` (human) are parsed, never gated.
fn is_vt_mode(id: &str) -> bool {
    id.starts_with("VT-") || id == "VT"
}

// ---------------------------------------------------------------------------
// Render (the human read-surface; S6 PHASE-04 embeds it at conclude/handover)
// ---------------------------------------------------------------------------

/// Verdict glyphs / labels — single-source named constants (STD-001). `WAIVED`
/// and `UNCHECKABLE` render distinctly from `PASS` / `FAIL`.
const GLYPH_PASS: &str = "✓";
const GLYPH_FAIL: &str = "✗";
const GLYPH_UNCHECKABLE: &str = "?";
const GLYPH_WAIVED: &str = "~";
const LABEL_PASS: &str = "PASS";
const LABEL_FAIL: &str = "FAIL";
const LABEL_UNCHECKABLE: &str = "UNCHECKABLE";
const LABEL_WAIVED: &str = "WAIVED";

/// Render the per-phase VT verdicts as a human-readable summary block. House
/// `Vec<String>` + `join` style (string-build clippy denies — no
/// `push_str(&format!)`). A plan with no VT rows renders an explicit empty note.
pub(crate) fn render_summary(reports: &[PhaseVtReport]) -> String {
    let mut lines: Vec<String> = vec!["VT verification summary:".to_string()];
    if reports.iter().all(|r| r.lines.is_empty()) {
        lines.push("  (no VT criteria to check)".to_string());
    }
    for report in reports {
        if report.lines.is_empty() {
            continue;
        }
        lines.push(format!("  {}:", report.phase_id));
        lines.extend(
            report
                .lines
                .iter()
                .map(|l| format!("    {}", render_line(l))),
        );
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

/// One rendered verdict line. Glyph + label are named constants; `Fail` and
/// `Waived` surface their reason.
fn render_line(line: &VtLine) -> String {
    let id = &line.id;
    match &line.verdict {
        VtVerdict::Pass => format!("{GLYPH_PASS} {LABEL_PASS}        {id}"),
        VtVerdict::Fail { reason } => format!("{GLYPH_FAIL} {LABEL_FAIL}        {id} — {reason}"),
        VtVerdict::Uncheckable => {
            format!("{GLYPH_UNCHECKABLE} {LABEL_UNCHECKABLE} {id} — no structured mandate")
        }
        VtVerdict::Waived { reason } => {
            format!("{GLYPH_WAIVED} {LABEL_WAIVED}      {id} — {reason}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{Plan, PlanPhase, VerificationCriterion};
    use std::cell::Cell;

    /// A bare VT row with the given id and no structured mandate.
    fn vt(id: &str) -> VerificationCriterion {
        VerificationCriterion {
            id: id.to_string(),
            expects: String::new(),
            test_file: None,
            keywords: vec![],
            patterns: vec![],
            waived: false,
            waived_reason: None,
        }
    }

    /// A reader serving a single (path, source) mapping; everything else missing.
    fn one(path: &'static str, source: &'static str) -> impl Fn(&str) -> Option<String> {
        move |p: &str| (p == path).then(|| source.to_string())
    }

    // ---- VT-1: four-verdict + waived short-circuit -------------------------

    #[test]
    fn pass_when_file_exists_and_keyword_present() {
        let mut c = vt("VT-1");
        c.test_file = Some("a.rs".to_string());
        c.keywords = vec!["check_vt".to_string()];
        let verdict = check_vt(&c, &one("a.rs", "fn check_vt() {}"));
        assert_eq!(verdict, VtVerdict::Pass);
    }

    #[test]
    fn fail_when_mandated_file_absent() {
        let mut c = vt("VT-1");
        c.test_file = Some("missing.rs".to_string());
        c.keywords = vec!["whatever".to_string()];
        let verdict = check_vt(&c, &no_files);
        assert!(matches!(verdict, VtVerdict::Fail { .. }));
    }

    #[test]
    fn fail_when_keyword_absent_from_source() {
        let mut c = vt("VT-1");
        c.test_file = Some("a.rs".to_string());
        c.keywords = vec!["nonexistent".to_string()];
        let verdict = check_vt(&c, &one("a.rs", "fn check_vt() {}"));
        assert!(matches!(verdict, VtVerdict::Fail { .. }));
    }

    #[test]
    fn uncheckable_when_keywords_but_no_test_file() {
        let mut c = vt("VT-1");
        c.keywords = vec!["x".to_string()]; // no test_file → A1
        assert_eq!(check_vt(&c, &no_files), VtVerdict::Uncheckable);
    }

    #[test]
    fn waived_short_circuits_before_any_read() {
        let mut c = vt("VT-1");
        c.waived = true;
        c.waived_reason = Some("infeasible — see /consult".to_string());
        c.test_file = Some("a.rs".to_string());
        // A reader that would panic if touched proves the waiver short-circuits.
        let touched = Cell::new(false);
        let reader = |_: &str| -> Option<String> {
            touched.set(true);
            None
        };
        let verdict = check_vt(&c, &reader);
        assert!(!touched.get(), "waiver must short-circuit before fs read");
        assert_eq!(
            verdict,
            VtVerdict::Waived {
                reason: "infeasible — see /consult".to_string()
            }
        );
    }

    #[test]
    fn existence_only_mandate_passes_without_keywords() {
        let mut c = vt("VT-1");
        c.test_file = Some("a.rs".to_string()); // no keywords/patterns
        assert_eq!(check_vt(&c, &one("a.rs", "anything")), VtVerdict::Pass);
    }

    #[test]
    fn pattern_line_anchored_match() {
        let mut c = vt("VT-1");
        c.test_file = Some("a.rs".to_string());
        c.patterns = vec![r"^\s*fn check_vt".to_string()];
        assert_eq!(
            check_vt(&c, &one("a.rs", "    fn check_vt() {}")),
            VtVerdict::Pass
        );
        // same token only mid-line (not line-anchored) → Fail.
        assert!(matches!(
            check_vt(&c, &one("a.rs", "let x = fn check_vt;")),
            VtVerdict::Fail { .. }
        ));
    }

    // ---- VT-2: raw substring (POL-002 — no host-language stripping) + the
    // `patterns` escalation ---------------------------------------------------

    #[test]
    fn keyword_as_string_argument_satisfies() {
        // The collision that drove Option D: an e2e references a CLI token as a
        // STRING literal (`cmd.arg("check")`). Plain raw substring satisfies it —
        // no string-stripping (POL-002), so legitimate e2e mandates do not
        // false-fail.
        let mut c = vt("VT-1");
        c.test_file = Some("e2e.rs".to_string());
        c.keywords = vec!["check".to_string(), "regression".to_string()];
        let src = r#"cmd.arg("check").arg("regression");"#;
        assert_eq!(check_vt(&c, &one("e2e.rs", src)), VtVerdict::Pass);
    }

    #[test]
    fn pattern_escalation_fails_when_shape_absent() {
        // `patterns` is the opt-in stronger shape: a line-anchored regex that
        // must match some source line, else Fail. The author owns the regex —
        // language-agnostic, unlike a baked-in comment stripper.
        let mut c = vt("VT-1");
        c.test_file = Some("a.rs".to_string());
        c.patterns = vec![r"^\s*assert_eq!\(.*census".to_string()];
        // shape present → Pass
        assert_eq!(
            check_vt(&c, &one("a.rs", "    assert_eq!(x, census);")),
            VtVerdict::Pass
        );
        // shape absent (token present but not in the mandated form) → Fail
        assert!(matches!(
            check_vt(&c, &one("a.rs", "let census = 1;")),
            VtVerdict::Fail { .. }
        ));
    }

    // ---- VT-3: non-gated paths + report shape ------------------------------

    fn phase(id: &str, vts: Vec<VerificationCriterion>) -> PlanPhase {
        PlanPhase {
            id: id.to_string(),
            name: String::new(),
            objective: String::new(),
            entrance_criteria: vec![],
            exit_criteria: vec![],
            verification: vts,
        }
    }

    #[test]
    fn va_vh_only_phase_emits_no_vt_lines() {
        let plan = Plan {
            phases: vec![phase("PHASE-01", vec![vt("VA-1"), vt("VH-1")])],
        };
        let reports = check_phases(&plan, &no_files);
        assert_eq!(reports.len(), 1);
        assert!(reports[0].lines.is_empty(), "VA/VH are never gated");
        assert!(!has_failure(&reports));
    }

    #[test]
    fn empty_plan_yields_empty_report_no_failure() {
        let plan = Plan { phases: vec![] };
        let reports = check_phases(&plan, &no_files);
        assert!(reports.is_empty());
        assert!(!has_failure(&reports));
    }

    #[test]
    fn has_failure_true_when_a_vt_fails() {
        let mut c = vt("VT-1");
        c.test_file = Some("missing.rs".to_string());
        let plan = Plan {
            phases: vec![phase("PHASE-01", vec![c])],
        };
        let reports = check_phases(&plan, &no_files);
        assert!(has_failure(&reports));
    }

    // ---- render --------------------------------------------------------------

    #[test]
    fn render_surfaces_all_four_verdicts() {
        // VT-1 (S6): every verdict — Pass / Fail / Uncheckable / Waived — renders,
        // each reason surfaced, the latter two distinctly from Pass/Fail.
        let reports = vec![PhaseVtReport {
            phase_id: "PHASE-01".to_string(),
            lines: vec![
                VtLine {
                    id: "VT-1".to_string(),
                    verdict: VtVerdict::Pass,
                },
                VtLine {
                    id: "VT-2".to_string(),
                    verdict: VtVerdict::Fail {
                        reason: "test_file missing.rs not found".to_string(),
                    },
                },
                VtLine {
                    id: "VT-3".to_string(),
                    verdict: VtVerdict::Uncheckable,
                },
                VtLine {
                    id: "VT-4".to_string(),
                    verdict: VtVerdict::Waived {
                        reason: "infeasible".to_string(),
                    },
                },
            ],
        }];
        let out = render_summary(&reports);
        assert!(out.contains(LABEL_PASS));
        assert!(out.contains(LABEL_FAIL));
        assert!(out.contains(LABEL_UNCHECKABLE));
        assert!(out.contains(LABEL_WAIVED));
        assert!(out.contains("missing.rs"), "the Fail reason must surface");
        assert!(out.contains("infeasible"), "the Waived reason must surface");
    }

    #[test]
    fn render_notes_empty_when_no_vt() {
        let out = render_summary(&[]);
        assert!(out.contains("no VT criteria"));
    }

    fn no_files(_: &str) -> Option<String> {
        None
    }
}
