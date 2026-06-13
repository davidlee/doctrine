# Dispatch fork landed-oracle: --merged and delta-emptiness both unsound; use an import receipt

The dispatch funnel imports a worker's delta with `git apply --3way --index`
(non-committing) onto base `B`, then the orchestrator commits separately. This
**severs git ancestry**: the fork branch `S` is never an ancestor of the
coordination commit. So a `gc`/cleanup "safe to delete this spent fork?" check
**cannot** use:

- `git branch --merged` — the apply-funnel branch is never merged, always reported
  unmerged. (Corollary: `git branch -d` always refuses it; deletion needs `-D`.)
- **delta-emptiness** (`git diff <B-or-HEAD>..<fork>` empty ⇒ landed) — also
  unsound. `B..fork` is the worker's whole delta, **never empty** for real work
  (refuses every imported fork). `HEAD..fork` after the batch commit is
  `(B+1)..S`; the instant a sibling moves the coordination HEAD (the common
  moved-shared-main case) the tree legitimately diverges ⇒ non-empty ⇒ refuses a
  spent fork. Either way the operator learns a `--force` reflex and the safety gate
  collapses to "delete whatever I point at" → reaps unmerged work.

**Sound oracle: an explicit import receipt.** `import` stamps a record keyed
`{base, fork-head}` into the withheld runtime tier on success; `gc` deletes only on
a positive receipt (`--force` is the explicit override). Landed-ness is a fact the
import *records*, not a property inferable from merge-status or tree content once
`apply --3way` has severed ancestry and HEAD has moved.

Surfaced by the SL-056 inquisition (design re-locked, commit f406981). Relevant
when implementing `doctrine worktree gc`. Related: import onto moved shared main
[[mem.pattern.dispatch.three-way-import-onto-moved-shared-main]]; re-anchor on
disjoint head move [[mem.pattern.dispatch.reanchor-base-on-disjoint-head-move]].
