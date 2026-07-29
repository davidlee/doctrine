# ISS-279: Entity id reservation has local reach; two trees allocate the same id unwarned

## Symptom

`RV-320` was allocated twice, twelve hours apart, by two trees of the same repo:

| | tree | raised | entity |
|---|---|---|---|
| SL-233's | `dispatch/233` (coordination worktree) | 2026-07-29 00:32 | projection-bounds sketch review, 7 findings |
| SL-237's | `edge` (primary) | 2026-07-29 12:59 | design review of SL-237 |

Nothing warned at allocation on either side.

## Why this is not ISS-221 or ISS-277

- **ISS-221** is a *within-tree* defect — the reservation scan reads all prefixes,
  so per-kind sequences share one counter and gap. Orthogonal.
- **ISS-277** is the *cure* for one kind — `reseat` cannot renumber a review
  because it reads the alias slug through the strict meta reader, which demands a
  `status` reviews deliberately never store (ADR-007 D-C8).

This item is the **prevention**: `reservation reach = "local"` means a
coordination worktree's counter and the primary tree's counter cannot see each
other, so a duplicate allocation is not merely possible but unremarkable.

## Why it is worse than a gapped id

The collision is **silent until merge**. It surfaced at `dispatch refresh-base`
as a conflict on `.doctrine/review/320/review-320.{toml,md}` — which is the
earliest it *can* surface. The exposure window is therefore however long the
coordination branch runs, and SL-233's has run across four phases.

Worse, trunk was **already misdirecting references** before the merge exposed
anything: `edge`/`main` carried `backlog-275`, observation `019fa925…`, and the
memory `mem.pattern.projection.bound-must-not-bind-the-record` (`ref = "RV-320"`)
all meaning *SL-233's* review, while the entity at `320` on main was SL-237's. A
reader on trunk following any of those references landed on the wrong entity, with
no diagnostic. The merge conflict exposed that state; it did not create it.

## Current workaround

Check both trees before every allocation and take the next id above the max of
each:

```bash
git ls-tree --name-only main .doctrine/review/ | grep -oE '[0-9]{3}$' | sort -n | tail -3
git ls-tree --name-only HEAD .doctrine/review/ | grep -oE '[0-9]{3}$' | sort -n | tail -3
```

There is no `--id` override, so the only lever is allocating from a tree whose
counter is already ahead. This is a per-agent ritual — it works only while every
agent remembers it, which is the definition of the wrong place for the check.

## Shape of a fix (not a decision)

Options worth weighing, none chosen here:

- widen reservation reach so a linked/coordination worktree consults the primary
  tree's allocations (`DOCTRINE_RESERVATION_FALLBACK` already exists for the jail
  case — see `mem.pattern.*` on jail reservation);
- a `--id` override so a collision is at least *repairable* without hand-work;
- a cheap collision detector at `refresh-base` / `prepare-review` that names the
  duplicate ids rather than surfacing them as an opaque file conflict.

## Links

- Realised in SL-233 PHASE-04 handover; account in `.doctrine/slice/233/notes.md`.
- Pattern memory: `mem.pattern.dispatch.bundle-trunk-backlog-id-collision`
  (extended with the review case and the `reseat` failure).
