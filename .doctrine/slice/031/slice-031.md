# Dispatch orchestrator funnel: worker-mode workers and import-verify-commit-record

## Context

This slice realises **IMP-003's funnel half** — the OQ-1 split deferred by
**SL-029** — under **ADR-006** (worktree posture, orchestrator-sole-writer).
SL-029 shipped the *lifecycle* (detection D1, creation ladder D5/D9,
`doctrine worktree provision`/`check-allowlist`, the `/worktree` skill's **solo**
side, commit-before-spawn + single-tree branch-point). This slice builds the
**worker-mode-ON funnel** on top of that lifecycle: the orchestrator-sole-writer
discipline (D2/D6/D7), worker-mode workers (D6a), and filling the `/dispatch`
placeholder.

The governing decisions mechanised here: the **worker-sole-writer invariant**
(D2) — workers mutate source only, return a structured report + source delta,
never commit doctrine state; **orchestrator pre-distill** (D6); **funnel
discipline** (D7) — incremental per-batch persistence in strict order
*import delta → verify → commit → record knowledge* on the coordination branch
(D8); the **branch-point check extended to the concurrent case** (D5); and
**worker-mode ON** for funnel workers (D6a, the half SL-029 left stubbed).

**Prerequisite already satisfied (reframed in `/design`).** IMP-002 (worker-mode
guard `DOCTRINE_WORKER=1` (D2a) + trunk-ref minting / reseat (D3)) was named as the
*open* execution prerequisite — but its substance **shipped under SL-032** (D2a
guard + `tests/e2e_worker_guard.rs` PHASE-01; D3 trunk minting PHASE-02; validate +
reseat PHASE-03). The gate is open: SL-031 is **not** execution-blocked. The only
residue is the 5 `&[]` minting placeholders (tagged `SL-031 §5.4`). IMP-002 the
backlog item is stale-open and is **reconciled/closed by this slice**. The original
A-1 ("blocked until IMP-002 lands") is retired. Reuses the `/worktree` skill SL-029
landed; fills `plugins/doctrine/skills/dispatch/SKILL.md` (placeholder).

## Scope & Objectives

- **Worker contract (D2/D6a).** Implement the `mode=worker` side of the
  `/worktree` skill (SL-029 landed solo only): worker-mode ON, worker mutates
  **source only**, **commits the source change to its fork branch** (the branch ref
  is the delta — OQ-3), returns a **structured report** (the returned message), and
  never commits doctrine state. Doctrine-mediated authored writes refuse under
  worker-mode (D2a guard, shipped SL-032; a raw source `git commit` is not a
  doctrine-mediated write, so it is permitted).
- **Orchestrator pre-distill (D6).** The worker receives a self-contained prompt
  — policy digest, design excerpts, pre-fetched memories, task spec, mandatory
  verification command. Workers do **not** read boot/governance or run `/boot`.
- **Funnel discipline (D7).** The orchestrator persists incrementally per batch
  in strict order **import delta → verify → commit → record knowledge** on the
  coordination branch (D8); knowledge always trails confirmed code; crash/overflow
  recovery = rebuild from coordination branch + `git worktree list`.
- **Branch-point check under concurrency (D5).** Extend SL-029's single-tree HEAD
  pre/post compare to the concurrent-batch case: a HEAD mismatch at import time →
  re-dispatch rather than silently merge against a moved base.
- **`/dispatch` skill (D5).** Fill the placeholder: mandatory isolation via the
  harness `Agent` isolation mechanism, orchestrator-sole-writer remit, the funnel
  loop above, and the recovery prose.
- **Shared kind-identity registry (SL-032 review F-2/F-5).** Wiring trunk-aware
  minting means each `*::run_new` must resolve its kind's `dir` to call
  `git::trunk_entity_ids` — making SL-031 the **second consumer** of per-kind
  identity (prefix/dir/stem) that `integrity::KINDS` already hand-copies from the
  owning modules' `entity::Kind` consts. Build the single registry both consumers
  derive from (closing the R-b silent-escape: a new numbered kind absent from the
  table escapes `validate`), and let `KindRef` **carry the runtime-state dir**
  instead of a `has_runtime_state` bool with a hardcoded `.doctrine/state/slice`
  (F-5). Do this here — SL-031 is the consumer that fixes the registry's shape;
  doing it in SL-032 would guess the shape blind, then reshape. Add the
  set-equality guard test (`KINDS` ⟺ the `Kind` consts) as part of the registry.
- **Deterministic worker provisioning at the harness seam (IMP-003 A-6, candidate
  — design-gated).** A **Claude-only, opt-in** `WorktreeCreate` hook that runs
  `doctrine worktree provision <fork>` when a worker spawns with
  `isolation: "worktree"`, closing the "relies on the worker remembering to
  provision" gap. **Provision stays the sole copier** — the hook only *guarantees
  it runs*, it is never a second copy path. An *optimisation over* the portable
  rung-3 fallback (`git worktree add` + provision), never a dependency. Whether
  this lands here or defers is OQ-1.

## Non-Goals

- **IMP-002 machinery** — the worker-mode guard (D2a) and trunk-ref minting /
  reseat (D3). Prerequisite, owned by IMP-002; this slice *consumes* it.
- **The SL-029 lifecycle half** — detection, creation ladder, `provision`/
  `check-allowlist`, solo `/execute` isolation. Already shipped; reused, not
  rebuilt.
- **Raw-tree confinement (D2b)** — OS-enforced worker confinement; deferred to
  ADR-008's bwrap spike. The funnel rests on the CLI guard (IMP-002) + prompt
  contract.
- **Project-local jail concerns (ADR-008)** — per-worktree `CARGO_TARGET_DIR`,
  bwrap, `sccache`.
- **Adversarial-review ledger (ADR-007)** — the orthogonal single-tree primitive.
- **Anchor-stability seam (ADR-006 Open)** — moving the memory anchor off the
  volatile branch sha; deferred until squash-orphaning proves common.

## Affected surface

- `plugins/doctrine/skills/dispatch/SKILL.md` — **fill placeholder**: orchestrator
  funnel, worker spawn via `Agent` isolation, the import→verify→commit→record
  loop, branch-point-under-concurrency, crash recovery. Authored in `plugins/`,
  not the gitignored `.doctrine/skills/` install copy.
- `plugins/doctrine/skills/worktree/SKILL.md` — implement the `mode=worker`
  contract (SL-029 stubbed it; solo shipped).
- `plugins/doctrine/skills/execute/SKILL.md` — clarify the worker-vs-solo boundary
  only if the funnel contract requires it (D6a already drawn in SL-029).
- `src/{slice,governance,spec,backlog,requirement}.rs` — wire the 5 `&[]` minting
  placeholders to `git::trunk_entity_ids(&root, KIND.dir)?` (production trunk-aware
  minting; the SL-032 §5.4 tail).
- `src/entity.rs` — `KindIdentity { prefix, dir, stem, state_dir }` embedded on
  `Kind` (folds `GovKind.stem`, adds `state_dir`).
- `src/integrity.rs` — `KINDS` references the kind consts (closes F-2); `reseat`
  reads `state_dir` (closes F-5); set-equality guard test (`KINDS` ⟺ kind consts).
- `src/worktree.rs` + `src/main.rs` — new `doctrine worktree branch-point-check`
  verb (OQ-2 mechanical seam; Read-classed).
- **No install/boot wiring** — OQ-1 deferred (no WorktreeCreate hook this slice).
- **Tests** — two-worktree non-colliding mint (VT); registry set-equality (VT);
  `branch-point-check` exit-0/1 (VT). Funnel ordering + worker contract are **VA**
  (skill conformance), not VT.

## Risks, assumptions, open questions

- **A-1 — RETIRED.** IMP-002's substance shipped under SL-032 (see Context); the
  slice is not execution-blocked. Replaced by **R-3** (registry refactor must keep
  existing suites green — behaviour-preservation gate).
- **OQ-1 — RESOLVED → DEFER.** The WorktreeCreate hook (A-6) is deferred: in the
  funnel the orchestrator provisions before the worker exists (D9), so the gap the
  hook closes is unreachable. Claude-only, project-wide-invasive, reopens force-copy
  risk. Stays an open backlog item. (design §6)
- **OQ-2 — RESOLVED → skill-prose funnel (VA) + one verb.** Ordering and the
  dispatch/batch/recovery loop are orchestrator skill-prose (VA). The single
  mechanical seam is a tested `doctrine worktree branch-point-check` verb (VT). No
  funnel-driver verb. (design §5.2 / §6)
- **OQ-3 — RESOLVED → fork branch ref is the delta.** Worker commits source to its
  fork branch; the shared object store makes import a local git op (no transport).
  Report = returned message. Patch-handback is the non-shared-store fallback.
  (design §5.3 / §6)
- **R-3 — registry refactor breaks the engine.** Folding `stem`/`state_dir` into one
  identity surface and pointing `integrity::KINDS` at the kind consts must keep
  validate/reseat/run_new suites green unchanged (behaviour-preservation gate).
- **R-1 — D2b residual gap.** A worker can still raw-edit main (the harness does
  not confine it to its worktree, ADR-006 D2b); the funnel rests on the CLI guard
  (IMP-002) + prompt contract. Known, deferred to ADR-008.
- **R-2 — squash-merge orphans coordination-branch memory anchors** (ADR-006
  Consequences). Convention: record on trunk/coordination branch. Deferred seam.

## Verification / closure intent

Done when: **(A)** trunk-aware minting is wired at all 5 `run_new` sites
(two-worktree non-colliding mint, VT) and the kind-identity registry is deduped —
`KINDS` references the consts, `reseat` uses `state_dir`, the set-equality guard
passes, existing suites green unchanged (F-2/F-5, VT); IMP-002 is reconciled to
done. **(B)** the `mode=worker` contract is implemented (source-only, commits to
its fork branch, returns a structured report, no degrade-to-in-place) with
doctrine-mediated writes refusing under worker-mode (D2a, already covered, VT); the
orchestrator funnel runs **import → verify → commit → record** in strict order per
batch (D7) — **VA** (skill conformance, not a unit test); the branch-point check
under concurrency ships as the tested `branch-point-check` verb (D5, VT) driving the
re-dispatch policy (VA); and `/dispatch` ships **filled** with the
orchestrator-sole-writer remit + crash/recovery prose. OQ-1 is resolved (deferred,
with rationale). ADR-006's funnel / D7 / D2a / branch-point-under-concurrency
Verification bullets are the conformance basis.

## Follow-Ups

- **Close IMP-003** once both SL-029 (lifecycle) and this slice (funnel) land;
  link IMP-003 ↔ SL-029 / SL-031 (backlog→slice relations are empty in v1 — the
  registry does not yet exist).
- **ADR-008 bwrap spike** — discharges the D2b raw-tree-confinement gap this slice
  rests on.
- **Anchor-stability seam** (ADR-006 Open) — if squash-orphaning of coordination-
  branch memory proves common.
