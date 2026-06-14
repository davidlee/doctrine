# Design SL-068: Dispatch candidates for safe audit interaction

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

SL-064/ADR-012 split dispatch output into exact evidence refs:

- `dispatch/<slice>`: coordination SSoT and projection journal source.
- `review/<slice>`: exact impl-bundle projection, parented to the fork-point.
- `phase/<slice>-NN`: exact per-phase code cuts, parented/chained from the same
  fork-point.

That exactness is valuable for audit, but the refs look like ordinary Git
branches. The SL-067 incident showed the trap: an external reviewer and the
orchestrator treated a dispatch worktree/branch as a normal review branch,
raised RV findings, dispositioned them, and applied "fix-now" changes in-place.
The work was useful, but the surface was unsafe and ambiguous.

SL-068 must add a deliberate post-execution audit interaction workflow:

- exact refs remain evidence;
- candidate/topic refs are normal interaction branches;
- audit-time fixes can be admitted back into the slice outcome;
- users can create multiple candidates for review, test merges, and experiments;
- final trunk integration remains post-audit and guarded.

## 2. Current State

`prepare_review` now projects from the pinned fork-point:

```text
trunk_base = merge-base(dispatch/<slice>, trunk)
review/<slice> parent = trunk_base
phase/<slice>-01 parent = trunk_base
phase/<slice>-NN parent = phase/<slice>-(NN-1)
```

This is the RV-030 correction. It protects audit exactness: a foreign trunk
commit landing after dispatch starts does not silently reparent the phase cuts.

`integrate --trunk` still plans the raw phase-chain tip as the trunk update and
requires it to fast-forward the target trunk ref. If trunk has moved past the
fork-point, the command refuses with "re-anchor required, not auto-resolved".
That is correct under ADR-012, but leaves no safe recovery or review/fix surface.

The observable hazards are:

- **Branch affordance mismatch:** `review/*` and `phase/*` are evidence refs, but
  users and reviewers read them as normal merge branches.
- **Worktree ambiguity:** a worktree checked out on a raw dispatch ref can look
  like a dangerous branch, a repair branch, or a landing branch depending on the
  reviewer.
- **Audit repair gap:** RV "fix-now" changes made after execution have no
  structured admission point.
- **Experiment gap:** users may need to try dispatch output against live trunk or
  another feature before accepting it.
- **UX opacity:** a diff against current trunk can bury a small real delta under
  thousands of expected stripped `.doctrine` paths.

## 3. Forces & Constraints

**ADR-012 exactness.** `review/*` and `phase/*` are the material reviewed by
audit. They must stay reconstructable from `dispatch/<slice>` and the journal.

**ADR-006 report-and-halt posture.** Dispatch must not silently land or
force-resolve trunk. Candidate creation may be an explicit interaction step, but
trunk projection still needs expected-tip/CAS protection.

**Audit happens after execution.** Reviewers must be able to raise RV findings
and apply accepted fixes during audit without smuggling changes through an
untracked branch.

**Multiple human workflows are legitimate.** A user may want a clean audit branch
on `main`, a throwaway branch merged with another feature, or a worktree to run an
interactive experiment.

**The storage rule still applies.** Candidate metadata is structured run state.
It belongs in TOML on the dispatch coordination branch, not in prose.

## 4. Guiding Principles

1. **Evidence refs are immutable.** Do not make `review/<slice>` mean "whatever
   the audit branch currently contains".
2. **Interaction refs are normal.** A candidate branch should behave like an
   ordinary Git branch because that is what humans and review tools expect.
3. **Admission is explicit.** A candidate does not become the slice outcome just
   because someone edited it.
4. **Auto-resolution is opt-in and local to candidates.** A 3-way merge can be
   used to create an interaction branch; it is not a trunk landing.
5. **Close lands what audit accepted.** Post-audit integration should target the
   admitted candidate tip, not the raw fork-point phase chain.

## 5. Proposed Design

### 5.1 System Model

Add a fourth dispatch output role:

| Role | Example ref | Mutability | Purpose |
|---|---|---:|---|
| Coordination | `dispatch/068` | orchestrator-owned | SSoT, ledgers, recovery |
| Evidence bundle | `review/068` | immutable CAS creation | exact impl-bundle audit input |
| Evidence code cuts | `phase/068-01` | immutable CAS creation | exact per-phase code output |
| Candidate | `candidate/068/audit` | normal branch | safe audit interaction, fixes, experiments |

Candidates are produced from evidence refs, not instead of them. A candidate has:

- a source: `review/<slice>` by default, or a specific `phase/<slice>-NN` /
  phase-chain tip for code-only experiments;
- a base: `refs/heads/main`, another feature branch, or an explicit commit;
- a target ref: default `refs/heads/candidate/<slice>/audit`, with user labels for
  alternates;
- an optional worktree path for interactive review/fix work;
- a ledger row on `dispatch/<slice>` recording source/base/target/tip/status.

The default candidate is the safe path for audit. Other candidates are allowed
for experiments but do not become close targets until admitted.

### 5.2 Interfaces & Contracts

Introduce a `doctrine dispatch candidate` verb family. Exact flag names can be
finalised in the plan against the CLI style, but the design contract is:

```text
doctrine dispatch candidate create \
  --slice 68 \
  --base refs/heads/main \
  --target refs/heads/candidate/068/audit \
  [--source refs/heads/review/068] \
  [--worktree ../doctrine-candidate-068-audit]

doctrine dispatch candidate status --slice 68

doctrine dispatch candidate admit \
  --slice 68 \
  --candidate refs/heads/candidate/068/audit \
  [--review RV-NNN]
```

`candidate create`:

- refuses if stage-1 prepare-review has not produced verified evidence rows;
- refuses to overwrite an existing target unless an explicit supersede path is
  used;
- creates a normal branch from `--base`;
- performs an explicit 3-way merge of `--source` into that branch;
- on clean merge, commits the merge result and records the candidate row;
- on conflict, reports and leaves the candidate worktree/branch for manual
  resolution, with status `conflicted`;
- never writes trunk or `edge`.

`candidate status`:

- lists evidence refs and candidate refs separately;
- reports each candidate's base, source, tip, status, and admitted/superseded
  state;
- prints the safe next commands instead of asking users to inspect raw refs.

`candidate admit`:

- requires the candidate branch to resolve and be cleanly committed;
- records the candidate tip as the admitted audit outcome in the dispatch ledger;
- supersedes any prior admitted candidate for the same role by append-only ledger
  status, not by mutating old rows in prose;
- may link to an RV that justified the audit repair.

Extend `dispatch sync --integrate` so the post-audit trunk target is the admitted
candidate when one exists. The trunk update remains fast-forward-only and
expected-tip guarded: if trunk moved since the candidate base, close recreates or
updates the candidate rather than silently merging at close.

### 5.3 Data, State & Ownership

Add a structured candidate ledger on the dispatch coordination branch:

```toml
[[candidate]]
label = "audit"
target_ref = "refs/heads/candidate/068/audit"
source_ref = "refs/heads/review/068"
source_oid = "<oid>"
base_ref = "refs/heads/main"
base_oid = "<oid>"
tip_oid = "<oid>"
status = "created" # created|conflicted|admitted|superseded|abandoned
kind = "audit"     # audit|experiment
review = "RV-NNN"  # optional
created_on = "<date>"
```

The ledger lives under `.doctrine/dispatch/<slice>/candidates.toml` on
`dispatch/<slice>`, beside the existing run ledgers. It is read by
`candidate status`, `candidate admit`, and close/integrate. It is not copied into
candidate branches and must not land on trunk.

Candidate branches are normal Git refs. Candidate worktrees are normal worktrees
without the dispatch worker marker. They may carry gitignored local state that
helps `candidate status` identify them, but the durable source of truth is the
dispatch ledger.

### 5.4 Lifecycle, Operations & Dynamics

Default happy path:

1. `/dispatch` completes execution on `dispatch/<slice>`.
2. `dispatch sync --prepare-review` creates immutable `review/*` and `phase/*`.
3. `dispatch candidate create --kind audit --base main` creates
   `candidate/<slice>/audit` from `review/<slice>`.
4. `/audit` and external reviewers inspect the candidate branch/worktree and may
   raise RV findings.
5. Accepted "fix-now" changes are committed on the candidate branch.
6. `dispatch candidate admit` records the candidate tip as the slice's accepted
   audit outcome.
7. `/close` integrates the admitted candidate after audit/reconcile, still under
   expected-tip protection.

Experiment path:

1. Create an additional candidate from the same evidence source with
   `--base refs/heads/other-feature` and `--kind experiment`.
2. Test interactions freely on that branch/worktree.
3. Either abandon it, or recreate/admit an audit candidate on the real close base.

Conflict path:

1. Candidate creation hits a merge conflict.
2. The command leaves a clearly named candidate worktree in conflicted state and
   records `status = "conflicted"`.
3. The user resolves and commits, then runs `candidate admit`.
4. Close still fast-forwards only to the admitted candidate tip; if trunk moved
   again, a new candidate round is required.

Raw evidence path:

- `review/*` and `phase/*` remain inspectable for exactness.
- Skills and command output should refer to them as evidence refs, not as
  ordinary branches to edit or land.

### 5.5 Invariants, Assumptions & Edge Cases

**I1. Evidence immutability.** Existing `review/*` and `phase/*` refs are created
once by CAS and never rewritten by candidate commands.

**I2. Candidate normality.** Candidate refs are allowed to have merge commits and
audit repair commits. That is their purpose.

**I3. Admission uniqueness.** At most one candidate per slice is the current
admitted close target. Prior admitted rows are superseded, not erased.

**I4. Close guard.** Close/integrate never performs a new 3-way merge. It only
fast-forwards the close target to an admitted candidate tip under CAS.

**I5. No hidden trunk writes.** Candidate create/admit/status do not mutate trunk.

**I6. Raw-ref warning.** If a command can identify that the current branch is
`dispatch/*`, `review/*`, or `phase/*`, it should print/refuse with guidance to
use a candidate unless the operation is explicitly read-only.

**I7. Source compatibility.** The default admissible source is `review/<slice>`
because it carries the impl bundle. Code-only phase candidates are experimental
unless the user explicitly admits a code-only close posture.

## 6. Open Questions & Unknowns

OQ-1. Should `candidate create` require an explicit `--worktree` in v1, or may it
create a branch in the current worktree when clean? Recommendation: require an
explicit worktree for audit candidates to avoid booby-trap reuse of the session
tree.

OQ-2. Should conflicting candidate creation leave the branch/worktree in conflict
state, or abort and record a failed attempt? Recommendation: leave it only when a
worktree path was explicitly requested; otherwise abort without creating a hidden
mess.

OQ-3. Should candidate refs be `candidate/<slice>/<label>` or
`topic/<slice>/<label>`? Recommendation: use `candidate/` for doctrine-owned
status semantics; users can create arbitrary topic branches from a candidate.

OQ-4. How much should `/code-review` and `/audit` be rewired in this slice?
Minimum: update dispatch guidance and status output. Full rewire may be follow-up
because IMP-023/IMP-042 already track review-skill integration debt.

## 7. Decisions, Rationale & Alternatives

**D1. Add candidates; do not redefine evidence refs.** This preserves ADR-012's
audit exactness while giving humans a normal Git branch.

**D2. Allow explicit 3-way merge only during candidate materialisation.** The
consult decision for SL-068 is "yes": Doctrine may add a normal candidate/topic
branch for audit interaction and experimentation. This is a narrow ADR-006/ADR-012
amendment, not a repeal of report-and-halt trunk integration.

**D3. Record candidate admission on `dispatch/<slice>`.** The coordination branch
already owns projection ledgers and recovery. Candidate provenance belongs there,
not in branch descriptions or prose notes.

**D4. Close targets the admitted candidate.** If audit fixed the slice, closing
the raw phase tip would drop those fixes. The admitted candidate is the actual
post-audit outcome.

Alternatives rejected:

- **Make `review/<slice>` a normal merge branch.** Reject: loses exact
  reconstructable evidence and makes audit drift invisible.
- **Keep ff-only and improve errors only.** Reject: still leaves no workflow for
  audit repair or experiments.
- **Auto-reanchor during close.** Reject: close would become the first place a
  merge happens, after audit, so audit would not have reviewed the actual output.
- **Require humans to run raw git.** Reject: repeats the SL-067 trap and gives no
  durable admission record.

## 8. Risks & Mitigations

**R1. Candidate branch sprawl.** Mitigation: `candidate status` groups by slice
and status; `gc` can later reap abandoned/superseded candidates after close.

**R2. Reviewers audit the wrong surface.** Mitigation: conclude/status output
names exact refs as evidence and names the candidate as the interaction branch.
Raw-ref write/land paths should refuse or warn.

**R3. Candidate merge hides important conflict choices.** Mitigation: clean
auto-merge is explicit and pre-audit; conflicts stop in candidate creation or are
resolved by human commits that audit can inspect.

**R4. Final trunk moves after admission.** Mitigation: close remains
fast-forward/CAS only. Recreate or update the candidate on the new base.

**R5. ADR conflict.** Mitigation: phase 1 must amend ADR-006/ADR-012 narrowly to
permit candidate materialisation while preserving exact refs and guarded trunk.

## 9. Quality Engineering & Validation

Verification should prove:

- `candidate create` refuses before verified prepare-review evidence exists.
- A clean trunk-advanced case creates a candidate whose diff against live trunk
  does not show phantom stripped `.doctrine` deletions.
- `review/*` and `phase/*` OIDs remain unchanged after candidate create/admit.
- Candidate admit records the candidate tip and close/integrate uses it.
- Close refuses when trunk has moved past the admitted candidate base/tip.
- A conflicting candidate create reports clearly and does not mutate trunk.
- Worker-mode and raw dispatch worktree guardrails refuse candidate/admit writes
  from worker-marked worktrees.
- Status output distinguishes evidence refs, candidates, admitted target, and
  next safe action.

Likely focused tests:

- `e2e_dispatch_candidate_clean_merge_from_review`.
- `e2e_dispatch_candidate_admit_then_integrate_ff`.
- `e2e_dispatch_candidate_trunk_moved_after_admit_refuses`.
- `e2e_dispatch_candidate_conflict_records_or_aborts`.
- `dispatch_candidate_is_orchestrator_classed`.

## 10. Review Notes

### Internal adversarial pass

F-1. **Does this weaken ADR-006 too broadly?** It could if candidate creation is
described as "auto-resolve allowed". The design narrows it: explicit candidate
materialisation only, before audit/close, never trunk, with conflicts visible.

F-2. **Does audit still review the exact dispatch output?** It can, but the
default review surface becomes a candidate. The design preserves exact refs for
forensics and makes the candidate the admitted post-audit outcome. Audit must
record which surface it reviewed.

F-3. **Can audit fixes bypass slice accountability?** Not if `candidate admit`
records the candidate tip and optional RV link on `dispatch/<slice>`, and close
uses only an admitted candidate.

F-4. **Is this too much for SL-068?** The minimal coherent slice is candidate
create/status/admit plus integrate targeting admitted candidate. Review-skill
rewiring and cleanup automation can follow.
