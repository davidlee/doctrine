# Research Document — SL-182 Design Review Support

<!-- Prepared for adversarial review of design.md (LOCKED, post-RV-200 revision).
     Maps design claims to source-code ground truth with file:fn references. -->

---

## 1. Current Repo State: What Exists vs What the Design Assumes

### 1.1 `src/worktree/` — modules that exist

| Module | File | Status | Notes |
|--------|------|--------|-------|
| `shared` | `src/worktree/shared.rs` | EXISTS | `is_linked_worktree()`, `resolve_common_dir()`, `matches()`, `resolve_commit()`, etc. 3662 bytes. |
| `create` | `src/worktree/create.rs` | EXISTS | `classify_create()`, `sanitise_name()`, `run_create_fork()`, `act_on_create()`. Full WorktreeCreate hook. |
| `fork` | `src/worktree/fork.rs` | EXISTS | `fork_core()` (add→provision→mark), `rollback_fork()`, `remove_worktree_dir()`, `run_fork()`. |
| `provision` | `src/worktree/provision.rs` | EXISTS | `run_provision()` (sole copier, reads `.worktreeinclude`). |
| `jail` | `src/worktree/jail.rs` | **DOES NOT EXIST** | Net-new — design's pure core (Decision, JailPolicy, bwrap builder, opaque_wrap, pathcheck, validate_policy). |
| `pretooluse` | `src/worktree/pretooluse.rs` | **DOES NOT EXIST** | Net-new — thin shell (stdin JSON in, hookSpecificOutput out, bwrap probe, resolve_exec). |

### 1.2 `mod.rs` — dispatch routing

`src/worktree/mod.rs` currently has NO `Pretooluse` variant in `WorktreeCommand`. It has `CreateFork`, `Fork`, `Provision`, `StampSubagent`, `VerifyWorker`, `Gc`, `Import`, `Land`, `Coordinate`, `CheckAllowlist`, and `Status`. Net-new: `Pretooluse` variant (design §5.1: "mod.rs gains `WorktreeCommand::Pretooluse`").

### 1.3 Current `hooks.json` (verbatim from `plugins/doctrine/hooks/hooks.json`)

```json
{
  "hooks": {
    "SessionStart": [{"hooks": [{"type": "command", "command": "doctrine boot --emit"}], "matcher": "*"}],
    "WorktreeCreate": [{"matcher": "*", "hooks": [{"type": "command", "command": "doctrine worktree create-fork"}]}]
  }
}
```

- **No PreToolUse entries** — net-new for this slice.
- **No WorktreeRemove entries** — net-new for the capture hook (OQ-2 / F-3).
- **Bare `doctrine` on PATH** — NO absolute path, NO `resolve_exec` substitution. This is the exact F-2 hole.

### 1.4 `install_hooks_plugin_for_claude()` — `src/skills.rs:1024-1049`

Current verbatim-copy behaviour:
```rust
let hooks = PluginAssets::get("doctrine/hooks/hooks.json")...;
crate::fsutil::write_atomic(&hooks_dir.join("hooks.json"), &hooks.data)?;
```

- **Does NOT call `resolve_exec()`** — no exec-path substitution happens.
- **Does NOT template the JSON** — `PluginAssets` data is raw bytes; the only textual content comes from `plugins/doctrine/hooks/hooks.json` as authored.
- **Result:** every `"command"` entry is a bare executable name on PATH.
- **Touch surface for F-2 fix:** Either (a) add `resolve_exec()` + JSON templating here, OR (b) embed a shim at `plugins/doctrine/hooks/shim.sh` that does `exec <resolved_path> "$@"` or `exit 2`, and reference that. Neither capability exists today.

### 1.5 `resolve_exec()` — `src/boot.rs:453-460`

```rust
pub(crate) fn resolve_exec() -> anyhow::Result<PathBuf> {
    let raw = std::env::current_exe()...;
    pick_exec(raw, std::path::Path::exists)
}
```

Used by: `boot.rs` (regenerate, status line, boot snapshot), `install.rs:163`, `corpus.rs:490`, `status.rs:405`. **NOT used by** `skills.rs:1024-1049` — the install path that writes hooks.json. This is the core of the F-2 gap: `resolve_exec` is *available* as a facility (it's a public `crate` function), but the plugin-install path doesn't call it.

### 1.6 `plugins/doctrine/hooks/hooks.json` — the embedded asset

Path: `plugins/doctrine/hooks/hooks.json`
Embedded via `rust-embed` (`PluginAssets`), referenced as `"doctrine/hooks/hooks.json"` in `skills.rs:1046`.

**Only source of truth** for the hooks JSON that gets materialized. If the design wants "carries an absolute doctrine path (or a tiny checked-in shim that exit-2s)" (§5.4), the **embedded asset text itself** must change (or the copy must template). Currently the asset is plain JSON with bare PATH commands.

### 1.7 `src/worktree/create.rs` — `fork_core()` provision step

```rust
// fork.rs:101-180 — fork_core(repo, base, branch, dir, worker)
// Step 1: git worktree add -b <branch> <dir> <base>
// Step 2: run_provision(Some(repo.to_path_buf()), dir)
// Step 3: if worker { write_marker(dir) }
```

**There is NO jail-policy provision step.** The design (§5.3) asserts: "the `worktree create-fork` hook — which runs at spawn and *does* know the new worktree — **provisions** that declaration into `<main>/.doctrine/state/dispatch/jail/<name>.toml`." But `fork_core()` only calls `run_provision()` (copies `.worktreeinclude`-listed files) and `write_marker()` (stamps the worker flag). Writing `<main>/.doctrine/state/dispatch/jail/<name>.toml` from the orchestrator's pre-spawn declaration is **net-new, unspecified work**.

The design identifies that `src/worktree/create.rs` is "in the touch-set (provision step)" (§5.3), but:
- The orchestrator writes the declaration to the **arming dir** (`<coord>/.doctrine/state/dispatch/spawn/` or similar) before spawn.
- The `create-fork` hook reads it and copies it to `jail/<name>.toml`.
- **Neither the read-from-arming-dir nor the write-to-jail/ path exists today.**

### 1.8 `WorktreeRemove` capture hook — does not exist

No `WorktreeRemove` hook handler exists anywhere in the codebase. No test for it. No `HookEvent` enum variant for it in the worktree subcommand routing. **Net-new** for F-3.

### 1.9 `SubagentStop` capture — does not exist

No `SubagentStop` handler exists. Design leads with "WorktreeRemove (and/or SubagentStop)" — the weaker, non-blocking hook. SubagentStop *does* have decision control (exit 2 prevents stopping, `hooks.md:658`), receives `agent_id` and `agent_transcript_path` (`hooks.md:1930-1957`), but has no handler, no test.

---

## 2. Ground-Truth Source Seams vs Design Claims

### 2.1 Soft Target 1 — Shared-profile model (§5.3, F-1)

**Design claim:** "The arm→spawn→create-fork-provision sequence is the named critical section; it must not interleave a second arming."

**Ground truth:**
- `dispatch-agent/SKILL.md` arm-spawn: writes ONE `base` file to `<coord>/.doctrine/state/dispatch/spawn/base`. Re-arming at B′ rewrites it. No lock, no semaphore.
- `src/worktree/create.rs:classify_create()`: reads `base` from that file; discriminates Fork v Passthrough by cwd == arming dir. No provision step for jail policy yet (net-new).
- Parallel fan-out: N `Agent` calls from same spawn dir, each triggers independent `WorktreeCreate` hooks. The hooks run concurrently. Each does `classify_create()` which reads the same `base` file.
- **No mechanism prevents interleaving** — the "critical section" claim is structural discipline (write once, read-many), not enforced by any lock or structure. The design asserts this is okay because "a single arming carries a single intent" — no differing-sibling leak. This is a *reasoning claim*, not a *structural guarantee*.

**Key insight:** The user steer accepted "parallel workers share one profile" over "baseline floor." The design correctly identifies this is a semantic constraint (all siblings get the same profile), not a race hazard (no second profile to contaminate). But the **arming-dir is stateless wrt jail policy** — the arming dir only carries `base` (a git sha). The jail policy declaration must be a **separate file** (design says it lives at the pre-spawn handshake location, provisioned by create-fork). That second file is net-new.

**Race analysis for parallel fan-out:**
- N concurrent `create-fork` hook invocations.
- Each reads cwd, reads `base`, calls `classify_create()` → all get `Fork{base, name}` with different `name`s.
- Each calls `fork_core()` → each does `git worktree add -b dispatch/<name>` → each provisions → each marks.
- Once `fork_core()` completes for one worker, its jail policy must be at `jail/<name>.toml`.
- **If two `fork_core()` calls finish before any PreToolUse fires** (which they will — the worker hasn't started), there's no race on the jail policy: each worker has its own `<name>`.
- The only thing "shared" per-arming is that all N siblings read from the **same declaration file** to produce their respective `jail/<name>.toml` files, but write to **different destinations** (different `<name>`). So no write-write race.
- **This is structurally safe**, provided create-fork reads the declaration before it provisions. The design's "single arming carries a single intent" reasoning is sound under this analysis — but it relies on `create-fork` reading the declaration from a deterministic location, which is net-new.

### 2.2 Soft Target 2 — Capture-before-remove (§5.4 / F-3)

**Design claim:** "A doctrine WorktreeRemove (and/or SubagentStop) hook captures git -C <worktree> diff (and untracked adds) into a patch at a path outside the worktree — under the coord tree's runtime state — before the harness removes the tree."

**Ground truth from `docs/claude/hooks.md`:**

| Hook | Decision control | Receives | Timing certainty |
|------|-----------------|----------|------------------|
| `WorktreeRemove` | **None** — "failures are logged in debug mode only" (`:2442, :680, :814`). Exit code 2 has no effect. | `worktree_path` (the original path from WorktreeCreate) (`:2465-2475`) | "This hook fires when a worktree is being removed" — **before or during removal**, but not documented as *before*. The auto-`git worktree remove` by claude is mentioned as cleanup. |
| `SubagentStop` | **Yes** — exit 2 prevents stopping (`:658, :782, :1930-1957`). Can `decision: "block"`. | `agent_id`, `agent_transcript_path`, `last_assistant_message` (`:1944-1957`). Also `background_tasks`, `session_crons`. Does **NOT** receive `worktree_path`. | Fires when "a subagent has finished responding" — before the harness tears down? Not explicitly documented. |

**Critical gap:** `WorktreeRemove` has no decision control — if it fires *during* or *just before* `git worktree remove`, it gets stdout+DATE but the removal proceeds regardless. The capture is RACE-DEPENDENT on the worktree still being on disk. There is ZERO documentation assurance that the worktree exists when WorktreeRemove fires.

`SubagentStop` *is* blocking-capable and CAN prevent teardown — but it receives NO `worktree_path`. The worktree context must be inferred from `agent_transcript_path` or looked up by `agent_id`. **If the design leads with WorktreeRemove but the safe path is SubagentStop, the ordering "WorktreeRemove (and/or SubagentStop)" is hedging around a real capability asymmetry.** SubagentStop can block long enough to capture the diff; WorktreeRemove cannot block at all and races.

**The abort criterion (OQ-2)** — "if the capture hook cannot observe the tree intact, escalate to Path C / IDE-024" — correctly hedges. But:
- **§5.4 has no test for this.** The test plan lists "Capture-before-remove (SECOND execute gate)" with an end-to-end e2e — a VA/VH, meaning human-verified, not automated.
- The design does not definitively commit to SubagentStop vs WorktreeRemove. It says "WorktreeRemove (and/or SubagentStop)" — the hedge is the problem for a reviewer. It should pick one, or structurally handle the race (e.g., SubagentStop to block, capture, then allow stop → WorktreeRemove fires as a no-op side effect).

### 2.3 Soft Target 3 — Fail-closed exec (§5.4 / F-2)

**Design claim:** "the materialized hooks.json carries an absolute doctrine path (or a tiny checked-in shim that exit-2s on exec/not-found), so a missing/stale binary denies rather than passes."

**Ground truth code audit:**

| Code site | What it does | Supports claim? |
|-----------|-------------|-----------------|
| `src/skills.rs:1046-1048` | `PluginAssets::get("doctrine/hooks/hooks.json")` → `write_atomic`. **Verbatim copy.** | **NO** — no resolve_exec, no path substitution. |
| `plugins/doctrine/hooks/hooks.json` | Bare `"doctrine"` on PATH. | **NO** — exactly the F-2 hole. |
| `src/boot.rs:453-460` | `resolve_exec()` exists and is used by boot/install/corpus/status. | YES — facility exists, just not wired. |
| `src/install.rs:163` | Calls `resolve_exec()` for other purposes. | YES — proves the pattern. |
| Cross-ref: `hooks.md:629-643` | "only exit code 2 blocks" + Warning. Bare PATH `command-not-found` (127) is **non-blocking** → tool PROCEEDS. | Confirms F-2 severity. |

**What the F-2 fix requires (net-new):**
1. **Option A: Modify `install_hooks_plugin_for_claude`** to template the `hooks.json` JSON, replacing the command string with `resolve_exec()?.to_string_lossy() + " worktree pretooluse"`. This prevents a stale RO binary from silently pass-through-ing.
2. **Option B: Write a shim** at `plugins/doctrine/hooks/shim.sh` (or `exit2.sh`) that `exec <resolved_doctrine> "$@"` and `exit 2` on failure. Reference that in `hooks.json` command entries. The hook process calls the shim, the shim execs the real binary (never a stale PATH version), and exits 2 if the real binary doesn't exist.
3. **Option C: Embed the absolute path at build time** into `plugins/doctrine/hooks/hooks.json` via a build.rs step or similar.

**The design says "or a tiny checked-in shim that exit-2s on exec/not-found"** — Option B. This is a plausible route (no code-gen, just a script + correct command reference). But the shim must be **materialized alongside `hooks.json`** by `install_hooks_plugin_for_claude`. Currently only `plugin.json` and `hooks.json` are written.

**Current code writes:**
- `.claude-plugin/plugin.json`
- `hooks/hooks.json`

**Does NOT write:** any shim file. This is net-new.

### 2.4 Soft Target 4 — Internal coherence after surgery (§5.1 resolve_exec vs §5.4 D-reg)

**§5.1 (lines ~84-92)** lists `resolve_exec` as a responsibility of the `pretooluse.rs` thin shell:
> "thin shell: stdin JSON in, hookSpecificOutput out, bwrap-presence probe, policy-file read, resolve_exec"

**§5.4 / D-reg** says the fail-closed exec fix is an INSTALL-time concern: the materialized `hooks.json` carries an absolute path (or a shim). The hook subcommand itself doesn't need `resolve_exec` — it IS the resolved binary.

**§7 D1** still says: "Rides the existing hook seam; reuses worktree resolution + resolve_exec."

**§7 D-reg** says: "invoking a resolved absolute doctrine (fail-closed exec, RV-200 F-2 — NOT bare PATH)."

**Verdict:** There is a vestigial tension. §5.1's `resolve_exec` could mean "the pretooluse subcommand uses resolve_exec to self-relocate" — but that's useless at hook-exec time (it's already running). Or it could mean "the materialization step uses resolve_exec" — but that's an install concern, not `pretooluse.rs`. The survivor from the rewrite is **§5.4's relocation to install-time**. The §5.1 reference to `resolve_exec` is stale but not contradictory (it's generic enough to cover "being resolved at install" as intent). The §7 D1 reference is more problematic — it says "reuses ... resolve_exec" in a §7 design-decision context that §7 D-reg then overrides. **Minor inconsistency, not a blocker, but polish targets.**

**Scope doc (slice-182.md):** Says "reuses worktree resolution + resolve_exec" under OQ-A — same mild tension. The scope doc predates the F-2 resolution; it's not been updated to reflect the install-time relocation. Minor.

### 2.5 Memory-relevant: `docs/claude/hooks.md` — PreToolUse stdin shape for Bash vs Edit|Write

The design hardcodes `tool_name` + `tool_input.command` (Bash) and `tool_input.file_path` (Edit/Write). Verified doc coverage:

- `PreToolUse` **Bash stdin** example (`hooks.md:630`): `{"tool_name": "Bash", "tool_input": {"command": "npm test"}}` and `{"cwd": ..., "agent_id": ..., "hook_event_name": "PreToolUse"}` — confirmed.
- `PreToolUse` **Edit|Write**: `tool_input.file_path` is mentioned in the docs under the generic Json schema section. The design drops `NotebookEdit`/`notebook_path` per F-6, retaining only `Edit|Write`. Docs show `Edit` as a valid tool_name. Confirmed.
- `updatedInput` format (`hooks.md:818, 1476`): replaces the **entire** `tool_input` object. For Bash: `{"command": "...", "description": "..."}`. For Write: `{"file_path": "...", "content": "..."}`. Confirmed — the design's opaque wrap (base64 the original command, emit wrapped `command` + `description` in `updatedInput`) is sound against doc.

---

## 3. Additional Relevant Materials Not in Selector List

The design's selector list (from the reviewer prompt) covers the core seam files. The following are also materially relevant:

### 3.1 `src/dispatch.rs` — orchestrator dispatch orchestration

**Why relevant:** The orchestrator declares the jail policy pre-spawn (§5.3). `dispatch-agent/SKILL.md` says "`dispatch arm-spawn --base <B>` writes the base file." The jail policy declaration must be written by the **same orchestration step** — `arm-spawn` or a companion command. Currently `dispatch arm-spawn` only writes `base`. A net-new policy-write step or expanded `arm-spawn` is needed. `src/dispatch.rs` contains the arm-spawn implementation and must be checked for how/whether it writes jail policy.

### 3.2 `src/worktree/marker.rs` — worker marker module

**Why relevant:** The design claims `create-fork` "provisions" the jail policy at `<main>/.doctrine/state/dispatch/jail/<name>.toml`, analogous to how `write_marker()` stamps the worker flag. The marker module at `src/worktree/marker.rs` implements `write_marker()`, `marker_present()`, `resolve_mode()`, etc. The jail policy provision step could follow the same pattern (a `write_jail_policy()` in a new or extended marker module).

### 3.3 `plugins/doctrine/hooks/.claude-plugin/plugin.json` (implicit)

Not listed but materialized alongside hooks. The F-2 exec-fix might need the shim embedded as an additional asset under `plugins/doctrine/hooks/` alongside `hooks.json`.

### 3.4 `scripts/pi-spawn-confined.sh` — the pi arm's bwrap core flags

**Why relevant:** D5 single-sources the bwrap core flags. This script is the authoritative reference for `--ro-bind / / --dev /dev --proc /proc --tmpfs /tmp --bind <wt> <wt> --chdir <wt> --die-with-parent`. The design promises a "parity test" against this script. Currently no such test exists. The script is at `scripts/pi-spawn-confined.sh`.

### 3.5 `.doctrine/backlog/risk/014/probe-h1/pretooluse-wrap.sh` and `pretooluse-pathcheck.sh`

**Why relevant:** The probe scripts are the proven reference implementation. The design must not diverge from their logic on settled points:
- `pretooluse-wrap.sh`: opaque base64 wrap, PASS-THROUGH when `agent_id` absent, bwrap core flags.
- `pretooluse-pathcheck.sh`: `realpath(file_path) ⊆ cwd` (ancestor rule).

Any divergence in the Rust implementation that changes the denial behaviour would be a regression from the proven probe.

### 3.6 `docs/claude/subagents-reference.md` — subagent lifecycle docs

**Why relevant:** SubagentStop timing (does it fire before or after worktree teardown?) may be documented here. The design's F-3 capture hook decision depends on this.

---

## 4. Magic Strings / STD-001 Audit

The design adds these new named paths/strings that MUST be single-sourced constants per STD-001:

| String | Where used | STD-001 status |
|--------|-----------|----------------|
| `.doctrine/state/dispatch/jail/<name>.toml` | §5.3 policy file location | **Net-new** — must be a named constant in `pretooluse.rs` or `shared.rs` |
| `.doctrine/state/dispatch/spawn` | ARMING_SUBPATH in `create.rs:202` | **ALREADY IN CODE** — `ARMING_SUBPATH` constant |
| `."worktrees"` | WORKTREES_SUBDIR in `create.rs:205` | **ALREADY IN CODE** — `WORKTREES_SUBDIR` constant |
| `jail/<name>.toml` file extension | §5.3 policy file | **Net-new** — `.toml` extension is a new constant |
| `worktree-jail: <reason>` | §5.2 deny reason prefix | **Net-new** — the deny-reason prefix |
| `bwrap-unavailable` | §5.5 edge case | **Net-new** — denial token |
| `cwd-not-a-worktree` | §5.5 deny reason | **Net-new** — already present in probe scripts |
| `PreToolUse` matcher regex `"Write\|Edit"` | §5.4 hook entry | **Net-new** — the matcher regex |

The design's adherence to STD-001 is adequate for the constants already present in code (`ARMING_SUBPATH`, `WORKTREES_SUBDIR`). New constants for jail policy paths, deny tokens, and matchers must be defined as named constants in the new modules. The design says "STD-001 follow-up if any magic strings emerge" in Follow-Ups, which is a hedge — the design should commit to STD-001 compliance from the start.

---

## 5. Key Function-Level References

### `src/skills.rs`
- `install_hooks_plugin_for_claude()` — line 1024: verbatim copy of hooks.json, NO resolve_exec wiring.
- Line 1046-1049: the `write_atomic` calls for both manifest and hooks.
- Line 1178: where `install_hooks_plugin_for_claude` is called from `run_install`.

### `src/boot.rs`
- `resolve_exec()` — line 453: the exec resolver the F-2 fix needs to wire into install.
- `pick_exec()` — called by resolve_exec; does the `(deleted)` sanitization (SL-124 D1).

### `src/worktree/create.rs`
- `classify_create()` — line 143: pure Fork-vs-Passthrough classifier (no provision).
- `sanitise_name()` — line 102: name validation.
- `run_create_fork()` — line 300: the full hook handler (stdin → classify → act).
- `act_on_create()` — line 217: where `fork_core()` is called for Fork, `run_provision()` for Passthrough.
- `ARMING_SUBPATH` — line 202: `".doctrine/state/dispatch/spawn"`.
- `WORKTREES_SUBDIR` — line 205: `".worktrees"`.

### `src/worktree/fork.rs`
- `fork_core()` — line 101: add→provision→mark. **No jail-policy-provision step.** The provision call at line ~147 is `run_provision(Some(repo.to_path_buf()), dir)` — the file-copy copier, not a policy writer.
- `rollback_fork()` — line 71: compensating rollback.
- `remove_worktree_dir()` — line 37: worktree dir cleanup (shared between fork and create).

### `src/worktree/provision.rs`
- `run_provision()` — line 81: the sole file-copy copier, reads `.worktreeinclude`.

### `plugins/doctrine/hooks/hooks.json`
- Lines 1-20: bare `doctrine boot --emit` and `doctrine worktree create-fork` commands. **No absolute paths.**

### `src/dispatch.rs`
- Not yet read in detail — `dispatch arm-spawn` implementation lives here, must be extended to write jail policy declaration.

### `docs/claude/hooks.md`
- `:629-643` — Exit code semantics: only 2 blocks, 1/127 = non-blocking.
- `:658` — SubagentStop CAN block (exit 2 "prevents the subagent from stopping").
- `:680, :814` — WorktreeRemove event: "No. Failures are logged in debug mode only."
- `:782, :1930-1957` — SubagentStop receives `agent_id`, `agent_transcript_path`, `last_assistant_message`.
- `:806` — PreToolUse decision control table: `hookSpecificOutput` decision pattern, `permissionDecision` field.
- `:818` — PreToolUse `updatedInput` field documented.
- `:1455-1515` — PreToolUse decision control details with full JSON schema.
- `:2442` — WorktreeRemove section header + "no decision control."
- `:2465-2475` — WorktreeRemove receives `worktree_path`.

### `docs/claude/plugins-reference.md`
- `:111-119` — Plugin hooks table: PreToolUse "Before a tool call executes. Can block it."
- `:388-394` — Plugin hooks NOT hot-reloaded; `/reload-plugins` or restart required.

### `.claude/skills/dispatch-agent/SKILL.md`
- Full file: arm-spawn → cd into spawn dir → Agent spawn → verify-worker → import cadence. No mention of WorktreeRemove/SubagentStop capture. No diff-capture step.

---

## 6. Summary of Net-New Work Not in Existing Code

| Need | Where | Status |
|------|-------|--------|
| `src/worktree/jail.rs` — pure Decision, JailPolicy, bwrap builder, opaque_wrap, pathcheck, validate_policy | NEW FILE | Entirely new |
| `src/worktree/pretooluse.rs` — thin shell, stdin→JSON→hookSpecificOutput | NEW FILE | Entirely new |
| `WorktreeCommand::Pretooluse` variant in `mod.rs` | `src/worktree/mod.rs` | New variant |
| PreToolUse entries in `hooks.json` | `plugins/doctrine/hooks/hooks.json` | New JSON entries |
| WorktreeRemove (or SubagentStop) capture hook entry in `hooks.json` | `plugins/doctrine/hooks/hooks.json` | New JSON entry + new handler |
| `resolve_exec()` wiring into `install_hooks_plugin_for_claude()` | `src/skills.rs:1024-1049` | New substitution logic |
| Fail-closed shim (if Option B is chosen) | `plugins/doctrine/hooks/shim.sh` (new file) + materialization | New asset + install |
| Jail policy declaration write from `dispatch arm-spawn` (or equivalent) | `src/dispatch.rs` | New write step alongside `base` file |
| Jail policy provision in `create-fork` — read declaration, write `jail/<name>.toml` | `src/worktree/create.rs` or new caller | New step alongside provision + mark |
| WorktreeRemove / SubagentStop capture handler — `git diff` snapshot to coord runtime state | NEW | Entirely new |
| D5 parity test against `scripts/pi-spawn-confined.sh` | NEW | New test |
| INV-5 shell-quoting test (space + single-quote in path) | NEW | New test |
| V-plugin gate (re-test PreToolUse-via-plugin for worktree subagent) | OPERATION | Manual e2e, not code |
| `scripts/pi-spawn-confined.sh` cross-ref comment | EXISTING FILE | Minor annotation |

---

## 7. Memory Evidence — Empirical Findings vs Documented API Claims

### 7.1 PreToolUse hooks fail OPEN (mem.fact.claude.pretooluse-hook-fail-open — trust: high, verified)

Recall: `mem_019f1a5cd5937cf3a0824f52e3ff4724`
- Only exit code 2 blocks (`docs/claude/hooks.md:629-643` + Warning). Exit 1 or 127 (command-not-found) are **non-blocking** — tool proceeds.
- **WorktreeCreate is the sole exception** where any non-zero aborts.
- Consequence: a bare-`doctrine` hook that resolves to a stale binary missing the `pretooluse` subcommand fails OPEN — the exact RSK-014 hole.
- **Verified empirically** (RSK-014 probe-h1) + documented cross-check (2026-07-01).

### 7.2 WorktreeRemove auto-destroys subagent worktree (mem.fact.claude.worktree-remove-auto-teardown — trust: high, verified)

Recall: `mem_019f1a5ce1f472219da91d0724bb766b`
- `isolation:worktree` subagent finish → WorktreeRemove fires → Claude auto-runs `git worktree remove`.
- **No decision control** (`hooks.md:2442/680/814`: "Failures are logged in debug mode only").
- Uncommitted worktree diff is DESTROYED unless captured before teardown.
- **Lifecycle asymmetry:** harness owns claude-arm worktree (auto-removed); orchestrator owns pi-arm worktree (persists until import). Not lifecycle-equivalent.
- The design's F-3 fix (capture-before-remove) is correct in direction. **The residual unknown documented in the memory (and by OQ-2) is whether WorktreeRemove observes the tree intact before `git worktree remove` completes.** This is NOT settled by docs or empirics.

### 7.3 SubagentStart is sync-blocking but NOT fail-closable (mem.pattern.dispatch.subagentstart-blocking-but-not-failclosable — trust: medium, verified)

Recall: `mem_019ec0a5bdb274b3a7cc1d5eaf4e34c5`
- SubagentStart runs synchronously and **gates the subagent** until the hook exits (proven empirically: sleep 3s → worker starts +7.0s; sleep 10s → +13.7s).
- BUT it has **no decision control** (hooks.md table: "Shows stderr to user only"). Exit 2 does NOT block the subagent.
- Contrast: **SubagentStop DOES have decision control** (exit 2 "prevents the subagent from stopping", hooks.md:658). This is the blocking-capable hook for F-3.
- The design leads with "WorktreeRemove (and/or SubagentStop)" — the empirical finding (confirmed by the memory) is that **SubagentStop is the structurally capable hook**, and WorktreeRemove is the weaker one. The "and/or" is a hedge around a real capability gap.

**Empirical implication for F-3:** If the design commits to SubagentStop (blocking-capable), it can block the stop, capture the diff, then allow the stop → WorktreeRemove fires as no-op. If it leads with WorktreeRemove (non-blocking, races), the capture timing is unresolvable by structure — only bye2e testing can confirm it works. The design SHOULD pick SubagentStop as the primary capture hook.

### 7.4 Claude Agent worktree integrates commit onto parent branch (mem.pattern.dispatch.claude-agent-worktree-integrates-commit-onto-parent — trust: medium, unverified)

Recall: `mem_019ec4a71f0f7592bc07d9f5dad8efdb` (SL-062 empirics, 2026-06-14)
- Observed: claude Agent `isolation: worktree` worker commits ended up **directly on the coordination branch (`main`)** after worktree collapse.
- Worker ran in a shared tree, not truly isolated checkout — but `.git/objects` was WRITABLE.
- **Key implication for SL-182:** With ro-`.git`, the worker CANNOT commit — no commit exists to integrate. The worktree teardown loses the uncommitted diff entirely.
- This empirically confirms the design's claim that "identical on both arms" was false — the pre-confinement funnel only worked because the worker could write to `.git`.
- **Unknown:** does the claude harness attempt to auto-commit the worktree diff if the agent tool call producing the Agent result succeeds? Or does it just run `git worktree remove` and the commit we observed was a WORKER self-commit that happened to succeed? The memory says the commit was ON main and worktree's .git/worktrees/HEAD was absent — consistent with worker self-commit + worktree removal, NOT a harness auto-integration.
- **Consequence:** with ro-`.git`, NA self-commit can happen. The funnel MUST capture the working-tree diff before teardown.

### 7.5 The confinement pattern memory (mem.pattern.dispatch.claude-worktree-subagent-bwrap-confinement — trust: high, verified)

Recall: `mem_019f18d2a9307cc38d5e4ba9749e6208`
- Full empirical record of the proven confinement apparatus. Key settled facts the Rust implementation must not regress:
  - `agent_id` present ⟺ subagent; absent ⟺ orchestrator. `isolation:none` subagent carries `agent_id` + cwd=repo-root → **denied at gate**.
  - `session_id` is SHARED — useless as discriminator.
  - Payload `cwd` == hook `pwd` == worktree — stable across concurrent subagents.
  - Fail-closed rule: pass through ONLY when `agent_id` absent. Never "jail when worktree, else pass" — that fails OPEN for `isolation:none`.
  - bwrap IS necessary for Bash (proven by necessity control). Native gives ZERO Bash containment.
  - Edit/Write needs BOTH native (repo) + our pathcheck (everywhere else). Not redundant.
  - Repo-root is DOUBLY covered (native preempts + pathwall's own ancestor rule).
  - **Hook registration loads at session start only.** But hook *script body* is re-read every call — a sentinel-gated toggle works live.
  - Honest residual: socket-reachable privileged peers (postgres, nix-daemon) are outside the jail.

### 7.6 Summary of Empiric vs Documented Claims

| Claim | Doc source | Empirical proof | Status
|-------|-----------|----------------|--------|
| PreToolUse only exit 2 blocks | hooks.md:629-643 | Probe (RSK-014) | **Verified both** |
| WorktreeRemove no decision control | hooks.md:2442/680/814 | Probe confirmed no block | **Verified both** |
| SubagentStop CAN block stop | hooks.md:658 | Not independently tested | **Doc only** — design must not assume without testing |
| SubagentStart blocks but can't fail-close | hooks.md table (no decision) | sleep experiment (SL-056) | **Verified** both |
| Agent worktree collapses commit onto parent | Not documented | Observed empirically (SL-062) | **Empiric only** — not in docs |
| Plugin PreToolUse fires for worktree subagent | plugins-reference.md:111-119 | NOT tested (probe used settings.local.json) | **Unproven** — V-plugin gate is correctly scoped |
| Plugin hooks not hot-reloaded | plugins-reference.md:394 | Confirmed empirically | **Verified both** |
