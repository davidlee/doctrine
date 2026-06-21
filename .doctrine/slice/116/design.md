# SL-116 Design — Split `worktree.rs` into a submodule folder

## Status

Design reviewed — internal hostile pass + external inquisition (RV-131, codex
GPT-5.5) integrated; all 5 ledger findings disposed and the partition made
executable. Granularity (D1), test disposition (D2), and the exhaustive layout
are settled. Ready for `/plan` on user lock.

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

- **D4 — `allowlist.rs` is the PURE LEAF; impure allowlist I/O moves to
  `provision.rs`.** Only the pure core (`Tier`/`WITHHELD`/`parse_allowlist`/
  `is_withheld`/`select_copies`/`allowlist_violations`/`glob_matches`/`MATCH_OPTS`)
  stays in `allowlist.rs` — no disk, no git → ADR-001 **leaf**. The impure
  `read_allowlist` + `ALLOWLIST_FILE` (disk reads) are consumed ONLY by the two
  provision verbs (`run_provision` L757, `run_check_allowlist` L813/817), so they
  co-locate in `provision.rs` and stay **file-private** — no widening, and
  `allowlist.rs` keeps its leaf purity. (Corrects RV-131 F-1: the first draft put
  `read_allowlist`/`ALLOWLIST_FILE` in `allowlist.rs`, which both forced a
  cross-file widen AND polluted the leaf with an impure seam.)

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

## Target layout — EXHAUSTIVE item→home→visibility map

Every top-level production item in `src/worktree.rs` (42 items; `grep -nE
'^(pub\(crate\) |pub )?(fn|struct|enum|impl|trait|const|static|type) '`) is
assigned below. `[X]` external (re-exported, §Public surface); `[s]` widen
private→`pub(super)`; `[p]` stays private (single-file); unmarked = existing
`pub(crate)`, no change.

```
src/worktree/  — tier per ADR-001 (see §ADR-001 obligation)
  mod.rs       (command)  WorktreeCommand[X], dispatch[X], pub(crate) use re-exports,
                          root() helper
  shared.rs    (engine)   is_linked_worktree[X], matches, target_dir_for_branch,
                          resolve_common_dir[s], resolve_commit[s],
                          gather_tree_clean[s], gather_fork_worktree[s]
  allowlist.rs (LEAF)     Tier, impl Display(Tier), Withhold, w[p], WITHHELD,
                          DERIVED_RUNTIME, MATCH_OPTS[p], glob_matches[p], Allowlist,
                          ParseError, parse_allowlist, is_withheld, Withheld,
                          Selection, select_copies, Violation, representative[p],
                          allowlist_violations
  marker.rs    (engine)   Cause, impl Cause, StatusLine, impl StatusLine,
                          describe_mode, DISPATCH_WORKER_AGENT_TYPE[X], marker_path,
                          marker_present, env_worker_set[X], write_marker,
                          remove_marker, resolve_mode[X], DUAL_CAUSE[X], run_status,
                          run_marker_clear
  provision.rs (engine)   ALLOWLIST_FILE[p], read_allowlist[p], verify_sibling_worktree[p],
                          enumerate_candidates[p], run_provision, run_check_allowlist
  import.rs    (engine)   DOCTRINE_PREFIX[p], CLAUDE_PREFIX[p], Apply, Refusal,
                          impl Refusal, classify_import, run_import
  land.rs      (engine)   Merge, LandRefusal, impl LandRefusal, ForkState,
                          classify_land, run_land
  coordinate.rs(command)  CoordOutcome, CoordAction, CoordRefusal, impl CoordRefusal,
                          classify_coordinate, base_has_slice_plan[p], coordinate[X],
                          run_coordinate, run_branch_point_check
  gc.rs        (engine)   GcState, GcPlan, GcRefusal, impl GcRefusal, GcVerdict,
                          classify_gc, gc_target_dir[p], reap_targets[p],
                          gather_landed[p], run_gc
  fork.rs      (engine)   project_env_contract[s], remove_worktree_dir[s],
                          rollback_fork[s], run_fork
  subagent.rs  (engine)   Stamp, StampRefusal, impl StampRefusal, classify_stamp,
                          SubagentPayload[p], cwd_shares_repo[p], primary_worktree[p],
                          run_stamp_subagent, WorkerVerify, WorkerVerifyRefusal,
                          impl WorkerVerifyRefusal, classify_worker_verify,
                          run_verify_worker
  test_helpers.rs (cfg-test) git, init_repo  [pub(crate) within cfg(test)]
```

Re-homings worth flagging: `CoordOutcome` was stranded in the allowlist region
(L371) → `coordinate.rs` (its owner); `run_branch_point_check` → `coordinate.rs`
(dispatch-coordination family; uses `resolve_commit`/`matches`); `base_has_slice_plan`
is **coordinate-only** (sole caller `coordinate` L2005) → `coordinate.rs`, not
`fork.rs`; `target_dir_for_branch` (gc L1471 + fork L1761) → `shared.rs`.

`mod worktree;` in `main.rs:71` resolves `worktree/` identically — no parent
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

## Visibility — complete cross-file widen set

The only non-mechanical change: items currently file-private (`fn`/`const`, no
vis) that become cross-file once split widen to **`pub(super)`** (visible within
the `worktree` module only, **not** re-exported). Full call-site-verified set (7),
each with its consumers across the new boundaries:

| item | home | consumed by (file) |
|---|---|---|
| `resolve_common_dir` | shared | shared (`is_linked_worktree`), provision (`verify_sibling_worktree`), subagent (`cwd_shares_repo` L2370/2373) |
| `resolve_commit` | shared | coordinate (`run_branch_point_check` L879), import (`run_import` L1005) |
| `gather_tree_clean` | shared | import (L1010/1022), land (`run_land` L1255) |
| `gather_fork_worktree` | shared | land (L1270), gc (L1571), coordinate (L1977) |
| `project_env_contract` | fork | fork (L1928), coordinate (`run_coordinate` L2099) |
| `rollback_fork` | fork | fork (L1913), coordinate (L2042) |
| `remove_worktree_dir` | fork | fork (`rollback_fork` L1819), coordinate (L2043) |

Already `pub(crate)` (no change, trivially sibling-reachable): `is_linked_worktree`
(also external), `matches`, `target_dir_for_branch`.

Stay **private** — single owner file, sole production caller verified:
`w`/`glob_matches`/`representative`/`MATCH_OPTS` (allowlist); `ALLOWLIST_FILE`/
`read_allowlist`/`verify_sibling_worktree`/`enumerate_candidates` (provision);
`DOCTRINE_PREFIX`/`CLAUDE_PREFIX` (import); `gc_target_dir`/`reap_targets`/
`gather_landed` (gc); `base_has_slice_plan` (coordinate); `SubagentPayload`/
`cwd_shares_repo`/`primary_worktree` (subagent).

Rule: widen private→`pub(super)`, never `pub(crate)`, for internal items — minimal
surface, no external leak. The audit above is the checklist; `cargo build` (E0603)
catches any miss at execution.

## ADR-001 layering obligation (BINDING — in-slice)

`.doctrine/adr/001/layering.toml` is the **binding** tier map, enforced by `just
gate`'s `MixedUmbrella` assertion. It currently carries one entry,
`worktree = "command"` (@98). Splitting the module makes `worktree` a **mixed
umbrella** (files spanning altitudes), which the map's rule (@12–13) requires be
sub-classified at `module::submodule` granularity — the same treatment `catalog`
already receives (`"catalog::scan" = "command"`, `"catalog::hydrate" = "engine"`,
… @104–107).

The slice **must, in-slice**, regenerate and commit the sub-classification:

1. Run the authoritative extractor —
   `cargo test --test architecture_layering dump_real_graph -- --nocapture --ignored`
   (the same tool that generated the map, SL-112) — and enter the resulting
   `"worktree::<file>"` entries, classified by **actual imports**, not by cohesion
   labels.
2. Expected shape (to be confirmed by the extractor, not asserted here):
   `worktree` umbrella + `mod.rs` = **command**; `"worktree::allowlist" = "leaf"`
   (pure — D4); `"worktree::coordinate" = "command"`; the remaining machine files
   (`shared`/`marker`/`provision`/`import`/`land`/`gc`/`fork`/`subagent`) =
   **engine** *iff* their imports stay inward.
3. **`coordinate` is COMMAND, not engine.** `coordinate` calls
   `crate::slice::run_phases` (`src/worktree.rs:2035`) — an upward edge into the
   command-tier `slice` module. This is the exact `worktree → slice` upward edge
   named in the slice's Non-Goals; it is **why `worktree` is `command` today** and
   why `coordinate` cannot be downgraded. The split does not fix or worsen it — but
   the layering entry must tell the truth about it.

`just gate` green (incl. `MixedUmbrella`) is an **exit criterion**. Omitting this
is the heresy RV-130 F-1 (SL-133) and RV-121 (SL-132) were caught committing.

## Verification

- **Behaviour-preservation gate is the proof.** 46 tests relocate (D2), bodies
  byte-unchanged → green = correctness. (VT) **Contingent on the partition being
  complete** (RV-131 F-4): byte-identity holds only because every test co-locates
  with the file owning the private symbol it exercises, and every cross-machine
  helper it touches is in the `pub(super)` set above. Spot-confirm at execution for
  tests reaching cross-machine helpers — `primary_worktree_resolves...` (L3222,
  → subagent, private, same file ✓), `rollback_fork_retracts...` (L3255, → fork,
  now `pub(super)` ✓), `base_has_slice_plan_tracks...` (L3290, → coordinate ✓),
  `coordinate_refuses_create...` (L3314, → coordinate ✓).
- **No new production tests** — pure mechanical move. New evidence is structural
  (VA):
  - `src/worktree.rs` gone; `src/worktree/` present; `mod worktree;` in `main.rs`
    unchanged.
  - 8 caller files compile **untouched** → public surface preserved.
  - `every_runtime_gitignore_glob_is_classified` green → `DERIVED_RUNTIME` still
    reachable (it rides `allowlist.rs` tests, const + guard together).
  - `architecture_layering` green → `worktree::<file>` sub-classification present
    and accurate (the ADR-001 obligation above).
- **Gate:** `just gate` (workspace `cargo clippy` zero-warn; dead_code denied;
  `MixedUmbrella` layering assertion) + `cargo test`.

## Migration mechanics & risks

**Mechanics:** extract concern-by-concern into the new files; `mod.rs` carries
`mod` decls + `pub(crate) use` re-exports; widen the 7-item `pub(super)` set
(§Visibility); add the `worktree::<file>` layering entries (§ADR-001 obligation);
relocate each test block to its machine; delete `worktree.rs`. Git sees a content
split, not a rename — no `git mv`.

**Risks (low, mechanical):**
- **Visibility drift** — under-widen → E0603 (caught immediately); over-widen →
  leak. Mitigate: the §Visibility table is the complete checklist; `pub(super)`
  only.
- **De-interleaving** — land/coordinate/fork are interleaved today; the cut is
  clean per the exhaustive map, but `cargo build` is the backstop.
- **Re-export completeness** — the **8**-symbol checklist (§Public surface);
  `cargo build` catches a miss.
- **Layering omission** — the binding `MixedUmbrella` gate (above); not optional.
- **Phasing** is `/plan`'s call (likely 1 mechanical phase, or split by
  file-group if isolation wanted).

## Non-goals (from slice scope)

- The `worktree.rs:1742 → slice::run_phases` upward coupling edge — note, do not
  fix here.
- Any behaviour / state-machine / allowlist-semantics change.
- Splitting any other oversized module (`main.rs` → SL-115; `memory.rs` unsliced).

## Adversarial review

### Internal hostile pass (IR-n, call-site verified against `src/worktree.rs`)

- **IR-1 — re-export checklist over-stated (9→8).** `gather_tree_clean` is
  private; its only out-of-file occurrence is a doc comment in `git.rs:1168`, not
  a caller. Removed from the public surface; `shared.rs` `pub(super)`.
- **IR-2 — `matches` cross-machine confirmed** (coordinate/import/gc) → `shared.rs`.
- **IR-3 — over-widening corrected** (first draft "widen these 3"):
  `verify_sibling_worktree`/`enumerate_candidates`/`primary_worktree` are
  single-machine, stay private.
- **Scope-text note:** slice `§Scope` cites `worktree::run_phases`; it is
  `slice::run_phases` (the Non-Goal upward edge). Design checklist grep-derived,
  did not inherit the error.

### External pass — RV-131, codex (GPT-5.5), inquisitor posture

The external adversary broke the "pure mechanical" plea. Five findings, all
verified against source and integrated:

- **F-1 (blocker) — `pub(super)` widen set too small.** Draft widened 3; real
  cross-file set is **7** (added `gather_fork_worktree`, `project_env_contract`,
  `rollback_fork`, `remove_worktree_dir`; `read_allowlist`/`ALLOWLIST_FILE`
  resolved by co-locating in `provision.rs` per D4 instead of widening). →
  §Visibility table. **Fixed.**
- **F-2 (blocker) — layout not exhaustive.** 7 orphans (incl. cross-machine
  `gather_fork_worktree`, `remove_worktree_dir`). → §Target layout now enumerates
  all 42 items with home + visibility. **Fixed.**
- **F-3 (blocker) — ADR-001 `layering.toml` unamended + mis-classified.** Binding
  map carries `worktree = "command"`; split → mixed umbrella needs per-file
  sub-classification; `coordinate` is command (imports `slice`), not engine. →
  new §ADR-001 obligation. **Fixed.**
- **F-4 (major) — byte-identity claim premature.** Now stated contingent on the
  completed partition, with the four at-risk tests spot-checked. → §Verification.
  **Fixed.**
- **F-5 (minor) — stale "9-symbol checklist".** → §Migration now "8-symbol".
  **Fixed.**
- **Acquittal:** codex confirmed the external re-export surface is exactly the 8
  named symbols and `gather_tree_clean` is not external — IR-1 / §Public surface
  stand.

The ledger (RV-131) carries the structured findings and their dispositions;
synthesis sealed there.
