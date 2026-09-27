# Review RV-398 — reconciliation of SL-268

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Conformance audit of SL-268 (Review ledger v2), all ten phases, after capsule c
generation 20 was merged into edge at `39212685a`. Evidence was gathered on the
adopted worktree `capsule/SL-268/c20` at `017802240`, which is the merged code.
The review verbs refuse a capsule worktree as a fork, so the ledger runs on edge.

**Lines of attack.**
- `doctrine check gate` green on the landed tree (build before validate).
- `slice verify-vt 268`: every plan VT keyword present.
- `slice conformance 268`: each undeclared / undelivered path dispositioned.
- Design sec-12 to sec-5 against code: `review_ledger` imports no command
  module; one `gates_as_blocker`; counters seed-plus-count; `resolve_prose` and
  the second-`-` refusal; `Target::parse`; `PassFacts.defects` printed by the
  design command; `BlockerRef.reason`; prime degrade and skip.
- Test flips: every changed pre-existing assertion is named (design sec-8).
- Design divergences recorded only in runtime phase sheets.
- Governance deliverables: SPEC-032 anchors, REV-064 drafted, RV-397 concluded.

**Invariants.** Behaviour-preservation for the split (PHASE-01/02); no
unnamed assertion change; the tech spec describes what landed (DEC-321).

## Synthesis

SL-268 conforms. `doctrine check gate` is green on the landed code (9 015
tests, 0 failed), `slice verify-vt 268` passes every VT in all ten phases, and
the design points spot-checked against code hold: `review_ledger` reaches only
leaf and engine modules; one `gates_as_blocker` feeds all three blocker
predicates; counters are seed plus count; prose resolution and the second-`-`
refusal live in `input`; `Target::parse` owns `@PHASE-NN`; the design command
prints `PassFacts.defects`; the close gate carries `BlockerRef.reason`. The
golden-first split held its behaviour-preservation gate, and every later change
to a pre-existing assertion is named in its phase sheet.

The findings are bookkeeping, not defects. Two design-target selectors
anticipated edits that turned out unnecessary (`F-1`). The undeclared paths are
comment and fixture follow-ons of the split and D2, or governance-phase outputs
(`F-2`). Four implementation choices refined the design and live only in
disposable phase sheets (`F-3`); SPEC-032 already describes them, so design.md
catches up at reconcile. ADR-007 and SPEC-003 still describe v1 until REV-064
is applied (`F-4`). The audit itself hit a tooling gap: the capsule's worktree
cannot run review verbs (`F-5`, ISS-494).

**Standing risks.** Legacy RVs across the corpus now read `active` until
concluded (design R2, accepted). Catalog-backed reads emit, but do not yet show,
vocabulary defects (ISS-492). Hand-written turn rows that look valid are
undetectable.

**Tradeoffs accepted.** Journal growth (R6); `contest --note` now required
(R5), with shipped callers updated by PHASE-09 and RV-397.

## Reconciliation Brief

### Per-slice (direct edit)
- `F-1`: `doctrine slice selector rm` the design-target selectors
  `src/commands/show.rs` and `plugins/doctrine/skills/walkthrough/SKILL.md`
  (load-bearing, what `slice conformance` reads); mirror by dropping them from
  design.md sec-7's table.
- `F-3`: design.md sec-3 — `FindingState { status: Vocab<FindingStatus> }`
  replaces `Option` plus `raw`; the `warnings` field is omitted when empty.
  sec-5 — `Primed` carries a `skipped` list; a slice whose selectors are all
  skipped is not degraded; fifo, socket and device literals are excluded with
  directories and symlinks. sec-6 — IMP-479 closed `wont-do`.

### Governance/spec (REV)
- `F-4`: REV-064 — approve and apply (ADR-007 D-C5, D-C8, D-C10 and knock-on
  wording; SPEC-003 container inventory gains SPEC-029 and SPEC-032).
- `F-4`: SPEC-032 — status `draft` → `active` (authored directly, REV-035
  precedent; anchors resolve).
