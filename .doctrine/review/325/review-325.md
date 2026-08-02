# Review RV-325 — design of SL-233

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

### What is under review, and why this gate exists

`SL-233` PHASE-08 rewrites `plugins/doctrine/skills/design/SKILL.md` (214 lines /
10,178 bytes) from an eight-state workflow machine into a thin
activation/recovery adapter. The entrance criterion `EN-2` requires a sketch
answering six questions (a)–(f), **and a terminal RV ledger raised by a reviewer
who is not its author**.

That second clause is the point. RV-315 finding F-17 established that **no
specification governs a skill body** — PRD-003 disclaims what a skill *says* and
SPEC-010 treats a skill as opaque payload. There is no spec to check this against
and no test that can fail. This ledger is the only scrutiny the artifact gets.

### Pre-reading, in this order

1. **`.doctrine/slice/233/sketches/target-machine.d2`** — diagram C, **the
   primary decision record**. Carries D1–D13 with evidence citations, written as
   an onboarding doc for exactly this review. Renders with `d2 <file> out.svg`.
   **Where it and the sketch differ, the diagram governs.**
2. **`.doctrine/slice/233/sketches/thin-adapter.md`** — the sketch, ~700 lines.
   Its `## For the reviewer` section ranks six attack surfaces by cost-if-wrong.
3. **`DEC-103`** (`doctrine knowledge show DEC-103`) — the ruling that reshaped
   both, and attack surface #1.
4. `plan.toml` PHASE-08 — `grep -n 'id *= *"PHASE-08"'`. The file is
   **column-aligned**, so `rg 'id = "PHASE-'` matches nothing.
5. Context if needed: `sketches/workflow-as-prose.d2` (diagram A, the machine the
   current prose claims) and `sketches/surface-as-built.d2` (diagram B).
   **Diagram B is stale** — it predates PHASE-16, says "writer acts — six" where
   there are now seven (`discharge`), and its gap g1 is closed. Check B's age
   before treating a difference as a conflict.

### The governing claim to break

> The incumbent skill is not a workflow machine that needs porting. It is a
> **prose imitation of a machine that already exists in Rust**, and the rewrite
> is mostly deletion.

If that breaks, the phase's shape changes rather than its details.

### Lines of attack, ranked by the sketch

The sketch's own ranked list is in `## For the reviewer` and is not repeated
here. Its top three:

1. **DEC-103 itself** — *instruction is delivered at the point of effect; prose
   is a failure to locate a delivery moment*. Ruled after the other twelve
   dispositions and it reversed D10, re-warranted D8, and collapsed the residue
   from three items to two. The sketch names three attacks on it, of which the
   sharpest is **is the rule too strong** — it licenses hanging an obligation on
   every edge where it fires, and the edge-3 runbook now carries fifteen content
   items plus four moved obligations. At what point do steps become noise an
   agent scrolls past, which is the incumbent's failure mode relocated into TOML?
2. **DEC-102 overclaims in the present tense** and the repair was ruled one way
   (D12, a third runbook) when a cheaper one existed (amend the record).
3. **`:111-113` is the sole genuine residue** and the whole "we found no
   mechanism" claim rests on it. The stage hymn is rendered every turn — if a
   per-message obligation can ride it, the residue is one item, not two.

### This sketch has already been self-checked — do not stop at what it admits

A self-check pass before this ledger found and repaired: a circular citation for
D5, a wrong line-range for state 8 that double-disposed retained content, three
conflicting guardrail censuses across two artefacts, stale Q1–Q4 labels, and two
deletions that dropped an obligation where it fires. All are disclosed in place.

**Two consequences for you.** First, the disclosures are evidence about method,
not absolution — the sketch itself observes that every one of those defects sat
in material no `EN-2` question interrogated, so *"what else is unnamed"* is a
live line. Second, **D5 has been verified and is not worth a turn**:
`can_advance` (`src/design_run/gate.rs:150-158`) is a nine-line literal
four-arm `matches!`, locked by an exhaustive 25-pair test
(`src/design_run/tests.rs:56-77`). The residual there is durability only — *is a
second forward edge ever wanted?*

### Out of scope — findings should say so rather than widen the gate

- **`EX-7`'s handover adapter.** All six `EN-2` questions are about the *design*
  skill. The handover convergence (DEC-058,
  `plugins/doctrine/skills/handover/SKILL.md`) is a declared design target of the
  phase but lands **unsketched and outside this gate**. Disclosed, not absorbed.
- **ISS-289** — PHASE-15 completed with an unmet `VT-2`. Audit judgement.
- **F12** — DEC-102 names IMP-372 by kind, not by id. Blocked on tree topology;
  fix at reconcile.
- **IMP-373 / IMP-374** — `set` mode and the authoring-rule third clause, both
  deliberately off this phase's critical path.

### One known defect left unrepaired, deliberately

DEC-102's `consequences` assert the `SKILL.md:98` / `CLAUDE.md` contradiction
*"is therefore fixed FOR THIS REPO"* in the present tense. It is not; `:98` is
verbatim unchanged until PHASE-08 lands. The repair was proposed and is not yet
ruled on. It was not silently amended, because editing an `accepted` decision
record to make a sketch look consistent is the wrong instinct. Raise it if you
think the tense matters more than the deferral.
