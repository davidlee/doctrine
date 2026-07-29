# A long-unintegrated dispatch bundle collides backlog ids with trunk; reseat the trunk-side item, not the bundle

When a dispatched slice's impl bundle (`review/<slice>`) sits unintegrated while
trunk keeps running the lifecycle, both sides allocate from the same id namespace
against a shared low-water mark. A backlog id the bundle minted (e.g. during the
dispatch) reads as *free* to trunk once the dispatch's allocation is only on the
un-landed bundle — so trunk re-mints the same id for a different item. At
audit/integration you find **one id pointing at two different entities**.

**Diagnosis.** `doctrine backlog show ISS-NNN` on the trunk tree vs
`git show review/<slice>:.doctrine/backlog/issue/NNN/backlog-NNN.toml` — different
slug/title/created-date ⇒ collision. Chronology settles ownership: the bundle
usually claimed the id first (older `created`); trunk's is the accidental reuse.

**Fix: reseat the trunk-side (mutable) item, never the bundle.** `review/*` is an
R2 immutable evidence ref (ADR-007) — editing it rewrites frozen evidence. The
trunk item is live authored state. `doctrine reseat ISS-NNN` (default `--to` =
next free trunk-aware id) moves the dir/toml/symlink. Reseat is outbound-only: it
lists prose citations to rewrite by hand — fix the item's **own** body heading
(`# ISS-NNN:` → new id) but **leave references that actually point at the bundle's
item** (they correctly keep the original id, which the bundle will land verbatim).
Then the bundle's `.doctrine/` integrates with no id clash.

**How to apply.** Do this *pre-integration*, on the primary/trunk tree. Path-limit
the commit to the two id dirs + symlink (agents share the index). This is
distinct from a code/authored-file merge conflict — see
[[mem.pattern.dispatch.close-deadlock-refresh-base-recovery]] for that; and from
the general un-landed-bundle audit posture, see
[[mem.pattern.dispatch.stale-bundle-web-map-dist-seed]] /
`mem_019f4c64…` (seed `web/map/dist` into the audit worktree first).

## Extension (SL-233 PHASE-04, 2026-07-29): it is not only backlog ids, and for a review `reseat` cannot execute the fix above

Two corrections from a second, differently-triggered instance:

**1. The trigger generalises past an unintegrated bundle.** Here the two
allocators were a **live coordination worktree** (`dispatch/233`) and the primary
tree (`edge`), both running normally — no long-parked bundle required. Root cause
is the same shared low-water mark with `reservation reach = "local"`, so treat any
two trees of one repo as colliding allocators, not just bundle-vs-trunk.

**2. For a REVIEW, `doctrine reseat` refuses — structurally, on every review.**
It reads the alias slug via the strict meta reader (`meta::read_meta` →
`dtoml::parse_entity_toml`), which requires a `status` field; a review's status is
derived from its findings (ADR-007 D-C8) and is deliberately never stored. So:

```
$ doctrine reseat RV-320 --to 322
Error: ... missing field `status`
```

Filed as **ISS-277**. Renumbering a review is therefore **hand-work** — move the
dir, rewrite the id in the toml/md and the slug symlink, and fix inbound prose
citations yourself. The "reseat the trunk-side item" instruction above still names
the right *target*; it just has no verb behind it for this kind.

**Detection point, and why the window is long.** It surfaced at
`dispatch refresh-base`, as a merge conflict on
`.doctrine/review/NNN/review-NNN.{toml,md}` — the earliest it *can* surface. Until
then trunk was already misdirecting references: backlog, observation and memory
records on `edge` cited `RV-320` meaning the *bundle-side* review while trunk's
entity at that id was a different one. So a clean-looking trunk can already be
inconsistent; the conflict exposes it rather than causing it.

**Prevention (cheap, do it every time).** Before any `<kind> new` from a
coordination tree, take the next id above the max across BOTH trees:

```bash
git ls-tree --name-only main .doctrine/<kind>/ | grep -oE '[0-9]{3}$' | sort -n | tail -3
git ls-tree --name-only HEAD .doctrine/<kind>/ | grep -oE '[0-9]{3}$' | sort -n | tail -3
```

There is no `--id` override, so allocating from the tree whose counter is already
ahead is the only lever. Prevention gap filed as **ISS-279**.
