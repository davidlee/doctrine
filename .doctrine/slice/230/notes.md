# Notes SL-230: Memory body-write verbs and corpus-aware verify gate

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-26 · plan (design locked after the confirming pass; plan authored, 0/6 phases) · 68bf7f21

### Produced

- **plan.toml / plan.md** — six phases, split by seam not by verb (`record` rides
  the scaffold fileset, `edit` rides the new `write_body`). Phase sheets
  materialised. Awaiting user approval before `ready`.
- **DEC-027** — the corpus-aware verify gate leaves SL-230 into **SL-232**. User
  ruling at RV-307 round 8. The governing record for the split boundary, the
  accepted R4 tradeoff, and the RV-307 disposition policy. **Its boundary table
  and finding sort were corrected** by the confirming pass (68bf7f21) — the table
  contradicted its own prose; the decision itself is untouched.
- **SL-232** — corpus-aware memory verify gate. Scope doc + inherited design
  (gate half of this slice's design.md, carried verbatim so eight rounds of
  review are not re-derived). Status `proposed`, design NOT locked, two open
  blockers (F-36, F-37).
- **ISS-247** — no verb removes a `needs` edge; re-pointing REV-034 needed a
  sanctioned TOML hand-edit.
- IMP-317 retitled (F-34/F-39) — the body was correct since round 7, the title
  still advertised the rejected shared-constructor model.
- RV-307 — round-7 raiser adjudication accepted F-19/F-20/F-27/F-29/F-30,
  returned F-6/F-22/F-24/F-25/F-26/F-28, and raised F-31–F-35. All five new
  charges reproduced by the responder. **All 35 findings now disposed
  `fix-now`; `await=raiser`.** Corrects a prior harvest: F-34/F-35 were recorded
  here as disposed when their *fixes* had landed but their ledger entries had
  not — they carried no disposition and no response until 8a625cb5.
- **DEC-020** — non-contribution classification leaves SL-230. User ruling at
  round 7. The governing record for the D10 shrink; read it before editing D10.
- REV-034 — SPEC-007 + REQ-147 amendment. Proposed, applied at close.
- IMP-317 — give `validate` and `retrieve` a history-stable scope dataflow.
- IMP-318 — persist attested coverage on the verification stamp.
- QUE-175 — retrieve-side scope drift question; body corrected per F-34.
- EVD-001, QUE-173 — from the pre-external design rounds.

### Learned

- F-31 — `git rev-list --all` is stable only over the current ref set, not over
  clones or branch deletion.
- F-32 — a textual wildcard-free prefix is not necessarily a resolvable path
  prefix; non-resolving outside pathspecs also abort the history probe.
- F-33 — a weak stamp contract and a strong scope contract cannot coexist as
  separate normative readings.
- F-34 — routing records are part of the integration sweep, not commentary
  outside the design.
- F-35 — pointer-only history must contain pointers only.
- **the dominant cost driver, named** — a property of a tool (stable, total,
  deterministic) is a *claim needing a falsifier*, not a premise. Measuring that
  a discriminator **works** is not evidence it is **stable**. Two of seven rounds
  went on successive locally-varying instruments for one class boundary
  (F-25 → F-31). Recorded in `case-notes.md`; a `/record-memory` candidate, since
  it is not memory-specific.
- **run the query** — D5's defect (below) survived eight rounds because § 10's
  live-data check read a count of 3 as confirmation the plumbing worked; the 3 was
  partly the stamp commit itself. F-17/F-23's defect, third instance, in the
  document that names it twice. When a design states a concrete query, execute it
  against the real corpus. Same `/record-memory` candidate as the entry above —
  both are one lesson at different altitudes.
- **mechanical narrowing damages joints, not blocks** — the reassembly's defects
  were all at seams: a retained paragraph whose justification was rewritten to be
  local and now cited a verb that had left; a routing notice naming a section that
  exists in both documents; an invariant pinned to the nearest-looking test.
  Cheapest instrument by far was `git show <narrowing-commit>^:<path>` — it settled
  the invented-id question, E10's non-existence, and step-numbering provenance in
  one read.
- **no kind has a prose-write verb, and `knowledge` has no `edit` at all** —
  correcting DEC-027 needed a raw-file write of `record-027.md`. This slice's own
  § 2 claim, paid for in the session that authored it. Bears on OQ-4: adoption for
  knowledge is a **new verb**, not a new flag, so D1's "adoption is a caller
  change" understates it there. In `case-notes.md`.

### Open

**This slice.** Design locked, plan authored, nothing blocking. Confirming pass
done (10 mechanical defects + 3 substantive on D5); RV-307 fully disposed.

- **Plan awaits user approval** before `slice status 230 ready` → `/phase-plan`
  PHASE-01. No code without an approved plan.
- **PHASE-03 and PHASE-04 must not be separated.** At PHASE-03's HEAD
  `edit --body` works and the attestation does not clear — strictly worse than
  today, since the footgun currently needs a forbidden raw-file write and after 03
  needs one supported command (§ 1). No intermediate HEAD between them is landable.
  Recorded in `plan.md`; the phase boundary reads innocuous and the consequence
  does not.
- **D5 = `memory.md`, not the item directory** (user ruling; I13, T42). The
  directory form reported drift on 30/30 anchored memories because `verify` stamps
  `verified_sha` and the stamp's own commit touches the directory being measured.
  Body-file form: 11/30. T42 is the falsifier — **a 30/30 result at any point means
  the pathspec regressed.**
- **DEC-027 is still status `proposed`** despite having been taken and executed.
  Left deliberately; settle it or leave it, but do not assume.
- **R4 runs unmitigated** until SL-232 lands. D4's affordability argument depended
  on the gate relaxation; the split removed that support. Accepted by DEC-027 —
  friction, not incorrectness — and the reason to sequence SL-232 next.
- New ids this slice: I10, I11, I12, I13, E14, OQ-7, T40, T41, T42. Moved ids are
  **not** reused. (**OQ-4 is not new** — it pre-dates the split verbatim, so it is
  reviewed text, not reassembly. Next free: I14, E15, T43.)
- OQ-4 — when do other kinds adopt `write_body`? Not here.

**Moved to SL-232 (do not track here).** OQ-2/3/5/6, D3/D9/D10/D11, I1-I4/I6-I9,
E2/E4-E9/E11-E13, R6/R7/R8, REV-034, QUE-173, QUE-175, IMP-317, IMP-318, and
RV-307 F-14/F-25/F-26/F-32/F-36/F-37/F-38 + F-39's first limb.

**F-14 added to that list by the confirming pass.** DEC-027 and this harvest both
counted it among the seven body-write findings, but it lands on I6 and T24 — both
SL-232's — so it was an orphan: named local by SL-230 and absent from SL-232's
inherited set. F-7 (→ R5) is the seventh local finding. Count unchanged.

### Next

1. Readiness check over design / plan / selectors / phase gates before execution
   (a fresh pass; the pass that produced them should not also clear them).
2. On approval: `slice status 230 ready` → `/phase-plan` PHASE-01 → `/execute`.
3. `/design` on SL-232, starting from F-37 and F-36 — **and F-14**, which this
   slice handed over. Enumerate-then-probe before asserting any replacement rule:
   that is § 8 R-A and the whole lesson of the eight rounds behind the inherited
   text.
