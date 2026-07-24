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
