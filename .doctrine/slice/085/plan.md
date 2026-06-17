# SL-085 — Implementation plan rationale

## Sequencing

Four phases, strictly ordered (each builds on the prior):

```
PHASE-01 (refactor + setup) → PHASE-02 (plan-next) → PHASE-03 (status) → PHASE-04 (skills)
```

**PHASE-01** is the foundation. The `worktree::coordinate` refactor is a
mechanical extraction — no behavior changes, just factoring a pure-core function
from a CLI entry point. `dispatch setup` is the first verb and the entry point
for every dispatch session; building it first proves the extraction works.

**PHASE-02** introduces the shared `render_phase_table` helper that both
plan-next and status consume. It's a pure read-only verb touching only plan.toml
and the runtime state tree — no git, no refs. The shared renderer is designed
for reuse but not over-generalized: it takes the phase data the caller already
computed, so status can add its own sections around it without touching the
renderer internals.

**PHASE-03** is the richest verb. It reuses the phase table from PHASE-02 and
adds coordination state (impure: git ref resolution, worktree list parsing),
trunk drift detection (stateless, rev-list count since fork-point), sync state (ref existence),
and candidate summary (reads `candidates.toml`). All operations are read-only.
The risk is in the impure shell — git invocations that need careful error
handling for absent refs and detached worktrees.

**PHASE-04 (skill shrink + embed verification)** changes no Rust behavior, but must refresh and verify embedded skill content. The three skill markdown files are edited in `plugins/`, re-installed via `doctrine install`, and re-embedded via `touch src/skills.rs && cargo build`. An integration test (EX-6) verifies embedded text matches the shrunk plugin source; without it `just gate` cannot detect a stale binary.

## Boundaries

| Phase | Touches |
|---|---|
| PHASE-01 | `src/worktree.rs` (extract + define `CoordOutcome`), `src/dispatch.rs` (new `run_setup`), `src/main.rs` (clap) |
| PHASE-02 | `src/dispatch.rs` (new `run_plan_next`, `render_phase_table`), `src/main.rs` (clap) |
| PHASE-03 | `src/dispatch.rs` (new `run_status`), `src/main.rs` (clap) |
| PHASE-04 | `plugins/doctrine/skills/dispatch/SKILL.md`, `plugins/doctrine/skills/dispatch-agent/SKILL.md`, `plugins/doctrine/skills/dispatch-subprocess/SKILL.md` |

PHASE-02 and PHASE-03 are NOT file-disjoint (both touch `src/dispatch.rs` and
`src/main.rs`). They must run sequentially. PHASE-04 is file-disjoint from all
code phases.

## Design coverage

Every design decision (D1–D10) maps to a phase:

| Decision | Phase |
|---|---|
| D1: No `dispatch import` | PHASE-04 (purge references from skills) |
| D2: Plan gate on setup | PHASE-01 |
| D3: Env contract KEY=value | PHASE-01 |
| D4: Blocked phases excluded from `next` | PHASE-02 |
| D5: No file-disjointness v1 | (non-goal, no phase) |
| D6: Trunk drift in status | PHASE-03 |
| D7: Candidate summary (brief) | PHASE-03 |
| D8: Shared phase-table renderer | PHASE-02 |
| D9: Extract `coordinate()` pure-core | PHASE-01 |
| D10: Read-only posture | PHASE-02, PHASE-03 |

## Risk

- **PHASE-01 extraction risk:** `run_coordinate`'s CLI I/O (stdout/stderr) must
  stay unchanged. The extraction is mechanical — move the coordination logic
  into `coordinate()`, leave the I/O in `run_coordinate`. Existing integration
  tests are the safety net.
- **PHASE-03 git fragility:** `git worktree list --porcelain` parsing and `git
  rev-list --count` are new shell code. Edge cases (bare repos, detached HEAD,
  absent refs) need explicit handling per the absent-ref table in design §4.
  Read-only posture limits blast radius.
- **PHASE-04 stale-embed risk:** After editing skill files, `touch
  src/skills.rs && cargo build` re-embeds. An integration test (EX-6) verifies
  the embedded text matches the shrunk plugin source. Without it, `just gate`
  cannot detect a stale binary.
