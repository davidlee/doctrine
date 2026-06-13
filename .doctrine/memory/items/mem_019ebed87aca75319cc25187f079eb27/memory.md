# Dispatch fork landed-oracle: --merged, delta-emptiness AND the import receipt all unsound; use a git patch-id check (git cherry) over all B..fork commits

> **Slug note:** this memory's key still reads `…needs-import-receipt`, which is
> *historical and now wrong* — the receipt was rejected (see below). The slug is a
> non-authoritative alias; the uid is identity. The conclusion below is current.

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
- **a runtime-tier import receipt** keyed `{base, fork-head}`, stamped by `import`
  on apply-success — **also unsound, and rejected** (SL-056 round-2 Charge I). The
  receipt certifies the **apply**, not the **commit**: it is born before the
  separate coordination commit, lives in the gitignored/disposable runtime tier
  (outside ADR-006 D7's rebuild-from-coordination-branch recovery), and **survives a
  crash-before-commit** — reading "landed" when no commit ever reached the branch,
  so a recovery-time `gc` reaps the only surviving copy of unmerged work. A flag in
  disposable state gating an irreversible `git branch -D` is no oracle.

**Sound oracle: a durable git patch-id check.** `gc` runs `git cherry
<coordination-HEAD> <fork-branch>` (merge-base computed internally; no `--base`
needed) and reaps **only when every commit in the fork's `B..fork` range is marked
`-`** (its patch is already present in coordination's history by patch-id). Any `+`
⇒ not (fully) landed ⇒ refuse. This is keyed on **durable git state after the
commit**, so it is **crash-proof** (a crash before the commit leaves no landed
patch ⇒ `+` ⇒ refuse), robust to a sibling moving HEAD (patch-id matches the
commit's patch, not a whole-tree diff), and robust to `apply --3way` severing
ancestry (patch-id ≠ ancestry). Ranging over **all** commits (not a single tip)
lets one oracle serve both callers: the single-commit dispatch fork and the
multi-commit solo `/execute` fork (all `-` after a normal merge). A
refused-then-re-dispatched (superseded) fork is reaped **not** by a stored record
but by `gc --superseded-head <SHA>`, reaping only when the named SHA equals the
branch's live head — no durable flag, SHA-keyed (SL-056 round-3 Charge A).

Surfaced by the SL-056 inquisition (design re-locked across three tribunals).
Relevant when implementing `doctrine worktree gc`. Related: import onto moved shared
main [[mem.pattern.dispatch.three-way-import-onto-moved-shared-main]]; re-anchor on
disjoint head move [[mem.pattern.dispatch.reanchor-base-on-disjoint-head-move]].
