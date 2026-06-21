# IMP-141: doctrine validate relation visibility

## Source

IMP-133 UX review, second pass (RF-4, RF-5).
See `.doctrine/backlog/improvement/133/ux-review-findings.md`.

## Problem (RF-4)

`validate_relations` in `relation_graph.rs` checks for dangling edges and
illegal rows. It IS called by `doctrine validate`. But there's no way to
run relation validation independently, and `validate --help` doesn't
mention that relation checks are included. A user concerned about edge
integrity has no discoverable path.

## Problem (RF-5)

The catalog graph contains 173 edges with `Raw()` labels (`related`: 156,
`descends_from`: 17). These are pre-migration forms authored before the
PHASE-04 migration to the validated `RELATION_RULES` table. They carry no
functional difference (resolve the same way), but their presence is a
signal that the migration is incomplete.

## Scope

- Add "includes relation edge validation (dangling refs, illegal labels)"
  to `doctrine validate --help`
- Add a `--verbose` flag to `validate` that reports raw-label edges as
  informational warnings
- Consider a `doctrine validate relations` subcommand for targeted checks
