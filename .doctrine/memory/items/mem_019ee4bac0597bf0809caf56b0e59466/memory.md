# Dispatch candidate verb can't ingest a hand-resolved merge conflict — close dead-ends

**The defect (durable lesson).** `doctrine dispatch candidate create` is
all-or-nothing: it runs its *own* internal 3-way merge and either records a clean
candidate or, on *any* conflict, parks the worktree at base with
`status=conflicted, merge_oid=""` and stops. There is **no verb to feed a manual
resolution back in.** Resolving + committing in the parked worktree and
`git checkout -B`-ing the branch does NOT help — `admit` validates the recorded
`merge_oid`, which stays empty ("no Doctrine merge to validate"); re-running
`create` recomputes the same conflict. So the close dead-ends even when the
underlying git conflict is trivial to resolve by hand. (admit-by-ref, mem
[[mem_019ee33fa5717e838785bb5976a8f939]], only advances an *already-recorded clean*
candidate — not a conflicted initial create.) The fix — an "it's complicated" path
that adopts a hand-made (base, source) merge — is tracked as **IMP-127**.
Deliberately *not* a `--force`: the merge still happens and is still validated; the
operator just performs it.

**Trigger — base drift (split lineage is one form).** The auto-merge conflicts
whenever trunk moves between bundle creation and close. SL-104: a phase landed
**directly on main** (PHASE-01 `c403177b`, via a `WIP: dirty tree` rescue commit)
while the dispatch bundle `review/<N>` branched from an earlier base (`844fe25b`)
that predated it and a sibling close (SL-126). An add/add conflict on a file both
lineages created independently is the tell. Same family: a sibling slice closing
first, any dirty-tree rescue commit. Root prevention: don't author a slice's
phases in the main tree while its dispatch bundle is in flight.

**Escape (user-approved, SL-104).** Abandon the candidate/admitted-OID seam and
direct-land:
1. In the parked worktree, `git merge --no-ff review/<N>`, resolve conflicts so
   no work is lost — for each conflicted file decide *which lineage owns it*:
   take the audited bundle's version where it's the matured slice deliverable;
   take main's where main carries unrelated newer work (e.g. a sibling slice's
   tests) the bundle's stale base lacks.
2. **Verify the resolution equals the intended delta**: from main,
   `git diff --cached` of `git checkout <merge> -- <files>` must show *only* the
   slice's intended additions, no reversions of sibling work.
3. Apply those files to main, finish any deferred reconcile code-edits, `just
   check`, commit `close(SL-<N>)`, then `slice status <N> done`.

**Why `done` still passes.** The SL-126 close-integration gate refuses
`reconcile → done` only when a dispatched slice's journal has an *integration-
pending trunk row*. A bundle that was never journal-integrated (no trunk row)
hits the `dispatch_ref_present_no_journal_succeeds` case and waves through — so
direct-landing does not trip the gate.

**Cost.** Skips the admitted-OID CAS provenance the close skill prescribes for
dispatched slices; the conflicted candidate row lingers as gitignored runtime
cruft (harmless). Prefer the journal path ([[mem_019ee36939ca7a70b8aa960cb478d94c]])
when the bundle base is fresh; this is the escape when lineage already split.
Root prevention: don't commit a slice's phases directly to main while a dispatch
bundle for the same slice is in flight.
