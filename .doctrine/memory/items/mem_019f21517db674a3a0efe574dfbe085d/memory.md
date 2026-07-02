# RISK-1 clears: long-lived fifo-reader + child spawn survive SL-183 Seatbelt write-floor (SL-185 PHASE-04 go)

SL-185 gated its mac-branch wiring (PHASE-04) behind **RISK-1**: SL-183's Seatbelt
probe wrapped only short-lived *bash*; it never covered a **long-lived `pi --mode
rpc` process reading a fifo and spawning children**. If `sandbox-exec` broke that
lifecycle, the whole launcher strategy (and the Rust `jail_prefix` mac arm) was
moot.

## Verdict: PASS (go for PHASE-04)

Probed cheap/early on macOS (Darwin 25.4.0, arm64) with a **faithful stand-in** —
real pi is Linux-only in the dev shell (flake.nix line ~275 gates
`pi`/`dirge`/`claude` behind `isLinux`). The stand-in emitted the byte-faithful
SL-183 floor (`seatbelt_profile`: `(allow default)`/`(deny file-write*)`,
deny-coarse-first PTMP+DUTMP, device sinks, allow-specific-last WT+TMP, xcrun_db
require-all) and ran under `sandbox-exec -D WT/TMP/PTMP/DUTMP -f floor.sb`:

- **Long-lived fifo read survived** — the worker read a fifo (delayed-close writer,
  the [[mem.pattern.pi.rpc-stdin-lifecycle]] workaround) and kept running; no early
  exit. This is the load-bearing RISK-1 claim.
- **Child inherits the floor** — a spawned `bash -c` child: WT write OK, outside
  write `Operation not permitted`.
- WT + TMP writes OK; `/dev/null` (device sink) OK; **outside-worktree writes
  denied** for both parent and child.
- **Independent canary verifier** (never trust the vector self-report — RSK-014
  idiom): the outside canary stayed `PRISTINE-OUT`, untouched.

## Residual — DISCHARGED at close (real pi, SL-185 audit RV-231, 2026-07-02)

The stand-in proved the sandbox *mechanics*; the SL-185 close-out audit (RV-231)
then confirmed them against **real pi 0.80.3 natively on this macOS host** (Darwin
25.4.0, arm64) — no flake escalation needed (pi installed at
`~/.npm-global/bin/pi`), driving the real shipped launcher path end-to-end
(`jail-prefix --extra-rw ~/.pi` → real worker fork → real `sandbox-exec` prefix →
real `pi --mode rpc`):

- **VH-3 (rpc event stream under confinement) — CONFIRMED.** Real `pi --mode rpc`
  booted under `sandbox-exec`, **stayed alive the whole fifo window** (rc=124
  timeout-kill, not early exit), and **round-tripped a JSON-RPC frame**
  (`{"id":1,"type":"response",…}`). The long-lived fifo-reader + child lifecycle
  holds against the real binary, not just the stand-in. (Independent of a working
  provider API key — the frame response itself proves the channel round-trips
  under confinement.)
- **OQ-b — settled empirically (was: user-asserted).** Real pi writes
  `~/.pi/agent/*.lock` (`proper-lockfile` mkdir) at boot **even with `--session-dir`
  under `$D`** — the session redirect covers session storage, not the
  `~/.pi/agent/` settings/trust locks. The `~/.pi` extra_rw grant is **provably
  load-bearing** (no grant ⇒ boot crash `EPERM mkdir …/trust.json.lock`) and
  **provably sufficient** (with grant ⇒ clean boot, zero permission errors).
  Shipped `pi-spawn-confined.sh:80-81` passes it correctly.
- **Write-floor + child inheritance re-confirmed on real prefix:** inside-`$WT`
  write OK, outside DENIED, spawned `/bin/bash` child inherited the floor, outside
  canary pristine.

No residual remains. RISK-1 fully cleared against real pi; SL-185 closed.

Floor builder: [[mem.pattern.dispatch.seatbelt-write-floor-rule-ordering]].
In-situ subagent nesting analog: [[mem.pattern.dispatch.seatbelt-insitu-subagent-nesting]].
