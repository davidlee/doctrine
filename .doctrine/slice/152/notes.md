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

- Slice status: **`started`**. Design locked → plan authored. All 3 pre-plan checks
  discharged (§5). **PHASE-01..04 `completed`**. **PHASE-05 implemented + VA-1 PASS;
  VH-1 (you-run-it live dry-run) OUTSTANDING** — phase held `in_progress` pending
  the User's defer-vs-run call. PHASE-06 secondary/droppable.
- **PHASE-05 `completed`** (VH-1 DEFERRED to next real dispatch, User call
  2026-06-25) — both dispatch-agent SKILL copies rewritten to the
  WorktreeCreate-hook contract: arm via `dispatch arm-spawn --base B` + cd into the
  spawn dir / cd back (positional disarm); post-spawn derive `name=basename(
  worktreePath)`, `branch=dispatch/<name>`, bind `verify-worker` to the DERIVED
  branch (NOT footer `worktreeBranch`, undefined live per PHASE-04 VA-1). Scope was
  larger than the post-spawn block — the whole Pre-spawn section encoded the retired
  placement-implicit base, rewritten too. Body 82→100 lines ⇒ shrinkage-gate budget
  bumped 82→100 (e2e_skills_dispatch_shrinkage.rs). VA-1 PASS (codex GPT-5.5: EX-1/
  EX-2/VA-1, no contradictions). Installer verb is now `doctrine install -s <skill>
  -y` (SL-088 consolidation; the skill-refresh memory is stale on the verb name).
- **PHASE-06 plugin-form LIVE PASS** (2026-06-25, real committed plugin, not the
  P1 scratch-probe) — `plugins/doctrine/hooks/hooks.json` carries BOTH hooks, bare
  `doctrine` (PATH form), project scope; plugin `doctrine@doctrine-edge` enabled.
  Plugin firing PROVEN: `~/.claude.json` `pluginUsage.doctrine@doctrine-edge`
  `lastUsedNumStartups == numStartups == 495` (this session). On `clear`-restart:
  SessionStart `doctrine boot` wrote `boot.md`, BOOT-SENTINEL present ⇒ **boot parity
  green**. One benign `isolation:worktree` Agent spawn ⇒ WorktreeCreate `doctrine
  worktree create-fork` fired: footer `worktreePath:…/.worktrees/agent-a1806b…`, tree
  `(detached HEAD)` at edge tip, unmarked, `.doctrine/` fully provisioned ⇒
  **benign-spawn discrimination + sole-copier green** (correct per create-fork's
  positional contract — native Claude would land in `.claude/worktrees/` w/ a named
  branch). Probe worktree removed. **Plugin form ≡ retired settings blocks.**
  - **LIVE EX-4 BUG WITNESSED:** a mid-session `doctrine install` (run to refresh
    skills for a background pi dispatch) re-added the absolute-path settings-block
    hooks ON TOP of the plugin ⇒ momentary **double-wiring** of SessionStart+
    WorktreeCreate. Removed the settings `hooks` block by hand (kept MCP/permissions/
    baseRef). This is precisely the mutual-exclusion (EX-1/EX-3) that EX-4 must fix:
    `run_install` (skills.rs:1071-1079) unconditionally re-wires create-fork into
    settings even when the plugin already carries it.
  - **EX-4 DONE** (2026-06-26): `doctrine install` (Claude) STOPS wiring boot +
    create-fork into settings; instead PRINTS plugin-install instructions
    (`/plugin marketplace add <repo>` + `/plugin install doctrine@doctrine`), repo from
    new `[install] repo` doctrine.toml key (default `davidlee/doctrine`, leaf module
    `src/install_config.rs` — ADR-001: dtoml is leaf, can't import command-tier install).
    Non-Claude prints `npx skills add <repo> --agent universal -y`. `install_refresh`
    Claude arm sets `hook = None` (boot no longer settings-wired — also affects `doctrine
    boot install`, accepted); baseRef/mcp/skills/agent-def/boot-import UNCHANGED.
    `HookSpec::boot`/`create_fork` + `install_claude_hook` retained as fallback
    (`expect(dead_code)`, test-only callers). Pure `install::post_install_instructions`
    builder (DRY — skills.rs reuses it). **VT-2 GREEN** (e2e_claude_install: settings has
    NO SessionStart/WorktreeCreate/SubagentStart, instructions printed). Both marketplace
    manifests retained (`.claude-plugin/` = Claude, `plugins/` = codex/universal).
  - **Verification footgun (durable):** the shared `CARGO_TARGET_DIR` jail flip-flopped
    the install BINARY stale↔fresh while the background pi dispatch built concurrently
    → false-GREEN then false-RED on `e2e_claude_install`. Pure-fn unit tests stayed honest
    (compiled in-suite). Definitive verification = isolated `CARGO_TARGET_DIR=/tmp/...`.
    `just gate` flaked RED on this; isolated run + isolated `clippy --workspace` both clean.
  - REMAINING follow-up (separate slice, RSK-2): marketplace auto-register /
    `enabledPlugins` automation — NOT in SL-152.
- **PHASE-04 `completed`** — VT-1..4 automated + green; **VA-1 PASS live** (2.1.181,
  jail binary): real `Agent isolation:worktree` from the armed cwd → hook created
  `.worktrees/agent-<hex>` at base B, `dispatch/<name>` branch, worker-marked (F7);
  benign spawn from coord root → detached, unmarked passthrough. Footer
  `worktreeBranch: undefined` confirmed (P2/D8 → PHASE-05 derives branch from
  `worktreePath`). **R-P4-1 confirmed live**: a pre-existing SubagentStart stamp is
  NOT pruned by install (benign already-marked). Full VA evidence: `phase-04.md`.
- **PHASE-04 mechanics** — `HookSpec::create_fork` (event
  `WorktreeCreate`, cosmetic matcher `"*"`) emitted at BOTH install sites
  (`skills.rs` run_install + `install.rs`), REPLACING the retired SL-123
  `stamp_subagent` ctor/command/predicate (removed; the `worktree marker
  --stamp-subagent` VERB + `classify_stamp` retained — D-P4-2). SL-124 merge-core
  normalize tests re-pointed stamp→create_fork (F-P4-1). New e2e
  `tests/e2e_dispatch_h1_integration.rs` (VT-4: arm-spawn→create-fork lands at B
  under moving main + verify-worker passes, F7). `e2e_claude_install` updated to the
  WorktreeCreate contract (VT-1 negative golden, F-P4-3). baseRef belt + verify-worker
  verb untouched (separate code paths — A1/A2). Full detail: `phase-04.md`.
- **PHASE-01 `completed`** — pure `classify_create` + `sanitise_name` in
  `src/worktree/create.rs` (VT-1 matrix + VT-2 sanitiser table).
- **PHASE-02 `completed`** — `doctrine worktree create-fork` shipped (the heart). The
  `fork_core` split (byte-identical core, D11) + the impure shell (`run_create_fork`,
  `act_on_create`, gather) all in `create.rs` alongside the classifier (mirrors
  `subagent.rs`). CLI-wired `WorktreeCommand::CreateFork`, guard-classed
  `Orchestrator("create-fork")`. **5 e2e tests green** (`tests/e2e_worktree_create_fork.rs`,
  VT-1..8); fork/provision/stamp suites green UNCHANGED (VT-6); `just check` clean. Lids
  on fork.rs/provision.rs REMOVED (masked only dead imports) + dead imports pruned;
  create.rs lid removed + module doc corrected (no longer claims "no git/disk"). New
  imperative refusal token **`no-root`** (cwd resolves but outside any git worktree —
  F-P2-2). Full findings: `phase-02.md` §Findings.
- **PHASE-03 `completed`** — `doctrine dispatch arm-spawn --base <B> [--slice <N>]` shipped
  (`dispatch.rs::run_arm_spawn`, `DispatchCommand::ArmSpawn`, guard-classed
  `Orchestrator("dispatch-arm-spawn")`). Writes `<coord>/.doctrine/state/dispatch/spawn/base`
  = `"<sha>\n"` via `fsutil::write_atomic` (the `fs::write` clippy seam — runtime write),
  prints the canonical spawn dir on stdout. **3 e2e green** (`tests/e2e_dispatch_arm_spawn.rs`:
  exact-sha + idempotent overwrite + fail-closed bad-base) + the withheld unit case
  (`allowlist.rs::is_withheld_classifies_each_tier` now pins `spawn/base → Tier::State`).
  Decisions: D-P3-1 base validated to create-fork's 4..=64-hex envelope (fail-closed `bad-base`
  at arm time); D-P3-2 NO `disarm` verb (positional cd-back, design §5.4); D-P3-3 `ARMING_SUBPATH`
  re-exported from `create.rs` (one contract anchor, no re-spelling); D-P3-4 root via `root::find`
  (sibling dispatch idiom = create-fork's `--show-toplevel` in a coord tree). `--slice` is
  diagnostic-only (stderr), arming dir is per-coord-tree not per-slice. `just gate` clean.
- **For PHASE-03 — the file contract create-fork now reads is LOCKED:** arming dir =
  `<root>/.doctrine/state/dispatch/spawn` (const `ARMING_SUBPATH` in create.rs); base
  file = `<arming_dir>/base`, contents a plausible sha (create-fork TRIMS it, accepts
  4..=64 hex), so `arm-spawn` may write `"<sha>\n"`. create-fork makes the fork at
  `<root>/.worktrees/<name>` on branch `dispatch/<name>`.
- Commits (edge): … `74411a43` /plan review → `a52bc872`/`58ed6ca6` PHASE-01 →
  (this commit) **`feat(SL-152) PHASE-02 create-fork`**. Runtime phase sheets gitignored.

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

- **PHASE-03 — `dispatch arm-spawn`** (`dispatch.rs` `DispatchCommand::ArmSpawn`). **DONE.**
  Writes `<coord>/.doctrine/state/dispatch/spawn/base = <sha>\n` (atomic), prints the dir.
  base-B source = `run_setup` stdout `base=` (dispatch.rs:446). Idempotent; arming dir is
  runtime-tier + D9-withheld (`Tier::State`, never provisioned — pinned in the allowlist test).
  Shared the `ARMING_SUBPATH` const from `create.rs` (D-P3-3); fail-closed base validation
  (D-P3-1); no `disarm` (D-P3-2). arm-spawn does NOT create `.worktrees/` (create-fork does).

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
4. **P1** (2.1.181, jail, 2026-06-25) — a `WorktreeCreate` hook declared in a **plugin**
   `plugin.json` (command via `${CLAUDE_PLUGIN_ROOT}`) fires **identically** to the
   settings-block form. Procedure: scratch marketplace+plugin `p1-probe@p1-probe`
   (enabled, user scope) wrapping the SAME `doctrine worktree create-fork` binary the
   settings form calls; settings WorktreeCreate block REMOVED (mutual exclusion, D7);
   restart; one benign `isolation:worktree` spawn. All three PASS criteria met:
   `/tmp/p1-probe.log` logged `P1-PROBE fired` + thin payload (`hook_event_name:
   "WorktreeCreate"`, `cwd:/workspace/doctrine`, `name:agent-adfe…`) with
   `${CLAUDE_PLUGIN_ROOT}` expanded; Agent footer carried `worktreePath:
   …/.worktrees/agent-adfe250d14e1b246a`; the `.worktrees/<name>` tree existed.
   ⇒ **P1 PASS** — declaration site is hook-firing-irrelevant; PHASE-06 migration
   (settings→plugin) is parity-safe. Probe artifacts cleaned up.

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

- `src/worktree/create.rs` — pure classifier + the **shipped** shell: `run_create_fork`
  (stdin → gather → classify → act → print path), `act_on_create(root, action) ->
  PathBuf` (the act seam; both arms canonicalise), `resolve_root` (`git -C cwd
  --show-toplevel`), consts `ARMING_SUBPATH`/`WORKTREES_SUBDIR`. No lid (all live).
- `src/worktree/fork.rs` — `fork_core(repo,base,branch,dir,worker)` is the SILENT
  byte-identical core (no stdout/stderr); `run_fork` = `fork_core` + env-contract
  emission (D11). `remove_worktree_dir` reused by create-fork's passthrough compensation
  (G3). Lid + dead imports removed; stale comment fixed (G6).
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
  ~~Expected yes; verify before relying.~~ **RESOLVED PASS 2026-06-25** (§6.4) —
  PHASE-06 unblocked.
- **worktreeBranch-when-named** — cheap confirming probe (does the footer populate
  `worktreeBranch` for a NAMED-branch hook fork?). Nice-to-have, not gating (D8).
- **WorktreeRemove / branch GC (F5/D10)** — retried workers leak `dispatch/<name>`
  branches; prune in a WorktreeRemove follow-up or `dispatch gc`. Follow-up slice
  (capture via `backlog new` if not already tracked).

---

## 10. PHASE-06 audit (RV-158, 2026-06-26)

Reconciliation audit of the plugin-hook migration. Reviewed edge HEAD
(`c371b839` 6a + `341867bd` EX-4); authored arm, no candidate branch. Full suite
+ clippy green **in an isolated target** (shared jail target gave a false RED —
stale binary, `mem.pattern.testing.shared-cargo-target-false-red`).

EX-3/EX-4/VT-1/VT-2/VA-1/VA-2 all confirmed. Three findings, all terminal:
- **F-1** (minor, *tolerated*) — SL-152 declares no `[[selector]]`, so `slice
  conformance` + `review prime` have no path-set (both report unavailable).
  Compensated with manual full-diff; selector retrofit low-value on a closing
  slice.
- **F-2** (minor, *verified* → reconcile per-slice edit) — plan EX-4 prose cites
  `[claude] plugin-marketplace`; shipped + design D12 use `[install] repo`
  (default `davidlee/doctrine`, User-confirmed). EX-4 id immutable → append a
  correction note to `plan.md` in reconcile.
- **F-3** (nit, *aligned*) — ADR-001 `install_config="leaf"` registered in-commit;
  routine living-registry upkeep, no REV.

No blocker. Reconciliation brief: one per-slice edit (F-2), zero governance/REV.
Handing to `/reconcile`.
