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

- **Worker-mode guard (D2a).** Honour `DOCTRINE_WORKER=1` at a central `main()`
  gate (OQ-1) over a pure, **exhaustive** `write_class(&Command)` classifier:
  hard-refuse **every** authored/memory/runtime mutation — a deliberate superset of
  D2a's mint/anchor list (entity `new`, `slice design/plan/notes/phases/phase`,
  status transitions, `memory record/verify`, `spec req add`, `backlog edit`, sync/
  install writers) — honouring the broader D2 ("workers write none of it"). Clear
  verb-named refusal; nonzero exit. Read paths stay open (workers read freely, D2).
- **Trunk-ref id allocation (D3).** Union local ids with **trunk-side committed
  ids** at the existing injected `scan` seam (a pure `next_id(local, trunk)`), so a
  minter does not collide with ids already on trunk. Best-effort **reduction**, not
  a lock — workers never mint (guard refuses); unpushed concurrent branches still
  collide → reseat backstops. Trunk peeled via the ladder `DOCTRINE_TRUNK_REF` →
  `origin/HEAD` → `main` → `master` (`^{commit}`); read by `ls-tree` on the
  **un-prefixed** `kind.dir` (already carries `.doctrine/`).
- **Detect (D3 — `validate`).** A new top-level verb scanning **each kind's**
  namespace (ids are per-namespace): dir==id, no intra-kind duplicate, alias target
  equality. **No `validate` command exists today.**
- **Reseat (D3 fallback).** `reseat <CANONICAL_REF>` (e.g. `SL-031`, never a bare
  id) renumbers the canonical-id triple (dir, `id` field, alias) to the next free
  id; refuses an occupied `--to` or an id with live runtime phase state; surfaces
  inbound prose citations as danglers (no auto-rewrite — outbound-only prose, v1).
- **Memory-record worktree warning (amendment).** `memory record` warns under
  linked-worktree context via a **new** shared `worktree::is_linked_worktree`
  helper — `worktree.rs` ships no self-detection seam today (only sibling
  verification), so this adds and shares one, not "reuses".

## Non-Goals

- **Raw-tree confinement (D2b)** — OS-enforced worker confinement (a worker
  hand-editing main or running bare `git commit`). Not CLI-stoppable; deferred to
  ADR-008's bwrap spike (tracked as the D2b residual gap).
- **The orchestrator funnel (SL-031)** — import→verify→commit→record, worker
  spawn, `/dispatch`. *Consumes* this slice's guard + minting; does not build them.
- **Worktree lifecycle (SL-029, done)** — provisioning ladder, `provision`/
  `check-allowlist`, the `/worktree` skill. Not rebuilt. (The new
  `is_linked_worktree` helper is shared *with* the provision path, but the
  lifecycle itself is out.)
- **Anchor-stability seam (ADR-006 Open)** — moving the memory anchor off the
  branch sha. Deferred until squash-orphaning proves common.

## Affected surface

- `src/entity.rs` — extract pure `next_id(local, trunk)`; widen `materialise` with
  a `trunk_ids: &[u32]` data param (Fresh arm only). Shared engine machinery —
  behaviour-preservation gate applies (R-1). No new impurity added.
- `src/git.rs` — `trunk_tree_ish` (peeled ladder) + `trunk_entity_ids(root,
  kind_dir)` via `ls-tree` on the un-prefixed `kind.dir`. Reuses existing git
  helpers.
- `src/main.rs` — pure exhaustive `write_class(&Command)` + the `DOCTRINE_WORKER`
  gate before the dispatch match; new `validate` + `reseat` subcommands.
- `src/worktree.rs` — new shared `is_linked_worktree(root)` (git-dir ≠
  git-common-dir).
- `src/memory.rs` — worktree-context warning on `record`, calling
  `is_linked_worktree`.
- **Override env** `DOCTRINE_TRUNK_REF` — documented; no config file (D1).
- **Tests** — exhaustive `write_class` table; per-`Write` refusal under
  `DOCTRINE_WORKER=1`; pure `next_id` table; trunk-ahead allocation + un-prefixed
  path regression + bad-`^{commit}` hard-error; `validate` flags dir/id mismatch,
  intra-kind dup, mis-targeted alias; reseat renumbers + guards; memory warning
  under a linked worktree; existing allocation suites green unchanged (R-1).

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
All five resolved in `design.md` (§6/§7); summarised here:
- **OQ-1 (resolved).** Central `main()` gate over a pure **exhaustive**
  `write_class` — superset of the D2a list (design D6).
- **OQ-2 (resolved).** Override = `DOCTRINE_TRUNK_REF` env (no config file); ids
  read by `ls-tree` on a peeled `^{commit}` tree-ish, no checkout.
- **OQ-3 (resolved).** New top-level `validate`, scanning **each kind's** namespace
  (ids per-namespace — not cross-kind).
- **OQ-4 (resolved).** Reseat renumbers the triple + reports danglers; no
  auto-rewrite (R-3).
- **OQ-5 (resolved).** Union at the existing injected `scan` seam, via pure
  `next_id`.
- **OQ-6 (PHASE-03 detail).** Exact `validate` rule set may tighten; not
  load-bearing for SL-031.

## Verification / closure intent

Done when: under `DOCTRINE_WORKER=1` the CLI hard-refuses **every** `Write`-classed
verb (exhaustive `write_class` — entity new, slice design/plan/notes/phases, status
transitions, memory record/verify, req add, backlog edit, sync/install) with a
verb-named message + nonzero exit, and `Read` verbs stay open — test-asserted per
class (D2a); ids union local ∪ trunk via pure `next_id`, with the peeled
`^{commit}` ladder + `DOCTRINE_TRUNK_REF` override, degrading to local allocation
when no trunk exists, without disturbing the existing allocation suites (R-1, R-2);
`validate` flags an intra-kind duplicate / dir-id mismatch / mis-targeted alias
(D3 detect-half) and `reseat SL-NNN` renumbers the triple, guards occupied-`--to`
and live-phase-state, and surfaces inbound citations (R-3); `memory record` warns
under a linked worktree via the new `is_linked_worktree` helper (amendment).
ADR-006's D2a / D3 / amendment Verification bullets are the conformance basis.

## Follow-Ups

- **Unblocks SL-031** (the orchestrator funnel) — once this lands, SL-031's
  execution dependency clears.
- **ADR-008 bwrap spike** — discharges the D2b raw-tree-confinement gap this slice
  scopes out.
- **Anchor-stability seam** (ADR-006 Open) — if squash-orphaning proves common.
