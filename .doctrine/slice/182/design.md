# Design SL-182: Claude-arm subagent write-confinement hooks

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

Status: LOCKED, then AMENDED at PHASE-05 (§10 amendment entry — funnel pivoted to
symmetric live-import after the teardown premise was disproved by live probe;
D-funnel-path / D-import-verb / INV-6; OQ-1 + OQ-2 resolved; then hardened by the
RV-205 inquisition — footer datum cited to `mem P2`, INV-6 made two-boundary, reap
gated on import success, Fork-path ASM grounded in hook-presence). Prior lock: internal
adversarial pass + `/inquisition` RV-200 + RV-201 + SL-183 cross-arm seam upstream +
RV-202 codex pass (corrected the upstream's `select_jailer` to capability-as-data) —
all integrated; §10. The Phase-1 probe (D7, §9) resolved the harness unknowns:
plugin-PreToolUse firing (PROVEN), `SubagentStop` blocking/tree-intact (PROVEN but
now moot — the capture funnel is retired), and the tree persists on disk post-return
(PHASE-05 probe) — so the claude arm imports the live tree, no capture. Governed by
ADR-008 (closes its claude-arm confinement gap), ADR-006 (D2b raw-tree confinement;
D-sole-writer). Originates from RSK-014 (probe-h1, PROVEN). Path C deferred → IDE-024
(now also the foreseen future gather strategy behind `--from-worktree`, D-import-verb);
selector-allowlist mode → IDE-025.

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
  `hookSpecificOutput`. Installed two ways with **divergent exec resolution**: the
  **settings** path bakes an absolute exec via `HookSpec::boot(resolve_exec())`
  (`src/boot.rs:1120`); the embedded **plugin `hooks.json`** path **byte-copies the
  asset verbatim** (`install_hooks_plugin_for_claude` → `write_atomic`,
  `src/skills.rs:1046`), so it ships **bare `doctrine`** — the F-1 fail-open anomaly
  D-reg closes by templating (§5.4). The probe scripts are the other anomaly.
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
  parse the command to inject flags (shell-undecidability). **Wrapper-agnostic** —
  `opaque_wrap` takes the jailer's argv as *input* (it quotes+assembles whatever
  wrapper argv it is handed + the base64 command), so it is reused unchanged when the
  wrapper is `sandbox-exec` not `bwrap` (SL-183 / brief §2).
- **DRY the proven flags.** Single-source the bwrap core with the pi arm.
- **Platform seam, not platform branch (SL-183 parity, brief §2/§7 D-mac2).**
  Everything platform-agnostic — `resolve_target`, `decide_bash`, `decide_write`,
  `pathcheck`, `opaque_wrap`, `validate_policy` — sits **above** a single named fork
  point (`Jailer`); only the argv/profile builder (`bwrap_argv`/`bwrap_core_argv`)
  sits **below** it. The macOS arm (IMP-045/SL-183) is a second `Jailer` impl
  (`seatbelt_profile` + `sandbox_exec_argv`) behind the same seam — *not* a refactor
  of this core. Designed now so SL-183 slots in; macOS impl deferred. See D8.
- **As simple as possible.** Land the floor (confine-to-worktree); defer clone
  topology (IDE-024) and selector-allowlist (IDE-025).

## 5. Proposed Design

### 5.1 System Model

Three new units under `src/worktree/`, layered:

```
 command      pretooluse.rs   (thin shell: stdin JSON in, hookSpecificOutput out,
                               host probe ⇒ `Backend` descriptor, policy-file read)
   |  calls (passes Backend in — capability is DATA, never read in the leaf)
 engine/leaf  jail.rs         (PURE)
                ├─ platform-agnostic core (ABOVE the seam): Decision, Target,
                │   JailPolicy, resolve_target, decide_bash, decide_write,
                │   opaque_wrap, pathcheck, validate_policy
                └─ Jailer seam (the SINGLE fork point, D8) — maps `Backend` → impl:
                    · Bwrap  → bwrap_core_argv / bwrap_argv          (this slice)
                    · Seatbelt → seatbelt_profile / sandbox_exec_argv (SL-183, deferred)
                    · Deny{reason} → deny  (absent / unsupported / degraded; C/§5.5)
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

**Pure core (`jail.rs`) — platform-agnostic, ABOVE the Jailer seam:**
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

// decide_* are backend-neutral: capability arrives as DATA from the shell (`Backend`,
// below), never read from the host here — the pure/imperative split (AGENTS.md). A
// `Deny{reason}` backend ⇒ `Decision::Deny` carrying that reason; capability-keyed,
// never a hardcoded else (C/§5.5). The shell owns host detection; the core only maps.
fn decide_bash(target: &Target, cmd: &str, desc: &str, policy: &JailPolicy,
               backend: &Backend) -> Decision;
fn decide_write(target: &Target, file_path: Option<&Path>, policy: &JailPolicy) -> Decision;

fn opaque_wrap(orig_cmd: &str, argv: &[OsString]) -> String;   // wrapper-agnostic (B): quotes+
                                                               // assembles ANY argv + b64 cmd
fn pathcheck(real: &Path, wt: &Path, extra_rw: &[PathBuf]) -> bool; // ∈ {wt} ∪ extra_rw
fn validate_policy(policy: &JailPolicy, main_root: &Path) -> Result<(), String>;
//  ^ STRICTLY platform-agnostic, the shared cross-arm contract (D, brief §2): zero
//    bwrap/namespace assumptions; reused UNCHANGED by SL-183 as its parity proof.
```

**Jailer seam (`jail.rs`) — the SINGLE fork point (D8), BELOW which backends differ:**
```rust
trait Jailer {                       // selected once per call from the `Backend` descriptor
    fn wrap_argv(&self, wt: &Path, policy: &JailPolicy) -> Vec<OsString>; // the wrapper argv
}                                    // opaque_wrap consumes whatever this returns

struct Bwrap;                        // THIS slice — Linux
impl Jailer for Bwrap { /* bwrap_core_argv (== pi arm) + extra_rw + network */ }
fn bwrap_core_argv(wt: &Path) -> Vec<OsString>;          // == pi arm core (D5 parity)
fn bwrap_argv(wt: &Path, policy: &JailPolicy) -> Vec<OsString>;

// struct Seatbelt;  — SL-183 / IMP-045 (deferred): seatbelt_profile + sandbox_exec_argv
//                     behind THIS trait; no core change.

// Capability is DATA, resolved by the shell's host probe (§5.1) and passed in — the
// pure core never reads OS / binary-presence / nesting state (AGENTS.md pure/imperative
// split). THREE states, not a bare Option: a backend that is *present-but-degraded*
// (e.g. SL-183 Seatbelt nesting refused, brief §3a) is a `Deny{reason}` — distinct from
// absent, so SL-183 widens nothing. The deny reason rides per-arm, so no flattening.
enum Backend {
    Bwrap,                   // Linux + bwrap present
    Seatbelt,                // macOS + sandbox-exec present + nesting OK (SL-183; never built today)
    Deny { reason: String }, // unsupported / absent / probed-but-degraded; e.g.
}                            //   Linux-no-bwrap ⇒ "bwrap-unavailable"; macOS-today ⇒ "seatbelt-unavailable"

fn select_jailer(backend: &Backend) -> Option<Box<dyn Jailer>>;  // PURE map, NO host read:
//  Bwrap ⇒ Some(Bwrap); Seatbelt ⇒ Some(Seatbelt) (SL-183); Deny{..} ⇒ None ⇒ deny (§5.5).
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

**Keying — corrected (A7), serial-scoped (RV-200 F-1), machinery named (RV-201
F-4).** The original draft keyed by `agent_id` written by the orchestrator
pre-spawn. **That is impossible:** `agent_id` is harness-assigned *at* spawn — the
orchestrator cannot know it beforehand. Resolved by riding the existing spawn
handshake across **two named files** with a defined lifecycle:

| File | Written by | Read by | Lifecycle |
|---|---|---|---|
| `<coord>/.doctrine/state/dispatch/spawn/jail.toml` (the **arming declaration**, beside the existing `base` in `ARMING_SUBPATH`, `src/worktree/create.rs:202`) | orchestrator `dispatch arm-spawn` (`src/dispatch.rs`) | `create-fork` hook | overwritten on every (re-)arm, in the **same arming step** as `base`; absent ⇒ Default floor |
| `<main>/.doctrine/state/dispatch/jail/<worktree-name>.toml` (the **provisioned policy**) | `create-fork` hook (`src/worktree/create.rs`, **NET-NEW** step) | `pretooluse` hook (`cwd → basename(worktree) → file`) | GC'd with worktree teardown |

`create-fork` already knows the new worktree (`name = agent-<id>`, payload) and
already runs `run_provision` + `write_marker`; the jail-policy provision is a
**third, net-new** step beside them (`classify_create`/`fork_core` write nothing
under `jail/` today — F-4). So the touch-set is `src/dispatch.rs` (declare,
`arm-spawn` extended to write `jail.toml` alongside `base`) **and**
`src/worktree/create.rs` (provision), patterned on `src/worktree/marker.rs`'s
`write_marker`.

**Pairing of `jail.toml` with `base` — by structure, not lock (RV-201 F-4).**
`base` is overwritten idempotently on re-arm (`src/dispatch.rs`); a stale
`jail.toml` paired with a fresh `base` (or vice versa) would mis-provision. The
contract: **`arm-spawn` writes both in one arming step**, and re-arm rewrites both;
there is no separate "update jail.toml" path. This is safe because **the
orchestrator is single-threaded and every claude `Agent` spawn call BLOCKS until
the worker completes** — a parallel batch issued in one turn blocks until *all* N
return — so the orchestrator has **no turn between arming and batch-completion in
which to re-arm**. The pairing atomicity is therefore enforced by the
blocking-call structure, not by a filesystem lock or asserted discipline (this
re-grounds the "must not interleave a second arming" claim below).

**INV-6 — no background spawn while the arming slot is live (RV-202).** The
blocking-call structure above holds **only** for foreground `Agent` calls. The
claude `Agent` tool supports `run_in_background: true`, which returns immediately
(`docs/claude/hooks.md:1428`) — a background spawn would hand the orchestrator a
turn mid-sequence and reopen the re-arm race. The structural guarantee is therefore
conditioned on a hard invariant: **`/dispatch` MUST NOT issue a `run_in_background`
`Agent` spawn against a live arming slot.** Today's `dispatch-agent` template omits
the field (foreground — `dispatch-agent/SKILL.md`), so the invariant holds by
construction; it is stated here so a future background-spawn optimisation cannot
silently void the pairing atomicity. (A per-spawn policy token — out of scope, the
pi-arm asymmetry — is the only thing that would make background spawns safe.)

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
  **arm → spawn → create-fork-provision** sequence cannot interleave a second
  arming — **not by discipline but by structure**: the blocking `Agent` call holds
  the orchestrator's single thread from arm through worker-completion, so no
  re-arm turn exists mid-sequence (the `jail.toml`↔`base` pairing above).
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

**Exec resolution must be FAIL-CLOSED — by INSTALL-TIME TEMPLATING (RV-201 F-1,
blocker; User-decided Option A).** A PreToolUse hook that errors with any non-`2`
exit is a *non-blocking* error and the tool call **proceeds**
(`docs/claude/hooks.md:629-643` + Warning: "only exit code 2 blocks");
`command-not-found` (127) qualifies. So a **bare `doctrine` on PATH** that resolves
to a binary predating this subcommand — or is simply absent — lets Bash/Edit/Write
run **unconfined**: the exact RSK-014 hole this slice closes, reopened by the
installer.

The fix lives at **materialization, not runtime** (the coherence twin F-5):
`install_hooks_plugin_for_claude` (`src/skills.rs:1024-1052`) today **verbatim
byte-copies** the embedded `hooks.json` (`PluginAssets::get` → `write_atomic`,
no substitution), so its bare-`doctrine` commands ship fail-OPEN — whereas the
*settings* path already bakes an absolute exec via `HookSpec::boot(resolve_exec())`
(`src/boot.rs:1120`). The plugin path is the anomaly. **D-reg Option A
(decided):** `install_hooks_plugin_for_claude` gains a **templating pass** —
rewrite the **leading `doctrine` token** of *every* `command` string in the
embedded `hooks.json` to `resolve_exec()`'s absolute path (`SessionStart`,
`WorktreeCreate`, the two `PreToolUse` entries — the `SubagentStop` capture entry
is **removed** at PHASE-05, no longer templated),
bringing the plugin path to parity with the settings path. Leading-token replace
(args untouched, so the checked-in asset stays valid as authored); the token
`doctrine` and each subcommand string are **STD-001 named constants**; the
absolute path is **single-quote-escaped** into the command string (it may contain
spaces — same quoting discipline as INV-5). *Rejected: an embedded shim — it
reintroduces the bash anomaly D1 explicitly rejected, needs a second materialized
asset, and still bakes `resolve_exec` into the shim (templating, one layer down).*

**Interaction with pre-baked installs (no client compilation assumed).**
`resolve_exec` = `current_exe()` (the resolved real path) → `pick_exec`, which
**bails** ("reinstall from a stable location") if that path is gone
(`src/boot.rs:433-456`). For the 99% GitHub-release flow — prebuilt binary at a
**stable location** (`~/.local/bin`, `/usr/local/bin`) — the baked path is fixed,
present, and survives in-place upgrades: templating is strictly safer than bare
PATH (which could resolve to a *different*, older binary). The one residual is
**content-addressed installs** (nix store): `current_exe()` bakes the
version-pinned store path, so a flake upgrade + store-GC *before* re-running
`doctrine claude install` leaves `hooks.json` pointing at a GC'd path → `127` →
fail-open for `pretooluse` (`SessionStart`/`WorktreeCreate` fail *closed* — the
latter aborts on any non-zero). Two guards: **(a)** the **reinstall-on-upgrade
invariant** — already required for memory/embed refresh, and asserted by
`pick_exec`'s bail at every other bake site; **(b)** a **V-plugin-gated inline
`|| exit 2` guard** appended to the `pretooluse` command (`<abs> worktree
pretooluse || exit 2`), which converts the vanish-case `127` (and any
mid-run crash) into a blocking `exit 2` → **deny** — closing even the nix window
without a bash asset, *iff* V-plugin confirms hook `command` is shell-run (§9).
(Bonus: an absolute resolved path also ensures V-plugin/e2e exercise the **dev
build under test**, not a stale RO binary.)

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

**Funnel convergence (objective 5) — orchestrator imports the LIVE worktree
(AMENDED 2026-07-01, PHASE-05 live probe — supersedes the SubagentStop-capture
design; §10 amendment entry + git history carry the retired text).** With
ro-`.git`, the jailed worker cannot self-commit, so the claude `/dispatch` funnel
still converges onto a **working-tree-diff** import. What changed is *where the
diff is read from*. The retired design assumed the harness auto-tears-down the
worker worktree on subagent finish, forcing a capture-before-teardown hook. **A
PHASE-05 live probe disproved that premise for the shipped config.**

**The teardown premise was false (probe, 2026-07-01).** Claude auto-runs `git
worktree remove` only for worktrees **it created natively**. Doctrine ships
`create-fork` **as** the `WorktreeCreate` hook, which "replaces the default git
behavior entirely" (`hooks.md:2390`) — Claude receives only a path and cannot
assume git. Per `hooks.md:2442`, a custom WorktreeCreate hook with **no**
WorktreeRemove hook leaves the directory **on disk**. Probe: one
`isolation:worktree` subagent under production hooks (create-fork present, no
WorktreeRemove) → post-return the worktree **persisted** (`.worktrees/agent-<id>`
in `git worktree list`, HEAD==B), tracked + untracked diff **intact**. PHASE-01's
F-T2 observed auto-remove only in the **native** path (no WorktreeCreate hook) and
wrongly generalised it into the teardown premise. See
`mem.fact.claude.worktree-remove-auto-teardown` (corrected).

**Symmetric live-import (decided — D-funnel-path, supersedes the Path-L capture).**
Both arms are now **lifecycle-symmetric**: the **orchestrator** owns the worktree
on both (pi: `worktree fork --worker` → import → remove; claude: harness-born tree
left on disk → import → remove). On the claude arm the Agent tool returns the
worker's `worktreePath` in its footer (**PROVEN** —
`mem_019efe28d60b7d51998f1f7912b8e7b8` P2, wtc-cwd probe on claude-code 2.1.181,
2026-06-25: a `WorktreeCreate`-hook-created worktree still returns `worktreePath`
as the normative datum plus `agentId`) and the tree is still on disk, so the
orchestrator:

1. reads `worktreePath` from the footer (`name = basename(worktreePath)`). **No
   correlator, no enumeration**: the footer is per-return, so each blocking Agent
   call — including every sibling of a parallel fan-out — self-identifies its own
   `worktreePath` + `agentId` (RV-205 F-1; the footer datum is proven, `mem P2`
   above; `docs/claude` is silent on it, doctrine's own probe is the proof);
2. `verify-worker --dir <worktreePath> --base B --branch dispatch/<name>` — the
   identity/base belt on the **live** tree (HEAD==B, isolated, marker,
   base-ancestor, dir↔branch coherence). **Unchanged Rust**; fail-closes
   (`no-worker-head`) if the tree is unexpectedly gone (INV-6);
3. `worktree import --from-worktree <worktreePath> --base B` — reads the live
   worker diff and applies it onto the coord tree (verb below);
4. **iff step 3 succeeded** (`classify_import` belt green **and** `git apply
   --index` exit 0), `git worktree remove --force <worktreePath>` — reaps the
   spent tree (symmetric with the pi arm's orchestrator-removes). `--force` is
   required: the tree is **intentionally dirty** (its uncommitted diff was just
   imported), which plain `git worktree remove` refuses. **On import failure the
   funnel HALTS and LEAVES the tree** (diagnostic, symmetric with `verify-worker`'s
   leave-in-place) — never `--force`-reaps the sole copy of an unimported delta
   (RV-205 F-3; a VT pins the import-fails path).

No `SubagentStop` hook, no capture file, no teardown race. The RV-202 correlator
seam is genuinely **void** — the footer supplies the path per-return, proven (not
assumed) by `mem P2`.

**The import verb — stable interface, swappable strategy (D-import-verb).**
`worktree import --from-worktree <worker_wt> --base <B>` (cwd = coord tree, the
apply target) is the load-bearing **contract**; the delta-extraction is a strategy
*behind* it. **Today's strategy = orchestrator working-tree diff:** gather
`git -C <worker_wt> diff HEAD` — belt-hardened (`core.quotePath=false`,
`--no-renames`) — plus untracked files via the index-free `git diff --no-index
/dev/null <f>` synthesis, into one unified patch; run the **unchanged** pure
`classify_import` (belt: `.doctrine/`/`.claude/` reject, `head_at_base` = coord
HEAD==B, `tree_clean`; `single_commit` **vacuously true** — a working-tree diff
carries no commits); `git apply --index` onto the coord tree, non-committing (the
funnel commits separately). **Future strategy** (foreseen, not built): cherry-pick
from a standalone clone (Path C / IDE-024), config-selected — rides the *same
verb*, swapping only the gather step. The gather logic + hardening flags are the
former `capture.rs::gather_worktree_patch`, **relocated** into `import.rs` (T2's
tested code survives, moved); the apply core is the former `run_import_patch`, fed
live instead of from a file. `import` router = **`--fork`** (pi/subprocess, frozen,
EX-4) **XOR** `--from-worktree` (claude); the file-based `--patch` surface is
removed.

**Delta-check home (OQ-1 resolved — Rust, `import.rs`).** The single-commit /
`B..S` check is the pure `classify_import` in `src/worktree/import.rs`, **not**
`src/dispatch.rs`. The `--from-worktree` arm reuses it unchanged; the **belt**
(`.doctrine/`/`.claude/` reject + `head_at_base` + `tree_clean`) is the
load-bearing check on this arm. The router funnel's "delta check `B..S` single
non-merge commit" / "import net-diff" steps are the pi-arm phrasing; the claude arm
reads `single_commit` vacuously and imports the live diff.

Retirement (touch-set): the whole `SubagentStop` capture apparatus — `capture.rs`,
the `worktree subagent-stop` command + guard classification, the `hooks.json`
`SubagentStop` entry + its install-templating test, and the file-based `import
--patch` surface — is **removed** (git history is the archive). `create-fork`
(WorktreeCreate) is unchanged; crucially **no** `WorktreeRemove` hook is ever
installed (INV-6).

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
- **Edge — per-worker scratch (E, brief §3b).** Scratch is private **by the arm's
  mechanism, not a portable `tmpfs` guarantee**: on the bwrap arm a private
  `--tmpfs /tmp` (ephemeral, never host /tmp, vanishes with the namespace); on the
  Seatbelt arm (SL-183) there is *no* tmpfs analog — privacy comes from
  `TMPDIR=<wt>/.tmp` + deny `/private/tmp`, and the scratch persists until teardown
  GC. In both, `/tmp` is denied for Edit/Write by restrictive default; loosen a run
  via `extra_rw`. Do **not** state "/tmp is private" as a cross-arm guarantee — it is
  false on macOS.
- **Edge — capability-keyed backend, not `else: deny` (C, brief §1/§6).** The shell's
  host probe resolves a **`Backend` descriptor** (data); `select_jailer` maps it. A
  platform with **no usable backend ⇒ `Backend::Deny{reason}` ⇒ `deny`** (fail-closed),
  never unwrapped pass-through. The reason rides the descriptor **per arm** —
  `"bwrap-unavailable"` on Linux, `"seatbelt-unavailable"` on macOS-today — not a
  flattened generic string. The `Deny{reason}` state is **three-valued, not a bare
  `None`**: it also carries *present-but-degraded* (SL-183 Seatbelt nesting refused,
  brief §3a), so SL-183 adds a variant arm, not a type change. **macOS is a NAMED arm
  that currently denies** (pending IMP-045/SL-183), not a hardcoded `else`. SL-183 is
  therefore a **capability flip** (a `Deny` reason → `Backend::Seatbelt` behind the same
  seam), not a control-flow rewrite. Aligns with RFC-012's capability ladder (`none` /
  `contained-writes` / …): a `Deny` backend is the `none` rung, the bwrap/Seatbelt arms
  the `contained-writes` rung.
- **Assumption (verify):** PreToolUse via `settings.local.json` fires for worktree
  subagents — proven (probe). The plugin `hooks.json` path is **not** assumed — it is
  V-plugin-gated with the settings.local path as a planned same-phase fallback
  (RV-200 F-5). The wire fields it relies on are doc-backed: `agent_id`
  (`hooks.md:595`), `updatedInput` (`hooks.md:818`), `permissionDecision`
  (`hooks.md:806`); only the plugin-*registration* firing is unproven.
- **INV-6 (A, PHASE-05) — no `WorktreeRemove` hook; enforced at two boundaries,
  limits named (RV-205 F-2).** The symmetric live-import funnel (§5.4) depends on the
  worker worktree **persisting** on disk post-return. This holds iff `create-fork` is
  the `WorktreeCreate` hook (replaces default git behavior ⇒ Claude does not
  auto-remove) **and no `WorktreeRemove` hook is installed** (`hooks.md:2390/2442` —
  "without one, the directory is left on disk"). Enforcement is **two-boundary**, and
  its blind spot is stated rather than papered over: **(i) install-time** — the
  installer MUST NOT ship a `WorktreeRemove` hook, pinned by an **assertion that no
  `WorktreeRemove` entry exists in the materialized `hooks.json`** (AF-3, a VT); **(ii)
  runtime** — the funnel runs `verify-worker --dir <worktreePath>` *before* import,
  which **fail-closes** (`no-worker-head`) if the tree is **gone**, halting loud rather
  than importing an absent delta. Boundary (ii) catches tree-**absence**, **not**
  tree-**mutation**: a hook that reset/cleaned the tree yet left it registered at
  HEAD==B would pass `verify-worker` while silently dropping the (esp. untracked)
  delta. That case is shut out by boundary (i) — no such hook ships — and by the
  ADR-006 sole-writer trust model; it is **not** claimed to be caught by
  `verify-worker`. The orchestrator reaps each tree explicitly (`git worktree remove`)
  after import. Leaked trees on an orchestrator crash mid-drive are an operability
  concern, not a correctness one (IMP-222: doctor warn + gc).
- **ASM (PHASE-05) — Fork-path teardown parity, grounded in hook-presence (RV-205
  F-4).** The persistence probe exercised `create-fork`'s **Passthrough** path (unarmed
  spawn); real dispatch uses the **Fork** path (armed at explicit B, `dispatch/<name>`).
  Persistence follows from **`WorktreeCreate`-hook presence** (`hooks.md:2390/2442` — a
  custom hook + no `WorktreeRemove` hook leaves the tree on disk), and `create-fork` is
  the **single `WorktreeCreate` binary for both classify outcomes** (Fork / Passthrough
  — they differ only in the checked-out branch, not in whether a `WorktreeRemove` hook
  exists). So the probe **did exercise the persistence mechanism**; Fork-path
  persistence transfers **by construction**, not by bare assumption. The one Fork-path
  delta still worth live confirmation is the **footer on the branch case** (`mem P2`
  probed a *detached* tree, `worktreeBranch: undefined`; `worktreePath` remained the
  normative datum) — VH-1 confirms both persistence and footer on the armed branch
  path, as **confirmation, not sole support**. **Fail-safe net remains:** were the
  Fork-path tree somehow torn down, the pre-import `verify-worker --dir` fail-closes
  (`no-worker-head`, INV-6) — the funnel halts loud, never a silent lossy import.

## 6. Open Questions & Unknowns

- **OQ-1 (RESOLVED — PHASE-05).** Funnel delta-check location = **Rust**, the pure
  `classify_import` in `src/worktree/import.rs` (NOT `src/dispatch.rs`, which carries
  only record-boundary/arming). The `--from-worktree` claude arm and the `--fork` pi
  arm both drive it; the belt is the load-bearing check on the claude arm (§5.4).
- **OQ-2 (RESOLVED — PHASE-05 live probe, 2026-07-01).** The original fear — the
  harness auto-removes the worktree on subagent finish, so the worker diff cannot be
  imported from a live tree — is **false for the shipped config**. With `create-fork`
  as the `WorktreeCreate` hook and **no** `WorktreeRemove` hook, the tree is **left on
  disk** post-return, diff intact (probe: `.worktrees/agent-<id>` persisted, HEAD==B,
  tracked+untracked present). The SubagentStop capture contingency (RV-201 F-2) **and**
  its correlator seam (RV-202) are therefore **retired** — the orchestrator reads
  `worktreePath` straight from the Agent footer (proven per-return datum,
  `mem_019efe28…` P2; RV-205 F-1) and imports the live tree directly (§5.4,
  D-funnel-path). The Path-C/IDE-024 escalation is no longer a lock-time risk; it
  survives only as the *foreseen future strategy* behind the `--from-worktree` verb
  (D-import-verb). Residual: Fork-path parity — persistence **and** the branch-case
  footer (§5.5 ASM), pinned at VH-1.
- **OQ-3 → V-plugin (first step in execute).** Plugin `hooks.json` is the chosen
  registration home (D-reg). Confirm PreToolUse-via-plugin fires for a worktree
  subagent before building on it; cross-check hook semantics against `docs/claude`
  (local official-docs cache — authoritative over web/subagent).

## 7. Decisions, Rationale & Alternatives

- **D1 — Rust subcommand** (`doctrine worktree pretooluse`), not bash. Rides the
  existing hook seam; reuses worktree resolution; testable; single bwrap-flag
  source. Startup ≈ 2 ms. (`resolve_exec` is **not** a runtime responsibility of
  this subcommand — at hook-exec the binary is already running, so `current_exe()`
  here would merely re-derive its own path; the only `resolve_exec` relevant to
  this slice is the **install-time** templating in D-reg/§5.4 — RV-201 F-5.) *Alt:
  bash scripts — rejected (anomaly, jq/bash dep, magic strings, untestable, DRY
  violation).*
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
  (The topology stands; **funnel convergence within Path L is now live-import**, not
  SubagentStop-capture — D-funnel-path, §5.4. Path C survives as the foreseen future
  gather strategy behind the `--from-worktree` verb — D-import-verb.)
- **D5 — single-sourced bwrap core flags** via `bwrap_core_argv` + a **parity
  test** against `pi-spawn-confined.sh`; leave the live pi script untouched. *Alt:
  extract `worktree jail-argv` consumed by both — true DRY but touches live pi
  dispatch → follow-up.*
- **D-reg — register via the plugin `hooks.json`** (`plugins/doctrine/hooks/`),
  made fail-closed by **install-time templating (Option A, RV-201 F-1 — User
  decided)**: `install_hooks_plugin_for_claude` rewrites every command's leading
  `doctrine` token to `resolve_exec()`'s absolute path at materialization, so the
  plugin path reaches parity with the settings path's `HookSpec` bake — NOT a bare
  PATH, and NOT the false "the runtime subcommand reuses resolve_exec" framing the
  reconcile previously carried (§5.4). Preferred on user steer — prior empirical
  tests show plugin hooks uniform with settings hooks; rides the existing
  auto-discovered seam. **Gated on V-plugin** (re-test PreToolUse-via-plugin fires
  for a worktree subagent *and* honours `updatedInput`; also confirms hook
  `command` is shell-run, gating the `|| exit 2` vanish-guard). *Alt-mechanism
  (rejected): an embedded exit-2 shim — bash anomaly D1 rejected, second asset,
  still templated. Alt-registration: `settings.local.json` via boot.rs `plan_hook`
  — the probe's proven path; a **planned contingency of the V-plugin phase** (built
  iff the plugin path fails the re-test), scoped and ready (RV-200 F-5).*
- **D6 — schema = `extra_rw` + `network`.** Dropped `extra_ro` (redundant under
  `--ro-bind / /`) and `strict/loose mode` (the floor *is* strict; loosening ==
  `extra_rw`). Footgun violations **deny** (fail-closed).
- **D7 — empirical harness probe BEFORE Rust (User steer).** Every unproven
  harness behaviour (plugin-PreToolUse firing + `updatedInput`; `SubagentStop`
  blocking + tree-intact timing; hook-`command` shell-run) is pinned by a
  disposable-shell probe (RSK-014 idiom) as the slice's **first phase**, ahead of
  any Rust. The `docs/claude` cache is a hypothesis, not proof — it documents none
  of the timing. Rust graduates a *proven* shape (§9 Phase 1). *Alt: trust the
  docs and build directly — rejected; the two tallest risks (R1 funnel-teardown,
  R2 plugin-registration) are harness behaviours doc-unconfirmed and cheapest to
  refute in shell.*
- **D8 — single `Jailer` seam, factored now for cross-arm parity (SL-183 upstream,
  brief §2/§7 D-mac2).** The platform-agnostic core (`resolve_target`, `decide_*`,
  `pathcheck`, `opaque_wrap`, `validate_policy`) sits above one named fork point; only
  the wrapper-argv/profile builder sits below it (`Bwrap` this slice; `Seatbelt` =
  SL-183, deferred). Three concrete shape commitments fall out, all **zero Linux
  behaviour change** — they only prevent SL-183 from having to refactor this core:
  **(i)** `opaque_wrap` is wrapper-agnostic — takes the jailer's argv as input (B);
  **(ii)** backend selection is **capability-as-data** — the shell resolves a `Backend`
  descriptor (`Bwrap | Seatbelt | Deny{reason}`) and `select_jailer(&Backend)` is a pure
  map, so host detection stays in the shell (pure/imperative split), the deny reason
  rides per-arm, and the three-valued `Deny{reason}` reserves the *present-but-degraded*
  state SL-183 needs (brief §3a) — macOS a named-but-denying arm, not a hardcoded `else`
  (C, §5.5); **(iii)** `validate_policy` carries **zero** bwrap/namespace
  assumptions and is the shared cross-arm contract, reused unchanged as SL-183's
  parity proof (D). The macOS decisions themselves (D-mac1 *allow-default-deny-write-
  except*, D-mac3 `TMPDIR` scratch, D-mac4 `network`→`(deny network*)`) live in the
  SL-183 brief, not here — this slice only guarantees the seam they hang off.
  *Alt: inline the bwrap argv into the decision logic — rejected; forces SL-183 to
  refactor a locked, behaviour-frozen core (behaviour-preservation gate).*
- **D-funnel-path — symmetric live-import (PHASE-05; SUPERSEDES the Path-L
  SubagentStop-capture funnel, RV-201 F-2 / RV-202).** Both dispatch arms are
  lifecycle-symmetric: the orchestrator owns the worker worktree and imports its
  **live** working-tree diff post-return, then reaps it. The claude arm's Agent
  footer hands `worktreePath` per-return (proven — `mem_019efe28…` P2, wtc-cwd probe
  2026-06-25; RV-205 F-1) and the tree persists on disk (INV-6), so no `SubagentStop`
  capture, no correlator, no teardown race. *Rationale:* the PHASE-05
  live probe (2026-07-01) disproved the teardown premise the capture design rested on
  (§5.4, OQ-2) — a custom `WorktreeCreate` hook + no `WorktreeRemove` hook leaves the
  tree on disk. *Alt (retired): SubagentStop awaited capture-before-remove — solved a
  non-problem; correlator (RV-202) and blocking-hook analysis (RV-201) were cost
  against a teardown that does not occur.*
- **D-import-verb — stable CLI verb, swappable gather strategy (PHASE-05).**
  `worktree import --from-worktree <dir> --base <B>` is the load-bearing interface;
  the delta-extraction sits behind it as a strategy the design expects to evolve into
  a config surface. Today: orchestrator working-tree diff (gather `diff HEAD` +
  index-free untracked → unchanged `classify_import` belt → `git apply --index`).
  Foreseen: cherry-pick from a standalone clone (Path C / IDE-024), config-selected —
  same verb, swap the gather. `import` router = `--fork` (pi, frozen) XOR
  `--from-worktree` (claude); file-based `--patch` removed. *Rationale:* the user
  flagged this mechanism will change and become configurable, so the **verb is the
  contract**, not the mechanism. `classify_import` pure core never moves
  (behaviour-preservation, EX-4). *Alt: keep the file-based `--patch` + shell gather —
  rejected; pushes the delicate untracked `--no-index` synthesis into untested skill
  prose and duplicates logic Rust already got right.*

## 8. Risks & Mitigations

- **R1 — funnel breakage.** Confinement removes claude self-commit → breaks the
  `B..S` delta-check. *Mit (AMENDED PHASE-05):* the claude arm imports the worker's
  **live** working-tree diff via `worktree import --from-worktree` (D-funnel-path,
  §5.4) — `single_commit` vacuous, belt intact. The companion fear (harness tears the
  tree down before import, RV-200 F-3) was **disproved** by the PHASE-05 probe (tree
  persists on disk, INV-6); the SubagentStop capture is retired. Pinned by VH-1 (live
  Fork-path funnel green).
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
- **R9 — the "no out-of-namespace executor" residual is PLATFORM-SPECIFIC, not
  closed (SL-183 upstream, brief §5).** On the NixOS/bwrap arm the
  delegation-to-a-reachable-executor vector is dead *because the closure ships no
  cron/at/systemd* — a property of this platform, not of the jail. **macOS always
  ships `launchd`**, which Seatbelt (not a namespace) does not remove: file-based
  delegation (LaunchAgent plist, crontab) is still write-floor-denied, but a
  pure-IPC `launchctl submit`/mach-service path is not. Frame this residual as
  platform-specific, **owned by RFC-012 / the future IPC-egress wall (a non-goal of
  this write floor)** — do not claim it "closed" cross-arm. SL-183's probe *measures*
  `launchctl submit`/`at` rather than assuming. (Sibling of the OQ-6
  socket-reachable-peer residual: postgres `COPY…TO PROGRAM`, nix-daemon.)

## 9. Quality Engineering & Validation

**Phase 1 — empirical harness probe (DISPOSABLE SHELL, PRE-RUST GATE; D7, User
steer).** Before *any* Rust is written, a throwaway probe — in the RSK-014
probe-h1 idiom (live `settings.local.json` hooks + shell scripts, `rm`-able) —
**empirically pins every unproven harness behaviour the design leans on.** The
`docs/claude` cache is treated as a hypothesis, not proof (it documents none of
the timing below). The probe must confirm, on the live harness:
1. **Plugin-PreToolUse fires** for an `isolation:worktree` subagent (Bash +
   Edit|Write) **and honours `updatedInput`** — the D-reg registration path
   (was "V-plugin"). Fail ⇒ settings.local fallback (planned same-phase, F-5).
2. **`SubagentStop` is genuinely blocking/awaited** (exit-2 holds the stop;
   `hooks.md:658` is doc-only, untested), **observes the worktree still on disk**
   before `git worktree remove` runs (OQ-2 / F-2), **AND its payload correlates to
   the right worktree** — it carries no `worktree_path` (RV-202), so the probe must
   prove a correlator resolves (`agent_id`→`agent-<id>`→provision-set, or
   transcript/`cwd` fallback; §5.4). Any of the three fails ⇒ abort to Path C /
   IDE-024 — the funnel's load-bearing timing *and* targeting.
   **[AMENDED PHASE-05 — this probe item is superseded.]** PHASE-01 confirmed all
   three, but the PHASE-05 persistence probe then showed the worktree is **left on
   disk** post-return (no auto-teardown under `create-fork` + no WorktreeRemove hook,
   INV-6), so the funnel imports the **live** tree and the `SubagentStop` capture +
   correlator are retired. The surviving validation is the "tree-persists +
   live-import" gate below.
3. **Hook `command` is shell-run** — gates the F-1 `|| exit 2` vanish-guard. Lower
   risk than (1)/(2): `docs/claude/hooks.md:337` shows shell-form when `args` is
   omitted (RV-202). The probe confirms it on the live harness; if commands are
   exec'd directly (not via a shell), the guard is dropped and the
   reinstall-on-upgrade invariant stands alone.

Only once all three are **pinned green** does the design's mechanism (Rust
`pretooluse.rs` + install templating + the `SubagentStop` capture) get built —
the apparatus graduates a *proven* shape, never an assumed one. The Rust gates
below (Unit/Integration) then re-express the probe's findings as durable tests.
*(Phasing: /plan sequences this as the first phase; the two former "execute gates"
fold into it. Rationale: harness behaviour is the slice's tallest risk and the
cheapest to refute in shell — do it before sinking Rust into a refuted premise.)*

- **Unit (pure, TDD red/green/refactor):** `resolve_target` (3 arms, topology-based
  recognition incl. a sibling-repo worktree → not-jailed-here, A1);
  `pathcheck` (⊆wt / escape / extra_rw-hit / `.git`-reject); `load_policy`
  (default / present / malformed); `bwrap_argv` (core + extra_rw + `network`);
  `opaque_wrap` (base64 round-trip **+ INV-5 path with space & single-quote**
  round-trips & executes; **wrapper-agnostic — asserts it assembles an arbitrary
  given argv, not a bwrap-shaped one, B**); `validate_policy` (reject `/`,
  root-ancestor, `.git` — **+ a no-namespace-assumption assertion locking it as the
  shared cross-arm contract, D**); **`select_jailer` capability dispatch (D8/C) — a
  PURE map over an injected `Backend` descriptor, so it runs on the Linux CI host with
  no macOS dependency: `Bwrap ⇒ Some(Bwrap)`; `Seatbelt ⇒ Some` (SL-183 stub);
  `Deny{reason} ⇒ None`; `decide_bash` on a `Deny{reason}` emits `Decision::Deny` with
  that reason (per-arm, e.g. `"bwrap-unavailable"`), never passes through**.
- **Integration (synthetic stdin → emitted JSON):** the probe escape battery
  re-expressed as cases; INV-2 repo-root-ancestor deny; orchestrator pass-through
  (no `agent_id`); isolation:none deny (`agent_id` + repo-root cwd); D5 parity
  test; **keying/provision (A7): `create-fork` provisions the declared policy to
  `jail/<name>.toml`; PreToolUse resolves it by `cwd → basename`**; **per-arming
  granularity (F-1): serial arming binds the sole intent per-worker; a parallel batch
  off one arming provisions the SAME profile to every sibling (shared, not leaked);
  absence ⇒ Default floor**; **fail-closed exec (F-1): the templated absolute exec
  plus the shell-form `|| exit 2` vanish-guard denies on a missing/non-resolving
  `doctrine` (shim rejected — RV-202), never passes through unconfined**.
- **V-plugin (pinned in Phase 1 probe — gate on D-reg):** confirm a PreToolUse hook
  registered via the plugin `hooks.json` fires for a worktree subagent (Bash +
  Edit/Write) **and honours `updatedInput`**, exactly as the probe proved via
  `settings.local.json`. Cross-check hook-event/matcher/`updatedInput` semantics
  against `docs/claude`. Fail ⇒ land the settings.local.json fallback **in this same
  phase** (F-5).
- **Tree-persists + live-import (AMENDED PHASE-05 — supersedes the capture-before-remove
  gate; OQ-2 resolved).** Confirm the worker `isolation:worktree` tree **persists on
  disk post-return** under production hooks (`create-fork` WorktreeCreate, no
  WorktreeRemove — INV-6), and the orchestrator imports its **live** working-tree diff
  via `worktree import --from-worktree` after `verify-worker --dir`. The Passthrough
  path is probe-proven (2026-07-01); the **Fork** path is pinned at VH-1 (§5.5 ASM).
  Fail (tree gone despite INV-6) ⇒ `verify-worker` fail-closes (`no-worker-head`), funnel
  halts loud — never a lossy import.
- **End-to-end (VA/VH — the riskiest leg):** live claude `/dispatch`, one jailed
  worker on the **Fork** path, escape vectors denied + canaries intact + funnel
  completes green (live `--from-worktree` import + orchestrator reap). Covers OQ-2
  end-to-end.
- **Behaviour-preservation:** existing worktree/dispatch suites stay green.

## 10. Review Notes

### PHASE-05 amendment — teardown premise disproved, funnel pivoted to live-import (2026-07-01)

Mid-PHASE-05 execution, a `/consult` on the T6 SKILL.md funnel surfaced a tension
(verify-worker targeting a torn-down tree), which on investigation exposed a **false
premise** in §5.4: the design assumed the harness auto-runs `git worktree remove`
on subagent finish. That was observed only in PHASE-01's **native** probe (no
`WorktreeCreate` hook); production ships `create-fork` **as** the WorktreeCreate
hook. A targeted live probe (one `isolation:worktree` subagent, production hooks,
no WorktreeRemove) showed the worktree **persists on disk** post-return, diff intact
— matching `docs/claude/hooks.md:2390/2442`.

Consequence — reversed **D-funnel-path** (Path-L SubagentStop-capture → symmetric
live-import), added **D-import-verb** (`import --from-worktree`, stable verb /
swappable gather), **INV-6** (enforced no-WorktreeRemove-hook), and the Fork-path
parity **ASM**. OQ-1 and OQ-2 resolved. The retired §5.4 capture design (correlator
RV-202, blocking-hook RV-201 F-2) is superseded — git history + this entry are its
provenance. Retirement of the capture apparatus (capture.rs, `subagent-stop` command,
`hooks.json` SubagentStop entry, file-based `import --patch`) and the refactor to
`--from-worktree` are the amended PHASE-05 touch-set. Corrected memory:
`mem.fact.claude.worktree-remove-auto-teardown`. Case note: RFC-011.

### `/inquisition` findings (RV-205, 2026-07-01) — hostile pass on the PHASE-05 amendment

Fourth adversarial round, arraigning the symmetric-live-import amendment itself
(commits `49dd6144` + `36e5b669`). Tried on the ledger (`doctrine review show
RV-205`); 4 findings (1 raised blocker, refuted on evidence). All reconciled here.

- **F-1 (raised blocker → resolved minor) — the footer datum was uncited, not
  unproven.** The amendment declared the RV-202 correlator "void" because "the footer
  hands the path," a field `docs/claude` documents nowhere. The inquisition forced the
  proof to surface: `mem_019efe28d60b7d51998f1f7912b8e7b8` P2 (wtc-cwd probe,
  claude-code 2.1.181, **2026-06-25**, SL-152) already proved a `WorktreeCreate`-hook-
  created worktree returns `worktreePath` (normative) **plus `agentId`** in the Agent
  footer. So RV-202 "void" is *justified*, and the parallel-fan-out worry dissolves
  (each blocking return self-identifies). The real defect was a **citation gap** — the
  proof was uncited, so the claim read as faith. → §5.4 step 1 / D-funnel-path / OQ-2 /
  the corrected memory now cite `mem P2`; the Fork-path **branch-case footer** joins
  persistence as the VH-1 residual (P2 probed a detached tree).
- **F-2 (major) — INV-6 enforcement boundary overclaimed.** "Enforced by
  `verify-worker` fail-close" covers tree-*absence*, not tree-*mutation*. → INV-6
  rewritten as **two-boundary** (install-time AF-3 assert no `WorktreeRemove` entry;
  runtime `verify-worker` absence-catch) with the mutation blind spot named and shut
  out by boundary (i) + sole-writer trust, not falsely claimed caught.
- **F-3 (minor) — ungated `--force` reap.** `git worktree remove --force` could
  immolate the sole copy of an unimported delta. → §5.4 step 4 gated on import success;
  halt-and-leave on failure; VT on the import-fails path.
- **F-4 (minor, acquitted upward) — Fork-path ASM was timid.** Persistence follows
  from `WorktreeCreate`-hook presence, and `create-fork` is one binary across both
  classify outcomes → the probe *did* exercise the mechanism. → ASM reworded to ground
  persistence in hook-presence (transfers by construction); VH-1 demoted to
  confirmation.

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

### `/inquisition` findings (RV-201, 2026-07-01) — codex GPT-5.5 + inquisitor — reconcile-introduced heresy

Second adversarial round on the post-RV-200 re-lock; 5 findings (1 option-bearing
blocker), tried against the source seams not the prose. RV-200's 10 findings left
settled. All reconciled in this revision.

- **F-1 (blocker, option-bearing) — the PREFERRED registration shipped FAIL-OPEN.**
  D-reg's "resolved absolute doctrine (NOT bare PATH)" was *false as-built*:
  `install_hooks_plugin_for_claude` (`src/skills.rs:1046-1049`) verbatim byte-copies
  the embedded `hooks.json` whose commands are **bare `doctrine`**
  (`plugins/doctrine/hooks/hooks.json:7,18`); `resolve_exec` was never on that path.
  Fail-closed held only on the settings.local fallback. **User decided Option A:**
  template every plugin-`hooks.json` command's leading `doctrine` token through
  `resolve_exec` at materialization (parity with the settings `HookSpec` bake) —
  *rejected* the embedded-shim alternative (bash anomaly D1 rejected). → §5.4 / D-reg
  rewritten; pre-baked-install interaction + V-plugin-gated `|| exit 2` vanish-guard +
  reinstall-on-upgrade invariant documented; false "resolve_exec already provides this"
  framing struck.
- **F-2 (major) — capture led with the wrong hook.** §5.4 led with `WorktreeRemove`
  (no decision control, not awaited, debug-log-only — `hooks.md:680/814/2442`) over
  `SubagentStop` (blocking-capable, awaited, carries `agent_id`+`agent_transcript_path`
  — `hooks.md:658/1930-1957`). → §5.4 / OQ-2 / §9 **commit to `SubagentStop`**;
  `WorktreeRemove` demoted to cleanup; stop-vs-`git worktree remove` ordering stated
  **unproven** (`subagents-reference.md` documents no timing) and pinned to the probe.
- **F-3 (major) — scope split-brain; "scope doc corrected" was a false attestation.**
  `slice-182.md` objective 3 still preached `agent_id` keying, "per-worker", `extra_ro`,
  strict/loose — all repudiated by locked D2/D6/F-1. → objective 3 rewritten to
  worktree-name key / per-arming / `extra_rw`+`network`; OQ-A's vestigial `resolve_exec`
  struck (scope twin of F-5). The attestation is now true.
- **F-4 (major) — shared-profile safety rested on unspecified machinery.** The
  declaration file was unnamed, unpaired with `base`, and the create-fork provision
  step net-new/unbuilt. → §5.3 names both files + lifecycle table; grounds the
  `jail.toml`↔`base` pairing and "no second arming" in the **blocking `Agent` call**
  (single-threaded orchestrator, batch blocks until all N return), not discipline;
  marks the create-fork provision NET-NEW (patterned on `marker.rs:write_marker`).
- **F-5 (minor) — vestigial `resolve_exec` in the runtime layer.** §5.1 + D1 still
  listed it as a `pretooluse.rs` responsibility; the fix is install-time. → struck from
  both (twin of F-1).

Plus User steer integrated: **D7 — empirical harness probe (disposable shell) BEFORE
Rust** pins every doc-unconfirmed harness behaviour (plugin firing, `SubagentStop`
timing, shell-run) as the first phase; docs are hypothesis, not proof (§9 Phase 1).

### codex pass (RV-202, 2026-07-01) — reconciled directly, no ledger

Third adversarial pass (codex GPT-5.5, read-only, source-verified) on the
post-RV-201 surfaces. 3 majors + 2 minors, no option-bearing blocker — all
mechanical or invariant-shaped, so **reconciled directly** rather than via a fourth
ledger cycle. Rationale: RV-200→201→202 each healed the cited surface and left an
unswept twin; breaking that prose-polishing loop, the **D7 probe** is the real
verification of the load-bearing harness unknowns, not another markdown read. This
pass re-swept for twins explicitly.

- **M1 (major) — §2 still carried the F-1 lie.** Current-state said the embedded
  plugin `hooks.json` is "exec-path-injected (`resolve_exec`)" — contradicting the
  F-1 reconcile (§5.4) and source (`write_atomic(&hooks.data)`, raw byte-copy,
  `src/skills.rs:1046`; asset bare `doctrine`, `hooks.json:8`). The RV-201 fix swept
  §5.4/D-reg but not §2. → §2 rewritten: settings path bakes `resolve_exec`, plugin
  path byte-copies bare (the F-1 anomaly). The unswept twin of F-1.
- **M2 (major) — `SubagentStop` worktree correlation overclaimed.** §5.4 called
  `agent_id`+`agent_transcript_path` "exactly the worktree-correlation a capture
  needs", then ran `git -C <worktree> diff`. But `SubagentStop` carries **no
  `worktree_path`** (`hooks.md:1930-1957`); `worktree_path` ships only on the
  unusable non-blocking `WorktreeRemove` (`hooks.md:2465`). F-2's trade bought
  blocking and lost the free correlator. → §5.4 adds the correlator-candidate
  analysis (`agent_id`→`agent-<id>`→provision-set; transcript/`cwd` fallback);
  OQ-2 + §9 probe now assert **correlation**, not just tree-intact; no correlator ⇒
  abort to Path C / IDE-024. The substantive finding of this pass.
- **M3 (major) — F-4 blocking premise overbroad.** "Every claude `Agent` spawn
  BLOCKS" ignores `run_in_background: true` (returns immediately, `hooks.md:1428`),
  which would hand the orchestrator a re-arm turn mid-sequence. Today's template is
  foreground, so it holds by construction. → **INV-6** added (§5.3): no background
  `Agent` spawn against a live arming slot; the structural atomicity is conditioned
  on it, so a future background optimisation can't silently void the pairing.
- **m1 (minor) — §9 "exit 2 / shim".** Shim was rejected (D-reg). → narrowed to
  templated absolute exec + shell-form `|| exit 2`.
- **m2 (minor) — slice-182.md summary residue.** "per-run … keyed on the worker
  binding" survived the objective-3 rewrite. → corrected to per-arming / worktree-name.
- **Acquittals:** F-1 leading-token replace coherent for the actual asset (both
  commands start bare `doctrine`, args trailing). Nix-GC window correctly framed.
  Shell-form is actually doc-proven (`hooks.md:337`) ⇒ D7 item 3 is *lower* risk
  than the prose implied (noted in §9). slice-182.md objective 3 confirmed matching
  locked D2/D6/F-1.

### SL-183 upstream — cross-arm seam contracts (2026-07-01, no behaviour change)

SL-183 (macOS Seatbelt arm, discharges IMP-045, `needs SL-182`) reuses `jail.rs`
wholesale and forks only the argv/profile builder. Five seam-shape requirements
upstreamed **before lock** so SL-183 slots in rather than retrofits — all are
contract/altitude shape, **zero Linux behaviour change** (brief:
`.doctrine/slice/183/seatbelt-seam-brief.md`).

- **A (load-bearing) — explicit `Jailer` seam.** The platform-agnostic core sits
  above one named fork point; only `bwrap_argv`/`bwrap_core_argv` below it. → §5.1
  diagram + §5.2 `trait Jailer` + D8. *Was: argv builder listed inline in the pure
  core — would have forced SL-183 to refactor.*
- **B (load-bearing) — `opaque_wrap` wrapper-agnostic.** Already took `argv` as a
  param (§5.2); now locked as a contract (§4 Opaque-wrap bullet, §5.2 comment, §9
  unit asserts arbitrary argv).
- **C (load-bearing) — capability-as-data dispatch, not `else: deny`.** The shell
  resolves a `Backend` descriptor (`Bwrap | Seatbelt | Deny{reason}`); `select_jailer`
  (§5.2) is a pure map; a `Deny{reason}` denies with a per-arm reason. macOS a
  named-but-denying arm; SL-183 = a capability flip, not a control-flow rewrite;
  aligned to RFC-012's ladder. *Was (pre-RV-202): a zero-arg `select_jailer() ->
  Option<Box<dyn Jailer>>` host lookup in the leaf — see the RV-202 correction below.
  Was (pre-upstream): "non-bwrap platform → deny bwrap-unavailable" hardcoded else.*
- **D (contract-framing) — `validate_policy` strictly platform-agnostic.** Locked as
  the shared cross-arm parity proof, zero bwrap/namespace assumptions (§5.2 + §9
  no-namespace-assumption assertion + D8.iii).
- **E (contract-framing) — scratch privacy scoped to the mechanism.** "/tmp is
  private" was a false cross-arm guarantee (Seatbelt has no tmpfs). → §5.5 reworded
  to per-worker-scratch-private-by-mechanism (tmpfs on Linux, `TMPDIR` redirect on
  macOS).
- **(note, non-blocking) — reachable-peer residual is platform-specific.** → R9
  (§8): the bwrap "no cron/systemd in the closure" property is NixOS-specific; macOS
  ships `launchd`; framed as owned by RFC-012/the IPC wall, not "closed" cross-arm.

The macOS-side decisions themselves (D-mac1..4, OQ-mac1/2) stay in the SL-183 brief;
this slice guarantees only the seam they hang off.

### RV-202 correction — capability is data, not a host lookup (2026-07-01, codex GPT-5.5)

Codex's pass on the upstream caught one real regression (F-1, major) + a wording flat
(F-2, minor). The upstream had introduced `fn select_jailer() -> Option<Box<dyn
Jailer>>` — a **zero-arg host lookup inside the PURE leaf**, which (a) regressed the
project's pure/imperative split (host detection in `jail.rs`, contra AGENTS.md / the
already-correct `bwrap_present: bool` it replaced), (b) collapsed *absent /
unsupported / present-but-degraded* into one opaque `None`, leaving no room for the
"Seatbelt present but nesting refused ⇒ deny" state the brief §3a requires — forcing
SL-183 to widen the type and refactor the very seam this upstream exists to freeze,
and (c) made EX-5/VT-8 ("macOS ⇒ deny") un-exercisable on a Linux CI host.

Fix (all three at once): capability is **data resolved by the shell** — `enum Backend
{ Bwrap, Seatbelt, Deny{reason} }` — passed into the core; `select_jailer(&Backend)`
is a pure map and `decide_bash(.., &Backend)` denies with the descriptor's per-arm
reason. Pure core stays pure; the three-valued `Deny{reason}` carries the degrade
state (SL-183 adds a variant arm, not a type change); VT-8 becomes a pure table test
over injected descriptors; and the Linux missing-bwrap reason stays `"bwrap-unavailable"`
(F-2 — no flattening). Landed in §5.1 diagram, §5.2 (`Backend` + signatures), §5.5
edge, D8.ii, §9 VT, plan EX-5/VT-8. Still **zero Linux happy-path behaviour change** —
the corrected claim wording (F-2): the seam shape moved, the bwrap arm's deny reason
and decisions did not.
