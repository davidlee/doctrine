# IMP-308: Allow pre-close trunk integration for dispatched slices

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

A dispatched slice's code lineage is *frozen* at its fork base (the `review/*` /
`phase/*` refs) by the standing "don't touch trunk until close" invariant, while
its authored truth (audit ledger, reconcile edits, harvest) matures on the moving
`edge`, and trunk advances independently as sibling slices close. By close time
these have drifted into a 3-way split (code on the frozen fork, reconcile on
`edge`, trunk with neither), and the correct close route
(`mem.pattern.dispatch.close-split-lineage-reconcile-on-edge`) has to be
reconstructed by hand — the status machine is lineage-blind and its "next" hint
points at the naive code-only path that silently strands the reconcile. Heavy
read-token burn per close; see SL-224 close and the RFC-011 case-notes entry
`[close; SL-224-split-lineage]`.

## Proposal

Relax the invariant: permit advancing trunk for a dispatched slice **before**
close — e.g. promote `edge → main` / integrate the code at conclude or post-audit
— so drift stays small and the close is a near-empty formality instead of a 3-way
reconciliation.

## Gotcha / constraint

Advancing `main` mid-flight can disturb **another concurrent close** that is
reading/advancing trunk during its own audit. So this needs an **optimistic lock /
advisory** — a "I'm closing, don't touch trunk" claim other agents can see and
respect — before it is safe to relax the invariant generally.

## Interim operating rule (until the lock exists)

Run **at most one audit-or-close at a time** across the repo. More than one
concurrent `> audit` / `close` risks racing trunk with no coordination primitive.

## Related

- `mem.pattern.dispatch.close-split-lineage-reconcile-on-edge` — the recovery
  route this would obviate; it already calls for a pre-close check diffing the
  admitted `close_target` tree against `edge` to flag the divergence in one
  command (a cheaper partial mitigation than full relaxation).
- RFC-011 — token-efficiency benchmark; the motivating cost signal.
- Motivating case: SL-224 close (2026-07-24).
