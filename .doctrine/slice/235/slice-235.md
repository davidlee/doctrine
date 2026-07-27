# Evaluation kit for managed design runs

## Context

Minted and abandoned in the same session. Recorded rather than deleted so the
reasoning survives and the option is not re-proposed from scratch.

The proposal: lift SL-233's phase 9 — the deterministic evaluation fixture,
moderator protocol, rubric, evidence collectors, and assertions — out of SL-233
into its own slice, sequenced `after` it, on the argument that the instrument
measuring a piece of work should not be authored in the same breath as the work.

## Why it was abandoned

**CHR-049 already provides the separation.** It is an open chore carrying
`originates_from` SL-233, and its intent is to run the live human-in-the-loop
measurement exercise *defined by the slice*, immediately after SL-233 closes and
its skill/prompt changes are installed. Its entry checks explicitly refuse
authored-or-worktree-only asset presence. So the part of evaluation that
genuinely must follow the landing already follows it, in a different session,
against installed bytes, with a human moderator.

What a separate slice would have added is moving the *authoring* of the kit out
too — and the kit is authored fixtures and assertions that do not need to be
post-close. The cost was concrete:

- reopening a design locked after five adversarial rounds on RV-315;
- superseding accepted **DEC-079**, which names SL-233 as the deliverer of the
  deterministic evaluation materials;
- rewriting design §5.5's implementation home, §9.5 entire, and §10's
  reconciliation bullet;
- weakening SL-233's closure gate — and whether the design silently moved the
  scope's closure intent outside that gate was RV-315's fourth line of
  interrogation, the one F-5, F-9 and F-11 came from.

Paying that to obtain a separation that already existed was not a trade worth
making.

## If this is ever revisited

The live argument is independence of the measuring instrument, and it concerns
*who runs* the evaluation rather than who authors it. Before minting a slice for
it, check first whether CHR-049's protocol can carry the independence
requirement — a different moderator, a preserved pre-SL-233 baseline, or a
review of the kit by someone other than its author are all cheaper than a slice.

## Summary

Superseded by existing structure: SL-233 phase 9 authors the kit, CHR-049 runs
it post-close.
