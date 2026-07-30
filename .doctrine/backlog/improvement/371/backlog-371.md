# IMP-371: Mandated evidence in the disposable tier can vanish

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

A verification criterion can mandate that its evidence be recorded *only* in a
tier that is designed to be thrown away.

## What surfaced it

SL-233 PHASE-04 `VA-7(3)` requires a demonstration's failing output be "recorded
in the phase sheet". Phase sheets are runtime state — gitignored, disposable,
`rm -rf`-able by construction (ADR-003 storage tiers). So a criterion can be
fully and honestly discharged and its evidence still disappear, leaving no
committed artefact. A later auditor cannot then distinguish *never demonstrated*
from *demonstrated, sheet discarded* — and the safe reading of that ambiguity is
the pessimistic one, so the work gets re-done or the claim gets doubted.

RV-321's raiser hit exactly this. It verified F-1, F-2 and F-4 from the tree, but
had to **contest F-3** even though F-3's fix had landed and worked: the mandated
output was absent from the sheet. Behaviour correct, record not. The responder's
remedy was to carry the same verbatim output in F-3's *response* — which is
authored and committed — so the ledger became self-sufficient. That remedy was
improvised at the point of need, which is the signal worth filing: it is the
right move and nothing names it.

## Not a defect in any one phase's authoring

`VA-7(3)` was written by this slice's own owner ruling, and the sheet convention
it names is the project's normal one — phase sheets routinely carry RED evidence
and that is a good habit. The gap is that the habit has no durability story.

## Candidate shapes — none chosen

1. A convention that a VA criterion mandating *recorded* evidence names an
   authored sink — the RV response, the reconcile brief, `notes.md` `## Harvest`
   — and treats the sheet as the working copy rather than the record.
2. A `/harvest` step at phase close that promotes the sheet's `## Findings`
   evidence into `notes.md`, so closing a phase is what makes its evidence
   durable.
3. Leave it, and declare the RV response the durable sink by default.

(3) is close to what already happens ad hoc; its value would be making the
choice deliberate rather than rediscovered per slice. (2) is the only one that
covers evidence with no RV finding attached to it.

## Related

- RV-321 F-3 — the concrete instance, and the response that carries the
  work-around.
- SL-233 PHASE-04 `VA-7(3)` — the criterion whose two legs came apart.
