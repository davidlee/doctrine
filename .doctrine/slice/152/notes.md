# Notes SL-152: Claude-arm WorktreeCreate worker creation

Durable per-slice scratchpad — tracked in git. Index + decisions + forward
threads. Authoritative design record is `design.md` (esp. §5, §7, §10); this
points at it, it does not duplicate it.

## State (2026-06-25)

Slice status: **`design`** → flipping to **`plan`** (all three pre-plan checks
discharged 2026-06-25, see below; F3 e2e confirmed the design, no change). Design
schema is **settled — no open design forks.** P1 (plugin parity) gates only the
secondary plugin step, carried into /plan.

Commits (edge): `9685a695` probes → `7b76de34` inquisition I2–I5 →
`700d1dd6` I1 positional arming → `d830e3f1` memory.

## Empirical harness facts (durable — proven, don't re-probe)

Three probe efforts, all consistent; **thin payload + hook-replaces-creation +
matcher-doesn't-scope** are settled across 2.1.181 (jail) and 2.1.187 (native).

1. **`probe.md`** (earlier, native 2.1.187) — payload shape genuine-thin
   (`{session_id, transcript_path, cwd, hook_event_name, name}`; no `agent_type`/
   base/path), hook replaces native creation, doctrine controls base out-of-band,
   matcher does NOT scope by agent_type. Docs' rich payload is ahead of the build.
   (Debris left in `/tmp/probe` per that note — native box, not this jail.)
2. **P3** (this session, 2.1.181) — payload `cwd` **follows the orchestrator's
   Bash cwd**; `cd` shifts it and the harness persists Bash cwd across tool calls.
   `cd .dispatch/SL-123` → next spawn's payload `cwd=…/.dispatch/SL-123`. ⇒ cwd is
   a per-spawn orchestrator-controlled channel. Each coord tree is its own git
   worktree, so `git -C <cwd> --show-toplevel` resolves the coord root from a
   subdir.
3. **P2** (this session) — Agent return footer carries **`worktreePath`** through
   hook-creation. `worktreeBranch` came back `undefined` for a **detached** tree
   (scratch hook used `add … HEAD`). ⇒ `worktreePath` is the normative datum.

Recorded as reusable memory **`mem.fact.dispatch.worktreecreate-cwd-channel`**
(high trust), linked to the spine **`mem.pattern.dispatch.worktreecreate-replace-
base-control`** and SL-152. Probe artifacts for P3/P2 were cleaned up (scratch
`WorktreeCreate` hook removed from `settings.local.json`, `/tmp/wtc-probe.log` +
scratch `.worktrees/*` trees removed).

## Key design decisions (see design.md §7 for full rationale)

- **Positional arming (D3/D4, the I1 resolution).** Discrimination = payload
  `cwd` **IS** the arming dir `<coord>/.doctrine/state/dispatch/spawn/`, NOT a
  file existing. Orchestrator `cd`s in to arm, `cd`s out to disarm (self-clearing;
  no load-bearing `disarm` verb). Arming dir carries ONLY a `base` file (`<sha>`;
  nothing else encoded into the path). Kills the false-positive window, dissolves
  the old F4 persistent-marker edge. Residual = a benign `isolation:worktree`
  spawn issued *while* cwd is the arming dir = the mechanical floor (payload has
  no class tag); `verify-worker` backstops.
- **One byte-identical core (D1).** `worktree create-fork` (hook side) is a new
  caller of the **unchanged** `fork --worker`, exactly as the subprocess arm
  calls it. `create-fork` ALWAYS resolves `root = git -C <payload.cwd>
  --show-toplevel` and passes it explicitly into `run_fork`/`run_provision`
  (I5 — never relies on process cwd; P3 proves *payload* cwd, not *process* cwd).
- **Footer-read location (D8 primary).** Orchestrator reads `worktreePath` from
  the footer; derives `name = basename`, `branch = dispatch/<name>` (I3). Does
  NOT depend on `worktreeBranch`.
- **Benign pass-through provisions via the same copier (D9/I2).** `.worktreeinclude`
  is non-empty here (`.doctrine/doctrine.just`, `web/map/dist/**`); hooks bypass
  native `.worktreeinclude`, so the benign path must replicate it or it regresses
  every benign isolated subagent (repo-global, fail-closed). OQ-2 closed: replicate.
- **`name` sanitiser (I4).** `classify_create` fail-closed rejects empty/whitespace/
  `/`/`..`/ref-invalid/colliding `name`; canonical slug only.
- **Retire SubagentStart stamp on the claude arm (D2).** `fork --worker` marks
  atomically inside `create-fork`; stamp would hit `already-marked` every
  dispatch. Stamp fires AFTER WorktreeCreate so can't feed base selection anyway.
  Backstop stays `verify-worker`. (The stamp hook is STILL wired in
  `settings.local.json` `SubagentStart matcher:dispatch-worker` — retirement is
  implementation work, not done.)

## Inquisition (codex/GPT-5.5, design.md §10) — all 5 dispositioned

I1 (blocker) → positional arming. I2 (blocker) → benign provisioning parity.
I3/I4/I5 (majors) → worktreePath normative / name sanitiser / locked root-forcing.
Both factual premises verified in-repo. Reviewer dismissed: `--show-toplevel`
addressing (sound); ADR-006 sole-writer (holds iff I1 window closed — it is).

## Pre-plan checks (ALL DISCHARGED 2026-06-25 — design confirmed, not changed)

1. **F3 — the spike. ✓ DISCHARGED (e2e green).** `worktree fork --worker` is
   CLI-wired and live: `mod.rs:288` `WorktreeCommand::Fork → run_fork`, guarded
   Orchestrator (`guard.rs:225`). The module `expect(unused)` covers sibling
   extracted helpers, NOT `run_fork`. Provision source is the COORD TREE — `run_fork`
   passes `run_provision(Some(repo), dir)` with `repo = root::find(--path|cwd)`;
   `run_provision` enumerates candidates from `source=root::find(path)`, and
   `verify_sibling_worktree` BAILS if `source==fork`. ⇒ the ISS-011 Defect C trap
   is structurally impossible. **E2E proof:** a gitignored sentinel
   (`web/map/dist/F3-SENTINEL.txt`, ABSENT from commit B) was provisioned into the
   fork → its bytes could only originate in the coord working tree. Worker marker
   landed at `.doctrine/state/dispatch/worker`; orchestrator `fork` refused under
   worker-mode (rollback clean). D1 "byte-identical core" thesis holds.
2. **arm-spawn base-B source. ✓ DISCHARGED.** `run_setup` (`dispatch.rs:446`) emits
   `base={outcome.dispatch_tip}` on stdout — the SAME coordination-tip B the
   subprocess arm already consumes for `fork --base`. The orchestrator captures that
   value (it already reads setup's stdout contract) and writes it into the arming
   `base` file; per-phase tip tracking is existing funnel behaviour. No new
   persistence on the SOURCE side — writing-into-base-file is orchestrator/skill
   (plan) work.
3. **`.worktrees/` gitignored. ✓ CONFIRMED** — `git check-ignore .worktrees/<x>`
   resolves (side-effect of the F3 e2e).

## /plan critical review (2026-06-25) — code+docs grounding (durable)

Four-agent grounding of the plan against `src/` and `docs/claude/`. Design-affecting
findings reconciled into design.md (D11, §5.2, §10); plan findings folded into
plan.toml. The load-bearing ones to remember:

- **G1 (→ D11).** `run_fork` emits its env contract (`CARGO_TARGET_DIR=`) on
  **stdout** (`fork.rs:209-211`); the WorktreeCreate protocol wants the path ALONE.
  create-fork must split the add+provision+mark core from the CLI env-contract
  emission (or subprocess-and-discard). Behaviour-preserving: the claude arm never
  consumed the env contract (`run_stamp_subagent` emits none), so the claude worker
  keeps inheriting the orchestrator `CARGO_TARGET_DIR`. Per-worktree target
  isolation on the claude arm stays a non-goal.
- **G2.** create-fork root resolution = `git -C payload.cwd --show-toplevel`
  (coord tree, parent context) — deliberately NOT `primary_worktree(cwd)`, which is
  the stamp's inside-fork resolution. Mirror run_stamp_subagent's gather→classify→act
  SHAPE, not its root resolution.
- **G3.** The benign pass-through (`git worktree add … HEAD` + provision) has no
  built-in rollback (run_fork does); add compensation (`remove_worktree_dir`) before
  the fail-closed exit or it leaks a half-created tree.
- **G4.** The SubagentStart stamp is install-emitted at TWO sites
  (`skills.rs:1056-1077`, `install.rs:366-385`), gated `!global`+Claude — D2 retires
  both. The WorktreeCreate hook is a NEW `HookSpec` ctor (event = free `&str`;
  matcher cosmetic/ignored for WorktreeCreate).
- **Stale comments** to fix when create-fork revives: `subagent.rs:137-139`
  ("DROPPED"), `fork.rs:51-52` (cleanup-sharing). Drop rationale obsoleted by
  positional arming.
- **`name` forms.** Sanitiser accepts BOTH `agent-<hex>` (P3) and moby
  `word-word-hex` (hooks.md:2419). Payload `agent_type` may appear per docs but P3
  saw thin (WorktreeCreate fires in parent) — design is agent_type-agnostic anyway.
- **Guard (G8).** Orchestrator-classing create-fork is safe: worker_guard keys off
  PROCESS cwd via `root::find(None)`; the hook fires in the markerless coord tree
  ⇒ non-worker ⇒ allowed. A spawn from inside a marked fork is refused fail-closed
  (acceptable — dispatch-workers carry no Agent tool, can't nest isolation spawns).
- **dispatch-agent SKILL** exists in TWO copies (`.agents/skills/…` and
  `plugins/doctrine/skills/…`, post-spawn block byte-identical) — PHASE-05 edits both.

## Code seams (for implementation/planning)

- `src/worktree/fork.rs:133` `run_fork` (the shared core; `fork.rs:1` unused-expect).
- `src/worktree/subagent.rs:162` `run_stamp_subagent` (the to-be-retired stamp;
  `classify_stamp:84`, `verify-worker:343`) — mirror its gather→classify→act shape
  for `create-fork`/`classify_create`.
- `src/boot.rs:1552/938/1197` hook emission; `src/dispatch.rs:407` setup.
- `.agents/skills/dispatch-agent/SKILL.md` — post-spawn contract MUST change:
  today binds `verify-worker`/funnel to footer `worktreeBranch` (lines ~57–62);
  I3 changes it to derive branch from `worktreePath`. Skill edit is plan/execute work.
- `src/worktree/marker.rs` `write_marker` — the worker-provenance stamp (DIFFERENT
  from the arming signal; stays, runs inside the fork).

## Open / deferred

- **P1** — plugin `hooks/hooks.json` parity vs settings-block. Gates only the
  secondary (plugin) emission (D7). Expected yes; verify before relying.
- **worktreeBranch-when-named** — cheap confirming probe (does the footer populate
  `worktreeBranch` for a NAMED-branch hook fork?). Now nice-to-have, not gating
  (D8 derives branch from path).
- **WorktreeRemove / branch GC (F5/D10)** — retried workers leak `dispatch/<name>`
  branches; prune in a WorktreeRemove follow-up or `dispatch gc`. Follow-up slice.
