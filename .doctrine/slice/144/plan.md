# Implementation Plan SL-144: ADR-005 full compliance: reference-doc IA, user hooks, restate-line audit

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Five phases turn `design.md`'s four-tier access model + hook contracts +
restate-line audit into edits to the `install/` ship surface. The spine is
**analyse before editing**: PHASE-01 produces the externalised, reviewed
artefacts (ownership matrix, doc→fact inventory, pointer graph, ledger) that
gate every later doc change, so the corrective phases execute against a fixed
target instead of drifting on taste. PHASE-02..05 then edit in batches the
**re-embed footgun** forces (each `install/*` edit needs `touch src/install.rs`
+ `cargo build` to ship), each phase ending green, with reachability + boot
parity gated last.

## Sequencing & Rationale

- **PHASE-01 first, and edit-free (design D6).** The external review's sharpest
  finding was that deferring the using-doctrine.md overconcentration question to
  a self-graded audit is marking-own-homework. So the matrix is a *pre-execution
  deliverable*: it decides ownership and the conditional `install/hooks.md` split
  up front, and becomes the artefact PHASE-05's coherence VA reconciles against.
  No `.md` content changes here — only analysis. Everything downstream cites it.

- **PHASE-02 currency before PHASE-04 restate-line.** A skill can only *cite*
  `using-doctrine.md`/`glossary.md` by name instead of restating a flag table if
  those docs are current and own the fact. Bringing the reference docs current
  first gives the restate-line fixes a legitimate point-at target. Currency also
  enforces single-ownership per kind (mention ≠ ownership) — the matrix decides
  which of the two docs owns each.

- **PHASE-03 hooks + retirement after the matrix, before reachability close.**
  The hook contracts and the reachability contract land in the surface PHASE-01
  assigned (using-doctrine.md, or hooks.md if split). boot-footer.md retirement
  is a *deletion* — it must leave the automatic ship set, so the file is deleted,
  not just de-referenced, guarded by a new install.rs absence test and a grep
  gate scoped to *file/asset references* (the surviving `boot-footer` token in
  boot.rs:123 is a legitimate retirement-concept comment, not a read path — the
  gate must not false-positive on it).

- **PHASE-04 restate-line + a regression tripwire.** Manual triage remediates the
  8 candidates (evidence-bound, R-C1); the tripwire is the control that makes
  compliance survive past close (external review E). It is a shell/`just` recipe,
  deliberately *not* wired into the `doctrine check` Rust path — this is a
  no-CLI-code slice (§1); a `doctrine check` integration is a follow-up.

- **PHASE-05 last: PUSH completeness + reachability close + parity gate.** The
  reference-forms block already projects into the boot snapshot (boot.rs
  assertion), so R-OQ-5 is verify-and-fix-if-gap. Reachability closes the pointer
  graph drafted in PHASE-01 against the *edited* tree. The final `doctrine boot
  --check` gates shipped == regenerated; the human acceptance criterion (VH-1)
  ties closure to an empty ledger, not a prose claim.

## Notes

- **Scope boundary — no CLI-code.** The tripwire (PHASE-04) and every embed
  regression ride shell/`just`/existing test seams. ISS-208 (undiscoverable
  `--boot-map`) and IDE-030 (stale-client currency) are named but out of scope.
- **Reachability is fresh-install only.** `build_plan` is write-if-absent, so the
  reachability claim is scoped to what ships from this repo; stale client copies
  are IDE-030's problem, not this slice's.
- **Verification modes are deliberate.** Currency and IA coherence resist a unit
  test — they are VA (agent reconciles against the PHASE-01 artefacts) rather
  than silently skipped. The two hard VTs (boot-footer absent; reference-forms
  projected) anchor the mechanical guarantees.
