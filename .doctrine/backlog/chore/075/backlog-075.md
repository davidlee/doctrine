# CHR-075: Five memories misdescribe `design show` after SL-246 PHASE-05 reclaimed it

> **Closed as a duplicate of `CHR-073`** at the `SL-246` audit. `CHR-073`
> (*Re-attest five memories that document `design show` as the envelope read*)
> was minted first, at `RV-370` `F-16`, and names the same five uids. The three
> things this item added have been folded into `CHR-073`'s body. Act on
> `CHR-073`; this row is kept only so the id is not reused.

SL-246 PHASE-05 moved `doctrine design show`'s **default** from the design run's
turn envelope to the **design document**. The envelope survives under
`--format prompt|json|status`; only the bare verb changed meaning.

Five memories that SL-246's design §5.6 routed to a memory review are therefore
no longer merely dated — they are **actively wrong**. At least one instructs
agents to read design-run state with `doctrine design show`, which as of
`7be7d1da4` renders the document instead.

## Why this is urgent rather than tidy

A stale memory is a mild cost: an agent reads it, finds it unhelpful, moves on. A
**wrong** memory is a trap — it is retrieved precisely when an agent does not
already know the answer, and it will send them to a verb that now does something
else and returns plausible-looking output. The corpus's whole value is that an
agent can trust it instead of rediscovering, so a confidently wrong row costs more
than its absence.

This compounds with `ISS-465` (`memory search` ranking too poor to compete with raw
grep): agents already bypass the sanctioned read path, so a bad row is less likely
to be noticed and corrected in passing.

## What to do

Route `/reviewing-memory` over the five rows §5.6 names. For each: correct the verb
to `doctrine design show --format prompt` where run state is meant, leave it alone
where the document is meant, and re-attest. Deliberately NOT done inside SL-246 —
memory-corpus review is its own routed stage, and the capsule-driver seat halts at
phase completion.

Surfaced by the PHASE-05/06 orchestrator during SL-246 (capsule-driver).
Related: `ISS-465`, and SL-246's `R6` (stale skill path) which is a reconcile input.
