# SL-111 — dispatch notes

## Progress (dispatch via claude arm)

Coordination branch `dispatch/111`, base B₀ = `30e2fb67` (origin/main at setup).

| Phase | Status | Coord commit | Notes |
|---|---|---|---|
| PHASE-01 Leaf kinds module | ✅ completed | `9b36f15d` | `src/kinds.rs` + `mod kinds;`. Membership test green. |
| PHASE-02 Re-key relation engine | ✅ completed | `83a12e04` | `relation.rs` element type → `&str` from `kinds::*`; 20 `&crate` aliases + groupings dropped; 7 ADR-001 cycles broken (closure grep empty). 90 relation tests green. |
| PHASE-03 Single-source command prefixes | ✅ completed | `d82df33f` | 23 command `*_KIND` const `prefix:` fields → `crate::kinds::<X>` across 12 modules. EX-2 (`relation_graph.rs:1615`) already done in PHASE-02; EX-3 (`integrity::KINDS`) inherited. 2268 tests green. |

Plus `9d8d468b` — `fix(SL-111): cargo fmt relation.rs` (PHASE-02 fmt drift: import
order `REC`/`RECORD` + a closure not one-lined; branch-hygiene, no behaviour change).

Coordination HEAD: `9d8d468b`. Branch gate-clean (fmt + clippy zero-warning;
`lint-js` fails environmentally — no `web/map/node_modules` in jail, change is Rust-only).

### PHASE-03 carry-forward
- **PHASE-03 EX-2 is already done.** `relation_graph.rs:1615` test-helper accessor took the
  mechanical `&str` edit in PHASE-02 (forced — it was a compile error blocking the relation
  suite). PHASE-03 covers only: re-point each command `*_KIND` const's `prefix:` field to
  `kinds::<X>` (EX-1, ~20 consts), and the `integrity::KINDS` no-op (EX-3, inherits transparently).
- PHASE-02 worker used idiomatic `.contains()` / `.to_vec()` rather than literal `*k == …` /
  `.copied().collect()` — plain `cargo clippy` (the gate) **denies** the literal forms
  (`manual_contains`, `iter_cloned_collect`). Apply the same idiom in PHASE-03.

## PHASE-03 update — root cause of finding #1 corrected (ISS-031)

PHASE-03 (2026-06-19, fresh session) ran the **canonical** claude arm —
`dispatch-worker` Agent with `isolation: worktree`, base by cwd-placement — and it
forked B **correctly** (worker commit `S^ == 0ff72b6f`, single commit, 2268 green).
The difference from PHASE-01/02 was not concurrency: it was **coordination-tree
placement**.

- Real root cause (ISS-031): the harness confines the persistent Bash cwd to the
  primary working dir (`/workspace/doctrine`). The PHASE-01/02 coord tree was an
  **outside sibling** (`/workspace/doctrine-dispatch-111`), so `cd` into it reverted
  every call → cwd stuck on `main` → `isolation:worktree` forked `main`. Finding #1's
  "advancing main, uncontrollable" was that symptom, not the cause.
- Fix: relocated coord **inside** the project (`dispatch setup --dir
  /workspace/doctrine/.dispatch/SL-111`). cwd parks persistently; the standard arm
  works — no `fork --worker`/no-isolation dance needed. **Prefer this** over finding
  #1's workaround.
- `verify-worker` still refuses `unstamped` (Agent worktrees carry no marker) — proved
  base==B directly by git (`HEAD^==B`, `rev-list --count B..HEAD==1`, `is-ancestor`).
- `import` newline bug (finding #2 below) recurred; same manual `git apply` workaround.

Findings filed: **ISS-031** (coord placement), **ISS-032** (import newline bug),
refining **ISS-029**; memory `mem.pattern.dispatch.claude-arm-coord-placement`.

## Dispatch tooling findings (durable — see also memory)

The claude arm of `/dispatch` has two sharp edges under **concurrent `main` churn**
(other agents committing to `main` throughout this session):

1. **Agent `isolation:worktree` forks off the session's *advancing* `main`, not off the
   coordination base B.** PHASE-01's harness worktree based at `11094dc9` (main HEAD at spawn),
   not B. _[PHASE-03 correction above: the true cause is coord-tree placement outside the
   cwd-jail (ISS-031), not concurrency. With coord inside the jail the standard arm forks B.]_ Two consequences:
   - The worker's single commit had `S^ = main`, not `S^ = B`, so `worktree import` (requires
     `S^==B`) and `verify-worker` (requires a stamped marker — harness worktrees are unstamped)
     both refuse. Worked around by `git rebase --onto B <main-base> <branch>` to replant the
     lone commit on B.
   - **Dependent phases can't see prior phases' output.** PHASE-02 needs PHASE-01's `kinds.rs`,
     which lives only on `dispatch/111`, never on `main`. A harness worktree off `main` would not
     compile. **Resolution (user-approved):** for dependent phases, pre-create the worker tree with
     `doctrine worktree fork --worker --base <coordHEAD> --branch worker/111/PHASE-NN --dir <path>`
     (stamped, off B, carries prior phases), then spawn the `dispatch-worker` Agent **without**
     `isolation:worktree`, bound by prompt to that dir (absolute paths only). Then `verify-worker`
     + the `S^==B` delta check pass cleanly. **Use this pattern for PHASE-03.**

2. **`doctrine worktree import` has a stdin-piping bug.** It fails `corrupt patch at <stdin>:<N>`
   where N = last line of its internally-captured `git diff B..fork`, for BOTH phases (well-formed
   patches that `git apply --3way --index --check` accepts cleanly from a file). Worked around by
   replicating import's exact operation manually: `git diff B..<fork> > p.patch` then
   `git apply --3way --index p.patch` into the coordination index (non-committing) — after
   confirming all of import's guards by hand (HEAD==B, clean tree, single non-merge commit `S^==B`,
   no `.doctrine/`/`.claude/` touch). **Candidate bug to file against the doctrine CLI.**

3. **Worker-marker confinement refuses `.doctrine/` CLI writes inside a stamped fork.** PHASE-02's
   `e2e_adr_cli_golden` / `e2e_relation_migration_storage` failed in the fork with
   `worker fork (signal: marker): refusing authored write` — they scaffold entities via the
   `doctrine` CLI. **Not regressions:** they pass on the markerless coordination tree (10 + 6 green).
   Always re-run the full verify (incl. e2e) on the coordination tree after import, not just trust
   the fork's result.

## Worktrees / refs outstanding (for cleanup at conclude)
- `worker/111/PHASE-02` branch + `/workspace/wt-111-p02` worktree — spent, landed by patch (not
  ancestry). `worktree gc` ancestry leg won't hold; patch-id leg may. Left intact for now.
- PHASE-01 harness worktree `/workspace/doctrine/.claude/worktrees/agent-a4beecb50bd0ff76c`
  (branch `worktree-agent-…`) — spent; harness-managed.
