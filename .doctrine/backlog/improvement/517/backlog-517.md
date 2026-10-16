# IMP-517: review dispose warns when a severe finding gets no route

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

`doctrine review dispose` accepts a severe (BLOCKER/MAJOR) finding with no
`--route` and says nothing. The design-run lock gate later refuses with "severe
findings carry no route". By then the raiser has usually verified the finding,
`review amend` refuses verified findings, and the only backfill is raiser
reopen → re-dispose → re-verify for each one.

Seen on SL-281 / RV-418 (2026-10-16): 13 findings disposed `fix-now` without a
route, all verified over four codex rounds, then the lock refused. The review
pass had to be disposed `waived` to get past the gate.

## Direction

- `dispose` warns, or refuses, when the finding's severity requires a route
  and none is given or already set. The warning names the route vocabulary.
  Refusing is probably right on ledgers whose consumer gate requires routes.
- Alternatively, let `amend` (responder) set a route on a verified finding
  without reopening it, since a route is metadata, not substance.

## Related

- RV-418, SL-281.
