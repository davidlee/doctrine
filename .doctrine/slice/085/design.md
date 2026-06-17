# SL-085 — Design: Push dispatch drive loop into CLI

## 1. Current vs target behavior

### Current

The dispatch subsystem spans ~909 lines across 4 skill files:

| File | Lines | Content |
|---|---|---|
| `dispatch/SKILL.md` | 321 | Drive loop, batching rules, funnel cadence, handover cadence, red flags |
| `dispatch-agent/SKILL.md` | 162 | Claude spawn template, boundary recording, verify-worker, residuals |
| `dispatch-subprocess/SKILL.md` | 119 | Codex/pi spawn template, fork+env contract, confined bwrap profile |
| `worktree/SKILL.md` | 307 | Fork lifecycle, provision model, marker state machine, import/land/gc |

The deterministic machinery already lives in CLI verbs (`worktree coordinate`,
`worktree fork`, `worktree import`, `worktree branch-point-check`, `worktree gc`,
`dispatch sync`, `dispatch candidate`, etc.). What bloats the skills is
**orchestration knowledge the orchestrator must carry**: which worktree, which
base, what the funnel order is, how to determine the next phase, when to conclude.

### Target

Three new CLI verbs replace paragraphs of skill prose:

| Verb | Replaces |
|---|---|
| `dispatch setup <SL-ID> --dir <path>` | The "Set up once — the coordination worktree" section + base-capture |
| `dispatch plan-next <SL-ID>` | The "Plan the next unit" step — phase ordering, status checking |
| `dispatch status <SL-ID>` | The "repeat until done" awareness + the conclude checklist |

The existing verbs (`worktree import`, `worktree branch-point-check`, etc.) stay
as-is — the orchestrator calls them directly, the same as today, but with less
surrounding prose because `setup` and `plan-next` already told it exactly what to
do. There is no `dispatch import` convenience wrapper (it saves zero tokens; the
verb is already a one-liner).

The skill files shrink to:

| File | Target lines | Content |
|---|---|---|
| `dispatch/SKILL.md` | ~40 | When to use, `dispatch setup` → spawn → funnel → `dispatch plan-next` loop, conclude, red flags |
| `dispatch-agent/SKILL.md` | ~25 | Spawn template: `Agent` tool, `subagent_type`, hook stamp, `verify-worker`, boundary recording, red flags |
| `dispatch-subprocess/SKILL.md` | ~20 | Spawn template: `worktree fork --worker`, `env -C` / bwrap, env contract |
| `worktree/SKILL.md` | unchanged | Fork lifecycle is already self-contained |

## 2. `dispatch setup` (new)

### Signature

```
doctrine dispatch setup --slice <N> --dir <path>
```

### Behavior

1. Resolve project root.
2. Read `<root>/.doctrine/slice/<N>/plan.toml`. Fail if absent or zero phases:
   `"no plan for SL-<N>; run 'doctrine slice plan <N>' first"`.
3. Delegate to `worktree coordinate --slice <N> --dir <path>` — the existing
   create-or-resume logic. A live worktree already on `dispatch/<slice>` is
   refused (`coordination-live`); a branch with no live worktree resumes.
4. Resolve the dispatch ref tip (`refs/heads/dispatch/<nnn>`).
5. Compute the trunk base: `git merge-base dispatch/<nnn> trunk`.
6. Emit env contract on stdout, one `KEY=value` per line (same shape as
   `worktree fork`'s contract). Human status on stderr.

### Env contract

```
coordination_dir=/path/to/dispatch/085
base=abc123def456...
slice=85
dispatch_ref=refs/heads/dispatch/085
trunk_base=def789abc012...
```

`base` is the tip of `dispatch/<slice>` post-setup — the `B` the orchestrator
captures. `trunk_base` is the merge-base with trunk at setup time (for drift
detection later).

### Code impact

- New `DispatchCommand::Setup` variant in `main.rs`.
- New `dispatch::run_setup(path, slice, dir)` entry point (~50 lines):
  1. Resolve root.
  2. Read + parse plan.toml via `Plan::parse`; gate on non-empty phase list.
  3. Call `worktree::coordinate(root, slice, &dir)` — a new pure-core function
     factored out of `worktree::run_coordinate`. Returns a `CoordOutcome` struct
     carrying the dispatch ref tip and trunk base.
  4. Emit the env contract on stdout.
- `worktree.rs` refactor: extract `fn coordinate(root: &Path, slice: u32, dir: &Path)
  -> Result<CoordOutcome>` from `run_coordinate`. `run_coordinate` wraps it with
  CLI I/O (unchanged behavior). `run_setup` wraps it with the env contract.
- New type `CoordOutcome { dispatch_tip: String, trunk_base: String }`.

### Verification

- **VT-1:** `dispatch setup --slice 85 --dir /tmp/coord` on a slice with a plan
  creates the coordination worktree, prints the env contract with all five keys,
  and exits 0.
- **VT-2:** `dispatch setup` on a slice with no plan.toml exits non-zero with
  `"no plan for SL-NNN"`.
- **VT-3:** `dispatch setup` on a slice whose coordination worktree is already
  live exits non-zero with `"coordination-live"`.
- **VT-4:** `dispatch setup` on a slice with a coordination branch but no live
  worktree resumes (reattaches) and prints the env contract.

## 3. `dispatch plan-next` (new)

### Signature

```
doctrine dispatch plan-next --slice <N> [--json]
```

### Behavior

Read-only. Reads plan.toml + runtime phase sheets, prints the ordered phase
rollup and identifies the next actionable phase(s).

1. Read plan.toml → ordered phase list (ids, names).
2. For each phase, read `.doctrine/state/slice/<N>/phases/<phase-id>.toml` to
   get its runtime status. Absent runtime sheet → `pending`.
3. Print the rollup table (id, status, name).
4. Identify `next`: all consecutive non-completed, non-blocked phases
   starting from the first one in plan order. A single `in_progress` phase
   gates — subsequent `pending` phases are excluded (don't start new work
   until the current phase lands). If the first non-completed phase is
   `in_progress`, `next` contains only that phase (resume). If all
   remaining phases are blocked, `next` is empty.

   Examples:
   - PHASE-03 `pending`, PHASE-04 `pending`, PHASE-05 `blocked` →
     `next: ["PHASE-03", "PHASE-04"]`
   - PHASE-03 `in_progress`, PHASE-04 `pending` → `next: ["PHASE-03"]`
   - PHASE-03 `blocked`, PHASE-04 `pending` → `next: ["PHASE-04"]`

### Output (human)

```
PHASE-01  completed   Templates & scaffold-output guard
PHASE-02  completed   Detection-gap closure
PHASE-03  blocked     Agent guidance
PHASE-04  pending     Memory recording

next: PHASE-04
```

When `next` is empty (all remaining blocked):

```
PHASE-01  completed   Templates & scaffold-output guard
PHASE-02  completed   Detection-gap closure
PHASE-03  blocked     Agent guidance

next: (none — all remaining phases are blocked)
```

### Output (--json)

```json
{
  "phases": [
    {"id": "PHASE-01", "name": "Templates & scaffold-output guard", "status": "completed"},
    {"id": "PHASE-02", "name": "Detection-gap closure", "status": "completed"},
    {"id": "PHASE-03", "name": "Agent guidance", "status": "blocked"},
    {"id": "PHASE-04", "name": "Memory recording", "status": "pending"}
  ],
  "next": ["PHASE-04"]
}
```

### Non-goal: file-disjointness

The plan.toml schema has no `files` field per phase. Adding one is an authored
schema change out of scope for this slice. The orchestrator still runs
`/phase-plan` for each candidate phase to get the file set and task breakdown.
`plan-next` replaces the mechanical "which phase is next" question, not the
`/phase-plan` expansion step.

### Code impact

- New `DispatchCommand::PlanNext` variant in `main.rs`.
- New `dispatch::run_plan_next(path, slice, json)` entry point (~60 lines):
  1. Resolve root.
  2. Read plan.toml via `Plan::parse`.
  3. Enumerate `state/slice/<N>/phases/` — for each phase in plan order, read
     the tracking TOML to get status. Reuses `state::PhaseStatus` and the
     existing `state` module's stem-reading paths.
  4. Build the ordered rollup, compute `next`.
  5. Render via the shared phase-table renderer (human) or serialize to JSON.
- Pure-core: no git, no refs — readable from anywhere.
- Shared renderer `fn render_phase_table(phases: &[(id, name, status)]) -> Table`
  used by both `plan-next` and `status`.

### Verification

- **VT-1:** `plan-next --slice 85` on a slice with 4 phases (2 completed, 1
  blocked, 1 pending) prints the rollup with correct statuses and `next: PHASE-04`.
- **VT-2:** `plan-next --slice 85` where all non-completed phases are blocked
  prints `next: (none — all remaining phases are blocked)` and exits 0.
- **VT-3:** `plan-next --slice 85` on a slice with no plan.toml exits non-zero.
- **VT-4:** `plan-next --slice 85 --json` emits valid JSON with the `next` array.

## 4. `dispatch status` (new)

### Signature

```
doctrine dispatch status --slice <N> [--json]
```

### Behavior

Read-only full dispatch rollup: coordination state, phase table, trunk drift,
sync state, candidate summary.

1. Coordination state: dispatch ref tip, live worktree path (via `git worktree
   list --porcelain`), trunk base.
2. **Trunk drift:** recompute fork-point `git merge-base dispatch/<N> trunk`,
   then `git merge-base --is-ancestor <fork_point> <live_trunk_tip>`.
   If no → `trunk: moved (N commits ahead of fork-point)`. If yes →
   `trunk: stable`. No stored state — recomputed each invocation.
3. Phase table: same as `plan-next`'s rollup (shared renderer).
4. Sync state: check whether `review/<slice>` ref exists → `prepared` or
   `not yet run`. Count `phase/<slice>-NN` refs.
5. Candidate summary: when `candidates.toml` has ≥1 row → `N candidate(s)
   (M admitted)`. Full detail is `dispatch candidate status`.
6. `next`: same as `plan-next`'s output.

### Output (human)

```
dispatch: refs/heads/dispatch/085  (abc123de)
coord:    /tmp/sl-085-coord (live)
trunk:    stable

phases:
  PHASE-01  completed   Templates & scaffold-output guard
  PHASE-02  completed   Detection-gap closure
  PHASE-03  blocked     Agent guidance
  PHASE-04  pending     Memory recording

sync:     not yet run
candidates: 0
next:     PHASE-04
```

When the coordination worktree has been removed (post-conclude):

```
dispatch: refs/heads/dispatch/085  (abc123de)
coord:    (removed)
trunk:    stable

phases:
  PHASE-01  completed   Templates & scaffold-output guard
  PHASE-02  completed   Detection-gap closure
  PHASE-03  completed   Agent guidance
  PHASE-04  completed   Memory recording

sync:     prepared — review/085, 4 phase cuts
candidates: 1 (1 admitted)
next:     (all phases completed — run 'dispatch sync --integrate' after audit)
```

### Code impact

- New `DispatchCommand::Status` variant in `main.rs`.
- New `dispatch::run_status(path, slice, json)` entry point (~80 lines):
  1. Resolve root.
  2. Coordination state: resolve dispatch ref tip, find live worktree via
     `git worktree list --porcelain`, compute trunk base.
  3. Trunk drift: resolve live trunk tip, run `git merge-base --is-ancestor`.
  4. Phase table via shared renderer.
  5. Sync state: check `review/<N>` + `phase/<N>-*` refs.
  6. Candidate summary via `read_candidates`.
  7. `next` from the same logic as `plan-next`.
  8. Render (human or JSON).
- Impure shell (git reads) but read-only — callable from anywhere.
- Reuses: phase table renderer (shared with `plan-next`), `read_candidates`
  (existing), `resolve_commit` (existing in `dispatch.rs`).

### Verification

- **VT-1:** `status --slice 85` on a freshly-set-up slice prints coordination
  state with live worktree, phase table, `sync: not yet run`, exits 0.
- **VT-2:** `status --slice 85` post-sync prints `sync: prepared — review/085,
  N phase cuts`.
- **VT-3:** `status --slice 85` with a moved trunk prints `trunk: moved (N commits
  ahead since setup)`.
- **VT-4:** `status --slice 85` with all phases completed prints
  `next: (all phases completed — run 'dispatch sync --prepare-review')`.
- **VT-5:** `status --slice 85` with coordination worktree removed prints
  `coord: (removed)` and still resolves dispatch ref tip from the branch.
- **VT-6:** `status --slice 85 --json` emits valid JSON with all sections.

## 5. Skill shrinkage

### dispatch/SKILL.md → ~40 lines

The shrunk skill retains:
- When to use (dispatch a whole slice unattended)
- The outer loop: `dispatch setup` → `dispatch plan-next` → spawn → funnel →
  repeat
- The funnel cadence (import → verify → branch-point → commit → record) —
  kept because the order is load-bearing and the orchestrator runs the
  individual verbs in sequence
- Handover cadence
- Conclude (sync, remove coord worktree, audit)
- Red flags

What's **removed** (now CLI-owned):
- The "Set up once" section → `dispatch setup`
- How to determine the next phase → `dispatch plan-next`
- The coordination worktree internals (`worktree coordinate` flags, resume
  semantics, trunk base computation)
- Per-arm fork creation details → arm skills (already separate)
- The boundary-recording step for the claude arm → stays in the arm skill
  (it's arm-specific)
- The candidate lifecycle prose → `dispatch candidate` CLI
- Detailed "Quick Reference" table entries that just restate CLI invocations

### dispatch-agent/SKILL.md → ~15 lines

The shrunk arm skill retains only the spawn template:
- `Agent` tool invocation with `subagent_type: dispatch-worker` and
  `isolation: worktree`
- The `verify-worker` post-spawn check
- The boundary-recording step (arm-specific)
- A pointer back to the router for the funnel

### dispatch-subprocess/SKILL.md → ~15 lines

The shrunk arm skill retains only the spawn template:
- `worktree fork --worker` invocation
- `env -C` / bwrap spawn line with `DOCTRINE_WORKER=1`
- The env contract sourcing
- A pointer back to the router for the funnel

### What does NOT change

- `worktree/SKILL.md` — the fork lifecycle is self-contained and already
  CLI-oriented
- The CLI verbs themselves (`worktree import`, `worktree branch-point-check`,
  `dispatch sync`, `dispatch candidate`, etc.) — unchanged in behavior
- The funnel cadence semantics (import → verify → branch-point → commit →
  record)
- The arm skill spawn mechanisms

## 6. Code impact summary

| File | Change | Lines |
|---|---|---|
| `src/main.rs` | New `DispatchCommand` variants: `Setup`, `PlanNext`, `Status`; new match arms | ~30 |
| `src/dispatch.rs` | New entry points: `run_setup`, `run_plan_next`, `run_status`; shared `render_phase_table` helper; new `CoordOutcome` type | ~190 |
| `src/worktree.rs` | Extract `fn coordinate()` pure-core from `run_coordinate`; `CoordOutcome` returned | ~15 refactor |
| `plugins/doctrine/skills/dispatch/SKILL.md` | Shrink from 321 → ~40 lines | ~280 removed |
| `plugins/doctrine/skills/dispatch-agent/SKILL.md` | Shrink from 162 → ~25 lines | ~137 removed |
| `plugins/doctrine/skills/dispatch-subprocess/SKILL.md` | Shrink from 119 → ~20 lines | ~99 removed |

Net: ~235 lines added to Rust, ~515 lines removed from skills.

## 7. Design decisions log

| ID | Decision |
|---|---|
| D1 | No `dispatch import` convenience verb. `worktree import` is already a one-liner; an alias saves zero tokens and adds a CLI surface. |
| D2 | `dispatch setup` gates on plan.toml existence + non-empty phase list before creating the coordination worktree. |
| D3 | Env contract format: `KEY=value` lines on stdout, matching `worktree fork`'s convention. Human status on stderr. |
| D4 | `plan-next` skips blocked phases in `next` output. Blocked = not actionable for dispatch. All-remaining-blocked prints an explicit `(none)` message. |
| D5 | File-disjointness is out of scope for v1. Adding a `files` field to plan.toml is an authored schema change. The orchestrator still runs `/phase-plan` for file sets. |
| D6 | `status` includes trunk drift detection: `git merge-base --is-ancestor <trunk_base> <live_trunk>` catches divergence early. |
| D7 | `status` shows a candidate summary (count + admitted count) but defers full detail to `dispatch candidate status` — no duplication. |
| D8 | Phase table rendering is a shared helper (`render_phase_table`) used by both `plan-next` and `status`. |
| D9 | `worktree::run_coordinate` is refactored to extract a pure-core `coordinate()` returning `CoordOutcome`, so `run_setup` can call it without duplicating I/O. |
| D10 | All three new verbs are read-only where possible; `setup` is the only mutating verb (creates/resumes the coordination worktree). |

## 8. Verification alignment

### New tests

- **`dispatch_setup_gates_on_plan`** — setup on a slice with no plan.toml exits non-zero
- **`dispatch_setup_creates_coordination`** — setup creates the worktree, emits valid env contract
- **`dispatch_setup_refuses_live_worktree`** — refuses when coordination-live
- **`dispatch_setup_resumes_branch`** — reattaches to an existing branch with no live worktree
- **`dispatch_plan_next_orders_phases`** — correct statuses, correct `next` (skips completed + blocked)
- **`dispatch_plan_next_all_blocked`** — `next` is empty with explicit message
- **`dispatch_plan_next_json`** — valid JSON output
- **`dispatch_status_full_rollup`** — coordination, phases, trunk drift, sync, candidates, next
- **`dispatch_status_trunk_drift`** — moved trunk reported correctly
- **`dispatch_status_post_sync`** — sync state shows `prepared`
- **`dispatch_status_post_conclude`** — coord removed, all phases completed, guidance text

### Existing suites (behaviour-preservation gate)

- `worktree coordinate` behavior unchanged (refactored, not re-implemented)
- `dispatch sync`, `dispatch candidate`, `dispatch record-boundary` unchanged
- All existing `worktree` and `dispatch` integration tests remain green
- `just gate` clean (clippy zero, fmt, tests pass)

## 9. Open questions

None remaining. All design decisions resolved.
