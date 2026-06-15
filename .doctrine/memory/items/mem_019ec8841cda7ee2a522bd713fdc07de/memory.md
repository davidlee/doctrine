# Candidate create/admit build seam: no-ff 3-way merge, zero-OID CAS, admission-by-OID

The SL-068 candidate ledger lives at `.doctrine/dispatch/<slice>/candidates.toml`
on `dispatch/<slice>` (`src/ledger.rs`: `Candidates`/`CandidateRow`/`CurrentAdmission`/
`Admission`; `status` is the ONLY mutable row field — supersession appends a fresh
row, never an in-place OID rewrite).

**create** (`src/dispatch.rs candidate_create`): provenance gate (source ref must be
a `Verified` prepare-review journal row; a `phase/<slice>-NN` source also refuses an
earlier `Failed` phase row) → resolve base/source OIDs → explicit no-ff 3-way merge
via `git::merge_tree` (`merge-tree --write-tree --merge-base`) + `commit_tree_merge`
(2-parent commit, parents = base_oid + source_oid) → zero-OID CAS branch creation
(`update_ref_cas(.., ZERO_OID)` refuses an existing ref) → record the row. Sequencing
is load-bearing: CAS precedes the row write, worktree materialises before the row, so
a refused/failed step leaves NO partial durable state (the ref is rolled back).
**A conflicted row carries an EMPTY `merge_oid`** — the branch is parked at base for
manual resolve+commit; distinct from a `created` row's real merge_oid.

**admit** (`candidate_admit`): I9 raw-evidence guard first → resolve candidate tip
→ find the recorded row by `target_ref` → role must match → refuse empty `merge_oid`
(unvalidatable) → provenance: `git::parents(merge_oid)` as a SET == {base_oid,
source_oid} AND `merge_oid` is an ancestor of the tip (`is_ancestor`, I3/R7) →
**re-read the ref and refuse if it moved** (EX-1) → record an immutable `admitted_oid`
into `current_admission.<role>` (supersede the prior, ≤1 current, I5). Writes ONLY
candidates.toml — never a ref (EX-4).

**integrate** (`integrate`/`plan_candidate_trunk_row`/`plan_candidate_edge_row`):
candidate-aware iff `read_candidates(slice).rows` is non-empty. Active ⇒ `--trunk`
sources `current_admission.close_target.admitted_oid`, `--edge` the `review_surface`
admitted_oid, via a CAS row `source_oid == planned_new_oid == admitted_oid`
(`projection_row`) — NEVER a close-time merge (I6). Targeting is by admitted OID, so
moving the candidate ref after admission does not change what integrate lands (I4).
Non-ff trunk refuses (re-admit a superseding candidate on the new base); a missing
required admission refuses rather than falling back to a raw ref. Not-active ⇒ the
legacy `phase_chain_tip`/raw-`review/<slice>` path is preserved unchanged (the
behaviour-preservation gate: `tests/e2e_dispatch_sync.rs` stays green untouched).

See [[mem.pattern.dispatch.worktree-import-corrupt-patch-use-checkout]] for the
funnel-side import substitute, and [[mem.pattern.detection.share-write-seam-identity-notion]].
