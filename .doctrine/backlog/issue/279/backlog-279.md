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

## What worked — `DOCTRINE_TRUNK_REF` (verified 2026-07-30, RV-324)

**The engine already has the lever.** The "no `--id` override, so the only lever
is allocating from a tree whose counter is already ahead" claim above is too
pessimistic — there is a second lever, and it costs nothing.

`entity::next_id(local, trunk)` (`src/entity.rs:215`) allocates
`max(local ∪ trunk) + 1`. `local` is the working tree's numeric dir names;
`trunk` is `git::trunk_entity_ids` (`src/git.rs:1646`), which runs
`git ls-tree -d --name-only <trunk> -- <kind_dir>/` against the **peeled trunk
ladder** — `DOCTRINE_TRUNK_REF` / `origin/HEAD` / `main` / `master`
(ADR-006 D3). The collision happens only because the default ladder lands on
`main`, which cannot see a live coordination branch.

So point the ladder at the coordination branch for the allocation:

```bash
DOCTRINE_TRUNK_REF=refs/heads/dispatch/233 doctrine review new --facet code-review --target SL-233
```

Verified by running the exact read the engine performs, from the primary tree:

```bash
git ls-tree -d --name-only refs/heads/dispatch/233 -- .doctrine/review/
# .doctrine/review/319 … 321 … 322 … 323
```

`321` and `323` exist only on the coordination branch. Under the default ladder
they are invisible and an `edge`-side allocation mints `323` — a live, already
armed collision at the time of writing, independent of any new review. Under the
override they are in the union and the allocation skips past them.

**Applied to RV-324.** The review was opened *in the coordination tree*, where
`local` already carried `323`, so it minted `324` with no override and no ritual.
The override is what protects the **other** direction: any `edge`-side allocation
while `dispatch/233` is live.

This does not close the issue — it is still a per-agent ritual, just a cheaper and
more reliable one than "check both trees and take the max". It does mean a fix
could be as small as *defaulting the ladder to include live coordination branches*
rather than widening reservation reach.

### Rejected: reserving an id with a placeholder directory

The obvious workaround — `mkdir .doctrine/review/NNN/` in the other tree to claim
the id — is **actively harmful**. Tested and backed out:

- an **empty** dir makes `doctrine doctor` hard-fail: `Error: read review 324`,
  aborting the run rather than reporting a finding;
- a **placeholder `.toml`** is worse. `RV-323` is a *real* entity on the
  coordination branch with 8 live citations on `edge` (two memories,
  `backlog-355`, two comparisons, three observations). Stubbing it made those
  citations resolve to a fake — `doctrine doctor` findings dropped 66 → 58,
  silencing eight *correct* dangling-citation reports.

That is precisely the silent-misdirection failure this issue argues is worse than
a gapped id, so the placebo reproduces the disease. Add it to the "Shape of a fix"
list as a non-option.

## Links

- Realised in SL-233 PHASE-04 handover; account in `.doctrine/slice/233/notes.md`.
- Pattern memory: `mem.pattern.dispatch.bundle-trunk-backlog-id-collision`
  (extended with the review case and the `reseat` failure).
- `DOCTRINE_TRUNK_REF` verification and the placeholder-stub rejection: observation
  `019fb11d-d0ba-7cd3-9a29-d21d878a41e5`; applied at **RV-324**.
