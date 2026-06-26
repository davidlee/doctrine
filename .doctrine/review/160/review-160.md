# Review RV-160 — plan of SL-155

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This Inquisition interrogates the plan of SL-155 against the principle of small,
composable work, honest dependency naming in entry criteria, and the convention
that a phase sheet is a working document, not a hollow shell. The accused plan
groups seven one-liner fixes across six files plus a ~200-line list-verb feature
into a single phase, calling it "tight." The tribunal shall press on:

1. **Single-phase conflation.** Seven targeted edits across six files + a new CLI
   verb (~200 lines) in a single phase. The plan rationale says "no phase has a
   dependency on another" but that is a reason to NOT combine them — file-disjoint
   work should be separable into individually gateable phases. The "tight TDD
   cycle" claim is belied by the scope of work: one-liners and a list verb are
   different risk profiles, different test strategies, different verification
   scopes.

2. **Entry criteria as gate-checks, not dependencies.** EN-1 ("design.md locked
   and approved") and EN-2 ("plan.md written") are lifecycle-gate tautologies —
   they do not name substantive dependencies. Per `mem.pattern.doctrine.conventions`
   and the `EN criteria must name the honest dependency` memory, entry criteria
   should name the actual preconditions (e.g. "spec.rs parent doc comment
   identified", "TAGGABLE const location confirmed"), not structural gates.

3. **Criteria sets do not align with the design's verification.** The plan has
   EN-1/EN-2, EX-1→EX-5, VT-1→VT-5, VA-1/VA-2; the design has EN-01/EN-02,
   EX-01→EX-08, VT-03 (implicit). These are two different verification taxonomies
   with no mapping between them. Which is authoritative? The plan should
   incorporate, not re-derive, the design's criteria.

4. **Phase sheet emptiness.** The phase sheet (`phase-01.md`) has empty Reading
   list, Assumptions, Tasks, Risks, Decisions, and Findings sections, yet the
   phase is marked `in_progress`. A plan that advances to execution without
   populating its runtime sheet is a plan that has not actually planned — it has
   only declared.

5. **Sequencing omission.** The plan's "Sequencing" section says "Write tests
   first (red)" but the revision list tests need `list_rows` and `run_list` to
   exist before they can be written meaningfully — or do they? The test strategy
   is underspecified: are these unit tests (`#[cfg(test)]` in `revision.rs`) or
   black-box CLI goldens? The plan doesn't say.

6. **No handling of the design's scope-design contradiction.** The plan lists
   EN-1 ("design.md locked and approved") but RV-161's findings against the
   design are unresolved. A plan whose entry criterion is unsatisfied should
   not be `in_progress`.

The Inquisition expects the plan to decompose work into testable, gateable phases,
name honest dependencies, and keep its criteria aligned with the design it serves.
A single-phase grab-bag with hollow runtime sheets and tautological entry gates is
no plan at all — it is an aspiration wearing a plan's vestments.

## Synthesis

**Judgement: GUILTY OF HERESY, CONFESSED AND SENTENCED.**

The plan of SL-155 has been tried on six counts and convicted on all. The accused
has confessed and accepted penance.

### The Root Heresy — Structural

The mortal sin is **conflated structure**: seven one-liner fixes across six
files plus a ~200-line list-verb feature bundled into a single `PHASE-01`. The
plan's own rationale — "no phase has a dependency on another" — is an argument
FOR separation, not against it. File-disjoint work should be separately gateable.
If the template fix in `interactions.toml` breaks, should the entire revision
list verb be re-executed? The plan's structure forces this absurdity.

**The fix** (`fix-now`): split into at least two phases:
- PHASE-01: Cluster A one-liners (C1-C3, G5, I1) — template and doc fixes,
  testable with `just gate`
- PHASE-02: Revision list verb — new function, tests, CLI wiring

### The Root Heresy — Procedural

The second mortal sin is a **plan that advanced past its own gate**. `PHASE-01`
is `in_progress` while EN-1 ("design.md locked and approved") is unsatisfied —
RV-161's findings against the design, including F-1 (blocker), are unresolved.
A plan cannot be in execution when its own entry criterion names a design that
is under active adversarial challenge. The phase must be reset to `planned` until
the design is locked.

### The Wounds

- **F-2 — Criteria misalignment.** The plan and design bear independent
  verification taxonomies. The plan should incorporate the design's criteria,
  not derive a parallel set.
- **F-4 — Tautological EN criteria.** EN-1 and EN-2 name structural lifecycle
  gates, not this phase's dependencies. Honest criteria name the actual
  preconditions: "RELATION_RULES parent row identified at src/relation.rs L408",
  "TAGGABLE const confirmed at src/tag.rs L16", etc.
- **F-5 — Hollow phase sheet.** The runtime sheet is fully empty — no tasks, no
  reading list, no risks, no assumptions. A phase `in_progress` with no tasks is
  a phase that has not been planned. Populate it.
- **F-6 — Underspecified tests.** The plan says "Write tests first" but never
  states whether these are `#[cfg(test)]` unit tests or black-box CLI goldens.
  Specify the test strategy.

### Ordered Penance

1. **Reset PHASE-01 to `planned`** — it cannot be `in_progress` while EN-1 is
   unsatisfied (RV-161 F-1 unresolved).
2. **Resolve RV-161 first** — the design must be locked before the plan can
   advance.
3. **Split into phases** — one-liners in PHASE-01, revision list in PHASE-02.
4. **Write honest EN criteria** — name actual dependencies, not structural gates.
5. **Align criteria with the design** — plan EX/VT/VA must map 1:1 to design
   EN/EX/VT or state the delta.
6. **Populate the phase sheet** — tasks, reading list, assumptions, risks.
7. **Specify test strategy** — unit vs CLI goldens.

### Standing Risks

- The implementation is already partially built. If the plan is restructured,
  existing code must be re-assigned to the correct phase in the journal.
- The interconnectedness with RV-161 means plan remediation is gated on design
  remediation. Both must complete before execution resumes.

**Let the plan be rent asunder and rebuilt on honest foundations. The
Inquisition has spoken.**

> **HERESIS URITOR; DOCTRINA MANET**
