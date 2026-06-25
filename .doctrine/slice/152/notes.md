# Notes SL-152: Claude-arm WorktreeCreate worker creation

Durable per-slice scratchpad (tracked in git) — a **context-bootstrap runsheet**
for phase planners & implementors. `design.md` (§5 contract, §7 decisions, §10
review) is the authoritative record; this points at it and frames the work, it
does not duplicate it. Read `design.md` for the *why*; read this for *where to
start, what bites, and which finding lands on which phase*.

---

## 0. Start here (orientation)

**What this slice does.** Make doctrine the creator of the claude-arm dispatch
worker's git worktree, via a `WorktreeCreate` hook → a new `doctrine worktree
create-fork` verb. Kills H1 (no native creation left to fall back to ⇒ no
wrong-base spawn) and collapses both `/dispatch` arms onto one add+provision+mark
core. Full framing: design §1, slice-152.md.

**Reading order for a fresh agent:**
1. this §0–§3 (state + per-phase runsheet + cross-cutting gotchas);
2. `design.md` §5 (contract), §7 (D1–D11), §10 (probes + inquisition + /plan review);
3. `plan.toml` for the phase you're taking (EX/VT are authoritative);
4. the phase's runtime sheet `state/slice/152/phases/phase-NN.md` (`/phase-plan`
   expands it just before execute);
5. `/retrieve-memory` scoped to the files in §4 before editing.

**The spine in one paragraph.** Orchestrator `cd`s into the arming dir
`<coord>/.doctrine/state/dispatch/spawn/` (holding a `base` file) → Agent spawn
fires `WorktreeCreate` → the hook runs `create-fork`, which reads payload
`{cwd,name}`, resolves the coord root from `cwd`, and — because cwd IS the arming
dir — forks at `base` on `dispatch/<name>` at `<coord>/.worktrees/<name>`,
provisioning + marking inside the fork. A spawn from anywhere else passes through
(plain worktree, same provisioning, no marker). Discrimination is **positional**
(cwd-as-channel), never a payload class tag.

---

## 1. State (2026-06-25)

- Slice status: **`ready`** (design locked → plan authored → ready). Design schema
  settled, **no open forks**. All 3 pre-plan checks discharged (§5).
- **PHASE-01 `in_progress`** — pure `classify_create` + `sanitise_name` in a new
  `src/worktree/create.rs`. Sheet filled (`phases/phase-01.md`); next is `/execute`
  (TDD T1–T6).
- Commits (edge): `9685a695` probes → `7b76de34` inquisition → `700d1dd6` positional
  arming → `d830e3f1` memory → `f3fa6187` pre-plan-discharge+flip → `9f119375` plan →
  `74411a43` /plan review (D11 + plan tighten). Runtime phase sheets are gitignored
  (not committed).

---

## 2. Per-phase runsheet (what each phase builds, what bites it)

Phases are bottom-up: pure core → shell → orchestrator → install → skill →
plugin. Each row lists the touch-points and the **/plan-review findings (§3) that
land on that phase** — read those before coding the phase.

- **PHASE-01 — pure `classify_create` + `sanitise_name`** (new `src/worktree/create.rs`).
  Mirror `classify_stamp` SHAPE (subagent.rs:84): flat resolved facts → verdict +
  named tokens. `Fork{base,name} | Passthrough{name}`; tokens `missing-cwd`/
  `bad-name`/`missing-base`/`bad-base`, `NameRefusal` `empty`/`whitespace`/`slash`/
  `dotdot`/`ref-invalid`. Sanitiser **rejects, never rewrites** (round-trip safety).
  Carries a module `#![expect(unused, …)]` lid (PHASE-02 reconciles). **Bites:** G7
  (both name forms). Pure — no I/O. See `phase-01.md` for D-P1/2/3.

- **PHASE-02 — `worktree create-fork` shell + CLI wiring** (the heart;
  `create.rs` + `mod.rs` `WorktreeCommand::CreateFork` + `guard.rs`). Gather→classify→act
  over PHASE-01. **Bites the most findings:** G1 (stdout = path ONLY; suppress
  run_fork's `CARGO_TARGET_DIR=` env contract — split core from CLI emission, D11),
  G2 (root = `git -C payload.cwd --show-toplevel`, NOT `primary_worktree(cwd)` —
  create-fork fires in the PARENT, not the fork), G3 (benign passthrough must
  compensate on failure — reuse `remove_worktree_dir`), G6 (reconcile the stale
  "create-fork DROPPED" comments), G8 (Orchestrator guard-class is safe from the
  markerless coord tree). Reconcile the `#![expect(unused)]` lids on fork.rs/
  provision.rs as functions go live.

- **PHASE-03 — `dispatch arm-spawn`** (`dispatch.rs` `DispatchCommand::ArmSpawn`).
  Writes `<coord>/.doctrine/state/dispatch/spawn/base = <sha>\n`, prints the dir.
  base-B source = `run_setup` stdout `base=` (dispatch.rs:446) — already surfaced.
  Idempotent; arming dir is runtime-tier + D9-withheld (never provisioned).

- **PHASE-04 — install emission + stamp retirement** (`boot.rs` new `HookSpec` ctor
  event `WorktreeCreate`; retire stamp). **Bites:** G4 — the stamp is emitted at
  **TWO** sites (`skills.rs:1056-1077`, `install.rs:366-385`, gated `!global`+Claude),
  retire both; new HookSpec matcher is cosmetic (WorktreeCreate ignores matchers).
  Keep `verify-worker` + the baseRef belt. Headline H1 test lands here (CLI-level
  VT + you-run-it VA).

- **PHASE-05 — dispatch-agent SKILL contract (I3)** — edit **BOTH** copies
  (`.agents/skills/dispatch-agent/SKILL.md` AND `plugins/doctrine/skills/dispatch-agent/SKILL.md`;
  post-spawn block byte-identical, ~lines 56-63). Arm via `arm-spawn`+cd; derive
  `branch = dispatch/<basename(worktreePath)>`; bind `verify-worker` to the derived
  branch, NOT footer `worktreeBranch`. Prose; VA/VH.

- **PHASE-06 — (secondary, DROPPABLE) plugin-hook migration** — gated on probe P1
  (plugin parity, §6). Move the hook into `plugins/doctrine/hooks/hooks.json` and
  REMOVE the settings block in the same step (mutual exclusion). Drop if it
  threatens the primary (RSK-2).

---

## 3. /plan critical review (2026-06-25) — the load-bearing findings

Four-agent grounding of the plan against `src/` and `docs/claude/`. Design-affecting
ones reconciled into design.md (D11, §5.2, §10); plan ones folded into plan.toml.
Mapped to phases in §2. Verbatim:

- **G1 (→ D11).** `run_fork` emits `CARGO_TARGET_DIR=` on **stdout** (fork.rs:209-211);
  the WorktreeCreate protocol wants the path ALONE. create-fork splits the
  add+provision+mark core from the CLI env-contract emission (or subprocess-and-discard).
  Behaviour-preserving — the claude arm never consumed the contract (`run_stamp_subagent`
  emits none), so the worker keeps inheriting the orchestrator's `CARGO_TARGET_DIR`.
  Per-worktree target isolation on the claude arm stays a non-goal.
- **G2.** create-fork root = `git -C payload.cwd --show-toplevel` (coord tree, parent
  context) — deliberately NOT `primary_worktree(cwd)` (the stamp's inside-fork
  resolution). Mirror the gather→classify→act SHAPE, not the resolution.
- **G3.** The benign passthrough has no built-in rollback (run_fork does); add
  compensation (`remove_worktree_dir`) before the fail-closed exit or it leaks a tree.
- **G4.** The SubagentStart stamp is install-emitted at TWO sites (skills.rs:1056-1077,
  install.rs:366-385), gated `!global`+Claude — D2 retires both. WorktreeCreate hook =
  a NEW `HookSpec` ctor (event a free `&str`; matcher cosmetic for WorktreeCreate).
- **G6 — stale comments** to fix when create-fork revives: `subagent.rs:137-139`
  ("create-fork is DROPPED"), `fork.rs:51-52` (cleanup "shared by … PHASE-10's
  create-fork"). The drop rationale (thin payload) is obsoleted by positional arming.
- **G7 — `name` forms.** Sanitiser accepts BOTH `agent-<hex>` (P3) and moby
  `word-word-hex` (hooks.md:2419). Payload `agent_type` may appear per docs'
  common-fields rule, but P3 saw thin (WorktreeCreate fires in the *parent*, before
  the child runs) — design is agent_type-agnostic anyway. Non-issue.
- **G8 — guard.** Orchestrator-classing create-fork is safe: `worker_guard` keys off
  PROCESS cwd via `root::find(None)`; the hook fires in the markerless coord tree ⇒
  non-worker ⇒ allowed. A spawn from inside a marked fork is refused fail-closed
  (acceptable — dispatch-workers carry no Agent tool, can't nest isolation spawns).
- **Footer `worktreePath`** is empirically confirmed (P2) though undocumented; docs'
  `worktreePath` is the unrelated HTTP-hook output field — do not conflate.

---

## 4. Cross-cutting gotchas (every implementer hits these)

- **Lint (the gate).** Repo clippy DENIES `print_stdout`, `format_push_string`,
  `expect_used`/`unwrap_used` (non-test too), `let_underscore_must_use`,
  `unused`/`dead_code` — `mem.pattern.lint.clippy-denies`,
  `mem.pattern.lint.string-build-no-push-format`. Pure compose fns RETURN a `String`
  (build via `Vec<String>`+`concat`, not `push_str(&format!)`); the impure shell does
  the single `writeln!(io::stdout(), …)?`. The gate is **plain `cargo clippy`
  (bins/lib only)** — NOT `--all-targets` — so **test code may `unwrap`/`assert`
  freely**. `just check` = fast inner loop (root pkg); `just gate` before every commit.
- **`#![expect(unused)]` lids.** New extracted/forward-declared items have no consumer
  until a later phase ⇒ `dead_code` would fire. Carry a module lid; the lid must stay
  *fulfilled* (something genuinely unused) or clippy flips to unfulfilled-expectation.
  Reconcile (remove/narrow) when the consumer lands (PHASE-02 EX-7).
- **Shared CARGO_TARGET_DIR false-RED** (`mem.pattern.testing.shared-cargo-target-false-red`):
  in a coord/worktree, main's compiled test binaries shadow the fork's → a false RED.
  `touch` test files, run suites individually (never bare `cargo test --workspace`),
  `env -u DOCTRINE_WORKER`. (PHASE-01 runs in the main tree — less exposed; matters
  once verifying in a fork.)
- **Stale jail binary.** Shared `CARGO_TARGET_DIR` ⇒ silently stale `~/.cargo/bin/doctrine`.
  Use `./target/debug/doctrine` (`cargo build` first); `just rebuild-stale` if suspected.
- **`index.lock`.** Transient stale lock seen this session — check `ps` for a live git
  proc before removing any `.git/index.lock`.

---

## 5. Pre-plan checks — ALL DISCHARGED (2026-06-25; design confirmed, not changed)

1. **F3 (the spike) ✓ e2e green.** `worktree fork --worker` is CLI-wired and live:
   `mod.rs:288` `WorktreeCommand::Fork → run_fork`, guarded Orchestrator (guard.rs:225).
   Provision source is the COORD TREE — `run_fork` passes `run_provision(Some(repo),
   dir)`; `run_provision` enumerates from `source=root::find(path)` and
   `verify_sibling_worktree` BAILS if `source==fork` ⇒ ISS-011 Defect C structurally
   impossible. Proof: a gitignored sentinel ABSENT from commit B was provisioned into
   the fork ⇒ bytes came from the coord working tree. Marker landed; orchestrator fork
   refused under worker-mode. D1 thesis holds.
2. **arm-spawn base-B source ✓.** `run_setup` (dispatch.rs:446) emits `base=<dispatch_tip>`
   on stdout — the SAME tip the subprocess arm feeds `fork --base`. Orchestrator captures
   it and writes the arming `base` file; per-phase tip tracking is existing funnel
   behaviour. Writing-into-base-file is orchestrator/skill (plan) work; no SOURCE change.
3. **`.worktrees/` gitignored ✓** — `git check-ignore .worktrees/<x>` resolves.

---

## 6. Empirical harness facts (durable — proven, don't re-probe)

Three probe efforts, consistent across 2.1.181 (jail) and 2.1.187 (native): **thin
payload + hook-replaces-creation + matcher-doesn't-scope.**

1. **`probe.md`** (native 2.1.187) — payload thin (`{session_id, transcript_path,
   cwd, hook_event_name, name}`; no `agent_type`/base/path); hook replaces native
   creation; matcher does NOT scope by agent_type. Docs' rich payload is ahead of build.
2. **P3** (2.1.181) — payload `cwd` follows the orchestrator's Bash cwd; `cd` shifts it
   and the harness persists Bash cwd across tool calls ⇒ cwd is a per-spawn
   orchestrator-controlled channel. Each coord tree is its own git worktree, so
   `git -C <cwd> --show-toplevel` resolves the coord root from a subdir.
3. **P2** — the Agent return footer carries `worktreePath` through hook-creation;
   `worktreeBranch` came back `undefined` for a detached tree ⇒ `worktreePath` is the
   normative datum.

Recorded as memory **`mem.fact.dispatch.worktreecreate-cwd-channel`** (high trust),
linked to **`mem.pattern.dispatch.worktreecreate-replace-base-control`** and SL-152.
Probe artifacts cleaned up.

---

## 7. Key design decisions (full rationale: design.md §7)

- **Positional arming (D3/D4 — I1 resolution).** Discrimination = payload `cwd` IS the
  arming dir, NOT a file existing. `cd` in to arm, `cd` out to disarm (self-clearing;
  no load-bearing `disarm`). Arming dir carries ONLY a `base` file. Residual = a benign
  spawn issued *while* cwd is the arming dir (the mechanical floor); `verify-worker`
  backstops.
- **One byte-identical core (D1) + the env-contract split (D11).** `create-fork` is a new
  caller of the unchanged add+provision+mark core; the env-contract stdout emission is
  arm-specific and create-fork suppresses it (G1).
- **Root forcing (I5).** `create-fork` ALWAYS resolves `root = git -C <payload.cwd>
  --show-toplevel` and passes it explicitly into the core (NOT process cwd; NOT
  `primary_worktree` — G2).
- **Footer-read location (D8 primary).** Orchestrator reads `worktreePath`; derives
  `name=basename`, `branch=dispatch/<name>` (I3). Not `worktreeBranch`.
- **Benign passthrough provisions via the same copier (D9/I2)** + compensates on
  failure (G3). `.worktreeinclude` is non-empty here.
- **`name` sanitiser (I4)** — fail-closed reject outside the ref+path-safe envelope.
- **Retire SubagentStart stamp on the claude arm (D2)** at both install sites (G4);
  backstop stays `verify-worker`.

Inquisition (codex/GPT-5.5, design §10): I1→positional arming, I2→benign provisioning
parity, I3/I4/I5→worktreePath-normative / sanitiser / locked root-forcing. All
dispositioned; both factual premises verified in-repo.

---

## 8. Code seams (where to cut)

- `src/worktree/create.rs` — **NEW** (PHASE-01 pure; PHASE-02 shell `run_create_fork`).
- `src/worktree/fork.rs:133` `run_fork` (shared core; emits env contract :209-211 — G1;
  `fork.rs:1` + `:51-52` lids/stale-comment — G6; `remove_worktree_dir:63` reuse — G3).
- `src/worktree/subagent.rs:162` `run_stamp_subagent` (mirror the SHAPE; `classify_stamp:84`;
  payload struct `:107-113`; `:137-139`/`:157` stale "DROPPED" comments — G6;
  `verify-worker:343` `run_verify_worker(base,dir,branch:Option)` — branch is an explicit
  arg, not derived).
- `src/worktree/marker.rs` — `write_marker` (`.doctrine/state/dispatch/worker`, presence-only);
  `resolve_mode:161` (worker-mode = `is_linked && marker_present || env` — G8).
- `src/worktree/mod.rs:74/117` `WorktreeCommand` (+`CreateFork`); `src/dispatch.rs:34`
  `DispatchCommand` (+`ArmSpawn`); both flat-match dispatch.
- `src/boot.rs` `HookSpec:927-934` (event = free `&str`), `install_claude_hook:1552`,
  `install_refresh:1230`/`install_baseref:1236`; stamp ctor `:971-978`. Emission sites:
  `src/skills.rs:1056-1077`, `src/install.rs:366-385` (G4).
- `guard.rs:225` `Fork→Orchestrator`; `worker_guard:328-364` (mode from process cwd — G8).
- SKILL (both copies): `.agents/skills/dispatch-agent/SKILL.md`,
  `plugins/doctrine/skills/dispatch-agent/SKILL.md` (post-spawn ~56-63 — I3).

Relevant memories to retrieve when cutting: `mem.pattern.lint.clippy-denies`,
`mem.pattern.lint.string-build-no-push-format`, `mem.pattern.worktree.primary-tree-resolver-…`
(the G2 contrast), `mem.pattern.testing.shared-cargo-target-false-red`,
`mem.fact.dispatch.worktreecreate-cwd-channel`.

---

## 9. Open / deferred

- **P1** — plugin `hooks/hooks.json` parity vs settings-block. Gates only PHASE-06.
  Expected yes; verify before relying.
- **worktreeBranch-when-named** — cheap confirming probe (does the footer populate
  `worktreeBranch` for a NAMED-branch hook fork?). Nice-to-have, not gating (D8).
- **WorktreeRemove / branch GC (F5/D10)** — retried workers leak `dispatch/<name>`
  branches; prune in a WorktreeRemove follow-up or `dispatch gc`. Follow-up slice
  (capture via `backlog new` if not already tracked).
