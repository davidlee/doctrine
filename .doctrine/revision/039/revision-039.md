# REV REV-039 — reconcile SL-228

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

Single governance item from SL-228's reconciliation brief (RV-312 finding F-5).
The slice's D10 leans on REQ-385 (SPEC-021 `FR-009`) for a claim that its own
benchmark refuted. VH-1 accepted that verdict on 2026-07-27, so this is settled
evidence, not a contested reading — the counter-examples below are cited, not
re-derived, and VH-1 is not reopened.

**The claim splits in two, and only one half survives.**

*Survives — positional guidance.* Every funnel verb is legality-gated on
position, refuses out-of-order execution, and names the expected next verb. This
is proven at the strongest available bar: a memory-blind orchestrator, with no
access to the corpus and no rescue, drove a full funnel and a crash recovery
unaided. `next` earns its forcing-function claim here. This half stays normative
and unqualified.

*Does not survive — prescription completeness.* The stronger reading — that a
refusal's text **is**, by itself, a sufficient recovery procedure — is false as
an unqualified present-tense property. Four counter-examples:

1. **ISS-250.** `dispatch_reap`'s not-landed refusal prescribes `dispatch_reap` —
   the verb that produced it. The only non-circular advice left was the CLI
   override pair; the blind subject took it, and the commit the guard had just
   protected was destroyed. The guard fired *correctly* and its own remedy text
   walked the operator around it.
2. The benchmark's `stale-record` refusal carried an **empty `detail`**.
3. PHASE-09's `unprovable-fork` refusal, whose entire detail was a branch name.
4. **ISS-254** (filed at this audit): two completeness refusals with *different
   causes* both prescribing `record-delta` — for one of them, wrongly.

The mechanical root is located and deliberately not fixed here: **24 sites
construct a refusal with a structurally empty `detail`, 14 of them in
`src/mcp_server/dispatch.rs`.** The pattern across all four is that a refusal
names the verb its **author** was thinking about rather than the one the
**operator** needs.

**Why demote rather than delete.** The aspiration is sound and load-bearing for
the zero-rescue posture — a requirement that drops it would lose the thing the
design is actually reaching for. So it becomes a **stated goal with a named
verification vehicle** rather than a claimed property. The vehicle is
**IMP-321**, already structurally sequenced (`after: SL-228`,
`references(originates_from): SL-228`, related to ISS-249/250/251/253). The
demotion is what makes the goal testable: an unqualified claim nothing verifies
is not a requirement, it is a slogan.

### Row: modify REQ-385 (`FR-009`)

**Before** (statement, carried on the requirement's title):

> Every funnel verb is legality-gated on funnel position: it refuses out-of-order
> execution and names the expected next verb (report-and-halt with prescription);
> conclude refuses after skipped or failed verification.

**After** — drop the `with prescription` over-claim from the normative statement
and land the split as elaboration in `requirement-385.md` (the TOML statement
stays the primary normative text per the storage rule; prose elaborates, never
duplicates):

> Every funnel verb is legality-gated on funnel position: it refuses out-of-order
> execution and names the expected next verb (report-and-halt); conclude refuses
> after skipped or failed verification.

`requirement-385.md` gains a Statement note recording that the positional half is
normative and proven, that prescription *completeness* is a goal verified by
IMP-321, and the four counter-examples' provenance.

### Also landed under this REV (manual rows)

The strong form is also stated in two places inside SL-228's `design.md`, which
is a per-slice artefact rather than governance. They are corrected in the same
pass so the split is not contradicted three lines from where it is made:

- **D10's Refs column** — the gloss `FR-009 (a refusal's text **is** the recovery
  procedure)`.
- **§2's `IllegalTransition` doc-comment** — "surfaced VERBATIM by verbs and
  rendered by `next`: the refusal text IS the recovery procedure".

### Inspected and found inapplicable

The brief's second governance item asks that the same split be applied "wherever
the tech spec restates D10's strong form". **SPEC-021 never restates it.** Its
prose was searched for `refus` / `prescri` / `remedy` / `recovery`: the hits are
D2's arm-routing mismatch refusal and the coord-worktree/fork refusals, none of
which touch prescription sufficiency. SPEC-021's own D1–D7 are an unrelated
decision set — they are not SL-228's D1–D10. No change row is raised for it;
recording the search rather than manufacturing an edit.

The D7 golden artifact under `.doctrine/spec/tech/021/` is likewise untouched: it
is pinned by a passing golden test and is correct as authored.

### Out of scope, stated

REQ-387 (`FR-011`) stays `pending`. Subprocess-arm full gating was deferred by
design, not missed, and marking it satisfied to tidy the close rollup would be
the same over-claim this REV exists to remove.
