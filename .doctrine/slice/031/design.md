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
   │  batch = dependency-disjoint tasks
   ├─ per worker:
   │    /worktree mode=worker  → git worktree add @base → provision → baseline✓
   │    Agent isolation:worktree, DOCTRINE_WORKER=1, pre-distilled prompt (D6)
   │        worker: mutate SOURCE only → run verify cmd → git commit to fork branch
   │                → return structured report (NOT a doctrine write)
   └─ funnel per worker (D7, strict order, incremental):
        import (cherry-pick/apply fork branch — shared object store)
        → verify (project cmd on coordination branch)
        → commit (orchestrator, from coordination branch)
        → record (memory / AC evidence / notes — trails the commit)
        guard at import: branch-point-check (fork base == coordination HEAD?)
```

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
  exit 0  if base == current coordination HEAD (default: `git rev-parse HEAD`)
  exit 1  otherwise  (→ orchestrator re-dispatches; never merges against moved base)
```

Pure compare in the leaf (`fn matches(base, head) -> bool`); the git read of HEAD
is the impure shell. Read-classed (no authored write) so it is callable under
worker-mode — though only the orchestrator uses it.

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

**Funnel order (D7), per worker, incremental:** import → verify → commit → record.
Knowledge always trails the confirmed commit; the coordination branch is the
durable store; orchestrator context is disposable.

**Branch-point under concurrency (D5 extended).** SL-029's single-tree check
guards *creation* (fork HEAD == source HEAD post-create). SL-031 adds the check at
**import time**: a batch's workers all fork from base `B`; importing worker-1 moves
the coordination HEAD to `B+1`, so worker-2's base is now stale relative to HEAD.

- Batches are **dependency-disjoint by construction** (the orchestrator's batching
  job) ⟹ within-batch deltas touch non-overlapping files ⟹ sequential apply onto
  the moving HEAD is clean. The branch-point check guards the case where HEAD moved
  for a reason **outside** the batch (an unexpected external commit / dirty tree) —
  mismatch ⟹ **re-dispatch** that worker from the new HEAD, never blind-merge.
- A genuine apply/merge **conflict** within a disjoint batch means the batching was
  wrong ⟹ **report and halt**, human re-plans (ADR-006: policy is report, never
  auto-resolve).

**Crash / overflow recovery.** Rebuild from the coordination branch + `git
worktree list`; in-flight forks are re-imported or re-dispatched. No orchestrator
state is load-bearing.

**IMP-002 reconcile.** On A's completion, transition IMP-002 to done with rationale
(substance shipped SL-032; wiring tail landed SL-031); record the IMP-003 ↔
SL-029/SL-031 follow-up for `/close`.

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
| Kind registry dedup (F-2/F-5) | VT | set-equality guard (`KINDS` ⟺ kind consts); `reseat` uses `state_dir`; existing suites green unchanged |
| `branch-point-check` verb | VT | pure `matches(base,head)` table; e2e exit-0/1 over the built binary |
| Worker-mode refuses authored writes (D2a) | VT | already covered (`tests/e2e_worker_guard.rs`); not re-implemented |
| `/worktree mode=worker` contract | VA | skill conformance: source-only, no-degrade, commit-to-branch, report shape |
| Funnel order import→verify→commit→record (D7) | VA | `/dispatch` conformance / audit read; knowledge-trails-code |
| Branch-point under concurrency (D5) | VA + VT | verb is VT; the re-dispatch policy is VA |
| IMP-003 / IMP-002 reconciliation | VH | `/close` confirms rollup + lifecycle transitions |

Conformance basis: ADR-006 Verification bullets (funnel / D7 / D2a / branch-point /
provision exclusion).

## 10. Review Notes

(adversarial pass appended after the draft is accepted)
