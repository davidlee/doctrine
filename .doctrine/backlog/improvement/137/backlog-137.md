# IMP-137: needs has no --remove/--prune counterpart

## Source

IMP-133 UX review, first pass (F-11).
See `.doctrine/backlog/improvement/133/ux-review-findings.md`.

## Problem

`after` supports `--remove` (remove one edge), `--prune` (drop dangling
edges), and `--rank`. `needs` has none of these. Removing a hard
prerequisite requires hand-editing the TOML.

The asymmetry may be intentional (hard deps should be deliberate), but
it's undocumented. A user who creates a wrong `needs` edge has no CLI
recovery path.

## Options

1. Add `--remove` to `needs` (like `after --remove`): validate target
   exists, remove one edge, echo confirmation
2. Add a note to `needs --help`: "Use `doctrine inspect <SRC>` to view
   existing edges; remove via TOML edit"
3. Both: add `--remove` for CLI recovery, keep cycle-check on append

## Merged duplicate (2026-10-01)

ISS-247 is closed as a duplicate of this item. It adds why this is more than a
convenience: ADR-017 makes an inbound `needs` on an unsettled record an
actionability gate, so a stale prerequisite blocks work that is no longer blocked
(e.g. the SL-230 → REV-034 edge after DEC-027's split). It also notes that `unlink`'s
refusal lists `references` three times (ISS-236).
