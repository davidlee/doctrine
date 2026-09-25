# Review RV-391 — design of SL-267

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

### Pass 2 — inquisition (2026-09-26, F-5..F-12)

A second, adversarial pass on design revision 28, held to `POL-002`, `ADR-005`,
`ADR-019`, `ADR-023`, `DEC-127` and the slice's own disciplines. Lines of
interrogation:

1. **Does the rule (`DEC-311`) admit every form the shipped corpus legitimately
   uses?** Checked: shipped memory keys, skill names.
2. **Are the audit's premises true against the corpus today?** Each `IMP-484` gap
   and each current-state claim re-read at source, not taken from the backlog prose.
3. **Is the reader test honest?** Does it run where a private id would *fail*
   to resolve, and does it cover every channel each swept path actually reaches?
4. **Does the design obey its own principles?** "One rule, one home" and "no third
   POL-002 rule" measured against what it mints.
5. **Is every site class covered?** Especially text addressed to maintainers,
   not client agents.

## Synthesis

### Pass 2 — inquisition (F-5..F-12)

**Judgement.** The design at run revision 28 was sound in shape but broke its own
law in three ways. Its grounding rule, as written, forbade the corpus's own
navigation: shipped memory keys and skill names (F-5). It built part of its
sufficiency axis on a gap that did not exist, because `IMP-484` said so and nobody
re-read the corpus (F-6). And its reader test would have passed in the one place
where a repo-private id still resolves, while covering only one of the several
channels `install/` actually reaches a client through (F-7, F-8). It also stated
"one rule, one home" while delivering five copies of that rule (F-9).

**Penance done** (revisions `9750594e4`, `c17d9229a`):

1. `DEC-311` and sec-5 widened to six admissible forms, the in-corpus relative
   path limited to targets installed next to the citing file; the rejection
   sentence struck (F-5).
2. The new ADR is the rule's single governance owner; `DEC-127` related as the
   `install/` precedent, not superseded (its shipping decision survives); the two
   local memories go to follow-up `CHR-081` (F-9).
3. The false observation gap swept from design sec-1/sec-6, `IMP-484`, and the
   scope doc (F-6).
4. sec-2 and sec-6 now agree on a per-channel table with one reach check each
   (F-8); the reader test runs in a scratch repo (F-7).
5. `prompt` reclassified as client-facing; sec-2 memory claim corrected; a
   maintainer-note class named (F-10..F-12).

**Open by protocol.** F-7 is routed `control`, an instrument route: it is
verified after `slice phases`, against the phase criterion its obligation
becomes, not in this pass. `/plan` must carry it as a criterion: in a scratch-repo
copy, one planted repo-private id per channel, each flagged by the read; a clean
read over a planted id fails.

**Form defect, tolerated.** The re-dispositions of F-6..F-9 carry the route token
but lost the vocab token (`route:owner-fix`, not `route:owner-fix fix-now`). Each
response records the action taken, so no finding was reopened for it; a later
reader should read these four as `fix-now`.

**Standing risks.** The drift gate is still deferred (`QUE-227`), so the corpus
can re-drift the moment this slice closes; the sweep's quality now rests entirely
on the scratch-repo control actually being built. The widened rule's form 6 (the
relative path) is the likeliest place for a future author to smuggle a
repo-private path back in; the gate slice should test it first.
