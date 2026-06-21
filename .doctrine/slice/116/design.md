# SL-116 Design — Split `worktree.rs` into a submodule folder

## Status

Design draft for adversarial review. Granularity, test disposition, and the
target layout are user-approved (this session). Verification + migration mechanics
follow.

## Problem

`src/worktree.rs` is 3539 lines (~2674 production L1–2674, ~864 test L2675–3539,
46 tests in one `mod tests`) — folder-shaped: one file bundling four distinct
concerns the 2026-06-19 architecture audit flagged SPLIT-CANDIDATE. The
lifecycle machines are physically **interleaved** (land split by coordinate; fork
between gc and coordinate; coordinate in two regions) — a single file lets that
rot. The project has an established submodule-folder convention (`catalog/`,
`priority/`, `map_server/`, `estimate/`).

This is a **pure mechanical cohesion split, behaviour-preserving**. No behaviour,
state-machine, or allowlist-semantics change.

## Decisions

- **D1 — per-machine granularity (chosen over the scope's coarse `lifecycle.rs`).**
  Each lifecycle state machine gets its own file. A single `lifecycle.rs` lands
  ~1200 lines even after extracting shared helpers — it reproduces the very
  folder-shaped smell being fixed. Per-machine matches the audit's own
  `enum Refusal` / `classify_*` (pure) / `run_*` (shell) triplet decomposition,
  yields single-responsibility files (max `gc.rs` ~405 lines), and the file count
  (~11) is in-house norm (`priority/`, `map_server/` run 6–8).
  - Alternatives weighed: **A** scope-default 4 files (rejected: `lifecycle.rs`
    too big); **C** two sub-family files `worktree_disk` + `integrate` (the
    pragmatic fallback if per-machine proves too granular).

- **D2 — tests co-locate per machine (T1).** Each machine file carries its own
  `#[cfg(test)] mod tests`; machine-specific test helpers (`fork_state`→land,
  `gc_state`→gc, `gitignore_representative`/`classified`→allowlist,
  `commit_slice_plan`→coordinate) ride their machine. Cross-cutting
  `git`/`init_repo` → a `#[cfg(test)]` `test_helpers.rs` (precedent:
  `catalog/test_helpers.rs`). Relocation ≠ change: bodies and assertions are
  byte-identical, so the behaviour-preservation gate holds.
  - Alternative **T2** (single `tests.rs`) rejected: reproduces the smell in
    tests and divorces tests from code.

- **D3 — `WorktreeCommand` enum + `dispatch` live in `mod.rs`.** The command
  surface sits at the folder root (conventional: `priority/mod.rs`,
  `map_server/mod.rs` keep verbs/router at root). `dispatch` routes to each
  machine's `run_*`.

- **D4 — allowlist-bound helpers stay with the allowlist, not in `shared.rs`.**
  `read_allowlist`, `ALLOWLIST_FILE`, `glob_matches`, `MATCH_OPTS` are
  allowlist-cohesive → `allowlist.rs`. `shared.rs` holds only the genuinely
  cross-machine worktree/git helpers.

- **D5 — `CoordOutcome` moves to `coordinate.rs`.** Today it is stranded in the
  allowlist region (L371); the split returns it to its owner.

- **D6 — marker + status/mode are one concern → `marker.rs`** (the audit's
  "marker / env worker-confinement" concern, incl. `Cause`/`StatusLine`/
  `describe_mode`).

- **D7 — subagent stamping + worker verify stay together in `subagent.rs`.**
  The audit's concern 4 is one concern with two classify/run sub-machines that
  share the subagent/worker-identity domain (`SubagentPayload`, `cwd_shares_repo`).
  ~400 lines, cohesive — kept as one file rather than splitting `stamp.rs` +
  `worker_verify.rs`. (D1 per-machine targets the lifecycle *family*; concern 4 is
  a single domain.)

## Target layout

```
src/worktree/
  mod.rs        WorktreeCommand enum + dispatch router + pub(crate) use re-exports
                (public surface) + shared root() helper
  shared.rs     genuinely cross-machine helpers (call-site verified, F-3):
                is_linked_worktree, resolve_common_dir, resolve_commit,
                matches (branch ref-eq), gather_tree_clean
  allowlist.rs  Tier/Withhold/WITHHELD/DERIVED_RUNTIME/MATCH_OPTS/glob_matches/
                Allowlist/ParseError/parse_allowlist/is_withheld/Selection/
                select_copies/Violation/allowlist_violations + ALLOWLIST_FILE/
                read_allowlist
  marker.rs     Cause/StatusLine/describe_mode + marker_path/marker_present/
                env_worker_set/write_marker/remove_marker/resolve_mode/DUAL_CAUSE/
                run_status/run_marker_clear + DISPATCH_WORKER_AGENT_TYPE
  provision.rs  run_provision, run_check_allowlist + verify_sibling_worktree,
                enumerate_candidates (private, single-machine — F-3)
  import.rs     Apply/Refusal/classify_import/run_import
  land.rs       Merge/LandRefusal/ForkState/classify_land/run_land
  coordinate.rs CoordOutcome/CoordAction/CoordRefusal/classify_coordinate/
                coordinate/run_coordinate/run_branch_point_check
  gc.rs         GcState/GcPlan/GcRefusal/GcVerdict/classify_gc/reap_targets/run_gc
  fork.rs       target_dir_for_branch/project_env_contract/rollback_fork/run_fork/
                base_has_slice_plan
  subagent.rs   Stamp/StampRefusal/classify_stamp/SubagentPayload/cwd_shares_repo/
                run_stamp_subagent + WorkerVerify/WorkerVerifyRefusal/
                classify_worker_verify/run_verify_worker + primary_worktree
                (private, single-machine — F-3)
  test_helpers.rs  #[cfg(test)] cross-cutting git/init_repo
```

**ADR-001 layering:** allowlist pure core = leaf; machines = engine; `mod.rs`
command = command tier. Sibling deps point inward (machines → `shared`/`allowlist`);
no cycle. `mod worktree;` in `main.rs` resolves the folder identically — no parent
change.

## Public surface (re-export checklist)

`mod.rs` must `pub(crate) use` the **8** externally-consumed symbols (8 caller
files: `main.rs`, `boot.rs`, `review.rs`, `commands/guard.rs`, `commands/cli.rs`,
`memory.rs`, `git.rs`, `dispatch.rs`):

`WorktreeCommand`, `dispatch`, `DISPATCH_WORKER_AGENT_TYPE`, `DUAL_CAUSE`,
`is_linked_worktree`, `resolve_mode`, `env_worker_set`, `coordinate`.

**F-1:** `gather_tree_clean` is NOT external — it is private (`fn`, no vis) and the
sole out-of-file mention is a doc comment in `git.rs:1168`, not a call. It stays
internal (`shared.rs`, `pub(super)`), consumed by `run_import` + `run_land`.

Miss a real symbol → caller breaks at `cargo build` (the re-export proof).

## Visibility

The only non-mechanical change: helpers currently file-private (`fn`, no vis) but
used by **multiple** machines once split widen to **`pub(super)`** (visible within
the `worktree` module only, **not** re-exported). Call-site-verified set (F-3):
`resolve_common_dir`, `resolve_commit`, `gather_tree_clean`. (`is_linked_worktree`
and `matches` are already `pub(crate)`.)

Single-machine helpers stay **private** in their owner file — no widening:
`verify_sibling_worktree` + `enumerate_candidates` (provision-only),
`primary_worktree` (subagent-only). Rule: widen to `pub(super)`, never
`pub(crate)`, for internal helpers — minimal surface, no external leak. Any
helper the move reveals as cross-machine joins the `pub(super)` set; the call-site
audit is re-run at execution.

## Verification

- **Behaviour-preservation gate is the proof.** 46 tests relocate (D2), bodies
  byte-unchanged → green = correctness. (VT)
- **No new production tests** — pure mechanical move. New evidence is structural
  (VA):
  - `src/worktree.rs` gone; `src/worktree/` present; `mod worktree;` in `main.rs`
    unchanged.
  - 8 caller files compile **untouched** → public surface preserved.
  - `every_runtime_gitignore_glob_is_classified` green → `DERIVED_RUNTIME` still
    reachable (it rides `allowlist.rs` tests, const + guard together).
- **Gate:** `just gate` (workspace `cargo clippy` zero-warn; dead_code denied) +
  `cargo test`.

## Migration mechanics & risks

**Mechanics:** extract concern-by-concern into the new files; `mod.rs` carries
`mod` decls + `pub(crate) use` re-exports; widen the shared private helpers to
`pub(super)`; relocate each test block to its machine; delete `worktree.rs`. Git
sees a content split, not a rename — no `git mv`.

**Risks (all low, mechanical):**
- **Visibility drift** — under-widen → compile error (caught immediately);
  over-widen → leak. Mitigate: `pub(super)` only for internal helpers.
- **De-interleaving** — land/coordinate/fork are interleaved today; clean
  separation may surface a helper used by two machines → `shared.rs`.
- **Re-export completeness** — the 9-symbol checklist; `cargo build` catches a
  miss.
- **Phasing** is `/plan`'s call (likely 1 mechanical phase, or split by
  file-group if isolation wanted).

## Non-goals (from slice scope)

- The `worktree.rs:1742 → slice::run_phases` upward coupling edge — note, do not
  fix here.
- Any behaviour / state-machine / allowlist-semantics change.
- Splitting any other oversized module (`main.rs` → SL-115; `memory.rs` unsliced).

## Adversarial review

Internal hostile pass (call-site verified against `src/worktree.rs`):

- **F-1 — re-export checklist was over-stated (9→8).** `gather_tree_clean` is
  private; its only out-of-file occurrence is a doc comment in `git.rs:1168`, not
  a caller. Removed from the public surface; relocated to `shared.rs` `pub(super)`
  (real callers `run_import` + `run_land`). Integrated.
- **F-2 — `matches` placement confirmed.** Production callers in coordinate
  (branch-point), import, gc → genuinely cross-machine → `shared.rs`. No change.
- **F-3 — over-widening in the visibility list.** `verify_sibling_worktree`,
  `enumerate_candidates` (provision-only) and `primary_worktree` (subagent-only)
  are single-machine — they stay private in their owner files, contradicting the
  first draft's "widen these 3". Corrected widen set:
  `{resolve_common_dir, resolve_commit, gather_tree_clean}`. Integrated.
- **Scope-text note (not a design defect):** slice `§Scope` cites
  `worktree::run_phases`; `run_phases` lives in `slice.rs` (`slice::run_phases`) —
  the upward edge named in Non-Goals. The design's checklist is grep-derived and
  did not inherit the error.

Residual: exact `pub(super)` set is execution-time call-site truth; the
preliminary set above is the checklist, re-audited during the move.

_Next: optional `/inquisition` or external adversarial pass, else `/plan`._
