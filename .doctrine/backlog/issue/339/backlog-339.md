# ISS-339: The conformance suite has never run off-jail

Nobody has run the shipped conformance suite on a real host.

## What was actually confirmed off-jail

The off-jail work ran **spikes** — shell reproductions of the backend's argv
shape. So what is confirmed on a real host is the *mechanism* rows 13 and 14 rest
on, not those rows executing there. `VA-1` asked for the credential measurements
and got them; this is not a shortfall against it.

## Why now

It is unusually cheap. `PHASE-10` `T13` made `doctrine-control backend verify`
exit **0** in-jail with all nineteen rows `Proven`, so the confirmation is a
single command on the owner's own shell.

And it is the only thing that would close `sec-9` residual 3's neighbourhood by
**measurement** rather than by argument.

## The audit's own evidence that this is not theoretical

`RV-352` ran the suite from a jail built without this slice's `flake.nix`
addition. `setsid` did not resolve, seven `conformance::tests` convicted, and
`backend verify` returned `not-admitted` with `Property(ProcessTreeTeardown)`
`Unproven`. The guard behaved correctly — but the episode is a live demonstration
that the suite's host preconditions are real, undocumented outside `flake.nix`,
and unexercised anywhere but this one jail.

## Relationship to `EX-14` in CI

Closing this by measurement does not settle `EX-14`'s CI question (`RV-352`
`F-8`) — that is a criterion-level decision about what a runner unable to satisfy
the backend should report. But an off-jail run is the cheapest way to learn what
the suite's real host requirements are before that decision is taken.

## References

`SL-248` `notes.md` § *Owed* item 175(b) · `sec-9` residual 3 · `VA-1` ·
`RV-352` `F-8` · sibling of `ISS-338`
