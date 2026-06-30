# Design SL-182: Claude-arm subagent write-confinement hooks

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

Status: LOCKED (internal adversarial pass + `/inquisition` RV-200 both integrated; §10). Governed by ADR-008 (closes its claude-arm
confinement gap), ADR-006 (D2b raw-tree confinement; D-sole-writer). Originates
from RSK-014 (probe-h1, PROVEN). Path C deferred → IDE-024; selector-allowlist
mode → IDE-025.

## 1. Design Problem

Graduate the RSK-014 probe-h1 apparatus — proven write-containment for a claude
`isolation: worktree` subagent — from throwaway bash scripts into installed
doctrine machinery, so claude dispatch workers are confined **by construction** on
the Linux/bwrap arm. Close the ADR-006 D2b / ADR-012 OQ-D impersonation gap on the
claude arm with a hard wall, not the cooperative marker (RSK-014).

## 2. Current State

- **Claude arm has no mount-ns confinement.** Native harness confines subagent
  `Edit`/`Write` to the shared repo checkout only; `Bash` is wholly unconfined
  (necessity-controlled, RSK-014 Exp 3); the entire non-repo filesystem (`/tmp`,
  `$HOME`, `/etc`, ssh keys) is writable. Worker-mode is a cooperative flag.
- **The pi arm IS confined** via `scripts/pi-spawn-confined.sh` (nested bwrap,
  `--ro-bind / /` + rw worktree). Linux-only. This is the proven flag set.
- **Proven probe** (claude-code 2.1.181, NixOS, bwrap 0.11.2): a
  `PreToolUse(Bash)` hook rewrites the command via `updatedInput` into a nested
  bwrap jail; a `PreToolUse(Edit|Write)` pathcheck denies non-worktree writes. The
  11-vector escape battery was contained; necessity + fail-open both closed.
  Apparatus: `.doctrine/backlog/risk/014/probe-h1/` (`pretooluse-wrap.sh`,
  `pretooluse-pathcheck.sh`). Recipe:
  [[mem.pattern.dispatch.claude-worktree-subagent-bwrap-confinement]].
- **Existing hook machinery is Rust subcommands** — `boot --emit`, `worktree
  create-fork`, `worktree marker --stamp-subagent` — read stdin JSON, emit
  `hookSpecificOutput`, installed via `plan_hook` (settings) / embedded
  `hooks.json` (plugin), exec-path-injected (`resolve_exec`). The probe scripts are
  the anomaly.
- **Claude `/dispatch` funnel expects a worker commit** — delta-check (step 2):
  "net diff `B..S`, single non-merge commit, `S^ == B`" (`dispatch/SKILL.md:46`).
  Today the claude worker self-commits (`.git` writable).

## 3. Forces & Constraints

- **ADR-001** module layering: leaf ← engine ← command, no cycles. Pure logic
  separated from the io shell (pure/imperative split, AGENTS.md).
- **ADR-006 D-sole-writer** — orchestrator is the sole writer; per-worker jail
  policy is orchestrator-authored.
- **ADR-008** — doctrine's jail *is* the sandbox that closes what the generic
  harness cannot; this slice is its claude-arm discharge.
- **POL-002** — platform independence: no host-project coupling; fail-closed on
  unsupported platforms, never silent degrade.
- **STD-001** — no magic strings: bind flags, paths, prefixes are named constants.
- **Proven-ground discipline** — ride the bwrap flag set and binding facts the
  probe validated; do not re-derive.
- **Behaviour-preservation** — the existing entity-engine suites stay green; this
  is additive machinery.

## 4. Guiding Principles

- **Fail closed.** Any ambiguity (unresolved worktree, missing bwrap, **hook exec
  failure / missing binary**, parse error, policy-invalid) → deny. The strictest
  **write-containment** jail is the safe default; absence of policy can only
  *tighten* **the write surface**. (Egress is deliberately NOT part of this floor —
  `network` defaults to `true` for parity with the pi core flags; egress containment
  is a non-goal here, owned by OQ-6. The "can only tighten" invariant is scoped to
  the write walls — RV-200 F-7.)
- **Bind to `cwd`, not identity.** `cwd == hook pwd == worktree`,
  concurrency-stable; `agent_id` present ⟺ subagent. Pass through ⟺ `agent_id`
  absent.
- **Opaque wrap.** base64 the original command, decode+exec inside the jail; never
  parse the command to inject flags (shell-undecidability).
- **DRY the proven flags.** Single-source the bwrap core with the pi arm.
- **As simple as possible.** Land the floor (confine-to-worktree); defer clone
  topology (IDE-024) and selector-allowlist (IDE-025).

## 5. Proposed Design

### 5.1 System Model

Three new units under `src/worktree/`, layered:

```
 command      pretooluse.rs   (thin shell: stdin JSON in, hookSpecificOutput out,
                               bwrap-presence probe, policy-file read, resolve_exec)
   |  calls
 engine/leaf  jail.rs         (PURE: Decision, JailPolicy, bwrap argv builder,
                               opaque wrap, pathcheck predicate, footgun validation)
   |  reuses
 leaf         shared.rs       (is_linked_worktree, worktree recognition)
```

`mod.rs` gains `WorktreeCommand::Pretooluse` (mirrors `CreateFork` dispatch).

### 5.2 Interfaces & Contracts

**CLI:** `doctrine worktree pretooluse` — stdin = PreToolUse JSON; stdout =
`hookSpecificOutput` JSON or nothing (pass-through); exit 0 always (deny is
expressed in JSON, not exit code).

**Stdin shape (subset consumed):**
```
{ agent_id?, cwd, tool_name, tool_input: { command?, description?, file_path? } }
```
(`NotebookEdit`/`notebook_path` dropped — RV-200 F-6: the authoritative
`docs/claude` cache defines no `NotebookEdit` tool or `notebook_path` field, only a
matcher-regex example. `Edit`/`Write` is the documented, probe-proven write surface.
A notebook write-vector is re-added only once V-plugin captures its real matcher
name + stdin schema; guarding an unread tool would be a latent jail hole.)

**Pure core (`jail.rs`):**
```rust
enum Decision {
    PassThrough,                                       // emit nothing
    Deny { reason: String },                           // permissionDecision:"deny"
    WrapBash { command: String, description: String }, // allow + updatedInput
}

enum Target { Orchestrator, Jail(PathBuf), Reject(String) }

fn resolve_target(agent_id: Option<&str>, cwd: &Path, worktrees_root: &Path) -> Target;
//  None                         => Orchestrator
//  Some & is_worktree(cwd)      => Jail(cwd)
//  Some & !is_worktree(cwd)     => Reject("cwd-not-a-worktree")

fn decide_bash(target: &Target, cmd: &str, desc: &str, policy: &JailPolicy,
               bwrap_present: bool) -> Decision;
fn decide_write(target: &Target, file_path: Option<&Path>, policy: &JailPolicy) -> Decision;

fn bwrap_core_argv(wt: &Path) -> Vec<OsString>;          // == pi arm core
fn bwrap_argv(wt: &Path, policy: &JailPolicy) -> Vec<OsString>;
fn opaque_wrap(orig_cmd: &str, argv: &[OsString]) -> String;
fn pathcheck(real: &Path, wt: &Path, extra_rw: &[PathBuf]) -> bool; // ∈ {wt} ∪ extra_rw
fn validate_policy(policy: &JailPolicy, main_root: &Path) -> Result<(), String>;
```

`is_worktree(cwd)` is **git-topology-based**, not path-prefix: `cwd` is a linked
worktree (`shared::is_linked_worktree`) **whose `git-common-dir` resolves to this
project's main `.git`**. This avoids coupling to a directory layout (`.worktrees/`
vs `.dispatch/`) — a sibling repo's worktree at the same prefix would not match
(A1). Replaces the probe's hard-coded `$ROOT/.worktrees/agent-*` glob.

**Emitted JSON (mirrors probe):**
- deny → `{hookSpecificOutput:{hookEventName,permissionDecision:"deny",permissionDecisionReason:"worktree-jail: <reason>"}}`
- wrap → `…permissionDecision:"allow",updatedInput:{command,description}}`
- pass-through → no stdout.

### 5.3 Data, State & Ownership

**Per-arming jail policy** — `<main>/.doctrine/state/dispatch/jail/<worktree-name>.toml`
(runtime state: gitignored, `rm -rf`-able). **Outside every worktree** (a worker
rw's its own tree — must not author its own policy) and **ro to the worker** — and
the ro-ness derives from the bwrap **`--ro-bind / /`** blanket plus the file living
outside the single rw-bound worktree (and outside any validated `extra_rw`), **not**
from a filesystem-ancestor relationship (RV-200 F-8: worktrees sit at
`<root>/.worktrees/<name>`, so `.doctrine/state/` is a *sibling*, not an ancestor —
the repo root is the true ancestor, per INV-2; do not conflate the two).

```toml
extra_rw = []      # absolute paths granted rw inside the jail; default []
network  = true    # false => --unshare-net; default preserves current behavior
```

```rust
#[derive(Deserialize)]
struct JailPolicy { #[serde(default)] extra_rw: Vec<PathBuf>,
                    #[serde(default = "default_true")] network: bool }
impl Default { extra_rw: vec![], network: true }
fn load_policy(main_root: &Path, worktree_name: &str) -> JailPolicy; // missing => Default
```

**Keying — corrected (A7), serial-scoped (RV-200 F-1).** The original draft keyed
by `agent_id` written by the orchestrator pre-spawn. **That is impossible:**
`agent_id` is harness-assigned *at* spawn — the orchestrator cannot know it
beforehand. Resolved by riding the existing spawn handshake: the orchestrator
(`dispatch arm-spawn`) **pre-declares** the intended policy to a deterministic
location alongside the `base` file the `WorktreeCreate` hook already reads; the
**`worktree create-fork` hook** — which runs at spawn and *does* know the new
worktree (`name = agent-<id>`, payload) — **provisions** that declaration into
`<main>/.doctrine/state/dispatch/jail/<name>.toml`. The PreToolUse hook then resolves
policy by `cwd → basename(worktree) → file`. So `src/worktree/create.rs` is in the
touch-set (provision step), not just `src/dispatch.rs` (declare step).

**The single-slot constraint (RV-200 F-1, load-bearing).** The arming dir holds
**one** `base` file, and `dispatch-agent` issues N parallel spawns off one arming
(`dispatch-agent/SKILL.md`: "arm once, then issue N spawns … all read the same B").
The harness-assigned `name` does not exist until `create-fork` fires, so there is
**no pre-spawn key that distinguishes parallel siblings** — the slot's natural
granularity is **per-arming**, not per-worker. (This cannot be fixed by inverting to
spawn-then-declare: the claude `Agent` call **blocks until the worker completes**, so
the orchestrator has no turn between spawn and the worker's first tool call in which
to write a name-keyed policy; and the worker recording its own id would breach
ADR-006 sole-writer. True per-worker policy *is* natively achievable on the
**pi/subprocess arm**, where the orchestrator runs `worktree fork --worker` itself
and knows the name before spawn — that asymmetry is the whole of this constraint, and
the per-worker case is out of scope for the claude arm here.)

**Resolution — profile granularity is per-arming: serial ⇒ per-worker, parallel ⇒
one shared profile (RV-200 F-1).**
- **Serial drive** (one in-flight worker per arming): the single declared intent is
  unambiguous — `create-fork` binds the sole declaration to the sole new worktree, so
  per-arming *is* per-worker. Custom `extra_rw`/`network` is honoured. The
  **arm → spawn → create-fork-provision** sequence is the named critical section; it
  must not interleave a second arming.
- **Parallel fan-out** (N spawns off one arming): the one declared profile is
  **shared by every worker in the batch.** This is *intentional sharing at
  per-arming granularity, not a leak.* The reasoning, recorded so it can be
  challenged later: the slot can hold exactly **one** intent, so there is no *second*
  intent for a sibling's profile to cross-contaminate — every worker provisions from
  the same declaration, and the orchestrator (ADR-006 sole-writer) is responsible for
  declaring a profile valid for **all** members of the batch it is about to fan out.
  A worker can therefore never receive a profile *more permissive than the
  orchestrator chose for its arming.* We deliberately chose **"parallel workers share
  one profile"** over the stricter **"parallel workers get only the baseline floor"**:
  the latter needlessly forbids a legitimate batch-wide widening (e.g. a fan-out of
  file-disjoint phases that all need `network=false` or a shared `extra_rw`), and
  buys no safety the shared-intent model lacks — both are immune to the
  differing-siblings leak, because under one slot there are never differing siblings.
  The only thing genuinely unavailable on the claude arm is *distinct* profiles for
  *concurrent* siblings (that needs the pi arm or a future per-spawn token).

**Ownership:** orchestrator (ADR-006 sole-writer) is the source of the policy;
`create-fork` is its trusted provisioner (already an orchestrator-classed hook). GC
with worktree teardown. **Absence ⇒ `Default` (strictest floor)** — a worker spawned
with no declared policy is still jailed to its worktree. No worker TOCTOU
(provisioned before the worker's first call, ro in jail, read by the hook process not
the worker's command). The earlier unconditional "no parallel-write contention; no
per-worker contention" claim is **retracted and replaced** by the per-arming
granularity model above: contention is impossible because a single arming carries a
single intent — not because each worker owns a private file the orchestrator could
race to populate.

### 5.4 Lifecycle, Operations & Dynamics

**Registration** (decided: D-reg) — via the embedded **plugin `hooks.json`**
(`plugins/doctrine/hooks/hooks.json`, RustEmbed → materialized to
`.claude/skills/doctrine/hooks/hooks.json`, auto-discovered), the same seam that
already carries `SessionStart`/`WorktreeCreate`. Two PreToolUse entries: matcher
`Bash` and matcher `Edit|Write`, both → `doctrine worktree pretooluse`.

**Exec resolution must be FAIL-CLOSED (RV-200 F-2, blocker).** A PreToolUse hook
that errors with any non-`2` exit is a *non-blocking* error and the tool call
**proceeds** (`docs/claude/hooks.md:629-643` + Warning: "only exit code 2 blocks");
`command-not-found` (127) qualifies. So a **bare `doctrine` on PATH** that resolves
to the RO `~/.cargo/bin/doctrine` predating this subcommand — or is simply absent —
would let Bash/Edit/Write run **unconfined**: the exact RSK-014 hole this slice
closes, reopened by the installer. Therefore the registration must invoke a
**resolved, present executable** (the `resolve_exec` path already named in §5.1/D1):
the materialized `hooks.json` carries an **absolute** `doctrine` path (or a tiny
checked-in shim that `exit 2`s on exec/`not-found`), so a missing/stale binary
**denies** rather than passes. This supersedes the earlier "bare exec on PATH, no
resolve_exec" wording and reconciles §5.1/D1 ↔ §5.4 onto one fail-closed story.
(Bonus: an absolute resolved path also ensures V-plugin/e2e exercise the **dev
build under test**, not the stale RO binary.)

D-reg is preferred on user steer (prior empirical tests show plugin hooks uniform
with settings hooks) **but gated** — a re-test (V-plugin, §9) must confirm
PreToolUse-via-plugin fires for a worktree subagent *and honours `updatedInput`*
before this is relied on; the probe proved the mechanism via `settings.local.json`,
not the plugin path. **The `settings.local.json` fallback is a PLANNED contingency
of the V-plugin phase, not deferred-on-failure (RV-200 F-5):** that phase lands the
settings.local install path *iff* the plugin path fails the re-test — it is not
pre-built, but it is scoped and ready, never aspirational. **Plugin `hooks/` changes
are not hot-reloaded** (`docs/claude/plugins-reference.md:394`) — pick them up via
`/reload-plugins` (lighter) or a session restart; the runbook documents this + the
Edit/Write escape hatch (a broken Bash wrapper is disablable via Edit +
`/reload-plugins`).

(Housekeeping: the live probe hooks were cleared from `settings.local.json` —
backup at `.claude/settings.local.json.backup` — so the slice installs onto a
clean slate.)

**Per-call flow:** harness → PreToolUse → `doctrine worktree pretooluse` (stdin) →
`resolve_target` → bash/write decision → emit. Binary startup ≈ 2 ms (measured),
negligible per call.

**Funnel convergence (objective 5) — diff captured BEFORE teardown (RV-200 F-3,
blocker).** With ro-`.git`, the jailed worker cannot self-commit (its object store
is the ro main `.git`), so the claude `/dispatch` funnel converges onto a
**working-tree-diff** import. The naïve "import the worker's worktree after it
returns" is **unsafe**, and the prior "already identical on both arms" premise was
**false**: the two arms are *not* lifecycle-equivalent. On the pi/subprocess arm the
**orchestrator** owns the worktree (`worktree fork --worker` → import → orchestrator
removes), so the tree persists until import. On the claude arm the **harness** owns
it: when the `isolation:worktree` subagent finishes, Claude auto-runs `git worktree
remove`, and `WorktreeRemove` has **no decision control** — failures are debug-log
only (`docs/claude/hooks.md:2442, :680, :814`). The hook **cannot block** teardown,
so an uncommitted diff is destroyed in the race between subagent-done and removal.

**Contingency (decided — snapshot before remove).** A doctrine **`WorktreeRemove`
(and/or `SubagentStop`) hook** captures `git -C <worktree> diff` (and untracked
adds) into a patch at a path **outside** the worktree — under the coord tree's
runtime state — *before* the harness removes the tree. The orchestrator imports
**that captured patch**, not the live worktree. This finally makes the cadence
genuinely identical on both arms (each imports a captured delta, not a live tree)
and keeps Path L + ro-`.git` intact. Touch-set gains the `WorktreeRemove` capture
hook alongside the existing `create-fork`. Edits to `dispatch-agent/SKILL.md`:
import source = the captured patch against B (not the `B..S` commit); relax the
single-commit delta-check on this arm; `verify-worker` adjusts. /plan confirms
whether the delta-check is skill-orchestration or Rust (`src/dispatch.rs`) and
scopes the touch.

**OQ-2 is a lock-time risk WITH a defined abort, not a bare "verify later".** The
capture-before-remove timing (does the `WorktreeRemove` hook reliably run, with the
worktree still on disk, before `git worktree remove` completes?) is the residual
unknown. Abort criterion: if the capture hook cannot observe the tree intact, Path L
is unworkable for the claude funnel and the slice **escalates to Path C / IDE-024**
(standalone clone + self-commit + cherry-pick) — the named fallback. This is proven
or refuted as the second execute gate (after V-plugin), §9.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1** pass through ⟺ `agent_id` absent. `agent_id` present + non-worktree
  cwd ⇒ deny (the `isolation:none` arm — proven denied, RSK-014 Exp 3).
- **INV-2** repo-root write denied by the **ancestor rule** (`realpath ⊄ wt`), not
  by native's race-win — pin in a synthetic-stdin test (recipe memory).
- **INV-3** `.git` is ro and **not tunable**: `validate_policy` rejects any
  `extra_rw` entry equal to `/`, an ancestor of `main_root`, or touching `.git`.
- **INV-4** Edit/Write allowlist = `{wt} ∪ extra_rw` — coherent with what the Bash
  jail rw-binds. **Safe only because `validate_policy` already rejected dangerous
  `extra_rw`** (root-ancestors/`.git`); the pathcheck trusts a validated policy
  (A6 cross-link to INV-3).
- **INV-5 (A3) — robust shell-quoting.** `opaque_wrap` interpolates `wt` and every
  `extra_rw` path into the emitted `updatedInput.command` shell string. All
  interpolated paths MUST be single-quote-escaped (paths may contain spaces; an
  `extra_rw` entry is orchestrator-supplied). The original command rides as
  charset-safe base64 (never re-parsed). Test: a worktree path / `extra_rw` with a
  space and a single quote round-trips and executes correctly.
- **Edge:** `/tmp` is a private `--tmpfs` for Bash (ephemeral, never host /tmp) and
  denied for Edit/Write — restrictive default; loosen a run via `extra_rw`.
- **Edge:** non-bwrap platform → `deny "bwrap-unavailable"` (fail-closed; macOS =
  IMP-045), never unwrapped pass-through.
- **Assumption (verify):** PreToolUse via `settings.local.json` fires for worktree
  subagents — proven (probe). The plugin `hooks.json` path is **not** assumed — it is
  V-plugin-gated with the settings.local path as a planned same-phase fallback
  (RV-200 F-5). The wire fields it relies on are doc-backed: `agent_id`
  (`hooks.md:595`), `updatedInput` (`hooks.md:818`), `permissionDecision`
  (`hooks.md:806`); only the plugin-*registration* firing is unproven.

## 6. Open Questions & Unknowns

- **OQ-1 (→/plan)** funnel delta-check location — skill-orchestration vs Rust
  (`src/dispatch.rs`). Scopes the objective-5 touch.
- **OQ-2 (lock-time risk, DEFINED ABORT — RV-200 F-3)** the harness auto-removes the
  worktree on subagent finish (`WorktreeRemove`, no decision control), so the worker
  diff cannot be imported from a live tree. **Resolved by design** to a
  capture-before-remove hook (§5.4); the residual unknown is purely whether that hook
  observes the tree intact before `git worktree remove`. **Abort:** if not, escalate
  to Path C / IDE-024. Second execute gate (§9), not an open-ended verify.
- **OQ-3 → V-plugin (first step in execute).** Plugin `hooks.json` is the chosen
  registration home (D-reg). Confirm PreToolUse-via-plugin fires for a worktree
  subagent before building on it; cross-check hook semantics against `docs/claude`
  (local official-docs cache — authoritative over web/subagent).

## 7. Decisions, Rationale & Alternatives

- **D1 — Rust subcommand** (`doctrine worktree pretooluse`), not bash. Rides the
  existing hook seam; reuses worktree resolution + `resolve_exec`; testable;
  single bwrap-flag source. Startup ≈ 2 ms. *Alt: bash scripts — rejected
  (anomaly, jq/bash dep, magic strings, untestable, DRY violation).*
- **D2 — policy file keyed by WORKTREE NAME, per-arming granularity** (RV-200 F-1/F-4;
  corrects the original `agent_id` keying §5.3 proved impossible). File at
  `<main>/.doctrine/state/dispatch/jail/<worktree-name>.toml`; orchestrator
  pre-declares the intent, `create-fork` provisions it under the name it learns at
  spawn. Granularity is per-arming: **serial ⇒ per-worker; parallel ⇒ one profile
  shared by the batch** (§5.3 rationale). *Alt: keyed by `agent_id` — impossible
  (harness-assigned at spawn, unknown pre-declaration); single run-level map (write
  contention, heavier); per-worker concurrent profiles (needs the pi arm / a future
  per-spawn token).*
- **D3 — `.git` hard ro, not a config knob.** Loosening `.git/objects` rw lets the
  worker write arbitrary blobs into the shared store = defeating the jail
  (case-notes SL-171). The no-self-commit consequence is the established funnel.
- **D4 — Path L (linked worktree + ro-`.git`).** Proven; converges both arms onto
  one funnel. *Alt: Path C (standalone clone, self-commit, cherry-pick) — better
  orchestrator efficiency but a topology change, likely ADR-altitude → IDE-024.*
- **D5 — single-sourced bwrap core flags** via `bwrap_core_argv` + a **parity
  test** against `pi-spawn-confined.sh`; leave the live pi script untouched. *Alt:
  extract `worktree jail-argv` consumed by both — true DRY but touches live pi
  dispatch → follow-up.*
- **D-reg — register via the plugin `hooks.json`** (`plugins/doctrine/hooks/`),
  invoking a **resolved absolute** `doctrine` (fail-closed exec, RV-200 F-2 — NOT
  bare PATH). Preferred on user steer — prior empirical tests show plugin hooks
  uniform with settings hooks; rides the existing auto-discovered seam. **Gated on
  V-plugin** (re-test PreToolUse-via-plugin fires for a worktree subagent *and*
  honours `updatedInput`). *Alt: `settings.local.json` via boot.rs `plan_hook` — the
  probe's proven path. It is a **planned contingency of the V-plugin phase** (built
  iff the plugin path fails the re-test), not pre-built and not merely "on failure"
  — scoped and ready (RV-200 F-5).*
- **D6 — schema = `extra_rw` + `network`.** Dropped `extra_ro` (redundant under
  `--ro-bind / /`) and `strict/loose mode` (the floor *is* strict; loosening ==
  `extra_rw`). Footgun violations **deny** (fail-closed).

## 8. Risks & Mitigations

- **R1 — funnel breakage.** Confinement removes claude self-commit → breaks the
  `B..S` delta-check, AND the harness tears the worktree down before import (RV-200
  F-3). *Mit:* capture-before-remove convergence (§5.4) in scope; capture-before-remove
  gate (§9, second execute gate) with a defined abort to Path C / IDE-024 before close.
- **R2 — plugin registration unproven for PreToolUse.** D-reg chooses the plugin
  path (user steer: empirically uniform). *Mit:* V-plugin re-tests it as the first
  execute step; settings.local.json fallback **built in that same phase iff the
  plugin path fails** (RV-200 F-5), not held aspirationally. Verify hook semantics
  against `docs/claude`, not web/subagent.
- **R3 — bwrap-flag drift** between the Rust builder and the pi script. *Mit:* D5
  parity test fails on divergence.
- **R4 — policy TOCTOU / forged-absent.** *Mit:* absence ⇒ strictest jail (can only
  tighten); provisioned before the worker's first call, ro in jail.
- **R5 — harness change reopens repo-root** (native checkout-guard dropped). *Mit:*
  INV-2 ancestor-rule deny holds independently (pinned test).
- **R6 — non-Linux silent hole.** *Mit:* INV — fail-closed deny when bwrap absent.
- **R7 — orchestrator pass-through is unconfined (god-mode).** Pass-through trusts
  an un-authenticated tell (`agent_id` absent; `session_id` shared). A phantom
  no-`agent_id` spawn would inherit it. *Accepted residual* — and defensible (RV-200
  F-9, acquitted): `agent_id` is **harness-stamped present-iff-subagent** (probe
  Exp 1/3), so a *confined worker cannot forge its absence*; the only no-`agent_id`
  context is the trusted orchestrator main thread (ADR-006 sole-writer). A phantom
  would require a harness-integrity failure that equally breaks worker/orchestrator
  discrimination everywhere — which OQ-5's "widest jail" would barely contain. The
  structural close is **OQ-5 — jail the orchestrator too**, deferred. Named, reasoned,
  not silent.
- **R8 — convergence imposes the pi arm's verify-cost on the claude arm.** Losing
  worker self-commit means the worker can no longer commit-gate a self-verify; the
  orchestrator inherits the pi arm's "can't trust worker green → re-run suite"
  cost (case-notes SL-171, hollow greens). This is a deliberate **efficiency
  regression traded for confinement** — exactly the driver for **IDE-024 (Path
  C)**. Named so the tradeoff is visible, not discovered post-hoc.

## 9. Quality Engineering & Validation

- **Unit (pure, TDD red/green/refactor):** `resolve_target` (3 arms, topology-based
  recognition incl. a sibling-repo worktree → not-jailed-here, A1);
  `pathcheck` (⊆wt / escape / extra_rw-hit / `.git`-reject); `load_policy`
  (default / present / malformed); `bwrap_argv` (core + extra_rw + `network`);
  `opaque_wrap` (base64 round-trip **+ INV-5 path with space & single-quote**
  round-trips & executes); `validate_policy` (reject `/`, root-ancestor, `.git`).
- **Integration (synthetic stdin → emitted JSON):** the probe escape battery
  re-expressed as cases; INV-2 repo-root-ancestor deny; orchestrator pass-through
  (no `agent_id`); isolation:none deny (`agent_id` + repo-root cwd); D5 parity
  test; **keying/provision (A7): `create-fork` provisions the declared policy to
  `jail/<name>.toml`; PreToolUse resolves it by `cwd → basename`**; **per-arming
  granularity (F-1): serial arming binds the sole intent per-worker; a parallel batch
  off one arming provisions the SAME profile to every sibling (shared, not leaked);
  absence ⇒ Default floor**; **fail-closed exec (F-2): a missing/non-resolving
  `doctrine` denies (exit 2 / shim), never passes through unconfined**.
- **V-plugin (FIRST execute step — gate on D-reg):** confirm a PreToolUse hook
  registered via the plugin `hooks.json` fires for a worktree subagent (Bash +
  Edit/Write) **and honours `updatedInput`**, exactly as the probe proved via
  `settings.local.json`. Cross-check hook-event/matcher/`updatedInput` semantics
  against `docs/claude`. Fail ⇒ land the settings.local.json fallback **in this same
  phase** (F-5).
- **Capture-before-remove (SECOND execute gate — OQ-2 / F-3):** confirm the doctrine
  `WorktreeRemove`/`SubagentStop` hook captures the worker's worktree diff to an
  outside-the-worktree patch **before** the harness's `git worktree remove`, and the
  orchestrator imports that patch. Fail (tree gone before capture) ⇒ **abort to Path
  C / IDE-024** (the named fallback), do not ship a lossy funnel.
- **End-to-end (VA/VH — the riskiest leg):** live claude `/dispatch`, one jailed
  worker, escape vectors denied + canaries intact + funnel completes green
  (captured-patch import). Covers OQ-2 end-to-end after the second gate.
- **Behaviour-preservation:** existing worktree/dispatch suites stay green.

## 10. Review Notes

### Internal adversarial pass (2026-07-01) — 8 findings, all integrated

- **A1 — fragile worktree recognition.** Path-prefix (`.worktrees/`) is a layout
  coupling and misses `.dispatch/`-style trees. → §5.2 now git-topology-based
  (`is_linked_worktree` + `git-common-dir == main .git`).
- **A3 — shell-quoting in `opaque_wrap`.** Interpolated `wt`/`extra_rw` paths could
  carry spaces/quotes and break the emitted command. → INV-5 + test.
- **A6 — pathcheck trusts extra_rw.** Allowlist `{wt} ∪ extra_rw` is safe only
  because `validate_policy` pre-rejected dangerous entries. → INV-4 cross-link.
- **A7 — KEYING FLAW (substantive).** Original `agent_id`-keyed, orchestrator-
  pre-writes model is impossible — `agent_id` is harness-assigned at spawn. →
  §5.3 rewritten: key by worktree name; `create-fork` provisions the orchestrator's
  pre-spawn declaration; `src/worktree/create.rs` added to touch-set.
- **A2/R7 — orchestrator pass-through is god-mode.** Named as accepted residual
  (OQ-5 deferred), not silent.
- **A5/R8 — convergence efficiency regression.** Converging claude→pi funnel
  inherits the verify-after-import cost; named as the IDE-024 driver.
- **A4 — `network=true` default = no egress containment.** Stated explicitly (the
  knob exists; the default does not close egress — consistent with the non-goal).
- **A8 — D5 parity-test mechanism.** Compare the Rust core against a checked-in
  expected vector + a cross-ref comment in `pi-spawn-confined.sh`; true shared
  consumption is the D5-alt follow-up.

Plus user steers integrated: D-reg flipped to the **plugin `hooks.json`** path
(gated on V-plugin); verify hook semantics against `docs/claude` (local
official-docs cache), not web/subagent.

**`docs/claude` cross-check (2026-07-01):** plugin hooks support `PreToolUse`
("same lifecycle events as user-defined hooks… Before a tool call executes. Can
block it", `plugins-reference.md:111-119`) — de-risks R2/D-reg; matcher regex
`Write|Edit` valid (`:98`); plugin `hooks/` not hot-reloaded, `/reload-plugins`
suffices (`:394`). The stdin wire fields ARE doc-backed (RV-200 F-10): `agent_id`
(`hooks.md:595`, "present only when the hook fires inside a subagent call" — which
also confirms hooks DO fire inside subagent calls), `updatedInput` (`hooks.md:818`),
`permissionDecision` (`hooks.md:806`). The probe remains the proof these fire for a
worktree subagent specifically; V-plugin's residual is narrowly the plugin
*registration* path, not the field semantics.

### `/inquisition` findings (RV-200, 2026-07-01) — codex GPT-5.5 + inquisitor, all integrated

Tried on the ledger (`doctrine review show RV-200`); 10 findings, 3 blockers; all
reconciled into this revision. Two carried User-decided remediation options.

- **F-1 (blocker) — per-arming, not per-worker.** Single-slot arming rendezvous
  can't key custom policy per parallel sibling. → §5.3 / D2 rewritten: serial ⇒
  per-worker; **parallel ⇒ one shared profile** (User steer: prefer "share one
  profile" over "baseline-only"); absence ⇒ Default floor; false "no contention"
  claim retracted.
- **F-2 (blocker) — bare-PATH hook fails OPEN.** Only `exit 2` blocks
  (`hooks.md:629-643`); a stale/missing binary runs unconfined. → §5.4 / D-reg /
  §4: registration invokes a **resolved absolute** exec (or exit-2 shim);
  §5.1/D1↔§5.4 reconciled to one fail-closed story.
- **F-3 (blocker) — `WorktreeRemove` auto-teardown destroys the diff.** Hook has no
  decision control (`hooks.md:2442/680/814`); "identical on both arms" was false. →
  §5.4: **capture diff before remove**; OQ-2 reframed as a lock-time risk with a
  defined abort to Path C / IDE-024.
- **F-4 (major) — stale `agent_id` keying** in D2 + scope. → D2 rewritten to
  worktree-name keying; scope doc corrected.
- **F-5 (major) — fallback forbidden, not planned.** → D-reg / §9: settings.local
  fallback is a planned same-phase contingency of the V-plugin gate.
- **F-6 (major) — undocumented `NotebookEdit`.** → §5.2 / §5.4: dropped to the
  proven `Edit|Write` surface; re-add only once V-plugin reads its schema.
- **F-7 (minor) — `network=true` vs "strictest floor".** → §4: invariant scoped to
  *write*-containment; egress an explicit non-goal (OQ-6).
- **F-8 (minor) — false "ancestor" rationale.** → §5.3: ro-ness pinned to
  `--ro-bind / /`, not ancestry.
- **F-9 (nit, ACQUITTED) — R7 god-mode residual is defensible.** `agent_id` is
  harness-stamped present-iff-subagent (probe), so a confined worker cannot forge its
  absence; the only no-`agent_id` context is the trusted orchestrator main thread;
  OQ-5 deferral is sound. Soft-target-4 answered: accepted, not must-land. (R7 text
  may gain the unspoofability premise as cosmetic polish.)
- **F-10 (nit) — §10 doc-coverage undersell.** → corrected in the cross-check above.
