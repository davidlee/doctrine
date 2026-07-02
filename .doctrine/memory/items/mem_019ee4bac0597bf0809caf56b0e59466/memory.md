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

**Does `done` pass? Depends on the journal — CORRECTED at SL-190.** The SL-126
close-integration gate (`ledger.rs::trunk_integration`) distinguishes two cases,
not one: a journal that is **absent / zero rows** → `NotDispatched` (waves
through — the SL-104 case); a journal that **has rows but none target trunk** →
`Blocked("no trunk row")` (refuses). A funnel-driven dispatched slice journals
`review/<N>` + `phase/<N>-NN` rows, so it hits the *second* arm and `done`
**refuses** after a direct-land — even though main genuinely holds the reviewed
code. SL-104 waved through only because its bundle was never journal-integrated;
do not generalise that to a phase-journaled slice. SL-190 had to additionally
**hand-write a verified trunk row** onto `dispatch/<N>`'s `journal.toml`
(`target_ref = trunk`, `planned_new_oid` = the landed merge tip, an ancestor of
trunk) before `done` passed. Neither integrate path can write that row for a
split-lineage slice (candidate path needs the IMP-127-blocked admit; legacy
`plan_trunk_row` needs a ff the split lineage forbids). Tracked as **IMP-236**
(give IMP-127's fix a sanctioned record-completed-integration path).

**Cost.** Skips the admitted-OID CAS provenance the close skill prescribes for
dispatched slices; the conflicted candidate row lingers as gitignored runtime
cruft (harmless). Prefer the journal path ([[mem_019ee36939ca7a70b8aa960cb478d94c]])
when the bundle base is fresh; this is the escape when lineage already split.
Root prevention: don't commit a slice's phases directly to main while a dispatch
bundle for the same slice is in flight.
