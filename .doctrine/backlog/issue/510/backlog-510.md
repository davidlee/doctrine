# ISS-510: Resolving an inquiry node takes repeated refusals to find the disposal shape

## What

Resolving (disposing) an inquiry node in a managed design run consistently takes about
three refused `design apply` submissions before the agent finds the accepted shape
(the `cp-` checkpoint disposal form). Neither the turn envelope nor the refusals show
the shape up front.

## Fix sketch

Print a worked disposal payload for the node's current state in the turn envelope (or
in the refusal), naming the accepted forms. Related: IMP-445 (name accepted tokens on a
refused change event).

## Evidence

5 observations: `019fd6c8` (2026-08-06), `01a0a42c` (09-15), `01a0d09d` (09-23, SL-261),
`01a0d28d` (09-24, SL-262), `01a0d891` (09-25, SL-266) — one per design run, every run.
