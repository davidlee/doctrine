# Notes SL-230: Memory body-write verbs and corpus-aware verify gate

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-26 · design (DEC-020 applied; RV-307 fully disposed) · 8a625cb5

### Produced

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

- RV-307 — **`await=raiser`**, 35 findings, all disposed. Round 8 is the raiser's
  confirmatory pass on the DEC-020 remediation.
- QUE-175 / **OQ-2** — still open as a question; body corrected (F-34) and
  re-verified at round 7. Gates IMP-317.
- **OQ-6 (new)** — should non-contribution ever refuse, and on which entries?
  Deferred out of the slice by DEC-020. The answer that survives cloning is a
  *declared* boundary, not a derived one — a schema change. Scope with QUE-173
  (digest) and IMP-318 (persist attested coverage); all three are the same
  change, each making an attestation say more than a sha.
- REV-034 — governance amendment reserved for close.
- R5 — masters remain outside every invalidation path.
- R7 — `validate` and `retrieve::git_facts` retain raw historical scope seams.
  Untouched by DEC-020 and slightly sharpened by it: with `verify` no longer
  refusing on non-contribution, the two weaker notions of scoped drift are the
  only signal for a moved source path.
- R8 — **widened by DEC-020**: the stamp does not record its covered surface,
  and the residual now spans all non-contribution rather than only the
  never-tracked. Routed as IMP-318.

### Decision awaiting the user

Seven rounds, 35 findings, nine blockers, and the finding rate had not decayed
through round 7. DEC-020 removed the surface that generated F-21/F-25/F-31, so
round 8 should be markedly quieter — **hypothesis, not promise**. If it returns
near-clean, lock. If it produces another decision-level refutation, the honest
read is that the claim-probe design is past the point where more rounds buy
correctness, and the alternative is a deliberate lock carrying named residual
risk into `/plan`. That is the user's call; surface it with the result rather
than opening round 9.
