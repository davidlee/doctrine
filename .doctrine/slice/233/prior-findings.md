# SL-233 — prior findings (the campaign's dedup corpus)

Built once at campaign session S0, per `IMP-024` §5 — the single highest-return
pass. Inject this file **verbatim into every raiser** on `RV-341` (Kind A) and
`RV-342` (Kind B). Without it each raiser re-discovers, and re-reports, defects
that are already found, already disposed, or already deliberately left
unrepaired — of which this slice has an unusual number.

Sources, all read in full: the 60 verified findings across `RV-315`, `RV-320`,
`RV-321`, `RV-323`, `RV-324`, `RV-325`; 112 items censused from the 16 runtime
phase sheets; 67 from `notes.md` `### Learned` + `### Open`; and the open
backlog. Raw censuses kept at `.doctrine/state/slice/233/campaign/s0/`
(disposable).

**If your finding is on this list, do not raise it.** If it is *adjacent* to an
entry, raise it and name the entry it is adjacent to.

## 0. Two id traps — read before citing anything

- The gitignored runtime phase sheets were **not** swept by the `DEC` reseat. Their 13 `DEC-099` occurrences now mean **`DEC-105`** (moved on this branch at `b8cfbf3d8`). Authored files are clean — `notes.md`'s eight are narrative *about* the move.
- Slash-compressed id runs (`DEC-101/102/103`) carry only one `DEC-NNN` token. Match `DEC-[0-9]{3}(/[0-9]{2,3})+`, or a whole-token scan reports a false clean.

## 1. Live, known, and deliberately unrepaired — raise nothing

- `ISS-289` — PHASE-15 `VT-2` mandates a test that was never written; `verify-vt 233` exits 1. An audit judgement, left unrepaired on purpose.
- `ISS-290` — `change_log.rs:217` declares `(Outcome, ValueKind::Token)`; `run.rs:1442` builds it with `PayloadTerm::label`. Nothing checks construction against the declared shape. Unrepaired pending an owner ruling.
- `ISS-291` — `SKILL.compact.md` still teaches the pre-SL-233 eight-stage design machine.
- `ISS-288` — the shipped runbook verifier resolves `doctrine` on `PATH`.
- `ISS-285` / `ISS-286` / `ISS-287` — exploring-gate conditions are unimplemented stubs; gate evidence subjects are section-only; shipped `reviewing.md` misdescribes what a section attestation binds.
- `RV-324` F-2 · `DEC-100` · PHASE-15 `D7-R` — `materialise`'s final-check→rename lost-update window is **explicitly tolerated for v0.0.1**. No surface may claim preservation or detection; that claim's absence is the disposition, not an oversight.
- `RV-324` F-5 → `IMP-370` — digest derivations are not single-sourced by kind. PHASE-15 `EX-12` excluded the consolidation deliberately; `IMP-370` owns it at slice close.
- `IMP-372` — the asset-override seam is deferred, not delivered; v1 ships the runbook embedded.
- `IMP-373` — cursorless runbook rendering deferred off PHASE-08's critical path, with a named repayment trigger.
- `IMP-367` — seven design-run read surfaces have no reader at all, including `AcceptanceAttestation::basis` (the auditable half of `DEC-088`).
- PHASE-07 F-12 · PHASE-14 F-7 — `RunHeader.next_obligation` and `FragmentGroup` have no writer anywhere; `FragmentGroup` is unreachable from any v1 `apply` payload.
- PHASE-14 `W2-F2` — `slice::dispatch`'s `SliceCommand::Design` arm is unreachable and bails. No test pins that claim.

## 2. Disclosed in the code and already adjudicated — adjacency only

- PHASE-12 F-4 — six gate conditions remain payload-*claimed* rather than derived. The asymmetry is disclosed, not resolved.
- PHASE-12 F-5 — the two `DEC-088` acceptance paths differ in strictness: the lock path refuses a blank `basis`, the checkpoint path does not.
- PHASE-12 F-6 — invalidation rows are asymmetric; a section edit kills an integrated review and acceptance, but no row reports it.
- PHASE-13 F-3 · F-6 — `Refusal::CarriageReturnInDocument` is fieldless; `parse`'s two new refusals are produced but swallowed by `unwrap_or_default()`.
- ~~PHASE-15 F-3 residual — the *leading* citation boundary is unchecked, so `XQUE-177` is accepted as a citation.~~ **CORRECTED at S4, 2026-08-02: the first half is false at head and a raiser must not suppress a finding on its account.** `citation()` (`legacy.rs:337`) tests the leading boundary via `continues_id` before it tests anything else, and `a_prefix_glued_to_preceding_text_is_not_a_citation` (`legacy.rs:637`) asserts `XQUE-177` → `None`. The entry recorded a residual that PHASE-15 `EX-7`/`EX-8` then closed; S0 built this corpus from the ledgers, which recorded the residual and not the fix. S4 pass A caught the contradiction against its own reading and said so — the one thing it beat pass B on. The **second** half stands: `line_cites` cannot be reused (crate out-degree zero), so the rule is deliberately duplicated in `legacy.rs` and the doc comment says not to "fix" that. A live residual does remain, but it is not the one recorded: `continues_id` is `is_ascii_alphanumeric`, so a **non**-alphanumeric lead such as `_QUE-177` still reads as a citation. Considered and judged not worth raising (not a realistic token in a design document) — recorded here so S5 need not re-derive it.
- PHASE-16 F-11 — the `regressed` arm is structurally empty on a read (`design resume` passes `&[]`); the gate is where regression is detected.
- `RV-321` F-1 · F-2 · F-4 — accepted and **carried forward to PHASE-04** (unvalidated persisted `PayloadTerm` strings; copied `264`/`from`/`to`/`reason` literals). Whether PHASE-04 discharged them is squarely in Kind B scope; the `RV-321` findings themselves are not.
- `RV-325` F-2 — accepted with a remedy proposed and **owner ratification still required** (`DEC-103`'s unbounded serial runbook accretion).
- `RV-325` F-15 · `RV-315` F-1 — contest rejected, clarified with no rule change / accepted-routed (`review prime` fail-hards on any selector that globs an entity root).
- `ROW_FRAMING_BYTES = 10` is a disclosed conservative reserve that sketch §(e) depends on (`RV-321`, PHASE-04).

## 3. Test quality — already found, and found systemically

- **`VA-NC` is partially met or weak-red in seven phases** — 03, 04, 05, 09, 10, 12, 14. Reds were obtained by mutation after green rather than chronologically first (PHASE-09's are missing-file reds). Every sheet discloses it. Do not re-report this per phase; a *new* instance must say which of the seven it is adjacent to.
- The design-run e2e `Fixture` is duplicated five to seven ways across crates — `IMP-357`, `IMP-368`, `IMP-369`. `tests/common/` has no opt-in boundary, so ~30 e2e crates compile everything added there.
- No test drives the byte fields to saturation: sketch §(e)'s 15,576 B saturated worst case remains an *arithmetic* claim, not a measured one (PHASE-04 `VA-2`).
- Assertions already logged as untriggerable or uncovered: PHASE-10 F-2 (`accepted_proposal_retains_its_attribution` has no guard that can make it fail) and PHASE-10 F-8 (proposed section declaration is digest-fed but exercised only by construction).
- `cargo test -- --list` is stderr-sensitive, so contamination reads as growth; `VA-ES` is a shrinkage alarm only (PHASE-16 F-13).

## 4. Criteria defects — not code defects, and criteria are the owner's

- `IMP-365` class, six instances on this slice — a `VA` criterion unrunnable or vacuous *as literally worded*: PHASE-03 F-1, PHASE-10 F-1 (`VA-3` red at head and staying red), PHASE-11 `VA-3` (`declaration` contains `ratio`), PHASE-13 `VA-2`.
- PHASE-09 `F-P09.3` — `VA-4`'s literal text conflicts with the `EX-8` round-4 settlement. Flagged and deliberately **not** amended.
- PHASE-09 `VH-1` was accepted **hedged** — "plausibly useful… prepared to run the session to find out". Do not read it as unqualified acceptance.
- PHASE-01 and PHASE-09 are zero-`src/` authored prose. They are outside the review funnel entirely and belong to the audit's spec-coherence leg.

## 5. Tooling and process, already filed — out of scope for both ledgers

- `IMP-346` (`dispatch next` cannot express a coordinator-only phase), `IMP-352` (~10 integration fixtures spawn the binary without `.current_dir()`, false-redding every marked worker fork), `IMP-353`, `IMP-366`, `IMP-371`, `IMP-377`, `ISS-273`, `ISS-274`, `ISS-275`, `ISS-277`, `ISS-279`, `ISS-292`.
