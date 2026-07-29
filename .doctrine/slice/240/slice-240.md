# Forward-sync live checkouts on ref advance

## Context

Doctrine's dispatch machinery lands most of its coordination state with
**working-tree-free** commits: compose a tree in the object db, then advance the
target ref by CAS (`commit_on_behalf` → `git::update_ref_cas`,
`src/dispatch.rs:5564`). That is deliberate and correct when nothing has the ref
checked out.

But the coordination worktree *does* have `dispatch/<slice>` checked out, and the
trunk worktree *does* have `main` checked out. Advancing a ref under a live
checkout moves `HEAD` while leaving the index and working tree at the old tree.
`git status` then reports the just-landed delta as a **phantom staged reversal**
(a staged deletion of a file that only exists in `HEAD`), and any subsequent
pathless — or even path-overlapping — commit in that tree carries the reversal in
and silently reverts the landed state.

This is one defect class with four recorded instances:

| Instance | Site | State |
|---|---|---|
| ISS-030 | `dispatch sync --integrate` (trunk checkout) | closed via SL-121 — **skill/docs mitigation only**, mechanism untouched |
| ISS-038 | same integrate path; the phantom rode a later `.doctrine` commit and reverted SL-122's code off `main` | open |
| ISS-274 | `dispatch record-boundary` — boundary row lands, coord tree left reporting a staged deletion of it | open |
| (memory `mem.pattern.dispatch.mcp-import-lands-object-db-coord-tree-stale`) | `dispatch_import` / `dispatch_conclude_phase` | recorded footgun, no fix |

ISS-274 asked whether the sibling verbs share the defect. They do, and more
broadly than the issue supposed: **every** funnel position transition lands via
`land_funnel_transitions` → `commit_on_behalf` (`src/dispatch.rs:5951`), so
`conclude_phase` — the travelled path — carries it, as does the MCP arm
(`src/mcp_server/dispatch.rs:570`).

**The remedy already exists and is proven, for exactly one caller.** SL-228
PHASE-05 built a conditional forward-sync for `dispatch verify`
(`forward_sync`, `src/dispatch.rs:6176`): derive the materialized baseline
empirically, refuse on untracked or out-of-set operator dirt, refuse on an edit
inside the reverse set, then `git::restore_paths` exactly the changed set and
*prove* the tree describes the tip. Its supporting primitives (`index_matches`,
`tree_clean_untracked`, `worktree_blob_oid`, `blob_oid_at`, `restore_paths`) are
already general. What is not general is its baseline derivation, which walks
`funnel_run` candidates — and its reach, which is one verb.

This slice generalises that seam so a ref advance under a live checkout is
sync-or-refuse **by construction**, not per-verb discipline. At a CAS site the
baseline is not even a search problem: `expected_old` names it exactly.

## Scope & Objectives

Fix the defect class at the mechanism, not the instances:

1. **Name the class in one primitive.** Extract the reusable core of
   `forward_sync` into a guarded resync keyed on an *explicitly supplied*
   baseline (`expected_old`) rather than a funnel-run walk, keeping the existing
   refusal legs (untracked dirt, dirt outside the reverse set, edits inside it)
   and the post-restore identity proof. Verify's current caller becomes one user
   of it, behaviour unchanged.
2. **Resolve the affected checkout.** A ref advance must discover which worktree
   (if any) has the target ref checked out — including linked worktrees, and
   including the case where it is *not* the tree the verb runs in — rather than
   assuming the cwd tree. No-checkout is the common, cheap case: nothing to do.
3. **Apply it at the ref-advance sites of the class**, covering ISS-274
   (`commit_on_behalf`: `record-boundary`, `conclude_phase`, every funnel
   transition, the MCP arm) and ISS-038 (`sync --integrate`'s trunk advance).
4. **Decide and honour the fail posture** per site: silent forward-sync where the
   tree is the funnel's to move, versus a hard refusal *before* the advance where
   the checkout is shared and dirty (ISS-038's argument). Refusals must leave
   ref, index, and worktree byte-unchanged — `commit_on_behalf`'s existing
   contract.
5. **Retire the discipline that stood in for the fix**: ISS-030's skill-level
   "remember to `git restore` after integrate" guidance, and the recorded footgun
   memory, become statements about a mechanism that now handles itself.

## Non-Goals

- **Not** a change to the working-tree-free landing strategy itself. Object-db
  compose + CAS stays; this slice only makes the *checked-out* consequence
  correct.
- **Not** auto-resolution of genuine conflicts or operator work. Doctrine reports
  and refuses (SPEC-021); a destructive `read-tree --reset -u` / `reset --keep`
  over a shared coordination tree is explicitly out — multiple agents share that
  index, and `reset --keep` cannot resync a branch that already advanced under
  the tree anyway (`mem_019ee2a5d84077d3a93c5a3ee52af7ab`).
- **Not** a rework of `dispatch verify`'s semantics or its empirical
  baseline-derivation rationale; verify keeps its behaviour, gaining only a
  shared implementation.
- **Not** the trunk-promotion / close-time gating work already scoped to SL-239,
  nor the runtime-state root resolution of SL-237. Coordinate, do not absorb.
- **Not** ref-advance sites with no live checkout by construction (ref creation
  from `ZERO_OID`, projection refs, reservation refs) beyond confirming they are
  outside the class.

## Affected surface

- `src/dispatch.rs` — `commit_on_behalf`, `land_boundary_row`,
  `land_funnel_transitions`, `run_record_boundary`, `forward_sync` /
  `tree_describes`, the `sync --integrate` advance.
- `src/git.rs` — the sync primitives (`index_matches`, `tree_clean_untracked`,
  `worktree_blob_oid`, `blob_oid_at`, `restore_paths`, `update_ref_cas`) and
  worktree enumeration.
- `src/mcp_server/dispatch.rs` — the MCP funnel arm's landing call.
- Skill/doc surfaces that currently carry the manual-restore workaround.

## Risks, assumptions, open questions

- **R1** — Resyncing a shared tree is destructive if the guard is wrong. The
  existing refusal legs are the mitigation; they must be preserved intact, not
  re-derived.
- **R2** — Behaviour-preservation: `dispatch verify` and the funnel suites are
  the proof that the extraction is faithful; they must stay green unchanged.
- **A1** — Assumed: at a CAS site the pre-advance tip (`expected_old`) is always
  the tree the live checkout materialized, *when* that checkout is clean. Dirty
  cases refuse rather than assume.
- **OQ-1** — Posture per site: forward-sync silently, sync-and-report, or
  pre-refuse on a dirty checkout? ISS-038 argues pre-refuse for shared trunk;
  ISS-274 argues silent sync for the funnel's own coord tree. Likely both, split
  by whether the tree is dispatch-owned. `/design` decides.
- **OQ-2** — Whether the guarded resync belongs in `git.rs` (a worktree
  primitive) or stays in `dispatch.rs` (a writer role) under ADR-001's layering.
- **OQ-3** — Whether integrate's fix is a pre-gate, a post-sync, or both, and
  whether that reopens/settles ISS-030's closed disposition.

## Verification / closure intent

- A regression test per class member: advance a ref under a live checkout, assert
  the tree describes the new tip (or that the verb refused with ref/index/tree
  byte-unchanged) — not merely that the ref moved.
- `dispatch tree-state` reads clean after `record-boundary` and after
  `conclude_phase` in a coordination worktree (ISS-274's reproduction).
- The ISS-038 chain is closed: a subsequent unrelated commit in the trunk tree
  cannot revert an integration.
- Existing dispatch/verify suites green unchanged (R2).
- ISS-274 and ISS-038 closed; ISS-030's mitigation superseded by mechanism.

## Summary

## Follow-Ups
