# Probe results — H2 (Seatbelt `sandbox-exec` macOS write-containment)

Host: macOS 26.4.1 (build 25E253), `sandbox-exec` `/usr/bin/sandbox-exec`,
`AI_AGENT=claude-code_2-1-197_agent`. Permission model: normal/ask (see
*Permission-mode confound* below). Date 2026-07-01.

**Pass 1 = orchestrator/interactive context, shell-only** (per the depth
decision). The in-situ subagent leg (M1-sub) + the auto-vs-yolo subagent
comparison are **pass 2**, explicitly deferred and flagged, not claimed.

Apparatus: `probe-brief-h2-seatbelt.md` (hypothesis), `seatbelt-jail.sh` (profile
+ argv builder — the shell analog of the Rust `seatbelt_profile`/`sandbox_exec_argv`
seam), `battery.sh` (13-vector escape battery), `canaries/{setup,verify}.sh`
(checksummed canaries + independent verifier).

---

## VERDICT — H2 SUPPORTED in the orchestrator context (M1-orch, M2, M4 ✓; M3 measured; M5 ✓ with two budgeted gaps)

A write-floor Seatbelt profile (`allow default` / `deny file-write*` / re-allow
under realpath'd WT+TMP) **contains every external write vector** in the battery
when invoked from the orchestrator Seatbelt context. All four canaries (parent
tree, shared-`.git` analog, `/private/tmp` alias, `$HOME`) **intact** across every
run. The wt stays writable; `/tmp`→`/private/tmp` alias resolves correctly. The
two gating unknowns (M1-orch nesting, M2 canonicalization) **hold**; the
`launchctl submit` IPC residual the brief feared is **empirically denied by
default**.

**Not yet proven (pass 2):** M1-sub — composition from inside a *real*
`isolation:worktree` subagent, where Claude Code's own native Seatbelt is active.
Orchestrator-context composition is the weaker claim; the subagent context is the
design's actual runtime and remains open.

---

## Sub-claim ledger

| Claim | Verdict | Evidence |
|---|---|---|
| **M1-orch** — floor composes from orchestrator Seatbelt | **SUPPORTED** | inside-wt write ALLOWED, every outside write `Operation not permitted`, canaries intact. The harness's own Seatbelt does not refuse a nested `sandbox-exec`. |
| **M1-sub** — composes inside a real worktree subagent | **DEFERRED (pass 2)** | needs PreToolUse+subagent in-situ rig; the true nesting test. |
| **M2** — canonicalization containment (resolved-path `subpath`) | **SUPPORTED** | V1 absolute, V2 `../` traversal, V3 symlink-deref, **V4 hardlink** (`ln` to outside target denied — the sharpest case), V5 shared-`.git`, V6 `/tmp` alias — **all BLOCKED**. Realpath'd `-D` params sufficient; no extra canonicalization needed. |
| **M3** — child / detached / IPC inheritance | **MEASURED — contained, residual bounded** | V8 python child: escape write denied (`PermissionError … parent_canary`), python still runs in-wt. V9 `nohup &` / V10 `setsid`: detached writes never land (canary intact +3s). **V11 `launchctl submit`: rc=1, no launchd job, canary pristine — IPC to launchd DENIED by default** (control proves submit works rc=0 unsandboxed). V12 `at`: `/usr/bin/at` write denied. |
| **M4** — `-D` round-trip + opaque base64 wrap | **SUPPORTED** | opaque base64 body decodes+execs inside the sandbox; original never runs unwrapped; realpath'd params land allow/deny on the right resolved path. (Quote/space round-trip exercised via the builder; the live-`updatedInput` leg is M1-sub/pass-2.) |
| **M5** — no-tmpfs resolution (`TMPDIR=<wt>/.tmp` + deny `/private/tmp`) | **SUPPORTED, two budgeted gaps** | V13: `TMPDIR=<wt>/.tmp`, `TMP_WROTE` in-wt; V6 `/private/tmp` write denied. Gaps F-A (deny-ordering) and F-E (`/var/folders` xcrun cache) below. |

---

## Findings (foundation-shaping — the reason the probe ran before any Rust)

### F-A — SBPL is **last-match-wins**; `deny /private/tmp` shadows `allow WT` (GATE)
macOS temp worktrees land **under `/private/tmp`**. Naively emitting `(deny
file-write* (subpath PTMP))` *after* the WT allow makes the deny the last match
for any path under `/private/tmp` — **including the worktree itself** → the floor
denies in-wt writes (`BLOCKED_INSIDE`, reproduced deterministically). **Fix
(applied):** emit the coarse `deny PTMP` **before** the specific WT/TMP/extra_rw
re-allows, so the narrower allow wins. The Rust `seatbelt_profile` MUST order
rules deny-coarse-first, allow-specific-last. **Design-load-bearing.**

### F-B — `(deny file-write*)` denies the device write surface → breaks tooling
The bare floor denies `/dev/null`, `/dev/stdout`, `/dev/stderr`, `/dev/tty*`,
`/dev/fd`, `/dev/dtracehelper`. Broke `python3` (and any tool redirecting to
`/dev/null`). **Fix (applied):** re-allow the standard device sinks (see emitted
profile). The Rust builder must include this device-sink allow-set as a constant.

### F-E — `/var/folders/$USER/T` (DARWIN_USER_TEMP_DIR) is a *second* tmp surface
`/usr/bin/python3` (the Xcode shim) writes an `xcrun_db` cache to
`/var/folders/.../T/` — the macOS per-user temp dir, **distinct from `/tmp`** and
**not** covered by the `TMPDIR` redirect (xcrun hardcodes it). Under the floor
that write denies, emitting noise. **Cosmetic here** (python still executed), but
tools that *depend* on that cache would break. D-mac3 must decide: also redirect
`/var/folders/$USER/T` (via `getconf DARWIN_USER_TEMP_DIR` / `$TMPDIR` default) or
allow it read-mostly. The macOS sibling of F-B. **Budget in design.**

### F-D — battery self-report lied; the independent canary verifier caught it
First V4 run reported `WROTE` — but the `ln` had been denied and the subsequent
`echo > wt/hl` wrote a *new in-wt file*, not the canary. The canary verifier
showed `parent_canary: OK` → no escape. **Fixed the battery** to branch on `ln`
success (`BLOCKED-LN-DENIED`). Lesson (RSK-014 discipline confirmed): **never
trust the vector's self-report; the independent checksum verifier is truth.**

### F-C — permission-mode is NOT a write confound (orchestrator context)
**Control A** (bare write, no sandbox, under the gate): **WROTE** → the
permission gate does not block the write *syscall*; it gates tool *invocation*,
not filesystem ops once a command runs. **Control B** (same write inside floor):
BLOCKED. So every battery denial is attributable to Seatbelt, mode-independent —
in the orchestrator context. (Confirmed under normal/ask; auto earlier gave
identical write behaviour, consistent with the gate being transparent to writes.)
**Subagent context is untested** — pass 2 runs under both auto and yolo with a
bare-write control baked in, so a subagent-context gate denial cannot masquerade
as Seatbelt.

---

## Residual & forward (not blockers on pass 1)

1. **M1-sub (pass 2, the real gate).** Wire the PreToolUse(Bash) `sandbox-exec`
   wrapper into settings; spawn a real `isolation:worktree` red-team subagent;
   independent canary verifier; confirm `updatedInput` honoured on macOS. Run
   under **both** permission modes. Abort/degrade contract: nesting-refused ⇒
   macOS arm = `deny worktree-subagent Bash`, never unwrapped.
2. **launchd IPC residual (brief §5) — measured LOWER than feared.** `launchctl
   submit` is denied by Seatbelt default on this version (no mach-lookup deny
   needed). Record as *measured-low*, not *closed*: `at` is denied too; OS-version
   variance and other launchd-adjacent vectors unmeasured. Still owned by the
   IPC/egress wall (non-goal), but the design can state it's empirically contained
   here rather than open.
3. **F-E / F-B device+temp surface → named constants (STD-001).** The device-sink
   allow-set and the `/var/folders` decision become single-sourced constants in
   the Rust profile builder.

## How to re-run
```
export PROBE_BASE=/path/to/gitignored/scratch
bash canaries/setup.sh
bash battery.sh        # drives 13 vectors inside the floor
bash canaries/verify.sh  # independent truth: any canary mutated?
```
Scripts are the committed authored evidence; run artifacts stay in gitignored
scratch (`PROBE_BASE`).
