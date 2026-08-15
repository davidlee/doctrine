# ISS-359: Reviewing runbook clears on a cited governance target set

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

The governing-context contract cannot distinguish two acts:

- **"I derived the governance target set"** — the agent enumerated the ADRs,
  policies, standards and specs bearing on this design; and
- **"I cited the decision that recorded it"** — the agent pointed at a prior
  `DEC-NNN` in which someone else derived a set, at some earlier time, for some
  possibly narrower scope.

Both discharge the contract. So a design can clear the **reviewing** runbook while
still carrying an undercounted governance target set.

## Why it matters

The reviewing runbook is the last gate before a design locks. Its governance step
exists to catch exactly the failure of designing against rules you never read. A
citation is cheap and always available — there is nearly always *some* prior
decision to point at — so the cheap path and the correct path have identical
observable outcomes, and the contract selects for the cheap one.

The consequence is silent: the run locks green with a target set that was never
re-derived against the current corpus, and nothing downstream re-checks.

Compounds with `ISS-357` (*governance-confirmed marker is snapshot-only*): one lets
a stale set through, the other reports it as current.

## Evidence

- `019ffa30-4e0d` — reviewing runbook can clear with a governance target set nobody re-derived

## Shape of a fix

The discriminator has to be **structural, not attested**. Options, cheapest first:

1. **Require the set, not the citation** — the discharge payload carries the
   enumerated target ids; a cited `DEC` may seed it but does not substitute for it.
2. **Re-derive and diff** — the engine computes the target set itself and refuses a
   discharge whose declared set is a strict subset.
3. **Age the citation** — a cited derivation older than the run's governance
   snapshot is inadmissible.

(2) is the only one that closes it without relying on agent honesty, and is the
same shape as `ISS-357`'s drift check — worth costing them together.

## References

- `DEC-123` — contract structure rides a const table, not the prose asset
- `IMP-373` — runbook set mode: coverage-set admission and its rendering bound
- `ISS-357` — governance-confirmed marker is snapshot-only
