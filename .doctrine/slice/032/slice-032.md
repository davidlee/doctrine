# Worker-mode CLI guard and trunk-ref id allocation with reseat

## Context

This slice builds the **doctrine-mediated enforcement half** of ADR-006's
worker-sole-writer invariant (**D2a**) plus **fork-safe id allocation** (**D3**).
It is the **prerequisite the funnel slice (SL-031) consumes** — named as such by
ADR-006, IMP-003, and SL-029. Pure CLI/engine work; no skill prose.

The governing decisions mechanised here:

- **D2a — worker-mode guard.** `DOCTRINE_WORKER=1` makes the CLI **hard-refuse**
  every doctrine-mediated authored write — exactly the writes that mint ids or
  anchor memory. Doctrine owns its CLI, so this is enforceable.
- **D3 — minting is a trunk-side act.** Ids allocate against the configured
  **trunk ref** (auto-detect `origin/HEAD` → `main`/`master`, overridable) so a
  worktree forks *after* its ids exist trunk-side. Cross-branch offline collisions
  are caught by detect (`validate`) + a reseat verb.
- **ADR-006 amendment — solo-in-worktree warning.** `memory record` detects
  worktree context (`GIT_DIR != GIT_COMMON`) and warns on squash-orphan risk,
  nudging record-on-trunk.

D2b (raw-tree confinement) is explicitly **out** — the harness does not confine
workers; that is OS-enforcement work for ADR-008. This slice gives the CLI-side
guarantee only.

## Scope & Objectives

- **Worker-mode guard (D2a).** Honour `DOCTRINE_WORKER=1` at a single chokepoint
  (OQ-1): hard-refuse the doctrine-mediated authored writes — `memory record`,
  entity creation (`slice/adr/spec/backlog new` and any id-minting verb), authored
  status transitions, and doctrine-driven commits. Clear refusal message; nonzero
  exit. Read paths stay open (workers read freely, ADR-006 D2).
- **Trunk-ref id allocation (D3).** Allocate ids against the configured trunk ref,
  not the working-tree scan, so concurrent worktrees mint disjoint ids. Trunk ref
  auto-detected via the ladder `origin/HEAD` → `main` → `master`, overridable
  (OQ-2). Composes with — does not replace — the existing
  `entity.rs::allocate_fresh` collision-retry seam (OQ-5).
- **Detect (D3 — `validate`).** A duplicate-id detector across the entity corpus
  (cross-branch offline collisions). **No `validate` command exists today** — this
  slice introduces the detect surface (OQ-3 settles its shape).
- **Reseat (D3 fallback).** A verb that renumbers a colliding entity — its dir,
  `mem.*`/id symlink, and the `id` field — and surfaces inbound prose citations
  that need hand-fixing (relations are outbound-only prose in v1; OQ-4).
- **Memory-record worktree warning (amendment).** `memory record` warns under
  worktree context, **reusing SL-029's `worktree.rs` detection seam** — no
  parallel `GIT_DIR != GIT_COMMON` implementation.

## Non-Goals

- **Raw-tree confinement (D2b)** — OS-enforced worker confinement (a worker
  hand-editing main or running bare `git commit`). Not CLI-stoppable; deferred to
  ADR-008's bwrap spike (tracked as the D2b residual gap).
- **The orchestrator funnel (SL-031)** — import→verify→commit→record, worker
  spawn, `/dispatch`. *Consumes* this slice's guard + minting; does not build them.
- **Worktree lifecycle (SL-029, done)** — detection ladder, `provision`/
  `check-allowlist`, the `/worktree` skill. Reused (the warning rides its
  detection), not rebuilt.
- **Anchor-stability seam (ADR-006 Open)** — moving the memory anchor off the
  branch sha. Deferred until squash-orphaning proves common.

## Affected surface

- `src/entity.rs` — `allocate_fresh` / the id-allocation seam: source candidate
  ids from the **trunk ref** rather than (or unioned with) the local working-tree
  scan. Shared engine machinery — behaviour-preservation gate applies (R-1).
- `src/git.rs` — trunk-ref resolution (`origin/HEAD` → `main`/`master` ladder,
  override) and reading entity ids present at that ref (e.g. `git ls-tree`).
  Reuses existing remote-selection / git helpers.
- **Worker-mode guard module + wiring** — a single gate honouring
  `DOCTRINE_WORKER=1`, checked in `src/main.rs` dispatch (or a thin guard seam the
  write verbs share). Chokepoint is OQ-1.
- **`validate` + `reseat` surface** — new subcommand(s) in `src/main.rs`; detect
  in the engine (`entity.rs`/`registry.rs`); reseat renumber logic.
- `src/memory.rs` — worktree-context warning on `record`, calling the
  `worktree.rs` detection.
- **Trunk-ref config** — where the override lives (env / project config / flag) is
  OQ-2.
- **Tests** — guard refuses each write class under `DOCTRINE_WORKER=1`; trunk-ref
  allocation across simulated worktrees; trunk-absent fallback; `validate` flags a
  planted duplicate; reseat renumbers cleanly; existing allocation suites stay
  green unchanged (R-1).

## Risks, assumptions, open questions

- **R-1 — behaviour-preservation gate.** Id allocation is shared entity-engine
  machinery; the existing `allocate_fresh` suites are the proof and must stay
  **green unchanged** (CLAUDE.md gate). Trunk-ref allocation must not alter
  solo-mode behaviour.
- **R-2 — trunk ref absent.** No remote / fresh repo / detached HEAD must degrade
  gracefully to local allocation — the solo path stays untouched (ADR-006 D1
  spirit). The detect ladder must have a defined no-trunk terminus.
- **R-3 — reseat vs outbound-only prose citations.** Relations are prose-only in
  v1 (ADR-004), so renumbering cannot reliably auto-rewrite inbound citations
  across the corpus; reseat must *surface* danglers, not silently leave them.
- **OQ-1 — guard chokepoint.** Single gate in `main.rs` dispatch vs a shared seam
  each write verb calls. Exact write set = the D2a list (mint/anchor writes).
- **OQ-2 — trunk-ref resolution + override.** Auto-detect ladder is given; the
  override mechanism (env var / project config / `--trunk-ref` flag) and *how* ids
  are read at the ref (ls-tree the ref vs require it checked out) are open.
- **OQ-3 — `validate` surface.** New top-level command vs riding an existing verb;
  v1 scope = duplicate-id detection only, or broader?
- **OQ-4 — reseat reference-rewriting scope.** Renumber dir/symlink/`id` only and
  report danglers, vs assisted prose-citation rewrite. Bounded by R-3.
- **OQ-5 — allocation composition.** How trunk-side candidate ids compose with the
  existing local collision-retry seam (union? trunk-authoritative with local
  retry as backstop?).

## Verification / closure intent

Done when: under `DOCTRINE_WORKER=1` the CLI hard-refuses each doctrine-mediated
authored write class (memory record, entity new, status transitions, commits) with
a clear message + nonzero exit, and read paths stay open — test-asserted per class
(ADR-006 D2a Verification bullet); ids allocate against the configured trunk ref
with the auto-detect ladder + override, and degrade to local allocation when no
trunk ref exists, without disturbing the existing allocation suites (R-1, R-2);
`validate` flags a planted duplicate id (ADR-006 D3 detect-half bullet) and the
reseat verb renumbers a colliding entity and surfaces inbound citations (R-3);
`memory record` warns under worktree context, reusing the SL-029 detection
(amendment bullet). ADR-006's D2a / D3 / amendment Verification bullets are the
conformance basis.

## Follow-Ups

- **Unblocks SL-031** (the orchestrator funnel) — once this lands, SL-031's
  execution dependency clears.
- **ADR-008 bwrap spike** — discharges the D2b raw-tree-confinement gap this slice
  scopes out.
- **Anchor-stability seam** (ADR-006 Open) — if squash-orphaning proves common.
