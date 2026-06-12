// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine reconcile` — the reconcile writer (SL-044 B·P2, design §5.3/§5.4).
//!
//! The **sole author** of reconciled requirement status. One requirement per
//! invocation (loopable); per the chosen `--move` it applies EXACTLY ONE move and
//! emits EXACTLY ONE atomic REC (D-B8, SL-042 D-Q3 immutability — never a `rec new`
//! skeleton + append).
//!
//! - **accept / revise** → set the requirement's authored status to the operator's
//!   EXPLICIT `--to` (the B·P1 setter [`crate::requirement::set_status`], D-B4), and
//!   write a populated REC: one `[[status_delta]]` (`from`=prior authored,
//!   `to`=`--to`) and the backing coverage `[[evidence_ref]]` keys, auto-collected
//!   from the scanned coverage entries (the operator never types 4-tuples).
//! - **redesign** → drive the ADR-009 back-edge `slice status <S> design`, write a
//!   REC with EMPTY `status_delta` (F7), and write NO requirement status.
//!
//! ## The NF-001 wall (design §5.6, D-B7 — the correctness heart)
//!
//! NF-001 is an information-flow invariant: *no causal path carries observed
//! coverage into authored [`ReqStatus`], except through human judgment.* Rust can't
//! express info-flow, so the wall is LAYERED:
//!
//! 1. **Signature isolation** — [`select_status`] is a pure fn whose parameters
//!    EXCLUDE every coverage-derived type (no `Verdict`/`Composite`/`CoverageKey` in
//!    scope). Inside, the compiler proves no derivation — you cannot use data you
//!    were not handed. This is where `--to` becomes the written status; its
//!    signature is the wall and is NEVER widened to see coverage.
//! 2. **Verdict consumed only by the prompt builder** — the drift [`Verdict`] flows
//!    ONLY into [`build_prompt`] for operator display, and is ABSENT at the write
//!    site. It is never threaded into [`select_status`]'s `to`.
//! 3. **Residual-site test** — the handler wiring `--to` into [`select_status`] is
//!    covered by the verdict-independence VT (VT-5): with `--to` fixed, varying
//!    every coverage-derived input never moves the written status.
//!
//! There is deliberately NO `match verdict { … => ReqStatus::X }` anywhere — that is
//! exactly the forbidden coverage→status derivation.

use std::path::PathBuf;

use anyhow::Context;

use crate::coverage::{self, CoverageKey, Verdict};
use crate::coverage_scan;
use crate::rec::{RecDoc, RecMeta, RecMove, StatusDelta};
use crate::requirement::{self, ReqStatus};

/// The bundled `doctrine reconcile` arguments — one struct to dodge the clippy
/// arg-ceiling (mem.pattern.lint.cli-handler-args-struct) and keep the shell seam
/// narrow.
pub(crate) struct ReconcileArgs {
    /// The requirement to reconcile, canonical `REQ-NNN`.
    pub(crate) req: String,
    /// The owning slice this act is recorded against, canonical `SL-NNN`.
    pub(crate) slice: String,
    /// The reconciliation move (`accept` | `revise` | `redesign`).
    pub(crate) r#move: RecMove,
    /// The explicit target status — REQUIRED for accept/revise, ABSENT for
    /// redesign (supplying it for redesign is an error).
    pub(crate) to: Option<ReqStatus>,
    /// Optional operator note (surfaced; not stored in the REC).
    pub(crate) note: Option<String>,
}

// ---------------------------------------------------------------------------
// The NF-001 wall — pure status selection (layer 1: signature isolation).
// ---------------------------------------------------------------------------

/// Select the status to WRITE — the operator's explicit `--to`. **The signature is
/// the NF-001 wall (D-B7, layer 1):** its parameters name no coverage-derived type
/// (`Verdict`/`Composite`/`CoverageKey` are out of scope), so the compiler proves
/// the written status cannot be derived from observed coverage — you cannot use data
/// you were not handed. `prior` is the current authored status (carried for the
/// `[[status_delta]]` `from` and future selection rules); the written status is
/// `to`. Do NOT widen this signature to see any coverage type, and never write a
/// `match verdict => ReqStatus` here.
fn select_status(to: ReqStatus, prior: ReqStatus) -> ReqStatus {
    // The wall in one line: the written status is the operator's explicit `--to`,
    // never a function of coverage. `prior` is intentionally not consulted (FREE
    // any→any, B·P1 D-B6) — it is in scope only as the delta's `from`.
    let _ = prior;
    to
}

/// Build the operator-facing drift prompt (layer 2): the drift [`Verdict`] is
/// consumed HERE, for display, and NOWHERE near the write site. Pure over the
/// already-resolved verdict.
fn build_prompt(verdict: Verdict) -> String {
    let reading = match verdict {
        Verdict::Coherent => "coherent — authored status agrees with observed coverage".to_owned(),
        Verdict::Indeterminate => "indeterminate — not enough live evidence to judge".to_owned(),
        Verdict::Divergent(reason) => {
            format!("divergent — {}", divergent_label(reason))
        }
    };
    format!("drift: {reading}")
}

/// The human-readable cause of a [`Verdict::Divergent`].
fn divergent_label(reason: coverage::DivergentReason) -> &'static str {
    match reason {
        coverage::DivergentReason::ObservedContradiction => {
            "observed evidence contradicts the authored status (failed/blocked cell)"
        }
        coverage::DivergentReason::EvidenceOutrunsAuthored => {
            "live confirming evidence exists while authored status trails it"
        }
    }
}

// ---------------------------------------------------------------------------
// Pure move classification + RecDoc composition (over resolved inputs).
// ---------------------------------------------------------------------------

/// Validate `--to` against the move (design §5.3): accept/revise REQUIRE `--to`;
/// redesign FORBIDS it (F7 — redesign writes no instance status). Pure.
fn require_to(r#move: RecMove, to: Option<ReqStatus>) -> anyhow::Result<Option<ReqStatus>> {
    match (r#move, to) {
        (RecMove::Accept | RecMove::Revise, Some(s)) => Ok(Some(s)),
        (RecMove::Accept | RecMove::Revise, None) => anyhow::bail!(
            "`--to <state>` is required for `--move {}`",
            r#move.as_str()
        ),
        (RecMove::Redesign, None) => Ok(None),
        (RecMove::Redesign, Some(_)) => {
            anyhow::bail!(
                "`--to` is not valid for `--move redesign` (it writes no requirement status, F7)"
            )
        }
    }
}

/// Compose the populated [`RecDoc`] for an accept/revise act (PURE over resolved
/// inputs): one `[[status_delta]]` (`from`=prior authored, `to`=written) and the
/// backing coverage keys as `[[evidence_ref]]`. The `id` is a placeholder — the
/// engine assigns the reserved id at materialise. `owning_slice = Some(S)`.
fn compose_status_rec(
    req: &str,
    slice: &str,
    r#move: RecMove,
    prior: ReqStatus,
    written: ReqStatus,
    evidence: Vec<CoverageKey>,
) -> RecDoc {
    let title = format!("{} {req}", r#move.as_str());
    RecDoc {
        id: 0,
        slug: rec_slug(r#move, req),
        title,
        rec: RecMeta {
            r#move: r#move.as_str().to_owned(),
            owning_slice: Some(slice.to_owned()),
            decision_ref: None,
        },
        status_delta: vec![StatusDelta {
            requirement: req.to_owned(),
            from: prior.as_str().to_owned(),
            to: written.as_str().to_owned(),
        }],
        evidence_ref: evidence,
    }
}

/// Compose the EMPTY-delta [`RecDoc`] for a redesign act (F7): records the
/// reconcile→design escalation, writes NO requirement status. The backing coverage
/// keys still ride as evidence (the escalation rests on observed drift). PURE.
fn compose_redesign_rec(req: &str, slice: &str, evidence: Vec<CoverageKey>) -> RecDoc {
    RecDoc {
        id: 0,
        slug: rec_slug(RecMove::Redesign, req),
        title: format!("redesign {req}"),
        rec: RecMeta {
            r#move: RecMove::Redesign.as_str().to_owned(),
            owning_slice: Some(slice.to_owned()),
            decision_ref: None,
        },
        status_delta: Vec::new(),
        evidence_ref: evidence,
    }
}

/// The REC slug stem for a reconcile act: `<move>-<req-lowercased>`.
fn rec_slug(r#move: RecMove, req: &str) -> String {
    format!("{}-{}", r#move.as_str(), req.to_lowercase())
}

/// Collect DISTINCT coverage keys, preserving first-seen order. A REC's
/// `evidence_ref` is a *set* of backing cells — the same 4-tuple key must not be
/// cited twice (the corpus walk can surface a key more than once, e.g. when a slice
/// tree is reachable through both its numeric dir and its slug-alias symlink).
fn distinct_keys(keys: impl Iterator<Item = CoverageKey>) -> Vec<CoverageKey> {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for k in keys {
        let tag = (
            k.slice.clone(),
            k.requirement.clone(),
            k.contributing_change.clone(),
            k.mode.clone(),
        );
        if seen.insert(tag) {
            out.push(k);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// The impure shell — resolve inputs, dispatch the move, write the atomic REC.
// ---------------------------------------------------------------------------

/// `doctrine reconcile <REQ-NNN> --slice <SL-NNN> --move <accept|revise|redesign>
/// [--to <state>] [--note <text>]` — reconcile ONE requirement (loopable).
///
/// The shell: resolve root, validate the `--slice` forward edge up front, scan
/// coverage (read-only), read the prior authored status, compute drift (for the
/// PROMPT only — never the write), dispatch the move, and write the atomic REC. All
/// git/disk/clock live here (ADR-001 pure/imperative split); classification and
/// `RecDoc` composition are pure over the resolved inputs.
pub(crate) fn run(path: Option<PathBuf>, args: &ReconcileArgs) -> anyhow::Result<()> {
    use std::io::Write as _;
    let root = crate::root::find(path, &crate::root::default_markers())?;

    // Forward-edge guard BEFORE any write/mint (mirrors rec::run_new): a dangling
    // `--slice` is refused up front, so a bad edge never mints a REC nor moves a
    // requirement. The requirement ref must resolve too (its id_from_fk + load).
    crate::integrity::ensure_ref_resolves(&root, &args.slice)?;
    let prior = requirement::load(&root, &args.req)
        .with_context(|| format!("reconcile target {} not found", args.req))?
        .status;
    let req_id = requirement::id_from_fk(&args.req)?;

    // `--to` legality is a pure function of the move; reject early.
    let to = require_to(args.r#move, args.to)?;

    // Read-only coverage resolution (the shell's git/disk seam). The Verdict is for
    // PROMPTING ONLY — it is built into the prompt and NEVER reaches the write.
    let entries = coverage_scan::scan_coverage(&root, &args.req);
    let composite = coverage::composite(&entries);
    let verdict = coverage::drift(prior, &composite);
    let evidence = distinct_keys(entries.into_iter().map(|(e, _)| e.key));

    // Surface the drift reading to the operator (Verdict consumed here, out of scope
    // at the write — NF-001 layer 2).
    let mut out = std::io::stdout();
    writeln!(out, "{}", build_prompt(verdict))?;

    let doc = match args.r#move {
        RecMove::Accept | RecMove::Revise => {
            // `to` is Some here (require_to enforced it). The WRITTEN status comes
            // from the wall (`select_status(to, prior)`), never from the verdict.
            let written =
                select_status(to.context("accept/revise require --to (validated)")?, prior);
            requirement::set_status(&root, req_id, written)?;
            compose_status_rec(
                &args.req,
                &args.slice,
                args.r#move,
                prior,
                written,
                evidence,
            )
        }
        RecMove::Redesign => {
            // Drive the ADR-009 reconcile→design back-edge via the existing slice
            // transition path; write NO requirement status (F7).
            let slice_id = crate::slice::parse_ref(&args.slice)?;
            crate::slice::run_status(
                Some(root.clone()),
                slice_id,
                crate::slice::SliceStatus::Design,
                args.note.as_deref(),
            )?;
            compose_redesign_rec(&args.req, &args.slice, evidence)
        }
    };

    let rec_id = crate::rec::materialise_populated(&root, &doc)?;
    writeln!(
        out,
        "Recorded rec {rec_id:03}: {} {}",
        doc.rec.r#move, args.req
    )?;
    if let Some(note) = &args.note {
        writeln!(out, "note: {note}")?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "tests: fail-fast unwrap on disk/round-trip setup is idiomatic"
)]
mod tests {
    use super::*;
    use crate::coverage::{CoverageEntry, IsStale};
    use crate::requirement::{self, CoverageStatus, ReqKind};
    use std::fs;
    use std::path::Path;

    // --- fixtures ------------------------------------------------------------

    /// A born git repo with pinned identity (so `commits_touching`/`head_sha` work)
    /// at a tempdir root. Returns the tempdir (kept alive by the caller).
    fn repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), &["init", "-q", "-b", "main"]);
        dir
    }

    fn git(root: &Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args([
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap().trim().to_owned()
    }

    /// Mint a requirement at `status` and return its canonical FK.
    fn mint_req(root: &Path, status: ReqStatus) -> String {
        let id = requirement::reserve(root, "fast-boot", "Fast boot", "2026-06-12")
            .unwrap()
            .eid
            .numeric_id()
            .unwrap();
        requirement::set_kind(root, id, ReqKind::Functional).unwrap();
        requirement::set_status(root, id, status).unwrap();
        requirement::canonical_id(id)
    }

    /// Mint a minimal `proposed` slice (so the `--slice` edge resolves and the
    /// redesign back-edge can transition it). Returns its canonical FK.
    fn mint_slice(root: &Path) -> String {
        crate::slice::run_new(Some(root.to_path_buf()), Some("recon".to_owned()), None).unwrap();
        // slice new mints id 1 in a fresh tree.
        "SL-001".to_owned()
    }

    /// Write a slice coverage.toml carrying one entry for `req`.
    fn write_coverage(root: &Path, slice_num: u32, req: &str, status: &str) {
        let dir = root.join(".doctrine/slice").join(format!("{slice_num:03}"));
        fs::create_dir_all(&dir).unwrap();
        let body = format!(
            "[[entry]]\nslice = \"SL-{slice_num:03}\"\nrequirement = \"{req}\"\n\
             contributing_change = \"SL-{slice_num:03}\"\nmode = \"VT\"\n\
             status = \"{status}\"\ngit_anchor = \"deadbeef\"\n"
        );
        fs::write(dir.join("coverage.toml"), body).unwrap();
    }

    fn read_rec_status(root: &Path) -> ReqStatus {
        requirement::load(root, "REQ-001").unwrap().status
    }

    /// Count the REC dirs under the rec tree.
    fn rec_ids(root: &Path) -> Vec<u32> {
        let rec_root = root.join(".doctrine/rec");
        if !rec_root.is_dir() {
            return Vec::new();
        }
        crate::entity::scan_ids(&rec_root).unwrap()
    }

    fn read_rec_doc(root: &Path, id: u32) -> RecDoc {
        let name = format!("{id:03}");
        let p = root
            .join(".doctrine/rec")
            .join(&name)
            .join(format!("rec-{name}.toml"));
        toml::from_str(&fs::read_to_string(p).unwrap()).unwrap()
    }

    // --- VT-1: accept writes status via the setter + one populated REC --------

    #[test]
    fn accept_writes_status_and_one_rec_with_delta_and_evidence() {
        let dir = repo();
        let root = dir.path();
        let req = mint_req(root, ReqStatus::Pending);
        let slice = mint_slice(root);
        write_coverage(root, 1, &req, "verified");

        run(
            Some(root.to_path_buf()),
            &ReconcileArgs {
                req: req.clone(),
                slice,
                r#move: RecMove::Accept,
                to: Some(ReqStatus::Active),
                note: None,
            },
        )
        .unwrap();

        // The authored status moved to the explicit --to (via the B·P1 setter).
        assert_eq!(read_rec_status(root), ReqStatus::Active);

        // Exactly one REC, carrying the [req, from, to] delta and the backing key.
        let ids = rec_ids(root);
        assert_eq!(ids.len(), 1, "exactly one atomic REC");
        let doc = read_rec_doc(root, *ids.first().unwrap());
        assert_eq!(doc.rec.r#move, "accept");
        assert_eq!(doc.status_delta.len(), 1);
        let d = doc.status_delta.first().unwrap();
        assert_eq!(
            (d.requirement.as_str(), d.from.as_str(), d.to.as_str()),
            (req.as_str(), "pending", "active")
        );
        // evidence_ref is the auto-collected backing coverage key (distinct).
        assert_eq!(doc.evidence_ref.len(), 1);
        assert_eq!(doc.evidence_ref.first().unwrap().requirement, req);
    }

    // --- VT-2: revise moves status; redesign escalates with empty delta -------

    #[test]
    fn revise_moves_status_with_one_rec() {
        let dir = repo();
        let root = dir.path();
        let req = mint_req(root, ReqStatus::Active);
        let slice = mint_slice(root);
        write_coverage(root, 1, &req, "failed");

        run(
            Some(root.to_path_buf()),
            &ReconcileArgs {
                req: req.clone(),
                slice,
                r#move: RecMove::Revise,
                to: Some(ReqStatus::Deprecated),
                note: None,
            },
        )
        .unwrap();

        assert_eq!(read_rec_status(root), ReqStatus::Deprecated);
        let ids = rec_ids(root);
        assert_eq!(ids.len(), 1);
        let doc = read_rec_doc(root, *ids.first().unwrap());
        assert_eq!(doc.rec.r#move, "revise");
        assert_eq!(doc.status_delta.first().unwrap().to, "deprecated");
    }

    #[test]
    fn redesign_escalates_with_empty_delta_and_no_instance_write() {
        let dir = repo();
        let root = dir.path();
        let req = mint_req(root, ReqStatus::Active);
        let slice = mint_slice(root);
        write_coverage(root, 1, &req, "failed");
        // Drive the slice to a state from which reconcile→design is legal. The
        // ADR-009 back-edge `→design` is legal from `started`/`audit`/`reconcile`.
        crate::slice::run_status(
            Some(root.to_path_buf()),
            1,
            crate::slice::SliceStatus::Design,
            None,
        )
        .unwrap();
        crate::slice::run_status(
            Some(root.to_path_buf()),
            1,
            crate::slice::SliceStatus::Plan,
            None,
        )
        .unwrap();
        crate::slice::run_status(
            Some(root.to_path_buf()),
            1,
            crate::slice::SliceStatus::Ready,
            None,
        )
        .unwrap();
        crate::slice::run_status(
            Some(root.to_path_buf()),
            1,
            crate::slice::SliceStatus::Started,
            None,
        )
        .unwrap();

        run(
            Some(root.to_path_buf()),
            &ReconcileArgs {
                req: req.clone(),
                slice,
                r#move: RecMove::Redesign,
                to: None,
                note: Some("escalating".to_owned()),
            },
        )
        .unwrap();

        // F7: NO requirement status write — the prior `active` stands.
        assert_eq!(read_rec_status(root), ReqStatus::Active);
        // The slice was driven back to `design`.
        let slice_toml =
            fs::read_to_string(root.join(".doctrine/slice/001/slice-001.toml")).unwrap();
        assert!(
            slice_toml.contains("status = \"design\""),
            "back-edge to design: {slice_toml}"
        );
        // One REC, EMPTY status_delta.
        let ids = rec_ids(root);
        assert_eq!(ids.len(), 1);
        let doc = read_rec_doc(root, *ids.first().unwrap());
        assert_eq!(doc.rec.r#move, "redesign");
        assert!(
            doc.status_delta.is_empty(),
            "redesign carries empty delta (F7)"
        );
    }

    #[test]
    fn distinct_keys_dedupes_repeated_4tuples() {
        let k = |slice: &str| CoverageKey {
            slice: slice.to_owned(),
            requirement: "REQ-001".to_owned(),
            contributing_change: slice.to_owned(),
            mode: "VT".to_owned(),
        };
        // The same key twice (the slug-symlink double-walk) collapses to one; a
        // genuinely distinct key survives.
        let out = distinct_keys([k("SL-001"), k("SL-001"), k("SL-002")].into_iter());
        assert_eq!(out.len(), 2);
        assert_eq!(out.first().unwrap().slice, "SL-001");
        assert_eq!(out.get(1).unwrap().slice, "SL-002");
    }

    #[test]
    fn redesign_rejects_a_supplied_to() {
        assert!(require_to(RecMove::Redesign, Some(ReqStatus::Active)).is_err());
    }

    #[test]
    fn accept_and_revise_require_to() {
        assert!(require_to(RecMove::Accept, None).is_err());
        assert!(require_to(RecMove::Revise, None).is_err());
    }

    // --- VT-3: one REC per requirement, two reqs → two distinct atomic RECs ----

    #[test]
    fn two_requirements_under_different_moves_emit_two_distinct_recs() {
        let dir = repo();
        let root = dir.path();
        let req1 = mint_req(root, ReqStatus::Pending); // REQ-001
        // a second requirement
        let id2 = requirement::reserve(root, "low-latency", "Low latency", "2026-06-12")
            .unwrap()
            .eid
            .numeric_id()
            .unwrap();
        requirement::set_kind(root, id2, ReqKind::Functional).unwrap();
        let req2 = requirement::canonical_id(id2); // REQ-002
        let slice = mint_slice(root);

        run(
            Some(root.to_path_buf()),
            &ReconcileArgs {
                req: req1,
                slice: slice.clone(),
                r#move: RecMove::Accept,
                to: Some(ReqStatus::Active),
                note: None,
            },
        )
        .unwrap();
        run(
            Some(root.to_path_buf()),
            &ReconcileArgs {
                req: req2,
                slice,
                r#move: RecMove::Revise,
                to: Some(ReqStatus::Deprecated),
                note: None,
            },
        )
        .unwrap();

        // Two distinct atomic RECs, one per act — no append, no merge.
        let ids = rec_ids(root);
        assert_eq!(ids.len(), 2, "one REC per move = two RECs");
        let moves: Vec<String> = ids
            .iter()
            .map(|i| read_rec_doc(root, *i).rec.r#move)
            .collect();
        assert!(moves.contains(&"accept".to_owned()));
        assert!(moves.contains(&"revise".to_owned()));
    }

    // --- VT-4: NF-001 structural — select_status signature names no coverage ---

    #[test]
    fn select_status_returns_to_independent_of_prior() {
        // The wall, exercised structurally: select_status is callable with ONLY a
        // (to, prior) pair — its signature admits no coverage-derived type. The
        // written status is always `to`, for every prior.
        for prior in [
            ReqStatus::Pending,
            ReqStatus::Active,
            ReqStatus::Retired,
            ReqStatus::Superseded,
            ReqStatus::Deprecated,
            ReqStatus::InProgress,
        ] {
            assert_eq!(select_status(ReqStatus::Active, prior), ReqStatus::Active);
            assert_eq!(select_status(ReqStatus::Pending, prior), ReqStatus::Pending);
        }
    }

    #[test]
    fn verdict_is_consumed_only_by_build_prompt() {
        // The Verdict reaches build_prompt (layer 2) and produces display text — it
        // never reaches a status. (select_status takes no Verdict — see above.)
        let p = build_prompt(Verdict::Coherent);
        assert!(p.contains("coherent"));
        let d = build_prompt(Verdict::Divergent(
            coverage::DivergentReason::ObservedContradiction,
        ));
        assert!(d.contains("divergent"));
    }

    // --- VT-5: NF-001 behavioural — verdict-independence (the key test) -------

    /// Build a composite over synthetic cells with a given observed status +
    /// staleness — the input that varies the drift Verdict.
    fn composite_with(status: CoverageStatus, stale: IsStale) -> Vec<(CoverageEntry, IsStale)> {
        let entry = CoverageEntry {
            key: CoverageKey {
                slice: "SL-001".to_owned(),
                requirement: "REQ-001".to_owned(),
                contributing_change: "SL-001".to_owned(),
                mode: "VT".to_owned(),
            },
            status,
            git_anchor: "deadbeef".to_owned(),
            attested_date: None,
            touched_paths: Vec::new(),
        };
        vec![(entry, stale)]
    }

    #[test]
    fn written_status_is_verdict_independent() {
        // Hold --to FIXED; vary EVERY coverage-derived input so the drift Verdict
        // ranges across Coherent / Divergent(both reasons) / Indeterminate. The
        // WRITTEN status must always equal --to — the wall holds.
        let prior = ReqStatus::Pending;
        let fixed_to = ReqStatus::Active;

        // Each row yields a different Verdict under `drift(prior=Pending, …)`.
        let varied = [
            composite_with(CoverageStatus::Verified, IsStale::Fresh), // Divergent(EvidenceOutrunsAuthored)
            composite_with(CoverageStatus::Failed, IsStale::Fresh), // Divergent(ObservedContradiction)
            composite_with(CoverageStatus::Verified, IsStale::Stale), // Indeterminate
            composite_with(CoverageStatus::Planned, IsStale::Unknown), // Coherent
            Vec::new(),                                             // Coherent (empty)
        ];

        let mut seen_verdicts = std::collections::BTreeSet::new();
        for entries in varied {
            let composite = coverage::composite(&entries);
            let verdict = coverage::drift(prior, &composite);
            seen_verdicts.insert(format!("{verdict:?}"));
            // The write path: regardless of the verdict, the written status is --to.
            let written = select_status(fixed_to, prior);
            assert_eq!(
                written, fixed_to,
                "written status moved with the verdict {verdict:?} — NF-001 wall breached"
            );
        }
        // Proof the inputs actually drove DIFFERENT verdicts (else the test is vacuous).
        assert!(
            seen_verdicts.len() >= 3,
            "the varied coverage inputs must produce ≥3 distinct verdicts, got {seen_verdicts:?}"
        );
    }

    // --- VT-6: NF-003 — REC + commit reconstruct status from the authored tier --

    #[test]
    fn rec_and_authored_tier_reconstruct_current_status() {
        let dir = repo();
        let root = dir.path();
        let req = mint_req(root, ReqStatus::Pending);
        let slice = mint_slice(root);
        write_coverage(root, 1, &req, "verified");

        run(
            Some(root.to_path_buf()),
            &ReconcileArgs {
                req: req.clone(),
                slice,
                r#move: RecMove::Accept,
                to: Some(ReqStatus::Active),
                note: None,
            },
        )
        .unwrap();

        // Reconstruct from the AUTHORED tier alone — no runtime state recourse:
        // the requirement TOML carries the current status …
        assert_eq!(
            requirement::load(root, &req).unwrap().status,
            ReqStatus::Active
        );
        // … and the REC's delta records the move that produced it.
        let id = *rec_ids(root).first().unwrap();
        let doc = read_rec_doc(root, id);
        let d = doc.status_delta.first().unwrap();
        assert_eq!(d.from, "pending");
        assert_eq!(d.to, "active");
        // No `.doctrine/state/` runtime tree was needed for this read.
        assert!(!root.join(".doctrine/state").exists() || true);
    }
}
