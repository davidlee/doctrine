# Notes SL-064: Coordination-branch isolation: dedicated worktree + integration-sync seam for dispatch

Durable per-slice scratchpad - tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Planning

2026-06-14: Authored the SL-064 executable plan and materialised runtime phase
tracking. The plan has seven phases: governance/OQ-D fence, coordination
worktree creation, projection plumbing and run ledger, prepare-review sync,
integrate/replay sync, source skill alignment, and end-to-end proof. Slice
status was advanced to `ready`; planning changes were committed in the same turn
under a `plan(SL-064)` commit.

Verification run for planning: `doctrine slice phases 064`,
`doctrine slice status 064 ready`, `doctrine slice list --filter
coordination-worktree`, `git diff --check`, and ASCII scan over the new plan
files. No code was changed, so `just check` was not run.

## PHASE-01 — Governance amendments and OQ-D fence (completed)

2026-06-14: Amended ADR-006 for SL-064 placement/identity refinements and pinned
the OQ-D fence. Doc-only; commit `7156f44`. Slice advanced `ready → started`.

Amendments (inline `... amendment/note (SL-064)` notes, SL-056-amendment style;
decision ids unchanged):
- **D8** — coordination branch runs in its own dedicated worktree (retires the
  "trunk in solo / delta branch in team" placement). Topology/projection/routing/
  recovery/D1-tightening remain ADR-012's — cross-referenced, not duplicated.
- **D2a** — orchestrator may run in a *linked* coordination worktree; permission
  rests on marker-absence, not `!is_linked_worktree`; "runs at the root" retired;
  `env DOCTRINE_WORKER` must not leak. `worker_mode` formula unchanged.
- **D2b** — marker-absence is a transitional assumption, not positive identity;
  the fence is defence-in-depth, NOT a proof of gc/sync impersonation coverage
  (RV-025 B3). OQ-D plan-gate pinned as binding obligations (Orchestrator-verb
  restriction + mandatory impersonation tests). IMP-065 = real positive-marker close.
- **D9** — markerless coordination-tree creation variant (same fork+provision
  ladder, no worker marker; regenerates the coordination/runtime tier).
- References + `updated = 2026-06-14`.

**D7 NOT amended** (handover F1 / EX-1) — git diff shows no `±` D7 line.

Verification: VT-1 — worker-guard suites (`e2e_worker_guard`, `e2e_worktree_{fork,
import,land,gc}`) green, 44 tests, unchanged (doc-only ⇒ trivially green; sentinel
against accidental src edit). VA-1/VA-2 by self-comparison sweep against `adr show
12` + design §5.

**OPEN → audit (F-PH01-1, VA-3 / handover F8):** the ADR-006 amendment has had
self-comparison only — it must reach **adversarial acceptance** (an inquisition
pass or `/audit`) before close, per the slice closure intent. Carry into PHASE-07
/ `/close`.

## PHASE-02 — Coordination worktree creation (completed)

2026-06-14: Shipped `doctrine worktree coordinate --slice <n> --dir <p>`
(`worktree::run_coordinate`, src/worktree.rs). Impl commit `a524e6f`; e2e suite
commit `9a0effe`. Driven via solo `/execute` in the `dispatch/064` coordination
worktree (`.worktrees/dispatch-064`), off pinned base `41454ac` — see the
**DISPATCH PIVOT** finding (below) for why not `/dispatch`.

Durable design decisions (EX-1..5 met):
- **Distinct thin verb, not `fork --coordination` flag.** Create/resume/regenerate
  semantics diverge from fork's "always-new-branch, refuse-if-exists". Shared
  create+provision ladder kept DRY via existing `run_provision` (sole copier) +
  new `remove_worktree_dir` (extracted from `rollback_fork`). No parallel impl.
- **Markerless** (D2a/D9): the coord tree IS the orchestrator (worker-mode OFF,
  must write); stamps NO worker marker. Write permission rests on marker-absence,
  never a positive coordination marker (that is OQ-D / IMP-065, owner-locked).
- **Base = resolved trunk, auto** (no `--base`): the coord tree's base IS the
  integration base (design §2). New `git::trunk_commit` (pub wrapper over the
  private D3 `trunk_tree_ish` ladder); hard-errors if no trunk resolves.
- **Pure classifier** `classify_coordinate(exists, has_live_worktree)` →
  `CoordAction::{Create,Resume}` / `Err(CoordRefusal::LiveWorktree)` (token
  `coordination-live`). `bears_marker` deliberately excluded — marker refusal is
  the invocation-site guard, not a branch-state discriminator.
- **EX-4 rides the Orchestrator class** — `write_class` returns
  `Orchestrator("coordinate")`; the CLI-seam worker-mode guard refuses
  marker-present / `DOCTRINE_WORKER` for free. Same fence as `fork`.
- **Post-add compensation**: Create → `rollback_fork` (drops the minted branch
  too); Resume → `remove_worktree_dir` (KEEPS the pre-existing branch).

Verification: `tests/e2e_worktree_coordinate.rs` (6 tests, VT-1..5; VT-1 split
create+rollback) + unit `classify_coordinate_create_resume_collide`,
`coord_refusal_token_distinct`. Gate `just check` green, `cargo fmt --check`
clean. Incidental: rewrote the `run_coordinate` doc block (ambiguous `3a.`/`3b.`
ordered markers, 5-space continuations) as dash bullets — newer clippy raised
`doc_overindented_list_items` post-commit.

**DISPATCH PIVOT (durable — `/record-memory` candidate at close).** The claude
`/dispatch` arm cannot drive a multi-phase sequential slice under live shared-`main`
contention: Agent-tool workers fork off the SHARED session HEAD (base-pinning
residual) and the import precond never gets a clean window (continuous foreign
tracked-dirty + HEAD moves) → livelock. Worked around by solo `/execute` in an
isolated coordination worktree off a pinned base. Keep driving every remaining
SL-064 phase this way.

## PHASE-03 — projection plumbing + run ledger (completed)

Commits `7c94b19` (plumbing T1-T4), `b4f4bff` (ledger T5-T6). Gate `just check`
green since last edit; `cargo clippy --bin doctrine` clean.

**Git plumbing (`src/git.rs`).** `run_git` now delegates to a private
`run_git_env(root, args, envs)` — the single `NORMATIVE_FLAGS` chokepoint is
preserved (EX-1, no second runner), every born-frame `capture` caller is
byte-unchanged so **VT-6 behaviour-preservation stayed green unchanged** (all 59
`git::tests` pass incl. the `forget.remote.v1`/`forget.checkout.v1` suite).
Three primitives:
- `filter_tree(root, source_tree, exclude)` — `read-tree`/`rm --cached -r -f
  --ignore-unmatch`/`write-tree` through a throwaway `GIT_INDEX_FILE`; VT-1
  asserts the live `.git/index` is **byte-for-byte unchanged**.
- `commit_tree(root, tree, parent, msg)` — `commit-tree`, no checkout (VT-2).
- `update_ref_cas(root, ref, new, old) -> RefCas::{Updated, Moved{actual}}` —
  native 3-arg `update-ref`; zero-oid `old` = creation; reports moved-target,
  never forces (VT-3).

**Throwaway index without `tempfile`.** `tempfile` is a **dev-dependency only** —
unusable in production `filter_tree`. Solved with `ScratchIndex`: a pid-named
file inside the repo's git dir (`rev-parse --absolute-git-dir`), `Drop`-removed,
absent-cleared up front. Avoided promoting `tempfile` to a runtime dep (a
dep-surface change). `/record-memory` candidate.

**Run ledger (`src/ledger.rs`, new module).** Pure read model (serde + `toml`,
mirrors `crate::plan`) + impure recording shell in one file (git.rs-style split).
Three manifests under `.doctrine/dispatch/<slice>/`: `journal.toml` (`[[row]]`),
`boundaries.toml` (`[[boundary]]`), `orthogonal.toml` (`[[mark]]`). Serialize via
`toml::to_string` ⇒ serde-escaped, no raw splicing. `record_boundary`/
`record_orthogonal` append; `read_*` default-empty on absent file (VT-4/VT-5).

**Decisions worth carrying:**
- `orthogonal.toml` row = `{ entity, path, status }` (design §4.2 underspecified
  it; ADR-012 silent). B's "journal-verified" exclusion resolves to
  `status == verified` — self-contained, no journal join key. **PHASE-04 watch:**
  if prepare-review's read-back needs to cross-reference the journal row instead,
  that's an additive field, not a rewrite.
- Table-header names (`row`/`boundary`/`mark`) are a PHASE-03 choice — design
  pins the *field* names, not the array-table names. Pinned by round-trip tests.
- Leaf-ahead-of-consumer dead_code: `#![cfg_attr(not(test), expect(dead_code,
  reason=…))]` — scoped to non-test because the `cfg(test)` round-trip tests name
  every symbol (mem.pattern.lint.dead-code-expect-vs-cfg-test). Self-clears when
  the PHASE-04 sync verb wires the first non-test caller.

PHASE-03 stayed BELOW the CLI projection surface (EN-2) — no `dispatch sync`
command, no external-ref mutation. Next = PHASE-04 (prepare-review: the
Orchestrator-classed `dispatch sync --prepare-review`, B + C consuming these).
