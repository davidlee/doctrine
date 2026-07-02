# Notes SL-185: Subprocess-arm Seatbelt confinement (macOS jail parity)

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Audit harvest (RV-231, 2026-07-02)

Delivery note: P01–03 were dispatch-delivered on the Linux host (sleipnir); their
runtime phase sheets + boundary rows are gitignored `.doctrine/state/` scratch that
did not travel with the branch. Durable substance reconstructed from the commit
trail, design §8 (AR/XR log), and the RISK-1 memory. PHASE-04 delivered solo on the
mac tree (its sheet was the blank template — findings captured here + in memory).

### RISK-1 / VH-3 — CLEARED against real pi (was: deferred)

The design gated all mac-branch wiring (PHASE-04) behind RISK-1 — "does a long-lived
`pi --mode rpc` fifo host + child spawn survive the SL-183 Seatbelt write-floor?"
PHASE-04 cleared it via a faithful stand-in + a live `~/.pi` observation and deferred
the full real-pi event stream (VH-3). **This audit closed the residual with real pi
0.80.3 on this macOS host (Darwin 25.4.0, arm64), driving the real shipped launcher
path** (`jail-prefix --extra-rw ~/.pi` → real worker fork → real `sandbox-exec` prefix
→ real `pi --mode rpc`):

- VH-3: pi rpc booted under `sandbox-exec`, stayed alive the whole fifo window, and
  emitted its JSON-RPC event stream (`{"id":1,"type":"response",…}` round-tripped).
  The long-lived-fifo + child lifecycle SL-183's short-lived-bash probe never covered
  holds against the real binary. (Independent of provider API key — the frame response
  proves the rpc channel round-trips under confinement.)
- VH-1: outside-`$WT` write denied (`Operation not permitted`), inside OK, spawned
  `/bin/bash` child inherited the floor; outside canary stayed pristine.
- VH-2: fail-closed proven by the converse — the same VH invoked WITHOUT the `~/.pi`
  grant crashed pi on boot (`EPERM mkdir ~/.pi/agent/trust.json.lock`).
- EX-1/EX-2 confirmed live on a mac: real `jail-prefix` resolved a real worker
  topology, materialized the real floor `jail.sb`, emitted a working 16-token
  NUL-delimited prefix. `cargo check --target aarch64-apple-darwin` clean (native).

### OQ-b — RESOLVED empirically

Real pi writes `~/.pi/agent/*.lock` (proper-lockfile `mkdirSync`) at boot **even with
`--session-dir` under `$D`** — the session redirect covers session storage, not the
`~/.pi/agent/` settings/trust locks. So the mac arm MUST grant `--extra-rw ~/.pi`
(realpath'd + validated, fail-closed if absent) — parity with the Linux arm's
`--bind ~/.pi`. Grant is provably load-bearing (no grant ⇒ boot crash) and provably
sufficient (with grant ⇒ clean boot, zero permission errors). Shipped
`pi-spawn-confined.sh:80-81` passes it correctly. → see
[[mem.pattern.dispatch.seatbelt-confines-longlived-fifo-worker]] (residual "VH-3
deferred / confirm against real pi" is now DISCHARGED — update at close).

### Two design-truth deviations (→ /reconcile per-slice direct edit)

- **F-1:** design.md §3 says `run_jail_prefix` lives in `mod.rs`; it was factored into
  a new `src/worktree/jail_prefix.rs` (clean improvement). Update §3 prose + selector
  list.
- **F-5 (mapfile, user-approved):** design §1/§3 prescribe `mapfile -d ''` (bash-4);
  macOS ships bash 3.2.57 (no mapfile). Shipped code uses the portable
  `while IFS= read -r -d '' tok || [ -n "$tok" ]` loop — strictly *more* correct
  (mapfile would drop the final non-NUL `env TMPDIR=` token; verified 16 vs 15). A
  "restore mapfile" edit would break macOS confinement — the design must record why.

### Follow-ups (non-blocking)

- **IMP-230** — de-dup the `pi-spawn-confined.sh` NUL-array reader from its e2e-test
  copy. Open, correctly deferred.
- Design (B) follow-up (unify Linux subprocess arm onto `jail-prefix`) + codex arm +
  IDE-025 remain out of scope per design §6.
