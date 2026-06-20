# SL-125 — implementation notes

Durable harvest from the PHASE-01 drive + RV-111 audit. The disposable runtime
phase sheet (`phases/phase-01.md`) is `rm -rf`-able; this survives.

## What landed (PHASE-01)

ISS-011 **Defect C** fixed. `run_stamp_subagent` (`src/worktree.rs`) now derives
the provision SOURCE from the repo's **primary worktree** via new helper
`primary_worktree(cwd)` (first `worktree <path>` of `git worktree list
--porcelain`, canonicalized) instead of `root::find` on the process cwd. The
`SubagentStart` hook fires inside the worker worktree, so the process cwd is the
fork; the old code made `source == fork` and `verify_sibling_worktree` bailed →
unstamped worker. The **R1 binding anchor** (`repo = root::find`, `cwd_valid`,
`classify_stamp`, every `StampRefusal` token, the `(Some,Some)` bind) is
behaviourally unchanged — only the R2 source role moved.

- Code commit `9ce7dc0c` on `dispatch/125`; candidate `cand-125-review-001`
  (`a2320694`) = net diff vs main of exactly `src/worktree.rs` +
  `tests/e2e_worktree_stamp.rs` (+135/-8).
- VT-1 (Defect-C pin) red→green; VT-2 unit (`primary_worktree`); VT-3 refusals
  unchanged; VT-4 cross-repo `bad-dir` (codex BLOCKER closure). 2073 bin + 11 e2e
  pass; clippy zero-warnings; fmt clean.

## Drive method

Driven via `/dispatch` (claude arm) — one worker, one source-delta commit, funnel
import→verify→commit. In-flight inline WIP (already-proven RED VT-1 + helper) was
carried into the worker via `/tmp/sl125-wip.patch`.

## Standing risks / deferred (from RV-111)

- **VH-1 deferred (F-2).** Worker-comes-up-stamped-with-no-hand-stamp is
  unverifiable until this integrates to `main` AND the orchestrator binary is
  rebuilt. **Defect C was live for this very drive** — the worker came up unstamped
  (old orchestrator binary) and was hand-stamped from the primary:
  `echo '{"cwd":"<worker>","agent_type":"dispatch-worker"}' | doctrine worktree marker --stamp-subagent --path <primary>`.
  Re-run the IMP-046 fresh-session probe post-integration to close VH-1.
- **`just check` not fully green (F-1).** `lint-js` fails on a pre-existing missing
  `@eslint/js` in `web/map/` — unrelated to this `.rs`-only slice. Rust gate green.
- **Source byte-equivalence scoped (F-4 → IDE-017).** primary == orchestrator holds
  only for the current single-file `.worktreeinclude`. FU-1 captured as IDE-017.

## Dispatch tooling gotchas (this drive)

- **RSK-010, setup-time.** `dispatch setup` forks the coordination worktree off the
  trunk ladder (`origin/HEAD` first), which lagged local `main` by 36 commits and
  lacked SL-125's `plan.toml` → setup hard-aborted (`Plan for slice 125 not
  found`). Workaround: `DOCTRINE_TRUNK_REF=main` on every `setup`/`sync` (local
  main is the de-facto trunk here). Env doesn't persist across shell calls — prefix
  each command. Appended to RSK-010 this drive.
- **Sharp edge.** `dispatch setup`'s rollback reverted *uncommitted* changes in the
  **session main working tree**. Save WIP to a patch before running setup.
