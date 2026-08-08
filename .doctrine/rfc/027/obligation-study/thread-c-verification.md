# Thread C — verification note

Verification of `raw/thread-c-frontier.md` (pi-research, 2026-08-08) against
primary sources. The raw file is kept verbatim; corrections live here.

Question put to the thread: **is the phase too coarse a unit of actionability?**
Leading hypothesis: EX criteria already *are* obligations.

## Verdict on the verdict

The thread answers **no — obligation-level dependency would not change the
frontier** — and the negative is broadly supported. That is the harder answer to
produce and it survives most checks. Two qualifications below; one of them is
larger than the frontier question itself.

### Confirmed

- **EX criteria are obligations.** The leading hypothesis holds. SL-233's EX rows
  name exact function signatures, test file paths, constant names and byte-level
  behaviours. No new concept is needed to identify a phase's obligations.
- **Requirements are not obligations.** Verified on SL-057: `REQ-256` appears in
  both PHASE-03 and PHASE-05, `REQ-255` in both PHASE-04 and PHASE-05
  (`.doctrine/slice/057/plan.toml:106,138,164`). A requirement is a cross-cutting
  concern several phases contribute to, so it cannot serve as a dependency node —
  a shared requirement node says nothing about ordering. This is a good argument
  and it is the thread's own.
- **Coupling is real, not incidental.** SL-233's own plan states the serialism is
  forced: phases 02–07 route through `src/design_run/` or
  `src/commands/design.rs`, so there is no file-disjoint pair to parallelise.

### Qualification 1 — it missed the best counter-example to its own verdict

The thread's edge table lists PHASE-10's dependency as EN-1 → PHASE-04 only. It
does not mention **PHASE-10 EN-2** (`.doctrine/slice/233/plan.toml:499`):

> "**PHASE-03 EX-4** is landed: receipt eviction already refuses to evict a
> receipt referenced by an outstanding delegation, so this phase consumes that
> guarantee rather than restating it."

That is a hand-authored, phase-qualified, **criterion-level** dependency — the
exact `OBL needs PHASE-NN.OBL-M` form under investigation, already written in
prose because the phase vocabulary could not express it. It does not overturn the
frontier conclusion (PHASE-10 is deferred architecturally, as the thread
correctly establishes), but it does falsify the stronger reading that authors
never reach below phase granularity. They do, when precision matters.

Related, also unremarked: SL-233 PHASE-11 EN-1 and PHASE-12 EN-1 both carry
`AMENDED by the 2026-07-29 split` clauses re-targeting a dependency from PHASE-06
onto its successors — a one-to-many split of a dependency *edge*, handled by
hand-editing prose with no lineage. That is brief 03 § D's late-dependency
correction, occurring in the wild.

### Qualification 2 — Doctrine already ships an "obligation" primitive

Thread A/B asserted *"there is no entity called 'obligation' in Doctrine"*.
**False, and the collision matters.**

`DEC-101` — *Obligation runbooks: ordered steps as asset data, verifiers as
executables* — is **accepted**, and SL-233 PHASE-16 ("Obligation runbook runner")
shipped it. `src/design_run/runbook.rs` exists; `install/design-prompts/*.toml`
carry the assets; SL-233 is `done`.

The shipped concept is at a different altitude — process obligations an agent
must discharge to advance a design-run stage, not plan work-units carrying
dependency edges. But it owns the word, and DEC-101's own context notes the four
process fragments already hold *"refusal-invariants wearing the word
'Obligation'"*.

**Two consequences.**

1. **Naming (STD-002).** A plan-level "obligation" would collide head-on with a
   shipped, accepted, user-visible vocabulary. Either the plan-level concept
   takes a different name, or the collision is deliberate and argued.

2. **The machinery is a precedent worth mining, not a rival.** PHASE-16 already
   solved problems RFC-027 poses as open:
   - **EX-2** — *"a discharge binds the DIGEST OF THE STEP'S DEFINITION (`id`,
     `text`, `required`, `verify`), not the step id alone. An id solves
     reference, not equivalence"* — editing a step's text makes its discharge
     stale by construction. That is a direct, shipped answer to RFC-027's open
     question 6 (how a revision marks downstream evidence stale) and to brief
     04's criterion-vs-binding staleness problem.
   - **EX-18** — the digest's canonical encoding is length-delimited and
     **versioned**, so it can change without silently invalidating persisted
     records.
   - **EX-15** — attested and verified discharges are distinguishable, and a step
     with no verifier is never rendered as verified. That is the VA/VH-vs-VT
     freshness distinction brief 04 asks for, already shipped.
   - **EX-19** — a scope fence: the runbook does not feed the closed `Condition`
     vocabulary, because *"truth may not flow from user-customisable material
     into a Doctrine-owned closed vocabulary"*. Directly relevant to whether
     project-supplied verifier adapters (brief 04) may satisfy Doctrine-owned
     criteria.

## Research reliability notes

Two process signals, recorded because this study leans on delegated research.

- **A fabricated reconciliation.** The thread reports *"SL-057 has 6 phases per
  `grep` but I only examined 5 … the sixth match is the header comment block"*.
  `grep -c '^\[\[phase\]\]' .doctrine/slice/057/plan.toml` returns **5**. There
  was no discrepancy; the explanation was invented to resolve one that did not
  exist. Harmless here, but it is confabulation in the shape of diligence.
- **Thread A/B's stale-comment failure repeated in a new form.** A/B trusted a
  code comment about the corpus; C asserted a corpus-wide absence ("no entity
  called obligation") without a positive control. SL-233 PHASE-16 EX-19 ends with
  the house rule both threads needed: *"Verify the fence by grep WITH A POSITIVE
  CONTROL."*

## Standing question, not yet a finding

`src/dispatch.rs:6826` sorts funnel rows under the comment *"Phase-id order IS
the ladder's order"*. SL-233's authored plan array order is `01–06, 13, 14, 11,
12, 10, 07, 16, 08, 09, 15` — deliberately re-sequenced after the 2026-07-29
split — while `compute_next_phases` is documented as *"the SOLE readiness
authority … scan in plan order"* (SL-206 PHASE-03 EX-2). The two orderings
disagree on this slice.

Whether that manifests as a defect depends on how `select_next`'s rungs consume
the sorted rows. **Not traced yet — do not cite as a defect until it is.**

## Where this leaves the hypothesis

RFC-027's H9 (obligation-level dependency improves actionability) is **not
supported** by three slices' evidence. The frontier does not move. What the
evidence does support is narrower and cheaper:

- EX criteria are already the obligations; no new storage is earned;
- the missing structure is not a dependency graph but the **unauthored EX→VT
  link** and the demotion of real criterion-level dependencies into EN prose;
- the grafting direction remains cheap if it is ever wanted (thread A/B), but on
  this evidence it currently has no consumer that would read a different answer.
