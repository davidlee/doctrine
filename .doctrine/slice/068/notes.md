# SL-068 notes

## Repro / diagnosis (2026-06-15) — entry point pinned, design deferred

Goal: pin why the `/dispatch` SL-067 deliverables (`review/067`, `phase/067-*`)
were non-integrable. Method: forensics on the existing on-disk artifacts (the
failure is already committed), no fresh run needed.

### Verified findings

**F1 — the "2317 deletions" are 99.7% by-design, not poison.** Diff vs main:

| branch | total D | `.doctrine/` (by-design strip) | real content gap |
|---|---|---|---|
| phase/067-02 | 2317 | 2309 | **8** |
| review/067 | 41 | 33 | **8** |

Phase branches are code-only cuts (`.doctrine` stripped, `plan_phases`
dispatch.rs:464). The real gap is **8 files = the REV feature** (`src/revision.rs`,
`install/templates/revision.{md,toml}`, 5 `tests/e2e_revision*.rs`) that landed on
main *during* the SL-067 run.

**F2 — the phase/review refs were parented on the LIVE TRUNK TIP, not a stale
base.** Ancestry: `91b05c4` (phase parent) is NOT an ancestor of `dispatch/067`;
the coord base `26a3125` is *older* than `91b05c4`
(`26a3125 → 91b05c4 → … → main tip`). So the projection parented on the live tip
— the exact thing the current code forbids (dispatch.rs:115 "Project off the
PINNED FORK-POINT … not the live trunk tip").

**F3 — that live-tip bug is ALREADY FIXED (RV-030), and the fix post-dates the
SL-067 dispatch.** Timeline (oldest→newest):
`26a3125 → 91b05c4 → 46c389d "SL-067 reviewable refs prepared" → ec44501
"fix(SL-064): RV-030 — pinned fork-point projection" → … → main`. SL-067's refs
were cut *before* `ec44501`, so they carry the pre-fix live-tip projection.
Confirmed: current binary computes `merge_base(dispatch/067, main) = 26a3125`
(fork-point), ≠ artifact's `91b05c4`. A fresh run today will not reproduce it.

**F4 — the durable gap survives RV-030: no re-anchor recovery when trunk advances
past the fork-point.** Even with correct fork-point pinning, the phase tip is
parented on `26a3125`; main is 24 commits ahead → `is_ancestor(main, phase_tip)`
false → `integrate --trunk main` fail-closes ("trunk moved; re-anchor required,
not auto-resolved", `plan_trunk_row` dispatch.rs:359). There is no re-anchor
verb. Named-but-demoted in dispatch SKILL.md:229-232 (IMP-043, "report, never
auto-resolve").

**F5 — NOT a stale jail binary; the fix post-dated the cut by 13 min.** Same
evening: prepare-review cut the refs at 23:39:09 (live-tip `91b05c4`); RV-030
fix `ec44501` landed on main at 23:52:30. The source didn't have the fix yet —
not a jail-CARGO_ stale-binary miss (that hazard is real,
`mem.pattern.build.jail-target-redirect`, just not load-bearing here). The two
`prepare-review` journal commits share the 23:39:09 timestamp = one invocation
(intent + applied-status), not a post-fix re-run. Note: prepare-review **cannot
self-correct** an already-cut ref — zero-oid CAS refuses to re-cut. Root context:
SL-067 was dispatched while SL-064 (carrying this fix) + the REV feature landed
on main concurrently — the fork-point fell ~24 commits behind *during* the run.

**F6 — the deliverables look like branches but are NOT mergeable branches.** The
pinned-fork-point projection refs are built for CAS ff-only replay against the
fork-point (ADR-012, dispatch.rs:115-121, audit-exactness). Reviewing/merging
them against *current main* is the category error that surfaces the phantom
deletions. A "normal-looking" topic branch (re-anchor audited units onto live
trunk via 3-way) would remove the footgun — likely conflict-free for SL-067
(slice touched backlog/tag; REV is disjoint files). Design direction: **add** a
re-anchored integration branch, keep the audit refs.

### Verdict — corrected diagnosis

SL-067's nightmare = (1) pre-RV-030 live-tip projection [already fixed] +
(2) trunk advancing past the fork-point with no re-anchor/recovery path and no
tooling to separate the 8 real deletions from the 2309 by-design strips.

The original SL-068 premise ("wire `--integrate` into `/close`") is dead: the
verb fail-closes regardless; wiring it would not have helped.

### Design direction (proposed, NOT locked — start here tomorrow)

**Leading proposal: conclude emits a 3-way `topic/<slice>` integration branch.**

Mechanic (stock git, no new verb needed for the spike):
```
git checkout -b topic/<slice> <live-main-tip>
git merge review/<slice>          # 3-way; merge-base = fork-point
```
A real 3-way merge (merge-base = fork-point) **keeps** trunk's post-fork
additions (the REV feature) instead of deleting them. The 2317 "deletions" were
a 2-way-diff artifact; 3-way dissolves them. Real conflicts (slice + trunk
touched the same lines) still stop — strictly better than today's
fail-close-on-*any*-trunk-movement.

Shape:
- **At conclude (once, NOT per phase)** emit `topic/<slice>` for review.
- **Audit reviews the topic branch** — clean diff vs main, no phantom-deletion trap.
- **Land at close, still POST-audit** (do not integrate pre-audit — the gate holds).
- **Keep** the existing `review/<slice>` + `phase/<slice>-NN` audit refs; the
  topic branch is *added*, not a replacement (audit-exactness preserved).

Why not per-phase: per-phase 3-way = the demoted **IMP-043 per-batch re-anchor**
— erodes audit-exactness (a cut stops being the worker's exact output) and moves
the base under later phases (the funnel assumes a stable `B` per batch).

Prerequisite (already met for fresh runs): the merge is clean only because
**RV-030** makes the ref's parent and tree coherent at the fork-point. On the
old SL-067 refs (parent=live-tip `91b05c4`, tree=fork-point `26a3125`) a 3-way
would *still* corrupt — so this fixes future runs, not the SL-067 salvage (sunk).

### The governance gate — `/consult` FIRST, then maybe ADR-012 amend

This is **not a skill-prose tweak**: ADR-006/012 deliberately chose **ff-only,
"report, never auto-resolve"** (audit-exactness; foreign trunk commits can't
silently reparent the cuts). `git merge` **auto-resolves** non-conflicting hunks
— the exact posture the ADRs forbid. Hot-editing `/dispatch` to say "3-way merge"
would silently contradict an accepted ADR (skill is not higher authority than
canon). So the decision must go through `/consult` + an ADR-012 amendment, not a
stealth edit.

`/consult` question to tee up:
> Dispatch integration is ff-only CAS replay, "never auto-resolve" (ADR-006/012),
> for audit-exactness. But when trunk advances past the fork-point mid-run the
> deliverables become non-integrable (fail-close, no recovery) and a naive
> 2-way merge shows phantom deletions. Proposal: at conclude, also emit a
> `topic/<slice>` produced by a 3-way merge of the audited units onto live trunk
> (auto-resolving non-conflicting hunks; real conflicts still report). Does the
> auto-resolve convenience justify amending the ff-only/never-auto-resolve
> posture — for an *added* integration branch that leaves the audit refs intact?

### First steps tomorrow (cold-start order)

1. `/consult` the ff-only-replay vs 3-way-topic auto-resolve tradeoff (framing above).
2. On a yes: `/design` — decide skill-only (orchestrator runs stock `git merge`)
   vs a CLI verb (`dispatch sync --integrate --3way` / a re-anchor verb); spec the
   conclude-time topic-branch emission; reconcile ADR-012.
3. Check backlog for **IMP-043** (per-batch re-anchor) — likely related/folds in.
4. Restore the secondary follow-up: `/close` `--integrate` wiring, now meaningful
   once a clean integration path exists.

Cross-ref: IMP-043 (backlog), RV-030 / SL-064, ADR-006, ADR-012, dispatch
SKILL.md:115-121 (pinned-base rationale) + :229-232 (demoted re-anchor).

## Design pivot (2026-06-15)

User consultation resolved the governance question: yes, dispatch may add an
explicit normal candidate interaction surface while preserving exact
`review/*` and `phase/*` refs as immutable evidence. The authored design now
targets a candidate/admission workflow:

- stage-1 sync still emits exact evidence refs;
- `dispatch candidate create` materialises normal branches/worktrees on an
  explicit base for audit review, "fix-now" changes, and experiments;
- `dispatch candidate admit` records accepted OIDs on `dispatch/<slice>`;
- close should integrate the admitted close-target OID under the existing
  post-audit expected-tip guard.

Verification for the design write: `just gate` passed after the design/scope
edits. The only observed noise was the existing user git hook warning about a
read-only `/home/david/.local/state/behaviour/...` path inside memory anchoring
tests; the gate exited cleanly.

## External design review integrated (2026-06-15)

A web GPT review accepted the concept but blocked design lock on the
admission/close contract. Integrated changes:

- close never creates, updates, rebases, merges, or repairs candidates;
- review-surface candidates and close-target candidates are distinct roles;
- admission binds immutable OIDs, not mutable candidate refs;
- candidates record `source_oid`, `base_oid`, and `merge_oid`; admission validates
  `merge_oid` ancestry before accepting an `admitted_oid`;
- candidate ledger wording now permits journaled status mutation instead of
  claiming append-only status fields;
- close appends a normal ADR-012 CAS journal row with
  `planned_new_oid = admitted_oid`;
- conflict and raw-ref/worktree guardrails are now normative in the design.
