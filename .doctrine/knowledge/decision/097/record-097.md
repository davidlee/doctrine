# DEC-097: Mint the phases convenience symlink in the primary only

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The problem this settles

`refresh_symlink` (`src/state.rs:345`) writes the convenience link

```
.doctrine/slice/<id>/phases  →  ../../state/slice/<id>/phases
```

relative, gitignored (`.gitignore:45`), documented *"never authority"*. Once
phase sheets are single-homed in the primary (DEC-095), that link resolves
**inside the fork** to a directory that will be empty — a plausible-looking lie
in every linked worktree.

Worktrees in this repo nest under the primary
(`/workspace/doctrine/.dispatch/SL-209`), so a relative path *could* climb out —
but only at a fixed depth, and a consuming project chooses its own worktree
layout (ADR-006 D1, policy-agnostic). Relative-to-primary is therefore not
generally derivable.

## Decision

`refresh_symlink` mints the link **only when the tree it is writing into is the
primary**. A linked worktree gets no convenience link at all.

Under DEC-095 this costs nothing to determine: `PrimaryRoot` already carries the
resolved primary, so the primary/not-primary comparison is available at the call
site without a second git call.

## Alternatives rejected

- **Mint it absolute when the primary differs.** Accurate everywhere, but it
  falsifies ADR-006 D4's third clause (*"the `phases` symlink is relative"*),
  widening the REV for a convenience feature, and bakes an absolute path into a
  fork that may outlive that path.
- **Leave it; accept a dangling link in forks.** Zero work, and defensible on
  the letter — gitignored, never followed, never authority. Rejected because the
  cost is not the broken link but that **a dangling link is indistinguishable
  from a real empty one**: it reads as "this slice has no phases" to a human or
  an agent glancing at the tree. That is precisely the failure mode SL-237
  exists to remove — a path that silently resolved somewhere plausible and
  wrong.

## Governance consequence

This option is the one that leaves **ADR-006 D4 clause 3 intact** — the symlink
stays relative wherever it exists. The REV restates D2's withholding parenthetical
and D4's per-worktree clause only; clause 3 needs no change.

Decided by the user in `/design`, 2026-07-29.
