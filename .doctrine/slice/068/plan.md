# Implementation Plan SL-068: Dispatch candidates for safe audit interaction

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

The design adds a candidate/admission layer beside the existing immutable
dispatch evidence refs. Exact `review/<slice>` and `phase/<slice>-NN` stay
forensic evidence; a new `candidate/<slice>/<label>` layer gives humans and
reviewers an ordinary Git branch to review, fix, and experiment on, with audit
fixes admitted back by immutable OID and close integrating only the admitted
close-target OID. The plan slices that surface into six phases that build
strictly bottom-up: the ledger, then each verb (create → status → admit), then
the candidate-aware close, then the discoverability surface that makes the whole
workflow ambiently findable.

The D9 governance precondition is already met before any code: ADR-006 D10 and
the ADR-012 candidate clauses are authored and accepted (notes.md, 2026-06-15).
PHASE-01 therefore re-confirms that acceptance at the data-model boundary rather
than spending a phase on the amendment.

## Sequencing & Rationale

**Bottom-up, one writable seam at a time.** Each verb in the design depends on
the ledger and on the verb before it, so the order is forced more than chosen:

- **PHASE-01 (ledger)** is the foundation every later phase reads and writes.
  It is pure data modelling on the proven `ledger.rs` manifest pattern
  (`Journal`/`Boundaries`/`Orthogonal`): a serde struct plus `parse`/`to_toml`
  plus an absent-file-defaults read. No git, no CLI — so it lands green in
  isolation and de-risks the storage contract (typed role/kind/payload,
  write-once OID identity, journaled status) before any behaviour leans on it.

- **PHASE-02 (create)** is the heaviest phase and the one new mechanic the
  design calls out as the *only* place the topology permits a 3-way
  auto-resolution. It owns provenance preconditions, zero-OID CAS branch
  creation, the no-ff 3-way merge seam (built on `git.rs` primitives — there is
  no high-level merge helper yet), conflict lifecycle, worktree provisioning,
  and the worker/raw-ref write guards. It is sequenced second because status and
  admit have nothing to display or admit until create can populate rows.

- **PHASE-03 (status)** is read-only and small, but precedes admit deliberately:
  the self-describing surface (evidence vs candidate, drift, next-safe-command)
  is what turns the SL-067 trap into a guided path, and admit's UX leans on the
  same drift/admission reporting. Keeping it a separate phase preserves a clean
  read/write split for TDD.

- **PHASE-04 (admit)** binds the immutable `admitted_oid` and validates merge
  provenance/ancestry with a read-revalidate-reread moved-ref guard. It depends
  on create's recorded `merge_oid`/`source_oid`/`base_oid` and on status to
  surface the resulting admission.

- **PHASE-05 (candidate-aware integrate)** wires the admitted close-target OID
  into the existing stage-2 `integrate` CAS replay. It is last among the
  mechanics because it consumes a current admission, and it must preserve the
  existing integrate behaviour (the behaviour-preservation gate): no close-time
  merge, moved-target refusal, admitted-OID targeting, `--edge` via a
  review-surface admission.

- **PHASE-06 (ambient discoverability)** is the OQ-1 minimum, deliberately
  scoped wider than dispatch SKILL.md. A shipped surface is invisible unless
  something points at it (`mem.pattern.distribution.shipped-not-reachable`), and
  a *condoned workflow* has to be discoverable from the ambient context a cold
  agent boots with — not only from the source. So this phase touches the whole
  discoverability stack: the dispatch skill that owns the workflow, the
  bracketing **/audit** and **/close** skills that hand off into and out of it,
  the **boot routing snapshot + CLAUDE.md known-gaps** that a session loads
  through the SessionStart hook, the **canon/governance projection** regenerated
  so the accepted ADR amendments and the new path are fresh, and **durable
  memory** so `retrieve-memory` surfaces the workflow and the admission-by-OID
  invariant. It runs last because guidance and memory should describe the real,
  exercised command shapes — not a plan that may still shift.

**Why these boundaries.** Phases 01–05 are mostly file-disjoint at the writable
seam (ledger struct → create fn → status fn → admit fn → integrate fn), which
keeps each TDD unit narrow and each phase endable green. The CLI wiring for each
verb rides in with its phase rather than as a separate surface phase, so a verb
is never half-wired across a phase boundary. PHASE-06 is the only phase that
edits authored skills/governance and records memory, isolating the
non-code discoverability work from the mechanic phases.

## Notes

- The design's `/code-review` and full `/audit` rewire (OQ-1, F-4) is explicitly
  out of v1 scope and routes to the existing IMP-023/IMP-042 review-skill debt;
  PHASE-06 records that deferral in backlog before close
  (`mem.system.lifecycle.defer-needs-backlog-before-close`).
- V1 close-target payload is `code` (D8); impl-bundle close-target support is
  deliberate/later and not on the default trunk path.
- PHASE-06 memory spans two classes (deliberately). The candidate workflow is a
  *product feature* doctrine clients use, so its orientation (the cli-command-map
  signpost, the workflow + admission-by-OID + the SL-067 evidence-ref-is-not-a-
  branch trap) is recorded as **global** masters (`memory/`, `--global`) that
  ship to every client via `memory sync`. The build-time gotchas (the `git.rs`
  3-way seam, create-conflict/CAS edges) stay **project-local** items
  (`.doctrine/memory/items/`, committed to this repo only). Installer ships the
  global corpus and creates an empty `items/` tree in a client — this repo's own
  items never ship.
- The invariants the design constraints (D9) demand be preserved —
  admission-by-OID, no-close-time-merge, provenance validation, raw-ref guards —
  are distributed as exit criteria across PHASE-02/04/05 rather than asserted
  once, so each is tested at the phase that introduces its seam.
