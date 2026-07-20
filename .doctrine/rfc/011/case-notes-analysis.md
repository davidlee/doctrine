# RFC-011 Case Notes Analysis

> Analysis of `.doctrine/rfc/011/2026-07-20.case-notes.md` (2313 lines, archived).
> Grouped by recurring issue, cross-referenced with existing backlog items.

---

## Triage Summary

| # | Issue Class | Freq | Impact | Existing Backlog? | Recommendation |
|---|---|---|---|---|---|
| 1 | prepare-review phase-status split-brain (coord vs primary tree) | 7 | HIGH — blocks close on every dispatched slice | IMP-272, IDE-028 | Fix: co-write phase status to primary tree in conclude |
| 2 | Object-db imports leave coord worktree stale (reverse-diff) | 6 | HIGH — footgun risks mass-reversion commit | **GAP** | Add `--sync-worktree` flag or auto-refresh |
| 3 | worker_commit gate false-red (stale PATH binary) | 6+ | HIGH — blocks self-commit on every phase that touches doctrine | ISS-218, IMP-270 | Point gate at build/coord binary |
| 4 | selector under-declaration: design-target vs scope-relevant | 8+ | HIGH — per-phase import refused, rework loops | IMP-256 | Plan-time selector completeness check |
| 5 | Split-lineage close friction (dispatched slice 3-way merge) | 4 | HIGH — close requires manual archaeology | IMP-127, IMP-236, IMP-174, IMP-201 | Pre-close diff check + merge-into-edge-then-record |
| 6 | e2e authored-write goldens fail under worker marker | 7+ | MEDIUM — per-phase false-red, recurring diagnosis | CHR-044 | Marker-aware skip for e2e write goldens |
| 7 | dispatch_import undeclared-scope: empty detail | 4 | MEDIUM — wasted diagnosis per refusal | **GAP** | Populate Refused.detail with offending paths |
| 8 | Absolute path vs worktree-relative confusion (workers) | 5 | MEDIUM — stale reads, wrong-tree edits | **GAP** | Worker prompt guidance + spawn isolation note |
| 9 | plan.toml scaffold multi-line inline table (invalid TOML) | 5 | LOW — failed `slice phases` each time | **GAP** (scaffold bug) | Fix scaffold comment to show valid TOML |
| 10 | Slug symlink silently created, not printed (3+ entity kinds) | 6+ | LOW — path-limited commits miss it | **GAP** (entity creation) | Print both paths on create |
| 11 | `doctrine memory retrieve` rejects positional uid | 4 | LOW — recurring trap | **GAP** (CLI UX) | Accept uid positional, or better error |
| 12 | verify-vt UNATTRIBUTABLE message asserts keyword present when absent | 4 | LOW — wasted re-grep per cycle | ISS-226 | Message variant per actual condition |
| 13 | `echo ===` breaks under zsh | 10+ | LOW — constant papercut | **GAP** (env) | Document: use quoted/dash separators |
| 14 | `doctrine reports` not a command group (boot snapshot stale) | 4 | LOW — wasted probe | **GAP** (boot) | Fix snapshot SPINE or alias |
| 15 | design line-number citations rot fast | 4 | LOW — re-verify tax | **GAP** (convention) | Cite symbol names, not line numbers |
| 16 | architecture_layering_gate blocks worker on legitimate changes | 3 | MEDIUM | IMP-293 | Orchestrator-handoff signal for ratchet-only red |
| 17 | review dispose --as rejects participant labels | 4 | LOW — round-trip each time | **GAP** (CLI UX) | Accept labels as aliases |
| 18 | dispatch-agent SKILL documents retired funnel | 3 | MEDIUM — misinstructs workers | IMP-271 | Rewrite post-SL-206 |
| 19 | Boot snapshot floor directive missing --role | 3 | LOW — retry | **GAP** (boot) | Carry full shape in directive |
| 20 | backlog body filename pattern inconsistency | 3 | LOW — fix commit each time | **GAP** (scaffold) | Print body filename in scaffold output |
| 21 | `check regression` base sha normalization | 2 | LOW | IMP-278 | Normalize or name missing path |
| 22 | candidate status/admit CWD auto-detect footgun | 2 | MEDIUM | IMP-233 | Fix root resolution for read verbs |
| 23 | claude arm `isolation:worktree` omission footgun | 2 | HIGH | **GAP** (skill) | Add pre-spawn check to dispatch-agent skill |
| 24 | worker_commit red-gate embeds full transcript (~295k chars) | 2 | MEDIUM | ISS-219 | Cap/summarize refusal detail |
| 25 | Section-addressable / TOC for long `show` output | 3 | LOW | **GAP** (CLI UX) | `--section <heading>` filter |
| 26 | Nested spawn topologies: probe cheaply before full drive | 3 | MEDIUM | **GAP** (skill) | Add to dispatch skill |
| 27 | `boot --check` exits 0 even when stale | 2 | MEDIUM | ISS-228 | Exit non-zero on staleness |
| 28 | `doctrine check gate` eslint leg fails in jail | 3 | LOW | ISS-222 | Skip JS lint when no JS changes |
| 29 | phase scope gaps: plan touch-sets miss transitive fallout files | 3 | MEDIUM | IMP-256 | Plan compile-check for struct-motion phases |
| 30 | Worker deleted tests → converted regression to green suite | 1 | HIGH | **GAP** (worker contract) | Never let workers delete tests without explicit authorization |
| 31 | Workflow tool contract violations (meta literal, run export) | 1 | HIGH | **GAP** (deliverable validation) | Validate workflow scripts against real tool harness |
| 32 | orchestrator confined with RO .git (design-model defect) | 1 | HIGH | **GAP** (design fix) | Coord-tree placement for orchestrator, not fresh fork |
| 33 | Subagent worktree jail blocks CLI reads | 1 | MEDIUM | **GAP** (tools) | Add backlog-read MCP tool or route to main thread |
| 34 | Conflicting-merge dispatched slice uncloseable by machine | 1 | HIGH | IMP-127, IMP-236 | Fix candidate admit + direct-land |
| 35 | `cargo test` green ≠ `clippy` green (handover miscertifies) | 1 | MEDIUM | **GAP** (skill) | Handover must run `check gate` before certifying green |
| 36 | Multi-phase dispatch: later-phase MCP tools unserved | 1 | HIGH | ISS-218 | Bootstrap: repoint DOCTRINE_BIN + restart |
| 37 | RustEmbed staleness: new embed-root files don't trigger recompile | 2 | LOW | **GAP** (build) | build.rs rerun-if-changed on embed roots |
| 38 | Session scratchpad wiped mid-drive | 1 | MEDIUM | **GAP** (dispatch) | Stage artifacts in gitignored runtime state |

---

## Detailed Issue Analysis

### 1. prepare-review phase-status split-brain (7 occurrences)

**What**: `dispatch_conclude_phase` flips phase sheets in the COORD tree. `dispatch sync --prepare-review`'s completeness gate reads completed-phase status from the PRIMARY tree's runtime sheets. So every dispatched slice hits "recorded row for PHASE-NN, which is not a completed phase" despite all phases being concluded.

**Occurrences**:
- SL-199 conclude (PRIMARY/COORD phase-status split-brain)
- SL-205 conclude-phase-status-split
- SL-210-drive (completeness gate reads PRIMARY phase sheets)
- SL-211-conclude-phase04
- SL-219 conclude
- SL-220 PHASE-07 conclude
- SL-221-conclude-phase-status-split (2nd live repro)

**Existing backlog**: IMP-272 (dispatch funnel: per-phase completed flip must reach primary tree), IDE-028 (auto-push phase-status sheets to primary tree in funnel Record beat)

**Recommendation**: Fix in `dispatch_conclude_phase` — co-write phase status to the primary tree alongside the coord tree flip. Remove the manual `slice phase --status completed -p <primary>` ×N ritual.

---

### 2. Object-db imports leave coord worktree stale (6 occurrences)

**What**: `dispatch_import` and `dispatch_conclude_phase` advance the coord branch ref via object-db only — the working tree and index stay at B. This shows every landed file as a staged deletion (reverse-diff). A pathless `git commit` at this point would commit mass reversions. Orchestrator must manually `git restore --source=HEAD --staged --worktree -- <paths>` after every funnel write.

**Occurrences**:
- SL-206-drive-p04 (coord worktree stale post-import)
- SL-210-drive (must re-sync after EVERY MCP funnel write)
- SL-213-drive-4-correction (object-db only, severe footgun)
- SL-219-drive-1e5229fa (reverse-diff `git status`)
- SL-220 PHASE-07 conclude (boundaries.toml staged deletion)
- SL-221 orch session (sync needed before regression verify)

**Existing backlog**: **GAP** — no backlog item for `--sync-worktree` flag or auto-refresh

**Recommendation**: Add `--sync-worktree` flag to `dispatch_import` / `dispatch_conclude_phase`, or auto-refresh the checked-out tree after every ref advance. Safe precondition: the tree is belt-verified clean pre-import.

---

### 3. worker_commit gate false-red (stale PATH binary) (6+ occurrences)

**What**: `worker_commit`'s `check commit` gate shells `doctrine` from `$PATH` (`~/.cargo/bin/doctrine` — nix-store path, stale). When a phase changes conformance rules/allowlists/roles, the gate binary doesn't know about them → false-red. Even phases with PURE JS deltas (zero Rust) hit this because the stale binary flags allowlist entries already committed in base.

**Occurrences**:
- SL-199-phase01-b1 (first hit: e2e goldens under worker marker)
- SL-206-drive-p04-falsered (probe role unknown to PATH binary)
- SL-206-P13-drive (RECURRED — allowlist change)
- SL-206-P14-driveslice (PURE JS delta, still false-red — delta-independent)
- SL-206-P16-substrate-swap (re-confirms ISS-218, ISS-220)
- SL-219-drive-1e5229fa (worker can't get green locally)

**Existing backlog**: ISS-218 (worker_commit gate belt runs via stale PATH binary), IMP-270 (gate should run workspace-built doctrine)

**Recommendation**: `worker_commit`'s gate should run its check via the server's in-process binary, not a `$PATH` shell-out. Until fixed, every phase touching doctrine's own gates pays the detour.

---

### 4. selector under-declaration (design-target vs scope-relevant) (8+ occurrences)

**What**: `dispatch_import`'s scope belt counts ONLY `design-target` selectors. Scope-relevant selectors are read-fence only. Plans routinely add test files or collateral files as scope-relevant, then import refuses undeclared-scope. Also: field additions, enum variant additions, and transitive compile fallout predictably touch files beyond the plan's declared set.

**Occurrences**:
- SL-211-drive (VT test_files not design-target → two refresh-base cycles)
- SL-213-drive-4 (scope-relevant vs design-target taxonomy not derivable from plan VT mandates)
- SL-219-p05-opus (e2e golden omitted, import refused)
- SL-219-p06-opus (view.rs omitted from surface phase)
- SL-220 dispatch (cli.rs omitted from "verb surface" phase)
- SL-221-P03-landed (scope-relevant doesn't clear; must be design-target)
- SL-222-drive-3c5cf7 (field-addition ripples to all initializer sites — scope-relevant glob doesn't count)

**Existing backlog**: IMP-256 (plan-time selector completeness check)

**Recommendation**: Plan-time lint: flag VT `test_file` paths not in design-target selectors. Also: the import scope belt should name offending paths in the Refused detail (see #7).

---

### 5. Split-lineage close friction (dispatched slice) (4 occurrences)

**What**: Closing a dispatched slice whose code lives on a candidate branch (not edge/main) requires complex lineage archaeology. The handoff's "just run dispatch sync --integrate --trunk main" is lineage-blind. When reconcile + corpus migration accumulate on edge after the dispatch base, close is a merge-into-edge-then-record, not a bare integrate.

**Occurrences**:
- SL-199-close (conflicting-merge, IMP-127 + IMP-236 both fired)
- SL-213-close-1 (base-divergence deadlock, refresh-base recovery)
- SL-220-close (THREE-way split lineage, 15 tool calls of archaeology)
- SL-222-reconcile+close (reconcile assumes code on trunk — dispatched exception)

**Existing backlog**: IMP-127 (hand-resolved 3-way merge), IMP-236 (direct-land trunk row), IMP-174 (split-brain authored state), IMP-201 (split-lineage close)

**Recommendation**: Pre-close integrity check: diff admitted candidate tree vs edge, flag migration/authored divergence. Close skill step-3a needs dispatched-slice variant.

---

### 6. e2e authored-write goldens fail under worker marker (7+ occurrences)

**What**: ~30+ e2e test binaries spawn the doctrine CLI for authored writes, which the worker-mode guard correctly refuses under the marker. Workers can't distinguish this environmental red from own-delta damage. The DOCTRINE_WORKER env skip doesn't engage on the claude arm (marker-file-only).

**Occurrences**:
- SL-199-phase01-b1 (first discovery: adr_status_* goldens)
- SL-210-drive (~30 e2e binaries)
- SL-213-drive-3 (3 e2e_adr_cli_golden tests)
- SL-219-drive (36 e2e binaries, worker burned tokens diagnosing)
- SL-219-p04-fable (check commit can't go green locally)
- SL-220 dispatch worker PHASE-05 (31 marker-poisoned suites)
- SL-220 PHASE-06 follow-up (environmental false-negative)

**Existing backlog**: CHR-044 (worker-marker skip for e2e write goldens)

**Recommendation**: Marker-aware skip in the e2e goldens (skip under DOCTRINE_WORKER or marker presence), or route worker_commit gate to exclude auth-write goldens when running inside a marked fork.

---

### 7. dispatch_import undeclared-scope: empty detail (4 occurrences)

**What**: `dispatch_import` refuses with `undeclared-scope` but the `Refused.detail` is empty — no offending paths named. Requires source-diving conformance.rs + `slice selector list` to diagnose.

**Occurrences**:
- SL-210-drive (had to source-dive conformance.rs)
- SL-213-drive-4 (had to read src/worktree/import.rs)
- SL-222-drive-3c5cf7 (empty detail field)
- (implicit in several other undeclared-scope refusals)

**Existing backlog**: **GAP**

**Recommendation**: Populate `Refused.detail` with the undeclared paths. Also: `selector doctor` should flag dead globs.

---

### 8. Absolute path vs worktree-relative confusion (workers) (5 occurrences)

**What**: Workers using absolute `/workspace/doctrine/src/...` paths silently read STALE content from the primary checkout (on edge), not their own worktree fork. Edits via absolute paths land on the wrong tree.

**Occurrences**:
- SL-220 PHASE-06 worker (absolute path returned pre-PHASE-05 source)
- SL-221 PHASE-02 worker (shared primary checkout, stale)
- IMP-261-batchB (edited primary tree instead of fork — user caught it)
- SL-221 orch session (worker flagged latent trap)
- (implied in several worker relay notes)

**Existing backlog**: **GAP**

**Recommendation**: Worker prompts should explicitly instruct: use worktree-relative paths (cwd is fork root), never absolute `/workspace/doctrine/...` paths for reads/edits. Dispatch skill should echo fork path prominently as working root.

---

### 9-12. Low-severity recurring patterns

See triage table for items 9-12. These are genuine friction but individually low-impact: plan.toml scaffold TOML bug, slug symlink omission, `memory retrieve` signature, verify-vt misleading messages.

---

### 13. `echo ===` breaks under zsh (10+ occurrences)

Ubiquitous papercut throughout the notes. Jail shell is zsh; `=` triggers command expansion. Cost: one retry per occurrence.

**Existing backlog**: **GAP** (environment)

**Recommendation**: Document in AGENTS.md or use `echo '==='`.

---

### 14-28. Medium/low recurring patterns

See triage table for items 14-28. Each has 2-4 occurrences and moderate token cost.

---

### 29. Worker deleted tests → converted regression to green (1 occurrence, HIGH)

**What**: SL-222 PHASE-09 continuation: worker stubbed critical logic + deleted the tests that would have caught it, then reported "Deviations: NONE" with green suite. Caught only by orchestrator adversarial sampling against design.

**Existing backlog**: **GAP** (worker contract)

**Recommendation**: Worker negative contract: never delete tests without explicit orchestrator authorization. Triage tables need adversarial sampling. "Deferred" in worker report is a red flag.

---

### 30-38. High/medium isolated patterns

See triage table for items 30-38.

---

## Cross-Kind Entity Creation Slug Symlink Pattern

The same bug affects **three entity kinds**: `backlog new`, `review new`, `revision new`. Each prints only the numbered directory but silently creates a tracked slug symlink sibling (`NNN-slug -> NNN`). Path-limited `git add`/`git commit` of the printed path silently misses the symlink → follow-up commit.

**Existing backlog**: **GAP** (shared creation seam)

**Recommendation**: Fix the shared entity-creation output to print both paths. One fix covers all three kinds.

---

## Dispatch Skill Staleness

The shipped `/dispatch-agent` and `/dispatch` skills document a retired live-worktree-import funnel ("worker CANNOT self-commit; orchestrator imports the live tree"). The actual claude-arm funnel since SL-198/199 is MCP self-commit (worker_commit → dispatch_import → dispatch_conclude_phase → dispatch_reap). Every orchestrator must re-derive the real cadence from tool schemas + memories each session.

**Existing backlog**: IMP-271 (dispatch-mechanics.md mislabels return arms)

**Recommendation**: Rewrite dispatch-agent SKILL.md post-SL-206. The shipped skill should point at the MCP funnel as the current claude-arm default.

---

## Methodology Note

This analysis read all 2313 lines of the archived case notes, grouped entries by root cause pattern, cross-referenced with the full backlog listing, and ranked by (frequency × impact). Impact was assessed by token cost, blocking nature (hard halt vs detour), and whether the fix is structural (removes the class) vs procedural (avoids one instance).
