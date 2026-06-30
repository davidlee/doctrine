# Design SL-182: Claude-arm subagent write-confinement hooks

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

Status: draft (pre-adversarial). Governed by ADR-008 (closes its claude-arm
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

- **Fail closed.** Any ambiguity (unresolved worktree, missing bwrap, parse error,
  policy-invalid) → deny. The strictest jail is the safe default; absence of
  policy can only *tighten*.
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
{ agent_id?, cwd, tool_name, tool_input: { command?, description?,
                                           file_path?, notebook_path? } }
```

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

`is_worktree(cwd)` = `shared::is_linked_worktree(cwd)` AND `cwd` under
`worktrees_root` (`<main>/.worktrees/`). Replaces the probe's hard-coded glob.

**Emitted JSON (mirrors probe):**
- deny → `{hookSpecificOutput:{hookEventName,permissionDecision:"deny",permissionDecisionReason:"worktree-jail: <reason>"}}`
- wrap → `…permissionDecision:"allow",updatedInput:{command,description}}`
- pass-through → no stdout.

### 5.3 Data, State & Ownership

**Per-worker jail policy** — `<main>/.doctrine/state/dispatch/jail/<agent_id>.toml`
(runtime state: gitignored, `rm -rf`-able). **Outside every worktree** (a worker
rw's its own tree — must not author its own policy) and **ro to the worker** (under
`<main>/.doctrine/state/`, ancestor of the worktree, ro-bound).

```toml
extra_rw = []      # absolute paths granted rw inside the jail; default []
network  = true    # false => --unshare-net; default preserves current behavior
```

```rust
#[derive(Deserialize)]
struct JailPolicy { #[serde(default)] extra_rw: Vec<PathBuf>,
                    #[serde(default = "default_true")] network: bool }
impl Default { extra_rw: vec![], network: true }
fn load_policy(main_root: &Path, agent_id: &str) -> JailPolicy; // missing file => Default
```

**Ownership:** orchestrator (ADR-006 sole-writer) authors the policy file at spawn,
*before* the worker's first tool call — written by the claude arm's
`dispatch arm-spawn` (`src/dispatch.rs`). **GC** with worktree teardown. Per-worker
file ⇒ no parallel-write contention; no worker TOCTOU (authored before spawn, ro
in jail, read by the hook process not the worker's command).

### 5.4 Lifecycle, Operations & Dynamics

**Registration** (decided: D-reg) — via `settings.local.json` through the boot.rs
`plan_hook` seam (the path the probe proved), exec-path-injected, idempotently
merged (`is_ours` ownership predicate, never clobbers foreign hooks). Two
PreToolUse entries: matcher `Bash` and matcher `Edit|Write|NotebookEdit`, both →
`<exec> worktree pretooluse`. **Hook registration loads at session start only — no
hot-reload**; install path documents the restart ritual + the Edit/Write escape
hatch (a broken Bash wrapper is disablable via Edit + restart).

**Per-call flow:** harness → PreToolUse → `doctrine worktree pretooluse` (stdin) →
`resolve_target` → bash/write decision → emit. Binary startup ≈ 2 ms (measured),
negligible per call.

**Funnel convergence (objective 5).** With ro-`.git`, the jailed worker cannot
self-commit (its object store is the ro main `.git`). The claude `/dispatch` funnel
converges onto the pi arm's **working-tree-diff** import (the cadence
`dispatch/SKILL.md` already *claims* is identical on both arms). Edits to
`dispatch-agent/SKILL.md`: import source = worktree diff against B (not `B..S`
commit); relax the single-commit delta-check on this arm; `verify-worker` adjusts.
/plan confirms whether the delta-check is skill-orchestration or Rust
(`src/dispatch.rs`) and scopes the touch.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1** pass through ⟺ `agent_id` absent. `agent_id` present + non-worktree
  cwd ⇒ deny (the `isolation:none` arm — proven denied, RSK-014 Exp 3).
- **INV-2** repo-root write denied by the **ancestor rule** (`realpath ⊄ wt`), not
  by native's race-win — pin in a synthetic-stdin test (recipe memory).
- **INV-3** `.git` is ro and **not tunable**: `validate_policy` rejects any
  `extra_rw` entry equal to `/`, an ancestor of `main_root`, or touching `.git`.
- **INV-4** Edit/Write allowlist = `{wt} ∪ extra_rw` — coherent with what the Bash
  jail rw-binds.
- **Edge:** `/tmp` is a private `--tmpfs` for Bash (ephemeral, never host /tmp) and
  denied for Edit/Write — restrictive default; loosen a run via `extra_rw`.
- **Edge:** non-bwrap platform → `deny "bwrap-unavailable"` (fail-closed; macOS =
  IMP-045), never unwrapped pass-through.
- **Assumption (verify):** PreToolUse via `settings.local.json` fires for worktree
  subagents — proven (probe). The plugin `hooks.json` path is **not** assumed.

## 6. Open Questions & Unknowns

- **OQ-1 (→/plan)** funnel delta-check location — skill-orchestration vs Rust
  (`src/dispatch.rs`). Scopes the objective-5 touch.
- **OQ-2 (verify in execute)** does the claude harness, when the worker leaves
  uncommitted worktree changes (commit blocked by ro-`.git`), still surface the
  worktree diff to the orchestrator import? End-to-end verification target (§9).
- **OQ-3 (deferred)** plugin `hooks.json` as the tidier registration home — verify
  it fires for PreToolUse before migrating (D-reg keeps settings.local.json now).

## 7. Decisions, Rationale & Alternatives

- **D1 — Rust subcommand** (`doctrine worktree pretooluse`), not bash. Rides the
  existing hook seam; reuses worktree resolution + `resolve_exec`; testable;
  single bwrap-flag source. Startup ≈ 2 ms. *Alt: bash scripts — rejected
  (anomaly, jq/bash dep, magic strings, untestable, DRY violation).*
- **D2 — per-worker policy file keyed by `agent_id`**, under
  `<main>/.doctrine/state/dispatch/jail/`. *Alt: single run-level map (write
  contention, heavier); two-tier (premature).*
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
- **D-reg — register via `settings.local.json`** (boot.rs `plan_hook`), the proven
  path. *Alt: plugin `hooks.json` — tidier, unproven for PreToolUse → OQ-3.*
- **D6 — schema = `extra_rw` + `network`.** Dropped `extra_ro` (redundant under
  `--ro-bind / /`) and `strict/loose mode` (the floor *is* strict; loosening ==
  `extra_rw`). Footgun violations **deny** (fail-closed).

## 8. Risks & Mitigations

- **R1 — funnel breakage.** Confinement removes claude self-commit → breaks the
  `B..S` delta-check. *Mit:* objective-5 convergence to working-tree-diff import is
  in scope; end-to-end verification gate (§9) before close.
- **R2 — registration path unproven for plugin.** *Mit:* D-reg uses the proven
  settings.local.json path; plugin migration gated behind OQ-3 verification.
- **R3 — bwrap-flag drift** between the Rust builder and the pi script. *Mit:* D5
  parity test fails on divergence.
- **R4 — policy TOCTOU / forged-absent.** *Mit:* absence ⇒ strictest jail (can only
  tighten); file authored before spawn, ro in jail.
- **R5 — harness change reopens repo-root** (native checkout-guard dropped). *Mit:*
  INV-2 ancestor-rule deny holds independently (pinned test).
- **R6 — non-Linux silent hole.** *Mit:* INV — fail-closed deny when bwrap absent.

## 9. Quality Engineering & Validation

- **Unit (pure, TDD red/green/refactor):** `resolve_target` (3 arms);
  `pathcheck` (⊆wt / escape / extra_rw-hit / `.git`-reject); `load_policy`
  (default / present / malformed); `bwrap_argv` (core + extra_rw + `network`);
  `opaque_wrap` (base64 round-trip, quoting-safe); `validate_policy` (reject `/`,
  root-ancestor, `.git`).
- **Integration (synthetic stdin → emitted JSON):** the probe escape battery
  re-expressed as cases; INV-2 repo-root-ancestor deny; orchestrator pass-through
  (no `agent_id`); isolation:none deny (`agent_id` + repo-root cwd); D5 parity
  test.
- **End-to-end (VA/VH — the riskiest leg):** live claude `/dispatch`, one jailed
  worker, escape vectors denied + canaries intact + funnel completes green
  (working-tree-diff import). Covers OQ-2.
- **Behaviour-preservation:** existing worktree/dispatch suites stay green.

## 10. Review Notes

(internal adversarial pass + any `/inquisition` findings recorded here)
