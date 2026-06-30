# Brief IMP-045 — Seatbelt (`sandbox-exec`) adaptation of the SL-182 jail on macOS

**Discharges:** the SL-182 macOS arm, today a fail-closed stub (`deny "bwrap-unavailable"`, §5.5 Edge / POL-002). Goal: a *real* write-containment arm on macOS with the **same** `Decision`/`Target`/policy/funnel, so only the argv/profile builder forks.

## 1. The one-line model difference

bwrap is a **mount namespace** (whitelists what is *visible*; `--ro-bind / /` makes everything read-only, the worktree rw). Seatbelt is a **profile sandbox over an unchanged filesystem** (whitelists *operations*). So the parity construction is the **inverse**: don't hide anything, **allow-default then deny the write operation class, then re-allow writes under the worktree**. This is *allow-default-deny-write-except* — not default-deny, which is the SBPL footgun that earns Seatbelt its reputation and which you sidestep entirely for a write floor.

```scheme
(version 1)
(allow default)                                  ; nothing hidden — reads open (parity: reads are a non-goal, §4)
(deny file-write*)                               ; the floor
(allow file-write* (subpath (param "WT")))       ; the worktree, rw
; one (allow file-write* (subpath (param "RW_n"))) per validated extra_rw entry
; (deny network*)  iff policy.network == false
```

Invoke opaquely, params not string-interpolation (the clean INV-5 analog):
```
sandbox-exec -D WT=<realpath wt> -D RW0=<realpath e0> … -p "$PROFILE" -- \
  bash -c "$(printf %s "$B64" | base64 -d)"
```
Original command rides as base64, never re-parsed; **children inherit the sandbox** (parity with bwrap's process-tree containment — confirm in the probe).

## 2. Element-by-element mapping (mirror `bwrap_core_argv` / `bwrap_argv`)

| bwrap (SL-182 proven) | Seatbelt analog | Note |
|---|---|---|
| `--ro-bind / /` | `(allow default)(deny file-write*)` | inverse logic; not a namespace |
| rw-bind worktree | `(allow file-write* (subpath (param "WT")))` | `file-write*` = the full write family (create/unlink/mode/…) |
| `.git` ro, **not tunable** (INV-3/D3) | nothing extra needed | worktree's real gitdir is `<main>/.git/worktrees/<name>` — *outside* wt → write-denied by the floor. No-self-commit consequence identical → same funnel |
| `extra_rw[]` (validated) | one `(allow file-write* (subpath (param "RW_n")))` each | `validate_policy` is **platform-agnostic, shared unchanged** (INV-3/INV-4: reject `/`, root-ancestors, `.git`) |
| `--tmpfs /tmp` (private, ephemeral) | **no analog** — see §3b | the real gap |
| `network=false ⇒ --unshare-net` | `(deny network*)` | coarser (syscall-deny, not iface removal); egress already a non-goal (§4/OQ-6) |
| `bwrap_present` probe | `sandbox-exec` presence + **nesting** probe | ships with macOS; presence ~given, *behaviour* is not (§3a) |
| absent tool ⇒ `deny` | unchanged | fail-closed; POL-002 — but macOS is now a real arm, not a deny |

`resolve_target`, `decide_bash`, `decide_write`, `pathcheck`, `opaque_wrap` (sans the argv), `validate_policy` are **all reused as-is**. Only `bwrap_argv` forks → add `seatbelt_profile(policy) -> String` + `sandbox_exec_argv(wt, policy) -> Vec<OsString>` behind a single `Jailer` seam (trait or runtime-os branch in `jail.rs`). This is the D5-analog single source; no new pipeline, ADR-001 layering intact.

## 3. The three honest fidelity gaps (state, don't hide — fail-closed ethos)

**(a) Nesting vs Claude Code's own Seatbelt — the macOS twin of the nested-`/proc` caveat, and the top probe risk.** Claude Code's native sandbox on macOS *is* Seatbelt. A `sandbox-exec` invoked from inside an already-Seatbelt'd harness process may be refused or silently weakened (nested profile composition is undocumented). **Must probe before anything else** — this is the macOS analog of "does nested bwrap get a `/proc`." If nesting is refused, the arm's contract degrades to "deny worktree-subagent Bash on macOS," never silent pass-through.

**(b) No true `tmpfs` → per-worker dir + explicit GC.** Seatbelt can't mount an ephemeral private `/tmp`. Parity options, in preference order:
1. **Point `TMPDIR` inside the worktree** (`<wt>/.tmp`): already rw, already GC'd with the tree, and **deny** global `/private/tmp`. Cleanest — collapses scratch into the existing rw scope, matches the SL-182 edge ("/tmp denied for Edit/Write; loosen via `extra_rw`"). Downside: tools that hard-code `/tmp`.
2. Per-invocation `mktemp -d`, add its realpath to the write-allow set, `rm -rf` on `SubagentStop`/teardown. Disjoint per instance (the RFC's scratch-unifier), but **persistent until GC'd**, not auto-discarded — teardown must clean it. Note the difference from Linux explicitly: on the bwrap arm `/tmp` writes vanish with the namespace; here they don't.

**(c) Reads stay open / paths not hidden.** Already the declared scope (reads are out-of-scope on both arms, §4). No action; just don't claim confidentiality the profile doesn't provide.

## 4. The footgun to budget for (INV-5 twin)

`subpath` matches the **resolved** path, and macOS aliases `/tmp→/private/tmp`, `/var→/private/var`, `/etc→/private/etc`. **Feed realpaths into every `-D` param** (`realpath wt`, `realpath` each `extra_rw`), or writes get mysteriously allowed/denied. Using `-D` params (not profile string-splicing) removes the quoting surface; still single-quote the param values and `-p` body. The symlink/hardlink escape vectors should be caught because Seatbelt evaluates the resolved target — but that is exactly canonicalization, so **prove it in the battery**, don't assume.

## 5. Residual: the always-on reachable peer (macOS-specific)

On the NixOS/bwrap arm the delegation-to-out-of-namespace-executor vector is dead (no cron/at/systemd in the closure). **macOS always ships `launchd`.** Seatbelt is not a namespace, so it does not remove launchd. The *file-based* delegation vectors (writing a LaunchAgent plist to `~/Library/LaunchAgents`, a crontab file) are contained by the write-floor (those paths are outside wt → denied). The residual is a **pure-IPC `launchctl submit` / mach-service** path, which write-denial doesn't cover and which would only close via mach-lookup denial — i.e. the default-deny rabbit hole this floor avoids. Record it as the macOS sibling of the postgres/nix-daemon reachable-peer residual (RSK-014): owned by the IPC/egress wall (non-goal here), not by the write floor. Probe `launchctl submit` and `at` so the residual is *measured*, not assumed.

## 6. Probe-first phase (D7 analog — Seatbelt is undocumented + deprecated, so this matters more, not less)

Disposable `sandbox-exec` shell probe **before any Rust**, pinning every doc-unconfirmed behaviour:
1. **Nesting (§3a)** — run the profile from inside a Claude Code macOS session; does it apply or get refused? Gate everything on this.
2. **Floor + canonicalization** — `(allow default)(deny file-write*)(allow … (param "WT"))`; write to wt (allowed), to `/private/tmp`, `$HOME`, repo-root, `/etc` (denied). Confirm `/tmp` alias resolves.
3. **Child inheritance** — `python -c`/heredoc + detached `&` child writes outside wt → denied.
4. **Escape battery (same 11 vectors as RSK-014 Exp 2)** — absolute, `../` traversal, symlink-deref, hardlink, shared-`.git`, child-proc, detached job, `$HOME`; **plus** macOS-only `launchctl submit` / `at` (§5).
5. **`-D` round-trip** — a wt path / `extra_rw` with a space and a quote.
6. **`updatedInput` honoured** for the `sandbox-exec`-wrapped command on macOS (the Bash-wall in-situ proof, mirroring H1b).

Pass criterion identical to H1: every external vector `denied` (here: `Operation not permitted` / sandbox violation), wt writable, wrapper confirmed applied. **Abort/degrade contract:** nesting-refused or canonicalization-leaky ⇒ macOS arm = `deny worktree-subagent Bash`, never unwrapped.

## 7. Decisions to record (idiom)

- **D-mac1** — Seatbelt = *allow-default-deny-write-except*, not default-deny. Rationale: write floor only; sidesteps the SBPL allowlist swamp.
- **D-mac2** — single `Jailer` seam; reuse all of `jail.rs` except the argv/profile builder (D5-analog).
- **D-mac3** — `TMPDIR=<wt>/.tmp` + deny `/private/tmp` (no-tmpfs resolution 3b.1); `mktemp` variant is the fallback if hard-coded-`/tmp` tooling forces it.
- **D-mac4** — `network` knob maps to `(deny network*)` with a stated coarseness caveat; egress remains the non-goal (parity with bwrap).
- **OQ-mac1** — nesting vs harness Seatbelt (probe gate, §3a/§6.1).
- **OQ-mac2** — launchd IPC residual: measure, then assign to the IPC/egress wall (§5).
- **Vanish risk** — deprecated since ~10.10, SBPL undocumented; mitigated by Anthropic's own sandbox-runtime + system `.sb` profiles depending on it. Low, not zero; note it.
