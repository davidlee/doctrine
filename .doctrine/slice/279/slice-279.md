# RV and REC as relation targets

## Context

A backlog item born from a review finding cannot record the review that raised
it: `doctrine link ISS-NNN references RV-NNN --role originates_from` is refused,
because `references` targets exclude RV and REC (`IMP-480`). Provenance survives
only as prose.

RFC-032 (review ledger effectiveness) decision `D14`
(`.doctrine/rfc/032/decision-frontier.md` § D14) admits RV as a `references`
target once its status is reachable from the engine tier. Since `SL-268`'s `D4`
split, derivation lives in `review_ledger` (engine), so it is. REC rides the same
change.

`D14` was scheduled in RFC-032 slice 2, then carved out of `SL-278` (the read
surface) to keep that slice contained. DEC-318 (keep RV's `Unavailable` status
arm until `D14`) now names this slice as owner of the arm's retirement and the
DEC-233 amendment.

## Scope & Objectives

1. Admit `RV` and `REC` as `references` targets (role `originates_from`
   included) in `RELATION_RULES`.
2. `partition::status_class` reads RV status through the engine-tier ledger
   module; retire DEC-233's `Unavailable` arm for RV (DEC-318). `done` already
   means concluded and every finding terminal (RFC-032 `D2`), so a `needs` edge
   on an RV cannot become actionable mid-pass.
3. Amend DEC-233 to match.

## Non-Goals

- Finding-granular citation (`RV-N#F-3`) — rejected by `D14`.
- The review read surface — `SL-278`.

## Summary

## Follow-Ups
