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

## PHASE-04 — prepare-review sync verb (completed)

The stage-1 projection verb `doctrine dispatch sync --prepare-review --slice N`.
New module `src/dispatch.rs`; e2e `tests/e2e_dispatch_sync.rs` (VT-1/2/3/4 + EX-5).
Gate `just check` green; clippy clean; per-branch target.

**EN-2 command shape (settled at phase-plan).** `dispatch` is a NEW top-level
verb class (NOT under `worktree` — provisioning vs projection are distinct seams,
ADR-012). `DispatchCommand::Sync { slice, prepare_review, path }`. Stage selector
is a clap `#[group(required, single)]` carrying one flag now (`--prepare-review`);
PHASE-05 adds `--integrate` to the SAME group — forward-compatible, no rework.
`write_class` → `Orchestrator("dispatch-sync")`, rides the existing worker-mode
fence (EX-1, pinned by `dispatch_sync_is_orchestrator` + VT-4).

**The sync sources the ledger from the BRANCH TIP TREE, not the working
filesystem.** First-cut used `ledger::read_boundaries/read_orthogonal` (which
`std::fs::read` the working tree) — RED: the e2e runs from `main`, where the
committed `.doctrine/dispatch/064/*.toml` are absent, so phases/orthogonal-exclude
came back empty. Fix: new `git::read_path_at(root, refish, path)` (`cat-file -p
<ref>:<path>`) + a generic `read_ledger::<T: DeserializeOwned + Default>` over the
dispatch tip. This is strictly more correct — single source (the branch the verb
projects), and it works in stage-2 where there is NO checkout (design §4.1's
working-tree-free thesis). The filesystem `read_*` are the funnel's
read-modify-write side, not the sync's read side — now `cfg_attr(not(test))`
dead-in-prod until the funnel rewires (PHASE-06). **PHASE-05 must likewise
tree-read, never assume a checkout.**

**Journal committed onto the branch via plumbing (EX-2), no checkout.** New
`git::tree_with_file(base_tree, path, content)` (scratch-index `read-tree` →
`hash-object -w --stdin` → `update-index --cacheinfo` → `write-tree`, mirrors
`filter_tree`) splices `journal.toml` into the tip tree; `commit_tree` + zero-CAS
advance `dispatch/<slice>`. Pending-intent journal commits BEFORE any external
ref CAS (ADR-012 D4 ordering); a second commit records applied/verified status
(recoverability). Symmetric with stage-2 (no worktree) by construction.

**B/C composition.** B = `filter_tree(tip_tree, [.doctrine/dispatch/<slice>] ++
verified-orthogonal paths)` → `commit_tree(parent = trunk_base)`. C = per
`boundaries.toml` row in order, `filter_tree(tree_of(code_end), [.doctrine])`,
parent-chained off the previous PLANNED commit (trunk_base for the first), skip
`code_start==code_end`. External refs created via zero-oid CAS ⇒ a stale prior
`review/*`/`phase/*` is reported + journalled `failed`, NEVER clobbered (EX-5);
trunk/`edge` never touched.

**Dead-code discipline.** Deleted PHASE-03's module-blanket
`#![cfg_attr(not(test), expect(dead_code))]`; replaced with per-symbol expects on
the still-ahead-of-consumer symbols (`parse`, funnel `record_*`/`store`,
`read_journal`, the filesystem `read_boundaries/read_orthogonal`) — per
mem.pattern.lint.blanket-dead-code-suppression-masks-siblings, so a regression in
a now-live sibling (e.g. `Boundaries::parse`, now wired) still surfaces.

**Open / hand-forward.** PHASE-05 integrate (trunk/`edge` push, journal replay)
+ PHASE-06 skill alignment (wires the funnel `record_*` + claude-arm boundary
capture, clearing the remaining ledger expects). PHASE-01 ADR-006 amendment still
needs adversarial acceptance before close (F-PH01-1). `/audit` RV verbs refuse on
a worktree fork — audit from the parent tree or after integrating.

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

## PHASE-06 — skill alignment + the funnel-time recording verb

Authored as prose-only, but phase-planning (probing residual-risk #2) found the
funnel-time recording surface had **no callable verb**: `ledger::record_boundary`/
`record_orthogonal` shipped (PHASE-03) as tested `pub(crate)` dead code, and
`dispatch` exposed only `sync`. An LLM orchestrator can't call a `pub(crate)` fn, so
the claude-arm `phase/<slice>-NN` cut had no way to get `boundaries.toml` populated.
EN-1 ("skills cite real surfaces") was unmet for that surface. **/consult → user
ruled option A:** wire the verb in PHASE-06 (appended VT-2), don't defer.

**Wired:** `doctrine dispatch record-boundary --slice --phase --code-start
--code-end` (Orchestrator-classed; resolves the oids; appends a `[[boundary]]` row).
`run_record_boundary` in `dispatch.rs` reuses `resolve_commit`. Removed the now-
unfulfilled `expect(dead_code)` on `store`/`record_boundary`.

**F-1 (latent bug, fixed):** `ledger::dispatch_dir` wrote **unpadded**
`.doctrine/dispatch/64/`, but `dispatch sync` (dispatch.rs) tree-reads **padded**
`…/064/` — the funnel writer and the sync reader disagreed. Masked because the
recording surface had no production caller until now; round-trip tests stayed green
(intra-ledger, writer==reader). Padded `dispatch_dir` to the canonical 3-digit
`<slice>` (matches the `dispatch/064` branch + the sync reader). Carry to audit.

**Scope held:** `record-orthogonal` left unexposed (still dead, reason updated) —
its driver is the deferred OQ-B classifier; empty `orthogonal.toml` is the correct
conservative EXCLUDE fallback ("reviewed once, never lost"). Captured as **IMP-071**.

**Skills (EX-1..6):** dispatch router — `worktree coordinate` before batch 1 (never
the session `main` tree); funnel step 7a `record-boundary` (claude arm); conclude is
stage-1 `--prepare-review` only (project refs, remove worktree dir, keep refs, audit
from parent/root); stage-2 `--integrate` is `/close`'s post-audit act; IMP-043 re-
anchor demoted to sync-time. Both arms describe behavior against `dispatch/<slice>`
(codex/pi: native fork = phase unit, no recording; claude: fork-less, synthesized
cut). worktree — `coordinate` documented as a third creation path (markerless,
create-or-resume, regenerates phase sheets). Re-embedded via touch `src/skills.rs`;
VT-1 (e2e_claude_install/e2e_skills_symlink/e2e_worktree_stamp) + gate green.

## PHASE-07 — end-to-end proof + closure prep

**No production code** — test + closure-prep only. The verbs all shipped
(PHASE-02..06); PHASE-07 proves the *seam* and readies the audit packet.

**T1 — cohesive lifecycle test (`tests/e2e_dispatch_lifecycle.rs`, VT-1).** The
per-stage suites (`e2e_worktree_coordinate`, `e2e_dispatch_sync`) each verify one
verb in isolation; none threads the whole run nor asserts the load-bearing
*session-main-untouched* invariant. The new test threads it from one fixture:
`coordinate` (markerless coord worktree off trunk) → commit phase code + `dispatch
record-boundary` + commit the ledger ON the coord tree → `sync --prepare-review`
(asserts `review/064` + `phase/064-01` resolve from the **shared common dir**, i.e.
visible to an audit run at root) → **INVARIANT (EX-1):** `main` ref unmoved AND
`git -C <root> status --porcelain` byte-empty across the run (orchestrator wrote
the *linked* tree, never session `main`; design §6 contention #1/#2 unreachable) →
`sync --integrate --trunk` (controlled trunk fast-forward, idempotent re-run) →
**conclude (EX-2):** `git worktree remove --force` the coord dir; `dispatch/064`
+ `phase/064-01` + `review/064` refs **survive** removal (deliverables live in the
common dir, not the worktree — the bug §2 fixes vs today's GC).

- Harness note: the e2e fixtures are **non-cargo** temp repos, so `coordinate`'s
  provision = checkout + regenerate sheets (no `cargo build`) — the cohesive test
  is cheap. Reuses the `init_repo`/`run`(DOCTRINE_WORKER-removed) patterns; helpers
  duplicated per the established one-crate-per-integration-test convention.
- Watch (R1, held): the invariant asserts the **source root** tree + root `HEAD`,
  distinct from the coord worktree — testing the coord dir would pass vacuously.

**T2 — behaviour-preservation gate (VT-2).** All worktree + dispatch + memory-sync
suites green **unchanged** (89 tests: e2e_worktree_* 67, e2e_dispatch_sync 14,
e2e_memory_sync 5, e2e_memory_record_worktree 2, + lifecycle 1). Cadence untouched.
Touch+re-run applied to defeat a shared-`CARGO_TARGET_DIR` false-green.

**T4 — backlog dispositions (EX-3).**
- **IMP-041 → resolved/done.** Cleanup ownership is answered by the §2 lifecycle
  (worktree-life < branch-life): conclude removes the dir, `dispatch/`+`phase/`
  branches are KEPT until integration, `gc` reaps post-integration. Proven by T1.
- **IMP-043 → demoted, kept OPEN (not closed).** The per-batch import re-anchor is
  relocated to **sync-time target movement**: `integrate --trunk` **reports** a
  moved/non-ff trunk and refuses — it never auto-3-way-re-anchors (the
  `--allow-reanchor` capability stays unbuilt future work, now scoped to sync).
  EX-3 sanctions "closed OR demoted"; demoted = open with new scope (title still
  says "import verb" — id is identity, slug is not authoritative).
- **IMP-065 (positive coordination marker, OQ-D) + IMP-071 (record-orthogonal
  wiring) remain OPEN** as the carried follow-ups (defer-needs-backlog-before-close).

**Residual risks carried to audit (EX-4).**
- **OQ-D / D2b — marker-absence is inherited, not closed.** v1 ships markerless
  coordination creation; the orchestrator's write permission rests on
  marker-*absence*, indistinguishable-by-absence from an unstamped worker (ADR-011
  D6/M2). The D2b fence (R-5 import belt / IMP-052 post-spawn check / env-worker
  catch / bwrap-no-push) is **defence-in-depth, NOT a coverage proof** (RV-025 B3):
  it does not prove the full Orchestrator verb class (`gc`/sync) is covered. The
  real close is the `/plan` Orchestrator-verb-restriction gate + the positive
  marker (**IMP-065**), not the fence.

**Audit handoff.** `/audit` must run from the **root** tree, not this worktree —
RV review verbs refuse on a worktree fork (mem.pattern.review.rv-verbs-refuse-on-
worktree-fork). After integrate lands `dispatch/064` onto trunk, drive the RV from
root.

## Audit (RV-030) — reconciliation, code-review hold

2026-06-14: Reconciliation audit on **RV-030** (facet=reconciliation, target
SL-064), driven from root. Two external review passes folded in: codex (GPT-5.5)
adversarial invariant-attack + a human/Opus full-file read. 10 findings raised,
all dispositioned. Audit found the implementation high-quality and design-faithful
(6/8 codex-attacked invariants held outright; 89 suites green; behaviour-
preservation gate held).

**Close-gate held by one OPEN blocker — F-1.** Stage-1 `prepare_review`
(`dispatch.rs:115`) parents `review/<slice>` + `phase/<slice>-NN` on the **live**
trunk tip (`git::trunk_commit()`), where design §4.2/§4.3 specify the pinned
`trunk_base_B`. A foreign commit to `main` between `coordinate` and `sync`
(coordination worktree isolates the working tree, NOT the trunk ref) silently
reparents the projection → per-phase diffs stop being exact; the §3/IMP-043
"integrate refuses moved trunk" net does not fire (it only covers post-stage-1
movement). Latent — no e2e moves trunk mid-run. **Disposition fix-now (User-ruled):
project off `merge-base(refs/heads/dispatch/<slice>, trunk)`** (pinned fork-point,
no new ledger state); keep `trunk_commit()` only at integrate's trunk push; add a
trunk-moved-during-run e2e. → memory candidate AFTER the fix proves out.

**Dispositions:**
- F-1 (blocker, live-trunk projection) — fix-now (merge-base); **OPEN/answered**, holds close.
- F-2 (major, OQ-D marker-absence fence) — **tolerated**, verified. Documented D2b residual; real close IMP-065. (codex wrongly thought impersonation tests missing — they exist: e2e_dispatch_sync VT-4, e2e_worktree_coordinate VT-2.)
- F-3 (minor, runtime rollup 2/8 vs notes 8/8) — fix-now, verified; reconcile sheets at the lifecycle move.
- F-4 (major, commit_journal hardcodes 'journal: prepare-review' both stages) — fix-now; answered/pending. Thread a &str msg param.
- F-5 (minor, ScratchIndex cross-PID crash debris + overclaiming doc) — fix-now; answered/pending. read_dir sweep of doctrine-filter-index.* + fix comment.
- F-6 (minor, settings.local.json full parse-serialize normalize) — **tolerated**, verified. Low risk, machine-written.
- F-7 (minor, integrate/prepare_review journal-cycle duplication) — follow-up → **IMP-075**, verified.
- F-8 (minor, phase_chain_tip ignores journal status → confusing 'no code units' error) — fix-now; answered/pending. Filter status==Verified / distinguish error.
- F-9 (minor, read_path_at no dedicated unit test) — fix-now; answered/pending. Add git::tests case.
- F-10 (nit, projection_row source_oid==planned_new_oid wart) — **tolerated**, verified. Doc the intentional equality.

**Remediation route (User-ruled): /handover → fix → /close.** Outstanding before
close: apply F-1 (merge-base) + the fix-now batch (F-4/F-5/F-8/F-9 + F-10 doc +
F-3 rollup reconcile), re-gate (`just check` — note the foreign backlog-golden red
is WIP, not SL-064), then `review verify` F-1/F-4/F-5/F-8/F-9 as raiser, and `/close`.

**Out of scope.** `just check` red on `e2e_backlog_list_order_golden` = foreign
WIP (SL-059 `tags` JSON field vs stale SL-053 golden); SL-064 touches no backlog
code; its own suites green.
