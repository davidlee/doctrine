# ISS-512: User-authority acceptance in reviewing also records design-accepted

## What

In a design run at the `reviewing` stage, any submission carrying a top-level
user-authority acceptance — on a section edit, a checkpoint disposal, anything — also
records `design-accepted`. An acceptance meant for one act silently becomes acceptance
of the whole design.

## Fix sketch

Scope the acceptance to the act it accompanies; record `design-accepted` only from an
explicit design-acceptance act. At minimum, report the extra recorded act in the apply
output.

## Evidence

4 observations, 2026-09-26 → 2026-09-29: `01a0dd21`, `01a0e2cb`, `01a0ec2a`, `01a0ec65`.
Correctness, not just friction: it records assent the user did not give to that
proposition.
