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

**Escape (updated at SL-211 — land the *payload*, then record).** Abandon the
candidate/admitted-OID seam and direct-land the earned **trunk payload** — never
`review/<N>`. `review/<N>` is a *review surface*, a different lineage the close
gate rejects as a trunk payload (SPEC-022); the payload is the
`.doctrine`-stripped cumulative code cut `phase/<N>-NN` (legacy) or the admitted
`close_target` candidate ref (candidate-active):
1. Land the payload out-of-band so it becomes an ancestor of trunk: `git merge
   --no-ff phase/<N>-NN` (legacy) or `git merge --no-ff <admitted-close_target>`
   (candidate-active). Resolve conflicts so no work is lost — for each conflicted
   file decide *which lineage owns it*: take the payload's version where it is the
   matured slice deliverable; take trunk's where trunk carries unrelated newer
   work (e.g. a sibling slice's tests) the stale base lacks.
2. **Verify the resolution equals the intended delta**: `git diff --cached` must
   show *only* the slice's intended additions, no reversions of sibling work.
3. Finish any deferred reconcile code-edits, `just check`, commit `close(SL-<N>)`.

**Then record the trunk row — `dispatch sync --record-integration` (SL-211,
supersedes the SL-190 hand-write).** The SL-126 close-integration gate
(`ledger.rs::trunk_integration`) distinguishes two cases: a journal that is
**absent / zero rows** → `NotDispatched` (waves through — the SL-104 case); a
journal that **has rows but none target trunk** → `Blocked("no trunk row")`
(refuses). A funnel-driven dispatched slice journals `review/<N>` + `phase/<N>-NN`
rows, so it hits the *second* arm and `done` **refuses** after a direct-land —
even though trunk genuinely holds the reviewed code (SL-104 waved through only
because its bundle was never journal-integrated; do not generalise that to a
phase-journaled slice). The sanctioned remedy is now:

    dispatch sync --slice N --record-integration --trunk <ref>

It resolves the earned payload from the ledger (identical to `--integrate`'s trunk
planning), asserts that payload is **already an ancestor** of trunk (the earned
check — the same `is_ancestor` standard the gate holds), and commits a single
**Verified** trunk row to `dispatch/<N>`, mutating no external ref. `slice status
N done` then reads that row and passes. **Do not hand-write the row.** SL-190 had
to hand-edit `journal.toml` (`target_ref = trunk`, `planned_new_oid` = the landed
payload tip) only because the verb did not yet exist — that gap was **IMP-236**,
shipped as this `--record-integration` stage (SL-211). (Neither *advancing*
integrate path can write the row for a split-lineage slice: the candidate path
needs the IMP-127-blocked admit; legacy `plan_trunk_row` needs a ff the split
lineage forbids — the recorder sidesteps both by *recording* the already-landed
payload rather than advancing trunk.)

**Cost.** Skips the admitted-OID CAS provenance the close skill prescribes for
dispatched slices; the conflicted candidate row lingers as gitignored runtime
cruft (harmless). Prefer the journal path ([[mem_019ee36939ca7a70b8aa960cb478d94c]])
when the bundle base is fresh; this is the escape when lineage already split.
Root prevention: don't commit a slice's phases directly to main while a dispatch
bundle for the same slice is in flight.
