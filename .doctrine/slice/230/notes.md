# Notes SL-230: Memory body-write verbs and corpus-aware verify gate

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-26 · design (RV-307 round 7 raised; DEC-020 taken) · 171a482a

### Produced

- RV-307 — round-7 raiser adjudication accepted F-19/F-20/F-27/F-29/F-30,
  returned F-6/F-22/F-24/F-25/F-26/F-28, and raised F-31–F-35. All five new
  charges reproduced by the responder. F-34/F-35 disposed; F-31/F-32/F-33 and
  the six contests are **undisposed pending DEC-020's application**.
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

- **DEC-020 not yet applied to the design.** D10, § 5.2's ordered algorithm,
  § 5.4, § 7, § 9's test matrix and `slice-230.md` all still describe the
  round-6 four-class cut. This is the next agent's whole task.
- RV-307 — `await=responder`, 35 findings. F-31/F-33 (blockers) and F-32 plus
  the six contests are undisposed; their responses depend on DEC-020's
  application, which is why they were held.
- **F-32 survives DEC-020** — the wildcard-free prefix is split at the first
  wildcard *character* rather than the last path separator before it, so
  `foo*/bar` misses tracked `foobar/bar`. Independent of the classification
  question; must still be fixed.
- **F-33's fix direction** — delete the strong reading (`design.md:606`,
  `slice-230.md`), keep the weak one. Not the reverse.
- QUE-175 — still open as a question; body corrected.
- QUE-173 — digest-based invalidation question. Scope with IMP-318 and DEC-020's
  deferred declared-boundary question; all three are the same schema change.
- REV-034 — governance amendment reserved for close.
- R5 — masters remain outside every invalidation path.
- R7 — `validate` and `retrieve::git_facts` retain raw historical scope seams.
- R8 — the stamp does not persist the covered surface; routed as IMP-318.
