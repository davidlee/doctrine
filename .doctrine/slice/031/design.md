# Design SL-031: Dispatch orchestrator funnel: worker-mode workers and import-verify-commit-record

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

Canonical technical design. Governs **IMP-003's funnel half** (the OQ-1 split
SL-029 deferred), under **ADR-006** (worktree posture, orchestrator-sole-writer):
D2/D2a/D5/D6/D6a/D7/D8 mechanised here; D1/D9 already shipped by SL-029. Layering
per **ADR-001** (leaf ← engine ← command, no cycles); pure/imperative split per
slices-spec § Architecture.

## 1. Design Problem

Two coupled deliverables, deliberately kept in one slice and phased A→B (the Rust
is concrete/testable/unblocked; the funnel is mostly skill-prose):

- **A — Production minting + kind registry.** Wire trunk-aware id allocation into
  every `*::run_new` (the 5 `&[]` placeholders), and dedup the scattered per-kind
  identity (SL-032 review F-2/F-5) behind one referenced registry with a
  set-equality guard.
- **B — The dispatch funnel.** Implement the `mode=worker` half of `/worktree`
  (SL-029 declared it, shipped solo only), fill the `/dispatch` placeholder with
  the orchestrator-sole-writer funnel (import→verify→commit→record), and add the
  branch-point check under concurrency.

## 2. Current State

**Prerequisite already satisfied (reframing).** The slice scope named IMP-002
(worker-mode guard D2a + trunk-ref minting/reseat D3) as an *open* execution
prerequisite. It is not open in substance — **all of it shipped under SL-032**:

| ADR-006 mechanism | Landed |
|---|---|
| D2a worker-mode guard (`DOCTRINE_WORKER=1`) | SL-032 PHASE-01 — `main.rs::worker_mode` + `write_class` Read/Write split; `tests/e2e_worker_guard.rs` (black-box goldens) |
| D3 trunk-ref minting (`entity::next_id(local,trunk)`, `git::trunk_entity_ids`, `DOCTRINE_TRUNK_REF` ladder) | SL-032 PHASE-02 |
| `validate` + `reseat` + `integrity::KINDS` | SL-032 PHASE-03 |

So **SL-031 is not execution-blocked**; the gate is open. IMP-002 the backlog
item is stale-open and is reconciled/closed by this slice (its wiring tail lands
here). The original assumption A-1 ("execution blocked until IMP-002 lands") is
retired.

**Residue this slice closes:**
- 5 `run_new` sites pass `&[]` for `trunk_ids`, each tagged `// trunk ids:
  production minting wires them in SL-031 (§5.4)` — `slice.rs:207`,
  `governance.rs:338`, `spec.rs:661`, `backlog.rs:539`, `requirement.rs:238`.
  **Production minting is not trunk-aware today**: two divergent worktrees would
  mint colliding ids (the exact hazard D3 exists to prevent).
- `integrity::KINDS` is a literal **parallel copy** of each module's identity
  (F-2) with `has_runtime_state: bool` + a hardcoded `.doctrine/state/slice` in
  `reseat` (F-5).
- `plugins/doctrine/skills/worktree/SKILL.md` declares the `mode=worker` contract
  but implements only `solo`.
- `plugins/doctrine/skills/dispatch/SKILL.md` is a placeholder.

**Crucial consequence — funnel workers never mint.** Under D2a every
doctrine-mediated authored write refuses in worker-mode; the **orchestrator**
mints, serially, on the coordination branch. Therefore the trunk-aware-minting
wiring (deliverable A) protects the **solo/team divergent-worktree** case
(SL-029's world), *not* the funnel workers. A and B share a slice for delivery
convenience and the shared `/worktree` seam, not because B depends on A's minting.

## 3. Forces & Constraints

- **ADR-006 governs.** D2 (worker-sole-writer), D2a (CLI guard — shipped), D5
  (branch-point check; extend to concurrency), D6 (orchestrator pre-distill), D6a
  (worker vs solo mode decides who writes, not location), D7 (funnel discipline,
  strict order), D8 (coordination branch), D9 (provision sole-copier — shipped).
- **ADR-001 layering.** `entity` is the engine leaf; `integrity` and the kind
  modules are command-layer over it. A registry consumed by both must not induce a
  cycle.
- **Behaviour-preservation gate.** Changing shared machinery (the kind identity
  surface, the entity engine) must keep existing validate/reseat/run_new suites
  green **unchanged** — they are the proof.
- **SL-029 precedent for the CLI/skill boundary.** A tested CLI verb exists *only*
  where a guarantee must be physically mechanical (copy-exclusion → `provision`).
  Orchestration is skill-prose. The funnel honours this.
- **Framework neutrality (D1).** No project-wide interception, no hardcoded build
  commands, no Claude-only dependency on the portable path.
- **Pure/imperative split.** No git/disk/clock in the pure core; the shell reads
  HEADs and passes them in (the `next_id(local, trunk)` pattern).

## 4. Guiding Principles

- **Finish, don't rebuild.** Reuse SL-029's `/worktree` lifecycle and SL-032's
  minting/guard/registry seams; add only the funnel-specific layer.
- **The mode, not the location, decides who writes (D6a).** Worker-mode ON ⟹
  source-only + funnel; solo ⟹ writes doctrine state directly.
- **Knowledge trails confirmed code (D7).** Record only after the commit lands on
  the coordination branch.
- **Report, never auto-merge.** Genuine cross-batch code coupling is a batching
  error surfaced to a human, not silently resolved (ADR-006 Negative).
- **Honest verification class.** Skill-prose orchestration is VA (agent/audit),
  not VT; only the mechanical seams (minting, registry membership, branch-point
  compare) carry VTs.

## 5. Proposed Design

### 5.1 System Model

```
 orchestrator (coordination branch, worker-mode OFF, sole writer)
   │  capture base B = git rev-parse HEAD   (commit-before-spawn: tree clean)
   │  batch = dependency-DISJOINT tasks (the batching guarantee)
   ├─ per worker (concurrent):
   │    /worktree mode=worker → git worktree add @B → provision → baseline✓
   │    Agent isolation:worktree, env DOCTRINE_WORKER=1, pre-distilled prompt (D6)
   │        worker: mutate SOURCE only → run verify cmd → git commit to fork branch
   │                → return structured report (NOT a doctrine write)
   └─ funnel per BATCH (D7, strict order — the cadence is the batch, not the worker):
        import   apply EVERY worker's delta onto B, NON-committing
                 (cherry-pick -n / git apply — shared object store)
        verify   run the project verify cmd on the combined working tree
        guard    branch-point-check --base B   (HEAD still == B? else re-dispatch)
        commit   ONE batch commit on the coordination branch  → HEAD = B+1
        record   memory / AC evidence / notes — trails the confirmed commit
   next batch forks from B+1.  conflict during import = batching error → report+halt.
```

Why per-batch, not per-worker: a per-worker commit advances HEAD, so the next
worker's delta would land on a moved base — exactly the "silently merge against a
moved base" D5 forbids — and a literal branch-point check would then spuriously
re-dispatch every worker after the first, defeating parallelism. Importing the whole
disjoint batch onto the single captured `B`, verifying once, and committing once
keeps every delta applied against the *same* base and makes "incremental per batch"
(D7) literally one commit per batch.

### 5.2 Interfaces & Contracts

**A — `entity::KindIdentity` (new, engine leaf).** The single per-kind identity
the engine, integrity, and minting all read:

```rust
pub(crate) struct KindIdentity {
    pub prefix: &'static str,             // "SL"            → canonical id
    pub dir: &'static str,                // ".doctrine/slice" → entity-tree root
    pub stem: &'static str,               // "slice"         → slice-007.toml
    pub state_dir: Option<&'static str>,  // ".doctrine/state/slice" | None (F-5)
}
```

- `entity::Kind` embeds `&'static KindIdentity` (replacing its bare `prefix`/`dir`,
  carrying `scaffold` as today). `governance::GovKind`'s `stem` folds into the
  identity. The fn-pointer `scaffold` stays on `Kind` (kind-specific behaviour, not
  data) — the engine `Kind` is still data, not a trait (mem
  `entity.kind-is-data-not-trait`).
- `integrity::KINDS: &[&'static KindIdentity]` **references** the kind consts
  (`&SLICE_KIND.identity`, …) — no re-typed literals (closes F-2). `reseat` reads
  `identity.state_dir` instead of `has_runtime_state` + hardcoded path (closes F-5).

**A — minting wiring.** Each `run_new` replaces `&[]` with
`git::trunk_entity_ids(&root, KIND.identity.dir)?`. No new signature — `materialise`
already takes `trunk_ids: &[u32]`.

**B — `doctrine worktree branch-point-check` (new verb, `src/worktree.rs`).**
The one mechanical seam of the funnel:

```
doctrine worktree branch-point-check --base <SHA> [--head <SHA>]
  --base = the orchestrator's pre-spawn captured base B
  exit 0  if base == coordination HEAD (default --head: `git rev-parse HEAD`)
  exit 1  otherwise  (→ orchestrator re-dispatches the batch; never commits on a moved base)
```

Pure compare in the leaf (`fn matches(base, head) -> bool`); the git read of HEAD
is the impure shell. Read-classed (no authored write) so it is callable under
worker-mode — though only the orchestrator uses it, at the batch-commit boundary.

**B — `/worktree mode=worker` contract** (implement the SL-029 stub):
- MUST NOT degrade to work-in-place (a worker with no real fork is a hard abort;
  the funnel's isolation is mandatory).
- Worker mutates source only; runs the orchestrator-supplied verify command;
  **commits the source change to its fork branch** (a raw `git commit` — *not* a
  doctrine-mediated write, so D2a does not refuse it); returns a structured report.
- Outputs add `{ fork_branch, head_sha_after }` to the SL-029 output shape.

**B — `/dispatch` skill** (fill placeholder): orchestrator-sole-writer remit;
dependency-batching; fork+provision+spawn; the D7 funnel loop; branch-point check
at import; conflict → report; crash recovery. Skill-prose (VA). Authored in
`plugins/doctrine/skills/dispatch/`, never the gitignored install copy.

### 5.3 Data, State & Ownership

- **Source delta (OQ-3).** The worker's **fork branch ref is the delta.** Git
  worktrees share the object DB (`git worktree add` under one common `.git`), so the
  orchestrator reads the branch directly — no patch transport. The "structured
  report" is the worker's returned agent message (what changed + verify result +
  memory-worthy notes), held in orchestrator context, never a doctrine artifact.
  **Fallback:** a non-shared-store worker (remote agent) hands back a
  `git format-patch` series; documented as the exception, not the default.
- **Who writes (D6a).** Worker: source only, in the fork, commits to fork branch.
  Orchestrator: every doctrine-mediated write (import-commit, memory, AC evidence,
  notes, status), on the coordination branch. The fork withholds the
  coordination/runtime tier by construction (D9 provision exclusion).
- **Kind identity (A).** Owned distributively by each kind module's const (where it
  already lives); `integrity::KINDS` is a *view* over those consts, not a second
  owner.

### 5.4 Lifecycle, Operations & Dynamics

**Funnel order (D7), the cadence is the batch:** import-all (non-committing) →
verify-combined → branch-point guard → one batch commit → record. Knowledge always
trails the confirmed commit; the coordination branch is the durable store;
orchestrator context is disposable.

**Worker environment (D2a engagement).** `DOCTRINE_WORKER=1` is set **nowhere
today** — the guard in `main.rs` is inert until the worker's process carries the
var. The orchestrator **sets `DOCTRINE_WORKER=1` in the worker's spawn environment**
(the `Agent`-tool env, when available) **and** the pre-distilled prompt mandates the
worker export it as its first act. This is the same *unenforced prompt-contract*
tier as the D2b raw-tree gap: doctrine's CLI refuses authored writes once the var is
present; nothing forces the var to be present. `/dispatch` owns making it present.

**Branch-point under concurrency (D5 extended).** SL-029's single-tree check guards
*creation* (fork HEAD == source HEAD post-create). SL-031 adds the check at the
**batch-commit boundary**: the orchestrator captured base `B` pre-spawn; before
committing the imported batch it asserts coordination HEAD is **still `B`** (no
out-of-band move while the batch ran). Because the whole disjoint batch is imported
onto `B` and committed once, HEAD only moves at the orchestrator's own batch commit
— so a mismatch means an **external** mover (a stray commit / dirty tree) ⟹
**re-dispatch the batch from the new HEAD**, never commit against a moved base.

- Batches are **dependency-disjoint by construction** (the orchestrator's batching
  job) ⟹ the workers' deltas touch non-overlapping files ⟹ they co-apply onto `B`
  cleanly.
- A genuine apply **conflict** when co-applying a disjoint batch means the batching
  was wrong ⟹ **report and halt**, human re-plans (ADR-006: policy is report, never
  auto-resolve).

**Crash / overflow recovery.** Rebuild from the coordination branch + `git
worktree list`; in-flight forks are re-imported or re-dispatched. No orchestrator
state is load-bearing.

**IMP-002 reconcile.** On A's completion, `doctrine backlog edit IMP-002 --status
<terminal> --resolution <…>` (a terminal status requires a resolution) with the
rationale (substance shipped SL-032; wiring tail landed SL-031). Record the IMP-003
↔ SL-029/SL-031 follow-up for `/close`.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV — minting is trunk-union.** After A, `next_id` everywhere unions local +
  trunk ids; `next_id(local, &[])` stays byte-identical to the old behaviour
  (SL-032 INV-1) so the gate holds for repos with no trunk.
- **INV — KINDS ⟺ kind consts.** The set-equality guard test asserts `KINDS`
  membership equals the set of live numbered-kind consts; a new kind missing from
  the registry fails the test (the guarded form of R-b).
- **ASM — shared object store.** The funnel's no-transport import assumes harness
  worktree isolation uses `git worktree` (shared `.git`). Claude Code does; the
  patch fallback covers the rest.
- **EDGE — worker raw-edits main (D2b).** Unchanged residual gap; the harness does
  not confine the worker to its fork. Funnel rests on D2a (shipped) + the prompt
  contract. Deferred to ADR-008.
- **EDGE — solo-in-worktree squash-orphan (R-2).** Memory recorded on a fork branch
  is orphaned by squash-merge; convention is record-on-trunk (already in `/worktree`
  prose + `memory record` nudge).

## 6. Open Questions & Unknowns

All resolved in `/design` (2026-06-10):

- **OQ-1 — WorktreeCreate hook (A-6): DEFER.** In the funnel the orchestrator
  provisions before the worker exists (D9), so the gap A-6 closes ("worker forgot
  to provision") is unreachable. The hook is Claude-only (never dependable →
  rung-3 fallback stays mandatory), project-wide-invasive (cuts against D1), and
  reopens force-copy risk SL-029 closed by the sole-copier. Stays an open backlog
  item; revisit only on evidence.
- **OQ-2 — CLI vs skill-prose: skill-prose funnel (VA) + one verb.** Ordering and
  the dispatch/batch/recovery loop are orchestrator skill-prose (VA). The single
  mechanical seam — the branch-point compare — is a tested verb (VT). No
  funnel-driver verb (would break the SL-029 prose-orchestration precedent and
  couple doctrine to the harness spawn model).
- **OQ-3 — source-delta: the fork branch ref is the delta** (shared object store;
  §5.3). Patch-handback is the non-shared-store fallback.

## 7. Decisions, Rationale & Alternatives

- **D-scope — one slice, A→B.** Alternative (split A out) rejected: A and B share
  the `/worktree` seam and the IMP-003 closure; phasing keeps them ordered without
  the overhead of a second slice. A ships testable value first and de-risks the
  reframing.
- **D-registry — referenced view (b1), not centralized table (b2).** `integrity::
  KINDS` references the distributed kind consts. Alternative (b2: one central
  literal table the Kind consts look up via const-fn) rejected: it inverts the
  existing "each module owns its Kind" ownership for marginal gain, and the scope
  doc already specifies a *set-equality guard test* — which presumes KINDS and the
  consts are distinct (b1). Residual R-b (type system can't *force* registration
  without an exhaustive-match seam) is guarded by the test, same posture as today.
- **D-delta — commit-to-fork-branch.** Alternatives: uncommitted working-tree diff
  (fragile re untracked files; defeats the committed-HEAD branch-point compare) and
  always-patch (redundant transport over a shared store). A raw `git commit` is not
  a doctrine-mediated write, so D2a permits it.
- **D-funnel-prose — orchestration is skill-prose.** Mirrors SL-029 OQ-3 (verb only
  at the mechanical seam). The funnel touches git + project-verify + memory — no
  doctrine-internal state to enforce in Rust without a heavy driver verb.

## 8. Risks & Mitigations

- **R-1 — D2b raw-tree gap.** A worker can still raw-edit main. Mitigation: D2a CLI
  guard (shipped) + prompt contract; OS confinement deferred to ADR-008. Accepted.
- **R-2 — squash-orphan of coordination/fork-branch memory.** Mitigation:
  record-on-trunk convention + `memory record` worktree nudge. Deferred seam
  (ADR-006 Open).
- **R-3 — registry refactor breaks the engine.** Mitigation: behaviour-preservation
  gate — existing validate/reseat/run_new suites must stay green unchanged; the
  refactor is a field-relocation, not a logic change.
- **R-4 — batching error surfaces as a merge conflict.** Mitigation: report-and-halt
  (never auto-merge); the branch-point check + disjoint-batch construction make the
  clean path the default.

## 9. Quality Engineering & Validation

| Item | Class | Evidence |
|---|---|---|
| Trunk-aware minting wired | VT | e2e: two divergent worktrees mint non-colliding ids; `next_id(local,&[])` unchanged |
| Kind registry dedup (F-2/F-5) | VT | F-2 (the copy) *closed* — `KINDS` references the consts; `reseat` uses `state_dir`; existing suites green unchanged. The membership test **pins** the set; R-b (forcing a new kind in) stays *guarded, not forced* — no exhaustive-match seam exists |
| `branch-point-check` verb | VT | pure `matches(base,head)` table; e2e exit-0/1 over the built binary |
| Worker-mode refuses authored writes (D2a) | VT | already covered (`tests/e2e_worker_guard.rs`); not re-implemented |
| `/worktree mode=worker` contract | VA | skill conformance: source-only, no-degrade, commit-to-branch, report shape |
| Funnel order import→verify→commit→record (D7) | VA | `/dispatch` conformance / audit read; knowledge-trails-code |
| Branch-point under concurrency (D5) | VA + VT | verb is VT; the re-dispatch policy is VA |
| IMP-003 / IMP-002 reconciliation | VH | `/close` confirms rollup + lifecycle transitions |

Conformance basis: ADR-006 Verification bullets (funnel / D7 / D2a / branch-point /
provision exclusion).

## 10. Review Notes

### Adversarial pass 1 (2026-06-10) — integrated

- **F-b (major, reasoning flaw) — funnel cadence.** The draft ran the funnel
  *per worker* (import→verify→commit each). That contradicts D7's "incremental
  *per batch*" and breaks branch-point logic: a per-worker commit moves HEAD, so
  the next worker's delta lands on a moved base (the exact thing D5 forbids), and a
  literal branch-point check would spuriously re-dispatch every worker after the
  first. **Fixed:** per-batch atomic — import all disjoint deltas onto the captured
  base `B` (non-committing), verify once, one batch commit, then record (§5.1/§5.4).
- **F-a (moderate) — verify must precede the durable commit.** Import is now
  explicitly non-committing (`cherry-pick -n` / `git apply`), so the project verify
  runs on the combined working tree *before* the single batch commit (§5.1/§5.4).
- **F-e (moderate, real gap) — `DOCTRINE_WORKER=1` is set nowhere.** Confirmed: no
  skill/install/wiring sets it; the `main.rs` guard is inert until the worker's
  process carries the var. **Fixed:** §5.4 makes the orchestrator set it in the
  worker's spawn env + mandate it in the pre-distilled prompt — the same
  unenforced prompt-contract tier as the D2b gap. `/dispatch` owns it.
- **F-d (minor, precision) — R-b is guarded, not closed.** The membership test
  *pins* the kind set but cannot *force* a new kind in (no exhaustive-match seam).
  §9 wording sharpened; F-2 (the copy) is what's closed, not R-b.
- **F-c (verified OK) — no layering cycle.** No kind module imports `integrity`, so
  `KINDS: &[&KindIdentity]` referencing the kind consts is acyclic.
- **F-f (verified OK) — IMP-002 reconcile mechanism exists.** `doctrine backlog edit
  IMP-002 --status … --resolution …` (terminal status requires a resolution).

Open after pass 1: none blocking. The funnel remains VA — its correctness rests on
skill-prose discipline + the prompt contract, not Rust enforcement (an accepted
posture, R-1/D2b lineage).

### Adversarial pass 2 (2026-06-10) — /inquisition; findings NOT yet applied to the body

Full charge sheet: `inquisition.md`. The reframe (§2) was re-verified and **holds**
(D2a guard `main.rs:1118`, `next_id(local,trunk)`, `trunk_entity_ids`, the five
`&[]`, `KINDS`/`reseat` all stand). Four charges are **load-bearing and block lock**;
they require body edits + two open design decisions (Q2/Q3), so they are recorded
here, not silently patched. **No governance conflict** — ADR-006 sanctions every
underlying posture (D2b Negative); the heresy is the design *overstating
prompt-contract as mechanism*.

- **C-I (MAJOR, supersedes F-e) — D2a fails OPEN; the env-belt does not exist.**
  The harness `Agent` tool exposes **no env parameter** (schema: `description ·
  isolation · model · prompt · run_in_background · subagent_type`; `isolation` ∈
  `{worktree}`). So §5.4's "set `DOCTRINE_WORKER=1` in the worker's spawn env (the
  `Agent`-tool env, when available)" is **never** available on the named harness —
  the conjunction collapses to "worker self-exports the var as its first act." A
  worker that omits it runs with the doctrine CLI **fully open**. F-e's "**Fixed**"
  is withdrawn: this is **prompt-contract only**, the same tier as D2b, and fails
  open. *Body edits owed:* strike "Fixed"/"shipped enforcement" framing in
  §5.4/§3/§9; re-class the worker-write refusal as VA-activation over a VT-mechanism.
- **C-II (MAJOR) — C-I undoes deliverable A inside the funnel.** A worker that did
  not arm the var may `doctrine slice new` in-fork; its mint is blind to siblings'
  concurrent mints on the coordination branch; `import` lands the authored mutation
  → **colliding ids — the exact D3 hazard A removes**. §2's "A and B independent …
  not because B depends on A" is false: D2 (worker-sole-writer) is what keeps A's
  guarantee intact inside dispatch. *Decision owed (Q2):* make `import`
  mechanically **reject any worker delta touching `.doctrine/` authored trees** —
  the one belt that IS enforceable (greppable changed-path check), unlike the env.
  Add as R-5 in §8.
- **C-III (MAJOR) — "dependency-disjoint ⟹ file-disjoint" is unsound.** §5.4's
  co-apply guarantee rests on a false syllogism: independent tasks routinely edit
  the same file (this slice itself: the verb + minting wiring both touch
  `main.rs`/`worktree.rs`). Nothing **constructs** file-disjointness. So
  `cherry-pick -n` conflict — "report-and-halt" — is the **common case, not the
  edge**; the per-batch atomic cadence (F-b's fix) is built on it. *Decision owed
  (Q3):* batching contract = **file-disjoint** (compute pairwise changed-path
  disjointness; serial-fallback for unavoidable shared-file tasks) — name it in §5.4.
- **C-IV (MODERATE) — the set-equality guard is a tautological pin; R-b is
  unguarded.** Confirmed via Q1: **no central `Kind` const spine exists** — consts
  are scattered across modules in two types (`Kind`, `GovKind`), no enumerable
  `&[&Kind]`, and the proposed `KindIdentity` refactor creates none. The only test
  (`integrity.rs:644`) pins `KINDS` against a **hand-written 12-prefix literal**; a
  13th kind added to neither escapes **both**. §5.2/§9/§5.5's claim "membership
  equals the set of live consts; a new kind fails the test" is **false**. *Body
  edit owed:* either build the const-spine the test can reflect over, or strike the
  claim and confess R-b stays a **hand-maintained pin** (no better than today).
- **C-V (MINOR) — `branch-point-check` is misnamed** (a HEAD-stationarity assert,
  not a branch-point/merge-base compute; discharges the D5 *concurrency extension*,
  not the creation-time check SL-029 shipped). Rename or scope-note §5.2/§9.
- **C-VI (MINOR) — patch-handback fallback "covers the rest" while specifying
  nothing** (§5.3/§5.5). Specify it to the same cadence, or descope remote-agent to
  out-of-v1.
- **C-VII (MINOR, disclosed) — IMP-003 "closure" is prose, no graph edge**
  (relations v1-empty). Align §1/§7 confidence to: status-flip-with-resolution +
  prose; edge deferred.

Open decisions before lock: **Q2** (import-time `.doctrine/`-path rejection — in
SL-031?), **Q3** (file-disjoint batching contract + serial-fallback), **Q4** (§9
D2a re-class). These are `/design` calls needing the User; the inquisitor does not
improvise past them.
