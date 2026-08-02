# Obligation: delegation

Hand one bounded obligation out; stay the sole writer.

- Delegation is proposal-only. The coordinator is the sole writer — a delegate
  proposes and never advances the run.
- An export names an obligation that must ALREADY exist. A node created by the
  same batch is not yet in the map, so create it first or the export is refused.
- One outstanding assignment per obligation. A second may be cut only once the
  first is settled, so that "who holds this obligation" always has one answer.
- A proposal is stale exactly when the obligation is no longer the one that was
  assigned. That is exact equality, not a judgement call — if the question moved,
  the answer is stale even if it still reads well.
- A proposal payload carrying a stage change, a declaration, evidence, an
  acceptance, an authored adoption or a traversal is refused, naming the key it
  objected to. Delegates contribute answers, not run-level writes.
