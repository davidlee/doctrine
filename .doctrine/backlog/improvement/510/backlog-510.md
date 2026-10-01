# IMP-510: Plan-sheet claims carry their derivation, re-run at phase entry

## What

Phase sheets, plan criteria and worker briefs state facts about the tree — counts,
file/touch sets, "this compiles", signatures, line locations — as prose fixed at
authoring time, with no recorded derivation. Earlier phases move the tree; nothing
re-checks the claims; workers pay at execution (a gate cycle, a probe round, or a
re-dispatch). EVD-031 measures this as the largest friction cluster (~100 of 758
observations: undercounts, claims from reading not compiling, citation rot,
artefact drift).

## Direction

Each factual claim in a sheet carries the command that produced it (the
"denominator"), and phase entry re-runs those commands, reporting any that moved.
One mechanism for the undercount, rot and drift classes. Bears on RFC-027 P1
(predictions authoritative but provisional) and Stages 3–4.

## Not yet

This is a design question, not a ready change: route through `/slice` and
`/design` (or settle it inside RFC-027 first). Cheap prerequisites that shrink
the problem: IMP-260 (read one phase's criteria), ISS-226 (verify-vt false
evidence), IMP-344 (cite symbols, not line numbers), IMP-365 (pre-run VA greps).
