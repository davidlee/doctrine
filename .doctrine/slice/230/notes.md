# Notes SL-230: Memory body-write verbs and corpus-aware verify gate

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-26 · design (round 8 disposed; slice split by DEC-027) · ca4bb4cd

### Produced

- **DEC-027** — the corpus-aware verify gate leaves SL-230 into **SL-232**. User
  ruling at RV-307 round 8. The governing record for the split boundary, the
  accepted R4 tradeoff, and the RV-307 disposition policy.
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

### Open

**This slice.** Nothing blocking. Design narrowed to the body-write seam and
attestation invalidation; scope doc reconciled; RV-307 fully disposed.

- **Design is NOT locked.** It was locked-pending-review until round 8; the split
  rewrote it, so the retained half needs a confirming pass before `/plan`. The 7
  findings on this half (F-3, F-8, F-9, F-10, F-12, F-14, F-17) are all verified
  and quiet since round 4, so the pass is a coherence check, not a re-review.
- **R4 runs unmitigated** until SL-232 lands. D4's affordability argument depended
  on the gate relaxation; the split removed that support. Accepted by DEC-027 —
  friction, not incorrectness — and the reason to sequence SL-232 next.
- New ids this slice: I10, I11, I12, E14, OQ-7, T40. Moved ids are **not** reused.
  (**OQ-4 is not new** — it pre-dates the split verbatim, so it is reviewed text,
  not reassembly. Next free: I13, E15, T41.)
- OQ-4 — when do other kinds adopt `write_body`? Not here.

**Moved to SL-232 (do not track here).** OQ-2/3/5/6, D3/D9/D10/D11, I1-I4/I6-I9,
E2/E4-E9/E11-E13, R6/R7/R8, REV-034, QUE-173, QUE-175, IMP-317, IMP-318, and
RV-307 F-14/F-25/F-26/F-32/F-36/F-37/F-38 + F-39's first limb.

**F-14 added to that list by the confirming pass.** DEC-027 and this harvest both
counted it among the seven body-write findings, but it lands on I6 and T24 — both
SL-232's — so it was an orphan: named local by SL-230 and absent from SL-232's
inherited set. F-7 (→ R5) is the seventh local finding. Count unchanged.

### Next

1. Confirming pass over SL-230's narrowed `design.md`, then `slice status 230 plan`.
2. `/design` on SL-232, starting from F-37 and F-36. Enumerate-then-probe before
   asserting any replacement rule — that is § 8 R-A and it is the whole lesson of
   the eight rounds behind the inherited text.
