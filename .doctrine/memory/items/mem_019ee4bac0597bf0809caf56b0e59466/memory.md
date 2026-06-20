# Split-lineage dispatch close: conflicted close_target, direct-land escape

**Symptom.** At `/close` of a dispatched slice, `candidate create --role
close_target --source refs/heads/review/<N> --base refs/heads/main` aborts:
"3-way merge ... conflicts". `--worktree` parks the branch at base, but resolving
+ committing there does NOT make it admittable — the recorded row stays
`status=conflicted` with `merge_oid=""`. `admit` refuses ("no Doctrine merge to
validate"); re-running `create` recomputes the same merge and re-conflicts; there
is **no CLI verb to promote a manually-resolved parked candidate** to admittable.
(admit-by-ref, mem [[mem_019ee33fa5717e838785bb5976a8f939]], only advances an
*already-recorded* clean candidate — not a conflicted initial create.)

**Root cause — split lineage.** The slice's work landed in two divergent places:
a phase committed **directly on main** (SL-104 PHASE-01 `c403177b`) while the
dispatch bundle `review/<N>` branched from a base (`844fe25b`) that *predated*
that direct landing (and a later sibling close, SL-126). So the audited bundle's
base is stale relative to main and the close_target merge can't auto-resolve. An
add/add conflict on a test file both lineages created independently is the tell.

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
