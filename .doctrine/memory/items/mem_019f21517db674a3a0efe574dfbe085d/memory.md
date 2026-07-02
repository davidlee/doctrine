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

## Residual (NOT covered by the stand-in)

The stand-in proves the **sandbox mechanics**, not pi-specific runtime behaviour.

- **OQ-b — RESOLVED (user, 2026-07-02): real `pi` DOES need `~/.pi` write access
  at runtime**, even with the session redirected under `$D`. So the mac arm MUST
  grant an `extra_rw` for `~/.pi` (+existence/realpath validation, fail-closed if
  absent) — matching the bwrap arm's `--bind ~/.pi`. The earlier "bwrap bind ≠
  needed write" caution does NOT apply here; the write is real.
- Still deferred to the **PHASE-04 VH** step: the actual pi rpc **event stream**
  under confinement (VH-4), confirmed against real pi (escalate via flake then).

Floor builder: [[mem.pattern.dispatch.seatbelt-write-floor-rule-ordering]].
In-situ subagent nesting analog: [[mem.pattern.dispatch.seatbelt-insitu-subagent-nesting]].
