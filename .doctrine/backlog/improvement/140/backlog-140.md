# IMP-140: Echo consistency for needs and backlog tag

## Source

IMP-133 UX review, first pass (F-7, F-13).
See `.doctrine/backlog/improvement/133/ux-review-findings.md`.

## Problem (F-7)

`needs`, `backlog needs`, and `backlog tag` don't distinguish initial
write from idempotent re-run. A user re-running `needs` gets the same
output as the first time. Compare to `link`/`unlink` which set the gold
standard: "already linked" / "not linked".

## Problem (F-13)

`backlog needs ISS-007 SL-132` echoes `ISS-007 needs SL-132` (canonical
form). Top-level `needs SL-060 SL-047` echoes `SL-060 needs SL-047`
(raw input). Both work but the inconsistency is a papercut.

## Scope

- `needs`: echo "already needs" on idempotent re-run
- `backlog needs`: echo "already needs" on idempotent re-run
- `backlog tag`: echo "tags unchanged" on no-op
- Unify canonical-id echo form across backlog and top-level paths
