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
- candidate refs are normal interaction branches;
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
5. **Close lands an admitted close-target OID.** Post-audit integration targets
   the immutable OID that audit admitted for close, never the current value of a
   mutable candidate branch.

## 5. Proposed Design

### 5.1 System Model

Add candidate refs as an explicit interaction layer beside the existing evidence
refs:

| Role | Example ref | Mutability | Purpose |
|---|---|---:|---|
| Coordination | `dispatch/068` | orchestrator-owned | SSoT, ledgers, recovery |
| Evidence bundle | `review/068` | immutable CAS creation | exact impl-bundle audit input |
| Evidence code cuts | `phase/068-01` | immutable CAS creation | exact per-phase code output |
| Review candidate | `candidate/068/review-001` | normal branch | safe audit review/fix surface |
| Close candidate | `candidate/068/close-001` | normal branch | admitted trunk payload |
| Scratch candidate | `candidate/068/try-rev` | normal branch | experiments against another base |

Candidates are produced from evidence refs, not instead of them. A candidate has:

- a role: `review_surface`, `close_target`, or `scratch`;
- a payload class: `impl_bundle` for `review/<slice>`-derived material, or
  `code` for phase-chain material;
- a source: `review/<slice>` for review-surface candidates by default, or the
  verified cumulative phase-chain tip for close-target candidates by default;
- a base: `refs/heads/main`, another feature branch, or an explicit commit;
- a target ref under `refs/heads/candidate/<slice>/...`;
- an optional worktree path for interactive review/fix work;
- a ledger row on `dispatch/<slice>` recording immutable source/base/merge OIDs.

The default audit workflow creates two candidates when both surfaces are needed:

- `review_surface`: sourced from `review/<slice>`, including the impl bundle, for
  external review and RV repair work.
- `close_target`: sourced from the phase-chain tip, preserving ADR-012's existing
  code-to-trunk payload semantics.

A project may explicitly create a `close_target` with `payload = "impl_bundle"`,
but that is a deliberate payload change and must be visible in the candidate
ledger and review surface.

### 5.2 Interfaces & Contracts

Introduce a `doctrine dispatch candidate` verb family. Exact flag names can be
finalised in the plan against the CLI style, but the design contract is:

```text
doctrine dispatch candidate create \
  --slice 68 \
  --role review-surface \
  --base refs/heads/main \
  --target refs/heads/candidate/068/review-001 \
  [--source refs/heads/review/068] \
  --worktree ../doctrine-candidate-068-review

doctrine dispatch candidate create \
  --slice 68 \
  --role close-target \
  --payload code \
  --base refs/heads/main \
  --target refs/heads/candidate/068/close-001 \
  [--source refs/heads/phase/068-02] \
  --worktree ../doctrine-candidate-068-close

doctrine dispatch candidate status --slice 68

doctrine dispatch candidate admit \
  --slice 68 \
  --role close-target \
  --candidate refs/heads/candidate/068/close-001 \
  [--review RV-NNN]
```

`candidate create`:

- requires the requested source ref to correspond to a verified prepare-review
  journal row;
- for a default review-surface candidate, requires verified `review/<slice>`;
- for a code close-target candidate, requires the selected phase row to be
  verified and no earlier non-empty phase-chain row to be failed;
- refuses an existing target ref; v1 supersession creates a new candidate row and
  a fresh target ref, linked by `supersedes`, rather than rewriting an old branch;
- creates a normal branch from `--base` with zero-OID CAS;
- performs an explicit no-ff 3-way merge of `--source` into that branch, so the
  initial `merge_oid` has `base_oid` and `source_oid` as parents;
- on clean merge, commits the merge result and records the candidate row with
  `merge_oid`;
- on conflict, reports and leaves the candidate worktree/branch for manual
  resolution only when `--worktree` was explicit, with status `conflicted`;
- without `--worktree`, aborts on conflict and creates no durable candidate row;
- requires an explicit worktree for `role = review_surface` in v1;
- never writes trunk or `edge`.

`candidate status`:

- lists evidence refs and candidate refs separately;
- reports each candidate's base, source, tip, status, and admitted/superseded
  state;
- prints the safe next commands instead of asking users to inspect raw refs.

`candidate admit`:

- requires the candidate branch to resolve to a committed, clean tip;
- reads the candidate ref, validates provenance, then re-reads before recording;
  if the ref moved during admission, it refuses with a moved-ref diagnostic;
- records an immutable `admitted_oid`; close uses that OID, not the candidate ref;
- requires `merge_oid` to be an ancestor of `admitted_oid`;
- requires `merge_oid` to be the Doctrine-created candidate merge, with
  `base_oid` and `source_oid` as parents;
- for `role = close_target`, replaces the current close admission with the new
  `admitted_oid` and records which prior admission it supersedes;
- may link to an RV that justified the audit repair.

Extend `dispatch sync --integrate` so post-audit trunk projection uses the current
`close_target` admission. Close/integrate never creates, updates, rebases, merges,
or repairs a candidate. It appends/replays normal ADR-012 CAS journal rows:

```toml
source_oid = "<admitted_close_target_oid>"
target_ref = "refs/heads/main"
expected_old_oid = "<current trunk oid at plan time>"
planned_new_oid = "<admitted_close_target_oid>"
status = "pending"
```

If `planned_new_oid` does not fast-forward `target_ref`, close refuses with a
named moved-target reason and instructs the user to create a superseding
close-target candidate on the new base. It never performs a close-time 3-way
merge. `--edge` consumes a `review_surface` admission in the same way; without the
needed admission, candidate-aware integrate refuses rather than falling back to a
raw ref silently.

### 5.3 Data, State & Ownership

Add a structured candidate ledger on the dispatch coordination branch. It is a
journaled state file, like the existing sync journal: status fields may be updated
by committed transitions, but identity/OID fields are never rewritten. Admission
history is represented explicitly; the design does not rely on an "append-only"
claim for mutable row status.

```toml
[[candidate]]
id = "cand-068-review-001"
label = "review-001"
kind = "audit"              # audit|experiment
role = "review_surface"     # review_surface|close_target|scratch
payload = "impl_bundle"     # impl_bundle|code
target_ref = "refs/heads/candidate/068/review-001"
source_ref = "refs/heads/review/068"
source_oid = "<oid>"
base_ref = "refs/heads/main"
base_oid = "<oid>"
merge_oid = "<oid>"         # the Doctrine-created no-ff merge commit
status = "created"          # created|conflicted|abandoned|superseded
supersedes = ""             # optional candidate id
reason = ""
created_by = "dispatch candidate create"
created_at = "<date>"

[current_admission.close_target]
candidate_id = "cand-068-close-001"
candidate_ref = "refs/heads/candidate/068/close-001"
expected_ref_oid = "<candidate ref oid at admission>"
admitted_oid = "<immutable oid close will target>"
review = "RV-NNN"
supersedes = ""             # optional prior admission id
admitted_at = "<date>"
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
3. `dispatch candidate create --role review-surface --base main --worktree ...`
   creates a safe review branch/worktree from `review/<slice>`.
4. `dispatch candidate create --role close-target --payload code --base main
   --worktree ...` creates the close payload candidate from the phase-chain tip.
5. `/audit` and external reviewers inspect candidates, raise RV findings, and may
   apply accepted "fix-now" commits to the relevant candidate branch.
6. `dispatch candidate admit --role review-surface` records the reviewed surface
   OID when useful for `edge`/review provenance.
7. `dispatch candidate admit --role close-target` records the immutable close OID.
8. `/close` integrates the admitted close-target OID after audit/reconcile, still
   under expected-tip protection.

If audit fixes made on the review-surface candidate affect the trunk payload,
they must also be present on the close-target candidate before close admission.
That can happen by editing the close candidate directly, cherry-picking the fix
commits, or recreating a close candidate from an explicitly chosen source. The
important rule is that close lands only the admitted close-target OID.

Experiment path:

1. Create an additional candidate from the same evidence source with
   `--base refs/heads/other-feature` and `--kind experiment`.
2. Test interactions freely on that branch/worktree.
3. Either abandon it, or recreate an audit/close candidate on the real base.

Conflict path:

1. Candidate creation hits a merge conflict.
2. Without an explicit `--worktree`, the command aborts and creates no durable
   candidate row.
3. With an explicit `--worktree`, the command leaves a clearly named candidate
   worktree in conflicted state and records `status = "conflicted"`.
4. The user resolves and commits, then runs `candidate admit`.
5. Close still fast-forwards only to the admitted close-target OID; if trunk moved
   again, a superseding candidate round is required.

Raw evidence path:

- `review/*` and `phase/*` remain inspectable for exactness.
- Skills and command output should refer to them as evidence refs, not as
  ordinary branches to edit or land.

### 5.5 Invariants, Assumptions & Edge Cases

**I1. Evidence immutability.** `review/*` and `phase/*` are CAS-created evidence
refs and are never rewritten by candidate commands.

**I2. Candidate normality.** Candidate refs are ordinary mutable Git branches for
review, repair, and experiments.

**I3. Candidate provenance.** A Doctrine-created candidate records immutable
source, base, merge, and target OIDs. Admission validates that the admitted tip
descends from the recorded candidate merge.

**I4. Admission by OID.** Close targets the admitted OID, never the current
candidate ref value. If the ref moves after admission, status reports drift but
close still targets the admitted OID until a newer admission supersedes it.

**I5. One close admission.** At most one candidate OID is current for close.
Supersession is durable and auditable.

**I6. No close-time merge.** Close only appends/replays CAS journal rows. If the
close target moved, close refuses and requires explicit new candidate
materialisation.

**I7. No hidden trunk writes.** Candidate create/status/admit never mutate trunk
or `edge`.

**I8. Payload role explicitness.** Review-surface candidates and close-target
candidates are distinct, especially where `review/*` contains impl-bundle
`.doctrine` material and `phase/*` is code-only.

**I9. Worker/raw-ref guards.** Candidate/admit/close writes are refused from
worker-marked worktrees and from worktrees checked out on raw evidence refs.
Read-only commands may warn; write-ish commands must refuse unless explicitly
operating by ref from a safe context.

## 6. Non-Blocking Follow-Up Question

OQ-1. How much should `/code-review` and `/audit` be rewired in this slice? This
is non-blocking for SL-068 acceptance. Minimum: update dispatch guidance and
status output. Full rewire may be follow-up because IMP-023/IMP-042 already track
review-skill integration debt.

## 7. Decisions, Rationale & Alternatives

**D1. Add candidates; do not redefine evidence refs.** This preserves ADR-012's
audit exactness while giving humans a normal Git branch.

**D2. Allow explicit 3-way merge only during candidate materialisation.** The
consult decision for SL-068 is "yes": Doctrine may add a normal candidate branch
for audit interaction and experimentation. This is a narrow ADR-006/ADR-012
amendment, not a repeal of report-and-halt trunk integration.

**D3. Record candidate admission on `dispatch/<slice>`.** The coordination branch
already owns projection ledgers and recovery. Candidate provenance belongs there,
not in branch descriptions or prose notes.

**D4. Close targets the admitted close-target OID.** If audit fixed the slice,
closing the raw phase tip would drop those fixes. The admitted close-target OID
is the actual post-audit trunk outcome.

**D5. Split review-surface and close-target roles.** This preserves ADR-012's
payload semantics by default: humans review the impl bundle on a review-surface
candidate, while trunk closes from a close-target candidate whose payload is
explicit.

**D6. Close never repairs candidates.** Trunk movement after admission is a
refusal, not a hidden rebase/merge. The user creates a superseding candidate on
the new base.

**D7. Use `candidate/`, not `topic/`, for Doctrine-owned refs.** `candidate/`
communicates status-bearing workflow ownership. Users can make arbitrary topic
branches from candidate refs.

**D8. V1 close-target payload is code.** V1 should implement code close-target
candidates plus review-surface candidates. Impl-bundle close-target candidates
are explicit/later, not part of the default trunk path.

**D9. Acceptance constraints for implementation.** Phase 1 must amend ADR-006 and
ADR-012 narrowly before or alongside implementation. The implementation plan must
preserve admission-by-OID, no-close-time-merge, provenance-validation, and
raw-ref guard invariants.

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
fast-forward/CAS only. It refuses with guidance to create a superseding candidate
on the new base.

**R5. ADR conflict.** Mitigation: phase 1 must amend ADR-006/ADR-012 narrowly to
permit candidate materialisation while preserving exact refs and guarded trunk.

**R6. Mutable candidate ref drifts after admission.** Mitigation: admission binds
an immutable OID. Status reports ref drift; close does not chase the ref.

**R7. Arbitrary branch admission.** Mitigation: admission validates the recorded
`merge_oid` provenance and requires the admitted OID to descend from it.

## 9. Quality Engineering & Validation

Verification should prove:

- `candidate create` refuses before verified prepare-review evidence exists.
- `candidate create` requires the requested source ref to be a verified
  prepare-review row.
- A clean trunk-advanced case creates a candidate whose diff against live trunk
  does not show phantom stripped `.doctrine` deletions.
- `review/*` and `phase/*` OIDs remain unchanged after candidate create/admit.
- Candidate admit records an immutable admitted OID and close/integrate uses it.
- Moving the candidate ref after admission does not change what close targets.
- Candidate admit rejects a tip that does not descend from recorded `merge_oid`.
- Candidate create uses zero-OID CAS and refuses an existing target.
- Candidate supersession records old and new candidates/admissions.
- Close writes a CAS journal row with `planned_new_oid = admitted_oid`.
- Review-surface and close-target candidates have distinct payload fixtures.
- Close refuses when trunk has moved past the admitted candidate OID.
- A conflicting candidate create without `--worktree` creates no durable row.
- A conflicting candidate create with `--worktree` records and displays the
  conflicted path.
- Worker-mode and raw dispatch worktree guardrails refuse candidate/admit writes
  from worker-marked worktrees.
- Write-ish commands refuse from worktrees checked out on `review/*` or
  `phase/*` with candidate guidance.
- Status output distinguishes evidence refs, candidates, admitted target, and
  next safe action.

Likely focused tests:

- `e2e_dispatch_candidate_clean_merge_from_review`.
- `e2e_dispatch_candidate_clean_merge_from_phase_chain`.
- `e2e_dispatch_candidate_admit_then_integrate_ff`.
- `e2e_dispatch_candidate_ref_moves_after_admit_close_uses_oid`.
- `e2e_dispatch_candidate_admit_rejects_unproven_tip`.
- `e2e_dispatch_candidate_create_zero_oid_cas_refuses_existing`.
- `e2e_dispatch_candidate_supersede_records_history`.
- `e2e_dispatch_candidate_trunk_moved_after_admit_refuses`.
- `e2e_dispatch_candidate_conflict_records_or_aborts`.
- `e2e_dispatch_candidate_unverified_source_refuses`.
- `e2e_dispatch_raw_evidence_worktree_write_refuses`.
- `dispatch_candidate_is_orchestrator_classed`.

## 10. Review Notes

### Internal adversarial pass

F-1. **Does this weaken ADR-006 too broadly?** It could if candidate creation is
described as "auto-resolve allowed". The design narrows it: explicit candidate
materialisation only, before audit/close, never trunk, with conflicts visible.

F-2. **Does audit still review the exact dispatch output?** It can, but the
default review surface becomes a candidate. The design preserves exact refs for
forensics and makes the close-target candidate the admitted trunk outcome. Audit
must record which surface it reviewed.

F-3. **Can audit fixes bypass slice accountability?** Not if `candidate admit`
records an immutable admitted OID and optional RV link on `dispatch/<slice>`, and
close uses only the current close-target admission.

F-4. **Is this too much for SL-068?** The minimal coherent slice is candidate
create/status/admit plus integrate targeting the admitted close-target OID.
Review-skill rewiring and cleanup automation can follow.

### External GPT review — integrated dispositions

F1. **Close semantics conflict.** Integrated. Close never creates, updates,
rebases, merges, or repairs candidates; moved trunk refuses and requires a
superseding candidate.

F2. **Review payload vs trunk payload conflated.** Integrated. The design splits
`review_surface` and `close_target` roles and makes payload class explicit.

F3. **Admission must bind an OID.** Integrated. Admission records
`admitted_oid`; close never chases mutable candidate refs.

F4. **Candidate provenance underspecified.** Integrated. Candidate creation
records `merge_oid`; admission validates merge ancestry before accepting a tip.

F5. **Append-only wording conflicted with mutable status.** Integrated. The
design now states the ledger is journaled state with mutable status, while OID
identity fields are immutable.

F6. **Candidate branch CAS rules underspecified.** Integrated. Candidate target
creation uses zero-OID CAS; supersede creates a new row/ref in v1.

F7. **Conflict lifecycle open.** Integrated. Conflict without explicit worktree
aborts with no durable row; conflict with worktree records conflicted state.

F8. **Kind not enough for admission.** Integrated. The ledger now separates
`kind`, `role`, `payload`, and current admission.

F9. **Verified evidence rows vague.** Integrated. Preconditions are source-row
specific, with stricter phase-chain checks for code close targets.

F10. **Integrate journal interaction missing.** Integrated. Close appends normal
ADR-012 CAS rows with `planned_new_oid = admitted_oid`; `edge` consumes a
review-surface admission.

## 11. Acceptance Record

Accepted for planning on 2026-06-15 with implementation constraints D8 and D9.
The accepted design keeps OQ-1 as non-blocking follow-up scope and resolves the
former impl-bundle close question as code close-target in v1.
